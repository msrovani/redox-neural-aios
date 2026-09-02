//! Self-evolve — observe → SKILL.md → WASM (ADR-010 Onda 7h).

use std::sync::{Mutex, OnceLock};

use skill_registry::{parse_skill_md, persist_skill_md, DynamicSkill, SkillRegistry, SkillStage};

use crate::skill_gen::maybe_auto_skill;
use crate::skill_observer::{is_auto_skill_candidate, normalize_key, observe_intent};
use crate::skill_opt::{maybe_compile_wasm, maybe_persist_wasm};
use crate::wasm_gen::generate_wasm_for_skill;

fn generated() -> &'static Mutex<Vec<String>> {
    static GEN: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    GEN.get_or_init(|| Mutex::new(Vec::new()))
}

pub struct EvolveReport {
    pub observed_hits: u32,
    pub generated: Option<String>,
    pub wasm_compiled: Option<String>,
    pub promoted: Option<String>,
}

pub fn observe_and_maybe_generate(
    registry: &mut SkillRegistry,
    text: &str,
) -> EvolveReport {
    let obs = observe_intent(text);
    let hits = obs.as_ref().map(|o| o.hits).unwrap_or(0);
    let key = normalize_key(text);

    let mut report = EvolveReport {
        observed_hits: hits,
        generated: None,
        wasm_compiled: None,
        promoted: None,
    };

    if !is_auto_skill_candidate(text) {
        return report;
    }

    if registry.has_generated_skill(&key) {
        return report;
    }

    if generated()
        .lock()
        .ok()
        .map(|g| g.iter().any(|n| n == &key))
        .unwrap_or(false)
    {
        return report;
    }

    let Some(md) = maybe_auto_skill(&key, hits) else {
        return report;
    };

    let parsed = match parse_skill_md(&md) {
        Ok(p) => p,
        Err(e) => {
            report.generated = Some(format!("SKILL.md reject: {e}"));
            return report;
        }
    };

    let skill = DynamicSkill::from_skill_md(&parsed, key.clone());
    let skill_name = skill.name.clone();
    registry.register_dynamic(skill);

    let _ = persist_skill_md(&skill_name, &md);

    if let Some(mut g) = generated().lock().ok() {
        g.push(key.clone());
    }

    report.generated = Some(format!(
        "registered SKILL.md `{skill_name}` (trigger=`{key}`, hits={hits})"
    ));

    report
}

pub struct DynamicOutcome {
    pub wasm_compiled: Option<String>,
    pub wasm_persisted: Option<String>,
}

pub fn record_dynamic_outcome(
    registry: &mut SkillRegistry,
    trigger_or_name: &str,
    success: bool,
) -> DynamicOutcome {
    let mut outcome = DynamicOutcome {
        wasm_compiled: None,
        wasm_persisted: None,
    };
    let key = normalize_key(trigger_or_name);
    let skill_name = registry
        .resolve_execute_name(&key)
        .or_else(|| registry.resolve_execute_name(trigger_or_name))
        .unwrap_or_else(|| trigger_or_name.to_string());

    let Some(skill) = registry.dynamic_mut(&skill_name) else {
        return outcome;
    };

    skill.record_run(success);

    if skill.stage == SkillStage::SkillMd && skill.eligible_for_wasm_compile() {
        let sample = skill.description.clone();
        let gen = generate_wasm_for_skill(&sample, &skill.name);
        if let Some(skill) = registry.dynamic_mut(&skill_name) {
            outcome.wasm_compiled = maybe_compile_wasm(skill, gen.wasm, &gen.export_fn);
        }
    }

    if let Some(skill) = registry.dynamic_mut(&skill_name) {
        outcome.wasm_persisted = maybe_persist_wasm(skill);
    }
    outcome
}

pub fn generated_skills() -> Vec<String> {
    generated().lock().ok().map(|g| g.clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use skill_registry::SkillRegistry;

    #[test]
    fn generates_skill_md_not_wasm_on_third_hit() {
        let mut reg = SkillRegistry::new();
        let phrase = "qual a temperatura em sp";
        for _ in 0..3 {
            let r = observe_and_maybe_generate(&mut reg, phrase);
            if r.generated.is_some() {
                assert!(r.generated.as_ref().unwrap().contains("SKILL.md"));
                let skill = reg.find_by_trigger("qual_a_temperatura_em_sp").expect("skill");
                assert_eq!(skill.stage, SkillStage::SkillMd);
                assert_eq!(skill.name, "a_temperatura_em_sp");
                assert!(skill.wasm.is_none());
                assert!(skill.workflow.iter().any(|s| s == "synthesize_response"));
                return;
            }
        }
        panic!("expected SKILL.md generation on third hit");
    }

    #[test]
    fn compiles_wasm_after_mature_runs() {
        let mut reg = SkillRegistry::new();
        for _ in 0..3 {
            observe_and_maybe_generate(&mut reg, "qual a temperatura hoje");
        }
        let name = reg.resolve_execute_name("qual_a_temperatura_hoje").unwrap();
        for _ in 0..3 {
            record_dynamic_outcome(&mut reg, &name, true);
        }
        let skill = reg.dynamic_mut(&name).unwrap();
        assert!(
            skill.stage == SkillStage::WasmMemory || skill.stage == SkillStage::WasmPersistent
        );
        assert!(skill.wasm.is_some());
    }
}
