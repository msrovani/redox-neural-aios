//! Caps kernel Redox — namespace de schemes como capability (ADR-001/002/011).
//! Espelha o modelo Redox: processo só vê schemes no seu namespace (nsmgr/userspace).
//! Sem patch de kernel: perfil persistido + probe + enforcement CapGate.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::os_caps::{caps_dir, load_cap_store, CapStore};
use crate::scheme_caps::{
    wasm_caps_from_grants, CAP_FS_BIT, CAP_LOG_BIT, CAP_NET_BIT, GRANT_FACTORY, GRANT_PKG_INSTALL,
    SCHEME_AIOS,
};

/// Schemes base sempre presentes no namespace AIOS (boot daemons).
pub const BASE_SCHEMES: &[&str] = &["file", "chan", "aios", "event", "null"];

/// Base restrita para skills WASM (sem `file` até grant `fs_read`).
pub const WASM_BASE_SCHEMES: &[&str] = &["chan", "aios", "null"];

fn base_for_role(role: &str) -> &'static [&'static str] {
    if role.eq_ignore_ascii_case("wasm_skill") {
        WASM_BASE_SCHEMES
    } else {
        BASE_SCHEMES
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapBackend {
    /// Host/dev: perfil JSON sob `/scheme/aios/caps/ns/`.
    ProfileBridge,
    /// Target Redox: probes `open`/`/scheme/*` (prep nsmgr FD).
    SchemeProbe,
    /// Futuro: FD de namespace via nsmgr/redox-rt.
    NsmgrFd,
}

impl CapBackend {
    pub fn detect() -> Self {
        if std::env::var("REDOX_NSMGR_FD")
            .ok()
            .filter(|v| !v.is_empty())
            .is_some()
        {
            return Self::NsmgrFd;
        }
        if is_redox_target() {
            return Self::SchemeProbe;
        }
        Self::ProfileBridge
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProfileBridge => "profile_bridge",
            Self::SchemeProbe => "scheme_probe",
            Self::NsmgrFd => "nsmgr_fd",
        }
    }
}

fn is_redox_target() -> bool {
    std::env::var("REDOX_OS_TARGET")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("redox") || v.eq_ignore_ascii_case("true"))
        .unwrap_or(cfg!(target_os = "redox"))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamespaceProfile {
    pub role: String,
    pub backend: String,
    pub schemes: Vec<String>,
    pub grants: Vec<String>,
    pub wasm_caps: u32,
}

impl NamespaceProfile {
    pub fn format(&self) -> String {
        format!(
            "=== REDOX NS ===\nrole={}\nbackend={}\nschemes=[{}]\ngrants=[{}]\nwasm_caps=0x{:x}\n=== END REDOX NS ===",
            self.role,
            self.backend,
            self.schemes.join(","),
            self.grants.join(","),
            self.wasm_caps
        )
    }

    pub fn allows_scheme(&self, scheme: &str) -> bool {
        let key = scheme.trim_end_matches(':').to_ascii_lowercase();
        self.schemes.iter().any(|s| s == &key)
    }
}

/// Mapeia grant CapGate → schemes Redox visíveis no namespace.
pub fn schemes_for_grant(grant: &str) -> &'static [&'static str] {
    match grant.to_ascii_lowercase().as_str() {
        "factory_exec" => &["aios", "chan"],
        "memory_recall" => &["memory"],
        "net_fetch" => &["tcp", "udp", "dns", "net"],
        "fs_read" => &["file"],
        "pkg_install" | "ota_apply" => &["file", "pkg"],
        "hitl_approve" => &["aios"],
        _ => &[],
    }
}

pub fn grant_for_scheme(scheme: &str) -> Option<&'static str> {
    match scheme.trim_end_matches(':').to_ascii_lowercase().as_str() {
        "memory" => Some("memory_recall"),
        "tcp" | "udp" | "dns" | "net" => Some("net_fetch"),
        "file" => Some("fs_read"),
        "pkg" => Some(GRANT_PKG_INSTALL),
        "aios" => Some(GRANT_FACTORY),
        "chan" => Some(GRANT_FACTORY),
        _ => None,
    }
}

/// Constrói namespace efetivo a partir do CapStore (paridade Redox namespace).
pub fn build_namespace(role: &str, store: &CapStore) -> NamespaceProfile {
    let mut schemes: Vec<String> = base_for_role(role).iter().map(|s| (*s).to_string()).collect();
    for grant in store.grant_names() {
        for s in schemes_for_grant(&grant) {
            schemes.push((*s).to_string());
        }
    }
    schemes.sort();
    schemes.dedup();

    let mut profile = NamespaceProfile {
        role: role.into(),
        backend: CapBackend::detect().as_str().into(),
        schemes,
        grants: store.grant_names(),
        wasm_caps: 0,
    };
    profile.wasm_caps = wasm_caps_from_namespace(&profile);
    // Fallback: se grants env/cache pedem bits extras (tools_net), une à bitmask.
    profile.wasm_caps |= wasm_caps_from_grants();
    profile
}

pub fn ns_dir() -> PathBuf {
    caps_dir().join("ns")
}

pub fn ns_path(role: &str) -> PathBuf {
    ns_dir().join(format!("{role}.json"))
}

pub fn save_namespace(profile: &NamespaceProfile) -> Result<PathBuf, String> {
    fs::create_dir_all(ns_dir()).map_err(|e| e.to_string())?;
    let path = ns_path(&profile.role);
    let json = serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    std::env::set_var("REDOX_NS_SCHEMES", profile.schemes.join(","));
    std::env::set_var("REDOX_CAP_BACKEND", &profile.backend);
    Ok(path)
}

pub fn load_namespace(role: &str) -> Result<NamespaceProfile, String> {
    let path = ns_path(role);
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Bootstrap: materializa perfil `hermes` (+ `wasm_skill` restrito) a partir dos grants.
pub fn bootstrap_redox_ns() -> Result<NamespaceProfile, String> {
    let store = load_cap_store();
    let hermes = build_namespace("hermes", &store);
    save_namespace(&hermes)?;
    write_nsmgr_hint(&hermes)?;

    let mut wasm_store = store.clone();
    wasm_store.grants.retain(|g| {
        matches!(
            g.grant.as_str(),
            "factory_exec" | "memory_recall" | "net_fetch" | "fs_read"
        )
    });
    if wasm_store.grants.is_empty() {
        wasm_store.upsert(GRANT_FACTORY, SCHEME_AIOS, false);
    }
    let wasm = build_namespace("wasm_skill", &wasm_store);
    save_namespace(&wasm)?;
    Ok(hermes)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeProbe {
    pub scheme: String,
    pub allowed: bool,
    pub present: bool,
    pub detail: String,
}

/// Probe honesto: no target tenta paths Redox; no host verifica bridge `/scheme/*`.
pub fn probe_scheme(scheme: &str, profile: &NamespaceProfile) -> SchemeProbe {
    let name = scheme.trim_end_matches(':').to_ascii_lowercase();
    let allowed = profile.allows_scheme(&name);
    if !allowed {
        return SchemeProbe {
            scheme: name,
            allowed: false,
            present: false,
            detail: "fora do namespace (CapGate deny)".into(),
        };
    }

    let (present, detail) = match CapBackend::detect() {
        CapBackend::NsmgrFd => (true, "nsmgr FD presente (assume routed)".into()),
        CapBackend::SchemeProbe => probe_redox_scheme_path(&name),
        CapBackend::ProfileBridge => probe_host_scheme_bridge(&name),
    };

    SchemeProbe {
        scheme: name,
        allowed: true,
        present,
        detail,
    }
}

fn probe_redox_scheme_path(name: &str) -> (bool, String) {
    let candidates = [
        PathBuf::from(format!("/{name}")),
        PathBuf::from(format!("/scheme/{name}")),
        PathBuf::from(format!("/scheme/{name}/.keep")),
    ];
    for p in &candidates {
        if p.exists() {
            return (true, format!("path ok {}", p.display()));
        }
    }
    (
        false,
        "scheme não visível em /scheme (nsmgr/daemon?)".into(),
    )
}

fn probe_host_scheme_bridge(name: &str) -> (bool, String) {
    let root = std::env::var("REDOX_SCHEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/scheme"));
    let p = root.join(name);
    if p.exists() {
        return (true, format!("bridge {}", p.display()));
    }
    let known = ["aios", "memory", "chan", "file", "event", "null", "audio"];
    if known.iter().any(|k| *k == name) {
        return (true, "known AIOS scheme (host bridge)".into());
    }
    (false, format!("ausente em {}", p.display()))
}

pub fn probe_namespace(profile: &NamespaceProfile) -> Vec<SchemeProbe> {
    profile
        .schemes
        .iter()
        .map(|s| probe_scheme(s, profile))
        .collect()
}

pub fn format_probes(probes: &[SchemeProbe]) -> String {
    let mut out = String::from("=== NS PROBE ===\n");
    for p in probes {
        out.push_str(&format!(
            "  {} allowed={} present={} — {}\n",
            p.scheme, p.allowed, p.present, p.detail
        ));
    }
    out.push_str("=== END NS PROBE ===");
    out
}

/// Enforcement CapGate: scheme permitido no namespace efetivo?
pub fn scheme_allowed(scheme: &str) -> bool {
    let role = std::env::var("REDOX_CAP_ROLE").unwrap_or_else(|_| "hermes".into());
    if let Ok(profile) = load_namespace(&role) {
        return profile.allows_scheme(scheme);
    }
    let store = load_cap_store();
    build_namespace(&role, &store).allows_scheme(scheme)
}

/// Converte namespace → bitmask WASM (enforce alinhado ao CapGate).
pub fn wasm_caps_from_namespace(profile: &NamespaceProfile) -> u32 {
    let mut caps = 0u32;
    if profile.allows_scheme("aios") || profile.allows_scheme("chan") {
        caps |= CAP_LOG_BIT;
    }
    if profile.allows_scheme("tcp")
        || profile.allows_scheme("udp")
        || profile.allows_scheme("net")
    {
        caps |= CAP_NET_BIT;
    }
    if profile.allows_scheme("file") {
        caps |= CAP_FS_BIT;
    }
    caps
}

pub fn redox_caps_summary() -> String {
    let store = load_cap_store();
    let profile = build_namespace("hermes", &store);
    format!(
        "backend={} ns=[{}] grants=[{}] wasm=0x{:x}",
        profile.backend,
        profile.schemes.join(","),
        profile.grants.join(","),
        profile.wasm_caps
    )
}

/// Exporta allowlist no formato esperado por scripts / futuro nsmgr.
pub fn export_nsmgr_hint(profile: &NamespaceProfile) -> String {
    format!(
        "# Redox namespace hint (userspace CapGate → nsmgr)\n# role={}\n# REDOX_NS_SCHEMES={}\n# attach when nsmgr FD available: REDOX_NSMGR_FD=<fd>\n",
        profile.role,
        profile.schemes.join(",")
    )
}

pub fn write_nsmgr_hint(profile: &NamespaceProfile) -> Result<PathBuf, String> {
    fs::create_dir_all(ns_dir()).map_err(|e| e.to_string())?;
    let path = ns_dir().join(format!("{}.nsmgr.txt", profile.role));
    fs::write(&path, export_nsmgr_hint(profile)).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os_caps::{save_cap_store, CapStore};
    use crate::scheme_caps::{GRANT_HITL, SCHEME_MEMORY, SCHEME_NET};

    #[test]
    fn namespace_includes_memory_when_granted() {
        let root = std::env::temp_dir().join(format!("redox-ns-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        std::env::set_var("REDOX_AIOS_CAPS_ROOT", &root);
        std::env::remove_var("REDOX_AIOS_CAPS");

        let mut store = CapStore::default();
        store.upsert("memory_recall", SCHEME_MEMORY, false);
        save_cap_store(&store).unwrap();
        let ns = build_namespace("hermes", &store);
        assert!(ns.allows_scheme("memory"));
        assert!(ns.allows_scheme("aios"));
        assert!(!ns.allows_scheme("tcp"));

        let path = save_namespace(&ns).unwrap();
        assert!(path.is_file());
        let probes = probe_namespace(&ns);
        assert!(probes.iter().any(|p| p.scheme == "aios" && p.allowed));

        std::env::remove_var("REDOX_AIOS_CAPS_ROOT");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn wasm_caps_follow_net_scheme() {
        let mut store = CapStore::default();
        store.upsert("net_fetch", SCHEME_NET, false);
        let ns = build_namespace("wasm_skill", &store);
        assert!(wasm_caps_from_namespace(&ns) & CAP_NET_BIT != 0);
    }

    #[test]
    fn grant_scheme_roundtrip() {
        assert_eq!(grant_for_scheme("tcp:"), Some("net_fetch"));
        assert!(schemes_for_grant(GRANT_HITL).contains(&"aios"));
    }
}
