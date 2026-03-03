#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::types::ComputationPacket;
use crate::web_search_plan::structured::types::StructuredRow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const COMPETITIVE_SCHEMA_VERSION: &str = "1.0.0";
pub const COMPETITIVE_ENGINE_ID: &str = "PH1.ANALYTICS";
pub const COMPETITIVE_NORM_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitiveErrorKind {
    InsufficientEvidence,
    PolicyViolation,
    CitationMismatch,
}

impl CompetitiveErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::PolicyViolation => "policy_violation",
            Self::CitationMismatch => "citation_mismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetitiveError {
    pub kind: CompetitiveErrorKind,
    pub message: String,
}

impl CompetitiveError {
    pub fn new(kind: CompetitiveErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingPeriod {
    Monthly,
    Yearly,
    OneTime,
    UsageBased,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriState {
    True,
    False,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionClass {
    WithinMarket,
    UpperTier,
    Premium,
    Outlier,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitiveEntity {
    pub entity_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_ms: Option<i64>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitivePrice {
    pub entity_id: String,
    pub price_value: String,
    pub currency: String,
    pub billing_period: BillingPeriod,
    pub tax_included: TriState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_to: Option<String>,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitiveFeature {
    pub entity_id: String,
    pub feature_key: String,
    pub feature_label: String,
    pub presence: TriState,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionSummary {
    pub classification: PositionClass,
    pub basis_metric: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparator_value: Option<String>,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwotBullet {
    pub text: String,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitiveSwot {
    pub strengths: Vec<SwotBullet>,
    pub weaknesses: Vec<SwotBullet>,
    pub opportunities: Vec<SwotBullet>,
    pub threats: Vec<SwotBullet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitiveComparison {
    pub target_entity: String,
    pub competitors: Vec<CompetitiveEntity>,
    pub pricing_table: Vec<CompetitivePrice>,
    pub feature_matrix: Vec<CompetitiveFeature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_summary: Option<PositionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swot: Option<CompetitiveSwot>,
    pub uncertainty_flags: Vec<String>,
    pub reason_codes: Vec<String>,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonPacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub target_entity: String,
    pub competitors: Vec<CompetitiveEntity>,
    pub pricing_table: Vec<CompetitivePrice>,
    pub feature_matrix: Vec<CompetitiveFeature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_summary: Option<PositionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swot: Option<CompetitiveSwot>,
    pub uncertainty_flags: Vec<String>,
    pub reason_codes: Vec<String>,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompetitiveRequest {
    pub trace_id: String,
    pub created_at_ms: i64,
    pub intended_consumers: Vec<String>,
    pub target_entity: String,
    pub evidence_packet: Value,
    pub structured_rows: Vec<StructuredRow>,
    pub computation_packet: Option<ComputationPacket>,
}
