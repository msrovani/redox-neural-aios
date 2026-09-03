//! Hermes router — intent → efêmera / skill / wasm / cortex.

use std::sync::Mutex;

use skill_registry::{DynamicSkill, SkillRegistry};

use crate::app_factory::FactoryStage;
use crate::cortex_client::CortexClient;
use crate::ephemeral::run_ephemeral;
use crate::event_client::EventClient;
use crate::factory_cycle::{emit_factory_stage, remember_factory_step, FactoryPhase};
use crate::intent::{command_to_skill, parse_command, Command};
use crate::prompt::system_prompt;
use crate::react::ReActTrace;
use crate::cookbook_bridge::{list_staged_recipes, promote_skill_to_recipe};
use crate::self_evolve::{observe_and_maybe_generate, record_dynamic_outcome};
use crate::sgdb_client::SgdbClient;
use crate::skill_observer::normalize_key;
use crate::tools::default_tool_registry;
use crate::workflow::execute_skill_workflow;

const BUILTIN_SKILLS: &[&str] = &[
    "echo", "time", "status", "remember", "recall", "skills", "help", "factory", "promote",
    "opir", "tools", "evolve", "lifecycle", "selfheal", "ota", "caps",
];

pub struct HermesRouter {
    registry: Mutex<SkillRegistry>,
    sgdb: SgdbClient,
    cortex: CortexClient,
    tools: std::sync::Mutex<crate::tools::ToolRegistry>,
    events: EventClient,
}

pub struct RouteResult {
    pub response: String,
    pub topic: &'static str,
    pub trace: Vec<String>,
}

impl HermesRouter {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry: Mutex::new(registry),
            sgdb: SgdbClient::new(),
            cortex: CortexClient::new(),
            tools: std::sync::Mutex::new(default_tool_registry()),
            events: EventClient::new(),
        }
    }

    pub fn with_events(registry: SkillRegistry, events: EventClient) -> Self {
        Self {
            registry: Mutex::new(registry),
            sgdb: SgdbClient::new(),
            cortex: CortexClient::new(),
            tools: std::sync::Mutex::new(default_tool_registry()),
            events,
        }
    }

    pub fn with_shared_registry(registry: Mutex<SkillRegistry>) -> Self {
        Self {
            registry,
            sgdb: SgdbClient::new(),
            cortex: CortexClient::new(),
            tools: std::sync::Mutex::new(default_tool_registry()),
            events: EventClient::new(),
        }
    }

    fn cortex_complete(&self, text: &str) -> String {
        match self.cortex.complete(text, Some(&system_prompt())) {
            Ok(r) => r,
            Err(e) => i18n_core::t_fmt("jarbas.offline", &[("error", &e)]),
        }
    }

    fn finish_intelligence(
        &self,
        text: &str,
        response: String,
        trace: &mut ReActTrace,
        lines: &mut Vec<String>,
    ) -> RouteResult {
        let _ = self
            .sgdb
            .remember(&format!("Q:{text} A:{response}"), "hermes");
        trace.advance("cortex+remember");
        lines.push(trace.line());
        RouteResult {
            response,
            topic: "HERMES_RESPONSE",
            trace: lines.clone(),
        }
    }

    fn finish_ephemeral(
        &self,
        text: &str,
        response: String,
        trace: &mut ReActTrace,
        lines: &mut Vec<String>,
        step_trace: &str,
    ) -> RouteResult {
        remember_factory_step(
            &self.sgdb,
            Some(&self.events),
            FactoryPhase::Remember,
            FactoryStage::Ephemeral,
            text,
        );
        emit_factory_stage(
            Some(&self.events),
            FactoryPhase::Act,
            FactoryStage::Ephemeral,
            step_trace,
        );
        let _ = self
            .sgdb
            .remember(&format!("Q:{text} A:{response}"), "hermes");
        trace.advance(format!("ephemeral+remember ({step_trace})"));
        lines.push(trace.line());
        RouteResult {
            response,
            topic: "HERMES_RESPONSE",
            trace: lines.clone(),
        }
    }

    fn finish_skill(
        &self,
        text: &str,
        skill: &str,
        response: Result<String, String>,
        registry: &mut SkillRegistry,
        trace: &mut ReActTrace,
        lines: &mut Vec<String>,
    ) -> RouteResult {
        let ok = response.is_ok();
        let body = match response {
            Ok(r) => r,
            Err(e) => format!("erro skill {skill}: {e}"),
        };
        let stage = if registry.dynamic_mut(skill).map(|s| s.wasm.is_some()).unwrap_or(false) {
            FactoryStage::WasmMemory
        } else {
            FactoryStage::SkillMd
        };
        if registry.dynamic_mut(skill).is_some() {
            let outcome = record_dynamic_outcome(registry, skill, ok);
            if let Some(msg) = outcome.wasm_compiled {
                emit_factory_stage(Some(&self.events), FactoryPhase::Verify, FactoryStage::WasmMemory, &msg);
                remember_factory_step(
                    &self.sgdb,
                    Some(&self.events),
                    FactoryPhase::Act,
                    FactoryStage::WasmMemory,
                    &msg,
                );
            }
            if let Some(msg) = outcome.wasm_persisted {
                emit_factory_stage(
                    Some(&self.events),
                    FactoryPhase::Verify,
                    FactoryStage::WasmPersistent,
                    &msg,
                );
                remember_factory_step(
                    &self.sgdb,
                    Some(&self.events),
                    FactoryPhase::Remember,
                    FactoryStage::WasmPersistent,
                    &msg,
                );
            }
        }
        remember_factory_step(
            &self.sgdb,
            Some(&self.events),
            FactoryPhase::Remember,
            stage,
            skill,
        );
        let _ = self
            .sgdb
            .remember(&format!("Q:{text} A:{body}"), "hermes");
        trace.advance("verify+remember");
        lines.push(trace.line());
        RouteResult {
            response: body,
            topic: "HERMES_RESPONSE",
            trace: lines.clone(),
        }
    }

    fn execute_dynamic(
        &self,
        skill: &DynamicSkill,
        input: &str,
    ) -> Result<String, String> {
        let tools = self.tools.lock().expect("tools");
        execute_skill_workflow(skill, input, &self.sgdb, &self.cortex, &tools)
    }

    pub fn handle_intent(&self, text: &str) -> RouteResult {
        let mut trace = ReActTrace::new(format!("intent: {text}"));
        let mut lines = vec![trace.line()];

        if let Some(blocked) = crate::hitl::gate_response(text) {
            trace.advance("hitl_blocked");
            lines.push(trace.line());
            return RouteResult {
                response: blocked,
                topic: "HERMES_HITL_BLOCKED",
                trace: lines,
            };
        }

        let mut registry = self.registry.lock().expect("skill registry");
        emit_factory_stage(
            Some(&self.events),
            FactoryPhase::Observe,
            FactoryStage::Ephemeral,
            text,
        );
        let evolve = observe_and_maybe_generate(&mut registry, text);
        if let Some(gen) = evolve.generated {
            trace.advance(format!("self_evolve: {gen}"));
            lines.push(trace.line());
            emit_factory_stage(
                Some(&self.events),
                FactoryPhase::Act,
                FactoryStage::SkillMd,
                &gen,
            );
            remember_factory_step(
                &self.sgdb,
                Some(&self.events),
                FactoryPhase::Remember,
                FactoryStage::SkillMd,
                &gen,
            );
        }
        if let Some(promo) = evolve.promoted {
            trace.advance(format!("promoted: {promo}"));
            lines.push(trace.line());
        }
        if let Some(wasm) = evolve.wasm_compiled {
            trace.advance(format!("wasm: {wasm}"));
            lines.push(trace.line());
        }

        let cmd = parse_command(text);
        trace.advance("parse_command");
        lines.push(trace.line());

        if let Command::Chat(ref chat) = cmd {
            let key = normalize_key(chat);
            if let Some(skill) = registry.find_by_trigger(&key).cloned() {
                trace.advance(format!("skill={} trigger={key}", skill.name));
                lines.push(trace.line());
                let response = self.execute_dynamic(&skill, chat);
                return self.finish_skill(
                    text,
                    &skill.name,
                    response,
                    &mut registry,
                    &mut trace,
                    &mut lines,
                );
            }
            trace.advance("ephemeral pipeline (degrau 0)");
            lines.push(trace.line());
            let tools = self.tools.lock().expect("tools");
            let ephem = run_ephemeral(chat, &self.sgdb, &self.cortex, &tools);
            let step_trace = ephem.traces.join(", ");
            return self.finish_ephemeral(
                text,
                ephem.response,
                &mut trace,
                &mut lines,
                &step_trace,
            );
        }

        if let Some((skill, input)) = command_to_skill(&cmd) {
            if skill == "promote" {
                trace.advance("cookbook promote");
                lines.push(trace.line());
                return self.handle_promote(text, &input, &mut registry, &mut trace, &mut lines);
            }

            let is_known = BUILTIN_SKILLS.contains(&skill)
                || registry.dynamic_names().iter().any(|n| *n == skill);
            if is_known {
                trace.advance(format!("execute skill={skill}"));
                lines.push(trace.line());
                if skill == "lifecycle" {
                    let out = crate::lifecycle_runner::run_lifecycle_cycle(&self.sgdb, &self.events);
                    return self.finish_skill(
                        text,
                        skill,
                        Ok(out),
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                if skill == "selfheal" {
                    let out = crate::lifecycle_runner::run_self_heal(&self.sgdb, &self.events);
                    return self.finish_skill(
                        text,
                        skill,
                        Ok(out),
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                if skill == "ota" {
                    let out = crate::ota_cmd::handle_ota(&input, &self.sgdb, &self.events);
                    return self.finish_skill(
                        text,
                        skill,
                        out,
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                if skill == "caps" {
                    let out = crate::caps_cmd::handle_caps(&input, &self.sgdb, &self.events);
                    return self.finish_skill(
                        text,
                        skill,
                        out,
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                if skill == "tools" {
                    let tools = self.tools.lock().expect("tools");
                    let list: Vec<String> = tools
                        .list()
                        .iter()
                        .map(|(id, desc)| format!("{id}: {desc}"))
                        .collect();
                    let providers = crate::tools::registered_provider_ids();
                    let provider_line = if providers.is_empty() {
                        "providers: (nenhum — REDOX_TOOLS_NET=1 REDOX_TOOLS_PROVIDERS=all)".into()
                    } else {
                        format!("providers: {}", providers.join(", "))
                    };
                    return self.finish_skill(
                        text,
                        skill,
                        Ok(format!("{}\n{}", list.join("\n"), provider_line)),
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                if skill == "evolve" {
                    let names = registry.dynamic_names();
                    let body = if names.is_empty() {
                        "nenhuma skill gerada ainda".into()
                    } else {
                        format!("skills geradas: {}", names.join(", "))
                    };
                    return self.finish_skill(
                        text,
                        skill,
                        Ok(body),
                        &mut registry,
                        &mut trace,
                        &mut lines,
                    );
                }
                let response = registry.execute(skill, &input);
                return self.finish_skill(text, skill, response, &mut registry, &mut trace, &mut lines);
            }

            trace.advance(format!("cortex skill={skill}"));
            lines.push(trace.line());
            let prompt = if input.is_empty() {
                format!("O usuário pediu a skill /{skill}. Responda de forma útil.")
            } else {
                format!(
                    "O usuário pediu a skill /{skill} com entrada: {input}. \
                     Execute mentalmente e responda."
                )
            };
            let response = self.cortex_complete(&prompt);
            return self.finish_intelligence(text, response, &mut trace, &mut lines);
        }

        if let Command::Unknown(ref unknown) = cmd {
            let key = normalize_key(unknown);
            if let Some(skill) = registry.find_by_trigger(&key).cloned() {
                trace.advance(format!("skill={}", skill.name));
                lines.push(trace.line());
                let response = self.execute_dynamic(&skill, unknown);
                return self.finish_skill(
                    text,
                    &skill.name,
                    response,
                    &mut registry,
                    &mut trace,
                    &mut lines,
                );
            }
            trace.advance("ephemeral pipeline (unknown)");
            lines.push(trace.line());
            let tools = self.tools.lock().expect("tools");
            let ephem = run_ephemeral(unknown, &self.sgdb, &self.cortex, &tools);
            return self.finish_ephemeral(
                text,
                ephem.response,
                &mut trace,
                &mut lines,
                &ephem.traces.join(", "),
            );
        }

        RouteResult {
            response: "nada a executar".into(),
            topic: "HERMES_RESPONSE",
            trace: lines,
        }
    }

    fn handle_promote(
        &self,
        text: &str,
        input: &str,
        registry: &mut SkillRegistry,
        trace: &mut ReActTrace,
        lines: &mut Vec<String>,
    ) -> RouteResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.first() == Some(&"list") {
            let staged = list_staged_recipes();
            let body = if staged.is_empty() {
                "nenhuma recipe staged".into()
            } else {
                format!(
                    "recipes staged ({}):\n{}",
                    staged.len(),
                    staged
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };
            return self.finish_skill(text, "promote", Ok(body), registry, trace, lines);
        }

        let skill_name = parts.first().copied().unwrap_or("");
        if skill_name.is_empty() {
            return self.finish_skill(
                text,
                "promote",
                Ok("uso: /promote <skill> [approve] | /promote list".into()),
                registry,
                trace,
                lines,
            );
        }

        let hitl_approved = parts.iter().any(|p| *p == "approve" || *p == "yes");
        let skill = registry
            .dynamic_mut(skill_name)
            .cloned()
            .ok_or_else(|| format!("skill dinâmica não encontrada: {skill_name}"));

        match skill {
            Ok(s) => match promote_skill_to_recipe(&s, hitl_approved) {
                Ok(result) => {
                    let msg = format!(
                        "recipe staged: {}\n{}",
                        result.recipe_path.display(),
                        result.pkgutils_cmd
                    );
                    emit_factory_stage(
                        Some(&self.events),
                        FactoryPhase::Act,
                        FactoryStage::AppRecipe,
                        &msg,
                    );
                    remember_factory_step(
                        &self.sgdb,
                        Some(&self.events),
                        FactoryPhase::Remember,
                        FactoryStage::AppRecipe,
                        &msg,
                    );
                    self.finish_skill(text, "promote", Ok(msg), registry, trace, lines)
                }
                Err(e) => self.finish_skill(text, "promote", Err(e), registry, trace, lines),
            },
            Err(e) => self.finish_skill(text, "promote", Err(e), registry, trace, lines),
        }
    }
}
