//! Boot observer — Observe→Plan→Remember + probe TCP + DeviceTree host (ADR-001/007).

use agent_core::{collect_stack_backends, probe_tcp, BackendTier};
use serde::Serialize;

use crate::event_client::EventClient;
use crate::sgdb_client::SgdbClient;

const BOOT_PHASES: &[&str] = &[
    "SafeHarbor",
    "MemoryCore",
    "SystemBringup",
    "Diagnostics",
    "AgentFleet",
    "Runtime",
];

const DAEMON_SOCKETS: &[(&str, &str)] = &[
    ("eventd", "127.0.0.1:7740"),
    ("sgdbd", "127.0.0.1:7741"),
    ("hermesd", "127.0.0.1:7742"),
    ("cortexd", "127.0.0.1:7743"),
    ("voiced", "127.0.0.1:7744"),
    ("jarbasd", "127.0.0.1:7745"),
];

#[derive(Clone, Debug, Serialize)]
pub struct DeviceNode {
    pub id: String,
    pub class: String,
    pub driver: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct BootReport {
    pub hostname: String,
    pub phases: Vec<&'static str>,
    pub daemons: Vec<&'static str>,
    pub daemons_online: Vec<String>,
    pub device_tree: Vec<DeviceNode>,
    pub backends_stub: usize,
    pub score: u8,
}

impl BootReport {
    pub fn collect() -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "redox-neural-aios".into());

        let daemons = DAEMON_SOCKETS.iter().map(|(n, _)| *n).collect();
        let daemons_online: Vec<String> = DAEMON_SOCKETS
            .iter()
            .filter(|(_, addr)| probe_tcp(addr, 200))
            .map(|(name, _)| (*name).to_string())
            .collect();

        let device_tree = collect_device_tree();
        let backends = collect_stack_backends();
        let backends_stub = backends
            .iter()
            .filter(|b| b.tier == BackendTier::Stub)
            .count();

        let mut score = 20u8;
        score += (daemons_online.len() as u8).saturating_mul(10).min(60);
        if backends_stub == 0 {
            score = score.saturating_add(10);
        } else if backends_stub <= 2 {
            score = score.saturating_add(5);
        }
        score = score.min(100);

        Self {
            hostname,
            phases: BOOT_PHASES.to_vec(),
            daemons,
            daemons_online,
            device_tree,
            backends_stub,
            score,
        }
    }

    pub fn format_score(&self) -> String {
        let mut out = String::from("=== BOOT SCORE ===\n");
        out.push_str(&format!("host={}\n", self.hostname));
        out.push_str(&format!("score={}/100\n", self.score));
        out.push_str(&format!("daemons_online={}/{}\n", self.daemons_online.len(), self.daemons.len()));
        out.push_str("phases=");
        out.push_str(
            &self
                .phases
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
        out.push_str("daemons=");
        out.push_str(
            &self
                .daemons
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
        out.push_str("online=");
        out.push_str(&self.daemons_online.join(","));
        out.push('\n');
        out.push_str(&format!("device_nodes={}\n", self.device_tree.len()));
        out.push_str(&format!("backends_stub={}\n", self.backends_stub));
        out.push_str("=== END BOOT SCORE ===");
        out
    }
}

pub fn collect_device_tree() -> Vec<DeviceNode> {
    let mut nodes = Vec::new();

    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    nodes.push(DeviceNode {
        id: "cpu0".into(),
        class: "processor".into(),
        driver: Some("host".into()),
        detail: format!("logical_cores={cores}"),
    });

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    nodes.push(DeviceNode {
        id: "platform0".into(),
        class: "platform".into(),
        driver: None,
        detail: format!("os={os} arch={arch}"),
    });

    if let Ok(lang) = std::env::var("REDOX_LANG") {
        nodes.push(DeviceNode {
            id: "locale0".into(),
            class: "config".into(),
            driver: None,
            detail: format!("locale={lang}"),
        });
    }

    nodes
}

pub fn publish_boot_evidence(events: &EventClient, report: &BootReport) {
    let _ = events.publish("BOOT_AI", "boot_observe_complete");
    let _ = event_bus::chan::publish_file("BOOT_AI", "boot_observe_complete");
    for phase in &report.phases {
        let _ = events.publish("BOOT_PHASE", phase);
    }
}

pub fn boot_observe_and_remember() -> Result<String, String> {
    let sgdb = SgdbClient::new();
    let events = EventClient::new();
    let report = BootReport::collect();
    publish_boot_evidence(&events, &report);

    let score_block = report.format_score();
    let device_json = serde_json::to_string(&report.device_tree).unwrap_or_else(|_| "[]".into());
    let evidence = format!(
        "boot_observe host={} score={} online={}/{} devices={} stub_backends={}",
        report.hostname,
        report.score,
        report.daemons_online.len(),
        report.daemons.len(),
        report.device_tree.len(),
        report.backends_stub
    );
    sgdb.remember(&evidence, "boot")?;
    sgdb.remember(&score_block, "boot/score")?;
    sgdb.remember(&device_json, "boot/devices")?;
    Ok(format!("{evidence}\n{score_block}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_score_format() {
        let report = BootReport::collect();
        let s = report.format_score();
        assert!(s.contains("=== BOOT SCORE ==="));
        assert!(s.contains("daemons_online="));
    }

    #[test]
    fn device_tree_has_cpu() {
        let tree = collect_device_tree();
        assert!(tree.iter().any(|n| n.class == "processor"));
    }
}
