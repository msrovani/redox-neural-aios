//! Evento tipado publicado no EventBus.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct CapabilityToken {
    pub id: u32,
    pub agent: String,
    pub skill: String,
}

impl CapabilityToken {
    pub fn system(agent: &str, skill: &str) -> Self {
        Self {
            id: 1,
            agent: agent.to_string(),
            skill: skill.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && !self.agent.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: u64,
    pub topic: String,
    pub payload: String,
    pub token: CapabilityToken,
}

impl Event {
    pub fn new(topic: impl Into<String>, payload: impl Into<String>, token: CapabilityToken) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            topic: topic.into(),
            payload: payload.into(),
            token,
        }
    }
}
