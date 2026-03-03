#![forbid(unsafe_code)]

use crate::web_search_plan::regulatory::filters::RegulatoryFilterOutcome;
use crate::web_search_plan::regulatory::trust_tier::TRUST_TIER_POLICY_VERSION;
use serde_json::{json, Map, Value};

pub const REGULATORY_PROVENANCE_VERSION: &str = "1.0.0";

pub fn attach_provenance(
    evidence_packet: &mut Value,
    outcome: &RegulatoryFilterOutcome,
    reason_codes: &[String],
) -> Result<(), String> {
    let obj = evidence_packet
        .as_object_mut()
        .ok_or_else(|| "evidence packet must be object".to_string())?;

    obj.insert(
        "sources".to_string(),
        Value::Array(outcome.retained_sources.clone()),
    );

    let trust_metadata = obj
        .entry("trust_metadata".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let trust_metadata_obj = trust_metadata
        .as_object_mut()
        .ok_or_else(|| "trust_metadata must be object".to_string())?;

    let source_tiers = outcome
        .source_trust_tiers
        .iter()
        .map(|record| {
            json!({
                "url": record.url,
                "trust_tier": record.trust_tier.as_str(),
                "source_jurisdiction": record.source_jurisdiction,
            })
        })
        .collect::<Vec<Value>>();

    trust_metadata_obj.insert(
        "regulatory".to_string(),
        json!({
            "provenance_version": REGULATORY_PROVENANCE_VERSION,
            "trust_tier_policy_version": TRUST_TIER_POLICY_VERSION,
            "jurisdiction_code": outcome.jurisdiction.jurisdiction_code,
            "jurisdiction_confidence": outcome.jurisdiction.confidence.as_str(),
            "jurisdiction_method": outcome.jurisdiction.method,
            "compliance_confidence": outcome.compliance_confidence.as_str(),
            "filtered_source_count": outcome.filtered_source_count,
            "source_trust_tiers": source_tiers,
            "reasons": reason_codes,
        }),
    );

    Ok(())
}
