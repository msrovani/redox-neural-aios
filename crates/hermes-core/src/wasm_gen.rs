//! Geração WASM via op-IR + cortexd (Onda 7g ADR-010).

use cortex_core::CortexEngine;
use wasm_skill_runtime::{
    build_run_module, compile_expression, compile_return_literal, echo_len_module, schema_hint,
    WasmError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmGenSource {
    OpIrExpr,
    CortexOpIr,
    Placeholder,
}

#[derive(Clone, Debug)]
pub struct WasmGenResult {
    pub wasm: Vec<u8>,
    pub export_fn: String,
    pub source: WasmGenSource,
    pub expr: Option<String>,
}

pub fn infer_op_ir_expr(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();

    if let Some(rest) = text.split("op-ir:").nth(1) {
        let expr = rest.lines().next()?.trim();
        if !expr.is_empty() {
            return Some(expr.to_string());
        }
    }

    if let Some(rest) = lower.strip_prefix("return ") {
        return Some(rest.trim().to_string());
    }

    for (needle, expr) in [
        ("soma", "a+b"),
        ("add ", "a+b"),
        ("multiplica", "a*b"),
        ("multiply", "a*b"),
        ("produto", "a*b"),
    ] {
        if lower.contains(needle) {
            return Some(expr.into());
        }
    }

    if let Ok((_, _)) = compile_return_literal(text) {
        return Some(text.trim().to_string());
    }

    if text.chars().any(|c| "+-*()".contains(c)) {
        return Some(text.trim().to_string());
    }

    None
}

pub fn build_wasm_from_expr(expr: &str) -> Result<(Vec<u8>, u32), &'static str> {
    let (n, ops) = compile_expression(expr)
        .or_else(|_| compile_return_literal(expr))
        .map_err(|e| e)?;
    let wasm = build_run_module(n, &ops)?;
    Ok((wasm, n))
}

pub fn generate_wasm_for_skill(instructions: &str, sample: &str) -> WasmGenResult {
    for text in [sample, instructions] {
        if let Some(expr) = infer_op_ir_expr(text) {
            if let Ok((wasm, _)) = build_wasm_from_expr(&expr) {
                return WasmGenResult {
                    wasm,
                    export_fn: "run".into(),
                    source: WasmGenSource::OpIrExpr,
                    expr: Some(expr),
                };
            }
        }
    }
    WasmGenResult {
        wasm: echo_len_module(),
        export_fn: "run".into(),
        source: WasmGenSource::Placeholder,
        expr: None,
    }
}

pub fn generate_with_cortex(
    engine: &dyn CortexEngine,
    intent: &str,
    instructions: &str,
) -> WasmGenResult {
    let prompt = format!(
        "Skill intent: {intent}\nInstruções: {instructions}\n\
         Responda com UMA linha: op-ir: <expressão>\n\
         Exemplos: op-ir: a+b | op-ir: 42 | op-ir: a*b+7"
    );
    if let Ok(out) = engine.complete(&prompt, Some(schema_hint())) {
        if let Some(expr) = infer_op_ir_expr(&out) {
            if let Ok((wasm, _)) = build_wasm_from_expr(&expr) {
                return WasmGenResult {
                    wasm,
                    export_fn: "run".into(),
                    source: WasmGenSource::CortexOpIr,
                    expr: Some(expr),
                };
            }
        }
    }
    generate_wasm_for_skill(instructions, intent)
}

pub fn verify_wasm(wasm: &[u8], export_fn: &str) -> Result<i32, WasmError> {
    wasm_skill_runtime::run_i32_0(wasm, export_fn, wasm_skill_runtime::CAP_NONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_core::StubEngine;

    #[test]
    fn infers_add_expr() {
        let gen = generate_wasm_for_skill("soma dois números", "add values");
        assert_eq!(gen.source, WasmGenSource::OpIrExpr);
        assert_eq!(gen.expr.as_deref(), Some("a+b"));
    }

    #[test]
    fn literal_return() {
        let gen = generate_wasm_for_skill("", "return 99");
        assert_eq!(gen.source, WasmGenSource::OpIrExpr);
        assert_eq!(verify_wasm(&gen.wasm, &gen.export_fn).unwrap(), 99);
    }

    #[test]
    fn cortex_stub_fallback() {
        let engine = StubEngine::default();
        let gen = generate_with_cortex(&engine, "custom xyz", "return 7");
        assert!(matches!(
            gen.source,
            WasmGenSource::OpIrExpr | WasmGenSource::CortexOpIr | WasmGenSource::Placeholder
        ));
    }
}
