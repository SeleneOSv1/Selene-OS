#![forbid(unsafe_code)]

pub mod apply;
pub mod domain_rules;
pub mod explain;
pub mod official_detector;
pub mod spam_signals;
pub mod trust_score;

pub use apply::{enrich_evidence_sources, SourceEnrichment, TrustApplyResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustErrorKind {
    PolicyViolation,
    InsufficientEvidence,
    ProviderUpstreamFailed,
}

impl TrustErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PolicyViolation => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustError {
    pub kind: TrustErrorKind,
    pub message: String,
}

impl TrustError {
    pub fn new(kind: TrustErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[cfg(test)]
pub mod trust_tests;
