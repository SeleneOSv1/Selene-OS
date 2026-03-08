#![forbid(unsafe_code)]

use crate::{ContractViolation, Validate};
use serde::{Deserialize, Serialize};

pub const PH1COMP_SCHEMA_VERSION: &str = "1.0.0";
pub const PH1COMP_ENGINE_ID: &str = "PH1.COMP";

fn validate_ascii_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_optional_ascii_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value.as_ref() {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

fn validate_ascii_token_vec(
    field: &'static str,
    values: &[String],
    max_len: usize,
    max_items: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too many entries",
        });
    }
    for value in values {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusMethod {
    Majority,
    Weighted,
    Threshold,
}

impl ConsensusMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Majority => "majority",
            Self::Weighted => "weighted",
            Self::Threshold => "threshold",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputationFailureClass {
    NormalizationFailure,
    ComputationOverflow,
    InvalidInputSet,
    ConsensusUnresolved,
    ConfidenceBelowThreshold,
    BudgetComputationFailure,
    OutlierHandlingFailure,
}

impl ComputationFailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NormalizationFailure => "NORMALIZATION_FAILURE",
            Self::ComputationOverflow => "COMPUTATION_OVERFLOW",
            Self::InvalidInputSet => "INVALID_INPUT_SET",
            Self::ConsensusUnresolved => "CONSENSUS_UNRESOLVED",
            Self::ConfidenceBelowThreshold => "CONFIDENCE_BELOW_THRESHOLD",
            Self::BudgetComputationFailure => "BUDGET_COMPUTATION_FAILURE",
            Self::OutlierHandlingFailure => "OUTLIER_HANDLING_FAILURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputationConfidenceBucket {
    High,
    Medium,
    Low,
    Insufficient,
}

impl ComputationConfidenceBucket {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::Insufficient => "INSUFFICIENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComputationConsensusStatus {
    NotRequested,
    MajorityReached,
    WeightedConsensusReached,
    ThresholdConsensusReached,
    Unresolved,
}

impl ComputationConsensusStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "NOT_REQUESTED",
            Self::MajorityReached => "MAJORITY_REACHED",
            Self::WeightedConsensusReached => "WEIGHTED_CONSENSUS_REACHED",
            Self::ThresholdConsensusReached => "THRESHOLD_CONSENSUS_REACHED",
            Self::Unresolved => "UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NormalizationKind {
    Currency,
    Unit,
    Time,
    Percentage,
    Scale,
}

impl NormalizationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Currency => "CURRENCY",
            Self::Unit => "UNIT",
            Self::Time => "TIME",
            Self::Percentage => "PERCENTAGE",
            Self::Scale => "SCALE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputationInputs {
    pub evidence_hash: String,
    pub policy_snapshot_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of_ms: Option<i64>,
    #[serde(default)]
    pub input_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub normalization_trace: Vec<NormalizationTraceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formula_version_refs: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_met: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_score: Option<NumericValue>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_method: Option<ConsensusMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_threshold_met: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_result_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_resolution_rationale: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<ComputationConfidenceBucket>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_threshold_met: Option<bool>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationTraceEntry {
    pub normalization_kind: NormalizationKind,
    pub rule_id: String,
    pub input_label: String,
    pub source_value: String,
    pub normalized_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_unit: Option<String>,
    pub applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationConfidencePosture {
    pub confidence_score: Option<String>,
    pub bucket: Option<ComputationConfidenceBucket>,
    pub minimum_threshold_met: bool,
}

impl ComputationConfidencePosture {
    pub fn v1(
        confidence_score: Option<String>,
        bucket: Option<ComputationConfidenceBucket>,
        minimum_threshold_met: bool,
    ) -> Result<Self, ContractViolation> {
        let posture = Self {
            confidence_score,
            bucket,
            minimum_threshold_met,
        };
        posture.validate()?;
        Ok(posture)
    }
}

impl Validate for ComputationConfidencePosture {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_ascii_token(
            "computation_confidence_posture.confidence_score",
            &self.confidence_score,
            32,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationSelectedResult {
    pub result_id: String,
    pub numeric_value: NumericValue,
    pub rank: Option<u16>,
    pub reason_code: Option<String>,
}

impl ComputationSelectedResult {
    pub fn v1(
        result_id: String,
        numeric_value: NumericValue,
        rank: Option<u16>,
        reason_code: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let result = Self {
            result_id,
            numeric_value,
            rank,
            reason_code,
        };
        result.validate()?;
        Ok(result)
    }
}

impl Validate for ComputationSelectedResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "computation_selected_result.result_id",
            &self.result_id,
            256,
        )?;
        validate_optional_ascii_token(
            "computation_selected_result.reason_code",
            &self.reason_code,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationConsensusResult {
    pub status: ComputationConsensusStatus,
    pub agreement_score: Option<String>,
    pub outlier_count: u16,
    pub conflict_resolution_rationale: Option<String>,
}

impl ComputationConsensusResult {
    pub fn v1(
        status: ComputationConsensusStatus,
        agreement_score: Option<String>,
        outlier_count: u16,
        conflict_resolution_rationale: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let result = Self {
            status,
            agreement_score,
            outlier_count,
            conflict_resolution_rationale,
        };
        result.validate()?;
        Ok(result)
    }
}

impl Validate for ComputationConsensusResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_ascii_token(
            "computation_consensus_result.agreement_score",
            &self.agreement_score,
            32,
        )?;
        validate_optional_ascii_token(
            "computation_consensus_result.conflict_resolution_rationale",
            &self.conflict_resolution_rationale,
            256,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationExecutionState {
    pub computation_packet_ref: Option<String>,
    pub normalization_trace: Vec<NormalizationTraceEntry>,
    pub consensus_result: Option<ComputationConsensusResult>,
    pub selected_result: Option<ComputationSelectedResult>,
    pub confidence_posture: Option<ComputationConfidencePosture>,
    pub failure_class: Option<ComputationFailureClass>,
    pub formula_version_refs: Vec<String>,
    pub reason_codes: Vec<String>,
}

impl ComputationExecutionState {
    pub fn v1(
        computation_packet_ref: Option<String>,
        normalization_trace: Vec<NormalizationTraceEntry>,
        consensus_result: Option<ComputationConsensusResult>,
        selected_result: Option<ComputationSelectedResult>,
        confidence_posture: Option<ComputationConfidencePosture>,
        failure_class: Option<ComputationFailureClass>,
        formula_version_refs: Vec<String>,
        reason_codes: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            computation_packet_ref,
            normalization_trace,
            consensus_result,
            selected_result,
            confidence_posture,
            failure_class,
            formula_version_refs,
            reason_codes,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for ComputationExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_ascii_token(
            "computation_execution_state.computation_packet_ref",
            &self.computation_packet_ref,
            128,
        )?;
        for trace in &self.normalization_trace {
            trace.validate()?;
        }
        if let Some(result) = self.consensus_result.as_ref() {
            result.validate()?;
        }
        if let Some(result) = self.selected_result.as_ref() {
            result.validate()?;
        }
        if let Some(posture) = self.confidence_posture.as_ref() {
            posture.validate()?;
        }
        validate_ascii_token_vec(
            "computation_execution_state.formula_version_refs",
            &self.formula_version_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "computation_execution_state.reason_codes",
            &self.reason_codes,
            128,
            32,
        )?;
        Ok(())
    }
}

impl Validate for NormalizationTraceEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "normalization_trace_entry.rule_id",
            &self.rule_id,
            128,
        )?;
        validate_ascii_token(
            "normalization_trace_entry.input_label",
            &self.input_label,
            128,
        )?;
        validate_ascii_token(
            "normalization_trace_entry.source_value",
            &self.source_value,
            64,
        )?;
        validate_ascii_token(
            "normalization_trace_entry.normalized_value",
            &self.normalized_value,
            64,
        )?;
        validate_optional_ascii_token(
            "normalization_trace_entry.source_unit",
            &self.source_unit,
            32,
        )?;
        validate_optional_ascii_token(
            "normalization_trace_entry.canonical_unit",
            &self.canonical_unit,
            32,
        )?;
        Ok(())
    }
}

impl Validate for ComputationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "computation_packet.schema_version",
            &self.schema_version,
            16,
        )?;
        validate_ascii_token(
            "computation_packet.produced_by",
            &self.produced_by,
            64,
        )?;
        validate_ascii_token_vec(
            "computation_packet.intended_consumers",
            &self.intended_consumers,
            64,
            32,
        )?;
        validate_ascii_token("computation_packet.trace_id", &self.trace_id, 128)?;
        self.inputs.validate()?;
        for aggregate in &self.aggregates {
            aggregate.validate()?;
        }
        for group in &self.consensus {
            group.validate()?;
        }
        for item in &self.confidence {
            item.validate()?;
        }
        validate_ascii_token_vec(
            "computation_packet.reason_codes",
            &self.reason_codes,
            128,
            32,
        )?;
        Ok(())
    }
}

impl Validate for ComputationInputs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "computation_inputs.evidence_hash",
            &self.evidence_hash,
            128,
        )?;
        validate_ascii_token(
            "computation_inputs.policy_snapshot_id",
            &self.policy_snapshot_id,
            128,
        )?;
        validate_ascii_token_vec(
            "computation_inputs.input_labels",
            &self.input_labels,
            128,
            128,
        )?;
        for trace in &self.normalization_trace {
            trace.validate()?;
        }
        validate_ascii_token_vec(
            "computation_inputs.formula_version_refs",
            &self.formula_version_refs,
            128,
            32,
        )?;
        Ok(())
    }
}

impl Validate for Aggregate {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token("aggregate.metric_id", &self.metric_id, 128)?;
        validate_ascii_token("aggregate.entity", &self.entity, 128)?;
        validate_ascii_token("aggregate.attribute", &self.attribute, 128)?;
        validate_optional_ascii_token("aggregate.unit", &self.unit, 32)?;
        validate_optional_ascii_token("aggregate.currency", &self.currency, 16)?;
        validate_ascii_token_vec("aggregate.source_refs", &self.source_refs, 256, 128)?;
        Ok(())
    }
}

impl Validate for ConsensusGroup {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token("consensus_group.group_id", &self.group_id, 128)?;
        validate_ascii_token("consensus_group.topic", &self.topic, 256)?;
        if let Some(selected_result_id) = self.selected_result_id.as_ref() {
            validate_ascii_token(
                "consensus_group.selected_result_id",
                selected_result_id,
                128,
            )?;
        }
        validate_optional_ascii_token(
            "consensus_group.conflict_resolution_rationale",
            &self.conflict_resolution_rationale,
            256,
        )?;
        Ok(())
    }
}

impl Validate for ConfidenceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token("confidence_item.claim_key", &self.claim_key, 256)?;
        validate_ascii_token(
            "confidence_item.confidence_score",
            &self.confidence_score,
            32,
        )?;
        Ok(())
    }
}
