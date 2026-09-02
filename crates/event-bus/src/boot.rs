//! Boot handshake — BOOT_AI nos tópicos EventBus + scheme `chan:`.

use crate::{chan, EventClient};

pub fn emit_boot_ai(agent: &str) {
    let payload = format!("{agent}_ready");
    let client = EventClient::new();
    if let Err(e) = client.publish("BOOT_AI", &payload) {
        eprintln!("[boot] BOOT_AI {payload} (eventd offline): {e}");
    }
    if let Err(e) = chan::publish_file("BOOT_AI", &payload) {
        eprintln!("[boot] chan BOOT_AI {payload}: {e}");
    }
}
