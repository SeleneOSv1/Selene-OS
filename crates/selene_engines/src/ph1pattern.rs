#![forbid(unsafe_code)]

use std::cmp::{max, min};
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1pattern::{
    PatternCapabilityId, PatternMineOfflineOk, PatternMineOfflineRequest, PatternProposalEmitOk,
    PatternProposalEmitRequest, PatternProposalItem, PatternProposalTarget, PatternRefuse,
    PatternSignal, PatternValidationStatus, Ph1PatternRequest, Ph1PatternResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PATTERN reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PATTERN_OK_MINE_OFFLINE: ReasonCodeId = ReasonCodeId(0x5041_0001);
    pub const PH1_PATTERN_OK_PROPOSAL_EMIT: ReasonCodeId = ReasonCodeId(0x5041_0002);

    pub const PH1_PATTERN_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5041_00F1);
    pub const PH1_PATTERN_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5041_00F2);
    pub const PH1_PATTERN_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5041_00F3);
    pub const PH1_PATTERN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5041_00F4);
    pub const PH1_PATTERN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5041_00F5);
    pub const PH1_PATTERN_OFFLINE_ONLY_REQUIRED: ReasonCodeId = ReasonCodeId(0x5041_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PatternConfig {
    pub max_signals: u8,
    pub max_proposals: u8,
    pub max_diagnostics: u8,
}

impl Ph1PatternConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_signals: 32,
            max_proposals: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PatternRuntime {
    config: Ph1PatternConfig,
}

impl Ph1PatternRuntime {
    pub fn new(config: Ph1PatternConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PATTERN_INPUT_SCHEMA_INVALID,
                "pattern request failed contract validation",
            );
        }

        match req {
            Ph1PatternRequest::PatternMineOffline(r) => self.run_mine_offline(r),
            Ph1PatternRequest::PatternProposalEmit(r) => self.run_proposal_emit(r),
        }
    }

    fn run_mine_offline(&self, req: &PatternMineOfflineRequest) -> Ph1PatternResponse {
        if !req.envelope.offline_pipeline_only {
            return self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_OFFLINE_ONLY_REQUIRED,
                "pattern capability is offline-only",
            );
        }

        if req.signals.is_empty() {
            return self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_UPSTREAM_INPUT_MISSING,
                "signals is empty",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if req.signals.len() > signal_budget {
            return self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_BUDGET_EXCEEDED,
                "signals exceeds configured budget",
            );
        }

        let proposal_budget = min(req.envelope.max_proposals, self.config.max_proposals) as usize;
        if proposal_budget == 0 {
            return self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_BUDGET_EXCEEDED,
                "proposal budget exceeded",
            );
        }

        let mut scored = req
            .signals
            .iter()
            .map(|signal| {
                (
                    format!("proposal_{}", signal.signal_id),
                    target_for_signal(signal),
                    proposal_confidence(signal),
                    signal.evidence_ref.clone(),
                    score(signal, req.analysis_window_days),
                )
            })
            .collect::<Vec<_>>();

        scored.sort_by(|a, b| b.4.cmp(&a.4).then(a.0.cmp(&b.0)));

        let mut seen = BTreeSet::new();
        let mut ordered_proposals = Vec::new();
        for (proposal_id, target, confidence_pct, evidence_ref, _) in scored {
            if !seen.insert(proposal_id.clone()) {
                continue;
            }
            if ordered_proposals.len() >= proposal_budget {
                break;
            }

            let item = match PatternProposalItem::v1(
                proposal_id,
                target,
                (ordered_proposals.len() + 1) as u8,
                confidence_pct,
                3,
                evidence_ref,
            ) {
                Ok(item) => item,
                Err(_) => {
                    return self.refuse(
                        PatternCapabilityId::PatternMineOffline,
                        reason_codes::PH1_PATTERN_INTERNAL_PIPELINE_ERROR,
                        "failed to construct proposal item",
                    )
                }
            };
            ordered_proposals.push(item);
        }

        if ordered_proposals.is_empty() {
            return self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_UPSTREAM_INPUT_MISSING,
                "no proposals could be mined",
            );
        }

        let selected_proposal_id = ordered_proposals[0].proposal_id.clone();
        match PatternMineOfflineOk::v1(
            reason_codes::PH1_PATTERN_OK_MINE_OFFLINE,
            selected_proposal_id,
            ordered_proposals,
            true,
            true,
        ) {
            Ok(ok) => Ph1PatternResponse::PatternMineOfflineOk(ok),
            Err(_) => self.refuse(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_INTERNAL_PIPELINE_ERROR,
                "failed to construct mine output",
            ),
        }
    }

    fn run_proposal_emit(&self, req: &PatternProposalEmitRequest) -> Ph1PatternResponse {
        if !req.envelope.offline_pipeline_only {
            return self.refuse(
                PatternCapabilityId::PatternProposalEmit,
                reason_codes::PH1_PATTERN_OFFLINE_ONLY_REQUIRED,
                "pattern capability is offline-only",
            );
        }

        if req.ordered_proposals.is_empty() {
            return self.refuse(
                PatternCapabilityId::PatternProposalEmit,
                reason_codes::PH1_PATTERN_UPSTREAM_INPUT_MISSING,
                "ordered_proposals is empty",
            );
        }

        let proposal_budget = min(req.envelope.max_proposals, self.config.max_proposals) as usize;
        if req.ordered_proposals.len() > proposal_budget {
            return self.refuse(
                PatternCapabilityId::PatternProposalEmit,
                reason_codes::PH1_PATTERN_BUDGET_EXCEEDED,
                "ordered_proposals exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();
        if req.ordered_proposals[0].proposal_id != req.selected_proposal_id {
            diagnostics.push("selected_not_first_in_ordered_proposals".to_string());
        }
        if !req
            .ordered_proposals
            .iter()
            .any(|item| item.proposal_id == req.selected_proposal_id)
        {
            diagnostics.push("selected_proposal_not_present_in_ordered_proposals".to_string());
        }
        if req
            .ordered_proposals
            .windows(2)
            .any(|pair| pair[0].confidence_pct < pair[1].confidence_pct)
        {
            diagnostics.push("confidence_not_sorted_desc".to_string());
        }

        let mut expected_rank = 1u8;
        for item in &req.ordered_proposals {
            if item.rank != expected_rank {
                diagnostics.push("rank_sequence_gap_detected".to_string());
                break;
            }
            expected_rank = expected_rank.saturating_add(1);
        }

        let mut proposal_ids = BTreeSet::new();
        if req
            .ordered_proposals
            .iter()
            .any(|item| !proposal_ids.insert(item.proposal_id.as_str()))
        {
            diagnostics.push("duplicate_proposal_id".to_string());
        }

        if req
            .ordered_proposals
            .iter()
            .any(|item| item.approval_tier != 3)
        {
            diagnostics.push("approval_tier_not_strict_3".to_string());
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                PatternValidationStatus::Ok,
                reason_codes::PH1_PATTERN_OK_PROPOSAL_EMIT,
            )
        } else {
            (
                PatternValidationStatus::Fail,
                reason_codes::PH1_PATTERN_VALIDATION_FAILED,
            )
        };

        match PatternProposalEmitOk::v1(reason_code, validation_status, diagnostics, true, true) {
            Ok(ok) => Ph1PatternResponse::PatternProposalEmitOk(ok),
            Err(_) => self.refuse(
                PatternCapabilityId::PatternProposalEmit,
                reason_codes::PH1_PATTERN_INTERNAL_PIPELINE_ERROR,
                "failed to construct proposal-emit output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PatternCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PatternResponse {
        let out = PatternRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PatternRefuse::v1 must construct for static messages");
        Ph1PatternResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1PatternRequest) -> PatternCapabilityId {
    match req {
        Ph1PatternRequest::PatternMineOffline(_) => PatternCapabilityId::PatternMineOffline,
        Ph1PatternRequest::PatternProposalEmit(_) => PatternCapabilityId::PatternProposalEmit,
    }
}

fn target_for_signal(signal: &PatternSignal) -> PatternProposalTarget {
    match signal.metric_key.as_str() {
        "provider_fallback_rate" | "provider_timeout_rate" => {
            PatternProposalTarget::PaeProviderRoutingWeights
        }
        "clarify_loop_rate" | "missing_field_repeat_rate" => {
            PatternProposalTarget::PruneClarificationOrdering
        }
        "prefetch_miss_rate" | "cache_stale_rate" => PatternProposalTarget::CachePrefetchHeuristics,
        _ => PatternProposalTarget::ContextRetrievalScoring,
    }
}

fn proposal_confidence(signal: &PatternSignal) -> u8 {
    let confidence =
        45 + (signal.occurrence_count.min(300) / 6) as i16 + signal.metric_value_bp / 120;
    confidence.clamp(0, 100) as u8
}

fn score(signal: &PatternSignal, analysis_window_days: u16) -> u8 {
    let mut score = signal.occurrence_count.min(1000) as i16 / 8;
    score += signal.metric_value_bp / 60;
    score += target_bonus(target_for_signal(signal));
    score += window_bonus(analysis_window_days);
    max(0, min(score, 100)) as u8
}

fn target_bonus(target: PatternProposalTarget) -> i16 {
    match target {
        PatternProposalTarget::PaeProviderRoutingWeights => 10,
        PatternProposalTarget::PruneClarificationOrdering => 9,
        PatternProposalTarget::CachePrefetchHeuristics => 8,
        PatternProposalTarget::ContextRetrievalScoring => 7,
    }
}

fn window_bonus(analysis_window_days: u16) -> i16 {
    if analysis_window_days >= 28 {
        8
    } else if analysis_window_days >= 14 {
        5
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1pattern::{PatternRequestEnvelope, PatternValidationStatus};

    fn runtime() -> Ph1PatternRuntime {
        Ph1PatternRuntime::new(Ph1PatternConfig::mvp_v1())
    }

    fn envelope(max_signals: u8, max_proposals: u8) -> PatternRequestEnvelope {
        PatternRequestEnvelope::v1(
            CorrelationId(2801),
            TurnId(251),
            max_signals,
            max_proposals,
            true,
        )
        .unwrap()
    }

    fn signal(
        signal_id: &str,
        metric_key: &str,
        metric_value_bp: i16,
        occurrence_count: u32,
    ) -> PatternSignal {
        PatternSignal::v1(
            signal_id.to_string(),
            "PH1.J".to_string(),
            metric_key.to_string(),
            metric_value_bp,
            occurrence_count,
            "evidence:pattern:2".to_string(),
        )
        .unwrap()
    }

    fn signals() -> Vec<PatternSignal> {
        vec![
            signal("sig_provider", "provider_fallback_rate", 240, 80),
            signal("sig_clarify", "clarify_loop_rate", 210, 75),
            signal("sig_context", "context_miss_rate", 180, 60),
        ]
    }

    #[test]
    fn at_pattern_01_mine_output_is_schema_valid() {
        let req = Ph1PatternRequest::PatternMineOffline(
            PatternMineOfflineRequest::v1(envelope(8, 4), signals(), 30).unwrap(),
        );
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1PatternResponse::PatternMineOfflineOk(ok) => {
                assert_eq!(ok.selected_proposal_id, ok.ordered_proposals[0].proposal_id);
            }
            _ => panic!("expected PatternMineOfflineOk"),
        }
    }

    #[test]
    fn at_pattern_02_output_order_is_deterministic() {
        let req = Ph1PatternRequest::PatternMineOffline(
            PatternMineOfflineRequest::v1(envelope(8, 4), signals(), 30).unwrap(),
        );

        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);

        let ordered1 = match out1 {
            Ph1PatternResponse::PatternMineOfflineOk(ok) => ok.ordered_proposals,
            _ => panic!("expected PatternMineOfflineOk"),
        };
        let ordered2 = match out2 {
            Ph1PatternResponse::PatternMineOfflineOk(ok) => ok.ordered_proposals,
            _ => panic!("expected PatternMineOfflineOk"),
        };

        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_pattern_03_budget_bound_is_enforced() {
        let req = Ph1PatternRequest::PatternMineOffline(
            PatternMineOfflineRequest::v1(envelope(1, 4), signals(), 30).unwrap(),
        );
        let out = runtime().run(&req);
        match out {
            Ph1PatternResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_PATTERN_BUDGET_EXCEEDED
                );
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_pattern_04_proposal_emit_fails_on_selection_drift() {
        let mined = runtime().run(&Ph1PatternRequest::PatternMineOffline(
            PatternMineOfflineRequest::v1(envelope(8, 4), signals(), 30).unwrap(),
        ));
        let mined_ok = match mined {
            Ph1PatternResponse::PatternMineOfflineOk(ok) => ok,
            _ => panic!("expected PatternMineOfflineOk"),
        };

        let req = Ph1PatternRequest::PatternProposalEmit(
            PatternProposalEmitRequest::v1(
                envelope(8, 4),
                mined_ok.ordered_proposals[1].proposal_id.clone(),
                mined_ok.ordered_proposals,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PatternResponse::PatternProposalEmitOk(ok) => {
                assert_eq!(ok.validation_status, PatternValidationStatus::Fail);
            }
            _ => panic!("expected PatternProposalEmitOk"),
        }
    }
}
