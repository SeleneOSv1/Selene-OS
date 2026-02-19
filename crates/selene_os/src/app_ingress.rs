#![forbid(unsafe_code)]

use selene_engines::ph1_voice_id::{
    simulation_profile_embedding_from_seed, EnrolledSpeaker as EngineEnrolledSpeaker,
    VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdRequest, SpeakerId, UserId, VoiceEmbeddingCaptureRef,
};
use selene_kernel_contracts::ph1d::PolicyContextRef;
use selene_kernel_contracts::ph1e::ToolResponse;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::InterruptCandidate;
use selene_kernel_contracts::ph1link::AppPlatform;
use selene_kernel_contracts::ph1m::MemoryCandidate;
use selene_kernel_contracts::ph1n::Ph1nResponse;
use selene_kernel_contracts::ph1x::{
    ConfirmAnswer, IdentityContext, Ph1xRequest, StepUpCapabilities, ThreadState,
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
use crate::simulation_executor::SimulationExecutor;

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

#[derive(Debug, Clone, Default)]
pub struct AppServerIngressRuntime {
    executor: SimulationExecutor,
}

impl AppServerIngressRuntime {
    pub fn new(executor: SimulationExecutor) -> Self {
        Self { executor }
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
            OsVoiceLiveTurnOutcome::Forwarded(forwarded) => Some(
                build_ph1x_request_from_voice_forward(
                    correlation_id,
                    turn_id,
                    app_platform,
                    forwarded,
                    x_build,
                )?,
            ),
            OsVoiceLiveTurnOutcome::NotInvokedDisabled | OsVoiceLiveTurnOutcome::Refused(_) => None,
        };

        Ok((outcome, ph1x_request))
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
) -> Result<Ph1xRequest, StorageError> {
    let mut req = Ph1xRequest::v1(
        correlation_id.0,
        turn_id.0,
        x_build.now,
        x_build.thread_state,
        x_build.session_state,
        IdentityContext::Voice(forwarded.voice_identity_assertion.clone()),
        x_build.policy_context_ref,
        x_build.memory_candidates,
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
    use selene_kernel_contracts::ph1_voice_id::{DeviceTrustLevel, Ph1VoiceIdResponse};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1n::{Chat, Ph1nResponse};
    use selene_kernel_contracts::ph1x::{IdentityContext, ThreadState};
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus};

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
}
