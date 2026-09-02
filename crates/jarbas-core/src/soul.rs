//! Configuração de persona JARBAS (soul.toml).

use std::fs;

#[derive(Clone, Debug)]
pub struct SoulConfig {
    pub name: String,
    pub tone: String,
    pub humor_level: f32,
    pub formality: f32,
    pub empathy: f32,
    pub wake_word: String,
    pub language: String,
    pub llm: String,
}

impl Default for SoulConfig {
    fn default() -> Self {
        Self {
            name: "JARBAS".into(),
            tone: "witty".into(),
            humor_level: 0.5,
            formality: 0.3,
            empathy: 0.8,
            wake_word: "jarbas".into(),
            language: "pt-BR".into(),
            llm: "falcon3-3b-instruct".into(),
        }
    }
}

impl SoulConfig {
    pub fn load() -> Self {
        if let Ok(path) = std::env::var("REDOX_SOUL_PATH") {
            if let Ok(cfg) = Self::from_file(&path) {
                return cfg;
            }
        }
        for path in ["/etc/jarbas/soul.toml", "config/soul.toml"] {
            if let Ok(cfg) = Self::from_file(path) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let text = fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())?;
        Ok(Self::parse_toml_like(&text))
    }

    fn parse_toml_like(text: &str) -> Self {
        let mut cfg = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            match key {
                "name" => cfg.name = val.to_string(),
                "tone" => cfg.tone = val.to_string(),
                "humor_level" => cfg.humor_level = val.parse().unwrap_or(cfg.humor_level),
                "formality" => cfg.formality = val.parse().unwrap_or(cfg.formality),
                "empathy" => cfg.empathy = val.parse().unwrap_or(cfg.empathy),
                "wake_word" => cfg.wake_word = val.to_string(),
                "language" => cfg.language = val.to_string(),
                "llm" => cfg.llm = val.to_string(),
                _ => {}
            }
        }
        cfg
    }

    pub fn greeting_prompt(&self) -> String {
        format!(
            "Você é {}, assistente do Redox Neural AIOS. Tom: {}. \
             Dê uma saudação curta de boot (2 frases) em {} confirmando que sistemas estão online.",
            self.name, self.tone, self.language
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_soul_fields() {
        let text = r#"
name = "JARBAS"
tone = "witty"
language = "pt-BR"
"#;
        let cfg = SoulConfig::parse_toml_like(text);
        assert_eq!(cfg.name, "JARBAS");
        assert_eq!(cfg.language, "pt-BR");
    }
}
