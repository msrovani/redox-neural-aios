//! Parser de intents — subset portado do neural-os-core Hermes.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Echo(String),
    Time,
    Status,
    Remember(String),
    Recall(String),
    Skills,
    Factory,
    Promote(String),
    OpIr(String),
    Chat(String),
    Unknown(String),
}

pub fn parse_command(line: &str) -> Command {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Command::Unknown(String::new());
    }

    if let Some(cmd) = trimmed.strip_prefix('/') {
        let mut parts = cmd.splitn(2, char::is_whitespace);
        let name = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("").trim().to_string();
        let name_lower = name.to_ascii_lowercase();

        return match name_lower.as_str() {
            "help" | "h" | "?" => Command::Help,
            "ask" | "chat" => Command::Chat(arg),
            "echo" => Command::Echo(arg),
            "time" | "hora" => Command::Time,
            "status" | "stats" | "mem" => Command::Status,
            "remember" => Command::Remember(arg),
            "recall" => Command::Recall(arg),
            "skills" | "show_skills" => Command::Skills,
            "factory" => Command::Factory,
            "promote" => Command::Promote(arg),
            "opir" | "op-ir" => Command::OpIr(arg),
            _ => Command::Unknown(trimmed.to_string()),
        };
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("que horas") || lower.contains("what time") {
        return Command::Time;
    }
    if lower.starts_with("lembre ") || lower.starts_with("remember ") {
        let text = trimmed
            .split_once(char::is_whitespace)
            .map(|(_, rest)| rest.trim().to_string())
            .unwrap_or_default();
        return Command::Remember(text);
    }
    if lower.starts_with("recall ") || lower.starts_with("busque ") {
        let text = trimmed
            .split_once(char::is_whitespace)
            .map(|(_, rest)| rest.trim().to_string())
            .unwrap_or_default();
        return Command::Recall(text);
    }

    Command::Chat(trimmed.to_string())
}

pub fn command_to_skill(cmd: &Command) -> Option<(&'static str, String)> {
    match cmd {
        Command::Echo(s) => Some(("echo", s.clone())),
        Command::Time => Some(("time", String::new())),
        Command::Status => Some(("status", String::new())),
        Command::Remember(s) => Some(("remember", s.clone())),
        Command::Recall(s) => Some(("recall", s.clone())),
        Command::Skills => Some(("skills", String::new())),
        Command::Help => Some(("help", String::new())),
        Command::Factory => Some(("factory", String::new())),
        Command::Promote(s) => Some(("promote", s.clone())),
        Command::OpIr(s) => Some(("opir", s.clone())),
        _ => None,
    }
}
