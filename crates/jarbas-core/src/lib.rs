//! Jarbas shell AI — sessão, soul, HUD terminal (Fase 5 / Onda A).

pub mod hermes_bridge;
pub mod hud;
pub mod orb;
pub mod session;
pub mod soul;

pub use hermes_bridge::JarbasBridge;
pub use hud::{render_chat_hud, HudFrame};
pub use orb::{OrbEmotion, SoulMirror};
pub use session::{ChatMessage, ChatRole, ChatSession};
pub use soul::SoulConfig;

pub const TOPIC_JARBAS_USER: &str = "JARBAS_CHAT_USER";
pub const TOPIC_JARBAS_ASSISTANT: &str = "JARBAS_CHAT_ASSISTANT";
pub const TOPIC_JARBAS_ORB: &str = "JARBAS_ORB_STATE";
pub const DEFAULT_JARBAS_SOCKET: &str = "127.0.0.1:7745";
