//! Tipos fundamentais do modelo Agent/Skill (neural-os-core compat).

use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    System,
    Driver,
    Inference,
    Router,
    Console,
    Network,
    Skill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleKind {
    Oneshot,
    Continuous,
    PollEvery(u64),
    EventDriven,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlowTrigger {
    Schedule(ScheduleKind),
    Start,
    Listen(String),
    Router(String),
}

#[derive(Clone, Debug)]
pub struct AgentManifest {
    pub name: &'static str,
    pub kind: AgentKind,
    pub schedule: ScheduleKind,
    pub auto_start: bool,
    pub persist: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTickResult {
    Pending,
    Done,
    Crashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Inactive,
    Active,
    Done,
    Crashed,
}

/// Trait executado por cada daemon-agente userspace.
pub trait Agent: Send {
    fn manifest(&self) -> &AgentManifest;
    fn tick(&mut self) -> AgentTickResult;
    fn poll_interval(&self) -> Duration {
        Duration::from_millis(100)
    }
}
