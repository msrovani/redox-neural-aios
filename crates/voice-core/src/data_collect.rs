//! DataCollector — persiste pares intent/response no SGDB (Fase 6).

use memory_core::MemoryClient;

const VOICE_SCOPE: &str = "voice";

pub struct DataCollector {
    memory: MemoryClient,
    enabled: bool,
}

impl DataCollector {
    pub fn from_env() -> Self {
        let enabled = std::env::var("REDOX_VOICE_REMEMBER")
            .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
            .unwrap_or(true);
        Self {
            memory: MemoryClient::new(),
            enabled,
        }
    }

    pub fn remember_pair(&self, intent: &str, response: &str) {
        if !self.enabled || intent.trim().is_empty() {
            return;
        }
        let text = format!("Q:{intent} A:{response}");
        let _ = self.memory.remember(&text, Some(VOICE_SCOPE));
    }
}

impl Default for DataCollector {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_env_zero() {
        std::env::set_var("REDOX_VOICE_REMEMBER", "0");
        let dc = DataCollector::from_env();
        assert!(!dc.enabled);
        std::env::remove_var("REDOX_VOICE_REMEMBER");
    }
}
