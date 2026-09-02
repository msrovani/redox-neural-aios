//! Registro de skills executáveis pelo Hermes (userspace std).

mod dynamic;
mod skill_md;

use std::collections::BTreeMap;

pub use dynamic::{
    factory_caps_enabled, load_persisted_skills, load_persisted_wasm, persist_wasm, skills_dir,
    BootLoadReport, DynamicSkill, SkillStage, AUTO_SKILL_MIN_HITS, PROMOTE_MIN_RUNS,
    PROMOTE_MIN_SUCCESS,
};
pub use skill_md::{parse_skill_md, persist_skill_md, verify_skill_md, ParsedSkillMd};

#[derive(Clone, Debug)]
pub struct SkillManifest {
    pub name: &'static str,
    pub description: &'static str,
    pub hitl_required: bool,
}

pub trait Skill: Send + Sync {
    fn manifest(&self) -> SkillManifest;
    fn execute(&self, input: &str) -> Result<String, String>;
}

#[derive(Clone, Debug, Default)]
pub struct ToolPolicy {
    pub enabled: bool,
    pub auto_approve: bool,
}

pub struct SkillRegistry {
    skills: BTreeMap<&'static str, Box<dyn Skill>>,
    dynamic: BTreeMap<String, DynamicSkill>,
    policies: BTreeMap<&'static str, ToolPolicy>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: BTreeMap::new(),
            dynamic: BTreeMap::new(),
            policies: BTreeMap::new(),
        }
    }

    pub fn register_dynamic(&mut self, skill: DynamicSkill) {
        let name = skill.name.clone();
        self.dynamic.insert(name, skill);
    }

    pub fn dynamic_mut(&mut self, name: &str) -> Option<&mut DynamicSkill> {
        self.dynamic.get_mut(name)
    }

    pub fn find_by_trigger(&self, trigger_key: &str) -> Option<&DynamicSkill> {
        self.dynamic
            .values()
            .find(|s| s.matches_trigger(trigger_key))
    }

    pub fn find_by_trigger_mut(&mut self, trigger_key: &str) -> Option<&mut DynamicSkill> {
        let name = self
            .dynamic
            .iter()
            .find(|(_, s)| s.matches_trigger(trigger_key))
            .map(|(n, _)| n.clone())?;
        self.dynamic.get_mut(&name)
    }

    pub fn resolve_execute_name(&self, trigger_key: &str) -> Option<String> {
        self.find_by_trigger(trigger_key)
            .map(|s| s.name.clone())
    }

    pub fn has_generated_skill(&self, trigger_key: &str) -> bool {
        self.find_by_trigger(trigger_key).is_some()
    }

    pub fn dynamic_names(&self) -> Vec<&str> {
        self.dynamic.keys().map(String::as_str).collect()
    }

    pub fn all_skill_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.skills.keys().map(|s| (*s).to_string()).collect();
        names.extend(self.dynamic.keys().cloned());
        names.sort();
        names
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        let name = skill.manifest().name;
        self.skills.insert(name, skill);
    }

    pub fn set_policy(&mut self, name: &'static str, policy: ToolPolicy) {
        self.policies.insert(name, policy);
    }

    pub fn list_names(&self) -> Vec<&'static str> {
        self.skills.keys().copied().collect()
    }

    pub fn list_manifests(&self) -> Vec<SkillManifest> {
        self.skills.values().map(|s| s.manifest()).collect()
    }

    pub fn execute(&self, name: &str, input: &str) -> Result<String, String> {
        if let Some(skill) = self.dynamic.get(name) {
            return skill.execute(input);
        }
        let skill = self
            .skills
            .get(name)
            .ok_or_else(|| format!("skill nao encontrada: {name}"))?;
        if let Some(policy) = self.policies.get(name) {
            if !policy.enabled {
                return Err(format!("skill desabilitada: {name}"));
            }
        }
        skill.execute(input)
    }

    pub fn execute_by_trigger(&self, trigger_key: &str, input: &str) -> Result<String, String> {
        let name = self
            .resolve_execute_name(trigger_key)
            .ok_or_else(|| format!("skill nao encontrada para trigger: {trigger_key}"))?;
        self.execute(&name, input)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
