#![forbid(unsafe_code)]

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyError {
    pub reason_code: &'static str,
    pub message: String,
}

impl ConsistencyError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            reason_code: "policy_violation",
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsistencyInputs<'a> {
    pub evidence_packet: &'a Value,
    pub competitive_packet: Option<&'a Value>,
    pub temporal_packet: Option<&'a Value>,
    pub risk_packet: Option<&'a Value>,
    pub merge_packet: Option<&'a Value>,
    pub report_packet: Option<&'a Value>,
}

pub fn validate_cross_mode_consistency(inputs: ConsistencyInputs<'_>) -> Result<(), ConsistencyError> {
    let evidence_refs = collect_evidence_refs(inputs.evidence_packet);
    if evidence_refs.is_empty() {
        return Err(ConsistencyError::new(
            "enterprise consistency requires evidence refs",
        ));
    }

    if let Some(packet) = inputs.competitive_packet {
        validate_competitive(packet, &evidence_refs)?;
    }
    if let Some(packet) = inputs.temporal_packet {
        validate_temporal(packet, &evidence_refs)?;
    }
    if let Some(packet) = inputs.risk_packet {
        validate_risk(packet, &evidence_refs)?;
    }
    if let Some(packet) = inputs.merge_packet {
        validate_merge(packet, &evidence_refs)?;
    }
    if let Some(packet) = inputs.report_packet {
        validate_report(packet, &evidence_refs)?;
    }

    Ok(())
}

fn collect_evidence_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            for key in ["url", "canonical_url"] {
                if let Some(value) = source.get(key).and_then(Value::as_str) {
                    let value = value.trim();
                    if !value.is_empty() {
                        refs.insert(value.to_string());
                    }
                }
            }
        }
    }

    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            for key in ["chunk_id", "source_url", "canonical_url"] {
                if let Some(value) = chunk.get(key).and_then(Value::as_str) {
                    let value = value.trim();
                    if !value.is_empty() {
                        refs.insert(value.to_string());
                    }
                }
            }
        }
    }

    refs
}

fn validate_competitive(packet: &Value, evidence_refs: &BTreeSet<String>) -> Result<(), ConsistencyError> {
    for path in ["/source_refs"] {
        for reference in read_string_array(packet.pointer(path))? {
            ensure_ref(reference.as_str(), evidence_refs, "competitive source_refs")?;
        }
    }

    if let Some(pricing) = packet.get("pricing_table").and_then(Value::as_array) {
        for (index, row) in pricing.iter().enumerate() {
            let price_value = row.get("price_value").and_then(Value::as_str).unwrap_or("unknown");
            let refs = read_string_array(row.get("source_refs"))?;
            if !price_value.eq_ignore_ascii_case("unknown") && refs.is_empty() {
                return Err(ConsistencyError::new(format!(
                    "competitive pricing row {} has value but no source_refs",
                    index
                )));
            }
            for reference in refs {
                ensure_ref(reference.as_str(), evidence_refs, "competitive pricing source_ref")?;
            }
        }
    }

    if let Some(features) = packet.get("feature_matrix").and_then(Value::as_array) {
        for (index, row) in features.iter().enumerate() {
            let presence = row.get("presence").and_then(Value::as_str).unwrap_or("unknown");
            let refs = read_string_array(row.get("source_refs"))?;
            if !presence.eq_ignore_ascii_case("unknown") && refs.is_empty() {
                return Err(ConsistencyError::new(format!(
                    "competitive feature row {} has presence but no source_refs",
                    index
                )));
            }
            for reference in refs {
                ensure_ref(reference.as_str(), evidence_refs, "competitive feature source_ref")?;
            }
        }
    }

    Ok(())
}

fn validate_temporal(packet: &Value, evidence_refs: &BTreeSet<String>) -> Result<(), ConsistencyError> {
    let Some(changes) = packet.get("changes").and_then(Value::as_array) else {
        return Ok(());
    };

    for (index, change) in changes.iter().enumerate() {
        let change_type = change
            .get("change_type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_ascii_lowercase();
        let prior_refs = read_string_array(change.get("citations_prior"))?;
        let new_refs = read_string_array(change.get("citations_new"))?;

        if matches!(change_type.as_str(), "added" | "modified" | "contradicted") && new_refs.is_empty()
        {
            return Err(ConsistencyError::new(format!(
                "temporal change {} requires citations_new",
                index
            )));
        }
        if change_type == "removed" && prior_refs.is_empty() {
            return Err(ConsistencyError::new(format!(
                "temporal change {} requires citations_prior",
                index
            )));
        }

        for reference in prior_refs.iter().chain(new_refs.iter()) {
            ensure_ref(reference.as_str(), evidence_refs, "temporal citation")?;
        }
    }
    Ok(())
}

fn validate_risk(packet: &Value, evidence_refs: &BTreeSet<String>) -> Result<(), ConsistencyError> {
    if let Some(factors) = packet.get("factor_breakdown").and_then(Value::as_array) {
        for (index, factor) in factors.iter().enumerate() {
            let refs = read_string_array(factor.get("evidence_refs"))?;
            if refs.is_empty() {
                return Err(ConsistencyError::new(format!(
                    "risk factor {} has no evidence_refs",
                    index
                )));
            }
            for reference in refs {
                ensure_ref(reference.as_str(), evidence_refs, "risk factor evidence_ref")?;
            }
        }
    }

    for reference in read_string_array(packet.get("evidence_refs"))? {
        ensure_ref(reference.as_str(), evidence_refs, "risk evidence_ref")?;
    }
    Ok(())
}

fn validate_merge(packet: &Value, evidence_refs: &BTreeSet<String>) -> Result<(), ConsistencyError> {
    if let Some(changes) = packet
        .pointer("/delta/changes_since_last_time")
        .and_then(Value::as_array)
    {
        for (index, change) in changes.iter().enumerate() {
            let refs = read_string_array(change.get("citations"))?;
            if refs.is_empty() {
                return Err(ConsistencyError::new(format!(
                    "merge delta {} has no citations",
                    index
                )));
            }
            for reference in refs {
                ensure_ref(reference.as_str(), evidence_refs, "merge delta citation")?;
            }
        }
    }

    if let Some(conflicts) = packet
        .pointer("/conflict_report/conflicts")
        .and_then(Value::as_array)
    {
        for conflict in conflicts {
            if let Some(rule) = conflict.get("resolution_rule").and_then(Value::as_str) {
                if rule != "external_evidence_prevails" {
                    return Err(ConsistencyError::new(format!(
                        "merge conflict resolution_rule {} is invalid",
                        rule
                    )));
                }
            }
            for reference in read_string_array(conflict.get("evidence_citations"))? {
                ensure_ref(reference.as_str(), evidence_refs, "merge conflict citation")?;
            }
        }
    }
    Ok(())
}

fn validate_report(packet: &Value, evidence_refs: &BTreeSet<String>) -> Result<(), ConsistencyError> {
    let Some(claims) = packet.get("claims").and_then(Value::as_array) else {
        return Ok(());
    };
    for (index, claim) in claims.iter().enumerate() {
        let text = claim.get("text").and_then(Value::as_str).unwrap_or("").trim();
        let refs = read_string_array(claim.get("citations"))?;
        if !text.is_empty() && refs.is_empty() {
            return Err(ConsistencyError::new(format!(
                "report claim {} has text but no citations",
                index
            )));
        }
        for reference in refs {
            ensure_ref(reference.as_str(), evidence_refs, "report citation")?;
        }
    }
    Ok(())
}

fn ensure_ref(reference: &str, evidence_refs: &BTreeSet<String>, context: &str) -> Result<(), ConsistencyError> {
    if evidence_refs.contains(reference) {
        Ok(())
    } else {
        Err(ConsistencyError::new(format!(
            "{} {} is missing from EvidencePacket",
            context, reference
        )))
    }
}

fn read_string_array(raw: Option<&Value>) -> Result<Vec<String>, ConsistencyError> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let array = raw.as_array().ok_or_else(|| {
        ConsistencyError::new("consistency expected array of strings")
    })?;
    let mut out = Vec::with_capacity(array.len());
    for entry in array {
        let value = entry.as_str().ok_or_else(|| {
            ConsistencyError::new("consistency expected string array entry")
        })?;
        let value = value.trim();
        if !value.is_empty() {
            out.push(value.to_string());
        }
    }
    Ok(out)
}
