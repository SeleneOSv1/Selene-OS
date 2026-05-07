#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1listen::{
    ListenCapabilityId, ListenCorrectionSnapshot, ListenRefuse, ListenRequestEnvelope,
    ListenSessionContext, ListenSignalCollectOk, ListenSignalCollectRequest, ListenSignalFilterOk,
    ListenSignalFilterRequest, ListenSignalWindow, ListenValidationStatus, Ph1ListenRequest,
    Ph1ListenResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LISTEN OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LISTEN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C49_0101);
    pub const PH1_LISTEN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C49_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ListenWiringConfig {
    pub listen_enabled: bool,
    pub max_signal_windows: u8,
    pub max_adjustments: u8,
    pub max_diagnostics: u8,
}

impl Ph1ListenWiringConfig {
    pub fn mvp_v1(listen_enabled: bool) -> Self {
        Self {
            listen_enabled,
            max_signal_windows: 24,
            max_adjustments: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_windows: Vec<ListenSignalWindow>,
    pub correction_snapshot: ListenCorrectionSnapshot,
    pub session_context: ListenSessionContext,
}

impl ListenTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_windows: Vec<ListenSignalWindow>,
        correction_snapshot: ListenCorrectionSnapshot,
        session_context: ListenSessionContext,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            signal_windows,
            correction_snapshot,
            session_context,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for ListenTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.correction_snapshot.validate()?;
        self.session_context.validate()?;
        if self.signal_windows.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_turn_input.signal_windows",
                reason: "must be <= 64",
            });
        }
        for signal_window in &self.signal_windows {
            signal_window.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_collect: ListenSignalCollectOk,
    pub signal_filter: ListenSignalFilterOk,
}

impl ListenForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_collect: ListenSignalCollectOk,
        signal_filter: ListenSignalFilterOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            signal_collect,
            signal_filter,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for ListenForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.signal_collect.validate()?;
        self.signal_filter.validate()?;
        if self.signal_filter.validation_status != ListenValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "listen_forward_bundle.signal_filter.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(ListenRefuse),
    Forwarded(ListenForwardBundle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34nListeningProofSource {
    ForegroundMic,
    ControlledReplayPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34nListeningTrialKind {
    Positive,
    QuietControl,
    SelfEchoControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage34nListeningProofConfig {
    pub max_capture_window_seconds: u16,
    pub max_positive_trials: u8,
    pub max_control_trials: u8,
    pub max_latency_ms: u16,
    pub min_confidence_bp: u16,
    pub max_wer_bp: u16,
}

impl Stage34nListeningProofConfig {
    pub fn controlled_stage34n() -> Self {
        Self {
            max_capture_window_seconds: 20,
            max_positive_trials: 6,
            max_control_trials: 6,
            max_latency_ms: 1200,
            min_confidence_bp: 8500,
            max_wer_bp: 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage34nListeningTrial {
    pub trial_id: String,
    pub kind: Stage34nListeningTrialKind,
    pub expected_transcript: Option<String>,
    pub observed_transcript: Option<String>,
    pub endpoint_final: bool,
    pub confidence_bp: u16,
    pub latency_ms: u16,
    pub committed_current_turn_transcript: bool,
    pub current_turn_only: bool,
    pub foreground_explicit_input: bool,
    pub background_listening: bool,
    pub answer_generated: bool,
    pub tts_requested: bool,
    pub voice_id_invoked: bool,
    pub provider_call_count: u8,
    pub tool_route_invoked: bool,
    pub protected_execution_requested: bool,
    pub raw_audio_committed: bool,
}

impl Stage34nListeningTrial {
    #[allow(clippy::too_many_arguments)]
    pub fn controlled(
        trial_id: impl Into<String>,
        kind: Stage34nListeningTrialKind,
        expected_transcript: Option<impl Into<String>>,
        observed_transcript: Option<impl Into<String>>,
        endpoint_final: bool,
        confidence_bp: u16,
        latency_ms: u16,
        committed_current_turn_transcript: bool,
        current_turn_only: bool,
    ) -> Self {
        Self {
            trial_id: trial_id.into(),
            kind,
            expected_transcript: expected_transcript.map(Into::into),
            observed_transcript: observed_transcript.map(Into::into),
            endpoint_final,
            confidence_bp,
            latency_ms,
            committed_current_turn_transcript,
            current_turn_only,
            foreground_explicit_input: true,
            background_listening: false,
            answer_generated: false,
            tts_requested: false,
            voice_id_invoked: false,
            provider_call_count: 0,
            tool_route_invoked: false,
            protected_execution_requested: false,
            raw_audio_committed: false,
        }
    }

    pub fn with_forbidden_downstream_work(mut self) -> Self {
        self.answer_generated = true;
        self.tts_requested = true;
        self.voice_id_invoked = true;
        self.provider_call_count = 1;
        self.tool_route_invoked = true;
        self.protected_execution_requested = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage34nListeningProofReport {
    pub source: Stage34nListeningProofSource,
    pub capture_window_seconds: u16,
    pub positive_trials: u8,
    pub control_trials: u8,
    pub max_latency_ms: u16,
    pub min_confidence_bp: u16,
    pub max_wer_bp: u16,
    pub provider_call_count: u8,
    pub raw_audio_committed: bool,
    pub background_listening: bool,
    pub transcript_boundary_current_turn_only: bool,
    pub downstream_work_absent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34nListeningProofError {
    CaptureWindowTooLong,
    MissingPositiveTrial,
    MissingControlTrial,
    TooManyPositiveTrials,
    TooManyControlTrials,
    MissingExpectedTranscript,
    MissingObservedTranscript,
    PositiveTranscriptNotCommitted,
    ControlCommittedTranscript,
    TranscriptNotCurrentTurnOnly,
    ForegroundInputMissing,
    EndpointIncomplete,
    ConfidenceBelowTarget,
    WerAboveTarget,
    LatencyAboveTarget,
    BackgroundListening,
    RawAudioCommitted,
    ProviderCallAttempted,
    DownstreamWorkAttempted,
    ProtectedExecutionAttempted,
}

pub fn verify_stage34n_controlled_listening_proof(
    config: Stage34nListeningProofConfig,
    source: Stage34nListeningProofSource,
    capture_window_seconds: u16,
    trials: &[Stage34nListeningTrial],
) -> Result<Stage34nListeningProofReport, Stage34nListeningProofError> {
    if capture_window_seconds > config.max_capture_window_seconds {
        return Err(Stage34nListeningProofError::CaptureWindowTooLong);
    }

    let mut positive_trials = 0u8;
    let mut control_trials = 0u8;
    let mut max_latency_ms = 0u16;
    let mut min_confidence_bp = u16::MAX;
    let mut max_wer_bp = 0u16;
    let mut provider_call_count = 0u8;

    for trial in trials {
        if !trial.foreground_explicit_input {
            return Err(Stage34nListeningProofError::ForegroundInputMissing);
        }
        if trial.background_listening {
            return Err(Stage34nListeningProofError::BackgroundListening);
        }
        if trial.raw_audio_committed {
            return Err(Stage34nListeningProofError::RawAudioCommitted);
        }
        if trial.provider_call_count > 0 {
            return Err(Stage34nListeningProofError::ProviderCallAttempted);
        }
        if trial.answer_generated
            || trial.tts_requested
            || trial.voice_id_invoked
            || trial.tool_route_invoked
        {
            return Err(Stage34nListeningProofError::DownstreamWorkAttempted);
        }
        if trial.protected_execution_requested {
            return Err(Stage34nListeningProofError::ProtectedExecutionAttempted);
        }
        if !trial.current_turn_only {
            return Err(Stage34nListeningProofError::TranscriptNotCurrentTurnOnly);
        }

        provider_call_count = provider_call_count.saturating_add(trial.provider_call_count);

        match trial.kind {
            Stage34nListeningTrialKind::Positive => {
                positive_trials = positive_trials.saturating_add(1);
                if positive_trials > config.max_positive_trials {
                    return Err(Stage34nListeningProofError::TooManyPositiveTrials);
                }
                if !trial.committed_current_turn_transcript {
                    return Err(Stage34nListeningProofError::PositiveTranscriptNotCommitted);
                }
                if !trial.endpoint_final {
                    return Err(Stage34nListeningProofError::EndpointIncomplete);
                }
                if trial.confidence_bp < config.min_confidence_bp {
                    return Err(Stage34nListeningProofError::ConfidenceBelowTarget);
                }
                if trial.latency_ms > config.max_latency_ms {
                    return Err(Stage34nListeningProofError::LatencyAboveTarget);
                }
                let expected = trial
                    .expected_transcript
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(Stage34nListeningProofError::MissingExpectedTranscript)?;
                let observed = trial
                    .observed_transcript
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(Stage34nListeningProofError::MissingObservedTranscript)?;
                let wer_bp = stage34n_word_error_rate_bp(expected, observed);
                if wer_bp > config.max_wer_bp {
                    return Err(Stage34nListeningProofError::WerAboveTarget);
                }
                max_wer_bp = max_wer_bp.max(wer_bp);
                max_latency_ms = max_latency_ms.max(trial.latency_ms);
                min_confidence_bp = min_confidence_bp.min(trial.confidence_bp);
            }
            Stage34nListeningTrialKind::QuietControl
            | Stage34nListeningTrialKind::SelfEchoControl => {
                control_trials = control_trials.saturating_add(1);
                if control_trials > config.max_control_trials {
                    return Err(Stage34nListeningProofError::TooManyControlTrials);
                }
                if trial.committed_current_turn_transcript {
                    return Err(Stage34nListeningProofError::ControlCommittedTranscript);
                }
                if trial
                    .observed_transcript
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Err(Stage34nListeningProofError::ControlCommittedTranscript);
                }
            }
        }
    }

    if positive_trials == 0 {
        return Err(Stage34nListeningProofError::MissingPositiveTrial);
    }
    if control_trials == 0 {
        return Err(Stage34nListeningProofError::MissingControlTrial);
    }

    Ok(Stage34nListeningProofReport {
        source,
        capture_window_seconds,
        positive_trials,
        control_trials,
        max_latency_ms,
        min_confidence_bp,
        max_wer_bp,
        provider_call_count,
        raw_audio_committed: false,
        background_listening: false,
        transcript_boundary_current_turn_only: true,
        downstream_work_absent: true,
    })
}

fn stage34n_word_error_rate_bp(expected: &str, observed: &str) -> u16 {
    let expected_words = stage34n_words(expected);
    let observed_words = stage34n_words(observed);
    if expected_words.is_empty() {
        return if observed_words.is_empty() { 0 } else { 10_000 };
    }

    let distance = stage34n_levenshtein_words(&expected_words, &observed_words);
    let bp = (distance * 10_000 + expected_words.len() / 2) / expected_words.len();
    bp.min(10_000) as u16
}

fn stage34n_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|ch| ch.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

fn stage34n_levenshtein_words(expected: &[String], observed: &[String]) -> usize {
    let mut previous = (0..=observed.len()).collect::<Vec<_>>();
    let mut current = vec![0; observed.len() + 1];

    for (row, expected_word) in expected.iter().enumerate() {
        current[0] = row + 1;
        for (col, observed_word) in observed.iter().enumerate() {
            let substitution_cost = usize::from(expected_word != observed_word);
            current[col + 1] = (previous[col + 1] + 1)
                .min(current[col] + 1)
                .min(previous[col] + substitution_cost);
        }
        previous.clone_from(&current);
    }

    previous[observed.len()]
}

pub trait Ph1ListenEngine {
    fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1ListenWiring<E>
where
    E: Ph1ListenEngine,
{
    config: Ph1ListenWiringConfig,
    engine: E,
}

impl<E> Ph1ListenWiring<E>
where
    E: Ph1ListenEngine,
{
    pub fn new(config: Ph1ListenWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signal_windows == 0 || config.max_signal_windows > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_signal_windows",
                reason: "must be within 1..=64",
            });
        }
        if config.max_adjustments == 0 || config.max_adjustments > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_adjustments",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &ListenTurnInput,
    ) -> Result<ListenWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.listen_enabled {
            return Ok(ListenWiringOutcome::NotInvokedDisabled);
        }
        if input.signal_windows.is_empty() {
            return Ok(ListenWiringOutcome::NotInvokedNoSignals);
        }

        let envelope = ListenRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signal_windows, 64),
            min(self.config.max_adjustments, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let collect_req = Ph1ListenRequest::ListenSignalCollect(ListenSignalCollectRequest::v1(
            envelope.clone(),
            input.signal_windows.clone(),
            input.correction_snapshot.clone(),
            input.session_context.clone(),
        )?);
        let collect_resp = self.engine.run(&collect_req);
        collect_resp.validate()?;

        let collect_ok = match collect_resp {
            Ph1ListenResponse::Refuse(refuse) => return Ok(ListenWiringOutcome::Refused(refuse)),
            Ph1ListenResponse::ListenSignalCollectOk(ok) => ok,
            Ph1ListenResponse::ListenSignalFilterOk(_) => {
                return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                    ListenCapabilityId::ListenSignalCollect,
                    reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-filter response for signal-collect request".to_string(),
                )?));
            }
        };

        let filter_req = Ph1ListenRequest::ListenSignalFilter(ListenSignalFilterRequest::v1(
            envelope,
            collect_ok.environment_profile_ref.clone(),
            collect_ok.selected_adjustment_id.clone(),
            collect_ok.ordered_adjustments.clone(),
            true,
        )?);
        let filter_resp = self.engine.run(&filter_req);
        filter_resp.validate()?;

        let filter_ok = match filter_resp {
            Ph1ListenResponse::Refuse(refuse) => return Ok(ListenWiringOutcome::Refused(refuse)),
            Ph1ListenResponse::ListenSignalFilterOk(ok) => ok,
            Ph1ListenResponse::ListenSignalCollectOk(_) => {
                return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                    ListenCapabilityId::ListenSignalFilter,
                    reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-collect response for signal-filter request".to_string(),
                )?));
            }
        };

        if filter_ok.validation_status != ListenValidationStatus::Ok {
            return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                ListenCapabilityId::ListenSignalFilter,
                reason_codes::PH1_LISTEN_VALIDATION_FAILED,
                "listen signal-filter validation failed".to_string(),
            )?));
        }

        let bundle =
            ListenForwardBundle::v1(input.correlation_id, input.turn_id, collect_ok, filter_ok)?;
        Ok(ListenWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1listen::{
        ListenAdjustmentHint, ListenCaptureProfile, ListenDeliveryPolicyHint,
        ListenEndpointProfile, ListenEnvironmentMode,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicListenEngine;

    impl Ph1ListenEngine for DeterministicListenEngine {
        fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse {
            match req {
                Ph1ListenRequest::ListenSignalCollect(r) => {
                    let mut adjustments = r
                        .signal_windows
                        .iter()
                        .enumerate()
                        .map(|(idx, window)| {
                            ListenAdjustmentHint::v1(
                                format!("adj_{}", window.window_id),
                                if r.session_context.session_mode_meeting {
                                    ListenEnvironmentMode::Meeting
                                } else {
                                    ListenEnvironmentMode::Office
                                },
                                ListenCaptureProfile::Standard,
                                ListenEndpointProfile::Balanced,
                                ListenDeliveryPolicyHint::VoicePreferred,
                                1000 - (idx as i16 * 100),
                                window.window_id.clone(),
                                window.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    adjustments.sort_by(|a, b| {
                        b.priority_bp
                            .cmp(&a.priority_bp)
                            .then(a.adjustment_id.cmp(&b.adjustment_id))
                    });

                    Ph1ListenResponse::ListenSignalCollectOk(
                        ListenSignalCollectOk::v1(
                            ReasonCodeId(201),
                            "env:office:standard:voice_preferred".to_string(),
                            adjustments[0].adjustment_id.clone(),
                            adjustments,
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ListenRequest::ListenSignalFilter(_r) => {
                    Ph1ListenResponse::ListenSignalFilterOk(
                        ListenSignalFilterOk::v1(
                            ReasonCodeId(202),
                            ListenValidationStatus::Ok,
                            vec![],
                            true,
                            true,
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

    struct DriftListenEngine;

    impl Ph1ListenEngine for DriftListenEngine {
        fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse {
            match req {
                Ph1ListenRequest::ListenSignalCollect(r) => {
                    let adjustment = ListenAdjustmentHint::v1(
                        format!("adj_{}", r.signal_windows[0].window_id),
                        ListenEnvironmentMode::Noisy,
                        ListenCaptureProfile::NoiseSuppressed,
                        ListenEndpointProfile::Balanced,
                        ListenDeliveryPolicyHint::TextPreferred,
                        900,
                        r.signal_windows[0].window_id.clone(),
                        r.signal_windows[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1ListenResponse::ListenSignalCollectOk(
                        ListenSignalCollectOk::v1(
                            ReasonCodeId(211),
                            "env:noisy:noise_suppressed:text_preferred".to_string(),
                            adjustment.adjustment_id.clone(),
                            vec![adjustment],
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ListenRequest::ListenSignalFilter(_r) => {
                    Ph1ListenResponse::ListenSignalFilterOk(
                        ListenSignalFilterOk::v1(
                            ReasonCodeId(212),
                            ListenValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_adjustments".to_string()],
                            true,
                            true,
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

    fn window(id: &str, noise_level_dbfs: i16) -> ListenSignalWindow {
        ListenSignalWindow::v1(
            id.to_string(),
            "PH1.K".to_string(),
            8200,
            7700,
            noise_level_dbfs,
            0,
            420,
            format!("listen:evidence:{}", id),
        )
        .unwrap()
    }

    fn correction() -> ListenCorrectionSnapshot {
        ListenCorrectionSnapshot::v1(2, 1, 1, 1400).unwrap()
    }

    fn context() -> ListenSessionContext {
        ListenSessionContext::v1(false, false, false, false).unwrap()
    }

    fn input() -> ListenTurnInput {
        ListenTurnInput::v1(
            CorrelationId(3501),
            TurnId(321),
            vec![window("w_1", -30), window("w_2", -45)],
            correction(),
            context(),
        )
        .unwrap()
    }

    #[test]
    fn at_listen_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(true),
            DeterministicListenEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ListenWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_listen_02_os_output_is_deterministic() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(true),
            DeterministicListenEngine,
        )
        .unwrap();

        let out1 = wiring.run_turn(&input()).unwrap();
        let out2 = wiring.run_turn(&input()).unwrap();

        match (out1, out2) {
            (ListenWiringOutcome::Forwarded(a), ListenWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.signal_collect, b.signal_collect);
                assert_eq!(a.signal_filter, b.signal_filter);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_listen_03_os_does_not_invoke_when_listen_is_disabled() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(false),
            DeterministicListenEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, ListenWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_listen_04_os_fails_closed_on_signal_filter_validation_drift() {
        let wiring =
            Ph1ListenWiring::new(Ph1ListenWiringConfig::mvp_v1(true), DriftListenEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ListenWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_LISTEN_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }

    fn stage34n_positive_trial() -> Stage34nListeningTrial {
        Stage34nListeningTrial::controlled(
            "positive_1",
            Stage34nListeningTrialKind::Positive,
            Some("Selene listening test one"),
            Some("Selene listening test one"),
            true,
            9400,
            420,
            true,
            true,
        )
    }

    fn stage34n_quiet_control() -> Stage34nListeningTrial {
        Stage34nListeningTrial::controlled(
            "quiet_1",
            Stage34nListeningTrialKind::QuietControl,
            Option::<String>::None,
            Option::<String>::None,
            false,
            0,
            0,
            false,
            true,
        )
    }

    fn stage34n_self_echo_control() -> Stage34nListeningTrial {
        Stage34nListeningTrial::controlled(
            "self_echo_1",
            Stage34nListeningTrialKind::SelfEchoControl,
            Option::<String>::None,
            Option::<String>::None,
            false,
            0,
            0,
            false,
            true,
        )
    }

    #[test]
    fn stage_34n_listening_manifest_records_device_input_trials_and_stop_rules() {
        let trials = vec![stage34n_positive_trial(), stage34n_quiet_control()];
        let report = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            20,
            &trials,
        )
        .unwrap();

        assert_eq!(
            report.source,
            Stage34nListeningProofSource::ControlledReplayPack
        );
        assert_eq!(report.capture_window_seconds, 20);
        assert_eq!(report.positive_trials, 1);
        assert_eq!(report.control_trials, 1);
        assert_eq!(report.provider_call_count, 0);
        assert!(report.transcript_boundary_current_turn_only);
        assert!(report.downstream_work_absent);
    }

    #[test]
    fn stage_34n_positive_listening_trial_commits_current_turn_transcript_only() {
        let trials = vec![stage34n_positive_trial(), stage34n_self_echo_control()];
        let report = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ForegroundMic,
            12,
            &trials,
        )
        .unwrap();

        assert_eq!(report.max_wer_bp, 0);
        assert_eq!(report.max_latency_ms, 420);
        assert_eq!(report.min_confidence_bp, 9400);
        assert!(!report.raw_audio_committed);
        assert!(!report.background_listening);
    }

    #[test]
    fn stage_34n_quiet_control_rejects_without_committed_transcript() {
        let mut quiet = stage34n_quiet_control();
        quiet.committed_current_turn_transcript = true;

        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[stage34n_positive_trial(), quiet],
        )
        .unwrap_err();

        assert_eq!(err, Stage34nListeningProofError::ControlCommittedTranscript);
    }

    #[test]
    fn stage_34n_listening_cannot_route_to_tts_voice_id_provider_or_protected_execution() {
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[
                stage34n_positive_trial().with_forbidden_downstream_work(),
                stage34n_quiet_control(),
            ],
        )
        .unwrap_err();

        assert_eq!(err, Stage34nListeningProofError::ProviderCallAttempted);

        let mut downstream = stage34n_positive_trial();
        downstream.answer_generated = true;
        downstream.tts_requested = true;
        downstream.voice_id_invoked = true;
        downstream.tool_route_invoked = true;
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[downstream, stage34n_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34nListeningProofError::DownstreamWorkAttempted);

        let mut protected = stage34n_positive_trial();
        protected.protected_execution_requested = true;
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[protected, stage34n_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(
            err,
            Stage34nListeningProofError::ProtectedExecutionAttempted
        );
    }

    #[test]
    fn stage_34n_endpoint_confidence_and_wer_gates_fail_closed() {
        let mut low_confidence = stage34n_positive_trial();
        low_confidence.confidence_bp = 7000;
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[low_confidence, stage34n_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34nListeningProofError::ConfidenceBelowTarget);

        let mut endpoint_open = stage34n_positive_trial();
        endpoint_open.endpoint_final = false;
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[endpoint_open, stage34n_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34nListeningProofError::EndpointIncomplete);

        let mut bad_transcript = stage34n_positive_trial();
        bad_transcript.observed_transcript = Some("different words entirely".to_string());
        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ControlledReplayPack,
            10,
            &[bad_transcript, stage34n_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34nListeningProofError::WerAboveTarget);
    }

    #[test]
    fn stage_34n_listening_artifacts_are_redacted_and_raw_audio_is_not_committed() {
        let mut raw_audio = stage34n_positive_trial();
        raw_audio.raw_audio_committed = true;

        let err = verify_stage34n_controlled_listening_proof(
            Stage34nListeningProofConfig::controlled_stage34n(),
            Stage34nListeningProofSource::ForegroundMic,
            15,
            &[raw_audio, stage34n_quiet_control()],
        )
        .unwrap_err();

        assert_eq!(err, Stage34nListeningProofError::RawAudioCommitted);
    }
}
