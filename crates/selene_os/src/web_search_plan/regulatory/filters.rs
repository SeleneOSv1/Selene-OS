#![forbid(unsafe_code)]

use crate::web_search_plan::regulatory::compliance_confidence::{
    assess_confidence, meets_required_threshold, ComplianceConfidence,
};
use crate::web_search_plan::regulatory::jurisdiction::{
    map_url_to_jurisdiction, resolve_jurisdiction, JurisdictionResolution,
};
use crate::web_search_plan::regulatory::trust_tier::{classify_source, TrustTier};
use crate::web_search_plan::regulatory::{RegulatoryError, RegulatoryErrorKind};
use serde_json::Value;
use std::collections::BTreeSet;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct RegulatoryFilterOutcome {
    pub jurisdiction: JurisdictionResolution,
    pub compliance_confidence: ComplianceConfidence,
    pub filtered_source_count: usize,
    pub retained_sources: Vec<Value>,
    pub source_trust_tiers: Vec<SourceTrustTierRecord>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTrustTierRecord {
    pub url: String,
    pub trust_tier: TrustTier,
    pub source_jurisdiction: Option<String>,
}

pub fn apply_filters(
    tool_request_packet: &Value,
    evidence_packet: &Value,
) -> Result<RegulatoryFilterOutcome, RegulatoryError> {
    let jurisdiction = resolve_jurisdiction(tool_request_packet, evidence_packet).ok_or_else(|| {
        RegulatoryError::new(
            RegulatoryErrorKind::JurisdictionMismatch,
            "failed to resolve jurisdiction for regulatory mode",
        )
    })?;

    if realtime_stale(evidence_packet) {
        return Err(RegulatoryError::new(
            RegulatoryErrorKind::StaleData,
            "upstream evidence is stale; regulatory mode refuses stale evidence",
        ));
    }

    let min_tier = minimum_trust_tier(tool_request_packet);
    let required_confidence = required_compliance_confidence(tool_request_packet);

    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            RegulatoryError::new(
                RegulatoryErrorKind::PolicyViolation,
                "evidence packet sources missing or invalid",
            )
        })?;

    let mut retained_sources = Vec::new();
    let mut source_trust_tiers = Vec::new();
    let mut filtered_source_count = 0usize;
    let mut retained_tiers = Vec::new();
    let mut retained_domains = BTreeSet::new();

    for source in sources {
        let source_url = source
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let trust_tier = classify_source(source);
        if trust_tier.rank() < min_tier.rank() {
            filtered_source_count += 1;
            continue;
        }

        let source_jurisdiction = source
            .get("jurisdiction_code")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_ascii_uppercase())
            .or_else(|| map_url_to_jurisdiction(source_url.as_str()));
        if let Some(source_code) = &source_jurisdiction {
            if source_code != &jurisdiction.jurisdiction_code {
                return Err(RegulatoryError::new(
                    RegulatoryErrorKind::JurisdictionMismatch,
                    format!(
                        "source jurisdiction {} mismatches resolved jurisdiction {}",
                        source_code, jurisdiction.jurisdiction_code
                    ),
                ));
            }
        }

        retained_tiers.push(trust_tier);
        if let Some(domain) = domain_from_url(source_url.as_str()) {
            retained_domains.insert(domain);
        }
        source_trust_tiers.push(SourceTrustTierRecord {
            url: source_url,
            trust_tier,
            source_jurisdiction,
        });
        retained_sources.push(source.clone());
    }

    if retained_sources.is_empty() {
        return Err(RegulatoryError::new(
            RegulatoryErrorKind::InsufficientRegulatoryEvidence,
            "no eligible sources remain after trust-tier filtering",
        ));
    }

    let all_fresh = !realtime_stale(evidence_packet);
    let compliance_confidence =
        assess_confidence(&retained_tiers, all_fresh, retained_domains.len());
    if !meets_required_threshold(compliance_confidence, required_confidence) {
        return Err(RegulatoryError::new(
            RegulatoryErrorKind::ComplianceConfidenceLow,
            format!(
                "compliance confidence {} below required {}",
                compliance_confidence.as_str(),
                required_confidence.as_str()
            ),
        ));
    }

    Ok(RegulatoryFilterOutcome {
        jurisdiction,
        compliance_confidence,
        filtered_source_count,
        retained_sources,
        source_trust_tiers,
        reason_codes: Vec::new(),
    })
}

fn minimum_trust_tier(tool_request_packet: &Value) -> TrustTier {
    tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("min_trust_tier"))
        .and_then(Value::as_str)
        .and_then(TrustTier::parse_threshold)
        .unwrap_or(TrustTier::High)
}

fn required_compliance_confidence(tool_request_packet: &Value) -> ComplianceConfidence {
    tool_request_packet
        .get("budgets")
        .and_then(Value::as_object)
        .and_then(|budgets| budgets.get("required_compliance_confidence"))
        .and_then(Value::as_str)
        .and_then(ComplianceConfidence::parse_threshold)
        .unwrap_or(ComplianceConfidence::Moderate)
}

fn realtime_stale(evidence_packet: &Value) -> bool {
    evidence_packet
        .pointer("/trust_metadata/realtime/stale")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn domain_from_url(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()))
}
