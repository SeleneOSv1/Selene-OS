#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const REFORMULATION_POLICY_VERSION: &str = "run33-reformulation-v1";
pub const MAX_REFORMULATION_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReformulationOutcome {
    pub triggered: bool,
    pub exhausted: bool,
    pub attempts_used: usize,
    pub rewrite_attempts: Vec<String>,
    pub reformulated_queries: Vec<String>,
}

pub fn apply_reformulation_ladder(
    query: &str,
    existing_attempts: &[String],
    triggered: bool,
    max_attempts: usize,
) -> ReformulationOutcome {
    let cap = max_attempts.min(MAX_REFORMULATION_ATTEMPTS);
    let mut rewrite_attempts = existing_attempts
        .iter()
        .map(|entry| normalize_text(entry))
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<String>>();

    let mut reformulated_queries = Vec::new();
    let normalized_query = normalize_text(query);
    if !normalized_query.is_empty() {
        reformulated_queries.push(normalized_query.clone());
    }

    if triggered {
        while rewrite_attempts.len() < cap {
            let attempt_number = rewrite_attempts.len() + 1;
            let rewritten = rewrite_query(normalized_query.as_str(), attempt_number);
            rewrite_attempts.push(format!("rewrite_{}:{}", attempt_number, rewritten));
            reformulated_queries.push(rewritten);
        }
    }

    let reformulated_queries = dedup_preserve_order(reformulated_queries);
    let attempts_used = rewrite_attempts.len().min(cap);
    let exhausted = triggered && cap > 0 && attempts_used >= cap;

    ReformulationOutcome {
        triggered,
        exhausted,
        attempts_used,
        rewrite_attempts,
        reformulated_queries,
    }
}

fn rewrite_query(query: &str, attempt_number: usize) -> String {
    match attempt_number {
        1 => format!("{} authoritative sources", query),
        2 => format!("{} primary evidence", query),
        _ => format!("{} source validation", query),
    }
}

fn normalize_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

fn dedup_preserve_order(entries: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for entry in entries {
        let key = entry.to_ascii_lowercase();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        out.push(entry);
    }
    out
}
