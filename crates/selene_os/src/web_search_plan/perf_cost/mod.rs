#![forbid(unsafe_code)]

pub mod audit_fields;
pub mod budgets;
pub mod concurrency;
pub mod degrade;
pub mod tiers;
pub mod timeouts;

use crate::web_search_plan::perf_cost::budgets::{budget_plan_for_tier, BudgetPlan};
use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier, TierCaps};
use crate::web_search_plan::perf_cost::timeouts::{timeout_envelope_for_tier, TimeoutEnvelope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfCostPolicySnapshot {
    pub tier: ImportanceTier,
    pub caps: TierCaps,
    pub budget_plan: BudgetPlan,
    pub timeout_envelope: TimeoutEnvelope,
}

impl PerfCostPolicySnapshot {
    pub fn from_importance_tier_str(raw: &str) -> Self {
        let tier = ImportanceTier::parse_or_default(raw);
        Self::from_tier(tier)
    }

    pub fn from_tier(tier: ImportanceTier) -> Self {
        Self {
            tier,
            caps: caps_for_tier(tier),
            budget_plan: budget_plan_for_tier(tier),
            timeout_envelope: timeout_envelope_for_tier(tier),
        }
    }
}

pub fn enforce_url_open_cap(url_open_ordinal: usize, url_open_cap: Option<usize>) -> Result<(), &'static str> {
    match url_open_cap {
        Some(cap) if url_open_ordinal >= cap => Err("budget_exhausted"),
        _ => Ok(()),
    }
}

#[cfg(test)]
pub mod perf_cost_tests;
