#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1feedback::{
    FeedbackCapabilityId, FeedbackEventCollectOk, FeedbackEventCollectRequest, FeedbackEventRecord,
    FeedbackRefuse, FeedbackRequestEnvelope, FeedbackSignalEmitOk, FeedbackSignalEmitRequest,
    FeedbackValidationStatus, Ph1FeedbackRequest, Ph1FeedbackResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1selfheal::{
    stable_card_id, FailureContainmentAction, FailureEvent, FailureProviderContext,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.FEEDBACK OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_FEEDBACK_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4642_0101);
    pub const PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4642_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1FeedbackWiringConfig {
    pub feedback_enabled: bool,
    pub max_events: u8,
    pub max_signals: u8,
}

impl Ph1FeedbackWiringConfig {
    pub fn mvp_v1(feedback_enabled: bool) -> Self {
        Self {
            feedback_enabled,
            max_events: 24,
            max_signals: 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub events: Vec<FeedbackEventRecord>,
}

impl FeedbackTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        events: Vec<FeedbackEventRecord>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            events,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for FeedbackTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.events.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_turn_input.events",
                reason: "must be <= 64",
            });
        }
        for event in &self.events {
            event.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub event_collect: FeedbackEventCollectOk,
    pub signal_emit: FeedbackSignalEmitOk,
}

impl FeedbackForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        event_collect: FeedbackEventCollectOk,
        signal_emit: FeedbackSignalEmitOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            event_collect,
            signal_emit,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for FeedbackForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.event_collect.validate()?;
        self.signal_emit.validate()?;
        if self.signal_emit.validation_status != FeedbackValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_forward_bundle.signal_emit.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoEvents,
    Refused(FeedbackRefuse),
    Forwarded(FeedbackForwardBundle),
}

pub fn map_feedback_event_to_failure_event(
    event: &FeedbackEventRecord,
    forward_bundle: &FeedbackForwardBundle,
    containment_action: FailureContainmentAction,
    escalation_required: bool,
    unresolved_reason: Option<String>,
    provider_context: Option<FailureProviderContext>,
) -> Result<FailureEvent, ContractViolation> {
    event.validate()?;
    forward_bundle.validate()?;

    if event.correlation_id != forward_bundle.correlation_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_feedback_event_to_failure_event.event.correlation_id",
            reason: "must match feedback_forward_bundle.correlation_id",
        });
    }
    if event.turn_id != forward_bundle.turn_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_feedback_event_to_failure_event.event.turn_id",
            reason: "must match feedback_forward_bundle.turn_id",
        });
    }

    let reason_hex = format!("{:08x}", event.reason_code.0);
    let route_domain = provider_context
        .as_ref()
        .map(|ctx| route_domain_key(ctx.route_domain))
        .unwrap_or("NONE");
    let fingerprint = stable_card_id(
        "fingerprint",
        &[
            feedback_event_type_key(event.event_type),
            feedback_path_type_key(event.path_type),
            reason_hex.as_str(),
            event.evidence_ref.as_str(),
            route_domain,
        ],
    )?;

    FailureEvent::v1(
        event.event_id.clone(),
        event.tenant_id.clone(),
        event.user_id.clone(),
        event.speaker_id.clone(),
        event.session_id.clone(),
        event.device_id.clone(),
        event.correlation_id,
        event.turn_id,
        event.event_type,
        event.reason_code,
        event.path_type,
        event.evidence_ref.clone(),
        event.idempotency_key.clone(),
        event.metrics.confidence_bucket,
        event.metrics.tool_status,
        event.metrics.latency_ms,
        event.metrics.retries,
        event.metrics.missing_fields.clone(),
        fingerprint,
        containment_action,
        escalation_required,
        unresolved_reason,
        provider_context,
    )
}

fn feedback_event_type_key(
    event_type: selene_kernel_contracts::ph1feedback::FeedbackEventType,
) -> &'static str {
    use selene_kernel_contracts::ph1feedback::FeedbackEventType as T;
    match event_type {
        T::SttReject => "STT_REJECT",
        T::SttRetry => "STT_RETRY",
        T::LanguageMismatch => "LANGUAGE_MISMATCH",
        T::UserCorrection => "USER_CORRECTION",
        T::ClarifyLoop => "CLARIFY_LOOP",
        T::ConfirmAbort => "CONFIRM_ABORT",
        T::ToolFail => "TOOL_FAIL",
        T::MemoryOverride => "MEMORY_OVERRIDE",
        T::DeliverySwitch => "DELIVERY_SWITCH",
        T::BargeIn => "BARGE_IN",
        T::VoiceIdFalseReject => "VOICE_ID_FALSE_REJECT",
        T::VoiceIdFalseAccept => "VOICE_ID_FALSE_ACCEPT",
        T::VoiceIdSpoofRisk => "VOICE_ID_SPOOF_RISK",
        T::VoiceIdMultiSpeaker => "VOICE_ID_MULTI_SPEAKER",
        T::VoiceIdDriftAlert => "VOICE_ID_DRIFT_ALERT",
        T::VoiceIdReauthFriction => "VOICE_ID_REAUTH_FRICTION",
        T::VoiceIdConfusionPair => "VOICE_ID_CONFUSION_PAIR",
        T::VoiceIdDrift => "VOICE_ID_DRIFT",
        T::VoiceIdLowQuality => "VOICE_ID_LOW_QUALITY",
    }
}

fn feedback_path_type_key(
    path_type: selene_kernel_contracts::ph1feedback::FeedbackPathType,
) -> &'static str {
    use selene_kernel_contracts::ph1feedback::FeedbackPathType as P;
    match path_type {
        P::Defect => "DEFECT",
        P::Improvement => "IMPROVEMENT",
    }
}

fn route_domain_key(route_domain: selene_kernel_contracts::ph1pae::PaeRouteDomain) -> &'static str {
    use selene_kernel_contracts::ph1pae::PaeRouteDomain as D;
    match route_domain {
        D::Stt => "STT",
        D::Tts => "TTS",
        D::Llm => "LLM",
        D::Tooling => "TOOLING",
    }
}

pub trait Ph1FeedbackEngine {
    fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1FeedbackWiring<E>
where
    E: Ph1FeedbackEngine,
{
    config: Ph1FeedbackWiringConfig,
    engine: E,
}

impl<E> Ph1FeedbackWiring<E>
where
    E: Ph1FeedbackEngine,
{
    pub fn new(config: Ph1FeedbackWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_events == 0 || config.max_events > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1feedback_wiring_config.max_events",
                reason: "must be within 1..=64",
            });
        }
        if config.max_signals == 0 || config.max_signals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1feedback_wiring_config.max_signals",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &FeedbackTurnInput,
    ) -> Result<FeedbackWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.feedback_enabled {
            return Ok(FeedbackWiringOutcome::NotInvokedDisabled);
        }
        if input.events.is_empty() {
            return Ok(FeedbackWiringOutcome::NotInvokedNoEvents);
        }

        let envelope = FeedbackRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_events, 64),
            min(self.config.max_signals, 32),
        )?;

        let collect_req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(envelope.clone(), input.events.clone())?,
        );
        let collect_resp = self.engine.run(&collect_req);
        collect_resp.validate()?;

        let collect_ok = match collect_resp {
            Ph1FeedbackResponse::Refuse(refuse) => {
                return Ok(FeedbackWiringOutcome::Refused(refuse))
            }
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => ok,
            Ph1FeedbackResponse::FeedbackSignalEmitOk(_) => {
                return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                    FeedbackCapabilityId::FeedbackEventCollect,
                    reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-emit response for event-collect request".to_string(),
                )?));
            }
        };

        let emit_req = Ph1FeedbackRequest::FeedbackSignalEmit(FeedbackSignalEmitRequest::v1(
            envelope,
            collect_ok.selected_candidate_id.clone(),
            collect_ok.ordered_signal_candidates.clone(),
        )?);
        let emit_resp = self.engine.run(&emit_req);
        emit_resp.validate()?;

        let emit_ok = match emit_resp {
            Ph1FeedbackResponse::Refuse(refuse) => {
                return Ok(FeedbackWiringOutcome::Refused(refuse))
            }
            Ph1FeedbackResponse::FeedbackSignalEmitOk(ok) => ok,
            Ph1FeedbackResponse::FeedbackEventCollectOk(_) => {
                return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                    FeedbackCapabilityId::FeedbackSignalEmit,
                    reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                    "unexpected event-collect response for signal-emit request".to_string(),
                )?));
            }
        };

        if emit_ok.validation_status != FeedbackValidationStatus::Ok {
            return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                FeedbackCapabilityId::FeedbackSignalEmit,
                reason_codes::PH1_FEEDBACK_VALIDATION_FAILED,
                "feedback signal-emit validation failed".to_string(),
            )?));
        }

        let bundle =
            FeedbackForwardBundle::v1(input.correlation_id, input.turn_id, collect_ok, emit_ok)?;
        Ok(FeedbackWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::Ph1dProviderTask;
    use selene_kernel_contracts::ph1feedback::{
        FeedbackConfidenceBucket, FeedbackEventType, FeedbackMetrics, FeedbackToolStatus,
    };
    use selene_kernel_contracts::ph1pae::{PaeProviderSlot, PaeRouteDomain};
    use selene_kernel_contracts::ph1selfheal::FailureContainmentAction;
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicFeedbackEngine;

    impl Ph1FeedbackEngine for DeterministicFeedbackEngine {
        fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
            match req {
                Ph1FeedbackRequest::FeedbackEventCollect(r) => {
                    let mut candidates = r
                        .events
                        .iter()
                        .enumerate()
                        .map(|(idx, event)| {
                            selene_kernel_contracts::ph1feedback::FeedbackSignalCandidate::v1(
                                format!("signal_{}", event.event_id),
                                event.event_type,
                                "correction_rate_by_intent".to_string(),
                                selene_kernel_contracts::ph1feedback::FeedbackSignalTarget::LearnPackage,
                                1200 - (idx as i16 * 100),
                                1,
                                event.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    candidates.sort_by(|a, b| {
                        b.signal_value_bp
                            .cmp(&a.signal_value_bp)
                            .then(a.candidate_id.cmp(&b.candidate_id))
                    });
                    Ph1FeedbackResponse::FeedbackEventCollectOk(
                        FeedbackEventCollectOk::v1(
                            ReasonCodeId(101),
                            candidates[0].candidate_id.clone(),
                            candidates,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1FeedbackRequest::FeedbackSignalEmit(_r) => {
                    Ph1FeedbackResponse::FeedbackSignalEmitOk(
                        FeedbackSignalEmitOk::v1(
                            ReasonCodeId(102),
                            FeedbackValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftFeedbackEngine;

    impl Ph1FeedbackEngine for DriftFeedbackEngine {
        fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
            match req {
                Ph1FeedbackRequest::FeedbackEventCollect(r) => {
                    let candidate = selene_kernel_contracts::ph1feedback::FeedbackSignalCandidate::v1(
                        format!("signal_{}", r.events[0].event_id),
                        r.events[0].event_type,
                        "correction_rate_by_intent".to_string(),
                        selene_kernel_contracts::ph1feedback::FeedbackSignalTarget::LearnPackage,
                        900,
                        1,
                        r.events[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1FeedbackResponse::FeedbackEventCollectOk(
                        FeedbackEventCollectOk::v1(
                            ReasonCodeId(111),
                            candidate.candidate_id.clone(),
                            vec![candidate],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1FeedbackRequest::FeedbackSignalEmit(_r) => {
                    Ph1FeedbackResponse::FeedbackSignalEmitOk(
                        FeedbackSignalEmitOk::v1(
                            ReasonCodeId(112),
                            FeedbackValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_candidates".to_string()],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn event(event_id: &str, event_type: FeedbackEventType) -> FeedbackEventRecord {
        FeedbackEventRecord::v1(
            event_id.to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3301),
            TurnId(301),
            event_type,
            ReasonCodeId(10),
            "evidence:feedback:1".to_string(),
            "idem:feedback:1".to_string(),
            FeedbackMetrics::v1(
                400,
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
    fn at_feedback_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3301),
            TurnId(301),
            vec![
                event("event_1", FeedbackEventType::SttReject),
                event("event_2", FeedbackEventType::UserCorrection),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FeedbackWiringOutcome::Forwarded(bundle) => assert!(bundle.validate().is_ok()),
            _ => panic!("expected Forwarded outcome"),
        }
    }

    #[test]
    fn at_feedback_02_os_output_is_deterministic() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3302),
            TurnId(302),
            vec![
                event("event_1", FeedbackEventType::SttReject),
                event("event_2", FeedbackEventType::UserCorrection),
            ],
        )
        .unwrap();

        let out_1 = wiring.run_turn(&input).unwrap();
        let out_2 = wiring.run_turn(&input).unwrap();

        let selected_1 = match out_1 {
            FeedbackWiringOutcome::Forwarded(bundle) => bundle.event_collect.selected_candidate_id,
            _ => panic!("expected Forwarded outcome"),
        };
        let selected_2 = match out_2 {
            FeedbackWiringOutcome::Forwarded(bundle) => bundle.event_collect.selected_candidate_id,
            _ => panic!("expected Forwarded outcome"),
        };
        assert_eq!(selected_1, selected_2);
    }

    #[test]
    fn at_feedback_03_os_does_not_invoke_when_feedback_is_disabled() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(false),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3303),
            TurnId(303),
            vec![event("event_1", FeedbackEventType::SttReject)],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert!(matches!(out, FeedbackWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_feedback_04_os_fails_closed_on_signal_emit_validation_drift() {
        let wiring =
            Ph1FeedbackWiring::new(Ph1FeedbackWiringConfig::mvp_v1(true), DriftFeedbackEngine)
                .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3304),
            TurnId(304),
            vec![event("event_1", FeedbackEventType::SttReject)],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FeedbackWiringOutcome::Refused(refuse) => assert_eq!(
                refuse.reason_code,
                reason_codes::PH1_FEEDBACK_VALIDATION_FAILED
            ),
            _ => panic!("expected Refused outcome"),
        }
    }

    #[test]
    fn at_feedback_05_mapper_builds_deterministic_failure_event() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let mut event = event("event_55", FeedbackEventType::ToolFail);
        event.correlation_id = CorrelationId(3355);
        event.turn_id = TurnId(355);
        let input =
            FeedbackTurnInput::v1(CorrelationId(3355), TurnId(355), vec![event.clone()]).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let FeedbackWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded feedback bundle");
        };
        let provider_context = FailureProviderContext::v1(
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            Ph1dProviderTask::OcrTextExtract,
            12_000,
            77,
            false,
        )
        .unwrap();

        let failure_a = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            true,
            Some("provider timeout".to_string()),
            Some(provider_context.clone()),
        )
        .unwrap();
        let failure_b = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            true,
            Some("provider timeout".to_string()),
            Some(provider_context),
        )
        .unwrap();
        assert_eq!(failure_a.fingerprint, failure_b.fingerprint);
        assert_eq!(failure_a.failure_id, event.event_id);
        assert!(failure_a.escalation_required);
        assert_eq!(
            failure_a.unresolved_reason.as_deref(),
            Some("provider timeout")
        );
    }

    #[test]
    fn at_feedback_06_mapper_fails_closed_on_correlation_mismatch() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let mut event = event("event_56", FeedbackEventType::SttReject);
        event.correlation_id = CorrelationId(3356);
        event.turn_id = TurnId(356);
        let input =
            FeedbackTurnInput::v1(CorrelationId(3356), TurnId(356), vec![event.clone()]).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let FeedbackWiringOutcome::Forwarded(mut bundle) = out else {
            panic!("expected forwarded feedback bundle");
        };
        bundle.correlation_id = CorrelationId(9_999);

        let err = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            false,
            None,
            None,
        )
        .expect_err("mapper should fail closed on correlation mismatch");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "map_feedback_event_to_failure_event.event.correlation_id"
                );
            }
            _ => panic!("expected invalid-value violation"),
        }
    }
}
