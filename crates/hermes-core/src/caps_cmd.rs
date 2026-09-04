//! Comando `/caps` — CapGate + namespace Redox (ADR-011).

use agent_core::os_caps::grant_to_scheme;
use agent_core::redox_caps::{format_probes, save_namespace, write_nsmgr_hint};
use agent_core::{
    bootstrap_caps, bootstrap_redox_ns, build_namespace, grant_active, load_cap_store,
    probe_namespace, refresh_caps_cache, save_cap_store, CapStore, GRANT_HITL,
};

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
        let ns = build_namespace("hermes", &store);
        let _ = sgdb.remember(&store.format(), "hermes/caps");
        return Ok(format!(
            "{}\n{}\n{}",
            store.format(),
            ns.format(),
            agent_core::cap_summary()
        ));
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
            let ns = sync_ns_from_store(&store)?;
            let msg = format!(
                "caps grant ok: {grant} → {scheme} ({}) ns=[{}]",
                path.display(),
                ns.schemes.join(",")
            );
            publish_and_remember(sgdb, events, &store, &msg);
            Ok(format!("{msg}\n{}\n{}", store.format(), ns.format()))
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
            let ns = sync_ns_from_store(&store)?;
            let msg = format!("caps revoke ok: {grant} ({})", path.display());
            publish_and_remember(sgdb, events, &store, &msg);
            Ok(format!("{msg}\n{}\n{}", store.format(), ns.format()))
        }
        "bootstrap" => {
            let store = bootstrap_caps()?;
            let ns = bootstrap_redox_ns()?;
            Ok(format!(
                "caps bootstrap\n{}\n{}",
                store.format(),
                ns.format()
            ))
        }
        "ns" | "namespace" => {
            let role = parts.next().unwrap_or("hermes");
            let store = load_cap_store();
            let ns = build_namespace(role, &store);
            let path = save_namespace(&ns)?;
            let hint = write_nsmgr_hint(&ns)?;
            let _ = sgdb.remember(&ns.format(), "hermes/caps");
            Ok(format!(
                "{}\nprofile={}\nhint={}",
                ns.format(),
                path.display(),
                hint.display()
            ))
        }
        "probe" => {
            let role = parts.next().unwrap_or("hermes");
            let store = load_cap_store();
            let ns = build_namespace(role, &store);
            let probes = probe_namespace(&ns);
            let out = format!("{}\n{}", ns.format(), format_probes(&probes));
            let _ = sgdb.remember(&out, "hermes/caps");
            let _ = events.publish(
                "CAPS_PROBE",
                &serde_json::json!({
                    "role": ns.role,
                    "backend": ns.backend,
                    "schemes": ns.schemes,
                })
                .to_string(),
            );
            Ok(out)
        }
        _ => Err(format!(
            "uso: /caps [list|grant <g>|revoke <g>|bootstrap|ns [role]|probe [role]] — recebido: {arg}"
        )),
    }
}

fn sync_ns_from_store(store: &CapStore) -> Result<agent_core::NamespaceProfile, String> {
    let ns = build_namespace("hermes", store);
    save_namespace(&ns)?;
    write_nsmgr_hint(&ns)?;
    let mut wasm_store = store.clone();
    wasm_store.grants.retain(|g| {
        matches!(
            g.grant.as_str(),
            "factory_exec" | "memory_recall" | "net_fetch" | "fs_read"
        )
    });
    if !wasm_store.grants.is_empty() {
        let wasm = build_namespace("wasm_skill", &wasm_store);
        save_namespace(&wasm)?;
    }
    Ok(ns)
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
            "ns": agent_core::redox_caps_summary(),
        })
        .to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_list_and_ns() {
        let root = std::env::temp_dir().join(format!("redox-aios-caps-cmd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::env::set_var("REDOX_AIOS_CAPS_ROOT", &root);
        std::env::remove_var("REDOX_AIOS_CAPS");
        let sgdb = SgdbClient::new();
        let events = EventClient::new();
        let out = handle_caps("list", &sgdb, &events).unwrap();
        assert!(out.contains("OS CAPS"));
        assert!(out.contains("REDOX NS"));
        let probe = handle_caps("probe", &sgdb, &events).unwrap();
        assert!(probe.contains("NS PROBE"));
        std::env::remove_var("REDOX_AIOS_CAPS_ROOT");
        let _ = std::fs::remove_dir_all(root);
    }
}
