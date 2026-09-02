//! STT — whisper.cpp (primário) + passthrough texto (dev).

use std::path::{Path, PathBuf};
use std::process::Command;

pub enum SttKind {
    Stub,
    Whisper,
}

pub trait SttEngine: Send + Sync {
    fn kind(&self) -> SttKind;
    fn transcribe_text(&self, hint: &str) -> Result<String, String>;
    fn transcribe_wav(&self, wav_path: &Path) -> Result<String, String>;
}

pub struct StubSttEngine;

impl SttEngine for StubSttEngine {
    fn kind(&self) -> SttKind {
        SttKind::Stub
    }

    fn transcribe_text(&self, hint: &str) -> Result<String, String> {
        let trimmed = hint.trim();
        if trimmed.is_empty() {
            return Err("entrada vazia (stub STT)".into());
        }
        Ok(trimmed.to_string())
    }

    fn transcribe_wav(&self, _wav_path: &Path) -> Result<String, String> {
        Err("STT stub — defina REDOX_STT_ENGINE=whisper".into())
    }
}

pub struct WhisperCppEngine {
    pub cli: PathBuf,
    pub model: PathBuf,
    pub language: String,
}

impl WhisperCppEngine {
    pub fn from_env() -> Result<Self, String> {
        let cli = std::env::var("REDOX_WHISPER_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("whisper-cli"));
        let model = std::env::var("REDOX_WHISPER_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_whisper_model());
        if !model.is_file() {
            return Err(format!(
                "modelo whisper não encontrado: {} (rode tools/download-voice-models.ps1)",
                model.display()
            ));
        }
        let language = std::env::var("REDOX_WHISPER_LANG").unwrap_or_else(|_| "pt".into());
        Ok(Self { cli, model, language })
    }
}

fn default_whisper_model() -> PathBuf {
    if let Ok(home) = std::env::var("REDOX_AIOS_HOME") {
        return PathBuf::from(home).join("models/whisper/ggml-base.bin");
    }
    std::env::current_dir()
        .unwrap_or_default()
        .join("models/whisper/ggml-base.bin")
}

impl SttEngine for WhisperCppEngine {
    fn kind(&self) -> SttKind {
        SttKind::Whisper
    }

    fn transcribe_text(&self, hint: &str) -> Result<String, String> {
        let trimmed = hint.trim();
        let path = PathBuf::from(trimmed);
        if path.extension().is_some_and(|e| e == "wav") && path.is_file() {
            return self.transcribe_wav(&path);
        }
        StubSttEngine.transcribe_text(trimmed)
    }

    fn transcribe_wav(&self, wav_path: &Path) -> Result<String, String> {
        if !wav_path.is_file() {
            return Err(format!("wav não encontrado: {}", wav_path.display()));
        }
        let output = Command::new(&self.cli)
            .arg("-m")
            .arg(&self.model)
            .arg("-f")
            .arg(wav_path)
            .arg("-l")
            .arg(&self.language)
            .arg("--no-timestamps")
            .arg("-nt")
            .output()
            .map_err(|e| format!("whisper-cli spawn: {e}"))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("whisper falhou: {err}"));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err("whisper sem transcrição".into());
        }
        Ok(text)
    }
}

pub fn stt_from_env() -> Box<dyn SttEngine> {
    match std::env::var("REDOX_STT_ENGINE")
        .unwrap_or_else(|_| "auto".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "stub" => Box::new(StubSttEngine),
        "whisper" | "whisper.cpp" => match WhisperCppEngine::from_env() {
            Ok(e) => Box::new(e),
            Err(e) => {
                eprintln!("[voice] whisper indisponível, usando stub: {e}");
                Box::new(StubSttEngine)
            }
        },
        _ => match WhisperCppEngine::from_env() {
            Ok(e) => Box::new(e),
            Err(_) => Box::new(StubSttEngine),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_passthrough() {
        let stt = StubSttEngine;
        assert_eq!(stt.transcribe_text("ola").unwrap(), "ola");
    }
}
