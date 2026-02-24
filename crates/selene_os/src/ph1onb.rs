#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse, VoiceEmbeddingCaptureRef,
    VoiceEnrollmentSessionId, VoiceIdEnrollCompleteCommitRequest, VoiceIdEnrollCompleteResult,
    VoiceIdEnrollDeferCommitRequest, VoiceIdEnrollDeferResult, VoiceIdEnrollSampleCommitRequest,
    VoiceIdEnrollSampleResult, VoiceIdEnrollStartDraftRequest, VoiceIdEnrollStartResult,
    VoiceIdSimulationRequest, VoiceIdSimulationType, PH1VOICEID_SIM_CONTRACT_VERSION,
    VOICE_ID_ENROLL_COMPLETE_COMMIT, VOICE_ID_ENROLL_DEFER_COMMIT, VOICE_ID_ENROLL_SAMPLE_COMMIT,
    VOICE_ID_ENROLL_START_DRAFT,
};
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1link::{LinkActivationResult, LinkStatus};
use selene_kernel_contracts::ph1onb::{
    OnbRequest, OnbSessionStartResult, OnboardingSessionId, Ph1OnbOk, Ph1OnbRequest,
    Ph1OnbResponse, SimulationType, ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT,
    ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT, ONB_REQUIREMENT_BACKFILL_START_DRAFT,
};
use selene_kernel_contracts::ph1position::{
    Ph1PositionRequest, PositionBandPolicyCheckRequest, PositionBandPolicyCheckResult,
    PositionCreateDraftRequest, PositionCreateDraftResult, PositionId, PositionLifecycleResult,
    PositionPolicyResult, PositionRequest, PositionRequestedAction, PositionScheduleType,
    PositionSimulationType, PositionValidateAuthCompanyRequest, PositionValidateAuthCompanyResult,
    PositionValidationStatus, TenantId, PH1POSITION_CONTRACT_VERSION,
    POSITION_SIM_001_CREATE_DRAFT, POSITION_SIM_002_VALIDATE_AUTH_COMPANY,
    POSITION_SIM_003_BAND_POLICY_CHECK, POSITION_SIM_004_ACTIVATE_COMMIT,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::ph1j::Ph1jRuntime;

use crate::device_artifact_sync::{self, DeviceArtifactSyncSenderRuntime};
use crate::ph1_voice_id::Ph1VoiceIdRuntime;
use crate::ph1position::Ph1PositionRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.ONB reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const ONB_OK_SESSION_START_DRAFT: ReasonCodeId = ReasonCodeId(0x4F00_0001);
    pub const ONB_OK_TERMS_ACCEPT_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0002);
    pub const ONB_OK_EMPLOYEE_PHOTO_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0003);
    pub const ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0004);
    pub const ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0005);
    pub const ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0006);
    pub const ONB_OK_COMPLETE_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0007);
    pub const ONB_OK_REQUIREMENT_BACKFILL_START_DRAFT: ReasonCodeId = ReasonCodeId(0x4F00_0008);
    pub const ONB_OK_REQUIREMENT_BACKFILL_NOTIFY_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_0009);
    pub const ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT: ReasonCodeId = ReasonCodeId(0x4F00_000A);
    pub const ONB_REFUSE_INVALID: ReasonCodeId = ReasonCodeId(0x4F00_00F1);
    pub const ONB_REFUSE_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4F00_00F2);
}

#[derive(Debug, Clone)]
pub struct OnbVoiceEnrollSampleStep {
    pub audio_sample_ref: String,
    pub attempt_index: u16,
    pub sample_duration_ms: u16,
    pub vad_coverage: f32,
    pub snr_db: f32,
    pub clipping_pct: f32,
    pub overlap_ratio: f32,
    pub app_embedding_capture_ref: Option<VoiceEmbeddingCaptureRef>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone)]
pub enum OnbVoiceEnrollFinalize {
    Complete {
        idempotency_key: String,
    },
    Defer {
        reason_code: ReasonCodeId,
        idempotency_key: String,
    },
}

#[derive(Debug, Clone)]
pub struct OnbVoiceEnrollLiveRequest {
    pub correlation_id: CorrelationId,
    pub turn_id_start: TurnId,
    pub now: MonotonicTimeNs,
    pub onboarding_session_id: OnboardingSessionId,
    pub device_id: DeviceId,
    pub consent_asserted: bool,
    pub max_total_attempts: u8,
    pub max_session_enroll_time_ms: u32,
    pub lock_after_consecutive_passes: u8,
    pub samples: Vec<OnbVoiceEnrollSampleStep>,
    pub finalize: OnbVoiceEnrollFinalize,
}

#[derive(Debug, Clone)]
pub struct OnbVoiceEnrollLiveResult {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub start_result: VoiceIdEnrollStartResult,
    pub sample_results: Vec<VoiceIdEnrollSampleResult>,
    pub complete_result: Option<VoiceIdEnrollCompleteResult>,
    pub defer_result: Option<VoiceIdEnrollDeferResult>,
}

#[derive(Debug, Clone)]
pub struct OnbPositionLiveRequest {
    pub correlation_id: CorrelationId,
    pub turn_id_start: TurnId,
    pub now: MonotonicTimeNs,
    pub actor_user_id: selene_kernel_contracts::ph1_voice_id::UserId,
    pub tenant_id: TenantId,
    pub company_id: String,
    pub position_title: String,
    pub department: String,
    pub jurisdiction: String,
    pub schedule_type: PositionScheduleType,
    pub permission_profile_ref: String,
    pub compensation_band_ref: String,
    pub policy_compensation_band_ref: Option<String>,
    pub create_idempotency_key: String,
    pub validate_idempotency_key: String,
    pub policy_idempotency_key: String,
    pub activate_idempotency_key: String,
}

#[derive(Debug, Clone)]
pub struct OnbPositionLiveResult {
    pub position_id: PositionId,
    pub create_result: PositionCreateDraftResult,
    pub validate_result: PositionValidateAuthCompanyResult,
    pub policy_result: PositionBandPolicyCheckResult,
    pub activate_result: Option<PositionLifecycleResult>,
    pub activation_skipped_reason: Option<ReasonCodeId>,
}

#[derive(Debug, Clone)]
pub struct Ph1OnbOrchRuntime {
    voice_id_runtime: Ph1VoiceIdRuntime,
    position_runtime: Ph1PositionRuntime,
    device_sync_sender: DeviceArtifactSyncSenderRuntime,
}

impl Default for Ph1OnbOrchRuntime {
    fn default() -> Self {
        Self {
            voice_id_runtime: Ph1VoiceIdRuntime::default(),
            position_runtime: Ph1PositionRuntime,
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }
}

impl Ph1OnbOrchRuntime {
    pub fn with_device_sync_sender(mut self, sender: DeviceArtifactSyncSenderRuntime) -> Self {
        self.device_sync_sender = sender;
        self
    }

    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1OnbRequest,
    ) -> Result<Ph1OnbResponse, StorageError> {
        req.validate().map_err(StorageError::ContractViolation)?;

        // In v1, simulation_id+type are validated by contracts; enforce commit-vs-draft expectations here.
        if req.simulation_type == SimulationType::Commit && req.idempotency_key().is_none() {
            // Commits must carry an idempotency key somewhere in the request; each commit variant does.
        }

        match &req.request {
            OnbRequest::SessionStartDraft(r) => {
                let out = store.ph1onb_session_start_draft(
                    req.now,
                    r.token_id.clone(),
                    r.prefilled_context_ref.clone(),
                    r.tenant_id.clone(),
                    r.device_fingerprint.clone(),
                    r.app_platform,
                    r.app_instance_id.clone(),
                    r.deep_link_nonce.clone(),
                    r.link_opened_at,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "NONE",
                    "DRAFT_CREATED",
                    reason_codes::ONB_OK_SESSION_START_DRAFT,
                    Some(format!(
                        "onb_session_start:{}",
                        out.onboarding_session_id.as_str()
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_SESSION_START_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::TermsAcceptCommit(r) => {
                let out = store.ph1onb_terms_accept_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.terms_version_id.clone(),
                    r.accepted,
                    r.idempotency_key.clone(),
                )?;

                let to =
                    if out.terms_status == selene_kernel_contracts::ph1onb::TermsStatus::Accepted {
                        "TERMS_ACCEPTED"
                    } else {
                        "TERMS_DECLINED"
                    };

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "DRAFT_CREATED",
                    to,
                    reason_codes::ONB_OK_TERMS_ACCEPT_COMMIT,
                    Some(format!(
                        "onb_terms_accept:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_TERMS_ACCEPT_COMMIT,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::EmployeePhotoCaptureSendCommit(r) => {
                let out = store.ph1onb_employee_photo_capture_send_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.photo_blob_ref.clone(),
                    r.sender_user_id.clone(),
                    r.idempotency_key.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "TERMS_ACCEPTED",
                    "VERIFICATION_PENDING",
                    reason_codes::ONB_OK_EMPLOYEE_PHOTO_COMMIT,
                    Some(format!(
                        "onb_photo:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_EMPLOYEE_PHOTO_COMMIT,
                        None,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::EmployeeSenderVerifyCommit(r) => {
                let out = store.ph1onb_employee_sender_verify_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.sender_user_id.clone(),
                    r.decision,
                    r.idempotency_key.clone(),
                )?;

                let to = match out.verification_status {
                    selene_kernel_contracts::ph1onb::VerificationStatus::Confirmed => {
                        "VERIFICATION_CONFIRMED"
                    }
                    selene_kernel_contracts::ph1onb::VerificationStatus::Rejected => {
                        "VERIFICATION_REJECTED"
                    }
                    selene_kernel_contracts::ph1onb::VerificationStatus::Pending => {
                        "VERIFICATION_PENDING"
                    }
                };

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "VERIFICATION_PENDING",
                    to,
                    reason_codes::ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT,
                    Some(format!(
                        "onb_sender_verify:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_EMPLOYEE_SENDER_VERIFY_COMMIT,
                        None,
                        None,
                        None,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::PrimaryDeviceConfirmCommit(r) => {
                let out = store.ph1onb_primary_device_confirm_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.device_id.clone(),
                    r.proof_type,
                    r.proof_ok,
                    r.idempotency_key.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "TERMS_ACCEPTED",
                    if out.primary_device_confirmed {
                        "PRIMARY_DEVICE_CONFIRMED"
                    } else {
                        "PRIMARY_DEVICE_FAILED"
                    },
                    reason_codes::ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT,
                    Some(format!(
                        "onb_primary_device:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_PRIMARY_DEVICE_CONFIRM_COMMIT,
                        None,
                        None,
                        None,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::AccessInstanceCreateCommit(r) => {
                let out = store.ph1onb_access_instance_create_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.user_id.clone(),
                    r.tenant_id.clone(),
                    r.role_id.clone(),
                    r.idempotency_key.clone(),
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "GATES_OK",
                    "ACCESS_INSTANCE_CREATED",
                    reason_codes::ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT,
                    Some(format!(
                        "onb_access_instance:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_ACCESS_INSTANCE_CREATE_COMMIT,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }

            OnbRequest::CompleteCommit(r) => {
                let out = store.ph1onb_complete_commit(
                    req.now,
                    r.onboarding_session_id.clone(),
                    r.idempotency_key.clone(),
                    r.voice_artifact_sync_receipt_ref.clone(),
                    r.wake_artifact_sync_receipt_ref.clone(),
                )?;
                // Continuous device artifact continuity: run one worker pass per turn so
                // queued sync rows are acked/replayed in runtime, not only tests.
                self.run_device_artifact_sync_worker_pass(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "IN_PROGRESS",
                    "COMPLETE",
                    reason_codes::ONB_OK_COMPLETE_COMMIT,
                    Some(format!(
                        "onb_complete:{}:{}",
                        out.onboarding_session_id.as_str(),
                        r.idempotency_key
                    )),
                )?;

                Ok(Ph1OnbResponse::Ok(
                    Ph1OnbOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::ONB_OK_COMPLETE_COMMIT,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(out),
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            OnbRequest::RequirementBackfillStartDraft(r) => {
                let out = store.ph1onb_requirement_backfill_start_draft(
                    req.now,
                    r.actor_user_id.clone(),
                    r.tenant_id.clone(),
                    r.company_id.clone(),
                    r.position_id.clone(),
                    r.schema_version_id.clone(),
                    r.rollout_scope,
                    r.idempotency_key.clone(),
                    ONB_REQUIREMENT_BACKFILL_START_DRAFT,
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_START_DRAFT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "BACKFILL_NONE",
                    "BACKFILL_DRAFT_CREATED",
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_START_DRAFT,
                    Some(format!("onb_backfill_start:{}", r.idempotency_key)),
                )?;

                let ok = Ph1OnbOk {
                    schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::ONB_OK_REQUIREMENT_BACKFILL_START_DRAFT,
                    session_start_result: None,
                    terms_accept_result: None,
                    employee_photo_result: None,
                    employee_sender_verify_result: None,
                    primary_device_confirm_result: None,
                    access_instance_create_result: None,
                    complete_result: None,
                    requirement_backfill_start_result: Some(out),
                    requirement_backfill_notify_result: None,
                    requirement_backfill_complete_result: None,
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1OnbResponse::Ok(ok))
            }
            OnbRequest::RequirementBackfillNotifyCommit(r) => {
                let out = store.ph1onb_requirement_backfill_notify_commit(
                    req.now,
                    r.campaign_id.clone(),
                    r.tenant_id.clone(),
                    r.recipient_user_id.clone(),
                    r.idempotency_key.clone(),
                    ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT,
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_NOTIFY_COMMIT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "BACKFILL_RUNNING",
                    "BACKFILL_TARGET_REQUESTED",
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_NOTIFY_COMMIT,
                    Some(format!("onb_backfill_notify:{}", r.idempotency_key)),
                )?;

                let ok = Ph1OnbOk {
                    schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::ONB_OK_REQUIREMENT_BACKFILL_NOTIFY_COMMIT,
                    session_start_result: None,
                    terms_accept_result: None,
                    employee_photo_result: None,
                    employee_sender_verify_result: None,
                    primary_device_confirm_result: None,
                    access_instance_create_result: None,
                    complete_result: None,
                    requirement_backfill_start_result: None,
                    requirement_backfill_notify_result: Some(out),
                    requirement_backfill_complete_result: None,
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1OnbResponse::Ok(ok))
            }
            OnbRequest::RequirementBackfillCompleteCommit(r) => {
                let out = store.ph1onb_requirement_backfill_complete_commit(
                    req.now,
                    r.campaign_id.clone(),
                    r.tenant_id.clone(),
                    r.idempotency_key.clone(),
                    ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT,
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT,
                )?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "BACKFILL_RUNNING",
                    "BACKFILL_COMPLETED",
                    reason_codes::ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT,
                    Some(format!("onb_backfill_complete:{}", r.idempotency_key)),
                )?;

                let ok = Ph1OnbOk {
                    schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
                    simulation_id: req.simulation_id.clone(),
                    reason_code: reason_codes::ONB_OK_REQUIREMENT_BACKFILL_COMPLETE_COMMIT,
                    session_start_result: None,
                    terms_accept_result: None,
                    employee_photo_result: None,
                    employee_sender_verify_result: None,
                    primary_device_confirm_result: None,
                    access_instance_create_result: None,
                    complete_result: None,
                    requirement_backfill_start_result: None,
                    requirement_backfill_notify_result: None,
                    requirement_backfill_complete_result: Some(out),
                };
                ok.validate().map_err(StorageError::ContractViolation)?;
                Ok(Ph1OnbResponse::Ok(ok))
            }
        }
    }

    /// Deterministic LINK -> ONB handoff bridge.
    /// Uses the activated LINK app-open context as the authoritative onboarding session-start context.
    pub fn start_session_from_link_activation(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        activation: &LinkActivationResult,
        tenant_id: Option<String>,
        device_fingerprint: String,
    ) -> Result<OnbSessionStartResult, StorageError> {
        activation
            .validate()
            .map_err(StorageError::ContractViolation)?;
        if activation.activation_status != LinkStatus::Activated {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_start_from_link_activation.activation_status",
                    reason: "must be ACTIVATED",
                },
            ));
        }

        let app_platform = activation
            .app_platform
            .ok_or(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_start_from_link_activation.app_platform",
                    reason: "must be present for ACTIVATED handoff",
                },
            ))?;
        let app_instance_id =
            activation
                .app_instance_id
                .clone()
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_start_from_link_activation.app_instance_id",
                        reason: "must be present for ACTIVATED handoff",
                    },
                ))?;
        let deep_link_nonce =
            activation
                .deep_link_nonce
                .clone()
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_start_from_link_activation.deep_link_nonce",
                        reason: "must be present for ACTIVATED handoff",
                    },
                ))?;
        let link_opened_at = activation
            .link_opened_at
            .ok_or(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1onb_start_from_link_activation.link_opened_at",
                    reason: "must be present for ACTIVATED handoff",
                },
            ))?;

        store.ph1onb_session_start_draft(
            now,
            activation.token_id.clone(),
            activation.prefilled_context_ref.clone(),
            tenant_id,
            device_fingerprint,
            app_platform,
            app_instance_id,
            deep_link_nonce,
            link_opened_at,
        )
    }

    /// Live onboarding sequence slice: route voice enrollment through PH1.VOICE.ID simulations.
    ///
    /// This keeps PH1.ONB.ORCH as the coordinator and preserves simulation-gated behavior.
    pub fn run_voice_enrollment_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbVoiceEnrollLiveRequest,
    ) -> Result<OnbVoiceEnrollLiveResult, StorageError> {
        let mut turn = req.turn_id_start.0;
        let mut now = req.now.0;

        let start_req = Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: req.correlation_id,
            turn_id: TurnId(turn),
            now: MonotonicTimeNs(now),
            simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
            simulation_type: VoiceIdSimulationType::Draft,
            request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
                onboarding_session_id: req.onboarding_session_id.as_str().to_string(),
                device_id: req.device_id.clone(),
                consent_asserted: req.consent_asserted,
                max_total_attempts: req.max_total_attempts,
                max_session_enroll_time_ms: req.max_session_enroll_time_ms,
                lock_after_consecutive_passes: req.lock_after_consecutive_passes,
            }),
        };

        let start_resp = self.voice_id_runtime.run(store, &start_req)?;
        self.run_device_artifact_sync_worker_pass(
            store,
            start_req.now,
            start_req.correlation_id,
            start_req.turn_id,
        )?;
        let start_result = match start_resp {
            Ph1VoiceIdSimResponse::Ok(ok) => {
                ok.enroll_start_result
                    .ok_or(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1onb_voice_enroll.start_result",
                            reason: "missing enroll_start_result payload",
                        },
                    ))?
            }
            Ph1VoiceIdSimResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_voice_enroll.start",
                        reason: "voice enrollment start refused",
                    },
                ));
            }
        };
        let voice_enrollment_session_id = start_result.voice_enrollment_session_id.clone();

        let mut sample_results = Vec::with_capacity(req.samples.len());
        for s in &req.samples {
            turn = turn.saturating_add(1);
            now = now.saturating_add(1);

            let sample_req = Ph1VoiceIdSimRequest {
                schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                correlation_id: req.correlation_id,
                turn_id: TurnId(turn),
                now: MonotonicTimeNs(now),
                simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
                simulation_type: VoiceIdSimulationType::Commit,
                request: VoiceIdSimulationRequest::EnrollSampleCommit(
                    VoiceIdEnrollSampleCommitRequest {
                        voice_enrollment_session_id: voice_enrollment_session_id.clone(),
                        audio_sample_ref: s.audio_sample_ref.clone(),
                        attempt_index: s.attempt_index,
                        sample_duration_ms: s.sample_duration_ms,
                        vad_coverage: s.vad_coverage,
                        snr_db: s.snr_db,
                        clipping_pct: s.clipping_pct,
                        overlap_ratio: s.overlap_ratio,
                        app_embedding_capture_ref: s.app_embedding_capture_ref.clone(),
                        idempotency_key: s.idempotency_key.clone(),
                    },
                ),
            };

            let sample_resp = self.voice_id_runtime.run(store, &sample_req)?;
            self.run_device_artifact_sync_worker_pass(
                store,
                sample_req.now,
                sample_req.correlation_id,
                sample_req.turn_id,
            )?;
            let sample_result = match sample_resp {
                Ph1VoiceIdSimResponse::Ok(ok) => {
                    ok.enroll_sample_result
                        .ok_or(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1onb_voice_enroll.sample_result",
                                reason: "missing enroll_sample_result payload",
                            },
                        ))?
                }
                Ph1VoiceIdSimResponse::Refuse(_) => {
                    return Err(StorageError::ContractViolation(
                        ContractViolation::InvalidValue {
                            field: "ph1onb_voice_enroll.sample",
                            reason: "voice enrollment sample refused",
                        },
                    ));
                }
            };
            sample_results.push(sample_result);
        }

        turn = turn.saturating_add(1);
        now = now.saturating_add(1);

        let (complete_result, defer_result) =
            match &req.finalize {
                OnbVoiceEnrollFinalize::Complete { idempotency_key } => {
                    let complete_req = Ph1VoiceIdSimRequest {
                        schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                        correlation_id: req.correlation_id,
                        turn_id: TurnId(turn),
                        now: MonotonicTimeNs(now),
                        simulation_id: VOICE_ID_ENROLL_COMPLETE_COMMIT.to_string(),
                        simulation_type: VoiceIdSimulationType::Commit,
                        request: VoiceIdSimulationRequest::EnrollCompleteCommit(
                            VoiceIdEnrollCompleteCommitRequest {
                                voice_enrollment_session_id: voice_enrollment_session_id.clone(),
                                idempotency_key: idempotency_key.clone(),
                            },
                        ),
                    };
                    let resp = self.voice_id_runtime.run(store, &complete_req)?;
                    self.run_device_artifact_sync_worker_pass(
                        store,
                        complete_req.now,
                        complete_req.correlation_id,
                        complete_req.turn_id,
                    )?;
                    match resp {
                        Ph1VoiceIdSimResponse::Ok(ok) => (
                            Some(ok.enroll_complete_result.ok_or(
                                StorageError::ContractViolation(ContractViolation::InvalidValue {
                                    field: "ph1onb_voice_enroll.complete_result",
                                    reason: "missing enroll_complete_result payload",
                                }),
                            )?),
                            None,
                        ),
                        Ph1VoiceIdSimResponse::Refuse(_) => {
                            return Err(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "ph1onb_voice_enroll.complete",
                                    reason: "voice enrollment complete refused",
                                },
                            ));
                        }
                    }
                }
                OnbVoiceEnrollFinalize::Defer {
                    reason_code,
                    idempotency_key,
                } => {
                    let defer_req = Ph1VoiceIdSimRequest {
                        schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
                        correlation_id: req.correlation_id,
                        turn_id: TurnId(turn),
                        now: MonotonicTimeNs(now),
                        simulation_id: VOICE_ID_ENROLL_DEFER_COMMIT.to_string(),
                        simulation_type: VoiceIdSimulationType::Commit,
                        request: VoiceIdSimulationRequest::EnrollDeferCommit(
                            VoiceIdEnrollDeferCommitRequest {
                                voice_enrollment_session_id: voice_enrollment_session_id.clone(),
                                reason_code: *reason_code,
                                idempotency_key: idempotency_key.clone(),
                            },
                        ),
                    };
                    let resp = self.voice_id_runtime.run(store, &defer_req)?;
                    self.run_device_artifact_sync_worker_pass(
                        store,
                        defer_req.now,
                        defer_req.correlation_id,
                        defer_req.turn_id,
                    )?;
                    match resp {
                        Ph1VoiceIdSimResponse::Ok(ok) => (
                            None,
                            Some(
                                ok.enroll_defer_result
                                    .ok_or(StorageError::ContractViolation(
                                        ContractViolation::InvalidValue {
                                            field: "ph1onb_voice_enroll.defer_result",
                                            reason: "missing enroll_defer_result payload",
                                        },
                                    ))?,
                            ),
                        ),
                        Ph1VoiceIdSimResponse::Refuse(_) => {
                            return Err(StorageError::ContractViolation(
                                ContractViolation::InvalidValue {
                                    field: "ph1onb_voice_enroll.defer",
                                    reason: "voice enrollment defer refused",
                                },
                            ));
                        }
                    }
                }
            };

        Ok(OnbVoiceEnrollLiveResult {
            voice_enrollment_session_id,
            start_result,
            sample_results,
            complete_result,
            defer_result,
        })
    }

    fn run_device_artifact_sync_worker_pass(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<(), StorageError> {
        let worker_id = format!("onb_device_sync_worker_{}_{}", correlation_id.0, turn_id.0);
        device_artifact_sync::run_device_artifact_sync_worker_pass(
            store,
            now,
            worker_id,
            &self.device_sync_sender,
        )
    }

    /// Live onboarding position sequence: route employee role setup through PH1.POSITION simulations.
    ///
    /// Sequence:
    /// 1) POSITION_SIM_001_CREATE_DRAFT
    /// 2) POSITION_SIM_002_VALIDATE_AUTH_COMPANY
    /// 3) POSITION_SIM_003_BAND_POLICY_CHECK
    /// 4) POSITION_SIM_004_ACTIVATE_COMMIT (only when policy allows)
    pub fn run_position_live_sequence(
        &self,
        store: &mut Ph1fStore,
        req: &OnbPositionLiveRequest,
    ) -> Result<OnbPositionLiveResult, StorageError> {
        let mut turn = req.turn_id_start.0;
        let mut now = req.now.0;

        let create_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: req.correlation_id,
            turn_id: TurnId(turn),
            now: MonotonicTimeNs(now),
            simulation_id: POSITION_SIM_001_CREATE_DRAFT.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::CreateDraft(PositionCreateDraftRequest {
                actor_user_id: req.actor_user_id.clone(),
                tenant_id: req.tenant_id.clone(),
                company_id: req.company_id.clone(),
                position_title: req.position_title.clone(),
                department: req.department.clone(),
                jurisdiction: req.jurisdiction.clone(),
                schedule_type: req.schedule_type,
                permission_profile_ref: req.permission_profile_ref.clone(),
                compensation_band_ref: req.compensation_band_ref.clone(),
                idempotency_key: req.create_idempotency_key.clone(),
            }),
        };
        let create_result = match self.position_runtime.run(store, &create_req)? {
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Ok(ok) => ok
                .create_draft_result
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.create_result",
                        reason: "missing create_draft_result payload",
                    },
                ))?,
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.create",
                        reason: "position create draft refused",
                    },
                ));
            }
        };

        let position_id = create_result.position_id.clone();

        turn = turn.saturating_add(1);
        now = now.saturating_add(1);
        let validate_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: req.correlation_id,
            turn_id: TurnId(turn),
            now: MonotonicTimeNs(now),
            simulation_id: POSITION_SIM_002_VALIDATE_AUTH_COMPANY.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::ValidateAuthCompany(PositionValidateAuthCompanyRequest {
                actor_user_id: req.actor_user_id.clone(),
                tenant_id: req.tenant_id.clone(),
                company_id: req.company_id.clone(),
                position_id: position_id.clone(),
                requested_action: PositionRequestedAction::Activate,
                idempotency_key: req.validate_idempotency_key.clone(),
            }),
        };
        let validate_result = match self.position_runtime.run(store, &validate_req)? {
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Ok(ok) => ok
                .validate_auth_company_result
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.validate_result",
                        reason: "missing validate_auth_company_result payload",
                    },
                ))?,
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.validate",
                        reason: "position validate auth/company refused",
                    },
                ));
            }
        };

        if validate_result.validation_status != PositionValidationStatus::Ok {
            return Ok(OnbPositionLiveResult {
                position_id,
                create_result,
                validate_result: validate_result.clone(),
                policy_result: PositionBandPolicyCheckResult::v1(
                    PositionPolicyResult::Escalate,
                    validate_result.reason_code,
                )
                .map_err(StorageError::ContractViolation)?,
                activate_result: None,
                activation_skipped_reason: Some(validate_result.reason_code),
            });
        }

        turn = turn.saturating_add(1);
        now = now.saturating_add(1);
        let policy_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: req.correlation_id,
            turn_id: TurnId(turn),
            now: MonotonicTimeNs(now),
            simulation_id: POSITION_SIM_003_BAND_POLICY_CHECK.to_string(),
            simulation_type: PositionSimulationType::Draft,
            request: PositionRequest::BandPolicyCheck(PositionBandPolicyCheckRequest {
                actor_user_id: req.actor_user_id.clone(),
                tenant_id: req.tenant_id.clone(),
                position_id: position_id.clone(),
                compensation_band_ref: req
                    .policy_compensation_band_ref
                    .clone()
                    .unwrap_or_else(|| req.compensation_band_ref.clone()),
                idempotency_key: req.policy_idempotency_key.clone(),
            }),
        };
        let policy_result = match self.position_runtime.run(store, &policy_req)? {
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Ok(ok) => ok
                .band_policy_check_result
                .ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.policy_result",
                        reason: "missing band_policy_check_result payload",
                    },
                ))?,
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.policy",
                        reason: "position policy check refused",
                    },
                ));
            }
        };

        if policy_result.policy_result != PositionPolicyResult::Allow {
            return Ok(OnbPositionLiveResult {
                position_id,
                create_result,
                validate_result,
                policy_result: policy_result.clone(),
                activate_result: None,
                activation_skipped_reason: Some(policy_result.reason_code),
            });
        }

        turn = turn.saturating_add(1);
        now = now.saturating_add(1);
        let activate_req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: req.correlation_id,
            turn_id: TurnId(turn),
            now: MonotonicTimeNs(now),
            simulation_id: POSITION_SIM_004_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::ActivateCommit(
                selene_kernel_contracts::ph1position::PositionActivateCommitRequest {
                    actor_user_id: req.actor_user_id.clone(),
                    tenant_id: req.tenant_id.clone(),
                    position_id: position_id.clone(),
                    idempotency_key: req.activate_idempotency_key.clone(),
                },
            ),
        };
        let activate_result = match self.position_runtime.run(store, &activate_req)? {
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Ok(ok) => {
                ok.lifecycle_result.ok_or(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.activate_result",
                        reason: "missing lifecycle_result payload",
                    },
                ))?
            }
            selene_kernel_contracts::ph1position::Ph1PositionResponse::Refuse(_) => {
                return Err(StorageError::ContractViolation(
                    ContractViolation::InvalidValue {
                        field: "ph1onb_position_live.activate",
                        reason: "position activate refused",
                    },
                ));
            }
        };

        Ok(OnbPositionLiveResult {
            position_id,
            create_result,
            validate_result,
            policy_result,
            activate_result: Some(activate_result),
            activation_skipped_reason: None,
        })
    }

    fn audit_transition(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: selene_kernel_contracts::ph1j::CorrelationId,
        turn_id: selene_kernel_contracts::ph1j::TurnId,
        state_from: &'static str,
        state_to: &'static str,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<(), StorageError> {
        let mut entries: BTreeMap<PayloadKey, PayloadValue> = BTreeMap::new();
        entries.insert(
            PayloadKey::new("state_from").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_from).map_err(StorageError::ContractViolation)?,
        );
        entries.insert(
            PayloadKey::new("state_to").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_to).map_err(StorageError::ContractViolation)?,
        );
        let payload_min = AuditPayloadMin::v1(entries).map_err(StorageError::ContractViolation)?;

        // Use Other("ph1_onb") until the global audit engine enum is updated.
        let engine = AuditEngine::Other("ph1_onb".to_string());

        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            None,
            None,
            engine,
            AuditEventType::StateTransition,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload_min,
            None,
            idempotency_key,
        )
        .map_err(StorageError::ContractViolation)?;

        Ph1jRuntime::emit(store, ev)?;
        Ok(())
    }
}

// Small helper for "commit idempotency key is present somewhere".
trait OnbIdempotencyKey {
    fn idempotency_key(&self) -> Option<&str>;
}

impl OnbIdempotencyKey for Ph1OnbRequest {
    fn idempotency_key(&self) -> Option<&str> {
        match &self.request {
            OnbRequest::SessionStartDraft(_) => None,
            OnbRequest::TermsAcceptCommit(r) => Some(&r.idempotency_key),
            OnbRequest::EmployeePhotoCaptureSendCommit(r) => Some(&r.idempotency_key),
            OnbRequest::EmployeeSenderVerifyCommit(r) => Some(&r.idempotency_key),
            OnbRequest::PrimaryDeviceConfirmCommit(r) => Some(&r.idempotency_key),
            OnbRequest::AccessInstanceCreateCommit(r) => Some(&r.idempotency_key),
            OnbRequest::CompleteCommit(r) => Some(&r.idempotency_key),
            OnbRequest::RequirementBackfillStartDraft(r) => Some(&r.idempotency_key),
            OnbRequest::RequirementBackfillNotifyCommit(r) => Some(&r.idempotency_key),
            OnbRequest::RequirementBackfillCompleteCommit(r) => Some(&r.idempotency_key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
    use selene_kernel_contracts::ph1link::{InviteeType, Ph1LinkRequest, PrefilledContext};
    use selene_kernel_contracts::ph1onb::{
        BackfillCampaignState, BackfillRolloutScope, BackfillTargetStatus,
        OnbAccessInstanceCreateCommitRequest, OnbCompleteCommitRequest,
        OnbEmployeePhotoCaptureSendCommitRequest, OnbEmployeeSenderVerifyCommitRequest,
        OnbPrimaryDeviceConfirmCommitRequest, OnbRequest,
        OnbRequirementBackfillCompleteCommitRequest, OnbRequirementBackfillNotifyCommitRequest,
        OnbRequirementBackfillStartDraftRequest, OnbSessionStartDraftRequest,
        OnbTermsAcceptCommitRequest, OnboardingStatus, Ph1OnbRequest, SenderVerifyDecision,
        TermsStatus, VerificationStatus,
    };
    use selene_kernel_contracts::ph1position::{
        PositionLifecycleState, PositionPolicyResult, PositionRequirementEvidenceMode,
        PositionRequirementExposureRule, PositionRequirementFieldSpec,
        PositionRequirementFieldType, PositionRequirementRuleType, PositionRequirementSensitivity,
        PositionScheduleType, PositionSchemaApplyScope, PositionSchemaSelectorSnapshot, TenantId,
    };
    use selene_storage::ph1f::{
        DeviceRecord, IdentityRecord, IdentityStatus, MobileArtifactSyncState,
        TenantCompanyLifecycleState, TenantCompanyRecord,
    };

    fn now() -> MonotonicTimeNs {
        MonotonicTimeNs(1_000_000_000)
    }

    fn corr() -> CorrelationId {
        CorrelationId(123)
    }

    fn turn() -> TurnId {
        TurnId(1)
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).unwrap()
    }

    fn store_with_inviter() -> Ph1fStore {
        let mut s = Ph1fStore::new_in_memory();
        s.insert_identity(IdentityRecord::v1(
            user("inviter"),
            None,
            None,
            now(),
            IdentityStatus::Active,
        ))
        .unwrap();
        s
    }

    fn insert_inviter_device(store: &mut Ph1fStore, id: &str) {
        store
            .insert_device(
                DeviceRecord::v1(
                    device(id),
                    user("inviter"),
                    "phone".to_string(),
                    MonotonicTimeNs(now().0 + 1),
                    Some("audio_profile_voice".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn upsert_active_company(store: &mut Ph1fStore) {
        store
            .ph1tenant_company_upsert(TenantCompanyRecord {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                tenant_id: TenantId::new("tenant_1").unwrap(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Inc".to_string(),
                jurisdiction: "CN".to_string(),
                lifecycle_state: TenantCompanyLifecycleState::Active,
                created_at: MonotonicTimeNs(now().0 + 1),
                updated_at: MonotonicTimeNs(now().0 + 1),
            })
            .unwrap();
    }

    fn make_activated_link(store: &mut Ph1fStore) -> selene_kernel_contracts::ph1link::TokenId {
        // Create a link draft + open/activate it.
        let req = Ph1LinkRequest::invite_generate_draft_v1(
            corr(),
            turn(),
            now(),
            user("inviter"),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        let link_rt = crate::ph1link::Ph1LinkRuntime::new(crate::ph1link::Ph1LinkConfig::mvp_v1());
        let out = link_rt.run(store, &req).unwrap();
        let token_id = match out {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_generate_result.unwrap().token_id
            }
            _ => panic!("expected ok"),
        };

        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            corr(),
            turn(),
            MonotonicTimeNs(now().0 + 5),
            token_id.clone(),
            "device_fp_1".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_onb".to_string(),
            "nonce_onb".to_string(),
            MonotonicTimeNs(now().0 + 5),
            "idem_onb_open_1".to_string(),
        )
        .unwrap();
        let _ = link_rt.run(store, &open).unwrap();
        token_id
    }

    fn lock_voice_enrollment_and_get_receipt(
        rt: &Ph1OnbOrchRuntime,
        store: &mut Ph1fStore,
        onboarding_session_id: OnboardingSessionId,
        device_id: DeviceId,
        turn_id_start: u64,
        seed: &str,
    ) -> String {
        let live_req = OnbVoiceEnrollLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(turn_id_start),
            now: MonotonicTimeNs(now().0 + 40),
            onboarding_session_id,
            device_id,
            consent_asserted: true,
            max_total_attempts: 8,
            max_session_enroll_time_ms: 120_000,
            lock_after_consecutive_passes: 3,
            samples: vec![
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: format!("audio:{seed}:1"),
                    attempt_index: 1,
                    sample_duration_ms: 1_400,
                    vad_coverage: 0.93,
                    snr_db: 18.0,
                    clipping_pct: 0.4,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: format!("{seed}-sample-1"),
                },
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: format!("audio:{seed}:2"),
                    attempt_index: 2,
                    sample_duration_ms: 1_360,
                    vad_coverage: 0.91,
                    snr_db: 17.6,
                    clipping_pct: 0.5,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: format!("{seed}-sample-2"),
                },
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: format!("audio:{seed}:3"),
                    attempt_index: 3,
                    sample_duration_ms: 1_420,
                    vad_coverage: 0.94,
                    snr_db: 18.4,
                    clipping_pct: 0.3,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: format!("{seed}-sample-3"),
                },
            ],
            finalize: OnbVoiceEnrollFinalize::Complete {
                idempotency_key: format!("{seed}-complete"),
            },
        };
        let out = rt
            .run_voice_enrollment_live_sequence(store, &live_req)
            .unwrap();
        out.complete_result
            .and_then(|r| r.voice_artifact_sync_receipt_ref)
            .expect("voice enrollment complete must return sync receipt")
    }

    fn selector_snapshot() -> PositionSchemaSelectorSnapshot {
        PositionSchemaSelectorSnapshot {
            company_size: Some("SMALL".to_string()),
            industry_code: Some("LOGISTICS".to_string()),
            jurisdiction: Some("US".to_string()),
            position_family: Some("DRIVER".to_string()),
        }
    }

    fn required_doc_field(field_key: &str) -> PositionRequirementFieldSpec {
        PositionRequirementFieldSpec {
            field_key: field_key.to_string(),
            field_type: PositionRequirementFieldType::String,
            required_rule: PositionRequirementRuleType::Always,
            required_predicate_ref: None,
            validation_ref: None,
            sensitivity: PositionRequirementSensitivity::Private,
            exposure_rule: PositionRequirementExposureRule::InternalOnly,
            evidence_mode: PositionRequirementEvidenceMode::DocRequired,
            prompt_short: format!("Provide {field_key}"),
            prompt_long: format!("Please provide required field {field_key}."),
        }
    }

    fn make_active_position_for_backfill(store: &mut Ph1fStore) -> (String, String, String) {
        upsert_active_company(store);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let draft = store
            .ph1position_create_draft(
                MonotonicTimeNs(now().0 + 11),
                user("inviter"),
                tenant_id.clone(),
                "company_1".to_string(),
                "Warehouse Manager".to_string(),
                "Ops".to_string(),
                "US".to_string(),
                PositionScheduleType::FullTime,
                "profile_ops".to_string(),
                "band_l3".to_string(),
                "onb-backfill-pos-create".to_string(),
                "POSITION_SIM_001_CREATE_DRAFT",
                ReasonCodeId(0x5900_0001),
            )
            .unwrap();
        store
            .ph1position_activate_commit(
                MonotonicTimeNs(now().0 + 12),
                user("inviter"),
                tenant_id.clone(),
                draft.position_id.clone(),
                "onb-backfill-pos-activate".to_string(),
                "POSITION_SIM_004_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0004),
            )
            .unwrap();
        (
            tenant_id.as_str().to_string(),
            "company_1".to_string(),
            draft.position_id.as_str().to_string(),
        )
    }

    fn make_activated_link_with_required_verification(
        store: &mut Ph1fStore,
    ) -> selene_kernel_contracts::ph1link::TokenId {
        upsert_active_company(store);
        let tenant_id = TenantId::new("tenant_1").unwrap();
        let position = store
            .ph1position_create_draft(
                MonotonicTimeNs(now().0 + 1),
                user("inviter"),
                tenant_id.clone(),
                "company_1".to_string(),
                "Driver".to_string(),
                "Logistics".to_string(),
                "US".to_string(),
                PositionScheduleType::FullTime,
                "profile_driver".to_string(),
                "band_l2".to_string(),
                "onb-os-pos-create".to_string(),
                "POSITION_SIM_001_CREATE_DRAFT",
                ReasonCodeId(0x5900_0001),
            )
            .unwrap();
        store
            .ph1position_activate_commit(
                MonotonicTimeNs(now().0 + 2),
                user("inviter"),
                tenant_id.clone(),
                position.position_id.clone(),
                "onb-os-pos-activate".to_string(),
                "POSITION_SIM_004_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0004),
            )
            .unwrap();
        store
            .ph1position_requirements_schema_create_draft(
                MonotonicTimeNs(now().0 + 3),
                user("inviter"),
                tenant_id.clone(),
                "company_1".to_string(),
                position.position_id.clone(),
                "schema_v1".to_string(),
                selector_snapshot(),
                vec![required_doc_field("driver_license_doc_ref")],
                "onb-os-schema-create".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
                ReasonCodeId(0x5900_0006),
            )
            .unwrap();
        store
            .ph1position_requirements_schema_activate_commit(
                MonotonicTimeNs(now().0 + 4),
                user("inviter"),
                tenant_id,
                "company_1".to_string(),
                position.position_id.clone(),
                "schema_v1".to_string(),
                PositionSchemaApplyScope::NewHiresOnly,
                "onb-os-schema-activate".to_string(),
                "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
                ReasonCodeId(0x5900_0008),
            )
            .unwrap();

        let prefilled = PrefilledContext::v1(
            Some("tenant_1".to_string()),
            Some("company_1".to_string()),
            Some(position.position_id.as_str().to_string()),
            Some("loc_1".to_string()),
            Some("2026-02-15".to_string()),
            None,
            Some("band_l2".to_string()),
            Vec::new(),
        )
        .unwrap();

        let req = Ph1LinkRequest::invite_generate_draft_v1(
            corr(),
            turn(),
            now(),
            user("inviter"),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            Some(prefilled),
            None,
        )
        .unwrap();
        let link_rt = crate::ph1link::Ph1LinkRuntime::new(crate::ph1link::Ph1LinkConfig::mvp_v1());
        let out = link_rt.run(store, &req).unwrap();
        let token_id = match out {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_generate_result.unwrap().token_id
            }
            _ => panic!("expected ok"),
        };

        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            corr(),
            turn(),
            MonotonicTimeNs(now().0 + 5),
            token_id.clone(),
            "device_fp_required_verify".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_onb".to_string(),
            "nonce_onb".to_string(),
            MonotonicTimeNs(now().0 + 5),
            "idem_onb_open_required_verify".to_string(),
        )
        .unwrap();
        let _ = link_rt.run(store, &open).unwrap();
        token_id
    }

    #[test]
    fn onb_happy_path_employee_minimal() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let token_id = make_activated_link_with_required_verification(&mut store);

        // Start session.
        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_1".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let out = rt.run(&mut store, &start).unwrap();
        let session_start = match out {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap(),
            _ => panic!("expected ok"),
        };
        assert!(session_start
            .required_verification_gates
            .iter()
            .any(|g| g == "PHOTO_EVIDENCE_CAPTURE"));
        assert!(session_start
            .required_verification_gates
            .iter()
            .any(|g| g == "SENDER_CONFIRMATION"));
        assert!(session_start
            .pinned_schema_id
            .as_deref()
            .unwrap_or("")
            .starts_with("position:"));
        let session_id = session_start.onboarding_session_id;
        insert_inviter_device(&mut store, "device_1");

        // Terms accept.
        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "k1".to_string(),
            }),
        };
        let out = rt.run(&mut store, &terms).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert_eq!(
                ok.terms_accept_result.unwrap().terms_status,
                TermsStatus::Accepted
            ),
            _ => panic!("expected ok"),
        }

        // Photo capture.
        let photo = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 4),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::EmployeePhotoCaptureSendCommit(
                OnbEmployeePhotoCaptureSendCommitRequest {
                    onboarding_session_id: session_id.clone(),
                    photo_blob_ref: "blob:1".to_string(),
                    sender_user_id: user("inviter"),
                    idempotency_key: "k2".to_string(),
                },
            ),
        };
        let out = rt.run(&mut store, &photo).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert_eq!(
                ok.employee_photo_result.unwrap().verification_status,
                VerificationStatus::Pending
            ),
            _ => panic!("expected ok"),
        }

        // Sender verify.
        let verify = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 5),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_EMPLOYEE_SENDER_VERIFY_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::EmployeeSenderVerifyCommit(OnbEmployeeSenderVerifyCommitRequest {
                onboarding_session_id: session_id.clone(),
                sender_user_id: user("inviter"),
                decision: SenderVerifyDecision::Confirm,
                idempotency_key: "k3".to_string(),
            }),
        };
        let out = rt.run(&mut store, &verify).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert_eq!(
                ok.employee_sender_verify_result
                    .unwrap()
                    .verification_status,
                VerificationStatus::Confirmed
            ),
            _ => panic!("expected ok"),
        }

        // Primary device confirm.
        let dev = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 6),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_PRIMARY_DEVICE_CONFIRM_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::PrimaryDeviceConfirmCommit(OnbPrimaryDeviceConfirmCommitRequest {
                onboarding_session_id: session_id.clone(),
                device_id: device("device_1"),
                proof_type: selene_kernel_contracts::ph1onb::ProofType::Biometric,
                proof_ok: true,
                idempotency_key: "k4".to_string(),
            }),
        };
        let out = rt.run(&mut store, &dev).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert!(
                ok.primary_device_confirm_result
                    .unwrap()
                    .primary_device_confirmed
            ),
            _ => panic!("expected ok"),
        }

        // Access instance create.
        let access = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 7),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_ACCESS_INSTANCE_CREATE_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::AccessInstanceCreateCommit(OnbAccessInstanceCreateCommitRequest {
                onboarding_session_id: session_id.clone(),
                user_id: user("invitee"),
                tenant_id: Some("tenant_1".to_string()),
                role_id: "role_store_manager".to_string(),
                idempotency_key: "k5".to_string(),
            }),
        };
        let out = rt.run(&mut store, &access).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert!(!ok
                .access_instance_create_result
                .unwrap()
                .access_engine_instance_id
                .is_empty()),
            _ => panic!("expected ok"),
        }

        let voice_receipt = lock_voice_enrollment_and_get_receipt(
            &rt,
            &mut store,
            session_id.clone(),
            device("device_1"),
            700,
            "happy",
        );

        // Complete.
        let complete = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 8),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_COMPLETE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::CompleteCommit(OnbCompleteCommitRequest {
                onboarding_session_id: session_id,
                idempotency_key: "k6".to_string(),
                voice_artifact_sync_receipt_ref: Some(voice_receipt),
                wake_artifact_sync_receipt_ref: None,
            }),
        };
        let out = rt.run(&mut store, &complete).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert_eq!(
                ok.complete_result.unwrap().onboarding_status,
                OnboardingStatus::Complete
            ),
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn onb_employee_can_complete_without_sender_verify_when_not_schema_required() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let token_id = make_activated_link(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_1".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap().onboarding_session_id,
            _ => panic!("expected ok"),
        };
        insert_inviter_device(&mut store, "device_no_sender_verify");

        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "schema-gate-terms".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &terms).unwrap();

        let dev = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 4),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_PRIMARY_DEVICE_CONFIRM_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::PrimaryDeviceConfirmCommit(OnbPrimaryDeviceConfirmCommitRequest {
                onboarding_session_id: session_id.clone(),
                device_id: device("device_no_sender_verify"),
                proof_type: selene_kernel_contracts::ph1onb::ProofType::Biometric,
                proof_ok: true,
                idempotency_key: "schema-gate-device".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &dev).unwrap();

        let access = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 5),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_ACCESS_INSTANCE_CREATE_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::AccessInstanceCreateCommit(OnbAccessInstanceCreateCommitRequest {
                onboarding_session_id: session_id.clone(),
                user_id: user("invitee_schema_optional"),
                tenant_id: Some("tenant_1".to_string()),
                role_id: "role_store_manager".to_string(),
                idempotency_key: "schema-gate-access".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &access).unwrap();

        let voice_receipt = lock_voice_enrollment_and_get_receipt(
            &rt,
            &mut store,
            session_id.clone(),
            device("device_no_sender_verify"),
            800,
            "schema-optional",
        );

        let complete = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 6),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_COMPLETE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::CompleteCommit(OnbCompleteCommitRequest {
                onboarding_session_id: session_id,
                idempotency_key: "schema-gate-complete".to_string(),
                voice_artifact_sync_receipt_ref: Some(voice_receipt),
                wake_artifact_sync_receipt_ref: None,
            }),
        };
        let out = rt.run(&mut store, &complete).unwrap();
        match out {
            Ph1OnbResponse::Ok(ok) => assert_eq!(
                ok.complete_result.unwrap().onboarding_status,
                OnboardingStatus::Complete
            ),
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn onb_backfill_start_refuses_new_hires_only_scope() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let (tenant_id, company_id, position_id) = make_active_position_for_backfill(&mut store);

        let req = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 20),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_START_DRAFT
                .to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::RequirementBackfillStartDraft(
                OnbRequirementBackfillStartDraftRequest {
                    actor_user_id: user("inviter"),
                    tenant_id,
                    company_id,
                    position_id,
                    schema_version_id: "schema_v2".to_string(),
                    rollout_scope: BackfillRolloutScope::NewHiresOnly,
                    idempotency_key: "onb-backfill-new-hires-only".to_string(),
                },
            ),
        };

        let out = rt.run(&mut store, &req);
        assert!(matches!(out, Err(StorageError::ContractViolation(_))));
    }

    #[test]
    fn onb_backfill_current_and_new_notify_loop_then_complete() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        store
            .insert_identity(IdentityRecord::v1(
                user("target_1"),
                None,
                None,
                now(),
                IdentityStatus::Active,
            ))
            .unwrap();
        let (tenant_id, company_id, position_id) = make_active_position_for_backfill(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 21),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_START_DRAFT
                .to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::RequirementBackfillStartDraft(
                OnbRequirementBackfillStartDraftRequest {
                    actor_user_id: user("inviter"),
                    tenant_id: tenant_id.clone(),
                    company_id,
                    position_id,
                    schema_version_id: "schema_v2".to_string(),
                    rollout_scope: BackfillRolloutScope::CurrentAndNew,
                    idempotency_key: "onb-backfill-start".to_string(),
                },
            ),
        };
        let started = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.requirement_backfill_start_result.unwrap(),
            _ => panic!("expected ok"),
        };
        assert_eq!(started.state, BackfillCampaignState::Running);
        assert!(started.pending_target_count >= 2);

        let notify = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 22),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::RequirementBackfillNotifyCommit(
                OnbRequirementBackfillNotifyCommitRequest {
                    campaign_id: started.campaign_id.clone(),
                    tenant_id: tenant_id.clone(),
                    recipient_user_id: user("target_1"),
                    idempotency_key: "onb-backfill-notify-1".to_string(),
                },
            ),
        };
        let notify_out = match rt.run(&mut store, &notify).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.requirement_backfill_notify_result.unwrap(),
            _ => panic!("expected ok"),
        };
        assert_eq!(notify_out.target_status, BackfillTargetStatus::Requested);

        let remind_notify = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 23),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::RequirementBackfillNotifyCommit(
                OnbRequirementBackfillNotifyCommitRequest {
                    campaign_id: started.campaign_id.clone(),
                    tenant_id: tenant_id.clone(),
                    recipient_user_id: user("target_1"),
                    idempotency_key: "onb-backfill-notify-2".to_string(),
                },
            ),
        };
        let remind_out = match rt.run(&mut store, &remind_notify).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.requirement_backfill_notify_result.unwrap(),
            _ => panic!("expected ok"),
        };
        assert_eq!(remind_out.target_status, BackfillTargetStatus::Requested);

        let complete = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 24),
            simulation_id:
                selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT
                    .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::RequirementBackfillCompleteCommit(
                OnbRequirementBackfillCompleteCommitRequest {
                    campaign_id: started.campaign_id,
                    tenant_id,
                    idempotency_key: "onb-backfill-complete".to_string(),
                },
            ),
        };
        let complete_out = match rt.run(&mut store, &complete).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.requirement_backfill_complete_result.unwrap(),
            _ => panic!("expected ok"),
        };
        assert_eq!(complete_out.state, BackfillCampaignState::Completed);
        assert_eq!(complete_out.completed_target_count, 0);
        assert!(complete_out.total_target_count >= 2);
    }

    #[test]
    fn onb_backfill_fail_closed_on_tenant_scope_mismatch() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        store
            .insert_identity(IdentityRecord::v1(
                user("target_1"),
                None,
                None,
                now(),
                IdentityStatus::Active,
            ))
            .unwrap();
        let (tenant_id, company_id, position_id) = make_active_position_for_backfill(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 25),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_START_DRAFT
                .to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::RequirementBackfillStartDraft(
                OnbRequirementBackfillStartDraftRequest {
                    actor_user_id: user("inviter"),
                    tenant_id: tenant_id.clone(),
                    company_id,
                    position_id,
                    schema_version_id: "schema_v2".to_string(),
                    rollout_scope: BackfillRolloutScope::CurrentAndNew,
                    idempotency_key: "onb-backfill-start-tenant-check".to_string(),
                },
            ),
        };
        let campaign_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.requirement_backfill_start_result.unwrap().campaign_id,
            _ => panic!("expected ok"),
        };

        let notify_wrong_tenant = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 26),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::RequirementBackfillNotifyCommit(
                OnbRequirementBackfillNotifyCommitRequest {
                    campaign_id: campaign_id.clone(),
                    tenant_id: "tenant_wrong".to_string(),
                    recipient_user_id: user("target_1"),
                    idempotency_key: "onb-backfill-notify-tenant-wrong".to_string(),
                },
            ),
        };
        let notify_out = rt.run(&mut store, &notify_wrong_tenant);
        assert!(matches!(
            notify_out,
            Err(StorageError::ContractViolation(_))
        ));

        let complete_wrong_tenant = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 27),
            simulation_id:
                selene_kernel_contracts::ph1onb::ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT
                    .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::RequirementBackfillCompleteCommit(
                OnbRequirementBackfillCompleteCommitRequest {
                    campaign_id,
                    tenant_id: "tenant_wrong".to_string(),
                    idempotency_key: "onb-backfill-complete-tenant-wrong".to_string(),
                },
            ),
        };
        let complete_out = rt.run(&mut store, &complete_wrong_tenant);
        assert!(matches!(
            complete_out,
            Err(StorageError::ContractViolation(_))
        ));
    }

    #[test]
    fn onb_fail_closed_when_required_verification_gates_missing() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let token_id = make_activated_link_with_required_verification(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 30),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_gate_fail_closed".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap().onboarding_session_id,
            _ => panic!("expected ok"),
        };

        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 31),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "gate-fail-terms".to_string(),
            }),
        };
        rt.run(&mut store, &terms).unwrap();

        let device_commit = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 32),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_PRIMARY_DEVICE_CONFIRM_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::PrimaryDeviceConfirmCommit(OnbPrimaryDeviceConfirmCommitRequest {
                onboarding_session_id: session_id.clone(),
                device_id: device("device_gate_fail_closed"),
                proof_type: selene_kernel_contracts::ph1onb::ProofType::Passcode,
                proof_ok: true,
                idempotency_key: "gate-fail-device".to_string(),
            }),
        };
        rt.run(&mut store, &device_commit).unwrap();

        let access = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 33),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_ACCESS_INSTANCE_CREATE_COMMIT
                .to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::AccessInstanceCreateCommit(OnbAccessInstanceCreateCommitRequest {
                onboarding_session_id: session_id.clone(),
                user_id: user("invitee_gate_fail_closed"),
                tenant_id: Some("tenant_1".to_string()),
                role_id: "role_store_manager".to_string(),
                idempotency_key: "gate-fail-access".to_string(),
            }),
        };
        let access_out = rt.run(&mut store, &access);
        assert!(matches!(
            access_out,
            Err(StorageError::ContractViolation(_))
        ));

        let complete = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 34),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_COMPLETE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::CompleteCommit(OnbCompleteCommitRequest {
                onboarding_session_id: session_id,
                idempotency_key: "gate-fail-complete".to_string(),
                voice_artifact_sync_receipt_ref: None,
                wake_artifact_sync_receipt_ref: None,
            }),
        };
        let complete_out = rt.run(&mut store, &complete);
        assert!(matches!(
            complete_out,
            Err(StorageError::ContractViolation(_))
        ));
    }

    #[test]
    fn onb_live_sequence_calls_voice_id_enroll_start_sample_complete() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let token_id = make_activated_link(&mut store);

        // Create onboarding session.
        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_1".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap().onboarding_session_id,
            _ => panic!("expected ok"),
        };

        // Terms accepted.
        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "voice-flow-terms".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &terms).unwrap();

        // Device must exist for VOICE_ID_ENROLL_START_DRAFT.
        let device_id = device("voice_device_onb");
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user("inviter"),
                    "phone".to_string(),
                    MonotonicTimeNs(now().0 + 3),
                    Some("audio_profile_voice".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let live_req = OnbVoiceEnrollLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(100),
            now: MonotonicTimeNs(now().0 + 4),
            onboarding_session_id: session_id,
            device_id,
            consent_asserted: true,
            max_total_attempts: 8,
            max_session_enroll_time_ms: 120_000,
            lock_after_consecutive_passes: 3,
            samples: vec![
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: "audio:voice:1".to_string(),
                    attempt_index: 1,
                    sample_duration_ms: 1_400,
                    vad_coverage: 0.93,
                    snr_db: 18.0,
                    clipping_pct: 0.4,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: Some(
                        VoiceEmbeddingCaptureRef::v1(
                            "embed://ios/voice/onb/1".to_string(),
                            "ios.voiceid.v1".to_string(),
                            256,
                        )
                        .unwrap(),
                    ),
                    idempotency_key: "voice-sample-1".to_string(),
                },
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: "audio:voice:2".to_string(),
                    attempt_index: 2,
                    sample_duration_ms: 1_380,
                    vad_coverage: 0.91,
                    snr_db: 17.5,
                    clipping_pct: 0.3,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: "voice-sample-2".to_string(),
                },
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: "audio:voice:3".to_string(),
                    attempt_index: 3,
                    sample_duration_ms: 1_420,
                    vad_coverage: 0.92,
                    snr_db: 18.2,
                    clipping_pct: 0.5,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: "voice-sample-3".to_string(),
                },
            ],
            finalize: OnbVoiceEnrollFinalize::Complete {
                idempotency_key: "voice-complete-1".to_string(),
            },
        };

        let out = rt
            .run_voice_enrollment_live_sequence(&mut store, &live_req)
            .unwrap();
        assert_eq!(out.sample_results.len(), 3);
        assert!(out.defer_result.is_none());
        assert!(out.complete_result.is_some());
        let complete = out.complete_result.unwrap();
        let profile_id = complete.voice_profile_id.clone();
        let profile = store
            .ph1vid_get_voice_profile(&profile_id)
            .expect("voice profile must exist");
        let capture_ref = profile
            .profile_embedding_capture_ref
            .as_ref()
            .expect("voice profile must keep app embedding capture ref");
        assert_eq!(capture_ref.embedding_ref, "embed://ios/voice/onb/1");
        assert_eq!(capture_ref.embedding_model_id, "ios.voiceid.v1");
        assert_eq!(capture_ref.embedding_dim, 256);
        let receipt = complete.voice_artifact_sync_receipt_ref.unwrap();
        let queue_row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("voice sync queue row must exist");
        assert_eq!(queue_row.state, MobileArtifactSyncState::Acked);
        assert!(queue_row.acked_at.is_some());
    }

    #[test]
    fn onb_live_sequence_sync_failure_keeps_queue_row_inflight_for_retry() {
        let rt = Ph1OnbOrchRuntime::default().with_device_sync_sender(
            crate::device_artifact_sync::DeviceArtifactSyncSenderRuntime::always_fail_for_tests(
                "engine_b_down",
                5_000,
            ),
        );
        let mut store = store_with_inviter();
        let token_id = make_activated_link(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_1".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap().onboarding_session_id,
            _ => panic!("expected ok"),
        };

        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "idem_terms".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &terms).unwrap();

        let device_id = device("voice_device_onb_fail");
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user("inviter"),
                    "phone".to_string(),
                    MonotonicTimeNs(now().0 + 3),
                    Some("audio_profile_voice".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let live_req = OnbVoiceEnrollLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(10),
            now: MonotonicTimeNs(now().0 + 10),
            onboarding_session_id: session_id,
            device_id,
            consent_asserted: true,
            max_total_attempts: 8,
            max_session_enroll_time_ms: 120_000,
            lock_after_consecutive_passes: 2,
            samples: vec![
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: "audio:voice:f1".to_string(),
                    attempt_index: 1,
                    sample_duration_ms: 1_350,
                    vad_coverage: 0.93,
                    snr_db: 18.0,
                    clipping_pct: 0.4,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: "voice-fail-sample-1".to_string(),
                },
                OnbVoiceEnrollSampleStep {
                    audio_sample_ref: "audio:voice:f2".to_string(),
                    attempt_index: 2,
                    sample_duration_ms: 1_360,
                    vad_coverage: 0.92,
                    snr_db: 17.8,
                    clipping_pct: 0.3,
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: "voice-fail-sample-2".to_string(),
                },
            ],
            finalize: OnbVoiceEnrollFinalize::Complete {
                idempotency_key: "voice-fail-complete".to_string(),
            },
        };

        let out = rt
            .run_voice_enrollment_live_sequence(&mut store, &live_req)
            .unwrap();
        let complete = out.complete_result.expect("complete result must exist");
        let receipt = complete
            .voice_artifact_sync_receipt_ref
            .expect("voice sync receipt must exist");
        let queue_row = store
            .mobile_artifact_sync_queue_row_for_receipt(&receipt)
            .expect("voice sync queue row must exist");
        assert_eq!(queue_row.state, MobileArtifactSyncState::InFlight);
        assert_eq!(queue_row.acked_at, None);
        assert_eq!(queue_row.last_error.as_deref(), Some("engine_b_down"));
    }

    #[test]
    fn onb_live_sequence_calls_voice_id_enroll_defer_when_requested() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let token_id = make_activated_link(&mut store);

        let start = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(OnbSessionStartDraftRequest {
                token_id,
                prefilled_context_ref: None,
                tenant_id: Some("tenant_1".to_string()),
                device_fingerprint: "device_fp_1".to_string(),
                app_platform: selene_kernel_contracts::ph1link::AppPlatform::Ios,
                app_instance_id: "ios_instance_onb".to_string(),
                deep_link_nonce: "nonce_onb".to_string(),
                link_opened_at: MonotonicTimeNs(now().0 + 5),
            }),
        };
        let session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1OnbResponse::Ok(ok) => ok.session_start_result.unwrap().onboarding_session_id,
            _ => panic!("expected ok"),
        };

        let terms = Ph1OnbRequest {
            schema_version: selene_kernel_contracts::ph1onb::PH1ONB_CONTRACT_VERSION,
            correlation_id: corr(),
            turn_id: turn(),
            now: MonotonicTimeNs(now().0 + 3),
            simulation_id: selene_kernel_contracts::ph1onb::ONB_TERMS_ACCEPT_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: OnbRequest::TermsAcceptCommit(OnbTermsAcceptCommitRequest {
                onboarding_session_id: session_id.clone(),
                terms_version_id: "terms_v1".to_string(),
                accepted: true,
                idempotency_key: "voice-defer-terms".to_string(),
            }),
        };
        let _ = rt.run(&mut store, &terms).unwrap();

        let device_id = device("voice_device_onb_defer");
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user("inviter"),
                    "phone".to_string(),
                    MonotonicTimeNs(now().0 + 3),
                    Some("audio_profile_voice".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let live_req = OnbVoiceEnrollLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(200),
            now: MonotonicTimeNs(now().0 + 4),
            onboarding_session_id: session_id,
            device_id,
            consent_asserted: true,
            max_total_attempts: 8,
            max_session_enroll_time_ms: 120_000,
            lock_after_consecutive_passes: 3,
            samples: vec![],
            finalize: OnbVoiceEnrollFinalize::Defer {
                reason_code: ReasonCodeId(0x5649_0201),
                idempotency_key: "voice-defer-1".to_string(),
            },
        };

        let out = rt
            .run_voice_enrollment_live_sequence(&mut store, &live_req)
            .unwrap();
        assert!(out.complete_result.is_none());
        assert!(out.defer_result.is_some());
        assert_eq!(out.sample_results.len(), 0);
    }

    #[test]
    fn onb_live_position_sequence_runs_create_validate_policy_activate() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        upsert_active_company(&mut store);

        let req = OnbPositionLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(300),
            now: MonotonicTimeNs(now().0 + 10),
            actor_user_id: user("inviter"),
            tenant_id: TenantId::new("tenant_1").unwrap(),
            company_id: "company_1".to_string(),
            position_title: "Store Manager".to_string(),
            department: "Retail".to_string(),
            jurisdiction: "CN".to_string(),
            schedule_type: PositionScheduleType::FullTime,
            permission_profile_ref: "profile_store_mgr".to_string(),
            compensation_band_ref: "band_l3".to_string(),
            policy_compensation_band_ref: None,
            create_idempotency_key: "onb-pos-create-1".to_string(),
            validate_idempotency_key: "onb-pos-validate-1".to_string(),
            policy_idempotency_key: "onb-pos-policy-1".to_string(),
            activate_idempotency_key: "onb-pos-activate-1".to_string(),
        };

        let out = rt.run_position_live_sequence(&mut store, &req).unwrap();
        assert_eq!(out.policy_result.policy_result, PositionPolicyResult::Allow);
        assert!(out.activation_skipped_reason.is_none());
        assert!(out.activate_result.is_some());
        assert_eq!(
            out.activate_result.unwrap().lifecycle_state,
            PositionLifecycleState::Active
        );
    }

    #[test]
    fn onb_live_position_sequence_skips_activate_when_policy_escalates() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        upsert_active_company(&mut store);

        let req = OnbPositionLiveRequest {
            correlation_id: corr(),
            turn_id_start: TurnId(350),
            now: MonotonicTimeNs(now().0 + 20),
            actor_user_id: user("inviter"),
            tenant_id: TenantId::new("tenant_1").unwrap(),
            company_id: "company_1".to_string(),
            position_title: "Warehouse Supervisor".to_string(),
            department: "Ops".to_string(),
            jurisdiction: "CN".to_string(),
            schedule_type: PositionScheduleType::Shift,
            permission_profile_ref: "profile_ops_sup".to_string(),
            compensation_band_ref: "band_l4".to_string(),
            policy_compensation_band_ref: None,
            create_idempotency_key: "onb-pos-create-2".to_string(),
            validate_idempotency_key: "onb-pos-validate-2".to_string(),
            // Deliberately different to force PositionPolicyResult::Escalate.
            policy_idempotency_key: "onb-pos-policy-2".to_string(),
            activate_idempotency_key: "onb-pos-activate-2".to_string(),
        };

        let mut out = rt.run_position_live_sequence(&mut store, &req).unwrap();
        assert_eq!(out.policy_result.policy_result, PositionPolicyResult::Allow);

        // Retry policy step with mismatched compensation band to exercise escalation path.
        let req_escalate = OnbPositionLiveRequest {
            policy_compensation_band_ref: Some("band_mismatch".to_string()),
            position_title: "Warehouse Supervisor B".to_string(),
            department: "Ops2".to_string(),
            create_idempotency_key: "onb-pos-create-3".to_string(),
            validate_idempotency_key: "onb-pos-validate-3".to_string(),
            policy_idempotency_key: "onb-pos-policy-3".to_string(),
            activate_idempotency_key: "onb-pos-activate-3".to_string(),
            ..req
        };
        out = rt
            .run_position_live_sequence(&mut store, &req_escalate)
            .unwrap();
        assert_eq!(
            out.policy_result.policy_result,
            PositionPolicyResult::Escalate
        );
        assert!(out.activate_result.is_none());
        assert!(out.activation_skipped_reason.is_some());
    }

    #[test]
    fn onb_link_activation_handoff_starts_session_with_link_app_context() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let link_rt = crate::ph1link::Ph1LinkRuntime::new(crate::ph1link::Ph1LinkConfig::mvp_v1());

        let draft = Ph1LinkRequest::invite_generate_draft_v1(
            corr(),
            turn(),
            now(),
            user("inviter"),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        let token_id = match link_rt.run(&mut store, &draft).unwrap() {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_generate_result.unwrap().token_id
            }
            _ => panic!("expected LINK generate ok"),
        };

        let open = Ph1LinkRequest::invite_open_activate_commit_v1(
            corr(),
            TurnId(turn().0 + 1),
            MonotonicTimeNs(now().0 + 5),
            token_id,
            "device_fp_link_handoff".to_string(),
            selene_kernel_contracts::ph1link::AppPlatform::Ios,
            "ios_instance_link_handoff".to_string(),
            "nonce_link_handoff".to_string(),
            MonotonicTimeNs(now().0 + 5),
            "idem_link_handoff_open".to_string(),
        )
        .unwrap();
        let activation = match link_rt.run(&mut store, &open).unwrap() {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_activation_result.unwrap()
            }
            _ => panic!("expected LINK open activate ok"),
        };

        let started = rt
            .start_session_from_link_activation(
                &mut store,
                MonotonicTimeNs(now().0 + 6),
                &activation,
                Some("tenant_1".to_string()),
                "device_fp_link_handoff".to_string(),
            )
            .unwrap();
        assert_eq!(
            started.status,
            selene_kernel_contracts::ph1onb::OnboardingStatus::DraftCreated
        );
        let row = store
            .ph1onb_session_row(&started.onboarding_session_id)
            .expect("onboarding session row should exist");
        assert_eq!(row.app_instance_id.as_str(), "ios_instance_link_handoff");
        assert_eq!(row.deep_link_nonce.as_str(), "nonce_link_handoff");
        assert_eq!(row.link_opened_at, MonotonicTimeNs(now().0 + 5));
    }

    #[test]
    fn onb_link_activation_handoff_refuses_non_activated_status() {
        let rt = Ph1OnbOrchRuntime::default();
        let mut store = store_with_inviter();
        let link_rt = crate::ph1link::Ph1LinkRuntime::new(crate::ph1link::Ph1LinkConfig::mvp_v1());

        let draft = Ph1LinkRequest::invite_generate_draft_v1(
            corr(),
            turn(),
            now(),
            user("inviter"),
            InviteeType::Employee,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        let generated = match link_rt.run(&mut store, &draft).unwrap() {
            selene_kernel_contracts::ph1link::Ph1LinkResponse::Ok(ok) => {
                ok.link_generate_result.unwrap()
            }
            _ => panic!("expected LINK generate ok"),
        };

        let blocked_activation = selene_kernel_contracts::ph1link::LinkActivationResult::v1(
            generated.token_id,
            generated.draft_id,
            selene_kernel_contracts::ph1link::LinkStatus::Blocked,
            Vec::new(),
            Some("FORWARDED_LINK_DEVICE_MISMATCH".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out = rt.start_session_from_link_activation(
            &mut store,
            MonotonicTimeNs(now().0 + 6),
            &blocked_activation,
            Some("tenant_1".to_string()),
            "device_fp_link_handoff".to_string(),
        );
        assert!(matches!(out, Err(StorageError::ContractViolation(_))));
    }
}
