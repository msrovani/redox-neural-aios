//! DynamicSkill — escada SkillMd → WASM (ADR-010 Onda 7h).

use std::path::PathBuf;

use wasm_skill_runtime::run_i32_0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillStage {
    /// Degrau 1 — SKILL.md registrada, workflow documentado.
    SkillMd,
    /// Degrau 2 — bytecode wasm em memória.
    WasmMemory,
    /// Degrau 2+ — `.wasm` persistido em disco.
    WasmPersistent,
}

#[derive(Clone, Debug)]
pub struct DynamicSkill {
    pub name: String,
    pub description: String,
    pub markdown: Option<String>,
    pub workflow: Vec<String>,
    pub trigger_keys: Vec<String>,
    pub instructions: String,
    pub wasm: Option<Vec<u8>>,
    pub export_fn: String,
    pub stage: SkillStage,
    pub runs: u32,
    pub success_rate: f32,
}

impl DynamicSkill {
    pub fn from_skill_md(
        parsed: &super::skill_md::ParsedSkillMd,
        trigger_key: impl Into<String>,
    ) -> Self {
        let trigger = parsed
            .trigger
            .clone()
            .unwrap_or_else(|| trigger_key.into());
        Self {
            name: parsed.name.clone(),
            description: parsed.description.clone(),
            markdown: Some(parsed.raw.clone()),
            workflow: parsed.workflow.clone(),
            trigger_keys: vec![trigger],
            instructions: parsed.contexto.clone(),
            wasm: None,
            export_fn: "run".into(),
            stage: SkillStage::SkillMd,
            runs: 0,
            success_rate: 0.0,
        }
    }

    pub fn ephemeral(
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            markdown: None,
            workflow: Vec::new(),
            trigger_keys: Vec::new(),
            instructions: instructions.into(),
            wasm: None,
            export_fn: "run".into(),
            stage: SkillStage::SkillMd,
            runs: 0,
            success_rate: 0.0,
        }
    }

    pub fn with_wasm(
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
        wasm: Vec<u8>,
        export_fn: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            markdown: None,
            workflow: Vec::new(),
            trigger_keys: Vec::new(),
            instructions: instructions.into(),
            wasm: Some(wasm),
            export_fn: export_fn.into(),
            stage: SkillStage::WasmMemory,
            runs: 0,
            success_rate: 0.0,
        }
    }

    pub fn record_run(&mut self, success: bool) {
        self.runs += 1;
        let prev = self.success_rate;
        self.success_rate = if self.runs == 1 {
            if success {
                1.0
            } else {
                0.0
            }
        } else {
            (prev * (self.runs as f32 - 1.0) + if success { 1.0 } else { 0.0 }) / self.runs as f32
        };
    }

    pub fn eligible_for_wasm_compile(&self) -> bool {
        self.stage == SkillStage::SkillMd
            && self.wasm.is_none()
            && self.runs >= PROMOTE_MIN_RUNS
            && self.success_rate >= PROMOTE_MIN_SUCCESS
    }

    pub fn eligible_for_file_promotion(&self) -> bool {
        self.wasm.is_some()
            && self.stage == SkillStage::WasmMemory
            && self.runs >= PROMOTE_MIN_RUNS
            && self.success_rate >= PROMOTE_MIN_SUCCESS
    }

    pub fn attach_wasm(&mut self, wasm: Vec<u8>, export_fn: impl Into<String>) {
        self.wasm = Some(wasm);
        self.export_fn = export_fn.into();
        self.stage = SkillStage::WasmMemory;
    }

    pub fn execute(&self, input: &str) -> Result<String, String> {
        if let Some(wasm) = &self.wasm {
            let caps = {
                let store = agent_core::load_cap_store();
                let ns = agent_core::build_namespace("wasm_skill", &store);
                agent_core::redox_caps::wasm_caps_from_namespace(&ns)
                    | agent_core::wasm_caps_from_grants()
            };
            let out = run_i32_0(wasm, &self.export_fn, caps)
                .map_err(|e| format!("wasm skill {}: {}", self.name, e.0))?;
            return Ok(format!(
                "[{}] wasm:{} input={} → {}",
                self.name, self.export_fn, input, out
            ));
        }

        if self.stage == SkillStage::SkillMd {
            let steps: String = self
                .workflow
                .iter()
                .enumerate()
                .map(|(i, s)| format!("  {}. {s}", i + 1))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok(format!(
                "[{}] SKILL.md (degrau 1)\n{}\n  input: {}",
                self.name, steps, input
            ));
        }

        Ok(format!(
            "[{}] fallback: {}\n  input: {}",
            self.name, self.instructions, input
        ))
    }

    pub fn promote_wasm_file(&mut self, wasm: Vec<u8>, export_fn: impl Into<String>) {
        self.wasm = Some(wasm);
        self.export_fn = export_fn.into();
        self.stage = SkillStage::WasmPersistent;
    }

    pub fn matches_trigger(&self, key: &str) -> bool {
        self.name == key || self.trigger_keys.iter().any(|t| t == key)
    }
}

pub const PROMOTE_MIN_RUNS: u32 = 3;
pub const PROMOTE_MIN_SUCCESS: f32 = 0.7;
pub const AUTO_SKILL_MIN_HITS: u32 = 3;

pub fn skills_dir() -> PathBuf {
    std::env::var("REDOX_SKILLS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                std::env::temp_dir().join("redox-aios-skills")
            } else {
                PathBuf::from("/var/lib/aios/skills")
            }
        })
}

pub fn factory_caps_enabled() -> bool {
    std::env::var("REDOX_FACTORY_CAPS")
        .map(|v| v != "0")
        .unwrap_or(true)
}

pub fn persist_wasm(name: &str, wasm: &[u8]) -> Result<PathBuf, String> {
    let dir = skills_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir skills: {e}"))?;
    let path = dir.join(format!("{name}.wasm"));
    std::fs::write(&path, wasm).map_err(|e| format!("write wasm: {e}"))?;
    Ok(path)
}

pub fn load_persisted_wasm(name: &str) -> Option<Vec<u8>> {
    let path = skills_dir().join(format!("{name}.wasm"));
    std::fs::read(path).ok()
}

#[derive(Clone, Debug, Default)]
pub struct BootLoadReport {
    pub skill_md_loaded: usize,
    pub wasm_loaded: usize,
    pub warnings: Vec<String>,
}

/// Carrega SKILL.md + `.wasm` persistidos de `/var/lib/aios/skills` (ADR-010 boot).
pub fn load_persisted_skills(registry: &mut crate::SkillRegistry) -> BootLoadReport {
    let mut report = BootLoadReport::default();
    let dir = skills_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        report
            .warnings
            .push(format!("skills_dir inacessível: {}", dir.display()));
        return report;
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                report
                    .warnings
                    .push(format!("ler {}: {e}", path.display()));
                continue;
            }
        };
        let parsed = match crate::skill_md::parse_skill_md(&raw) {
            Ok(p) => p,
            Err(e) => {
                report
                    .warnings
                    .push(format!("parse {}: {e}", path.display()));
                continue;
            }
        };
        let trigger = parsed
            .trigger
            .clone()
            .unwrap_or_else(|| stem.replace('-', "_"));
        let mut skill = DynamicSkill::from_skill_md(&parsed, trigger);
        if let Some(wasm) = load_persisted_wasm(stem) {
            skill.promote_wasm_file(wasm, "run");
            report.wasm_loaded += 1;
        }
        registry.register_dynamic(skill);
        report.skill_md_loaded += 1;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_md::parse_skill_md;
    use wasm_skill_runtime::echo_len_module;

    const SAMPLE_MD: &str = r#"---
schema: 1
kind: skill
name: demo
description: demo
contexto: test
acionaveis: ["on_demand"]
provenance: hermes_created
sandbox_status: none
---

## Contexto
c

## Goal
g

## Acionaveis
- on_demand

## Workflow
1. step_a
2. step_b

## Pre-Flight
- [ ] ok

## Success Criteria
- [ ] ok

## Failure Policy
retry
"#;

    #[test]
    fn skill_md_execute() {
        let parsed = parse_skill_md(SAMPLE_MD).unwrap();
        let skill = DynamicSkill::from_skill_md(&parsed, "demo_trigger");
        let out = skill.execute("hello").unwrap();
        assert!(out.contains("SKILL.md"));
        assert!(out.contains("step_a"));
    }

    #[test]
    fn wasm_compile_threshold() {
        let parsed = parse_skill_md(SAMPLE_MD).unwrap();
        let mut skill = DynamicSkill::from_skill_md(&parsed, "t");
        for _ in 0..3 {
            skill.record_run(true);
        }
        assert!(skill.eligible_for_wasm_compile());
        skill.attach_wasm(echo_len_module(), "run");
        assert!(skill.eligible_for_file_promotion());
    }
}
