#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};

pub const HOP_BUDGET_POLICY_VERSION: &str = "1.0.0";
pub const DEFAULT_MAX_HOPS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopBudget {
    pub policy_version: &'static str,
    pub max_hops: usize,
    pub max_total_time_ms: u64,
    pub max_time_per_hop_ms: u64,
    pub max_provider_calls_total: usize,
    pub max_url_opens_total: usize,
}

impl HopBudget {
    pub fn for_tier(tier: ImportanceTier) -> Self {
        let caps = caps_for_tier(tier);
        let max_hops = match tier {
            ImportanceTier::Low => 3,
            ImportanceTier::Medium => 4,
            ImportanceTier::High => 5,
        };
        let max_total_time_ms = caps.total_timeout_per_turn_ms;
        let max_time_per_hop_ms = (max_total_time_ms / max_hops as u64).max(1);

        Self {
            policy_version: HOP_BUDGET_POLICY_VERSION,
            max_hops,
            max_total_time_ms,
            max_time_per_hop_ms,
            max_provider_calls_total: caps.max_total_provider_calls_per_turn,
            max_url_opens_total: caps.max_queries.saturating_mul(caps.max_urls_opened_per_query),
        }
    }

    pub fn from_importance_tier(raw: &str) -> Self {
        Self::for_tier(ImportanceTier::parse_or_default(raw))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopBudgetError {
    pub reason_code: &'static str,
    pub message: String,
}

impl HopBudgetError {
    fn budget_exhausted(message: impl Into<String>) -> Self {
        Self {
            reason_code: "budget_exhausted",
            message: message.into(),
        }
    }

    fn timeout_exceeded(message: impl Into<String>) -> Self {
        Self {
            reason_code: "timeout_exceeded",
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopBudgetTracker {
    budget: HopBudget,
    hops_executed: usize,
    total_time_ms: u64,
    total_provider_calls: usize,
    total_url_opens: usize,
}

impl HopBudgetTracker {
    pub fn new(budget: HopBudget) -> Self {
        Self {
            budget,
            hops_executed: 0,
            total_time_ms: 0,
            total_provider_calls: 0,
            total_url_opens: 0,
        }
    }

    pub const fn budget(&self) -> HopBudget {
        self.budget
    }

    pub fn check_hop_start(&self, hop_index: usize) -> Result<(), HopBudgetError> {
        if hop_index >= self.budget.max_hops {
            return Err(HopBudgetError::budget_exhausted(format!(
                "hop_index {} exceeds max_hops {}",
                hop_index, self.budget.max_hops
            )));
        }
        Ok(())
    }

    pub fn record_hop_usage(
        &mut self,
        elapsed_ms: u64,
        provider_calls: usize,
        url_opens: usize,
    ) -> Result<(), HopBudgetError> {
        if elapsed_ms > self.budget.max_time_per_hop_ms {
            return Err(HopBudgetError::timeout_exceeded(format!(
                "elapsed_ms {} exceeds max_time_per_hop_ms {}",
                elapsed_ms, self.budget.max_time_per_hop_ms
            )));
        }

        let projected_total_time = self.total_time_ms.saturating_add(elapsed_ms);
        if projected_total_time > self.budget.max_total_time_ms {
            return Err(HopBudgetError::timeout_exceeded(format!(
                "total_time_ms {} exceeds max_total_time_ms {}",
                projected_total_time, self.budget.max_total_time_ms
            )));
        }

        let projected_provider_calls = self.total_provider_calls.saturating_add(provider_calls);
        if projected_provider_calls > self.budget.max_provider_calls_total {
            return Err(HopBudgetError::budget_exhausted(format!(
                "provider_calls {} exceeds max_provider_calls_total {}",
                projected_provider_calls, self.budget.max_provider_calls_total
            )));
        }

        let projected_url_opens = self.total_url_opens.saturating_add(url_opens);
        if projected_url_opens > self.budget.max_url_opens_total {
            return Err(HopBudgetError::budget_exhausted(format!(
                "url_opens {} exceeds max_url_opens_total {}",
                projected_url_opens, self.budget.max_url_opens_total
            )));
        }

        self.hops_executed = self.hops_executed.saturating_add(1);
        self.total_time_ms = projected_total_time;
        self.total_provider_calls = projected_provider_calls;
        self.total_url_opens = projected_url_opens;
        Ok(())
    }

    pub const fn hops_executed(&self) -> usize {
        self.hops_executed
    }

    pub const fn total_time_ms(&self) -> u64 {
        self.total_time_ms
    }

    pub const fn total_provider_calls(&self) -> usize {
        self.total_provider_calls
    }

    pub const fn total_url_opens(&self) -> usize {
        self.total_url_opens
    }
}
