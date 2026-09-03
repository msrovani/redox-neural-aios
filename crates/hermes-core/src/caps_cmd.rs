//! Comando `/caps` — CapGate scheme `aios:/caps` (ADR-011).

use agent_core::{
    bootstrap_caps, grant_active, load_cap_store, refresh_caps_cache, save_cap_store,
    CapStore, GRANT_HITL,
};
use agent_core::os_caps::grant_to_scheme;

use crate::event_client::EventClient;
use crate::sgdb_client::SgdbClient;

pub fn handle_caps(
    arg: &str,
    sgdb: &SgdbClient,
    events: &EventClient,
) -> Result<String, String> {
    let _ = bootstrap_caps();
    let arg = arg.trim();
    if arg.is_empty() || arg.eq_ignore_ascii_case("list") || arg.eq_ignore_ascii_case("status") {
        let store = refresh_caps_cache();
        let _ = sgdb.remember(&store.format(), "hermes/caps");
        return Ok(format!("{}\n{}", store.format(), agent_core::cap_summary()));
    }

    let mut parts = arg.split_whitespace();
    let op = parts.next().unwrap_or("").to_ascii_lowercase();
    match op.as_str() {
        "grant" | "add" => {
            let grant = parts
                .next()
                .ok_or_else(|| "uso: /caps grant <name>".to_string())?
                .to_ascii_lowercase();
            if needs_hitl_to_mutate(&grant) && !grant_active(GRANT_HITL) {
                return Err(format!(
                    "grant {grant} exige hitl_approve em REDOX_AIOS_CAPS ou /caps grant hitl_approve"
                ));
            }
            let mut store = load_cap_store();
            let scheme = grant_to_scheme(&grant);
            store.upsert(&grant, scheme, grant == GRANT_HITL || grant == "pkg_install");
            let path = save_cap_store(&store)?;
            refresh_caps_cache();
            let msg = format!("caps grant ok: {grant} → {scheme} ({})", path.display());
            publish_and_remember(sgdb, events, &store, &msg);
            Ok(format!("{msg}\n{}", store.format()))
        }
        "revoke" | "rm" => {
            if !grant_active(GRANT_HITL) {
                return Err("revoke exige grant hitl_approve".into());
            }
            let grant = parts
                .next()
                .ok_or_else(|| "uso: /caps revoke <name>".to_string())?
                .to_ascii_lowercase();
            let mut store = load_cap_store();
            if !store.revoke(&grant) {
                return Err(format!("grant ausente: {grant}"));
            }
            let path = save_cap_store(&store)?;
            refresh_caps_cache();
            let msg = format!("caps revoke ok: {grant} ({})", path.display());
            publish_and_remember(sgdb, events, &store, &msg);
            Ok(format!("{msg}\n{}", store.format()))
        }
        "bootstrap" => {
            let store = bootstrap_caps()?;
            Ok(format!("caps bootstrap\n{}", store.format()))
        }
        _ => Err(format!(
            "uso: /caps [list|grant <g>|revoke <g>|bootstrap] — recebido: {arg}"
        )),
    }
}

fn needs_hitl_to_mutate(grant: &str) -> bool {
    matches!(
        grant,
        "pkg_install" | "ota_apply" | "net_fetch" | "fs_read"
    )
}

fn publish_and_remember(
    sgdb: &SgdbClient,
    events: &EventClient,
    store: &CapStore,
    msg: &str,
) {
    let _ = sgdb.remember(msg, "hermes/caps");
    let _ = events.publish(
        "CAPS_UPDATE",
        &serde_json::json!({
            "source": store.source,
            "grants": store.grant_names(),
            "detail": msg,
        })
        .to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_list_works() {
        let root = std::env::temp_dir().join("redox-aios-caps-cmd-test");
        let _ = std::fs::remove_dir_all(&root);
        std::env::set_var("REDOX_AIOS_CAPS_ROOT", &root);
        std::env::remove_var("REDOX_AIOS_CAPS");
        let sgdb = SgdbClient::new();
        let events = EventClient::new();
        let out = handle_caps("list", &sgdb, &events).unwrap();
        assert!(out.contains("OS CAPS"));
        std::env::remove_var("REDOX_AIOS_CAPS_ROOT");
        let _ = std::fs::remove_dir_all(root);
    }
}
