//! Hermes orchestrator core — Redox AIOS.

pub mod app_factory;
pub mod boot_observe;
pub mod cookbook_bridge;
pub mod cortex_client;
pub mod ephemeral;
pub mod event_client;
pub mod factory_boot;
pub mod factory_cycle;
pub mod hitl;
pub mod intent;
pub mod prompt;
pub mod react;
pub mod router;
pub mod self_evolve;
pub mod sgdb_client;
pub mod skill_gen;
pub mod skill_observer;
pub mod skill_opt;
pub mod skills;
pub mod tools;
pub mod wasm_gen;
pub mod workflow;

pub use factory_boot::{boot_skill_registry, format_boot_report, FactoryBootReport};
pub use intent::{parse_command, Command};
pub use router::{HermesRouter, RouteResult};
pub use skills::register_builtin_skills;

pub const TOPIC_USER_INTENT: &str = "USER_INTENT";
pub const TOPIC_HERMES_RESPONSE: &str = "HERMES_RESPONSE";
pub const TOPIC_FACTORY_STAGE: &str = "FACTORY_STAGE";
pub const TOPIC_FACTORY_REMEMBER: &str = "FACTORY_REMEMBER";
pub const TOPIC_FACTORY_BOOT: &str = "FACTORY_BOOT";
pub const DEFAULT_HERMES_SOCKET: &str = "127.0.0.1:7742";

#[cfg(test)]
mod tests {
    use super::*;
    use intent::Command;
    use skill_registry::SkillRegistry;

    #[test]
    fn parse_slash_commands() {
        assert_eq!(parse_command("/echo ola"), Command::Echo("ola".into()));
        assert_eq!(parse_command("/time"), Command::Time);
        assert!(matches!(parse_command("que horas sao"), Command::Time));
    }

    #[test]
    fn router_echo_without_sgdb() {
        let mut registry = SkillRegistry::new();
        registry.register(Box::new(skills::EchoSkillForTest));
        let router = HermesRouter::new(registry);
        let result = router.handle_intent("/echo teste");
        assert!(result.response.contains("teste"));
    }

    #[test]
    fn hitl_blocks_destructive() {
        let registry = SkillRegistry::new();
        let router = HermesRouter::new(registry);
        let result = router.handle_intent("rm -rf /");
        assert!(result.response.contains("HITL"));
    }
}
