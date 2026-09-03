//! Runner de lifecycle agents — wire SGDB + EventBus (ADR-001 adesão neural-os-core).

use agent_core::{
    AutoLearnAgent, LifecycleTick, OptimizerAgent, SelfHealAgent, SleepCycleAgent,
};

use crate::event_client::EventClient;
use crate::sgdb_client::SgdbClient;

pub const TOPIC_LIFECYCLE: &str = "LIFECYCLE_TICK";
pub const TOPIC_HEALTH_ISSUE: &str = "HEALTH_ISSUE";

fn publish_tick(events: &EventClient, tick: &LifecycleTick) {
    let payload = serde_json::json!({
        "agent": tick.agent,
        "phase": tick.phase,
        "detail": tick.detail,
        "scope": tick.remember_scope,
    })
    .to_string();
    let _ = events.publish(TOPIC_LIFECYCLE, &payload);
}

fn remember_tick(sgdb: &SgdbClient, tick: &LifecycleTick) {
    let _ = sgdb.remember(&tick.line(), tick.remember_scope);
}

/// Executa um ciclo completo dos 4 agentes de lifecycle ADR-001.
pub fn run_lifecycle_cycle(sgdb: &SgdbClient, events: &EventClient) -> String {
    let mut lines = Vec::new();

    let mut heal = SelfHealAgent::default();
    let (heal_tick, report) = heal.run_once();
    remember_tick(sgdb, &heal_tick);
    publish_tick(events, &heal_tick);
    if !report.healthy() {
        let _ = events.publish(TOPIC_HEALTH_ISSUE, &report.summary());
        let _ = sgdb.remember(&report.format(), "hermes/selfheal");
    }
    lines.push(heal_tick.line());
    lines.push(report.format());

    let mut opt = OptimizerAgent::default();
    let opt_tick = opt.run_once();
    remember_tick(sgdb, &opt_tick);
    publish_tick(events, &opt_tick);
    lines.push(opt_tick.line());

    let recalled = sgdb
        .recall("gap OR stub OR offline OR unmatched", "hermes", 8)
        .unwrap_or_else(|_| "(sem recall)".into());

    let mut sleep = SleepCycleAgent::default();
    for tick in sleep.run_full_cycle(&recalled) {
        remember_tick(sgdb, &tick);
        publish_tick(events, &tick);
        lines.push(tick.line());
    }

    let mut learn = AutoLearnAgent::default();
    let learn_tick = learn.run_once(&recalled);
    remember_tick(sgdb, &learn_tick);
    publish_tick(events, &learn_tick);
    lines.push(learn_tick.line());

    lines.join("\n")
}

/// Atalho: só SelfHeal (scan + proposta).
pub fn run_self_heal(sgdb: &SgdbClient, events: &EventClient) -> String {
    let mut heal = SelfHealAgent::default();
    let (tick, report) = heal.run_once();
    remember_tick(sgdb, &tick);
    publish_tick(events, &tick);
    if !report.healthy() {
        let _ = events.publish(TOPIC_HEALTH_ISSUE, &report.summary());
    }
    let _ = sgdb.remember(&report.format(), "hermes/selfheal");
    format!("{}\n{}", tick.line(), report.format())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_cycle_contains_phases() {
        let sgdb = SgdbClient::new();
        let events = EventClient::new();
        let out = run_lifecycle_cycle(&sgdb, &events);
        assert!(out.contains("self_heal"));
        assert!(out.contains("optimizer"));
        assert!(out.contains("sleep_cycle"));
        assert!(out.contains("auto_learn"));
        assert!(out.contains("REPLAY") || out.contains("phase=REPLAY"));
    }
}
