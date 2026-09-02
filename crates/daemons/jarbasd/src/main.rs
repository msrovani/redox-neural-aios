//! jarbasd — shell AI Redox AIOS (chat HUD + Soul Mirror + Falcon3 via Hermes).

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use jarbas_core::{
    render_chat_hud, ChatSession, HudFrame, JarbasBridge, SoulMirror, DEFAULT_JARBAS_SOCKET,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct JarbasState {
    bridge: JarbasBridge,
    session: ChatSession,
    mirror: SoulMirror,
    status: String,
}

impl JarbasState {
    fn new() -> Self {
        Self {
            bridge: JarbasBridge::new(),
            session: ChatSession::new(64),
            mirror: SoulMirror::default(),
            status: "pronto".into(),
        }
    }
}

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/jarbasd.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

pub fn handle_request(state: &mut JarbasState, line: &str) -> String {
    let cmd: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({"ok":false,"error":e.to_string()}).to_string(),
    };

    match cmd.get("cmd").and_then(|c| c.as_str()) {
        Some("ping") => serde_json::json!({"ok":true,"result":"pong"}).to_string(),
        Some("status") => serde_json::json!({
            "ok": true,
            "result": {
                "name": state.bridge.soul.name,
                "llm": state.bridge.soul.llm,
                "language": state.bridge.soul.language,
                "orb": state.mirror.emotion.label(),
                "messages": state.session.history().len(),
            }
        })
        .to_string(),
        Some("soul") => serde_json::json!({
            "ok": true,
            "result": {
                "name": state.bridge.soul.name,
                "tone": state.bridge.soul.tone,
                "wake_word": state.bridge.soul.wake_word,
                "llm": state.bridge.soul.llm,
            }
        })
        .to_string(),
        Some("orb") | Some("hud") => {
            let hud = render_chat_hud(&HudFrame {
                soul: &state.bridge.soul,
                mirror: &state.mirror,
                session: &state.session,
                status_line: &state.status,
            });
            serde_json::json!({"ok":true,"result":{"hud":hud,"emotion":state.mirror.emotion.label()}}).to_string()
        }
        Some("history") => serde_json::json!({
            "ok": true,
            "result": state.session.history(),
        })
        .to_string(),
        Some("greet") | Some("boot_greeting") => {
            state.status = "boot greeting".into();
            let response = state
                .bridge
                .boot_greeting(&mut state.session, &mut state.mirror);
            state.status = "online".into();
            serde_json::json!({"ok":true,"result":{"response":response}}).to_string()
        }
        Some("chat") => {
            let text = cmd.get("text").and_then(|t| t.as_str()).unwrap_or("");
            state.status = "pensando...".into();
            let response = state
                .bridge
                .chat(text, &mut state.session, &mut state.mirror);
            state.status = "online".into();
            serde_json::json!({
                "ok": true,
                "result": {
                    "response": response,
                    "emotion": state.mirror.emotion.label(),
                }
            })
            .to_string()
        }
        Some("voice") => {
            let text = cmd.get("text").and_then(|t| t.as_str()).unwrap_or("");
            match state
                .bridge
                .voice_utterance(text, &mut state.session, &mut state.mirror)
            {
                Ok(response) => serde_json::json!({
                    "ok": true,
                    "result": {"response": response, "emotion": state.mirror.emotion.label()}
                })
                .to_string(),
                Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
            }
        }
        other => serde_json::json!({"ok":false,"error":format!("cmd invalido: {other:?}")})
            .to_string(),
    }
}

fn handle_client(state: Arc<Mutex<JarbasState>>, stream: TcpStream) {
    let reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = {
            let mut guard = state.lock().expect("jarbas state");
            handle_request(&mut guard, &line)
        };
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

fn main() {
    log_line(&format!("jarbasd v{VERSION} — Redox Neural AIOS shell AI"));

    let state = Arc::new(Mutex::new(JarbasState::new()));
    {
        let guard = state.lock().expect("jarbas state");
        let _ = std::env::set_var("REDOX_SOUL_LANGUAGE", &guard.bridge.soul.language);
        log_line(&format!(
            "[jarbasd] soul={} llm={} lang={} wake={}",
            guard.bridge.soul.name,
            guard.bridge.soul.llm,
            guard.bridge.soul.language,
            guard.bridge.soul.wake_word
        ));
    }
    {
        let mut guard = state.lock().expect("jarbas state");
        if std::env::var("REDOX_JARBAS_BOOT_GREET")
            .map(|v| v != "0")
            .unwrap_or(true)
        {
            let greet = {
                let JarbasState {
                    bridge,
                    session,
                    mirror,
                    ..
                } = &mut *guard;
                let prompt = bridge.soul.greeting_prompt();
                session.push(jarbas_core::ChatRole::System, "boot greeting");
                bridge.chat(&prompt, session, mirror)
            };
            log_line(&format!(
                "[jarbasd] boot greeting: {}",
                greet.chars().take(80).collect::<String>()
            ));
        }
    }

    let bind = std::env::var("REDOX_JARBAS_SOCKET")
        .unwrap_or_else(|_| DEFAULT_JARBAS_SOCKET.to_string());
    let listener = TcpListener::bind(&bind).unwrap_or_else(|e| {
        log_line(&format!("[jarbasd] FATAL bind {bind}: {e}"));
        std::process::exit(1);
    });

    let _ = fs::write("/tmp/jarbasd.pid", std::process::id().to_string());
    log_line(&format!("[jarbasd] socket em {bind}"));
    log_line("[jarbasd] comandos: chat | hud | greet | history | soul | voice | status | ping");

    for stream in listener.incoming().flatten() {
        handle_client(state.clone(), stream);
    }
}
