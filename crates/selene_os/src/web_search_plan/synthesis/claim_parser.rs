#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CitationRefKind {
    ChunkId,
    SourceUrl,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CitationRef {
    pub kind: CitationRefKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicClaim {
    pub claim_id: String,
    pub text: String,
    pub citations: Vec<CitationRef>,
}

pub fn parse_atomic_claims(answer_text: &str) -> Vec<AtomicClaim> {
    let mut claims = Vec::new();

    for line in answer_text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") {
            continue;
        }

        let body = trimmed.trim_start_matches("- ").trim();
        let citations = extract_marked_citations(body);
        // Only bullets with explicit citation markers are treated as factual claims.
        if citations.is_empty() {
            continue;
        }
        let claim_text = strip_marked_citations(body)
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        if claim_text.is_empty() {
            continue;
        }

        let claim_id = format!("claim_{:03}", claims.len() + 1);
        claims.push(AtomicClaim {
            claim_id,
            text: claim_text,
            citations,
        });
    }

    claims
}

fn extract_marked_citations(text: &str) -> Vec<CitationRef> {
    let mut citations = Vec::new();
    let mut remainder = text;

    while let Some(open_idx) = remainder.find('[') {
        let after_open = &remainder[open_idx + 1..];
        let Some(close_rel) = after_open.find(']') else {
            break;
        };

        let marker = after_open[..close_rel].trim();
        if let Some(value) = marker.strip_prefix("chunk:") {
            let value = value.trim();
            if !value.is_empty() {
                citations.push(CitationRef {
                    kind: CitationRefKind::ChunkId,
                    value: value.to_string(),
                });
            }
        } else if let Some(value) = marker.strip_prefix("url:") {
            let value = value.trim();
            if !value.is_empty() {
                citations.push(CitationRef {
                    kind: CitationRefKind::SourceUrl,
                    value: value.to_string(),
                });
            }
        }

        remainder = &after_open[close_rel + 1..];
    }

    citations
}

fn strip_marked_citations(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut remainder = text;

    while let Some(open_idx) = remainder.find('[') {
        out.push_str(&remainder[..open_idx]);
        let after_open = &remainder[open_idx + 1..];
        let Some(close_rel) = after_open.find(']') else {
            out.push('[');
            out.push_str(after_open);
            return out;
        };
        remainder = &after_open[close_rel + 1..];
    }

    out.push_str(remainder);
    out
}
