//! ReAct 7 fases — rastreamento de ciclo cognitivo (simplificado).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReActPhase {
    Observe,
    Think,
    Plan,
    Execute,
    Verify,
    Learn,
}

impl ReActPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Observe => "OBSERVE",
            Self::Think => "THINK",
            Self::Plan => "PLAN",
            Self::Execute => "EXECUTE",
            Self::Verify => "VERIFY",
            Self::Learn => "LEARN",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Observe => Self::Think,
            Self::Think => Self::Plan,
            Self::Plan => Self::Execute,
            Self::Execute => Self::Verify,
            Self::Verify => Self::Learn,
            Self::Learn => Self::Observe,
        }
    }
}

pub struct ReActTrace {
    pub phase: ReActPhase,
    pub detail: String,
}

impl ReActTrace {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            phase: ReActPhase::Observe,
            detail: detail.into(),
        }
    }

    pub fn advance(&mut self, detail: impl Into<String>) {
        self.phase = self.phase.next();
        self.detail = detail.into();
    }

    pub fn line(&self) -> String {
        format!("[ReAct:{}] {}", self.phase.label(), self.detail)
    }
}
