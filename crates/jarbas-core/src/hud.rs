//! HUD terminal — chat + Soul Mirror ASCII.

use i18n_core::t;
use crate::orb::SoulMirror;
use crate::session::{ChatRole, ChatSession};
use crate::soul::SoulConfig;

pub struct HudFrame<'a> {
    pub soul: &'a SoulConfig,
    pub mirror: &'a SoulMirror,
    pub session: &'a ChatSession,
    pub status_line: &'a str,
}

pub fn render_chat_hud(frame: &HudFrame<'_>) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════╗\n");
    out.push_str(&format!(
        "║  {} — {:<22} │  LLM: {:<18} ║\n",
        frame.soul.name,
        t("jarbas.hud.title"),
        frame.soul.llm
    ));
    out.push_str("╠══════════════════════════════════════════════════════════╣\n");
    out.push_str(&frame.mirror.ascii_orb());
    out.push_str("╠══════════════════════════════════════════════════════════╣\n");

    if frame.session.history().is_empty() {
        out.push_str(&format!("║  {:<56} ║\n", t("jarbas.hud.empty")));
    } else {
        for msg in frame.session.history().iter().rev().take(6).rev() {
            let prefix = match msg.role {
                ChatRole::User => "Você",
                ChatRole::Assistant => &frame.soul.name,
                ChatRole::System => "SYS",
            };
            for line in wrap_lines(&format!("{prefix}: {}", msg.content), 56) {
                out.push_str(&format!("║  {line:<56} ║\n"));
            }
        }
    }

    out.push_str("╠══════════════════════════════════════════════════════════╣\n");
    out.push_str(&format!("║  {:<56} ║\n", frame.status_line));
    out.push_str("╚══════════════════════════════════════════════════════════╝\n");
    out
}

fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::ChatSession;

    #[test]
    fn hud_renders_name() {
        let soul = SoulConfig::default();
        let mirror = SoulMirror::default();
        let mut session = ChatSession::new(10);
        session.push(ChatRole::User, "oi");
        let hud = render_chat_hud(&HudFrame {
            soul: &soul,
            mirror: &mirror,
            session: &session,
            status_line: "online",
        });
        assert!(hud.contains("JARBAS"));
        assert!(hud.contains("Você: oi"));
    }
}
