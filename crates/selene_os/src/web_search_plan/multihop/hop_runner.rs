#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::multihop::cycle_detect::CycleDetector;
use crate::web_search_plan::multihop::hop_audit::{build_hop_proof_chain, can_mark_complete, HopProofChain};
use crate::web_search_plan::multihop::hop_budget::{HopBudget, HopBudgetTracker};
use crate::web_search_plan::multihop::hop_plan::{Hop, HopPlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRunSummary {
    pub provider_id: String,
    pub endpoint: String,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopExecutionOutput {
    pub provider_runs: Vec<ProviderRunSummary>,
    pub source_urls: Vec<String>,
    pub elapsed_ms: u64,
    pub provider_calls: usize,
    pub url_opens: usize,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopExecutionError {
    pub reason_code: String,
    pub message: String,
    pub elapsed_ms: u64,
    pub provider_calls: usize,
    pub url_opens: usize,
}

impl HopExecutionError {
    pub fn new(
        reason_code: impl Into<String>,
        message: impl Into<String>,
        elapsed_ms: u64,
        provider_calls: usize,
        url_opens: usize,
    ) -> Self {
        Self {
            reason_code: reason_code.into(),
            message: message.into(),
            elapsed_ms,
            provider_calls,
            url_opens,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopExecutionRecord {
    pub hop_index: usize,
    pub query: String,
    pub provider_runs: Vec<ProviderRunSummary>,
    pub source_urls: Vec<String>,
    pub evidence_hash: String,
    pub success: bool,
    pub reason_code: Option<String>,
    pub time_spent_ms: u64,
}

pub trait HopExecutor {
    fn execute(&mut self, hop: &Hop) -> Result<HopExecutionOutput, HopExecutionError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopRunResult {
    pub plan_id: String,
    pub hop_records: Vec<HopExecutionRecord>,
    pub stop_reason: String,
    pub reason_codes: Vec<String>,
    pub proof_chain: HopProofChain,
    pub cycle_detected: bool,
    pub final_answer_allowed: bool,
}

pub fn execute_hop_plan(
    plan: &HopPlan,
    budget: HopBudget,
    executor: &mut impl HopExecutor,
) -> Result<HopRunResult, String> {
    let mut tracker = HopBudgetTracker::new(budget);
    let mut cycle_detector = CycleDetector::default();
    let mut records = Vec::new();
    let mut reason_codes = Vec::new();
    let mut stop_reason = "success".to_string();
    let mut cycle_detected = false;

    for hop in &plan.hops {
        if let Err(err) = tracker.check_hop_start(hop.hop_index) {
            stop_reason = err.reason_code.to_string();
            push_reason_code(&mut reason_codes, err.reason_code);
            push_reason_code(&mut reason_codes, "insufficient_evidence");
            break;
        }

        if let Err(err) = cycle_detector.register_sub_query(hop.sub_query.as_str()) {
            stop_reason = err.reason_code.to_string();
            cycle_detected = true;
            push_reason_code(&mut reason_codes, err.reason_code);
            break;
        }

        match executor.execute(hop) {
            Ok(output) => {
                let mut cycle_violation: Option<String> = None;
                for canonical_url in &output.source_urls {
                    if let Err(err) = cycle_detector.register_canonical_url(canonical_url.as_str()) {
                        cycle_violation = Some(err.reason_code.to_string());
                        cycle_detected = true;
                        break;
                    }
                }

                let mut reason_code = output.reason_code.clone();
                if reason_code.is_none() {
                    reason_code = cycle_violation.clone();
                }
                let success = reason_code.is_none();

                let evidence_hash = hash_hop_evidence(hop, &output.provider_runs, &output.source_urls);
                records.push(HopExecutionRecord {
                    hop_index: hop.hop_index,
                    query: hop.sub_query.clone(),
                    provider_runs: output.provider_runs.clone(),
                    source_urls: output.source_urls.clone(),
                    evidence_hash,
                    success,
                    reason_code: reason_code.clone(),
                    time_spent_ms: output.elapsed_ms,
                });

                if let Err(err) =
                    tracker.record_hop_usage(output.elapsed_ms, output.provider_calls, output.url_opens)
                {
                    stop_reason = err.reason_code.to_string();
                    push_reason_code(&mut reason_codes, err.reason_code);
                    push_reason_code(&mut reason_codes, "insufficient_evidence");
                    break;
                }

                if let Some(code) = reason_code {
                    stop_reason = code.clone();
                    push_reason_code(&mut reason_codes, code.as_str());
                    if code == "budget_exhausted" || code == "timeout_exceeded" {
                        push_reason_code(&mut reason_codes, "insufficient_evidence");
                    }
                    break;
                }
            }
            Err(err) => {
                if let Err(budget_err) =
                    tracker.record_hop_usage(err.elapsed_ms, err.provider_calls, err.url_opens)
                {
                    stop_reason = budget_err.reason_code.to_string();
                    push_reason_code(&mut reason_codes, budget_err.reason_code);
                    push_reason_code(&mut reason_codes, "insufficient_evidence");
                } else {
                    stop_reason = err.reason_code.clone();
                    push_reason_code(&mut reason_codes, err.reason_code.as_str());
                }

                records.push(HopExecutionRecord {
                    hop_index: hop.hop_index,
                    query: hop.sub_query.clone(),
                    provider_runs: Vec::new(),
                    source_urls: Vec::new(),
                    evidence_hash: hash_hop_evidence(hop, &[], &[]),
                    success: false,
                    reason_code: Some(stop_reason.clone()),
                    time_spent_ms: err.elapsed_ms,
                });
                break;
            }
        }
    }

    if (stop_reason == "budget_exhausted" || stop_reason == "timeout_exceeded")
        && !reason_codes.iter().any(|code| code == "insufficient_evidence")
    {
        push_reason_code(&mut reason_codes, "insufficient_evidence");
    }

    let proof_chain = build_hop_proof_chain(
        plan,
        records.as_slice(),
        stop_reason.as_str(),
        reason_codes.as_slice(),
        cycle_detected,
    )?;
    let final_answer_allowed = can_mark_complete(&proof_chain);

    Ok(HopRunResult {
        plan_id: plan.plan_id.clone(),
        hop_records: records,
        stop_reason,
        reason_codes,
        proof_chain,
        cycle_detected,
        final_answer_allowed,
    })
}

fn push_reason_code(reason_codes: &mut Vec<String>, reason_code: &str) {
    if !reason_codes.iter().any(|code| code == reason_code) {
        reason_codes.push(reason_code.to_string());
    }
}

fn hash_hop_evidence(
    hop: &Hop,
    provider_runs: &[ProviderRunSummary],
    source_urls: &[String],
) -> String {
    let provider_serialized = provider_runs
        .iter()
        .map(|run| format!("{}|{}|{}", run.provider_id, run.endpoint, run.success))
        .collect::<Vec<String>>()
        .join("\x1e");
    let source_serialized = source_urls.join("\x1e");
    let material = format!(
        "hop_index={}\x1fquery={}\x1fprovider_runs={}\x1fsource_urls={}",
        hop.hop_index, hop.sub_query, provider_serialized, source_serialized
    );
    sha256_hex(material.as_bytes())
}
