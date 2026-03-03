#![forbid(unsafe_code)]

pub mod cycle_detect;
pub mod hop_audit;
pub mod hop_budget;
pub mod hop_plan;
pub mod hop_runner;

pub use cycle_detect::{CycleDetectionError, CycleDetector};
pub use hop_audit::{can_mark_complete, HopProof, HopProofChain};
pub use hop_budget::{HopBudget, HopBudgetError, HopBudgetTracker, DEFAULT_MAX_HOPS};
pub use hop_plan::{
    build_hop_plan, ExpectedOutput, Hop, HopMode, HopPlan, HopPlanInput, StopCondition,
};
pub use hop_runner::{
    execute_hop_plan, HopExecutionError, HopExecutionOutput, HopExecutionRecord, HopExecutor,
    HopRunResult, ProviderRunSummary,
};

#[cfg(test)]
pub mod multihop_tests;
