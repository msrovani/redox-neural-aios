//! Ciclo cognitivo Observe → Plan → Act → Verify → Remember (ADR-001 mand. 3).

use crate::app_factory::FactoryStage;
use crate::event_client::EventClient;
use crate::sgdb_client::SgdbClient;
use crate::{TOPIC_FACTORY_STAGE, TOPIC_FACTORY_REMEMBER};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactoryPhase {
    Observe,
    Plan,
    Act,
    Verify,
    Remember,
}

impl FactoryPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Plan => "plan",
            Self::Act => "act",
            Self::Verify => "verify",
            Self::Remember => "remember",
        }
    }
}

pub fn emit_factory_stage(
    events: Option<&EventClient>,
    phase: FactoryPhase,
    stage: FactoryStage,
    detail: &str,
) {
    let Some(events) = events else {
        return;
    };
    let payload = serde_json::json!({
        "phase": phase.as_str(),
        "stage": stage.as_str(),
        "detail": detail,
    })
    .to_string();
    let _ = events.publish(TOPIC_FACTORY_STAGE, &payload);
}

pub fn remember_factory_step(
    sgdb: &SgdbClient,
    events: Option<&EventClient>,
    phase: FactoryPhase,
    stage: FactoryStage,
    detail: &str,
) {
    let scope = "hermes/factory";
    let text = format!(
        "factory phase={} stage={} detail={}",
        phase.as_str(),
        stage.as_str(),
        detail.chars().take(240).collect::<String>()
    );
    let _ = sgdb.remember(&text, scope);
    if let Some(events) = events {
        let payload = serde_json::json!({
            "phase": phase.as_str(),
            "stage": stage.as_str(),
            "scope": scope,
        })
        .to_string();
        let _ = events.publish(TOPIC_FACTORY_REMEMBER, &payload);
    }
}
