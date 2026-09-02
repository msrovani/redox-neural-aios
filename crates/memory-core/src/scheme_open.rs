//! Bridge URI `memory:` via diretório `open/` — Fase 2b até handler `open()` nativo Redox.
//!
//! Cliente grava `{root}/open/in/{id}.uri`, sgdbd responde em `{root}/open/out/{id}.json`.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::scheme_uri::{health_uri, parse_memory_uri, recall_uri, remember_uri};

const RPC_TIMEOUT: Duration = Duration::from_secs(5);

pub fn open_root(root: &Path) -> PathBuf {
    root.join("open")
}

pub fn ensure_open_dirs(root: &Path) -> Result<(), String> {
    fs::create_dir_all(open_root(root).join("in")).map_err(|e| e.to_string())?;
    fs::create_dir_all(open_root(root).join("out")).map_err(|e| e.to_string())?;
    Ok(())
}

/// Converte URI `memory:` em corpo JSON-RPC para `handle_request`.
pub fn uri_to_body(uri: &str) -> Result<serde_json::Value, String> {
    let params = parse_memory_uri(uri)?;
    let op = params
        .get("op")
        .map(String::as_str)
        .ok_or_else(|| "URI memory: sem operação".to_string())?;
    match op {
        "remember" => {
            let text = params
                .get("text")
                .ok_or_else(|| "remember: falta text".to_string())?;
            let mut body = serde_json::json!({ "cmd": "remember", "text": text });
            if let Some(s) = params.get("scope") {
                body["scope"] = s.clone().into();
            }
            Ok(body)
        }
        "recall" => {
            let query = params
                .get("query")
                .ok_or_else(|| "recall: falta query".to_string())?;
            let k = params
                .get("k")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(5);
            let mut body = serde_json::json!({ "cmd": "recall", "query": query, "k": k });
            if let Some(s) = params.get("scope") {
                body["scope"] = s.clone().into();
            }
            Ok(body)
        }
        "health" => Ok(serde_json::json!({ "cmd": "health" })),
        "ping" => Ok(serde_json::json!({ "cmd": "ping" })),
        other => Err(format!("operação memory: desconhecida: {other}")),
    }
}

pub fn body_to_uri(body: &serde_json::Value) -> Result<String, String> {
    match body.get("cmd").and_then(|c| c.as_str()) {
        Some("remember") => {
            let text = body
                .get("text")
                .and_then(|t| t.as_str())
                .ok_or_else(|| "remember: falta text".to_string())?;
            let scope = body.get("scope").and_then(|s| s.as_str());
            Ok(remember_uri(text, scope))
        }
        Some("recall") => {
            let query = body
                .get("query")
                .or_else(|| body.get("text"))
                .and_then(|t| t.as_str())
                .ok_or_else(|| "recall: falta query".to_string())?;
            let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let scope = body.get("scope").and_then(|s| s.as_str());
            Ok(recall_uri(query, scope, k))
        }
        Some("health") => Ok(health_uri().to_string()),
        Some("ping") => Ok("memory:ping".to_string()),
        Some(other) => Err(format!("cmd não mapeável para URI: {other}")),
        None => Err("cmd ausente".into()),
    }
}

fn request_id() -> String {
    format!(
        "{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

fn parse_response(response: &str) -> Result<serde_json::Value, String> {
    let line = response.lines().next().unwrap_or(response).trim();
    let val: serde_json::Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(val.get("result").cloned().unwrap_or(serde_json::Value::Null))
    } else {
        Err(val
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("scheme memory URI erro")
            .to_string())
    }
}

/// RPC via URI — canal `open/` (Fase 2b).
pub fn rpc_uri(root: &Path, uri: &str) -> Result<serde_json::Value, String> {
    ensure_open_dirs(root)?;
    let id = request_id();
    let req_path = open_root(root).join("in").join(format!("{id}.uri"));
    let resp_path = open_root(root).join("out").join(format!("{id}.json"));

    fs::write(&req_path, uri).map_err(|e| e.to_string())?;

    let deadline = Instant::now() + RPC_TIMEOUT;
    loop {
        if resp_path.exists() {
            let raw = fs::read_to_string(&resp_path).map_err(|e| e.to_string())?;
            let _ = fs::remove_file(&resp_path);
            let _ = fs::remove_file(&req_path);
            return parse_response(&raw);
        }
        if Instant::now() >= deadline {
            let _ = fs::remove_file(&req_path);
            return Err(format!(
                "scheme memory URI: timeout aguardando {} (sgdbd open watcher?)",
                resp_path.display()
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn rpc_uri_at(uri: &str) -> Result<serde_json::Value, String> {
    rpc_uri(&crate::scheme::scheme_root(), uri)
}

pub fn rpc_body(body: serde_json::Value) -> Result<serde_json::Value, String> {
    let uri = body_to_uri(&body)?;
    rpc_uri_at(&uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_to_body_health() {
        let body = uri_to_body("memory:health").unwrap();
        assert_eq!(body.get("cmd").and_then(|c| c.as_str()), Some("health"));
    }

    #[test]
    fn body_to_uri_roundtrip_remember() {
        let body = serde_json::json!({ "cmd": "remember", "text": "ok", "scope": "boot" });
        let uri = body_to_uri(&body).unwrap();
        let back = uri_to_body(&uri).unwrap();
        assert_eq!(back, body);
    }
}
