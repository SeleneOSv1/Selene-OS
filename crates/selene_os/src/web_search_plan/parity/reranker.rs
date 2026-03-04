#![forbid(unsafe_code)]

pub const RERANK_WEIGHTS_VERSION: &str = "run33-rerank-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RerankWeights {
    pub w_relevance: i32,
    pub w_trust: i32,
    pub w_freshness: i32,
    pub w_corroboration: i32,
    pub w_spam_risk: i32,
}

impl Default for RerankWeights {
    fn default() -> Self {
        Self {
            w_relevance: 5,
            w_trust: 4,
            w_freshness: 3,
            w_corroboration: 2,
            w_spam_risk: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankInput {
    pub stable_id: String,
    pub canonical_url: String,
    pub relevance: i32,
    pub trust: i32,
    pub freshness: i32,
    pub corroboration: i32,
    pub spam_risk: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankOutput {
    pub stable_id: String,
    pub canonical_url: String,
    pub final_score: i64,
    pub trust: i32,
    pub freshness: i32,
}

pub fn rerank_candidates(inputs: &[RerankInput], weights: RerankWeights) -> Vec<RerankOutput> {
    let mut outputs = inputs
        .iter()
        .map(|input| RerankOutput {
            stable_id: input.stable_id.clone(),
            canonical_url: input.canonical_url.clone(),
            final_score: score(input, weights),
            trust: input.trust,
            freshness: input.freshness,
        })
        .collect::<Vec<RerankOutput>>();

    outputs.sort_by(|left, right| {
        right
            .final_score
            .cmp(&left.final_score)
            .then(right.trust.cmp(&left.trust))
            .then(right.freshness.cmp(&left.freshness))
            .then(left.canonical_url.cmp(&right.canonical_url))
            .then(left.stable_id.cmp(&right.stable_id))
    });
    outputs
}

fn score(input: &RerankInput, weights: RerankWeights) -> i64 {
    (weights.w_relevance as i64 * input.relevance as i64)
        .saturating_add(weights.w_trust as i64 * input.trust as i64)
        .saturating_add(weights.w_freshness as i64 * input.freshness as i64)
        .saturating_add(weights.w_corroboration as i64 * input.corroboration as i64)
        .saturating_sub(weights.w_spam_risk as i64 * input.spam_risk as i64)
}
