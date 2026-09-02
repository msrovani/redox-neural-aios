//! EventBus in-process (Fase 0). Fase 1: bridge para scheme `chan:`.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::event::Event;

pub struct Receiver {
    queue: Arc<Mutex<VecDeque<Event>>>,
}

impl Receiver {
    pub fn try_receive(&self) -> Option<Event> {
        self.queue.lock().ok()?.pop_front()
    }

    pub fn has_pending(&self) -> bool {
        self.queue
            .lock()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }
}

#[derive(Default)]
pub struct EventBus {
    subscribers: Mutex<HashMap<String, Vec<Arc<Mutex<VecDeque<Event>>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, topic: &str) -> Receiver {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut subs = self.subscribers.lock().expect("event-bus lock");
        subs.entry(topic.to_string())
            .or_default()
            .push(queue.clone());
        Receiver { queue }
    }

    pub fn publish(&self, event: Event) -> Result<(), &'static str> {
        if !event.token.is_valid() {
            return Err("token de capacidade invalido");
        }
        let subs = self.subscribers.lock().expect("event-bus lock");
        if let Some(queues) = subs.get(&event.topic) {
            for q in queues {
                if let Ok(mut guard) = q.lock() {
                    guard.push_back(event.clone());
                }
            }
        }
        Ok(())
    }

    pub fn topic_count(&self) -> usize {
        self.subscribers
            .lock()
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::CapabilityToken;

    #[test]
    fn publish_subscribe_roundtrip() {
        let bus = EventBus::new();
        let rx = bus.subscribe("BOOT_PHASE");
        let token = CapabilityToken::system("eventd", "publish");
        bus.publish(Event::new("BOOT_PHASE", "MemoryCore", token))
            .unwrap();
        let ev = rx.try_receive().expect("event");
        assert_eq!(ev.topic, "BOOT_PHASE");
        assert_eq!(ev.payload, "MemoryCore");
    }
}
