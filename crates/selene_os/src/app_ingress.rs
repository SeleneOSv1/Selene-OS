#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use selene_engines::ph1_voice_id::{
    reason_codes as voice_id_reason_codes, simulation_profile_embedding_from_seed,
    EnrolledSpeaker as EngineEnrolledSpeaker, VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_engines::ph1e::{Ph1eConfig, Ph1eRuntime};
use selene_engines::ph1emocore::{
    Ph1EmoCoreConfig as EnginePh1EmoCoreConfig, Ph1EmoCoreRuntime as EnginePh1EmoCoreRuntime,
};
use selene_engines::ph1emoguide::{
    Ph1EmoGuideConfig as EnginePh1EmoGuideConfig, Ph1EmoGuideRuntime as EnginePh1EmoGuideRuntime,
};
use selene_engines::ph1persona::{
    Ph1PersonaConfig as EnginePh1PersonaConfig, Ph1PersonaRuntime as EnginePh1PersonaRuntime,
};
use selene_engines::ph1simfinder::{
    FinderFieldSpec, FinderGoldMapping, FinderRunRequest, FinderRuntimeConfig,
    FinderSimulationCatalogEntry, Ph1SimFinderRuntime,
};
use selene_kernel_contracts::ph1_voice_id::{
    IdentityTierV2, Ph1VoiceIdRequest, Ph1VoiceIdResponse, SpeakerId, SpoofLivenessStatus, UserId,
    VoiceEmbeddingCaptureRef, VOICE_ID_ENROLL_COMPLETE_COMMIT, VOICE_ID_ENROLL_SAMPLE_COMMIT,
    VOICE_ID_ENROLL_START_DRAFT,
};
use selene_kernel_contracts::ph1agent::AgentInputPacket;
use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
use selene_kernel_contracts::ph1e::{
    CacheStatus, SourceMetadata, SourceRef, ToolName, ToolRequest, ToolResponse, ToolResult,
    ToolStructuredField, ToolTextSnippet,
};
use selene_kernel_contracts::ph1emocore::{
    EmoClassifyProfileCommitRequest, EmoCoreOutcome, EmoCoreRequest, EmoCoreSimulationType,
    EmoPersonalityType, EmoSignalBundle, Ph1EmoCoreRequest, Ph1EmoCoreResponse, EMO_SIM_001,
    PH1EMOCORE_CONTRACT_VERSION,
};
use selene_kernel_contracts::ph1emoguide::{
    EmoGuideInteractionSignals, EmoGuideProfileBuildRequest, EmoGuideProfileValidateRequest,
    EmoGuideRequestEnvelope, EmoGuideValidationStatus, Ph1EmoGuideRequest, Ph1EmoGuideResponse,
};
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventId, CorrelationId, DeviceId, PayloadKey, ProofFailureClass,
    ProofProtectedActionClass, ProofRetentionClass, TurnId,
};
use selene_kernel_contracts::ph1k::InterruptCandidate;
use selene_kernel_contracts::ph1link::{
    AppPlatform, InviteeType, LinkStatus, Ph1LinkRequest, Ph1LinkResponse, TokenId,
    LINK_INVITE_DRAFT_UPDATE_COMMIT, LINK_INVITE_OPEN_ACTIVATE_COMMIT,
};
use selene_kernel_contracts::ph1m::{MemoryCandidate, MemoryConfidence};
use selene_kernel_contracts::ph1n::{FieldKey, IntentDraft, IntentType, Ph1nResponse};
use selene_kernel_contracts::ph1onb::{
    OnbAccessInstanceCreateCommitRequest, OnbCompleteCommitRequest,
    OnbEmployeePhotoCaptureSendCommitRequest, OnbEmployeeSenderVerifyCommitRequest,
    OnbPrimaryDeviceConfirmCommitRequest, OnbRequest, OnbTermsAcceptCommitRequest,
    OnboardingNextStep, OnboardingSessionId, OnboardingStatus, Ph1OnbRequest, Ph1OnbResponse,
    ProofType, SenderVerifyDecision, SimulationType, TermsStatus, VerificationStatus,
    ONB_ACCESS_INSTANCE_CREATE_COMMIT, ONB_COMPLETE_COMMIT, ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT,
    ONB_EMPLOYEE_SENDER_VERIFY_COMMIT, ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, ONB_SESSION_START_DRAFT,
    ONB_TERMS_ACCEPT_COMMIT,
};
use selene_kernel_contracts::ph1persona::{
    PersonaDeliveryPolicyRef, PersonaPreferenceKey, PersonaPreferenceSignal,
    PersonaProfileValidateRequest, PersonaRequestEnvelope, PersonaValidationStatus,
    Ph1PersonaRequest, Ph1PersonaResponse,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1simcat::{SimulationId, SimulationStatus, SimulationVersion};
use selene_kernel_contracts::ph1simfinder::{
    reason_codes as sim_finder_reason_codes, FinderFallbackPolicy, FinderRiskTier,
    FinderTerminalPacket,
};
use selene_kernel_contracts::ph1tts::StyleProfileRef;
use selene_kernel_contracts::ph1w::{
    Ph1wRequest, Ph1wResponse, WakeEnrollCompleteCommitRequest, WakeEnrollDeferCommitRequest,
    WakeEnrollSampleCommitRequest, WakeEnrollStartDraftRequest,
    WakeEnrollStatus as ContractWakeEnrollStatus, WakeEnrollmentSessionId, WakeRequest,
    WakeSampleResult, WakeSimulationType, PH1W_CONTRACT_VERSION, WAKE_ENROLL_COMPLETE_COMMIT,
    WAKE_ENROLL_DEFER_COMMIT, WAKE_ENROLL_SAMPLE_COMMIT, WAKE_ENROLL_START_DRAFT,
};
use selene_kernel_contracts::ph1x::{
    ConfirmAnswer, DispatchRequest, IdentityContext, PendingState, Ph1xDirective, Ph1xRequest,
    Ph1xResponse, StepUpCapabilities, ThreadState,
};
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, AuthorityExecutionState, AuthorityPolicyDecision, IdentityExecutionState,
    IdentityRecoveryState, IdentityTrustTier, IdentityVerificationConsistencyLevel,
    MemoryConsistencyLevel, MemoryEligibilityDecision, MemoryExecutionState, MemoryTrustLevel,
    OnboardingReadinessState, PlatformRuntimeContext, ProofExecutionState, RuntimeEntryTrigger,
    RuntimeExecutionEnvelope, SimulationCertificationState,
};
use selene_kernel_contracts::runtime_governance::{
    GovernanceClusterConsistency, GovernanceDecisionLogEntry, GovernanceDriftSignal,
    GovernanceExecutionState, GovernanceProtectedActionClass,
};
use selene_kernel_contracts::runtime_law::{
    RuntimeLawDecisionLogEntry, RuntimeLawEvaluationContext, RuntimeLawResponseClass,
    RuntimeProtectedActionClass,
};
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};
use selene_storage::ph1f::{
    AgentExecutionLedgerRowInput, BcastPolicyLedgerRow, BcastPolicySettingKey, DeviceRecord,
    IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
};

use crate::device_artifact_sync::DeviceArtifactSyncWorkerPassMetrics;
use crate::ph1comp::Ph1CompRuntime;
use crate::ph1j::{Ph1jRuntime, ProtectedProofWriteRequest};
use crate::ph1onb::{OnbVoiceEnrollFinalize, OnbVoiceEnrollLiveRequest, OnbVoiceEnrollSampleStep};
use crate::ph1os::{
    OsTopLevelTurnInput, OsTopLevelTurnPath, OsTurnInput, OsVoiceLiveTurnInput,
    OsVoiceLiveTurnOutcome, OsVoicePlatform, OsVoiceTrigger, OsVoiceTurnContext,
};
use crate::ph1w::Ph1wRuntime;
use crate::ph1x::{Ph1xConfig, Ph1xRuntime};
use crate::runtime_governance::{RuntimeGovernanceDecision, RuntimeGovernanceRuntime};
use crate::runtime_law::{RuntimeLawDecision, RuntimeLawRuntime};
use crate::simulation_executor::{
    simulation_id_for_intent_draft_v1, SimulationDispatchOutcome, SimulationExecutor,
};

#[derive(Debug, Clone)]
pub struct AppVoiceIngressRequest {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub runtime_execution_envelope: RuntimeExecutionEnvelope,
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
        let runtime_execution_envelope = fallback_runtime_execution_envelope_for_app_voice_request(
            correlation_id,
            turn_id,
            app_platform,
            trigger,
            &voice_id_request,
            &actor_user_id,
            device_id.as_ref(),
        )?;
        Self::v1_with_runtime_execution_envelope(
            correlation_id,
            turn_id,
            runtime_execution_envelope,
            app_platform,
            trigger,
            voice_id_request,
            actor_user_id,
            tenant_id,
            device_id,
            enrolled_speakers,
            observation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_runtime_execution_envelope(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        runtime_execution_envelope: RuntimeExecutionEnvelope,
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
        runtime_execution_envelope.validate()?;
        if let Some(device_id) = device_id.as_ref() {
            device_id.validate()?;
            if runtime_execution_envelope.device_identity != *device_id {
                return Err(ContractViolation::InvalidValue {
                    field: "app_voice_ingress_request.runtime_execution_envelope.device_identity",
                    reason: "must match app_voice_ingress_request.device_id",
                });
            }
        }
        if runtime_execution_envelope.actor_identity != actor_user_id {
            return Err(ContractViolation::InvalidValue {
                field: "app_voice_ingress_request.runtime_execution_envelope.actor_identity",
                reason: "must match app_voice_ingress_request.actor_user_id",
            });
        }
        if runtime_execution_envelope.platform != app_platform {
            return Err(ContractViolation::InvalidValue {
                field: "app_voice_ingress_request.runtime_execution_envelope.platform",
                reason: "must match app_voice_ingress_request.app_platform",
            });
        }
        if runtime_execution_envelope.turn_id != turn_id {
            return Err(ContractViolation::InvalidValue {
                field: "app_voice_ingress_request.runtime_execution_envelope.turn_id",
                reason: "must match app_voice_ingress_request.turn_id",
            });
        }
        if runtime_execution_envelope.session_id != voice_id_request.session_state_ref.session_id {
            return Err(ContractViolation::InvalidValue {
                field: "app_voice_ingress_request.runtime_execution_envelope.session_id",
                reason: "must match app_voice_ingress_request.voice_id_request.session_state_ref.session_id",
            });
        }
        Ok(Self {
            correlation_id,
            turn_id,
            runtime_execution_envelope,
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

fn fallback_runtime_execution_envelope_for_app_voice_request(
    correlation_id: CorrelationId,
    turn_id: TurnId,
    app_platform: AppPlatform,
    trigger: OsVoiceTrigger,
    voice_id_request: &Ph1VoiceIdRequest,
    actor_user_id: &UserId,
    device_id: Option<&DeviceId>,
) -> Result<RuntimeExecutionEnvelope, ContractViolation> {
    let fallback_device_id = match device_id {
        Some(device_id) => device_id.clone(),
        None => DeviceId::new(format!("app_ingress_device_{}", correlation_id.0))?,
    };
    RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
        format!("corr-{}", correlation_id.0),
        format!("trace:voice:{}:{}", correlation_id.0, turn_id.0),
        format!("turn:{}:{}", correlation_id.0, turn_id.0),
        actor_user_id.clone(),
        fallback_device_id,
        app_platform,
        PlatformRuntimeContext::default_for_platform_and_trigger(
            app_platform,
            match trigger {
                OsVoiceTrigger::Explicit => RuntimeEntryTrigger::Explicit,
                OsVoiceTrigger::WakeWord => RuntimeEntryTrigger::WakeWord,
            },
        )?,
        voice_id_request.session_state_ref.session_id,
        turn_id,
        Some(turn_id.0),
        AdmissionState::SessionResolved,
        None,
    )
}

#[derive(Debug, Clone)]
pub struct AppInviteLinkOpenRequest {
    pub correlation_id: CorrelationId,
    pub idempotency_key: String,
    pub token_id: TokenId,
    pub token_signature: String,
    pub tenant_id: Option<String>,
    pub app_platform: AppPlatform,
    pub device_fingerprint: String,
    pub app_instance_id: String,
    pub deep_link_nonce: String,
}

impl AppInviteLinkOpenRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        idempotency_key: String,
        token_id: TokenId,
        token_signature: String,
        tenant_id: Option<String>,
        app_platform: AppPlatform,
        device_fingerprint: String,
        app_instance_id: String,
        deep_link_nonce: String,
    ) -> Result<Self, ContractViolation> {
        correlation_id.validate()?;
        token_id.validate()?;
        app_platform.validate()?;
        validate_ascii_token(
            "app_invite_link_open_request.idempotency_key",
            &idempotency_key,
            128,
        )?;
        validate_ascii_token(
            "app_invite_link_open_request.token_signature",
            &token_signature,
            192,
        )?;
        validate_ascii_token(
            "app_invite_link_open_request.device_fingerprint",
            &device_fingerprint,
            256,
        )?;
        validate_ascii_token(
            "app_invite_link_open_request.app_instance_id",
            &app_instance_id,
            128,
        )?;
        validate_ascii_token(
            "app_invite_link_open_request.deep_link_nonce",
            &deep_link_nonce,
            128,
        )?;
        if let Some(tenant_id) = tenant_id.as_ref() {
            validate_ascii_token("app_invite_link_open_request.tenant_id", tenant_id, 64)?;
        }
        Ok(Self {
            correlation_id,
            idempotency_key,
            token_id,
            token_signature,
            tenant_id,
            app_platform,
            device_fingerprint,
            app_instance_id,
            deep_link_nonce,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInviteLinkOpenOutcome {
    pub onboarding_session_id: String,
    pub next_step: OnboardingNextStep,
    pub required_fields: Vec<String>,
    pub required_verification_gates: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AppOnboardingContinueAction {
    AskMissingSubmit {
        field_value: Option<String>,
    },
    PlatformSetupReceipt {
        receipt_kind: String,
        receipt_ref: String,
        signer: String,
        payload_hash: String,
    },
    TermsAccept {
        terms_version_id: String,
        accepted: bool,
    },
    PrimaryDeviceConfirm {
        device_id: DeviceId,
        proof_ok: bool,
    },
    VoiceEnrollLock {
        device_id: DeviceId,
        sample_seed: String,
    },
    WakeEnrollStartDraft {
        device_id: DeviceId,
    },
    WakeEnrollSampleCommit {
        device_id: DeviceId,
        sample_pass: bool,
    },
    WakeEnrollCompleteCommit {
        device_id: DeviceId,
    },
    WakeEnrollDeferCommit {
        device_id: DeviceId,
    },
    EmployeePhotoCaptureSend {
        photo_blob_ref: String,
    },
    EmployeeSenderVerifyCommit {
        decision: SenderVerifyDecision,
    },
    EmoPersonaLock,
    AccessProvisionCommit,
    CompleteCommit,
}

#[derive(Debug, Clone)]
pub struct AppOnboardingContinueRequest {
    pub correlation_id: CorrelationId,
    pub onboarding_session_id: OnboardingSessionId,
    pub idempotency_key: String,
    pub tenant_id: Option<String>,
    pub action: AppOnboardingContinueAction,
}

impl AppOnboardingContinueRequest {
    pub fn v1(
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        idempotency_key: String,
        tenant_id: Option<String>,
        action: AppOnboardingContinueAction,
    ) -> Result<Self, ContractViolation> {
        correlation_id.validate()?;
        onboarding_session_id.validate()?;
        validate_ascii_token(
            "app_onboarding_continue_request.idempotency_key",
            &idempotency_key,
            128,
        )?;
        if let Some(tenant_id) = tenant_id.as_ref() {
            validate_ascii_token("app_onboarding_continue_request.tenant_id", tenant_id, 64)?;
        }
        match &action {
            AppOnboardingContinueAction::AskMissingSubmit { field_value } => {
                if let Some(value) = field_value.as_ref() {
                    validate_ascii_token(
                        "app_onboarding_continue_request.ask_missing.field_value",
                        value,
                        256,
                    )?;
                }
            }
            AppOnboardingContinueAction::TermsAccept {
                terms_version_id, ..
            } => validate_ascii_token(
                "app_onboarding_continue_request.terms_accept.terms_version_id",
                terms_version_id,
                64,
            )?,
            AppOnboardingContinueAction::PlatformSetupReceipt {
                receipt_kind,
                receipt_ref,
                signer,
                payload_hash,
            } => {
                validate_ascii_token(
                    "app_onboarding_continue_request.platform_setup_receipt.receipt_kind",
                    receipt_kind,
                    64,
                )?;
                validate_ascii_token(
                    "app_onboarding_continue_request.platform_setup_receipt.receipt_ref",
                    receipt_ref,
                    192,
                )?;
                validate_ascii_token(
                    "app_onboarding_continue_request.platform_setup_receipt.signer",
                    signer,
                    64,
                )?;
                validate_ascii_token(
                    "app_onboarding_continue_request.platform_setup_receipt.payload_hash",
                    payload_hash,
                    64,
                )?;
            }
            AppOnboardingContinueAction::PrimaryDeviceConfirm { device_id, .. } => {
                device_id.validate()?;
            }
            AppOnboardingContinueAction::VoiceEnrollLock {
                device_id,
                sample_seed,
            } => {
                device_id.validate()?;
                validate_ascii_token(
                    "app_onboarding_continue_request.voice_enroll_lock.sample_seed",
                    sample_seed,
                    64,
                )?;
            }
            AppOnboardingContinueAction::WakeEnrollStartDraft { device_id }
            | AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id }
            | AppOnboardingContinueAction::WakeEnrollDeferCommit { device_id } => {
                device_id.validate()?;
            }
            AppOnboardingContinueAction::WakeEnrollSampleCommit { device_id, .. } => {
                device_id.validate()?;
            }
            AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } => {
                validate_ascii_token(
                    "app_onboarding_continue_request.employee_photo_capture_send.photo_blob_ref",
                    photo_blob_ref,
                    192,
                )?;
            }
            AppOnboardingContinueAction::EmployeeSenderVerifyCommit { .. } => {}
            AppOnboardingContinueAction::EmoPersonaLock => {}
            AppOnboardingContinueAction::AccessProvisionCommit => {}
            AppOnboardingContinueAction::CompleteCommit => {}
        }
        Ok(Self {
            correlation_id,
            onboarding_session_id,
            idempotency_key,
            tenant_id,
            action,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppOnboardingContinueNextStep {
    AskMissing,
    PlatformSetup,
    Terms,
    PrimaryDeviceConfirm,
    VoiceEnroll,
    WakeEnroll,
    SenderVerification,
    EmoPersonaLock,
    AccessProvision,
    Complete,
    Ready,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppOnboardingContinueOutcome {
    pub onboarding_session_id: String,
    pub next_step: AppOnboardingContinueNextStep,
    pub blocking_field: Option<String>,
    pub blocking_question: Option<String>,
    pub remaining_missing_fields: Vec<String>,
    pub remaining_platform_receipt_kinds: Vec<String>,
    pub voice_artifact_sync_receipt_ref: Option<String>,
    pub access_engine_instance_id: Option<String>,
    pub onboarding_status: Option<OnboardingStatus>,
}

#[derive(Debug, Clone)]
pub struct AppVoicePh1xBuildInput {
    pub now: MonotonicTimeNs,
    pub thread_key: Option<String>,
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
    pub runtime_execution_envelope: RuntimeExecutionEnvelope,
    pub voice_outcome: OsVoiceLiveTurnOutcome,
    pub session_state: SessionState,
    pub next_move: AppVoiceTurnNextMove,
    pub ph1x_request: Option<Ph1xRequest>,
    pub ph1x_response: Option<Ph1xResponse>,
    pub dispatch_outcome: Option<SimulationDispatchOutcome>,
    pub tool_response: Option<ToolResponse>,
    pub response_text: Option<String>,
    pub reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone)]
pub struct AppServerIngressRuntime {
    executor: SimulationExecutor,
    ph1x_runtime: Ph1xRuntime,
    ph1e_runtime: Ph1eRuntime,
    ph1simfinder_runtime: Ph1SimFinderRuntime,
    ph1j_runtime: Ph1jRuntime,
    ph1comp_runtime: Ph1CompRuntime,
    runtime_governance: RuntimeGovernanceRuntime,
    runtime_law: RuntimeLawRuntime,
    agent_input_packet_build_count: RefCell<u64>,
    last_agent_input_packet: RefCell<Option<AgentInputPacket>>,
    last_finder_terminal_packet: RefCell<Option<FinderTerminalPacket>>,
}

impl Default for AppServerIngressRuntime {
    fn default() -> Self {
        Self::new(SimulationExecutor::default())
    }
}

impl AppServerIngressRuntime {
    pub fn new(executor: SimulationExecutor) -> Self {
        Self::new_with_runtime_governance_and_ph1j(
            executor,
            RuntimeGovernanceRuntime::default(),
            Ph1jRuntime::default(),
        )
    }

    pub fn new_with_runtime_governance(
        executor: SimulationExecutor,
        runtime_governance: RuntimeGovernanceRuntime,
    ) -> Self {
        Self::new_with_runtime_governance_and_ph1j(
            executor,
            runtime_governance,
            Ph1jRuntime::default(),
        )
    }

    pub fn new_with_runtime_governance_and_ph1j(
        executor: SimulationExecutor,
        runtime_governance: RuntimeGovernanceRuntime,
        ph1j_runtime: Ph1jRuntime,
    ) -> Self {
        Self::new_with_runtime_governance_ph1j_and_law(
            executor,
            runtime_governance,
            ph1j_runtime,
            RuntimeLawRuntime::default(),
        )
    }

    pub fn new_with_runtime_governance_ph1j_and_law(
        executor: SimulationExecutor,
        runtime_governance: RuntimeGovernanceRuntime,
        ph1j_runtime: Ph1jRuntime,
        runtime_law: RuntimeLawRuntime,
    ) -> Self {
        Self {
            executor,
            ph1x_runtime: Ph1xRuntime::new(Ph1xConfig::mvp_v1()),
            ph1e_runtime: Ph1eRuntime::new(Ph1eConfig::mvp_v1()),
            ph1simfinder_runtime: Ph1SimFinderRuntime::new(FinderRuntimeConfig::mvp_v1()),
            ph1j_runtime,
            ph1comp_runtime: Ph1CompRuntime,
            runtime_governance,
            runtime_law,
            agent_input_packet_build_count: RefCell::new(0),
            last_agent_input_packet: RefCell::new(None),
            last_finder_terminal_packet: RefCell::new(None),
        }
    }

    pub fn runtime_governance(&self) -> &RuntimeGovernanceRuntime {
        &self.runtime_governance
    }

    pub fn runtime_law(&self) -> &RuntimeLawRuntime {
        &self.runtime_law
    }

    pub fn ph1j_runtime(&self) -> &Ph1jRuntime {
        &self.ph1j_runtime
    }

    pub fn runtime_governance_policy_version(&self) -> &str {
        self.runtime_governance.policy_version()
    }

    pub fn runtime_governance_decision_log_snapshot(&self) -> Vec<GovernanceDecisionLogEntry> {
        self.runtime_governance.decision_log_snapshot()
    }

    fn governance_reason_code_for_state(
        &self,
        governance_state: &GovernanceExecutionState,
    ) -> Option<String> {
        let sequence = governance_state
            .decision_log_ref
            .as_deref()?
            .strip_prefix("gov_decision_")?
            .parse::<u64>()
            .ok()?;
        self.runtime_governance_decision_log_snapshot()
            .into_iter()
            .find(|entry| entry.sequence == sequence)
            .map(|entry| entry.reason_code)
    }

    fn ph1x_fail_closed_respond_payload_metadata(
        &self,
        runtime_execution_envelope: &RuntimeExecutionEnvelope,
    ) -> BTreeMap<String, String> {
        let mut payload_metadata = BTreeMap::new();
        let Some(identity_state) = runtime_execution_envelope.identity_state.as_ref() else {
            return payload_metadata;
        };

        payload_metadata.insert(
            "identity_consistency_level".to_string(),
            identity_consistency_level_literal(identity_state.consistency_level).to_string(),
        );
        payload_metadata.insert(
            "identity_trust_tier".to_string(),
            identity_trust_tier_literal(identity_state.trust_tier).to_string(),
        );
        payload_metadata.insert(
            "identity_recovery_state".to_string(),
            identity_recovery_state_literal(identity_state.recovery_state).to_string(),
        );
        payload_metadata.insert(
            "identity_tier_v2".to_string(),
            identity_tier_v2_literal(identity_state.identity_tier_v2).to_string(),
        );
        payload_metadata.insert(
            "identity_spoof_liveness_status".to_string(),
            spoof_liveness_status_literal(identity_state.spoof_liveness_status).to_string(),
        );
        payload_metadata.insert(
            "identity_step_up_required".to_string(),
            identity_state.step_up_required.to_string(),
        );
        payload_metadata.insert(
            "identity_cluster_drift_detected".to_string(),
            identity_state.cluster_drift_detected.to_string(),
        );
        if let Some(reason_code) = identity_reason_code(identity_state) {
            payload_metadata.insert(
                "identity_reason_code_hex".to_string(),
                format!("0x{:X}", reason_code.0),
            );
        }
        if let Some(governance_state) = runtime_execution_envelope.governance_state.as_ref() {
            if let Some(governance_reason_code) =
                self.governance_reason_code_for_state(governance_state)
            {
                payload_metadata
                    .insert("governance_reason_code".to_string(), governance_reason_code);
            }
        }

        payload_metadata
    }

    pub fn runtime_law_policy_version(&self) -> &str {
        self.runtime_law.policy_version()
    }

    pub fn runtime_law_decision_log_snapshot(&self) -> Vec<RuntimeLawDecisionLogEntry> {
        self.runtime_law.decision_log_snapshot()
    }

    pub fn govern_persistence_signal(
        &self,
        envelope: Option<&RuntimeExecutionEnvelope>,
        action_class: GovernanceProtectedActionClass,
        signal_reason: &str,
        note: Option<String>,
    ) -> RuntimeGovernanceDecision {
        self.runtime_governance.govern_persistence_signal(
            envelope,
            action_class,
            signal_reason,
            note,
        )
    }

    pub fn observe_runtime_governance_node_policy_version(
        &self,
        node_id: &str,
        observed_policy_version: Option<&str>,
    ) -> RuntimeGovernanceDecision {
        self.runtime_governance
            .observe_node_policy_version(node_id, observed_policy_version)
    }

    fn emit_voice_turn_proof_and_attach(
        &self,
        store: &mut Ph1fStore,
        out: &AppVoiceTurnExecutionOutcome,
        finder_terminal: Option<&FinderTerminalPacket>,
        actor_tenant_id: Option<&str>,
        received_at: MonotonicTimeNs,
        executed_at: MonotonicTimeNs,
    ) -> Result<RuntimeExecutionEnvelope, StorageError> {
        let runtime_execution_envelope = &out.runtime_execution_envelope;
        let law_action_class = voice_turn_runtime_law_action_class(out);
        let law_context =
            RuntimeLawEvaluationContext::v1(None, None, None, None, executed_at, false)
                .map_err(StorageError::ContractViolation)?;
        let (simulation_id, simulation_version, simulation_certification_state) =
            proof_simulation_trace_for_voice_turn(store, out, finder_terminal, actor_tenant_id)?;
        let proof_request = ProtectedProofWriteRequest::v1(
            runtime_execution_envelope.clone(),
            voice_turn_protected_action_class(out),
            proof_authority_decision_reference(runtime_execution_envelope),
            proof_policy_rule_identifiers(runtime_execution_envelope),
            runtime_execution_envelope
                .governance_state
                .as_ref()
                .map(|state| state.governance_policy_version.clone()),
            simulation_id,
            simulation_version,
            simulation_certification_state,
            voice_turn_execution_outcome_token(out),
            voice_turn_failure_class_token(out),
            out.reason_code.into_iter().collect(),
            received_at,
            executed_at,
            ProofRetentionClass::ComplianceRetention,
            Some(proof_verifier_metadata_ref(runtime_execution_envelope)),
        )
        .map_err(StorageError::ContractViolation)?;
        match self.ph1j_runtime.emit_protected_proof(store, proof_request) {
            Ok(receipt) => {
                let proof_state = proof_execution_state_from_receipt(receipt)?;
                self.runtime_governance
                    .govern_protected_action_proof_state(
                        GovernanceProtectedActionClass::VoiceTurnExecution,
                        runtime_execution_envelope.session_id.map(|value| value.0),
                        Some(runtime_execution_envelope.turn_id.0),
                        &proof_state,
                    )
                    .map_err(|decision| {
                        runtime_governance_storage_error(
                            "app_voice_turn_execution_outcome.runtime_execution_envelope.proof_state",
                            &decision,
                        )
                    })?;
                let proof_envelope = runtime_execution_envelope_with_voice_turn_proof(
                    runtime_execution_envelope,
                    proof_state,
                )?;
                self.runtime_law
                    .govern_completion(&proof_envelope, law_action_class, &law_context)
                    .map_err(|decision| {
                        runtime_law_storage_error(
                            "app_voice_turn_execution_outcome.runtime_execution_envelope.law_state",
                            &decision,
                        )
                    })
            }
            Err(error) => {
                let proof_state = proof_execution_state_from_error(&error)?;
                let governance_decision = self
                    .runtime_governance
                    .govern_protected_action_proof_state(
                        GovernanceProtectedActionClass::VoiceTurnExecution,
                        runtime_execution_envelope.session_id.map(|value| value.0),
                        Some(runtime_execution_envelope.turn_id.0),
                        &proof_state,
                    )
                    .unwrap_err();
                let governed_envelope = runtime_execution_envelope
                    .with_proof_state(Some(proof_state))
                    .and_then(|value| {
                        value.with_governance_state(Some(
                            governance_decision.governance_state.clone(),
                        ))
                    })
                    .map_err(StorageError::ContractViolation)?;
                self.runtime_law
                    .govern_completion(&governed_envelope, law_action_class, &law_context)
                    .map_err(|decision| {
                        runtime_law_storage_error(
                            "app_voice_turn_execution_outcome.runtime_execution_envelope.law_state",
                            &decision,
                        )
                    })
            }
        }
    }

    pub fn run_voice_turn(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
    ) -> Result<OsVoiceLiveTurnOutcome, StorageError> {
        self.run_voice_turn_with_governed_envelope(store, request)
            .map(|(outcome, _)| outcome)
    }

    fn run_voice_turn_with_governed_envelope(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
    ) -> Result<(OsVoiceLiveTurnOutcome, RuntimeExecutionEnvelope), StorageError> {
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

        let governed_runtime_execution_envelope = self
            .runtime_governance
            .govern_voice_turn_execution(
                &request
                    .runtime_execution_envelope
                    .with_admission_state(AdmissionState::ExecutionAdmitted)
                    .map_err(StorageError::ContractViolation)?,
            )
            .map_err(|decision| {
                runtime_governance_storage_error(
                    "app_voice_ingress_request.runtime_execution_envelope",
                    &decision,
                )
            })?;

        let live_turn_input = OsVoiceLiveTurnInput::v1_with_runtime_execution_envelope(
            top_level_turn_input,
            governed_runtime_execution_envelope.clone(),
            request.voice_id_request,
            request.actor_user_id,
            request.tenant_id,
            request.device_id,
            resolved_enrolled_speakers,
            request.observation,
        )
        .map_err(StorageError::ContractViolation)?;

        let outcome = self
            .executor
            .execute_os_voice_live_turn(store, live_turn_input)?;
        match outcome {
            OsVoiceLiveTurnOutcome::Forwarded(forwarded) => Ok((
                OsVoiceLiveTurnOutcome::Forwarded(forwarded.clone()),
                forwarded.runtime_execution_envelope,
            )),
            OsVoiceLiveTurnOutcome::NotInvokedDisabled => Ok((
                OsVoiceLiveTurnOutcome::NotInvokedDisabled,
                governed_runtime_execution_envelope,
            )),
            OsVoiceLiveTurnOutcome::Refused(refuse) => Ok((
                OsVoiceLiveTurnOutcome::Refused(refuse),
                governed_runtime_execution_envelope,
            )),
        }
    }

    pub fn run_invite_link_open_and_start_onboarding(
        &self,
        store: &mut Ph1fStore,
        request: AppInviteLinkOpenRequest,
        now: MonotonicTimeNs,
    ) -> Result<AppInviteLinkOpenOutcome, StorageError> {
        if now.0 == 0 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_invite_link_open_request.now",
                    reason: "must be > 0",
                },
            ));
        }

        let link_record = store.ph1link_get_link(&request.token_id).cloned().ok_or(
            StorageError::ForeignKeyViolation {
                table: "links.token_id",
                key: request.token_id.as_str().to_string(),
            },
        )?;
        let link_tenant = link_record
            .prefilled_context
            .as_ref()
            .and_then(|ctx| ctx.tenant_id.as_ref())
            .map(|tenant| TenantId::new(tenant.to_string()))
            .transpose()
            .map_err(StorageError::ContractViolation)?;
        let request_tenant = request
            .tenant_id
            .as_ref()
            .map(|tenant| TenantId::new(tenant.to_string()))
            .transpose()
            .map_err(StorageError::ContractViolation)?;
        if let (Some(request_tenant), Some(link_tenant)) = (&request_tenant, &link_tenant) {
            if request_tenant != link_tenant {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_invite_link_open_request.tenant_id",
                        reason: "must match link tenant scope",
                    },
                ));
            }
        }
        let effective_tenant = request_tenant.or(link_tenant).ok_or({
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "app_invite_link_open_request.tenant_id",
                reason: "missing tenant scope for invite-open activation",
            })
        })?;
        self.executor.ensure_simulation_chain_active_for_tenant(
            store,
            &effective_tenant,
            &[LINK_INVITE_OPEN_ACTIVATE_COMMIT, ONB_SESSION_START_DRAFT],
            "app_invite_link_open_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;

        let turn_id = TurnId(1);
        let link_req = Ph1LinkRequest::invite_open_activate_commit_v1(
            request.correlation_id,
            turn_id,
            now,
            request.token_id.clone(),
            request.token_signature.clone(),
            request.device_fingerprint.clone(),
            request.app_platform,
            request.app_instance_id.clone(),
            request.deep_link_nonce.clone(),
            now,
            request.idempotency_key.clone(),
        )
        .map_err(StorageError::ContractViolation)?;
        let link_response = self.executor.execute_link(store, &link_req)?;
        let activation = match link_response {
            Ph1LinkResponse::Ok(ok) => ok.link_activation_result.ok_or({
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1link_response.link_activation_result",
                    reason: "invite-open activation result must be present",
                })
            })?,
            Ph1LinkResponse::Refuse(refuse) => {
                let _ = refuse;
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1link_response",
                        reason: "LINK_OPEN_ACTIVATE_REFUSED",
                    },
                ));
            }
        };
        if activation.activation_status != LinkStatus::Activated {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1link_response.link_activation_result.activation_status",
                    reason: "link open/activate must resolve to ACTIVATED",
                },
            ));
        }

        let onb_req = Ph1OnbRequest::session_start_draft_v1(
            request.correlation_id,
            turn_id,
            now,
            request.token_id,
            activation.prefilled_context_ref.clone(),
            Some(effective_tenant.as_str().to_string()),
            request.device_fingerprint,
            request.app_platform,
            request.app_instance_id,
            request.deep_link_nonce,
            now,
        )
        .map_err(StorageError::ContractViolation)?;
        let onb_response = self.executor.execute_onb(store, &onb_req)?;
        let session_start = match onb_response {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.ok_or({
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "ph1onb_response.session_start_result",
                    reason: "onboarding session start result must be present",
                })
            })?,
            Ph1OnbResponse::Refuse(refuse) => {
                let _ = refuse;
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_response",
                        reason: "ONB_SESSION_START_REFUSED",
                    },
                ));
            }
        };

        Ok(AppInviteLinkOpenOutcome {
            onboarding_session_id: session_start.onboarding_session_id.as_str().to_string(),
            next_step: session_start.next_step,
            required_fields: activation.missing_required_fields,
            required_verification_gates: session_start.required_verification_gates,
        })
    }

    pub fn run_onboarding_continue(
        &self,
        store: &mut Ph1fStore,
        request: AppOnboardingContinueRequest,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        if now.0 == 0 {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.now",
                    reason: "must be > 0",
                },
            ));
        }

        let AppOnboardingContinueRequest {
            correlation_id,
            onboarding_session_id,
            idempotency_key,
            tenant_id,
            action,
        } = request;

        let session = store
            .ph1onb_session_row(&onboarding_session_id)
            .cloned()
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;
        let link_tenant = store
            .ph1link_get_link(&session.token_id)
            .and_then(|link| link.prefilled_context.as_ref())
            .and_then(|ctx| ctx.tenant_id.clone());
        let request_tenant = tenant_id
            .as_ref()
            .map(|tenant| TenantId::new(tenant.clone()))
            .transpose()
            .map_err(StorageError::ContractViolation)?;
        let session_tenant = session
            .tenant_id
            .as_ref()
            .map(|tenant| TenantId::new(tenant.clone()))
            .transpose()
            .map_err(StorageError::ContractViolation)?;
        let link_tenant = link_tenant
            .as_ref()
            .map(|tenant| TenantId::new(tenant.clone()))
            .transpose()
            .map_err(StorageError::ContractViolation)?;

        if let (Some(request_tenant), Some(session_tenant)) = (&request_tenant, &session_tenant) {
            if request_tenant != session_tenant {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.tenant_id",
                        reason: "must match onboarding session tenant scope",
                    },
                ));
            }
        }
        if let (Some(request_tenant), Some(link_tenant)) = (&request_tenant, &link_tenant) {
            if request_tenant != link_tenant {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.tenant_id",
                        reason: "must match link tenant scope",
                    },
                ));
            }
        }
        let effective_tenant = request_tenant.or(session_tenant).or(link_tenant).ok_or({
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.tenant_id",
                reason: "missing tenant scope for onboarding continuation",
            })
        })?;
        let turn_id = TurnId(1);

        match action {
            AppOnboardingContinueAction::AskMissingSubmit { field_value } => {
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    LINK_INVITE_DRAFT_UPDATE_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;

                let ask = store.ph1onb_ask_missing_field_turn(
                    now,
                    onboarding_session_id.clone(),
                    field_value,
                    idempotency_key,
                )?;
                match ask.kind {
                    selene_storage::ph1f::OnbAskMissingOutcomeKind::Prompt
                    | selene_storage::ph1f::OnbAskMissingOutcomeKind::Updated => {
                        let remaining_platform_receipt_kinds = store
                            .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
                        let next_step = if ask.remaining_missing_fields.is_empty() {
                            if remaining_platform_receipt_kinds.is_empty() {
                                AppOnboardingContinueNextStep::Terms
                            } else {
                                AppOnboardingContinueNextStep::PlatformSetup
                            }
                        } else {
                            AppOnboardingContinueNextStep::AskMissing
                        };
                        let blocking_field =
                            if next_step == AppOnboardingContinueNextStep::AskMissing {
                                ask.field_key
                                    .clone()
                                    .or(ask.remaining_missing_fields.first().cloned())
                            } else {
                                None
                            };
                        let blocking_question = blocking_field
                            .as_deref()
                            .map(onboarding_missing_field_question);
                        Ok(AppOnboardingContinueOutcome {
                            onboarding_session_id: onboarding_session_id.as_str().to_string(),
                            next_step,
                            blocking_field,
                            blocking_question,
                            remaining_missing_fields: ask.remaining_missing_fields,
                            remaining_platform_receipt_kinds,
                            voice_artifact_sync_receipt_ref: None,
                            access_engine_instance_id: None,
                            onboarding_status: None,
                        })
                    }
                    selene_storage::ph1f::OnbAskMissingOutcomeKind::Escalated => Err(
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_ASK_MISSING_REPEAT_ESCALATION",
                        }),
                    ),
                }
            }
            AppOnboardingContinueAction::PlatformSetupReceipt {
                receipt_kind,
                receipt_ref,
                signer,
                payload_hash,
            } => {
                if store
                    .ph1onb_session_row(&onboarding_session_id)
                    .map(|rec| !rec.missing_fields.is_empty())
                    .unwrap_or(false)
                {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_ASK_MISSING_REQUIRED_BEFORE_PLATFORM_SETUP",
                        },
                    ));
                }
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    LINK_INVITE_DRAFT_UPDATE_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let receipt_outcome = store.ph1onb_platform_setup_receipt_commit(
                    now,
                    onboarding_session_id.clone(),
                    receipt_kind,
                    receipt_ref,
                    signer,
                    payload_hash,
                    idempotency_key,
                )?;
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: if receipt_outcome.remaining_required_receipt_kinds.is_empty() {
                        AppOnboardingContinueNextStep::Terms
                    } else {
                        AppOnboardingContinueNextStep::PlatformSetup
                    },
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: receipt_outcome
                        .remaining_required_receipt_kinds,
                    voice_artifact_sync_receipt_ref: None,
                    access_engine_instance_id: None,
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::TermsAccept {
                terms_version_id,
                accepted,
            } => {
                if store
                    .ph1onb_session_row(&onboarding_session_id)
                    .map(|rec| !rec.missing_fields.is_empty())
                    .unwrap_or(false)
                {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_ASK_MISSING_REQUIRED_BEFORE_TERMS",
                        },
                    ));
                }
                let remaining_platform_receipt_kinds =
                    store.ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
                if !remaining_platform_receipt_kinds.is_empty() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_PLATFORM_SETUP_REQUIRED_BEFORE_TERMS",
                        },
                    ));
                }
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    ONB_TERMS_ACCEPT_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let req = Ph1OnbRequest {
                    schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: ONB_TERMS_ACCEPT_COMMIT.to_string(),
                    simulation_type: SimulationType::Commit,
                    request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                        onboarding_session_id: onboarding_session_id.clone(),
                        terms_version_id,
                        accepted,
                        idempotency_key,
                    }),
                };
                req.validate().map_err(StorageError::ContractViolation)?;
                let out = self.executor.execute_onb(store, &req)?;
                let terms_status = match out {
                    Ph1OnbResponse::Ok(ok) => {
                        ok.terms_accept_result
                            .ok_or({
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1onb_response.terms_accept_result",
                                    reason: "terms accept result must be present",
                                })
                            })?
                            .terms_status
                    }
                    Ph1OnbResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1onb_response",
                                reason: "ONB_TERMS_ACCEPT_REFUSED",
                            },
                        ));
                    }
                };
                let sender_verification_pending =
                    onboarding_sender_verification_pending(store, &onboarding_session_id)?;
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: if terms_status == TermsStatus::Accepted {
                        if sender_verification_pending {
                            AppOnboardingContinueNextStep::SenderVerification
                        } else {
                            AppOnboardingContinueNextStep::PrimaryDeviceConfirm
                        }
                    } else {
                        AppOnboardingContinueNextStep::Blocked
                    },
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: None,
                    access_engine_instance_id: None,
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::PrimaryDeviceConfirm {
                device_id,
                proof_ok,
            } => {
                self.runtime_governance
                    .govern_protected_action_proof(
                        GovernanceProtectedActionClass::PrimaryDeviceConfirmation,
                        None,
                        Some(turn_id.0),
                        proof_ok,
                    )
                    .map_err(|decision| {
                        runtime_governance_storage_error(
                            "app_onboarding_continue_request.action.primary_device_confirm.proof_ok",
                            &decision,
                        )
                    })?;
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    ONB_PRIMARY_DEVICE_CONFIRM_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let req = Ph1OnbRequest {
                    schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: ONB_PRIMARY_DEVICE_CONFIRM_COMMIT.to_string(),
                    simulation_type: SimulationType::Commit,
                    request: OnbRequest::PrimaryDeviceConfirmCommit(
                        OnbPrimaryDeviceConfirmCommitRequest {
                            onboarding_session_id: onboarding_session_id.clone(),
                            device_id,
                            proof_type: ProofType::Biometric,
                            proof_ok,
                            idempotency_key,
                        },
                    ),
                };
                req.validate().map_err(StorageError::ContractViolation)?;
                let out = self.executor.execute_onb(store, &req)?;
                let primary_device_confirmed = match out {
                    Ph1OnbResponse::Ok(ok) => {
                        ok.primary_device_confirm_result
                            .ok_or({
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1onb_response.primary_device_confirm_result",
                                    reason: "primary device confirm result must be present",
                                })
                            })?
                            .primary_device_confirmed
                    }
                    Ph1OnbResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1onb_response",
                                reason: "ONB_PRIMARY_DEVICE_CONFIRM_REFUSED",
                            },
                        ));
                    }
                };
                let sender_verification_pending =
                    onboarding_sender_verification_pending(store, &onboarding_session_id)?;
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: if primary_device_confirmed {
                        if sender_verification_pending {
                            AppOnboardingContinueNextStep::SenderVerification
                        } else {
                            AppOnboardingContinueNextStep::VoiceEnroll
                        }
                    } else {
                        AppOnboardingContinueNextStep::PrimaryDeviceConfirm
                    },
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: None,
                    access_engine_instance_id: None,
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::VoiceEnrollLock {
                device_id,
                sample_seed,
            } => {
                let session = store
                    .ph1onb_session_row(&onboarding_session_id)
                    .cloned()
                    .ok_or(StorageError::ForeignKeyViolation {
                        table: "onboarding_sessions.onboarding_session_id",
                        key: onboarding_session_id.as_str().to_string(),
                    })?;
                if !session.missing_fields.is_empty() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_ASK_MISSING_REQUIRED_BEFORE_VOICE_ENROLL",
                        },
                    ));
                }
                let remaining_platform_receipt_kinds =
                    store.ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
                if !remaining_platform_receipt_kinds.is_empty() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_PLATFORM_SETUP_REQUIRED_BEFORE_VOICE_ENROLL",
                        },
                    ));
                }
                if session.terms_status != Some(TermsStatus::Accepted) {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_TERMS_REQUIRED_BEFORE_VOICE_ENROLL",
                        },
                    ));
                }
                if !session.primary_device_confirmed {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL",
                        },
                    ));
                }
                let expected_device_id =
                    session
                        .primary_device_device_id
                        .ok_or(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "app_onboarding_continue_request.action",
                                reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL",
                            },
                        ))?;
                if expected_device_id != device_id {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field:
                                "app_onboarding_continue_request.action.voice_enroll_lock.device_id",
                            reason: "ONB_PRIMARY_DEVICE_DEVICE_MISMATCH_FOR_VOICE_ENROLL",
                        },
                    ));
                }
                if onboarding_sender_verification_pending(store, &onboarding_session_id)? {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_VOICE_ENROLL",
                        },
                    ));
                }
                self.executor.ensure_simulation_chain_active_for_tenant(
                    store,
                    &effective_tenant,
                    &[
                        VOICE_ID_ENROLL_START_DRAFT,
                        VOICE_ID_ENROLL_SAMPLE_COMMIT,
                        VOICE_ID_ENROLL_COMPLETE_COMMIT,
                    ],
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;

                let samples = vec![
                    OnbVoiceEnrollSampleStep {
                        audio_sample_ref: format!("audio:{sample_seed}:1"),
                        attempt_index: 1,
                        sample_duration_ms: 1_380,
                        vad_coverage: 0.93,
                        snr_db: 18.1,
                        clipping_pct: 0.3,
                        overlap_ratio: 0.0,
                        app_embedding_capture_ref: None,
                        idempotency_key: format!("{idempotency_key}-voice-sample-1"),
                    },
                    OnbVoiceEnrollSampleStep {
                        audio_sample_ref: format!("audio:{sample_seed}:2"),
                        attempt_index: 2,
                        sample_duration_ms: 1_420,
                        vad_coverage: 0.92,
                        snr_db: 18.0,
                        clipping_pct: 0.4,
                        overlap_ratio: 0.0,
                        app_embedding_capture_ref: None,
                        idempotency_key: format!("{idempotency_key}-voice-sample-2"),
                    },
                    OnbVoiceEnrollSampleStep {
                        audio_sample_ref: format!("audio:{sample_seed}:3"),
                        attempt_index: 3,
                        sample_duration_ms: 1_450,
                        vad_coverage: 0.94,
                        snr_db: 18.3,
                        clipping_pct: 0.2,
                        overlap_ratio: 0.0,
                        app_embedding_capture_ref: None,
                        idempotency_key: format!("{idempotency_key}-voice-sample-3"),
                    },
                ];
                let voice_out = self.executor.execute_onb_voice_enrollment_live_sequence(
                    store,
                    &OnbVoiceEnrollLiveRequest {
                        correlation_id,
                        turn_id_start: turn_id,
                        now,
                        onboarding_session_id: onboarding_session_id.clone(),
                        device_id,
                        consent_asserted: true,
                        max_total_attempts: 8,
                        max_session_enroll_time_ms: 120_000,
                        lock_after_consecutive_passes: 3,
                        samples,
                        finalize: OnbVoiceEnrollFinalize::Complete {
                            idempotency_key: format!("{idempotency_key}-voice-complete"),
                        },
                    },
                )?;
                let voice_artifact_sync_receipt_ref = voice_out
                    .complete_result
                    .and_then(|r| r.voice_artifact_sync_receipt_ref);
                if voice_artifact_sync_receipt_ref.is_none() {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action.voice_enroll_lock",
                            reason: "voice enrollment complete must return sync receipt",
                        },
                    ));
                }
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: onboarding_next_step_after_voice_enroll(
                        store,
                        &onboarding_session_id,
                        session.app_platform,
                    ),
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds,
                    voice_artifact_sync_receipt_ref,
                    access_engine_instance_id: None,
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::WakeEnrollStartDraft { device_id } => {
                ensure_wake_enrollment_action_allowed(store, &onboarding_session_id, &device_id)?;
                self.executor.ensure_simulation_chain_active_for_tenant(
                    store,
                    &effective_tenant,
                    &[
                        WAKE_ENROLL_START_DRAFT,
                        WAKE_ENROLL_SAMPLE_COMMIT,
                        WAKE_ENROLL_COMPLETE_COMMIT,
                        WAKE_ENROLL_DEFER_COMMIT,
                    ],
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;

                let user_id = onboarding_user_for_device(store, &device_id)?;
                let wake_req = Ph1wRequest {
                    schema_version: PH1W_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
                    simulation_type: WakeSimulationType::Draft,
                    request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                        user_id,
                        device_id,
                        onboarding_session_id: Some(onboarding_session_id.as_str().to_string()),
                        allow_ios_wake_override: false,
                        pass_target: 3,
                        max_attempts: 8,
                        enrollment_timeout_ms: 180_000,
                        idempotency_key,
                    }),
                };
                wake_req
                    .validate()
                    .map_err(StorageError::ContractViolation)?;
                let wake_out = Ph1wRuntime.run(store, &wake_req)?;
                let wake_status = match wake_out {
                    Ph1wResponse::Ok(ok) => {
                        ok.enroll_start_result
                            .ok_or({
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1w_response.enroll_start_result",
                                    reason: "wake enroll start result must be present",
                                })
                            })?
                            .wake_enroll_status
                    }
                    Ph1wResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1w_response",
                                reason: "ONB_WAKE_ENROLL_START_REFUSED",
                            },
                        ));
                    }
                };
                let next_step = onboarding_next_step_after_wake_status(wake_status);

                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step,
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: store
                        .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
                    access_engine_instance_id: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .and_then(|row| row.access_engine_instance_id.clone()),
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::WakeEnrollSampleCommit {
                device_id,
                sample_pass,
            } => {
                ensure_wake_enrollment_action_allowed(store, &onboarding_session_id, &device_id)?;
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    WAKE_ENROLL_SAMPLE_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let wake_session = store
                    .ph1w_latest_session_for_onboarding_device(&onboarding_session_id, &device_id)
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_WAKE_ENROLL_START_REQUIRED_BEFORE_SAMPLE",
                        })
                    })?
                    .wake_enrollment_session_id
                    .clone();
                let wake_session_id = WakeEnrollmentSessionId::new(wake_session)
                    .map_err(StorageError::ContractViolation)?;
                let wake_req = Ph1wRequest {
                    schema_version: PH1W_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: WAKE_ENROLL_SAMPLE_COMMIT.to_string(),
                    simulation_type: WakeSimulationType::Commit,
                    request: WakeRequest::EnrollSampleCommit(WakeEnrollSampleCommitRequest {
                        wake_enrollment_session_id: wake_session_id,
                        sample_duration_ms: 1_380,
                        vad_coverage: 0.93,
                        snr_db: 18.1,
                        clipping_pct: 0.3,
                        rms_dbfs: -18.0,
                        noise_floor_dbfs: -58.0,
                        peak_dbfs: -3.0,
                        overlap_ratio: 0.0,
                        result: if sample_pass {
                            WakeSampleResult::Pass
                        } else {
                            WakeSampleResult::Fail
                        },
                        reason_code: if sample_pass {
                            None
                        } else {
                            Some(ReasonCodeId(0x5700_3001))
                        },
                        idempotency_key,
                    }),
                };
                wake_req
                    .validate()
                    .map_err(StorageError::ContractViolation)?;
                let wake_out = Ph1wRuntime.run(store, &wake_req)?;
                let wake_status = match wake_out {
                    Ph1wResponse::Ok(ok) => {
                        ok.enroll_sample_result
                            .ok_or({
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1w_response.enroll_sample_result",
                                    reason: "wake enroll sample result must be present",
                                })
                            })?
                            .wake_enroll_status
                    }
                    Ph1wResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1w_response",
                                reason: "ONB_WAKE_ENROLL_SAMPLE_REFUSED",
                            },
                        ));
                    }
                };

                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: onboarding_next_step_after_wake_status(wake_status),
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: store
                        .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
                    access_engine_instance_id: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .and_then(|row| row.access_engine_instance_id.clone()),
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id } => {
                ensure_wake_enrollment_action_allowed(store, &onboarding_session_id, &device_id)?;
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    WAKE_ENROLL_COMPLETE_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let wake_session = store
                    .ph1w_latest_session_for_onboarding_device(&onboarding_session_id, &device_id)
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_WAKE_ENROLL_START_REQUIRED_BEFORE_COMPLETE",
                        })
                    })?
                    .wake_enrollment_session_id
                    .clone();
                let wake_session_id = WakeEnrollmentSessionId::new(wake_session.clone())
                    .map_err(StorageError::ContractViolation)?;
                let wake_profile_id = format!(
                    "wake_profile_{}",
                    short_hash_hex(&[
                        onboarding_session_id.as_str(),
                        wake_session.as_str(),
                        device_id.as_str(),
                    ])
                );
                let wake_req = Ph1wRequest {
                    schema_version: PH1W_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: WAKE_ENROLL_COMPLETE_COMMIT.to_string(),
                    simulation_type: WakeSimulationType::Commit,
                    request: WakeRequest::EnrollCompleteCommit(WakeEnrollCompleteCommitRequest {
                        wake_enrollment_session_id: wake_session_id,
                        wake_profile_id,
                        idempotency_key,
                    }),
                };
                wake_req
                    .validate()
                    .map_err(StorageError::ContractViolation)?;
                let wake_out = Ph1wRuntime.run(store, &wake_req)?;
                let wake_status = match wake_out {
                    Ph1wResponse::Ok(ok) => {
                        ok.enroll_complete_result
                            .ok_or({
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1w_response.enroll_complete_result",
                                    reason: "wake enroll complete result must be present",
                                })
                            })?
                            .wake_enroll_status
                    }
                    Ph1wResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1w_response",
                                reason: "ONB_WAKE_ENROLL_COMPLETE_REFUSED",
                            },
                        ));
                    }
                };
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: onboarding_next_step_after_wake_status(wake_status),
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: store
                        .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
                    access_engine_instance_id: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .and_then(|row| row.access_engine_instance_id.clone()),
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::WakeEnrollDeferCommit { device_id } => {
                ensure_wake_enrollment_action_allowed(store, &onboarding_session_id, &device_id)?;
                self.executor.ensure_simulation_active_for_tenant(
                    store,
                    &effective_tenant,
                    WAKE_ENROLL_DEFER_COMMIT,
                    "app_onboarding_continue_request.simulation_id",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
                    "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
                )?;
                let wake_session = store
                    .ph1w_latest_session_for_onboarding_device(&onboarding_session_id, &device_id)
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action",
                            reason: "ONB_WAKE_ENROLL_START_REQUIRED_BEFORE_DEFER",
                        })
                    })?
                    .wake_enrollment_session_id
                    .clone();
                let wake_session_id = WakeEnrollmentSessionId::new(wake_session)
                    .map_err(StorageError::ContractViolation)?;
                let wake_req = Ph1wRequest {
                    schema_version: PH1W_CONTRACT_VERSION,
                    correlation_id,
                    turn_id,
                    now,
                    simulation_id: WAKE_ENROLL_DEFER_COMMIT.to_string(),
                    simulation_type: WakeSimulationType::Commit,
                    request: WakeRequest::EnrollDeferCommit(WakeEnrollDeferCommitRequest {
                        wake_enrollment_session_id: wake_session_id,
                        deferred_until: None,
                        reason_code: ReasonCodeId(0x5700_3002),
                        idempotency_key,
                    }),
                };
                wake_req
                    .validate()
                    .map_err(StorageError::ContractViolation)?;
                let wake_out = Ph1wRuntime.run(store, &wake_req)?;
                match wake_out {
                    Ph1wResponse::Ok(ok) => {
                        if ok.enroll_defer_result.is_none() {
                            return Err(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "ph1w_response.enroll_defer_result",
                                    reason: "wake enroll defer result must be present",
                                },
                            ));
                        }
                    }
                    Ph1wResponse::Refuse(_) => {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1w_response",
                                reason: "ONB_WAKE_ENROLL_DEFER_REFUSED",
                            },
                        ));
                    }
                };
                Ok(AppOnboardingContinueOutcome {
                    onboarding_session_id: onboarding_session_id.as_str().to_string(),
                    next_step: AppOnboardingContinueNextStep::WakeEnroll,
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .map(|r| r.missing_fields.clone())
                        .unwrap_or_default(),
                    remaining_platform_receipt_kinds: store
                        .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
                    voice_artifact_sync_receipt_ref: store
                        .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
                    access_engine_instance_id: store
                        .ph1onb_session_row(&onboarding_session_id)
                        .and_then(|row| row.access_engine_instance_id.clone()),
                    onboarding_status: None,
                })
            }
            AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } => self
                .run_onboarding_employee_photo_capture_send(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    photo_blob_ref,
                    idempotency_key,
                    now,
                ),
            AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision } => self
                .run_onboarding_employee_sender_verify_commit(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    decision,
                    idempotency_key,
                    now,
                ),
            AppOnboardingContinueAction::EmoPersonaLock => self.run_onboarding_emo_persona_lock(
                store,
                correlation_id,
                turn_id,
                onboarding_session_id,
                effective_tenant,
                idempotency_key,
                now,
            ),
            AppOnboardingContinueAction::AccessProvisionCommit => self
                .run_onboarding_access_provision(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    idempotency_key,
                    now,
                ),
            AppOnboardingContinueAction::CompleteCommit => self.run_onboarding_complete(
                store,
                correlation_id,
                turn_id,
                onboarding_session_id,
                effective_tenant,
                idempotency_key,
                now,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_onboarding_employee_photo_capture_send(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        onboarding_session_id: OnboardingSessionId,
        effective_tenant: TenantId,
        photo_blob_ref: String,
        idempotency_key: String,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        self.executor.ensure_simulation_active_for_tenant(
            store,
            &effective_tenant,
            ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT,
            "app_onboarding_continue_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;
        let sender_user_id = onboarding_sender_user_id_for_session(store, &onboarding_session_id)?;
        let req = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::EmployeePhotoCaptureSendCommit(
                OnbEmployeePhotoCaptureSendCommitRequest {
                    onboarding_session_id: onboarding_session_id.clone(),
                    photo_blob_ref,
                    sender_user_id,
                    idempotency_key,
                },
            ),
        };
        req.validate().map_err(StorageError::ContractViolation)?;
        let out = self.executor.execute_onb(store, &req)?;
        let verification_status = match out {
            Ph1OnbResponse::Ok(ok) => {
                ok.employee_photo_result
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "ph1onb_response.employee_photo_result",
                            reason: "employee photo result must be present",
                        })
                    })?
                    .verification_status
            }
            Ph1OnbResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_response",
                        reason: "ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_REFUSED",
                    },
                ));
            }
        };
        let onboarding_status = match verification_status {
            VerificationStatus::Pending => OnboardingStatus::VerificationPending,
            VerificationStatus::Confirmed => OnboardingStatus::VerificationConfirmed,
            VerificationStatus::Rejected => OnboardingStatus::VerificationRejected,
        };
        Ok(AppOnboardingContinueOutcome {
            onboarding_session_id: onboarding_session_id.as_str().to_string(),
            next_step: AppOnboardingContinueNextStep::SenderVerification,
            blocking_field: None,
            blocking_question: None,
            remaining_missing_fields: store
                .ph1onb_session_row(&onboarding_session_id)
                .map(|r| r.missing_fields.clone())
                .unwrap_or_default(),
            remaining_platform_receipt_kinds: store
                .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
            voice_artifact_sync_receipt_ref: store
                .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
            access_engine_instance_id: store
                .ph1onb_session_row(&onboarding_session_id)
                .and_then(|row| row.access_engine_instance_id.clone()),
            onboarding_status: Some(onboarding_status),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_onboarding_employee_sender_verify_commit(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        onboarding_session_id: OnboardingSessionId,
        effective_tenant: TenantId,
        decision: SenderVerifyDecision,
        idempotency_key: String,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        self.executor.ensure_simulation_active_for_tenant(
            store,
            &effective_tenant,
            ONB_EMPLOYEE_SENDER_VERIFY_COMMIT,
            "app_onboarding_continue_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;
        let sender_user_id = onboarding_sender_user_id_for_session(store, &onboarding_session_id)?;
        let req = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: ONB_EMPLOYEE_SENDER_VERIFY_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::EmployeeSenderVerifyCommit(OnbEmployeeSenderVerifyCommitRequest {
                onboarding_session_id: onboarding_session_id.clone(),
                sender_user_id,
                decision,
                idempotency_key,
            }),
        };
        req.validate().map_err(StorageError::ContractViolation)?;
        let out = self.executor.execute_onb(store, &req)?;
        let verification_status = match out {
            Ph1OnbResponse::Ok(ok) => {
                ok.employee_sender_verify_result
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "ph1onb_response.employee_sender_verify_result",
                            reason: "employee sender verify result must be present",
                        })
                    })?
                    .verification_status
            }
            Ph1OnbResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_response",
                        reason: "ONB_EMPLOYEE_SENDER_VERIFY_REFUSED",
                    },
                ));
            }
        };
        let next_step = match verification_status {
            VerificationStatus::Pending => AppOnboardingContinueNextStep::SenderVerification,
            VerificationStatus::Rejected => AppOnboardingContinueNextStep::Blocked,
            VerificationStatus::Confirmed => {
                let session = store.ph1onb_session_row(&onboarding_session_id).ok_or(
                    StorageError::ForeignKeyViolation {
                        table: "onboarding_sessions.onboarding_session_id",
                        key: onboarding_session_id.as_str().to_string(),
                    },
                )?;
                if !session.primary_device_confirmed {
                    AppOnboardingContinueNextStep::PrimaryDeviceConfirm
                } else if store
                    .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id)
                    .is_none()
                {
                    AppOnboardingContinueNextStep::VoiceEnroll
                } else {
                    onboarding_next_step_after_voice_enroll(
                        store,
                        &onboarding_session_id,
                        session.app_platform,
                    )
                }
            }
        };
        let onboarding_status = match verification_status {
            VerificationStatus::Pending => OnboardingStatus::VerificationPending,
            VerificationStatus::Confirmed => OnboardingStatus::VerificationConfirmed,
            VerificationStatus::Rejected => OnboardingStatus::VerificationRejected,
        };
        Ok(AppOnboardingContinueOutcome {
            onboarding_session_id: onboarding_session_id.as_str().to_string(),
            next_step,
            blocking_field: None,
            blocking_question: None,
            remaining_missing_fields: store
                .ph1onb_session_row(&onboarding_session_id)
                .map(|r| r.missing_fields.clone())
                .unwrap_or_default(),
            remaining_platform_receipt_kinds: store
                .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
            voice_artifact_sync_receipt_ref: store
                .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id),
            access_engine_instance_id: store
                .ph1onb_session_row(&onboarding_session_id)
                .and_then(|row| row.access_engine_instance_id.clone()),
            onboarding_status: Some(onboarding_status),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_onboarding_emo_persona_lock(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        onboarding_session_id: OnboardingSessionId,
        effective_tenant: TenantId,
        idempotency_key: String,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        let session = store
            .ph1onb_session_row(&onboarding_session_id)
            .cloned()
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;
        if !session.missing_fields.is_empty() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_ASK_MISSING_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        let remaining_platform_receipt_kinds =
            store.ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
        if !remaining_platform_receipt_kinds.is_empty() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PLATFORM_SETUP_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        if session.terms_status != Some(TermsStatus::Accepted) {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_TERMS_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        if !session.primary_device_confirmed {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        if onboarding_sender_verification_pending(store, &onboarding_session_id)? {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        self.executor.ensure_simulation_active_for_tenant(
            store,
            &effective_tenant,
            EMO_SIM_001,
            "app_onboarding_continue_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;

        let device_id = session.primary_device_device_id.clone().ok_or({
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
            })
        })?;
        let voice_profile_locked = store
            .ph1vid_voice_profile_rows()
            .iter()
            .any(|profile| profile.device_id == device_id);
        if !voice_profile_locked {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_VOICE_ENROLL_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
                },
            ));
        }
        ensure_wake_enrollment_completed_for_platform(
            store,
            &onboarding_session_id,
            "ONB_WAKE_ENROLL_REQUIRED_BEFORE_EMO_PERSONA_LOCK",
        )?;
        let (persona_user_id, persona_speaker_id) = ensure_onboarding_persona_subject(
            store,
            &effective_tenant,
            &onboarding_session_id,
            &device_id,
            now,
        )?;

        let emo_signals = emo_signal_bundle_for_onboarding_session(&onboarding_session_id)
            .map_err(StorageError::ContractViolation)?;
        let emo_req = Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: EMO_SIM_001.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::ClassifyProfileCommit(EmoClassifyProfileCommitRequest {
                tenant_id: effective_tenant.clone(),
                requester_user_id: persona_user_id.clone(),
                session_id: onboarding_session_id.as_str().to_string(),
                consent_asserted: true,
                identity_verified: true,
                signals: emo_signals,
                idempotency_key: format!("{idempotency_key}-emo-core"),
            }),
        };
        emo_req
            .validate()
            .map_err(StorageError::ContractViolation)?;
        let emo_runtime = EnginePh1EmoCoreRuntime::new(EnginePh1EmoCoreConfig::mvp_v1());
        let emo_ok = match emo_runtime.run(&emo_req) {
            Ph1EmoCoreResponse::Ok(ok) => ok,
            Ph1EmoCoreResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_CORE_CLASSIFY_REFUSED",
                    },
                ));
            }
        };
        if !emo_ok.tone_only || !emo_ok.no_meaning_drift || !emo_ok.no_execution_authority {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_EMO_CORE_TONE_ONLY_GUARDS_REQUIRED",
                },
            ));
        }
        let personality_type = match &emo_ok.outcome {
            EmoCoreOutcome::ClassifyProfile(outcome) => outcome.personality_type,
            _ => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_CORE_CLASSIFY_OUTCOME_REQUIRED",
                    },
                ));
            }
        };

        let emo_guide_runtime = EnginePh1EmoGuideRuntime::new(EnginePh1EmoGuideConfig::mvp_v1());
        let emo_guide_envelope = EmoGuideRequestEnvelope::v1(correlation_id, turn_id, 120, 3, 8)
            .map_err(StorageError::ContractViolation)?;
        let emo_guide_signals = emoguide_signals_for_personality(personality_type)
            .map_err(StorageError::ContractViolation)?;
        let emo_guide_build_req = Ph1EmoGuideRequest::EmoGuideProfileBuild(
            EmoGuideProfileBuildRequest::v1(
                emo_guide_envelope.clone(),
                persona_speaker_id.clone(),
                emo_guide_signals.clone(),
                None,
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let emo_guide_build_ok = match emo_guide_runtime.run(&emo_guide_build_req) {
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(ok) => ok,
            Ph1EmoGuideResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_GUIDE_BUILD_REFUSED",
                    },
                ));
            }
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_GUIDE_BUILD_OUTCOME_REQUIRED",
                    },
                ));
            }
        };
        if !emo_guide_build_ok.tone_only
            || !emo_guide_build_ok.no_meaning_drift
            || !emo_guide_build_ok.no_execution_authority
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_EMO_GUIDE_TONE_ONLY_GUARDS_REQUIRED",
                },
            ));
        }

        let emo_guide_validate_req = Ph1EmoGuideRequest::EmoGuideProfileValidate(
            EmoGuideProfileValidateRequest::v1(
                emo_guide_envelope,
                persona_speaker_id.clone(),
                emo_guide_signals,
                None,
                emo_guide_build_ok.profile.clone(),
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let emo_guide_validate_ok = match emo_guide_runtime.run(&emo_guide_validate_req) {
            Ph1EmoGuideResponse::EmoGuideProfileValidateOk(ok) => ok,
            Ph1EmoGuideResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_GUIDE_VALIDATE_REFUSED",
                    },
                ));
            }
            Ph1EmoGuideResponse::EmoGuideProfileBuildOk(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_EMO_GUIDE_VALIDATE_OUTCOME_REQUIRED",
                    },
                ));
            }
        };
        if emo_guide_validate_ok.validation_status != EmoGuideValidationStatus::Ok
            || !emo_guide_validate_ok.tone_only
            || !emo_guide_validate_ok.no_meaning_drift
            || !emo_guide_validate_ok.no_execution_authority
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_EMO_GUIDE_VALIDATE_FAILED",
                },
            ));
        }

        let persona_runtime = EnginePh1PersonaRuntime::new(EnginePh1PersonaConfig::mvp_v1());
        let persona_envelope = PersonaRequestEnvelope::v1(correlation_id, turn_id, 16, 8)
            .map_err(StorageError::ContractViolation)?;
        let persona_signals = vec![PersonaPreferenceSignal::v1(
            PersonaPreferenceKey::ResponseToneTarget,
            persona_tone_value_for_personality(personality_type).to_string(),
            format!("emo:{}", onboarding_session_id.as_str()),
        )
        .map_err(StorageError::ContractViolation)?];
        let persona_speaker_id_for_validate = persona_speaker_id.clone();
        let persona_build_req = Ph1PersonaRequest::PersonaProfileBuild(
            selene_kernel_contracts::ph1persona::PersonaProfileBuildRequest::v1(
                persona_envelope.clone(),
                persona_user_id.as_str().to_string(),
                persona_speaker_id,
                persona_signals.clone(),
                emo_guide_build_ok
                    .profile
                    .stability_window_turns
                    .saturating_sub(3),
                Some(emo_guide_build_ok.profile.style_profile_ref),
                None,
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let persona_build_ok = match persona_runtime.run(&persona_build_req) {
            Ph1PersonaResponse::PersonaProfileBuildOk(ok) => ok,
            Ph1PersonaResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_PERSONA_BUILD_REFUSED",
                    },
                ));
            }
            Ph1PersonaResponse::PersonaProfileValidateOk(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_PERSONA_BUILD_OUTCOME_REQUIRED",
                    },
                ));
            }
        };
        if !persona_build_ok.tone_only
            || !persona_build_ok.no_meaning_drift
            || !persona_build_ok.no_execution_authority
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PERSONA_BUILD_GUARDS_REQUIRED",
                },
            ));
        }

        let persona_validate_req = Ph1PersonaRequest::PersonaProfileValidate(
            PersonaProfileValidateRequest::v1(
                persona_envelope,
                persona_user_id.as_str().to_string(),
                persona_speaker_id_for_validate,
                persona_signals,
                emo_guide_build_ok
                    .profile
                    .stability_window_turns
                    .saturating_sub(3),
                Some(emo_guide_build_ok.profile.style_profile_ref),
                None,
                persona_build_ok.profile_snapshot.clone(),
            )
            .map_err(StorageError::ContractViolation)?,
        );
        let persona_validate_ok = match persona_runtime.run(&persona_validate_req) {
            Ph1PersonaResponse::PersonaProfileValidateOk(ok) => ok,
            Ph1PersonaResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_PERSONA_VALIDATE_REFUSED",
                    },
                ));
            }
            Ph1PersonaResponse::PersonaProfileBuildOk(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_onboarding_continue_request.action",
                        reason: "ONB_PERSONA_VALIDATE_OUTCOME_REQUIRED",
                    },
                ));
            }
        };
        if persona_validate_ok.validation_status != PersonaValidationStatus::Ok
            || !persona_validate_ok.tone_only
            || !persona_validate_ok.no_meaning_drift
            || !persona_validate_ok.no_execution_authority
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PERSONA_VALIDATE_FAILED",
                },
            ));
        }

        let persona_lock_audit_event_id = store.ph1persona_profile_commit(
            now,
            effective_tenant.as_str().to_string(),
            correlation_id,
            turn_id,
            None,
            persona_user_id,
            device_id,
            style_profile_ref_token(persona_build_ok.profile_snapshot.style_profile_ref)
                .to_string(),
            persona_delivery_policy_token(persona_build_ok.profile_snapshot.delivery_policy_ref)
                .to_string(),
            persona_build_ok.profile_snapshot.preferences_snapshot_ref,
            persona_validate_ok.reason_code,
            format!("{idempotency_key}-persona-commit"),
        )?;
        store.ph1onb_emo_persona_lock_commit(
            now,
            onboarding_session_id.clone(),
            persona_lock_audit_event_id,
        )?;

        Ok(AppOnboardingContinueOutcome {
            onboarding_session_id: onboarding_session_id.as_str().to_string(),
            next_step: AppOnboardingContinueNextStep::AccessProvision,
            blocking_field: None,
            blocking_question: None,
            remaining_missing_fields: store
                .ph1onb_session_row(&onboarding_session_id)
                .map(|r| r.missing_fields.clone())
                .unwrap_or_default(),
            remaining_platform_receipt_kinds: store
                .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
            voice_artifact_sync_receipt_ref: session.voice_artifact_sync_receipt_ref,
            access_engine_instance_id: None,
            onboarding_status: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_onboarding_access_provision(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        onboarding_session_id: OnboardingSessionId,
        effective_tenant: TenantId,
        idempotency_key: String,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        self.executor.ensure_simulation_active_for_tenant(
            store,
            &effective_tenant,
            ONB_ACCESS_INSTANCE_CREATE_COMMIT,
            "app_onboarding_continue_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;

        let session = store
            .ph1onb_session_row(&onboarding_session_id)
            .cloned()
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;
        if onboarding_sender_verification_pending(store, &onboarding_session_id)? {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_ACCESS_PROVISION",
                },
            ));
        }
        if store
            .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id)
            .is_none()
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_VOICE_ENROLL_REQUIRED_BEFORE_ACCESS_PROVISION",
                },
            ));
        }
        ensure_wake_enrollment_completed_for_platform(
            store,
            &onboarding_session_id,
            "ONB_WAKE_ENROLL_REQUIRED_BEFORE_ACCESS_PROVISION",
        )?;
        let device_id = session.primary_device_device_id.clone().ok_or({
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_ACCESS_PROVISION",
            })
        })?;
        if session.emo_persona_lock_audit_event_id.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_ACCESS_PROVISION",
                },
            ));
        }
        let (user_id, _) = ensure_onboarding_persona_subject(
            store,
            &effective_tenant,
            &onboarding_session_id,
            &device_id,
            now,
        )?;
        let role_id = onboarding_default_role_id(session.invitee_type).to_string();
        let req = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: ONB_ACCESS_INSTANCE_CREATE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::AccessInstanceCreateCommit(OnbAccessInstanceCreateCommitRequest {
                onboarding_session_id: onboarding_session_id.clone(),
                user_id,
                tenant_id: Some(effective_tenant.as_str().to_string()),
                role_id,
                idempotency_key,
            }),
        };
        req.validate().map_err(StorageError::ContractViolation)?;
        let out = self.executor.execute_onb(store, &req)?;
        let access_engine_instance_id = match out {
            Ph1OnbResponse::Ok(ok) => {
                ok.access_instance_create_result
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "ph1onb_response.access_instance_create_result",
                            reason: "access instance create result must be present",
                        })
                    })?
                    .access_engine_instance_id
            }
            Ph1OnbResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_response",
                        reason: "ONB_ACCESS_INSTANCE_CREATE_REFUSED",
                    },
                ));
            }
        };

        let voice_artifact_sync_receipt_ref =
            store.ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id);
        Ok(AppOnboardingContinueOutcome {
            onboarding_session_id: onboarding_session_id.as_str().to_string(),
            next_step: AppOnboardingContinueNextStep::Complete,
            blocking_field: None,
            blocking_question: None,
            remaining_missing_fields: store
                .ph1onb_session_row(&onboarding_session_id)
                .map(|r| r.missing_fields.clone())
                .unwrap_or_default(),
            remaining_platform_receipt_kinds: store
                .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
            voice_artifact_sync_receipt_ref,
            access_engine_instance_id: Some(access_engine_instance_id),
            onboarding_status: Some(OnboardingStatus::AccessInstanceCreated),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_onboarding_complete(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        onboarding_session_id: OnboardingSessionId,
        effective_tenant: TenantId,
        idempotency_key: String,
        now: MonotonicTimeNs,
    ) -> Result<AppOnboardingContinueOutcome, StorageError> {
        self.executor.ensure_simulation_active_for_tenant(
            store,
            &effective_tenant,
            ONB_COMPLETE_COMMIT,
            "app_onboarding_continue_request.simulation_id",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED",
            "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE",
        )?;
        if onboarding_sender_verification_pending(store, &onboarding_session_id)? {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_COMPLETE",
                },
            ));
        }
        let voice_artifact_sync_receipt_ref = store
            .ph1onb_latest_locked_voice_receipt_ref(&onboarding_session_id)
            .ok_or({
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_VOICE_ENROLL_REQUIRED_BEFORE_COMPLETE",
                })
            })?;
        ensure_wake_enrollment_completed_for_platform(
            store,
            &onboarding_session_id,
            "ONB_WAKE_ENROLL_REQUIRED_BEFORE_COMPLETE",
        )?;
        let session = store
            .ph1onb_session_row(&onboarding_session_id)
            .cloned()
            .ok_or(StorageError::ForeignKeyViolation {
                table: "onboarding_sessions.onboarding_session_id",
                key: onboarding_session_id.as_str().to_string(),
            })?;
        if session.primary_device_device_id.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_COMPLETE",
                },
            ));
        }
        if session.emo_persona_lock_audit_event_id.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_COMPLETE",
                },
            ));
        }
        if session.access_engine_instance_id.is_none() {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_ACCESS_PROVISION_REQUIRED_BEFORE_COMPLETE",
                },
            ));
        }
        let wake_artifact_sync_receipt_ref =
            store.ph1onb_latest_complete_wake_receipt_ref(&onboarding_session_id);
        let req = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: ONB_COMPLETE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::CompleteCommit(OnbCompleteCommitRequest {
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key,
                voice_artifact_sync_receipt_ref: Some(voice_artifact_sync_receipt_ref.clone()),
                wake_artifact_sync_receipt_ref: wake_artifact_sync_receipt_ref.clone(),
            }),
        };
        req.validate().map_err(StorageError::ContractViolation)?;
        let out = self.executor.execute_onb(store, &req)?;
        let onboarding_status = match out {
            Ph1OnbResponse::Ok(ok) => {
                ok.complete_result
                    .ok_or({
                        StorageError::ContractViolation(ContractViolation::InvalidValue {
                            field: "ph1onb_response.complete_result",
                            reason: "complete result must be present",
                        })
                    })?
                    .onboarding_status
            }
            Ph1OnbResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_response",
                        reason: "ONB_COMPLETE_REFUSED",
                    },
                ));
            }
        };
        let access_engine_instance_id = store
            .ph1onb_session_row(&onboarding_session_id)
            .and_then(|row| row.access_engine_instance_id.clone());
        Ok(AppOnboardingContinueOutcome {
            onboarding_session_id: onboarding_session_id.as_str().to_string(),
            next_step: AppOnboardingContinueNextStep::Ready,
            blocking_field: None,
            blocking_question: None,
            remaining_missing_fields: store
                .ph1onb_session_row(&onboarding_session_id)
                .map(|r| r.missing_fields.clone())
                .unwrap_or_default(),
            remaining_platform_receipt_kinds: store
                .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?,
            voice_artifact_sync_receipt_ref: Some(voice_artifact_sync_receipt_ref),
            access_engine_instance_id,
            onboarding_status: Some(onboarding_status),
        })
    }

    pub fn run_voice_turn_and_build_ph1x_request(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<(OsVoiceLiveTurnOutcome, Option<Ph1xRequest>), StorageError> {
        self.run_voice_turn_and_build_ph1x_request_internal(store, request, x_build)
            .map(|(voice_outcome, _, ph1x_request)| (voice_outcome, ph1x_request))
    }

    fn run_voice_turn_and_build_ph1x_request_internal(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<
        (
            OsVoiceLiveTurnOutcome,
            RuntimeExecutionEnvelope,
            Option<Ph1xRequest>,
        ),
        StorageError,
    > {
        let correlation_id = request.correlation_id;
        let turn_id = request.turn_id;
        let app_platform = request.app_platform;
        let request_session_id = request.voice_id_request.session_state_ref.session_id;
        let request_tenant_id = request.tenant_id.clone();
        let (outcome, governed_runtime_execution_envelope) =
            self.run_voice_turn_with_governed_envelope(store, request)?;

        let ph1x_request = match &outcome {
            OsVoiceLiveTurnOutcome::Forwarded(forwarded) => {
                Some(self.build_ph1x_request_for_forwarded_voice(
                    store,
                    ForwardedVoicePh1xRequestInput {
                        correlation_id,
                        turn_id,
                        app_platform,
                        forwarded,
                        request_session_id,
                        tenant_id: request_tenant_id.as_deref(),
                        x_build,
                    },
                )?)
            }
            OsVoiceLiveTurnOutcome::NotInvokedDisabled | OsVoiceLiveTurnOutcome::Refused(_) => None,
        };

        let runtime_execution_envelope = if ph1x_request.is_some() {
            self.last_agent_input_packet
                .borrow()
                .as_ref()
                .and_then(|packet| packet.runtime_execution_envelope.clone())
                .unwrap_or(governed_runtime_execution_envelope)
        } else {
            governed_runtime_execution_envelope
        };

        Ok((outcome, runtime_execution_envelope, ph1x_request))
    }

    pub fn run_voice_turn_end_to_end(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        let correlation_id = request.correlation_id;
        let turn_id = request.turn_id;
        let received_at = request.voice_id_request.now;
        let request_session_id = request.voice_id_request.session_state_ref.session_id;
        let request_session_state = request.voice_id_request.session_state_ref.session_state;
        let actor_user_id = request.actor_user_id.clone();
        let actor_device_id = request.device_id.clone();
        let actor_tenant_id = request.tenant_id.clone();
        let dispatch_now = x_build.now;
        let (voice_outcome, runtime_execution_envelope, ph1x_request) =
            self.run_voice_turn_and_build_ph1x_request_internal(store, request, x_build)?;
        let Some(ph1x_request) = ph1x_request else {
            let mut out = app_voice_turn_execution_outcome_from_voice_only(
                runtime_execution_envelope,
                request_session_state,
                voice_outcome,
            );
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, None)?;
            if !matches!(out.next_move, AppVoiceTurnNextMove::NotInvokedDisabled) {
                out.runtime_execution_envelope = self.emit_voice_turn_proof_and_attach(
                    store,
                    &out,
                    None,
                    actor_tenant_id.as_deref(),
                    received_at,
                    dispatch_now,
                )?;
            }
            return Ok(out);
        };

        let last_agent_input_packet = self.last_agent_input_packet.borrow().clone();
        let finder_terminal = self.run_finder_terminal_packet_for_turn(
            store,
            &actor_user_id,
            actor_tenant_id.as_deref(),
            last_agent_input_packet.as_ref(),
        )?;
        *self.last_finder_terminal_packet.borrow_mut() = finder_terminal.clone();
        let mut dev_intake_audit_event_id: Option<AuditEventId> = None;

        let mut out = if let Some(terminal) = finder_terminal.clone() {
            match terminal {
                FinderTerminalPacket::SimulationMatch(packet) => {
                    let nlp_output =
                        ph1x_request
                            .nlp_output
                            .as_ref()
                            .ok_or(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "ph1x_request.nlp_output",
                                    reason: "FINDER_MATCH_REQUIRES_INTENT_DRAFT",
                                },
                            ))?;
                    let Ph1nResponse::IntentDraft(intent_draft) = nlp_output else {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1x_request.nlp_output",
                                reason: "FINDER_MATCH_REQUIRES_INTENT_DRAFT",
                            },
                        ));
                    };
                    let intent_simulation_id =
                        simulation_id_for_intent_draft_v1(intent_draft)?.to_string();
                    if intent_simulation_id != packet.simulation_id {
                        return Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1x_request.nlp_output.intent_draft.intent_type",
                                reason: "FINDER_MATCH_EXECUTION_MISMATCH",
                            },
                        ));
                    }
                    self.run_ph1x_and_dispatch_with_access_fail_closed(
                        store,
                        runtime_execution_envelope.clone(),
                        voice_outcome,
                        request_session_state,
                        ph1x_request,
                        &actor_user_id,
                        actor_device_id.as_ref(),
                        actor_tenant_id.as_deref(),
                        request_session_id,
                        dispatch_now,
                    )?
                }
                FinderTerminalPacket::Clarify(packet) => AppVoiceTurnExecutionOutcome {
                    runtime_execution_envelope: runtime_execution_envelope.clone(),
                    voice_outcome,
                    session_state: request_session_state,
                    next_move: AppVoiceTurnNextMove::Clarify,
                    ph1x_request: Some(ph1x_request),
                    ph1x_response: None,
                    dispatch_outcome: None,
                    tool_response: None,
                    response_text: Some(packet.question),
                    reason_code: Some(packet.reason_code),
                },
                FinderTerminalPacket::Refuse(packet) => AppVoiceTurnExecutionOutcome {
                    runtime_execution_envelope: runtime_execution_envelope.clone(),
                    voice_outcome,
                    session_state: request_session_state,
                    next_move: AppVoiceTurnNextMove::Refused,
                    ph1x_request: Some(ph1x_request),
                    ph1x_response: None,
                    dispatch_outcome: None,
                    tool_response: None,
                    response_text: Some(packet.message),
                    reason_code: Some(packet.reason_code),
                },
                FinderTerminalPacket::MissingSimulation(packet) => {
                    let runtime_execution_envelope = missing_simulation_runtime_execution_envelope(
                        &self.ph1comp_runtime,
                        &runtime_execution_envelope,
                        &packet,
                        dispatch_now.0 as i64,
                    )?;
                    let tenant_id = normalized_tenant_scope_for_dev_intake(
                        store,
                        &actor_user_id,
                        actor_tenant_id.as_deref(),
                    )?;
                    let idempotency_key = format!(
                        "{}:{}:{}",
                        packet.idempotency_key, packet.tenant_id, packet.user_id
                    );
                    let dev_intake_event_id = store.ph1simfinder_dev_intake_commit(
                        dispatch_now,
                        tenant_id,
                        correlation_id,
                        turn_id,
                        request_session_id,
                        actor_user_id.clone(),
                        actor_device_id.clone(),
                        packet.requested_capability_name_normalized.clone(),
                        packet.proposed_simulation_family.clone(),
                        packet.no_match_proof_ref.clone(),
                        packet.dedupe_fingerprint.clone(),
                        packet.worthiness_score_bp,
                        packet.reason_code,
                        idempotency_key,
                    )?;
                    dev_intake_audit_event_id = Some(dev_intake_event_id);
                    if let Some(actor_device_id) = actor_device_id.as_ref() {
                        let _ = store.ph1x_respond_commit_with_payload_metadata(
                            dispatch_now,
                            packet.tenant_id.clone(),
                            correlation_id,
                            turn_id,
                            None,
                            actor_user_id.clone(),
                            actor_device_id.clone(),
                            "MISSING_SIMULATION_NOTIFY_SUBMITTED".to_string(),
                            packet.reason_code,
                            format!("ph1x_missing_sim_notify:{}:{}", correlation_id.0, turn_id.0),
                            self.ph1x_fail_closed_respond_payload_metadata(
                                &runtime_execution_envelope,
                            ),
                        )?;
                    }
                    AppVoiceTurnExecutionOutcome {
                        runtime_execution_envelope,
                        voice_outcome,
                        session_state: request_session_state,
                        next_move: AppVoiceTurnNextMove::Refused,
                        ph1x_request: Some(ph1x_request),
                        ph1x_response: None,
                        dispatch_outcome: None,
                        tool_response: None,
                        response_text: Some(
                            "I can't do that yet; I've submitted it for review.".to_string(),
                        ),
                        reason_code: Some(packet.reason_code),
                    }
                }
            }
        } else {
            self.run_ph1x_and_dispatch_with_access_fail_closed(
                store,
                runtime_execution_envelope.clone(),
                voice_outcome,
                request_session_state,
                ph1x_request,
                &actor_user_id,
                actor_device_id.as_ref(),
                actor_tenant_id.as_deref(),
                request_session_id,
                dispatch_now,
            )?
        };

        // Guardrail: persona hints are tone-only and must never affect
        // simulation candidate selection, access checks, or ACTIVE sim gating.
        let persona_style_hint =
            latest_persona_style_hint_for_actor(store, &actor_user_id, actor_tenant_id.as_deref());
        out.response_text = out.response_text.take().map(|response_text| {
            apply_persona_style_hint_to_response_text(response_text, persona_style_hint.as_deref())
        });
        out = self.finalize_voice_turn_outcome(
            store,
            out,
            finder_terminal.as_ref(),
            &actor_user_id,
            actor_device_id.as_ref(),
            actor_tenant_id.as_deref(),
            request_session_id,
            correlation_id,
            turn_id,
            received_at,
            dispatch_now,
        )?;
        if let Some(terminal) = finder_terminal.as_ref() {
            self.record_agent_execution_terminal_packet(
                store,
                &actor_user_id,
                actor_tenant_id.as_deref(),
                request_session_id,
                correlation_id,
                turn_id,
                dispatch_now,
                last_agent_input_packet.as_ref(),
                terminal,
                &out,
                dev_intake_audit_event_id,
            )?;
        }

        Ok(out)
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
        self.run_voice_turn_end_to_end(store, request, x_build)
    }

    fn run_finder_terminal_packet_for_turn(
        &self,
        store: &Ph1fStore,
        actor_user_id: &UserId,
        actor_tenant_id: Option<&str>,
        agent_input_packet: Option<&AgentInputPacket>,
    ) -> Result<Option<FinderTerminalPacket>, StorageError> {
        let Some(packet) = agent_input_packet else {
            return Ok(None);
        };
        let Some(run_request) = build_finder_run_request_for_simulation_intent(
            store,
            actor_user_id,
            actor_tenant_id,
            packet,
        )?
        else {
            return Ok(None);
        };
        let terminal = self
            .ph1simfinder_runtime
            .run(&run_request)
            .map_err(StorageError::ContractViolation)?;
        Ok(Some(terminal))
    }

    #[allow(clippy::too_many_arguments)]
    fn record_agent_execution_terminal_packet(
        &self,
        store: &mut Ph1fStore,
        actor_user_id: &UserId,
        actor_tenant_id: Option<&str>,
        request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        agent_input_packet: Option<&AgentInputPacket>,
        terminal: &FinderTerminalPacket,
        out: &AppVoiceTurnExecutionOutcome,
        dev_intake_audit_event_id: Option<AuditEventId>,
    ) -> Result<(), StorageError> {
        let tenant_id =
            normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
        let thread_key = agent_input_packet
            .and_then(|packet| packet.thread_key.clone())
            .unwrap_or_else(|| format!("corr:{}:turn:{}", correlation_id.0, turn_id.0));
        let (finder_packet_kind, simulation_id, fallback_reason_code) = match terminal {
            FinderTerminalPacket::SimulationMatch(packet) => (
                "SIMULATION_MATCH".to_string(),
                Some(packet.simulation_id.clone()),
                packet.reason_code,
            ),
            FinderTerminalPacket::Clarify(packet) => {
                ("CLARIFY".to_string(), None, packet.reason_code)
            }
            FinderTerminalPacket::Refuse(packet) => {
                ("REFUSE".to_string(), None, packet.reason_code)
            }
            FinderTerminalPacket::MissingSimulation(packet) => {
                ("MISSING_SIMULATION".to_string(), None, packet.reason_code)
            }
        };
        let execution_stage = agent_execution_stage_token_for_terminal(terminal, out.next_move);
        let reason_code = out.reason_code.unwrap_or(fallback_reason_code);
        let (
            access_decision,
            confirm_decision,
            active_simulation_proof_ref,
            simulation_idempotency_key,
        ) = match terminal {
            FinderTerminalPacket::SimulationMatch(packet) => (
                access_decision_for_match_outcome(out),
                confirm_decision_for_match_outcome(packet.confirm_required, out.next_move),
                Some(packet.active_check_proof_ref.clone()),
                Some(packet.idempotency_key.clone()),
            ),
            FinderTerminalPacket::Clarify(_) => ("N_A".to_string(), "N_A".to_string(), None, None),
            FinderTerminalPacket::Refuse(_) => ("N_A".to_string(), "N_A".to_string(), None, None),
            FinderTerminalPacket::MissingSimulation(_) => {
                ("N_A".to_string(), "N_A".to_string(), None, None)
            }
        };
        let dispatch_outcome_proof_ref = out
            .runtime_execution_envelope
            .proof_state
            .as_ref()
            .and_then(|state| state.proof_record_ref.clone())
            .or(out
                .dispatch_outcome
                .as_ref()
                .map(dispatch_outcome_proof_ref_token));
        let idempotency_key = format!(
            "agent_exec:{}:{}:{}:{}",
            correlation_id.0, turn_id.0, finder_packet_kind, execution_stage
        );
        let _ = store.append_agent_execution_ledger_row(AgentExecutionLedgerRowInput {
            created_at: now,
            tenant_id,
            user_id: actor_user_id.clone(),
            session_id: request_session_id,
            correlation_id,
            turn_id,
            thread_key,
            finder_packet_kind,
            execution_stage,
            simulation_id,
            access_decision,
            confirm_decision,
            active_simulation_proof_ref,
            simulation_idempotency_key,
            dispatch_outcome_proof_ref,
            reason_code,
            dev_intake_audit_event_id,
            idempotency_key: Some(idempotency_key),
        })?;
        Ok(())
    }

    fn execute_decided_ph1x_response(
        &self,
        store: &mut Ph1fStore,
        input: Ph1xDispatchRunInput<'_>,
        ph1x_response: Ph1xResponse,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        let Ph1xDispatchRunInput {
            runtime_execution_envelope,
            voice_outcome,
            session_state,
            ph1x_request,
            actor_user_id,
            dispatch_now,
        } = input;

        let mut out = AppVoiceTurnExecutionOutcome {
            runtime_execution_envelope,
            voice_outcome,
            session_state,
            next_move: AppVoiceTurnNextMove::Wait,
            ph1x_request: Some(ph1x_request),
            ph1x_response: Some(ph1x_response.clone()),
            dispatch_outcome: None,
            tool_response: None,
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
                    let tool_response = maybe_build_message_policy_tool_response(
                        store,
                        actor_user_id,
                        tool_request,
                    )?
                    .unwrap_or_else(|| self.ph1e_runtime.run(tool_request));
                    let tool_response_for_followup = tool_response.clone();
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
                        tool_response_for_followup,
                    )?;
                    let tool_followup_response = self
                        .ph1x_runtime
                        .decide(&tool_followup_request)
                        .map_err(StorageError::ContractViolation)?;

                    out.ph1x_request = Some(tool_followup_request);
                    out.ph1x_response = Some(tool_followup_response.clone());
                    out.tool_response = Some(tool_response);
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
                            actor_user_id.clone(),
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

    #[allow(clippy::too_many_arguments)]
    fn run_ph1x_and_dispatch_with_access_fail_closed(
        &self,
        store: &mut Ph1fStore,
        runtime_execution_envelope: RuntimeExecutionEnvelope,
        voice_outcome: OsVoiceLiveTurnOutcome,
        session_state: SessionState,
        ph1x_request: Ph1xRequest,
        actor_user_id: &UserId,
        actor_device_id: Option<&DeviceId>,
        actor_tenant_id: Option<&str>,
        request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
        dispatch_now: MonotonicTimeNs,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        let correlation_id = CorrelationId(ph1x_request.correlation_id);
        let turn_id = TurnId(ph1x_request.turn_id);
        let ph1x_response = self
            .ph1x_runtime
            .decide(&ph1x_request)
            .map_err(StorageError::ContractViolation)?;
        if let Some(drift_fail_closed) = classify_governance_drift_fail_closed_for_directive(
            &runtime_execution_envelope,
            &ph1x_response.directive,
            ph1x_response.reason_code,
        ) {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    drift_fail_closed.audit_response_kind.to_string(),
                    drift_fail_closed.reason_code,
                    format!(
                        "ph1x_governance_drift_fail_closed:{}:{}:{}",
                        correlation_id.0, turn_id.0, drift_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            return Ok(AppVoiceTurnExecutionOutcome {
                runtime_execution_envelope,
                voice_outcome,
                session_state,
                next_move: AppVoiceTurnNextMove::Refused,
                ph1x_request: Some(ph1x_request),
                ph1x_response: Some(ph1x_response),
                dispatch_outcome: None,
                tool_response: None,
                response_text: Some(drift_fail_closed.user_message.to_string()),
                reason_code: Some(drift_fail_closed.reason_code),
            });
        }
        match self.execute_decided_ph1x_response(
            store,
            Ph1xDispatchRunInput {
                runtime_execution_envelope: runtime_execution_envelope.clone(),
                voice_outcome: voice_outcome.clone(),
                session_state,
                ph1x_request: ph1x_request.clone(),
                actor_user_id,
                dispatch_now,
            },
            ph1x_response,
        ) {
            Ok(outcome) => Ok(outcome),
            Err(err) => {
                let Some(access_failure) = classify_access_fail_closed_error(&err) else {
                    return Err(err);
                };
                if let Some(actor_device_id) = actor_device_id {
                    let tenant_id = normalized_tenant_scope_for_dev_intake(
                        store,
                        actor_user_id,
                        actor_tenant_id,
                    )?;
                    let audit_session_id = request_session_id
                        .filter(|session_id| store.get_session(session_id).is_some());
                    let payload_metadata =
                        self.ph1x_fail_closed_respond_payload_metadata(&runtime_execution_envelope);
                    let _ = store.ph1x_respond_commit_with_payload_metadata(
                        dispatch_now,
                        tenant_id,
                        correlation_id,
                        turn_id,
                        audit_session_id,
                        actor_user_id.clone(),
                        actor_device_id.clone(),
                        access_failure.audit_response_kind.to_string(),
                        access_failure.reason_code,
                        format!(
                            "ph1x_access_fail_closed:{}:{}:{}",
                            correlation_id.0, turn_id.0, access_failure.audit_response_kind
                        ),
                        payload_metadata,
                    )?;
                }
                Ok(AppVoiceTurnExecutionOutcome {
                    runtime_execution_envelope,
                    voice_outcome,
                    session_state,
                    next_move: AppVoiceTurnNextMove::Refused,
                    ph1x_request: Some(ph1x_request),
                    ph1x_response: None,
                    dispatch_outcome: None,
                    tool_response: None,
                    response_text: Some(access_failure.user_message.to_string()),
                    reason_code: Some(access_failure.reason_code),
                })
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finalize_voice_turn_outcome(
        &self,
        store: &mut Ph1fStore,
        mut out: AppVoiceTurnExecutionOutcome,
        finder_terminal: Option<&FinderTerminalPacket>,
        actor_user_id: &UserId,
        actor_device_id: Option<&DeviceId>,
        actor_tenant_id: Option<&str>,
        request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        received_at: MonotonicTimeNs,
        dispatch_now: MonotonicTimeNs,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        out.runtime_execution_envelope =
            runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        if let Some(recovery_fail_closed) = classify_identity_recovery_fail_closed_outcome(&out) {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    recovery_fail_closed.audit_response_kind.to_string(),
                    recovery_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_recovery_fail_closed:{}:{}:{}",
                        correlation_id.0, turn_id.0, recovery_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(recovery_fail_closed.user_message.to_string());
            out.reason_code = Some(recovery_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(governance_quarantine_fail_closed) =
            classify_governance_quarantine_identity_recovery_fail_closed_outcome(&out)
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    governance_quarantine_fail_closed
                        .audit_response_kind
                        .to_string(),
                    governance_quarantine_fail_closed.audit_reason_code,
                    format!(
                        "ph1x_identity_recovery_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        governance_quarantine_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(governance_quarantine_fail_closed.user_message.to_string());
            out.reason_code = None;
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(identity_posture_fail_closed) =
            classify_low_confidence_identity_posture_fail_closed_outcome(&out)?
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    identity_posture_fail_closed.audit_response_kind.to_string(),
                    identity_posture_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_posture_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        identity_posture_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(identity_posture_fail_closed.user_message.to_string());
            out.reason_code = Some(identity_posture_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(identity_posture_fail_closed) =
            classify_gray_zone_margin_identity_posture_fail_closed_outcome(&out)?
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    identity_posture_fail_closed.audit_response_kind.to_string(),
                    identity_posture_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_posture_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        identity_posture_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(identity_posture_fail_closed.user_message.to_string());
            out.reason_code = Some(identity_posture_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(identity_posture_fail_closed) =
            classify_echo_unsafe_identity_posture_fail_closed_outcome(&out)?
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    identity_posture_fail_closed.audit_response_kind.to_string(),
                    identity_posture_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_posture_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        identity_posture_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(identity_posture_fail_closed.user_message.to_string());
            out.reason_code = Some(identity_posture_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(identity_posture_fail_closed) =
            classify_no_speech_identity_posture_fail_closed_outcome(&out)?
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    identity_posture_fail_closed.audit_response_kind.to_string(),
                    identity_posture_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_posture_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        identity_posture_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(identity_posture_fail_closed.user_message.to_string());
            out.reason_code = Some(identity_posture_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        if let Some(identity_posture_fail_closed) =
            classify_multi_speaker_identity_posture_fail_closed_outcome(&out)?
        {
            if let Some(actor_device_id) = actor_device_id {
                let tenant_id =
                    normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
                let audit_session_id =
                    request_session_id.filter(|session_id| store.get_session(session_id).is_some());
                let payload_metadata =
                    self.ph1x_fail_closed_respond_payload_metadata(&out.runtime_execution_envelope);
                let _ = store.ph1x_respond_commit_with_payload_metadata(
                    dispatch_now,
                    tenant_id,
                    correlation_id,
                    turn_id,
                    audit_session_id,
                    actor_user_id.clone(),
                    actor_device_id.clone(),
                    identity_posture_fail_closed.audit_response_kind.to_string(),
                    identity_posture_fail_closed.reason_code,
                    format!(
                        "ph1x_identity_posture_fail_closed:{}:{}:{}",
                        correlation_id.0,
                        turn_id.0,
                        identity_posture_fail_closed.audit_response_kind
                    ),
                    payload_metadata,
                )?;
            }
            out.next_move = AppVoiceTurnNextMove::Refused;
            out.response_text = Some(identity_posture_fail_closed.user_message.to_string());
            out.reason_code = Some(identity_posture_fail_closed.reason_code);
            out.runtime_execution_envelope =
                runtime_execution_envelope_with_authority_state_for_outcome(&out, finder_terminal)?;
        }
        out.runtime_execution_envelope = self.emit_voice_turn_proof_and_attach(
            store,
            &out,
            finder_terminal,
            actor_tenant_id,
            received_at,
            dispatch_now,
        )?;
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
        input: ForwardedVoicePh1xRequestInput<'_>,
    ) -> Result<Ph1xRequest, StorageError> {
        let agent_input_packet = self.build_agent_input_packet_for_forwarded_voice(
            store,
            input.correlation_id,
            input.turn_id,
            input.forwarded,
            input.request_session_id,
            input.tenant_id,
            input.x_build,
        )?;
        *self.last_agent_input_packet.borrow_mut() = Some(agent_input_packet.clone());
        build_ph1x_request_from_agent_input_packet(input.app_platform, &agent_input_packet)
    }

    pub fn debug_agent_input_packet_build_count(&self) -> u64 {
        *self.agent_input_packet_build_count.borrow()
    }

    pub fn debug_last_agent_input_packet(&self) -> Option<AgentInputPacket> {
        self.last_agent_input_packet.borrow().clone()
    }

    pub fn debug_last_finder_terminal_packet(&self) -> Option<FinderTerminalPacket> {
        self.last_finder_terminal_packet.borrow().clone()
    }

    #[allow(clippy::too_many_arguments)]
    fn build_agent_input_packet_for_forwarded_voice(
        &self,
        store: &mut Ph1fStore,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        forwarded: &crate::ph1os::OsVoiceLiveForwardBundle,
        request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
        tenant_id: Option<&str>,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<AgentInputPacket, StorageError> {
        *self.agent_input_packet_build_count.borrow_mut() += 1;
        let runtime_execution_envelope = forwarded.runtime_execution_envelope.clone();
        let voice_identity_assertion =
            canonical_forwarded_voice_identity_assertion(&runtime_execution_envelope)?.clone();
        let identity_state = runtime_execution_envelope
            .identity_state
            .clone()
            .ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "os_voice_live_forward_bundle.runtime_execution_envelope.identity_state",
                    reason:
                        "must carry canonical embedded identity state for forwarded packet build",
                })
            })?;
        let topic_hint = memory_topic_hint_from_nlp_output(x_build.nlp_output.as_ref());
        let runtime_memory_candidates = if memory_governance_blocked(&runtime_execution_envelope) {
            Vec::new()
        } else if identity_state_allows_memory_scope(&identity_state)
            && canonical_forwarded_voice_identity_confirmed(&runtime_execution_envelope)?
        {
            self.executor
                .collect_context_memory_candidates_for_voice_turn(
                    store,
                    x_build.now,
                    correlation_id,
                    turn_id,
                    &voice_identity_assertion,
                    x_build.policy_context_ref,
                    topic_hint,
                )
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let memory_state = memory_execution_state_from_candidates(
            &runtime_execution_envelope,
            &identity_state,
            &runtime_memory_candidates,
            None,
        )?;
        let runtime_execution_envelope = runtime_execution_envelope
            .with_memory_state(Some(memory_state))
            .map_err(StorageError::ContractViolation)?;
        let request_session_id = request_session_id.or(runtime_execution_envelope.session_id);
        let (sim_catalog_snapshot_hash, sim_catalog_snapshot_version) =
            simulation_catalog_snapshot_for_agent_input(store, tenant_id);
        let transcript_text = transcript_text_from_nlp_output(x_build.nlp_output.as_ref());
        let trace_id = runtime_execution_envelope.trace_id.clone();
        let packet_hash = agent_input_packet_hash_hex(
            correlation_id,
            turn_id,
            x_build.now,
            &trace_id,
            transcript_text.as_deref(),
            x_build.locale.as_deref(),
            request_session_id,
            x_build.session_state,
            x_build.thread_key.as_deref(),
            &x_build.thread_state,
            &runtime_memory_candidates,
            sim_catalog_snapshot_version,
            &sim_catalog_snapshot_hash,
            forwarded.identity_prompt_scope_key.as_deref(),
        );
        AgentInputPacket::v1_with_runtime_execution_envelope(
            correlation_id.0,
            turn_id.0,
            Some(runtime_execution_envelope),
            x_build.now,
            trace_id,
            packet_hash,
            transcript_text,
            x_build.locale.clone(),
            None,
            voice_identity_assertion,
            forwarded.identity_prompt_scope_key.clone(),
            request_session_id,
            x_build.session_state,
            x_build.thread_key.clone(),
            x_build.thread_state,
            x_build.policy_context_ref,
            runtime_memory_candidates,
            x_build.confirm_answer,
            x_build.nlp_output,
            x_build.tool_response,
            x_build.interruption,
            x_build.last_failure_reason_code,
            None,
            None,
            sim_catalog_snapshot_hash,
            sim_catalog_snapshot_version,
        )
        .map_err(StorageError::ContractViolation)
    }
}

fn app_voice_turn_execution_outcome_from_voice_only(
    runtime_execution_envelope: RuntimeExecutionEnvelope,
    session_state: SessionState,
    voice_outcome: OsVoiceLiveTurnOutcome,
) -> AppVoiceTurnExecutionOutcome {
    match voice_outcome {
        OsVoiceLiveTurnOutcome::NotInvokedDisabled => AppVoiceTurnExecutionOutcome {
            runtime_execution_envelope,
            voice_outcome: OsVoiceLiveTurnOutcome::NotInvokedDisabled,
            session_state,
            next_move: AppVoiceTurnNextMove::NotInvokedDisabled,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            tool_response: None,
            response_text: None,
            reason_code: None,
        },
        OsVoiceLiveTurnOutcome::Refused(refuse) => AppVoiceTurnExecutionOutcome {
            runtime_execution_envelope,
            voice_outcome: OsVoiceLiveTurnOutcome::Refused(refuse.clone()),
            session_state,
            next_move: AppVoiceTurnNextMove::Refused,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            tool_response: None,
            response_text: Some(refuse.message.clone()),
            reason_code: Some(refuse.reason_code),
        },
        OsVoiceLiveTurnOutcome::Forwarded(forwarded) => AppVoiceTurnExecutionOutcome {
            runtime_execution_envelope,
            voice_outcome: OsVoiceLiveTurnOutcome::Forwarded(forwarded),
            session_state,
            next_move: AppVoiceTurnNextMove::Wait,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            tool_response: None,
            response_text: None,
            reason_code: None,
        },
    }
}

fn canonical_forwarded_voice_identity_assertion(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Result<&Ph1VoiceIdResponse, StorageError> {
    runtime_execution_envelope
        .voice_identity_assertion
        .as_ref()
        .ok_or_else(|| {
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field:
                    "os_voice_live_forward_bundle.runtime_execution_envelope.voice_identity_assertion",
                reason:
                    "must carry canonical embedded voice identity assertion for forwarded packet build",
            })
        })
}

fn canonical_forwarded_voice_identity_confirmed(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Result<bool, StorageError> {
    Ok(matches!(
        canonical_forwarded_voice_identity_assertion(runtime_execution_envelope)?,
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok)
            if ok.identity_v2.identity_tier_v2 == IdentityTierV2::Confirmed
    ))
}

fn runtime_governance_storage_error(
    field: &'static str,
    decision: &RuntimeGovernanceDecision,
) -> StorageError {
    StorageError::ContractViolation(ContractViolation::InvalidValue {
        field,
        reason: runtime_governance_reason_literal(decision),
    })
}

fn runtime_law_storage_error(field: &'static str, decision: &RuntimeLawDecision) -> StorageError {
    StorageError::ContractViolation(ContractViolation::InvalidValue {
        field,
        reason: runtime_law_reason_literal(decision),
    })
}

struct Ph1xDispatchRunInput<'a> {
    runtime_execution_envelope: RuntimeExecutionEnvelope,
    voice_outcome: OsVoiceLiveTurnOutcome,
    session_state: SessionState,
    ph1x_request: Ph1xRequest,
    actor_user_id: &'a UserId,
    dispatch_now: MonotonicTimeNs,
}

struct ForwardedVoicePh1xRequestInput<'a> {
    correlation_id: CorrelationId,
    turn_id: TurnId,
    app_platform: AppPlatform,
    forwarded: &'a crate::ph1os::OsVoiceLiveForwardBundle,
    request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
    tenant_id: Option<&'a str>,
    x_build: AppVoicePh1xBuildInput,
}

type ProofSimulationTrace = (
    Option<SimulationId>,
    Option<SimulationVersion>,
    Option<String>,
);

fn proof_failure_class_for_storage_error(error: &StorageError) -> ProofFailureClass {
    match error {
        StorageError::ProofFailure { class, .. } => *class,
        StorageError::ContractViolation(_) => ProofFailureClass::ProofCanonicalizationFailure,
        StorageError::AppendOnlyViolation { .. } => ProofFailureClass::ProofChainIntegrityFailure,
        StorageError::ForeignKeyViolation { .. } | StorageError::DuplicateKey { .. } => {
            ProofFailureClass::ProofWriteFailure
        }
    }
}

fn proof_execution_state_from_error(
    error: &StorageError,
) -> Result<ProofExecutionState, StorageError> {
    let failure_class = proof_failure_class_for_storage_error(error);
    ProofExecutionState::v1(
        None,
        selene_kernel_contracts::ph1j::ProofWriteOutcome::Failed,
        Some(failure_class),
        match failure_class {
            ProofFailureClass::ProofChainIntegrityFailure
            | ProofFailureClass::ProofSignatureFailure => {
                selene_kernel_contracts::ph1j::ProofChainStatus::ChainBreakDetected
            }
            _ => selene_kernel_contracts::ph1j::ProofChainStatus::NotChecked,
        },
        match failure_class {
            ProofFailureClass::ProofVerificationUnavailable => {
                selene_kernel_contracts::ph1j::ProofVerificationPosture::VerificationUnavailable
            }
            _ => selene_kernel_contracts::ph1j::ProofVerificationPosture::NotRequested,
        },
        selene_kernel_contracts::ph1j::TimestampTrustPosture::RuntimeMonotonic,
        None,
    )
    .map_err(StorageError::ContractViolation)
}

fn voice_turn_runtime_law_action_class(
    out: &AppVoiceTurnExecutionOutcome,
) -> RuntimeProtectedActionClass {
    match out.next_move {
        AppVoiceTurnNextMove::Dispatch | AppVoiceTurnNextMove::Respond => {
            RuntimeProtectedActionClass::ProofRequired
        }
        AppVoiceTurnNextMove::Confirm
        | AppVoiceTurnNextMove::Clarify
        | AppVoiceTurnNextMove::Refused
        | AppVoiceTurnNextMove::Wait
        | AppVoiceTurnNextMove::NotInvokedDisabled => RuntimeProtectedActionClass::LowRisk,
    }
}

fn voice_turn_protected_action_class(
    out: &AppVoiceTurnExecutionOutcome,
) -> ProofProtectedActionClass {
    match out.dispatch_outcome {
        Some(SimulationDispatchOutcome::MemoryPropose(_))
        | Some(SimulationDispatchOutcome::MemoryRecall(_))
        | Some(SimulationDispatchOutcome::MemoryForget(_)) => {
            ProofProtectedActionClass::MemoryAuthoritativeMutation
        }
        Some(SimulationDispatchOutcome::Link(_))
        | Some(SimulationDispatchOutcome::LinkDelivered { .. }) => {
            ProofProtectedActionClass::ProtectedLinkGeneration
        }
        Some(SimulationDispatchOutcome::AccessGatePassed { .. })
        | Some(SimulationDispatchOutcome::AccessStepUp { .. })
        | Some(SimulationDispatchOutcome::CapreqLifecycle { .. }) => {
            ProofProtectedActionClass::AccessControlledAction
        }
        Some(_) => ProofProtectedActionClass::SimulationAuthorizedMutation,
        None if matches!(out.next_move, AppVoiceTurnNextMove::Refused) => {
            ProofProtectedActionClass::IdentitySensitiveExecution
        }
        None => ProofProtectedActionClass::VoiceTurnExecution,
    }
}

fn voice_turn_failure_class_token(out: &AppVoiceTurnExecutionOutcome) -> Option<String> {
    if matches!(out.next_move, AppVoiceTurnNextMove::Refused) {
        Some(
            selene_kernel_contracts::runtime_execution::FailureClass::PolicyViolation
                .as_str()
                .to_string(),
        )
    } else {
        None
    }
}

fn voice_turn_execution_outcome_token(out: &AppVoiceTurnExecutionOutcome) -> String {
    match out.next_move {
        AppVoiceTurnNextMove::Confirm => "CONFIRM".to_string(),
        AppVoiceTurnNextMove::Clarify => "CLARIFY".to_string(),
        AppVoiceTurnNextMove::Respond => "RESPOND".to_string(),
        AppVoiceTurnNextMove::Dispatch => out
            .dispatch_outcome
            .as_ref()
            .map(dispatch_outcome_proof_ref_token)
            .unwrap_or_else(|| "DISPATCH".to_string()),
        AppVoiceTurnNextMove::Refused => "REFUSED".to_string(),
        AppVoiceTurnNextMove::Wait => "WAIT".to_string(),
        AppVoiceTurnNextMove::NotInvokedDisabled => "NOT_INVOKED_DISABLED".to_string(),
    }
}

fn proof_policy_rule_identifiers(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Vec<String> {
    runtime_execution_envelope
        .governance_state
        .as_ref()
        .and_then(|state| state.last_rule_id.clone())
        .into_iter()
        .collect()
}

fn proof_authority_reason_token(reason_code: Option<u64>) -> String {
    reason_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn proof_authority_decision_reference(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Option<String> {
    runtime_execution_envelope.authority_state.as_ref().map(|state| {
        let policy_context_ref = state
            .policy_context_ref
            .map(|value| {
                format!(
                    "privacy_mode={};do_not_disturb={};safety_tier={}",
                    value.privacy_mode,
                    value.do_not_disturb,
                    match value.safety_tier {
                        SafetyTier::Standard => "STANDARD",
                        SafetyTier::Strict => "STRICT",
                    }
                )
            })
            .unwrap_or_else(|| "-".to_string());
        format!(
            "policy_decision={};policy_context={};identity_scope_required={};identity_scope_satisfied={};memory_scope_allowed={};reason_code={}",
            state.policy_decision.as_str(),
            policy_context_ref,
            state.identity_scope_required,
            state.identity_scope_satisfied,
            state.memory_scope_allowed,
            proof_authority_reason_token(state.reason_code),
        )
    })
}

fn proof_verifier_metadata_ref(runtime_execution_envelope: &RuntimeExecutionEnvelope) -> String {
    if let Some(session_id) = runtime_execution_envelope.session_id {
        format!(
            "session:{}:turn:{}:request:{}",
            session_id.0,
            runtime_execution_envelope.turn_id.0,
            runtime_execution_envelope.request_id
        )
    } else {
        format!("request:{}", runtime_execution_envelope.request_id)
    }
}

fn proof_simulation_trace_for_voice_turn(
    store: &Ph1fStore,
    out: &AppVoiceTurnExecutionOutcome,
    finder_terminal: Option<&FinderTerminalPacket>,
    actor_tenant_id: Option<&str>,
) -> Result<ProofSimulationTrace, StorageError> {
    let runtime_execution_envelope = &out.runtime_execution_envelope;
    let simulation_id = match finder_terminal {
        Some(FinderTerminalPacket::SimulationMatch(packet)) => Some(
            SimulationId::new(packet.simulation_id.clone())
                .map_err(StorageError::ContractViolation)?,
        ),
        _ => proof_simulation_id_from_voice_outcome(out)?,
    };
    let simulation_version =
        if let (Some(tenant_id), Some(simulation_id)) = (actor_tenant_id, simulation_id.as_ref()) {
            let tenant_id =
                TenantId::new(tenant_id.to_string()).map_err(StorageError::ContractViolation)?;
            store
                .simulation_catalog_current_row(&tenant_id, simulation_id)
                .map(|row| row.simulation_version)
        } else {
            None
        };
    let simulation_certification_state = runtime_execution_envelope
        .authority_state
        .as_ref()
        .map(|state| state.simulation_certification_state.as_str().to_string());
    Ok((
        simulation_id,
        simulation_version,
        simulation_certification_state,
    ))
}

fn proof_simulation_id_from_voice_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<SimulationId>, StorageError> {
    let Some(ph1x_request) = out.ph1x_request.as_ref() else {
        return Ok(None);
    };
    if let Some(Ph1nResponse::IntentDraft(intent_draft)) = ph1x_request.nlp_output.as_ref() {
        return proof_simulation_id_for_intent_draft(intent_draft);
    }
    match ph1x_request.thread_state.pending.as_ref() {
        Some(PendingState::Confirm { intent_draft, .. })
        | Some(PendingState::StepUp { intent_draft, .. }) => {
            proof_simulation_id_for_intent_draft(intent_draft)
        }
        _ => Ok(None),
    }
}

fn proof_simulation_id_for_intent_draft(
    intent_draft: &IntentDraft,
) -> Result<Option<SimulationId>, StorageError> {
    match simulation_id_for_intent_draft_v1(intent_draft) {
        Ok(simulation_id) => SimulationId::new(simulation_id.to_string())
            .map(Some)
            .map_err(StorageError::ContractViolation),
        Err(StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }))
            if field == "simulation_candidate_dispatch.intent_draft.intent_type"
                && reason == "SIM_DISPATCH_GUARD_SIMULATION_ID_INVALID" =>
        {
            // Missing-simulation paths for unsupported intent families have no canonical
            // simulation id to carry into proof tracing.
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

fn proof_execution_state_from_receipt(
    receipt: selene_kernel_contracts::ph1j::ProofWriteReceipt,
) -> Result<ProofExecutionState, StorageError> {
    ProofExecutionState::v1(
        Some(receipt.proof_record_ref),
        receipt.proof_write_outcome,
        None,
        receipt.proof_chain_status,
        receipt.proof_verification_posture,
        receipt.timestamp_trust_posture,
        receipt.verifier_metadata_ref,
    )
    .map_err(StorageError::ContractViolation)
}

fn runtime_execution_envelope_with_voice_turn_proof(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    proof_state: ProofExecutionState,
) -> Result<RuntimeExecutionEnvelope, StorageError> {
    runtime_execution_envelope
        .with_proof_state(Some(proof_state))
        .map_err(StorageError::ContractViolation)
}

const GOVERNED_SUBSYSTEM_MEMORY_ENGINE: &str = "MEMORY_ENGINE";
const GOVERNED_SUBSYSTEM_AUTHORITY_LAYER: &str = "AUTHORITY_LAYER";
const GOVERNED_SUBSYSTEM_IDENTITY_VOICE_ENGINE: &str = "IDENTITY_VOICE_ENGINE";

fn missing_simulation_runtime_execution_envelope(
    ph1comp_runtime: &Ph1CompRuntime,
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    packet: &selene_kernel_contracts::ph1simfinder::MissingSimulationPacket,
    created_at_ms: i64,
) -> Result<RuntimeExecutionEnvelope, StorageError> {
    let (_, computation_state) = ph1comp_runtime
        .build_missing_simulation_packet_and_state(
            runtime_execution_envelope,
            packet,
            created_at_ms,
        )
        .map_err(|_| {
            StorageError::ContractViolation(ContractViolation::InvalidValue {
                field:
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.computation_state",
                reason: "ph1comp_missing_simulation_computation_failed",
            })
        })?;
    runtime_execution_envelope
        .with_computation_state(Some(computation_state))
        .map_err(StorageError::ContractViolation)
}

fn identity_state_allows_memory_scope(identity_state: &IdentityExecutionState) -> bool {
    matches!(identity_state.trust_tier, IdentityTrustTier::Verified)
        && !identity_state.step_up_required
        && identity_state.recovery_state == IdentityRecoveryState::None
}

fn governance_quarantines_subsystem(
    governance_state: &GovernanceExecutionState,
    subsystem_id: &str,
) -> bool {
    governance_state
        .quarantined_subsystems
        .iter()
        .any(|subsystem| subsystem == subsystem_id)
}

fn memory_governance_blocked(runtime_execution_envelope: &RuntimeExecutionEnvelope) -> bool {
    runtime_execution_envelope
        .governance_state
        .as_ref()
        .map(|state| {
            state.safe_mode_active
                || state.cluster_consistency == GovernanceClusterConsistency::Diverged
                || governance_quarantines_subsystem(state, GOVERNED_SUBSYSTEM_MEMORY_ENGINE)
                || governance_quarantines_subsystem(state, GOVERNED_SUBSYSTEM_AUTHORITY_LAYER)
        })
        .unwrap_or(false)
}

fn memory_consistency_level_from_envelope(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> MemoryConsistencyLevel {
    match runtime_execution_envelope
        .persistence_state
        .as_ref()
        .map(|state| state.recovery_mode)
    {
        Some(selene_kernel_contracts::runtime_execution::PersistenceRecoveryMode::Normal) | None => {
            MemoryConsistencyLevel::StrictLedger
        }
        Some(
            selene_kernel_contracts::runtime_execution::PersistenceRecoveryMode::Recovering
            | selene_kernel_contracts::runtime_execution::PersistenceRecoveryMode::DegradedRecovery,
        ) => MemoryConsistencyLevel::RecoveryRebuild,
        Some(
            selene_kernel_contracts::runtime_execution::PersistenceRecoveryMode::QuarantinedLocalState,
        ) => MemoryConsistencyLevel::EventualView,
    }
}

fn memory_confidence_floor(candidates: &[MemoryCandidate]) -> Option<MemoryConfidence> {
    if candidates
        .iter()
        .any(|candidate| candidate.confidence == MemoryConfidence::Low)
    {
        return Some(MemoryConfidence::Low);
    }
    if candidates
        .iter()
        .any(|candidate| candidate.confidence == MemoryConfidence::Med)
    {
        return Some(MemoryConfidence::Med);
    }
    if candidates
        .iter()
        .any(|candidate| candidate.confidence == MemoryConfidence::High)
    {
        return Some(MemoryConfidence::High);
    }
    None
}

fn memory_trust_level_from_confidence_floor(
    confidence_floor: Option<MemoryConfidence>,
    ledger_write_accepted: bool,
) -> MemoryTrustLevel {
    match confidence_floor {
        Some(MemoryConfidence::High) if ledger_write_accepted => MemoryTrustLevel::Verified,
        Some(MemoryConfidence::High) => MemoryTrustLevel::HighConfidence,
        Some(MemoryConfidence::Med | MemoryConfidence::Low) => MemoryTrustLevel::LowConfidence,
        None => MemoryTrustLevel::Unverified,
    }
}

fn memory_execution_state_from_candidates(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    identity_state: &IdentityExecutionState,
    candidates: &[MemoryCandidate],
    reason_code: Option<ReasonCodeId>,
) -> Result<MemoryExecutionState, StorageError> {
    let governance_blocked = memory_governance_blocked(runtime_execution_envelope);
    let confidence_floor = memory_confidence_floor(candidates);
    let eligibility_decision = if governance_blocked {
        MemoryEligibilityDecision::GovernedBlocked
    } else if !identity_state_allows_memory_scope(identity_state) {
        MemoryEligibilityDecision::IdentityScopeBlocked
    } else if candidates.is_empty() {
        MemoryEligibilityDecision::NoEligibleCandidates
    } else {
        MemoryEligibilityDecision::Eligible
    };
    MemoryExecutionState::v1(
        true,
        memory_consistency_level_from_envelope(runtime_execution_envelope),
        memory_trust_level_from_confidence_floor(confidence_floor, false),
        eligibility_decision,
        confidence_floor,
        candidates.len() as u16,
        false,
        0,
        governance_blocked,
        reason_code.map(|code| u64::from(code.0)),
    )
    .map_err(StorageError::ContractViolation)
}

fn memory_execution_state_from_dispatch_outcome(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    existing_state: Option<&MemoryExecutionState>,
    dispatch_outcome: Option<&SimulationDispatchOutcome>,
) -> Result<Option<MemoryExecutionState>, StorageError> {
    let governance_blocked = memory_governance_blocked(runtime_execution_envelope);
    let Some(dispatch_outcome) = dispatch_outcome else {
        return Ok(existing_state.cloned());
    };
    let state = match dispatch_outcome {
        SimulationDispatchOutcome::MemoryPropose(resp) => {
            let confidence_floor = if resp.ledger_events.is_empty() {
                None
            } else if resp
                .ledger_events
                .iter()
                .any(|event| event.confidence == MemoryConfidence::Low)
            {
                Some(MemoryConfidence::Low)
            } else if resp
                .ledger_events
                .iter()
                .any(|event| event.confidence == MemoryConfidence::Med)
            {
                Some(MemoryConfidence::Med)
            } else {
                Some(MemoryConfidence::High)
            };
            let ledger_write_accepted = !resp.ledger_events.is_empty();
            let reason_code = resp
                .decisions
                .first()
                .map(|decision| decision.reason_code)
                .or_else(|| resp.ledger_events.first().map(|event| event.reason_code));
            MemoryExecutionState::v1(
                true,
                memory_consistency_level_from_envelope(runtime_execution_envelope),
                memory_trust_level_from_confidence_floor(confidence_floor, ledger_write_accepted),
                if governance_blocked {
                    MemoryEligibilityDecision::GovernedBlocked
                } else if ledger_write_accepted {
                    MemoryEligibilityDecision::Eligible
                } else {
                    MemoryEligibilityDecision::PolicyBlocked
                },
                confidence_floor,
                resp.decisions.len() as u16,
                ledger_write_accepted,
                resp.ledger_events.len() as u16,
                governance_blocked,
                reason_code.map(|code| u64::from(code.0)),
            )
            .map_err(StorageError::ContractViolation)?
        }
        SimulationDispatchOutcome::MemoryForget(resp) => {
            let confidence_floor = resp.ledger_event.as_ref().map(|event| event.confidence);
            let ledger_write_accepted = resp.forgotten && resp.ledger_event.is_some();
            let reason_code = resp
                .fail_reason_code
                .or_else(|| resp.ledger_event.as_ref().map(|event| event.reason_code));
            MemoryExecutionState::v1(
                true,
                memory_consistency_level_from_envelope(runtime_execution_envelope),
                memory_trust_level_from_confidence_floor(confidence_floor, ledger_write_accepted),
                if governance_blocked {
                    MemoryEligibilityDecision::GovernedBlocked
                } else if ledger_write_accepted {
                    MemoryEligibilityDecision::Eligible
                } else {
                    MemoryEligibilityDecision::PolicyBlocked
                },
                confidence_floor,
                if resp.forgotten { 1 } else { 0 },
                ledger_write_accepted,
                if resp.ledger_event.is_some() { 1 } else { 0 },
                governance_blocked,
                reason_code.map(|code| u64::from(code.0)),
            )
            .map_err(StorageError::ContractViolation)?
        }
        SimulationDispatchOutcome::MemoryRecall(resp) => memory_execution_state_from_candidates(
            runtime_execution_envelope,
            runtime_execution_envelope
                .identity_state
                .as_ref()
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state",
                        reason: "memory recall requires identity execution state",
                    },
                ))?,
            &resp.candidates,
            resp.fail_reason_code,
        )?,
        _ => return Ok(existing_state.cloned()),
    };
    Ok(Some(state))
}

fn runtime_execution_envelope_with_authority_state_for_outcome(
    out: &AppVoiceTurnExecutionOutcome,
    finder_terminal: Option<&FinderTerminalPacket>,
) -> Result<RuntimeExecutionEnvelope, StorageError> {
    let mut runtime_execution_envelope = out.runtime_execution_envelope.clone();
    if let Some(memory_state) = memory_execution_state_from_dispatch_outcome(
        &runtime_execution_envelope,
        runtime_execution_envelope.memory_state.as_ref(),
        out.dispatch_outcome.as_ref(),
    )? {
        runtime_execution_envelope = runtime_execution_envelope
            .with_memory_state(Some(memory_state))
            .map_err(StorageError::ContractViolation)?;
    }
    let authority_state =
        authority_execution_state_for_outcome(out, finder_terminal, &runtime_execution_envelope)?;
    runtime_execution_envelope
        .with_authority_state(Some(authority_state))
        .map_err(StorageError::ContractViolation)
}

fn authority_execution_state_for_outcome(
    out: &AppVoiceTurnExecutionOutcome,
    finder_terminal: Option<&FinderTerminalPacket>,
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Result<AuthorityExecutionState, StorageError> {
    let policy_context_ref = out
        .ph1x_request
        .as_ref()
        .map(|request| request.policy_context_ref);
    let simulation_certification_state =
        simulation_certification_state_for_outcome(out, finder_terminal);
    let policy_decision = authority_policy_decision_for_outcome(out);
    let identity_scope_required = out
        .ph1x_request
        .as_ref()
        .and_then(intent_type_from_ph1x_request)
        .map(intent_requires_identity_scope)
        .unwrap_or(false);
    let identity_scope_satisfied = if !identity_scope_required {
        true
    } else {
        runtime_execution_envelope
            .identity_state
            .as_ref()
            .map(|state| {
                matches!(
                    state.trust_tier,
                    IdentityTrustTier::Verified | IdentityTrustTier::HighConfidence
                ) && !state.step_up_required
            })
            .unwrap_or(false)
    };
    let memory_scope_allowed = runtime_execution_envelope
        .memory_state
        .as_ref()
        .map(|state| state.eligibility_decision == MemoryEligibilityDecision::Eligible)
        .unwrap_or(false);
    AuthorityExecutionState::v1(
        policy_context_ref,
        simulation_certification_state,
        OnboardingReadinessState::NotApplicable,
        policy_decision,
        identity_scope_required,
        identity_scope_satisfied,
        memory_scope_allowed,
        out.reason_code.map(|code| u64::from(code.0)),
    )
    .map_err(StorageError::ContractViolation)
}

fn refused_outcome_requires_step_up(reason_code: Option<ReasonCodeId>) -> bool {
    matches!(
        reason_code,
        Some(code)
            if code == sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED
                || code == voice_id_reason_codes::VID_REAUTH_REQUIRED
                || code == voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED
                || code == voice_id_reason_codes::VID_SPOOF_RISK
    )
}

fn simulation_certification_state_for_outcome(
    out: &AppVoiceTurnExecutionOutcome,
    finder_terminal: Option<&FinderTerminalPacket>,
) -> SimulationCertificationState {
    if let Some(dispatch_outcome) = out.dispatch_outcome.as_ref() {
        return match dispatch_outcome {
            SimulationDispatchOutcome::AccessStepUp { .. } => {
                SimulationCertificationState::StepUpRequired
            }
            _ => SimulationCertificationState::CertifiedActive,
        };
    }
    if matches!(
        finder_terminal,
        Some(FinderTerminalPacket::MissingSimulation(_))
    ) {
        return SimulationCertificationState::MissingSimulationPath;
    }
    if out.reason_code
        == Some(crate::simulation_executor::reason_codes::SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE)
    {
        return SimulationCertificationState::InactiveSimulation;
    }
    if refused_outcome_requires_step_up(out.reason_code) {
        return SimulationCertificationState::StepUpRequired;
    }
    if matches!(
        out.ph1x_response.as_ref().map(|response| &response.directive),
        Some(Ph1xDirective::Dispatch(dispatch))
            if matches!(dispatch.dispatch_request, DispatchRequest::AccessStepUp(_))
    ) {
        return SimulationCertificationState::StepUpRequired;
    }
    SimulationCertificationState::NotRequested
}

fn authority_policy_decision_for_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> AuthorityPolicyDecision {
    match out.next_move {
        AppVoiceTurnNextMove::Confirm | AppVoiceTurnNextMove::Clarify => {
            AuthorityPolicyDecision::PendingConfirmation
        }
        AppVoiceTurnNextMove::Dispatch => match out.dispatch_outcome {
            Some(SimulationDispatchOutcome::AccessStepUp { .. }) => {
                AuthorityPolicyDecision::StepUpRequired
            }
            _ => AuthorityPolicyDecision::Allowed,
        },
        AppVoiceTurnNextMove::Respond => AuthorityPolicyDecision::Allowed,
        AppVoiceTurnNextMove::Refused => {
            if refused_outcome_requires_step_up(out.reason_code) {
                AuthorityPolicyDecision::StepUpRequired
            } else {
                AuthorityPolicyDecision::Denied
            }
        }
        AppVoiceTurnNextMove::Wait | AppVoiceTurnNextMove::NotInvokedDisabled => {
            AuthorityPolicyDecision::NotRequested
        }
    }
}

fn intent_type_from_ph1x_request(request: &Ph1xRequest) -> Option<IntentType> {
    match request.nlp_output.as_ref() {
        Some(Ph1nResponse::IntentDraft(intent_draft)) => Some(intent_draft.intent_type),
        _ => None,
    }
}

fn intent_requires_identity_scope(intent_type: IntentType) -> bool {
    matches!(
        intent_type,
        IntentType::MemoryRememberRequest
            | IntentType::MemoryForgetRequest
            | IntentType::MemoryQuery
            | IntentType::SendMoney
            | IntentType::CreateInviteLink
            | IntentType::CapreqManage
            | IntentType::AccessSchemaManage
            | IntentType::AccessEscalationVote
            | IntentType::AccessInstanceCompileRefresh
    )
}

fn runtime_governance_reason_literal(decision: &RuntimeGovernanceDecision) -> &'static str {
    match decision.reason_code.as_str() {
        crate::runtime_governance::reason_codes::GOV_ENVELOPE_SESSION_REQUIRED => {
            "governance_policy_block GOV_ENVELOPE_SESSION_REQUIRED"
        }
        crate::runtime_governance::reason_codes::GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED => {
            "governance_policy_block GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED"
        }
        crate::runtime_governance::reason_codes::GOV_PERSISTENCE_DEGRADED => {
            "governance_degrade GOV_PERSISTENCE_DEGRADED"
        }
        crate::runtime_governance::reason_codes::GOV_PERSISTENCE_STALE_REJECTED => {
            "session_conflict GOV_PERSISTENCE_STALE_REJECTED"
        }
        crate::runtime_governance::reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED => {
            "session_conflict GOV_PERSISTENCE_QUARANTINE_REQUIRED"
        }
        crate::runtime_governance::reason_codes::GOV_PROOF_REQUIRED => {
            "governance_policy_block GOV_PROOF_REQUIRED"
        }
        crate::runtime_governance::reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN => {
            "governance_safe_mode GOV_GOVERNANCE_INTEGRITY_UNCERTAIN"
        }
        crate::runtime_governance::reason_codes::GOV_POLICY_VERSION_DRIFT => {
            "governance_degrade GOV_POLICY_VERSION_DRIFT"
        }
        crate::runtime_governance::reason_codes::GOV_SUBSYSTEM_CERTIFICATION_REGRESSED => {
            "governance_safe_mode GOV_SUBSYSTEM_CERTIFICATION_REGRESSED"
        }
        crate::runtime_governance::reason_codes::GOV_SAFE_MODE_ACTIVE => {
            "governance_safe_mode GOV_SAFE_MODE_ACTIVE"
        }
        _ => match decision.response_class {
            selene_kernel_contracts::runtime_governance::GovernanceResponseClass::Quarantine => {
                "session_conflict runtime_governance_quarantine"
            }
            selene_kernel_contracts::runtime_governance::GovernanceResponseClass::Block => {
                "governance_policy_block runtime_governance_block"
            }
            selene_kernel_contracts::runtime_governance::GovernanceResponseClass::SafeMode => {
                "governance_safe_mode runtime_governance_safe_mode"
            }
            selene_kernel_contracts::runtime_governance::GovernanceResponseClass::Degrade => {
                "governance_degrade runtime_governance_degrade"
            }
            selene_kernel_contracts::runtime_governance::GovernanceResponseClass::Allow
            | selene_kernel_contracts::runtime_governance::GovernanceResponseClass::AllowWithWarning => {
                "runtime_governance runtime_governance_allowed"
            }
        },
    }
}

fn runtime_law_reason_literal(decision: &RuntimeLawDecision) -> &'static str {
    match decision.response_class {
        RuntimeLawResponseClass::SafeMode => "runtime_law_safe_mode final_runtime_law_safe_mode",
        RuntimeLawResponseClass::Quarantine => {
            "runtime_law_quarantine final_runtime_law_quarantine"
        }
        RuntimeLawResponseClass::Block => "runtime_law_block final_runtime_law_block",
        RuntimeLawResponseClass::Degrade => "runtime_law_degrade final_runtime_law_degrade",
        RuntimeLawResponseClass::Allow | RuntimeLawResponseClass::AllowWithWarning => {
            "runtime_law_allow final_runtime_law_allow"
        }
    }
}

fn agent_execution_stage_token_for_terminal(
    terminal: &FinderTerminalPacket,
    next_move: AppVoiceTurnNextMove,
) -> String {
    match terminal {
        FinderTerminalPacket::SimulationMatch(_) => match next_move {
            AppVoiceTurnNextMove::Confirm => "MATCH_CONFIRM".to_string(),
            AppVoiceTurnNextMove::Dispatch => "MATCH_DISPATCH".to_string(),
            AppVoiceTurnNextMove::Clarify => "MATCH_CLARIFY".to_string(),
            AppVoiceTurnNextMove::Respond => "MATCH_RESPOND".to_string(),
            AppVoiceTurnNextMove::Refused => "MATCH_REFUSED".to_string(),
            AppVoiceTurnNextMove::Wait => "MATCH_WAIT".to_string(),
            AppVoiceTurnNextMove::NotInvokedDisabled => "MATCH_NOT_INVOKED".to_string(),
        },
        FinderTerminalPacket::Clarify(_) => "CLARIFY_QUESTION".to_string(),
        FinderTerminalPacket::Refuse(_) => "REFUSE_FAIL_CLOSED".to_string(),
        FinderTerminalPacket::MissingSimulation(_) => "MISSING_SIM_DEV_INTAKE".to_string(),
    }
}

#[derive(Debug, Clone, Copy)]
struct AccessFailClosedBehavior {
    reason_code: ReasonCodeId,
    user_message: &'static str,
    audit_response_kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct IdentityRecoveryFailClosedBehavior {
    reason_code: ReasonCodeId,
    user_message: &'static str,
    audit_response_kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct IdentityPostureFailClosedBehavior {
    reason_code: ReasonCodeId,
    user_message: &'static str,
    audit_response_kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct GovernanceDriftFailClosedBehavior {
    reason_code: ReasonCodeId,
    user_message: &'static str,
    audit_response_kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct GovernanceQuarantineFailClosedBehavior {
    audit_reason_code: ReasonCodeId,
    user_message: &'static str,
    audit_response_kind: &'static str,
}

fn identity_consistency_level_literal(value: IdentityVerificationConsistencyLevel) -> &'static str {
    match value {
        IdentityVerificationConsistencyLevel::StrictVerified => "STRICT_VERIFIED",
        IdentityVerificationConsistencyLevel::HighConfidenceVerified => "HIGH_CONFIDENCE_VERIFIED",
        IdentityVerificationConsistencyLevel::DegradedVerification => "DEGRADED_VERIFICATION",
        IdentityVerificationConsistencyLevel::RecoveryRestricted => "RECOVERY_RESTRICTED",
    }
}

fn identity_trust_tier_literal(value: IdentityTrustTier) -> &'static str {
    match value {
        IdentityTrustTier::Verified => "VERIFIED",
        IdentityTrustTier::HighConfidence => "HIGH_CONFIDENCE",
        IdentityTrustTier::Conditional => "CONDITIONAL",
        IdentityTrustTier::Restricted => "RESTRICTED",
        IdentityTrustTier::Rejected => "REJECTED",
    }
}

fn identity_recovery_state_literal(value: IdentityRecoveryState) -> &'static str {
    match value {
        IdentityRecoveryState::None => "NONE",
        IdentityRecoveryState::ReauthRequired => "REAUTH_REQUIRED",
        IdentityRecoveryState::ReEnrollmentRequired => "RE_ENROLLMENT_REQUIRED",
        IdentityRecoveryState::RecoveryRestricted => "RECOVERY_RESTRICTED",
    }
}

fn identity_tier_v2_literal(value: IdentityTierV2) -> &'static str {
    match value {
        IdentityTierV2::Confirmed => "CONFIRMED",
        IdentityTierV2::Probable => "PROBABLE",
        IdentityTierV2::Unknown => "UNKNOWN",
    }
}

fn spoof_liveness_status_literal(value: SpoofLivenessStatus) -> &'static str {
    match value {
        SpoofLivenessStatus::Live => "LIVE",
        SpoofLivenessStatus::SuspectedSpoof => "SUSPECTED_SPOOF",
        SpoofLivenessStatus::Unknown => "UNKNOWN",
    }
}

fn identity_reason_code_or(
    identity_state: &IdentityExecutionState,
    fallback: ReasonCodeId,
) -> ReasonCodeId {
    identity_state
        .reason_code
        .and_then(|code| u32::try_from(code).ok())
        .map(ReasonCodeId)
        .unwrap_or(fallback)
}

fn identity_reason_code(identity_state: &IdentityExecutionState) -> Option<ReasonCodeId> {
    identity_state
        .reason_code
        .and_then(|code| u32::try_from(code).ok())
        .map(ReasonCodeId)
}

fn posture_fail_closed_reason_code(reason_code: Option<ReasonCodeId>) -> bool {
    matches!(
        reason_code,
        Some(code)
            if code == voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE
                || code == voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN
                || code == voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE
                || code == voice_id_reason_codes::VID_FAIL_NO_SPEECH
                || code == voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT
    )
}

fn voice_identity_reason_code(assertion: &Ph1VoiceIdResponse) -> Option<ReasonCodeId> {
    match assertion {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.reason_code,
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(unknown) => Some(unknown.reason_code),
    }
}

fn canonical_posture_identity_state_matches_shape(
    identity_state: &IdentityExecutionState,
    posture_reason_code: ReasonCodeId,
) -> bool {
    match posture_reason_code {
        code
            if code == voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE
                || code == voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN =>
        {
            identity_state.consistency_level
                == IdentityVerificationConsistencyLevel::DegradedVerification
                && identity_state.trust_tier == IdentityTrustTier::Conditional
                && !identity_state.step_up_required
                && identity_state.recovery_state == IdentityRecoveryState::None
        }
        code
            if code == voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE
                || code == voice_id_reason_codes::VID_FAIL_NO_SPEECH
                || code == voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT =>
        {
            identity_state.consistency_level
                == IdentityVerificationConsistencyLevel::DegradedVerification
                && identity_state.trust_tier == IdentityTrustTier::Restricted
                && !identity_state.step_up_required
                && identity_state.recovery_state == IdentityRecoveryState::None
        }
        _ => false,
    }
}

fn require_canonical_posture_identity_state_shape(
    identity_state: &IdentityExecutionState,
    posture_reason_code: ReasonCodeId,
) -> Result<(), StorageError> {
    if canonical_posture_identity_state_matches_shape(identity_state, posture_reason_code) {
        Ok(())
    } else {
        Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state",
                reason: "must match canonical posture identity state shape for fail-closed classification",
            },
        ))
    }
}

fn canonical_posture_fail_closed_identity_state(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<&IdentityExecutionState>, StorageError> {
    if !matches!(
        out.next_move,
        AppVoiceTurnNextMove::Dispatch | AppVoiceTurnNextMove::Respond
    ) {
        return Ok(None);
    }
    let identity_state_reason_code = out
        .runtime_execution_envelope
        .identity_state
        .as_ref()
        .and_then(identity_reason_code);
    let voice_assertion_reason_code = out
        .runtime_execution_envelope
        .voice_identity_assertion
        .as_ref()
        .and_then(voice_identity_reason_code);
    let posture_reason_code = identity_state_reason_code.or(voice_assertion_reason_code);
    if !posture_fail_closed_reason_code(posture_reason_code) {
        return Ok(None);
    }
    let voice_identity_assertion = out
        .runtime_execution_envelope
        .voice_identity_assertion
        .as_ref()
        .ok_or(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field:
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.voice_identity_assertion",
                reason: "must carry canonical voice identity assertion for posture fail-closed classification",
            },
        ))?;
    let identity_state = out
        .runtime_execution_envelope
        .identity_state
        .as_ref()
        .ok_or(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state",
                reason: "must carry canonical identity state for posture fail-closed classification",
            },
        ))?;
    let voice_assertion_reason_code = voice_identity_reason_code(voice_identity_assertion);
    if !posture_fail_closed_reason_code(voice_assertion_reason_code) {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field:
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.voice_identity_assertion.reason_code",
                reason:
                    "must carry canonical posture reason code for posture fail-closed classification",
            },
                ));
    }
    let posture_reason_code =
        voice_assertion_reason_code.expect("posture voice assertion reason must remain present");
    if identity_reason_code(identity_state) != voice_assertion_reason_code {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field:
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state.reason_code",
                reason:
                    "must match canonical voice identity assertion reason code for posture fail-closed classification",
            },
        ));
    }
    require_canonical_posture_identity_state_shape(identity_state, posture_reason_code)?;
    Ok(Some(identity_state))
}

fn classify_identity_recovery_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Option<IdentityRecoveryFailClosedBehavior> {
    if !matches!(
        out.next_move,
        AppVoiceTurnNextMove::Dispatch | AppVoiceTurnNextMove::Respond
    ) {
        return None;
    }
    let identity_state = out.runtime_execution_envelope.identity_state.as_ref()?;
    match identity_state.recovery_state {
        IdentityRecoveryState::ReauthRequired
            if identity_reason_code(identity_state)
                == Some(voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED) =>
        {
            Some(IdentityRecoveryFailClosedBehavior {
                reason_code: identity_reason_code_or(
                    identity_state,
                    voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED,
                ),
                user_message: "I need you to confirm this device before I can continue.",
                audit_response_kind: "IDENTITY_DEVICE_CLAIM_REQUIRED_FAIL_CLOSED",
            })
        }
        IdentityRecoveryState::ReauthRequired => Some(IdentityRecoveryFailClosedBehavior {
            reason_code: identity_reason_code_or(
                identity_state,
                voice_id_reason_codes::VID_REAUTH_REQUIRED,
            ),
            user_message: "I need you to reauthenticate before I can continue.",
            audit_response_kind: "IDENTITY_REAUTH_REQUIRED_FAIL_CLOSED",
        }),
        IdentityRecoveryState::ReEnrollmentRequired
            if identity_reason_code(identity_state)
                == Some(voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED) =>
        {
            Some(IdentityRecoveryFailClosedBehavior {
                reason_code: identity_reason_code_or(
                    identity_state,
                    voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED,
                ),
                user_message:
                    "I couldn't find an enrolled voice profile for you, so I can't continue.",
                audit_response_kind: "IDENTITY_PROFILE_NOT_ENROLLED_FAIL_CLOSED",
            })
        }
        IdentityRecoveryState::ReEnrollmentRequired => Some(IdentityRecoveryFailClosedBehavior {
            reason_code: identity_reason_code_or(
                identity_state,
                voice_id_reason_codes::VID_ENROLLMENT_REQUIRED,
            ),
            user_message: "I need you to re-enroll your voice before I can continue.",
            audit_response_kind: "IDENTITY_REENROLLMENT_REQUIRED_FAIL_CLOSED",
        }),
        IdentityRecoveryState::RecoveryRestricted
            if identity_reason_code(identity_state)
                == Some(voice_id_reason_codes::VID_SPOOF_RISK) =>
        {
            Some(IdentityRecoveryFailClosedBehavior {
                reason_code: identity_reason_code_or(
                    identity_state,
                    voice_id_reason_codes::VID_SPOOF_RISK,
                ),
                user_message: "I detected a possible spoofing risk, so I can't continue.",
                audit_response_kind: "IDENTITY_SPOOF_RISK_FAIL_CLOSED",
            })
        }
        IdentityRecoveryState::None | IdentityRecoveryState::RecoveryRestricted => None,
    }
}

fn classify_governance_quarantine_identity_recovery_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Option<GovernanceQuarantineFailClosedBehavior> {
    if !matches!(
        out.next_move,
        AppVoiceTurnNextMove::Dispatch | AppVoiceTurnNextMove::Respond
    ) {
        return None;
    }
    let identity_state = out.runtime_execution_envelope.identity_state.as_ref()?;
    let governance_state = out.runtime_execution_envelope.governance_state.as_ref()?;
    if identity_state.consistency_level != IdentityVerificationConsistencyLevel::RecoveryRestricted
        || identity_state.trust_tier != IdentityTrustTier::Restricted
        || !identity_state.step_up_required
        || identity_state.recovery_state != IdentityRecoveryState::RecoveryRestricted
        || identity_reason_code(identity_state).is_some()
        || (!governance_state.safe_mode_active
            && !governance_quarantines_subsystem(
                governance_state,
                GOVERNED_SUBSYSTEM_IDENTITY_VOICE_ENGINE,
            ))
    {
        return None;
    }
    Some(GovernanceQuarantineFailClosedBehavior {
        audit_reason_code: crate::ph1gov::reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR,
        user_message: "I can't continue because identity governance is quarantined right now.",
        audit_response_kind: "IDENTITY_GOVERNANCE_QUARANTINE_FAIL_CLOSED",
    })
}

fn classify_low_confidence_identity_posture_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<IdentityPostureFailClosedBehavior>, StorageError> {
    let Some(identity_state) = canonical_posture_fail_closed_identity_state(out)? else {
        return Ok(None);
    };
    if identity_reason_code(identity_state) != Some(voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE)
    {
        return Ok(None);
    }
    Ok(Some(IdentityPostureFailClosedBehavior {
        reason_code: identity_reason_code_or(
            identity_state,
            voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE,
        ),
        user_message: "I couldn't verify your identity strongly enough, so I can't continue.",
        audit_response_kind: "IDENTITY_LOW_CONFIDENCE_FAIL_CLOSED",
    }))
}

fn classify_gray_zone_margin_identity_posture_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<IdentityPostureFailClosedBehavior>, StorageError> {
    let Some(identity_state) = canonical_posture_fail_closed_identity_state(out)? else {
        return Ok(None);
    };
    if identity_reason_code(identity_state)
        != Some(voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN)
    {
        return Ok(None);
    }
    Ok(Some(IdentityPostureFailClosedBehavior {
        reason_code: identity_reason_code_or(
            identity_state,
            voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN,
        ),
        user_message: "I got an ambiguous identity result, so I can't continue.",
        audit_response_kind: "IDENTITY_GRAY_ZONE_MARGIN_FAIL_CLOSED",
    }))
}

fn classify_echo_unsafe_identity_posture_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<IdentityPostureFailClosedBehavior>, StorageError> {
    let Some(identity_state) = canonical_posture_fail_closed_identity_state(out)? else {
        return Ok(None);
    };
    if identity_reason_code(identity_state) != Some(voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE) {
        return Ok(None);
    }
    Ok(Some(IdentityPostureFailClosedBehavior {
        reason_code: identity_reason_code_or(
            identity_state,
            voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE,
        ),
        user_message: "I detected an echo-unsafe voice condition, so I can't continue.",
        audit_response_kind: "IDENTITY_ECHO_UNSAFE_FAIL_CLOSED",
    }))
}

fn classify_no_speech_identity_posture_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<IdentityPostureFailClosedBehavior>, StorageError> {
    let Some(identity_state) = canonical_posture_fail_closed_identity_state(out)? else {
        return Ok(None);
    };
    if identity_reason_code(identity_state) != Some(voice_id_reason_codes::VID_FAIL_NO_SPEECH) {
        return Ok(None);
    }
    Ok(Some(IdentityPostureFailClosedBehavior {
        reason_code: identity_reason_code_or(
            identity_state,
            voice_id_reason_codes::VID_FAIL_NO_SPEECH,
        ),
        user_message: "I couldn't detect speech clearly enough, so I can't continue.",
        audit_response_kind: "IDENTITY_NO_SPEECH_FAIL_CLOSED",
    }))
}

fn classify_multi_speaker_identity_posture_fail_closed_outcome(
    out: &AppVoiceTurnExecutionOutcome,
) -> Result<Option<IdentityPostureFailClosedBehavior>, StorageError> {
    let Some(identity_state) = canonical_posture_fail_closed_identity_state(out)? else {
        return Ok(None);
    };
    if identity_reason_code(identity_state)
        != Some(voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT)
    {
        return Ok(None);
    }
    Ok(Some(IdentityPostureFailClosedBehavior {
        reason_code: identity_reason_code_or(
            identity_state,
            voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
        ),
        user_message: "I detected multiple speakers, so I can't continue.",
        audit_response_kind: "IDENTITY_MULTI_SPEAKER_FAIL_CLOSED",
    }))
}

// Drift fail-closed decisions must be made before a protected dispatch executes,
// so this classifier relies on the already-attached governed identity and
// governance posture that runtime law would otherwise surface as drift visibility.
fn classify_governance_drift_fail_closed_for_directive(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    directive: &Ph1xDirective,
    reason_code: ReasonCodeId,
) -> Option<GovernanceDriftFailClosedBehavior> {
    if !matches!(
        directive,
        Ph1xDirective::Respond(_) | Ph1xDirective::Dispatch(_)
    ) {
        return None;
    }
    let identity_state = runtime_execution_envelope.identity_state.as_ref()?;
    let governance_state = runtime_execution_envelope.governance_state.as_ref()?;
    if !identity_state.cluster_drift_detected
        || governance_state.cluster_consistency == GovernanceClusterConsistency::Consistent
        || governance_state.safe_mode_active
    {
        return None;
    }
    if governance_state
        .drift_signals
        .contains(&GovernanceDriftSignal::PolicyVersionDrift)
    {
        return Some(GovernanceDriftFailClosedBehavior {
            reason_code,
            user_message: "I can't continue because policy versions are out of sync right now.",
            audit_response_kind: "GOVERNANCE_POLICY_DRIFT_FAIL_CLOSED",
        });
    }
    Some(GovernanceDriftFailClosedBehavior {
        reason_code,
        user_message: "I can't continue because governance state is out of sync right now.",
        audit_response_kind: "GOVERNANCE_CLUSTER_DRIFT_FAIL_CLOSED",
    })
}

fn classify_access_fail_closed_error(err: &StorageError) -> Option<AccessFailClosedBehavior> {
    let StorageError::ContractViolation(ContractViolation::InvalidValue { reason, .. }) = err
    else {
        return None;
    };
    match *reason {
        "ACCESS_SCOPE_VIOLATION" => Some(AccessFailClosedBehavior {
            reason_code: sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED,
            user_message: "I can't proceed because your access policy blocks this action.",
            audit_response_kind: "ACCESS_DENIED_FAIL_CLOSED",
        }),
        "ACCESS_AP_REQUIRED" => Some(AccessFailClosedBehavior {
            reason_code: sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED,
            user_message: "I need approval before I can do that.",
            audit_response_kind: "ACCESS_AP_REQUIRED_FAIL_CLOSED",
        }),
        _ => None,
    }
}

fn access_decision_for_match_outcome(out: &AppVoiceTurnExecutionOutcome) -> String {
    if out.next_move == AppVoiceTurnNextMove::Confirm {
        return "PENDING".to_string();
    }
    match out.reason_code {
        Some(code) if code == sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED => {
            "DENY".to_string()
        }
        Some(code) if code == sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED => {
            "ESCALATE".to_string()
        }
        _ if out.next_move == AppVoiceTurnNextMove::Dispatch => "ALLOW".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

fn confirm_decision_for_match_outcome(
    confirm_required: bool,
    next_move: AppVoiceTurnNextMove,
) -> String {
    if !confirm_required {
        return "NOT_REQUIRED".to_string();
    }
    match next_move {
        AppVoiceTurnNextMove::Confirm => "REQUIRED_PENDING".to_string(),
        AppVoiceTurnNextMove::Dispatch => "REQUIRED_SATISFIED".to_string(),
        AppVoiceTurnNextMove::Refused => "REQUIRED_FAIL_CLOSED".to_string(),
        _ => "REQUIRED_OTHER".to_string(),
    }
}

fn dispatch_outcome_proof_ref_token(outcome: &SimulationDispatchOutcome) -> String {
    fn kind_token(outcome: &SimulationDispatchOutcome) -> &'static str {
        match outcome {
            SimulationDispatchOutcome::BroadcastDeliverySend { .. } => "BCAST_DELIVERY_SEND",
            SimulationDispatchOutcome::BroadcastMhpHandoff { .. } => "BCAST_MHP_HANDOFF",
            SimulationDispatchOutcome::BroadcastMhpReminderFired(_) => "BCAST_MHP_REMINDER_FIRED",
            SimulationDispatchOutcome::BroadcastMhpFollowupDecision { .. } => {
                "BCAST_MHP_FOLLOWUP_DECISION"
            }
            SimulationDispatchOutcome::BroadcastMhpAppThreadReplyConcluded { .. } => {
                "BCAST_MHP_APP_REPLY_CONCLUDED"
            }
            SimulationDispatchOutcome::MemoryPropose(_) => "MEMORY_PROPOSE",
            SimulationDispatchOutcome::MemoryRecall(_) => "MEMORY_RECALL",
            SimulationDispatchOutcome::MemoryForget(_) => "MEMORY_FORGET",
            SimulationDispatchOutcome::Link(_) => "LINK",
            SimulationDispatchOutcome::LinkDelivered { .. } => "LINK_DELIVERED",
            SimulationDispatchOutcome::Reminder(_) => "REMINDER",
            SimulationDispatchOutcome::CalendarDraftCreated { .. } => "CALENDAR_DRAFT_CREATED",
            SimulationDispatchOutcome::Onboarding(_) => "ONBOARDING",
            SimulationDispatchOutcome::Position(_) => "POSITION",
            SimulationDispatchOutcome::VoiceId(_) => "VOICE_ID",
            SimulationDispatchOutcome::Wake(_) => "WAKE",
            SimulationDispatchOutcome::AccessGatePassed { .. } => "ACCESS_GATE_PASSED",
            SimulationDispatchOutcome::AccessStepUp { .. } => "ACCESS_STEP_UP",
            SimulationDispatchOutcome::CapreqLifecycle { .. } => "CAPREQ_LIFECYCLE",
            SimulationDispatchOutcome::BcastWaitPolicyUpdated { .. } => "BCAST_WAIT_POLICY_UPDATED",
            SimulationDispatchOutcome::BcastUrgentFollowupPolicyUpdated { .. } => {
                "BCAST_URGENT_FOLLOWUP_POLICY_UPDATED"
            }
        }
    }
    format!(
        "dispatch_proof:{}:{}",
        kind_token(outcome),
        short_hash_hex(&[&format!("{outcome:?}")])
    )
}

fn response_text_for_dispatch_outcome(outcome: &SimulationDispatchOutcome) -> String {
    match outcome {
        SimulationDispatchOutcome::LinkDelivered { .. } => "I sent the link.".to_string(),
        SimulationDispatchOutcome::Link(_) => "I generated the link.".to_string(),
        SimulationDispatchOutcome::Reminder(rem) => match rem {
            selene_kernel_contracts::ph1rem::Ph1RemResponse::Ok(ok) => {
                match ok.simulation_id.as_str() {
                    "REMINDER_UPDATE_COMMIT" => "I updated that reminder.".to_string(),
                    "REMINDER_CANCEL_COMMIT" => "I canceled that reminder.".to_string(),
                    _ => "I scheduled that reminder.".to_string(),
                }
            }
            selene_kernel_contracts::ph1rem::Ph1RemResponse::Refuse(_) => {
                "I couldn't complete that reminder request.".to_string()
            }
        },
        SimulationDispatchOutcome::CalendarDraftCreated { .. } => {
            "Draft created; not sent to external calendar yet.".to_string()
        }
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

fn latest_persona_style_hint_for_actor(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    tenant_id: Option<&str>,
) -> Option<String> {
    let style_profile_ref_key = PayloadKey::new("style_profile_ref").ok()?;
    store.audit_events().iter().rev().find_map(|event| {
        if !matches!(&event.engine, AuditEngine::Other(name) if name == "PH1.PERSONA") {
            return None;
        }
        if event.user_id.as_ref() != Some(actor_user_id) {
            return None;
        }
        if let Some(tenant_id) = tenant_id {
            if event.tenant_id.as_deref() != Some(tenant_id) {
                return None;
            }
        }
        event
            .payload_min
            .entries
            .get(&style_profile_ref_key)
            .map(|value| value.as_str().to_string())
    })
}

fn apply_persona_style_hint_to_response_text(
    response_text: String,
    style_hint: Option<&str>,
) -> String {
    match style_hint {
        Some("gentle") => format!("Certainly. {response_text}"),
        Some("dominant") => format!("Directly: {response_text}"),
        _ => response_text,
    }
}

fn onboarding_missing_field_question(field_key: &str) -> String {
    match field_key {
        "tenant_id" => "Which tenant should I use for this onboarding?".to_string(),
        "company_id" => "What company should I use for this onboarding?".to_string(),
        "position_id" => "What position should I set for this onboarding?".to_string(),
        "location_id" => "What location should I set for this onboarding?".to_string(),
        "start_date" => "What is the start date?".to_string(),
        "working_hours" => "What working hours should I set?".to_string(),
        "compensation_tier_ref" => "What compensation tier should I use?".to_string(),
        "jurisdiction_tags" => "Which jurisdiction tags should I set?".to_string(),
        _ => format!("What value should I use for {field_key}?"),
    }
}

fn onboarding_sender_verification_pending(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
) -> Result<bool, StorageError> {
    const GATE_SENDER_CONFIRMATION: &str = "SENDER_CONFIRMATION";
    let session = store.ph1onb_session_row(onboarding_session_id).ok_or(
        StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        },
    )?;
    let required = session
        .required_verification_gates
        .iter()
        .any(|gate| gate == GATE_SENDER_CONFIRMATION);
    Ok(required && session.verification_status != Some(VerificationStatus::Confirmed))
}

fn onboarding_sender_user_id_for_session(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
) -> Result<UserId, StorageError> {
    let session = store.ph1onb_session_row(onboarding_session_id).ok_or(
        StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        },
    )?;
    let link =
        store
            .ph1link_get_link(&session.token_id)
            .ok_or(StorageError::ForeignKeyViolation {
                table: "links.token_id",
                key: session.token_id.as_str().to_string(),
            })?;
    Ok(link.inviter_user_id.clone())
}

fn wake_enrollment_required_for_platform(app_platform: AppPlatform) -> bool {
    matches!(
        app_platform,
        AppPlatform::Android | AppPlatform::Tablet | AppPlatform::Desktop
    )
}

fn wake_enrollment_completed_for_session(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
) -> bool {
    store
        .ph1onb_latest_complete_wake_receipt_ref(onboarding_session_id)
        .is_some()
}

fn onboarding_next_step_after_voice_enroll(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
    app_platform: AppPlatform,
) -> AppOnboardingContinueNextStep {
    if wake_enrollment_required_for_platform(app_platform)
        && !wake_enrollment_completed_for_session(store, onboarding_session_id)
    {
        AppOnboardingContinueNextStep::WakeEnroll
    } else {
        AppOnboardingContinueNextStep::EmoPersonaLock
    }
}

fn onboarding_next_step_after_wake_status(
    wake_status: ContractWakeEnrollStatus,
) -> AppOnboardingContinueNextStep {
    if wake_status == ContractWakeEnrollStatus::Complete {
        AppOnboardingContinueNextStep::EmoPersonaLock
    } else {
        AppOnboardingContinueNextStep::WakeEnroll
    }
}

fn ensure_wake_enrollment_completed_for_platform(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
    missing_reason: &'static str,
) -> Result<(), StorageError> {
    let session = store.ph1onb_session_row(onboarding_session_id).ok_or(
        StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        },
    )?;
    if wake_enrollment_required_for_platform(session.app_platform)
        && !wake_enrollment_completed_for_session(store, onboarding_session_id)
    {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: missing_reason,
            },
        ));
    }
    Ok(())
}

fn onboarding_user_for_device(
    store: &Ph1fStore,
    device_id: &DeviceId,
) -> Result<UserId, StorageError> {
    store
        .get_device(device_id)
        .map(|device| device.user_id.clone())
        .ok_or_else(|| StorageError::ForeignKeyViolation {
            table: "devices.device_id",
            key: device_id.as_str().to_string(),
        })
}

fn ensure_wake_enrollment_action_allowed(
    store: &Ph1fStore,
    onboarding_session_id: &OnboardingSessionId,
    device_id: &DeviceId,
) -> Result<AppPlatform, StorageError> {
    let session = store.ph1onb_session_row(onboarding_session_id).ok_or(
        StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        },
    )?;
    if session.app_platform == AppPlatform::Ios {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ios_wake_disabled",
            },
        ));
    }
    if session.status == OnboardingStatus::Complete {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_SESSION_NOT_ACTIVE_FOR_WAKE_ENROLL",
            },
        ));
    }
    if !session.missing_fields.is_empty() {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_ASK_MISSING_REQUIRED_BEFORE_WAKE_ENROLL",
            },
        ));
    }
    let remaining_platform_receipt_kinds =
        store.ph1onb_remaining_platform_receipt_kinds(onboarding_session_id)?;
    if !remaining_platform_receipt_kinds.is_empty() {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_PLATFORM_SETUP_REQUIRED_BEFORE_WAKE_ENROLL",
            },
        ));
    }
    if session.terms_status != Some(TermsStatus::Accepted) {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_TERMS_REQUIRED_BEFORE_WAKE_ENROLL",
            },
        ));
    }
    if !session.primary_device_confirmed {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_WAKE_ENROLL",
            },
        ));
    }
    let expected_device_id =
        session
            .primary_device_device_id
            .as_ref()
            .ok_or(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_WAKE_ENROLL",
                },
            ))?;
    if expected_device_id != device_id {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action.wake_enroll.device_id",
                reason: "ONB_PRIMARY_DEVICE_DEVICE_MISMATCH_FOR_WAKE_ENROLL",
            },
        ));
    }
    if onboarding_sender_verification_pending(store, onboarding_session_id)? {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "app_onboarding_continue_request.action",
                reason: "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_WAKE_ENROLL",
            },
        ));
    }
    Ok(session.app_platform)
}

fn onboarding_default_role_id(invitee_type: InviteeType) -> &'static str {
    match invitee_type {
        InviteeType::Company => "company_default",
        InviteeType::Customer => "customer_default",
        InviteeType::Employee => "employee_default",
        InviteeType::FamilyMember => "family_member_default",
        InviteeType::Friend => "friend_default",
        InviteeType::Associate => "associate_default",
    }
}

fn ensure_onboarding_persona_subject(
    store: &mut Ph1fStore,
    tenant_id: &TenantId,
    onboarding_session_id: &OnboardingSessionId,
    device_id: &DeviceId,
    now: MonotonicTimeNs,
) -> Result<(UserId, String), StorageError> {
    if let Some(existing_device) = store.get_device(device_id).cloned() {
        let user_id = existing_device.user_id;
        if store.get_identity(&user_id).is_none() {
            store.insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                now,
                IdentityStatus::Active,
            ))?;
        }
        let speaker_id = store
            .get_identity(&user_id)
            .and_then(|record| {
                record
                    .speaker_id
                    .as_ref()
                    .map(|speaker| speaker.as_str().to_string())
            })
            .unwrap_or_else(|| {
                format!(
                    "spk_onb_{}",
                    short_hash_hex(&[onboarding_session_id.as_str(), user_id.as_str()])
                )
            });
        return Ok((user_id, speaker_id));
    }

    let user_id = UserId::new(format!(
        "{}:onb:{}",
        tenant_id.as_str(),
        short_hash_hex(&[onboarding_session_id.as_str(), device_id.as_str()])
    ))
    .map_err(StorageError::ContractViolation)?;
    if store.get_identity(&user_id).is_none() {
        store.insert_identity(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            now,
            IdentityStatus::Active,
        ))?;
    }
    if store.get_device(device_id).is_none() {
        let record = DeviceRecord::v1(
            device_id.clone(),
            user_id.clone(),
            "onboarding_device".to_string(),
            now,
            None,
        )
        .map_err(StorageError::ContractViolation)?;
        store.insert_device(record)?;
    }
    let speaker_id = format!(
        "spk_onb_{}",
        short_hash_hex(&[onboarding_session_id.as_str(), user_id.as_str()])
    );
    Ok((user_id, speaker_id))
}

fn emo_signal_bundle_for_onboarding_session(
    onboarding_session_id: &OnboardingSessionId,
) -> Result<EmoSignalBundle, ContractViolation> {
    let variant = stable_seed_u64(&[onboarding_session_id.as_str()]) % 3;
    match variant {
        0 => EmoSignalBundle::v1(74, 22, 30, 40),
        1 => EmoSignalBundle::v1(38, 18, 10, 80),
        _ => EmoSignalBundle::v1(54, 20, 18, 54),
    }
}

fn emoguide_signals_for_personality(
    personality_type: EmoPersonalityType,
) -> Result<EmoGuideInteractionSignals, ContractViolation> {
    match personality_type {
        EmoPersonalityType::Domineering => EmoGuideInteractionSignals::v1(24, 3, 4, 16, 6),
        EmoPersonalityType::Passive => EmoGuideInteractionSignals::v1(24, 2, 1, 5, 16),
        EmoPersonalityType::Undetermined => EmoGuideInteractionSignals::v1(24, 2, 2, 11, 11),
    }
}

fn persona_tone_value_for_personality(personality_type: EmoPersonalityType) -> &'static str {
    match personality_type {
        EmoPersonalityType::Domineering => "dominant",
        EmoPersonalityType::Passive => "gentle",
        EmoPersonalityType::Undetermined => "balanced",
    }
}

fn style_profile_ref_token(style_profile_ref: StyleProfileRef) -> &'static str {
    match style_profile_ref {
        StyleProfileRef::Dominant => "dominant",
        StyleProfileRef::Gentle => "gentle",
    }
}

fn persona_delivery_policy_token(delivery_policy_ref: PersonaDeliveryPolicyRef) -> &'static str {
    match delivery_policy_ref {
        PersonaDeliveryPolicyRef::VoiceAllowed => "voice",
        PersonaDeliveryPolicyRef::TextOnly => "text",
        PersonaDeliveryPolicyRef::Silent => "silent",
    }
}

fn validate_ascii_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn os_voice_platform_from_app_platform(app_platform: AppPlatform) -> OsVoicePlatform {
    match app_platform {
        AppPlatform::Ios => OsVoicePlatform::Ios,
        AppPlatform::Android => OsVoicePlatform::Android,
        AppPlatform::Tablet => OsVoicePlatform::Tablet,
        AppPlatform::Desktop => OsVoicePlatform::Desktop,
    }
}

fn expected_always_on_voice_sequence(trigger: OsVoiceTrigger) -> Vec<String> {
    let mut seq = vec!["PH1.K".to_string()];
    if trigger.wake_stage_required() {
        seq.push("PH1.W".to_string());
        seq.push("PH1.L".to_string());
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

fn maybe_build_message_policy_tool_response(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    tool_request: &ToolRequest,
) -> Result<Option<ToolResponse>, StorageError> {
    if !matches!(tool_request.tool_name, ToolName::ConnectorQuery) {
        return Ok(None);
    }
    if !is_message_policy_query(tool_request.query.as_str()) {
        return Ok(None);
    }

    let tenant_id = resolve_actor_single_tenant(store, actor_user_id)?;
    let changes = store.bcast_policy_changes_for_tenant(&tenant_id, 10);
    let non_urgent_wait_seconds = store.bcast_non_urgent_wait_seconds_for_tenant(&tenant_id);
    let urgent_followup_immediate = store.bcast_urgent_followup_immediate_for_tenant(&tenant_id);
    let max_followup_attempts = store.bcast_max_followup_attempts_for_tenant(&tenant_id);
    let reminder_default_time = store
        .bcast_reminder_default_time_for_tenant(&tenant_id)
        .unwrap_or("unset")
        .to_string();
    let current = store.bcast_policy_current_row(&tenant_id);

    let non_urgent_updated_ns =
        latest_setting_update_ns(&changes, BcastPolicySettingKey::NonUrgentWaitSeconds);
    let urgent_updated_ns =
        latest_setting_update_ns(&changes, BcastPolicySettingKey::UrgentFollowupMode);
    let max_followup_updated_ns =
        latest_setting_update_ns(&changes, BcastPolicySettingKey::MaxFollowupAttempts);
    let reminder_default_updated_ns =
        latest_setting_update_ns(&changes, BcastPolicySettingKey::ReminderDefaultTime);

    let urgent_mode = if urgent_followup_immediate {
        "immediate"
    } else {
        "wait"
    };
    let current_provenance = format!(
        "ph1f.bcast_policy_current tenant={} path=/ph1f/bcast-policy/{}/current",
        tenant_id.as_str(),
        tenant_id.as_str(),
    );
    let first_change_summary = changes.first().map(|row| {
        format!(
            "setting={} {}->{} by={} at_ns={} reason_code={} idempotency_key={}",
            row.setting_key.as_str(),
            row.old_value,
            row.new_value,
            row.updated_by_user_id.as_str(),
            row.created_at.0,
            row.reason_code.0,
            row.idempotency_key,
        )
    });

    let mut extracted_fields = vec![
        ToolStructuredField {
            key: "non_urgent_wait_seconds".to_string(),
            value: non_urgent_wait_seconds.to_string(),
        },
        ToolStructuredField {
            key: "non_urgent_wait_updated_at_ns".to_string(),
            value: non_urgent_updated_ns,
        },
        ToolStructuredField {
            key: "urgent_followup_mode".to_string(),
            value: urgent_mode.to_string(),
        },
        ToolStructuredField {
            key: "urgent_followup_updated_at_ns".to_string(),
            value: urgent_updated_ns,
        },
        ToolStructuredField {
            key: "max_followup_attempts".to_string(),
            value: max_followup_attempts.to_string(),
        },
        ToolStructuredField {
            key: "max_followup_attempts_updated_at_ns".to_string(),
            value: max_followup_updated_ns,
        },
        ToolStructuredField {
            key: "reminder_default_time".to_string(),
            value: reminder_default_time,
        },
        ToolStructuredField {
            key: "reminder_default_time_updated_at_ns".to_string(),
            value: reminder_default_updated_ns,
        },
        ToolStructuredField {
            key: "policy_current_provenance".to_string(),
            value: current_provenance,
        },
        ToolStructuredField {
            key: "policy_change_1".to_string(),
            value: first_change_summary
                .unwrap_or_else(|| "no policy change rows recorded".to_string()),
        },
    ];

    if let Some(current_row) = current {
        extracted_fields.push(ToolStructuredField {
            key: "policy_updated_by_user_id".to_string(),
            value: current_row.updated_by_user_id.as_str().to_string(),
        });
        extracted_fields.push(ToolStructuredField {
            key: "policy_version".to_string(),
            value: current_row.policy_version.to_string(),
        });
    }
    extracted_fields.push(ToolStructuredField {
        key: "policy_change_count".to_string(),
        value: changes.len().to_string(),
    });

    let citations = if changes.is_empty() {
        vec![ToolTextSnippet {
            title: "No policy changes recorded".to_string(),
            snippet: format!(
                "Tenant {} is using default BCAST policy values.",
                tenant_id.as_str()
            ),
            url: format!(
                "https://selene.local/ph1f/bcast-policy/{}/ledger",
                tenant_id.as_str()
            ),
        }]
    } else {
        changes
            .iter()
            .take(5)
            .map(|row| ToolTextSnippet {
                title: format!("Policy change {}", row.setting_key.as_str()),
                snippet: format!(
                    "{} -> {} | by={} | at_ns={} | reason_code={} | idempotency_key={}",
                    row.old_value,
                    row.new_value,
                    row.updated_by_user_id.as_str(),
                    row.created_at.0,
                    row.reason_code.0,
                    row.idempotency_key,
                ),
                url: format!(
                    "https://selene.local/ph1f/bcast-policy/{}/event/{}",
                    row.tenant_id.as_str(),
                    row.bcast_policy_event_id
                ),
            })
            .collect()
    };

    let source_metadata = SourceMetadata {
        schema_version: selene_kernel_contracts::ph1e::PH1E_CONTRACT_VERSION,
        provider_hint: Some("ph1f_bcast_policy_registry".to_string()),
        retrieved_at_unix_ms: 1_700_000_000_000,
        sources: vec![
            SourceRef {
                title: "PH1.F current policy projection".to_string(),
                url: format!(
                    "https://selene.local/ph1f/bcast-policy/{}/current",
                    tenant_id.as_str()
                ),
            },
            SourceRef {
                title: "PH1.F policy change ledger".to_string(),
                url: format!(
                    "https://selene.local/ph1f/bcast-policy/{}/ledger",
                    tenant_id.as_str()
                ),
            },
        ],
    };
    let tool_result = ToolResult::ConnectorQuery {
        summary: format!(
            "Message policy settings for tenant {} with {} recent change rows.",
            tenant_id.as_str(),
            changes.len()
        ),
        extracted_fields,
        citations,
    };
    let response = ToolResponse::ok_v1(
        tool_request.request_id,
        tool_request.query_hash,
        tool_result,
        source_metadata,
        None,
        ReasonCodeId(0x4500_0001),
        CacheStatus::Bypassed,
    )
    .map_err(StorageError::ContractViolation)?;
    Ok(Some(response))
}

fn latest_setting_update_ns(
    changes: &[BcastPolicyLedgerRow],
    setting_key: BcastPolicySettingKey,
) -> String {
    changes
        .iter()
        .find(|row| row.setting_key == setting_key)
        .map(|row| row.created_at.0.to_string())
        .unwrap_or_else(|| "default".to_string())
}

fn is_message_policy_query(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    lower.contains("message policy")
        || lower.contains("policy settings")
        || lower.contains("list my policies")
        || lower.contains("policy change")
        || lower.contains("policy audit")
}

fn resolve_actor_single_tenant(
    store: &Ph1fStore,
    actor_user_id: &UserId,
) -> Result<TenantId, StorageError> {
    let mut inferred: Option<String> = None;
    for (tenant_id, user_id) in store.ph2access_instance_rows().keys() {
        if user_id != actor_user_id {
            continue;
        }
        if let Some(existing) = &inferred {
            if existing != tenant_id {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "app_voice_turn_execution_outcome.tool_request.query",
                        reason: "message policy query requires a single tenant scope",
                    },
                ));
            }
        } else {
            inferred = Some(tenant_id.clone());
        }
    }
    let tenant = inferred.ok_or({
        StorageError::ContractViolation(ContractViolation::InvalidValue {
            field: "app_voice_turn_execution_outcome.tool_request.query",
            reason: "message policy query requires actor tenant access instance",
        })
    })?;
    TenantId::new(tenant).map_err(StorageError::ContractViolation)
}

fn normalized_tenant_scope_for_dev_intake(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    actor_tenant_id: Option<&str>,
) -> Result<String, StorageError> {
    if let Some(tenant_id) = actor_tenant_id {
        return Ok(TenantId::new(tenant_id.to_string())
            .map_err(StorageError::ContractViolation)?
            .as_str()
            .to_string());
    }
    Ok(resolve_actor_single_tenant(store, actor_user_id)?
        .as_str()
        .to_string())
}

fn build_finder_run_request_for_simulation_intent(
    store: &Ph1fStore,
    actor_user_id: &UserId,
    actor_tenant_id: Option<&str>,
    packet: &AgentInputPacket,
) -> Result<Option<FinderRunRequest>, StorageError> {
    let Some(Ph1nResponse::IntentDraft(intent_draft)) = packet.nlp_output.as_ref() else {
        return Ok(None);
    };
    if !intent_type_requires_simulation_finder(intent_draft.intent_type) {
        return Ok(None);
    }

    let tenant_id = normalized_tenant_scope_for_dev_intake(store, actor_user_id, actor_tenant_id)?;
    let simulation_catalog_snapshot_version = packet.sim_catalog_snapshot_version.max(1);
    let transcript_text = packet
        .srl_repaired_transcript
        .clone()
        .or(packet.transcript_text.clone())
        .unwrap_or_else(|| intent_draft_transcript_fallback(intent_draft));
    let intent_family = finder_intent_family(intent_draft.intent_type);
    let simulation_catalog = match simulation_id_for_intent_draft_v1(intent_draft) {
        Ok(simulation_id) => match simulation_status_for_tenant(store, &tenant_id, simulation_id) {
            Some(status) => {
                let entry = FinderSimulationCatalogEntry::v1(
                    simulation_id.to_string(),
                    intent_family.clone(),
                    status,
                    100,
                    finder_access_actions_for_intent(intent_draft.intent_type),
                    finder_risk_tier_for_intent(intent_draft.intent_type),
                    finder_confirm_required_for_intent(intent_draft.intent_type),
                    FinderFallbackPolicy::Refuse,
                    finder_synonym_terms_from_transcript(&intent_family, &transcript_text),
                    finder_required_field_specs_from_intent(intent_draft)?,
                    vec!["runtime.default".to_string()],
                )
                .map_err(StorageError::ContractViolation)?;
                vec![entry]
            }
            None => Vec::new(),
        },
        Err(_) => Vec::new(),
    };

    let run_request = FinderRunRequest::v1(
        tenant_id,
        actor_user_id.as_str().to_string(),
        packet.correlation_id,
        packet.turn_id,
        packet
            .thread_key
            .clone()
            .unwrap_or_else(|| format!("corr:{}:thread", packet.correlation_id)),
        packet.now,
        transcript_text,
        packet.sim_catalog_snapshot_hash.clone(),
        simulation_catalog_snapshot_version,
        simulation_catalog,
        Vec::<FinderGoldMapping>::new(),
        10_000,
        7_500,
        7_500,
        0,
        0,
        0,
        1,
        "runtime".to_string(),
        5_000,
        5_000,
        5_000,
        5_000,
        2_500,
        "tenant_only".to_string(),
        "{\"required\":[]}".to_string(),
        vec!["AT-AGENT-INGRESS-FINDER".to_string()],
    )
    .map_err(StorageError::ContractViolation)?;
    Ok(Some(run_request))
}

fn finder_required_field_specs_from_intent(
    intent_draft: &IntentDraft,
) -> Result<Vec<FinderFieldSpec>, StorageError> {
    let mut specs = Vec::new();
    for missing_field in &intent_draft.required_fields_missing {
        let field_name = finder_field_name(*missing_field);
        let detector_terms = finder_detector_terms_for_field(intent_draft, *missing_field);
        let spec = FinderFieldSpec::required_v1(
            field_name.clone(),
            detector_terms,
            7_500,
            7_500,
            6_500,
            finder_clarify_question(*missing_field),
            finder_allowed_answer_formats(*missing_field),
        )
        .map_err(StorageError::ContractViolation)?;
        specs.push(spec);
    }
    Ok(specs)
}

fn finder_detector_terms_for_field(intent_draft: &IntentDraft, key: FieldKey) -> Vec<String> {
    let mut terms = intent_draft
        .fields
        .iter()
        .filter(|field| field.key == key)
        .filter_map(|field| {
            field
                .value
                .normalized_value
                .clone()
                .or(Some(field.value.original_span.clone()))
        })
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if terms.is_empty() {
        terms.push(finder_field_name(key));
    }
    terms.sort();
    terms.dedup();
    terms
}

fn finder_field_name(key: FieldKey) -> String {
    format!("{:?}", key).to_ascii_lowercase()
}

fn finder_clarify_question(key: FieldKey) -> String {
    match key {
        FieldKey::RecipientContact => "What contact should I use?".to_string(),
        FieldKey::Recipient => "Who is the recipient?".to_string(),
        FieldKey::Task => "What should I do?".to_string(),
        FieldKey::When => "What exact date and time should I use?".to_string(),
        _ => format!("What is the {}?", finder_field_name(key)),
    }
}

fn finder_allowed_answer_formats(key: FieldKey) -> Vec<String> {
    match key {
        FieldKey::RecipientContact => {
            vec![
                "+14155550100".to_string(),
                "tom@example.invalid".to_string(),
            ]
        }
        FieldKey::When => vec![
            "Tomorrow at 7 PM".to_string(),
            "2026-03-02T19:00".to_string(),
        ],
        _ => vec!["Short text".to_string(), "Typed value".to_string()],
    }
}

fn intent_draft_transcript_fallback(intent_draft: &IntentDraft) -> String {
    let mut parts = vec![finder_intent_family(intent_draft.intent_type)];
    parts.extend(intent_draft.fields.iter().filter_map(|field| {
        field
            .value
            .normalized_value
            .clone()
            .or(Some(field.value.original_span.clone()))
    }));
    parts.extend(
        intent_draft
            .evidence_spans
            .iter()
            .map(|span| span.verbatim_excerpt.clone()),
    );
    parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn finder_synonym_terms_from_transcript(intent_family: &str, transcript: &str) -> Vec<String> {
    let mut terms = transcript
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| term.len() >= 3)
        .take(16)
        .collect::<Vec<_>>();
    terms.push(intent_family.to_ascii_lowercase());
    terms.sort();
    terms.dedup();
    if terms.is_empty() {
        vec![intent_family.to_ascii_lowercase()]
    } else {
        terms
    }
}

fn simulation_status_for_tenant(
    store: &Ph1fStore,
    tenant_id: &str,
    simulation_id: &str,
) -> Option<SimulationStatus> {
    let tenant_id = TenantId::new(tenant_id.to_string()).ok()?;
    let simulation_id =
        selene_kernel_contracts::ph1simcat::SimulationId::new(simulation_id.to_string()).ok()?;
    store
        .simulation_catalog_current()
        .get(&(tenant_id, simulation_id))
        .map(|row| row.status)
}

fn finder_intent_family(intent_type: IntentType) -> String {
    format!("{intent_type:?}").to_ascii_lowercase()
}

fn finder_confirm_required_for_intent(intent_type: IntentType) -> bool {
    !matches!(
        intent_type,
        IntentType::MemoryRememberRequest | IntentType::MemoryQuery
    )
}

fn finder_risk_tier_for_intent(intent_type: IntentType) -> FinderRiskTier {
    if matches!(
        intent_type,
        IntentType::CreateInviteLink | IntentType::SendMoney
    ) {
        FinderRiskTier::High
    } else {
        FinderRiskTier::Medium
    }
}

fn finder_access_actions_for_intent(intent_type: IntentType) -> Vec<String> {
    let action = match intent_type {
        IntentType::CreateInviteLink => "LINK_SEND_INVITE",
        IntentType::SetReminder
        | IntentType::UpdateReminder
        | IntentType::CancelReminder
        | IntentType::CreateCalendarEvent => "REMINDER_SCHEDULE_ACCESS",
        IntentType::UpdateBcastWaitPolicy => "BCAST_WAIT_POLICY_UPDATE",
        IntentType::UpdateBcastUrgentFollowupPolicy => "BCAST_URGENT_FOLLOWUP_POLICY_UPDATE",
        IntentType::CapreqManage => "CAPREQ_MANAGE",
        IntentType::AccessSchemaManage => "ACCESS_SCHEMA_MANAGE",
        IntentType::AccessEscalationVote => "ACCESS_ESCALATION_VOTE",
        IntentType::AccessInstanceCompileRefresh => "ACCESS_INSTANCE_COMPILE",
        _ => "SIMULATION_EXECUTE",
    };
    vec![action.to_string()]
}

fn intent_type_requires_simulation_finder(intent_type: IntentType) -> bool {
    !matches!(
        intent_type,
        IntentType::TimeQuery
            | IntentType::WeatherQuery
            | IntentType::WebSearchQuery
            | IntentType::NewsQuery
            | IntentType::UrlFetchAndCiteQuery
            | IntentType::DocumentUnderstandQuery
            | IntentType::PhotoUnderstandQuery
            | IntentType::DataAnalysisQuery
            | IntentType::DeepResearchQuery
            | IntentType::RecordModeQuery
            | IntentType::ConnectorQuery
            | IntentType::ListReminders
            | IntentType::Continue
            | IntentType::MoreDetail
    )
}

fn build_ph1x_request_from_agent_input_packet(
    app_platform: AppPlatform,
    packet: &AgentInputPacket,
) -> Result<Ph1xRequest, StorageError> {
    let effective_policy_context_ref =
        merge_thread_policy_context(packet.policy_context_ref, &packet.thread_state);
    let voice_identity_assertion = canonical_agent_packet_voice_identity_assertion(packet)?.clone();
    let mut req = Ph1xRequest::v1(
        packet.correlation_id,
        packet.turn_id,
        packet.now,
        packet.thread_state.clone(),
        packet.session_state,
        IdentityContext::Voice(voice_identity_assertion),
        effective_policy_context_ref,
        packet.memory_candidates.clone(),
        packet.confirm_answer,
        packet.nlp_output.clone(),
        packet.tool_response.clone(),
        packet.interruption.clone(),
        packet.language_hint.clone(),
        packet.last_failure_reason_code,
    )
    .map_err(StorageError::ContractViolation)?;
    let step_up_capabilities = match app_platform {
        AppPlatform::Ios | AppPlatform::Android | AppPlatform::Tablet => {
            StepUpCapabilities::v1(true, true)
        }
        AppPlatform::Desktop => StepUpCapabilities::v1(false, true),
    };
    req = req
        .with_step_up_capabilities(Some(step_up_capabilities))
        .map_err(StorageError::ContractViolation)?;
    req = req
        .with_identity_prompt_scope_key(packet.identity_prompt_scope_key.clone())
        .map_err(StorageError::ContractViolation)?;
    Ok(req)
}

fn canonical_agent_packet_voice_identity_assertion(
    packet: &AgentInputPacket,
) -> Result<&Ph1VoiceIdResponse, StorageError> {
    let canonical_voice_identity_assertion = packet
        .runtime_execution_envelope
        .as_ref()
        .and_then(|runtime_execution_envelope| {
            runtime_execution_envelope.voice_identity_assertion.as_ref()
        })
        .ok_or(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "agent_input_packet.runtime_execution_envelope.voice_identity_assertion",
                reason: "must carry canonical embedded voice identity assertion for ph1x request",
            },
        ))?;
    if packet.voice_identity_assertion != *canonical_voice_identity_assertion {
        return Err(StorageError::ContractViolation(
            ContractViolation::InvalidValue {
                field: "agent_input_packet.voice_identity_assertion",
                reason: "must match canonical embedded voice identity assertion for ph1x request",
            },
        ));
    }
    Ok(canonical_voice_identity_assertion)
}

fn transcript_text_from_nlp_output(nlp_output: Option<&Ph1nResponse>) -> Option<String> {
    match nlp_output {
        Some(Ph1nResponse::Chat(chat)) => Some(chat.response_text.clone()),
        Some(Ph1nResponse::IntentDraft(draft)) => Some(intent_draft_transcript_fallback(draft)),
        _ => None,
    }
}

fn simulation_catalog_snapshot_for_agent_input(
    store: &Ph1fStore,
    tenant_id: Option<&str>,
) -> (String, u64) {
    let mut rows: Vec<String> = store
        .simulation_catalog_current()
        .iter()
        .filter(|((tenant, _), _)| {
            tenant_id
                .map(|value| tenant.as_str() == value)
                .unwrap_or(true)
        })
        .map(|((tenant, simulation_id), row)| {
            format!(
                "{}|{}|{}|{:?}|{:?}|{}",
                tenant.as_str(),
                simulation_id.as_str(),
                row.simulation_version.0,
                row.simulation_type,
                row.status,
                row.source_event_id
            )
        })
        .collect();
    rows.sort();
    let snapshot_text = if rows.is_empty() {
        "simcat:empty".to_string()
    } else {
        rows.join("\n")
    };
    let hash = agent_hash_hex(&snapshot_text);
    let version = store
        .simulation_catalog_events()
        .iter()
        .filter(|row| {
            tenant_id
                .map(|value| row.tenant_id.as_str() == value)
                .unwrap_or(true)
        })
        .map(|row| row.simulation_catalog_event_id)
        .max()
        .unwrap_or(0);
    (hash, version)
}

#[allow(clippy::too_many_arguments)]
fn agent_input_packet_hash_hex(
    correlation_id: CorrelationId,
    turn_id: TurnId,
    now: MonotonicTimeNs,
    trace_id: &str,
    transcript_text: Option<&str>,
    language_hint: Option<&str>,
    session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
    session_state: SessionState,
    thread_key: Option<&str>,
    thread_state: &ThreadState,
    memory_candidates: &[MemoryCandidate],
    sim_catalog_snapshot_version: u64,
    sim_catalog_snapshot_hash: &str,
    identity_prompt_scope_key: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("corr={}", correlation_id.0),
        format!("turn={}", turn_id.0),
        format!("now={}", now.0),
        format!("trace={trace_id}"),
        format!("session={}", session_id.map(|v| v.0).unwrap_or(0)),
        format!("session_state={session_state:?}"),
        format!("thread_key={}", thread_key.unwrap_or("none")),
        format!("thread_pending={}", thread_state.pending.is_some()),
        format!(
            "thread_project={}",
            thread_state.project_id.as_deref().unwrap_or("none")
        ),
        format!(
            "identity_prompt_scope_key={}",
            identity_prompt_scope_key.unwrap_or("none")
        ),
        format!(
            "sim_catalog={}#{}",
            sim_catalog_snapshot_hash, sim_catalog_snapshot_version
        ),
        format!(
            "transcript={}",
            transcript_text
                .map(truncate_for_packet_hash)
                .unwrap_or_default()
        ),
        format!("lang={}", language_hint.unwrap_or("none")),
    ];
    lines.extend(memory_candidates.iter().map(|candidate| {
        format!(
            "mem:{}:{}:{}",
            candidate.memory_key.as_str(),
            candidate.memory_value.verbatim,
            candidate.last_seen_at.0
        )
    }));
    lines.sort();
    agent_hash_hex(&lines.join("\n"))
}

fn truncate_for_packet_hash(text: &str) -> String {
    const MAX: usize = 128;
    if text.len() <= MAX {
        return text.to_string();
    }
    text[..MAX].to_string()
}

fn agent_hash_hex(value: &str) -> String {
    // FNV-1a 64-bit: deterministic and platform-independent.
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
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

fn merge_thread_policy_context(
    base: PolicyContextRef,
    thread_state: &ThreadState,
) -> PolicyContextRef {
    let mut privacy_mode = base.privacy_mode;
    let mut do_not_disturb = base.do_not_disturb;
    let mut safety_tier = base.safety_tier;
    if let Some(flags) = thread_state.thread_policy_flags {
        privacy_mode |= flags.force_privacy_mode;
        do_not_disturb |= flags.force_do_not_disturb;
        if flags.force_strict_safety {
            safety_tier = SafetyTier::Strict;
        }
    }
    PolicyContextRef::v1(privacy_mode, do_not_disturb, safety_tier)
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
    use crate::runtime_governance::{
        attach_identity_state_for_governed_voice_turn, RuntimeGovernanceConfig,
        RuntimeGovernanceRuntime,
    };
    use selene_engines::ph1_voice_id::reason_codes as voice_id_reason_codes;
    use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
    use selene_engines::ph1e::{Ph1eProviderConfig, Ph1eProxyConfig, Ph1eProxyMode};
    use selene_kernel_contracts::ph1_voice_id::{
        DeviceTrustLevel, DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse,
        SpeakerAssertionOk, SpeakerAssertionUnknown, SpeakerLabel,
    };
    use selene_kernel_contracts::ph1bcast::{BCAST_CREATE_DRAFT, BCAST_DELIVER_COMMIT};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1delivery::DELIVERY_SEND_COMMIT;
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1link::{
        InviteeType, LinkStatus, TokenId, LINK_INVITE_GENERATE_DRAFT,
        LINK_INVITE_OPEN_ACTIVATE_COMMIT,
    };
    use selene_kernel_contracts::ph1m::{
        MemoryCandidate, MemoryConfidence, MemoryKey, MemoryProvenance, MemorySensitivityFlag,
        MemoryUsePolicy, MemoryValue,
    };
    use selene_kernel_contracts::ph1n::{
        Chat, EvidenceSpan, FieldKey, FieldValue, IntentDraft, IntentField, IntentType,
        OverallConfidence, Ph1nResponse, SensitivityLevel, TranscriptHash,
    };
    use selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT;
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1simcat::{
        SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType,
        SimulationVersion,
    };
    use selene_kernel_contracts::ph1x::{
        ConfirmAnswer, IdentityContext, PendingState, Ph1xDirective, ThreadPolicyFlags, ThreadState,
    };
    use selene_kernel_contracts::runtime_execution::{
        AuthorityPolicyDecision, IdentityRecoveryState, IdentityTrustTier,
        MemoryEligibilityDecision, OnboardingReadinessState, SimulationCertificationState,
    };
    use selene_kernel_contracts::{
        ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState,
    };
    use selene_storage::ph1f::{
        AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessVerificationLevel,
        BcastPolicyUpdateValue, DeviceRecord, IdentityRecord, IdentityStatus,
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

    fn reauth_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_REAUTH_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn device_claim_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn reenrollment_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_ENROLLMENT_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn profile_not_enrolled_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn spoof_risk_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_metrics_and_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_SPOOF_RISK,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                4500,
                None,
                selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::SuspectedSpoof,
                vec![],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn low_confidence_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn gray_zone_margin_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                Some(user_id),
                None,
            )
            .unwrap(),
        )
    }

    fn echo_unsafe_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                None,
                None,
            )
            .unwrap(),
        )
    }

    fn no_speech_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_NO_SPEECH,
                vec![],
                None,
                None,
            )
            .unwrap(),
        )
    }

    fn multi_speaker_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
                vec![
                    DiarizationSegment::v1(
                        MonotonicTimeNs(1),
                        MonotonicTimeNs(2),
                        Some(SpeakerLabel::speaker_a()),
                    )
                    .unwrap(),
                    DiarizationSegment::v1(
                        MonotonicTimeNs(3),
                        MonotonicTimeNs(4),
                        Some(SpeakerLabel::speaker_b()),
                    )
                    .unwrap(),
                ],
                None,
                None,
            )
            .unwrap(),
        )
    }

    fn governance_quarantined_confirmed_voice_assertion(
        runtime: &AppServerIngressRuntime,
        runtime_execution_envelope: &RuntimeExecutionEnvelope,
    ) -> (Ph1VoiceIdResponse, GovernanceExecutionState) {
        let decision = runtime
            .runtime_governance()
            .debug_quarantine_identity_voice_engine_for_tests(
                runtime_execution_envelope.session_id.map(|value| value.0),
                Some(runtime_execution_envelope.turn_id.0),
            );
        (
            confirmed_voice_assertion(runtime_execution_envelope.actor_identity.clone()),
            decision.governance_state,
        )
    }

    fn governance_reason_code_for_state(
        runtime: &AppServerIngressRuntime,
        governance_state: &GovernanceExecutionState,
    ) -> Option<String> {
        let sequence = governance_state
            .decision_log_ref
            .as_deref()?
            .strip_prefix("gov_decision_")?
            .parse::<u64>()
            .ok()?;
        runtime
            .runtime_governance_decision_log_snapshot()
            .into_iter()
            .find(|entry| entry.sequence == sequence)
            .map(|entry| entry.reason_code)
    }

    fn ph1x_payload_value<'a>(
        row: &'a selene_kernel_contracts::ph1j::AuditEvent,
        key: &str,
    ) -> Option<&'a str> {
        row.payload_min
            .entries
            .get(&PayloadKey::new(key).unwrap())
            .map(|value| value.as_str())
    }

    fn assert_ph1x_payload_absent(row: &selene_kernel_contracts::ph1j::AuditEvent, key: &str) {
        assert_eq!(
            ph1x_payload_value(row, key),
            None,
            "{key} must remain absent from PH1.X fail-closed respond rows",
        );
    }

    fn find_ph1x_respond_row<'a>(
        rows: &'a [&'a selene_kernel_contracts::ph1j::AuditEvent],
        expected_response_kind: &str,
    ) -> &'a selene_kernel_contracts::ph1j::AuditEvent {
        rows.iter()
            .copied()
            .find(|row| ph1x_payload_value(row, "response_kind") == Some(expected_response_kind))
            .unwrap_or_else(|| {
                panic!("{expected_response_kind} fail-closed response must emit PH1.X audit row")
            })
    }

    fn assert_ph1x_fail_closed_respond_payload(
        runtime: &AppServerIngressRuntime,
        row: &selene_kernel_contracts::ph1j::AuditEvent,
        out: &AppVoiceTurnExecutionOutcome,
        expected_response_kind: &str,
    ) {
        assert!(
            out.ph1x_response.is_some(),
            "fail-closed PH1.X respond rows must retain the PH1.X response"
        );
        assert_ph1x_fail_closed_respond_payload_core(runtime, row, out, expected_response_kind);
    }

    fn assert_access_fail_closed_respond_payload(
        runtime: &AppServerIngressRuntime,
        row: &selene_kernel_contracts::ph1j::AuditEvent,
        out: &AppVoiceTurnExecutionOutcome,
        expected_response_kind: &str,
    ) {
        assert!(
            out.ph1x_response.is_none(),
            "access fail-closed PH1.X respond rows must not retain the PH1.X response"
        );
        assert_ph1x_fail_closed_respond_payload_core(runtime, row, out, expected_response_kind);
    }

    fn assert_missing_simulation_notify_respond_payload(
        runtime: &AppServerIngressRuntime,
        row: &selene_kernel_contracts::ph1j::AuditEvent,
        out: &AppVoiceTurnExecutionOutcome,
    ) {
        assert!(
            out.ph1x_response.is_none(),
            "missing simulation notify PH1.X respond rows must not retain the PH1.X response"
        );
        assert_ph1x_fail_closed_respond_payload_core(
            runtime,
            row,
            out,
            "MISSING_SIMULATION_NOTIFY_SUBMITTED",
        );
    }

    fn assert_authority_state_and_proof_simulation_certification(
        store: &Ph1fStore,
        out: &AppVoiceTurnExecutionOutcome,
        expected_policy_decision: AuthorityPolicyDecision,
        expected_simulation_certification_state: SimulationCertificationState,
        expected_proof_state: &str,
    ) {
        let authority_state = out
            .runtime_execution_envelope
            .authority_state
            .as_ref()
            .expect("authority state should be attached");
        assert_eq!(authority_state.policy_decision, expected_policy_decision);
        assert_eq!(
            authority_state.simulation_certification_state,
            expected_simulation_certification_state
        );
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out.runtime_execution_envelope.request_id, 4)
            .expect("proof rows should be readable");
        assert_eq!(proof_rows.len(), 1);
        assert_eq!(
            proof_rows[0].simulation_certification_state.as_deref(),
            Some(expected_proof_state)
        );
    }

    fn assert_proof_authority_reason_token(
        store: &Ph1fStore,
        out: &AppVoiceTurnExecutionOutcome,
        expected_reason_token: &str,
    ) {
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out.runtime_execution_envelope.request_id, 4)
            .expect("proof rows should be readable");
        assert_eq!(proof_rows.len(), 1);
        let authority_reference = proof_rows[0]
            .authority_decision_reference
            .as_deref()
            .expect("proof authority decision reference should be attached");
        assert!(
            authority_reference.contains(&format!("reason_code={expected_reason_token}")),
            "expected authority decision reference '{authority_reference}' to contain reason_code={expected_reason_token}"
        );
    }

    fn assert_ph1x_fail_closed_respond_payload_core(
        runtime: &AppServerIngressRuntime,
        row: &selene_kernel_contracts::ph1j::AuditEvent,
        out: &AppVoiceTurnExecutionOutcome,
        expected_response_kind: &str,
    ) {
        let identity_state = out
            .runtime_execution_envelope
            .identity_state
            .as_ref()
            .expect("identity state must remain attached to fail-closed PH1.X rows");
        let expected_identity_reason_code_hex =
            identity_reason_code(identity_state).map(|code| format!("0x{:X}", code.0));
        let expected_governance_reason_code = out
            .runtime_execution_envelope
            .governance_state
            .as_ref()
            .and_then(|state| governance_reason_code_for_state(runtime, state));

        assert_eq!(ph1x_payload_value(row, "directive"), Some("respond"));
        assert_eq!(
            ph1x_payload_value(row, "response_kind"),
            Some(expected_response_kind)
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some(identity_consistency_level_literal(
                identity_state.consistency_level
            ))
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some(identity_trust_tier_literal(identity_state.trust_tier))
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some(identity_recovery_state_literal(
                identity_state.recovery_state
            ))
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_tier_v2"),
            Some(identity_tier_v2_literal(identity_state.identity_tier_v2))
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_spoof_liveness_status"),
            Some(spoof_liveness_status_literal(
                identity_state.spoof_liveness_status
            ))
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some(if identity_state.step_up_required {
                "true"
            } else {
                "false"
            })
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_cluster_drift_detected"),
            Some(if identity_state.cluster_drift_detected {
                "true"
            } else {
                "false"
            })
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            expected_identity_reason_code_hex.as_deref()
        );
        assert_eq!(
            ph1x_payload_value(row, "governance_reason_code"),
            expected_governance_reason_code.as_deref()
        );
        assert_ph1x_payload_absent(row, "decision_log_family");
        assert_ph1x_payload_absent(row, "decision_v1");
        assert_ph1x_payload_absent(row, "voice_decision");
        assert_ph1x_payload_absent(row, "voice_reason_code_hex");
    }

    fn run_access_ap_required_fail_closed_turn() -> (
        AppServerIngressRuntime,
        Ph1fStore,
        AppVoiceTurnExecutionOutcome,
        CorrelationId,
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_escalate_user").unwrap();
        let device_id = DeviceId::new("finder_exec_escalate_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_link_send_escalate_access_instance(&mut store, &actor_user_id, "tenant_1");
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

        let request_1 = AppVoiceIngressRequest::v1(
            CorrelationId(9806),
            TurnId(9906),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build_1 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(25),
            thread_key: Some("finder_exec_06_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_1, x_build_1)
            .unwrap();
        assert_eq!(out_1.next_move, AppVoiceTurnNextMove::Confirm);

        let thread_state_after_confirm = out_1
            .ph1x_response
            .expect("confirm run should include ph1x response")
            .thread_state;
        let response_correlation_id = CorrelationId(9807);
        let request_2 = AppVoiceIngressRequest::v1(
            response_correlation_id,
            TurnId(9907),
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
        let x_build_2 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(26),
            thread_key: Some("finder_exec_06_thread".to_string()),
            thread_state: thread_state_after_confirm,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_2 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_2, x_build_2)
            .expect("escalated access must return deterministic fail-closed response");
        (runtime, store, out_2, response_correlation_id)
    }

    fn run_access_denied_fail_closed_turn() -> (
        AppServerIngressRuntime,
        Ph1fStore,
        AppVoiceTurnExecutionOutcome,
        CorrelationId,
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_deny_user").unwrap();
        let device_id = DeviceId::new("finder_exec_deny_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_link_send_denied_access_instance(&mut store, &actor_user_id, "tenant_1");
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

        let request_1 = AppVoiceIngressRequest::v1(
            CorrelationId(9810),
            TurnId(9910),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build_1 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(29),
            thread_key: Some("finder_exec_09_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_1, x_build_1)
            .unwrap();
        assert_eq!(out_1.next_move, AppVoiceTurnNextMove::Confirm);

        let thread_state_after_confirm = out_1
            .ph1x_response
            .expect("confirm run should include ph1x response")
            .thread_state;
        let response_correlation_id = CorrelationId(9811);
        let request_2 = AppVoiceIngressRequest::v1(
            response_correlation_id,
            TurnId(9911),
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
        let x_build_2 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(30),
            thread_key: Some("finder_exec_09_thread".to_string()),
            thread_state: thread_state_after_confirm,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_2 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_2, x_build_2)
            .expect("denied access must return deterministic fail-closed response");
        (runtime, store, out_2, response_correlation_id)
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

    fn invite_link_send_draft_broken_english(
        recipient: &str,
        recipient_contact: &str,
        tenant_id: &str,
    ) -> Ph1nResponse {
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
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: 58,
                    verbatim_excerpt: "selene pls send link tom now, number +14155550100, thanks"
                        .to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Private,
                true,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn order_pizza_draft() -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::BookTable,
                SchemaVersion(1),
                vec![
                    IntentField {
                        key: FieldKey::Task,
                        value: FieldValue::verbatim("order pizza".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::Place,
                        value: FieldValue::verbatim("dominos".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                    IntentField {
                        key: FieldKey::When,
                        value: FieldValue::verbatim("now".to_string()).unwrap(),
                        confidence: OverallConfidence::High,
                    },
                ],
                vec![],
                OverallConfidence::High,
                vec![EvidenceSpan {
                    field: FieldKey::Task,
                    transcript_hash: TranscriptHash(1),
                    start_byte: 0,
                    end_byte: 30,
                    verbatim_excerpt: "selene order pizza from dominos".to_string(),
                }],
                ReasonCodeId(1),
                SensitivityLevel::Private,
                true,
                vec![],
                vec![],
            )
            .unwrap(),
        )
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

    fn runtime_with_search_tool_fixtures() -> AppServerIngressRuntime {
        AppServerIngressRuntime {
            ph1e_runtime: Ph1eRuntime::new_with_provider_config(
                Ph1eConfig::mvp_v1(),
                Ph1eProviderConfig {
                    brave_api_key: Some("fixture_brave_key".to_string()),
                    brave_web_url: "https://search.selene.ai/res/v1/web/search".to_string(),
                    brave_news_url: "https://search.selene.ai/res/v1/news/search".to_string(),
                    brave_web_fixture_json: Some(
                        r#"{"web":{"results":[{"title":"Selene web result","url":"https://search.selene.ai/result-1","description":"Provider-backed web snippet"}]}}"#
                            .to_string(),
                    ),
                    brave_news_fixture_json: Some(
                        r#"{"results":[{"title":"Selene news result","url":"https://news.selene.ai/story-1","description":"Provider-backed news snippet"}]}"#
                            .to_string(),
                    ),
                    openai_api_key: None,
                    openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
                    openai_model: "gpt-4o-mini".to_string(),
                    user_agent: "selene-os-app-ingress-test/1.0".to_string(),
                    proxy_config: Ph1eProxyConfig {
                        mode: Ph1eProxyMode::Off,
                        http_proxy_url: None,
                        https_proxy_url: None,
                    },
                    url_fetch_fixture_html: Some(
                        "<html><body><h1>Selene URL source</h1><p>This is a deterministic fixture page for URL fetch and citation chunking with provenance metadata.</p></body></html>"
                            .to_string(),
                    ),
                },
            ),
            ..AppServerIngressRuntime::default()
        }
    }

    fn runtime_with_search_tool_fixtures_and_runtime_governance(
        runtime_governance: RuntimeGovernanceRuntime,
    ) -> AppServerIngressRuntime {
        AppServerIngressRuntime {
            ph1e_runtime: Ph1eRuntime::new_with_provider_config(
                Ph1eConfig::mvp_v1(),
                Ph1eProviderConfig {
                    brave_api_key: Some("fixture_brave_key".to_string()),
                    brave_web_url: "https://search.selene.ai/res/v1/web/search".to_string(),
                    brave_news_url: "https://search.selene.ai/res/v1/news/search".to_string(),
                    brave_web_fixture_json: Some(
                        r#"{"web":{"results":[{"title":"Selene web result","url":"https://search.selene.ai/result-1","description":"Provider-backed web snippet"}]}}"#
                            .to_string(),
                    ),
                    brave_news_fixture_json: Some(
                        r#"{"results":[{"title":"Selene news result","url":"https://news.selene.ai/story-1","description":"Provider-backed news snippet"}]}"#
                            .to_string(),
                    ),
                    openai_api_key: None,
                    openai_responses_url: "https://api.openai.com/v1/responses".to_string(),
                    openai_model: "gpt-4o-mini".to_string(),
                    user_agent: "selene-os-app-ingress-test/1.0".to_string(),
                    proxy_config: Ph1eProxyConfig {
                        mode: Ph1eProxyMode::Off,
                        http_proxy_url: None,
                        https_proxy_url: None,
                    },
                    url_fetch_fixture_html: Some(
                        "<html><body><h1>Selene URL source</h1><p>This is a deterministic fixture page for URL fetch and citation chunking with provenance metadata.</p></body></html>"
                            .to_string(),
                    ),
                },
            ),
            runtime_governance,
            ..AppServerIngressRuntime::default()
        }
    }

    fn run_protected_response_turn_with_identity_assertion(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        device_id: DeviceId,
        assertion: Ph1VoiceIdResponse,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> AppVoiceTurnExecutionOutcome {
        run_protected_response_turn_with_identity_assertion_and_governance_state(
            runtime,
            store,
            actor_user_id,
            device_id,
            assertion,
            None,
            correlation_id,
            turn_id,
        )
    }

    fn run_protected_response_turn_with_identity_assertion_and_governance_state(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        device_id: DeviceId,
        assertion: Ph1VoiceIdResponse,
        governance_state: Option<GovernanceExecutionState>,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> AppVoiceTurnExecutionOutcome {
        let request = AppVoiceIngressRequest::v1(
            correlation_id,
            turn_id,
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let request_session_id = request.voice_id_request.session_state_ref.session_id;
        let request_session_state = request.voice_id_request.session_state_ref.session_state;
        let received_at = request.voice_id_request.now;

        let outcome = runtime.run_voice_turn(store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(mut forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };
        forwarded.voice_identity_assertion = assertion;
        if let Some(governance_state) = governance_state {
            forwarded.runtime_execution_envelope = forwarded
                .runtime_execution_envelope
                .with_governance_state(Some(governance_state))
                .unwrap();
        }
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(17),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(web_search_draft("search the web for selene tool parity")),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let ph1x_request = runtime
            .build_ph1x_request_for_forwarded_voice(
                store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id,
                    turn_id,
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let finder_terminal = runtime
            .run_finder_terminal_packet_for_turn(
                store,
                &actor_user_id,
                Some("tenant_1"),
                Some(&packet),
            )
            .unwrap();
        assert!(finder_terminal.is_none());
        let runtime_execution_envelope = packet
            .runtime_execution_envelope
            .clone()
            .expect("runtime execution envelope must be attached");
        let out = runtime
            .run_ph1x_and_dispatch_with_access_fail_closed(
                store,
                runtime_execution_envelope,
                OsVoiceLiveTurnOutcome::Forwarded(forwarded),
                request_session_state,
                ph1x_request,
                &actor_user_id,
                Some(&device_id),
                Some("tenant_1"),
                request_session_id,
                MonotonicTimeNs(17),
            )
            .unwrap();
        runtime
            .finalize_voice_turn_outcome(
                store,
                out,
                finder_terminal.as_ref(),
                &actor_user_id,
                Some(&device_id),
                Some("tenant_1"),
                request_session_id,
                correlation_id,
                turn_id,
                received_at,
                MonotonicTimeNs(17),
            )
            .unwrap()
    }

    fn run_protected_chat_response_turn_with_identity_assertion(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        device_id: DeviceId,
        assertion: Ph1VoiceIdResponse,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> AppVoiceTurnExecutionOutcome {
        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            runtime,
            store,
            actor_user_id,
            device_id,
            assertion,
            correlation_id,
            turn_id,
        );
        finalize_pending_protected_chat_response_turn(runtime, store, pending).unwrap()
    }

    struct PendingProtectedChatResponseTurn {
        out: AppVoiceTurnExecutionOutcome,
        finder_terminal: Option<FinderTerminalPacket>,
        actor_user_id: UserId,
        device_id: DeviceId,
        request_session_id: Option<selene_kernel_contracts::ph1l::SessionId>,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        received_at: MonotonicTimeNs,
    }

    fn prepare_protected_chat_response_turn_with_identity_assertion(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        actor_user_id: UserId,
        device_id: DeviceId,
        assertion: Ph1VoiceIdResponse,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> PendingProtectedChatResponseTurn {
        let request = AppVoiceIngressRequest::v1(
            correlation_id,
            turn_id,
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let request_session_id = request.voice_id_request.session_state_ref.session_id;
        let request_session_state = request.voice_id_request.session_state_ref.session_state;
        let received_at = request.voice_id_request.now;

        let outcome = runtime.run_voice_turn(store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(mut forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };
        forwarded.voice_identity_assertion = assertion;
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(17),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(
                Chat::v1("give me a direct answer".to_string(), ReasonCodeId(1))
                    .map(Ph1nResponse::Chat)
                    .unwrap(),
            ),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let ph1x_request = runtime
            .build_ph1x_request_for_forwarded_voice(
                store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id,
                    turn_id,
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let finder_terminal = runtime
            .run_finder_terminal_packet_for_turn(
                store,
                &actor_user_id,
                Some("tenant_1"),
                Some(&packet),
            )
            .unwrap();
        assert!(finder_terminal.is_none());
        let runtime_execution_envelope = packet
            .runtime_execution_envelope
            .clone()
            .expect("runtime execution envelope must be attached");
        let out = runtime
            .run_ph1x_and_dispatch_with_access_fail_closed(
                store,
                runtime_execution_envelope,
                OsVoiceLiveTurnOutcome::Forwarded(forwarded),
                request_session_state,
                ph1x_request,
                &actor_user_id,
                Some(&device_id),
                Some("tenant_1"),
                request_session_id,
                MonotonicTimeNs(17),
            )
            .unwrap();
        PendingProtectedChatResponseTurn {
            out,
            finder_terminal,
            actor_user_id,
            device_id,
            request_session_id,
            correlation_id,
            turn_id,
            received_at,
        }
    }

    fn finalize_pending_protected_chat_response_turn(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        pending: PendingProtectedChatResponseTurn,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
        runtime.finalize_voice_turn_outcome(
            store,
            pending.out,
            pending.finder_terminal.as_ref(),
            &pending.actor_user_id,
            Some(&pending.device_id),
            Some("tenant_1"),
            pending.request_session_id,
            pending.correlation_id,
            pending.turn_id,
            pending.received_at,
            MonotonicTimeNs(17),
        )
    }

    fn assert_posture_finalization_requires_canonical_identity_state(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        mut pending: PendingProtectedChatResponseTurn,
    ) {
        pending.out.runtime_execution_envelope = pending
            .out
            .runtime_execution_envelope
            .with_identity_state(None)
            .unwrap();
        let err = finalize_pending_protected_chat_response_turn(runtime, store, pending)
            .expect_err("missing posture identity state must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state"
                );
                assert_eq!(
                    reason,
                    "must carry canonical identity state for posture fail-closed classification"
                );
            }
            other => panic!("expected posture identity-state contract violation, got {other:?}"),
        }
    }

    fn assert_posture_finalization_requires_reason_alignment(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        mut pending: PendingProtectedChatResponseTurn,
        divergent_reason_code: ReasonCodeId,
    ) {
        let voice_reason_code = pending
            .out
            .runtime_execution_envelope
            .voice_identity_assertion
            .as_ref()
            .and_then(voice_identity_reason_code)
            .expect("posture voice assertion reason code must remain attached");
        assert!(posture_fail_closed_reason_code(Some(voice_reason_code)));
        let identity_state = pending
            .out
            .runtime_execution_envelope
            .identity_state
            .as_mut()
            .expect("posture identity state must remain attached");
        identity_state.reason_code = Some(u64::from(divergent_reason_code.0));
        assert_ne!(identity_reason_code(identity_state), Some(voice_reason_code));

        let err = finalize_pending_protected_chat_response_turn(runtime, store, pending)
            .expect_err("divergent posture reason codes must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state.reason_code"
                );
                assert_eq!(
                    reason,
                    "must match canonical voice identity assertion reason code for posture fail-closed classification"
                );
            }
            other => panic!("expected posture reason-alignment contract violation, got {other:?}"),
        }
    }

    fn assert_posture_finalization_requires_canonical_voice_identity_assertion(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        mut pending: PendingProtectedChatResponseTurn,
    ) {
        pending.out.runtime_execution_envelope = pending
            .out
            .runtime_execution_envelope
            .with_voice_identity_assertion(None)
            .unwrap();
        let err = finalize_pending_protected_chat_response_turn(runtime, store, pending)
            .expect_err("missing posture voice identity assertion must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.voice_identity_assertion"
                );
                assert_eq!(
                    reason,
                    "must carry canonical voice identity assertion for posture fail-closed classification"
                );
            }
            other => panic!("expected posture voice-assertion contract violation, got {other:?}"),
        }
    }

    fn assert_posture_finalization_requires_canonical_voice_assertion_posture_reason(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        mut pending: PendingProtectedChatResponseTurn,
        divergent_reason_code: ReasonCodeId,
    ) {
        assert!(!posture_fail_closed_reason_code(Some(divergent_reason_code)));
        let voice_identity_assertion = pending
            .out
            .runtime_execution_envelope
            .voice_identity_assertion
            .as_mut()
            .expect("posture voice identity assertion must remain attached");
        match voice_identity_assertion {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => {
                ok.reason_code = Some(divergent_reason_code);
            }
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(unknown) => {
                unknown.reason_code = divergent_reason_code;
            }
        }

        let err = finalize_pending_protected_chat_response_turn(runtime, store, pending)
            .expect_err("non-posture voice assertion reason must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.voice_identity_assertion.reason_code"
                );
                assert_eq!(
                    reason,
                    "must carry canonical posture reason code for posture fail-closed classification"
                );
            }
            other => panic!(
                "expected posture voice-assertion reason contract violation, got {other:?}"
            ),
        }
    }

    fn assert_posture_finalization_requires_canonical_identity_state_shape(
        runtime: &AppServerIngressRuntime,
        store: &mut Ph1fStore,
        mut pending: PendingProtectedChatResponseTurn,
        divergent_trust_tier: IdentityTrustTier,
    ) {
        let identity_state = pending
            .out
            .runtime_execution_envelope
            .identity_state
            .as_mut()
            .expect("posture identity state must remain attached");
        identity_state.trust_tier = divergent_trust_tier;

        let err = finalize_pending_protected_chat_response_turn(runtime, store, pending)
            .expect_err("non-canonical posture identity-state shape must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.identity_state"
                );
                assert_eq!(
                    reason,
                    "must match canonical posture identity state shape for fail-closed classification"
                );
            }
            other => panic!(
                "expected posture identity-state shape contract violation, got {other:?}"
            ),
        }
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

    fn data_analysis_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::DataAnalysisQuery,
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

    fn deep_research_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::DeepResearchQuery,
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

    fn record_mode_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::RecordModeQuery,
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

    fn connector_query_draft(query: &str) -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::ConnectorQuery,
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

    fn seed_link_send_denied_access_instance(store: &mut Ph1fStore, actor: &UserId, tenant: &str) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                "role.link_sender_denied".to_string(),
                AccessMode::A,
                "{\"allow\":[]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Active,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
    }

    fn seed_link_send_escalate_access_instance(
        store: &mut Ph1fStore,
        actor: &UserId,
        tenant: &str,
    ) {
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                tenant.to_string(),
                actor.clone(),
                "role.link_sender_escalate".to_string(),
                AccessMode::A,
                "{\"allow\":[\"LINK_INVITE\",\"DELIVERY_SEND\"]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Restricted,
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

    fn seed_persona_profile_for_actor(
        store: &mut Ph1fStore,
        actor: &UserId,
        device: &DeviceId,
        tenant: &str,
        style_profile_ref: &str,
        idempotency_key: &str,
    ) {
        store
            .ph1persona_profile_commit(
                MonotonicTimeNs(25),
                tenant.to_string(),
                CorrelationId(98_001),
                TurnId(98_001),
                None,
                actor.clone(),
                device.clone(),
                style_profile_ref.to_string(),
                "voice".to_string(),
                "prefs_v1".to_string(),
                ReasonCodeId(0x5800_00E1),
                idempotency_key.to_string(),
            )
            .unwrap();
    }

    fn seed_invite_link_for_click(
        store: &mut Ph1fStore,
        inviter_user_id: &UserId,
        tenant_id: &str,
        now: MonotonicTimeNs,
    ) -> (TokenId, String) {
        let (link, _) = store
            .ph1link_invite_generate_draft(
                now,
                inviter_user_id.clone(),
                InviteeType::Employee,
                Some(tenant_id.to_string()),
                None,
                None,
                None,
            )
            .expect("link draft generation should succeed");
        (link.token_id, link.token_signature)
    }

    fn seed_invite_link_for_click_with_employee_prefilled_context(
        store: &mut Ph1fStore,
        inviter_user_id: &UserId,
        tenant_id: &str,
        now: MonotonicTimeNs,
    ) -> (TokenId, String) {
        let prefilled = selene_kernel_contracts::ph1link::PrefilledContext::v1(
            Some(tenant_id.to_string()),
            Some("company_1".to_string()),
            Some("position_1".to_string()),
            Some("loc_1".to_string()),
            Some("2026-03-01".to_string()),
            None,
            Some("band_l2".to_string()),
            vec!["US".to_string()],
        )
        .expect("prefilled context must be valid");
        let (link, _) = store
            .ph1link_invite_generate_draft(
                now,
                inviter_user_id.clone(),
                InviteeType::Employee,
                Some(tenant_id.to_string()),
                None,
                Some(prefilled),
                None,
            )
            .expect("link draft generation should succeed");
        (link.token_id, link.token_signature)
    }

    fn seed_employee_company_and_position(
        store: &mut Ph1fStore,
        tenant_id: &str,
        now: MonotonicTimeNs,
    ) {
        let tenant = TenantId::new(tenant_id.to_string()).expect("tenant id must be valid");
        store
            .ph1tenant_company_upsert(selene_storage::ph1f::TenantCompanyRecord {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                tenant_id: tenant.clone(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Co".to_string(),
                jurisdiction: "US".to_string(),
                lifecycle_state: selene_storage::ph1f::TenantCompanyLifecycleState::Active,
                created_at: now,
                updated_at: now,
            })
            .expect("tenant company seed must succeed");

        let position = selene_kernel_contracts::ph1position::PositionRecord::v1(
            tenant.clone(),
            "company_1".to_string(),
            selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
            "Operator".to_string(),
            "Operations".to_string(),
            "US".to_string(),
            selene_kernel_contracts::ph1position::PositionScheduleType::FullTime,
            "profile_ops".to_string(),
            "band_l2".to_string(),
            selene_kernel_contracts::ph1position::PositionLifecycleState::Active,
            now,
            now,
        )
        .expect("position seed must be valid");
        store
            .ph1position_upsert(position)
            .expect("position seed upsert must succeed");
    }

    fn seed_employee_position_schema_requiring_sender_verification(
        store: &mut Ph1fStore,
        actor_user_id: &UserId,
        tenant_id: &str,
        now: MonotonicTimeNs,
    ) {
        let tenant = TenantId::new(tenant_id.to_string()).expect("tenant id must be valid");
        let selector = selene_kernel_contracts::ph1position::PositionSchemaSelectorSnapshot {
            company_size: Some("SMALL".to_string()),
            industry_code: Some("LOGISTICS".to_string()),
            jurisdiction: Some("US".to_string()),
            position_family: Some("OPS".to_string()),
        };
        let doc_field = selene_kernel_contracts::ph1position::PositionRequirementFieldSpec {
            field_key: "working_hours".to_string(),
            field_type: selene_kernel_contracts::ph1position::PositionRequirementFieldType::String,
            required_rule:
                selene_kernel_contracts::ph1position::PositionRequirementRuleType::Always,
            required_predicate_ref: None,
            validation_ref: None,
            sensitivity:
                selene_kernel_contracts::ph1position::PositionRequirementSensitivity::Private,
            exposure_rule:
                selene_kernel_contracts::ph1position::PositionRequirementExposureRule::InternalOnly,
            evidence_mode:
                selene_kernel_contracts::ph1position::PositionRequirementEvidenceMode::DocRequired,
            prompt_short: "Provide working hours".to_string(),
            prompt_long: "Please provide working hours evidence.".to_string(),
        };
        store
            .ph1position_requirements_schema_create_draft(
                now,
                actor_user_id.clone(),
                tenant.clone(),
                "company_1".to_string(),
                selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
                "schema_v1".to_string(),
                selector,
                vec![doc_field],
                "onb-schema-create".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
                ReasonCodeId(0x5900_0006),
            )
            .expect("position schema draft must be created");
        store
            .ph1position_requirements_schema_activate_commit(
                MonotonicTimeNs(now.0.saturating_add(1)),
                actor_user_id.clone(),
                tenant,
                "company_1".to_string(),
                selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
                "schema_v1".to_string(),
                selene_kernel_contracts::ph1position::PositionSchemaApplyScope::NewHiresOnly,
                "onb-schema-activate".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0008),
            )
            .expect("position schema activation must succeed");
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

    fn recanonicalize_forwarded_bundle_for_tests(
        forwarded: &mut crate::ph1os::OsVoiceLiveForwardBundle,
    ) {
        let runtime_execution_envelope = forwarded
            .runtime_execution_envelope
            .with_voice_identity_assertion(Some(forwarded.voice_identity_assertion.clone()))
            .expect(
                "forwarded runtime envelope should allow voice-assertion reset before test recanonicalization",
            );
        let canonical_runtime_execution_envelope = attach_identity_state_for_governed_voice_turn(
            &runtime_execution_envelope
                .with_identity_state(None)
                .expect(
                    "forwarded runtime envelope should allow identity-state reset before test recanonicalization",
                ),
            &forwarded.voice_identity_assertion,
        )
        .expect("test forwarded runtime envelope must allow canonical identity-state attachment");
        forwarded.runtime_execution_envelope = canonical_runtime_execution_envelope.clone();
        forwarded.top_level_bundle.runtime_execution_envelope =
            Some(canonical_runtime_execution_envelope);
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
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(forwarded.top_level_bundle.path, OsTopLevelTurnPath::Voice);
        assert_eq!(
            forwarded.top_level_bundle.runtime_execution_envelope,
            Some(forwarded.runtime_execution_envelope.clone())
        );
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
    fn at_ingress_01b_run_voice_turn_with_governed_envelope_adopts_forwarded_canonical_runtime_envelope(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_direct_adopt_user").unwrap();
        let device_id = DeviceId::new("ingress_direct_adopt_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9105),
            TurnId(9205),
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

        let (outcome, runtime_execution_envelope) = runtime
            .run_voice_turn_with_governed_envelope(&mut store, request)
            .unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };

        assert_eq!(
            runtime_execution_envelope,
            forwarded.runtime_execution_envelope
        );
        assert_eq!(
            forwarded.top_level_bundle.runtime_execution_envelope,
            Some(forwarded.runtime_execution_envelope.clone())
        );
    }

    #[test]
    fn at_ingress_02_android_wake_routes_with_wake_stage() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_android_user").unwrap();
        let device_id = DeviceId::new("ingress_android_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let mut request = AppVoiceIngressRequest::v1(
            CorrelationId(9102),
            TurnId(9202),
            AppPlatform::Android,
            OsVoiceTrigger::WakeWord,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        request
            .runtime_execution_envelope
            .platform_context
            .integrity_status =
            selene_kernel_contracts::runtime_execution::ClientIntegrityStatus::Attested;
        request
            .runtime_execution_envelope
            .platform_context
            .attestation_ref = Some("attestation:android:wake:1".to_string());
        request
            .runtime_execution_envelope
            .platform_context
            .capture_artifact_trust_verified = true;
        request
            .runtime_execution_envelope
            .platform_context
            .capture_artifact_observed_at_ns = Some(2);
        request
            .runtime_execution_envelope
            .platform_context
            .capture_artifact_retention_deadline_ns = Some(20);
        request
            .runtime_execution_envelope
            .validate()
            .expect("attested android wake runtime envelope must validate");

        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert!(forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.W".to_string()));
        assert!(forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.L".to_string()));
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
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
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
        assert!(!forwarded
            .top_level_bundle
            .always_on_sequence
            .contains(&"PH1.L".to_string()));
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
            thread_key: None,
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
        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let canonical_voice_identity_assertion = envelope.voice_identity_assertion.clone().expect(
            "agent packet runtime envelope should carry canonical voice identity assertion",
        );
        assert!(
            envelope.identity_state.is_some(),
            "agent packet runtime envelope should carry canonical identity state"
        );
        assert_eq!(
            packet.voice_identity_assertion,
            canonical_voice_identity_assertion
        );
        assert_eq!(
            ph1x_request.identity_context,
            IdentityContext::Voice(canonical_voice_identity_assertion)
        );
    }

    #[test]
    fn at_ingress_04c_packet_builder_directly_adopts_forwarded_runtime_execution_envelope() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_packet_direct_user").unwrap();
        let device_id = DeviceId::new("ingress_packet_direct_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9106),
            TurnId(9206),
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

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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

        let ph1x_request = runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9106),
                    turn_id: TurnId(9206),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        assert_eq!(
            envelope
                .clone()
                .with_memory_state(None)
                .expect("packet runtime envelope should allow memory-state reset"),
            forwarded.runtime_execution_envelope
        );
        assert!(envelope.identity_state.is_some());
        assert_eq!(
            ph1x_request.identity_prompt_scope_key,
            forwarded.identity_prompt_scope_key
        );
        assert_eq!(
            forwarded.top_level_bundle.runtime_execution_envelope,
            Some(forwarded.runtime_execution_envelope.clone())
        );
    }

    #[test]
    fn at_ingress_04d_packet_builder_uses_canonical_runtime_envelope_voice_identity_assertion() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_packet_assertion_user").unwrap();
        let device_id = DeviceId::new("ingress_packet_assertion_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9107),
            TurnId(9207),
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

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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

        let ph1x_request = runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9107),
                    turn_id: TurnId(9207),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let canonical_voice_identity_assertion = envelope.voice_identity_assertion.clone().expect(
            "agent packet runtime envelope should carry canonical voice identity assertion",
        );
        assert!(
            envelope.identity_state.is_some(),
            "agent packet runtime envelope should carry canonical identity state"
        );
        assert_eq!(
            packet.voice_identity_assertion,
            canonical_voice_identity_assertion
        );
        assert_eq!(
            ph1x_request.identity_prompt_scope_key,
            forwarded.identity_prompt_scope_key
        );
    }

    #[test]
    fn at_ingress_04e_ph1x_request_identity_context_uses_canonical_runtime_envelope_voice_identity_assertion(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_identity_context_user").unwrap();
        let device_id = DeviceId::new("ingress_identity_context_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9108),
            TurnId(9208),
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
            panic!("expected forwarded outcome");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id.clone());
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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
        let packet = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9108),
                TurnId(9208),
                &forwarded,
                None,
                Some("tenant_1"),
                x_build,
            )
            .unwrap();
        let canonical_voice_identity_assertion = packet
            .runtime_execution_envelope
            .as_ref()
            .and_then(|runtime_execution_envelope| {
                runtime_execution_envelope.voice_identity_assertion.clone()
            })
            .expect(
                "packet runtime execution envelope should carry canonical voice identity assertion",
            );

        let ph1x_request =
            build_ph1x_request_from_agent_input_packet(AppPlatform::Desktop, &packet)
                .expect("ph1x request should use canonical embedded voice identity assertion");
        assert_eq!(
            packet.voice_identity_assertion,
            canonical_voice_identity_assertion
        );
        assert_eq!(
            ph1x_request.identity_context,
            IdentityContext::Voice(canonical_voice_identity_assertion.clone())
        );
        assert_eq!(
            ph1x_request.identity_prompt_scope_key,
            forwarded.identity_prompt_scope_key
        );
    }

    #[test]
    fn at_ingress_04f_ph1x_request_build_fails_closed_when_packet_lacks_embedded_canonical_voice_identity_assertion(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id =
            UserId::new("tenant_1:ingress_identity_context_fail_closed_user").unwrap();
        let device_id = DeviceId::new("ingress_identity_context_fail_closed_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9109),
            TurnId(9209),
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
            panic!("expected forwarded outcome");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id.clone());
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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
        let mut packet = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9109),
                TurnId(9209),
                &forwarded,
                None,
                Some("tenant_1"),
                x_build,
            )
            .unwrap();
        packet.voice_identity_assertion = device_claim_required_voice_assertion(actor_user_id);
        packet.runtime_execution_envelope = Some(
            packet
                .runtime_execution_envelope
                .clone()
                .expect("packet should carry runtime execution envelope")
                .with_voice_identity_assertion(None)
                .expect(
                    "runtime execution envelope should allow clearing embedded voice identity assertion",
                ),
        );

        let err = build_ph1x_request_from_agent_input_packet(AppPlatform::Desktop, &packet)
            .expect_err(
                "ph1x request build must fail closed when embedded canonical voice identity assertion is missing",
            );
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "agent_input_packet.runtime_execution_envelope.voice_identity_assertion"
                );
                assert_eq!(
                    reason,
                    "must carry canonical embedded voice identity assertion for ph1x request"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn at_ingress_04i_ph1x_request_build_fails_closed_when_packet_voice_identity_assertion_diverges_from_embedded_canonical_assertion(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_pkt_assertion_align_fail_user").unwrap();
        let device_id = DeviceId::new("ingress_pkt_assertion_align_fail_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9112),
            TurnId(9212),
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
            panic!("expected forwarded outcome");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id.clone());
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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
        let mut packet = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9112),
                TurnId(9212),
                &forwarded,
                None,
                Some("tenant_1"),
                x_build,
            )
            .unwrap();
        packet.voice_identity_assertion = reauth_required_voice_assertion(actor_user_id);

        let err = build_ph1x_request_from_agent_input_packet(AppPlatform::Desktop, &packet)
            .expect_err(
                "ph1x request build must fail closed when packet voice identity assertion diverges from canonical embedded assertion",
            );
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "agent_input_packet.voice_identity_assertion");
                assert_eq!(
                    reason,
                    "must match canonical embedded voice identity assertion for ph1x request"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn at_ingress_04g_packet_builder_fails_closed_when_forwarded_bundle_lacks_embedded_canonical_voice_identity_assertion(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id =
            UserId::new("tenant_1:forwarded_packet_assertion_fail_closed_user").unwrap();
        let device_id = DeviceId::new("forwarded_packet_assertion_fail_closed_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9110),
            TurnId(9210),
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
            panic!("expected forwarded outcome");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id.clone());
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);
        forwarded.runtime_execution_envelope = forwarded
            .runtime_execution_envelope
            .with_voice_identity_assertion(None)
            .expect(
                "forwarded runtime envelope should allow clearing embedded canonical voice identity assertion",
            );
        forwarded.top_level_bundle.runtime_execution_envelope =
            Some(forwarded.runtime_execution_envelope.clone());

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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

        let err = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9110),
                TurnId(9210),
                &forwarded,
                None,
                Some("tenant_1"),
                x_build,
            )
            .expect_err(
                "forwarded packet build must fail closed when embedded canonical voice identity assertion is missing",
            );
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "os_voice_live_forward_bundle.runtime_execution_envelope.voice_identity_assertion"
                );
                assert_eq!(
                    reason,
                    "must carry canonical embedded voice identity assertion for forwarded packet build"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn at_ingress_04h_packet_builder_fails_closed_when_forwarded_bundle_lacks_embedded_canonical_identity_state(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:fwd_pkt_idstate_fail_user").unwrap();
        let device_id = DeviceId::new("fwd_pkt_idstate_fail_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9111),
            TurnId(9211),
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
            panic!("expected forwarded outcome");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);
        forwarded.runtime_execution_envelope = forwarded
            .runtime_execution_envelope
            .with_identity_state(None)
            .expect(
                "forwarded runtime envelope should allow clearing embedded canonical identity state",
            );
        forwarded.top_level_bundle.runtime_execution_envelope =
            Some(forwarded.runtime_execution_envelope.clone());

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(4),
            thread_key: None,
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

        let err = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9111),
                TurnId(9211),
                &forwarded,
                None,
                Some("tenant_1"),
                x_build,
            )
            .expect_err(
                "forwarded packet build must fail closed when embedded canonical identity state is missing",
            );
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "os_voice_live_forward_bundle.runtime_execution_envelope.identity_state"
                );
                assert_eq!(
                    reason,
                    "must carry canonical embedded identity state for forwarded packet build"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
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
            thread_key: None,
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
    fn at_mem_01_probable_identity_returns_empty_memory_candidates() {
        run5_voice_builder_skips_ph1m_and_ignores_external_memory_when_identity_not_confirmed();
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
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9502),
                TurnId(9602),
                confirmed_assertion,
                None,
                MemoryKey::new("invite_contact_tom_sms").unwrap(),
                MemoryValue::v1("+14155550100".to_string(), None).unwrap(),
                "Tom contact memory".to_string(),
            )
            .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
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
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9502),
                    turn_id: TurnId(9602),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
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
    fn at_mem_02_confirmed_identity_returns_candidates_when_present() {
        run5_voice_builder_uses_confirmed_identity_memory_context_to_resolve_tom_contact();
    }

    #[test]
    fn at_mem_03_confirmed_identity_wrong_user_never_sees_other_users_memory() {
        let runtime = AppServerIngressRuntime::default();
        let owner_user_id = UserId::new("tenant_1:run5_owner_user").unwrap();
        let owner_device_id = DeviceId::new("run5_owner_device_1").unwrap();
        let actor_user_id = UserId::new("tenant_1:run5_other_user").unwrap();
        let actor_device_id = DeviceId::new("run5_other_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &owner_user_id, &owner_device_id);
        seed_actor(&mut store, &actor_user_id, &actor_device_id);

        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9513),
                TurnId(9613),
                confirmed_voice_assertion(owner_user_id),
                None,
                MemoryKey::new("invite_contact_tom_sms").unwrap(),
                MemoryValue::v1("+14155550100".to_string(), None).unwrap(),
                "Tom contact memory".to_string(),
            )
            .unwrap();

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9514),
            TurnId(9614),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(actor_device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(mut forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
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
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9514),
                    turn_id: TurnId(9614),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        assert!(
            ph1x_request.memory_candidates.is_empty(),
            "actor should never receive another user's memory candidates"
        );
        assert_eq!(runtime.executor.debug_memory_context_lookup_count(), 1);
    }

    #[test]
    fn at_mem_04_memory_provenance_contains_session_id() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run5_provenance_user").unwrap();
        let device_id = DeviceId::new("run5_provenance_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let expected_session_id = SessionId(9515);
        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9515),
                TurnId(9615),
                confirmed_voice_assertion(actor_user_id.clone()),
                Some(expected_session_id),
                MemoryKey::new("invite_contact_tom_sms").unwrap(),
                MemoryValue::v1("+14155550188".to_string(), None).unwrap(),
                "Tom contact memory".to_string(),
            )
            .unwrap();

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9515),
            TurnId(9615),
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
        forwarded.voice_identity_assertion = confirmed_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
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
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9515),
                    turn_id: TurnId(9615),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();
        let candidate = ph1x_request
            .memory_candidates
            .iter()
            .find(|candidate| candidate.memory_key.as_str() == "invite_contact_tom_sms")
            .expect("confirmed identity should receive the seeded memory candidate");
        assert_eq!(candidate.provenance.session_id, Some(expected_session_id));
        assert_eq!(runtime.executor.debug_memory_context_lookup_count(), 1);
    }

    #[test]
    fn at_harmonize_01_confirmed_identity_and_memory_attach_to_agent_envelope() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_mem_user").unwrap();
        let device_id = DeviceId::new("harmonize_mem_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9521),
            TurnId(9621),
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
        let confirmed_assertion = confirmed_voice_assertion(actor_user_id.clone());
        forwarded.voice_identity_assertion = confirmed_assertion.clone();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);
        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9521),
                TurnId(9621),
                confirmed_assertion,
                None,
                MemoryKey::new("invite_contact_jane_sms").unwrap(),
                MemoryValue::v1("+14155550121".to_string(), None).unwrap(),
                "Jane contact memory".to_string(),
            )
            .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9521),
                    turn_id: TurnId(9621),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Verified);
        assert!(!identity_state.cluster_drift_detected);
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert!(memory_state.cloud_authoritative);
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::Eligible
        );
        assert_eq!(memory_state.candidate_count, 1);
    }

    #[test]
    fn at_harmonize_02_reauth_required_identity_attaches_recovery_and_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_reauth_user").unwrap();
        let device_id = DeviceId::new("harmonize_reauth_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9522),
            TurnId(9622),
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
        forwarded.voice_identity_assertion = reauth_required_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9522),
                    turn_id: TurnId(9622),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReauthRequired
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02c_device_claim_required_identity_attaches_step_up_and_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_device_claim_user").unwrap();
        let device_id = DeviceId::new("harmonize_device_claim_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9523),
            TurnId(9623),
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
        forwarded.voice_identity_assertion = device_claim_required_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9523),
                    turn_id: TurnId(9623),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReauthRequired
        );
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED.0
            ))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02i_profile_not_enrolled_identity_attaches_reenrollment_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_profile_not_enrolled_user").unwrap();
        let device_id = DeviceId::new("harmonize_profile_not_enrolled_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9532),
            TurnId(9632),
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
        forwarded.voice_identity_assertion = profile_not_enrolled_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9532),
                    turn_id: TurnId(9632),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReEnrollmentRequired
        );
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED.0
            ))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02j_governance_quarantine_identity_attaches_recovery_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_governance_quarantine_user").unwrap();
        let device_id = DeviceId::new("harmonize_governance_quarantine_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9533),
            TurnId(9633),
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
        let (assertion, governance_state) = governance_quarantined_confirmed_voice_assertion(
            &runtime,
            &forwarded.runtime_execution_envelope,
        );
        forwarded.voice_identity_assertion = assertion;
        forwarded.runtime_execution_envelope = forwarded
            .runtime_execution_envelope
            .with_governance_state(Some(governance_state))
            .unwrap();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9533),
                    turn_id: TurnId(9633),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::RecoveryRestricted
        );
        assert_eq!(identity_state.reason_code, None);
        let governance_state = envelope
            .governance_state
            .expect("governance state should remain attached");
        assert!(governance_state.decision_log_ref.is_some());
        assert!(governance_state
            .quarantined_subsystems
            .contains(&GOVERNED_SUBSYSTEM_IDENTITY_VOICE_ENGINE.to_string()));
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02d_low_confidence_identity_attaches_degraded_conditional_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_low_conf_user").unwrap();
        let device_id = DeviceId::new("harmonize_low_conf_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9527),
            TurnId(9627),
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
        forwarded.voice_identity_assertion = low_confidence_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9527),
                    turn_id: TurnId(9627),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Conditional);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE.0))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02h_gray_zone_margin_identity_attaches_degraded_conditional_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_gray_zone_user").unwrap();
        let device_id = DeviceId::new("harmonize_gray_zone_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9531),
            TurnId(9631),
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
        forwarded.voice_identity_assertion = gray_zone_margin_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9531),
                    turn_id: TurnId(9631),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Conditional);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN.0
            ))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02e_echo_unsafe_identity_attaches_degraded_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_echo_unsafe_user").unwrap();
        let device_id = DeviceId::new("harmonize_echo_unsafe_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9528),
            TurnId(9628),
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
        forwarded.voice_identity_assertion = echo_unsafe_voice_assertion();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9528),
                    turn_id: TurnId(9628),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE.0))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02f_no_speech_identity_attaches_degraded_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_no_speech_user").unwrap();
        let device_id = DeviceId::new("harmonize_no_speech_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9529),
            TurnId(9629),
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
        forwarded.voice_identity_assertion = no_speech_voice_assertion();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9529),
                    turn_id: TurnId(9629),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_NO_SPEECH.0))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02g_multi_speaker_identity_attaches_degraded_restricted_trust() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_multi_speaker_user").unwrap();
        let device_id = DeviceId::new("harmonize_multi_speaker_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9530),
            TurnId(9630),
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
        forwarded.voice_identity_assertion = multi_speaker_voice_assertion();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9530),
                    turn_id: TurnId(9630),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT.0
            ))
        );
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::IdentityScopeBlocked
        );
    }

    #[test]
    fn at_harmonize_02b_spoof_risk_identity_is_rejected_and_recovery_restricted() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_spoof_user").unwrap();
        let device_id = DeviceId::new("harmonize_spoof_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9524),
            TurnId(9624),
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
        forwarded.voice_identity_assertion = spoof_risk_voice_assertion(actor_user_id);
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9524),
                    turn_id: TurnId(9624),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Rejected);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::RecoveryRestricted
        );
        assert!(identity_state.step_up_required);
    }

    #[test]
    fn at_harmonize_03_governance_drift_blocks_memory_and_marks_identity_cluster_drift() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_drift_user").unwrap();
        let device_id = DeviceId::new("harmonize_drift_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        let _ = runtime
            .observe_runtime_governance_node_policy_version("node_drift", Some("2026.04.01.v1"));

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9523),
            TurnId(9623),
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
        let confirmed_assertion = confirmed_voice_assertion(actor_user_id.clone());
        forwarded.voice_identity_assertion = confirmed_assertion.clone();
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);
        runtime
            .executor
            .debug_seed_memory_candidate_for_tests(
                &mut store,
                MonotonicTimeNs(6),
                CorrelationId(9523),
                TurnId(9623),
                confirmed_assertion,
                None,
                MemoryKey::new("invite_contact_drift_sms").unwrap(),
                MemoryValue::v1("+14155550123".to_string(), None).unwrap(),
                "Drift contact memory".to_string(),
            )
            .unwrap();

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Drift", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9523),
                    turn_id: TurnId(9623),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert!(identity_state.cluster_drift_detected);
        let memory_state = envelope
            .memory_state
            .expect("memory state should be attached");
        assert_eq!(
            memory_state.eligibility_decision,
            MemoryEligibilityDecision::GovernedBlocked
        );
        assert_eq!(memory_state.candidate_count, 0);
        assert_eq!(runtime.executor.debug_memory_context_lookup_count(), 0);
    }

    #[test]
    fn at_ingress_04b_forwarded_voice_outcome_attaches_identity_state_from_canonical_runtime_governance(
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:ingress_canonical_user").unwrap();
        let device_id = DeviceId::new("ingress_canonical_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9525),
            TurnId(9625),
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
        forwarded.voice_identity_assertion = reauth_required_voice_assertion(actor_user_id);

        let expected_envelope = attach_identity_state_for_governed_voice_turn(
            &forwarded
                .runtime_execution_envelope
                .with_voice_identity_assertion(Some(forwarded.voice_identity_assertion.clone()))
                .expect("forwarded runtime envelope should allow voice-assertion reset before canonical recompute")
                .with_identity_state(None)
                .expect("forwarded runtime envelope should allow identity-state reset before canonical recompute"),
            &forwarded.voice_identity_assertion,
        )
        .expect("canonical runtime governance helper must attach identity state");
        recanonicalize_forwarded_bundle_for_tests(&mut forwarded);
        let forwarded_runtime_execution_envelope = forwarded.runtime_execution_envelope.clone();
        assert!(
            forwarded_runtime_execution_envelope.identity_state.is_some(),
            "forwarded runtime envelope must now carry canonical identity state before packet build"
        );
        assert_eq!(
            forwarded.top_level_bundle.runtime_execution_envelope,
            Some(expected_envelope.clone())
        );

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(7),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Jane", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };
        runtime
            .build_ph1x_request_for_forwarded_voice(
                &mut store,
                ForwardedVoicePh1xRequestInput {
                    correlation_id: CorrelationId(9525),
                    turn_id: TurnId(9625),
                    app_platform: AppPlatform::Desktop,
                    forwarded: &forwarded,
                    request_session_id: None,
                    tenant_id: Some("tenant_1"),
                    x_build,
                },
            )
            .unwrap();

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        let envelope = packet
            .runtime_execution_envelope
            .expect("agent packet should carry runtime execution envelope");
        assert_eq!(
            forwarded_runtime_execution_envelope.identity_state,
            expected_envelope.identity_state
        );
        assert_eq!(envelope.identity_state, expected_envelope.identity_state);
        let identity_state = envelope
            .identity_state
            .expect("identity state should be attached");
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReauthRequired
        );
    }

    #[test]
    fn at_identity_recovery_01_reauth_required_protected_voice_turn_fails_closed_with_explicit_reauth_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_reauth_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_reauth_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            reauth_required_voice_assertion(actor_user_id),
            CorrelationId(9820),
            TurnId(9920),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I need you to reauthenticate before I can continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_REAUTH_REQUIRED)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9820));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_REAUTH_REQUIRED_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_REAUTH_REQUIRED_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("RECOVERY_RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("REAUTH_REQUIRED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("true")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490007")
        );
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::StepUpRequired,
            SimulationCertificationState::StepUpRequired,
            "STEP_UP_REQUIRED",
        );
        assert_proof_authority_reason_token(
            &store,
            &out,
            &u64::from(voice_id_reason_codes::VID_REAUTH_REQUIRED.0).to_string(),
        );
    }

    #[test]
    fn at_identity_recovery_02_reenrollment_required_protected_voice_turn_fails_closed_with_explicit_reenrollment_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_reenroll_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_reenroll_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            reenrollment_required_voice_assertion(actor_user_id),
            CorrelationId(9821),
            TurnId(9921),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I need you to re-enroll your voice before I can continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_ENROLLMENT_REQUIRED)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9821));
        let row =
            find_ph1x_respond_row(&response_rows, "IDENTITY_REENROLLMENT_REQUIRED_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_REENROLLMENT_REQUIRED_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("RE_ENROLLMENT_REQUIRED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490006")
        );
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::Denied,
            SimulationCertificationState::NotRequested,
            "NOT_REQUESTED",
        );
    }

    #[test]
    fn at_identity_recovery_03_spoof_risk_protected_voice_turn_fails_closed_with_explicit_spoof_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_spoof_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_spoof_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            spoof_risk_voice_assertion(actor_user_id),
            CorrelationId(9822),
            TurnId(9922),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I detected a possible spoofing risk, so I can't continue.")
        );
        assert_eq!(out.reason_code, Some(voice_id_reason_codes::VID_SPOOF_RISK));
        let response_rows = store.ph1x_audit_rows(CorrelationId(9822));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_SPOOF_RISK_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_SPOOF_RISK_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("RECOVERY_RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("REJECTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("RECOVERY_RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("true")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_spoof_liveness_status"),
            Some("SUSPECTED_SPOOF")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490008")
        );
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::StepUpRequired,
            SimulationCertificationState::StepUpRequired,
            "STEP_UP_REQUIRED",
        );
        assert_proof_authority_reason_token(
            &store,
            &out,
            &u64::from(voice_id_reason_codes::VID_SPOOF_RISK.0).to_string(),
        );
    }

    #[test]
    fn at_identity_recovery_04_device_claim_required_protected_voice_turn_fails_closed_with_explicit_device_claim_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_device_claim_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_device_claim_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            device_claim_required_voice_assertion(actor_user_id),
            CorrelationId(9823),
            TurnId(9923),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I need you to confirm this device before I can continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9823));
        let row =
            find_ph1x_respond_row(&response_rows, "IDENTITY_DEVICE_CLAIM_REQUIRED_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_DEVICE_CLAIM_REQUIRED_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("REAUTH_REQUIRED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("true")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490009")
        );
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::StepUpRequired,
            SimulationCertificationState::StepUpRequired,
            "STEP_UP_REQUIRED",
        );
        assert_proof_authority_reason_token(
            &store,
            &out,
            &u64::from(voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED.0).to_string(),
        );
    }

    #[test]
    fn at_identity_recovery_05_profile_not_enrolled_protected_voice_turn_fails_closed_with_explicit_profile_not_enrolled_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id =
            UserId::new("tenant_1:identity_profile_not_enrolled_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_profile_not_enrolled_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            profile_not_enrolled_voice_assertion(actor_user_id),
            CorrelationId(9824),
            TurnId(9924),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I couldn't find an enrolled voice profile for you, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9824));
        let row =
            find_ph1x_respond_row(&response_rows, "IDENTITY_PROFILE_NOT_ENROLLED_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_PROFILE_NOT_ENROLLED_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("RE_ENROLLMENT_REQUIRED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490005")
        );
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::Denied,
            SimulationCertificationState::NotRequested,
            "NOT_REQUESTED",
        );
        assert_proof_authority_reason_token(&store, &out, "-");
    }

    #[test]
    fn at_identity_recovery_06_governance_quarantine_protected_voice_turn_fails_closed_with_explicit_governance_quarantine_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id =
            UserId::new("tenant_1:identity_governance_quarantine_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_governance_quarantine_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9825),
            TurnId(9925),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };
        let (assertion, governance_state) = governance_quarantined_confirmed_voice_assertion(
            &runtime,
            &forwarded.runtime_execution_envelope,
        );

        let out = run_protected_response_turn_with_identity_assertion_and_governance_state(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            assertion,
            Some(governance_state),
            CorrelationId(9825),
            TurnId(9925),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I can't continue because identity governance is quarantined right now.")
        );
        assert_eq!(out.reason_code, None);
        let governance_state = out
            .runtime_execution_envelope
            .governance_state
            .as_ref()
            .expect("governance state must remain attached");
        assert!(governance_state.decision_log_ref.is_some());
        assert_eq!(
            governance_reason_code_for_state(&runtime, governance_state).as_deref(),
            Some(crate::runtime_governance::reason_codes::GOV_SUBSYSTEM_CERTIFICATION_REGRESSED)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9825));
        let row =
            find_ph1x_respond_row(&response_rows, "IDENTITY_GOVERNANCE_QUARANTINE_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_GOVERNANCE_QUARANTINE_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("RECOVERY_RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("RECOVERY_RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("true")
        );
        assert_ph1x_payload_absent(row, "identity_reason_code_hex");
        assert_authority_state_and_proof_simulation_certification(
            &store,
            &out,
            AuthorityPolicyDecision::Denied,
            SimulationCertificationState::NotRequested,
            "NOT_REQUESTED",
        );
    }

    #[test]
    fn at_identity_drift_01_cluster_drift_protected_voice_turn_fails_closed_with_explicit_cluster_drift_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures_and_runtime_governance(
            RuntimeGovernanceRuntime::new(
                RuntimeGovernanceConfig::mvp_v1().with_policy_window(
                    selene_kernel_contracts::runtime_governance::GovernancePolicyWindow::v1(
                        "2026.03.08.v1".to_string(),
                        "2026.03.08.v1".to_string(),
                        "2026.04.01.v1".to_string(),
                    )
                    .unwrap(),
                ),
            ),
        );
        let actor_user_id = UserId::new("tenant_1:identity_cluster_drift_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_cluster_drift_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        let _ = runtime
            .observe_runtime_governance_node_policy_version("node_compat", Some("2026.04.01.v1"));

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            confirmed_voice_assertion(actor_user_id),
            CorrelationId(9824),
            TurnId(9924),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I can't continue because governance state is out of sync right now.")
        );
        assert_eq!(
            out.reason_code,
            out.ph1x_response
                .as_ref()
                .map(|response| response.reason_code)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9824));
        let row = find_ph1x_respond_row(&response_rows, "GOVERNANCE_CLUSTER_DRIFT_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "GOVERNANCE_CLUSTER_DRIFT_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_cluster_drift_detected"),
            Some("true")
        );
    }

    #[test]
    fn at_identity_drift_02_policy_version_drift_protected_voice_turn_fails_closed_with_explicit_policy_drift_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_policy_drift_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_policy_drift_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        let _ = runtime
            .observe_runtime_governance_node_policy_version("node_drift", Some("2026.04.01.v1"));

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            confirmed_voice_assertion(actor_user_id),
            CorrelationId(9825),
            TurnId(9925),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I can't continue because policy versions are out of sync right now.")
        );
        assert_eq!(
            out.reason_code,
            out.ph1x_response
                .as_ref()
                .map(|response| response.reason_code)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9825));
        let row = find_ph1x_respond_row(&response_rows, "GOVERNANCE_POLICY_DRIFT_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "GOVERNANCE_POLICY_DRIFT_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_cluster_drift_detected"),
            Some("true")
        );
    }

    #[test]
    fn at_identity_posture_01_low_confidence_protected_voice_turn_fails_closed_with_explicit_low_confidence_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_low_conf_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_low_conf_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9826),
            TurnId(9926),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I couldn't verify your identity strongly enough, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE)
        );
        let identity_state = out
            .runtime_execution_envelope
            .identity_state
            .as_ref()
            .expect("identity state must remain attached");
        let voice_identity_assertion = out
            .runtime_execution_envelope
            .voice_identity_assertion
            .as_ref()
            .expect("voice identity assertion must remain attached");
        assert_eq!(
            identity_reason_code(identity_state),
            Some(voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE)
        );
        assert_eq!(
            identity_reason_code(identity_state),
            voice_identity_reason_code(voice_identity_assertion)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9826));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_LOW_CONFIDENCE_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_LOW_CONFIDENCE_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("CONDITIONAL")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("NONE")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490002")
        );
    }

    #[test]
    fn at_identity_posture_02_echo_unsafe_protected_voice_turn_fails_closed_with_explicit_echo_unsafe_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_echo_unsafe_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_echo_unsafe_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            echo_unsafe_voice_assertion(),
            CorrelationId(9827),
            TurnId(9927),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I detected an echo-unsafe voice condition, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9827));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_ECHO_UNSAFE_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_ECHO_UNSAFE_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("NONE")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490004")
        );
    }

    #[test]
    fn at_identity_posture_03_no_speech_protected_voice_turn_fails_closed_with_explicit_no_speech_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_no_speech_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_no_speech_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            no_speech_voice_assertion(),
            CorrelationId(9828),
            TurnId(9928),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I couldn't detect speech clearly enough, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_NO_SPEECH)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9828));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_NO_SPEECH_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_NO_SPEECH_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("NONE")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490001")
        );
    }

    #[test]
    fn at_identity_posture_04_multi_speaker_protected_voice_turn_fails_closed_with_explicit_multi_speaker_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_multi_speaker_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_multi_speaker_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            multi_speaker_voice_assertion(),
            CorrelationId(9829),
            TurnId(9929),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I detected multiple speakers, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9829));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_MULTI_SPEAKER_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_MULTI_SPEAKER_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("RESTRICTED")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("NONE")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490003")
        );
    }

    #[test]
    fn at_identity_posture_05_gray_zone_margin_protected_voice_turn_fails_closed_with_explicit_gray_zone_margin_response(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:identity_gray_zone_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_gray_zone_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let out = run_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            gray_zone_margin_voice_assertion(actor_user_id),
            CorrelationId(9830),
            TurnId(9930),
        );

        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I got an ambiguous identity result, so I can't continue.")
        );
        assert_eq!(
            out.reason_code,
            Some(voice_id_reason_codes::VID_FAIL_GRAY_ZONE_MARGIN)
        );
        let response_rows = store.ph1x_audit_rows(CorrelationId(9830));
        let row = find_ph1x_respond_row(&response_rows, "IDENTITY_GRAY_ZONE_MARGIN_FAIL_CLOSED");
        assert_ph1x_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "IDENTITY_GRAY_ZONE_MARGIN_FAIL_CLOSED",
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_trust_tier"),
            Some("CONDITIONAL")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_recovery_state"),
            Some("NONE")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_step_up_required"),
            Some("false")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x5649000A")
        );
    }

    #[test]
    fn at_identity_posture_06_low_confidence_protected_voice_turn_fails_closed_when_outcome_lacks_canonical_identity_state(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id =
            UserId::new("tenant_1:identity_low_conf_missing_state_runtime_user").unwrap();
        let device_id = DeviceId::new("identity_low_conf_missing_state_runtime_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9831),
            TurnId(9931),
        );

        assert_posture_finalization_requires_canonical_identity_state(
            &runtime, &mut store, pending,
        );
    }

    #[test]
    fn at_identity_posture_07_multi_speaker_protected_voice_turn_fails_closed_when_outcome_lacks_canonical_identity_state(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_multi_speaker_missing_state").unwrap();
        let device_id = DeviceId::new("id_multi_speaker_missing_state_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            multi_speaker_voice_assertion(),
            CorrelationId(9832),
            TurnId(9932),
        );

        assert_posture_finalization_requires_canonical_identity_state(
            &runtime, &mut store, pending,
        );
    }

    #[test]
    fn at_identity_posture_08_low_confidence_protected_voice_turn_fails_closed_when_identity_state_reason_code_diverges_from_canonical_voice_assertion(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_low_conf_reason_diverge").unwrap();
        let device_id = DeviceId::new("id_low_conf_reason_diverge_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9833),
            TurnId(9933),
        );

        assert_posture_finalization_requires_reason_alignment(
            &runtime,
            &mut store,
            pending,
            voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE,
        );
    }

    #[test]
    fn at_identity_posture_09_echo_unsafe_protected_voice_turn_fails_closed_when_identity_state_reason_code_diverges_from_canonical_voice_assertion(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_echo_reason_diverge").unwrap();
        let device_id = DeviceId::new("id_echo_reason_diverge_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            echo_unsafe_voice_assertion(),
            CorrelationId(9834),
            TurnId(9934),
        );

        assert_posture_finalization_requires_reason_alignment(
            &runtime,
            &mut store,
            pending,
            voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE,
        );
    }

    #[test]
    fn at_identity_posture_10_low_confidence_protected_voice_turn_fails_closed_when_outcome_lacks_canonical_voice_identity_assertion(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_low_conf_missing_assertion").unwrap();
        let device_id = DeviceId::new("id_low_conf_missing_assertion_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9835),
            TurnId(9935),
        );

        assert_posture_finalization_requires_canonical_voice_identity_assertion(
            &runtime,
            &mut store,
            pending,
        );
    }

    #[test]
    fn at_identity_posture_11_multi_speaker_protected_voice_turn_fails_closed_when_outcome_lacks_canonical_voice_identity_assertion(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_multi_missing_assertion").unwrap();
        let device_id = DeviceId::new("id_multi_missing_assertion_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            multi_speaker_voice_assertion(),
            CorrelationId(9836),
            TurnId(9936),
        );

        assert_posture_finalization_requires_canonical_voice_identity_assertion(
            &runtime,
            &mut store,
            pending,
        );
    }

    #[test]
    fn at_identity_posture_12_low_confidence_protected_voice_turn_fails_closed_when_voice_assertion_reason_code_is_not_posture_family(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_low_conf_non_posture_reason").unwrap();
        let device_id = DeviceId::new("id_low_conf_non_posture_reason_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9837),
            TurnId(9937),
        );

        assert_posture_finalization_requires_canonical_voice_assertion_posture_reason(
            &runtime,
            &mut store,
            pending,
            voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED,
        );
    }

    #[test]
    fn at_identity_posture_13_echo_unsafe_protected_voice_turn_fails_closed_when_voice_assertion_reason_code_is_not_posture_family(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_echo_non_posture_reason").unwrap();
        let device_id = DeviceId::new("id_echo_non_posture_reason_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            echo_unsafe_voice_assertion(),
            CorrelationId(9838),
            TurnId(9938),
        );

        assert_posture_finalization_requires_canonical_voice_assertion_posture_reason(
            &runtime,
            &mut store,
            pending,
            voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED,
        );
    }

    #[test]
    fn at_identity_posture_14_low_confidence_protected_voice_turn_fails_closed_when_identity_state_shape_is_not_canonical_for_posture_family(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_low_conf_bad_shape").unwrap();
        let device_id = DeviceId::new("id_low_conf_bad_shape_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id.clone(),
            device_id,
            low_confidence_voice_assertion(actor_user_id),
            CorrelationId(9839),
            TurnId(9939),
        );

        assert_posture_finalization_requires_canonical_identity_state_shape(
            &runtime,
            &mut store,
            pending,
            IdentityTrustTier::Restricted,
        );
    }

    #[test]
    fn at_identity_posture_15_echo_unsafe_protected_voice_turn_fails_closed_when_identity_state_shape_is_not_canonical_for_posture_family(
    ) {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:id_echo_bad_shape").unwrap();
        let device_id = DeviceId::new("id_echo_bad_shape_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let pending = prepare_protected_chat_response_turn_with_identity_assertion(
            &runtime,
            &mut store,
            actor_user_id,
            device_id,
            echo_unsafe_voice_assertion(),
            CorrelationId(9840),
            TurnId(9940),
        );

        assert_posture_finalization_requires_canonical_identity_state_shape(
            &runtime,
            &mut store,
            pending,
            IdentityTrustTier::Conditional,
        );
    }

    #[test]
    fn at_agentpkt_01_packet_contains_all_required_fields() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:agentpkt_user").unwrap();
        let device_id = DeviceId::new("agentpkt_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

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
            now: MonotonicTimeNs(10),
            thread_key: Some("agentpkt_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(Ph1nResponse::Chat(
                Chat::v1("agent packet test".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let (_outcome, ph1x_request) = runtime
            .run_voice_turn_and_build_ph1x_request(&mut store, request, x_build)
            .unwrap();
        assert!(ph1x_request.is_some());

        let packet = runtime
            .debug_last_agent_input_packet()
            .expect("agent packet should be captured");
        assert_eq!(packet.correlation_id, 9601);
        assert_eq!(packet.turn_id, 9701);
        assert_eq!(packet.thread_key.as_deref(), Some("agentpkt_thread"));
        assert_eq!(packet.session_state, SessionState::Active);
        assert!(packet.session_id.is_some());
        assert!(!packet.trace_id.is_empty());
        assert!(!packet.packet_hash.is_empty());
        assert!(!packet.sim_catalog_snapshot_hash.is_empty());
        assert!(packet.sim_catalog_snapshot_version > 0);
        assert_eq!(runtime.debug_agent_input_packet_build_count(), 1);
    }

    #[test]
    fn at_agentpkt_02_packet_hash_stable_for_same_inputs() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:agentpkt_hash_user").unwrap();
        let device_id = DeviceId::new("agentpkt_hash_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

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
        let request_session_id = request.voice_id_request.session_state_ref.session_id;
        let outcome = runtime.run_voice_turn(&mut store, request).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded voice turn");
        };

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(11),
            thread_key: Some("agentpkt_hash_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(Ph1nResponse::Chat(
                Chat::v1("same deterministic input".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let packet_a = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9602),
                TurnId(9702),
                &forwarded,
                request_session_id,
                Some("tenant_1"),
                x_build.clone(),
            )
            .unwrap();
        let packet_b = runtime
            .build_agent_input_packet_for_forwarded_voice(
                &mut store,
                CorrelationId(9602),
                TurnId(9702),
                &forwarded,
                request_session_id,
                Some("tenant_1"),
                x_build,
            )
            .unwrap();
        assert_eq!(packet_a.packet_hash, packet_b.packet_hash);
    }

    #[test]
    fn at_agentpkt_03_packet_built_once_per_turn() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:agentpkt_once_user").unwrap();
        let device_id = DeviceId::new("agentpkt_once_device_1").unwrap();
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
            now: MonotonicTimeNs(12),
            thread_key: Some("agentpkt_once_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(Ph1nResponse::Chat(
                Chat::v1("single build test".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let before = runtime.debug_agent_input_packet_build_count();
        let (_outcome, ph1x_request) = runtime
            .run_voice_turn_and_build_ph1x_request(&mut store, request, x_build)
            .unwrap();
        let after = runtime.debug_agent_input_packet_build_count();
        assert!(ph1x_request.is_some());
        assert_eq!(after, before + 1);
    }

    #[test]
    fn at_emo_01_persona_changes_only_style_not_sim_candidate() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:emo_style_user").unwrap();
        let device_id = DeviceId::new("emo_style_device_1").unwrap();
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

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(41),
            thread_key: None,
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

        let request_plain = AppVoiceIngressRequest::v1(
            CorrelationId(9691),
            TurnId(9791),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let out_plain = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_plain, x_build.clone())
            .unwrap();

        seed_persona_profile_for_actor(
            &mut store,
            &actor_user_id,
            &device_id,
            "tenant_1",
            "gentle",
            "at_emo_01_persona_profile",
        );

        let request_persona = AppVoiceIngressRequest::v1(
            CorrelationId(9692),
            TurnId(9792),
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
        let out_persona = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_persona, x_build)
            .unwrap();

        let dispatch_plain = match out_plain
            .ph1x_response
            .clone()
            .expect("plain run should include PH1.X response")
            .directive
        {
            Ph1xDirective::Dispatch(dispatch) => dispatch.dispatch_request,
            _ => panic!("plain run should dispatch simulation candidate"),
        };
        let dispatch_persona = match out_persona
            .ph1x_response
            .clone()
            .expect("persona run should include PH1.X response")
            .directive
        {
            Ph1xDirective::Dispatch(dispatch) => dispatch.dispatch_request,
            _ => panic!("persona run should dispatch simulation candidate"),
        };

        assert_eq!(dispatch_plain, dispatch_persona);
        assert_eq!(out_plain.reason_code, out_persona.reason_code);
        assert_eq!(out_plain.next_move, AppVoiceTurnNextMove::Dispatch);
        assert_eq!(out_persona.next_move, AppVoiceTurnNextMove::Dispatch);
        assert!(matches!(
            &out_plain.dispatch_outcome,
            Some(SimulationDispatchOutcome::LinkDelivered { .. })
        ));
        assert!(matches!(
            &out_persona.dispatch_outcome,
            Some(SimulationDispatchOutcome::LinkDelivered { .. })
        ));
        assert_eq!(out_plain.response_text.as_deref(), Some("I sent the link."));
        assert_eq!(
            out_persona.response_text.as_deref(),
            Some("Certainly. I sent the link.")
        );
    }

    #[test]
    fn at_emo_02_persona_cannot_bypass_access_or_active_sim_checks() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:emo_guard_user").unwrap();
        let device_id = DeviceId::new("emo_guard_device_1").unwrap();
        let mut access_store = Ph1fStore::new_in_memory();
        seed_actor(&mut access_store, &actor_user_id, &device_id);
        seed_link_send_denied_access_instance(&mut access_store, &actor_user_id, "tenant_1");
        for (simulation_id, simulation_type) in [
            (LINK_INVITE_GENERATE_DRAFT, SimulationType::Draft),
            (BCAST_CREATE_DRAFT, SimulationType::Draft),
            (BCAST_DELIVER_COMMIT, SimulationType::Commit),
            (DELIVERY_SEND_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut access_store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(51),
            thread_key: None,
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

        let access_out_plain = runtime
            .run_desktop_voice_turn_end_to_end(
                &mut access_store,
                AppVoiceIngressRequest::v1(
                    CorrelationId(9693),
                    TurnId(9793),
                    AppPlatform::Desktop,
                    OsVoiceTrigger::Explicit,
                    sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
                    actor_user_id.clone(),
                    Some("tenant_1".to_string()),
                    Some(device_id.clone()),
                    Vec::new(),
                    no_observation(),
                )
                .unwrap(),
                x_build.clone(),
            )
            .expect("missing access grant must return deterministic fail-closed response");
        assert_eq!(access_out_plain.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            access_out_plain.reason_code,
            Some(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED)
        );

        seed_persona_profile_for_actor(
            &mut access_store,
            &actor_user_id,
            &device_id,
            "tenant_1",
            "dominant",
            "at_emo_02_access_guard_persona",
        );
        let access_out_persona = runtime
            .run_desktop_voice_turn_end_to_end(
                &mut access_store,
                AppVoiceIngressRequest::v1(
                    CorrelationId(9694),
                    TurnId(9794),
                    AppPlatform::Desktop,
                    OsVoiceTrigger::Explicit,
                    sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
                    actor_user_id.clone(),
                    Some("tenant_1".to_string()),
                    Some(device_id.clone()),
                    Vec::new(),
                    no_observation(),
                )
                .unwrap(),
                x_build.clone(),
            )
            .expect("persona hint must not bypass access guard");
        assert_eq!(access_out_persona.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            access_out_persona.reason_code,
            Some(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED)
        );

        let mut inactive_store = Ph1fStore::new_in_memory();
        seed_actor(&mut inactive_store, &actor_user_id, &device_id);
        seed_link_send_access_instance(&mut inactive_store, &actor_user_id, "tenant_1");
        for (simulation_id, simulation_type) in [
            (LINK_INVITE_GENERATE_DRAFT, SimulationType::Draft),
            (BCAST_CREATE_DRAFT, SimulationType::Draft),
            (BCAST_DELIVER_COMMIT, SimulationType::Commit),
            (DELIVERY_SEND_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut inactive_store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Disabled,
            );
        }

        let active_err_plain = runtime
            .run_desktop_voice_turn_end_to_end(
                &mut inactive_store,
                AppVoiceIngressRequest::v1(
                    CorrelationId(9695),
                    TurnId(9795),
                    AppPlatform::Desktop,
                    OsVoiceTrigger::Explicit,
                    sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
                    actor_user_id.clone(),
                    Some("tenant_1".to_string()),
                    Some(device_id.clone()),
                    Vec::new(),
                    no_observation(),
                )
                .unwrap(),
                x_build.clone(),
            )
            .expect_err("inactive simulations must fail closed");
        match active_err_plain {
            StorageError::ContractViolation(ContractViolation::InvalidValue { reason, .. }) => {
                assert_eq!(reason, "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE");
            }
            other => panic!(
                "expected contract violation SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE, got {other:?}"
            ),
        }

        seed_persona_profile_for_actor(
            &mut inactive_store,
            &actor_user_id,
            &device_id,
            "tenant_1",
            "gentle",
            "at_emo_02_active_guard_persona",
        );
        let active_err_persona = runtime
            .run_desktop_voice_turn_end_to_end(
                &mut inactive_store,
                AppVoiceIngressRequest::v1(
                    CorrelationId(9696),
                    TurnId(9796),
                    AppPlatform::Desktop,
                    OsVoiceTrigger::Explicit,
                    sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
                    actor_user_id.clone(),
                    Some("tenant_1".to_string()),
                    Some(device_id),
                    Vec::new(),
                    no_observation(),
                )
                .unwrap(),
                x_build,
            )
            .expect_err("persona hint must not bypass active simulation guard");
        match active_err_persona {
            StorageError::ContractViolation(ContractViolation::InvalidValue { reason, .. }) => {
                assert_eq!(reason, "SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE");
            }
            other => panic!(
                "expected contract violation SIM_DISPATCH_GUARD_SIMULATION_NOT_ACTIVE, got {other:?}"
            ),
        }
    }

    #[test]
    fn at_finder_exec_01_broken_english_match_confirm_then_dispatch() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_match_user").unwrap();
        let device_id = DeviceId::new("finder_exec_match_device_1").unwrap();
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

        let request_1 = AppVoiceIngressRequest::v1(
            CorrelationId(9801),
            TurnId(9901),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build_1 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(20),
            thread_key: Some("finder_exec_01_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_1, x_build_1)
            .unwrap();
        assert_eq!(out_1.next_move, AppVoiceTurnNextMove::Confirm);
        let agent_rows = store.agent_execution_ledger_rows_for_correlation(CorrelationId(9801));
        assert_eq!(agent_rows.len(), 1);
        assert_eq!(agent_rows[0].finder_packet_kind, "SIMULATION_MATCH");
        assert_eq!(agent_rows[0].execution_stage, "MATCH_CONFIRM");
        assert_eq!(
            agent_rows[0].simulation_id.as_deref(),
            Some(LINK_INVITE_GENERATE_DRAFT)
        );
        assert_eq!(agent_rows[0].access_decision, "PENDING");
        assert_eq!(agent_rows[0].confirm_decision, "REQUIRED_PENDING");
        assert!(agent_rows[0].active_simulation_proof_ref.is_some());
        assert!(agent_rows[0].simulation_idempotency_key.is_some());
        assert!(agent_rows[0].dispatch_outcome_proof_ref.is_some());
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::SimulationMatch(packet) => {
                assert_eq!(packet.simulation_id, LINK_INVITE_GENERATE_DRAFT);
            }
            other => panic!("expected simulation match packet, got {other:?}"),
        }

        let thread_state_after_confirm = out_1
            .ph1x_response
            .expect("confirm run should include ph1x response")
            .thread_state;
        let request_2 = AppVoiceIngressRequest::v1(
            CorrelationId(9802),
            TurnId(9902),
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
        let x_build_2 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(21),
            thread_key: Some("finder_exec_01_thread".to_string()),
            thread_state: thread_state_after_confirm,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_2 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_2, x_build_2)
            .unwrap();
        assert_eq!(out_2.next_move, AppVoiceTurnNextMove::Dispatch);
        assert!(matches!(
            out_2.dispatch_outcome,
            Some(SimulationDispatchOutcome::LinkDelivered { .. })
        ));
        let proof_state = out_2
            .runtime_execution_envelope
            .proof_state
            .as_ref()
            .expect("dispatch outcome must attach proof state");
        assert!(proof_state.proof_record_ref.is_some());
        let law_state = out_2
            .runtime_execution_envelope
            .law_state
            .as_ref()
            .expect("dispatch outcome must attach final runtime law state");
        assert_eq!(
            law_state.protected_action_class,
            RuntimeProtectedActionClass::ProofRequired
        );
        assert!(matches!(
            law_state.final_law_response_class,
            RuntimeLawResponseClass::Allow
                | RuntimeLawResponseClass::AllowWithWarning
                | RuntimeLawResponseClass::Degrade
        ));
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out_2.runtime_execution_envelope.request_id, 4)
            .unwrap();
        assert_eq!(proof_rows.len(), 1);
        assert_eq!(
            proof_rows[0]
                .simulation_id
                .as_ref()
                .map(|value| value.as_str()),
            Some(LINK_INVITE_GENERATE_DRAFT)
        );
        assert_eq!(proof_rows[0].simulation_version, Some(SimulationVersion(1)));
        assert_eq!(
            proof_rows[0].policy_version.as_deref(),
            Some(runtime.runtime_governance_policy_version())
        );
        assert_eq!(
            proof_rows[0].turn_id,
            Some(out_2.runtime_execution_envelope.turn_id)
        );
        // Confirm replay turn does not rebuild a new finder packet; execution proof row count
        // remains tied to the finder-bearing turn.
        let agent_rows = store.agent_execution_ledger_rows_for_correlation(CorrelationId(9801));
        assert_eq!(agent_rows.len(), 1);
    }

    #[test]
    fn at_finder_exec_02_missing_field_returns_single_clarify_question() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_clarify_user").unwrap();
        let device_id = DeviceId::new("finder_exec_clarify_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9803),
            TurnId(9903),
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
            now: MonotonicTimeNs(22),
            thread_key: Some("finder_exec_02_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_draft_missing_contact("Tom", "tenant_1")),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Clarify);
        assert!(out.ph1x_response.is_none());
        assert!(out
            .response_text
            .as_deref()
            .unwrap_or_default()
            .contains("contact"));
        let rows = store.agent_execution_ledger_rows_for_correlation(CorrelationId(9803));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].finder_packet_kind, "CLARIFY");
        assert_eq!(rows[0].execution_stage, "CLARIFY_QUESTION");
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::Clarify(packet) => {
                assert_eq!(packet.attempt_index, 1);
                assert_eq!(packet.max_attempts, 2);
            }
            other => panic!("expected clarify packet, got {other:?}"),
        }
    }

    #[test]
    fn at_ph1j_01_proof_write_failure_blocks_protected_voice_turn() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:proof_block_user").unwrap();
        let device_id = DeviceId::new("proof_block_device_1").unwrap();
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

        let request_1 = AppVoiceIngressRequest::v1(
            CorrelationId(98_120),
            TurnId(99_120),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build_1 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(22),
            thread_key: Some("proof_block_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_1, x_build_1)
            .expect("confirm path should succeed before proof failure is injected");
        assert_eq!(out_1.next_move, AppVoiceTurnNextMove::Confirm);
        runtime
            .ph1j_runtime()
            .force_failure_for_tests(Some(ProofFailureClass::ProofStorageUnavailable));

        let request_2 = AppVoiceIngressRequest::v1(
            CorrelationId(98_121),
            TurnId(99_121),
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
        let x_build_2 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(23),
            thread_key: Some("proof_block_thread".to_string()),
            thread_state: out_1
                .ph1x_response
                .expect("confirm run should include ph1x response")
                .thread_state,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };

        let err = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_2, x_build_2)
            .expect_err("proof failure must block protected voice completion");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_turn_execution_outcome.runtime_execution_envelope.law_state"
                );
                assert_eq!(reason, "runtime_law_block final_runtime_law_block");
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runtime
            .runtime_law_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry
                    .reason_codes
                    .contains(&crate::runtime_law::reason_codes::LAW_PROOF_REQUIRED.to_string())
                    && entry.turn_id == Some(99_121)
            }));
        assert!(runtime
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code == crate::runtime_governance::reason_codes::GOV_PROOF_REQUIRED
                    && entry.turn_id == Some(99_121)
            }));
    }

    fn run_missing_sim_invite_link_turn() -> (
        AppServerIngressRuntime,
        Ph1fStore,
        AppVoiceTurnExecutionOutcome,
        CorrelationId,
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_missing_user").unwrap();
        let device_id = DeviceId::new("finder_exec_missing_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_link_send_access_instance(&mut store, &actor_user_id, "tenant_1");

        let correlation_id = CorrelationId(9804);
        let request = AppVoiceIngressRequest::v1(
            correlation_id,
            TurnId(9904),
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
            now: MonotonicTimeNs(23),
            thread_key: Some("finder_exec_03_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        (runtime, store, out, correlation_id)
    }

    fn run_missing_sim_unsupported_intent_turn() -> (
        AppServerIngressRuntime,
        Ph1fStore,
        AppVoiceTurnExecutionOutcome,
        CorrelationId,
    ) {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_pizza_user").unwrap();
        let device_id = DeviceId::new("finder_exec_pizza_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let correlation_id = CorrelationId(9809);
        let request = AppVoiceIngressRequest::v1(
            correlation_id,
            TurnId(9909),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id),
            UserId::new("tenant_1:finder_exec_pizza_user").unwrap(),
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(28),
            thread_key: Some("finder_exec_08_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(order_pizza_draft()),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        (runtime, store, out, correlation_id)
    }

    #[test]
    fn at_finder_exec_03_missing_sim_routes_to_dev_intake_ledger_row() {
        let (runtime, store, out, correlation_id) = run_missing_sim_invite_link_turn();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I can't do that yet; I've submitted it for review.")
        );
        let computation_state = out
            .runtime_execution_envelope
            .computation_state
            .as_ref()
            .expect("missing simulation path must attach computation state");
        assert_eq!(
            computation_state.formula_version_refs,
            vec!["ph1.comp.missing_simulation.v1".to_string()]
        );
        assert!(computation_state.selected_result.is_some());
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::MissingSimulation(_) => {}
            other => panic!("expected missing simulation packet, got {other:?}"),
        }
        let rows = store.ph1simfinder_dev_intake_rows(correlation_id);
        assert_eq!(rows.len(), 1);
        let dedupe_fingerprint = rows[0]
            .payload_min
            .entries
            .get(&PayloadKey::new("dedupe_fingerprint").unwrap())
            .expect("dedupe_fingerprint payload key must exist")
            .as_str()
            .to_string();
        assert!(dedupe_fingerprint.starts_with("tenant_1:"));
        let worthiness_score = rows[0]
            .payload_min
            .entries
            .get(&PayloadKey::new("worthiness_score_bp").unwrap())
            .expect("worthiness_score_bp payload key must exist")
            .as_str()
            .to_string();
        assert!(!worthiness_score.is_empty());
        let requester_user_id = rows[0]
            .payload_min
            .entries
            .get(&PayloadKey::new("requester_user_id").unwrap())
            .expect("requester_user_id payload key must exist")
            .as_str()
            .to_string();
        assert_eq!(requester_user_id, "tenant_1:finder_exec_missing_user");
        let notify_rows = store.ph1x_audit_rows(correlation_id);
        let row = find_ph1x_respond_row(&notify_rows, "MISSING_SIMULATION_NOTIFY_SUBMITTED");
        assert_eq!(
            row.reason_code,
            out.reason_code
                .expect("missing simulation path must preserve reason code")
        );
        let agent_rows = store.agent_execution_ledger_rows_for_correlation(correlation_id);
        assert_eq!(agent_rows.len(), 1);
        assert_eq!(agent_rows[0].finder_packet_kind, "MISSING_SIMULATION");
        assert_eq!(agent_rows[0].execution_stage, "MISSING_SIM_DEV_INTAKE");
        assert!(agent_rows[0].dev_intake_audit_event_id.is_some());
    }

    #[test]
    fn at_finder_exec_04_inactive_sim_returns_refuse_with_proof_trace() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_inactive_user").unwrap();
        let device_id = DeviceId::new("finder_exec_inactive_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Draft,
        );

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9805),
            TurnId(9905),
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
            now: MonotonicTimeNs(24),
            thread_key: Some("finder_exec_04_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::Refuse(packet) => {
                assert_eq!(
                    packet.reason_code,
                    selene_kernel_contracts::ph1simfinder::reason_codes::SIM_FINDER_SIMULATION_INACTIVE
                );
                assert!(
                    packet
                        .evidence_refs
                        .iter()
                        .any(|proof| proof.starts_with("catalog.inactive.hit:")),
                    "inactive proofs must be present"
                );
            }
            other => panic!("expected refuse packet, got {other:?}"),
        }
    }

    #[test]
    fn at_harmonize_04_authority_state_marks_certified_active_dispatch() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_authority_user").unwrap();
        let device_id = DeviceId::new("harmonize_authority_device_1").unwrap();
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

        let request_1 = AppVoiceIngressRequest::v1(
            CorrelationId(9851),
            TurnId(9951),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id.clone()),
            actor_user_id.clone(),
            Some("tenant_1".to_string()),
            Some(device_id.clone()),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build_1 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(20),
            thread_key: Some("harmonize_authority_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_1, x_build_1)
            .unwrap();
        let thread_state_after_confirm = out_1
            .ph1x_response
            .expect("confirm turn should include ph1x response")
            .thread_state;

        let request_2 = AppVoiceIngressRequest::v1(
            CorrelationId(9852),
            TurnId(9952),
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
        let x_build_2 = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(21),
            thread_key: Some("harmonize_authority_thread".to_string()),
            thread_state: thread_state_after_confirm,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: Some(ConfirmAnswer::Yes),
            nlp_output: None,
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_2 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request_2, x_build_2)
            .unwrap();
        let authority_state = out_2
            .runtime_execution_envelope
            .authority_state
            .as_ref()
            .expect("authority state should be attached");
        assert_eq!(
            authority_state.simulation_certification_state,
            SimulationCertificationState::CertifiedActive
        );
        assert_eq!(
            authority_state.policy_decision,
            AuthorityPolicyDecision::Allowed
        );
        assert_eq!(
            authority_state.onboarding_readiness_state,
            OnboardingReadinessState::NotApplicable
        );
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out_2.runtime_execution_envelope.request_id, 4)
            .expect("proof rows should be readable");
        assert_eq!(proof_rows.len(), 1);
        assert_eq!(
            proof_rows[0].simulation_certification_state.as_deref(),
            Some("CERTIFIED_ACTIVE")
        );
        assert_proof_authority_reason_token(
            &store,
            &out_2,
            &u64::from(crate::ph1x::reason_codes::X_CONFIRM_YES_DISPATCH.0).to_string(),
        );
    }

    #[test]
    fn at_harmonize_05_missing_simulation_attaches_denied_authority_state() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:harmonize_missing_user").unwrap();
        let device_id = DeviceId::new("harmonize_missing_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_link_send_access_instance(&mut store, &actor_user_id, "tenant_1");

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9853),
            TurnId(9953),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(3), actor_user_id),
            UserId::new("tenant_1:harmonize_missing_user").unwrap(),
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();
        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(23),
            thread_key: Some("harmonize_missing_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(invite_link_send_draft_broken_english(
                "Tom",
                "+14155550100",
                "tenant_1",
            )),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        let authority_state = out
            .runtime_execution_envelope
            .authority_state
            .expect("authority state should be attached");
        assert_eq!(
            authority_state.simulation_certification_state,
            SimulationCertificationState::MissingSimulationPath
        );
        assert_eq!(
            authority_state.policy_decision,
            AuthorityPolicyDecision::Denied
        );
    }

    #[test]
    fn at_finder_exec_05_persona_changes_tone_only_not_sim_selection() {
        at_emo_01_persona_changes_only_style_not_sim_candidate();
    }

    #[test]
    fn at_finder_exec_06_access_escalate_fails_closed_ap_required() {
        let (runtime, store, out_2, response_correlation_id) =
            run_access_ap_required_fail_closed_turn();
        assert_eq!(out_2.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out_2.response_text.as_deref(),
            Some("I need approval before I can do that.")
        );
        assert_eq!(
            out_2.reason_code,
            Some(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED)
        );
        assert!(out_2.ph1x_response.is_none());
        let ap_rows = store.ph1x_audit_rows(response_correlation_id);
        let row = find_ph1x_respond_row(&ap_rows, "ACCESS_AP_REQUIRED_FAIL_CLOSED");
        assert_eq!(
            row.reason_code,
            sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED
        );
        assert_access_fail_closed_respond_payload(
            &runtime,
            row,
            &out_2,
            "ACCESS_AP_REQUIRED_FAIL_CLOSED",
        );
        let authority_state = out_2
            .runtime_execution_envelope
            .authority_state
            .as_ref()
            .expect("authority state should be attached");
        assert_eq!(
            authority_state.policy_decision,
            AuthorityPolicyDecision::StepUpRequired
        );
        assert_eq!(
            authority_state.simulation_certification_state,
            SimulationCertificationState::StepUpRequired
        );
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out_2.runtime_execution_envelope.request_id, 4)
            .expect("proof rows should be readable");
        assert_eq!(proof_rows.len(), 1);
        assert_eq!(
            proof_rows[0].simulation_certification_state.as_deref(),
            Some("STEP_UP_REQUIRED")
        );
        assert_proof_authority_reason_token(
            &store,
            &out_2,
            &u64::from(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED.0).to_string(),
        );
    }

    #[test]
    fn at_finder_exec_07_idempotent_replay_produces_no_duplicate_execution_rows() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:finder_exec_idem_user").unwrap();
        let device_id = DeviceId::new("finder_exec_idem_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9808),
            TurnId(9908),
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
            now: MonotonicTimeNs(27),
            thread_key: Some("finder_exec_07_thread".to_string()),
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(order_pizza_draft()),
            tool_response: None,
            interruption: None,
            locale: Some("en-US".to_string()),
            last_failure_reason_code: None,
        };
        let out_1 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request.clone(), x_build.clone())
            .unwrap();
        let out_2 = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out_1.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(out_2.next_move, AppVoiceTurnNextMove::Refused);

        let dev_rows = store.ph1simfinder_dev_intake_rows(CorrelationId(9808));
        assert_eq!(dev_rows.len(), 1);
        let worthiness_score = dev_rows[0]
            .payload_min
            .entries
            .get(&PayloadKey::new("worthiness_score_bp").unwrap())
            .expect("worthiness score payload key must exist")
            .as_str()
            .to_string();
        assert!(!worthiness_score.is_empty());
        let agent_rows = store.agent_execution_ledger_rows_for_correlation(CorrelationId(9808));
        assert_eq!(agent_rows.len(), 1);
        assert_eq!(agent_rows[0].finder_packet_kind, "MISSING_SIMULATION");
    }

    #[test]
    fn at_finder_exec_08_order_pizza_routes_to_missing_simulation_dev_intake() {
        let (runtime, store, out, correlation_id) = run_missing_sim_unsupported_intent_turn();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out.response_text.as_deref(),
            Some("I can't do that yet; I've submitted it for review.")
        );
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::MissingSimulation(packet) => {
                assert!(packet
                    .requested_capability_name_normalized
                    .contains("booktable"));
            }
            other => panic!("expected missing simulation packet, got {other:?}"),
        }
        let dev_rows = store.ph1simfinder_dev_intake_rows(correlation_id);
        assert_eq!(dev_rows.len(), 1);
        let capability_name = dev_rows[0]
            .payload_min
            .entries
            .get(&PayloadKey::new("capability_name").unwrap())
            .expect("capability_name payload key must exist")
            .as_str()
            .to_string();
        assert!(capability_name.contains("booktable"));
        let notify_rows = store.ph1x_audit_rows(correlation_id);
        let row = find_ph1x_respond_row(&notify_rows, "MISSING_SIMULATION_NOTIFY_SUBMITTED");
        assert_eq!(
            row.reason_code,
            out.reason_code
                .expect("missing simulation path must preserve reason code")
        );
    }

    #[test]
    fn at_finder_exec_09_access_deny_fails_closed_deterministically() {
        let (runtime, store, out_2, response_correlation_id) = run_access_denied_fail_closed_turn();
        assert_eq!(out_2.next_move, AppVoiceTurnNextMove::Refused);
        assert_eq!(
            out_2.response_text.as_deref(),
            Some("I can't proceed because your access policy blocks this action.")
        );
        assert_eq!(
            out_2.reason_code,
            Some(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED)
        );
        assert!(out_2.ph1x_response.is_none());
        let deny_rows = store.ph1x_audit_rows(response_correlation_id);
        let row = find_ph1x_respond_row(&deny_rows, "ACCESS_DENIED_FAIL_CLOSED");
        assert_eq!(
            row.reason_code,
            sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED
        );
        assert_access_fail_closed_respond_payload(
            &runtime,
            row,
            &out_2,
            "ACCESS_DENIED_FAIL_CLOSED",
        );
        let authority_state = out_2
            .runtime_execution_envelope
            .authority_state
            .as_ref()
            .expect("authority state should be attached");
        assert_eq!(
            authority_state.policy_decision,
            AuthorityPolicyDecision::Denied
        );
        assert_eq!(
            authority_state.simulation_certification_state,
            SimulationCertificationState::NotRequested
        );
        let proof_rows = store
            .proof_records_by_request_id_bounded(&out_2.runtime_execution_envelope.request_id, 4)
            .expect("proof rows should be readable");
        assert_eq!(proof_rows.len(), 1);
        assert_eq!(
            proof_rows[0].simulation_certification_state.as_deref(),
            Some("NOT_REQUESTED")
        );
        assert_proof_authority_reason_token(
            &store,
            &out_2,
            &u64::from(sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED.0).to_string(),
        );
    }

    #[test]
    fn at_finder_exec_10_access_ap_required_fail_closed_audit_surfaces_identity_context() {
        let (runtime, store, out, response_correlation_id) =
            run_access_ap_required_fail_closed_turn();
        let response_rows = store.ph1x_audit_rows(response_correlation_id);
        let row = find_ph1x_respond_row(&response_rows, "ACCESS_AP_REQUIRED_FAIL_CLOSED");

        assert_access_fail_closed_respond_payload(
            &runtime,
            row,
            &out,
            "ACCESS_AP_REQUIRED_FAIL_CLOSED",
        );
        assert_eq!(
            row.reason_code,
            sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_AP_REQUIRED
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490002")
        );
    }

    #[test]
    fn at_finder_exec_11_access_denied_fail_closed_audit_surfaces_identity_context() {
        let (runtime, store, out, response_correlation_id) = run_access_denied_fail_closed_turn();
        let response_rows = store.ph1x_audit_rows(response_correlation_id);
        let row = find_ph1x_respond_row(&response_rows, "ACCESS_DENIED_FAIL_CLOSED");

        assert_access_fail_closed_respond_payload(&runtime, row, &out, "ACCESS_DENIED_FAIL_CLOSED");
        assert_eq!(
            row.reason_code,
            sim_finder_reason_codes::SIM_FINDER_REFUSE_ACCESS_DENIED
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_consistency_level"),
            Some("DEGRADED_VERIFICATION")
        );
        assert_eq!(
            ph1x_payload_value(row, "identity_reason_code_hex"),
            Some("0x56490002")
        );
    }

    #[test]
    fn at_finder_exec_12_missing_sim_notify_audit_surfaces_identity_context_for_invite_link_path() {
        let (runtime, store, out, correlation_id) = run_missing_sim_invite_link_turn();
        let response_rows = store.ph1x_audit_rows(correlation_id);
        let row = find_ph1x_respond_row(&response_rows, "MISSING_SIMULATION_NOTIFY_SUBMITTED");

        assert_missing_simulation_notify_respond_payload(&runtime, row, &out);
        assert_eq!(
            row.reason_code,
            out.reason_code
                .expect("missing simulation notify path must preserve reason code")
        );
    }

    #[test]
    fn at_finder_exec_13_missing_sim_notify_audit_surfaces_identity_context_for_unsupported_intent_path(
    ) {
        let (runtime, store, out, correlation_id) = run_missing_sim_unsupported_intent_turn();
        let response_rows = store.ph1x_audit_rows(correlation_id);
        let row = find_ph1x_respond_row(&response_rows, "MISSING_SIMULATION_NOTIFY_SUBMITTED");

        assert_missing_simulation_notify_respond_payload(&runtime, row, &out);
        assert_eq!(
            row.reason_code,
            out.reason_code
                .expect("missing simulation notify path must preserve reason code")
        );
    }

    #[test]
    fn run6_desktop_voice_turn_end_to_end_returns_clarify_for_missing_recipient_contact() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:run6_clarify_user").unwrap();
        let device_id = DeviceId::new("run6_clarify_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_GENERATE_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

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
            thread_key: None,
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
            .contains("contact"));
        assert!(out.ph1x_response.is_none());
        match runtime
            .debug_last_finder_terminal_packet()
            .expect("finder packet should be captured")
        {
            FinderTerminalPacket::Clarify(clarify) => {
                assert_eq!(clarify.missing_field, "recipientcontact");
            }
            other => panic!("expected finder clarify packet, got {other:?}"),
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
            thread_key: None,
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
        let runtime = runtime_with_search_tool_fixtures();
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
            thread_key: None,
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
        assert!(response_text.contains("https://search.selene.ai/result-1"));
        assert!(!response_text.contains("example.invalid"));
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
    fn run_a2_ios_voice_turn_end_to_end_dispatches_web_search_and_returns_provenance() {
        let runtime = runtime_with_search_tool_fixtures();
        let actor_user_id = UserId::new("tenant_1:runa2_ios_websearch_user").unwrap();
        let device_id = DeviceId::new("runa2_ios_websearch_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(96031),
            TurnId(97031),
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
            now: MonotonicTimeNs(101),
            thread_key: None,
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
            .run_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        assert_eq!(out.next_move, AppVoiceTurnNextMove::Respond);
        let response_text = out.response_text.expect("respond output must include text");
        assert!(response_text.contains("https://search.selene.ai/result-1"));
        assert!(!response_text.contains("example.invalid"));
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
        let runtime = runtime_with_search_tool_fixtures();
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
            thread_key: None,
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
        assert!(response_text.contains("https://news.selene.ai/story-1"));
        assert!(!response_text.contains("example.invalid"));
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
        let runtime = runtime_with_search_tool_fixtures();
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
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(url_fetch_and_cite_draft(
                "open this URL and cite it: https://docs.selene.ai/spec",
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
        assert!(response_text.contains("https://docs.selene.ai/spec#chunk-"));
        assert!(!response_text.contains("example.invalid"));
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
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(document_understand_draft("read this PDF and summarize it")),
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
            thread_key: None,
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

    #[test]
    fn run_da_desktop_voice_turn_end_to_end_dispatches_data_analysis_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runda_data_user").unwrap();
        let device_id = DeviceId::new("runda_data_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9608),
            TurnId(9708),
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
            now: MonotonicTimeNs(15),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(data_analysis_draft(
                "analyze this csv and show summary stats with chart hints",
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
    fn run_dr_desktop_voice_turn_end_to_end_dispatches_deep_research_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:rundr_research_user").unwrap();
        let device_id = DeviceId::new("rundr_research_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9609),
            TurnId(9709),
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
            now: MonotonicTimeNs(16),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(deep_research_draft(
                "do deep research on AI chip export controls with sources",
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
    fn run_rm_desktop_voice_turn_end_to_end_dispatches_record_mode_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runrm_record_user").unwrap();
        let device_id = DeviceId::new("runrm_record_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9610),
            TurnId(9710),
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
            now: MonotonicTimeNs(17),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(record_mode_draft(
                "summarize this meeting recording and list action items",
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
        assert!(response_text.contains("Action items:"));
        assert!(response_text.contains("Recording evidence refs:"));
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
    fn run_cn_desktop_voice_turn_end_to_end_dispatches_connector_query_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runcn_connector_user").unwrap();
        let device_id = DeviceId::new("runcn_connector_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9612),
            TurnId(9712),
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
            now: MonotonicTimeNs(19),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(connector_query_draft(
                "search connectors for q3 roadmap notes in gmail and drive",
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
    fn run_co_desktop_voice_turn_end_to_end_connector_query_honors_explicit_scope() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runco_connector_user").unwrap();
        let device_id = DeviceId::new("runco_connector_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9613),
            TurnId(9713),
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
            now: MonotonicTimeNs(20),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(connector_query_draft(
                "search slack and notion for incident notes",
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
        assert!(response_text.contains("/slack/"));
        assert!(response_text.contains("/notion/"));
        assert!(!response_text.contains("/gmail/"));
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
    fn run_cp_desktop_voice_turn_end_to_end_message_policy_query_returns_settings_and_audit() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runcp_policy_user").unwrap();
        let device_id = DeviceId::new("runcp_policy_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);
        store
            .ph2access_upsert_instance_commit(
                MonotonicTimeNs(1),
                "tenant_1".to_string(),
                actor_user_id.clone(),
                "role.policy_viewer".to_string(),
                AccessMode::A,
                "{\"allow\":[\"BCAST_POLICY_UPDATE\"]}".to_string(),
                true,
                AccessVerificationLevel::PasscodeTime,
                AccessDeviceTrustLevel::Dtl4,
                AccessLifecycleState::Active,
                "policy_snapshot_v1".to_string(),
                None,
            )
            .unwrap();
        let tenant_id = TenantId::new("tenant_1").unwrap();
        store
            .append_bcast_policy_update_event(
                tenant_id.clone(),
                BcastPolicyUpdateValue::NonUrgentWaitSeconds(420),
                actor_user_id.clone(),
                CorrelationId(9614),
                "runcp_policy_update_1".to_string(),
                MonotonicTimeNs(21),
                ReasonCodeId(0x4243_00B1),
            )
            .unwrap();
        store
            .append_bcast_policy_update_event(
                tenant_id,
                BcastPolicyUpdateValue::UrgentFollowupMode { immediate: false },
                actor_user_id.clone(),
                CorrelationId(9614),
                "runcp_policy_update_2".to_string(),
                MonotonicTimeNs(22),
                ReasonCodeId(0x4243_00B2),
            )
            .unwrap();

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9614),
            TurnId(9714),
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
            now: MonotonicTimeNs(23),
            thread_key: None,
            thread_state: ThreadState::empty_v1(),
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(connector_query_draft(
                "Selene, show my message policy settings",
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
        assert!(response_text.contains("non_urgent_wait_seconds"));
        assert!(response_text.contains("urgent_followup_mode"));
        assert!(response_text.contains("max_followup_attempts"));
        assert!(response_text.contains("idempotency_key"));
        assert!(response_text.contains("Retrieved at (unix_ms):"));
        assert!(out.dispatch_outcome.is_none());
    }

    #[test]
    fn run_pr_desktop_voice_turn_applies_thread_policy_flags_to_ph1x_policy_context() {
        let runtime = AppServerIngressRuntime::default();
        let actor_user_id = UserId::new("tenant_1:runpr_policy_user").unwrap();
        let device_id = DeviceId::new("runpr_policy_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &actor_user_id, &device_id);

        let request = AppVoiceIngressRequest::v1(
            CorrelationId(9611),
            TurnId(9711),
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

        let policy_flags = ThreadPolicyFlags::v1(true, true, true).unwrap();
        let thread_state = ThreadState::empty_v1()
            .with_thread_policy_flags(Some(policy_flags))
            .unwrap();
        let x_build = AppVoicePh1xBuildInput {
            now: MonotonicTimeNs(18),
            thread_key: None,
            thread_state,
            session_state: SessionState::Active,
            policy_context_ref: PolicyContextRef::v1(false, false, SafetyTier::Standard),
            memory_candidates: vec![],
            confirm_answer: None,
            nlp_output: Some(web_search_draft("search the web for H100 pricing")),
            tool_response: None,
            interruption: None,
            locale: None,
            last_failure_reason_code: None,
        };

        let out = runtime
            .run_desktop_voice_turn_end_to_end(&mut store, request, x_build)
            .unwrap();
        let ph1x_request = out
            .ph1x_request
            .expect("turn should retain final PH1.X follow-up request");
        assert!(ph1x_request.policy_context_ref.privacy_mode);
        assert!(ph1x_request.policy_context_ref.do_not_disturb);
        assert!(matches!(
            ph1x_request.policy_context_ref.safety_tier,
            SafetyTier::Strict
        ));
    }

    #[test]
    fn run1_invite_link_click_starts_onboarding_with_active_simulations() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:run1_inviter").unwrap();
        let inviter_device_id = DeviceId::new("run1_inviter_device_1").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_OPEN_ACTIVATE_COMMIT,
            SimulationType::Commit,
            SimulationStatus::Active,
        );
        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            ONB_SESSION_START_DRAFT,
            SimulationType::Draft,
            SimulationStatus::Active,
        );

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(40),
        );
        let req = AppInviteLinkOpenRequest::v1(
            CorrelationId(9801),
            "run1-invite-click-idem-1".to_string(),
            token_id.clone(),
            token_signature,
            Some("tenant_1".to_string()),
            AppPlatform::Ios,
            "run1-device-fingerprint-a".to_string(),
            "ios_instance_run1".to_string(),
            "run1_nonce_9801".to_string(),
        )
        .unwrap();
        let out = runtime
            .run_invite_link_open_and_start_onboarding(&mut store, req, MonotonicTimeNs(41))
            .unwrap();

        assert!(out.onboarding_session_id.starts_with("onb_"));
        assert_eq!(out.next_step, OnboardingNextStep::AskMissing);
        assert!(!out.required_fields.is_empty());
        assert_eq!(
            store
                .ph1link_get_link(&token_id)
                .expect("link row must exist")
                .status,
            LinkStatus::Activated
        );
    }

    #[test]
    fn run1_invite_link_click_fails_closed_when_onboarding_simulation_not_active() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:run1_inviter_guard").unwrap();
        let inviter_device_id = DeviceId::new("run1_inviter_device_2").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        seed_simulation_catalog_status(
            &mut store,
            "tenant_1",
            LINK_INVITE_OPEN_ACTIVATE_COMMIT,
            SimulationType::Commit,
            SimulationStatus::Active,
        );

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(50),
        );
        let req = AppInviteLinkOpenRequest::v1(
            CorrelationId(9802),
            "run1-invite-click-idem-2".to_string(),
            token_id.clone(),
            token_signature,
            Some("tenant_1".to_string()),
            AppPlatform::Ios,
            "run1-device-fingerprint-b".to_string(),
            "ios_instance_run1".to_string(),
            "run1_nonce_9802".to_string(),
        )
        .unwrap();
        let err = runtime
            .run_invite_link_open_and_start_onboarding(&mut store, req, MonotonicTimeNs(51))
            .expect_err("missing ONB simulation should fail closed");

        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_invite_link_open_request.simulation_id");
                assert_eq!(reason, "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED");
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(
            store
                .ph1link_get_link(&token_id)
                .expect("link row must exist")
                .status,
            LinkStatus::DraftCreated
        );
    }

    #[test]
    fn runc_onboarding_ask_missing_prompts_once_and_escalates_on_repeat() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runc_missing_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runc_missing_inviter_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(80),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9901),
                    "runc-missing-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "runc-missing-fp".to_string(),
                    "ios_instance_runc".to_string(),
                    "nonce_runc_missing".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(81),
            )
            .unwrap();
        assert_eq!(start.next_step, OnboardingNextStep::AskMissing);
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let prompt = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9901),
                    onboarding_session_id.clone(),
                    "runc-missing-ask-1".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(82),
            )
            .unwrap();
        assert_eq!(prompt.next_step, AppOnboardingContinueNextStep::AskMissing);
        assert!(prompt.blocking_question.is_some());
        assert!(prompt.blocking_field.is_some());

        let err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9901),
                    onboarding_session_id,
                    "runc-missing-ask-2".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(83),
            )
            .expect_err("second missing-field non-answer must escalate");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_ASK_MISSING_REPEAT_ESCALATION");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn runc_onboarding_terms_device_voice_progression_from_link_click() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runc_flow_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runc_flow_inviter_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);
        seed_employee_company_and_position(&mut store, "tenant_1", MonotonicTimeNs(89));

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
            (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
            (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
            (WAKE_ENROLL_START_DRAFT, SimulationType::Draft),
            (WAKE_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
            (WAKE_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
            (WAKE_ENROLL_DEFER_COMMIT, SimulationType::Commit),
            (EMO_SIM_001, SimulationType::Commit),
            (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
            (ONB_COMPLETE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(90),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9902),
                    "runc-flow-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Android,
                    "runc-flow-fp".to_string(),
                    "android_instance_runc_flow".to_string(),
                    "nonce_runc_flow".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(91),
            )
            .unwrap();
        assert_eq!(start.next_step, OnboardingNextStep::AskMissing);
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-ask-prompt-1".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(92),
            )
            .unwrap();
        assert_eq!(ask_out.next_step, AppOnboardingContinueNextStep::AskMissing);

        for idx in 0..8 {
            if ask_out.next_step != AppOnboardingContinueNextStep::AskMissing {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask-missing step must include one blocking field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9902),
                        onboarding_session_id.clone(),
                        format!("runc-flow-ask-value-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(93 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(
            ask_out.next_step,
            AppOnboardingContinueNextStep::PlatformSetup
        );
        assert!(ask_out.remaining_missing_fields.is_empty());
        assert!(!ask_out.remaining_platform_receipt_kinds.is_empty());

        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9902),
                        onboarding_session_id.clone(),
                        format!("runc-flow-platform-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::PlatformSetupReceipt {
                            receipt_kind: receipt_kind.clone(),
                            receipt_ref: format!("receipt:runc-flow:{receipt_kind}"),
                            signer: "selene_mobile_app".to_string(),
                            payload_hash: format!("{:064x}", idx + 1),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(101 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(platform_out.next_step, AppOnboardingContinueNextStep::Terms);
        assert!(platform_out.remaining_platform_receipt_kinds.is_empty());

        let terms = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(110),
            )
            .unwrap();
        assert_eq!(
            terms.next_step,
            AppOnboardingContinueNextStep::PrimaryDeviceConfirm
        );

        let device_confirm = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-device".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PrimaryDeviceConfirm {
                        device_id: inviter_device_id,
                        proof_ok: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(111),
            )
            .unwrap();
        assert_eq!(
            device_confirm.next_step,
            AppOnboardingContinueNextStep::VoiceEnroll
        );

        let access_before_voice_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-access-before-voice".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AccessProvisionCommit,
                )
                .unwrap(),
                MonotonicTimeNs(133),
            )
            .expect_err("access must fail before voice enrollment lock");
        match access_before_voice_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_VOICE_ENROLL_REQUIRED_BEFORE_ACCESS_PROVISION");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let voice = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-voice".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("runc_flow_inviter_device").unwrap(),
                        sample_seed: "runc_flow_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(112),
            )
            .unwrap();
        assert_eq!(voice.next_step, AppOnboardingContinueNextStep::WakeEnroll);
        assert!(voice.voice_artifact_sync_receipt_ref.is_some());

        let complete_before_wake_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9915),
                    onboarding_session_id.clone(),
                    "runc-flow-complete-before-wake".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::CompleteCommit,
                )
                .unwrap(),
                MonotonicTimeNs(112),
            )
            .expect_err("complete must fail before wake enrollment");
        match complete_before_wake_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_WAKE_ENROLL_REQUIRED_BEFORE_COMPLETE");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let wake_start = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-wake-start".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::WakeEnrollStartDraft {
                        device_id: DeviceId::new("runc_flow_inviter_device").unwrap(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(113),
            )
            .unwrap();
        assert_eq!(
            wake_start.next_step,
            AppOnboardingContinueNextStep::WakeEnroll
        );

        for idx in 0..3 {
            let wake_sample = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9902),
                        onboarding_session_id.clone(),
                        format!("runc-flow-wake-sample-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::WakeEnrollSampleCommit {
                            device_id: DeviceId::new("runc_flow_inviter_device").unwrap(),
                            sample_pass: true,
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(114 + idx as u64),
                )
                .unwrap();
            if idx < 2 {
                assert_eq!(
                    wake_sample.next_step,
                    AppOnboardingContinueNextStep::WakeEnroll
                );
            }
        }

        let wake_complete = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-wake-complete".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::WakeEnrollCompleteCommit {
                        device_id: DeviceId::new("runc_flow_inviter_device").unwrap(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(118),
            )
            .unwrap();
        assert_eq!(
            wake_complete.next_step,
            AppOnboardingContinueNextStep::EmoPersonaLock
        );

        let access_before_emo_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9912),
                    onboarding_session_id.clone(),
                    "runc-flow-access-before-emo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AccessProvisionCommit,
                )
                .unwrap(),
                MonotonicTimeNs(119),
            )
            .expect_err("access must fail before emo/persona lock");
        match access_before_emo_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(
                    reason,
                    "ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_ACCESS_PROVISION"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let complete_before_emo_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9913),
                    onboarding_session_id.clone(),
                    "runc-flow-complete-before-emo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::CompleteCommit,
                )
                .unwrap(),
                MonotonicTimeNs(120),
            )
            .expect_err("complete must fail before emo/persona lock");
        match complete_before_emo_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_COMPLETE");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let emo = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-emo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::EmoPersonaLock,
                )
                .unwrap(),
                MonotonicTimeNs(121),
            )
            .unwrap();
        assert_eq!(
            emo.next_step,
            AppOnboardingContinueNextStep::AccessProvision
        );

        let complete_before_access_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9914),
                    onboarding_session_id.clone(),
                    "runc-flow-complete-before-access".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::CompleteCommit,
                )
                .unwrap(),
                MonotonicTimeNs(122),
            )
            .expect_err("complete must fail before access provisioning");
        match complete_before_access_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_ACCESS_PROVISION_REQUIRED_BEFORE_COMPLETE");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let access = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-access".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AccessProvisionCommit,
                )
                .unwrap(),
                MonotonicTimeNs(123),
            )
            .unwrap();
        assert_eq!(access.next_step, AppOnboardingContinueNextStep::Complete);
        assert!(access.access_engine_instance_id.is_some());
        assert_eq!(
            access.onboarding_status,
            Some(OnboardingStatus::AccessInstanceCreated)
        );

        let complete = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9902),
                    onboarding_session_id.clone(),
                    "runc-flow-complete".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::CompleteCommit,
                )
                .unwrap(),
                MonotonicTimeNs(124),
            )
            .unwrap();
        assert_eq!(complete.next_step, AppOnboardingContinueNextStep::Ready);
        assert_eq!(complete.onboarding_status, Some(OnboardingStatus::Complete));
        assert!(complete.voice_artifact_sync_receipt_ref.is_some());
        assert_eq!(store.ph1persona_audit_rows(CorrelationId(9902)).len(), 1);
        let session_row = store
            .ph1onb_session_row(&onboarding_session_id)
            .expect("onboarding session must exist");
        assert_eq!(session_row.status, OnboardingStatus::Complete);
        assert!(session_row.access_engine_instance_id.is_some());
        assert_eq!(
            store
                .ph1link_get_link(&session_row.token_id)
                .expect("link must exist")
                .status,
            LinkStatus::Consumed
        );
        assert_eq!(
            store
                .ph1onb_session_row(&onboarding_session_id)
                .expect("onboarding session must exist")
                .missing_fields
                .len(),
            0
        );
    }

    #[test]
    fn rund_onboarding_rejects_wake_enroll_actions_for_ios() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:rund_ios_wake_inviter").unwrap();
        let inviter_device_id = DeviceId::new("rund_ios_wake_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);
        seed_employee_company_and_position(&mut store, "tenant_1", MonotonicTimeNs(125));

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(126),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9906),
                    "rund-ios-wake-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "rund-ios-wake-fp".to_string(),
                    "ios_instance_rund".to_string(),
                    "nonce_rund_ios_wake".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(127),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9906),
                    onboarding_session_id,
                    "rund-ios-wake-action".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::WakeEnrollStartDraft {
                        device_id: DeviceId::new("rund_ios_wake_device").unwrap(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(128),
            )
            .expect_err("ios wake enroll actions must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ios_wake_disabled");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rung_onboarding_sender_verification_gate_routes_to_sender_verification_step() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:rung_verify_inviter").unwrap();
        let inviter_device_id = DeviceId::new("rung_verify_inviter_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);
        seed_employee_company_and_position(&mut store, "tenant_1", MonotonicTimeNs(119));
        seed_employee_position_schema_requiring_sender_verification(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(120),
        );

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
            (
                ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT,
                SimulationType::Commit,
            ),
            (ONB_EMPLOYEE_SENDER_VERIFY_COMMIT, SimulationType::Commit),
            (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
            (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
            (EMO_SIM_001, SimulationType::Commit),
            (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
            (ONB_COMPLETE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) =
            seed_invite_link_for_click_with_employee_prefilled_context(
                &mut store,
                &inviter_user_id,
                "tenant_1",
                MonotonicTimeNs(121),
            );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9911),
                    "rung-verify-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "rung-verify-fp".to_string(),
                    "ios_instance_rung_verify".to_string(),
                    "nonce_rung_verify".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(122),
            )
            .unwrap();
        assert!(start
            .required_verification_gates
            .contains(&"SENDER_CONFIRMATION".to_string()));
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-ask-prompt".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(123),
            )
            .unwrap();
        while ask_out.next_step == AppOnboardingContinueNextStep::AskMissing {
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask step must expose one missing field");
            let value = match field_key.as_str() {
                "driver_license_doc_ref" => "doc:driver:license:1",
                "working_hours" => "09:00-17:00",
                "jurisdiction_tags" => "US,CA",
                "compensation_tier_ref" => "band_l2",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9911),
                        onboarding_session_id.clone(),
                        format!("rung-verify-ask-{field_key}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(124),
                )
                .unwrap();
        }
        assert_eq!(
            ask_out.next_step,
            AppOnboardingContinueNextStep::PlatformSetup
        );
        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9911),
                        onboarding_session_id.clone(),
                        format!("rung-verify-platform-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::PlatformSetupReceipt {
                            receipt_kind: receipt_kind.clone(),
                            receipt_ref: format!("receipt:rung-verify:{receipt_kind}"),
                            signer: "selene_mobile_app".to_string(),
                            payload_hash: format!("{:064x}", idx + 1),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(125 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(platform_out.next_step, AppOnboardingContinueNextStep::Terms);

        let terms = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(130),
            )
            .unwrap();
        assert_eq!(
            terms.next_step,
            AppOnboardingContinueNextStep::SenderVerification
        );

        let access_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-access-blocked".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AccessProvisionCommit,
                )
                .unwrap(),
                MonotonicTimeNs(130),
            )
            .expect_err("access provision must block before sender verification");
        match access_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(
                    reason,
                    "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_ACCESS_PROVISION"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let photo = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-photo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::EmployeePhotoCaptureSend {
                        photo_blob_ref: "blob:photo:rung:1".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(131),
            )
            .unwrap();
        assert_eq!(
            photo.next_step,
            AppOnboardingContinueNextStep::SenderVerification
        );
        assert_eq!(
            photo.onboarding_status,
            Some(OnboardingStatus::VerificationPending)
        );

        let verify = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-sender".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::EmployeeSenderVerifyCommit {
                        decision: SenderVerifyDecision::Confirm,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(132),
            )
            .unwrap();
        assert_eq!(
            verify.next_step,
            AppOnboardingContinueNextStep::PrimaryDeviceConfirm
        );
        assert_eq!(
            verify.onboarding_status,
            Some(OnboardingStatus::VerificationConfirmed)
        );

        let device_confirm = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-device".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PrimaryDeviceConfirm {
                        device_id: inviter_device_id,
                        proof_ok: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(133),
            )
            .unwrap();
        assert_eq!(
            device_confirm.next_step,
            AppOnboardingContinueNextStep::VoiceEnroll
        );

        let voice = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-voice".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("rung_verify_inviter_device").unwrap(),
                        sample_seed: "rung_verify_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(134),
            )
            .unwrap();
        assert_eq!(
            voice.next_step,
            AppOnboardingContinueNextStep::EmoPersonaLock
        );

        let emo = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-emo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::EmoPersonaLock,
                )
                .unwrap(),
                MonotonicTimeNs(135),
            )
            .unwrap();
        assert_eq!(
            emo.next_step,
            AppOnboardingContinueNextStep::AccessProvision
        );

        let access = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-access".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AccessProvisionCommit,
                )
                .unwrap(),
                MonotonicTimeNs(136),
            )
            .unwrap();
        assert_eq!(access.next_step, AppOnboardingContinueNextStep::Complete);
        assert!(access.access_engine_instance_id.is_some());

        let complete = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9911),
                    onboarding_session_id.clone(),
                    "rung-verify-complete".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::CompleteCommit,
                )
                .unwrap(),
                MonotonicTimeNs(137),
            )
            .unwrap();
        assert_eq!(complete.next_step, AppOnboardingContinueNextStep::Ready);
        assert_eq!(complete.onboarding_status, Some(OnboardingStatus::Complete));
    }

    #[test]
    fn runi_onboarding_voice_enroll_fail_closed_when_stage_not_ready() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:runi_voice_guard_inviter").unwrap();
        let inviter_device_id = DeviceId::new("runi_voice_guard_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);
        seed_employee_company_and_position(&mut store, "tenant_1", MonotonicTimeNs(139));

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
            (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
            (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(140),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9921),
                    "runi-voice-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "runi-voice-fp".to_string(),
                    "ios_instance_runi_voice".to_string(),
                    "nonce_runi_voice".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(141),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let early_voice_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-early".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("runi_voice_guard_device").unwrap(),
                        sample_seed: "runi_voice_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(142),
            )
            .expect_err("voice enroll must fail when ask-missing still pending");
        match early_voice_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_ASK_MISSING_REQUIRED_BEFORE_VOICE_ENROLL");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-ask-prompt".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(143),
            )
            .unwrap();
        while ask_out.next_step == AppOnboardingContinueNextStep::AskMissing {
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask step must expose one missing field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9921),
                        onboarding_session_id.clone(),
                        format!("runi-voice-ask-{field_key}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(144),
                )
                .unwrap();
        }
        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9921),
                        onboarding_session_id.clone(),
                        format!("runi-voice-platform-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::PlatformSetupReceipt {
                            receipt_kind: receipt_kind.clone(),
                            receipt_ref: format!("receipt:runi-voice:{receipt_kind}"),
                            signer: "selene_mobile_app".to_string(),
                            payload_hash: format!("{:064x}", idx + 1),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(145 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(platform_out.next_step, AppOnboardingContinueNextStep::Terms);
        runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(151),
            )
            .unwrap();

        let pre_primary_voice_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-pre-primary".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("runi_voice_guard_device").unwrap(),
                        sample_seed: "runi_voice_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(152),
            )
            .expect_err("voice enroll must fail before primary device confirm");
        match pre_primary_voice_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(
                    reason,
                    "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }

        runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-primary".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PrimaryDeviceConfirm {
                        device_id: inviter_device_id,
                        proof_ok: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(153),
            )
            .unwrap();

        let wrong_device_voice_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9921),
                    onboarding_session_id.clone(),
                    "runi-voice-wrong-device".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("runi_voice_other_device").unwrap(),
                        sample_seed: "runi_voice_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(154),
            )
            .expect_err("voice enroll must fail for non-primary device");
        match wrong_device_voice_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_onboarding_continue_request.action.voice_enroll_lock.device_id"
                );
                assert_eq!(
                    reason,
                    "ONB_PRIMARY_DEVICE_DEVICE_MISMATCH_FOR_VOICE_ENROLL"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rune_onboarding_emo_persona_lock_requires_active_simulation() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:rune_flow_inviter").unwrap();
        let inviter_device_id = DeviceId::new("rune_flow_inviter_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
            (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
            (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
            (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(114),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9905),
                    "rune-flow-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "rune-flow-fp".to_string(),
                    "ios_instance_rune_flow".to_string(),
                    "nonce_rune_flow".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(115),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9905),
                    onboarding_session_id.clone(),
                    "rune-flow-ask-prompt-1".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(116),
            )
            .unwrap();
        for idx in 0..8 {
            if ask_out.next_step != AppOnboardingContinueNextStep::AskMissing {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask-missing step must include one blocking field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9905),
                        onboarding_session_id.clone(),
                        format!("rune-flow-ask-value-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(117 + idx as u64),
                )
                .unwrap();
        }
        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9905),
                        onboarding_session_id.clone(),
                        format!("rune-flow-platform-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::PlatformSetupReceipt {
                            receipt_kind: receipt_kind.clone(),
                            receipt_ref: format!("receipt:rune-flow:{receipt_kind}"),
                            signer: "selene_mobile_app".to_string(),
                            payload_hash: format!("{:064x}", idx + 1),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(125 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(platform_out.next_step, AppOnboardingContinueNextStep::Terms);

        runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9905),
                    onboarding_session_id.clone(),
                    "rune-flow-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(130),
            )
            .unwrap();
        runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9905),
                    onboarding_session_id.clone(),
                    "rune-flow-device".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PrimaryDeviceConfirm {
                        device_id: inviter_device_id,
                        proof_ok: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(131),
            )
            .unwrap();
        let voice = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9905),
                    onboarding_session_id.clone(),
                    "rune-flow-voice".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::VoiceEnrollLock {
                        device_id: DeviceId::new("rune_flow_inviter_device").unwrap(),
                        sample_seed: "rune_flow_seed".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(132),
            )
            .unwrap();
        assert_eq!(
            voice.next_step,
            AppOnboardingContinueNextStep::EmoPersonaLock
        );

        let err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9905),
                    onboarding_session_id,
                    "rune-flow-emo".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::EmoPersonaLock,
                )
                .unwrap(),
                MonotonicTimeNs(133),
            )
            .expect_err("missing emo simulation must fail closed");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.simulation_id");
                assert_eq!(reason, "SIM_DISPATCH_GUARD_SIMULATION_NOT_REGISTERED");
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(store.ph1persona_audit_rows(CorrelationId(9905)).is_empty());
    }

    #[test]
    fn rund_onboarding_terms_blocked_until_platform_receipts_complete() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:rund_terms_block_inviter").unwrap();
        let inviter_device_id = DeviceId::new("rund_terms_block_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(130),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9903),
                    "rund-terms-block-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "rund-terms-block-fp".to_string(),
                    "ios_instance_rund_terms_block".to_string(),
                    "nonce_rund_terms_block".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(131),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9903),
                    onboarding_session_id.clone(),
                    "rund-terms-block-ask-prompt".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(132),
            )
            .unwrap();
        for idx in 0..8 {
            if ask_out.next_step != AppOnboardingContinueNextStep::AskMissing {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask-missing must return blocking field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9903),
                        onboarding_session_id.clone(),
                        format!("rund-terms-block-ask-value-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(133 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(
            ask_out.next_step,
            AppOnboardingContinueNextStep::PlatformSetup
        );
        assert!(!ask_out.remaining_platform_receipt_kinds.is_empty());

        let err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9903),
                    onboarding_session_id,
                    "rund-terms-block-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(145),
            )
            .expect_err("terms must be blocked before platform setup receipts are complete");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_PLATFORM_SETUP_REQUIRED_BEFORE_TERMS");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rund_onboarding_platform_setup_receipt_rejects_invalid_signer_and_hash() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:rund_platform_guard_inviter").unwrap();
        let inviter_device_id = DeviceId::new("rund_platform_guard_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(150),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(9904),
                    "rund-platform-guard-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Ios,
                    "rund-platform-guard-fp".to_string(),
                    "ios_instance_rund_platform_guard".to_string(),
                    "nonce_rund_platform_guard".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(151),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9904),
                    onboarding_session_id.clone(),
                    "rund-platform-guard-ask-prompt".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(152),
            )
            .unwrap();
        for idx in 0..8 {
            if ask_out.next_step != AppOnboardingContinueNextStep::AskMissing {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask-missing must return blocking field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(9904),
                        onboarding_session_id.clone(),
                        format!("rund-platform-guard-ask-value-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(153 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(
            ask_out.next_step,
            AppOnboardingContinueNextStep::PlatformSetup
        );
        let receipt_kind = ask_out
            .remaining_platform_receipt_kinds
            .first()
            .cloned()
            .expect("platform setup must require at least one receipt");

        let bad_signer_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9904),
                    onboarding_session_id.clone(),
                    "rund-platform-guard-bad-signer".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PlatformSetupReceipt {
                        receipt_kind: receipt_kind.clone(),
                        receipt_ref: "receipt:rund-platform:bad-signer".to_string(),
                        signer: "not_allowed_signer".to_string(),
                        payload_hash: format!("{:064x}", 1),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(170),
            )
            .expect_err("invalid signer must fail closed");
        match bad_signer_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "ph1onb_platform_setup_receipt_commit.signer");
                assert_eq!(reason, "must match platform signer policy");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let bad_hash_err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(9904),
                    onboarding_session_id,
                    "rund-platform-guard-bad-hash".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PlatformSetupReceipt {
                        receipt_kind,
                        receipt_ref: "receipt:rund-platform:bad-hash".to_string(),
                        signer: "selene_mobile_app".to_string(),
                        payload_hash: "BAD_HASH".to_string(),
                    },
                )
                .unwrap(),
                MonotonicTimeNs(171),
            )
            .expect_err("invalid payload hash must fail closed");
        match bad_hash_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "ph1onb_platform_setup_receipt_commit.payload_hash");
                assert_eq!(reason, "must be lowercase hex sha256 (64 chars)");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn run_gov_voice_turn_safe_mode_blocks_execution_and_records_decision() {
        let runtime = AppServerIngressRuntime::new_with_runtime_governance(
            SimulationExecutor::default(),
            crate::runtime_governance::RuntimeGovernanceRuntime::new(
                RuntimeGovernanceConfig::mvp_v1().with_force_integrity_failure(true),
            ),
        );
        let actor_user_id = UserId::new("tenant_1:gov_safe_mode_user").unwrap();
        let device_id = DeviceId::new("gov_safe_mode_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        let request = AppVoiceIngressRequest::v1(
            CorrelationId(88_001),
            TurnId(89_001),
            AppPlatform::Desktop,
            OsVoiceTrigger::Explicit,
            sample_voice_id_request(MonotonicTimeNs(1), actor_user_id.clone()),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            no_observation(),
        )
        .unwrap();

        let err = runtime
            .run_voice_turn(&mut store, request)
            .expect_err("forced governance integrity failure must block voice execution");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_voice_ingress_request.runtime_execution_envelope"
                );
                assert_eq!(
                    reason,
                    "governance_safe_mode GOV_GOVERNANCE_INTEGRITY_UNCERTAIN"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runtime
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code
                    == crate::runtime_governance::reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN
            }));
    }

    #[test]
    fn run_gov_primary_device_confirmation_requires_proof_and_records_decision() {
        let runtime = AppServerIngressRuntime::default();
        let inviter_user_id = UserId::new("tenant_1:gov_proof_inviter").unwrap();
        let inviter_device_id = DeviceId::new("gov_proof_device").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        seed_actor(&mut store, &inviter_user_id, &inviter_device_id);
        seed_employee_company_and_position(&mut store, "tenant_1", MonotonicTimeNs(89));

        for (simulation_id, simulation_type) in [
            (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
            (ONB_SESSION_START_DRAFT, SimulationType::Draft),
            (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
            (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
            (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
        ] {
            seed_simulation_catalog_status(
                &mut store,
                "tenant_1",
                simulation_id,
                simulation_type,
                SimulationStatus::Active,
            );
        }

        let (token_id, token_signature) = seed_invite_link_for_click(
            &mut store,
            &inviter_user_id,
            "tenant_1",
            MonotonicTimeNs(90),
        );
        let start = runtime
            .run_invite_link_open_and_start_onboarding(
                &mut store,
                AppInviteLinkOpenRequest::v1(
                    CorrelationId(88_101),
                    "gov-proof-start".to_string(),
                    token_id,
                    token_signature,
                    Some("tenant_1".to_string()),
                    AppPlatform::Android,
                    "gov-proof-fp".to_string(),
                    "android_instance_gov_proof".to_string(),
                    "nonce_gov_proof".to_string(),
                )
                .unwrap(),
                MonotonicTimeNs(91),
            )
            .unwrap();
        let onboarding_session_id = OnboardingSessionId::new(start.onboarding_session_id).unwrap();

        let mut ask_out = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(88_101),
                    onboarding_session_id.clone(),
                    "gov-proof-ask-prompt".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::AskMissingSubmit { field_value: None },
                )
                .unwrap(),
                MonotonicTimeNs(92),
            )
            .unwrap();
        for idx in 0..8 {
            if ask_out.next_step != AppOnboardingContinueNextStep::AskMissing {
                break;
            }
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ask-missing step must include one blocking field");
            let field_value = match field_key.as_str() {
                "tenant_id" => "tenant_1",
                "company_id" => "company_1",
                "position_id" => "position_1",
                "location_id" => "loc_1",
                "start_date" => "2026-03-01",
                "working_hours" => "09:00-17:00",
                "compensation_tier_ref" => "band_l2",
                "jurisdiction_tags" => "US,CA",
                _ => "value_1",
            };
            ask_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(88_101),
                        onboarding_session_id.clone(),
                        format!("gov-proof-ask-value-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::AskMissingSubmit {
                            field_value: Some(field_value.to_string()),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(93 + idx as u64),
                )
                .unwrap();
        }
        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            platform_out = runtime
                .run_onboarding_continue(
                    &mut store,
                    AppOnboardingContinueRequest::v1(
                        CorrelationId(88_101),
                        onboarding_session_id.clone(),
                        format!("gov-proof-platform-{idx}"),
                        Some("tenant_1".to_string()),
                        AppOnboardingContinueAction::PlatformSetupReceipt {
                            receipt_kind: receipt_kind.clone(),
                            receipt_ref: format!("receipt:gov-proof:{receipt_kind}"),
                            signer: "selene_mobile_app".to_string(),
                            payload_hash: format!("{:064x}", idx + 1),
                        },
                    )
                    .unwrap(),
                    MonotonicTimeNs(101 + idx as u64),
                )
                .unwrap();
        }
        assert_eq!(platform_out.next_step, AppOnboardingContinueNextStep::Terms);

        let terms = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(88_101),
                    onboarding_session_id.clone(),
                    "gov-proof-terms".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::TermsAccept {
                        terms_version_id: "terms_v1".to_string(),
                        accepted: true,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(110),
            )
            .unwrap();
        assert_eq!(
            terms.next_step,
            AppOnboardingContinueNextStep::PrimaryDeviceConfirm
        );

        let err = runtime
            .run_onboarding_continue(
                &mut store,
                AppOnboardingContinueRequest::v1(
                    CorrelationId(88_101),
                    onboarding_session_id,
                    "gov-proof-device".to_string(),
                    Some("tenant_1".to_string()),
                    AppOnboardingContinueAction::PrimaryDeviceConfirm {
                        device_id: inviter_device_id,
                        proof_ok: false,
                    },
                )
                .unwrap(),
                MonotonicTimeNs(111),
            )
            .expect_err("primary device confirmation must fail closed without proof");
        match err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(
                    field,
                    "app_onboarding_continue_request.action.primary_device_confirm.proof_ok"
                );
                assert_eq!(reason, "governance_policy_block GOV_PROOF_REQUIRED");
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(runtime
            .runtime_governance_decision_log_snapshot()
            .iter()
            .any(|entry| {
                entry.reason_code == crate::runtime_governance::reason_codes::GOV_PROOF_REQUIRED
            }));
    }

    #[test]
    fn run_a_response_text_for_calendar_draft_is_explicit_draft_only() {
        let response =
            response_text_for_dispatch_outcome(&SimulationDispatchOutcome::CalendarDraftCreated {
                reminder_id: "rem_test".to_string(),
            });
        assert_eq!(
            response,
            "Draft created; not sent to external calendar yet."
        );
    }

    #[test]
    fn run_a_response_text_for_reminder_update_is_explicit() {
        let response = response_text_for_dispatch_outcome(&SimulationDispatchOutcome::Reminder(
            selene_kernel_contracts::ph1rem::Ph1RemResponse::Ok(
                selene_kernel_contracts::ph1rem::Ph1RemOk::v1(
                    "REMINDER_UPDATE_COMMIT".to_string(),
                    ReasonCodeId(1),
                    selene_kernel_contracts::ph1rem::ReminderId::new("rem_test").unwrap(),
                    None,
                    selene_kernel_contracts::ph1rem::ReminderState::Scheduled,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap(),
            ),
        ));
        assert_eq!(response, "I updated that reminder.");
    }

    #[test]
    fn run_a_response_text_for_reminder_cancel_is_explicit() {
        let response = response_text_for_dispatch_outcome(&SimulationDispatchOutcome::Reminder(
            selene_kernel_contracts::ph1rem::Ph1RemResponse::Ok(
                selene_kernel_contracts::ph1rem::Ph1RemOk::v1(
                    "REMINDER_CANCEL_COMMIT".to_string(),
                    ReasonCodeId(1),
                    selene_kernel_contracts::ph1rem::ReminderId::new("rem_test").unwrap(),
                    None,
                    selene_kernel_contracts::ph1rem::ReminderState::Canceled,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap(),
            ),
        ));
        assert_eq!(response, "I canceled that reminder.");
    }
}
