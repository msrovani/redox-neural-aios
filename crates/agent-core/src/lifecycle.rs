//! Agentes de lifecycle ADR-001 (skeleton host — sem HW).

use std::time::Duration;

use crate::types::{Agent, AgentKind, AgentManifest, AgentTickResult, ScheduleKind};

macro_rules! lifecycle_agent {
    ($struct_name:ident, $name:literal, $kind:expr) => {
        pub struct $struct_name {
            tick: u64,
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self { tick: 0 }
            }
        }

        impl Agent for $struct_name {
            fn manifest(&self) -> &AgentManifest {
                static MANIFEST: AgentManifest = AgentManifest {
                    name: $name,
                    kind: $kind,
                    schedule: ScheduleKind::PollEvery(3600),
                    auto_start: false,
                    persist: true,
                };
                &MANIFEST
            }

            fn tick(&mut self) -> AgentTickResult {
                self.tick += 1;
                AgentTickResult::Pending
            }

            fn poll_interval(&self) -> Duration {
                Duration::from_secs(3600)
            }
        }
    };
}

lifecycle_agent!(OptimizerAgent, "optimizer", AgentKind::Inference);
lifecycle_agent!(SleepCycleAgent, "sleep_cycle", AgentKind::System);
lifecycle_agent!(AutoLearnAgent, "auto_learn", AgentKind::Skill);

pub fn lifecycle_agent_names() -> &'static [&'static str] {
    &["optimizer", "sleep_cycle", "auto_learn"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_agents_tick() {
        let mut opt = OptimizerAgent::default();
        assert_eq!(opt.manifest().name, "optimizer");
        assert!(matches!(opt.tick(), AgentTickResult::Pending));
    }
}
