//! Caps OS nativas — bridge scheme `aios:/caps` (ADR-001 Fase 2 / ADR-011).
//! Persistência userspace até capabilities Redox kernel; sync com `REDOX_AIOS_CAPS`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::aios_registry::aios_root;
use crate::scheme_caps::{
    GRANT_FACTORY, GRANT_HITL, GRANT_PKG_INSTALL, SCHEME_AIOS, SCHEME_FS, SCHEME_MEMORY,
    SCHEME_NET,
};

const GRANTS_FILE: &str = "grants.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapToken {
    pub grant: String,
    pub scheme: String,
    pub issued_at: u64,
    pub hitl: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapStore {
    pub version: u32,
    pub source: String,
    pub grants: Vec<CapToken>,
}

impl Default for CapStore {
    fn default() -> Self {
        Self {
            version: 1,
            source: "default".into(),
            grants: vec![CapToken {
                grant: GRANT_FACTORY.into(),
                scheme: SCHEME_AIOS.into(),
                issued_at: now_secs(),
                hitl: false,
            }],
        }
    }
}

impl CapStore {
    pub fn grant_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.grants.iter().map(|g| g.grant.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    pub fn has(&self, grant: &str) -> bool {
        let key = grant.to_ascii_lowercase();
        self.grants.iter().any(|g| g.grant == key)
    }

    pub fn upsert(&mut self, grant: &str, scheme: &str, hitl: bool) {
        let key = grant.to_ascii_lowercase();
        if let Some(existing) = self.grants.iter_mut().find(|g| g.grant == key) {
            existing.scheme = scheme.into();
            existing.hitl = hitl;
            existing.issued_at = now_secs();
        } else {
            self.grants.push(CapToken {
                grant: key,
                scheme: scheme.into(),
                issued_at: now_secs(),
                hitl,
            });
        }
        self.source = "scheme".into();
    }

    pub fn revoke(&mut self, grant: &str) -> bool {
        let key = grant.to_ascii_lowercase();
        let before = self.grants.len();
        self.grants.retain(|g| g.grant != key);
        self.source = "scheme".into();
        before != self.grants.len()
    }

    pub fn format(&self) -> String {
        let mut out = format!(
            "=== OS CAPS ===\nsource={}\nversion={}\ncount={}\n",
            self.source,
            self.version,
            self.grants.len()
        );
        for g in &self.grants {
            out.push_str(&format!(
                "  {} → {} hitl={} issued={}\n",
                g.grant, g.scheme, g.hitl, g.issued_at
            ));
        }
        out.push_str("=== END OS CAPS ===");
        out
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn caps_dir() -> PathBuf {
    std::env::var("REDOX_AIOS_CAPS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| aios_root().join("caps"))
}

pub fn grants_path() -> PathBuf {
    caps_dir().join(GRANTS_FILE)
}

fn ensure_caps_dir() -> Result<(), String> {
    fs::create_dir_all(caps_dir()).map_err(|e| e.to_string())
}

pub fn scheme_to_grant(scheme: &str) -> Option<&'static str> {
    match scheme.trim_end_matches(':') {
        "aios" => Some(GRANT_FACTORY),
        "net" => Some("net_fetch"),
        "file" | "fs" => Some("fs_read"),
        "memory" => Some("memory_recall"),
        "pkg" => Some(GRANT_PKG_INSTALL),
        "hitl" => Some(GRANT_HITL),
        _ => None,
    }
}

pub fn grant_to_scheme(grant: &str) -> &'static str {
    match grant.to_ascii_lowercase().as_str() {
        "factory_exec" => SCHEME_AIOS,
        "net_fetch" => SCHEME_NET,
        "fs_read" => SCHEME_FS,
        "memory_recall" => SCHEME_MEMORY,
        "pkg_install" | "ota_apply" => SCHEME_AIOS,
        "hitl_approve" => SCHEME_AIOS,
        _ => SCHEME_AIOS,
    }
}

/// Carrega store: arquivo scheme → env CSV → default.
pub fn load_cap_store() -> CapStore {
    if let Ok(store) = load_from_file(&grants_path()) {
        return store;
    }
    if let Ok(csv) = std::env::var("REDOX_AIOS_CAPS") {
        let mut store = CapStore {
            version: 1,
            source: "env".into(),
            grants: Vec::new(),
        };
        for part in csv.split(',') {
            let g = part.trim().to_ascii_lowercase();
            if g.is_empty() {
                continue;
            }
            let scheme = grant_to_scheme(&g);
            store.upsert(&g, scheme, g == GRANT_HITL || g == GRANT_PKG_INSTALL);
        }
        if store.grants.is_empty() {
            return CapStore::default();
        }
        return store;
    }
    CapStore::default()
}

fn load_from_file(path: &Path) -> Result<CapStore, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut store: CapStore = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    store.source = "scheme".into();
    Ok(store)
}

pub fn save_cap_store(store: &CapStore) -> Result<PathBuf, String> {
    ensure_caps_dir()?;
    let path = grants_path();
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    // Espelha CSV no env do processo (filhos herdam se spawnados daqui).
    std::env::set_var("REDOX_AIOS_CAPS", store.grant_names().join(","));
    Ok(path)
}

/// Bootstrap: se não houver arquivo, materializa a partir de env/default.
pub fn bootstrap_caps() -> Result<CapStore, String> {
    let path = grants_path();
    if path.is_file() {
        return load_from_file(&path);
    }
    let store = load_cap_store();
    save_cap_store(&store)?;
    Ok(store)
}

static CACHE: Mutex<Option<CapStore>> = Mutex::new(None);

pub fn refresh_caps_cache() -> CapStore {
    let store = load_cap_store();
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(store.clone());
    }
    store
}

pub fn cached_grants() -> Vec<String> {
    if let Ok(guard) = CACHE.lock() {
        if let Some(ref store) = *guard {
            return store.grant_names();
        }
    }
    refresh_caps_cache().grant_names()
}

pub fn grant_active_os(name: &str) -> bool {
    let key = name.to_ascii_lowercase();
    cached_grants().iter().any(|g| g == &key)
        || std::env::var("REDOX_AIOS_CAPS")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_ascii_lowercase())
                    .any(|s| s == key)
            })
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_save_load() {
        let root = std::env::temp_dir().join(format!(
            "redox-aios-caps-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("mkdir");
        std::env::set_var("REDOX_AIOS_CAPS_ROOT", &root);
        std::env::remove_var("REDOX_AIOS_CAPS");

        let mut store = CapStore::default();
        store.upsert(GRANT_HITL, SCHEME_AIOS, true);
        let saved = save_cap_store(&store).expect("save");
        assert!(saved.is_file(), "grants.json missing at {}", saved.display());
        let loaded = load_from_file(&saved).expect("load");
        assert!(loaded.has(GRANT_HITL));
        assert!(loaded.has(GRANT_FACTORY));

        std::env::remove_var("REDOX_AIOS_CAPS_ROOT");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn grant_scheme_mapping() {
        assert_eq!(grant_to_scheme("net_fetch"), SCHEME_NET);
        assert_eq!(scheme_to_grant("memory:"), Some("memory_recall"));
    }
}
