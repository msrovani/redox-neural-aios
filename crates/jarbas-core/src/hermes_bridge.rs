//! Ponte Jarbas → Hermes (Falcon3) + eventos.

use voice_core::{EventClient, HermesClient, VoicePipeline};

use crate::orb::SoulMirror;
use crate::session::{ChatRole, ChatSession};
use crate::soul::SoulConfig;
use crate::{TOPIC_JARBAS_ASSISTANT, TOPIC_JARBAS_ORB, TOPIC_JARBAS_USER};

pub struct JarbasBridge {
    pub soul: SoulConfig,
    pub hermes: HermesClient,
    pub events: EventClient,
    pub voice: Option<VoicePipeline>,
}

impl JarbasBridge {
    pub fn new() -> Self {
        let voice_enabled = std::env::var("REDOX_JARBAS_VOICE")
            .map(|v| v != "0" && v.to_ascii_lowercase() != "false")
            .unwrap_or(false);
        Self {
            soul: SoulConfig::load(),
            hermes: HermesClient::new(),
            events: EventClient::new(),
            voice: if voice_enabled {
                Some(VoicePipeline::from_env())
            } else {
                None
            },
        }
    }

    pub fn chat(&self, text: &str, session: &mut ChatSession, mirror: &mut SoulMirror) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return "Digite uma mensagem.".into();
        }

        session.push(ChatRole::User, trimmed);
        let _ = self.events.publish(TOPIC_JARBAS_USER, trimmed);

        mirror.emotion = crate::orb::OrbEmotion::Think;
        let response = match self.hermes.intent(trimmed) {
            Ok(r) => r,
            Err(e) => format!("Não consegui falar com o Hermes: {e}"),
        };

        mirror.set_from_response(&response);
        session.push(ChatRole::Assistant, &response);
        let _ = self.events.publish(TOPIC_JARBAS_ASSISTANT, &response);
        let _ = self.events.publish(TOPIC_JARBAS_ORB, mirror.emotion.label());
        response
    }

    pub fn boot_greeting(&self, session: &mut ChatSession, mirror: &mut SoulMirror) -> String {
        let prompt = self.soul.greeting_prompt();
        session.push(ChatRole::System, "boot greeting");
        self.chat(&prompt, session, mirror)
    }

    pub fn voice_utterance(
        &self,
        text: &str,
        session: &mut ChatSession,
        mirror: &mut SoulMirror,
    ) -> Result<String, String> {
        let pipeline = self
            .voice
            .as_ref()
            .ok_or_else(|| "voz desabilitada (REDOX_JARBAS_VOICE=1 para ativar)".to_string())?;
        let result = pipeline.process_utterance(text)?;
        session.push(ChatRole::User, &result.transcript);
        session.push(ChatRole::Assistant, &result.response);
        mirror.set_from_response(&result.response);
        let _ = self.events.publish(TOPIC_JARBAS_ASSISTANT, &result.response);
        Ok(result.response)
    }
}

impl Default for JarbasBridge {
    fn default() -> Self {
        Self::new()
    }
}
