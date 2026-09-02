//! Trait CortexEngine + stub de desenvolvimento.

use crate::inference::AdaptiveEngine;

pub trait CortexEngine: Send + Sync {
    fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String, String>;
}

/// Engine de desenvolvimento — quando modelo Falcon3 não está instalado.
pub struct StubEngine {
    pub persona: String,
}

impl Default for StubEngine {
    fn default() -> Self {
        Self {
            persona: "JARBAS".into(),
        }
    }
}

impl CortexEngine for StubEngine {
    fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String, String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Ok(format!("{persona}: estou ouvindo.", persona = self.persona));
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("ola") || lower.contains("olá") || lower.starts_with("oi") {
            return Ok(format!(
                "{persona}: Olá! Sou o assistente do Redox AIOS. \
                 Instale Falcon3 com tools/download-falcon3.ps1 para respostas reais.",
                persona = self.persona
            ));
        }

        if let Some(sys) = system.filter(|s| !s.trim().is_empty()) {
            return Ok(format!(
                "{persona}: [{sys}] Recebi: {trimmed} (stub — modelo não carregado)",
                persona = self.persona
            ));
        }

        Ok(format!(
            "{persona}: {trimmed} (stub — rode download-falcon3.ps1)",
            persona = self.persona
        ))
    }
}

pub fn engine_from_env() -> AdaptiveEngine {
    AdaptiveEngine::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_greets() {
        let engine = StubEngine::default();
        let out = engine.complete("ola Hermes", None).unwrap();
        assert!(out.contains("Olá") || out.contains("Falcon3"));
    }
}
