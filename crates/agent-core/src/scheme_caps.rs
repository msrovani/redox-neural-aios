//! CapGate ↔ scheme capabilities Redox (ADR-001 Fase 2 / ADR-011).
//! Grants: scheme `aios:/caps` → env `REDOX_AIOS_CAPS` → default seguro.

use crate::os_caps::{cached_grants, grant_active_os, refresh_caps_cache};

/// Schemes Redox usados pela factory (userspace).
pub const SCHEME_MEMORY: &str = "memory:";
pub const SCHEME_NET: &str = "net:";
pub const SCHEME_FS: &str = "file:";
pub const SCHEME_AIOS: &str = "aios:";
pub const SCHEME_CHAN: &str = "chan:";

pub const CAP_LOG_BIT: u32 = 1 << 0;
pub const CAP_NET_BIT: u32 = 1 << 1;
pub const CAP_FS_BIT: u32 = 1 << 2;

/// Grant explícito de operador (HITL aprovado).
pub const GRANT_HITL: &str = "hitl_approve";
pub const GRANT_FACTORY: &str = "factory_exec";
pub const GRANT_PKG_INSTALL: &str = "pkg_install";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemeCapGrant {
    pub scheme: &'static str,
    pub grant: &'static str,
    pub wasm_bit: u32,
}

pub fn cap_catalog() -> &'static [SchemeCapGrant] {
    &[
        SchemeCapGrant {
            scheme: SCHEME_AIOS,
            grant: GRANT_FACTORY,
            wasm_bit: CAP_LOG_BIT,
        },
        SchemeCapGrant {
            scheme: SCHEME_NET,
            grant: "net_fetch",
            wasm_bit: CAP_NET_BIT,
        },
        SchemeCapGrant {
            scheme: SCHEME_FS,
            grant: "fs_read",
            wasm_bit: CAP_FS_BIT,
        },
        SchemeCapGrant {
            scheme: SCHEME_MEMORY,
            grant: "memory_recall",
            wasm_bit: 0,
        },
    ]
}

/// Grants ativos: cache scheme/env (ADR-011 caps OS).
pub fn active_grants() -> Vec<String> {
    let mut grants = cached_grants();
    if grants.is_empty() {
        grants = refresh_caps_cache().grant_names();
    }
    if grants.is_empty() {
        return vec![GRANT_FACTORY.into()];
    }
    grants
}

pub fn grant_active(name: &str) -> bool {
    grant_active_os(name)
}

/// Bitmask WASM efetiva a partir de grants (CapGate).
pub fn wasm_caps_from_grants() -> u32 {
    let mut caps = 0u32;
    if !factory_caps_enabled() {
        return 0;
    }
    for entry in cap_catalog() {
        if entry.wasm_bit != 0 && grant_active(entry.grant) {
            caps |= entry.wasm_bit;
        }
    }
    if grant_active(GRANT_FACTORY) {
        caps |= CAP_LOG_BIT;
    }
    if grant_active("net_fetch") || tools_net_env() {
        caps |= CAP_NET_BIT;
    }
    caps
}

pub fn factory_caps_enabled() -> bool {
    std::env::var("REDOX_FACTORY_CAPS")
        .map(|v| v != "0")
        .unwrap_or(true)
}

fn tools_net_env() -> bool {
    std::env::var("REDOX_TOOLS_NET")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Promoção cookbook exige grant explícito além de HITL textual.
pub fn allows_pkg_install(hitl_approved: bool) -> bool {
    hitl_approved && (grant_active(GRANT_PKG_INSTALL) || grant_active(GRANT_HITL))
}

pub fn cap_summary() -> String {
    let store = refresh_caps_cache();
    format!(
        "source={} grants=[{}] wasm_caps=0x{:x}",
        store.source,
        store.grant_names().join(","),
        wasm_caps_from_grants()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_caps_include_log_by_default() {
        let prev = std::env::var("REDOX_AIOS_CAPS").ok();
        let prev_root = std::env::var("REDOX_AIOS_CAPS_ROOT").ok();
        let root = std::env::temp_dir().join("redox-aios-caps-default-test");
        let _ = std::fs::remove_dir_all(&root);
        std::env::set_var("REDOX_AIOS_CAPS_ROOT", &root);
        std::env::remove_var("REDOX_AIOS_CAPS");
        let _ = crate::os_caps::bootstrap_caps();
        assert!(wasm_caps_from_grants() & CAP_LOG_BIT != 0);
        match prev {
            Some(v) => std::env::set_var("REDOX_AIOS_CAPS", v),
            None => std::env::remove_var("REDOX_AIOS_CAPS"),
        }
        match prev_root {
            Some(v) => std::env::set_var("REDOX_AIOS_CAPS_ROOT", v),
            None => std::env::remove_var("REDOX_AIOS_CAPS_ROOT"),
        }
        let _ = std::fs::remove_dir_all(root);
    }
}
