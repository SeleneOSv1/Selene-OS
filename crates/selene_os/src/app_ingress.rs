#![forbid(unsafe_code)]

use selene_engines::ph1_voice_id::{
    simulation_profile_embedding_from_seed, EnrolledSpeaker as EngineEnrolledSpeaker,
    VoiceIdObservation as EngineVoiceIdObservation,
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
use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdRequest, SpeakerId, UserId, VoiceEmbeddingCaptureRef,
    VOICE_ID_ENROLL_COMPLETE_COMMIT, VOICE_ID_ENROLL_SAMPLE_COMMIT, VOICE_ID_ENROLL_START_DRAFT,
};
use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
use selene_kernel_contracts::ph1e::ToolResponse;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1k::InterruptCandidate;
use selene_kernel_contracts::ph1link::{
    AppPlatform, InviteeType, LinkStatus, Ph1LinkRequest, Ph1LinkResponse, TokenId,
    LINK_INVITE_DRAFT_UPDATE_COMMIT, LINK_INVITE_OPEN_ACTIVATE_COMMIT,
};
use selene_kernel_contracts::ph1m::MemoryCandidate;
use selene_kernel_contracts::ph1n::{FieldKey, Ph1nResponse};
use selene_kernel_contracts::ph1persona::{
    PersonaDeliveryPolicyRef, PersonaPreferenceKey, PersonaPreferenceSignal,
    PersonaProfileValidateRequest, PersonaRequestEnvelope, PersonaValidationStatus,
    Ph1PersonaRequest, Ph1PersonaResponse,
};
use selene_kernel_contracts::ph1onb::{
    OnbAccessInstanceCreateCommitRequest, OnbCompleteCommitRequest,
    OnbEmployeePhotoCaptureSendCommitRequest, OnbEmployeeSenderVerifyCommitRequest,
    OnbPrimaryDeviceConfirmCommitRequest, OnbRequest, OnbTermsAcceptCommitRequest,
    OnboardingNextStep, OnboardingSessionId, OnboardingStatus, Ph1OnbRequest, Ph1OnbResponse,
    ProofType, SenderVerifyDecision, SimulationType, TermsStatus, VerificationStatus,
    ONB_ACCESS_INSTANCE_CREATE_COMMIT, ONB_COMPLETE_COMMIT,
    ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT, ONB_EMPLOYEE_SENDER_VERIFY_COMMIT,
    ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, ONB_SESSION_START_DRAFT, ONB_TERMS_ACCEPT_COMMIT,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1tts::StyleProfileRef;
use selene_kernel_contracts::ph1x::{
    ConfirmAnswer, DispatchRequest, IdentityContext, Ph1xDirective, Ph1xRequest, Ph1xResponse,
    StepUpCapabilities, ThreadState,
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
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
};

use crate::device_artifact_sync::DeviceArtifactSyncWorkerPassMetrics;
use crate::ph1os::{
    OsTopLevelTurnInput, OsTopLevelTurnPath, OsTurnInput, OsVoiceLiveTurnInput,
    OsVoiceLiveTurnOutcome, OsVoicePlatform, OsVoiceTrigger, OsVoiceTurnContext,
};
use crate::ph1onb::{
    OnbVoiceEnrollFinalize, OnbVoiceEnrollLiveRequest, OnbVoiceEnrollSampleStep,
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
    pub tool_response: Option<ToolResponse>,
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
        let effective_tenant = request_tenant.or(link_tenant).ok_or_else(|| {
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
            Ph1LinkResponse::Ok(ok) => ok.link_activation_result.ok_or_else(|| {
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
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.ok_or_else(|| {
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
        let effective_tenant = request_tenant
            .or(session_tenant)
            .or(link_tenant)
            .ok_or_else(|| {
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
                        let blocking_field = if next_step == AppOnboardingContinueNextStep::AskMissing
                        {
                            ask.field_key
                                .clone()
                                .or_else(|| ask.remaining_missing_fields.first().cloned())
                        } else {
                            None
                        };
                        let blocking_question =
                            blocking_field.as_deref().map(onboarding_missing_field_question);
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
                    selene_storage::ph1f::OnbAskMissingOutcomeKind::Escalated => {
                        Err(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "app_onboarding_continue_request.action",
                                reason: "ONB_ASK_MISSING_REPEAT_ESCALATION",
                            },
                        ))
                    }
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
                let remaining_platform_receipt_kinds = store
                    .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
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
                    Ph1OnbResponse::Ok(ok) => ok
                        .terms_accept_result
                        .ok_or_else(|| {
                            StorageError::ContractViolation(ContractViolation::InvalidValue {
                                field: "ph1onb_response.terms_accept_result",
                                reason: "terms accept result must be present",
                            })
                        })?
                        .terms_status,
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
            AppOnboardingContinueAction::PrimaryDeviceConfirm { device_id, proof_ok } => {
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
                    Ph1OnbResponse::Ok(ok) => ok
                        .primary_device_confirm_result
                        .ok_or_else(|| {
                            StorageError::ContractViolation(ContractViolation::InvalidValue {
                                field: "ph1onb_response.primary_device_confirm_result",
                                reason: "primary device confirm result must be present",
                            })
                        })?
                        .primary_device_confirmed,
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
                let remaining_platform_receipt_kinds = store
                    .ph1onb_remaining_platform_receipt_kinds(&onboarding_session_id)?;
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
                                reason:
                                    "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL",
                            },
                        ))?;
                if expected_device_id != device_id {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "app_onboarding_continue_request.action.voice_enroll_lock.device_id",
                            reason:
                                "ONB_PRIMARY_DEVICE_DEVICE_MISMATCH_FOR_VOICE_ENROLL",
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
                let voice_artifact_sync_receipt_ref =
                    voice_out.complete_result.and_then(|r| r.voice_artifact_sync_receipt_ref);
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
                    next_step: AppOnboardingContinueNextStep::EmoPersonaLock,
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
            AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } => {
                self.run_onboarding_employee_photo_capture_send(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    photo_blob_ref,
                    idempotency_key,
                    now,
                )
            }
            AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision } => {
                self.run_onboarding_employee_sender_verify_commit(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    decision,
                    idempotency_key,
                    now,
                )
            }
            AppOnboardingContinueAction::EmoPersonaLock => self.run_onboarding_emo_persona_lock(
                store,
                correlation_id,
                turn_id,
                onboarding_session_id,
                effective_tenant,
                idempotency_key,
                now,
            ),
            AppOnboardingContinueAction::AccessProvisionCommit => {
                self.run_onboarding_access_provision(
                    store,
                    correlation_id,
                    turn_id,
                    onboarding_session_id,
                    effective_tenant,
                    idempotency_key,
                    now,
                )
            }
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
            Ph1OnbResponse::Ok(ok) => ok
                .employee_photo_result
                .ok_or_else(|| {
                    StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field: "ph1onb_response.employee_photo_result",
                        reason: "employee photo result must be present",
                    })
                })?
                .verification_status,
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
            Ph1OnbResponse::Ok(ok) => ok
                .employee_sender_verify_result
                .ok_or_else(|| {
                    StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field: "ph1onb_response.employee_sender_verify_result",
                        reason: "employee sender verify result must be present",
                    })
                })?
                .verification_status,
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
                    AppOnboardingContinueNextStep::EmoPersonaLock
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

        let device_id = session.primary_device_device_id.clone().ok_or_else(|| {
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
        emo_req.validate().map_err(StorageError::ContractViolation)?;
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
        let emo_guide_build_req =
            Ph1EmoGuideRequest::EmoGuideProfileBuild(EmoGuideProfileBuildRequest::v1(
                emo_guide_envelope.clone(),
                persona_speaker_id.clone(),
                emo_guide_signals.clone(),
                None,
            )
            .map_err(StorageError::ContractViolation)?);
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

        let emo_guide_validate_req =
            Ph1EmoGuideRequest::EmoGuideProfileValidate(EmoGuideProfileValidateRequest::v1(
                emo_guide_envelope,
                persona_speaker_id.clone(),
                emo_guide_signals,
                None,
                emo_guide_build_ok.profile.clone(),
            )
            .map_err(StorageError::ContractViolation)?);
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
            style_profile_ref_token(persona_build_ok.profile_snapshot.style_profile_ref).to_string(),
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
        let device_id = session.primary_device_device_id.clone().ok_or_else(|| {
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
            Ph1OnbResponse::Ok(ok) => ok
                .access_instance_create_result
                .ok_or_else(|| {
                    StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field: "ph1onb_response.access_instance_create_result",
                        reason: "access instance create result must be present",
                    })
                })?
                .access_engine_instance_id,
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
            .ok_or_else(|| {
                StorageError::ContractViolation(ContractViolation::InvalidValue {
                    field: "app_onboarding_continue_request.action",
                    reason: "ONB_VOICE_ENROLL_REQUIRED_BEFORE_COMPLETE",
                })
            })?;
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
            Ph1OnbResponse::Ok(ok) => ok
                .complete_result
                .ok_or_else(|| {
                    StorageError::ContractViolation(ContractViolation::InvalidValue {
                        field: "ph1onb_response.complete_result",
                        reason: "complete result must be present",
                    })
                })?
                .onboarding_status,
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

    pub fn run_voice_turn_end_to_end(
        &self,
        store: &mut Ph1fStore,
        request: AppVoiceIngressRequest,
        x_build: AppVoicePh1xBuildInput,
    ) -> Result<AppVoiceTurnExecutionOutcome, StorageError> {
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
                    let tool_response = self.ph1e_runtime.run(tool_request);
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
            tool_response: None,
            response_text: None,
            reason_code: None,
        },
        OsVoiceLiveTurnOutcome::Refused(refuse) => AppVoiceTurnExecutionOutcome {
            voice_outcome: OsVoiceLiveTurnOutcome::Refused(refuse.clone()),
            next_move: AppVoiceTurnNextMove::Refused,
            ph1x_request: None,
            ph1x_response: None,
            dispatch_outcome: None,
            tool_response: None,
            response_text: Some(refuse.message.clone()),
            reason_code: Some(refuse.reason_code),
        },
        OsVoiceLiveTurnOutcome::Forwarded(forwarded) => AppVoiceTurnExecutionOutcome {
            voice_outcome: OsVoiceLiveTurnOutcome::Forwarded(forwarded),
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
    let session = store
        .ph1onb_session_row(onboarding_session_id)
        .ok_or(StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        })?;
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
    let session = store
        .ph1onb_session_row(onboarding_session_id)
        .ok_or(StorageError::ForeignKeyViolation {
            table: "onboarding_sessions.onboarding_session_id",
            key: onboarding_session_id.as_str().to_string(),
        })?;
    let link = store.ph1link_get_link(&session.token_id).ok_or(
        StorageError::ForeignKeyViolation {
            table: "links.token_id",
            key: session.token_id.as_str().to_string(),
        },
    )?;
    Ok(link.inviter_user_id.clone())
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
    let effective_policy_context_ref =
        merge_thread_policy_context(x_build.policy_context_ref, &x_build.thread_state);
    let mut req = Ph1xRequest::v1(
        correlation_id.0,
        turn_id.0,
        x_build.now,
        x_build.thread_state,
        x_build.session_state,
        IdentityContext::Voice(forwarded.voice_identity_assertion.clone()),
        effective_policy_context_ref,
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
        ConfirmAnswer, IdentityContext, PendingState, Ph1xDirective, ThreadPolicyFlags,
        ThreadState,
    };
    use selene_kernel_contracts::{
        ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState,
    };
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
            required_rule: selene_kernel_contracts::ph1position::PositionRequirementRuleType::Always,
            required_predicate_ref: None,
            validation_ref: None,
            sensitivity: selene_kernel_contracts::ph1position::PositionRequirementSensitivity::Private,
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
    fn run_a2_ios_voice_turn_end_to_end_dispatches_web_search_and_returns_provenance() {
        let runtime = AppServerIngressRuntime::default();
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
                    AppPlatform::Ios,
                    "runc-flow-fp".to_string(),
                    "ios_instance_runc_flow".to_string(),
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
        assert_eq!(ask_out.next_step, AppOnboardingContinueNextStep::PlatformSetup);
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
        assert_eq!(device_confirm.next_step, AppOnboardingContinueNextStep::VoiceEnroll);

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
        assert_eq!(voice.next_step, AppOnboardingContinueNextStep::EmoPersonaLock);
        assert!(voice.voice_artifact_sync_receipt_ref.is_some());

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
                MonotonicTimeNs(113),
            )
            .expect_err("access must fail before emo/persona lock");
        match access_before_emo_err {
            StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason }) => {
                assert_eq!(field, "app_onboarding_continue_request.action");
                assert_eq!(reason, "ONB_EMO_PERSONA_LOCK_REQUIRED_BEFORE_ACCESS_PROVISION");
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
                MonotonicTimeNs(114),
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
                MonotonicTimeNs(115),
        )
        .unwrap();
        assert_eq!(emo.next_step, AppOnboardingContinueNextStep::AccessProvision);

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
                MonotonicTimeNs(116),
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
                MonotonicTimeNs(117),
            )
            .unwrap();
        assert_eq!(access.next_step, AppOnboardingContinueNextStep::Complete);
        assert!(access.access_engine_instance_id.is_some());
        assert_eq!(access.onboarding_status, Some(OnboardingStatus::AccessInstanceCreated));

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
                MonotonicTimeNs(118),
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
                .ph1link_get_link(
                    &session_row.token_id
                )
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
            (ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT, SimulationType::Commit),
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

        let (token_id, token_signature) = seed_invite_link_for_click_with_employee_prefilled_context(
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
        assert!(
            start
                .required_verification_gates
                .contains(&"SENDER_CONFIRMATION".to_string())
        );
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
        assert_eq!(ask_out.next_step, AppOnboardingContinueNextStep::PlatformSetup);
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
        assert_eq!(terms.next_step, AppOnboardingContinueNextStep::SenderVerification);

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
                assert_eq!(reason, "ONB_SENDER_VERIFICATION_REQUIRED_BEFORE_ACCESS_PROVISION");
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
        assert_eq!(photo.next_step, AppOnboardingContinueNextStep::SenderVerification);
        assert_eq!(photo.onboarding_status, Some(OnboardingStatus::VerificationPending));

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
        assert_eq!(device_confirm.next_step, AppOnboardingContinueNextStep::VoiceEnroll);

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
        assert_eq!(voice.next_step, AppOnboardingContinueNextStep::EmoPersonaLock);

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
        assert_eq!(emo.next_step, AppOnboardingContinueNextStep::AccessProvision);

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

        let (token_id, token_signature) =
            seed_invite_link_for_click(&mut store, &inviter_user_id, "tenant_1", MonotonicTimeNs(140));
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
                assert_eq!(reason, "ONB_PRIMARY_DEVICE_CONFIRM_REQUIRED_BEFORE_VOICE_ENROLL");
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
                assert_eq!(reason, "ONB_PRIMARY_DEVICE_DEVICE_MISMATCH_FOR_VOICE_ENROLL");
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
        assert_eq!(voice.next_step, AppOnboardingContinueNextStep::EmoPersonaLock);

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
        assert_eq!(ask_out.next_step, AppOnboardingContinueNextStep::PlatformSetup);
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
        assert_eq!(ask_out.next_step, AppOnboardingContinueNextStep::PlatformSetup);
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
}
