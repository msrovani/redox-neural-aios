//! Agent-core — modelo Agent/Skill para Redox AIOS (userspace, std).
//! Portado de neural-os-core; tipos e traits compartilhados entre daemons.

pub mod types;

pub use types::{
    Agent, AgentKind, AgentManifest, AgentState, AgentTickResult, FlowTrigger, ScheduleKind,
};
