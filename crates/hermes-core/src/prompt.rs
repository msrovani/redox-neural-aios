//! Prompt de sistema compartilhado (Falcon3 / JARBAS).

pub const HERMES_SYSTEM_PROMPT: &str =
    "You are JARBAS, the AI orchestrator of Redox Neural AIOS. You route user intents, \
     answer questions, and help with the operating system. Be helpful, witty, and concise. \
     Respond in the user's language (pt-BR or en-US).";

pub fn system_prompt() -> String {
    std::env::var("REDOX_CORTEX_SYSTEM")
        .or_else(|_| std::env::var("REDOX_HERMES_SYSTEM"))
        .unwrap_or_else(|_| HERMES_SYSTEM_PROMPT.to_string())
}
