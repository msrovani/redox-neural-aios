//! Skills builtin do Hermes (Fase 2).

use i18n_core::t;
use skill_registry::{Skill, SkillManifest, SkillRegistry};

use crate::sgdb_client::SgdbClient;

pub struct EchoSkillForTest;
impl Skill for EchoSkillForTest {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "echo",
            description: "Repete o texto",
            hitl_required: false,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(input.to_string())
    }
}

struct EchoSkill;
impl Skill for EchoSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "echo",
            description: "Repete o texto",
            hitl_required: false,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(input.to_string())
    }
}

struct TimeSkill;
impl Skill for TimeSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "time",
            description: "Timestamp do sistema",
            hitl_required: false,
        }
    }
    fn execute(&self, _input: &str) -> Result<String, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Ok(format!("epoch={secs}"))
    }
}

struct HelpSkill;
impl Skill for HelpSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "help",
            description: "Ajuda de comandos",
            hitl_required: false,
        }
    }
    fn execute(&self, _input: &str) -> Result<String, String> {
        Ok(format!("{}\n{}", t("help.commands"), t("help.nl")))
    }
}

struct SkillsListSkill;
impl Skill for SkillsListSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "skills",
            description: "Lista skills registradas",
            hitl_required: false,
        }
    }
    fn execute(&self, _input: &str) -> Result<String, String> {
        Ok("echo, time, status, remember, recall, help, skills".into())
    }
}

struct SgdbStatusSkill {
    client: SgdbClient,
}
impl Skill for SgdbStatusSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "status",
            description: "Health do neural-sgdb",
            hitl_required: false,
        }
    }
    fn execute(&self, _input: &str) -> Result<String, String> {
        self.client.health()
    }
}

struct RememberSkill {
    client: SgdbClient,
}
impl Skill for RememberSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "remember",
            description: "Memoriza no SGDB (scope hermes)",
            hitl_required: false,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        self.client.remember(input, "hermes")
    }
}

struct RecallSkill {
    client: SgdbClient,
}
impl Skill for RecallSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "recall",
            description: "Recall lexical no SGDB",
            hitl_required: false,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        self.client.recall(input, "hermes", 5)
    }
}

pub fn register_builtin_skills(registry: &mut SkillRegistry) {
    let sgdb = SgdbClient::new();
    registry.register(Box::new(EchoSkill));
    registry.register(Box::new(TimeSkill));
    registry.register(Box::new(HelpSkill));
    registry.register(Box::new(SkillsListSkill));
    registry.register(Box::new(SgdbStatusSkill {
        client: sgdb.clone_for_skills(),
    }));
    registry.register(Box::new(RememberSkill {
        client: sgdb.clone_for_skills(),
    }));
    registry.register(Box::new(RecallSkill { client: sgdb }));
}
