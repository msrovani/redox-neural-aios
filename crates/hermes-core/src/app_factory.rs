//! Runtime App Factory — Caminho A (wasmi) userspace (ADR-010).

use agent_core::permission_gate::{gate_enabled, impact_level, ImpactLevel};
use i18n_core::t;
use memory_core::MemoryBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactoryStage {
    Ephemeral,
    SkillMd,
    WasmMemory,
    WasmPersistent,
    AppRecipe,
}

impl FactoryStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ephemeral => "ephemeral",
            Self::SkillMd => "skill_md",
            Self::WasmMemory => "wasm_memory",
            Self::WasmPersistent => "wasm_file",
            Self::AppRecipe => "app_recipe",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FactoryBackendReport {
    pub hitl_enabled: bool,
    pub tools_net: bool,
    pub factory_caps: bool,
    pub memory_backend: String,
    pub path_b_gated: bool,
}

pub fn factory_backend_report() -> FactoryBackendReport {
    FactoryBackendReport {
        hitl_enabled: gate_enabled(),
        tools_net: crate::tools::tools_net_enabled(),
        factory_caps: skill_registry::factory_caps_enabled(),
        memory_backend: format!("{:?}", MemoryBackend::from_env()),
        path_b_gated: true,
    }
}

pub fn format_ladder_help() -> String {
    t("factory.ladder.help")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppBackend {
    WasmInterp,
    WasmJit,
    NativeRustSubset,
}

#[derive(Clone, Debug)]
pub struct AppRequest {
    pub desc: String,
    pub trusted: bool,
    pub perf_critical: bool,
}

#[derive(Clone, Debug)]
pub struct Recommendation {
    pub backend: AppBackend,
    pub rationale: String,
    pub requires_hitl: bool,
    pub degraded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactoryOutcome {
    RanWasm(i32),
    Denied(String),
    AwaitingIsolation(AppBackend),
}

pub fn analyze_and_recommend(req: &AppRequest) -> Recommendation {
    if !req.trusted {
        return Recommendation {
            backend: AppBackend::WasmInterp,
            rationale: "código não-confiável → wasmi sandbox (Caminho A)".into(),
            requires_hitl: impact_level(&req.desc) != ImpactLevel::Low,
            degraded: false,
        };
    }
    if req.perf_critical {
        return Recommendation {
            backend: AppBackend::WasmJit,
            rationale: "perf crítica — Caminho B gated até isolamento Ring3".into(),
            requires_hitl: true,
            degraded: true,
        };
    }
    Recommendation {
        backend: AppBackend::WasmInterp,
        rationale: "default seguro wasmi".into(),
        requires_hitl: false,
        degraded: false,
    }
}

pub fn execute_path_a(wasm: &[u8], export_fn: &str) -> FactoryOutcome {
    if gate_enabled() && impact_level("factory execute") == ImpactLevel::Critical {
        return FactoryOutcome::Denied("HITL blocked factory".into());
    }
    match wasm_skill_runtime::run_i32_0(wasm, export_fn, wasm_skill_runtime::CAP_LOG) {
        Ok(v) => FactoryOutcome::RanWasm(v),
        Err(e) => FactoryOutcome::Denied(e.0),
    }
}

pub fn execute_recommended(rec: &Recommendation, wasm: &[u8], export_fn: &str) -> FactoryOutcome {
    match rec.backend {
        AppBackend::WasmInterp => execute_path_a(wasm, export_fn),
        AppBackend::WasmJit | AppBackend::NativeRustSubset => {
            FactoryOutcome::AwaitingIsolation(rec.backend.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_skill_runtime::echo_len_module;

    #[test]
    fn untrusted_recommends_wasmi() {
        let rec = analyze_and_recommend(&AppRequest {
            desc: "user script".into(),
            trusted: false,
            perf_critical: false,
        });
        assert_eq!(rec.backend, AppBackend::WasmInterp);
    }

    #[test]
    fn path_a_runs_wasm() {
        let rec = analyze_and_recommend(&AppRequest {
            desc: "test".into(),
            trusted: false,
            perf_critical: false,
        });
        match execute_recommended(&rec, &echo_len_module(), "run") {
            FactoryOutcome::RanWasm(v) => assert_eq!(v, 42),
            other => panic!("unexpected {other:?}"),
        }
    }
}
