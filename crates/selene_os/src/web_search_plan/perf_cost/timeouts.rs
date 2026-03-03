#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutEnvelope {
    pub per_provider_timeout_ms: u64,
    pub total_timeout_per_turn_ms: u64,
    pub url_fetch_total_timeout_ms: u64,
}

pub fn timeout_envelope_for_tier(tier: ImportanceTier) -> TimeoutEnvelope {
    let caps = caps_for_tier(tier);
    TimeoutEnvelope {
        per_provider_timeout_ms: caps.timeout_per_provider_ms,
        total_timeout_per_turn_ms: caps.total_timeout_per_turn_ms,
        url_fetch_total_timeout_ms: caps.url_fetch_total_timeout_ms,
    }
}

pub fn clamp_provider_timeout(requested_timeout_ms: u64, tier: ImportanceTier) -> u64 {
    let envelope = timeout_envelope_for_tier(tier);
    requested_timeout_ms
        .min(envelope.per_provider_timeout_ms)
        .min(envelope.total_timeout_per_turn_ms)
}

pub fn clamp_url_fetch_total_timeout(requested_timeout_ms: u64, tier: ImportanceTier) -> u64 {
    let envelope = timeout_envelope_for_tier(tier);
    requested_timeout_ms
        .min(envelope.url_fetch_total_timeout_ms)
        .min(envelope.total_timeout_per_turn_ms)
}

pub fn ensure_turn_timeout_not_exceeded(
    elapsed_ms: u64,
    tier: ImportanceTier,
) -> Result<(), &'static str> {
    let envelope = timeout_envelope_for_tier(tier);
    if elapsed_ms > envelope.total_timeout_per_turn_ms {
        Err("timeout_exceeded")
    } else {
        Ok(())
    }
}
