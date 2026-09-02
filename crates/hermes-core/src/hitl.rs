//! HITL gate — bloqueia intents destrutivos (Fase 2 stub).

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
    let lower = text.to_ascii_lowercase();
    DESTRUCTIVE_PATTERNS
        .iter()
        .any(|p| lower.contains(&p.to_ascii_lowercase()))
}

pub fn hitl_enabled() -> bool {
    std::env::var("REDOX_HERMES_HITL")
        .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
        .unwrap_or(true)
}

pub fn gate_response(text: &str) -> Option<String> {
    if !hitl_enabled() || !is_destructive(text) {
        return None;
    }
    Some(
        "⛔ HITL: ação destrutiva bloqueada. Confirme explicitamente no terminal se realmente deseja prosseguir.".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_rm_rf() {
        assert!(is_destructive("please rm -rf / now"));
    }

    #[test]
    fn allows_benign() {
        assert!(!is_destructive("que horas são"));
    }
}
