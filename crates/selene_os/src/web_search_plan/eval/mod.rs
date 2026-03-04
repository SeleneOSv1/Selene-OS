#![forbid(unsafe_code)]

pub mod corpus_packs;
pub mod metrics;
pub mod report;
pub mod thresholds;

pub use corpus_packs::{
    load_corpus_pack, load_default_corpus_packs, validate_corpus_pack, EvalCase, EvalCorpusPack,
    DEFAULT_CORPUS_PACK_FILES,
};
pub use metrics::{evaluate_case, evaluate_cases, CaseEvaluation, CaseMetrics};
pub use report::{generate_eval_report, ContinuousEvalConfig, ContinuousEvalOutcome, EvalReport};
pub use thresholds::{
    evaluate_thresholds, load_thresholds, validate_thresholds, EvalThresholds, ThresholdOutcome,
};

#[cfg(test)]
pub mod eval_tests;
