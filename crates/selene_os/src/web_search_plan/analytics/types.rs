#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::StructuredRow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ANALYTICS_SCHEMA_VERSION: &str = "1.0.0";
pub const ANALYTICS_ENGINE_ID: &str = "PH1.ANALYTICS";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateMethod {
    Mean,
    Median,
    TrimmedMean,
    WeightedMean,
    Min,
    Max,
    P25,
    P50,
    P75,
    Stddev,
}

impl AggregateMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::Median => "median",
            Self::TrimmedMean => "trimmed_mean",
            Self::WeightedMean => "weighted_mean",
            Self::Min => "min",
            Self::Max => "max",
            Self::P25 => "p25",
            Self::P50 => "p50",
            Self::P75 => "p75",
            Self::Stddev => "stddev",
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputationInputs {
    pub evidence_hash: String,
    pub policy_snapshot_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NumericValue {
    Int { value: i64 },
    Decimal { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateWindow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    pub metric_id: String,
    pub entity: String,
    pub attribute: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<AggregateWindow>,
    pub method: AggregateMethod,
    pub value: NumericValue,
    pub sample_size: u32,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusCandidate {
    pub value: NumericValue,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusOutlier {
    pub value: NumericValue,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusGroup {
    pub group_id: String,
    pub topic: String,
    pub candidates: Vec<ConsensusCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen: Option<NumericValue>,
    pub agreement_score: String,
    pub outliers: Vec<ConsensusOutlier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfidenceFactors {
    pub sample_size: u32,
    pub trust_tier_mix: String,
    pub recency: String,
    pub conflict: String,
    pub outliers: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfidenceItem {
    pub claim_key: String,
    pub confidence_score: String,
    pub factors: ConfidenceFactors,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputationPacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub inputs: ComputationInputs,
    pub aggregates: Vec<Aggregate>,
    pub consensus: Vec<ConsensusGroup>,
    pub confidence: Vec<ConfidenceItem>,
    pub reason_codes: Vec<String>,
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
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusBuildResult {
    pub groups: Vec<ConsensusGroup>,
    pub reason_codes: Vec<String>,
}
