//! Barge-in — interrompe TTS quando há atividade de voz (VAD scheme ou flag).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static SPEAKING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

const DEFAULT_AUDIO_ROOT: &str = "/scheme/audio";

fn vad_path() -> PathBuf {
    let root = std::env::var("REDOX_AUDIO_SCHEME_ROOT")
        .unwrap_or_else(|_| DEFAULT_AUDIO_ROOT.into());
    PathBuf::from(root).join("vad").join("active")
}

pub fn barge_in_enabled() -> bool {
    std::env::var("REDOX_BARGE_IN")
        .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
        .unwrap_or(true)
}

pub fn vad_active() -> bool {
    if !barge_in_enabled() {
        return false;
    }
    if CANCEL.load(Ordering::Relaxed) {
        return true;
    }
    vad_path().exists()
}

pub fn request_cancel() {
    CANCEL.store(true, Ordering::Relaxed);
    if let Ok(root) = std::env::var("REDOX_AUDIO_SCHEME_ROOT") {
        let _ = fs::write(PathBuf::from(root).join("vad").join("active"), "");
    }
}

pub fn clear_cancel() {
    CANCEL.store(false, Ordering::Relaxed);
    let _ = fs::remove_file(vad_path());
}

pub fn set_speaking(active: bool) {
    SPEAKING.store(active, Ordering::Relaxed);
    if !active {
        clear_cancel();
    }
}

pub fn is_speaking() -> bool {
    SPEAKING.load(Ordering::Relaxed)
}
