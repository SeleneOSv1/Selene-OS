#![forbid(unsafe_code)]

use crate::web_search_plan::planning::scoring::ScoreBreakdown;
use crate::web_search_plan::planning::SearchCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedCandidate {
    pub candidate: SearchCandidate,
    pub score: ScoreBreakdown,
}

pub fn sort_ranked_candidates(mut ranked: Vec<RankedCandidate>) -> Vec<RankedCandidate> {
    ranked.sort_by(|left, right| {
        right
            .score
            .final_score
            .cmp(&left.score.final_score)
            .then(right.candidate.trust_tier.cmp(&left.candidate.trust_tier))
            .then(
                right
                    .candidate
                    .freshness_score
                    .cmp(&left.candidate.freshness_score),
            )
            .then(
                left.candidate
                    .canonical_url
                    .cmp(&right.candidate.canonical_url),
            )
            .then(left.candidate.url.cmp(&right.candidate.url))
    });
    ranked
}
