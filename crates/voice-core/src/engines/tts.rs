//! TTS — piper (primário) + stub textual.

use std::path::PathBuf;
use std::process::Command;

pub enum TtsKind {
    Stub,
    Piper,
}

#[derive(Clone, Debug)]
pub struct TtsOutput {
    pub label: String,
    pub wav_path: Option<PathBuf>,
}

pub trait TtsEngine: Send + Sync {
    fn kind(&self) -> TtsKind;
    fn synthesize(&self, text: &str) -> Result<TtsOutput, String>;
}

pub struct StubTtsEngine;

impl TtsEngine for StubTtsEngine {
    fn kind(&self) -> TtsKind {
        TtsKind::Stub
    }

    fn synthesize(&self, text: &str) -> Result<TtsOutput, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(TtsOutput {
                label: "[TTS stub: silêncio]".into(),
                wav_path: None,
            });
        }
        let preview: String = trimmed.chars().take(80).collect();
        Ok(TtsOutput {
            label: format!("[TTS stub len={} preview=\"{preview}\"]", trimmed.len()),
            wav_path: None,
        })
    }
}

pub struct PiperEngine {
    pub cli: PathBuf,
    pub model: PathBuf,
    pub output_dir: PathBuf,
}

impl PiperEngine {
    pub fn from_env() -> Result<Self, String> {
        let cli = std::env::var("REDOX_PIPER_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("piper"));
        let model = std::env::var("REDOX_PIPER_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_piper_model());
        if !model.is_file() {
            return Err(format!(
                "modelo piper não encontrado: {} (rode tools/download-voice-models.ps1)",
                model.display()
            ));
        }
        let output_dir = std::env::var("REDOX_TTS_OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("redox-tts"));
        std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            cli,
            model,
            output_dir,
        })
    }
}

fn default_piper_model() -> PathBuf {
    if let Ok(home) = std::env::var("REDOX_AIOS_HOME") {
        return PathBuf::from(home).join("models/piper/pt_BR-faber-medium.onnx");
    }
    std::env::current_dir()
        .unwrap_or_default()
        .join("models/piper/pt_BR-faber-medium.onnx")
}

impl TtsEngine for PiperEngine {
    fn kind(&self) -> TtsKind {
        TtsKind::Piper
    }

    fn synthesize(&self, text: &str) -> Result<TtsOutput, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(TtsOutput {
                label: String::new(),
                wav_path: None,
            });
        }

        let out = self.output_dir.join(format!(
            "tts_{}.wav",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));

        let mut child = Command::new(&self.cli)
            .arg("--model")
            .arg(&self.model)
            .arg("--output_file")
            .arg(&out)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("piper spawn: {e}"))?;

        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(trimmed.as_bytes())
                .map_err(|e| e.to_string())?;
        }
        let result = child.wait_with_output().map_err(|e| e.to_string())?;
        if !result.status.success() {
            let err = String::from_utf8_lossy(&result.stderr);
            return Err(format!("piper falhou: {err}"));
        }
        if !out.is_file() {
            return Err("piper não gerou wav".into());
        }

        Ok(TtsOutput {
            label: format!("piper:{}", out.display()),
            wav_path: Some(out),
        })
    }
}

pub fn tts_from_env() -> Box<dyn TtsEngine> {
    match std::env::var("REDOX_TTS_ENGINE")
        .unwrap_or_else(|_| "auto".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "stub" => Box::new(StubTtsEngine),
        "piper" => match PiperEngine::from_env() {
            Ok(e) => Box::new(e),
            Err(e) => {
                eprintln!("[voice] piper indisponível, usando stub: {e}");
                Box::new(StubTtsEngine)
            }
        },
        _ => match PiperEngine::from_env() {
            Ok(e) => Box::new(e),
            Err(_) => Box::new(StubTtsEngine),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_tts() {
        let tts = StubTtsEngine;
        let out = tts.synthesize("oi").unwrap();
        assert!(out.label.contains("TTS stub"));
    }
}
