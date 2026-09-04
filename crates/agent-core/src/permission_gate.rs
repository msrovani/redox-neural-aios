//! permission_gate — classificação HITL por impacto (ADR-001 mandamento 2).
//! Integrado com `scheme_caps` — grants via `REDOX_AIOS_CAPS`.

use crate::scheme_caps::{grant_active, GRANT_FACTORY, GRANT_HITL, GRANT_PKG_INSTALL};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

const CRITICAL_PATTERNS: &[&str] = &[
    "delete /",
    "rm -rf",
    "rm -r /",
    "format c:",
    "format /",
    "dd if=",
    "mkfs.",
    "drop database",
    "apague tudo",
    "deletar tudo",
    "formate ",
    "wipe disk",
    "zerar disco",
];

const HIGH_PATTERNS: &[&str] = &[
    "apt install",
    "apt-get install",
    "yum install",
    "pacman -s",
    "cargo install",
    "pip install --system",
    "curl ",
    "| bash",
    "| sh",
    "wget ",
    "chmod 777",
    "chmod -r 777",
    "chown root",
    "system update",
    "ota update",
    "firmware flash",
    "disable firewall",
    "ufw disable",
    "iptables -f",
    "instalar pacote",
    "atualizar sistema",
    "desabilitar firewall",
];

const MEDIUM_PATTERNS: &[&str] = &[
    "change policy",
    "alterar politica",
    "mudar politica",
    "export secrets",
    "enviar senha",
    "disable hitl",
    "refox_hermes_hitl=0",
];

fn matches_any(lower: &str, patterns: &[&str]) -> bool {
    patterns
        .iter()
        .any(|p| lower.contains(&p.to_ascii_lowercase()))
}

pub fn impact_level(text: &str) -> ImpactLevel {
    let lower = text.to_ascii_lowercase();
    if matches_any(&lower, CRITICAL_PATTERNS) {
        return ImpactLevel::Critical;
    }
    if matches_any(&lower, HIGH_PATTERNS) {
        return ImpactLevel::High;
    }
    if matches_any(&lower, MEDIUM_PATTERNS) {
        return ImpactLevel::Medium;
    }
    ImpactLevel::Low
}

pub fn requires_hitl(level: ImpactLevel) -> bool {
    matches!(level, ImpactLevel::Critical | ImpactLevel::High)
}

pub fn gate_enabled() -> bool {
    std::env::var("REDOX_HERMES_HITL")
        .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
        .unwrap_or(true)
}

/// Grant scheme exigido para o nível de impacto (CapGate Fase 2).
pub fn required_grant(level: ImpactLevel) -> Option<&'static str> {
    match level {
        ImpactLevel::Critical | ImpactLevel::High => Some(GRANT_HITL),
        ImpactLevel::Medium => Some(GRANT_FACTORY),
        ImpactLevel::Low => None,
    }
}

/// Retorna mensagem se a ação exige grant ausente em `REDOX_AIOS_CAPS`.
pub fn missing_scheme_grant(text: &str) -> Option<String> {
    if !gate_enabled() {
        return None;
    }
    let level = impact_level(text);
    if !requires_hitl(level) {
        return None;
    }
    let grant = required_grant(level)?;
    if grant_active(grant) {
        return None;
    }
    Some(format!("grant ausente: {grant} (REDOX_AIOS_CAPS)"))
}

/// Promoção de pacote exige grant explícito além de HITL textual.
pub fn missing_pkg_grant() -> Option<String> {
    if grant_active(GRANT_PKG_INSTALL) || grant_active(GRANT_HITL) {
        return None;
    }
    Some(format!("grant ausente: {GRANT_PKG_INSTALL} (REDOX_AIOS_CAPS)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_critical() {
        assert_eq!(impact_level("rm -rf /"), ImpactLevel::Critical);
    }

    #[test]
    fn classifies_high_install() {
        assert_eq!(impact_level("apt install malware"), ImpactLevel::High);
    }

    #[test]
    fn classifies_benign_low() {
        assert_eq!(impact_level("que horas são"), ImpactLevel::Low);
    }

    #[test]
    fn missing_grant_when_hitl_required() {
        let prev = std::env::var("REDOX_AIOS_CAPS").ok();
        std::env::set_var("REDOX_AIOS_CAPS", "factory_exec");
        assert!(missing_scheme_grant("apt install foo").is_some());
        match prev {
            Some(v) => std::env::set_var("REDOX_AIOS_CAPS", v),
            None => std::env::remove_var("REDOX_AIOS_CAPS"),
        }
    }
}
