//! CapGate ↔ scheme capabilities Redox (ADR-001 Fase 2).
//! Mapeia bitmask WASM (`wasm-skill-runtime`) para schemes OS e grants de env.

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

/// Grants ativos via `REDOX_AIOS_CAPS` (CSV) ou defaults seguros.
pub fn active_grants() -> Vec<String> {
    std::env::var("REDOX_AIOS_CAPS")
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_else(|_| vec!["factory_exec".into()])
}

pub fn grant_active(name: &str) -> bool {
    let key = name.to_ascii_lowercase();
    active_grants().iter().any(|g| g == &key)
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
    let grants = active_grants().join(",");
    format!("grants=[{grants}] wasm_caps=0x{:x}", wasm_caps_from_grants())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_caps_include_log_by_default() {
        let prev = std::env::var("REDOX_AIOS_CAPS").ok();
        std::env::remove_var("REDOX_AIOS_CAPS");
        assert!(wasm_caps_from_grants() & CAP_LOG_BIT != 0);
        match prev {
            Some(v) => std::env::set_var("REDOX_AIOS_CAPS", v),
            None => {}
        }
    }
}
