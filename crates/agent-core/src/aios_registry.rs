//! Scheme `aios:` file bridge — registro de agentes (Fase 1 host).

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const DEFAULT_AIOS_ROOT: &str = "/scheme/aios";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRegistration {
    pub name: String,
    pub kind: String,
    pub socket: Option<String>,
    pub trust_token: u32,
    pub auto_start: bool,
}

pub fn aios_root() -> PathBuf {
    std::env::var("REDOX_AIOS_SCHEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_AIOS_ROOT))
}

pub fn bridge_enabled() -> bool {
    std::env::var("REDOX_AIOS_SCHEME")
        .map(|v| v != "0")
        .unwrap_or(true)
}

pub fn register_agent(reg: &AgentRegistration) -> Result<PathBuf, String> {
    if !bridge_enabled() {
        return Ok(aios_root().join("agents").join(format!("{}.json", reg.name)));
    }

    let dir = aios_root().join("agents");
    fs::create_dir_all(&dir).map_err(|e| format!("criar {dir:?}: {e}"))?;
    let path = dir.join(format!("{}.json", reg.name));
    let json = serde_json::to_string_pretty(reg).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("gravar {path:?}: {e}"))?;
    Ok(path)
}

pub fn list_agents() -> Result<Vec<AgentRegistration>, String> {
    let dir = aios_root().join("agents");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(reg) = serde_json::from_str::<AgentRegistration>(&data) {
            out.push(reg);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn register_fleet(defaults: &[(&str, &str, Option<&str>)]) -> Result<usize, String> {
    let mut n = 0usize;
    for (name, kind, socket) in defaults {
        register_agent(&AgentRegistration {
            name: (*name).into(),
            kind: (*kind).into(),
            socket: socket.map(String::from),
            trust_token: 1,
            auto_start: true,
        })?;
        n += 1;
    }
    Ok(n)
}

pub fn touch_registry_marker() -> Result<(), String> {
    if !bridge_enabled() {
        return Ok(());
    }
    let marker = aios_root().join(".registry");
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    fs::create_dir_all(aios_root()).map_err(|e| e.to_string())?;
    fs::write(marker, format!("updated={ts}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn registry_path_for(name: &str) -> PathBuf {
    aios_root().join("agents").join(format!("{name}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_list_roundtrip() {
        let root = std::env::temp_dir().join("redox-aios-registry-test");
        let _ = fs::remove_dir_all(&root);
        std::env::set_var("REDOX_AIOS_SCHEME_ROOT", &root);

        register_agent(&AgentRegistration {
            name: "eventd".into(),
            kind: "system".into(),
            socket: Some("127.0.0.1:7740".into()),
            trust_token: 1,
            auto_start: true,
        })
        .expect("register");

        let agents = list_agents().expect("list");
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "eventd");

        std::env::remove_var("REDOX_AIOS_SCHEME_ROOT");
        let _ = fs::remove_dir_all(root);
    }
}
