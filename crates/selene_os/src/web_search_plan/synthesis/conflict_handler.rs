#![forbid(unsafe_code)]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictClaim {
    pub claim_text: String,
    pub source_url: String,
    pub chunk_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictGroup {
    pub group_id: String,
    pub claims: Vec<ConflictClaim>,
}

pub fn detect_conflicts(evidence_packet: &Value) -> Vec<ConflictGroup> {
    let content_chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut groups: BTreeMap<String, Vec<ConflictClaim>> = BTreeMap::new();
    for chunk in content_chunks {
        let Some(group_id) = chunk
            .get("contradiction_group_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|group| !group.is_empty())
        else {
            continue;
        };

        let claim_text = chunk
            .get("text_excerpt")
            .and_then(Value::as_str)
            .or_else(|| chunk.get("excerpt").and_then(Value::as_str))
            .unwrap_or("")
            .trim()
            .to_string();
        let source_url = chunk
            .get("source_url")
            .and_then(Value::as_str)
            .or_else(|| {
                chunk
                    .get("citation")
                    .and_then(Value::as_object)
                    .and_then(|citation| citation.get("source_url"))
                    .and_then(Value::as_str)
            })
            .unwrap_or("")
            .trim()
            .to_string();
        let chunk_id = chunk
            .get("chunk_id")
            .and_then(Value::as_str)
            .or_else(|| {
                chunk
                    .get("citation")
                    .and_then(Value::as_object)
                    .and_then(|citation| citation.get("chunk_id"))
                    .and_then(Value::as_str)
            })
            .unwrap_or("")
            .trim()
            .to_string();

        groups
            .entry(group_id.to_string())
            .or_default()
            .push(ConflictClaim {
                claim_text,
                source_url,
                chunk_id,
            });
    }

    let mut out = Vec::new();
    for (group_id, mut claims) in groups {
        let unique_claims: BTreeSet<String> = claims
            .iter()
            .map(|claim| claim.claim_text.to_ascii_lowercase())
            .collect();
        let unique_sources: BTreeSet<String> = claims
            .iter()
            .map(|claim| claim.source_url.to_ascii_lowercase())
            .collect();

        if unique_claims.len() < 2 || unique_sources.len() < 2 {
            continue;
        }

        claims.sort_by(|a, b| {
            a.source_url
                .cmp(&b.source_url)
                .then(a.chunk_id.cmp(&b.chunk_id))
                .then(a.claim_text.cmp(&b.claim_text))
        });
        out.push(ConflictGroup { group_id, claims });
    }

    out
}
