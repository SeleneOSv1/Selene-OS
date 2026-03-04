#![forbid(unsafe_code)]

pub const AMBIGUITY_POLICY_VERSION: &str = "run33-ambiguity-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClarificationQuestion {
    pub missing_field: String,
    pub question: String,
    pub uncertainty_reduction_score: i32,
}

pub fn select_single_best_clarification(query: &str) -> Option<ClarificationQuestion> {
    let normalized = normalize_query(query);
    if normalized.is_empty() {
        return Some(ClarificationQuestion {
            missing_field: "query_scope".to_string(),
            question: "What should this search focus on?".to_string(),
            uncertainty_reduction_score: 10,
        });
    }

    let mut options = Vec::new();
    if entity_is_ambiguous(&normalized) {
        options.push(ClarificationQuestion {
            missing_field: "entity".to_string(),
            question: "Which entity should this search focus on?".to_string(),
            uncertainty_reduction_score: 9,
        });
    }
    if timeframe_missing(&normalized) {
        options.push(ClarificationQuestion {
            missing_field: "timeframe".to_string(),
            question: "What time window should be used?".to_string(),
            uncertainty_reduction_score: 8,
        });
    }
    if metric_missing(&normalized) {
        options.push(ClarificationQuestion {
            missing_field: "metric".to_string(),
            question: "Which metric or claim should be prioritized?".to_string(),
            uncertainty_reduction_score: 7,
        });
    }

    options.sort_by(|left, right| {
        right
            .uncertainty_reduction_score
            .cmp(&left.uncertainty_reduction_score)
            .then(left.question.cmp(&right.question))
    });
    options.into_iter().next()
}

fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_ascii_lowercase()
}

fn entity_is_ambiguous(query: &str) -> bool {
    const AMBIGUOUS_TOKENS: &[&str] = &[" this ", " that ", " it ", " they ", " those "];
    let padded = format!(" {} ", query);
    AMBIGUOUS_TOKENS.iter().any(|token| padded.contains(token))
}

fn timeframe_missing(query: &str) -> bool {
    const TIME_TOKENS: &[&str] = &[
        "today", "latest", "current", "yesterday", "last", "this", "q1", "q2", "q3", "q4",
        "202", "201", "month", "year",
    ];
    !TIME_TOKENS.iter().any(|token| query.contains(token))
}

fn metric_missing(query: &str) -> bool {
    const METRIC_TOKENS: &[&str] = &[
        "price",
        "risk",
        "revenue",
        "growth",
        "policy",
        "compliance",
        "status",
        "timeline",
        "compare",
        "difference",
        "impact",
    ];
    !METRIC_TOKENS.iter().any(|token| query.contains(token))
}
