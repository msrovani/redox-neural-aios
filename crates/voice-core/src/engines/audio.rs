//! Reprodução de áudio WAV (host dev ou scheme `audio:`).

use std::path::Path;
use std::process::Command;

use super::scheme_audio;

pub fn play_wav(path: &Path) -> Result<(), String> {
    if scheme_audio::scheme_enabled() {
        return scheme_audio::play_wav_scheme(path);
    }

    crate::barge_in::set_speaking(true);
    let result = play_wav_host(path);
    crate::barge_in::set_speaking(false);
    result
}

fn play_wav_host(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("wav não encontrado: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "(New-Object System.Media.SoundPlayer '{}').PlaySync()",
            path.display()
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status()
            .map_err(|e| format!("play wav: {e}"))?;
        if status.success() {
            return Ok(());
        }
    }

    for player in ["ffplay", "aplay", "paplay"] {
        if Command::new(player)
            .arg(path)
            .args(if player == "ffplay" {
                vec!["-nodisp", "-autoexit"]
            } else {
                vec![]
            })
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(());
        }
    }

    Err(format!(
        "nenhum player disponível para {}",
        path.display()
    ))
}
