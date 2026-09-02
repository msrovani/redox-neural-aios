//! Montador op-IR → WASM (port userspace ADR-010 / neural-os ADR-0059 F4).
//! Gramática mínima: expressões i32 sem memória/imports — determinístico e seguro.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValType {
    I32,
}

pub type BlockResult = Option<ValType>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    LocalGet(u32),
    I32Const(i32),
    I32Add,
    I32Sub,
    I32Mul,
    Drop,
    I32LtS,
    I32GtS,
    I32Eq,
    I32Eqz,
    Block(BlockResult),
    Loop(BlockResult),
    If(BlockResult),
    Else,
    Br(u32),
    BrIf(u32),
    End,
    I32Select,
}

fn block_result_byte(r: BlockResult) -> u8 {
    match r {
        None => 0x40,
        Some(ValType::I32) => 0x7f,
    }
}

fn uleb(mut n: u64, out: &mut Vec<u8>) {
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        out.push(b);
        if n == 0 {
            break;
        }
    }
}

fn sleb(mut v: i64, out: &mut Vec<u8>) {
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        let sign = b & 0x40;
        let more = !((v == 0 && sign == 0) || (v == -1 && sign != 0));
        if more {
            b |= 0x80;
        }
        out.push(b);
        if !more {
            break;
        }
    }
}

fn section(id: u8, content: &[u8], out: &mut Vec<u8>) {
    out.push(id);
    uleb(content.len() as u64, out);
    out.extend_from_slice(content);
}

pub fn validate(n_params: u32, ops: &[Op]) -> Result<(), &'static str> {
    let mut depth: i32 = 0;
    let mut block_stack: Vec<(i32, i32, bool)> = Vec::new();
    for op in ops {
        match op {
            Op::LocalGet(i) => {
                if *i >= n_params {
                    return Err("op-IR: local fora de faixa");
                }
                depth += 1;
            }
            Op::I32Const(_) => depth += 1,
            Op::I32Add | Op::I32Sub | Op::I32Mul => {
                if depth < 2 {
                    return Err("op-IR: stack underflow em binop");
                }
                depth -= 1;
            }
            Op::Drop => {
                if depth < 1 {
                    return Err("op-IR: stack underflow em Drop");
                }
                depth -= 1;
            }
            Op::I32LtS | Op::I32GtS | Op::I32Eq => {
                if depth < 2 {
                    return Err("op-IR: stack underflow em cmp");
                }
                depth -= 1;
            }
            Op::I32Eqz => {
                if depth < 1 {
                    return Err("op-IR: stack underflow em I32Eqz");
                }
            }
            Op::I32Select => {
                if depth < 3 {
                    return Err("op-IR: stack underflow em I32Select");
                }
                depth -= 2;
            }
            Op::Block(result) | Op::If(result) => {
                if matches!(op, Op::If(_)) {
                    if depth < 1 {
                        return Err("op-IR: stack underflow em If");
                    }
                    depth -= 1;
                }
                let arity = if *result == Some(ValType::I32) { 1 } else { 0 };
                block_stack.push((depth, arity, false));
                depth += arity;
            }
            Op::Loop(result) => {
                let arity = if *result == Some(ValType::I32) { 1 } else { 0 };
                block_stack.push((depth, arity, true));
                depth += arity;
            }
            Op::Else => {
                if block_stack.is_empty() {
                    return Err("op-IR: Else sem If");
                }
                let (entry_depth, arity, _) = *block_stack.last().unwrap();
                depth = entry_depth + arity;
            }
            Op::Br(target) | Op::BrIf(target) => {
                if matches!(op, Op::BrIf(_)) && depth < 1 {
                    return Err("op-IR: stack underflow em BrIf");
                }
                if block_stack.is_empty() {
                    return Err("op-IR: br sem bloco");
                }
                if (*target as usize) >= block_stack.len() {
                    return Err("op-IR: br target fora de faixa");
                }
                if matches!(op, Op::BrIf(_)) {
                    depth -= 1;
                }
            }
            Op::End => {
                if block_stack.is_empty() {
                    return Err("op-IR: End sem bloco");
                }
                let (entry_depth, arity, _) = block_stack.pop().unwrap();
                depth = entry_depth + arity;
            }
        }
    }
    if !block_stack.is_empty() {
        return Err("op-IR: blocos não fechados");
    }
    if depth != 1 {
        return Err("op-IR: deve sobrar exatamente 1 valor i32");
    }
    Ok(())
}

fn ensure_ends(ops: &[Op]) -> Vec<Op> {
    let mut out = Vec::new();
    let mut open = 0i32;
    for op in ops {
        match op {
            Op::Block(_) | Op::Loop(_) | Op::If(_) => {
                out.push(*op);
                open += 1;
            }
            Op::End => {
                out.push(*op);
                open -= 1;
            }
            _ => out.push(*op),
        }
    }
    for _ in 0..open {
        out.push(Op::End);
    }
    out
}

pub fn build_run_module(n_params: u32, ops: &[Op]) -> Result<Vec<u8>, &'static str> {
    let ops_owned = ensure_ends(ops);
    validate(n_params, &ops_owned)?;

    let mut out = Vec::new();
    out.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

    let mut ty = Vec::new();
    uleb(1, &mut ty);
    ty.push(0x60);
    uleb(n_params as u64, &mut ty);
    for _ in 0..n_params {
        ty.push(0x7f);
    }
    uleb(1, &mut ty);
    ty.push(0x7f);
    section(0x01, &ty, &mut out);

    let mut fun = Vec::new();
    uleb(1, &mut fun);
    uleb(0, &mut fun);
    section(0x03, &fun, &mut out);

    let mut exp = Vec::new();
    uleb(1, &mut exp);
    uleb(3, &mut exp);
    exp.extend_from_slice(b"run");
    exp.push(0x00);
    uleb(0, &mut exp);
    section(0x07, &exp, &mut out);

    let mut body = Vec::new();
    uleb(0, &mut body);
    for op in &ops_owned {
        match op {
            Op::LocalGet(i) => {
                body.push(0x20);
                uleb(*i as u64, &mut body);
            }
            Op::I32Const(v) => {
                body.push(0x41);
                sleb(*v as i64, &mut body);
            }
            Op::I32Add => body.push(0x6a),
            Op::I32Sub => body.push(0x6b),
            Op::I32Mul => body.push(0x6c),
            Op::Drop => body.push(0x1a),
            Op::I32LtS => body.push(0x48),
            Op::I32GtS => body.push(0x4a),
            Op::I32Eq => body.push(0x46),
            Op::I32Eqz => body.push(0x45),
            Op::I32Select => body.push(0x1b),
            Op::Block(r) => {
                body.push(0x02);
                body.push(block_result_byte(*r));
            }
            Op::Loop(r) => {
                body.push(0x03);
                body.push(block_result_byte(*r));
            }
            Op::If(r) => {
                body.push(0x04);
                body.push(block_result_byte(*r));
            }
            Op::Else => body.push(0x05),
            Op::Br(t) => {
                body.push(0x0c);
                uleb(*t as u64, &mut body);
            }
            Op::BrIf(t) => {
                body.push(0x0d);
                uleb(*t as u64, &mut body);
            }
            Op::End => body.push(0x0b),
        }
    }
    body.push(0x0b);

    let mut code = Vec::new();
    uleb(1, &mut code);
    uleb(body.len() as u64, &mut code);
    code.extend_from_slice(&body);
    section(0x0a, &code, &mut out);
    Ok(out)
}

struct ExprParser<'a> {
    src: &'a [u8],
    pos: usize,
    params: &'a mut Vec<Vec<u8>>,
}

impl<'a> ExprParser<'a> {
    fn new(src: &'a str, params: &'a mut Vec<Vec<u8>>) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            params,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> u8 {
        self.src.get(self.pos).copied().unwrap_or(0)
    }

    fn skip_ws(&mut self) {
        while !self.at_end() && self.peek().is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn ident_start(c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_'
    }

    fn ident_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_'
    }

    fn param_index(&mut self, name: &[u8]) -> Result<u32, &'static str> {
        if name.len() >= 2 && name[0] == b'p' && name[1..].iter().all(|b| b.is_ascii_digit()) {
            let text = std::str::from_utf8(&name[1..]).map_err(|_| "op-IR: pN inválido")?;
            let idx = text.parse::<u32>().map_err(|_| "op-IR: pN inválido")?;
            while (self.params.len() as u32) <= idx {
                self.params.push(Vec::new());
            }
            return Ok(idx);
        }
        if let Some(i) = self.params.iter().position(|n| n.as_slice() == name) {
            return Ok(i as u32);
        }
        let i = self.params.len() as u32;
        self.params.push(name.to_vec());
        Ok(i)
    }

    fn parse_expr(&mut self, ops: &mut Vec<Op>) -> Result<(), &'static str> {
        self.parse_additive(ops)
    }

    fn parse_additive(&mut self, ops: &mut Vec<Op>) -> Result<(), &'static str> {
        self.parse_term(ops)?;
        loop {
            self.skip_ws();
            match self.peek() {
                b'+' => {
                    self.pos += 1;
                    self.parse_term(ops)?;
                    ops.push(Op::I32Add);
                }
                b'-' => {
                    self.pos += 1;
                    self.parse_term(ops)?;
                    ops.push(Op::I32Sub);
                }
                _ => return Ok(()),
            }
        }
    }

    fn parse_term(&mut self, ops: &mut Vec<Op>) -> Result<(), &'static str> {
        self.parse_factor(ops)?;
        loop {
            self.skip_ws();
            if self.peek() == b'*' {
                self.pos += 1;
                self.parse_factor(ops)?;
                ops.push(Op::I32Mul);
            } else {
                return Ok(());
            }
        }
    }

    fn parse_factor(&mut self, ops: &mut Vec<Op>) -> Result<(), &'static str> {
        self.skip_ws();
        match self.peek() {
            b'(' => {
                self.pos += 1;
                self.parse_expr(ops)?;
                self.skip_ws();
                if self.peek() != b')' {
                    return Err("op-IR: falta ')'");
                }
                self.pos += 1;
                Ok(())
            }
            b'0'..=b'9' | b'-' => {
                let start = self.pos;
                if self.peek() == b'-' {
                    self.pos += 1;
                }
                let d0 = self.pos;
                while !self.at_end() && self.peek().is_ascii_digit() {
                    self.pos += 1;
                }
                if self.pos == d0 {
                    return Err("op-IR: número malformado");
                }
                let text = std::str::from_utf8(&self.src[start..self.pos])
                    .map_err(|_| "op-IR: não-utf8")?;
                let v: i32 = text.parse().map_err(|_| "op-IR: i32 fora de faixa")?;
                ops.push(Op::I32Const(v));
                Ok(())
            }
            c if Self::ident_start(c) => {
                let start = self.pos;
                while !self.at_end() && Self::ident_char(self.peek()) {
                    self.pos += 1;
                }
                let name = &self.src[start..self.pos];
                let idx = self.param_index(name)?;
                ops.push(Op::LocalGet(idx));
                Ok(())
            }
            _ => Err("op-IR: fator inválido"),
        }
    }
}

/// Compila expressão aritmética para op-IR (ex: `a*b+7`, `(a+b)*2`).
pub fn compile_expression(source: &str) -> Result<(u32, Vec<Op>), &'static str> {
    let mut params = Vec::new();
    let mut parser = ExprParser::new(source.trim(), &mut params);
    let mut ops = Vec::new();
    parser.skip_ws();
    if parser.at_end() {
        return Err("op-IR: expressão vazia");
    }
    parser.parse_expr(&mut ops)?;
    parser.skip_ws();
    if !parser.at_end() {
        return Err("op-IR: trailing input");
    }
    let n = params.len() as u32;
    validate(n, &ops)?;
    Ok((n, ops))
}

/// Atalho: `return 42` ou literal numérico.
pub fn compile_return_literal(source: &str) -> Result<(u32, Vec<Op>), &'static str> {
    let trimmed = source.trim();
    let expr = trimmed
        .strip_prefix("return")
        .map(str::trim)
        .unwrap_or(trimmed);
    compile_expression(expr)
}

pub fn schema_hint() -> &'static str {
    "Emita UMA expressão op-IR i32: identificadores a,b,p0.. ou literais. Ex: a+b, a*b+7, 42"
}

pub fn build_and_run_2(ops: &[Op], a: i32, b: i32) -> Result<i32, crate::WasmError> {
    let wasm = build_run_module(2, ops)?;
    crate::run_i32_2(&wasm, "run", a, b, crate::CAP_NONE)
}

pub fn self_test() -> bool {
    let ops = [
        Op::LocalGet(0),
        Op::LocalGet(1),
        Op::I32Mul,
        Op::I32Const(7),
        Op::I32Add,
    ];
    matches!(build_and_run_2(&ops, 6, 7), Ok(49))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_mul_add() {
        let (n, ops) = compile_expression("a*b+7").expect("parse");
        assert_eq!(n, 2);
        let wasm = build_run_module(n, &ops).expect("build");
        assert_eq!(crate::run_i32_2(&wasm, "run", 6, 7, crate::CAP_NONE).unwrap(), 49);
    }

    #[test]
    fn compile_literal() {
        let (n, ops) = compile_return_literal("return 42").expect("parse");
        assert_eq!(n, 0);
        let wasm = build_run_module(n, &ops).expect("build");
        assert_eq!(crate::run_i32_0(&wasm, "run", crate::CAP_NONE).unwrap(), 42);
    }

    #[test]
    fn rejects_invalid() {
        assert!(compile_expression("").is_err());
        assert!(compile_expression("a+").is_err());
    }
}
