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
        Ok("echo, time, status, remember, recall, help, skills, factory, opir, promote, tools, evolve, lifecycle, selfheal, ota (+ dynamic)".into())
    }
}

struct PromoteSkill;
impl Skill for PromoteSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "promote",
            description: "Promove skill WASM madura para recipe cookbook (HITL)",
            hitl_required: true,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(format!(
            "use /promote <skill> approve — input atual: {input:?}\n\
             /promote list — recipes staged"
        ))
    }
}

struct OpIrSkill;
impl Skill for OpIrSkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "opir",
            description: "Self-test op-IR → WASM (Onda 7g)",
            hitl_required: false,
        }
    }
    fn execute(&self, input: &str) -> Result<String, String> {
        use crate::wasm_gen::{generate_wasm_for_skill, verify_wasm};
        use wasm_skill_runtime::self_test as op_ir_self_test;

        let op_ir_ok = op_ir_self_test();
        let expr = if input.trim().is_empty() {
            "a*b+7"
        } else {
            input.trim()
        };
        let gen = generate_wasm_for_skill(expr, expr);
        let run = verify_wasm(&gen.wasm, &gen.export_fn)
            .map(|v| v.to_string())
            .unwrap_or_else(|e| format!("ERR: {}", e.0));
        Ok(format!(
            "op-IR self-test={op_ir_ok} source={:?} expr={:?} run→{run}",
            gen.source,
            gen.expr
        ))
    }
}

struct FactorySkill;
impl Skill for FactorySkill {
    fn manifest(&self) -> SkillManifest {
        SkillManifest {
            name: "factory",
            description: "Self-test Runtime App Factory (wasmi)",
            hitl_required: false,
        }
    }
    fn execute(&self, _input: &str) -> Result<String, String> {
        use crate::app_factory::{
            analyze_and_recommend, execute_recommended, factory_backend_report, format_ladder_help,
            AppRequest,
        };
        use wasm_skill_runtime::{add_module, echo_len_module};
        let rec = analyze_and_recommend(&AppRequest {
            desc: "factory self-test".into(),
            trusted: false,
            perf_critical: false,
        });
        let add = wasm_skill_runtime::run_i32_2(
            &add_module(),
            "add",
            2,
            3,
            wasm_skill_runtime::CAP_LOG,
        )
        .map(|v| v.to_string())
        .unwrap_or_else(|e| format!("add FAIL: {}", e.0));
        let run = match execute_recommended(&rec, &echo_len_module(), "run") {
            crate::app_factory::FactoryOutcome::RanWasm(v) => v.to_string(),
            crate::app_factory::FactoryOutcome::Denied(m) => format!("DENIED: {m}"),
            crate::app_factory::FactoryOutcome::AwaitingIsolation(b) => {
                format!("AWAITING: {b:?}")
            }
        };
        let op_ir = wasm_skill_runtime::self_test();
        let backends = factory_backend_report();
        Ok(format!(
            "{}\nadd=2+3→{add} run→{run} op-IR={op_ir} net={} hitl={} caps={}\n\n{}",
            t("factory.self_test.ok"),
            if backends.tools_net { "on" } else { "off" },
            if backends.hitl_enabled { "on" } else { "off" },
            if backends.factory_caps { "on" } else { "off" },
            format_ladder_help(),
        ))
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
    registry.register(Box::new(FactorySkill));
    registry.register(Box::new(OpIrSkill));
    registry.register(Box::new(PromoteSkill));
    registry.register(Box::new(SgdbStatusSkill {
        client: sgdb.clone_for_skills(),
    }));
    registry.register(Box::new(RememberSkill {
        client: sgdb.clone_for_skills(),
    }));
    registry.register(Box::new(RecallSkill { client: sgdb }));
}
