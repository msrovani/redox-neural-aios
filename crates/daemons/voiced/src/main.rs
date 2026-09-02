//! voiced — pipeline Jarvis E2E (whisper STT + piper TTS + Falcon3).

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use event_bus::emit_boot_ai;
use voice_core::{SttKind, TtsKind, VoicePipeline, DEFAULT_VOICE_SOCKET};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/voiced.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

fn stt_label(kind: SttKind) -> &'static str {
    match kind {
        SttKind::Stub => "stub",
        SttKind::Whisper => "whisper.cpp",
    }
}

fn tts_label(kind: TtsKind) -> &'static str {
    match kind {
        TtsKind::Stub => "stub",
        TtsKind::Piper => "piper",
    }
}

pub fn handle_request(pipeline: &VoicePipeline, line: &str) -> String {
    let cmd: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({"ok":false,"error":e.to_string()}).to_string(),
    };

    match cmd.get("cmd").and_then(|c| c.as_str()) {
        Some("ping") => serde_json::json!({"ok":true,"result":"pong"}).to_string(),
        Some("status") => {
            let stt_tier = match pipeline.stt_kind() {
                SttKind::Stub => "stub",
                SttKind::Whisper => "production",
            };
            let tts_tier = match pipeline.tts_kind() {
                TtsKind::Stub => "stub",
                TtsKind::Piper => "production",
            };
            serde_json::json!({
                "ok": true,
                "result": {
                    "stt": stt_label(pipeline.stt_kind()),
                    "stt_tier": stt_tier,
                    "tts": tts_label(pipeline.tts_kind()),
                    "tts_tier": tts_tier,
                    "degraded": matches!(pipeline.stt_kind(), SttKind::Stub) || matches!(pipeline.tts_kind(), TtsKind::Stub),
                    "wake_word": pipeline.wake.word,
                    "require_wake": pipeline.require_wake,
                    "auto_play": pipeline.auto_play,
                    "hermes": voice_core::DEFAULT_HERMES_SOCKET,
                }
            })
            .to_string()
        }
        Some("transcribe") => {
            let wav = cmd
                .get("wav")
                .or_else(|| cmd.get("path"))
                .and_then(|p| p.as_str())
                .unwrap_or("");
            match pipeline.transcribe_wav(PathBuf::from(wav).as_path()) {
                Ok(text) => serde_json::json!({"ok":true,"result":{"transcript":text}}).to_string(),
                Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
            }
        }
        Some("utterance") => {
            let text = cmd.get("text").and_then(|t| t.as_str()).unwrap_or("");
            match pipeline.process_utterance(text) {
                Ok(result) => serde_json::json!({
                    "ok": true,
                    "result": {
                        "transcript": result.transcript,
                        "response": result.response,
                        "tts": result.tts,
                        "tts_wav": result.tts_wav,
                    }
                })
                .to_string(),
                Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
            }
        }
        Some("listen") => {
            if cmd.get("scheme").and_then(|s| s.as_bool()).unwrap_or(false) {
                let dest = cmd
                    .get("dest")
                    .and_then(|d| d.as_str())
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("/tmp/scheme_capture.wav"));
                match pipeline.listen_scheme(dest.as_path()) {
                    Ok(result) => serde_json::json!({
                        "ok": true,
                        "result": {
                            "transcript": result.transcript,
                            "response": result.response,
                            "tts": result.tts,
                            "tts_wav": result.tts_wav,
                        }
                    })
                    .to_string(),
                    Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
                }
            } else if let Some(wav) = cmd.get("wav").and_then(|w| w.as_str()) {
                match pipeline.transcribe_wav(PathBuf::from(wav).as_path()) {
                    Ok(transcript) => match pipeline.process_transcript(&transcript) {
                        Ok(result) => serde_json::json!({
                            "ok": true,
                            "result": {
                                "transcript": result.transcript,
                                "response": result.response,
                                "tts": result.tts,
                                "tts_wav": result.tts_wav,
                            }
                        })
                        .to_string(),
                        Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
                    },
                    Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
                }
            } else {
                let text = cmd.get("text").and_then(|t| t.as_str()).unwrap_or("");
                handle_request(pipeline, &serde_json::json!({"cmd":"utterance","text":text}).to_string())
            }
        }
        Some("say") => {
            let text = cmd.get("text").and_then(|t| t.as_str()).unwrap_or("");
            match pipeline.say(text) {
                Ok(result) => serde_json::json!({
                    "ok":true,
                    "result":{"tts":result.tts,"tts_wav":result.tts_wav}
                })
                .to_string(),
                Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
            }
        }
        other => serde_json::json!({"ok":false,"error":format!("cmd invalido: {other:?}")})
            .to_string(),
    }
}

fn handle_client(pipeline: Arc<VoicePipeline>, stream: TcpStream) {
    let reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_request(&pipeline, &line);
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
}

fn main() {
    log_line(&format!("voiced v{VERSION} — Redox AIOS Jarvis pipeline"));

    let pipeline = Arc::new(VoicePipeline::from_env());
    let bind = std::env::var("REDOX_VOICE_SOCKET")
        .unwrap_or_else(|_| DEFAULT_VOICE_SOCKET.to_string());

    let listener = TcpListener::bind(&bind).unwrap_or_else(|e| {
        log_line(&format!("[voiced] FATAL bind {bind}: {e}"));
        std::process::exit(1);
    });

    let _ = fs::write("/tmp/voiced.pid", std::process::id().to_string());
    log_line(&format!(
        "[voiced] socket={bind} stt={} tts={} wake='{}' degraded={}",
        stt_label(pipeline.stt_kind()),
        tts_label(pipeline.tts_kind()),
        pipeline.wake.word,
        matches!(pipeline.stt_kind(), SttKind::Stub) || matches!(pipeline.tts_kind(), TtsKind::Stub)
    ));
    emit_boot_ai("voiced");
    log_line("[voiced] cmds: listen | transcribe | utterance | say | status | ping");

    for stream in listener.incoming().flatten() {
        handle_client(pipeline.clone(), stream);
    }
}
