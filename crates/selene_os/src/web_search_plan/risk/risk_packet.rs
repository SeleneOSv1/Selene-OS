#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::risk::calibration::{risk_model_version, ROUNDING_SCALE};
use crate::web_search_plan::risk::confidence::build_confidence;
use crate::web_search_plan::risk::factors::{extract_factor_scores, FactorId, FactorScore};
use crate::web_search_plan::risk::guardrails::{disclaimer_text, enforce_non_advice_guardrails};
use crate::web_search_plan::risk::scoring::{compute_composite_risk, RiskLevel};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RiskRequest {
    pub trace_id: String,
    pub created_at_ms: i64,
    pub intended_consumers: Vec<String>,
    pub evidence_packet: Value,
    pub computation_packet: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactorScoreOutput {
    pub factor_id: String,
    pub score: String,
    pub weight: String,
    pub evidence_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskPacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub risk_score: String,
    pub risk_level: String,
    pub factor_breakdown: Vec<FactorScoreOutput>,
    pub confidence_score: String,
    pub evidence_refs: Vec<String>,
    pub risk_model_version: String,
    pub disclaimer: String,
    pub uncertainty_flags: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskBuildError {
    pub reason_code: &'static str,
    pub message: String,
}

impl RiskBuildError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

pub fn build_risk_packet(request: &RiskRequest) -> Result<RiskPacket, RiskBuildError> {
    if request.trace_id.trim().is_empty() {
        return Err(RiskBuildError::new(
            "policy_violation",
            "risk request trace_id is required",
        ));
    }

    let factor_result = extract_factor_scores(
        &request.evidence_packet,
        request.computation_packet.as_ref(),
    )
    .map_err(|error| RiskBuildError::new("policy_violation", error))?;

    if factor_result.factors.is_empty() {
        return Err(RiskBuildError::new(
            "insufficient_evidence",
            "no risk factors had enough evidence",
        ));
    }

    let composite = compute_composite_risk(&factor_result.factors)
        .map_err(|error| RiskBuildError::new("insufficient_evidence", error))?;
    let confidence = build_confidence(
        factor_result.factors.as_slice(),
        &request.evidence_packet,
        request.computation_packet.as_ref(),
        factor_result.contradiction_present,
    );

    let mut reason_codes = factor_result.reason_codes.clone();
    let mut uncertainty_flags = Vec::new();
    if !factor_result.missing_factors.is_empty() {
        uncertainty_flags.push(format!(
            "missing_factors:{}",
            factor_result
                .missing_factors
                .iter()
                .map(|factor| factor.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        ));
    }
    if confidence.confidence_low {
        uncertainty_flags.push("confidence_low".to_string());
    }

    let mut evidence_refs = factor_result
        .factors
        .iter()
        .flat_map(|factor| factor.evidence_refs.iter().cloned())
        .collect::<Vec<String>>();
    evidence_refs.sort();
    evidence_refs.dedup();

    for evidence_ref in &evidence_refs {
        if !factor_result.known_refs.contains(evidence_ref) {
            return Err(RiskBuildError::new(
                "policy_violation",
                format!("evidence_ref {} is not present in evidence packet", evidence_ref),
            ));
        }
    }

    let factor_breakdown = factor_result
        .factors
        .iter()
        .map(to_factor_output)
        .collect::<Vec<FactorScoreOutput>>();
    let texts_for_guardrails = factor_breakdown
        .iter()
        .filter_map(|factor| factor.notes.clone())
        .collect::<Vec<String>>();
    enforce_non_advice_guardrails(texts_for_guardrails.as_slice())
        .map_err(|error| RiskBuildError::new(error.reason_code, error.message))?;

    let disclaimer = disclaimer_text().to_string();
    enforce_non_advice_guardrails(std::slice::from_ref(&disclaimer))
        .map_err(|error| RiskBuildError::new(error.reason_code, error.message))?;

    let mut packet = RiskPacket {
        schema_version: "1.0.0".to_string(),
        produced_by: "PH1.ANALYTICS".to_string(),
        intended_consumers: if request.intended_consumers.is_empty() {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        } else {
            request.intended_consumers.clone()
        },
        created_at_ms: request.created_at_ms,
        trace_id: request.trace_id.clone(),
        risk_score: decimal_to_string(composite.risk_score),
        risk_level: risk_level_to_string(composite.risk_level).to_string(),
        factor_breakdown,
        confidence_score: decimal_to_string(confidence.confidence_score),
        evidence_refs,
        risk_model_version: risk_model_version().to_string(),
        disclaimer,
        uncertainty_flags,
        reason_codes: {
            reason_codes.sort();
            reason_codes.dedup();
            reason_codes
        },
    };

    validate_risk_packet(&packet)?;
    packet.factor_breakdown.sort_by(|left, right| left.factor_id.cmp(&right.factor_id));
    packet.evidence_refs.sort();
    Ok(packet)
}

fn to_factor_output(factor: &FactorScore) -> FactorScoreOutput {
    FactorScoreOutput {
        factor_id: factor.factor_id.as_str().to_string(),
        score: decimal_to_string(factor.score),
        weight: decimal_to_string(factor.weight),
        evidence_refs: factor.evidence_refs.clone(),
        notes: factor.notes.clone(),
    }
}

fn validate_risk_packet(packet: &RiskPacket) -> Result<(), RiskBuildError> {
    let registry = load_packet_schema_registry()
        .map_err(|error| RiskBuildError::new("policy_violation", error))?;
    let value = serde_json::to_value(packet)
        .map_err(|error| RiskBuildError::new("policy_violation", error.to_string()))?;
    validate_packet("RiskPacket", &value, &registry)
        .map_err(|error| RiskBuildError::new("policy_violation", error))
}

fn risk_level_to_string(level: RiskLevel) -> &'static str {
    level.as_str()
}

fn decimal_to_string(value: Decimal) -> String {
    value
        .round_dp_with_strategy(
            ROUNDING_SCALE,
            rust_decimal::RoundingStrategy::MidpointAwayFromZero,
        )
        .normalize()
        .to_string()
}

#[allow(dead_code)]
fn _factor_id_parse(raw: &str) -> Option<FactorId> {
    match raw {
        "financial_stress" => Some(FactorId::FinancialStress),
        "legal_events" => Some(FactorId::LegalEvents),
        "regulatory_events" => Some(FactorId::RegulatoryEvents),
        "negative_news_cluster" => Some(FactorId::NegativeNewsCluster),
        "operational_reliability" => Some(FactorId::OperationalReliability),
        _ => None,
    }
}
