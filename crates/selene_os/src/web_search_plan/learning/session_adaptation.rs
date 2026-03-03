#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const SESSION_ADAPTATION_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineExecutionPolicy {
    pub fallback_priority: Vec<String>,
    pub retry_attempts: u8,
    pub cooldown_failures_before: u8,
    pub open_budget_per_query: u8,
}

impl Default for BaselineExecutionPolicy {
    fn default() -> Self {
        Self {
            fallback_priority: vec![
                "brave_web_search".to_string(),
                "openai_web_search".to_string(),
            ],
            retry_attempts: 3,
            cooldown_failures_before: 2,
            open_budget_per_query: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptationRequest {
    pub reordered_fallback_priority: Option<Vec<String>>,
    pub retry_attempts: Option<u8>,
    pub cooldown_failures_before: Option<u8>,
    pub open_budget_per_query: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptedSessionPolicy {
    pub fallback_priority: Vec<String>,
    pub retry_attempts: u8,
    pub cooldown_failures_before: u8,
    pub open_budget_per_query: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAdaptation {
    pub session_id: String,
    pub adaptation_id: String,
    pub failure_signature_id: String,
    pub policy: AdaptedSessionPolicy,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptationError {
    FallbackPriorityExpansion,
    RetryIncreaseBlocked,
    CooldownRelaxationBlocked,
    OpenBudgetIncreaseBlocked,
    InvalidBaseline,
}

impl SessionAdaptation {
    pub fn is_active(&self, now_ms: i64, session_active: bool) -> bool {
        session_active && now_ms <= self.expires_at_ms
    }
}

pub fn apply_bounded_adaptation(
    session_id: &str,
    failure_signature_id: &str,
    baseline: &BaselineExecutionPolicy,
    request: &AdaptationRequest,
    now_ms: i64,
    ttl_ms: i64,
) -> Result<SessionAdaptation, AdaptationError> {
    if baseline.retry_attempts == 0
        || baseline.cooldown_failures_before == 0
        || baseline.open_budget_per_query == 0
        || baseline.fallback_priority.is_empty()
    {
        return Err(AdaptationError::InvalidBaseline);
    }

    let fallback_priority = match &request.reordered_fallback_priority {
        Some(priority) => validate_reordered_priority(&baseline.fallback_priority, priority)?,
        None => baseline.fallback_priority.clone(),
    };

    let retry_attempts = request.retry_attempts.unwrap_or(baseline.retry_attempts);
    if retry_attempts == 0 || retry_attempts > baseline.retry_attempts {
        return Err(AdaptationError::RetryIncreaseBlocked);
    }

    let cooldown_failures_before = request
        .cooldown_failures_before
        .unwrap_or(baseline.cooldown_failures_before);
    if cooldown_failures_before == 0 || cooldown_failures_before > baseline.cooldown_failures_before
    {
        return Err(AdaptationError::CooldownRelaxationBlocked);
    }

    let open_budget_per_query = request
        .open_budget_per_query
        .unwrap_or(baseline.open_budget_per_query);
    if open_budget_per_query == 0 || open_budget_per_query > baseline.open_budget_per_query {
        return Err(AdaptationError::OpenBudgetIncreaseBlocked);
    }

    let adapted_policy = AdaptedSessionPolicy {
        fallback_priority,
        retry_attempts,
        cooldown_failures_before,
        open_budget_per_query,
    };

    Ok(SessionAdaptation {
        session_id: session_id.to_string(),
        adaptation_id: adaptation_id(session_id, failure_signature_id, &adapted_policy),
        failure_signature_id: failure_signature_id.to_string(),
        policy: adapted_policy,
        created_at_ms: now_ms,
        expires_at_ms: now_ms.saturating_add(ttl_ms.max(0)),
        version: SESSION_ADAPTATION_VERSION.to_string(),
    })
}

pub fn adaptation_id(
    session_id: &str,
    failure_signature_id: &str,
    policy: &AdaptedSessionPolicy,
) -> String {
    let material = format!(
        "v={}|session_id={}|failure_signature_id={}|fallback={}|retry={}|cooldown={}|open_budget={}",
        SESSION_ADAPTATION_VERSION,
        session_id,
        failure_signature_id,
        policy.fallback_priority.join(","),
        policy.retry_attempts,
        policy.cooldown_failures_before,
        policy.open_budget_per_query,
    );

    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_reordered_priority(
    baseline: &[String],
    candidate: &[String],
) -> Result<Vec<String>, AdaptationError> {
    if baseline.len() != candidate.len() {
        return Err(AdaptationError::FallbackPriorityExpansion);
    }

    let mut baseline_sorted = baseline.to_vec();
    baseline_sorted.sort();
    let mut candidate_sorted = candidate.to_vec();
    candidate_sorted.sort();

    if baseline_sorted != candidate_sorted {
        return Err(AdaptationError::FallbackPriorityExpansion);
    }

    Ok(candidate.to_vec())
}
