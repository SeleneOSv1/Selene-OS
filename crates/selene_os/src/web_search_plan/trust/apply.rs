#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::TRUST_MODEL_VERSION;
use crate::web_search_plan::trust::explain::build_factors;
use crate::web_search_plan::trust::official_detector::detect_from_url;
use crate::web_search_plan::trust::spam_signals::compute_spam_signals;
use crate::web_search_plan::trust::trust_score::{
    compute_trust_score, parse_corroboration_count, parse_published_at_ms,
};
use crate::web_search_plan::trust::{TrustError, TrustErrorKind};
use crate::web_search_plan::url::canonical::canonicalize_url;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceEnrichment {
    pub index: usize,
    pub url: String,
    pub canonical_url: String,
    pub trust_tier: String,
    pub trust_score: f64,
    pub spam_risk_score: f64,
    pub trust_factors: Vec<String>,
    pub official_source: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustApplyResult {
    pub evidence_packet: Value,
    pub enriched_sources: Vec<SourceEnrichment>,
}

pub fn enrich_evidence_sources(
    evidence_packet: &Value,
    now_ms: i64,
) -> Result<TrustApplyResult, TrustError> {
    let mut updated = evidence_packet.clone();
    let obj = updated.as_object_mut().ok_or_else(|| {
        TrustError::new(
            TrustErrorKind::PolicyViolation,
            "evidence packet must be object",
        )
    })?;

    let sources = obj
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TrustError::new(
                TrustErrorKind::InsufficientEvidence,
                "evidence packet sources must be an array",
            )
        })?
        .clone();
    if sources.is_empty() {
        return Err(TrustError::new(
            TrustErrorKind::InsufficientEvidence,
            "evidence packet sources cannot be empty for trust scoring",
        ));
    }

    let host_counts = compute_host_counts(&sources);
    let mut new_sources = Vec::with_capacity(sources.len());
    let mut enrichments = Vec::with_capacity(sources.len());

    for (index, source_value) in sources.iter().enumerate() {
        let mut source_obj = source_value.as_object().cloned().ok_or_else(|| {
            TrustError::new(
                TrustErrorKind::PolicyViolation,
                format!("source at index {} must be object", index),
            )
        })?;

        let url = source_obj
            .get("url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                TrustError::new(
                    TrustErrorKind::PolicyViolation,
                    format!("source at index {} missing url", index),
                )
            })?
            .to_string();
        let canonical_url = source_obj
            .get("canonical_url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| derive_canonical_url(url.as_str()));

        let title = source_obj
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let snippet = source_obj
            .get("snippet")
            .and_then(Value::as_str)
            .unwrap_or_default();

        let detector = detect_from_url(url.as_str());
        let spam = compute_spam_signals(url.as_str(), title, snippet);
        let host_based_corroboration = detector
            .host
            .as_ref()
            .and_then(|host| host_counts.get(host).copied())
            .unwrap_or(0);
        let explicit_corroboration = parse_corroboration_count(source_value);
        let corroboration_count = explicit_corroboration.max(host_based_corroboration);
        let published_at_ms = parse_published_at_ms(source_value);
        let breakdown = compute_trust_score(
            detector.trust_tier,
            &spam,
            detector.host.as_deref(),
            published_at_ms,
            now_ms,
            corroboration_count,
        );
        let factors = build_factors(
            detector.trust_tier,
            &detector.reasons,
            &spam.reasons,
            &breakdown,
        );

        source_obj.insert("canonical_url".to_string(), Value::String(canonical_url.clone()));
        source_obj.insert(
            "trust_tier".to_string(),
            Value::String(detector.trust_tier.as_str().to_string()),
        );
        source_obj.insert("trust_score".to_string(), json!(breakdown.trust_score));
        source_obj.insert("spam_risk_score".to_string(), json!(spam.spam_risk_score));
        source_obj.insert(
            "trust_factors".to_string(),
            Value::Array(factors.iter().map(|factor| Value::String(factor.clone())).collect()),
        );
        source_obj.insert("official_source".to_string(), Value::Bool(detector.official_source));
        source_obj.insert(
            "trust_model_version".to_string(),
            Value::String(TRUST_MODEL_VERSION.to_string()),
        );

        new_sources.push(Value::Object(source_obj));
        enrichments.push(SourceEnrichment {
            index,
            url,
            canonical_url,
            trust_tier: detector.trust_tier.as_str().to_string(),
            trust_score: breakdown.trust_score,
            spam_risk_score: spam.spam_risk_score,
            trust_factors: factors,
            official_source: detector.official_source,
        });
    }

    obj.insert("sources".to_string(), Value::Array(new_sources));
    let trust_metadata = obj
        .entry("trust_metadata".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !trust_metadata.is_object() {
        return Err(TrustError::new(
            TrustErrorKind::PolicyViolation,
            "trust_metadata must be object",
        ));
    }
    let trust_metadata_obj = trust_metadata
        .as_object_mut()
        .ok_or_else(|| TrustError::new(TrustErrorKind::PolicyViolation, "trust_metadata invalid"))?;
    trust_metadata_obj.insert(
        "trust_model".to_string(),
        json!({
            "trust_model_version": TRUST_MODEL_VERSION,
            "scored_source_count": enrichments.len(),
            "scored_at_ms": now_ms,
        }),
    );

    Ok(TrustApplyResult {
        evidence_packet: updated,
        enriched_sources: enrichments,
    })
}

fn derive_canonical_url(url: &str) -> String {
    canonicalize_url(url)
        .map(|canonical| canonical.canonical_url)
        .unwrap_or_else(|_| url.trim().to_ascii_lowercase())
}

fn compute_host_counts(sources: &[Value]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source in sources {
        let Some(url) = source.get("url").and_then(Value::as_str) else {
            continue;
        };
        let Some(host) = url::Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()))
        else {
            continue;
        };
        *counts.entry(host).or_insert(0) += 1;
    }
    counts
}
