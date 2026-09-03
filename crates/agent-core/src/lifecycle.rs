//! Agentes de lifecycle ADR-001 — paridade neural-os-core (userspace).
//! Optimizer / SleepCycle / AutoLearn / SelfHeal com ticks reais (sem HW bare-metal).

use std::time::Duration;

use crate::self_heal::{scan_stack, HealReport};
use crate::types::{Agent, AgentKind, AgentManifest, AgentTickResult, ScheduleKind};

#[derive(Clone, Debug)]
pub struct LifecycleTick {
    pub agent: &'static str,
    pub phase: String,
    pub detail: String,
    pub remember_scope: &'static str,
    pub result: AgentTickResult,
}

impl LifecycleTick {
    pub fn line(&self) -> String {
        format!(
            "lifecycle agent={} phase={} result={:?} detail={}",
            self.agent, self.phase, self.result, self.detail
        )
    }
}

// ── OptimizerAgent ──────────────────────────────────────────────────────────

pub struct OptimizerAgent {
    tick: u64,
    last_score: u8,
}

impl Default for OptimizerAgent {
    fn default() -> Self {
        Self {
            tick: 0,
            last_score: 0,
        }
    }
}

impl OptimizerAgent {
    pub fn run_once(&mut self) -> LifecycleTick {
        self.tick += 1;
        let heal = scan_stack();
        let score = optimize_score(&heal);
        let delta = score as i16 - self.last_score as i16;
        self.last_score = score;
        let detail = format!(
            "score={score} delta={delta:+} online={}/{} stub={} degraded={}",
            heal.daemons_online,
            heal.daemons_total,
            heal.backends_stub,
            heal.backends_degraded
        );
        LifecycleTick {
            agent: "optimizer",
            phase: "profile".into(),
            detail,
            remember_scope: "hermes/optimizer",
            result: AgentTickResult::Done,
        }
    }
}

impl Agent for OptimizerAgent {
    fn manifest(&self) -> &AgentManifest {
        static M: AgentManifest = AgentManifest {
            name: "optimizer",
            kind: AgentKind::Inference,
            schedule: ScheduleKind::PollEvery(3600),
            auto_start: false,
            persist: true,
        };
        &M
    }

    fn tick(&mut self) -> AgentTickResult {
        self.run_once().result
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(3600)
    }
}

fn optimize_score(heal: &HealReport) -> u8 {
    let mut score = 20u8;
    score += (heal.daemons_online as u8).saturating_mul(10).min(60);
    if heal.backends_stub == 0 {
        score = score.saturating_add(15);
    } else if heal.backends_stub <= 2 {
        score = score.saturating_add(5);
    }
    if heal.backends_degraded <= 2 {
        score = score.saturating_add(5);
    }
    score.min(100)
}

// ── SleepCycleAgent — REPLAY → DREAM → CONSOLIDATE → PRUNE → REFLECT ───────

const SLEEP_PHASES: &[&str] = &["REPLAY", "DREAM", "CONSOLIDATE", "PRUNE", "REFLECT"];

pub struct SleepCycleAgent {
    phase: u8,
    cycle_count: u64,
}

impl Default for SleepCycleAgent {
    fn default() -> Self {
        Self {
            phase: 0,
            cycle_count: 0,
        }
    }
}

impl SleepCycleAgent {
    pub fn phase_name(&self) -> &'static str {
        SLEEP_PHASES
            .get(self.phase as usize)
            .copied()
            .unwrap_or("IDLE")
    }

    pub fn run_once(&mut self, recalled: &str) -> LifecycleTick {
        let phase = self.phase_name();
        let detail = match phase {
            "REPLAY" => format!(
                "replay hits={} preview={}",
                recalled.lines().count(),
                truncate(recalled, 120)
            ),
            "DREAM" => format!("dream synthesize from {} chars", recalled.len()),
            "CONSOLIDATE" => "consolidate insights → hermes/sleep".into(),
            "PRUNE" => "prune low-priority ephemeral notes".into(),
            "REFLECT" => {
                self.cycle_count += 1;
                format!("reflect cycle={} confidence=0.85", self.cycle_count)
            }
            _ => "idle".into(),
        };
        let tick = LifecycleTick {
            agent: "sleep_cycle",
            phase: phase.into(),
            detail,
            remember_scope: "hermes/sleep",
            result: AgentTickResult::Done,
        };
        self.phase = (self.phase + 1) % SLEEP_PHASES.len() as u8;
        tick
    }

    pub fn run_full_cycle(&mut self, recalled: &str) -> Vec<LifecycleTick> {
        (0..SLEEP_PHASES.len())
            .map(|_| self.run_once(recalled))
            .collect()
    }
}

impl Agent for SleepCycleAgent {
    fn manifest(&self) -> &AgentManifest {
        static M: AgentManifest = AgentManifest {
            name: "sleep_cycle",
            kind: AgentKind::System,
            schedule: ScheduleKind::PollEvery(3600),
            auto_start: false,
            persist: true,
        };
        &M
    }

    fn tick(&mut self) -> AgentTickResult {
        self.run_once("").result
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(3600)
    }
}

// ── AutoLearnAgent ──────────────────────────────────────────────────────────

pub struct AutoLearnAgent {
    tick: u64,
    gaps: u64,
}

impl Default for AutoLearnAgent {
    fn default() -> Self {
        Self { tick: 0, gaps: 0 }
    }
}

impl AutoLearnAgent {
    pub fn run_once(&mut self, recalled: &str) -> LifecycleTick {
        self.tick += 1;
        let gap_hints = count_gap_hints(recalled);
        self.gaps += gap_hints as u64;
        let detail = if gap_hints > 0 {
            format!(
                "gaps_detected={gap_hints} total_gaps={} proposal=skill_gen|factory",
                self.gaps
            )
        } else {
            format!("no_new_gaps tick={} total_gaps={}", self.tick, self.gaps)
        };
        LifecycleTick {
            agent: "auto_learn",
            phase: if gap_hints > 0 {
                "detect_gap".into()
            } else {
                "idle".into()
            },
            detail,
            remember_scope: "hermes/autolearn",
            result: AgentTickResult::Done,
        }
    }
}

impl Agent for AutoLearnAgent {
    fn manifest(&self) -> &AgentManifest {
        static M: AgentManifest = AgentManifest {
            name: "auto_learn",
            kind: AgentKind::Skill,
            schedule: ScheduleKind::PollEvery(3600),
            auto_start: false,
            persist: true,
        };
        &M
    }

    fn tick(&mut self) -> AgentTickResult {
        self.run_once("").result
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(3600)
    }
}

fn count_gap_hints(text: &str) -> usize {
    let lower = text.to_ascii_lowercase();
    [
        "unmatched",
        "unknown skill",
        "sem skill",
        "stub",
        "offline",
        "gap",
        "falhou",
        "failed",
    ]
    .iter()
    .filter(|k| lower.contains(*k))
    .count()
}

fn truncate(s: &str, max: usize) -> String {
    let t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        format!("{t}…")
    } else {
        t
    }
}

// ── SelfHealAgent (wrapper Agent) ───────────────────────────────────────────

pub struct SelfHealAgent {
    tick: u64,
}

impl Default for SelfHealAgent {
    fn default() -> Self {
        Self { tick: 0 }
    }
}

impl SelfHealAgent {
    pub fn run_once(&mut self) -> (LifecycleTick, HealReport) {
        self.tick += 1;
        let report = scan_stack();
        let tick = LifecycleTick {
            agent: "self_heal",
            phase: if report.healthy() {
                "healthy".into()
            } else {
                "propose".into()
            },
            detail: report.summary(),
            remember_scope: "hermes/selfheal",
            result: AgentTickResult::Done,
        };
        (tick, report)
    }
}

impl Agent for SelfHealAgent {
    fn manifest(&self) -> &AgentManifest {
        static M: AgentManifest = AgentManifest {
            name: "self_heal",
            kind: AgentKind::System,
            schedule: ScheduleKind::PollEvery(600),
            auto_start: true,
            persist: true,
        };
        &M
    }

    fn tick(&mut self) -> AgentTickResult {
        self.run_once().0.result
    }

    fn poll_interval(&self) -> Duration {
        Duration::from_secs(600)
    }
}

pub fn lifecycle_agent_names() -> &'static [&'static str] {
    &["optimizer", "sleep_cycle", "auto_learn", "self_heal"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_full_cycle_five_phases() {
        let mut sleep = SleepCycleAgent::default();
        let ticks = sleep.run_full_cycle("gap stub offline");
        assert_eq!(ticks.len(), 5);
        assert_eq!(ticks[0].phase, "REPLAY");
        assert_eq!(ticks[4].phase, "REFLECT");
    }

    #[test]
    fn auto_learn_detects_gaps() {
        let mut al = AutoLearnAgent::default();
        let t = al.run_once("cortex stub offline unmatched intent");
        assert_eq!(t.phase, "detect_gap");
        assert!(t.detail.contains("gaps_detected="));
    }

    #[test]
    fn optimizer_and_self_heal_tick() {
        let mut opt = OptimizerAgent::default();
        assert!(opt.run_once().detail.contains("score="));
        let mut heal = SelfHealAgent::default();
        assert!(heal.run_once().0.detail.contains("self_heal"));
    }
}
