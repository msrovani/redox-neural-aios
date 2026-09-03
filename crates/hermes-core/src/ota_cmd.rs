//! Comando `/ota` — skeleton auto-upgrade HITL (ADR-001 / ADR-011).

use agent_core::{apply_update, check_update};

use crate::event_client::EventClient;
use crate::sgdb_client::SgdbClient;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn handle_ota(
    arg: &str,
    sgdb: &SgdbClient,
    events: &EventClient,
) -> Result<String, String> {
    let arg = arg.trim().to_ascii_lowercase();
    let proposal = check_update(VERSION);
    let _ = sgdb.remember(&proposal.format(), "hermes/ota");
    let _ = events.publish(
        "OTA_CHECK",
        &serde_json::json!({
            "channel": proposal.channel,
            "current": proposal.current,
            "candidate": proposal.candidate,
        })
        .to_string(),
    );

    if arg.is_empty() || arg == "check" || arg == "status" {
        return Ok(proposal.format());
    }

    if arg == "approve" || arg.starts_with("approve ") {
        let result = apply_update(&proposal, true)?;
        let _ = sgdb.remember(&result, "hermes/ota");
        let _ = events.publish("OTA_STAGED", &result);
        return Ok(result);
    }

    Err(format!(
        "uso: /ota [check|approve] — recebido: {arg}\n{}",
        proposal.format()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ota_check_returns_proposal() {
        let sgdb = SgdbClient::new();
        let events = EventClient::new();
        let out = handle_ota("check", &sgdb, &events).unwrap();
        assert!(out.contains("OTA PROPOSAL"));
    }
}
