#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier, TierCaps};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradeStep {
    ReduceMaxResultsFromSearchToTierMinimum,
    ReduceMaxUrlsOpenedPerQueryToOne,
    DisableUrlOpensSnippetOnly,
    FailClosed,
}

impl DegradeStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReduceMaxResultsFromSearchToTierMinimum => {
                "reduce_max_results_from_search_to_tier_minimum"
            }
            Self::ReduceMaxUrlsOpenedPerQueryToOne => "reduce_max_urls_opened_per_query_to_one",
            Self::DisableUrlOpensSnippetOnly => "disable_url_opens_snippet_only_mode",
            Self::FailClosed => "fail_closed",
        }
    }
}

pub const DEGRADE_ORDER: [DegradeStep; 4] = [
    DegradeStep::ReduceMaxResultsFromSearchToTierMinimum,
    DegradeStep::ReduceMaxUrlsOpenedPerQueryToOne,
    DegradeStep::DisableUrlOpensSnippetOnly,
    DegradeStep::FailClosed,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionCaps {
    pub max_results_from_search: usize,
    pub max_urls_opened_per_query: usize,
    pub snippet_only_mode: bool,
}

impl ExecutionCaps {
    pub const fn from_tier(tier: ImportanceTier) -> Self {
        let caps = caps_for_tier(tier);
        Self {
            max_results_from_search: caps.max_results_from_search,
            max_urls_opened_per_query: caps.max_urls_opened_per_query,
            snippet_only_mode: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DegradeDecision {
    pub degraded: bool,
    pub step: Option<DegradeStep>,
    pub fail_closed: bool,
    pub execution_caps: ExecutionCaps,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DegradeController {
    tier: ImportanceTier,
    next_step_index: usize,
    current_caps: ExecutionCaps,
}

impl DegradeController {
    pub const fn new(tier: ImportanceTier) -> Self {
        Self {
            tier,
            next_step_index: 0,
            current_caps: ExecutionCaps::from_tier(tier),
        }
    }

    pub const fn current_caps(&self) -> ExecutionCaps {
        self.current_caps
    }

    pub fn advance(&mut self) -> DegradeDecision {
        let step = DEGRADE_ORDER
            .get(self.next_step_index)
            .copied()
            .unwrap_or(DegradeStep::FailClosed);

        self.next_step_index = self.next_step_index.saturating_add(1);
        self.current_caps = apply_degrade_step(self.current_caps, self.tier, step);

        DegradeDecision {
            degraded: true,
            step: Some(step),
            fail_closed: matches!(step, DegradeStep::FailClosed),
            execution_caps: self.current_caps,
            reason_code: "budget_exhausted",
        }
    }
}

pub const fn apply_degrade_step(
    mut current: ExecutionCaps,
    _tier: ImportanceTier,
    step: DegradeStep,
) -> ExecutionCaps {
    match step {
        DegradeStep::ReduceMaxResultsFromSearchToTierMinimum => {
            current.max_results_from_search = TierCaps::minimum_search_results();
        }
        DegradeStep::ReduceMaxUrlsOpenedPerQueryToOne => {
            current.max_urls_opened_per_query = 1;
        }
        DegradeStep::DisableUrlOpensSnippetOnly => {
            current.max_urls_opened_per_query = 0;
            current.snippet_only_mode = true;
        }
        DegradeStep::FailClosed => {}
    }

    current
}
