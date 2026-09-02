//! Engines stub — desenvolvimento sem microfone/modelos reais.

pub struct StubWakeWord {
    pub word: String,
}

impl StubWakeWord {
    pub fn detect_in_text(&self, text: &str) -> bool {
        text.to_ascii_lowercase()
            .contains(&self.word.to_ascii_lowercase())
    }

    pub fn strip_prefix(&self, text: &str) -> String {
        let lower = text.to_ascii_lowercase();
        let wake = self.word.to_ascii_lowercase();
        if let Some(idx) = lower.find(&wake) {
            let after = idx + wake.len();
            text[after..]
                .trim()
                .trim_start_matches([',', '.', '!', '?'])
                .trim()
                .to_string()
        } else {
            text.trim().to_string()
        }
    }
}

pub struct StubStt;

impl StubStt {
    /// Simula transcrição — no host o texto já vem pronto (CLI/utterance).
    pub fn transcribe(&self, audio_hint: &str) -> Result<String, String> {
        let trimmed = audio_hint.trim();
        if trimmed.is_empty() {
            return Err("audio vazio (stub STT)".into());
        }
        Ok(trimmed.to_string())
    }
}

pub struct StubTts;

impl StubTts {
    /// Simula síntese — retorna marcador textual (PCM real na Fase 4+).
    pub fn synthesize(&self, text: &str) -> Result<String, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok("[TTS stub: silêncio]".into());
        }
        let preview: String = trimmed.chars().take(80).collect();
        Ok(format!(
            "[TTS stub len={} preview=\"{preview}\"]",
            trimmed.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wake_strip() {
        let wake = StubWakeWord {
            word: "jarbas".into(),
        };
        assert!(wake.detect_in_text("ei jarbas que horas"));
        assert_eq!(
            wake.strip_prefix("jarbas, que horas são"),
            "que horas são"
        );
    }

    #[test]
    fn tts_preview() {
        let tts = StubTts;
        let out = tts.synthesize("Olá mundo").unwrap();
        assert!(out.contains("TTS stub"));
    }
}
