#![forbid(unsafe_code)]

use std::cmp::{max, min};
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1feedback::{
    FeedbackCapabilityId, FeedbackEventCollectOk, FeedbackEventCollectRequest, FeedbackEventRecord,
    FeedbackEventType, FeedbackGoldStatus, FeedbackPathType, FeedbackRefuse,
    FeedbackSignalCandidate, FeedbackSignalEmitOk, FeedbackSignalEmitRequest, FeedbackSignalTarget,
    FeedbackValidationStatus, Ph1FeedbackRequest, Ph1FeedbackResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.FEEDBACK reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_FEEDBACK_OK_EVENT_COLLECT: ReasonCodeId = ReasonCodeId(0x4642_0001);
    pub const PH1_FEEDBACK_OK_SIGNAL_EMIT: ReasonCodeId = ReasonCodeId(0x4642_0002);

    pub const PH1_FEEDBACK_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4642_00F1);
    pub const PH1_FEEDBACK_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4642_00F2);
    pub const PH1_FEEDBACK_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4642_00F3);
    pub const PH1_FEEDBACK_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4642_00F4);
    pub const PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4642_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1FeedbackConfig {
    pub max_events: u8,
    pub max_signals: u8,
    pub max_diagnostics: u8,
}

impl Ph1FeedbackConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_events: 24,
            max_signals: 12,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1FeedbackRuntime {
    config: Ph1FeedbackConfig,
}

impl Ph1FeedbackRuntime {
    pub fn new(config: Ph1FeedbackConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_FEEDBACK_INPUT_SCHEMA_INVALID,
                "feedback request failed contract validation",
            );
        }

        match req {
            Ph1FeedbackRequest::FeedbackEventCollect(r) => self.run_event_collect(r),
            Ph1FeedbackRequest::FeedbackSignalEmit(r) => self.run_signal_emit(r),
        }
    }

    fn run_event_collect(&self, req: &FeedbackEventCollectRequest) -> Ph1FeedbackResponse {
        if req.events.is_empty() {
            return self.refuse(
                FeedbackCapabilityId::FeedbackEventCollect,
                reason_codes::PH1_FEEDBACK_UPSTREAM_INPUT_MISSING,
                "events is empty",
            );
        }

        let event_budget = min(req.envelope.max_events, self.config.max_events) as usize;
        if req.events.len() > event_budget {
            return self.refuse(
                FeedbackCapabilityId::FeedbackEventCollect,
                reason_codes::PH1_FEEDBACK_BUDGET_EXCEEDED,
                "events exceeds configured budget",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if signal_budget == 0 {
            return self.refuse(
                FeedbackCapabilityId::FeedbackEventCollect,
                reason_codes::PH1_FEEDBACK_BUDGET_EXCEEDED,
                "signal budget exceeded",
            );
        }

        let mut candidate_build = req
            .events
            .iter()
            .map(build_candidate_from_event)
            .collect::<Result<Vec<_>, _>>();

        let candidates = match candidate_build.as_mut() {
            Ok(candidates) => {
                candidates.sort_by(|a, b| {
                    b.signal_value_bp
                        .cmp(&a.signal_value_bp)
                        .then(a.candidate_id.cmp(&b.candidate_id))
                });
                candidates.truncate(signal_budget);
                candidates.clone()
            }
            Err(_) => {
                return self.refuse(
                    FeedbackCapabilityId::FeedbackEventCollect,
                    reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                    "failed to build signal candidates",
                );
            }
        };

        if candidates.is_empty() {
            return self.refuse(
                FeedbackCapabilityId::FeedbackEventCollect,
                reason_codes::PH1_FEEDBACK_UPSTREAM_INPUT_MISSING,
                "no candidates could be derived",
            );
        }

        let selected_candidate_id = candidates[0].candidate_id.clone();
        match FeedbackEventCollectOk::v1(
            reason_codes::PH1_FEEDBACK_OK_EVENT_COLLECT,
            selected_candidate_id,
            candidates,
            true,
            true,
        ) {
            Ok(ok) => Ph1FeedbackResponse::FeedbackEventCollectOk(ok),
            Err(_) => self.refuse(
                FeedbackCapabilityId::FeedbackEventCollect,
                reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                "failed to construct feedback collect output",
            ),
        }
    }

    fn run_signal_emit(&self, req: &FeedbackSignalEmitRequest) -> Ph1FeedbackResponse {
        if req.ordered_signal_candidates.is_empty() {
            return self.refuse(
                FeedbackCapabilityId::FeedbackSignalEmit,
                reason_codes::PH1_FEEDBACK_UPSTREAM_INPUT_MISSING,
                "ordered_signal_candidates is empty",
            );
        }

        let signal_budget = min(req.envelope.max_signals, self.config.max_signals) as usize;
        if req.ordered_signal_candidates.len() > signal_budget {
            return self.refuse(
                FeedbackCapabilityId::FeedbackSignalEmit,
                reason_codes::PH1_FEEDBACK_BUDGET_EXCEEDED,
                "ordered_signal_candidates exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();
        if req.ordered_signal_candidates[0].candidate_id != req.selected_candidate_id {
            diagnostics.push("selected_not_first_in_ordered_candidates".to_string());
        }
        if !req
            .ordered_signal_candidates
            .iter()
            .any(|candidate| candidate.candidate_id == req.selected_candidate_id)
        {
            diagnostics.push("selected_candidate_not_present_in_ordered_candidates".to_string());
        }
        if req
            .ordered_signal_candidates
            .windows(2)
            .any(|pair| pair[0].signal_value_bp < pair[1].signal_value_bp)
        {
            diagnostics.push("signal_value_not_sorted_desc".to_string());
        }

        let mut candidate_ids = BTreeSet::new();
        if req
            .ordered_signal_candidates
            .iter()
            .any(|candidate| !candidate_ids.insert(candidate.candidate_id.as_str()))
        {
            diagnostics.push("duplicate_candidate_id".to_string());
        }

        let emits_learn = req.ordered_signal_candidates.iter().any(|candidate| {
            matches!(
                candidate.target,
                FeedbackSignalTarget::LearnPackage | FeedbackSignalTarget::PaeScorecard
            )
        });
        let emits_pae = req
            .ordered_signal_candidates
            .iter()
            .any(|candidate| matches!(candidate.target, FeedbackSignalTarget::PaeScorecard));

        if !emits_learn && !emits_pae {
            diagnostics.push("no_emit_target_present".to_string());
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                FeedbackValidationStatus::Ok,
                reason_codes::PH1_FEEDBACK_OK_SIGNAL_EMIT,
            )
        } else {
            (
                FeedbackValidationStatus::Fail,
                reason_codes::PH1_FEEDBACK_VALIDATION_FAILED,
            )
        };

        match FeedbackSignalEmitOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            emits_learn,
            emits_pae,
            true,
            true,
        ) {
            Ok(ok) => Ph1FeedbackResponse::FeedbackSignalEmitOk(ok),
            Err(_) => self.refuse(
                FeedbackCapabilityId::FeedbackSignalEmit,
                reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                "failed to construct feedback signal-emit output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: FeedbackCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1FeedbackResponse {
        let out = FeedbackRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("FeedbackRefuse::v1 must construct for static messages");
        Ph1FeedbackResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1FeedbackRequest) -> FeedbackCapabilityId {
    match req {
        Ph1FeedbackRequest::FeedbackEventCollect(_) => FeedbackCapabilityId::FeedbackEventCollect,
        Ph1FeedbackRequest::FeedbackSignalEmit(_) => FeedbackCapabilityId::FeedbackSignalEmit,
    }
}

fn build_candidate_from_event(
    event: &FeedbackEventRecord,
) -> Result<FeedbackSignalCandidate, selene_kernel_contracts::ContractViolation> {
    let signal_key = match event.event_type {
        FeedbackEventType::SttReject => "stt_reject_rate_by_env",
        FeedbackEventType::SttRetry => "stt_retry_rate",
        FeedbackEventType::LanguageMismatch => "language_mismatch_rate",
        FeedbackEventType::UserCorrection => "correction_rate_by_intent",
        FeedbackEventType::ClarifyLoop => "clarify_turns_per_intent",
        FeedbackEventType::ConfirmAbort => "confirm_abort_rate",
        FeedbackEventType::ToolFail => "tool_timeout_rate",
        FeedbackEventType::MemoryOverride => "memory_override_rate",
        FeedbackEventType::DeliverySwitch => "text_only_rate_in_meetings",
        FeedbackEventType::BargeIn => "barge_in_rate",
        FeedbackEventType::VoiceIdFalseReject => "voice_id_false_reject_rate",
        FeedbackEventType::VoiceIdFalseAccept => "voice_id_false_accept_rate",
        FeedbackEventType::VoiceIdMultiSpeaker => "voice_id_multi_speaker_rate",
        FeedbackEventType::VoiceIdDriftAlert => "voice_id_drift_alert_rate",
        FeedbackEventType::VoiceIdReauthFriction => "voice_id_reauth_friction_rate",
        FeedbackEventType::VoiceIdSpoofRisk => "voice_id_spoof_risk_rate",
        FeedbackEventType::VoiceIdConfusionPair => "voice_id_confusion_pair_rate",
        FeedbackEventType::VoiceIdDrift => "voice_id_drift_rate",
        FeedbackEventType::VoiceIdLowQuality => "voice_id_low_quality_rate",
    };
    let target = match event.event_type {
        FeedbackEventType::SttReject
        | FeedbackEventType::SttRetry
        | FeedbackEventType::LanguageMismatch
        | FeedbackEventType::ToolFail => FeedbackSignalTarget::PaeScorecard,
        _ => FeedbackSignalTarget::LearnPackage,
    };
    let signal_value_bp = score_event(event);
    let sample_count = max(event.metrics.retries as u32, 1);

    FeedbackSignalCandidate::v2(
        format!("signal_{}", event.event_id),
        event.event_type,
        signal_key.to_string(),
        target,
        event.path_type,
        event.gold_case_id.clone(),
        event.gold_status,
        signal_value_bp,
        sample_count,
        event.evidence_ref.clone(),
    )
}

fn score_event(event: &FeedbackEventRecord) -> i16 {
    let event_weight = match event.event_type {
        FeedbackEventType::SttReject => 950,
        FeedbackEventType::SttRetry => 840,
        FeedbackEventType::LanguageMismatch => 820,
        FeedbackEventType::UserCorrection => 780,
        FeedbackEventType::ClarifyLoop => 760,
        FeedbackEventType::ConfirmAbort => 620,
        FeedbackEventType::ToolFail => 900,
        FeedbackEventType::MemoryOverride => 700,
        FeedbackEventType::DeliverySwitch => 580,
        FeedbackEventType::BargeIn => 640,
        FeedbackEventType::VoiceIdFalseReject => 930,
        FeedbackEventType::VoiceIdFalseAccept => 980,
        FeedbackEventType::VoiceIdMultiSpeaker => 940,
        FeedbackEventType::VoiceIdDriftAlert => 760,
        FeedbackEventType::VoiceIdReauthFriction => 710,
        FeedbackEventType::VoiceIdSpoofRisk => 990,
        FeedbackEventType::VoiceIdConfusionPair => 910,
        FeedbackEventType::VoiceIdDrift => 700,
        FeedbackEventType::VoiceIdLowQuality => 680,
    };
    let latency_component = (event.metrics.latency_ms as i32 / 20).min(600);
    let retries_component = (event.metrics.retries as i32) * 40;
    let missing_fields_component = (event.metrics.missing_fields.len() as i32) * 35;
    let path_component = match event.path_type {
        FeedbackPathType::Defect => 180,
        FeedbackPathType::Improvement => 0,
    };
    let gold_component = match event.gold_status {
        FeedbackGoldStatus::Verified => 120,
        FeedbackGoldStatus::Pending => 40,
        FeedbackGoldStatus::NotRequired => 0,
    };
    let total = event_weight
        + latency_component
        + retries_component
        + missing_fields_component
        + path_component
        + gold_component;
    total.clamp(-20_000, 20_000) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1feedback::{
        FeedbackConfidenceBucket, FeedbackEventCollectRequest, FeedbackEventType,
        FeedbackGoldProvenanceMethod, FeedbackGoldStatus, FeedbackMetrics, FeedbackPathType,
        FeedbackRequestEnvelope, FeedbackSignalEmitRequest, FeedbackToolStatus,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1FeedbackRuntime {
        Ph1FeedbackRuntime::new(Ph1FeedbackConfig::mvp_v1())
    }

    fn envelope(max_events: u8, max_signals: u8) -> FeedbackRequestEnvelope {
        FeedbackRequestEnvelope::v1(CorrelationId(3201), TurnId(291), max_events, max_signals)
            .unwrap()
    }

    fn event(
        event_id: &str,
        event_type: FeedbackEventType,
        latency_ms: u32,
    ) -> FeedbackEventRecord {
        FeedbackEventRecord::v1(
            event_id.to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3201),
            TurnId(291),
            event_type,
            ReasonCodeId(9),
            format!("evidence:feedback:{}", event_id),
            format!("idem:feedback:{}", event_id),
            FeedbackMetrics::v1(
                latency_ms,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn at_feedback_01_collect_output_is_schema_valid() {
        let req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(
                envelope(8, 4),
                vec![event("event_1", FeedbackEventType::SttReject, 420)],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_feedback_02_collect_order_is_deterministic() {
        let req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(
                envelope(8, 4),
                vec![
                    event("event_a", FeedbackEventType::SttReject, 400),
                    event("event_b", FeedbackEventType::ToolFail, 700),
                ],
            )
            .unwrap(),
        );

        let out_1 = runtime().run(&req);
        let out_2 = runtime().run(&req);

        let first_1 = match out_1 {
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => ok.selected_candidate_id,
            _ => panic!("expected FeedbackEventCollectOk"),
        };
        let first_2 = match out_2 {
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => ok.selected_candidate_id,
            _ => panic!("expected FeedbackEventCollectOk"),
        };
        assert_eq!(first_1, first_2);
    }

    #[test]
    fn at_feedback_03_budget_bound_is_enforced() {
        let req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(
                envelope(1, 1),
                vec![
                    event("event_a", FeedbackEventType::SttReject, 400),
                    event("event_b", FeedbackEventType::ToolFail, 700),
                ],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1FeedbackResponse::Refuse(refuse) => assert_eq!(
                refuse.reason_code,
                reason_codes::PH1_FEEDBACK_BUDGET_EXCEEDED
            ),
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_feedback_04_signal_emit_fails_on_selection_drift() {
        let collect_req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(
                envelope(8, 4),
                vec![
                    event("event_a", FeedbackEventType::SttReject, 400),
                    event("event_b", FeedbackEventType::ToolFail, 700),
                ],
            )
            .unwrap(),
        );
        let collect_ok = match runtime().run(&collect_req) {
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => ok,
            _ => panic!("expected FeedbackEventCollectOk"),
        };

        let ordered = collect_ok.ordered_signal_candidates;
        assert!(ordered.len() >= 2, "test requires at least two candidates");
        let drifted =
            FeedbackSignalEmitRequest::v1(envelope(8, 4), ordered[1].candidate_id.clone(), ordered)
                .unwrap();
        let out = runtime().run(&Ph1FeedbackRequest::FeedbackSignalEmit(drifted));
        match out {
            Ph1FeedbackResponse::FeedbackSignalEmitOk(ok) => {
                assert_eq!(ok.validation_status, FeedbackValidationStatus::Fail)
            }
            _ => panic!("expected FeedbackSignalEmitOk"),
        }
    }

    #[test]
    fn at_feedback_05_defect_path_is_prioritized_over_improvement() {
        let defect = FeedbackEventRecord::v2(
            "event_defect".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3201),
            TurnId(291),
            FeedbackEventType::UserCorrection,
            FeedbackPathType::Defect,
            None,
            None,
            FeedbackGoldStatus::NotRequired,
            ReasonCodeId(9),
            "evidence:feedback:defect".to_string(),
            "idem:feedback:defect".to_string(),
            FeedbackMetrics::v1(
                420,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap();
        let improvement = FeedbackEventRecord::v2(
            "event_improvement".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3201),
            TurnId(291),
            FeedbackEventType::UserCorrection,
            FeedbackPathType::Improvement,
            Some("gold_case_1".to_string()),
            Some(FeedbackGoldProvenanceMethod::VerifiedHumanCorrection),
            FeedbackGoldStatus::Verified,
            ReasonCodeId(9),
            "evidence:feedback:improvement".to_string(),
            "idem:feedback:improvement".to_string(),
            FeedbackMetrics::v1(
                420,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap();

        let req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(envelope(8, 4), vec![improvement, defect]).unwrap(),
        );
        let out = runtime().run(&req);
        match out {
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => {
                assert_eq!(ok.selected_candidate_id, "signal_event_defect".to_string());
            }
            _ => panic!("expected FeedbackEventCollectOk"),
        }
    }
}
