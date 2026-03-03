#![forbid(unsafe_code)]

pub mod compliance_confidence;
pub mod filters;
pub mod jurisdiction;
pub mod provenance;
pub mod trust_tier;

use crate::web_search_plan::regulatory::filters::apply_filters;
use crate::web_search_plan::regulatory::provenance::attach_provenance;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegulatoryErrorKind {
    InsufficientRegulatoryEvidence,
    JurisdictionMismatch,
    ComplianceConfidenceLow,
    StaleData,
    PolicyViolation,
}

impl RegulatoryErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InsufficientRegulatoryEvidence => "insufficient_regulatory_evidence",
            Self::JurisdictionMismatch => "jurisdiction_mismatch",
            Self::ComplianceConfidenceLow => "compliance_confidence_low",
            Self::StaleData => "stale_data",
            Self::PolicyViolation => "policy_violation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegulatoryError {
    pub kind: RegulatoryErrorKind,
    pub message: String,
}

impl RegulatoryError {
    pub fn new(kind: RegulatoryErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegulatoryResult {
    pub evidence_packet: Value,
    pub jurisdiction_code: String,
    pub compliance_confidence: String,
    pub filtered_source_count: usize,
    pub reason_codes: Vec<String>,
}

pub fn apply_regulatory_mode(
    tool_request_packet: &Value,
    evidence_packet: &Value,
) -> Result<RegulatoryResult, RegulatoryError> {
    ensure_regulatory_mode(tool_request_packet)?;

    let outcome = apply_filters(tool_request_packet, evidence_packet)?;
    let mut updated_evidence_packet = evidence_packet.clone();
    attach_provenance(&mut updated_evidence_packet, &outcome, &outcome.reason_codes)
        .map_err(|error| RegulatoryError::new(RegulatoryErrorKind::PolicyViolation, error))?;

    Ok(RegulatoryResult {
        evidence_packet: updated_evidence_packet,
        jurisdiction_code: outcome.jurisdiction.jurisdiction_code,
        compliance_confidence: outcome.compliance_confidence.as_str().to_string(),
        filtered_source_count: outcome.filtered_source_count,
        reason_codes: outcome.reason_codes,
    })
}

fn ensure_regulatory_mode(tool_request_packet: &Value) -> Result<(), RegulatoryError> {
    let enabled_in_budgets = tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("regulatory_mode"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if enabled_in_budgets {
        return Ok(());
    }

    let query = tool_request_packet
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_compliance_query = ["regulatory", "compliance", "law", "statute", "rule"]
        .iter()
        .any(|needle| query.contains(needle));

    if is_compliance_query {
        Ok(())
    } else {
        Err(RegulatoryError::new(
            RegulatoryErrorKind::PolicyViolation,
            "regulatory mode requires budgets.regulatory_mode=true or compliance query",
        ))
    }
}

#[cfg(test)]
pub mod regulatory_tests;
