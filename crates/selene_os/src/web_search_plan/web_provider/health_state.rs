#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealthState {
    Healthy,
    Degraded,
    Cooldown,
}

impl ProviderHealthState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Cooldown => "cooldown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthPolicy {
    pub failures_before_cooldown: u32,
    pub cooldown_ms: i64,
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            failures_before_cooldown: 2,
            cooldown_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct HealthRecord {
    consecutive_failures: u32,
    cooldown_until_ms: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderHealthTracker {
    records: BTreeMap<String, HealthRecord>,
}

impl ProviderHealthTracker {
    pub fn snapshot(
        &self,
        provider_id: &str,
        now_ms: i64,
        policy: HealthPolicy,
    ) -> ProviderHealthState {
        let Some(record) = self.records.get(provider_id) else {
            return ProviderHealthState::Healthy;
        };

        if let Some(until) = record.cooldown_until_ms {
            if now_ms < until {
                return ProviderHealthState::Cooldown;
            }
        }

        if record.consecutive_failures >= policy.failures_before_cooldown {
            ProviderHealthState::Cooldown
        } else if record.consecutive_failures > 0 {
            ProviderHealthState::Degraded
        } else {
            ProviderHealthState::Healthy
        }
    }

    pub fn should_skip_lead(&self, provider_id: &str, now_ms: i64, policy: HealthPolicy) -> bool {
        matches!(
            self.snapshot(provider_id, now_ms, policy),
            ProviderHealthState::Cooldown
        )
    }

    pub fn record_success(&mut self, provider_id: &str) {
        self.records.remove(provider_id);
    }

    pub fn record_failure(&mut self, provider_id: &str, now_ms: i64, policy: HealthPolicy) {
        let record = self.records.entry(provider_id.to_string()).or_default();
        record.consecutive_failures = record.consecutive_failures.saturating_add(1);

        if record.consecutive_failures >= policy.failures_before_cooldown {
            record.cooldown_until_ms = Some(now_ms.saturating_add(policy.cooldown_ms));
        }
    }
}
