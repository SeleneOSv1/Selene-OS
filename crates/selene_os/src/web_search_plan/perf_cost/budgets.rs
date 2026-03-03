#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    X,
    Search,
    E,
    D,
    Write,
    Tts,
}

impl Stage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Search => "SEARCH",
            Self::E => "E",
            Self::D => "D",
            Self::Write => "WRITE",
            Self::Tts => "TTS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageDeadlinesMs {
    pub x: u64,
    pub search: u64,
    pub e: u64,
    pub d: u64,
    pub write: u64,
    pub tts: u64,
}

impl StageDeadlinesMs {
    pub const fn total(self) -> u64 {
        self.x + self.search + self.e + self.d + self.write + self.tts
    }

    pub const fn for_stage(self, stage: Stage) -> u64 {
        match stage {
            Stage::X => self.x,
            Stage::Search => self.search,
            Stage::E => self.e,
            Stage::D => self.d,
            Stage::Write => self.write,
            Stage::Tts => self.tts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetPlan {
    pub tier: ImportanceTier,
    pub stage_deadlines_ms: StageDeadlinesMs,
    pub absolute_deadline_ms: u64,
}

pub fn budget_plan_for_tier(tier: ImportanceTier) -> BudgetPlan {
    let stage_deadlines_ms = match tier {
        ImportanceTier::Low => StageDeadlinesMs {
            x: 250,
            search: 450,
            e: 1_500,
            d: 800,
            write: 300,
            tts: 200,
        },
        ImportanceTier::Medium => StageDeadlinesMs {
            x: 400,
            search: 800,
            e: 3_200,
            d: 1_500,
            write: 700,
            tts: 400,
        },
        ImportanceTier::High => StageDeadlinesMs {
            x: 600,
            search: 1_200,
            e: 5_600,
            d: 2_500,
            write: 1_300,
            tts: 800,
        },
    };

    let absolute_deadline_ms = caps_for_tier(tier).total_timeout_per_turn_ms;
    BudgetPlan {
        tier,
        stage_deadlines_ms,
        absolute_deadline_ms,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetViolation {
    pub stage: Stage,
    pub elapsed_ms: u64,
    pub deadline_ms: u64,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone)]
pub struct StageBudgetTracker {
    plan: BudgetPlan,
    stage_timings_ms: BTreeMap<Stage, u64>,
}

impl StageBudgetTracker {
    pub fn new(plan: BudgetPlan) -> Self {
        Self {
            plan,
            stage_timings_ms: BTreeMap::new(),
        }
    }

    pub const fn plan(&self) -> BudgetPlan {
        self.plan
    }

    pub fn record_stage_timing(&mut self, stage: Stage, elapsed_ms: u64) -> Result<(), BudgetViolation> {
        let stage_deadline = self.plan.stage_deadlines_ms.for_stage(stage);
        if elapsed_ms > stage_deadline {
            return Err(BudgetViolation {
                stage,
                elapsed_ms,
                deadline_ms: stage_deadline,
                reason_code: "budget_exhausted",
            });
        }

        let projected_total = self
            .total_elapsed_ms()
            .saturating_sub(*self.stage_timings_ms.get(&stage).unwrap_or(&0))
            .saturating_add(elapsed_ms);
        if projected_total > self.plan.absolute_deadline_ms {
            return Err(BudgetViolation {
                stage,
                elapsed_ms: projected_total,
                deadline_ms: self.plan.absolute_deadline_ms,
                reason_code: "timeout_exceeded",
            });
        }

        self.stage_timings_ms.insert(stage, elapsed_ms);
        Ok(())
    }

    pub fn stage_timings_ms(&self) -> &BTreeMap<Stage, u64> {
        &self.stage_timings_ms
    }

    pub fn total_elapsed_ms(&self) -> u64 {
        self.stage_timings_ms.values().copied().sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCallBudget {
    pub max_total_provider_calls_per_turn: usize,
    pub max_fallback_invocations_per_turn: usize,
    pub max_retries_per_provider: usize,
    total_provider_calls: usize,
    fallback_invocations: usize,
}

impl ProviderCallBudget {
    pub fn for_tier(tier: ImportanceTier) -> Self {
        let caps = caps_for_tier(tier);
        Self {
            max_total_provider_calls_per_turn: caps.max_total_provider_calls_per_turn,
            max_fallback_invocations_per_turn: caps.max_fallback_invocations_per_turn,
            max_retries_per_provider: caps.max_retries_per_provider,
            total_provider_calls: 0,
            fallback_invocations: 0,
        }
    }

    pub fn record_lead_call(&mut self) -> Result<(), &'static str> {
        self.total_provider_calls = self.total_provider_calls.saturating_add(1);
        if self.total_provider_calls > self.max_total_provider_calls_per_turn {
            return Err("budget_exhausted");
        }
        Ok(())
    }

    pub fn record_fallback_call(&mut self) -> Result<(), &'static str> {
        self.total_provider_calls = self.total_provider_calls.saturating_add(1);
        self.fallback_invocations = self.fallback_invocations.saturating_add(1);

        if self.fallback_invocations > self.max_fallback_invocations_per_turn {
            return Err("budget_exhausted");
        }
        if self.total_provider_calls > self.max_total_provider_calls_per_turn {
            return Err("budget_exhausted");
        }
        Ok(())
    }

    pub fn validate_retry_count(&self, retries: usize) -> Result<(), &'static str> {
        if retries > self.max_retries_per_provider {
            Err("budget_exhausted")
        } else {
            Ok(())
        }
    }

    pub const fn total_provider_calls(&self) -> usize {
        self.total_provider_calls
    }

    pub const fn fallback_invocations(&self) -> usize {
        self.fallback_invocations
    }
}
