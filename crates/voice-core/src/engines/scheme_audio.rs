//! Bridge scheme `audio:` — MIC/SPK via arquivos (Fase 4, prep Redox nativo).

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_AUDIO_ROOT: &str = "/scheme/audio";
const PLAY_TIMEOUT: Duration = Duration::from_secs(30);

pub fn audio_root() -> PathBuf {
    std::env::var("REDOX_AUDIO_SCHEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_AUDIO_ROOT))
}

pub fn scheme_enabled() -> bool {
    std::env::var("REDOX_AUDIO_BACKEND")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "scheme" | "audio"))
        .unwrap_or(false)
}

pub fn ensure_dirs() -> Result<PathBuf, String> {
    let root = audio_root();
    fs::create_dir_all(root.join("mic")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("spk")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("vad")).map_err(|e| e.to_string())?;
    Ok(root)
}

/// Solicita playback via scheme — copia WAV para `spk/in/{id}.wav`.
pub fn play_wav_scheme(path: &Path) -> Result<(), String> {
    let root = ensure_dirs()?;
    if !path.is_file() {
        return Err(format!("wav não encontrado: {}", path.display()));
    }

    let id = format!(
        "{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let dest = root.join("spk").join("in").join(format!("{id}.wav"));
    fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::copy(path, &dest).map_err(|e| e.to_string())?;

    let done = root.join("spk").join("out").join(format!("{id}.done"));
    let deadline = Instant::now() + PLAY_TIMEOUT;
    while Instant::now() < deadline {
        if crate::barge_in::vad_active() {
            let _ = fs::write(root.join("spk").join("cancel"), "");
            return Err(i18n_core::t("voice.barge_in.during_tts"));
        }
        if done.exists() {
            let _ = fs::remove_file(&done);
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!("scheme audio: timeout playback {id}"))
}

/// Captura último WAV disponível em `mic/out/latest.wav`.
pub fn capture_wav_scheme(dest: &Path) -> Result<(), String> {
    let root = ensure_dirs()?;
    let src = root.join("mic").join("out").join("latest.wav");
    if !src.is_file() {
        return Err(format!(
            "scheme audio: sem captura em {} (MIC ativo?)",
            src.display()
        ));
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&src, dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_disabled_by_default() {
        std::env::remove_var("REDOX_AUDIO_BACKEND");
        assert!(!scheme_enabled());
    }
}
