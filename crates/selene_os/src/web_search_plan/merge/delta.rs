#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_EXTERNAL_FINDINGS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Contradicted,
    Unchanged,
}

impl ChangeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Modified => "modified",
            Self::Contradicted => "contradicted",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalFinding {
    pub topic_key: String,
    pub statement: String,
    pub citations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contradiction_group_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaChange {
    pub topic_key: String,
    pub prior_statement: String,
    pub new_statement: String,
    pub change_type: ChangeType,
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaBuildResult {
    pub changes: Vec<DeltaChange>,
    pub reason_codes: Vec<String>,
}

pub fn extract_external_findings(
    evidence_packet: &Value,
    allowed_refs: &BTreeSet<String>,
) -> Result<Vec<ExternalFinding>, String> {
    let explicit = extract_explicit_findings(evidence_packet, allowed_refs)?;
    if !explicit.is_empty() {
        return Ok(explicit);
    }

    let chunk_findings = extract_chunk_findings(evidence_packet, allowed_refs);
    if !chunk_findings.is_empty() {
        return Ok(chunk_findings);
    }

    Ok(extract_source_findings(evidence_packet, allowed_refs))
}

pub fn build_delta(
    prior_key_points: &[String],
    external_findings: &[ExternalFinding],
) -> DeltaBuildResult {
    let mut external_by_topic = BTreeMap::new();
    let mut external_sorted = external_findings.to_vec();
    external_sorted.sort_by(|left, right| {
        (
            left.topic_key.as_str(),
            left.statement.as_str(),
            left.citations.first().map(String::as_str).unwrap_or(""),
        )
            .cmp(&(
                right.topic_key.as_str(),
                right.statement.as_str(),
                right.citations.first().map(String::as_str).unwrap_or(""),
            ))
    });
    for finding in external_sorted {
        external_by_topic
            .entry(finding.topic_key.clone())
            .or_insert(finding);
    }

    let mut prior_by_topic = BTreeMap::new();
    for prior in prior_key_points {
        let normalized = normalize_statement(prior.as_str(), true);
        if normalized.is_empty() {
            continue;
        }
        let topic_key = topic_key_for_statement(prior.as_str());
        prior_by_topic.entry(topic_key).or_insert_with(|| {
            prior
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ")
                .trim()
                .to_string()
        });
    }

    let mut changes = Vec::new();
    let mut reason_codes = BTreeSet::new();

    for (topic_key, finding) in external_by_topic {
        match prior_by_topic.get(topic_key.as_str()) {
            None => {
                changes.push(DeltaChange {
                    topic_key,
                    prior_statement: String::new(),
                    new_statement: finding.statement,
                    change_type: ChangeType::Added,
                    citations: finding.citations,
                });
            }
            Some(prior_statement) => {
                let normalized_prior = normalize_statement(prior_statement, false);
                let normalized_new = normalize_statement(finding.statement.as_str(), false);
                if normalized_prior == normalized_new {
                    continue;
                }
                let change_type = if is_contradiction(prior_statement.as_str(), finding.statement.as_str()) {
                    ChangeType::Contradicted
                } else {
                    ChangeType::Modified
                };
                changes.push(DeltaChange {
                    topic_key,
                    prior_statement: prior_statement.clone(),
                    new_statement: finding.statement,
                    change_type,
                    citations: finding.citations,
                });
            }
        }
    }

    let unresolved_prior = prior_by_topic
        .keys()
        .filter(|topic_key| {
            !changes
                .iter()
                .any(|change| change.topic_key.as_str() == topic_key.as_str())
        })
        .count();
    if unresolved_prior > 0 && external_findings.is_empty() {
        reason_codes.insert("insufficient_evidence".to_string());
    }

    changes.sort_by(|left, right| {
        (
            left.topic_key.as_str(),
            left.change_type.as_str(),
            left.new_statement.as_str(),
        )
            .cmp(&(
                right.topic_key.as_str(),
                right.change_type.as_str(),
                right.new_statement.as_str(),
            ))
    });

    DeltaBuildResult {
        changes,
        reason_codes: reason_codes.into_iter().collect(),
    }
}

pub fn topic_key_for_statement(statement: &str) -> String {
    let normalized = normalize_statement(statement, true);
    if normalized.is_empty() {
        return sha256_hex("topic:empty".as_bytes());
    }
    sha256_hex(format!("topic:{}", normalized).as_bytes())
}

pub fn is_contradiction(prior_statement: &str, new_statement: &str) -> bool {
    let prior_numbers = extract_numeric_tokens(prior_statement);
    let new_numbers = extract_numeric_tokens(new_statement);
    if !prior_numbers.is_empty() && !new_numbers.is_empty() && prior_numbers != new_numbers {
        return true;
    }

    let prior_negated = contains_negation(prior_statement);
    let new_negated = contains_negation(new_statement);
    prior_negated != new_negated
}

fn extract_explicit_findings(
    evidence_packet: &Value,
    allowed_refs: &BTreeSet<String>,
) -> Result<Vec<ExternalFinding>, String> {
    let Some(entries) = evidence_packet
        .pointer("/trust_metadata/merge/top_findings")
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };

    let mut findings = Vec::new();
    for entry in entries {
        let Some(entry_obj) = entry.as_object() else {
            continue;
        };
        let statement = entry_obj
            .get("statement")
            .and_then(Value::as_str)
            .or_else(|| entry_obj.get("text").and_then(Value::as_str))
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        if statement.is_empty() {
            continue;
        }

        let citations = entry_obj
            .get("citations")
            .and_then(Value::as_array)
            .map(|array| {
                array
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|citation| !citation.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        let normalized_citations = normalize_citations(citations, allowed_refs);
        if normalized_citations.is_empty() {
            return Err(format!(
                "explicit top finding '{}' has no citations resolvable in EvidencePacket",
                statement
            ));
        }

        let topic_key = entry_obj
            .get("topic_key")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|topic| !topic.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| topic_key_for_statement(statement.as_str()));

        let contradiction_group_id = entry_obj
            .get("contradiction_group_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        findings.push(ExternalFinding {
            topic_key,
            statement,
            citations: normalized_citations,
            contradiction_group_id,
        });
    }

    Ok(sort_and_limit_findings(findings))
}

fn extract_chunk_findings(
    evidence_packet: &Value,
    allowed_refs: &BTreeSet<String>,
) -> Vec<ExternalFinding> {
    let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    for chunk in chunks {
        let chunk_id = chunk
            .get("chunk_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        if chunk_id.is_empty() || !allowed_refs.contains(chunk_id) {
            continue;
        }
        let statement = chunk
            .get("normalized_text")
            .and_then(Value::as_str)
            .or_else(|| chunk.get("text").and_then(Value::as_str))
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        if statement.is_empty() {
            continue;
        }
        findings.push(ExternalFinding {
            topic_key: topic_key_for_statement(statement.as_str()),
            statement,
            citations: vec![chunk_id.to_string()],
            contradiction_group_id: chunk
                .get("contradiction_group_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
        });
    }

    sort_and_limit_findings(findings)
}

fn extract_source_findings(evidence_packet: &Value, allowed_refs: &BTreeSet<String>) -> Vec<ExternalFinding> {
    let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for source in sources {
        let source_url = source
            .get("url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        if source_url.is_empty() || !allowed_refs.contains(source_url) {
            continue;
        }
        let statement = source
            .get("snippet")
            .and_then(Value::as_str)
            .or_else(|| source.get("title").and_then(Value::as_str))
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        if statement.is_empty() {
            continue;
        }
        findings.push(ExternalFinding {
            topic_key: topic_key_for_statement(statement.as_str()),
            statement,
            citations: vec![source_url.to_string()],
            contradiction_group_id: source
                .get("contradiction_group_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
        });
    }

    sort_and_limit_findings(findings)
}

fn sort_and_limit_findings(mut findings: Vec<ExternalFinding>) -> Vec<ExternalFinding> {
    findings.sort_by(|left, right| {
        (
            left.topic_key.as_str(),
            left.statement.as_str(),
            left.citations.first().map(String::as_str).unwrap_or(""),
        )
            .cmp(&(
                right.topic_key.as_str(),
                right.statement.as_str(),
                right.citations.first().map(String::as_str).unwrap_or(""),
            ))
    });
    findings.dedup_by(|left, right| {
        left.topic_key == right.topic_key
            && left.statement == right.statement
            && left.citations == right.citations
    });
    if findings.len() > MAX_EXTERNAL_FINDINGS {
        findings.truncate(MAX_EXTERNAL_FINDINGS);
    }
    findings
}

fn normalize_citations(citations: Vec<String>, allowed_refs: &BTreeSet<String>) -> Vec<String> {
    let mut out = citations
        .into_iter()
        .filter(|citation| allowed_refs.contains(citation.as_str()))
        .collect::<Vec<String>>();
    out.sort();
    out.dedup();
    out
}

fn normalize_statement(raw: &str, strip_digits: bool) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphabetic() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }
        if !strip_digits && ch.is_ascii_digit() {
            out.push(ch);
            continue;
        }
        out.push(' ');
    }
    out.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn contains_negation(raw: &str) -> bool {
    const NEGATION_TOKENS: &[&str] = &["no", "not", "never", "without", "none"];
    let normalized = normalize_statement(raw, false);
    normalized
        .split_whitespace()
        .any(|token| NEGATION_TOKENS.contains(&token))
}

fn extract_numeric_tokens(raw: &str) -> Vec<String> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            numbers.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        numbers.push(current);
    }
    numbers
}
