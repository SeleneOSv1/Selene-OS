#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};
use sha2::{Digest, Sha256};

pub const SESSION_ADAPTATION_SCHEMA_VERSION: &str = "1.0.0";
pub const SESSION_ADAPTATION_ID_VERSION: &str = "1.0.0";
pub const MAX_ADAPTIVE_RETRY_ATTEMPTS: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBaselinePolicy {
    pub session_id: String,
    pub importance_tier: ImportanceTier,
    pub lead_provider: Option<String>,
    pub fallback_priority: Vec<String>,
    pub retry_attempts: usize,
    pub cooldown_failures_before: u32,
    pub open_budget_per_query: usize,
    pub max_provider_fan_out: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptationRequest {
    pub reordered_fallback_priority: Option<Vec<String>>,
    pub retry_attempts: Option<usize>,
    pub cooldown_failures_before: Option<u32>,
    pub skip_lead_provider: bool,
    pub reduce_open_budget_per_query: Option<usize>,
    pub force_snippet_only: bool,
    pub provider_fan_out_override: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAdaptedPolicy {
    pub importance_tier: ImportanceTier,
    pub lead_provider: Option<String>,
    pub fallback_priority: Vec<String>,
    pub retry_attempts: usize,
    pub cooldown_failures_before: u32,
    pub open_budget_per_query: usize,
    pub snippet_only_mode: bool,
    pub max_provider_fan_out: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAdaptation {
    pub session_id: String,
    pub failure_signature_id: String,
    pub adaptation_id: String,
    pub policy: SessionAdaptedPolicy,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdaptationError {
    InvalidBaseline,
    FallbackExpansionBlocked,
    RetryIncreaseBlocked,
    RetryStormNotStopped,
    CooldownRelaxationBlocked,
    OpenBudgetIncreaseBlocked,
    ProviderFanoutChangeBlocked,
}

impl SessionBaselinePolicy {
    pub fn for_tier(
        session_id: impl Into<String>,
        importance_tier: ImportanceTier,
        lead_provider: Option<String>,
        fallback_priority: Vec<String>,
    ) -> Self {
        let caps = caps_for_tier(importance_tier);
        let fallback_count = fallback_priority.len().max(1);
        Self {
            session_id: session_id.into(),
            importance_tier,
            lead_provider,
            fallback_priority,
            retry_attempts: caps.max_retries_per_provider.max(1),
            cooldown_failures_before: 2,
            open_budget_per_query: caps.max_urls_opened_per_query,
            max_provider_fan_out: fallback_count,
        }
    }
}

impl SessionAdaptation {
    pub fn is_active(&self, now_ms: i64, session_active: bool) -> bool {
        session_active && now_ms <= self.expires_at_ms
    }
}

pub fn apply_bounded_adaptation(
    failure_signature_id: &str,
    baseline: &SessionBaselinePolicy,
    request: &AdaptationRequest,
    now_ms: i64,
    ttl_ms: i64,
) -> Result<SessionAdaptation, AdaptationError> {
    if baseline.fallback_priority.is_empty()
        || baseline.cooldown_failures_before == 0
        || baseline.open_budget_per_query == 0
        || baseline.max_provider_fan_out == 0
    {
        return Err(AdaptationError::InvalidBaseline);
    }

    if request.provider_fan_out_override.is_some() {
        return Err(AdaptationError::ProviderFanoutChangeBlocked);
    }

    let fallback_priority = match &request.reordered_fallback_priority {
        Some(candidate) => validate_reordered_fallback_priority(
            baseline.fallback_priority.as_slice(),
            candidate.as_slice(),
        )?,
        None => baseline.fallback_priority.clone(),
    };

    let retry_attempts = request.retry_attempts.unwrap_or(baseline.retry_attempts);
    if retry_attempts > baseline.retry_attempts {
        return Err(AdaptationError::RetryIncreaseBlocked);
    }
    if retry_attempts > MAX_ADAPTIVE_RETRY_ATTEMPTS {
        return Err(AdaptationError::RetryStormNotStopped);
    }

    let cooldown_failures_before = request
        .cooldown_failures_before
        .unwrap_or(baseline.cooldown_failures_before);
    if cooldown_failures_before == 0 || cooldown_failures_before > baseline.cooldown_failures_before
    {
        return Err(AdaptationError::CooldownRelaxationBlocked);
    }

    let open_budget_per_query = request
        .reduce_open_budget_per_query
        .unwrap_or(baseline.open_budget_per_query);
    let tier_caps = caps_for_tier(baseline.importance_tier);
    if open_budget_per_query > baseline.open_budget_per_query
        || open_budget_per_query > tier_caps.max_urls_opened_per_query
    {
        return Err(AdaptationError::OpenBudgetIncreaseBlocked);
    }

    let lead_provider = if request.skip_lead_provider {
        None
    } else {
        baseline.lead_provider.clone()
    };

    let adapted_policy = SessionAdaptedPolicy {
        importance_tier: baseline.importance_tier,
        lead_provider,
        fallback_priority,
        retry_attempts,
        cooldown_failures_before,
        open_budget_per_query,
        snippet_only_mode: request.force_snippet_only,
        max_provider_fan_out: baseline.max_provider_fan_out,
    };

    Ok(SessionAdaptation {
        session_id: baseline.session_id.clone(),
        failure_signature_id: failure_signature_id.to_string(),
        adaptation_id: adaptation_id(
            baseline.session_id.as_str(),
            failure_signature_id,
            &adapted_policy,
        ),
        policy: adapted_policy,
        created_at_ms: now_ms,
        expires_at_ms: now_ms.saturating_add(ttl_ms.max(0)),
        schema_version: SESSION_ADAPTATION_SCHEMA_VERSION.to_string(),
    })
}

pub fn adaptation_id(
    session_id: &str,
    failure_signature_id: &str,
    policy: &SessionAdaptedPolicy,
) -> String {
    let lead_provider = policy
        .lead_provider
        .as_ref()
        .map(|value| value.as_str())
        .unwrap_or("none");
    let material = format!(
        "version={}|session_id={}|failure_signature_id={}|importance_tier={}|lead_provider={}|fallback_priority={}|retry_attempts={}|cooldown_failures_before={}|open_budget_per_query={}|snippet_only_mode={}|max_provider_fan_out={}",
        SESSION_ADAPTATION_ID_VERSION,
        session_id.trim(),
        failure_signature_id.trim(),
        policy.importance_tier.as_str(),
        lead_provider.trim(),
        policy.fallback_priority.join(","),
        policy.retry_attempts,
        policy.cooldown_failures_before,
        policy.open_budget_per_query,
        policy.snippet_only_mode,
        policy.max_provider_fan_out
    );
    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_reordered_fallback_priority(
    baseline: &[String],
    candidate: &[String],
) -> Result<Vec<String>, AdaptationError> {
    if baseline.len() != candidate.len() {
        return Err(AdaptationError::FallbackExpansionBlocked);
    }

    let mut baseline_sorted = baseline
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<String>>();
    baseline_sorted.sort();
    let mut candidate_sorted = candidate
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<String>>();
    candidate_sorted.sort();

    if baseline_sorted != candidate_sorted {
        return Err(AdaptationError::FallbackExpansionBlocked);
    }

    Ok(candidate.to_vec())
}
