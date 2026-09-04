//! hermesd — orquestrador Hermes Redox AIOS.

mod lifecycle_poll;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use agent_core::{collect_stack_backends, lifecycle_agent_names, register_fleet};
use event_bus::emit_boot_ai;
use hermes_core::{
    boot_observe, boot_skill_registry, event_client::EventClient, format_boot_report, HermesRouter,
    DEFAULT_HERMES_SOCKET, TOPIC_FACTORY_BOOT, TOPIC_HERMES_RESPONSE, TOPIC_USER_INTENT,
};

const FLEET: &[(&str, &str, Option<&str>)] = &[
    ("eventd", "system", Some("127.0.0.1:7740")),
    ("sgdbd", "system", Some("127.0.0.1:7741")),
    ("hermesd", "router", Some("127.0.0.1:7742")),
    ("cortexd", "inference", Some("127.0.0.1:7743")),
    ("voiced", "skill", Some("127.0.0.1:7744")),
    ("jarbasd", "console", Some("127.0.0.1:7745")),
    ("optimizer", "inference", None),
    ("sleep_cycle", "system", None),
    ("auto_learn", "skill", None),
    ("self_heal", "system", None),
];

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

fn register_aios_fleet() {
    match register_fleet(FLEET) {
        Ok(n) => log_line(&format!("[hermesd] aios: registry {n} agentes")),
        Err(e) => log_line(&format!("[hermesd] aios: registry WARN: {e}")),
    }
    log_line(&format!(
        "[hermesd] lifecycle agents: {}",
        lifecycle_agent_names().join(", ")
    ));
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
            Some("backends") => serde_json::json!({
                "ok": true,
                "result": collect_stack_backends(),
            })
            .to_string(),
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

    let events = EventClient::new();
    let (registry, boot) = boot_skill_registry();
    log_line(&format!("[hermesd] {}", format_boot_report(&boot)));
    let _ = events.publish(
        TOPIC_FACTORY_BOOT,
        &serde_json::json!({
            "skills_md": boot.load.skill_md_loaded,
            "skills_wasm": boot.load.wasm_loaded,
            "skills_dir": boot.skills_dir,
            "hitl": boot.backends.hitl_enabled,
            "tools_net": boot.backends.tools_net,
        })
        .to_string(),
    );

    let router = HermesRouter::with_events(registry, events.clone());

    register_aios_fleet();

    match boot_observe::boot_observe_and_remember() {
        Ok(ev) => log_line(&format!("[hermesd] boot_observe OK: {ev}")),
        Err(e) => log_line(&format!("[hermesd] boot_observe WARN: {e}")),
    }

    {
        let sgdb = hermes_core::sgdb_client::SgdbClient::new();
        let heal = hermes_core::run_self_heal(&sgdb, &events);
        log_line(&format!("[hermesd] self_heal boot:\n{heal}"));
    }

    match agent_core::bootstrap_caps() {
        Ok(store) => log_line(&format!(
            "[hermesd] caps bootstrap source={} grants=[{}]",
            store.source,
            store.grant_names().join(",")
        )),
        Err(e) => log_line(&format!("[hermesd] caps bootstrap WARN: {e}")),
    }

    match agent_core::bootstrap_redox_ns() {
        Ok(ns) => log_line(&format!(
            "[hermesd] redox ns role={} backend={} schemes=[{}] {}",
            ns.role,
            ns.backend,
            ns.schemes.join(","),
            agent_core::redox_caps_summary()
        )),
        Err(e) => log_line(&format!("[hermesd] redox ns WARN: {e}")),
    }

    emit_boot_ai("hermesd");

    let _poll_stop = lifecycle_poll::spawn_lifecycle_poll(events.clone(), |msg| log_line(msg));

    let bind = std::env::var("REDOX_HERMES_SOCKET")
        .unwrap_or_else(|_| DEFAULT_HERMES_SOCKET.to_string());
    let listener = TcpListener::bind(&bind).unwrap_or_else(|e| {
        log_line(&format!("[hermesd] FATAL bind {bind}: {e}"));
        std::process::exit(1);
    });

    let _ = fs::write("/tmp/hermesd.pid", std::process::id().to_string());
    log_line(&format!("[hermesd] intent socket em {bind}"));
    log_line("[hermesd] skills: echo time status remember recall help skills factory opir promote lifecycle selfheal ota caps | cmds: intent backends ping");

    let factory = router.handle_intent("/factory");
    log_line(&format!(
        "[hermesd] factory self-test: {}",
        factory.response.lines().next().unwrap_or("")
    ));
    let opir = router.handle_intent("/opir a*b+7");
    log_line(&format!("[hermesd] op-IR self-test: {}", opir.response));

    for stream in listener.incoming().flatten() {
        handle_client(&router, &events, stream);
    }
}
