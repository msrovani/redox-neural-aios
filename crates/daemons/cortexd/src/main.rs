//! cortexd — inferência Falcon3-3B-Instruct-1.58bit (Redox AIOS).

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use cortex_core::{engine_from_env, CortexEngine, DEFAULT_CORTEX_SOCKET};
use event_bus::emit_boot_ai;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/cortexd.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

pub fn handle_request(engine: &cortex_core::AdaptiveEngine, line: &str) -> String {
    let cmd: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({"ok":false,"error":e.to_string()}).to_string(),
    };

    match cmd.get("cmd").and_then(|c| c.as_str()) {
        Some("ping") => serde_json::json!({"ok":true,"result":"pong"}).to_string(),
        Some("status") => serde_json::json!({
            "ok": true,
            "result": {
                "engine": engine.engine_name(),
                "tier": engine.backend_tier(),
                "degraded": engine.is_degraded(),
                "model": std::env::var("REDOX_CORTEX_MODEL").unwrap_or_else(|_| "default".into()),
            }
        })
        .to_string(),
        Some("complete") => {
            let prompt = cmd
                .get("prompt")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            let system = cmd.get("system").and_then(|s| s.as_str());
            match engine.complete(prompt, system) {
                Ok(result) => serde_json::json!({"ok":true,"result":result}).to_string(),
                Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
            }
        }
        other => serde_json::json!({"ok":false,"error":format!("cmd invalido: {other:?}")})
            .to_string(),
    }
}

fn handle_client(engine: Arc<cortex_core::AdaptiveEngine>, stream: TcpStream) {
    let reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_request(&engine, &line);
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

fn main() {
    log_line(&format!("cortexd v{VERSION} — Redox AIOS cortex (Falcon3)"));

    let engine = Arc::new(engine_from_env());
    log_line(&format!(
        "[cortexd] engine={} tier={} degraded={}",
        engine.engine_name(),
        engine.backend_tier(),
        engine.is_degraded()
    ));

    emit_boot_ai("cortexd");

    let bind = std::env::var("REDOX_CORTEX_SOCKET")
        .unwrap_or_else(|_| DEFAULT_CORTEX_SOCKET.to_string());
    let listener = TcpListener::bind(&bind).unwrap_or_else(|e| {
        log_line(&format!("[cortexd] FATAL bind {bind}: {e}"));
        std::process::exit(1);
    });

    let _ = fs::write("/tmp/cortexd.pid", std::process::id().to_string());
    log_line(&format!("[cortexd] socket em {bind}"));
    log_line("[cortexd] comandos: complete | status | ping");

    for stream in listener.incoming().flatten() {
        handle_client(engine.clone(), stream);
    }
}
