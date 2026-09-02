//! Boot observer — Observe→Plan→Remember no SGDB (Fase 2).

use crate::sgdb_client::SgdbClient;

const BOOT_PHASES: &[&str] = &[
    "SafeHarbor",
    "MemoryCore",
    "SystemBringup",
    "Diagnostics",
    "AgentFleet",
    "Runtime",
];

pub struct BootReport {
    pub hostname: String,
    pub phases: Vec<&'static str>,
    pub daemons: Vec<&'static str>,
    pub score: u8,
}

impl BootReport {
    pub fn collect() -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "redox-neural-aios".into());

        let daemons = vec!["eventd", "sgdbd", "hermesd", "cortexd", "voiced", "jarbasd"];
        let score = 60u8
            + (if std::env::var("REDOX_SGDB_SOCKET").is_ok() {
                5
            } else {
                10
            })
            + 15; // stack scaffold presente

        Self {
            hostname,
            phases: BOOT_PHASES.to_vec(),
            daemons,
            score: score.min(100),
        }
    }

    pub fn format_score(&self) -> String {
        let mut out = String::from("=== BOOT SCORE ===\n");
        out.push_str(&format!("host={}\n", self.hostname));
        out.push_str(&format!("score={}/100\n", self.score));
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
        out.push_str("\n=== END BOOT SCORE ===");
        out
    }
}

pub fn boot_observe_and_remember() -> Result<String, String> {
    let sgdb = SgdbClient::new();
    let report = BootReport::collect();
    let score_block = report.format_score();
    let evidence = format!(
        "boot_observe host={} score={} phases={}",
        report.hostname,
        report.score,
        report.phases.len()
    );
    sgdb.remember(&evidence, "boot")?;
    sgdb.remember(&score_block, "boot/score")?;
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
        assert!(s.contains("score="));
    }
}
