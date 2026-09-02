//! Agent-core — modelo Agent/Skill para Redox AIOS (userspace, std).
//! Portado de neural-os-core; tipos e traits compartilhados entre daemons.

pub mod aios_registry;
pub mod backend;
pub mod lifecycle;
pub mod permission_gate;
pub mod scheme_caps;
pub mod types;

pub use aios_registry::{register_agent, register_fleet, AgentRegistration};
pub use backend::{collect_stack_backends, probe_tcp, BackendReport, BackendTier};
pub use lifecycle::{lifecycle_agent_names, AutoLearnAgent, OptimizerAgent, SleepCycleAgent};
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
