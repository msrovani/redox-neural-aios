//! Backend honesty — reporta tiers degradados/stub (ADR-001 mandamento 4).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendTier {
    Production,
    Degraded,
    Stub,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackendReport {
    pub component: String,
    pub tier: BackendTier,
    pub detail: String,
}

pub fn probe_tcp(addr: &str, timeout_ms: u64) -> bool {
    let Ok(socket) = addr.parse::<SocketAddr>() else {
        return false;
    };
    TcpStream::connect_timeout(&socket, Duration::from_millis(timeout_ms)).is_ok()
}

pub fn probe_json_status(addr: &str, timeout_ms: u64) -> Option<String> {
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().ok()?,
        Duration::from_millis(timeout_ms),
    )
    .ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(timeout_ms))).ok()?;
    stream.set_write_timeout(Some(Duration::from_millis(timeout_ms))).ok()?;
    writeln!(stream, r#"{{"cmd":"status"}}"#).ok()?;
    stream.flush().ok()?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    Some(buf)
}

pub fn collect_stack_backends() -> Vec<BackendReport> {
    let mut reports = Vec::new();

    reports.push(memory_backend());
    reports.push(event_backend());
    reports.push(cortex_backend());
    reports.push(voice_stt_backend());
    reports.push(voice_tts_backend());
    reports.push(caps_backend());
    reports
}

fn caps_backend() -> BackendReport {
    let backend = crate::redox_caps::CapBackend::detect();
    let summary = crate::redox_caps_summary();
    let (tier, detail) = match backend {
        crate::redox_caps::CapBackend::NsmgrFd => (
            BackendTier::Production,
            format!("nsmgr FD ativo; {summary}"),
        ),
        crate::redox_caps::CapBackend::SchemeProbe => (
            BackendTier::Degraded,
            format!("scheme probe (prep nsmgr); {summary}"),
        ),
        crate::redox_caps::CapBackend::ProfileBridge => (
            BackendTier::Degraded,
            format!("profile bridge userspace; {summary}"),
        ),
    };
    BackendReport {
        component: "caps".into(),
        tier,
        detail,
    }
}

fn memory_backend() -> BackendReport {
    let native = std::env::var("REDOX_MEMORY_SCHEME_NATIVE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let backend = std::env::var("REDOX_MEMORY_BACKEND").unwrap_or_else(|_| {
        if native {
            "scheme".into()
        } else {
            "tcp".into()
        }
    });
    let label = memory_core::backend_label();
    let tier = if native {
        BackendTier::Degraded
    } else {
        match backend.to_ascii_lowercase().as_str() {
            "scheme" | "memory" => BackendTier::Degraded,
            "tcp" => BackendTier::Degraded,
            _ => BackendTier::Stub,
        }
    };
    BackendReport {
        component: "memory".into(),
        tier,
        detail: format!("backend={backend} mode={label} caps={}", crate::cap_summary()),
    }
}

fn event_backend() -> BackendReport {
    let bridge = std::env::var("REDOX_CHAN_BRIDGE")
        .map(|v| v != "0")
        .unwrap_or(true);
    BackendReport {
        component: "event".into(),
        tier: if bridge {
            BackendTier::Degraded
        } else {
            BackendTier::Stub
        },
        detail: if bridge {
            "chan file bridge".into()
        } else {
            "in-process only".into()
        },
    }
}

fn cortex_backend() -> BackendReport {
    let addr = std::env::var("REDOX_CORTEX_SOCKET")
        .unwrap_or_else(|_| "127.0.0.1:7743".into());
    if let Some(body) = probe_json_status(&addr, 300) {
        if body.contains("\"engine\":\"stub\"") || body.contains("stub") {
            return BackendReport {
                component: "cortex".into(),
                tier: BackendTier::Stub,
                detail: "cortexd responde; engine=stub".into(),
            };
        }
        if body.contains("falcon") {
            return BackendReport {
                component: "cortex".into(),
                tier: BackendTier::Production,
                detail: "cortexd responde; falcon3".into(),
            };
        }
    }

    let force_stub = std::env::var("REDOX_CORTEX_ENGINE")
        .map(|v| v.eq_ignore_ascii_case("stub"))
        .unwrap_or(false);
    BackendReport {
        component: "cortex".into(),
        tier: if force_stub {
            BackendTier::Stub
        } else {
            BackendTier::Degraded
        },
        detail: "Falcon3 indisponível ou cortexd offline".into(),
    }
}

fn voice_stt_backend() -> BackendReport {
    let engine = std::env::var("REDOX_STT_ENGINE").unwrap_or_else(|_| "stub".into());
    BackendReport {
        component: "stt".into(),
        tier: match engine.to_ascii_lowercase().as_str() {
            "whisper" => BackendTier::Production,
            "stub" => BackendTier::Stub,
            _ => BackendTier::Degraded,
        },
        detail: format!("engine={engine}"),
    }
}

fn voice_tts_backend() -> BackendReport {
    let engine = std::env::var("REDOX_TTS_ENGINE").unwrap_or_else(|_| "stub".into());
    BackendReport {
        component: "tts".into(),
        tier: match engine.to_ascii_lowercase().as_str() {
            "piper" => BackendTier::Production,
            "stub" => BackendTier::Stub,
            _ => BackendTier::Degraded,
        },
        detail: format!("engine={engine}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_includes_memory() {
        let reports = collect_stack_backends();
        assert!(reports.iter().any(|r| r.component == "memory"));
        assert!(reports.iter().any(|r| r.component == "caps"));
    }
}
