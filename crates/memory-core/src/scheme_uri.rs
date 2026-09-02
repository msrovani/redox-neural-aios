//! URIs scheme `memory:` — contrato Fase 2 ADR-005 (handler nativo Redox).

use std::collections::BTreeMap;

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn decode(s: &str) -> Result<String, String> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                .map_err(|e| e.to_string())?;
            let val = u8::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
            out.push(val);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|e| e.to_string())
}

/// `memory:remember?text=...&scope=...`
pub fn remember_uri(text: &str, scope: Option<&str>) -> String {
    let mut q = format!("text={}", encode(text));
    if let Some(s) = scope {
        q.push_str(&format!("&scope={}", encode(s)));
    }
    format!("memory:remember?{q}")
}

/// `memory:recall?query=...&scope=...&k=N`
pub fn recall_uri(query: &str, scope: Option<&str>, k: usize) -> String {
    let mut q = format!("query={}", encode(query));
    if let Some(s) = scope {
        q.push_str(&format!("&scope={}", encode(s)));
    }
    q.push_str(&format!("&k={k}"));
    format!("memory:recall?{q}")
}

pub fn health_uri() -> &'static str {
    "memory:health"
}

/// Parse mínimo para testes e handler futuro.
pub fn parse_memory_uri(uri: &str) -> Result<BTreeMap<String, String>, String> {
    let rest = uri
        .strip_prefix("memory:")
        .ok_or_else(|| "URI não é memory:".to_string())?;
    let (op, query) = rest
        .split_once('?')
        .ok_or_else(|| "URI memory: sem query".to_string())?;
    let mut map = BTreeMap::new();
    map.insert("op".into(), op.to_string());
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), decode(v)?);
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_remember_uri() {
        let uri = remember_uri("boot ok", Some("boot"));
        let parsed = parse_memory_uri(&uri).unwrap();
        assert_eq!(parsed.get("op").map(String::as_str), Some("remember"));
        assert_eq!(parsed.get("text").map(String::as_str), Some("boot ok"));
        assert_eq!(parsed.get("scope").map(String::as_str), Some("boot"));
    }
}
