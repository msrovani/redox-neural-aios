//! Bridge scheme `chan:` — persiste eventos em arquivos (Fase 1).
//!
//! Cada publish grava `{REDOX_CHAN_ROOT}/topics/{topic}/{ts}.json`.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_CHAN_ROOT: &str = "/scheme/chan";

pub fn chan_root() -> PathBuf {
    std::env::var("REDOX_CHAN_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CHAN_ROOT))
}

pub fn bridge_enabled() -> bool {
    std::env::var("REDOX_CHAN_BRIDGE")
        .map(|v| v != "0")
        .unwrap_or(true)
}

pub fn publish_file(topic: &str, payload: &str) -> Result<(), String> {
    if !bridge_enabled() {
        return Ok(());
    }

    let safe_topic = topic
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect::<String>();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let dir = chan_root().join("topics").join(&safe_topic);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "topic": topic,
        "payload": payload,
        "ts": ts,
    });
    let path = dir.join(format!("{ts}.json"));
    fs::write(&path, serde_json::to_string(&body).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_file_creates_topic_dir() {
        let tmp = std::env::temp_dir().join("redox-chan-test");
        std::env::set_var("REDOX_CHAN_ROOT", tmp.to_string_lossy().as_ref());
        std::env::set_var("REDOX_CHAN_BRIDGE", "1");
        publish_file("BOOT_PHASE", "MemoryCore").expect("publish");
        assert!(tmp.join("topics").join("BOOT_PHASE").exists());
        let _ = fs::remove_dir_all(&tmp);
        std::env::remove_var("REDOX_CHAN_ROOT");
        std::env::remove_var("REDOX_CHAN_BRIDGE");
    }
}
