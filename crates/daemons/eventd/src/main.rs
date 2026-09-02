//! eventd — daemon EventBus do Redox AIOS.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use agent_core::{Agent, AgentKind, AgentManifest, AgentTickResult, ScheduleKind};
use event_bus::{emit_boot_ai, handle_remote, CapabilityToken, Event, EventBus};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONTROL_ADDR: &str = "127.0.0.1:7740";

struct EventdAgent {
    tick: u64,
    boot_phases_emitted: bool,
}

impl EventdAgent {
    fn emit_boot_phases(bus: &EventBus) {
        let phases = [
            "SafeHarbor",
            "MemoryCore",
            "SystemBringup",
            "Diagnostics",
            "AgentFleet",
            "Runtime",
        ];
        let token = CapabilityToken::system("eventd", "boot_phase");
        for phase in phases {
            let _ = bus.publish(Event::new("BOOT_PHASE", phase, token.clone()));
            let _ = event_bus::chan::publish_file("BOOT_PHASE", phase);
            log_line(&format!("[eventd] BOOT_PHASE={phase}"));
        }
        let _ = bus.publish(Event::new("BOOT_AI", "eventd_ready", token));
        let _ = event_bus::chan::publish_file("BOOT_AI", "eventd_ready");
        log_line("[eventd] BOOT_AI=eventd_ready");
    }
}

impl Agent for EventdAgent {
    fn manifest(&self) -> &AgentManifest {
        static MANIFEST: AgentManifest = AgentManifest {
            name: "eventd",
            kind: AgentKind::System,
            schedule: ScheduleKind::Continuous,
            auto_start: true,
            persist: true,
        };
        &MANIFEST
    }

    fn tick(&mut self) -> AgentTickResult {
        self.tick += 1;
        if !self.boot_phases_emitted {
            self.boot_phases_emitted = true;
        }
        AgentTickResult::Pending
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(1)
    }
}

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/eventd.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

fn handle_control_client(bus: Arc<EventBus>, stream: TcpStream) {
    let reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_remote(&bus, &line);
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

fn listen_control_port(bus: Arc<EventBus>) {
    thread::spawn(move || {
        if let Ok(listener) = TcpListener::bind(CONTROL_ADDR) {
            log_line(&format!("[eventd] EventBus remoto em {CONTROL_ADDR}"));
            for stream in listener.incoming().flatten() {
                let b = bus.clone();
                handle_control_client(b, stream);
            }
        }
    });
}

fn main() {
    log_line(&format!("eventd v{VERSION} — Redox AIOS EventBus"));
    let bus = Arc::new(EventBus::new());
    EventdAgent::emit_boot_phases(&bus);
    emit_boot_ai("eventd");
    let _ = fs::write("/tmp/eventd.pid", std::process::id().to_string());
    listen_control_port(bus.clone());

    let mut agent = EventdAgent {
        tick: 0,
        boot_phases_emitted: true,
    };
    loop {
        let _ = agent.tick();
        thread::sleep(agent.poll_interval());
    }
}
