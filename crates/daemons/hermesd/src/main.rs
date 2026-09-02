//! hermesd — orquestrador Hermes Redox AIOS.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use hermes_core::{
    boot_observe, event_client::EventClient, register_builtin_skills, HermesRouter,
    DEFAULT_HERMES_SOCKET, TOPIC_HERMES_RESPONSE, TOPIC_USER_INTENT,
};
use skill_registry::SkillRegistry;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hermesd.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

pub fn handle_intent_line(router: &HermesRouter, events: &EventClient, line: &str) -> String {
    let text = match serde_json::from_str::<serde_json::Value>(line) {
        Ok(v) => v
            .get("text")
            .or_else(|| v.get("intent"))
            .and_then(|t| t.as_str())
            .unwrap_or(line)
            .to_string(),
        Err(_) => line.trim().to_string(),
    };

    let _ = events.publish(TOPIC_USER_INTENT, &text);
    let result = router.handle_intent(&text);
    let _ = events.publish(TOPIC_HERMES_RESPONSE, &result.response);

    serde_json::json!({
        "ok": true,
        "topic": result.topic,
        "response": result.response,
        "trace": result.trace,
    })
    .to_string()
}

fn handle_client(router: &HermesRouter, events: &EventClient, stream: TcpStream) {
    let reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let cmd: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let _ = writeln!(
                    writer,
                    "{}",
                    serde_json::json!({"ok":false,"error":e.to_string()})
                );
                continue;
            }
        };
        let response = match cmd.get("cmd").and_then(|c| c.as_str()) {
            Some("ping") => serde_json::json!({"ok":true,"result":"pong"}).to_string(),
            Some("intent") => handle_intent_line(router, events, &line),
            other => {
                serde_json::json!({"ok":false,"error":format!("cmd invalido: {other:?}")}).to_string()
            }
        };
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

fn main() {
    log_line(&format!("hermesd v{VERSION} — Redox AIOS Hermes orchestrator"));

    let mut registry = SkillRegistry::new();
    register_builtin_skills(&mut registry);
    let router = HermesRouter::new(registry);
    let events = EventClient::new();

    match boot_observe::boot_observe_and_remember() {
        Ok(ev) => log_line(&format!("[hermesd] boot_observe OK: {ev}")),
        Err(e) => log_line(&format!("[hermesd] boot_observe WARN: {e}")),
    }

    let bind = std::env::var("REDOX_HERMES_SOCKET")
        .unwrap_or_else(|_| DEFAULT_HERMES_SOCKET.to_string());
    let listener = TcpListener::bind(&bind).unwrap_or_else(|e| {
        log_line(&format!("[hermesd] FATAL bind {bind}: {e}"));
        std::process::exit(1);
    });

    let _ = fs::write("/tmp/hermesd.pid", std::process::id().to_string());
    log_line(&format!("[hermesd] intent socket em {bind}"));
    log_line("[hermesd] skills: echo time status remember recall help skills");

    for stream in listener.incoming().flatten() {
        handle_client(&router, &events, stream);
    }
}
