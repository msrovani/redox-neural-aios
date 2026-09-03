//! OTA skeleton — auto-upgrade com gate HITL (ADR-001 mand. 2).
//! Userspace: propõe atualização; não aplica sem aprovação explícita.

use crate::permission_gate::{gate_enabled, ImpactLevel};
use crate::scheme_caps::{grant_active, GRANT_HITL};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtaChannel {
    Stable,
    Nightly,
    Local,
}

impl OtaChannel {
    pub fn from_env() -> Self {
        match std::env::var("REDOX_OTA_CHANNEL")
            .unwrap_or_else(|_| "stable".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "nightly" => Self::Nightly,
            "local" | "dev" => Self::Local,
            _ => Self::Stable,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Nightly => "nightly",
            Self::Local => "local",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtaProposal {
    pub channel: String,
    pub current: String,
    pub candidate: String,
    pub impact: ImpactLevel,
    pub requires_hitl: bool,
    pub detail: String,
}

impl OtaProposal {
    pub fn format(&self) -> String {
        format!(
            "=== OTA PROPOSAL ===\nchannel={}\ncurrent={}\ncandidate={}\nimpact={:?}\nhitl={}\ndetail={}\n=== END OTA ===",
            self.channel,
            self.current,
            self.candidate,
            self.impact,
            self.requires_hitl,
            self.detail
        )
    }
}

/// Verifica se há update candidato (skeleton — compara versão env).
pub fn check_update(current_version: &str) -> OtaProposal {
    let channel = OtaChannel::from_env();
    let candidate = std::env::var("REDOX_OTA_CANDIDATE")
        .unwrap_or_else(|_| format!("{current_version}+proposed"));
    let available = candidate != current_version
        && std::env::var("REDOX_OTA_AVAILABLE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    let detail = if available {
        format!(
            "update disponível em canal {} — aguarda HITL (/ota approve)",
            channel.as_str()
        )
    } else {
        format!(
            "sem update (canal {}); defina REDOX_OTA_AVAILABLE=1 para simular",
            channel.as_str()
        )
    };

    OtaProposal {
        channel: channel.as_str().into(),
        current: current_version.into(),
        candidate,
        impact: ImpactLevel::High,
        requires_hitl: gate_enabled(),
        detail,
    }
}

/// Aplica OTA só com HITL + grant — skeleton nunca escreve pacotes reais.
pub fn apply_update(proposal: &OtaProposal, approved: bool) -> Result<String, String> {
    if proposal.requires_hitl && !approved {
        return Err("OTA bloqueado: HITL não aprovado".into());
    }
    if !grant_active(GRANT_HITL) && !grant_active("ota_apply") {
        return Err("OTA bloqueado: grant hitl_approve ou ota_apply ausente (REDOX_AIOS_CAPS)".into());
    }
    Ok(format!(
        "OTA staged (skeleton): {} → {} canal={} — cookbook/pkg install pendente",
        proposal.current, proposal.candidate, proposal.channel
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_without_flag_is_noop() {
        std::env::remove_var("REDOX_OTA_AVAILABLE");
        let p = check_update("0.0.1");
        assert!(p.detail.contains("sem update"));
    }

    #[test]
    fn apply_requires_grant() {
        let prev = std::env::var("REDOX_AIOS_CAPS").ok();
        std::env::set_var("REDOX_AIOS_CAPS", "factory_exec");
        let p = check_update("0.0.1");
        assert!(apply_update(&p, true).is_err());
        match prev {
            Some(v) => std::env::set_var("REDOX_AIOS_CAPS", v),
            None => std::env::remove_var("REDOX_AIOS_CAPS"),
        }
    }
}
