//! Registro de skills executáveis pelo Hermes (userspace std).

use std::collections::BTreeMap;

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
    policies: BTreeMap<&'static str, ToolPolicy>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: BTreeMap::new(),
            policies: BTreeMap::new(),
        }
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
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
