//! Agent-core — modelo Agent/Skill para Redox AIOS (userspace, std).
//! Portado de neural-os-core; tipos e traits compartilhados entre daemons.

pub mod aios_registry;
pub mod backend;
pub mod lifecycle;
pub mod os_caps;
pub mod ota;
pub mod permission_gate;
pub mod redox_caps;
pub mod scheme_caps;
pub mod self_heal;
pub mod types;

pub use aios_registry::{register_agent, register_fleet, AgentRegistration};
pub use backend::{collect_stack_backends, probe_tcp, BackendReport, BackendTier};
pub use lifecycle::{
    lifecycle_agent_names, AutoLearnAgent, LifecycleTick, OptimizerAgent, SelfHealAgent,
    SleepCycleAgent,
};
pub use os_caps::{
    bootstrap_caps, grant_active_os, load_cap_store, refresh_caps_cache, save_cap_store, CapStore,
    CapToken,
};
pub use ota::{apply_update, check_update, OtaChannel, OtaProposal};
pub use redox_caps::{
    bootstrap_redox_ns, build_namespace, probe_namespace, redox_caps_summary, scheme_allowed,
    CapBackend, NamespaceProfile, SchemeProbe,
};
pub use self_heal::{scan_stack, HealIssue, HealReport, HealSeverity};
pub use permission_gate::{
    gate_enabled, impact_level, missing_pkg_grant, missing_scheme_grant, requires_hitl,
    required_grant, ImpactLevel,
};
pub use scheme_caps::{
    active_grants, allows_pkg_install, cap_summary, grant_active, wasm_caps_from_grants,
    GRANT_HITL, GRANT_PKG_INSTALL, SCHEME_AIOS, SCHEME_MEMORY, SCHEME_NET,
};
pub use types::{
    Agent, AgentKind, AgentManifest, AgentState, AgentTickResult, FlowTrigger, ScheduleKind,
};
