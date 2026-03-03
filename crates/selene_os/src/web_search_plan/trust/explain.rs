#![forbid(unsafe_code)]

use crate::web_search_plan::trust::official_detector::TrustTier;
use crate::web_search_plan::trust::trust_score::TrustScoreBreakdown;
use std::collections::BTreeSet;

pub fn build_factors(
    trust_tier: TrustTier,
    detector_reasons: &[String],
    spam_reasons: &[String],
    score: &TrustScoreBreakdown,
) -> Vec<String> {
    let mut set = BTreeSet::new();
    set.insert(format!("TRUST_TIER_{}", trust_tier.as_str()));

    for reason in detector_reasons {
        set.insert(reason.clone());
    }
    for reason in spam_reasons {
        set.insert(reason.clone());
    }

    if score.recency_available && score.recency_bonus > 0.0 {
        set.insert("RECENCY_BONUS".to_string());
    }
    if score.corroboration_bonus > 0.0 {
        set.insert("CORROBORATION_BONUS".to_string());
    }
    if score.reputation_adjustment > 0.0 {
        set.insert("DOMAIN_REPUTATION_BONUS".to_string());
    } else if score.reputation_adjustment < 0.0 {
        set.insert("DOMAIN_REPUTATION_PENALTY".to_string());
    }
    if score.spam_penalty > 0.0 {
        set.insert("SPAM_PENALTY_APPLIED".to_string());
    }

    set.into_iter().collect()
}
