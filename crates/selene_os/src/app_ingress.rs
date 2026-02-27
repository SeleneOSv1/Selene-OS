#![forbid(unsafe_code)]

use selene_engines::ph1_voice_id::{
    simulation_profile_embedding_from_seed, EnrolledSpeaker as EngineEnrolledSpeaker,
    VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_engines::ph1e::{Ph1eConfig, Ph1eRuntime};
use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdRequest, SpeakerId, UserId, VoiceEmbeddingCaptureRef,
};
use selene_kernel_contracts::ph1d::PolicyContextRef;
use selene_kernel_contracts::ph1e::ToolResponse;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::InterruptCandidate;
use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::ph1m::MemoryCandidate;
use selene_kernel_contracts::ph1n::{FieldKey, Ph1nResponse};
use selene_kernel_contracts::ph1x::{
    ConfirmAnswer, DispatchRequest, IdentityContext, Ph1xDirective, Ph1xRequest, Ph1xResponse,
    StepUpCapabilities, ThreadState,
};
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};
use selene_storage::ph1f::{Ph1fStore, StorageError};

use crate::device_artifact_sync::DeviceArtifactSyncWorkerPassMetrics;
use crate::ph1os::{
    OsTopLevelTurnInput, OsTopLevelTurnPath, OsTurnInput, OsVoiceLiveTurnInput,
    OsVoiceLiveTurnOutcome, OsVoicePlatform, OsVoiceTrigger, OsVoiceTurnContext,
};
use crate::ph1x::{Ph1xConfig, Ph1xRuntime};
use crate::simulation_executor::{SimulationDispatchOutcome, SimulationExecutor};

#[derive(Debug, Clone)]
pub struct AppVoiceIngressRequest {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub app_platform: AppPlatform,
    pub trigger: OsVoiceTrigger,
    pub voice_id_request: Ph1VoiceIdRequest,
    pub actor_user_id: UserId,
    pub tenant_id: Option<String>,
    pub device_id: Option<DeviceId>,
    pub enrolled_speakers: Vec<EngineEnrolledSpeaker>,
    pub observation: EngineVoiceIdObservation,
}

impl AppVoiceIngressRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        app_platform: AppPlatform,
        trigger: OsVoiceTrigger,
        voice_id_request: Ph1VoiceIdRequest,
        actor_user_id: UserId,
        tenant_id: Option<String>,
        device_id: Option<DeviceId>,
        enrolled_speakers: Vec<EngineEnrolledSpeaker>,
        observation: EngineVoiceIdObservation,
    ) -> Result<Self, ContractViolation> {
        app_platform.validate()?;
        voice_id_request.validate()?;
        if let Some(device_id) = device_id.as_ref() {
            device_id.validate()?;
        }
        Ok(Self {
            correlation_id,
            turn_id,
            app_platform,
            trigger,
            voice_id_request,
            actor_user_id,
            tenant_id,
            device_id,
            enrolled_speakers,
            observation,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppVoicePh1xBuildInput {
    pub now: MonotonicTimeNs,
    pub thread_state: ThreadState,
    pub session_state: SessionState,
    pub policy_context_ref: PolicyContextRef,
    pub memory_candidates: Vec<MemoryCandidate>,
    pub confirm_answer: Option<ConfirmAnswer>,
    pub nlp_output: Option<Ph1nResponse>,
    pub tool_response: Option<ToolResponse>,
    pub interruption: Option<InterruptCandidate>,
    pub locale: Option<String>,
    pub last_failure_reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppVoiceTurnNextMove {
    NotInvokedDisabled,
    Refused,
    Confirm,
    Clarify,
    Respond,
    Dispatch,
    Wait,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppVoiceTurnExecutionOutcome {
    pub voice_outcome: OsVoiceLiveTurnOutcome,
    pub next_move: AppVoiceTurnNextMove,
    pub ph1x_request: Option<Ph1xRequest>,
    pub ph1x_response: Option<Ph1xResponse>,
    pub dispatch_outcome: Option<SimulationDispatchOutcome>,
    pub response_text: Option<String>,
    pub reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone)]
pub struct AppServerIngressRuntime {
    executor: SimulationExecutor,
    ph1x_runtime: Ph1xRuntime,
    ph1e_runtime: Ph1eRuntime,
}

impl Default for AppServerIngressRuntime {
    fn default() -> Self {
        Self::new(SimulationExecutor::default())
    }
}

impl AppServerIngressRuntime {
    pub fn new(executor: SimulationExecutor) -> Self {
        Self {
            executor,
            ph1x_runtime: Ph1xRuntime::new(Ph1xConfig::mvp_v1()),
            ph1e_runtime: Ph1eRuntime::new(Ph1eConfig::mvp_v1()),
        }
    }

    pub fn run_voice_turn(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
    ) -> Result<OsVoiceLiveTurnOutcome, StorageError> {
        let resolved_enrolled_speakers =
            locked_enrolled_speakers_from_store(store, request.tenant_id.as_deref())?;
        let top_level_turn_input = OsTopLevelTurnInput::v1(
            request.correlation_id,
            request.turn_id,
            OsTopLevelTurnPath::Voice,
            Some(OsVoiceTurnContext::v1(
                os_voice_platform_from_app_platform(request.app_platform),
                request.trigger,
            )),
            expected_always_on_voice_sequence(request.trigger),
            Vec::new(),
            1,
            default_os_turn_input(request.correlation_id, request.turn_id)
                .map_err(StorageError::ContractViolation)?,
        )
        .map_err(StorageError::ContractViolation)?;

        let live_turn_input = OsVoiceLiveTurnInput::v1(
            top_level_turn_input,
            request.voice_id_request,
            request.actor_user_id,
            request.tenant_id,
            request.device_id,
            resolved_enrolled_speakers,
            request.observation,
        )
        .map_err(StorageError::ContractViolation)?;

        // Default app/server voice ingress path.
        self.executor
            .execute_os_voice_live_turn(store, live_turn_input)
    }

    pub fn run_voice_turn_and_build_ph1x_request(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<(OsVoiceLiveTurnOutcome, Option<Ph1xRequest>), StorageError> {
        let correlation_id = request.correlation_id;
        let turn_id = request.turn_id;
        let app_platform = request.app_platform.clone();
        let outcome = self.run_voice_turn(store, request)?;

        let ph1x_request = match &outcome {
            OsVoiceLiveTurnOutcome::Forwarded(forwarded) => {
                Some(self.build_ph1x_request_for_forwarded_voice(
                    store,
                    correlation_id,
                    turn_id,
                    app_platform,
                    forwarded,
                    x_build,
                )?)
            }
            OsVoiceLiveTurnOutcome::NotInvokedDisabled | OsVoiceLiveTurnOutcome::Refused(_) => None,
        };

        Ok((outcome, ph1x_request))
    }

    pub fn run_desktop_voice_turn_end_to_end(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        if request.app_platform != AppPlatform::Desktop {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_voice_ingress_request.app_platform",
                    reason: "run_desktop_voice_turn_end_to_end requires AppPlatform::Desktop",
                },
            ));
        }

        let actor_user_id = request.actor_user_id.clone();
        let dispatch_now = x_build.now;
        let (voice_outcome, ph1x_request) =
            self.run_voice_turn_and_build_ph1x_request(store, request, x_build)?;
        let Some(ph1x_request) = ph1x_request else {
            return Ok(app_voice_turn_execution_outcome_from_voice_only(
                voice_outcome,
            ));
        };

        let ph1x_response = self
            .ph1x_runtime
            .decide(&ph1x_request)
            .map_err(StorageError::ContractViolation)?;

        let mut out = AppVoiceTurnExecutionOutcome {
            voice_outcome,
            next_move: AppVoiceTurnNextMove::Wait,
            ph1x_request: Some(ph1x_request),
            ph1x_response: Some(ph1x_response.clone()),
            dispatch_outcome: None,
            response_text: None,
            reason_code: Some(ph1x_response.reason_code),
        };

        match &ph1x_response.directive {
            Ph1xDirective::Confirm(confirm) => {
                out.next_move = AppVoiceTurnNextMove::Confirm;
                out.response_text = Some(confirm.text.clone());
            }
            Ph1xDirective::Clarify(clarify) => {
                out.next_move = AppVoiceTurnNextMove::Clarify;
                out.response_text = Some(clarify.question.clone());
            }
            Ph1xDirective::Respond(respond) => {
                out.next_move = AppVoiceTurnNextMove::Respond;
                out.response_text = Some(respond.response_text.clone());
            }
            Ph1xDirective::Wait(wait) => {
                out.next_move = AppVoiceTurnNextMove::Wait;
                out.response_text = wait.reason.clone();
            }
            Ph1xDirective::Dispatch(dispatch) => match &dispatch.dispatch_request {
                DispatchRequest::Tool(tool_request) => {
                    let tool_response = self.ph1e_runtime.run(tool_request);
                    let tool_followup_request = build_tool_followup_ph1x_request(
                        out.ph1x_request
                            .as_ref()
                            .ok_or(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "app_voice_turn_execution_outcome.ph1x_request",
                                    reason: "must be present before tool follow-up",
                                },
                            ))?,
                        &ph1x_response,
                        tool_response,
                    )?;
                    let tool_followup_response = self
                        .ph1x_runtime
                        .decide(&tool_followup_request)
                        .map_err(StorageError::ContractViolation)?;

                    out.ph1x_request = Some(tool_followup_request);
                    out.ph1x_response = Some(tool_followup_response.clone());
                    out.reason_code = Some(tool_followup_response.reason_code);
                    match tool_followup_response.directive {
                        Ph1xDirective::Confirm(confirm) => {
                            out.next_move = AppVoiceTurnNextMove::Confirm;
                            out.response_text = Some(confirm.text);
                        }
                        Ph1xDirective::Clarify(clarify) => {
                            out.next_move = AppVoiceTurnNextMove::Clarify;
                            out.response_text = Some(clarify.question);
                        }
                        Ph1xDirective::Respond(respond) => {
                            out.next_move = AppVoiceTurnNextMove::Respond;
                            out.response_text = Some(respond.response_text);
                        }
                        Ph1xDirective::Wait(wait) => {
                            out.next_move = AppVoiceTurnNextMove::Wait;
                            out.response_text = wait.reason;
                        }
                        Ph1xDirective::Dispatch(_) => {
                            return Err(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "ph1x_response.directive.dispatch_request",
                                    reason: "tool follow-up must complete without another dispatch",
                                },
                            ));
                        }
                    }
                }
                DispatchRequest::SimulationCandidate(_) | DispatchRequest::AccessStepUp(_) => {
                    let dispatch_outcome =
                        self.executor.execute_ph1x_dispatch_simulation_candidate(
                            store,
                            actor_user_id,
                            dispatch_now,
                            &ph1x_response,
                        )?;
                    out.next_move = AppVoiceTurnNextMove::Dispatch;
                    out.response_text = Some(response_text_for_dispatch_outcome(&dispatch_outcome));
                    out.dispatch_outcome = Some(dispatch_outcome);
                }
            },
        }

        Ok(out)
    }

    pub fn run_device_artifact_sync_worker_pass(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<(), StorageError> {
        let _ = self.run_device_artifact_sync_worker_pass_with_metrics(
            store,
            now,
            correlation_id,
            turn_id,
        )?;
        Ok(())
    }

    pub fn run_device_artifact_sync_worker_pass_with_metrics(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
        self.executor
            .execute_device_artifact_sync_worker_pass_with_metrics(
                store,
                now,
                correlation_id,
                turn_id,
            )
    }

    fn build_ph1x_request_for_forwarded_voice(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        app_platform: AppPlatform,
        forwarded: &crate::ph1os::OsVoiceLiveForwardBundle,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<Ph1xRequest, StorageError> {
        let topic_hint = memory_topic_hint_from_nlp_output(x_build.nlp_output.as_ref());
        let runtime_memory_candidates = if forwarded.identity_confirmed() {
            self.executor
                .collect_context_memory_candidates_for_voice_turn(
                    store,
                    x_build.now,
                    correlation_id,
                    turn_id,
                    &forwarded.voice_identity_assertion,
                    x_build.policy_context_ref,
                    topic_hint,
                )
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        build_ph1x_request_from_voice_forward(
            correlation_id,
            turn_id,
            app_platform,
            forwarded,
            x_build,
            runtime_memory_candidates,
        )
    }
}

fn app_voice_turn_execution_outcome_from_voice_only(
    voice_outcome: OsVoiceLiveTurnOutcome,
) -> AppVoiceTurnExecutionOutcome {
    match voice_outcome {
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => AppVoiceTurnExecutionOutcome {
            voice_outcome: OsVoiceLiveTurnOutcome::NotInvokedDisabled,
            next_move: AppVoiceTurnNextMove::NotInvokedDisabled,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            response_text: None,
            reason_code: None,
        },
        OsVoiceLiveTurnOutcome::Refused(refuse) => AppVoiceTurnExecutionOutcome {
            voice_outcome: OsVoiceLiveTurnOutcome::Refused(refuse.clone()),
            next_move: AppVoiceTurnNextMove::Refused,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            response_text: Some(refuse.message.clone()),
            reason_code: Some(refuse.reason_code),
        },
        OsVoiceLiveTurnOutcome::Forwarded(forwarded) => AppVoiceTurnExecutionOutcome {
            voice_outcome: OsVoiceLiveTurnOutcome::Forwarded(forwarded),
            next_move: AppVoiceTurnNextMove::Wait,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            response_text: None,
            reason_code: None,
        },
    }
}

fn response_text_for_dispatch_outcome(outcome: &SimulationDispatchOutcome) -> String {
    match outcome {
        SimulationDispatchOutcome::LinkDelivered { .. } => "I sent the link.".to_string(),
        SimulationDispatchOutcome::Link(_) => "I generated the link.".to_string(),
        SimulationDispatchOutcome::Reminder(_) => "I scheduled that reminder.".to_string(),
        SimulationDispatchOutcome::AccessStepUp { outcome, .. } => match outcome {
            selene_kernel_contracts::ph1x::StepUpOutcome::Continue => {
                "Access verification passed.".to_string()
            }
            selene_kernel_contracts::ph1x::StepUpOutcome::Refuse => {
                "I can't proceed with that request.".to_string()
            }
            selene_kernel_contracts::ph1x::StepUpOutcome::Defer => {
                "I need step-up verification before I continue.".to_string()
            }
        },
        _ => "Done.".to_string(),
    }
}

fn os_voice_platform_from_app_platform(app_platform: AppPlatform) -> OsVoicePlatform {
    match app_platform {
        AppPlatform::Ios => OsVoicePlatform::Ios,
        AppPlatform::Android => OsVoicePlatform::Android,
        AppPlatform::Desktop => OsVoicePlatform::Desktop,
    }
}

fn expected_always_on_voice_sequence(trigger: OsVoiceTrigger) -> Vec<String> {
    let mut seq = vec!["PH1.K".to_string()];
    if trigger.wake_stage_required() {
        seq.push("PH1.W".to_string());
    }
    seq.extend([
        "PH1.VOICE.ID".to_string(),
        "PH1.C".to_string(),
        "PH1.SRL".to_string(),
        "PH1.NLP".to_string(),
        "PH1.CONTEXT".to_string(),
        "PH1.POLICY".to_string(),
        "PH1.X".to_string(),
    ]);
    seq
}

fn default_os_turn_input(
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> Result<OsTurnInput, ContractViolation> {
    OsTurnInput::v1(
        correlation_id,
        turn_id,
        true,
        true,
        true,
        false,
        false,
        true,
        false,
        false,
        true,
        true,
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
    )
}

fn locked_enrolled_speakers_from_store(
    store: &Ph1fStore,
    tenant_scope: Option<&str>,
) -> Result<Vec<EngineEnrolledSpeaker>, StorageError> {
    let mut enrolled = Vec::new();
    for profile in store.ph1vid_voice_profile_rows() {
        let Some(device) = store.get_device(&profile.device_id) else {
            continue;
        };
        if let Some(tenant_id) = tenant_scope {
            let Some((profile_tenant, _)) = device.user_id.as_str().split_once(':') else {
                continue;
            };
            if profile_tenant != tenant_id {
                continue;
            }
        }
        let fingerprint = stable_seed_u64(&[
            profile.voice_profile_id.as_str(),
            profile.device_id.as_str(),
            device.user_id.as_str(),
        ]);
        let speaker_id = SpeakerId::new(format!(
            "spk_{}",
            short_hash_hex(&[profile.voice_profile_id.as_str(), device.user_id.as_str()])
        ))
        .map_err(StorageError::ContractViolation)?;
        let profile_embedding = profile
            .profile_embedding_capture_ref
            .as_ref()
            .map(profile_embedding_from_capture_ref)
            .or_else(|| Some(simulation_profile_embedding_from_seed(fingerprint)));
        enrolled.push(EngineEnrolledSpeaker {
            speaker_id,
            user_id: Some(device.user_id.clone()),
            fingerprint,
            profile_embedding,
        });
    }
    Ok(enrolled)
}

fn profile_embedding_from_capture_ref(capture_ref: &VoiceEmbeddingCaptureRef) -> [i16; 16] {
    let seed = stable_seed_u64(&[
        capture_ref.embedding_ref.as_str(),
        capture_ref.embedding_model_id.as_str(),
        &capture_ref.embedding_dim.to_string(),
    ]);
    simulation_profile_embedding_from_seed(seed)
}

fn stable_seed_u64(parts: &[&str]) -> u64 {
    let mut hex = short_hash_hex(parts);
    if hex.len() < 16 {
        hex.push_str("0000000000000000");
    }
    u64::from_str_radix(&hex[..16], 16).unwrap_or(0x9E37_79B9_7F4A_7C15)
}

fn short_hash_hex(parts: &[&str]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for part in parts {
        for &b in part.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(PRIME);
        }
        h ^= b'|' as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}

fn build_ph1x_request_from_voice_forward(
    correlation_id: CorrelationId,
    turn_id: TurnId,
    app_platform: AppPlatform,
    forwarded: &crate::ph1os::OsVoiceLiveForwardBundle,
    x_build: AppVoicePh1xBuildInput,
    runtime_memory_candidates: Vec<MemoryCandidate>,
) -> Result<Ph1xRequest, StorageError> {
    let mut req = Ph1xRequest::v1(
        correlation_id.0,
        turn_id.0,
        x_build.now,
        x_build.thread_state,
        x_build.session_state,
        IdentityContext::Voice(forwarded.voice_identity_assertion.clone()),
        x_build.policy_context_ref,
        runtime_memory_candidates,
        x_build.confirm_answer,
        x_build.nlp_output,
        x_build.tool_response,
        x_build.interruption,
        x_build.locale,
        x_build.last_failure_reason_code,
    )
    .map_err(StorageError::ContractViolation)?;
    let step_up_capabilities = match app_platform {
        AppPlatform::Ios | AppPlatform::Android => StepUpCapabilities::v1(true, true),
        AppPlatform::Desktop => StepUpCapabilities::v1(false, true),
    };
    req = req
        .with_step_up_capabilities(Some(step_up_capabilities))
        .map_err(StorageError::ContractViolation)?;
    req = req
        .with_identity_prompt_scope_key(forwarded.identity_prompt_scope_key.clone())
        .map_err(StorageError::ContractViolation)?;
    Ok(req)
}

fn build_tool_followup_ph1x_request(
    base_request: &Ph1xRequest,
    dispatch_response: &Ph1xResponse,
    tool_response: ToolResponse,
) -> Result<Ph1xRequest, StorageError> {
    let mut req = Ph1xRequest::v1(
        base_request.correlation_id,
        base_request.turn_id,
        base_request.now,
        dispatch_response.thread_state.clone(),
        base_request.session_state,
        base_request.identity_context.clone(),
        base_request.policy_context_ref,
        base_request.memory_candidates.clone(),
        None,
        None,
        Some(tool_response),
        None,
        base_request.locale.clone(),
        None,
    )
    .map_err(StorageError::ContractViolation)?;
    req = req
        .with_step_up_capabilities(base_request.step_up_capabilities)
        .map_err(StorageError::ContractViolation)?;
    req = req
        .with_identity_prompt_scope_key(base_request.identity_prompt_scope_key.clone())
        .map_err(StorageError::ContractViolation)?;
    Ok(req)
}

fn memory_topic_hint_from_nlp_output(nlp_output: Option<&Ph1nResponse>) -> Option<String> {
    let Ph1nResponse::IntentDraft(draft) = nlp_output? else {
        return None;
    };
    [FieldKey::Recipient, FieldKey::Person, FieldKey::Task]
        .iter()
        .find_map(|key| {
            draft
                .fields
                .iter()
                .find(|field| field.key == *key)
                .and_then(|field| {
                    field
                        .value
                        .normalized_value
                        .as_ref()
                        .or(Some(&field.value.original_span))
                })
        })
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
    use selene_kernel_contracts::ph1_voice_id::{
        DeviceTrustLevel, DiarizationSegment, Ph1VoiceIdResponse, SpeakerAssertionOk, SpeakerLabel,
    };
    use selene_kernel_contracts::ph1bcast::{BCAST_CREATE_DRAFT, BCAST_DELIVER_COMMIT};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1delivery::DELIVERY_SEND_COMMIT;
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1link::LINK_INVITE_GENERATE_DRAFT;
    use selene_kernel_contracts::ph1m::{
        MemoryCandidate, MemoryConfidence, MemoryKey, MemoryProvenance, MemorySensitivityFlag,
        MemoryUsePolicy, MemoryValue,
    };
    use selene_kernel_contracts::ph1n::{
        Chat, EvidenceSpan, FieldKey, FieldValue, IntentDraft, IntentField, IntentType,
        OverallConfidence, Ph1nResponse, SensitivityLevel, TranscriptHash,
    };
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1simcat::{
        SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType,
        SimulationVersion,
    };
    use selene_kernel_contracts::ph1x::{
        ConfirmAnswer, IdentityContext, PendingState, Ph1xDirective, ThreadState,
    };
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState};
    use selene_storage::ph1f::{
        AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessVerificationLevel,
        DeviceRecord, IdentityRecord, IdentityStatus,
    };

    fn sample_voice_id_request(now: MonotonicTimeNs, owner_user_id: UserId) -> Ph1VoiceIdRequest {
        let stream_id = AudioStreamId(1);
        let processed_stream_ref = AudioStreamRef::v1(
            stream_id,
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
        );
        let vad_events = vec![VadEvent::v1(
            stream_id,
            MonotonicTimeNs(now.0.saturating_sub(2_000_000)),
            now,
            Confidence::new(0.95).unwrap(),
            SpeechLikeness::new(0.95).unwrap(),
        )];
        let session_snapshot = SessionSnapshot {
            schema_version: SchemaVersion(1),
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        };
        let device_id = AudioDeviceId::new("ingress_mic_device_1").unwrap();
        Ph1VoiceIdRequest::v1(
            now,
            processed_stream_ref,
            vad_events,
            device_id,
            session_snapshot,
            None,
            false,
            DeviceTrustLevel::Trusted,
            Some(owner_user_id),
        )
        .unwrap()
    }

    fn seed_actor(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) {
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn no_observation() -> EngineVoiceIdObservation {
        EngineVoiceIdObservation {
            primary_fingerprint: None,
            secondary_fingerprint: None,
            primary_embedding: None,
            secondary_embedding: None,
            spoof_risk: false,
        }
    }

    fn confirmed_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1(
                SpeakerId::new("spk_ingress_confirmed").unwrap(),
                Some(user_id),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                SpeakerLabel::speaker_a(),
            )
            .unwrap(),
        )
    }

    fn invite_link_draft_missing_contact(recipient: &str, tenant_id: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::CreateInviteLink,
                SchemaVersion(1),
                vec![
                    IntentField {
                        key: FieldKey::InviteeType,
                        value: FieldValue::normalized(
                            "associate".to_string(),
                            "associate".to_string(),
                        )
                        .unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::DeliveryMethod,
                        value: FieldValue::normalized("send".to_string(), "sms".to_string())
                            .unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::Recipient,
                        value: FieldValue::verbatim(recipient.to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::TenantId,
                        value: FieldValue::verbatim(tenant_id.to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                ],
                vec![FieldKey::RecipientContact],
                OverallConfidence::High,
                vec![],
                ReasonCodeId(1),
                SensitivityLevel::Private,
                true,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn invite_link_send_draft(
        recipient: &str,
        recipient_contact: &str,
        tenant_id: &str,
    ) -> IntentDraft {
        IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::normalized("associate".to_string(), "associate".to_string())
                        .unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::DeliveryMethod,
                    value: FieldValue::normalized("send".to_string(), "sms".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::Recipient,
                    value: FieldValue::verbatim(recipient.to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::RecipientContact,
                    value: FieldValue::verbatim(recipient_contact.to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim(tenant_id.to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
            ],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn web_search_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::WebSearchQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: query.len() as u32,
                    verbatim_excerpt: query.to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn news_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::NewsQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: query.len() as u32,
                    verbatim_excerpt: query.to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn url_fetch_and_cite_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::UrlFetchAndCiteQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: query.len() as u32,
                    verbatim_excerpt: query.to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn document_understand_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::DocumentUnderstandQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: query.len() as u32,
                    verbatim_excerpt: query.to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn photo_understand_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::PhotoUnderstandQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: query.len() as u32,
                    verbatim_excerpt: query.to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn seed_link_send_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                "role.link_sender".to_string(),
                AccessMode::A,
                "{\"allow\":[\"LINK_INVITE\",\"DELIVERY_SEND\"]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Active,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_simulation_catalog_status(
        store: &mut Ph1fStore,
        tenant: &str,
        simulation_id: &str,
        simulation_type: SimulationType,
        status: SimulationStatus,
    ) {
        let event = SimulationCatalogEventInput::v1(
            MonotonicTimeNs(1),
            TenantId::new(tenant.to_string()).unwrap(),
            SimulationId::new(simulation_id.to_string()).unwrap(),
            SimulationVersion(1),
            simulation_type,
            status,
            "PH1.X".to_string(),
            "reads_v1".to_string(),
            "writes_v1".to_string(),
            ReasonCodeId(1),
            None,
        )
        .unwrap();
        store.append_simulation_catalog_event(event).unwrap();
    }

    fn external_injected_memory_candidate() -> MemoryCandidate {
        MemoryCandidate::v1(
            MemoryKey::new("invite_contact_tom_sms").unwrap(),
            MemoryValue::v1("+14155550100".to_string(), None).unwrap(),
            MemoryConfidence::High,
            MonotonicTimeNs(1),
            "External candidate".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
            MemorySensitivityFlag::Low,
            MemoryUsePolicy::AlwaysUsable,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_ingress_01_ios_explicit_routes_through_os_live_default() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_ios_user").unwrap();
        let device_id = DeviceId::new("ingress_ios_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9101),
            TurnId(9201),
            AppPlatform::Ios,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(forwarded.top_level_bundle.path, OsTopLevelTurnPath::Voice);
        assert!(!forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.W".to_string()));
        assert!(matches!(
            forwarded.voice_identity_assertion,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));
    }

    #[test]
    fn at_ingress_02_android_wake_routes_with_wake_stage() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_android_user").unwrap();
        let device_id = DeviceId::new("ingress_android_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9102),
            TurnId(9202),
            AppPlatform::Android,
            OsVoiceTrigger::WakeWord,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert!(forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.W".to_string()));
    }

    #[test]
    fn at_ingress_03_desktop_explicit_routes_through_os_live_default() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_desktop_user").unwrap();
        let device_id = DeviceId::new("ingress_desktop_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9103),
            TurnId(9203),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(forwarded.top_level_bundle.path, OsTopLevelTurnPath::Voice);
        assert!(!forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.W".to_string()));
    }

    #[test]
    fn at_ingress_04_voice_forward_builds_ph1x_request_with_scope_key() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_bridge_user").unwrap();
        let device_id = DeviceId::new("ingress_bridge_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9104),
            TurnId(9204),
            AppPlatform::Ios,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let (outcome, ph1x_request) = runtime
            .run_voice_turn_and_build_ph1x_request(&mut store, request, x_build)
            .unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        let ph1x_request = ph1x_request.expect("forwarded voice turn must build ph1x request");
        assert_eq!(ph1x_request.correlation_id, 9104);
        assert_eq!(ph1x_request.turn_id, 9204);
        assert_eq!(
            ph1x_request.identity_prompt_scope_key,
            forwarded.identity_prompt_scope_key
        );
        assert!(matches!(
            ph1x_request.identity_context,
            IdentityContext::Voice(_)
        ));
    }

    #[test]
    fn run5_voice_builder_skips_ph1m_and_ignores_external_memory_when_identity_not_confirmed() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run5_unknown_user").unwrap();
        let device_id = DeviceId::new("run5_unknown_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9501),
            TurnId(9601),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![external_injected_memory_candidate()],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Tom", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let (_outcome, ph1x_request) = runtime
            .run_voice_turn_and_build_ph1x_request(&mut store, request, x_build)
            .unwrap();
        let ph1x_request = ph1x_request.expect("voice turn should build ph1x request");
        assert!(ph1x_request.memory_candidates.is_empty());
        assert_eq!(runtime.executor.debug_memory_context_lookup_count(), 0);
    }

    #[test]
    fn run5_voice_builder_uses_confirmed_identity_memory_context_to_resolve_tom_contact() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run5_confirmed_user").unwrap();
        let device_id = DeviceId::new("run5_confirmed_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9502),
            TurnId(9602),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(mut forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };
        let confirmed_assertion = confirmed_voice_assertion(actor_user_id);
        forwarded.voice_identity_assertion = confirmed_assertion.clone();

        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9502),
                TurnId(9602),
                confirmed_assertion,
                MemoryKey::new("invite_contact_tom_sms").unwrap(),
                MemoryValue::v1("+14155550100".to_string(), None).unwrap(),
                "Tom contact memory".to_string(),
            )
            .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Tom", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let ph1x_request = runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                CorrelationId(9502),
                TurnId(9602),
                AppPlatform::Desktop,
                &forwarded,
                x_build,
            )
            .unwrap();
        assert!(ph1x_request
            .memory_candidates
            .iter()
            .any(|candidate| candidate.memory_key.as_str() == "invite_contact_tom_sms"));
        assert_eq!(runtime.executor.debug_memory_context_lookup_count(), 1);

        let out = runtime.ph1x_runtime.decide(&ph1x_request).unwrap();
        assert!(!matches!(
            out.directive,
            selene_kernel_contracts::ph1x::Ph1xDirective::Clarify(_)
        ));
    }

    #[test]
    fn run6_desktop_voice_turn_end_to_end_returns_clarify_for_missing_recipient_contact() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run6_clarify_user").unwrap();
        let device_id = DeviceId::new("run6_clarify_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9601),
            TurnId(9701),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(8),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Tom", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Clarify);
        assert!(out.dispatch_outcome.is_none());
        assert!(out
            .response_text
            .as_deref()
            .unwrap_or_default()
            .contains("Tom"));
        match out
            .ph1x_response
            .expect("clarify outcome must include PH1.X response")
            .directive
        {
            Ph1xDirective::Clarify(clarify) => {
                assert_eq!(clarify.what_is_missing, vec![FieldKey::RecipientContact]);
            }
            _ => panic!("expected PH1.X Clarify directive"),
        }
    }

    #[test]
    fn run6_desktop_voice_turn_end_to_end_dispatches_send_link_with_guarded_executor() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run6_dispatch_user").unwrap();
        let device_id = DeviceId::new("run6_dispatch_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        seed_link_send_access_instance(&mut store, &actor_user_id, "tenant_1");
        for (simulation_id, simulation_type) in [
            (LINK_INVITE_GENERATE_DRAFT, SimulationType::Draft),
            (BCAST_CREATE_DRAFT, SimulationType::Draft),
            (BCAST_DELIVER_COMMIT, SimulationType::Commit),
            (DELIVERY_SEND_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9602),
            TurnId(9702),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(9),
            thread_state: ThreadState::v1(
                Some(PendingState::Confirm {
                    intent_draft: invite_link_send_draft("Tom", "+14155550100", "tenant_1"),
                    attempts: 1,
                }),
                None,
            ),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Dispatch);
        assert_eq!(out.response_text.as_deref(), Some("I sent the link."));
        match out
            .dispatch_outcome
            .expect("dispatch outcome is required for dispatch next move")
        {
            SimulationDispatchOutcome::LinkDelivered {
                delivery_emitted, ..
            } => {
                assert!(delivery_emitted);
            }
            _ => panic!("expected LinkDelivered dispatch outcome"),
        }
        assert!(matches!(
            out.ph1x_response
                .expect("dispatch outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Dispatch(_)
        ));
    }

    #[test]
    fn run_a_desktop_voice_turn_end_to_end_dispatches_web_search_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runa_websearch_user").unwrap();
        let device_id = DeviceId::new("runa_websearch_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9603),
            TurnId(9703),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(10),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(web_search_draft("search the web for selene tool parity")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("https://example.com/search-result"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
        assert!(matches!(
            out.ph1x_response
                .expect("respond outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Respond(_)
        ));
    }

    #[test]
    fn run_b_desktop_voice_turn_end_to_end_dispatches_news_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runb_news_user").unwrap();
        let device_id = DeviceId::new("runb_news_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9604),
            TurnId(9704),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(11),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(news_draft("what's the latest news about selene tools")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("https://example.com/news"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
        assert!(matches!(
            out.ph1x_response
                .expect("respond outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Respond(_)
        ));
    }

    #[test]
    fn run_c_desktop_voice_turn_end_to_end_dispatches_url_fetch_and_cite_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runc_urlfetch_user").unwrap();
        let device_id = DeviceId::new("runc_urlfetch_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9605),
            TurnId(9705),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(12),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(url_fetch_and_cite_draft(
                "open this URL and cite it: https://example.com/spec",
            )),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("Citations:"));
        assert!(response_text.contains("https://example.com"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
        assert!(matches!(
            out.ph1x_response
                .expect("respond outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Respond(_)
        ));
    }

    #[test]
    fn run_d_desktop_voice_turn_end_to_end_dispatches_document_understand_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:rund_doc_user").unwrap();
        let device_id = DeviceId::new("rund_doc_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9606),
            TurnId(9706),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(13),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(document_understand_draft(
                "read this PDF and summarize it",
            )),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("Summary:"));
        assert!(response_text.contains("Extracted fields:"));
        assert!(response_text.contains("Citations:"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
        assert!(matches!(
            out.ph1x_response
                .expect("respond outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Respond(_)
        ));
    }

    #[test]
    fn run_e_desktop_voice_turn_end_to_end_dispatches_photo_understand_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:rune_photo_user").unwrap();
        let device_id = DeviceId::new("rune_photo_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9607),
            TurnId(9707),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(14),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(photo_understand_draft("what does this screenshot say")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("Summary:"));
        assert!(response_text.contains("Extracted fields:"));
        assert!(response_text.contains("Citations:"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
        assert!(matches!(
            out.ph1x_response
                .expect("respond outcome must include PH1.X response")
                .directive,
            Ph1xDirective::Respond(_)
        ));
    }
}
