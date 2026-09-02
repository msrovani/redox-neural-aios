//! Backend scheme `memory:` via arquivos (Fase 1 — bridge até handler nativo).
//!
//! Protocolo: cliente grava `{root}/in/{id}.json`, lê `{root}/out/{id}.json`.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_SCHEME_ROOT: &str = "/scheme/memory";
const RPC_TIMEOUT: Duration = Duration::from_secs(5);

pub fn scheme_root() -> PathBuf {
    std::env::var("REDOX_MEMORY_SCHEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SCHEME_ROOT))
}

pub fn ensure_dirs(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("in")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("out")).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn rpc(body: serde_json::Value) -> Result<serde_json::Value, String> {
    let root = scheme_root();
    ensure_dirs(&root)?;

    let id = format!(
        "{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let req_path = root.join("in").join(format!("{id}.json"));
    let resp_path = root.join("out").join(format!("{id}.json"));

    let line = serde_json::to_string(&body).map_err(|e| e.to_string())?;
    fs::write(&req_path, format!("{line}\n")).map_err(|e| e.to_string())?;

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
                "scheme memory: timeout aguardando {} (sgdbd scheme watcher ativo?)",
                resp_path.display()
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
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
            .unwrap_or("scheme memory erro")
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_root_default() {
        std::env::remove_var("REDOX_MEMORY_SCHEME_ROOT");
        assert_eq!(scheme_root(), PathBuf::from(DEFAULT_SCHEME_ROOT));
    }
}
