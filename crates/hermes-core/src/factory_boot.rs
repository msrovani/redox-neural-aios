//! Boot da Runtime App Factory — reload skills persistidas + relatório honesto (ADR-001/010).

use skill_registry::{load_persisted_skills, BootLoadReport, SkillRegistry};

use crate::app_factory::{factory_backend_report, FactoryBackendReport};
use crate::skills::register_builtin_skills;

#[derive(Clone, Debug)]
pub struct FactoryBootReport {
    pub load: BootLoadReport,
    pub backends: FactoryBackendReport,
    pub skills_dir: String,
}

/// Registro Hermes pronto para boot AIOS: builtins + skills persistidas em `/var/lib/aios/skills`.
pub fn boot_skill_registry() -> (SkillRegistry, FactoryBootReport) {
    let mut registry = SkillRegistry::new();
    register_builtin_skills(&mut registry);
    let load = load_persisted_skills(&mut registry);
    let backends = factory_backend_report();
    let skills_dir = skill_registry::skills_dir().display().to_string();
    (
        registry,
        FactoryBootReport {
            load,
            backends,
            skills_dir,
        },
    )
}

pub fn format_boot_report(report: &FactoryBootReport) -> String {
    format!(
        "factory_boot skills_dir={} md={} wasm={} net={} hitl={} caps={}{}",
        report.skills_dir,
        report.load.skill_md_loaded,
        report.load.wasm_loaded,
        if report.backends.tools_net { "on" } else { "off" },
        if report.backends.hitl_enabled { "on" } else { "off" },
        if report.backends.factory_caps { "on" } else { "off" },
        if report.load.warnings.is_empty() {
            String::new()
        } else {
            format!(" warnings={}", report.load.warnings.len())
        }
    )
}
