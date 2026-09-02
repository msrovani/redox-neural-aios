//! Orquestração do pipeline Jarvis — STT/TTS reais + Hermes/Falcon3.

use std::path::Path;

use crate::engines::audio::play_wav;
use crate::engines::stt::{stt_from_env, SttKind};
use crate::engines::tts::{tts_from_env, TtsKind};
use crate::event_client::EventClient;
use crate::hermes_client::HermesClient;
use crate::stub::StubWakeWord;
use crate::{
    TOPIC_VOICE_STT, TOPIC_VOICE_TTS_END, TOPIC_VOICE_TTS_START, TOPIC_VOICE_WAKE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceResult {
    pub transcript: String,
    pub response: String,
    pub tts: String,
    pub tts_wav: Option<String>,
}

pub struct VoicePipeline {
    pub wake: StubWakeWord,
    pub stt: Box<dyn crate::engines::stt::SttEngine>,
    pub tts: Box<dyn crate::engines::tts::TtsEngine>,
    pub hermes: HermesClient,
    pub events: EventClient,
    pub require_wake: bool,
    pub auto_play: bool,
}

impl VoicePipeline {
    pub fn from_env() -> Self {
        let wake_word =
            std::env::var("REDOX_WAKE_WORD").unwrap_or_else(|_| "jarbas".to_string());
        let require_wake = std::env::var("REDOX_VOICE_REQUIRE_WAKE")
            .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
            .unwrap_or(false);
        let auto_play = std::env::var("REDOX_TTS_AUTO_PLAY")
            .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
            .unwrap_or(true);

        Self {
            wake: StubWakeWord { word: wake_word },
            stt: stt_from_env(),
            tts: tts_from_env(),
            hermes: HermesClient::new(),
            events: EventClient::new(),
            require_wake,
            auto_play,
        }
    }

    pub fn stt_kind(&self) -> SttKind {
        self.stt.kind()
    }

    pub fn tts_kind(&self) -> TtsKind {
        self.tts.kind()
    }

    pub fn normalize_input(&self, raw: &str) -> Result<String, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err("entrada vazia".into());
        }
        if self.require_wake && !self.wake.detect_in_text(trimmed) {
            return Err(format!(
                "wake word '{}' não detectada",
                self.wake.word
            ));
        }
        if self.wake.detect_in_text(trimmed) {
            let _ = self.events.publish(TOPIC_VOICE_WAKE, &self.wake.word);
            Ok(self.wake.strip_prefix(trimmed))
        } else {
            Ok(trimmed.to_string())
        }
    }

    pub fn transcribe_wav(&self, wav_path: &Path) -> Result<String, String> {
        let text = self.stt.transcribe_wav(wav_path)?;
        let _ = self.events.publish(TOPIC_VOICE_STT, &text);
        Ok(text)
    }

    pub fn process_utterance(&self, raw: &str) -> Result<VoiceResult, String> {
        let normalized = self.normalize_input(raw)?;
        let transcript = self.stt.transcribe_text(&normalized)?;
        let _ = self.events.publish(TOPIC_VOICE_STT, &transcript);

        let response = self.hermes.intent(&transcript)?;
        self.speak_response(&response, &transcript)
    }

    pub fn process_transcript(&self, transcript: &str) -> Result<VoiceResult, String> {
        let transcript = transcript.trim();
        if transcript.is_empty() {
            return Err("transcrição vazia".into());
        }
        if crate::barge_in::vad_active() {
            crate::barge_in::clear_cancel();
        }
        let _ = self.events.publish(TOPIC_VOICE_STT, transcript);
        let response = self.hermes.intent(transcript)?;
        self.speak_response(&response, transcript)
    }

    /// Captura WAV via scheme `audio:` e processa pipeline completo.
    pub fn listen_scheme(&self, dest_wav: &Path) -> Result<VoiceResult, String> {
        crate::engines::capture_wav_scheme(dest_wav)?;
        let transcript = self.transcribe_wav(dest_wav)?;
        self.process_transcript(&transcript)
    }

    fn speak_response(&self, response: &str, _transcript: &str) -> Result<VoiceResult, String> {
        if crate::barge_in::vad_active() {
            return Err("barge-in: fala do usuário detectada antes do TTS".into());
        }
        let _ = self.events.publish(TOPIC_VOICE_TTS_START, response);
        let tts_out = self.tts.synthesize(response)?;
        if self.auto_play {
            if let Some(ref wav) = tts_out.wav_path {
                match play_wav(wav) {
                    Ok(()) => {}
                    Err(e) if e.contains("barge-in") => {
                        let _ = self.events.publish(TOPIC_VOICE_TTS_END, "barge-in");
                        return Ok(VoiceResult {
                            transcript: _transcript.to_string(),
                            response: response.to_string(),
                            tts: "barge-in".into(),
                            tts_wav: tts_out.wav_path.map(|p| p.display().to_string()),
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        let _ = self.events.publish(TOPIC_VOICE_TTS_END, &tts_out.label);
        Ok(VoiceResult {
            transcript: _transcript.to_string(),
            response: response.to_string(),
            tts: tts_out.label,
            tts_wav: tts_out.wav_path.map(|p| p.display().to_string()),
        })
    }

    pub fn say(&self, text: &str) -> Result<VoiceResult, String> {
        let _ = self.events.publish(TOPIC_VOICE_TTS_START, text);
        let tts_out = self.tts.synthesize(text)?;
        if self.auto_play {
            if let Some(ref wav) = tts_out.wav_path {
                let _ = play_wav(wav);
            }
        }
        let _ = self.events.publish(TOPIC_VOICE_TTS_END, &tts_out.label);
        Ok(VoiceResult {
            transcript: String::new(),
            response: text.to_string(),
            tts: tts_out.label,
            tts_wav: tts_out.wav_path.map(|p| p.display().to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_without_wake_required() {
        let mut pipe = VoicePipeline::from_env();
        pipe.require_wake = false;
        assert_eq!(
            pipe.normalize_input("que horas são").unwrap(),
            "que horas são"
        );
    }
}
