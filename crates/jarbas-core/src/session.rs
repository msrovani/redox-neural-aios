//! Sessão de chat Jarbas.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct ChatSession {
    pub messages: Vec<ChatMessage>,
    pub max_messages: usize,
}

impl ChatSession {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
        }
    }

    pub fn push(&mut self, role: ChatRole, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role,
            content: content.into(),
        });
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn history(&self) -> &[ChatMessage] {
        &self.messages
    }
}
