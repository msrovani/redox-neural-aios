//! Soul Mirror — estado emocional simplificado (terminal / Onda A).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrbEmotion {
    Idle,
    Think,
    Speak,
    Alert,
    Dream,
}

impl OrbEmotion {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Think => "THINK",
            Self::Speak => "SPEAK",
            Self::Alert => "ALERT",
            Self::Dream => "DREAM",
        }
    }

    pub fn from_text(text: &str) -> Self {
        let lower = text.to_ascii_lowercase();
        if lower.contains("erro") || lower.contains("error") || lower.contains("falha") {
            return Self::Alert;
        }
        if lower.contains("?") || lower.contains("pens") {
            return Self::Think;
        }
        if lower.len() > 120 {
            return Self::Speak;
        }
        Self::Idle
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoulMirror {
    pub emotion: OrbEmotion,
    pub pulse: u8,
}

impl Default for SoulMirror {
    fn default() -> Self {
        Self {
            emotion: OrbEmotion::Idle,
            pulse: 50,
        }
    }
}

impl SoulMirror {
    pub fn set_from_response(&mut self, response: &str) {
        self.emotion = OrbEmotion::from_text(response);
        self.pulse = ((response.len() % 40) as u8).saturating_add(40);
    }

    pub fn ascii_orb(&self) -> String {
        let (ring, core) = match self.emotion {
            OrbEmotion::Idle => ("○ ○ ○", "◎"),
            OrbEmotion::Think => ("◌ ◌ ◌", "◉"),
            OrbEmotion::Speak => ("● ● ●", "◉"),
            OrbEmotion::Alert => ("! ! !", "◈"),
            OrbEmotion::Dream => ("~ ~ ~", "◇"),
        };
        format!(
            "    {ring}\n   {core} JARBAS\n    [{emo}] pulse={pulse}\n",
            ring = ring,
            core = core,
            emo = self.emotion.label(),
            pulse = self.pulse
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_on_error() {
        assert_eq!(
            OrbEmotion::from_text("erro ao conectar"),
            OrbEmotion::Alert
        );
    }
}
