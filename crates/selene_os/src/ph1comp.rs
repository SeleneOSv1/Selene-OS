#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::decimal::{
    decimal_to_numeric_value, decimal_to_string, is_outlier, mad, median, round_decimal,
};
use crate::web_search_plan::analytics::packet_builder::build_computation_packet as build_analytics_computation_packet;
use crate::web_search_plan::analytics::types::{AnalyticsError, AnalyticsRequest};
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use rust_decimal::Decimal;
use selene_kernel_contracts::ph1comp::{
    Aggregate, AggregateMethod, ComputationConfidenceBucket, ComputationConfidencePosture,
    ComputationConsensusResult, ComputationConsensusStatus, ComputationExecutionState,
    ComputationFailureClass, ComputationInputs, ComputationPacket, ComputationSelectedResult,
    ConsensusCandidate, ConsensusGroup, ConsensusMethod, ConsensusOutlier, ConfidenceFactors,
    ConfidenceItem, NormalizationKind, NormalizationTraceEntry, NumericValue, PH1COMP_ENGINE_ID,
    PH1COMP_SCHEMA_VERSION,
};
use selene_kernel_contracts::ph1simfinder::MissingSimulationPacket;
use selene_kernel_contracts::{ContractViolation, RuntimeExecutionEnvelope, Validate};
use serde_json::{json, to_value, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

const FORMULA_ANALYTICS_V1: &str = "ph1.comp.analytics.v1";
const FORMULA_RANKING_V1: &str = "ph1.comp.ranking.v1";
const FORMULA_CONSENSUS_V1: &str = "ph1.comp.consensus.v1";
const FORMULA_BUDGET_V1: &str = "ph1.comp.budget.v1";
const FORMULA_MISSING_SIM_V1: &str = "ph1.comp.missing_simulation.v1";
const CANONICAL_BUDGET_UNIT: &str = "microunits";
const CANONICAL_TIME_UNIT: &str = "ms";
const CANONICAL_PERCENT_UNIT: &str = "ratio";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1CompError {
    pub class: ComputationFailureClass,
    pub message: String,
    pub reason_codes: Vec<String>,
}

impl Ph1CompError {
    fn new(
        class: ComputationFailureClass,
        message: impl Into<String>,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            class,
            message: message.into(),
            reason_codes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedScoreComponentInput {
    pub component_id: String,
    pub normalized_score_bp: u16,
    pub weight_bp: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedCandidateInput {
    pub candidate_id: String,
    pub tie_break_key: String,
    pub priority_score_bp: u16,
    pub confidence_bp: u16,
    pub threshold_bp: Option<u16>,
    pub components: Vec<WeightedScoreComponentInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusSignalInput {
    pub candidate_id: String,
    pub value: NumericValue,
    pub weight_bp: u16,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationInput {
    Currency {
        label: String,
        amount: Decimal,
        from: String,
        to: String,
        rate: Decimal,
    },
    Unit {
        label: String,
        value: Decimal,
        from: String,
        to: String,
        factor: Decimal,
    },
    TimeSeconds {
        label: String,
        seconds: i64,
    },
    TimeMilliseconds {
        label: String,
        milliseconds: i64,
    },
    PercentageBasisPoints {
        label: String,
        basis_points: u16,
    },
    PercentageRatio {
        label: String,
        ratio: Decimal,
    },
    Scale {
        label: String,
        value: Decimal,
        from_scale: u32,
        to_scale: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetQuotaComputationRequest {
    pub budget_limit_microunits: u64,
    pub budget_used_microunits: u64,
    pub reserved_microunits: u64,
    pub quota_limit: u32,
    pub quota_used: u32,
    pub threshold_bp: u16,
}

#[derive(Debug, Clone, Default)]
pub struct Ph1CompRuntime;

impl Ph1CompRuntime {
    pub fn build_analytics_computation_packet(
        &self,
        mut request: AnalyticsRequest,
    ) -> Result<ComputationPacket, AnalyticsError> {
        if request.intended_consumers.is_empty() {
            request.intended_consumers = vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
                "PH1.LAW".to_string(),
            ];
        }
        let mut packet = build_analytics_computation_packet(request)?;
        if !packet
            .inputs
            .formula_version_refs
            .iter()
            .any(|value| value == FORMULA_ANALYTICS_V1)
        {
            packet
                .inputs
                .formula_version_refs
                .insert(0, FORMULA_ANALYTICS_V1.to_string());
        }
        packet.validate().map_err(|error| {
            AnalyticsError::new(
                crate::web_search_plan::analytics::types::AnalyticsErrorKind::PolicyViolation,
                format!("canonical PH1.COMP packet invalid: {error:?}"),
            )
        })?;
        Ok(packet)
    }

    pub fn rank_candidates(
        &self,
        trace_id: impl Into<String>,
        created_at_ms: i64,
        policy_snapshot_id: impl Into<String>,
        mut candidates: Vec<RankedCandidateInput>,
    ) -> Result<ComputationPacket, Ph1CompError> {
        if candidates.is_empty() {
            return Err(Ph1CompError::new(
                ComputationFailureClass::InvalidInputSet,
                "candidate set must not be empty",
                vec!["invalid_candidate_set".to_string()],
            ));
        }

        candidates.sort_by(|left, right| {
            (left.candidate_id.as_str(), left.tie_break_key.as_str())
                .cmp(&(right.candidate_id.as_str(), right.tie_break_key.as_str()))
        });

        let mut aggregates = Vec::new();
        let mut confidence = Vec::new();
        let mut packet_reason_codes = BTreeSet::new();
        let mut scored = Vec::new();

        for candidate in &candidates {
            if candidate.components.is_empty() {
                return Err(Ph1CompError::new(
                    ComputationFailureClass::InvalidInputSet,
                    format!("candidate {} has no score components", candidate.candidate_id),
                    vec!["invalid_candidate_component_set".to_string()],
                ));
            }
            let total_weight = candidate
                .components
                .iter()
                .fold(0u32, |acc, item| acc.saturating_add(item.weight_bp as u32));
            if total_weight == 0 {
                return Err(Ph1CompError::new(
                    ComputationFailureClass::InvalidInputSet,
                    format!("candidate {} total weight must be > 0", candidate.candidate_id),
                    vec!["invalid_candidate_component_weight".to_string()],
                ));
            }
            let weighted_total = candidate.components.iter().try_fold(0u64, |acc, item| {
                acc.checked_add(item.normalized_score_bp as u64 * item.weight_bp as u64)
            });
            let Some(weighted_total) = weighted_total else {
                return Err(Ph1CompError::new(
                    ComputationFailureClass::ComputationOverflow,
                    format!("candidate {} score overflowed", candidate.candidate_id),
                    vec!["candidate_score_overflow".to_string()],
                ));
            };
            let final_score_bp = divide_round(weighted_total, total_weight as u64) as u16;
            let threshold_met = candidate
                .threshold_bp
                .map(|threshold| final_score_bp >= threshold)
                .unwrap_or(true);
            if !threshold_met {
                packet_reason_codes.insert("ranking_threshold_not_met".to_string());
            }
            let final_score_decimal = bp_to_ratio_decimal(final_score_bp);
            let confidence_decimal = bp_to_ratio_decimal(candidate.confidence_bp);
            scored.push(ScoredCandidate {
                candidate_id: candidate.candidate_id.clone(),
                tie_break_key: candidate.tie_break_key.clone(),
                priority_score_bp: candidate.priority_score_bp,
                confidence_bp: candidate.confidence_bp,
                final_score_bp,
                threshold_met,
            });
            aggregates.push(Aggregate {
                metric_id: format!("candidate.{}.final_score", candidate.candidate_id),
                entity: candidate.candidate_id.clone(),
                attribute: "final_score".to_string(),
                unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                currency: None,
                window: None,
                method: AggregateMethod::WeightedMean,
                value: decimal_to_numeric_value(final_score_decimal),
                sample_size: candidate.components.len() as u32,
                source_refs: candidate
                    .components
                    .iter()
                    .map(|item| item.component_id.clone())
                    .collect(),
                rank: None,
                threshold_met: Some(threshold_met),
                priority_score: Some(decimal_to_numeric_value(bp_to_ratio_decimal(
                    candidate.priority_score_bp,
                ))),
            });
            confidence.push(ConfidenceItem {
                claim_key: format!("candidate:{}:final_score", candidate.candidate_id),
                confidence_score: decimal_to_string(confidence_decimal),
                factors: ConfidenceFactors {
                    sample_size: candidate.components.len() as u32,
                    trust_tier_mix: decimal_to_string(confidence_decimal),
                    recency: decimal_to_string(Decimal::ONE),
                    conflict: decimal_to_string(Decimal::ONE),
                    outliers: decimal_to_string(Decimal::ONE),
                },
                bucket: Some(confidence_bucket_from_bp(candidate.confidence_bp)),
                minimum_threshold_met: Some(candidate.confidence_bp >= 5_000),
            });
        }

        scored.sort_by(compare_scored_candidates);
        for (index, candidate) in scored.iter().enumerate() {
            if let Some(aggregate) = aggregates.iter_mut().find(|aggregate| {
                aggregate.metric_id == format!("candidate.{}.final_score", candidate.candidate_id)
            }) {
                aggregate.rank = Some((index + 1) as u16);
            }
        }

        let selected = scored.first().cloned();
        let consensus_group = build_ranking_consensus_group(scored.as_slice());
        let reason_codes = packet_reason_codes.into_iter().collect::<Vec<String>>();
        let packet = ComputationPacket {
            schema_version: PH1COMP_SCHEMA_VERSION.to_string(),
            produced_by: PH1COMP_ENGINE_ID.to_string(),
            intended_consumers: vec!["PH1.LAW".to_string(), "PH1.GOV".to_string(), "PH1.J".to_string()],
            created_at_ms,
            trace_id: trace_id.into(),
            inputs: ComputationInputs {
                evidence_hash: stable_hash_ref(&json!({"kind": "ranking", "candidates": candidates.iter().map(|candidate| candidate.candidate_id.as_str()).collect::<Vec<_>>() }))?,
                policy_snapshot_id: policy_snapshot_id.into(),
                as_of_ms: None,
                input_count: candidates.len() as u32,
                input_labels: candidates.iter().map(|candidate| candidate.candidate_id.clone()).collect(),
                normalization_trace: candidates
                    .iter()
                    .flat_map(|candidate| {
                        candidate.components.iter().map(|component| NormalizationTraceEntry {
                            normalization_kind: NormalizationKind::Percentage,
                            rule_id: "bp_to_ratio".to_string(),
                            input_label: format!("{}:{}", candidate.candidate_id, component.component_id),
                            source_value: component.normalized_score_bp.to_string(),
                            normalized_value: decimal_to_string(bp_to_ratio_decimal(component.normalized_score_bp)),
                            source_unit: Some("bp".to_string()),
                            canonical_unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                            applied: true,
                        })
                    })
                    .collect(),
                formula_version_refs: vec![FORMULA_RANKING_V1.to_string()],
            },
            aggregates,
            consensus: vec![consensus_group],
            confidence,
            reason_codes,
        };
        packet
            .validate()
            .map_err(|error| Ph1CompError::new(ComputationFailureClass::InvalidInputSet, format!("ranking packet invalid: {error:?}"), vec!["invalid_ranking_packet".to_string()]))?;
        if let Some(selected) = selected {
            if !selected.threshold_met {
                return Err(Ph1CompError::new(
                    ComputationFailureClass::ConfidenceBelowThreshold,
                    "top ranked candidate failed threshold posture",
                    vec!["ranking_threshold_not_met".to_string()],
                ));
            }
        }
        Ok(packet)
    }

    pub fn evaluate_consensus(
        &self,
        trace_id: impl Into<String>,
        created_at_ms: i64,
        policy_snapshot_id: impl Into<String>,
        topic: impl Into<String>,
        signals: Vec<ConsensusSignalInput>,
        threshold_bp: u16,
    ) -> Result<ComputationPacket, Ph1CompError> {
        if signals.is_empty() {
            return Err(Ph1CompError::new(
                ComputationFailureClass::InvalidInputSet,
                "consensus input set must not be empty",
                vec!["invalid_consensus_input_set".to_string()],
            ));
        }
        let topic = topic.into();
        let mut numeric_signals = Vec::new();
        for signal in &signals {
            let Some(decimal_value) = numeric_value_to_decimal(&signal.value) else {
                return Err(Ph1CompError::new(
                    ComputationFailureClass::InvalidInputSet,
                    format!("signal {} is not numeric", signal.candidate_id),
                    vec!["non_numeric_consensus_signal".to_string()],
                ));
            };
            numeric_signals.push((signal.clone(), decimal_value));
        }

        let mut ordered_values = numeric_signals.iter().map(|(_, value)| *value).collect::<Vec<_>>();
        ordered_values.sort();
        let center = median(ordered_values.as_slice());
        let mad_value = mad(ordered_values.as_slice());

        let mut outliers = Vec::new();
        let mut vote_map: BTreeMap<String, ConsensusAccumulator> = BTreeMap::new();
        let mut total_weight = 0u32;

        for (signal, decimal_value) in numeric_signals {
            let outlier = is_outlier(decimal_value, center, mad_value);
            if outlier {
                outliers.push(ConsensusOutlier {
                    value: signal.value.clone(),
                    sources: vec![signal.source_ref.clone()],
                    decision: Some("excluded_as_outlier".to_string()),
                });
                continue;
            }
            total_weight = total_weight.saturating_add(signal.weight_bp as u32);
            let key = format!("{}::{}", signal.candidate_id, numeric_key(&signal.value));
            let entry = vote_map.entry(key).or_insert_with(|| ConsensusAccumulator {
                candidate_id: signal.candidate_id.clone(),
                value: signal.value.clone(),
                weight_bp: 0,
                votes: 0,
                sources: BTreeSet::new(),
            });
            entry.weight_bp = entry.weight_bp.saturating_add(signal.weight_bp as u32);
            entry.votes = entry.votes.saturating_add(1);
            entry.sources.insert(signal.source_ref);
        }

        let mut ranked = vote_map.into_values().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right.weight_bp
                .cmp(&left.weight_bp)
                .then(right.votes.cmp(&left.votes))
                .then(left.candidate_id.cmp(&right.candidate_id))
                .then(numeric_key(&left.value).cmp(&numeric_key(&right.value)))
        });

        if ranked.is_empty() {
            return Err(Ph1CompError::new(
                ComputationFailureClass::OutlierHandlingFailure,
                "all consensus inputs were excluded as outliers",
                vec!["consensus_all_outliers".to_string()],
            ));
        }

        let best = &ranked[0];
        let agreement_bp = if total_weight == 0 {
            0
        } else {
            divide_round((best.weight_bp as u64) * 10_000, total_weight as u64) as u16
        };
        let threshold_met = agreement_bp >= threshold_bp;
        if !threshold_met {
            return Err(Ph1CompError::new(
                ComputationFailureClass::ConsensusUnresolved,
                "weighted consensus did not reach threshold",
                vec!["weighted_consensus_unresolved".to_string()],
            ));
        }

        let consensus = ConsensusGroup {
            group_id: stable_hash_ref(&json!({"topic": topic, "winner": best.candidate_id, "weight_bp": best.weight_bp}))?,
            topic: topic.clone(),
            candidates: ranked
                .iter()
                .map(|entry| ConsensusCandidate {
                    value: entry.value.clone(),
                    sources: entry.sources.iter().cloned().collect(),
                })
                .collect(),
            chosen: Some(best.value.clone()),
            agreement_score: decimal_to_string(bp_to_ratio_decimal(agreement_bp)),
            outliers,
            consensus_method: Some(ConsensusMethod::Weighted),
            minimum_threshold_met: Some(true),
            selected_result_id: Some(best.candidate_id.clone()),
            conflict_resolution_rationale: Some("weighted_consensus_threshold_met".to_string()),
        };

        let packet = ComputationPacket {
            schema_version: PH1COMP_SCHEMA_VERSION.to_string(),
            produced_by: PH1COMP_ENGINE_ID.to_string(),
            intended_consumers: vec!["PH1.LAW".to_string(), "PH1.GOV".to_string()],
            created_at_ms,
            trace_id: trace_id.into(),
            inputs: ComputationInputs {
                evidence_hash: stable_hash_ref(&json!({"kind": "consensus", "topic": topic, "signals": signals.len()}))?,
                policy_snapshot_id: policy_snapshot_id.into(),
                as_of_ms: None,
                input_count: signals.len() as u32,
                input_labels: signals.iter().map(|signal| signal.candidate_id.clone()).collect(),
                normalization_trace: vec![],
                formula_version_refs: vec![FORMULA_CONSENSUS_V1.to_string()],
            },
            aggregates: ranked
                .iter()
                .enumerate()
                .map(|(index, entry)| Aggregate {
                    metric_id: format!("consensus.{}.weight", entry.candidate_id),
                    entity: entry.candidate_id.clone(),
                    attribute: "consensus_weight".to_string(),
                    unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                    currency: None,
                    window: None,
                    method: AggregateMethod::WeightedMean,
                    value: decimal_to_numeric_value(bp_to_ratio_decimal(entry.weight_bp as u16)),
                    sample_size: entry.votes as u32,
                    source_refs: entry.sources.iter().cloned().collect(),
                    rank: Some((index + 1) as u16),
                    threshold_met: Some(index == 0),
                    priority_score: None,
                })
                .collect(),
            consensus: vec![consensus],
            confidence: vec![ConfidenceItem {
                claim_key: format!("consensus:{}", topic),
                confidence_score: decimal_to_string(bp_to_ratio_decimal(agreement_bp)),
                factors: ConfidenceFactors {
                    sample_size: signals.len() as u32,
                    trust_tier_mix: decimal_to_string(bp_to_ratio_decimal((total_weight.min(10_000)) as u16)),
                    recency: decimal_to_string(Decimal::ONE),
                    conflict: decimal_to_string(bp_to_ratio_decimal(agreement_bp)),
                    outliers: decimal_to_string(bp_to_ratio_decimal(
                        (10_000u32
                            .saturating_sub(ranked.len().saturating_sub(1) as u32 * 1_000)
                            .min(10_000)) as u16,
                    )),
                },
                bucket: Some(confidence_bucket_from_bp(agreement_bp)),
                minimum_threshold_met: Some(true),
            }],
            reason_codes: vec!["weighted_consensus_threshold_met".to_string()],
        };
        packet
            .validate()
            .map_err(|error| Ph1CompError::new(ComputationFailureClass::InvalidInputSet, format!("consensus packet invalid: {error:?}"), vec!["invalid_consensus_packet".to_string()]))?;
        Ok(packet)
    }

    pub fn normalize_inputs(
        &self,
        inputs: &[NormalizationInput],
    ) -> Result<Vec<NormalizationTraceEntry>, Ph1CompError> {
        let mut trace = Vec::new();
        for input in inputs {
            match input {
                NormalizationInput::Currency {
                    label,
                    amount,
                    from,
                    to,
                    rate,
                } => {
                    let normalized = round_decimal(*amount * *rate);
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Currency,
                        rule_id: format!("currency:{}->{}", from, to),
                        input_label: label.clone(),
                        source_value: decimal_to_string(*amount),
                        normalized_value: decimal_to_string(normalized),
                        source_unit: Some(from.clone()),
                        canonical_unit: Some(to.clone()),
                        applied: true,
                    });
                }
                NormalizationInput::Unit {
                    label,
                    value,
                    from,
                    to,
                    factor,
                } => {
                    let normalized = round_decimal(*value * *factor);
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Unit,
                        rule_id: format!("unit:{}->{}", from, to),
                        input_label: label.clone(),
                        source_value: decimal_to_string(*value),
                        normalized_value: decimal_to_string(normalized),
                        source_unit: Some(from.clone()),
                        canonical_unit: Some(to.clone()),
                        applied: true,
                    });
                }
                NormalizationInput::TimeSeconds { label, seconds } => {
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Time,
                        rule_id: "time:s_to_ms".to_string(),
                        input_label: label.clone(),
                        source_value: seconds.to_string(),
                        normalized_value: seconds.saturating_mul(1_000).to_string(),
                        source_unit: Some("s".to_string()),
                        canonical_unit: Some(CANONICAL_TIME_UNIT.to_string()),
                        applied: true,
                    });
                }
                NormalizationInput::TimeMilliseconds { label, milliseconds } => {
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Time,
                        rule_id: "time:ms_to_ms".to_string(),
                        input_label: label.clone(),
                        source_value: milliseconds.to_string(),
                        normalized_value: milliseconds.to_string(),
                        source_unit: Some(CANONICAL_TIME_UNIT.to_string()),
                        canonical_unit: Some(CANONICAL_TIME_UNIT.to_string()),
                        applied: true,
                    });
                }
                NormalizationInput::PercentageBasisPoints { label, basis_points } => {
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Percentage,
                        rule_id: "percent:bp_to_ratio".to_string(),
                        input_label: label.clone(),
                        source_value: basis_points.to_string(),
                        normalized_value: decimal_to_string(bp_to_ratio_decimal(*basis_points)),
                        source_unit: Some("bp".to_string()),
                        canonical_unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                        applied: true,
                    });
                }
                NormalizationInput::PercentageRatio { label, ratio } => {
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Percentage,
                        rule_id: "percent:ratio_to_ratio".to_string(),
                        input_label: label.clone(),
                        source_value: decimal_to_string(*ratio),
                        normalized_value: decimal_to_string(*ratio),
                        source_unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                        canonical_unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                        applied: true,
                    });
                }
                NormalizationInput::Scale {
                    label,
                    value,
                    from_scale,
                    to_scale,
                } => {
                    let normalized = scale_decimal(*value, *from_scale, *to_scale)?;
                    trace.push(NormalizationTraceEntry {
                        normalization_kind: NormalizationKind::Scale,
                        rule_id: format!("scale:{}->{}", from_scale, to_scale),
                        input_label: label.clone(),
                        source_value: decimal_to_string(*value),
                        normalized_value: decimal_to_string(normalized),
                        source_unit: Some(from_scale.to_string()),
                        canonical_unit: Some(to_scale.to_string()),
                        applied: true,
                    });
                }
            }
        }
        Ok(trace)
    }

    pub fn compute_budget_quota_packet(
        &self,
        trace_id: impl Into<String>,
        created_at_ms: i64,
        policy_snapshot_id: impl Into<String>,
        request: BudgetQuotaComputationRequest,
    ) -> Result<ComputationPacket, Ph1CompError> {
        if request.budget_limit_microunits == 0 || request.quota_limit == 0 {
            return Err(Ph1CompError::new(
                ComputationFailureClass::BudgetComputationFailure,
                "budget_limit_microunits and quota_limit must be > 0",
                vec!["invalid_budget_quota_limits".to_string()],
            ));
        }
        let projected_used = request
            .budget_used_microunits
            .checked_add(request.reserved_microunits)
            .ok_or_else(|| {
                Ph1CompError::new(
                    ComputationFailureClass::ComputationOverflow,
                    "projected budget used overflowed",
                    vec!["budget_projection_overflow".to_string()],
                )
            })?;
        let remaining = request.budget_limit_microunits.saturating_sub(projected_used);
        let budget_ratio_bp = divide_round(
            projected_used.saturating_mul(10_000),
            request.budget_limit_microunits,
        ) as u16;
        let quota_ratio_bp = divide_round(
            (request.quota_used as u64).saturating_mul(10_000),
            request.quota_limit as u64,
        ) as u16;
        let threshold_crossed = budget_ratio_bp >= request.threshold_bp || quota_ratio_bp >= request.threshold_bp;

        let packet = ComputationPacket {
            schema_version: PH1COMP_SCHEMA_VERSION.to_string(),
            produced_by: PH1COMP_ENGINE_ID.to_string(),
            intended_consumers: vec!["PH1.COST".to_string(), "PH1.QUOTA".to_string(), "PH1.LAW".to_string()],
            created_at_ms,
            trace_id: trace_id.into(),
            inputs: ComputationInputs {
                evidence_hash: stable_hash_ref(&json!({"kind": "budget_quota", "budget_limit": request.budget_limit_microunits, "quota_limit": request.quota_limit}))?,
                policy_snapshot_id: policy_snapshot_id.into(),
                as_of_ms: None,
                input_count: 4,
                input_labels: vec![
                    "budget_limit_microunits".to_string(),
                    "budget_used_microunits".to_string(),
                    "reserved_microunits".to_string(),
                    "quota_used".to_string(),
                ],
                normalization_trace: vec![],
                formula_version_refs: vec![FORMULA_BUDGET_V1.to_string()],
            },
            aggregates: vec![
                budget_aggregate("budget_remaining", remaining, None, Some(false)),
                budget_aggregate("budget_consumption_ratio", budget_ratio_bp as u64, Some(CANONICAL_PERCENT_UNIT), Some(threshold_crossed)),
                budget_aggregate("quota_consumption_ratio", quota_ratio_bp as u64, Some(CANONICAL_PERCENT_UNIT), Some(threshold_crossed)),
            ],
            consensus: vec![ConsensusGroup {
                group_id: stable_hash_ref(&json!({"kind": "budget_threshold", "crossed": threshold_crossed}))?,
                topic: "budget_quota_threshold".to_string(),
                candidates: vec![ConsensusCandidate {
                    value: NumericValue::Int { value: if threshold_crossed { 1 } else { 0 } },
                    sources: vec!["budget_ratio".to_string(), "quota_ratio".to_string()],
                }],
                chosen: Some(NumericValue::Int { value: if threshold_crossed { 1 } else { 0 } }),
                agreement_score: decimal_to_string(Decimal::ONE),
                outliers: vec![],
                consensus_method: Some(ConsensusMethod::Threshold),
                minimum_threshold_met: Some(!threshold_crossed),
                selected_result_id: Some(if threshold_crossed {
                    "threshold_crossed".to_string()
                } else {
                    "threshold_clear".to_string()
                }),
                conflict_resolution_rationale: Some("budget_quota_threshold_evaluated".to_string()),
            }],
            confidence: vec![ConfidenceItem {
                claim_key: "budget_quota:posture".to_string(),
                confidence_score: decimal_to_string(Decimal::ONE),
                factors: ConfidenceFactors {
                    sample_size: 4,
                    trust_tier_mix: decimal_to_string(Decimal::ONE),
                    recency: decimal_to_string(Decimal::ONE),
                    conflict: decimal_to_string(Decimal::ONE),
                    outliers: decimal_to_string(Decimal::ONE),
                },
                bucket: Some(ComputationConfidenceBucket::High),
                minimum_threshold_met: Some(true),
            }],
            reason_codes: if threshold_crossed {
                vec!["budget_threshold_crossed".to_string()]
            } else {
                vec!["budget_threshold_clear".to_string()]
            },
        };
        packet
            .validate()
            .map_err(|error| Ph1CompError::new(ComputationFailureClass::BudgetComputationFailure, format!("budget packet invalid: {error:?}"), vec!["invalid_budget_packet".to_string()]))?;
        Ok(packet)
    }

    pub fn computation_state_from_packet(
        &self,
        packet: &ComputationPacket,
        failure_class: Option<ComputationFailureClass>,
    ) -> Result<ComputationExecutionState, ContractViolation> {
        let packet_ref = Some(packet_ref(packet).map_err(|_| ContractViolation::InvalidValue {
            field: "computation_packet",
            reason: "canonical packet hash failed",
        })?);
        let consensus_result = packet.consensus.first().map(|group| {
            ComputationConsensusResult::v1(
                consensus_status_for_group(group),
                Some(group.agreement_score.clone()),
                group.outliers.len() as u16,
                group.conflict_resolution_rationale.clone(),
            )
        }).transpose()?;
        let selected_result = if let Some(result) = selected_result_from_packet(packet)? {
            Some(result)
        } else if let Some(aggregate) = packet.aggregates.iter().find(|aggregate| aggregate.rank == Some(1)) {
            Some(ComputationSelectedResult::v1(
                aggregate.entity.clone(),
                aggregate.value.clone(),
                aggregate.rank,
                aggregate.threshold_met.map(|met| {
                    if met {
                        "threshold_met".to_string()
                    } else {
                        "threshold_not_met".to_string()
                    }
                }),
            )?)
        } else {
            None
        };
        let confidence_posture = packet.confidence.first().map(|item| {
            ComputationConfidencePosture::v1(
                Some(item.confidence_score.clone()),
                item.bucket,
                item.minimum_threshold_met.unwrap_or(true),
            )
        }).transpose()?;
        ComputationExecutionState::v1(
            packet_ref,
            packet.inputs.normalization_trace.clone(),
            consensus_result,
            selected_result,
            confidence_posture,
            failure_class,
            packet.inputs.formula_version_refs.clone(),
            packet.reason_codes.clone(),
        )
    }

    pub fn build_missing_simulation_packet_and_state(
        &self,
        runtime_execution_envelope: &RuntimeExecutionEnvelope,
        packet: &MissingSimulationPacket,
        created_at_ms: i64,
    ) -> Result<(ComputationPacket, ComputationExecutionState), Ph1CompError> {
        let metrics = [
            ("estimated_frequency_score", packet.estimated_frequency_score_bp),
            ("estimated_value_score", packet.estimated_value_score_bp),
            ("estimated_roi_score", packet.estimated_roi_score_bp),
            ("estimated_feasibility_score", packet.estimated_feasibility_score_bp),
            ("estimated_risk_score", 10_000u16.saturating_sub(packet.estimated_risk_score_bp)),
            ("worthiness_score", packet.worthiness_score_bp),
        ];
        let normalization_trace = metrics
            .iter()
            .map(|(metric_id, value_bp)| NormalizationTraceEntry {
                normalization_kind: NormalizationKind::Percentage,
                rule_id: "percent:bp_to_ratio".to_string(),
                input_label: (*metric_id).to_string(),
                source_value: value_bp.to_string(),
                normalized_value: decimal_to_string(bp_to_ratio_decimal(*value_bp)),
                source_unit: Some("bp".to_string()),
                canonical_unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                applied: true,
            })
            .collect::<Vec<_>>();
        let packet_value = ComputationPacket {
            schema_version: PH1COMP_SCHEMA_VERSION.to_string(),
            produced_by: PH1COMP_ENGINE_ID.to_string(),
            intended_consumers: vec!["PH1.LAW".to_string(), "PH1.J".to_string(), "PH1.GOV".to_string()],
            created_at_ms,
            trace_id: runtime_execution_envelope.trace_id.clone(),
            inputs: ComputationInputs {
                evidence_hash: stable_hash_ref(&json!({
                    "dedupe_fingerprint": packet.dedupe_fingerprint,
                    "requested_capability": packet.requested_capability_name_normalized,
                    "proposed_simulation_family": packet.proposed_simulation_family,
                }))?,
                policy_snapshot_id: FORMULA_MISSING_SIM_V1.to_string(),
                as_of_ms: None,
                input_count: metrics.len() as u32,
                input_labels: metrics.iter().map(|(metric_id, _)| (*metric_id).to_string()).collect(),
                normalization_trace,
                formula_version_refs: vec![FORMULA_MISSING_SIM_V1.to_string()],
            },
            aggregates: metrics
                .iter()
                .enumerate()
                .map(|(index, (metric_id, value_bp))| Aggregate {
                    metric_id: (*metric_id).to_string(),
                    entity: packet.proposed_simulation_family.clone(),
                    attribute: (*metric_id).to_string(),
                    unit: Some(CANONICAL_PERCENT_UNIT.to_string()),
                    currency: None,
                    window: None,
                    method: AggregateMethod::WeightedMean,
                    value: decimal_to_numeric_value(bp_to_ratio_decimal(*value_bp)),
                    sample_size: 1,
                    source_refs: vec![packet.no_match_proof_ref.clone()],
                    rank: Some((index + 1) as u16),
                    threshold_met: Some(*value_bp >= 5_000),
                    priority_score: None,
                })
                .collect(),
            consensus: vec![ConsensusGroup {
                group_id: stable_hash_ref(&json!({"missing_simulation": packet.proposed_simulation_family, "worthiness": packet.worthiness_score_bp}))?,
                topic: "missing_simulation_worthiness".to_string(),
                candidates: vec![ConsensusCandidate {
                    value: decimal_to_numeric_value(bp_to_ratio_decimal(packet.worthiness_score_bp)),
                    sources: vec![packet.no_match_proof_ref.clone()],
                }],
                chosen: Some(decimal_to_numeric_value(bp_to_ratio_decimal(packet.worthiness_score_bp))),
                agreement_score: decimal_to_string(Decimal::ONE),
                outliers: vec![],
                consensus_method: Some(ConsensusMethod::Threshold),
                minimum_threshold_met: Some(packet.worthiness_score_bp >= 5_000),
                selected_result_id: Some(packet.proposed_simulation_family.clone()),
                conflict_resolution_rationale: Some("missing_simulation_worthiness_selected".to_string()),
            }],
            confidence: vec![ConfidenceItem {
                claim_key: format!("missing_simulation:{}:worthiness", packet.proposed_simulation_family),
                confidence_score: decimal_to_string(bp_to_ratio_decimal(packet.worthiness_score_bp)),
                factors: ConfidenceFactors {
                    sample_size: metrics.len() as u32,
                    trust_tier_mix: decimal_to_string(bp_to_ratio_decimal(packet.worthiness_score_bp)),
                    recency: decimal_to_string(Decimal::ONE),
                    conflict: decimal_to_string(Decimal::ONE),
                    outliers: decimal_to_string(Decimal::ONE),
                },
                bucket: Some(confidence_bucket_from_bp(packet.worthiness_score_bp)),
                minimum_threshold_met: Some(packet.worthiness_score_bp >= 5_000),
            }],
            reason_codes: vec![format!("reason_code_{}", packet.reason_code.0)],
        };
        let execution_state = self
            .computation_state_from_packet(&packet_value, None)
            .map_err(|error| {
                Ph1CompError::new(
                    ComputationFailureClass::InvalidInputSet,
                    format!("missing simulation computation state invalid: {error:?}"),
                    vec!["invalid_missing_simulation_computation_state".to_string()],
                )
            })?;
        Ok((packet_value, execution_state))
    }
}

fn selected_result_from_packet(
    packet: &ComputationPacket,
) -> Result<Option<ComputationSelectedResult>, ContractViolation> {
    for group in &packet.consensus {
        if let (Some(result_id), Some(value)) = (&group.selected_result_id, &group.chosen) {
            return ComputationSelectedResult::v1(
                result_id.clone(),
                value.clone(),
                Some(1),
                group.conflict_resolution_rationale.clone(),
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn consensus_status_for_group(group: &ConsensusGroup) -> ComputationConsensusStatus {
    match (group.consensus_method, group.chosen.is_some()) {
        (Some(ConsensusMethod::Majority), true) => ComputationConsensusStatus::MajorityReached,
        (Some(ConsensusMethod::Weighted), true) => ComputationConsensusStatus::WeightedConsensusReached,
        (Some(ConsensusMethod::Threshold), true) => {
            ComputationConsensusStatus::ThresholdConsensusReached
        }
        _ => ComputationConsensusStatus::Unresolved,
    }
}

fn packet_ref(packet: &ComputationPacket) -> Result<String, String> {
    let value = to_value(packet).map_err(|error| format!("serialize packet: {error}"))?;
    let hash = hash_canonical_json(&value)?;
    Ok(format!("ph1comp:{}", hash))
}

fn stable_hash_ref(value: &Value) -> Result<String, Ph1CompError> {
    hash_canonical_json(value).map_err(|error| {
        Ph1CompError::new(
            ComputationFailureClass::InvalidInputSet,
            format!("canonical hash failed: {}", error),
            vec!["canonical_hash_failed".to_string()],
        )
    })
}

fn bp_to_ratio_decimal(bp: u16) -> Decimal {
    round_decimal(Decimal::from(bp as i64) / Decimal::from(10_000))
}

fn confidence_bucket_from_bp(score_bp: u16) -> ComputationConfidenceBucket {
    if score_bp >= 8_000 {
        ComputationConfidenceBucket::High
    } else if score_bp >= 6_000 {
        ComputationConfidenceBucket::Medium
    } else if score_bp >= 4_000 {
        ComputationConfidenceBucket::Low
    } else {
        ComputationConfidenceBucket::Insufficient
    }
}

fn numeric_key(value: &NumericValue) -> String {
    match value {
        NumericValue::Int { value } => format!("int:{}", value),
        NumericValue::Decimal { value } => format!("decimal:{}", value),
    }
}

fn numeric_value_to_decimal(value: &NumericValue) -> Option<Decimal> {
    match value {
        NumericValue::Int { value } => Some(Decimal::from(*value)),
        NumericValue::Decimal { value } => value.parse::<Decimal>().ok(),
    }
}

fn divide_round(numerator: u64, denominator: u64) -> u32 {
    if denominator == 0 {
        return 0;
    }
    ((numerator.saturating_mul(2).saturating_add(denominator)) / (denominator.saturating_mul(2))) as u32
}

fn scale_decimal(
    value: Decimal,
    from_scale: u32,
    to_scale: u32,
) -> Result<Decimal, Ph1CompError> {
    if from_scale == to_scale {
        return Ok(round_decimal(value));
    }
    let factor = if from_scale < to_scale {
        Decimal::from(10u64.saturating_pow(to_scale - from_scale))
    } else {
        Decimal::ONE / Decimal::from(10u64.saturating_pow(from_scale - to_scale))
    };
    Ok(round_decimal(value * factor))
}

fn compare_scored_candidates(left: &ScoredCandidate, right: &ScoredCandidate) -> Ordering {
    right
        .final_score_bp
        .cmp(&left.final_score_bp)
        .then(right.priority_score_bp.cmp(&left.priority_score_bp))
        .then(right.confidence_bp.cmp(&left.confidence_bp))
        .then(left.tie_break_key.cmp(&right.tie_break_key))
        .then(left.candidate_id.cmp(&right.candidate_id))
}

fn build_ranking_consensus_group(scored: &[ScoredCandidate]) -> ConsensusGroup {
    let chosen = scored.first();
    ConsensusGroup {
        group_id: format!("ranking:{}", scored.len()),
        topic: "candidate_ranking".to_string(),
        candidates: scored
            .iter()
            .map(|candidate| ConsensusCandidate {
                value: decimal_to_numeric_value(bp_to_ratio_decimal(candidate.final_score_bp)),
                sources: vec![candidate.candidate_id.clone()],
            })
            .collect(),
        chosen: chosen.map(|candidate| decimal_to_numeric_value(bp_to_ratio_decimal(candidate.final_score_bp))),
        agreement_score: chosen
            .map(|candidate| decimal_to_string(bp_to_ratio_decimal(candidate.final_score_bp)))
            .unwrap_or_else(|| "0".to_string()),
        outliers: vec![],
        consensus_method: Some(ConsensusMethod::Threshold),
        minimum_threshold_met: chosen.map(|candidate| candidate.threshold_met),
        selected_result_id: chosen.map(|candidate| candidate.candidate_id.clone()),
        conflict_resolution_rationale: Some("deterministic_ranking_selected_top_candidate".to_string()),
    }
}

fn budget_aggregate(
    metric_id: &str,
    value: u64,
    unit: Option<&str>,
    threshold_met: Option<bool>,
) -> Aggregate {
    let numeric_value = if unit == Some(CANONICAL_PERCENT_UNIT) {
        decimal_to_numeric_value(bp_to_ratio_decimal(value as u16))
    } else {
        NumericValue::Int { value: value as i64 }
    };
    Aggregate {
        metric_id: metric_id.to_string(),
        entity: "budget_quota".to_string(),
        attribute: metric_id.to_string(),
        unit: unit.map(|value| value.to_string()).or_else(|| Some(CANONICAL_BUDGET_UNIT.to_string())),
        currency: None,
        window: None,
        method: AggregateMethod::WeightedMean,
        value: numeric_value,
        sample_size: 1,
        source_refs: vec![metric_id.to_string()],
        rank: Some(1),
        threshold_met,
        priority_score: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConsensusAccumulator {
    candidate_id: String,
    value: NumericValue,
    weight_bp: u32,
    votes: u32,
    sources: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScoredCandidate {
    candidate_id: String,
    tie_break_key: String,
    priority_score_bp: u16,
    confidence_bp: u16,
    final_score_bp: u16,
    threshold_met: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{DeviceId, TurnId};
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::runtime_execution::{
        AdmissionState, DeviceClass, DeviceTrustClass, NetworkProfile, PlatformRuntimeContext,
        PlatformTriggerPolicy, RuntimeEntryTrigger, RuntimeExecutionEnvelope,
    };

    fn sample_envelope() -> RuntimeExecutionEnvelope {
        RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state(
            "req_comp_1".to_string(),
            "trace_comp_1".to_string(),
            "idem_comp_1".to_string(),
            UserId::new("tenant_1:comp_user").unwrap(),
            DeviceId::new("comp_device_1").unwrap(),
            AppPlatform::Desktop,
            PlatformRuntimeContext::v1(
                AppPlatform::Desktop,
                "1.0.0".to_string(),
                DeviceClass::Desktop,
                "desktop-client-1".to_string(),
                "desktop-default".to_string(),
                NetworkProfile::Standard,
                vec![],
                vec![],
                DeviceTrustClass::StandardDevice,
                selene_kernel_contracts::runtime_execution::ClientIntegrityStatus::IntegrityUnavailable,
                selene_kernel_contracts::runtime_execution::ClientCompatibilityStatus::Compatible,
                None,
                None,
                RuntimeEntryTrigger::Explicit,
                PlatformTriggerPolicy::WakeOrExplicit,
                true,
            )
            .unwrap(),
            Some(SessionId(7001)),
            TurnId(7002),
            Some(1),
            AdmissionState::ExecutionAdmitted,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_comp_01_identical_inputs_produce_identical_packets() {
        let runtime = Ph1CompRuntime;
        let packet_1 = runtime
            .rank_candidates(
                "trace_a",
                1,
                "policy_v1",
                vec![RankedCandidateInput {
                    candidate_id: "alpha".to_string(),
                    tie_break_key: "a".to_string(),
                    priority_score_bp: 7_000,
                    confidence_bp: 8_000,
                    threshold_bp: Some(5_000),
                    components: vec![
                        WeightedScoreComponentInput {
                            component_id: "rel".to_string(),
                            normalized_score_bp: 8_000,
                            weight_bp: 6_000,
                        },
                        WeightedScoreComponentInput {
                            component_id: "trust".to_string(),
                            normalized_score_bp: 7_000,
                            weight_bp: 4_000,
                        },
                    ],
                }],
            )
            .unwrap();
        let packet_2 = runtime
            .rank_candidates(
                "trace_a",
                1,
                "policy_v1",
                vec![RankedCandidateInput {
                    candidate_id: "alpha".to_string(),
                    tie_break_key: "a".to_string(),
                    priority_score_bp: 7_000,
                    confidence_bp: 8_000,
                    threshold_bp: Some(5_000),
                    components: vec![
                        WeightedScoreComponentInput {
                            component_id: "rel".to_string(),
                            normalized_score_bp: 8_000,
                            weight_bp: 6_000,
                        },
                        WeightedScoreComponentInput {
                            component_id: "trust".to_string(),
                            normalized_score_bp: 7_000,
                            weight_bp: 4_000,
                        },
                    ],
                }],
            )
            .unwrap();
        assert_eq!(packet_1, packet_2);
    }

    #[test]
    fn at_comp_02_deterministic_tie_breaking_works() {
        let runtime = Ph1CompRuntime;
        let packet = runtime
            .rank_candidates(
                "trace_b",
                2,
                "policy_v1",
                vec![
                    RankedCandidateInput {
                        candidate_id: "beta".to_string(),
                        tie_break_key: "b".to_string(),
                        priority_score_bp: 6_000,
                        confidence_bp: 8_000,
                        threshold_bp: Some(5_000),
                        components: vec![WeightedScoreComponentInput {
                            component_id: "score".to_string(),
                            normalized_score_bp: 8_000,
                            weight_bp: 10_000,
                        }],
                    },
                    RankedCandidateInput {
                        candidate_id: "alpha".to_string(),
                        tie_break_key: "a".to_string(),
                        priority_score_bp: 6_000,
                        confidence_bp: 8_000,
                        threshold_bp: Some(5_000),
                        components: vec![WeightedScoreComponentInput {
                            component_id: "score".to_string(),
                            normalized_score_bp: 8_000,
                            weight_bp: 10_000,
                        }],
                    },
                ],
            )
            .unwrap();
        let top = packet
            .consensus
            .first()
            .and_then(|group| group.selected_result_id.as_ref())
            .cloned()
            .unwrap();
        assert_eq!(top, "alpha");
    }

    #[test]
    fn at_comp_03_weighted_consensus_works() {
        let runtime = Ph1CompRuntime;
        let packet = runtime
            .evaluate_consensus(
                "trace_c",
                3,
                "policy_v1",
                "pricing",
                vec![
                    ConsensusSignalInput {
                        candidate_id: "usd_100".to_string(),
                        value: NumericValue::Int { value: 100 },
                        weight_bp: 6_000,
                        source_ref: "source_1".to_string(),
                    },
                    ConsensusSignalInput {
                        candidate_id: "usd_100".to_string(),
                        value: NumericValue::Int { value: 100 },
                        weight_bp: 2_000,
                        source_ref: "source_2".to_string(),
                    },
                    ConsensusSignalInput {
                        candidate_id: "usd_120".to_string(),
                        value: NumericValue::Int { value: 120 },
                        weight_bp: 2_000,
                        source_ref: "source_3".to_string(),
                    },
                ],
                6_000,
            )
            .unwrap();
        assert_eq!(
            packet.consensus[0].selected_result_id.as_deref(),
            Some("usd_100")
        );
    }

    #[test]
    fn at_comp_04_outlier_handling_is_deterministic() {
        let runtime = Ph1CompRuntime;
        let packet = runtime
            .evaluate_consensus(
                "trace_d",
                4,
                "policy_v1",
                "metric",
                vec![
                    ConsensusSignalInput {
                        candidate_id: "m1".to_string(),
                        value: NumericValue::Int { value: 100 },
                        weight_bp: 4_000,
                        source_ref: "source_1".to_string(),
                    },
                    ConsensusSignalInput {
                        candidate_id: "m2".to_string(),
                        value: NumericValue::Int { value: 101 },
                        weight_bp: 4_000,
                        source_ref: "source_2".to_string(),
                    },
                    ConsensusSignalInput {
                        candidate_id: "m3".to_string(),
                        value: NumericValue::Int { value: 350 },
                        weight_bp: 2_000,
                        source_ref: "source_3".to_string(),
                    },
                ],
                5_000,
            )
            .unwrap();
        assert!(packet.consensus[0]
            .outliers
            .iter()
            .any(|outlier| matches!(outlier.value, NumericValue::Int { value: 350 })));
    }

    #[test]
    fn at_comp_05_heterogeneous_normalization_produces_canonical_values() {
        let runtime = Ph1CompRuntime;
        let trace = runtime
            .normalize_inputs(&[
                NormalizationInput::Currency {
                    label: "price".to_string(),
                    amount: Decimal::new(100, 0),
                    from: "EUR".to_string(),
                    to: "USD".to_string(),
                    rate: Decimal::new(11, 1),
                },
                NormalizationInput::Unit {
                    label: "distance".to_string(),
                    value: Decimal::new(5, 0),
                    from: "km".to_string(),
                    to: "m".to_string(),
                    factor: Decimal::new(1000, 0),
                },
                NormalizationInput::TimeSeconds {
                    label: "latency".to_string(),
                    seconds: 2,
                },
                NormalizationInput::PercentageBasisPoints {
                    label: "confidence".to_string(),
                    basis_points: 7_500,
                },
            ])
            .unwrap();
        assert_eq!(trace.len(), 4);
        assert_eq!(trace[0].normalized_value, "110");
        assert_eq!(trace[1].normalized_value, "5000");
        assert_eq!(trace[2].normalized_value, "2000");
        assert_eq!(trace[3].normalized_value, "0.75");
    }

    #[test]
    fn at_comp_06_budget_quota_calculation_is_deterministic() {
        let runtime = Ph1CompRuntime;
        let packet_1 = runtime
            .compute_budget_quota_packet(
                "trace_e",
                5,
                "policy_v1",
                BudgetQuotaComputationRequest {
                    budget_limit_microunits: 1_000,
                    budget_used_microunits: 400,
                    reserved_microunits: 100,
                    quota_limit: 10,
                    quota_used: 4,
                    threshold_bp: 7_000,
                },
            )
            .unwrap();
        let packet_2 = runtime
            .compute_budget_quota_packet(
                "trace_e",
                5,
                "policy_v1",
                BudgetQuotaComputationRequest {
                    budget_limit_microunits: 1_000,
                    budget_used_microunits: 400,
                    reserved_microunits: 100,
                    quota_limit: 10,
                    quota_used: 4,
                    threshold_bp: 7_000,
                },
            )
            .unwrap();
        assert_eq!(packet_1, packet_2);
    }

    #[test]
    fn at_comp_07_failure_classes_surface_correctly() {
        let runtime = Ph1CompRuntime;
        let err = runtime
            .evaluate_consensus(
                "trace_f",
                6,
                "policy_v1",
                "unresolved",
                vec![
                    ConsensusSignalInput {
                        candidate_id: "a".to_string(),
                        value: NumericValue::Int { value: 10 },
                        weight_bp: 5_000,
                        source_ref: "s1".to_string(),
                    },
                    ConsensusSignalInput {
                        candidate_id: "b".to_string(),
                        value: NumericValue::Int { value: 20 },
                        weight_bp: 5_000,
                        source_ref: "s2".to_string(),
                    },
                ],
                7_000,
            )
            .expect_err("unresolved consensus must fail deterministically");
        assert_eq!(err.class, ComputationFailureClass::ConsensusUnresolved);
    }

    #[test]
    fn at_comp_08_computation_state_attaches_to_runtime_envelope() {
        let runtime = Ph1CompRuntime;
        let packet = runtime
            .compute_budget_quota_packet(
                "trace_g",
                7,
                "policy_v1",
                BudgetQuotaComputationRequest {
                    budget_limit_microunits: 1_000,
                    budget_used_microunits: 250,
                    reserved_microunits: 150,
                    quota_limit: 10,
                    quota_used: 3,
                    threshold_bp: 7_500,
                },
            )
            .unwrap();
        let state = runtime.computation_state_from_packet(&packet, None).unwrap();
        let envelope = sample_envelope().with_computation_state(Some(state)).unwrap();
        assert!(envelope.computation_state.is_some());
        assert_eq!(
            envelope
                .computation_state
                .as_ref()
                .unwrap()
                .formula_version_refs,
            vec![FORMULA_BUDGET_V1.to_string()]
        );
    }
}
