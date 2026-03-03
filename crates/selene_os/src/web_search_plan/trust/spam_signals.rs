#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::{CLICKBAIT_KEYWORDS, TRACKING_QUERY_PARAMS};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct SpamSignals {
    pub spam_risk_score: f64,
    pub reasons: Vec<String>,
}

pub fn compute_spam_signals(url: &str, title: &str, snippet: &str) -> SpamSignals {
    let mut score: f64 = 0.0;
    let mut reasons = Vec::new();

    if let Ok(parsed) = Url::parse(url) {
        let query_count = parsed.query_pairs().count();
        if query_count >= 4 {
            score += 0.15;
            reasons.push("URL_QUERY_DENSITY_PENALTY".to_string());
        }
        if query_count >= 8 {
            score += 0.1;
            reasons.push("URL_EXCESSIVE_QUERY_PENALTY".to_string());
        }

        let tracking_hits = parsed
            .query_pairs()
            .filter(|(key, _)| is_tracking_param(key.as_ref()))
            .count();
        if tracking_hits > 0 {
            score += 0.2;
            reasons.push("URL_TRACKING_PARAM_PENALTY".to_string());
        }
        if tracking_hits >= 3 {
            score += 0.1;
            reasons.push("URL_MULTIPLE_TRACKERS_PENALTY".to_string());
        }

        if let Some(host) = parsed.host_str() {
            let segments = host.split('.').count();
            if segments >= 5 {
                score += 0.1;
                reasons.push("SUSPICIOUS_SUBDOMAIN_PENALTY".to_string());
            }
        }
    } else {
        score += 0.15;
        reasons.push("INVALID_URL_PENALTY".to_string());
    }

    let title_exclamation_count = title.chars().filter(|ch| *ch == '!').count();
    if title_exclamation_count >= 3 {
        score += 0.12;
        reasons.push("TITLE_EXCESSIVE_PUNCTUATION_PENALTY".to_string());
    }

    let caps_ratio = uppercase_ratio(title);
    if caps_ratio > 0.45 {
        score += 0.12;
        reasons.push("TITLE_ALL_CAPS_RATIO_PENALTY".to_string());
    }

    let combined_text = format!(
        "{} {}",
        title.to_ascii_lowercase(),
        snippet.to_ascii_lowercase()
    );
    let keyword_hits = CLICKBAIT_KEYWORDS
        .iter()
        .filter(|keyword| combined_text.contains(**keyword))
        .count();
    if keyword_hits > 0 {
        score += 0.12;
        reasons.push("SPAM_KEYWORD_PENALTY".to_string());
    }
    if keyword_hits >= 2 {
        score += 0.08;
        reasons.push("MULTI_SPAM_KEYWORD_PENALTY".to_string());
    }

    let clamped = score.clamp(0.0, 1.0);
    SpamSignals {
        spam_risk_score: round6(clamped),
        reasons,
    }
}

fn uppercase_ratio(text: &str) -> f64 {
    let mut upper = 0usize;
    let mut alpha = 0usize;
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            alpha += 1;
            if ch.is_ascii_uppercase() {
                upper += 1;
            }
        }
    }
    if alpha == 0 {
        0.0
    } else {
        upper as f64 / alpha as f64
    }
}

fn is_tracking_param(param: &str) -> bool {
    let lower = param.to_ascii_lowercase();
    TRACKING_QUERY_PARAMS
        .iter()
        .any(|candidate| lower == *candidate)
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
