#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::StructuredRow;
use selene_kernel_contracts::ph1comp::{PH1COMP_ENGINE_ID, PH1COMP_SCHEMA_VERSION};
use serde_json::Value;

pub const ANALYTICS_SCHEMA_VERSION: &str = PH1COMP_SCHEMA_VERSION;
pub const ANALYTICS_ENGINE_ID: &str = PH1COMP_ENGINE_ID;

pub use selene_kernel_contracts::ph1comp::{
    Aggregate, AggregateMethod, AggregateWindow, ComputationConfidenceBucket,
    ComputationInputs, ComputationPacket, ConsensusCandidate, ConsensusGroup, ConsensusMethod,
    ConsensusOutlier, ConfidenceFactors, ConfidenceItem, NormalizationKind,
    NormalizationTraceEntry, NumericValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticsErrorKind {
    InvalidInput,
    InsufficientEvidence,
    Conflict,
    UnitMismatch,
    PolicyViolation,
    CitationMismatch,
}

impl AnalyticsErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidInput => "policy_violation",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::Conflict => "conflicting_evidence_detected",
            Self::UnitMismatch => "policy_violation",
            Self::PolicyViolation => "policy_violation",
            Self::CitationMismatch => "citation_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsError {
    pub kind: AnalyticsErrorKind,
    pub message: String,
}

impl AnalyticsError {
    pub fn new(kind: AnalyticsErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone)]
pub struct AnalyticsRequest {
    pub trace_id: String,
    pub created_at_ms: i64,
    pub intended_consumers: Vec<String>,
    pub policy_snapshot_id: String,
    pub as_of_ms: Option<i64>,
    pub evidence_packet: Value,
    pub structured_rows: Vec<StructuredRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericSample {
    pub metric_id: String,
    pub entity: String,
    pub attribute: String,
    pub unit: Option<String>,
    pub currency: Option<String>,
    pub as_of_ms: Option<i64>,
    pub source_ref: String,
    pub source_url: String,
    pub value_repr: NumericValue,
    pub trust_weight: u32,
    pub value_decimal: rust_decimal::Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateGroup {
    pub metric_id: String,
    pub entity: String,
    pub attribute: String,
    pub unit: Option<String>,
    pub currency: Option<String>,
    pub as_of_ms: Option<i64>,
    pub samples: Vec<NumericSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateBuildResult {
    pub aggregates: Vec<Aggregate>,
    pub groups: Vec<AggregateGroup>,
    pub normalization_trace: Vec<NormalizationTraceEntry>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusBuildResult {
    pub groups: Vec<ConsensusGroup>,
    pub reason_codes: Vec<String>,
}
