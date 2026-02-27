#![forbid(unsafe_code)]

use std::cell::RefCell;

use selene_engines::ph1m::{Ph1mConfig, Ph1mRuntime};
use selene_kernel_contracts::ph1_voice_id::{
    DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, Ph1VoiceIdSimRequest,
    Ph1VoiceIdSimResponse, VoiceEmbeddingCaptureRef, VoiceIdSimulationRequest,
};
use selene_kernel_contracts::ph1_voice_id::{SpeakerId, UserId};
use selene_kernel_contracts::ph1access::{
    ACCESS_AP_SCHEMA_ACTIVATE_COMMIT, ACCESS_AP_SCHEMA_CREATE_DRAFT,
    ACCESS_AP_SCHEMA_RETIRE_COMMIT, ACCESS_AP_SCHEMA_UPDATE_COMMIT, ACCESS_INSTANCE_COMPILE_COMMIT,
};
use selene_kernel_contracts::ph1bcast::{
    BcastOutcome, BcastRecipientState, BcastRequest, BcastSimulationType, Ph1BcastRequest,
    Ph1BcastResponse, BCAST_REMINDER_FIRED_COMMIT,
};
use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestStatus, CapreqId, Ph1CapreqRequest,
    Ph1CapreqResponse, CAPREQ_APPROVE_COMMIT, CAPREQ_CANCEL_REVOKE, CAPREQ_CREATE_DRAFT,
    CAPREQ_FULFILL_COMMIT, CAPREQ_REJECT_COMMIT, CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT,
};
use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
use selene_kernel_contracts::ph1delivery::{
    DeliveryChannel, Ph1DeliveryRequest, Ph1DeliveryResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::{
    AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
    Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
};
use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
use selene_kernel_contracts::ph1link::{
    AppPlatform, Ph1LinkRequest, Ph1LinkResponse, LINK_INVITE_GENERATE_DRAFT,
};
use selene_kernel_contracts::ph1m::{
    MemoryConfidence, MemoryConsent, MemoryKey, MemoryLayer, MemoryProposedItem, MemoryProvenance,
    MemoryRetentionMode, MemorySensitivityFlag, MemoryValue, Ph1mForgetRequest, Ph1mForgetResponse,
    Ph1mProposeRequest, Ph1mProposeResponse, Ph1mRecallRequest, Ph1mRecallResponse,
    Ph1mThreadDigestUpsertRequest,
};
use selene_kernel_contracts::ph1n::{FieldKey, FieldValue, IntentDraft, IntentType};
use selene_kernel_contracts::ph1onb::{OnboardingSessionId, Ph1OnbRequest, Ph1OnbResponse};
use selene_kernel_contracts::ph1os::{Ph1OsRequest, Ph1OsResponse};
use selene_kernel_contracts::ph1policy::{
    Ph1PolicyRequest, Ph1PolicyResponse, PolicyPromptDecision, PolicyPromptDedupeDecideRequest,
    PolicyRequestEnvelope, PolicyRulesetGetActiveRequest,
};
use selene_kernel_contracts::ph1position::{Ph1PositionRequest, Ph1PositionResponse, TenantId};
use selene_kernel_contracts::ph1rem::{
    Ph1RemRequest, Ph1RemResponse, ReminderChannel, ReminderLocalTimeMode, ReminderPriorityLevel,
    ReminderType, REMINDER_SCHEDULE_COMMIT,
};
use selene_kernel_contracts::ph1simcat::{SimulationId, SimulationStatus};
use selene_kernel_contracts::ph1w::{Ph1wRequest, Ph1wResponse, WakeRequest};
use selene_kernel_contracts::ph1x::{
    AccessStepUpDispatch, DispatchRequest, Ph1xDirective, Ph1xResponse, StepUpChallengeMethod,
    StepUpOutcome, StepUpResult,
};
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};
use selene_storage::ph1f::{
    AccessDecision, AccessMode, AccessVerificationLevel, Ph1fStore, StorageError,
};

use crate::device_artifact_sync::{
    self, DeviceArtifactSyncSenderRuntime, DeviceArtifactSyncWorkerPassMetrics,
};
use crate::ph1_voice_id::{
    Ph1VoiceIdLiveConfig, Ph1VoiceIdLiveRuntime, Ph1VoiceIdRuntime, VoiceIdentityChannel,
};
use crate::ph1capreq::Ph1CapreqRuntime;
use crate::ph1link::{Ph1LinkConfig, Ph1LinkRuntime};
use crate::ph1m::{
    MemoryOperation, MemoryTurnInput, MemoryTurnOutput, MemoryWiringOutcome, Ph1mWiring,
    Ph1mWiringConfig,
};
use crate::ph1onb::{
    OnbPositionLiveRequest, OnbPositionLiveResult, OnbVoiceEnrollLiveRequest,
    OnbVoiceEnrollLiveResult, Ph1OnbOrchRuntime,
};
use crate::ph1os::{
    OsTopLevelTurnInput, OsTopLevelTurnPath, OsTopLevelWiringOutcome, OsTurnInput,
    OsVoiceLiveTurnInput, OsVoiceLiveTurnOutcome, OsVoicePlatform, OsVoiceTrigger,
    OsVoiceTurnContext, Ph1OsEngine, Ph1OsTopLevelConfig, Ph1OsTopLevelWiring,
    Ph1OsVoiceLiveRuntime, Ph1OsWiring, Ph1OsWiringConfig,
};
use crate::ph1position::Ph1PositionRuntime;
use crate::ph1rem::Ph1RemRuntime;
use crate::ph1w::Ph1wRuntime;
use selene_engines::ph1_voice_id::{
    simulation_profile_embedding_from_seed, EnrolledSpeaker as EngineEnrolledSpeaker,
    VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_engines::ph1bcast::Ph1BcastRuntime;
use selene_engines::ph1delivery::Ph1DeliveryRuntime;
use selene_engines::ph1os::{Ph1OsConfig as EngineOsConfig, Ph1OsRuntime as EngineOsRuntime};
use selene_engines::ph1policy::{Ph1PolicyConfig, Ph1PolicyRuntime};

/// Minimal Simulation Executor (v1).
///
/// Hard rule (constitution): No Simulation -> No Execution.
///
/// In this repo's current slice, the executor supports PH1.LINK, PH1.REM, PH1.ONB,
/// PH1.POSITION, PH1.W, PH1.VOICE.ID enrollment simulations, and the
/// PH1.BCAST.MHP <-> PH1.REM handoff bridge.
/// Other simulations are added incrementally and must be registered in docs/08_SIMULATION_CATALOG.md.
#[derive(Debug, Clone)]
pub struct SimulationExecutor {
    bcast: Ph1BcastRuntime,
    delivery: Ph1DeliveryRuntime,
    link: Ph1LinkRuntime,
    memory: RefCell<Ph1mWiring<Ph1mRuntime>>,
    onb: Ph1OnbOrchRuntime,
    position: Ph1PositionRuntime,
    rem: Ph1RemRuntime,
    policy: Ph1PolicyRuntime,
    capreq: Ph1CapreqRuntime,
    voice_id_live: Ph1VoiceIdLiveRuntime,
    voice_id: Ph1VoiceIdRuntime,
    wake: Ph1wRuntime,
    os_top_level: Ph1OsTopLevelWiring<EngineBackedOsRuntime>,
    device_sync_sender: DeviceArtifactSyncSenderRuntime,
}

#[derive(Debug, Clone)]
struct EngineBackedOsRuntime {
    runtime: EngineOsRuntime,
}

impl Ph1OsEngine for EngineBackedOsRuntime {
    fn run(&self, req: &Ph1OsRequest) -> Ph1OsResponse {
        self.runtime.run(req)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimulationDispatchOutcome {
    BroadcastDeliverySend {
        bcast: Ph1BcastResponse,
        delivery: Ph1DeliveryResponse,
    },
    BroadcastMhpHandoff {
        bcast: Ph1BcastResponse,
        reminder: Ph1RemResponse,
    },
    BroadcastMhpReminderFired(Ph1BcastResponse),
    BroadcastMhpFollowupDecision {
        bcast: Ph1BcastResponse,
        followup_source: String,
        policy_gate_ok: bool,
        followup_subject_ref: String,
        followup_recipient_user_id: String,
        followup_active_speaker_user_id: String,
        followup_delivery_mode: BcastFollowupDeliveryMode,
        followup_text_only_reason: Option<BcastFollowupTextOnlyReason>,
        followup_voice_ref: Option<String>,
        followup_text_ref: Option<String>,
        policy_prompt_dedupe_key: Option<String>,
    },
    BroadcastMhpAppThreadReplyConcluded {
        bcast: Ph1BcastResponse,
        wife_forward_ref: String,
        voice_interruption_suppressed: bool,
    },
    MemoryPropose(Ph1mProposeResponse),
    MemoryRecall(Ph1mRecallResponse),
    MemoryForget(Ph1mForgetResponse),
    Link(Ph1LinkResponse),
    Reminder(Ph1RemResponse),
    Onboarding(Ph1OnbResponse),
    Position(Ph1PositionResponse),
    VoiceId(Ph1VoiceIdSimResponse),
    Wake(Ph1wResponse),
    AccessGatePassed {
        requested_action: String,
    },
    AccessStepUp {
        requested_action: String,
        challenge_method: StepUpChallengeMethod,
        outcome: StepUpOutcome,
        reason_code: ReasonCodeId,
    },
    CapreqLifecycle {
        capreq_id: CapreqId,
        capreq_event_id: u64,
        action: CapabilityRequestAction,
        status: CapabilityRequestStatus,
    },
}

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    pub const SIM_DISPATCH_GUARD_ACCESS_REQUIRED: ReasonCodeId = ReasonCodeId(0x5349_0001);
    pub const SIM_DISPATCH_GUARD_SIMULATION_ID_INVALID: ReasonCodeId = ReasonCodeId(0x5349_0002);
    pub const SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED: ReasonCodeId =
        ReasonCodeId(0x5349_0003);
    pub const SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE: ReasonCodeId = ReasonCodeId(0x5349_0004);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcastFollowupDeliveryMode {
    Voice,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcastFollowupTextOnlyReason {
    UserRequestedText,
    CannotSpeak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcastFollowupDeliveryHint {
    VoiceDefault,
    TextOnly(BcastFollowupTextOnlyReason),
}

impl Default for BcastFollowupDeliveryHint {
    fn default() -> Self {
        Self::VoiceDefault
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BcastFollowupPolicyGateDecision {
    gate_ok: bool,
    prompt_dedupe_key: Option<String>,
}

impl Default for SimulationExecutor {
    fn default() -> Self {
        Self {
            bcast: Ph1BcastRuntime::default(),
            delivery: Ph1DeliveryRuntime::default(),
            link: Ph1LinkRuntime::new(Ph1LinkConfig::mvp_v1()),
            memory: RefCell::new(new_ph1m_wiring()),
            onb: Ph1OnbOrchRuntime::default(),
            position: Ph1PositionRuntime,
            rem: Ph1RemRuntime::default(),
            policy: Ph1PolicyRuntime::new(Ph1PolicyConfig::mvp_v1()),
            capreq: Ph1CapreqRuntime,
            voice_id_live: Ph1VoiceIdLiveRuntime::default(),
            voice_id: Ph1VoiceIdRuntime,
            wake: Ph1wRuntime,
            os_top_level: new_os_top_level_wiring(),
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }
}

impl SimulationExecutor {
    pub fn new(link: Ph1LinkRuntime, onb: Ph1OnbOrchRuntime) -> Self {
        Self {
            bcast: Ph1BcastRuntime::default(),
            delivery: Ph1DeliveryRuntime::default(),
            link,
            memory: RefCell::new(new_ph1m_wiring()),
            onb,
            position: Ph1PositionRuntime,
            rem: Ph1RemRuntime::default(),
            policy: Ph1PolicyRuntime::new(Ph1PolicyConfig::mvp_v1()),
            capreq: Ph1CapreqRuntime,
            voice_id_live: Ph1VoiceIdLiveRuntime::default(),
            voice_id: Ph1VoiceIdRuntime,
            wake: Ph1wRuntime,
            os_top_level: new_os_top_level_wiring(),
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }

    pub fn new_with_wake(link: Ph1LinkRuntime, onb: Ph1OnbOrchRuntime, wake: Ph1wRuntime) -> Self {
        Self {
            bcast: Ph1BcastRuntime::default(),
            delivery: Ph1DeliveryRuntime::default(),
            link,
            memory: RefCell::new(new_ph1m_wiring()),
            onb,
            position: Ph1PositionRuntime,
            rem: Ph1RemRuntime::default(),
            policy: Ph1PolicyRuntime::new(Ph1PolicyConfig::mvp_v1()),
            capreq: Ph1CapreqRuntime,
            voice_id_live: Ph1VoiceIdLiveRuntime::default(),
            voice_id: Ph1VoiceIdRuntime,
            wake,
            os_top_level: new_os_top_level_wiring(),
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }

    pub fn new_with_position_and_wake(
        link: Ph1LinkRuntime,
        onb: Ph1OnbOrchRuntime,
        position: Ph1PositionRuntime,
        wake: Ph1wRuntime,
    ) -> Self {
        Self {
            bcast: Ph1BcastRuntime::default(),
            delivery: Ph1DeliveryRuntime::default(),
            link,
            memory: RefCell::new(new_ph1m_wiring()),
            onb,
            position,
            rem: Ph1RemRuntime::default(),
            policy: Ph1PolicyRuntime::new(Ph1PolicyConfig::mvp_v1()),
            capreq: Ph1CapreqRuntime,
            voice_id_live: Ph1VoiceIdLiveRuntime::default(),
            voice_id: Ph1VoiceIdRuntime,
            wake,
            os_top_level: new_os_top_level_wiring(),
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }

    pub fn set_voice_id_live_config(&mut self, config: Ph1VoiceIdLiveConfig) {
        self.voice_id_live = Ph1VoiceIdLiveRuntime::new(config);
    }

    pub fn set_device_sync_sender(&mut self, sender: DeviceArtifactSyncSenderRuntime) {
        self.device_sync_sender = sender;
    }

    pub fn execute_link(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1LinkRequest,
    ) -> Result<Ph1LinkResponse, StorageError> {
        if is_legacy_link_delivery_simulation_id(&req.simulation_id) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_request.simulation_id",
                    reason:
                        "LEGACY_DO_NOT_WIRE: delivery is owned by LINK_DELIVER_INVITE via PH1.BCAST + PH1.DELIVERY",
                },
            ));
        }
        self.link.run(store, req)
    }

    pub fn execute_onb(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1OnbRequest,
    ) -> Result<Ph1OnbResponse, StorageError> {
        self.onb.run(store, req)
    }

    pub fn execute_onb_voice_enrollment_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbVoiceEnrollLiveRequest,
    ) -> Result<OnbVoiceEnrollLiveResult, StorageError> {
        self.onb.run_voice_enrollment_live_sequence(store, req)
    }

    pub fn execute_onb_position_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbPositionLiveRequest,
    ) -> Result<OnbPositionLiveResult, StorageError> {
        self.onb.run_position_live_sequence(store, req)
    }

    pub fn execute_position(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1PositionRequest,
    ) -> Result<Ph1PositionResponse, StorageError> {
        self.position.run(store, req)
    }

    pub fn execute_rem(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1RemRequest,
    ) -> Result<Ph1RemResponse, StorageError> {
        self.rem.run(store, req)
    }

    pub fn execute_capreq(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1CapreqRequest,
    ) -> Result<Ph1CapreqResponse, StorageError> {
        self.capreq.run(store, req)
    }

    fn os_voice_platform_from_app_platform(app_platform: AppPlatform) -> OsVoicePlatform {
        match app_platform {
            AppPlatform::Ios => OsVoicePlatform::Ios,
            AppPlatform::Android => OsVoicePlatform::Android,
            AppPlatform::Desktop => OsVoicePlatform::Desktop,
        }
    }

    fn default_os_turn_input(
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<OsTurnInput, StorageError> {
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
        .map_err(StorageError::ContractViolation)
    }

    fn expected_always_on_sequence_for_voice(trigger: OsVoiceTrigger) -> Vec<String> {
        let mut seq = vec!["PH1.K".to_string()];
        if trigger == OsVoiceTrigger::WakeWord {
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

    fn resolve_app_platform_for_onboarding_sid(
        store: &Ph1fStore,
        onboarding_session_id: &OnboardingSessionId,
    ) -> Option<AppPlatform> {
        store
            .ph1onb_session_row(onboarding_session_id)
            .map(|session| session.app_platform)
    }

    fn resolve_voice_id_platform(
        &self,
        store: &Ph1fStore,
        req: &Ph1VoiceIdSimRequest,
    ) -> AppPlatform {
        match &req.request {
            VoiceIdSimulationRequest::EnrollStartDraft(r) => {
                OnboardingSessionId::new(r.onboarding_session_id.clone())
                    .ok()
                    .and_then(|sid| Self::resolve_app_platform_for_onboarding_sid(store, &sid))
                    .unwrap_or(AppPlatform::Ios)
            }
            VoiceIdSimulationRequest::EnrollSampleCommit(r) => store
                .ph1vid_get_enrollment_session(r.voice_enrollment_session_id.as_str())
                .and_then(|enroll| {
                    Self::resolve_app_platform_for_onboarding_sid(
                        store,
                        &enroll.onboarding_session_id,
                    )
                })
                .unwrap_or(AppPlatform::Ios),
            VoiceIdSimulationRequest::EnrollCompleteCommit(r) => store
                .ph1vid_get_enrollment_session(r.voice_enrollment_session_id.as_str())
                .and_then(|enroll| {
                    Self::resolve_app_platform_for_onboarding_sid(
                        store,
                        &enroll.onboarding_session_id,
                    )
                })
                .unwrap_or(AppPlatform::Ios),
            VoiceIdSimulationRequest::EnrollDeferCommit(r) => store
                .ph1vid_get_enrollment_session(r.voice_enrollment_session_id.as_str())
                .and_then(|enroll| {
                    Self::resolve_app_platform_for_onboarding_sid(
                        store,
                        &enroll.onboarding_session_id,
                    )
                })
                .unwrap_or(AppPlatform::Ios),
        }
    }

    fn resolve_wake_platform(&self, store: &Ph1fStore, req: &Ph1wRequest) -> AppPlatform {
        match &req.request {
            WakeRequest::EnrollStartDraft(r) => r
                .onboarding_session_id
                .as_ref()
                .and_then(|sid| OnboardingSessionId::new(sid.clone()).ok())
                .and_then(|sid| Self::resolve_app_platform_for_onboarding_sid(store, &sid))
                .unwrap_or(AppPlatform::Android),
            WakeRequest::EnrollSampleCommit(r) => store
                .ph1w_get_enrollment_session(r.wake_enrollment_session_id.as_str())
                .and_then(|enroll| enroll.onboarding_session_id.as_ref())
                .and_then(|sid| Self::resolve_app_platform_for_onboarding_sid(store, sid))
                .unwrap_or(AppPlatform::Android),
            WakeRequest::EnrollCompleteCommit(r) => store
                .ph1w_get_enrollment_session(r.wake_enrollment_session_id.as_str())
                .and_then(|enroll| enroll.onboarding_session_id.as_ref())
                .and_then(|sid| Self::resolve_app_platform_for_onboarding_sid(store, sid))
                .unwrap_or(AppPlatform::Android),
            WakeRequest::EnrollDeferCommit(r) => store
                .ph1w_get_enrollment_session(r.wake_enrollment_session_id.as_str())
                .and_then(|enroll| enroll.onboarding_session_id.as_ref())
                .and_then(|sid| Self::resolve_app_platform_for_onboarding_sid(store, sid))
                .unwrap_or(AppPlatform::Android),
        }
    }

    fn validate_voice_top_level_path(
        &self,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        app_platform: AppPlatform,
        trigger: OsVoiceTrigger,
    ) -> Result<(), StorageError> {
        let input = OsTopLevelTurnInput::v1(
            correlation_id,
            turn_id,
            OsTopLevelTurnPath::Voice,
            Some(OsVoiceTurnContext::v1(
                Self::os_voice_platform_from_app_platform(app_platform),
                trigger,
            )),
            Self::expected_always_on_sequence_for_voice(trigger),
            Vec::new(),
            0,
            Self::default_os_turn_input(correlation_id, turn_id)?,
        )
        .map_err(StorageError::ContractViolation)?;
        let outcome = self
            .os_top_level
            .run_turn(&input)
            .map_err(StorageError::ContractViolation)?;
        match outcome {
            OsTopLevelWiringOutcome::Forwarded(_) => Ok(()),
            OsTopLevelWiringOutcome::Refused(_) | OsTopLevelWiringOutcome::NotInvokedDisabled => {
                Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "simulation_executor.voice_top_level_path",
                        reason: "voice top-level wiring must forward for valid runtime requests",
                    },
                ))
            }
        }
    }

    fn run_device_artifact_sync_worker_pass(
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

    fn run_device_artifact_sync_worker_pass_with_metrics(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
        let worker_id = format!("sim_device_sync_worker_{}_{}", correlation_id.0, turn_id.0);
        device_artifact_sync::run_device_artifact_sync_worker_pass_with_metrics(
            store,
            now,
            worker_id,
            &self.device_sync_sender,
        )
    }

    pub fn execute_device_artifact_sync_worker_pass(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<(), StorageError> {
        self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)
    }

    pub fn execute_device_artifact_sync_worker_pass_with_metrics(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<DeviceArtifactSyncWorkerPassMetrics, StorageError> {
        self.run_device_artifact_sync_worker_pass_with_metrics(store, now, correlation_id, turn_id)
    }

    pub fn execute_voice_id(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1VoiceIdSimRequest,
    ) -> Result<Ph1VoiceIdSimResponse, StorageError> {
        let platform = self.resolve_voice_id_platform(store, req);
        self.validate_voice_top_level_path(
            req.correlation_id,
            req.turn_id,
            platform,
            OsVoiceTrigger::Explicit,
        )?;
        let resp = self.voice_id.run(store, req)?;
        // Always run one worker pass so replay/ack can progress continuously, not only on complete.
        self.run_device_artifact_sync_worker_pass(store, req.now, req.correlation_id, req.turn_id)?;
        Ok(resp)
    }

    /// Live PH1.OS voice entrypoint for app/server ingress traffic.
    /// This runs top-level OS voice orchestration and Voice-ID signal emission in one path.
    pub fn execute_os_voice_live_turn(
        &self,
        store: &mut Ph1fStore,
        input: OsVoiceLiveTurnInput,
    ) -> Result<OsVoiceLiveTurnOutcome, StorageError> {
        let runtime =
            Ph1OsVoiceLiveRuntime::new(self.os_top_level.clone(), self.voice_id_live.clone());
        runtime.run_turn(store, input)
    }

    pub fn execute_wake(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1wRequest,
    ) -> Result<Ph1wResponse, StorageError> {
        let platform = self.resolve_wake_platform(store, req);
        self.validate_voice_top_level_path(
            req.correlation_id,
            req.turn_id,
            platform,
            OsVoiceTrigger::WakeWord,
        )?;
        let resp = self.wake.run(store, req)?;
        // Always run one worker pass so replay/ack can progress continuously, not only on complete.
        self.run_device_artifact_sync_worker_pass(store, req.now, req.correlation_id, req.turn_id)?;
        Ok(resp)
    }

    /// Canonical BCAST -> DELIVERY bridge for provider send attempts:
    /// 1) PH1.BCAST validates lifecycle and emits `delivery_request_ref`.
    /// 2) PH1.DELIVERY performs provider send under the same simulation context.
    /// 3) Returns both responses for deterministic audit/replay and downstream state updates.
    pub fn run_broadcast_deliver_with_delivery(
        &self,
        req: &Ph1BcastRequest,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        let deliver_req = match &req.request {
            BcastRequest::DeliverCommit(v) => v,
            _ => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_delivery_bridge.request",
                        reason: "must be BcastRequest::DeliverCommit",
                    },
                ))
            }
        };
        if req.simulation_type != BcastSimulationType::Commit {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_delivery_bridge.simulation_type",
                    reason: "must be COMMIT for BCAST deliver bridge",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;

        let (delivery_request_ref, broadcast_id) = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::DeliverCommit(v) => (
                    v.delivery_request_ref.clone(),
                    v.broadcast_id.as_str().to_string(),
                ),
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_delivery_bridge.bcast_outcome",
                            reason: "must be DeliverCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_delivery_bridge.bcast_response",
                        reason: "BCAST deliver commit refused",
                    },
                ))
            }
        };

        let channel = match deliver_req.delivery_method {
            selene_kernel_contracts::ph1bcast::BcastDeliveryMethod::SeleneApp => {
                DeliveryChannel::AppPush
            }
            selene_kernel_contracts::ph1bcast::BcastDeliveryMethod::Sms => DeliveryChannel::Sms,
            selene_kernel_contracts::ph1bcast::BcastDeliveryMethod::Whatsapp => {
                DeliveryChannel::Whatsapp
            }
            selene_kernel_contracts::ph1bcast::BcastDeliveryMethod::Wechat => {
                DeliveryChannel::Wechat
            }
            selene_kernel_contracts::ph1bcast::BcastDeliveryMethod::Email => DeliveryChannel::Email,
        };
        let provider_ref =
            Ph1DeliveryRuntime::default_provider_ref_for_channel(channel).to_string();

        let delivery_req = Ph1DeliveryRequest::send_commit_v1(
            req.correlation_id,
            req.turn_id,
            req.now,
            deliver_req.tenant_id.clone(),
            broadcast_id,
            deliver_req.recipient_id.as_str().to_string(),
            channel,
            deliver_req.delivery_plan_ref.clone(),
            provider_ref,
            deliver_req.simulation_context.clone(),
            format!(
                "delivery_send:{}:{}",
                deliver_req.idempotency_key, delivery_request_ref
            ),
        )
        .map_err(StorageError::ContractViolation)?;

        let delivery_resp = self.delivery.run(&delivery_req);
        delivery_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;

        Ok(SimulationDispatchOutcome::BroadcastDeliverySend {
            bcast: bcast_resp,
            delivery: delivery_resp,
        })
    }

    /// Canonical BCAST.MHP handoff bridge:
    /// 1) PH1.BCAST commits defer with `handoff_to_reminder=true` (state -> REMINDER_SET),
    /// 2) Selene OS schedules PH1.REM timing (`BCAST_MHP_FOLLOWUP`),
    /// 3) returns both responses for deterministic audit/replay.
    ///
    /// This preserves ownership boundaries:
    /// - PH1.BCAST owns lifecycle state.
    /// - PH1.REM owns reminder timing mechanics only.
    pub fn run_broadcast_mhp_defer_with_reminder(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1BcastRequest,
        recipient_user_id: UserId,
        reminder_priority: ReminderPriorityLevel,
        prompt_dedupe_key: Option<&str>,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        let defer_req = match &req.request {
            BcastRequest::DeferCommit(v) => v,
            _ => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_handoff.request",
                        reason: "must be BcastRequest::DeferCommit",
                    },
                ))
            }
        };
        if req.simulation_type != BcastSimulationType::Commit {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_handoff.simulation_type",
                    reason: "must be COMMIT for BCAST defer handoff",
                },
            ));
        }
        if !defer_req.handoff_to_reminder {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_handoff.handoff_to_reminder",
                    reason: "must be true for BCAST->REM handoff",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;

        let defer_result = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::DeferCommit(v) => v,
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_mhp_handoff.bcast_outcome",
                            reason: "must be DeferCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_handoff.bcast_response",
                        reason: "BCAST defer commit refused",
                    },
                ))
            }
        };
        if defer_result.recipient_state != BcastRecipientState::ReminderSet {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_handoff.bcast_state",
                    reason: "must transition to REMINDER_SET before REM scheduling",
                },
            ));
        }

        let retry_at_s = defer_result.retry_at.0.to_string();
        let dedupe = prompt_dedupe_key.unwrap_or("none");
        let reminder_idempotency_key = format!(
            "bcast_mhp_followup:{}",
            short_hash_hex(&[
                defer_req.broadcast_id.as_str(),
                defer_req.recipient_id.as_str(),
                retry_at_s.as_str(),
                dedupe,
            ])
        );
        let reminder_request_text = format!(
            "bcast_mhp_followup:{}:{}",
            defer_req.broadcast_id.as_str(),
            defer_req.recipient_id.as_str()
        );
        let rem_req = Ph1RemRequest::schedule_commit_v1(
            req.correlation_id,
            req.turn_id,
            req.now,
            defer_req.tenant_id.clone(),
            recipient_user_id,
            None,
            ReminderType::BcastMhpFollowup,
            reminder_request_text,
            retry_at_s,
            "UTC".to_string(),
            ReminderLocalTimeMode::LocalTime,
            reminder_priority,
            None,
            vec![ReminderChannel::PhoneApp],
            reminder_idempotency_key,
        )?;
        let rem_resp = self.execute_rem(store, &rem_req)?;

        Ok(SimulationDispatchOutcome::BroadcastMhpHandoff {
            bcast: bcast_resp,
            reminder: rem_resp,
        })
    }

    /// Execute delivery and produce urgent follow-up decision when delivery enters FOLLOWUP immediately.
    pub fn run_broadcast_mhp_deliver_and_maybe_followup(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        self.run_broadcast_mhp_deliver_and_maybe_followup_with_delivery_hint(
            req,
            tenant_id,
            recipient_user_id,
            active_speaker_user_id,
            subject_ref,
            prompt_dedupe_keys,
            BcastFollowupDeliveryHint::VoiceDefault,
        )
    }

    /// Execute delivery and produce urgent follow-up decision when delivery enters FOLLOWUP immediately.
    ///
    /// Hard rule:
    /// - Follow-up communication is VOICE by default.
    /// - TEXT is allowed only when explicitly requested by the user or when the user cannot speak.
    pub fn run_broadcast_mhp_deliver_and_maybe_followup_with_delivery_hint(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
        followup_delivery_hint: BcastFollowupDeliveryHint,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_deliver_followup.recipient_user_id",
            recipient_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_deliver_followup.active_speaker_user_id",
            active_speaker_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_deliver_followup.subject_ref",
            subject_ref,
            256,
        )?;
        if recipient_user_id != active_speaker_user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_deliver_followup.active_speaker_user_id",
                    reason: "must match recipient_user_id for targeted followup",
                },
            ));
        }

        if !matches!(req.request, BcastRequest::DeliverCommit(_)) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_deliver_followup.request",
                    reason: "must be BcastRequest::DeliverCommit",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        let (broadcast_id, recipient_id, followup_required) = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::DeliverCommit(v) => (
                    v.broadcast_id.as_str().to_string(),
                    v.recipient_id.as_str().to_string(),
                    v.followup_immediate,
                ),
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_mhp_deliver_followup.bcast_outcome",
                            reason: "must be DeliverCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_deliver_followup.bcast_response",
                        reason: "BCAST deliver commit refused",
                    },
                ))
            }
        };

        if !followup_required {
            let (
                followup_delivery_mode,
                followup_text_only_reason,
                followup_voice_ref,
                followup_text_ref,
            ) = resolve_followup_delivery_refs(
                false,
                followup_delivery_hint,
                "URGENT_POST_DELIVERY",
                broadcast_id.as_str(),
                recipient_id.as_str(),
                &req.simulation_id,
            );
            return Ok(SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
                bcast: bcast_resp,
                followup_source: "URGENT_POST_DELIVERY".to_string(),
                policy_gate_ok: false,
                followup_subject_ref: subject_ref.to_string(),
                followup_recipient_user_id: recipient_user_id.to_string(),
                followup_active_speaker_user_id: active_speaker_user_id.to_string(),
                followup_delivery_mode,
                followup_text_only_reason,
                followup_voice_ref,
                followup_text_ref,
                policy_prompt_dedupe_key: None,
            });
        }

        let policy = self.evaluate_bcast_followup_policy_gate(
            req.correlation_id,
            req.turn_id,
            req.now,
            tenant_id,
            Some(recipient_user_id),
            broadcast_id.as_str(),
            recipient_id.as_str(),
            "urgent_post_delivery",
            prompt_dedupe_keys,
        )?;
        let (
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
        ) = resolve_followup_delivery_refs(
            policy.gate_ok,
            followup_delivery_hint,
            "URGENT_POST_DELIVERY",
            broadcast_id.as_str(),
            recipient_id.as_str(),
            &req.simulation_id,
        );

        Ok(SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
            bcast: bcast_resp,
            followup_source: "URGENT_POST_DELIVERY".to_string(),
            policy_gate_ok: policy.gate_ok,
            followup_subject_ref: subject_ref.to_string(),
            followup_recipient_user_id: recipient_user_id.to_string(),
            followup_active_speaker_user_id: active_speaker_user_id.to_string(),
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
            policy_prompt_dedupe_key: policy.prompt_dedupe_key,
        })
    }

    /// Execute WAITING timeout follow-up transition (`WAITING -> FOLLOWUP`) and gate voice interruption via PH1.POLICY.
    pub fn run_broadcast_mhp_wait_timeout_followup(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        self.run_broadcast_mhp_wait_timeout_followup_with_delivery_hint(
            req,
            tenant_id,
            recipient_user_id,
            active_speaker_user_id,
            subject_ref,
            prompt_dedupe_keys,
            BcastFollowupDeliveryHint::VoiceDefault,
        )
    }

    /// Execute WAITING timeout follow-up transition (`WAITING -> FOLLOWUP`) and gate voice interruption via PH1.POLICY.
    ///
    /// Hard rule:
    /// - Follow-up communication is VOICE by default.
    /// - TEXT is allowed only when explicitly requested by the user or when the user cannot speak.
    pub fn run_broadcast_mhp_wait_timeout_followup_with_delivery_hint(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
        followup_delivery_hint: BcastFollowupDeliveryHint,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_wait_timeout_followup.recipient_user_id",
            recipient_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_wait_timeout_followup.active_speaker_user_id",
            active_speaker_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_wait_timeout_followup.subject_ref",
            subject_ref,
            256,
        )?;
        if recipient_user_id != active_speaker_user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_wait_timeout_followup.active_speaker_user_id",
                    reason: "must match recipient_user_id for targeted followup",
                },
            ));
        }

        if !matches!(req.request, BcastRequest::EscalateCommit(_)) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_wait_timeout_followup.request",
                    reason: "must be BcastRequest::EscalateCommit",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        let (broadcast_id, recipient_id) = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::EscalateCommit(v) => (
                    v.broadcast_id.as_str().to_string(),
                    v.recipient_id.as_str().to_string(),
                ),
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_mhp_wait_timeout_followup.bcast_outcome",
                            reason: "must be EscalateCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_wait_timeout_followup.bcast_response",
                        reason: "BCAST wait-timeout followup refused",
                    },
                ))
            }
        };

        let policy = self.evaluate_bcast_followup_policy_gate(
            req.correlation_id,
            req.turn_id,
            req.now,
            tenant_id,
            Some(recipient_user_id),
            broadcast_id.as_str(),
            recipient_id.as_str(),
            "wait_timeout_followup",
            prompt_dedupe_keys,
        )?;
        let (
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
        ) = resolve_followup_delivery_refs(
            policy.gate_ok,
            followup_delivery_hint,
            "WAIT_TIMEOUT_FOLLOWUP",
            broadcast_id.as_str(),
            recipient_id.as_str(),
            &req.simulation_id,
        );

        Ok(SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
            bcast: bcast_resp,
            followup_source: "WAIT_TIMEOUT_FOLLOWUP".to_string(),
            policy_gate_ok: policy.gate_ok,
            followup_subject_ref: subject_ref.to_string(),
            followup_recipient_user_id: recipient_user_id.to_string(),
            followup_active_speaker_user_id: active_speaker_user_id.to_string(),
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
            policy_prompt_dedupe_key: policy.prompt_dedupe_key,
        })
    }

    /// Resume BCAST.MHP lifecycle on reminder fire (`REMINDER_SET -> REMINDER_FIRED`).
    pub fn run_broadcast_mhp_mark_reminder_fired(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        self.run_broadcast_mhp_mark_reminder_fired_with_delivery_hint(
            req,
            tenant_id,
            recipient_user_id,
            active_speaker_user_id,
            subject_ref,
            prompt_dedupe_keys,
            BcastFollowupDeliveryHint::VoiceDefault,
        )
    }

    /// Resume BCAST.MHP lifecycle on reminder fire (`REMINDER_SET -> REMINDER_FIRED`).
    ///
    /// Hard rule:
    /// - Follow-up communication is VOICE by default.
    /// - TEXT is allowed only when explicitly requested by the user or when the user cannot speak.
    pub fn run_broadcast_mhp_mark_reminder_fired_with_delivery_hint(
        &self,
        req: &Ph1BcastRequest,
        tenant_id: &str,
        recipient_user_id: &str,
        active_speaker_user_id: &str,
        subject_ref: &str,
        prompt_dedupe_keys: &[String],
        followup_delivery_hint: BcastFollowupDeliveryHint,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_reminder_fired.recipient_user_id",
            recipient_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_reminder_fired.active_speaker_user_id",
            active_speaker_user_id,
            128,
        )?;
        validate_nonempty_bounded_text(
            "broadcast_mhp_reminder_fired.subject_ref",
            subject_ref,
            256,
        )?;
        if recipient_user_id != active_speaker_user_id {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_reminder_fired.active_speaker_user_id",
                    reason: "must match recipient_user_id for targeted followup",
                },
            ));
        }

        if req.simulation_id != BCAST_REMINDER_FIRED_COMMIT {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_reminder_fired.simulation_id",
                    reason: "must be BCAST_REMINDER_FIRED_COMMIT",
                },
            ));
        }
        if !matches!(req.request, BcastRequest::ReminderFiredCommit(_)) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_reminder_fired.request",
                    reason: "must be BcastRequest::ReminderFiredCommit",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        let (broadcast_id, recipient_id) = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::ReminderFiredCommit(v) => {
                    if v.recipient_state != BcastRecipientState::ReminderFired {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "broadcast_mhp_reminder_fired.recipient_state",
                                reason: "must be REMINDER_FIRED",
                            },
                        ));
                    }
                    (
                        v.broadcast_id.as_str().to_string(),
                        v.recipient_id.as_str().to_string(),
                    )
                }
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_mhp_reminder_fired.bcast_outcome",
                            reason: "must be ReminderFiredCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_reminder_fired.bcast_response",
                        reason: "BCAST reminder fired commit refused",
                    },
                ))
            }
        };

        let policy = self.evaluate_bcast_followup_policy_gate(
            req.correlation_id,
            req.turn_id,
            req.now,
            tenant_id,
            Some(recipient_user_id),
            broadcast_id.as_str(),
            recipient_id.as_str(),
            "reminder_fired_followup",
            prompt_dedupe_keys,
        )?;
        let (
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
        ) = resolve_followup_delivery_refs(
            policy.gate_ok,
            followup_delivery_hint,
            "REMINDER_FIRED_FOLLOWUP",
            broadcast_id.as_str(),
            recipient_id.as_str(),
            &req.simulation_id,
        );

        Ok(SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
            bcast: bcast_resp,
            followup_source: "REMINDER_FIRED_FOLLOWUP".to_string(),
            policy_gate_ok: policy.gate_ok,
            followup_subject_ref: subject_ref.to_string(),
            followup_recipient_user_id: recipient_user_id.to_string(),
            followup_active_speaker_user_id: active_speaker_user_id.to_string(),
            followup_delivery_mode,
            followup_text_only_reason,
            followup_voice_ref,
            followup_text_ref,
            policy_prompt_dedupe_key: policy.prompt_dedupe_key,
        })
    }

    /// App-thread direct reply: conclude JD thread and auto-forward response to wife app thread.
    /// Voice interruption is suppressed by design on this path.
    pub fn run_broadcast_mhp_app_thread_reply_conclude(
        &self,
        req: &Ph1BcastRequest,
        wife_thread_ref: &str,
        reply_payload_ref: &str,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;
        if !matches!(req.request, BcastRequest::AckCommit(_)) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_app_thread_reply.request",
                    reason: "must be BcastRequest::AckCommit",
                },
            ));
        }
        if wife_thread_ref.trim().is_empty() || reply_payload_ref.trim().is_empty() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_app_thread_reply",
                    reason: "wife_thread_ref and reply_payload_ref are required",
                },
            ));
        }

        let bcast_resp = self.bcast.run(req);
        bcast_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        let (broadcast_id, recipient_id) = match &bcast_resp {
            Ph1BcastResponse::Ok(ok) => match &ok.outcome {
                BcastOutcome::AckCommit(v) => {
                    if v.recipient_state != BcastRecipientState::Concluded {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "broadcast_mhp_app_thread_reply.recipient_state",
                                reason: "must be CONCLUDED",
                            },
                        ));
                    }
                    (
                        v.broadcast_id.as_str().to_string(),
                        v.recipient_id.as_str().to_string(),
                    )
                }
                _ => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "broadcast_mhp_app_thread_reply.bcast_outcome",
                            reason: "must be AckCommit",
                        },
                    ))
                }
            },
            Ph1BcastResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "broadcast_mhp_app_thread_reply.bcast_response",
                        reason: "BCAST app-thread reply conclude refused",
                    },
                ))
            }
        };

        let wife_forward_ref = format!(
            "bcast_wife_forward_{}",
            short_hash_hex(&[
                broadcast_id.as_str(),
                recipient_id.as_str(),
                wife_thread_ref,
                reply_payload_ref,
                &req.simulation_id,
            ])
        );

        Ok(
            SimulationDispatchOutcome::BroadcastMhpAppThreadReplyConcluded {
                bcast: bcast_resp,
                wife_forward_ref,
                voice_interruption_suppressed: true,
            },
        )
    }

    fn evaluate_bcast_followup_policy_gate(
        &self,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        tenant_id: &str,
        user_id: Option<&str>,
        broadcast_id: &str,
        recipient_id: &str,
        followup_scope: &str,
        prompt_dedupe_keys: &[String],
    ) -> Result<BcastFollowupPolicyGateDecision, StorageError> {
        let envelope = PolicyRequestEnvelope::v1(correlation_id, turn_id, 8)
            .map_err(StorageError::ContractViolation)?;
        let ruleset_req = Ph1PolicyRequest::PolicyRulesetGetActive(
            PolicyRulesetGetActiveRequest::v1(
                envelope.clone(),
                tenant_id.to_string(),
                user_id.map(|v| v.to_string()),
                now.0,
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let ruleset_resp = self.policy.run(&ruleset_req);
        ruleset_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        if matches!(ruleset_resp, Ph1PolicyResponse::Refuse(_)) {
            return Ok(BcastFollowupPolicyGateDecision {
                gate_ok: false,
                prompt_dedupe_key: None,
            });
        }

        let work_order_id = format!("bcast:{broadcast_id}:{recipient_id}:{followup_scope}");
        let prompt_req = Ph1PolicyRequest::PolicyPromptDedupeDecide(
            PolicyPromptDedupeDecideRequest::v1(
                envelope,
                tenant_id.to_string(),
                work_order_id,
                now.0,
                vec!["bcast_voice_interrupt".to_string()],
                vec![],
                vec![],
                prompt_dedupe_keys.to_vec(),
                vec![],
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let prompt_resp = self.policy.run(&prompt_req);
        prompt_resp
            .validate()
            .map_err(StorageError::ContractViolation)?;
        match prompt_resp {
            Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok) => {
                Ok(BcastFollowupPolicyGateDecision {
                    gate_ok: ok.decision == PolicyPromptDecision::Ask,
                    prompt_dedupe_key: ok.prompt_dedupe_key,
                })
            }
            Ph1PolicyResponse::Refuse(_) => Ok(BcastFollowupPolicyGateDecision {
                gate_ok: false,
                prompt_dedupe_key: None,
            }),
            Ph1PolicyResponse::PolicyRulesetGetActiveOk(_) => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "broadcast_mhp_policy_gate.prompt_response",
                    reason: "unexpected ruleset response for prompt gate request",
                },
            )),
        }
    }

    /// Execute a PH1.X `Dispatch(SimulationCandidate)` by mapping it to a concrete simulation call.
    ///
    /// Hard rule: No Simulation -> No Execution. This method only accepts a SimulationCandidate dispatch.
    pub fn execute_ph1x_dispatch_simulation_candidate(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        now: MonotonicTimeNs,
        x: &Ph1xResponse,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        let dispatch = match &x.directive {
            Ph1xDirective::Dispatch(d) => d,
            _ => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1x_response.directive",
                        reason: "must be Dispatch(SimulationCandidate)",
                    },
                ))
            }
        };

        match &dispatch.dispatch_request {
            DispatchRequest::SimulationCandidate(c) => {
                self.guard_simulation_candidate_dispatch_v1(
                    store,
                    &actor_user_id,
                    now,
                    &c.intent_draft,
                )?;
                self.execute_simulation_candidate_v1(
                    store,
                    actor_user_id,
                    now,
                    CorrelationId(x.correlation_id),
                    TurnId(x.turn_id),
                    x.idempotency_key.as_deref(),
                    &c.intent_draft,
                )
            }
            DispatchRequest::AccessStepUp(step_up) => self.execute_access_step_up_dispatch_v1(
                store,
                actor_user_id,
                now,
                CorrelationId(x.correlation_id),
                TurnId(x.turn_id),
                x.idempotency_key.as_deref(),
                step_up,
            ),
            DispatchRequest::Tool(_) => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1x_response.directive.dispatch_request",
                    reason:
                        "tool dispatch must be handled by PH1.E (not SimulationExecutor step-up/sim dispatch path)",
                },
            )),
        }
    }

    pub fn step_up_result_from_dispatch_outcome(
        outcome: &SimulationDispatchOutcome,
    ) -> Option<StepUpResult> {
        match outcome {
            SimulationDispatchOutcome::AccessStepUp {
                challenge_method,
                outcome,
                reason_code,
                ..
            } => Some(
                StepUpResult::v1(*outcome, *challenge_method, *reason_code)
                    .expect("step-up dispatch outcome must map to valid StepUpResult"),
            ),
            _ => None,
        }
    }

    fn guard_simulation_candidate_dispatch_v1(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        now: MonotonicTimeNs,
        d: &IntentDraft,
    ) -> Result<(), StorageError> {
        self.ensure_dispatch_access_allow_v1(store, actor_user_id, now, d)?;
        let (tenant_id, simulation_id) =
            simulation_catalog_guard_target_v1(store, actor_user_id, d)?;
        let Some(sim_row) = store.simulation_catalog_current_row(&tenant_id, &simulation_id) else {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.simulation_id",
                    reason: "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                },
            ));
        };
        if sim_row.status != SimulationStatus::Active {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.simulation_id",
                    reason: "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                },
            ));
        }
        Ok(())
    }

    fn ensure_dispatch_access_allow_v1(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        now: MonotonicTimeNs,
        d: &IntentDraft,
    ) -> Result<(), StorageError> {
        match d.intent_type {
            IntentType::CreateInviteLink => {
                let tenant_id = resolve_invite_tenant_id(store, d, actor_user_id)?;
                self.enforce_link_access_gate(store, actor_user_id, &tenant_id, now)
            }
            IntentType::CapreqManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_capreq_access_gate(store, actor_user_id, &tenant_id, now)
            }
            IntentType::AccessSchemaManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_schema_gate(store, actor_user_id, &tenant_id, now)
            }
            IntentType::AccessEscalationVote => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_escalation_vote_gate(store, actor_user_id, &tenant_id, now)
            }
            IntentType::AccessInstanceCompileRefresh => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_instance_compile_gate(store, actor_user_id, &tenant_id, now)
            }
            _ => Ok(()),
        }
    }

    fn execute_access_step_up_dispatch_v1(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        x_idempotency_key: Option<&str>,
        dispatch: &AccessStepUpDispatch,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        const ACCESS_STEP_UP_START: ReasonCodeId = ReasonCodeId(0x4153_0001);
        const ACCESS_STEP_UP_DEFER_UNVERIFIED: ReasonCodeId = ReasonCodeId(0x4153_0002);

        let tenant_id = resolve_reminder_tenant_id(store, &dispatch.intent_draft, &actor_user_id)?;
        let Some(access_instance) =
            store.ph2access_get_instance_by_tenant_user(tenant_id.as_str(), &actor_user_id)
        else {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "access_step_up_dispatch.access_instance_id",
                    reason: "missing access instance for actor_user_id + tenant_id",
                },
            ));
        };
        let access_instance_id = access_instance.access_instance_id.clone();
        let access_device_trust_level = access_instance.device_trust_level;
        let access_verification_level = access_instance.verification_level;

        let audit_key_prefix = x_idempotency_key
            .map(|k| format!("access_step_up:{k}"))
            .unwrap_or_else(|| format!("access_step_up:{}:{}", correlation_id.0, turn_id.0));
        store.ph1access_capreq_step_up_audit_commit(
            now,
            tenant_id.as_str().to_string(),
            correlation_id,
            turn_id,
            actor_user_id.clone(),
            "START".to_string(),
            "PENDING".to_string(),
            dispatch.requested_action.clone(),
            dispatch.challenge_method.as_str().to_string(),
            ACCESS_STEP_UP_START,
            format!("{audit_key_prefix}:start"),
        )?;

        let gate = store.ph1access_gate_decide(
            actor_user_id.clone(),
            access_instance_id,
            dispatch.requested_action.clone(),
            AccessMode::A,
            access_device_trust_level,
            false,
            now,
        )?;

        let (outcome, reason_code) = match gate.access_decision {
            AccessDecision::Deny => (StepUpOutcome::Refuse, gate.reason_code),
            AccessDecision::Escalate => (StepUpOutcome::Defer, gate.reason_code),
            AccessDecision::Allow => {
                if step_up_challenge_satisfied(access_verification_level, dispatch.challenge_method)
                {
                    (StepUpOutcome::Continue, gate.reason_code)
                } else {
                    (StepUpOutcome::Defer, ACCESS_STEP_UP_DEFER_UNVERIFIED)
                }
            }
        };

        store.ph1access_capreq_step_up_audit_commit(
            now,
            tenant_id.as_str().to_string(),
            correlation_id,
            turn_id,
            actor_user_id,
            "FINISH".to_string(),
            outcome.as_str().to_string(),
            dispatch.requested_action.clone(),
            dispatch.challenge_method.as_str().to_string(),
            reason_code,
            format!("{audit_key_prefix}:finish"),
        )?;

        Ok(SimulationDispatchOutcome::AccessStepUp {
            requested_action: dispatch.requested_action.clone(),
            challenge_method: dispatch.challenge_method,
            outcome,
            reason_code,
        })
    }

    fn execute_simulation_candidate_v1(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        x_idempotency_key: Option<&str>,
        d: &IntentDraft,
    ) -> Result<SimulationDispatchOutcome, StorageError> {
        match d.intent_type {
            IntentType::SetReminder => {
                let tenant_id = resolve_reminder_tenant_id(store, d, &actor_user_id)?;
                let task = required_field_value(d, FieldKey::Task)?;
                let when = required_field_value(d, FieldKey::When)?;
                let idempotency_key = x_idempotency_key
                    .map(|k| format!("reminder_schedule:{k}"))
                    .unwrap_or_else(|| {
                        format!("reminder_schedule:{}:{}", correlation_id.0, turn_id.0)
                    });
                let req = Ph1RemRequest::schedule_commit_v1(
                    correlation_id,
                    turn_id,
                    now,
                    tenant_id,
                    actor_user_id.clone(),
                    None,
                    ReminderType::Task,
                    field_str(task).to_string(),
                    field_str(when).to_string(),
                    "UTC".to_string(),
                    ReminderLocalTimeMode::LocalTime,
                    ReminderPriorityLevel::Normal,
                    None,
                    vec![ReminderChannel::Text],
                    idempotency_key,
                )?;
                let resp = self.execute_rem(store, &req)?;
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::Reminder(resp))
            }
            IntentType::MemoryRememberRequest => {
                let task = required_field_value(d, FieldKey::Task)?;
                let subject_text = field_str(task).to_string();
                validate_nonempty_bounded_text(
                    "simulation_candidate_dispatch.intent_draft.fields.task",
                    &subject_text,
                    256,
                )?;
                let resolved_voice_identity = resolve_voice_identity_assertion(
                    &self.os_top_level,
                    &self.voice_id_live,
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                )?;
                let ResolvedVoiceIdentity {
                    assertion: speaker_assertion,
                    identity_prompt_scope_key: _identity_prompt_scope_key,
                } = resolved_voice_identity;
                let memory_key = derive_memory_key_from_subject(&subject_text)?;
                let proposal = MemoryProposedItem::v1(
                    memory_key,
                    MemoryValue::v1(subject_text.clone(), None)
                        .map_err(StorageError::ContractViolation)?,
                    MemoryLayer::Micro,
                    MemorySensitivityFlag::Low,
                    MemoryConfidence::High,
                    MemoryConsent::ExplicitRemember,
                    truncate_utf8(&subject_text, 256),
                    MemoryProvenance::v1(None, None).map_err(StorageError::ContractViolation)?,
                )
                .map_err(StorageError::ContractViolation)?;
                let req = Ph1mProposeRequest::v1(
                    now,
                    speaker_assertion,
                    PolicyContextRef::v1(false, false, SafetyTier::Standard),
                    vec![proposal],
                )
                .map_err(StorageError::ContractViolation)?;
                let input =
                    MemoryTurnInput::v1(correlation_id, turn_id, MemoryOperation::Propose(req))
                        .map_err(StorageError::ContractViolation)?;
                let output = self.execute_memory_turn_output(store, &input)?;
                let MemoryTurnOutput::Propose(resp) = output else {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "simulation_candidate_dispatch.ph1m.output",
                            reason: "expected propose output",
                        },
                    ));
                };
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::MemoryPropose(resp))
            }
            IntentType::MemoryForgetRequest => {
                let task = required_field_value(d, FieldKey::Task)?;
                let subject_text = field_str(task).to_string();
                validate_nonempty_bounded_text(
                    "simulation_candidate_dispatch.intent_draft.fields.task",
                    &subject_text,
                    256,
                )?;
                let resolved_voice_identity = resolve_voice_identity_assertion(
                    &self.os_top_level,
                    &self.voice_id_live,
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                )?;
                let ResolvedVoiceIdentity {
                    assertion: speaker_assertion,
                    identity_prompt_scope_key: _identity_prompt_scope_key,
                } = resolved_voice_identity;
                let target_key = derive_memory_key_from_subject(&subject_text)?;
                let req = Ph1mForgetRequest::v1(
                    now,
                    speaker_assertion,
                    PolicyContextRef::v1(false, false, SafetyTier::Standard),
                    target_key,
                )
                .map_err(StorageError::ContractViolation)?;
                let input =
                    MemoryTurnInput::v1(correlation_id, turn_id, MemoryOperation::Forget(req))
                        .map_err(StorageError::ContractViolation)?;
                let output = self.execute_memory_turn_output(store, &input)?;
                let MemoryTurnOutput::Forget(resp) = output else {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "simulation_candidate_dispatch.ph1m.output",
                            reason: "expected forget output",
                        },
                    ));
                };
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::MemoryForget(resp))
            }
            IntentType::MemoryQuery => {
                let resolved_voice_identity = resolve_voice_identity_assertion(
                    &self.os_top_level,
                    &self.voice_id_live,
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                )?;
                let ResolvedVoiceIdentity {
                    assertion: speaker_assertion,
                    identity_prompt_scope_key: _identity_prompt_scope_key,
                } = resolved_voice_identity;
                let requested_keys = match optional_field_value(d, FieldKey::Task) {
                    Some(task) => {
                        let subject_text = field_str(task).to_string();
                        validate_nonempty_bounded_text(
                            "simulation_candidate_dispatch.intent_draft.fields.task",
                            &subject_text,
                            256,
                        )?;
                        vec![derive_memory_key_from_subject(&subject_text)?]
                    }
                    None => vec![],
                };
                let req = Ph1mRecallRequest::v1(
                    now,
                    speaker_assertion,
                    PolicyContextRef::v1(false, false, SafetyTier::Standard),
                    requested_keys,
                    false,
                    10,
                )
                .map_err(StorageError::ContractViolation)?;
                let input =
                    MemoryTurnInput::v1(correlation_id, turn_id, MemoryOperation::Recall(req))
                        .map_err(StorageError::ContractViolation)?;
                let output = self.execute_memory_turn_output(store, &input)?;
                let MemoryTurnOutput::Recall(resp) = output else {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "simulation_candidate_dispatch.ph1m.output",
                            reason: "expected recall output",
                        },
                    ));
                };
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::MemoryRecall(resp))
            }
            IntentType::CreateInviteLink => {
                let invitee_type =
                    parse_invitee_type(required_field_value(d, FieldKey::InviteeType)?)?;
                let tenant_id = resolve_invite_tenant_id(store, d, &actor_user_id)?;
                self.enforce_link_access_gate(store, &actor_user_id, &tenant_id, now)?;
                let tenant_id = Some(tenant_id.as_str().to_string());

                let req = Ph1LinkRequest::invite_generate_draft_v1(
                    correlation_id,
                    turn_id,
                    now,
                    actor_user_id.clone(),
                    invitee_type,
                    tenant_id,
                    None,
                    None,
                    None,
                )?;
                let resp = self.execute_link(store, &req)?;
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::Link(resp))
            }
            IntentType::CapreqManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_capreq_access_gate(store, &actor_user_id, &tenant_id, now)?;
                let requested_capability_id =
                    optional_field_value(d, FieldKey::RequestedCapabilityId)
                        .map(|v| field_str(v).to_string());
                let target_scope_ref = optional_field_value(d, FieldKey::TargetScopeRef)
                    .map(|v| field_str(v).to_string());
                let justification = optional_field_value(d, FieldKey::Justification)
                    .map(|v| field_str(v).to_string());

                let action = parse_capreq_action(optional_field_value(d, FieldKey::CapreqAction))?;

                let action_token = capreq_action_token(action);
                let idempotency_key = x_idempotency_key
                    .map(|k| format!("capreq_{action_token}:{k}"))
                    .unwrap_or_else(|| {
                        format!("capreq_{action_token}:{}:{}", correlation_id.0, turn_id.0)
                    });

                let req = match action {
                    CapabilityRequestAction::CreateDraft => Ph1CapreqRequest::create_draft_v1(
                        correlation_id,
                        turn_id,
                        now,
                        actor_user_id.clone(),
                        tenant_id,
                        required_capreq_snapshot_field(
                            requested_capability_id,
                            "simulation_candidate_dispatch.intent_draft.fields.requested_capability_id",
                        )?,
                        required_capreq_snapshot_field(
                            target_scope_ref,
                            "simulation_candidate_dispatch.intent_draft.fields.target_scope_ref",
                        )?,
                        required_capreq_snapshot_field(
                            justification,
                            "simulation_candidate_dispatch.intent_draft.fields.justification",
                        )?,
                        idempotency_key,
                    )?,
                    CapabilityRequestAction::SubmitForApproval => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::submit_for_approval_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id.clone(),
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Approve => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::approve_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id.clone(),
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Reject => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::reject_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id.clone(),
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Fulfill => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::fulfill_commit_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id.clone(),
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                    CapabilityRequestAction::Cancel => {
                        let capreq_id = resolve_capreq_id(
                            d,
                            action,
                            &tenant_id,
                            requested_capability_id.as_deref(),
                            target_scope_ref.as_deref(),
                            justification.as_deref(),
                        )?;
                        Ph1CapreqRequest::cancel_revoke_v1(
                            correlation_id,
                            turn_id,
                            now,
                            actor_user_id.clone(),
                            tenant_id,
                            capreq_id,
                            idempotency_key,
                        )?
                    }
                };

                let resp = self.execute_capreq(store, &req)?;
                match resp {
                    Ph1CapreqResponse::Ok(ok) => {
                        let lifecycle = ok.lifecycle_result;
                        self.best_effort_ph1m_capture_turn_digest(
                            store,
                            &actor_user_id,
                            now,
                            correlation_id,
                            turn_id,
                            d,
                            x_idempotency_key,
                        );
                        Ok(SimulationDispatchOutcome::CapreqLifecycle {
                            capreq_id: lifecycle.capreq_id,
                            capreq_event_id: lifecycle.capreq_event_id,
                            action: lifecycle.action,
                            status: lifecycle.status,
                        })
                    }
                    Ph1CapreqResponse::Refuse(_) => Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1capreq_response",
                            reason: "refuse is unexpected for simulation-candidate dispatch",
                        },
                    )),
                }
            }
            IntentType::AccessSchemaManage => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                let action = parse_access_ap_action(required_field_value(d, FieldKey::ApAction)?)?;
                let _review_channel = parse_access_review_channel(required_field_value(
                    d,
                    FieldKey::AccessReviewChannel,
                )?)?;
                let _access_profile_id = required_field_value(d, FieldKey::AccessProfileId)?;
                let _schema_version_id = required_field_value(d, FieldKey::SchemaVersionId)?;
                if matches!(action, AccessApAction::CreateDraft | AccessApAction::Update) {
                    let _profile_payload = required_field_value(d, FieldKey::ProfilePayloadJson)?;
                }
                if action == AccessApAction::Activate {
                    let _rule_action = parse_access_rule_action(required_field_value(
                        d,
                        FieldKey::AccessRuleAction,
                    )?)?;
                }
                self.enforce_access_schema_gate(store, &actor_user_id, &tenant_id, now)?;
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_SCHEMA_MANAGE".to_string(),
                })
            }
            IntentType::AccessEscalationVote => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_escalation_vote_gate(store, &actor_user_id, &tenant_id, now)?;
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_ESCALATION_VOTE".to_string(),
                })
            }
            IntentType::AccessInstanceCompileRefresh => {
                let tenant_id = parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?;
                self.enforce_access_instance_compile_gate(store, &actor_user_id, &tenant_id, now)?;
                self.best_effort_ph1m_capture_turn_digest(
                    store,
                    &actor_user_id,
                    now,
                    correlation_id,
                    turn_id,
                    d,
                    x_idempotency_key,
                );
                Ok(SimulationDispatchOutcome::AccessGatePassed {
                    requested_action: "ACCESS_INSTANCE_COMPILE_REFRESH".to_string(),
                })
            }
            _ => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.intent_type",
                    reason: "unsupported in this slice",
                },
            )),
        }
    }

    fn execute_memory_turn_output(
        &self,
        store: &mut Ph1fStore,
        input: &MemoryTurnInput,
    ) -> Result<MemoryTurnOutput, StorageError> {
        let outcome = self
            .memory
            .borrow_mut()
            .run_turn_and_persist(store, input)?;
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => Ok(bundle.output),
            MemoryWiringOutcome::NotInvokedDisabled => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.ph1m",
                    reason: "memory_wiring_disabled",
                },
            )),
            MemoryWiringOutcome::Refused(_) => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.ph1m",
                    reason: "memory_wiring_refused",
                },
            )),
        }
    }

    // Best-effort PH1.M continuity capture for live simulation dispatches.
    // This must not alter primary simulation outcomes.
    fn best_effort_ph1m_capture_turn_digest(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: &UserId,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        d: &IntentDraft,
        x_idempotency_key: Option<&str>,
    ) {
        let speaker_assertion = match resolve_voice_identity_assertion(
            &self.os_top_level,
            &self.voice_id_live,
            store,
            actor_user_id,
            now,
            correlation_id,
            turn_id,
        ) {
            Ok(ResolvedVoiceIdentity {
                assertion,
                identity_prompt_scope_key: _identity_prompt_scope_key,
            }) => assertion,
            Err(_) => return,
        };
        let policy_context_ref = PolicyContextRef::v1(false, false, SafetyTier::Standard);
        let thread_digest = match build_memory_thread_digest(now, correlation_id, turn_id, d) {
            Ok(v) => v,
            Err(_) => return,
        };

        let idempotency_key = x_idempotency_key
            .map(|k| format!("ph1m_turn_digest:{k}"))
            .unwrap_or_else(|| format!("ph1m_turn_digest:{}:{}", correlation_id.0, turn_id.0));
        let req = match Ph1mThreadDigestUpsertRequest::v1(
            now,
            speaker_assertion,
            policy_context_ref,
            MemoryRetentionMode::Default,
            thread_digest,
            idempotency_key,
        ) {
            Ok(v) => v,
            Err(_) => return,
        };
        let input = match MemoryTurnInput::v1(
            correlation_id,
            turn_id,
            MemoryOperation::ThreadDigestUpsert(req),
        ) {
            Ok(v) => v,
            Err(_) => return,
        };
        let _ = self.memory.borrow_mut().run_turn_and_persist(store, &input);
    }

    fn enforce_capreq_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "CAPREQ_MANAGE",
            "simulation_candidate_dispatch.capreq.access_instance_id",
            "simulation_candidate_dispatch.capreq.access_decision",
            now,
        )
    }

    fn enforce_link_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "LINK_INVITE",
            "simulation_candidate_dispatch.link.access_instance_id",
            "simulation_candidate_dispatch.link.access_decision",
            now,
        )
    }

    fn enforce_access_schema_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_SCHEMA_MANAGE",
            "simulation_candidate_dispatch.access_schema.access_instance_id",
            "simulation_candidate_dispatch.access_schema.access_decision",
            now,
        )
    }

    fn enforce_access_escalation_vote_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_ESCALATION_VOTE",
            "simulation_candidate_dispatch.access_escalation_vote.access_instance_id",
            "simulation_candidate_dispatch.access_escalation_vote.access_decision",
            now,
        )
    }

    fn enforce_access_instance_compile_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        self.enforce_access_gate(
            store,
            actor_user_id,
            tenant_id,
            "ACCESS_INSTANCE_COMPILE_REFRESH",
            "simulation_candidate_dispatch.access_instance_compile.access_instance_id",
            "simulation_candidate_dispatch.access_instance_compile.access_decision",
            now,
        )
    }

    fn enforce_access_gate(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &TenantId,
        requested_action: &str,
        field_access_instance: &'static str,
        field_access_decision: &'static str,
        now: MonotonicTimeNs,
    ) -> Result<(), StorageError> {
        let Some(access_instance) =
            store.ph2access_get_instance_by_tenant_user(tenant_id.as_str(), actor_user_id)
        else {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_instance,
                    reason: "missing access instance for actor_user_id + tenant_id",
                },
            ));
        };

        let gate = store.ph1access_gate_decide(
            actor_user_id.clone(),
            access_instance.access_instance_id.clone(),
            requested_action.to_string(),
            AccessMode::A,
            access_instance.device_trust_level,
            false,
            now,
        )?;

        match gate.access_decision {
            AccessDecision::Allow => Ok(()),
            AccessDecision::Deny => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_decision,
                    reason: "ACCESS_SCOPE_VIOLATION",
                },
            )),
            AccessDecision::Escalate => Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: field_access_decision,
                    reason: "ACCESS_AP_REQUIRED",
                },
            )),
        }
    }
}

fn step_up_challenge_satisfied(
    verification_level: AccessVerificationLevel,
    challenge_method: StepUpChallengeMethod,
) -> bool {
    match challenge_method {
        StepUpChallengeMethod::DeviceBiometric => matches!(
            verification_level,
            AccessVerificationLevel::Biometric | AccessVerificationLevel::StepUp
        ),
        StepUpChallengeMethod::DevicePasscode => matches!(
            verification_level,
            AccessVerificationLevel::PasscodeTime
                | AccessVerificationLevel::Biometric
                | AccessVerificationLevel::StepUp
        ),
    }
}

fn is_legacy_link_delivery_simulation_id(simulation_id: &str) -> bool {
    matches!(
        simulation_id,
        "LINK_INVITE_SEND_COMMIT"
            | "LINK_INVITE_RESEND_COMMIT"
            | "LINK_DELIVERY_FAILURE_HANDLING_COMMIT"
    )
}

const MEMORY_ATOM_UPSERT_COMMIT: &str = "MEMORY_ATOM_UPSERT_COMMIT";
const MEMORY_FORGET_COMMIT: &str = "MEMORY_FORGET_COMMIT";
const MEMORY_RECALL_QUERY_COMMIT: &str = "MEMORY_RECALL_QUERY_COMMIT";
const ACCESS_BOARD_VOTE_COMMIT: &str = "ACCESS_BOARD_VOTE_COMMIT";

fn field_value<'a>(d: &'a IntentDraft, k: FieldKey) -> Option<&'a FieldValue> {
    d.fields.iter().find(|f| f.key == k).map(|f| &f.value)
}

fn optional_field_value<'a>(d: &'a IntentDraft, k: FieldKey) -> Option<&'a FieldValue> {
    field_value(d, k)
}

fn required_field_value<'a>(
    d: &'a IntentDraft,
    k: FieldKey,
) -> Result<&'a FieldValue, StorageError> {
    field_value(d, k).ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields",
            reason: "missing required field",
        })
    })
}

fn field_str(v: &FieldValue) -> &str {
    v.normalized_value
        .as_deref()
        .unwrap_or(v.original_span.as_str())
        .trim()
}

fn parse_invitee_type(
    v: &FieldValue,
) -> Result<selene_kernel_contracts::ph1link::InviteeType, StorageError> {
    use selene_kernel_contracts::ph1link::InviteeType::*;
    let s = field_str(v).to_ascii_lowercase();
    match s.as_str() {
        "company" => Ok(Company),
        "customer" => Ok(Customer),
        "employee" => Ok(Employee),
        "family_member" | "familymember" => Ok(FamilyMember),
        "friend" => Ok(Friend),
        "associate" => Ok(Associate),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.invitee_type",
                reason:
                    "must be one of: company, customer, employee, family_member, friend, associate",
            },
        )),
    }
}

fn parse_tenant_id(v: &FieldValue) -> Result<TenantId, StorageError> {
    TenantId::new(field_str(v).to_string()).map_err(StorageError::ContractViolation)
}

fn infer_tenant_scope_from_user_id(actor_user_id: &UserId) -> Option<String> {
    let (tenant_scope, _) = actor_user_id.as_str().split_once(':')?;
    let tenant_scope = tenant_scope.trim();
    if tenant_scope.is_empty() {
        return None;
    }
    Some(tenant_scope.to_string())
}

fn resolve_reminder_tenant_id(
    store: &Ph1fStore,
    d: &IntentDraft,
    actor_user_id: &UserId,
) -> Result<TenantId, StorageError> {
    if let Some(v) = optional_field_value(d, FieldKey::TenantId) {
        return parse_tenant_id(v);
    }

    resolve_runtime_scope_tenant_id(store, actor_user_id)
}

fn resolve_runtime_scope_tenant_id(
    store: &Ph1fStore,
    actor_user_id: &UserId,
) -> Result<TenantId, StorageError> {
    if let Some((tenant_scope, _local_user)) = actor_user_id.as_str().split_once(':') {
        if !tenant_scope.trim().is_empty() {
            return TenantId::new(tenant_scope.to_string())
                .map_err(StorageError::ContractViolation);
        }
    }

    let mut inferred: Option<String> = None;
    for ((tenant_id, user_id), _row) in store.ph2access_instance_rows() {
        if user_id == actor_user_id {
            if let Some(existing) = &inferred {
                if existing != tenant_id {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "simulation_candidate_dispatch.intent_draft.fields.tenant_id",
                            reason: "missing tenant_id and actor maps to multiple tenants",
                        },
                    ));
                }
            } else {
                inferred = Some(tenant_id.clone());
            }
        }
    }

    let tenant = inferred.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.tenant_id",
            reason: "missing tenant_id and no tenant could be inferred",
        })
    })?;

    TenantId::new(tenant).map_err(StorageError::ContractViolation)
}

fn resolve_invite_tenant_id(
    store: &Ph1fStore,
    d: &IntentDraft,
    actor_user_id: &UserId,
) -> Result<TenantId, StorageError> {
    // Runtime scope is authoritative for invite dispatch. Transcript-derived tenant hints never override it.
    let runtime_tenant_id = resolve_runtime_scope_tenant_id(store, actor_user_id)?;
    if let Some(v) = optional_field_value(d, FieldKey::TenantId) {
        if let Ok(tenant_hint) = parse_tenant_id(v) {
            if tenant_hint == runtime_tenant_id {
                return Ok(runtime_tenant_id);
            }
        }
    }
    Ok(runtime_tenant_id)
}

fn simulation_catalog_guard_target_v1(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    d: &IntentDraft,
) -> Result<(TenantId, SimulationId), StorageError> {
    let tenant_id = match d.intent_type {
        IntentType::CreateInviteLink => resolve_invite_tenant_id(store, d, actor_user_id)?,
        IntentType::CapreqManage
        | IntentType::AccessSchemaManage
        | IntentType::AccessEscalationVote
        | IntentType::AccessInstanceCompileRefresh => {
            parse_tenant_id(required_field_value(d, FieldKey::TenantId)?)?
        }
        _ => resolve_reminder_tenant_id(store, d, actor_user_id)?,
    };
    let simulation_id = simulation_id_for_intent_draft_v1(d)?;
    let simulation_id =
        SimulationId::new(simulation_id.to_string()).map_err(StorageError::ContractViolation)?;
    Ok((tenant_id, simulation_id))
}

fn simulation_id_for_intent_draft_v1(d: &IntentDraft) -> Result<&'static str, StorageError> {
    match d.intent_type {
        IntentType::SetReminder => Ok(REMINDER_SCHEDULE_COMMIT),
        IntentType::MemoryRememberRequest => Ok(MEMORY_ATOM_UPSERT_COMMIT),
        IntentType::MemoryForgetRequest => Ok(MEMORY_FORGET_COMMIT),
        IntentType::MemoryQuery => Ok(MEMORY_RECALL_QUERY_COMMIT),
        IntentType::CreateInviteLink => Ok(LINK_INVITE_GENERATE_DRAFT),
        IntentType::CapreqManage => {
            match parse_capreq_action(optional_field_value(d, FieldKey::CapreqAction))? {
                CapabilityRequestAction::CreateDraft => Ok(CAPREQ_CREATE_DRAFT),
                CapabilityRequestAction::SubmitForApproval => Ok(CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT),
                CapabilityRequestAction::Approve => Ok(CAPREQ_APPROVE_COMMIT),
                CapabilityRequestAction::Reject => Ok(CAPREQ_REJECT_COMMIT),
                CapabilityRequestAction::Fulfill => Ok(CAPREQ_FULFILL_COMMIT),
                CapabilityRequestAction::Cancel => Ok(CAPREQ_CANCEL_REVOKE),
            }
        }
        IntentType::AccessSchemaManage => {
            match parse_access_ap_action(required_field_value(d, FieldKey::ApAction)?)? {
                AccessApAction::CreateDraft => Ok(ACCESS_AP_SCHEMA_CREATE_DRAFT),
                AccessApAction::Update => Ok(ACCESS_AP_SCHEMA_UPDATE_COMMIT),
                AccessApAction::Activate => Ok(ACCESS_AP_SCHEMA_ACTIVATE_COMMIT),
                AccessApAction::Retire => Ok(ACCESS_AP_SCHEMA_RETIRE_COMMIT),
            }
        }
        IntentType::AccessEscalationVote => Ok(ACCESS_BOARD_VOTE_COMMIT),
        IntentType::AccessInstanceCompileRefresh => Ok(ACCESS_INSTANCE_COMPILE_COMMIT),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.intent_type",
                reason: "SIM_DISPATCH_GUARD_SIMULATION_ID_INVALID",
            },
        )),
    }
}

fn derive_memory_key_from_subject(subject: &str) -> Result<MemoryKey, StorageError> {
    let mut base = String::new();
    let mut prev_underscore = false;
    for ch in subject.chars() {
        if ch.is_ascii_alphanumeric() {
            base.push(ch.to_ascii_lowercase());
            prev_underscore = false;
        } else if !prev_underscore {
            base.push('_');
            prev_underscore = true;
        }
    }
    let base = base.trim_matches('_');
    let key_base = if base.is_empty() {
        "memory_item"
    } else if base.len() > 48 {
        &base[..48]
    } else {
        base
    };
    let key = format!("{key_base}_{}", short_hash_hex(&[subject]));
    MemoryKey::new(key).map_err(StorageError::ContractViolation)
}

fn new_ph1m_wiring() -> Ph1mWiring<Ph1mRuntime> {
    Ph1mWiring::new(
        Ph1mWiringConfig::mvp_v1(true),
        Ph1mRuntime::new(Ph1mConfig::mvp_v1()),
    )
    .expect("PH1.M wiring mvp_v1 config must be valid")
}

fn new_os_top_level_wiring() -> Ph1OsTopLevelWiring<EngineBackedOsRuntime> {
    let engine = EngineBackedOsRuntime {
        runtime: EngineOsRuntime::new(EngineOsConfig::mvp_v1()),
    };
    let os_wiring = Ph1OsWiring::new(Ph1OsWiringConfig::mvp_v1(true), engine)
        .expect("PH1.OS wiring mvp_v1 config must be valid");
    Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring)
        .expect("PH1.OS top-level mvp_v1 config must be valid")
}

#[derive(Debug, Clone)]
struct ResolvedVoiceIdentity {
    assertion: Ph1VoiceIdResponse,
    identity_prompt_scope_key: Option<String>,
}

fn resolve_voice_identity_assertion(
    os_top_level: &Ph1OsTopLevelWiring<EngineBackedOsRuntime>,
    voice_id_live: &Ph1VoiceIdLiveRuntime,
    store: &mut Ph1fStore,
    actor_user_id: &UserId,
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> Result<ResolvedVoiceIdentity, StorageError> {
    let mut enrolled: Vec<EngineEnrolledSpeaker> = Vec::new();
    let mut actor_primary: Option<u64> = None;
    let mut actor_secondary: Option<u64> = None;
    let mut actor_primary_embedding_capture_ref: Option<(u64, VoiceEmbeddingCaptureRef)> = None;
    let mut actor_app_platform: Option<(u64, AppPlatform)> = None;
    let mut actor_tenant_id: Option<(u64, String)> = None;
    let mut actor_feedback_device_id: Option<(u64, DeviceId)> = None;
    for profile in store.ph1vid_voice_profile_rows() {
        let Some(device) = store.get_device(&profile.device_id) else {
            continue;
        };
        let profile_seed = stable_seed_u64(&[
            profile.voice_profile_id.as_str(),
            profile.device_id.as_str(),
            device.user_id.as_str(),
        ]);
        if device.user_id == *actor_user_id {
            if actor_primary.is_none() {
                actor_primary = Some(profile_seed);
            } else if actor_secondary.is_none() && actor_primary != Some(profile_seed) {
                actor_secondary = Some(profile_seed);
            }
            if let Some(onboarding_session) =
                store.ph1onb_session_row(&profile.onboarding_session_id)
            {
                let should_replace = actor_app_platform
                    .as_ref()
                    .map(|(created_at_ns, _)| profile.created_at.0 >= *created_at_ns)
                    .unwrap_or(true);
                if should_replace {
                    actor_app_platform =
                        Some((profile.created_at.0, onboarding_session.app_platform));
                    actor_tenant_id = onboarding_session
                        .tenant_id
                        .as_ref()
                        .map(|tenant_id| (profile.created_at.0, tenant_id.clone()));
                    actor_feedback_device_id =
                        Some((profile.created_at.0, profile.device_id.clone()));
                    actor_primary_embedding_capture_ref = profile
                        .profile_embedding_capture_ref
                        .as_ref()
                        .map(|capture_ref| (profile.created_at.0, capture_ref.clone()));
                }
            }
        }
        let speaker_id = SpeakerId::new(format!(
            "spk_{}",
            short_hash_hex(&[profile.voice_profile_id.as_str(), device.user_id.as_str()])
        ))
        .map_err(StorageError::ContractViolation)?;
        let profile_embedding = profile
            .profile_embedding_capture_ref
            .as_ref()
            .map(profile_embedding_from_capture_ref)
            .or_else(|| Some(simulation_profile_embedding_from_seed(profile_seed)));
        enrolled.push(EngineEnrolledSpeaker {
            speaker_id,
            user_id: Some(device.user_id.clone()),
            fingerprint: profile_seed,
            profile_embedding,
        });
    }

    let req = build_voice_id_request_for_actor(
        now,
        actor_user_id,
        actor_primary_embedding_capture_ref
            .as_ref()
            .map(|(_, capture_ref)| capture_ref.clone()),
    )?;
    let request_primary_embedding = req
        .app_primary_embedding_capture_ref
        .as_ref()
        .map(profile_embedding_from_capture_ref);
    let primary_fingerprint = if request_primary_embedding.is_some() {
        None
    } else {
        actor_primary
    };
    let obs = EngineVoiceIdObservation {
        // Fail closed: never synthesize a fingerprint from user identity alone.
        // Runtime matching must rely on enrolled voice profiles.
        primary_fingerprint,
        secondary_fingerprint: actor_secondary,
        primary_embedding: request_primary_embedding
            .or_else(|| actor_primary.map(simulation_profile_embedding_from_seed)),
        secondary_embedding: actor_secondary.map(simulation_profile_embedding_from_seed),
        spoof_risk: false,
    };
    let channel = if req.wake_event.is_some() {
        VoiceIdentityChannel::WakeWord
    } else {
        VoiceIdentityChannel::Explicit
    };
    let trigger = if channel == VoiceIdentityChannel::WakeWord {
        OsVoiceTrigger::WakeWord
    } else {
        OsVoiceTrigger::Explicit
    };
    let resolved_tenant_id = actor_tenant_id
        .as_ref()
        .map(|(_, tenant_id)| tenant_id.clone())
        .or_else(|| infer_tenant_scope_from_user_id(actor_user_id));
    let app_platform = actor_app_platform
        .as_ref()
        .map(|(_, app_platform)| *app_platform)
        .unwrap_or(AppPlatform::Ios);
    let top_level_turn_input = OsTopLevelTurnInput::v1(
        correlation_id,
        turn_id,
        OsTopLevelTurnPath::Voice,
        Some(OsVoiceTurnContext::v1(
            SimulationExecutor::os_voice_platform_from_app_platform(app_platform),
            trigger,
        )),
        SimulationExecutor::expected_always_on_sequence_for_voice(trigger),
        Vec::new(),
        0,
        SimulationExecutor::default_os_turn_input(correlation_id, turn_id)?,
    )
    .map_err(StorageError::ContractViolation)?;
    let live_turn_input = OsVoiceLiveTurnInput::v1(
        top_level_turn_input,
        req,
        actor_user_id.clone(),
        resolved_tenant_id,
        actor_feedback_device_id.map(|(_, device_id)| device_id),
        enrolled,
        obs,
    )
    .map_err(StorageError::ContractViolation)?;
    let runtime = Ph1OsVoiceLiveRuntime::new(os_top_level.clone(), voice_id_live.clone());
    let outcome = runtime.run_turn(store, live_turn_input)?;
    match outcome {
        OsVoiceLiveTurnOutcome::Forwarded(bundle) => Ok(ResolvedVoiceIdentity {
            assertion: bundle.voice_identity_assertion,
            identity_prompt_scope_key: bundle.identity_prompt_scope_key,
        }),
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_executor.voice_live_runtime",
                reason: "voice live runtime is disabled",
            },
        )),
        OsVoiceLiveTurnOutcome::Refused(_) => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_executor.voice_live_runtime",
                reason: "voice live runtime refused the request",
            },
        )),
    }
}

fn build_voice_id_request_for_actor(
    now: MonotonicTimeNs,
    actor_user_id: &UserId,
    app_primary_embedding_capture_ref: Option<VoiceEmbeddingCaptureRef>,
) -> Result<Ph1VoiceIdRequest, StorageError> {
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
        Confidence::new(0.95).map_err(StorageError::ContractViolation)?,
        SpeechLikeness::new(0.95).map_err(StorageError::ContractViolation)?,
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
    let device_id = AudioDeviceId::new(format!(
        "sim_mic_{}",
        short_hash_hex(&[actor_user_id.as_str()])
    ))
    .map_err(StorageError::ContractViolation)?;
    Ph1VoiceIdRequest::v1_with_risk_signals_and_embedding_capture(
        now,
        processed_stream_ref,
        vad_events,
        device_id,
        session_snapshot,
        None,
        false,
        DeviceTrustLevel::Trusted,
        Some(actor_user_id.clone()),
        vec![],
        app_primary_embedding_capture_ref,
    )
    .map_err(StorageError::ContractViolation)
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

fn build_memory_thread_digest(
    now: MonotonicTimeNs,
    correlation_id: CorrelationId,
    turn_id: TurnId,
    d: &IntentDraft,
) -> Result<selene_kernel_contracts::ph1m::MemoryThreadDigest, ContractViolation> {
    let thread_id = format!(
        "thread_{}_{}",
        intent_type_token(d.intent_type),
        short_hash_hex(&[
            intent_type_token(d.intent_type),
            &correlation_id.0.to_string(),
            &turn_id.0.to_string(),
        ])
    );
    let mut summary = vec![format!("intent: {}", intent_type_token(d.intent_type))];
    for field in d.fields.iter().take(2) {
        summary.push(format!(
            "{}: {}",
            field_key_token(field.key),
            truncate_ascii(field_str(&field.value), 120)
        ));
        if summary.len() == 3 {
            break;
        }
    }
    let title = if let Some(first) = d.fields.first() {
        format!(
            "{} - {}",
            intent_type_token(d.intent_type),
            truncate_ascii(field_str(&first.value), 64)
        )
    } else {
        format!("{} turn", intent_type_token(d.intent_type))
    };
    selene_kernel_contracts::ph1m::MemoryThreadDigest::v1(
        thread_id,
        title,
        summary,
        false,
        !d.required_fields_missing.is_empty(),
        now,
        1,
    )
}

fn intent_type_token(intent: IntentType) -> &'static str {
    match intent {
        IntentType::SetReminder => "SET_REMINDER",
        IntentType::MemoryRememberRequest => "MEMORY_REMEMBER_REQUEST",
        IntentType::MemoryForgetRequest => "MEMORY_FORGET_REQUEST",
        IntentType::MemoryQuery => "MEMORY_QUERY",
        IntentType::CreateInviteLink => "CREATE_INVITE_LINK",
        IntentType::CapreqManage => "CAPREQ_MANAGE",
        IntentType::AccessSchemaManage => "ACCESS_SCHEMA_MANAGE",
        IntentType::AccessEscalationVote => "ACCESS_ESCALATION_VOTE",
        IntentType::AccessInstanceCompileRefresh => "ACCESS_INSTANCE_COMPILE_REFRESH",
        IntentType::CreateCalendarEvent => "CREATE_CALENDAR_EVENT",
        IntentType::BookTable => "BOOK_TABLE",
        IntentType::SendMoney => "SEND_MONEY",
        IntentType::TimeQuery => "TIME_QUERY",
        IntentType::WeatherQuery => "WEATHER_QUERY",
        IntentType::Continue => "CONTINUE",
        IntentType::MoreDetail => "MORE_DETAIL",
    }
}

fn field_key_token(key: FieldKey) -> &'static str {
    match key {
        FieldKey::When => "when",
        FieldKey::Task => "task",
        FieldKey::Person => "person",
        FieldKey::Place => "place",
        FieldKey::PartySize => "party_size",
        FieldKey::Amount => "amount",
        FieldKey::Recipient => "recipient",
        FieldKey::InviteeType => "invitee_type",
        FieldKey::DeliveryMethod => "delivery_method",
        FieldKey::RecipientContact => "recipient_contact",
        FieldKey::TenantId => "tenant_id",
        FieldKey::RequestedCapabilityId => "requested_capability_id",
        FieldKey::TargetScopeRef => "target_scope_ref",
        FieldKey::Justification => "justification",
        FieldKey::CapreqAction => "capreq_action",
        FieldKey::CapreqId => "capreq_id",
        FieldKey::AccessProfileId => "access_profile_id",
        FieldKey::SchemaVersionId => "schema_version_id",
        FieldKey::ApScope => "ap_scope",
        FieldKey::ApAction => "ap_action",
        FieldKey::ProfilePayloadJson => "profile_payload_json",
        FieldKey::AccessReviewChannel => "access_review_channel",
        FieldKey::AccessRuleAction => "access_rule_action",
        FieldKey::EscalationCaseId => "escalation_case_id",
        FieldKey::BoardPolicyId => "board_policy_id",
        FieldKey::TargetUserId => "target_user_id",
        FieldKey::AccessInstanceId => "access_instance_id",
        FieldKey::VoteAction => "vote_action",
        FieldKey::VoteValue => "vote_value",
        FieldKey::OverrideResult => "override_result",
        FieldKey::PositionId => "position_id",
        FieldKey::OverlayIdList => "overlay_id_list",
        FieldKey::CompileReason => "compile_reason",
        FieldKey::IntentChoice => "intent_choice",
        FieldKey::ReferenceTarget => "reference_target",
    }
}

fn truncate_ascii(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    value[..max_len].to_string()
}

fn truncate_utf8(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessApAction {
    CreateDraft,
    Update,
    Activate,
    Retire,
}

fn parse_access_ap_action(v: &FieldValue) -> Result<AccessApAction, StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "CREATE_DRAFT" => Ok(AccessApAction::CreateDraft),
        "UPDATE" => Ok(AccessApAction::Update),
        "ACTIVATE" => Ok(AccessApAction::Activate),
        "RETIRE" => Ok(AccessApAction::Retire),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.ap_action",
                reason: "must be one of: CREATE_DRAFT, UPDATE, ACTIVATE, RETIRE",
            },
        )),
    }
}

fn parse_access_review_channel(v: &FieldValue) -> Result<(), StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "PHONE_DESKTOP" | "READ_OUT_LOUD" => Ok(()),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.access_review_channel",
                reason: "must be PHONE_DESKTOP or READ_OUT_LOUD",
            },
        )),
    }
}

fn parse_access_rule_action(v: &FieldValue) -> Result<(), StorageError> {
    let s = field_str(v).to_ascii_uppercase();
    match s.as_str() {
        "AGREE" | "DISAGREE" | "EDIT" | "DELETE" | "DISABLE" | "ADD_CUSTOM_RULE" => Ok(()),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.access_rule_action",
                reason: "must be one of: AGREE, DISAGREE, EDIT, DELETE, DISABLE, ADD_CUSTOM_RULE",
            },
        )),
    }
}

fn parse_capreq_action(v: Option<&FieldValue>) -> Result<CapabilityRequestAction, StorageError> {
    let Some(v) = v else {
        return Ok(CapabilityRequestAction::CreateDraft);
    };
    let s = field_str(v).to_ascii_lowercase();
    match s.as_str() {
        "create_draft" | "create" | "draft" => Ok(CapabilityRequestAction::CreateDraft),
        "submit_for_approval" | "submit" => Ok(CapabilityRequestAction::SubmitForApproval),
        "approve" => Ok(CapabilityRequestAction::Approve),
        "reject" => Ok(CapabilityRequestAction::Reject),
        "fulfill" | "fulfilled" => Ok(CapabilityRequestAction::Fulfill),
        "cancel" | "revoke" => Ok(CapabilityRequestAction::Cancel),
        _ => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.capreq_action",
                reason: "must be one of: create_draft, submit_for_approval, approve, reject, fulfill, cancel",
            },
        )),
    }
}

fn capreq_action_token(action: CapabilityRequestAction) -> &'static str {
    match action {
        CapabilityRequestAction::CreateDraft => "create_draft",
        CapabilityRequestAction::SubmitForApproval => "submit_for_approval",
        CapabilityRequestAction::Approve => "approve",
        CapabilityRequestAction::Reject => "reject",
        CapabilityRequestAction::Fulfill => "fulfill",
        CapabilityRequestAction::Cancel => "cancel",
    }
}

fn resolve_capreq_id(
    d: &IntentDraft,
    action: CapabilityRequestAction,
    tenant_id: &TenantId,
    requested_capability_id: Option<&str>,
    target_scope_ref: Option<&str>,
    justification: Option<&str>,
) -> Result<CapreqId, StorageError> {
    if let Some(v) = optional_field_value(d, FieldKey::CapreqId) {
        return CapreqId::new(field_str(v).to_string()).map_err(StorageError::ContractViolation);
    }

    match action {
        CapabilityRequestAction::CreateDraft => derive_capreq_id(
            tenant_id,
            requested_capability_id,
            target_scope_ref,
            justification,
        ),
        CapabilityRequestAction::SubmitForApproval => {
            if requested_capability_id.is_some()
                && target_scope_ref.is_some()
                && justification.is_some()
            {
                derive_capreq_id(
                    tenant_id,
                    requested_capability_id,
                    target_scope_ref,
                    justification,
                )
            } else {
                Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "simulation_candidate_dispatch.intent_draft.fields.capreq_id",
                        reason: "required for submit when request snapshot fields are missing",
                    },
                ))
            }
        }
        CapabilityRequestAction::Approve
        | CapabilityRequestAction::Reject
        | CapabilityRequestAction::Fulfill
        | CapabilityRequestAction::Cancel => Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.fields.capreq_id",
                reason: "required for lifecycle actions after draft creation",
            },
        )),
    }
}

fn derive_capreq_id(
    tenant_id: &TenantId,
    requested_capability_id: Option<&str>,
    target_scope_ref: Option<&str>,
    justification: Option<&str>,
) -> Result<CapreqId, StorageError> {
    let requested_capability_id = requested_capability_id.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.requested_capability_id",
            reason: "required",
        })
    })?;
    let target_scope_ref = target_scope_ref.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.target_scope_ref",
            reason: "required",
        })
    })?;
    let justification = justification.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "simulation_candidate_dispatch.intent_draft.fields.justification",
            reason: "required",
        })
    })?;

    CapreqId::new(format!(
        "capreq_{}",
        short_hash_hex(&[
            tenant_id.as_str(),
            requested_capability_id,
            target_scope_ref,
            justification,
        ])
    ))
    .map_err(StorageError::ContractViolation)
}

fn required_capreq_snapshot_field(
    value: Option<String>,
    field: &'static str,
) -> Result<String, StorageError> {
    let value = value.ok_or_else(|| {
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field,
            reason: "required",
        })
    })?;
    if value.trim().is_empty() {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field,
                reason: "required",
            },
        ));
    }
    Ok(value)
}

fn validate_nonempty_bounded_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), StorageError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field,
                reason: "required",
            },
        ));
    }
    if value.len() > max_len {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field,
                reason: "exceeds max length",
            },
        ));
    }
    Ok(())
}

fn resolve_followup_delivery_refs(
    policy_gate_ok: bool,
    followup_delivery_hint: BcastFollowupDeliveryHint,
    followup_source: &str,
    broadcast_id: &str,
    recipient_id: &str,
    simulation_id: &str,
) -> (
    BcastFollowupDeliveryMode,
    Option<BcastFollowupTextOnlyReason>,
    Option<String>,
    Option<String>,
) {
    let (delivery_mode, text_only_reason) = match followup_delivery_hint {
        BcastFollowupDeliveryHint::VoiceDefault => (BcastFollowupDeliveryMode::Voice, None),
        BcastFollowupDeliveryHint::TextOnly(reason) => {
            (BcastFollowupDeliveryMode::Text, Some(reason))
        }
    };
    if !policy_gate_ok {
        return (delivery_mode, text_only_reason, None, None);
    }

    let ref_suffix = short_hash_hex(&[followup_source, broadcast_id, recipient_id, simulation_id]);
    match delivery_mode {
        BcastFollowupDeliveryMode::Voice => (
            delivery_mode,
            None,
            Some(format!("bcast_followup_{ref_suffix}")),
            None,
        ),
        BcastFollowupDeliveryMode::Text => (
            delivery_mode,
            text_only_reason,
            None,
            Some(format!("bcast_followup_text_{ref_suffix}")),
        ),
    }
}

fn short_hash_hex(parts: &[&str]) -> String {
    // FNV-1a 64-bit; deterministic and bounded for id/payload derivation.
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for part in parts {
        for &b in part.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(PRIME);
        }
        // Stable delimiter to avoid accidental concatenation ambiguity.
        h ^= b'|' as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse, VoiceEnrollmentSessionId,
        VoiceIdEnrollCompleteCommitRequest, VoiceIdEnrollSampleCommitRequest,
        VoiceIdEnrollStartDraftRequest, VoiceIdSimulationRequest, VoiceIdSimulationType,
        PH1VOICEID_SIM_CONTRACT_VERSION, VOICE_ID_ENROLL_COMPLETE_COMMIT,
        VOICE_ID_ENROLL_SAMPLE_COMMIT, VOICE_ID_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1bcast::{
        BcastAckCommitRequest, BcastAckStatus, BcastDeferCommitRequest, BcastDeliveryMethod,
        BcastDraftCreateRequest, BcastEscalateCommitRequest, BcastOutcome, BcastRecipientRegion,
        BcastRecipientState, BcastReminderFiredCommitRequest, BcastRequest, BcastSimulationType,
        BroadcastClassification, BroadcastRecipientId, Ph1BcastRequest, Ph1BcastResponse,
        BCAST_ACK_COMMIT, BCAST_CREATE_DRAFT, BCAST_DEFER_COMMIT, BCAST_DELIVER_COMMIT,
        BCAST_ESCALATE_COMMIT, BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS, BCAST_REMINDER_FIRED_COMMIT,
        PH1BCAST_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1capreq::{
        CapabilityRequestAction, CapabilityRequestStatus, CapreqId,
    };
    use selene_kernel_contracts::ph1delivery::{DeliveryOutcome, Ph1DeliveryResponse};
    use selene_kernel_contracts::ph1j::{AuditEngine, DeviceId, PayloadKey};
    use selene_kernel_contracts::ph1link::InviteeType;
    use selene_kernel_contracts::ph1n::{IntentField, OverallConfidence, SensitivityLevel};
    use selene_kernel_contracts::ph1position::{
        Ph1PositionRequest, Ph1PositionResponse, PositionCreateDraftRequest, PositionRequest,
        PositionScheduleType, PositionSimulationType, PH1POSITION_CONTRACT_VERSION,
        POSITION_SIM_001_CREATE_DRAFT,
    };
    use selene_kernel_contracts::ph1rem::{Ph1RemResponse, ReminderPriorityLevel};
    use selene_kernel_contracts::ph1simcat::{
        SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType,
        SimulationVersion,
    };
    use selene_kernel_contracts::ph1w::{
        Ph1wRequest, Ph1wResponse, WakeEnrollStartDraftRequest, WakeRequest, WakeSimulationType,
        PH1W_CONTRACT_VERSION, WAKE_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1x::{
        DeliveryHint, DispatchDirective, Ph1xDirective, Ph1xResponse, StepUpActionClass,
        StepUpChallengeMethod, StepUpOutcome, ThreadState,
    };
    use selene_kernel_contracts::{ReasonCodeId, SchemaVersion};
    use selene_storage::ph1f::{
        AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessVerificationLevel,
        DeviceRecord, IdentityRecord, IdentityStatus, MemoryThreadEventKind,
        MobileArtifactSyncState, TenantCompanyLifecycleState, TenantCompanyRecord,
    };

    fn capreq_field(key: FieldKey, value: &str) -> IntentField {
        IntentField {
            key,
            value: FieldValue::verbatim(value.to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }
    }

    fn capreq_draft(fields: Vec<IntentField>) -> IntentDraft {
        IntentDraft::v1(
            IntentType::CapreqManage,
            SchemaVersion(1),
            fields,
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

    fn reminder_draft(task: &str, when: &str) -> IntentDraft {
        IntentDraft::v1(
            IntentType::SetReminder,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::Task,
                    value: FieldValue::verbatim(task.to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::When,
                    value: FieldValue::verbatim(when.to_string()).unwrap(),
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

    fn memory_draft(intent_type: IntentType, task: Option<&str>) -> IntentDraft {
        let mut fields = Vec::new();
        if let Some(task) = task {
            fields.push(IntentField {
                key: FieldKey::Task,
                value: FieldValue::verbatim(task.to_string()).unwrap(),
                confidence: OverallConfidence::High,
            });
        }
        IntentDraft::v1(
            intent_type,
            SchemaVersion(1),
            fields,
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            matches!(intent_type, IntentType::MemoryForgetRequest),
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn simulation_executor_with_memory_enabled(memory_enabled: bool) -> SimulationExecutor {
        SimulationExecutor {
            memory: std::cell::RefCell::new(
                Ph1mWiring::new(
                    Ph1mWiringConfig::mvp_v1(memory_enabled),
                    Ph1mRuntime::new(Ph1mConfig::mvp_v1()),
                )
                .expect("PH1.M test wiring config must be valid"),
            ),
            ..SimulationExecutor::default()
        }
    }

    fn seed_locked_voice_profile_for_actor(
        store: &mut Ph1fStore,
        exec: &SimulationExecutor,
        actor: &UserId,
        tenant_id: &str,
    ) {
        let actor_hash = short_hash_hex(&[actor.as_str()]);
        let device_id = DeviceId::new(format!("voice-device-{actor_hash}")).unwrap();
        let device_fp = format!("voice-device-fp-{actor_hash}");
        let app_instance_id = "legacy_app_instance".to_string();
        let deep_link_nonce = "legacy_nonce_4".to_string();
        let link_opened_at = MonotonicTimeNs(4);
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    Some(format!("audio_prof_{actor_hash}")),
                )
                .unwrap(),
            )
            .unwrap();
        let (link_rec, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(3),
                actor.clone(),
                InviteeType::Employee,
                Some(tenant_id.to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        let _ = store
            .ph1link_invite_open_activate_commit(
                link_opened_at,
                link_rec.token_id.clone(),
                device_fp.clone(),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(5),
                link_rec.token_id,
                None,
                Some(tenant_id.to_string()),
                device_fp,
                selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id,
                deep_link_nonce,
                link_opened_at,
            )
            .unwrap();
        let start = exec
            .execute_voice_id(
                store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1900),
                    turn_id: TurnId(1),
                    now: MonotonicTimeNs(10),
                    simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
                    simulation_type: VoiceIdSimulationType::Draft,
                    request: VoiceIdSimulationRequest::EnrollStartDraft(
                        VoiceIdEnrollStartDraftRequest {
                            onboarding_session_id: onb.onboarding_session_id.as_str().to_string(),
                            device_id: device_id.clone(),
                            consent_asserted: true,
                            max_total_attempts: 8,
                            max_session_enroll_time_ms: 120_000,
                            lock_after_consecutive_passes: 2,
                        },
                    ),
                },
            )
            .unwrap();
        let voice_enrollment_session_id = match start {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                ok.enroll_start_result
                    .expect("voice enrollment start result must exist")
                    .voice_enrollment_session_id
            }
            _ => panic!("expected voice enrollment start OK"),
        };
        for attempt_index in 1..=2u16 {
            let _ = exec
                .execute_voice_id(
                    store,
                    &Ph1VoiceIdSimRequest {
                        schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                        correlation_id: CorrelationId(1900),
                        turn_id: TurnId(1 + u64::from(attempt_index)),
                        now: MonotonicTimeNs(10 + u64::from(attempt_index)),
                        simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
                        simulation_type: VoiceIdSimulationType::Commit,
                        request: VoiceIdSimulationRequest::EnrollSampleCommit(
                            VoiceIdEnrollSampleCommitRequest {
                                voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                    voice_enrollment_session_id.as_str(),
                                )
                                .unwrap(),
                                audio_sample_ref: format!(
                                    "audio:seed:{actor_hash}:{attempt_index}"
                                ),
                                attempt_index,
                                sample_duration_ms: 1_350,
                                vad_coverage: 0.93,
                                snr_db: 18.0,
                                clipping_pct: 0.4,
                                overlap_ratio: 0.0,
                                app_embedding_capture_ref: None,
                                idempotency_key: format!(
                                    "seed-voice-sample-{actor_hash}-{attempt_index}"
                                ),
                            },
                        ),
                    },
                )
                .unwrap();
        }
        let _ = exec
            .execute_voice_id(
                store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1900),
                    turn_id: TurnId(9),
                    now: MonotonicTimeNs(20),
                    simulation_id: VOICE_ID_ENROLL_COMPLETE_COMMIT.to_string(),
                    simulation_type: VoiceIdSimulationType::Commit,
                    request: VoiceIdSimulationRequest::EnrollCompleteCommit(
                        VoiceIdEnrollCompleteCommitRequest {
                            voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                voice_enrollment_session_id.as_str(),
                            )
                            .unwrap(),
                            idempotency_key: format!("seed-voice-complete-{actor_hash}"),
                        },
                    ),
                },
            )
            .unwrap();
    }

    fn capreq_x(turn_id: u64, draft: IntentDraft, idempotency_key: &str) -> Ph1xResponse {
        Ph1xResponse::v1(
            10,
            turn_id,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some(idempotency_key.to_string()),
        )
        .unwrap()
    }

    fn run_capreq(
        exec: &SimulationExecutor,
        store: &mut Ph1fStore,
        actor: &UserId,
        turn_id: u64,
        now: u64,
        idempotency_key: &str,
        fields: Vec<IntentField>,
    ) -> SimulationDispatchOutcome {
        let tenant = fields
            .iter()
            .find(|f| f.key == FieldKey::TenantId)
            .map(|f| field_str(&f.value).to_string())
            .expect("capreq tests require FieldKey::TenantId");
        seed_capreq_access_instance(store, actor, &tenant);

        let x = capreq_x(turn_id, capreq_draft(fields), idempotency_key);
        exec.execute_ph1x_dispatch_simulation_candidate(
            store,
            actor.clone(),
            MonotonicTimeNs(now),
            &x,
        )
        .unwrap()
    }

    fn seed_capreq_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        seed_capreq_access_instance_with(
            store,
            actor,
            tenant,
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );
    }

    fn seed_capreq_access_instance_with(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        seed_access_instance_with_permissions(
            store,
            actor,
            tenant,
            "role.capreq_manager",
            "{\"allow\":[\"CAPREQ_MANAGE\"]}",
            effective_access_mode,
            identity_verified,
            device_trust_level,
            lifecycle_state,
        );
    }

    fn seed_access_instance_with_permissions(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        role_template_id: &str,
        baseline_permissions_json: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                role_template_id.to_string(),
                effective_access_mode,
                baseline_permissions_json.to_string(),
                identity_verified,
                AccessVerificationLevel::PasscodeTime,
                device_trust_level,
                lifecycle_state,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_link_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        seed_link_access_instance_with(
            store,
            actor,
            tenant,
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );
    }

    fn seed_link_access_instance_with(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
        effective_access_mode: AccessMode,
        identity_verified: bool,
        device_trust_level: AccessDeviceTrustLevel,
        lifecycle_state: AccessLifecycleState,
    ) {
        seed_access_instance_with_permissions(
            store,
            actor,
            tenant,
            "role.link_inviter",
            "{\"allow\":[\"LINK_INVITE\"]}",
            effective_access_mode,
            identity_verified,
            device_trust_level,
            lifecycle_state,
        );
    }

    fn seed_simulation_catalog_status(
        store: &mut Ph1fStore,
        tenant: &str,
        simulation_id: &str,
        simulation_type: SimulationType,
        status: SimulationStatus,
    ) {
        let tenant_id = TenantId::new(tenant.to_string()).unwrap();
        let simulation_id = SimulationId::new(simulation_id.to_string()).unwrap();
        let event = SimulationCatalogEventInput::v1(
            MonotonicTimeNs(1),
            tenant_id,
            simulation_id,
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

    fn access_field(key: FieldKey, value: &str) -> IntentField {
        IntentField {
            key,
            value: FieldValue::verbatim(value.to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }
    }

    fn access_draft(intent_type: IntentType, fields: Vec<IntentField>) -> IntentDraft {
        IntentDraft::v1(
            intent_type,
            SchemaVersion(1),
            fields,
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

    fn access_x(turn_id: u64, draft: IntentDraft, idempotency_key: &str) -> Ph1xResponse {
        Ph1xResponse::v1(
            10,
            turn_id,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some(idempotency_key.to_string()),
        )
        .unwrap()
    }

    fn access_step_up_x(
        turn_id: u64,
        draft: IntentDraft,
        requested_action: &str,
        challenge_method: StepUpChallengeMethod,
        idempotency_key: &str,
    ) -> Ph1xResponse {
        Ph1xResponse::v1(
            10,
            turn_id,
            Ph1xDirective::Dispatch(
                DispatchDirective::access_step_up_v1(
                    draft,
                    StepUpActionClass::AccessGovernance,
                    requested_action.to_string(),
                    challenge_method,
                )
                .unwrap(),
            ),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some(idempotency_key.to_string()),
        )
        .unwrap()
    }

    #[test]
    fn at_sim_exec_01_ph1x_sim_candidate_create_invite_link_runs_ph1link_generate_draft() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::normalized("employee".to_string(), "employee".to_string())
                        .unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            22,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(123), &x)
            .unwrap();

        match out {
            SimulationDispatchOutcome::Link(r) => match r {
                Ph1LinkResponse::Ok(ok) => {
                    assert_eq!(ok.simulation_id, "LINK_INVITE_GENERATE_DRAFT");
                    assert!(ok.link_generate_result.is_some());
                }
                Ph1LinkResponse::Refuse(_) => panic!("expected ok"),
            },
            _ => panic!("expected link outcome"),
        }
    }

    #[test]
    fn run2_send_link_tenant_infers_scope_in_dispatch() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:inviter-tenant-infer-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![IntentField {
                key: FieldKey::InviteeType,
                value: FieldValue::normalized("associate".to_string(), "associate".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            }],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Private,
            true,
            vec![],
            vec![],
        )
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            23,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-tenant-infer-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(123), &x)
            .unwrap();

        match out {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => {
                assert_eq!(ok.simulation_id, "LINK_INVITE_GENERATE_DRAFT");
                assert!(ok.link_generate_result.is_some());
            }
            _ => panic!("expected link outcome"),
        }
    }

    #[test]
    fn run2_send_link_tenant_hint_is_non_authoritative_in_dispatch() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:inviter-tenant-hint-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

        let draft = IntentDraft::v1(
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
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_999".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            24,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-tenant-hint-nonauthoritative-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(123), &x)
            .unwrap();

        match out {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => {
                assert_eq!(ok.simulation_id, "LINK_INVITE_GENERATE_DRAFT");
                assert!(ok.link_generate_result.is_some());
            }
            _ => panic!("expected link outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01_ph1x_sim_candidate_create_invite_link_requires_active_simulation_catalog() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();
        let actor = UserId::new("inviter-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::normalized("employee".to_string(), "employee".to_string())
                        .unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
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
        .unwrap();
        let x = Ph1xResponse::v1(
            10,
            22,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-guard-1".to_string()),
        )
        .unwrap();

        let missing = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor.clone(),
            MonotonicTimeNs(123),
            &x,
        );
        assert!(matches!(
            missing,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.simulation_id",
                    reason: "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                }
            ))
        ));

        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Disabled,
        );
        let inactive = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(124),
            &x,
        );
        assert!(matches!(
            inactive,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.simulation_id",
                    reason: "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                }
            ))
        ));
        assert_eq!(
            reason_codes::SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE,
            ReasonCodeId(0x5349_0004)
        );
    }

    #[test]
    fn at_sim_exec_01a_set_reminder_routes_to_ph1rem_schedule_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:user_1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_locked_voice_profile_for_actor(&mut store, &exec, &actor, "tenant_1");

        let draft = reminder_draft("file payroll", "in 5 minutes");
        let x = Ph1xResponse::v1(
            11,
            23,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-rem-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_000_000_000),
                &x,
            )
            .unwrap();

        match out {
            SimulationDispatchOutcome::Reminder(Ph1RemResponse::Ok(ok)) => {
                assert_eq!(ok.simulation_id, "REMINDER_SCHEDULE_COMMIT");
                assert_eq!(
                    ok.state,
                    selene_kernel_contracts::ph1rem::ReminderState::Scheduled
                );
                assert!(ok.reminder_id.as_str().starts_with("rem_"));
            }
            _ => panic!("expected reminder outcome"),
        }
        let thread_rows = store.ph1m_thread_ledger_rows();
        assert_eq!(thread_rows.len(), 1);
        assert_eq!(
            thread_rows[0].event_kind,
            MemoryThreadEventKind::ThreadDigestUpsert
        );
        assert!(thread_rows[0].digest.thread_title.contains("SET_REMINDER"));
    }

    #[test]
    fn at_sim_exec_01g_memory_remember_dispatch_persists_memory_rows() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:user_memory").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_locked_voice_profile_for_actor(&mut store, &exec, &actor, "tenant_1");

        let draft = memory_draft(
            IntentType::MemoryRememberRequest,
            Some("Benji is my preferred name"),
        );
        let x = Ph1xResponse::v1(
            11,
            2401,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-remember-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_000_000_000),
                &x,
            )
            .unwrap();

        match out {
            SimulationDispatchOutcome::MemoryPropose(resp) => {
                assert!(!resp.decisions.is_empty());
                assert!(!resp.ledger_events.is_empty());
            }
            _ => panic!("expected memory propose outcome"),
        }
        assert_eq!(store.memory_ledger_rows().len(), 1);
        assert_eq!(store.memory_current().len(), 1);
    }

    #[test]
    fn at_sim_exec_01h_memory_forget_dispatch_removes_current_memory() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:user_memory_forget").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_locked_voice_profile_for_actor(&mut store, &exec, &actor, "tenant_1");

        let remember_draft = memory_draft(
            IntentType::MemoryRememberRequest,
            Some("Parking spot is B12"),
        );
        let remember_x = Ph1xResponse::v1(
            11,
            2402,
            Ph1xDirective::Dispatch(
                DispatchDirective::simulation_candidate_v1(remember_draft).unwrap(),
            ),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-remember-2".to_string()),
        )
        .unwrap();
        let remember_out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(123_100_000_000),
                &remember_x,
            )
            .unwrap();
        assert!(matches!(
            remember_out,
            SimulationDispatchOutcome::MemoryPropose(_)
        ));
        assert_eq!(store.memory_current().len(), 1);

        let forget_draft =
            memory_draft(IntentType::MemoryForgetRequest, Some("Parking spot is B12"));
        let forget_x = Ph1xResponse::v1(
            11,
            2403,
            Ph1xDirective::Dispatch(
                DispatchDirective::simulation_candidate_v1(forget_draft).unwrap(),
            ),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-forget-1".to_string()),
        )
        .unwrap();
        let forget_out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_200_000_000),
                &forget_x,
            )
            .unwrap();
        match forget_out {
            SimulationDispatchOutcome::MemoryForget(resp) => assert!(resp.forgotten),
            _ => panic!("expected memory forget outcome"),
        }
        assert_eq!(store.memory_current().len(), 1);
        let key = derive_memory_key_from_subject("Parking spot is B12").unwrap();
        let current = store
            .memory_current()
            .get(&(UserId::new("tenant_1:user_memory_forget").unwrap(), key))
            .expect("expected forgotten tombstone row");
        assert!(!current.active);
        assert!(current.memory_value.is_none());
        assert_eq!(store.memory_ledger_rows().len(), 2);
    }

    #[test]
    fn at_sim_exec_01i_memory_query_dispatch_returns_candidates() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:user_memory_query").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_locked_voice_profile_for_actor(&mut store, &exec, &actor, "tenant_1");

        let remember_draft = memory_draft(
            IntentType::MemoryRememberRequest,
            Some("Trip to Japan in March"),
        );
        let remember_x = Ph1xResponse::v1(
            11,
            2404,
            Ph1xDirective::Dispatch(
                DispatchDirective::simulation_candidate_v1(remember_draft).unwrap(),
            ),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-remember-3".to_string()),
        )
        .unwrap();
        let remember_out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(123_300_000_000),
                &remember_x,
            )
            .unwrap();
        assert!(matches!(
            remember_out,
            SimulationDispatchOutcome::MemoryPropose(_)
        ));

        let query_draft = memory_draft(IntentType::MemoryQuery, Some("Trip to Japan in March"));
        let query_x = Ph1xResponse::v1(
            11,
            2405,
            Ph1xDirective::Dispatch(
                DispatchDirective::simulation_candidate_v1(query_draft).unwrap(),
            ),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-query-1".to_string()),
        )
        .unwrap();
        let query_out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_400_000_000),
                &query_x,
            )
            .unwrap();
        match query_out {
            SimulationDispatchOutcome::MemoryRecall(resp) => {
                assert!(!resp.candidates.is_empty());
            }
            _ => panic!("expected memory recall outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01j_memory_dispatch_fails_closed_when_memory_wiring_disabled() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = simulation_executor_with_memory_enabled(false);

        let actor = UserId::new("tenant_1:user_memory_disabled").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let draft = memory_draft(
            IntentType::MemoryRememberRequest,
            Some("This should fail because PH1.M is disabled"),
        );
        let x = Ph1xResponse::v1(
            11,
            2406,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-disabled-1".to_string()),
        )
        .unwrap();

        let err = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_500_000_000),
                &x,
            )
            .expect_err("memory dispatch must fail closed when memory wiring is disabled");

        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "simulation_candidate_dispatch.ph1m");
                assert_eq!(reason, "memory_wiring_disabled");
            }
            _ => panic!("expected memory_wiring_disabled contract violation"),
        }
        assert!(store.memory_ledger_rows().is_empty());
        assert!(store.memory_current().is_empty());
    }

    #[test]
    fn at_sim_exec_01k_memory_dispatch_fails_closed_without_identity_assertion() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("tenant_1:user_memory_unknown").unwrap();
        let draft = memory_draft(
            IntentType::MemoryRememberRequest,
            Some("This should fail because speaker identity is unknown"),
        );
        let x = Ph1xResponse::v1(
            11,
            2407,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-memory-unknown-1".to_string()),
        )
        .unwrap();

        let err = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(123_600_000_000),
                &x,
            )
            .expect_err("memory dispatch must fail when speaker identity is unknown");

        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "ph1m.persistence.user_id");
                assert_eq!(reason, "speaker assertion must be OK for persistence");
            }
            _ => panic!("expected unknown-speaker contract violation"),
        }
        let feedback_rows = store.ph1feedback_audit_rows(CorrelationId(11));
        assert_eq!(feedback_rows.len(), 1);
        let payload_map = &feedback_rows[0].payload_min.entries;
        assert!(payload_map.contains_key(
            &PayloadKey::new("feedback_event_type").expect("feedback_event_type key is valid")
        ));
        assert!(payload_map.contains_key(
            &PayloadKey::new("learn_signal_type").expect("learn_signal_type key is valid")
        ));
    }

    #[test]
    fn at_sim_exec_01k2_voice_feedback_signal_commits_scoped_feedback_and_learn_audit() {
        let mut store = Ph1fStore::new_in_memory();
        let actor = UserId::new("tenant_1:voice_signal_user").unwrap();
        let device_id = DeviceId::new("voice-signal-device-1").unwrap();
        let runtime = Ph1VoiceIdLiveRuntime::default();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
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
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let req = build_voice_id_request_for_actor(MonotonicTimeNs(3), &actor, None)
            .expect("voice-id request must be valid");
        let context = crate::ph1_voice_id::VoiceIdentityRuntimeContext::from_tenant_app_platform(
            Some("tenant_1".to_string()),
            Some(AppPlatform::Ios),
            VoiceIdentityChannel::Explicit,
        );
        let out = runtime
            .run_identity_assertion_with_signal_emission(
                &mut store,
                &req,
                context,
                Vec::new(),
                EngineVoiceIdObservation {
                    primary_fingerprint: None,
                    secondary_fingerprint: None,
                    primary_embedding: None,
                    secondary_embedding: None,
                    spoof_risk: false,
                },
                crate::ph1_voice_id::VoiceIdentitySignalScope::v1(
                    MonotonicTimeNs(3),
                    CorrelationId(4401),
                    TurnId(1),
                    actor.clone(),
                    Some("tenant_1".to_string()),
                    Some(device_id),
                ),
            )
            .expect("scoped feedback/learn emission must succeed");
        assert!(matches!(
            out,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));

        let feedback_rows = store.ph1feedback_audit_rows(CorrelationId(4401));
        assert_eq!(feedback_rows.len(), 1);
        let feedback_payload = &feedback_rows[0].payload_min.entries;
        assert!(feedback_payload.contains_key(
            &PayloadKey::new("feedback_event_type").expect("feedback_event_type key is valid")
        ));
        assert!(feedback_payload
            .contains_key(&PayloadKey::new("signal_bucket").expect("signal_bucket key is valid")));
        assert!(feedback_payload
            .contains_key(&PayloadKey::new("path_type").expect("path_type key is valid")));

        let learn_bundle_rows = store.ph1feedback_learn_signal_bundle_rows(CorrelationId(4401));
        assert_eq!(learn_bundle_rows.len(), 1);
        assert_eq!(
            learn_bundle_rows[0].feedback_event_type,
            selene_kernel_contracts::ph1feedback::FeedbackEventType::VoiceIdFalseReject
        );
        assert_eq!(
            learn_bundle_rows[0].learn_signal_type,
            selene_kernel_contracts::ph1learn::LearnSignalType::VoiceIdFalseReject
        );

        let learn_rows = store
            .audit_events_by_correlation(CorrelationId(4401))
            .into_iter()
            .filter(|row| {
                matches!(&row.engine, AuditEngine::Other(engine_id) if engine_id == "PH1.LEARN")
            })
            .collect::<Vec<_>>();
        assert_eq!(learn_rows.len(), 1);
        let learn_payload = &learn_rows[0].payload_min.entries;
        assert!(learn_payload.contains_key(
            &PayloadKey::new("learn_signal_type").expect("learn_signal_type key is valid")
        ));
    }

    #[test]
    fn at_sim_exec_01b_bcast_mhp_defer_hands_off_to_rem_and_returns_handoff_refs() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_1").unwrap();
        let recipient = UserId::new("tenant_1:recipient_1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(901);
        let turn_id = TurnId(31);
        let now = MonotonicTimeNs(1_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_1").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Emergency,
                content_payload_ref: "payload_ref_1".to_string(),
                prompt_dedupe_key: Some("pd_1".to_string()),
                idempotency_key: "idem_bcast_draft_1".to_string(),
            }),
        };
        let draft_resp = exec.bcast.run(&draft_req);
        let broadcast_id = match draft_resp {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(32),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region:
                        selene_kernel_contracts::ph1bcast::BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_1".to_string(),
                    simulation_context: "sim_ctx_1".to_string(),
                    idempotency_key: "idem_bcast_deliver_1".to_string(),
                },
            ),
        };
        let deliver_resp = exec.bcast.run(&deliver_req);
        assert!(matches!(deliver_resp, Ph1BcastResponse::Ok(_)));

        let defer_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(33),
            now: MonotonicTimeNs(now.0 + 2),
            simulation_id: BCAST_DEFER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeferCommit(BcastDeferCommitRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender,
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id.clone(),
                defer_until: MonotonicTimeNs(now.0 + 10_000_000_000),
                handoff_to_reminder: true,
                idempotency_key: "idem_bcast_defer_1".to_string(),
            }),
        };

        let out = exec
            .run_broadcast_mhp_defer_with_reminder(
                &mut store,
                &defer_req,
                recipient,
                ReminderPriorityLevel::Normal,
                Some("pd_1"),
            )
            .unwrap();

        match out {
            SimulationDispatchOutcome::BroadcastMhpHandoff { bcast, reminder } => {
                match bcast {
                    Ph1BcastResponse::Ok(ok) => match ok.outcome {
                        BcastOutcome::DeferCommit(v) => {
                            assert_eq!(v.recipient_state, BcastRecipientState::ReminderSet);
                            assert!(v.handoff_to_reminder);
                        }
                        _ => panic!("expected defer commit outcome"),
                    },
                    _ => panic!("expected bcast ok"),
                }
                match reminder {
                    Ph1RemResponse::Ok(ok) => {
                        assert_eq!(ok.simulation_id, "REMINDER_SCHEDULE_COMMIT");
                        assert_eq!(
                            ok.state,
                            selene_kernel_contracts::ph1rem::ReminderState::Scheduled
                        );
                    }
                    _ => panic!("expected reminder schedule ok"),
                }
            }
            _ => panic!("expected broadcast+reminder handoff outcome"),
        }

        let reminder_fired_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(34),
            now: MonotonicTimeNs(now.0 + 20_000_000_000),
            simulation_id: BCAST_REMINDER_FIRED_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::ReminderFiredCommit(BcastReminderFiredCommitRequest {
                tenant_id,
                sender_user_id: UserId::new("tenant_1:sender_1").unwrap(),
                broadcast_id,
                recipient_id,
                reminder_ref: "rem_123".to_string(),
                idempotency_key: "idem_bcast_rem_fired_1".to_string(),
            }),
        };

        let fired_out = exec
            .run_broadcast_mhp_mark_reminder_fired(
                &reminder_fired_req,
                "tenant_1",
                "tenant_1:recipient_1",
                "tenant_1:recipient_1",
                "subject:bcast_mhp_followup",
                &[],
            )
            .unwrap();
        match fired_out {
            SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
                bcast,
                followup_source,
                policy_gate_ok,
                followup_subject_ref,
                followup_recipient_user_id,
                followup_active_speaker_user_id,
                followup_delivery_mode,
                followup_voice_ref,
                ..
            } => match bcast {
                Ph1BcastResponse::Ok(ok) => match ok.outcome {
                    BcastOutcome::ReminderFiredCommit(v) => {
                        assert_eq!(v.recipient_state, BcastRecipientState::ReminderFired);
                        assert_eq!(followup_source, "REMINDER_FIRED_FOLLOWUP");
                        assert!(policy_gate_ok);
                        assert_eq!(followup_subject_ref, "subject:bcast_mhp_followup");
                        assert_eq!(followup_recipient_user_id, "tenant_1:recipient_1");
                        assert_eq!(followup_active_speaker_user_id, "tenant_1:recipient_1");
                        assert_eq!(followup_delivery_mode, BcastFollowupDeliveryMode::Voice);
                        assert!(followup_voice_ref.is_some());
                    }
                    _ => panic!("expected reminder fired outcome"),
                },
                _ => panic!("expected reminder fired bcast ok"),
            },
            _ => panic!("expected reminder fired dispatch outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01c_bcast_urgent_post_delivery_followup_is_policy_gated() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_u").unwrap();
        let recipient = UserId::new("tenant_1:recipient_u").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(902);
        let turn_id = TurnId(40);
        let now = MonotonicTimeNs(2_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_u").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Emergency,
                content_payload_ref: "payload_urgent".to_string(),
                prompt_dedupe_key: Some("pd_u".to_string()),
                idempotency_key: "idem_bcast_draft_u".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(41),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id,
                    sender_user_id: sender,
                    broadcast_id,
                    recipient_id,
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_u".to_string(),
                    simulation_context: "sim_ctx_u".to_string(),
                    idempotency_key: "idem_bcast_deliver_u".to_string(),
                },
            ),
        };
        let out = exec
            .run_broadcast_mhp_deliver_and_maybe_followup(
                &deliver_req,
                "tenant_1",
                recipient.as_str(),
                recipient.as_str(),
                "subject:urgent_followup",
                &[],
            )
            .unwrap();

        match out {
            SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
                bcast,
                followup_source,
                policy_gate_ok,
                followup_subject_ref,
                followup_recipient_user_id,
                followup_active_speaker_user_id,
                followup_delivery_mode,
                followup_voice_ref,
                ..
            } => {
                assert_eq!(followup_source, "URGENT_POST_DELIVERY");
                assert!(policy_gate_ok);
                assert_eq!(followup_subject_ref, "subject:urgent_followup");
                assert_eq!(followup_recipient_user_id, recipient.as_str());
                assert_eq!(followup_active_speaker_user_id, recipient.as_str());
                assert_eq!(followup_delivery_mode, BcastFollowupDeliveryMode::Voice);
                assert!(followup_voice_ref.is_some());
                match bcast {
                    Ph1BcastResponse::Ok(ok) => match ok.outcome {
                        BcastOutcome::DeliverCommit(v) => {
                            assert!(v.followup_immediate);
                            assert_eq!(v.recipient_state, BcastRecipientState::Followup);
                        }
                        _ => panic!("expected deliver outcome"),
                    },
                    _ => panic!("expected bcast ok"),
                }
            }
            _ => panic!("expected followup decision outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01c1_bcast_urgent_followup_uses_text_only_when_explicitly_requested() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_ut").unwrap();
        let recipient = UserId::new("tenant_1:recipient_ut").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(1902);
        let turn_id = TurnId(140);
        let now = MonotonicTimeNs(2_100_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_ut").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Emergency,
                content_payload_ref: "payload_urgent_text".to_string(),
                prompt_dedupe_key: Some("pd_ut".to_string()),
                idempotency_key: "idem_bcast_draft_ut".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(141),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id,
                    sender_user_id: sender,
                    broadcast_id,
                    recipient_id,
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_ut".to_string(),
                    simulation_context: "sim_ctx_ut".to_string(),
                    idempotency_key: "idem_bcast_deliver_ut".to_string(),
                },
            ),
        };

        let out = exec
            .run_broadcast_mhp_deliver_and_maybe_followup_with_delivery_hint(
                &deliver_req,
                "tenant_1",
                recipient.as_str(),
                recipient.as_str(),
                "subject:urgent_followup_text",
                &[],
                BcastFollowupDeliveryHint::TextOnly(BcastFollowupTextOnlyReason::UserRequestedText),
            )
            .unwrap();

        match out {
            SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
                followup_source,
                policy_gate_ok,
                followup_subject_ref,
                followup_recipient_user_id,
                followup_active_speaker_user_id,
                followup_delivery_mode,
                followup_text_only_reason,
                followup_voice_ref,
                followup_text_ref,
                ..
            } => {
                assert_eq!(followup_source, "URGENT_POST_DELIVERY");
                assert!(policy_gate_ok);
                assert_eq!(followup_subject_ref, "subject:urgent_followup_text");
                assert_eq!(followup_recipient_user_id, recipient.as_str());
                assert_eq!(followup_active_speaker_user_id, recipient.as_str());
                assert_eq!(followup_delivery_mode, BcastFollowupDeliveryMode::Text);
                assert_eq!(
                    followup_text_only_reason,
                    Some(BcastFollowupTextOnlyReason::UserRequestedText)
                );
                assert!(followup_voice_ref.is_none());
                assert!(followup_text_ref.is_some());
            }
            _ => panic!("expected followup decision outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01c2_bcast_followup_fails_closed_on_speaker_mismatch() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_um").unwrap();
        let recipient = UserId::new("tenant_1:recipient_um").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(1903);
        let now = MonotonicTimeNs(2_200_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_um").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(150),
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Emergency,
                content_payload_ref: "payload_urgent_mismatch".to_string(),
                prompt_dedupe_key: Some("pd_um".to_string()),
                idempotency_key: "idem_bcast_draft_um".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(151),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id,
                    sender_user_id: sender,
                    broadcast_id,
                    recipient_id,
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_um".to_string(),
                    simulation_context: "sim_ctx_um".to_string(),
                    idempotency_key: "idem_bcast_deliver_um".to_string(),
                },
            ),
        };

        let out = exec.run_broadcast_mhp_deliver_and_maybe_followup(
            &deliver_req,
            "tenant_1",
            recipient.as_str(),
            "tenant_1:someone_else",
            "subject:urgent_followup_mismatch",
            &[],
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_sim_exec_01d_bcast_non_urgent_wait_timeout_followup_is_policy_gated_after_window() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_w").unwrap();
        let recipient = UserId::new("tenant_1:recipient_w").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(903);
        let now = MonotonicTimeNs(3_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_w").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(50),
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Priority,
                content_payload_ref: "payload_wait".to_string(),
                prompt_dedupe_key: Some("pd_w".to_string()),
                idempotency_key: "idem_bcast_draft_w".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(51),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_w".to_string(),
                    simulation_context: "sim_ctx_w".to_string(),
                    idempotency_key: "idem_bcast_deliver_w".to_string(),
                },
            ),
        };
        let _deliver = exec.bcast.run(&deliver_req);

        let escalate_early = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(52),
            now: MonotonicTimeNs(now.0 + 2),
            simulation_id: BCAST_ESCALATE_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::EscalateCommit(BcastEscalateCommitRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                broadcast_id: broadcast_id.clone(),
                recipient_id: recipient_id.clone(),
                escalation_reason: "waiting_timeout".to_string(),
                idempotency_key: "idem_bcast_escalate_early".to_string(),
            }),
        };
        assert!(exec
            .run_broadcast_mhp_wait_timeout_followup(
                &escalate_early,
                "tenant_1",
                recipient.as_str(),
                recipient.as_str(),
                "subject:non_urgent_wait",
                &[],
            )
            .is_err());

        let escalate_late = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(53),
            now: MonotonicTimeNs(now.0 + BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS + 3),
            simulation_id: BCAST_ESCALATE_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::EscalateCommit(BcastEscalateCommitRequest {
                tenant_id,
                sender_user_id: sender,
                broadcast_id,
                recipient_id,
                escalation_reason: "waiting_timeout".to_string(),
                idempotency_key: "idem_bcast_escalate_late".to_string(),
            }),
        };
        let out = exec
            .run_broadcast_mhp_wait_timeout_followup(
                &escalate_late,
                "tenant_1",
                recipient.as_str(),
                recipient.as_str(),
                "subject:non_urgent_wait",
                &[],
            )
            .unwrap();
        match out {
            SimulationDispatchOutcome::BroadcastMhpFollowupDecision {
                followup_source,
                policy_gate_ok,
                followup_subject_ref,
                followup_recipient_user_id,
                followup_active_speaker_user_id,
                followup_delivery_mode,
                followup_voice_ref,
                ..
            } => {
                assert_eq!(followup_source, "WAIT_TIMEOUT_FOLLOWUP");
                assert!(policy_gate_ok);
                assert_eq!(followup_subject_ref, "subject:non_urgent_wait");
                assert_eq!(followup_recipient_user_id, recipient.as_str());
                assert_eq!(followup_active_speaker_user_id, recipient.as_str());
                assert_eq!(followup_delivery_mode, BcastFollowupDeliveryMode::Voice);
                assert!(followup_voice_ref.is_some());
            }
            _ => panic!("expected followup decision outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01e_bcast_app_thread_reply_auto_forwards_and_suppresses_voice_interrupt() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_a").unwrap();
        let recipient = UserId::new("tenant_1:recipient_a").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                recipient.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(904);
        let now = MonotonicTimeNs(4_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_a").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(60),
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Priority,
                content_payload_ref: "payload_app".to_string(),
                prompt_dedupe_key: Some("pd_a".to_string()),
                idempotency_key: "idem_bcast_draft_a".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };
        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(61),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::SeleneApp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: false,
                    delivery_plan_ref: "delivery_plan_a".to_string(),
                    simulation_context: "sim_ctx_a".to_string(),
                    idempotency_key: "idem_bcast_deliver_a".to_string(),
                },
            ),
        };
        let _deliver = exec.bcast.run(&deliver_req);

        let ack_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(62),
            now: MonotonicTimeNs(now.0 + 2),
            simulation_id: BCAST_ACK_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::AckCommit(BcastAckCommitRequest {
                tenant_id,
                recipient_user_id: recipient,
                broadcast_id,
                recipient_id,
                ack_status: BcastAckStatus::Received,
                idempotency_key: "idem_bcast_ack_a".to_string(),
            }),
        };
        let out = exec
            .run_broadcast_mhp_app_thread_reply_conclude(
                &ack_req,
                "wife_thread_1",
                "reply_payload_1",
            )
            .unwrap();
        match out {
            SimulationDispatchOutcome::BroadcastMhpAppThreadReplyConcluded {
                bcast,
                wife_forward_ref,
                voice_interruption_suppressed,
            } => {
                assert!(voice_interruption_suppressed);
                assert!(!wife_forward_ref.is_empty());
                match bcast {
                    Ph1BcastResponse::Ok(ok) => match ok.outcome {
                        BcastOutcome::AckCommit(v) => {
                            assert_eq!(v.recipient_state, BcastRecipientState::Concluded);
                        }
                        _ => panic!("expected ack outcome"),
                    },
                    _ => panic!("expected bcast ack ok"),
                }
            }
            _ => panic!("expected app-thread conclude outcome"),
        }
    }

    #[test]
    fn at_sim_exec_01f_bcast_fallback_order_e2e_global_path_locked() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let sender = UserId::new("tenant_1:sender_f").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                sender.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let correlation_id = CorrelationId(905);
        let now = MonotonicTimeNs(5_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_f").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(70),
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "jd".to_string(),
                classification: BroadcastClassification::Priority,
                content_payload_ref: "payload_fallback".to_string(),
                prompt_dedupe_key: Some("pd_f".to_string()),
                idempotency_key: "idem_bcast_draft_f".to_string(),
            }),
        };
        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(r) => r.broadcast_id,
                _ => panic!("expected draft create"),
            },
            _ => panic!("expected draft ok"),
        };

        let sms = exec.bcast.run(&Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(71),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::Sms,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: true,
                    delivery_plan_ref: "delivery_sms".to_string(),
                    simulation_context: "sim_ctx_sms".to_string(),
                    idempotency_key: "idem_bcast_sms".to_string(),
                },
            ),
        });
        assert!(matches!(sms, Ph1BcastResponse::Ok(_)));

        let skip = exec.bcast.run(&Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(72),
            now: MonotonicTimeNs(now.0 + 2),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::Email,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: true,
                    delivery_plan_ref: "delivery_skip".to_string(),
                    simulation_context: "sim_ctx_skip".to_string(),
                    idempotency_key: "idem_bcast_skip".to_string(),
                },
            ),
        });
        match skip {
            Ph1BcastResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    selene_engines::ph1bcast::reason_codes::BCAST_FAIL_FALLBACK_POLICY
                );
            }
            _ => panic!("expected fallback skip refusal"),
        }

        let whatsapp = exec.bcast.run(&Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(73),
            now: MonotonicTimeNs(now.0 + 3),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id: tenant_id.clone(),
                    sender_user_id: sender.clone(),
                    broadcast_id: broadcast_id.clone(),
                    recipient_id: recipient_id.clone(),
                    delivery_method: BcastDeliveryMethod::Whatsapp,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: true,
                    delivery_plan_ref: "delivery_whatsapp".to_string(),
                    simulation_context: "sim_ctx_whatsapp".to_string(),
                    idempotency_key: "idem_bcast_whatsapp".to_string(),
                },
            ),
        });
        assert!(matches!(whatsapp, Ph1BcastResponse::Ok(_)));

        let email = exec.bcast.run(&Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(74),
            now: MonotonicTimeNs(now.0 + 4),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id,
                    sender_user_id: sender,
                    broadcast_id,
                    recipient_id,
                    delivery_method: BcastDeliveryMethod::Email,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: true,
                    delivery_plan_ref: "delivery_email".to_string(),
                    simulation_context: "sim_ctx_email".to_string(),
                    idempotency_key: "idem_bcast_email".to_string(),
                },
            ),
        });
        assert!(matches!(email, Ph1BcastResponse::Ok(_)));
    }

    #[test]
    fn at_sim_exec_01g_bcast_deliver_is_wired_to_ph1_delivery_send() {
        let exec = SimulationExecutor::default();

        let correlation_id = CorrelationId(906);
        let now = MonotonicTimeNs(6_000_000_000_000);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let sender = UserId::new("tenant_1:sender_g").unwrap();
        let recipient_id = BroadcastRecipientId::new("recipient_g").unwrap();

        let draft_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(80),
            now,
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant_id.clone(),
                sender_user_id: sender.clone(),
                audience_spec: "audience_g".to_string(),
                classification: BroadcastClassification::Priority,
                content_payload_ref: "payload_g".to_string(),
                prompt_dedupe_key: Some("pd_g".to_string()),
                idempotency_key: "idem_bcast_draft_g".to_string(),
            }),
        };

        let broadcast_id = match exec.bcast.run(&draft_req) {
            Ph1BcastResponse::Ok(ok) => match ok.outcome {
                BcastOutcome::DraftCreate(v) => v.broadcast_id,
                _ => panic!("expected draft create outcome"),
            },
            _ => panic!("expected draft create response"),
        };

        let deliver_req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id,
            turn_id: TurnId(81),
            now: MonotonicTimeNs(now.0 + 1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(
                selene_kernel_contracts::ph1bcast::BcastDeliverCommitRequest {
                    tenant_id,
                    sender_user_id: sender,
                    broadcast_id,
                    recipient_id,
                    delivery_method: BcastDeliveryMethod::Sms,
                    recipient_region: BcastRecipientRegion::Global,
                    app_unavailable: true,
                    delivery_plan_ref: "delivery_plan_g".to_string(),
                    simulation_context: "sim_ctx_g".to_string(),
                    idempotency_key: "idem_bcast_deliver_g".to_string(),
                },
            ),
        };

        let out = exec
            .run_broadcast_deliver_with_delivery(&deliver_req)
            .unwrap();
        match out {
            SimulationDispatchOutcome::BroadcastDeliverySend { bcast, delivery } => {
                assert!(matches!(bcast, Ph1BcastResponse::Ok(_)));
                match delivery {
                    Ph1DeliveryResponse::Ok(ok) => match ok.outcome {
                        DeliveryOutcome::Send(v) => {
                            assert!(v.delivery_attempt_id.starts_with("delivery_"));
                            assert!(v.delivery_proof_ref.contains("kms://delivery/sms/default"));
                        }
                        _ => panic!("expected delivery send outcome"),
                    },
                    _ => panic!("expected delivery ok response"),
                }
            }
            _ => panic!("expected BroadcastDeliverySend outcome"),
        }
    }

    #[test]
    fn at_sim_exec_14_link_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-deny-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            31,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-deny-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(190),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_15_link_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-escalate-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            32,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-escalate-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(191),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_16_link_tenant_scope_mismatch_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-scope-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CreateInviteLink,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::InviteeType,
                    value: FieldValue::verbatim("employee".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_2".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            33,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-link-scope-1".to_string()),
        )
        .unwrap();

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(192),
            &x,
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.link.access_instance_id",
                    reason: "missing access instance for actor_user_id + tenant_id",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_17_link_allow_path_idempotent_across_retries() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("inviter-link-allow-retry-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_link_access_instance(&mut store, &actor, "tenant_1");

        let mk_x = |turn_id: u64| {
            let draft = IntentDraft::v1(
                IntentType::CreateInviteLink,
                SchemaVersion(1),
                vec![
                    IntentField {
                        key: FieldKey::InviteeType,
                        value: FieldValue::verbatim("employee".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::TenantId,
                        value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
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
            .unwrap();
            Ph1xResponse::v1(
                10,
                turn_id,
                Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
                ThreadState::empty_v1(),
                None,
                DeliveryHint::AudibleAndText,
                ReasonCodeId(1),
                Some("idem-link-allow-retry-1".to_string()),
            )
            .unwrap()
        };

        let out1 = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(193),
                &mk_x(34),
            )
            .unwrap();
        let token_1 = match out1 {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => ok
                .link_generate_result
                .expect("link_generate_result")
                .token_id
                .as_str()
                .to_string(),
            _ => panic!("expected link ok"),
        };

        let out2 = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(194),
                &mk_x(35),
            )
            .unwrap();
        let token_2 = match out2 {
            SimulationDispatchOutcome::Link(Ph1LinkResponse::Ok(ok)) => ok
                .link_generate_result
                .expect("link_generate_result")
                .token_id
                .as_str()
                .to_string(),
            _ => panic!("expected link ok"),
        };

        assert_eq!(token_1, token_2);
    }

    #[test]
    fn at_sim_exec_18_legacy_link_delivery_simulation_ids_fail_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let mut req = Ph1LinkRequest::invite_generate_draft_v1(
            CorrelationId(88),
            TurnId(1),
            MonotonicTimeNs(200),
            UserId::new("inviter-legacy-guard").unwrap(),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        req.simulation_id = "LINK_INVITE_SEND_COMMIT".to_string();

        let out = exec.execute_link(&mut store, &req);
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_request.simulation_id",
                    reason: "LEGACY_DO_NOT_WIRE: delivery is owned by LINK_DELIVER_INVITE via PH1.BCAST + PH1.DELIVERY",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_02_execute_wake_start_draft_runs_ph1w_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("wake-user-1").unwrap();
        let device_id = DeviceId::new("wake-device-1").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
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
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let req = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(777),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(123),
            simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
            simulation_type: WakeSimulationType::Draft,
            request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                user_id: actor,
                device_id,
                onboarding_session_id: None,
                allow_ios_wake_override: false,
                pass_target: 5,
                max_attempts: 12,
                enrollment_timeout_ms: 300_000,
                idempotency_key: "wake-start-1".to_string(),
            }),
        };

        let out = exec.execute_wake(&mut store, &req).unwrap();
        match out {
            Ph1wResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, WAKE_ENROLL_START_DRAFT);
                assert!(ok.enroll_start_result.is_some());
            }
            Ph1wResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn at_sim_exec_05_ph1x_sim_candidate_capreq_manage_runs_capreq_create_draft() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance(&mut store, &actor, "tenant_1");

        let draft = IntentDraft::v1(
            IntentType::CapreqManage,
            SchemaVersion(1),
            vec![
                IntentField {
                    key: FieldKey::TenantId,
                    value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::RequestedCapabilityId,
                    value: FieldValue::verbatim("payroll.approve".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::TargetScopeRef,
                    value: FieldValue::verbatim("store_17".to_string()).unwrap(),
                    confidence: OverallConfidence::High,
                },
                IntentField {
                    key: FieldKey::Justification,
                    value: FieldValue::verbatim("monthly payroll processing".to_string()).unwrap(),
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
        .unwrap();

        let x = Ph1xResponse::v1(
            10,
            23,
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(draft).unwrap()),
            ThreadState::empty_v1(),
            None,
            DeliveryHint::AudibleAndText,
            ReasonCodeId(1),
            Some("idem-capreq-1".to_string()),
        )
        .unwrap();

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(200),
                &x,
            )
            .unwrap();

        let (capreq_id, capreq_event_id, action, status) = match out {
            SimulationDispatchOutcome::CapreqLifecycle {
                capreq_id,
                capreq_event_id,
                action,
                status,
            } => (capreq_id, capreq_event_id, action, status),
            _ => panic!("expected capreq draft outcome"),
        };
        assert!(capreq_event_id > 0);
        assert_eq!(action, CapabilityRequestAction::CreateDraft);
        assert_eq!(status, CapabilityRequestStatus::Draft);

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Draft);
        assert_eq!(current.last_action, CapabilityRequestAction::CreateDraft);
        assert_eq!(current.requester_user_id, actor);
        assert_eq!(store.capreq_events().len(), 1);
        assert_eq!(
            store.capreq_events()[0].capreq_id,
            CapreqId::new(capreq_id.as_str().to_string()).unwrap()
        );
    }

    #[test]
    fn at_sim_exec_06_capreq_submit_for_approval_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-2").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            100,
            "idem-capreq-2-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let submitted = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            101,
            "idem-capreq-2-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        match submitted {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::SubmitForApproval);
                assert_eq!(status, CapabilityRequestStatus::PendingApproval);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::PendingApproval);
        assert_eq!(
            current.last_action,
            CapabilityRequestAction::SubmitForApproval
        );
        assert_eq!(store.capreq_events().len(), 2);
    }

    #[test]
    fn at_sim_exec_07_capreq_approve_and_fulfill_transitions() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-3").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            110,
            "idem-capreq-3-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let _ = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            111,
            "idem-capreq-3-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        let approved = run_capreq(
            &exec,
            &mut store,
            &actor,
            3,
            112,
            "idem-capreq-3-approve",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "approve"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match approved {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Approve);
                assert_eq!(status, CapabilityRequestStatus::Approved);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let fulfilled = run_capreq(
            &exec,
            &mut store,
            &actor,
            4,
            113,
            "idem-capreq-3-fulfill",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "fulfill"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match fulfilled {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Fulfill);
                assert_eq!(status, CapabilityRequestStatus::Fulfilled);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Fulfilled);
        assert_eq!(current.last_action, CapabilityRequestAction::Fulfill);
        assert_eq!(store.capreq_events().len(), 4);
    }

    #[test]
    fn at_sim_exec_08_capreq_reject_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-4").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            120,
            "idem-capreq-4-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let _ = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            121,
            "idem-capreq-4-submit",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "submit_for_approval"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );

        let rejected = run_capreq(
            &exec,
            &mut store,
            &actor,
            3,
            122,
            "idem-capreq-4-reject",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "reject"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match rejected {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Reject);
                assert_eq!(status, CapabilityRequestStatus::Rejected);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Rejected);
        assert_eq!(current.last_action, CapabilityRequestAction::Reject);
        assert_eq!(store.capreq_events().len(), 3);
    }

    #[test]
    fn at_sim_exec_09_capreq_cancel_transition() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-5").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            130,
            "idem-capreq-5-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let canceled = run_capreq(
            &exec,
            &mut store,
            &actor,
            2,
            131,
            "idem-capreq-5-cancel",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::CapreqAction, "cancel"),
                capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
            ],
        );
        match canceled {
            SimulationDispatchOutcome::CapreqLifecycle { action, status, .. } => {
                assert_eq!(action, CapabilityRequestAction::Cancel);
                assert_eq!(status, CapabilityRequestStatus::Canceled);
            }
            _ => panic!("expected capreq lifecycle outcome"),
        }

        let tenant_id = TenantId::new("tenant_1").unwrap();
        let current = store
            .capreq_current_row(&tenant_id, &capreq_id)
            .expect("capreq current row exists");
        assert_eq!(current.status, CapabilityRequestStatus::Canceled);
        assert_eq!(current.last_action, CapabilityRequestAction::Cancel);
        assert_eq!(store.capreq_events().len(), 2);
    }

    #[test]
    fn at_sim_exec_10_capreq_invalid_transition_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-6").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();

        let created = run_capreq(
            &exec,
            &mut store,
            &actor,
            1,
            140,
            "idem-capreq-6-create",
            vec![
                capreq_field(FieldKey::TenantId, "tenant_1"),
                capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                capreq_field(FieldKey::TargetScopeRef, "store_17"),
                capreq_field(FieldKey::Justification, "monthly payroll processing"),
            ],
        );
        let capreq_id = match created {
            SimulationDispatchOutcome::CapreqLifecycle { capreq_id, .. } => capreq_id,
            _ => panic!("expected capreq lifecycle outcome"),
        };

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(141),
            &capreq_x(
                2,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::CapreqAction, "approve"),
                    capreq_field(FieldKey::CapreqId, capreq_id.as_str()),
                ]),
                "idem-capreq-6-approve-invalid",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "capreq_transition",
                    ..
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_11_capreq_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-7").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(150),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-7-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_12_capreq_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-8").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance_with(
            &mut store,
            &actor,
            "tenant_1",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(151),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_1"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-8-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_13_capreq_tenant_scope_mismatch_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("capreq-actor-9").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_capreq_access_instance(&mut store, &actor, "tenant_1");

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(152),
            &capreq_x(
                1,
                capreq_draft(vec![
                    capreq_field(FieldKey::TenantId, "tenant_2"),
                    capreq_field(FieldKey::RequestedCapabilityId, "payroll.approve"),
                    capreq_field(FieldKey::TargetScopeRef, "store_17"),
                    capreq_field(FieldKey::Justification, "monthly payroll processing"),
                ]),
                "idem-capreq-9-create",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.capreq.access_instance_id",
                    reason: "missing access instance for actor_user_id + tenant_id",
                }
            ))
        ));
        assert_eq!(store.capreq_events().len(), 0);
        assert_eq!(store.capreq_current().len(), 0);
    }

    #[test]
    fn at_sim_exec_19_access_schema_manage_gate_allow_returns_gate_passed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(200),
                &access_x(
                    1,
                    access_draft(
                        IntentType::AccessSchemaManage,
                        vec![
                            access_field(FieldKey::TenantId, "tenant_1"),
                            access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                            access_field(FieldKey::AccessProfileId, "AP_CLERK"),
                            access_field(FieldKey::SchemaVersionId, "v1"),
                            access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                            access_field(
                                FieldKey::ProfilePayloadJson,
                                "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                            ),
                        ],
                    ),
                    "idem-access-schema-allow-1",
                ),
            )
            .unwrap();
        assert!(matches!(
            out,
            SimulationDispatchOutcome::AccessGatePassed {
                requested_action
            } if requested_action == "ACCESS_SCHEMA_MANAGE"
        ));
    }

    #[test]
    fn at_sim_exec_20_access_escalation_vote_access_deny_blocks_governed_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-vote-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_vote_admin",
            "{\"allow\":[\"ACCESS_ESCALATION_VOTE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Suspended,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(201),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessEscalationVote,
                    vec![access_field(FieldKey::TenantId, "tenant_1")],
                ),
                "idem-access-vote-deny-1",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.access_escalation_vote.access_decision",
                    reason: "ACCESS_SCOPE_VIOLATION",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_21_access_instance_compile_access_escalate_requires_approval_before_commit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-compile-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_compile_admin",
            "{\"allow\":[\"ACCESS_INSTANCE_COMPILE_REFRESH\"]}",
            AccessMode::R,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(202),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessInstanceCompileRefresh,
                    vec![access_field(FieldKey::TenantId, "tenant_1")],
                ),
                "idem-access-compile-escalate-1",
            ),
        );
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.access_instance_compile.access_decision",
                    reason: "ACCESS_AP_REQUIRED",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_22_access_schema_manage_missing_review_channel_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-2").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(203),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                        access_field(FieldKey::AccessProfileId, "AP_CLERK"),
                        access_field(FieldKey::SchemaVersionId, "v1"),
                        access_field(
                            FieldKey::ProfilePayloadJson,
                            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                        ),
                    ],
                ),
                "idem-access-schema-missing-channel-1",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields",
                    reason: "missing required field",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_23_access_schema_manage_read_out_loud_gate_allow_returns_gate_passed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-3").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor,
                MonotonicTimeNs(204),
                &access_x(
                    1,
                    access_draft(
                        IntentType::AccessSchemaManage,
                        vec![
                            access_field(FieldKey::TenantId, "tenant_1"),
                            access_field(FieldKey::ApAction, "CREATE_DRAFT"),
                            access_field(FieldKey::AccessProfileId, "AP_CEO"),
                            access_field(FieldKey::SchemaVersionId, "v1"),
                            access_field(FieldKey::AccessReviewChannel, "READ_OUT_LOUD"),
                            access_field(
                                FieldKey::ProfilePayloadJson,
                                "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
                            ),
                        ],
                    ),
                    "idem-access-schema-read-out-loud-1",
                ),
            )
            .unwrap();

        assert!(matches!(
            out,
            SimulationDispatchOutcome::AccessGatePassed {
                requested_action
            } if requested_action == "ACCESS_SCHEMA_MANAGE"
        ));
    }

    #[test]
    fn at_sim_exec_24_access_schema_manage_activate_missing_rule_action_fails_closed() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-4").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let out = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(205),
            &access_x(
                1,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "ACTIVATE"),
                        access_field(FieldKey::AccessProfileId, "AP_CEO"),
                        access_field(FieldKey::SchemaVersionId, "v2"),
                        access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                    ],
                ),
                "idem-access-schema-activate-missing-rule-action-1",
            ),
        );

        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields",
                    reason: "missing required field",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_25_access_schema_manage_activate_rule_actions_bounded_and_validated() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("access-schema-actor-5").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_schema_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let allowed_actions = [
            "AGREE",
            "DISAGREE",
            "EDIT",
            "DELETE",
            "DISABLE",
            "ADD_CUSTOM_RULE",
        ];

        for (idx, action) in allowed_actions.iter().enumerate() {
            let review_channel = if idx % 2 == 0 {
                "PHONE_DESKTOP"
            } else {
                "READ_OUT_LOUD"
            };
            let out = exec
                .execute_ph1x_dispatch_simulation_candidate(
                    &mut store,
                    actor.clone(),
                    MonotonicTimeNs(206 + idx as u64),
                    &access_x(
                        idx as u64 + 1,
                        access_draft(
                            IntentType::AccessSchemaManage,
                            vec![
                                access_field(FieldKey::TenantId, "tenant_1"),
                                access_field(FieldKey::ApAction, "ACTIVATE"),
                                access_field(FieldKey::AccessProfileId, "AP_CEO"),
                                access_field(FieldKey::SchemaVersionId, &format!("v{}", idx + 1)),
                                access_field(FieldKey::AccessReviewChannel, review_channel),
                                access_field(FieldKey::AccessRuleAction, action),
                            ],
                        ),
                        &format!("idem-access-schema-activate-action-{}", idx + 1),
                    ),
                )
                .unwrap();
            assert!(matches!(
                out,
                SimulationDispatchOutcome::AccessGatePassed {
                    requested_action
                } if requested_action == "ACCESS_SCHEMA_MANAGE"
            ));
        }

        let invalid = exec.execute_ph1x_dispatch_simulation_candidate(
            &mut store,
            actor,
            MonotonicTimeNs(213),
            &access_x(
                999,
                access_draft(
                    IntentType::AccessSchemaManage,
                    vec![
                        access_field(FieldKey::TenantId, "tenant_1"),
                        access_field(FieldKey::ApAction, "ACTIVATE"),
                        access_field(FieldKey::AccessProfileId, "AP_CEO"),
                        access_field(FieldKey::SchemaVersionId, "v999"),
                        access_field(FieldKey::AccessReviewChannel, "PHONE_DESKTOP"),
                        access_field(FieldKey::AccessRuleAction, "UNBOUNDED_ACTION"),
                    ],
                ),
                "idem-access-schema-activate-action-invalid",
            ),
        );
        assert!(matches!(
            invalid,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "simulation_candidate_dispatch.intent_draft.fields.access_rule_action",
                    reason:
                        "must be one of: AGREE, DISAGREE, EDIT, DELETE, DISABLE, ADD_CUSTOM_RULE",
                }
            ))
        ));
    }

    #[test]
    fn at_sim_exec_03_execute_position_create_draft_runs_ph1position_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("position-actor-1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .ph1tenant_company_upsert(TenantCompanyRecord {
                schema_version: SchemaVersion(1),
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Inc".to_string(),
                jurisdiction: "CN".to_string(),
                lifecycle_state: TenantCompanyLifecycleState::Active,
                created_at: MonotonicTimeNs(1),
                updated_at: MonotonicTimeNs(1),
            })
            .unwrap();

        let req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(900),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(123),
            simulation_id: POSITION_SIM_001_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::CreateDraft(PositionCreateDraftRequest {
                actor_user_id: actor,
                tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                position_title: "Store Manager".to_string(),
                department: "Retail".to_string(),
                jurisdiction: "CN".to_string(),
                schedule_type: PositionScheduleType::FullTime,
                permission_profile_ref: "profile_store_mgr".to_string(),
                compensation_band_ref: "band_l3".to_string(),
                idempotency_key: "position-create-1".to_string(),
            }),
        };

        let out = exec.execute_position(&mut store, &req).unwrap();
        match out {
            Ph1PositionResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, POSITION_SIM_001_CREATE_DRAFT);
                assert!(ok.create_draft_result.is_some());
            }
            Ph1PositionResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn at_sim_exec_04_execute_voice_id_enroll_start_runs_ph1voiceid_runtime() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("voice-actor-1").unwrap();
        let device_id = DeviceId::new("voice-device-1").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
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
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_vid_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let (link_rec, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(2),
                actor.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        let _ = store
            .ph1link_invite_open_activate_commit(
                MonotonicTimeNs(3),
                link_rec.token_id.clone(),
                "voice-device-fp-1".to_string(),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(4),
                link_rec.token_id,
                None,
                Some("tenant_1".to_string()),
                "voice-device-fp-1".to_string(),
                selene_kernel_contracts::ph1link::AppPlatform::Ios,
                "legacy_app_instance".to_string(),
                "legacy_nonce_3".to_string(),
                MonotonicTimeNs(3),
            )
            .unwrap();

        let req = Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: CorrelationId(1001),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(10),
            simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
            simulation_type: VoiceIdSimulationType::Draft,
            request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
                onboarding_session_id: onb.onboarding_session_id.as_str().to_string(),
                device_id,
                consent_asserted: true,
                max_total_attempts: 8,
                max_session_enroll_time_ms: 120_000,
                lock_after_consecutive_passes: 3,
            }),
        };

        let out = exec.execute_voice_id(&mut store, &req).unwrap();
        match out {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, VOICE_ID_ENROLL_START_DRAFT);
                assert!(ok.enroll_start_result.is_some());
            }
            Ph1VoiceIdSimResponse::Refuse(_) => panic!("expected ok"),
        }
    }

    #[test]
    fn at_sim_exec_04b_voice_complete_runs_mobile_sync_worker_ack_path() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();

        let actor = UserId::new("voice-actor-2").unwrap();
        let device_id = DeviceId::new("voice-device-2").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
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
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_vid_2".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let (link_rec, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(2),
                actor.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        let _ = store
            .ph1link_invite_open_activate_commit(
                MonotonicTimeNs(3),
                link_rec.token_id.clone(),
                "voice-device-fp-2".to_string(),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(4),
                link_rec.token_id,
                None,
                Some("tenant_1".to_string()),
                "voice-device-fp-2".to_string(),
                selene_kernel_contracts::ph1link::AppPlatform::Ios,
                "legacy_app_instance".to_string(),
                "legacy_nonce_3".to_string(),
                MonotonicTimeNs(3),
            )
            .unwrap();

        let start = exec
            .execute_voice_id(
                &mut store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1002),
                    turn_id: TurnId(1),
                    now: MonotonicTimeNs(10),
                    simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
                    simulation_type: VoiceIdSimulationType::Draft,
                    request: VoiceIdSimulationRequest::EnrollStartDraft(
                        VoiceIdEnrollStartDraftRequest {
                            onboarding_session_id: onb.onboarding_session_id.as_str().to_string(),
                            device_id: device_id.clone(),
                            consent_asserted: true,
                            max_total_attempts: 8,
                            max_session_enroll_time_ms: 120_000,
                            lock_after_consecutive_passes: 2,
                        },
                    ),
                },
            )
            .unwrap();
        let voice_enrollment_session_id = match start {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                ok.enroll_start_result
                    .expect("start result must exist")
                    .voice_enrollment_session_id
            }
            _ => panic!("expected voice enroll start ok"),
        };

        let commit_sample = |store: &mut Ph1fStore,
                             exec: &SimulationExecutor,
                             attempt_index: u16,
                             idempotency_key: &str| {
            exec.execute_voice_id(
                store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1002),
                    turn_id: TurnId(2 + u64::from(attempt_index)),
                    now: MonotonicTimeNs(10 + u64::from(attempt_index)),
                    simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
                    simulation_type: VoiceIdSimulationType::Commit,
                    request: VoiceIdSimulationRequest::EnrollSampleCommit(
                        VoiceIdEnrollSampleCommitRequest {
                            voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                voice_enrollment_session_id.as_str(),
                            )
                            .unwrap(),
                            audio_sample_ref: format!("audio:voice:{attempt_index}"),
                            attempt_index,
                            sample_duration_ms: 1_350,
                            vad_coverage: 0.93,
                            snr_db: 18.0,
                            clipping_pct: 0.4,
                            overlap_ratio: 0.0,
                            app_embedding_capture_ref: None,
                            idempotency_key: idempotency_key.to_string(),
                        },
                    ),
                },
            )
            .unwrap();
        };
        commit_sample(&mut store, &exec, 1, "vid-sync-worker-sample-1");
        commit_sample(&mut store, &exec, 2, "vid-sync-worker-sample-2");

        let complete = exec
            .execute_voice_id(
                &mut store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1002),
                    turn_id: TurnId(9),
                    now: MonotonicTimeNs(20),
                    simulation_id: VOICE_ID_ENROLL_COMPLETE_COMMIT.to_string(),
                    simulation_type: VoiceIdSimulationType::Commit,
                    request: VoiceIdSimulationRequest::EnrollCompleteCommit(
                        VoiceIdEnrollCompleteCommitRequest {
                            voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                voice_enrollment_session_id.as_str(),
                            )
                            .unwrap(),
                            idempotency_key: "vid-sync-worker-complete".to_string(),
                        },
                    ),
                },
            )
            .unwrap();

        let receipt = match complete {
            Ph1VoiceIdSimResponse::Ok(ok) => ok
                .enroll_complete_result
                .expect("complete result must exist")
                .voice_artifact_sync_receipt_ref
                .expect("voice sync receipt must exist"),
            _ => panic!("expected voice enroll complete ok"),
        };

        let row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("queue row must exist");
        assert_eq!(row.state, MobileArtifactSyncState::Acked);
        assert!(row.acked_at.is_some());
    }

    #[test]
    fn at_sim_exec_04c_voice_complete_sync_failure_stays_inflight_for_retry() {
        let mut store = Ph1fStore::new_in_memory();
        let mut exec = SimulationExecutor::default();
        exec.set_device_sync_sender(
            crate::device_artifact_sync::DeviceArtifactSyncSenderRuntime::always_fail_for_tests(
                "engine_b_unreachable",
                5_000,
            ),
        );

        let actor = UserId::new("voice-actor-3").unwrap();
        let device_id = DeviceId::new("voice-device-3").unwrap();

        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
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
                    actor.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    Some("audio_prof_vid_3".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let (link_rec, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(2),
                actor.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        let _ = store
            .ph1link_invite_open_activate_commit(
                MonotonicTimeNs(3),
                link_rec.token_id.clone(),
                "voice-device-fp-3".to_string(),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(4),
                link_rec.token_id,
                None,
                Some("tenant_1".to_string()),
                "voice-device-fp-3".to_string(),
                selene_kernel_contracts::ph1link::AppPlatform::Ios,
                "legacy_app_instance".to_string(),
                "legacy_nonce_3".to_string(),
                MonotonicTimeNs(3),
            )
            .unwrap();

        let start = exec
            .execute_voice_id(
                &mut store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1003),
                    turn_id: TurnId(1),
                    now: MonotonicTimeNs(10),
                    simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
                    simulation_type: VoiceIdSimulationType::Draft,
                    request: VoiceIdSimulationRequest::EnrollStartDraft(
                        VoiceIdEnrollStartDraftRequest {
                            onboarding_session_id: onb.onboarding_session_id.as_str().to_string(),
                            device_id: device_id.clone(),
                            consent_asserted: true,
                            max_total_attempts: 8,
                            max_session_enroll_time_ms: 120_000,
                            lock_after_consecutive_passes: 2,
                        },
                    ),
                },
            )
            .unwrap();
        let voice_enrollment_session_id = match start {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                ok.enroll_start_result
                    .expect("start result must exist")
                    .voice_enrollment_session_id
            }
            _ => panic!("expected voice enroll start ok"),
        };

        let commit_sample = |store: &mut Ph1fStore,
                             exec: &SimulationExecutor,
                             attempt_index: u16,
                             idempotency_key: &str| {
            exec.execute_voice_id(
                store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1003),
                    turn_id: TurnId(2 + u64::from(attempt_index)),
                    now: MonotonicTimeNs(10 + u64::from(attempt_index)),
                    simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
                    simulation_type: VoiceIdSimulationType::Commit,
                    request: VoiceIdSimulationRequest::EnrollSampleCommit(
                        VoiceIdEnrollSampleCommitRequest {
                            voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                voice_enrollment_session_id.as_str(),
                            )
                            .unwrap(),
                            audio_sample_ref: format!("audio:voice:{attempt_index}"),
                            attempt_index,
                            sample_duration_ms: 1_350,
                            vad_coverage: 0.93,
                            snr_db: 18.0,
                            clipping_pct: 0.4,
                            overlap_ratio: 0.0,
                            app_embedding_capture_ref: None,
                            idempotency_key: idempotency_key.to_string(),
                        },
                    ),
                },
            )
            .unwrap();
        };
        commit_sample(&mut store, &exec, 1, "vid-sync-worker-fail-sample-1");
        commit_sample(&mut store, &exec, 2, "vid-sync-worker-fail-sample-2");

        let complete = exec
            .execute_voice_id(
                &mut store,
                &Ph1VoiceIdSimRequest {
                    schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                    correlation_id: CorrelationId(1003),
                    turn_id: TurnId(9),
                    now: MonotonicTimeNs(20),
                    simulation_id: VOICE_ID_ENROLL_COMPLETE_COMMIT.to_string(),
                    simulation_type: VoiceIdSimulationType::Commit,
                    request: VoiceIdSimulationRequest::EnrollCompleteCommit(
                        VoiceIdEnrollCompleteCommitRequest {
                            voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                                voice_enrollment_session_id.as_str(),
                            )
                            .unwrap(),
                            idempotency_key: "vid-sync-worker-fail-complete".to_string(),
                        },
                    ),
                },
            )
            .unwrap();

        let receipt = match complete {
            Ph1VoiceIdSimResponse::Ok(ok) => ok
                .enroll_complete_result
                .expect("complete result must exist")
                .voice_artifact_sync_receipt_ref
                .expect("voice sync receipt must exist"),
            _ => panic!("expected voice enroll complete ok"),
        };

        let row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("queue row must exist");
        assert_eq!(row.state, MobileArtifactSyncState::InFlight);
        assert_eq!(row.acked_at, None);
        assert_eq!(row.last_error.as_deref(), Some("engine_b_unreachable"));
    }

    #[test]
    fn at_sim_exec_04d_resolve_voice_identity_carries_prompt_scope_key() {
        let mut store = Ph1fStore::new_in_memory();
        let os_top_level = new_os_top_level_wiring();
        let voice_id_live = Ph1VoiceIdLiveRuntime::default();
        let actor_user_id = UserId::new("tenant_1:voice_scope_actor").unwrap();

        let first = resolve_voice_identity_assertion(
            &os_top_level,
            &voice_id_live,
            &mut store,
            &actor_user_id,
            MonotonicTimeNs(5),
            CorrelationId(8701),
            TurnId(1),
        )
        .unwrap();
        let second = resolve_voice_identity_assertion(
            &os_top_level,
            &voice_id_live,
            &mut store,
            &actor_user_id,
            MonotonicTimeNs(6),
            CorrelationId(8701),
            TurnId(2),
        )
        .unwrap();

        let first_key = first
            .identity_prompt_scope_key
            .expect("scope key must be present");
        let second_key = second
            .identity_prompt_scope_key
            .expect("scope key must be present");
        assert!(first_key.starts_with("vidscope:v1:"));
        assert_eq!(first_key, second_key);
    }

    #[test]
    fn at_sim_exec_14_access_step_up_returns_continue_and_emits_start_finish_audit() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();
        let actor = UserId::new("tenant_1:stepup_actor_1").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let x = access_step_up_x(
            1,
            access_draft(
                IntentType::AccessSchemaManage,
                vec![access_field(FieldKey::TenantId, "tenant_1")],
            ),
            "ACCESS_SCHEMA_MANAGE",
            StepUpChallengeMethod::DevicePasscode,
            "idem-stepup-continue",
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(2), &x)
            .unwrap();
        match out {
            SimulationDispatchOutcome::AccessStepUp {
                outcome,
                challenge_method,
                ..
            } => {
                assert_eq!(outcome, StepUpOutcome::Continue);
                assert_eq!(challenge_method, StepUpChallengeMethod::DevicePasscode);
            }
            _ => panic!("expected AccessStepUp outcome"),
        }

        let audit_rows = store.ph1access_capreq_audit_rows(CorrelationId(10));
        assert_eq!(audit_rows.len(), 2);
    }

    #[test]
    fn at_sim_exec_15_access_step_up_returns_refuse_for_access_deny() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();
        let actor = UserId::new("tenant_1:stepup_actor_2").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.read_only",
            "{\"allow\":[\"LINK_INVITE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let x = access_step_up_x(
            2,
            access_draft(
                IntentType::AccessSchemaManage,
                vec![access_field(FieldKey::TenantId, "tenant_1")],
            ),
            "ACCESS_SCHEMA_MANAGE",
            StepUpChallengeMethod::DevicePasscode,
            "idem-stepup-refuse",
        );

        let out = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(2), &x)
            .unwrap();
        match out {
            SimulationDispatchOutcome::AccessStepUp { outcome, .. } => {
                assert_eq!(outcome, StepUpOutcome::Refuse);
            }
            _ => panic!("expected AccessStepUp outcome"),
        }
    }

    #[test]
    fn at_sim_exec_16_access_step_up_biometric_requirement_defers_and_is_replay_stable() {
        let mut store = Ph1fStore::new_in_memory();
        let exec = SimulationExecutor::default();
        let actor = UserId::new("tenant_1:stepup_actor_3").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                actor.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        seed_access_instance_with_permissions(
            &mut store,
            &actor,
            "tenant_1",
            "role.access_admin",
            "{\"allow\":[\"ACCESS_SCHEMA_MANAGE\"]}",
            AccessMode::A,
            true,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
        );

        let x = access_step_up_x(
            3,
            access_draft(
                IntentType::AccessSchemaManage,
                vec![access_field(FieldKey::TenantId, "tenant_1")],
            ),
            "ACCESS_SCHEMA_MANAGE",
            StepUpChallengeMethod::DeviceBiometric,
            "idem-stepup-defer",
        );

        let out1 = exec
            .execute_ph1x_dispatch_simulation_candidate(
                &mut store,
                actor.clone(),
                MonotonicTimeNs(3),
                &x,
            )
            .unwrap();
        let out2 = exec
            .execute_ph1x_dispatch_simulation_candidate(&mut store, actor, MonotonicTimeNs(3), &x)
            .unwrap();
        assert_eq!(out1, out2);

        let step_up_result = SimulationExecutor::step_up_result_from_dispatch_outcome(&out1)
            .expect("step-up outcome should map to step-up result");
        assert_eq!(step_up_result.outcome, StepUpOutcome::Defer);
        assert_eq!(
            step_up_result.challenge_method,
            StepUpChallengeMethod::DeviceBiometric
        );
    }
}
