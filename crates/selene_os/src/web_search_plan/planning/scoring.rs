#![forbid(unsafe_code)]

pub const SCORING_WEIGHTS_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreSignals {
    pub relevance: i32,
    pub trust_tier: i32,
    pub freshness_score: i32,
    pub corroboration_count: i32,
    pub spam_risk: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringWeights {
    pub w_relevance: i32,
    pub w_trust_tier: i32,
    pub w_freshness: i32,
    pub w_corroboration: i32,
    pub w_spam_risk: i32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            w_relevance: 5,
            w_trust_tier: 4,
            w_freshness: 3,
            w_corroboration: 2,
            w_spam_risk: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringPolicy {
    pub policy_snapshot_id: String,
    pub weights_version: String,
    pub weights: ScoringWeights,
}

impl ScoringPolicy {
    pub fn new(policy_snapshot_id: impl Into<String>, weights: ScoringWeights) -> Self {
        Self {
            policy_snapshot_id: policy_snapshot_id.into(),
            weights_version: SCORING_WEIGHTS_VERSION.to_string(),
            weights,
        }
    }
}

impl Default for ScoringPolicy {
    fn default() -> Self {
        Self::new("policy-snapshot-default", ScoringWeights::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreBreakdown {
    pub final_score: i64,
    pub relevance_component: i64,
    pub trust_component: i64,
    pub freshness_component: i64,
    pub corroboration_component: i64,
    pub spam_component: i64,
}

pub fn score_with_policy(policy: &ScoringPolicy, signals: ScoreSignals) -> ScoreBreakdown {
    let relevance_component = policy.weights.w_relevance as i64 * signals.relevance as i64;
    let trust_component = policy.weights.w_trust_tier as i64 * signals.trust_tier as i64;
    let freshness_component = policy.weights.w_freshness as i64 * signals.freshness_score as i64;
    let corroboration_component =
        policy.weights.w_corroboration as i64 * signals.corroboration_count as i64;
    let spam_component = policy.weights.w_spam_risk as i64 * signals.spam_risk as i64;

    ScoreBreakdown {
        final_score: relevance_component
            .saturating_add(trust_component)
            .saturating_add(freshness_component)
            .saturating_add(corroboration_component)
            .saturating_sub(spam_component),
        relevance_component,
        trust_component,
        freshness_component,
        corroboration_component,
        spam_component,
    }
}
