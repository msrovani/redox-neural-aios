//! SelfHeal userspace — paridade conceitual neural-os-core (ADR-001 mand. 2).
//! Detecta gaps de daemons/backends e propõe cura; sem PCI/firmware bare-metal.

use crate::backend::{collect_stack_backends, probe_tcp, BackendReport, BackendTier};

const DAEMON_SOCKETS: &[(&str, &str)] = &[
    ("eventd", "127.0.0.1:7740"),
    ("sgdbd", "127.0.0.1:7741"),
    ("hermesd", "127.0.0.1:7742"),
    ("cortexd", "127.0.0.1:7743"),
    ("voiced", "127.0.0.1:7744"),
    ("jarbasd", "127.0.0.1:7745"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealSeverity {
    Info,
    Warn,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealIssue {
    pub component: String,
    pub severity: HealSeverity,
    pub detail: String,
    pub proposal: String,
}

#[derive(Clone, Debug)]
pub struct HealReport {
    pub issues: Vec<HealIssue>,
    pub daemons_online: usize,
    pub daemons_total: usize,
    pub backends_stub: usize,
    pub backends_degraded: usize,
}

impl HealReport {
    pub fn healthy(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| matches!(i.severity, HealSeverity::Critical | HealSeverity::Warn))
    }

    pub fn summary(&self) -> String {
        format!(
            "self_heal online={}/{} stub={} degraded={} issues={}",
            self.daemons_online,
            self.daemons_total,
            self.backends_stub,
            self.backends_degraded,
            self.issues.len()
        )
    }

    pub fn format(&self) -> String {
        let mut out = String::from("=== SELF HEAL ===\n");
        out.push_str(&self.summary());
        out.push('\n');
        if self.issues.is_empty() {
            out.push_str("status=healthy\n");
        } else {
            for issue in &self.issues {
                out.push_str(&format!(
                    "[{:?}] {} — {} → {}\n",
                    issue.severity, issue.component, issue.detail, issue.proposal
                ));
            }
        }
        out.push_str("=== END SELF HEAL ===");
        out
    }
}

/// Escaneia stack host e produz relatório de cura (HITL — só propõe).
pub fn scan_stack() -> HealReport {
    let mut issues = Vec::new();
    let mut online = 0usize;

    for (name, addr) in DAEMON_SOCKETS {
        if probe_tcp(addr, 200) {
            online += 1;
        } else {
            issues.push(HealIssue {
                component: (*name).into(),
                severity: HealSeverity::Warn,
                detail: format!("daemon offline em {addr}"),
                proposal: format!("reiniciar {name} (init.d / start-stack)"),
            });
        }
    }

    let backends = collect_stack_backends();
    let mut stub = 0usize;
    let mut degraded = 0usize;
    for b in &backends {
        match b.tier {
            BackendTier::Stub => {
                stub += 1;
                issues.push(HealIssue {
                    component: b.component.clone(),
                    severity: HealSeverity::Info,
                    detail: format!("backend stub: {}", b.detail),
                    proposal: "habilitar engine produção ou documentar gap".into(),
                });
            }
            BackendTier::Degraded => {
                degraded += 1;
                issues.push(HealIssue {
                    component: b.component.clone(),
                    severity: HealSeverity::Info,
                    detail: format!("backend degradado: {}", b.detail),
                    proposal: "avançar scheme nativo / engine real".into(),
                });
            }
            BackendTier::Production => {}
        }
    }

    if online == 0 {
        issues.push(HealIssue {
            component: "stack".into(),
            severity: HealSeverity::Critical,
            detail: "nenhum daemon cognitivo online".into(),
            proposal: "rodar tools/start-stack.ps1 ou boot QEMU aios-minimal".into(),
        });
    }

    HealReport {
        issues,
        daemons_online: online,
        daemons_total: DAEMON_SOCKETS.len(),
        backends_stub: stub,
        backends_degraded: degraded,
    }
}

pub fn backend_snapshot() -> Vec<BackendReport> {
    collect_stack_backends()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_produces_report() {
        let report = scan_stack();
        assert_eq!(report.daemons_total, 6);
        assert!(report.format().contains("SELF HEAL"));
    }
}
