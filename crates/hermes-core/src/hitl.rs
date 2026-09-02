//! HITL gate — delega classificação a agent-core::permission_gate (ADR-001).

pub use agent_core::permission_gate::{gate_enabled as hitl_enabled, impact_level, ImpactLevel};

const DESTRUCTIVE_PATTERNS: &[&str] = &[
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
];

pub fn is_destructive(text: &str) -> bool {
    impact_level(text) == ImpactLevel::Critical
        || DESTRUCTIVE_PATTERNS
            .iter()
            .any(|p| text.to_ascii_lowercase().contains(&p.to_ascii_lowercase()))
}

pub fn gate_response(text: &str) -> Option<String> {
    if !hitl_enabled() {
        return None;
    }
    if let Some(msg) = agent_core::missing_scheme_grant(text) {
        return Some(msg);
    }
    match impact_level(text) {
        ImpactLevel::Critical => Some(i18n_core::t("hermes.hitl.blocked")),
        ImpactLevel::High => Some(i18n_core::t("hermes.hitl.high_impact")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_rm_rf() {
        assert!(is_destructive("please rm -rf / now"));
    }

    #[test]
    fn blocks_apt_install() {
        let prev = std::env::var("REDOX_HERMES_HITL").ok();
        std::env::set_var("REDOX_HERMES_HITL", "1");
        assert_eq!(gate_response("apt install foo").is_some(), true);
        match prev {
            Some(v) => std::env::set_var("REDOX_HERMES_HITL", v),
            None => std::env::remove_var("REDOX_HERMES_HITL"),
        }
    }

    #[test]
    fn allows_benign() {
        assert!(!is_destructive("que horas são"));
        assert!(gate_response("que horas são").is_none());
    }
}
