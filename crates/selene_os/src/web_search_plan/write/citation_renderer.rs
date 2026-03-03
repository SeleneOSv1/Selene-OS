#![forbid(unsafe_code)]

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CitationKind {
    SourceUrl,
    ChunkId,
}

impl CitationKind {
    pub const fn as_packet_type(&self) -> &'static str {
        match self {
            Self::SourceUrl => "source_url",
            Self::ChunkId => "chunk_id",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CitationRef {
    pub kind: CitationKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEvidenceLine {
    pub text: String,
    pub citations: Vec<CitationRef>,
}

#[derive(Debug, Clone)]
pub struct CitationBuildOutput {
    pub citation_map: Map<String, Value>,
    pub key_by_ref: BTreeMap<CitationRef, String>,
    pub ordered_refs: Vec<CitationRef>,
    pub ordered_url_keys: Vec<String>,
}

pub fn parse_evidence_lines(answer_text: &str) -> Vec<ParsedEvidenceLine> {
    let mut in_evidence_section = false;
    let mut out = Vec::new();

    for raw_line in answer_text.lines() {
        let line = raw_line.trim();
        if line.eq_ignore_ascii_case("Evidence") || line.eq_ignore_ascii_case("Evidence:") {
            in_evidence_section = true;
            continue;
        }

        if !in_evidence_section {
            continue;
        }

        if line.eq_ignore_ascii_case("Citations")
            || line.eq_ignore_ascii_case("Citations:")
            || line.eq_ignore_ascii_case("Optional Uncertainty")
            || line.eq_ignore_ascii_case("Optional Uncertainty:")
        {
            break;
        }

        if !line.starts_with("- ") {
            continue;
        }

        let body = line.trim_start_matches("- ").trim();
        let (claim_text, citations) = parse_marked_citations(body);
        out.push(ParsedEvidenceLine {
            text: normalize_whitespace(&claim_text),
            citations,
        });
    }

    out
}

pub fn build_citation_map(
    answer_text: &str,
    synthesis_citations: &[Value],
) -> Result<CitationBuildOutput, String> {
    let evidence_lines = parse_evidence_lines(answer_text);

    let mut ordered_refs = Vec::new();
    let mut seen: BTreeSet<CitationRef> = BTreeSet::new();
    for line in &evidence_lines {
        for citation in &line.citations {
            if seen.insert(citation.clone()) {
                ordered_refs.push(citation.clone());
            }
        }
    }

    for value in synthesis_citations {
        let object = value
            .as_object()
            .ok_or_else(|| "synthesis citation entry must be object".to_string())?;
        let kind_raw = object
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| "synthesis citation entry missing type".to_string())?;
        let citation_value = object
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| "synthesis citation entry missing value".to_string())?
            .trim()
            .to_string();
        if citation_value.is_empty() {
            continue;
        }

        let kind = match kind_raw {
            "source_url" => CitationKind::SourceUrl,
            "chunk_id" => CitationKind::ChunkId,
            other => return Err(format!("unsupported synthesis citation type {}", other)),
        };

        let citation = CitationRef {
            kind,
            value: citation_value,
        };
        if seen.insert(citation.clone()) {
            ordered_refs.push(citation);
        }
    }

    let mut citation_map = Map::new();
    let mut key_by_ref = BTreeMap::new();
    let mut ordered_url_keys = Vec::new();

    // URL references are keyed first so user-facing citation keys are contiguous and stable.
    let mut keyed_refs = Vec::new();
    keyed_refs.extend(
        ordered_refs
            .iter()
            .filter(|citation| citation.kind == CitationKind::SourceUrl)
            .cloned(),
    );
    keyed_refs.extend(
        ordered_refs
            .iter()
            .filter(|citation| citation.kind == CitationKind::ChunkId)
            .cloned(),
    );

    for (index, citation) in keyed_refs.iter().enumerate() {
        let key = format!("C{}", index + 1);
        citation_map.insert(
            key.clone(),
            json!({
                "type": citation.kind.as_packet_type(),
                "value": citation.value,
            }),
        );
        key_by_ref.insert(citation.clone(), key.clone());
        if citation.kind == CitationKind::SourceUrl {
            ordered_url_keys.push(key);
        }
    }

    Ok(CitationBuildOutput {
        citation_map,
        key_by_ref,
        ordered_refs,
        ordered_url_keys,
    })
}

pub fn citation_keys_for_bullet(
    citations: &[CitationRef],
    key_by_ref: &BTreeMap<CitationRef, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();

    for citation in citations {
        if citation.kind != CitationKind::SourceUrl {
            continue;
        }
        let Some(key) = key_by_ref.get(citation) else {
            continue;
        };
        if seen.insert(key.clone()) {
            out.push(key.clone());
        }
    }

    out
}

pub fn strip_marker_tokens(input: &str) -> String {
    normalize_whitespace(&parse_marked_citations(input).0)
}

fn parse_marked_citations(input: &str) -> (String, Vec<CitationRef>) {
    let mut claims = String::with_capacity(input.len());
    let mut citations = Vec::new();
    let mut remainder = input;

    while let Some(open_idx) = remainder.find('[') {
        claims.push_str(&remainder[..open_idx]);

        let after_open = &remainder[open_idx + 1..];
        let Some(close_rel) = after_open.find(']') else {
            claims.push('[');
            claims.push_str(after_open);
            return (claims, citations);
        };

        let marker = after_open[..close_rel].trim();
        if let Some(value) = marker.strip_prefix("url:") {
            let value = value.trim();
            if !value.is_empty() {
                citations.push(CitationRef {
                    kind: CitationKind::SourceUrl,
                    value: value.to_string(),
                });
            }
        } else if let Some(value) = marker.strip_prefix("chunk:") {
            let value = value.trim();
            if !value.is_empty() {
                citations.push(CitationRef {
                    kind: CitationKind::ChunkId,
                    value: value.to_string(),
                });
            }
        } else {
            claims.push('[');
            claims.push_str(marker);
            claims.push(']');
        }

        remainder = &after_open[close_rel + 1..];
    }

    claims.push_str(remainder);

    (claims, dedupe_preserving_order(&citations))
}

fn dedupe_preserving_order(input: &[CitationRef]) -> Vec<CitationRef> {
    let mut out = Vec::new();
    let mut seen: BTreeSet<CitationRef> = BTreeSet::new();
    for item in input {
        if seen.insert(item.clone()) {
            out.push(item.clone());
        }
    }
    out
}

fn normalize_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}
