#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1link::{LinkId, PrefilledContextRef};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1ONB_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

// Simulation IDs (authoritative strings; must match docs/08_SIMULATION_CATALOG.md).
pub const ONB_SESSION_START_DRAFT: &str = "ONB_SESSION_START_DRAFT";
pub const ONB_TERMS_ACCEPT_COMMIT: &str = "ONB_TERMS_ACCEPT_COMMIT";
pub const ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT: &str = "ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT";
pub const ONB_EMPLOYEE_SENDER_VERIFY_COMMIT: &str = "ONB_EMPLOYEE_SENDER_VERIFY_COMMIT";
pub const ONB_PRIMARY_DEVICE_CONFIRM_COMMIT: &str = "ONB_PRIMARY_DEVICE_CONFIRM_COMMIT";
pub const ONB_ACCESS_INSTANCE_CREATE_COMMIT: &str = "ONB_ACCESS_INSTANCE_CREATE_COMMIT";
pub const ONB_COMPLETE_COMMIT: &str = "ONB_COMPLETE_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnboardingSessionId(String);

impl OnboardingSessionId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for OnboardingSessionId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "onboarding_session_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "onboarding_session_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.0.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "onboarding_session_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnboardingNextStep {
    Install,
    Terms,
    LoadPrefilled,
    AskMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermsStatus {
    Accepted,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerificationStatus {
    Pending,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SenderVerifyDecision {
    Confirm,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProofType {
    Biometric,
    Passcode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnboardingStatus {
    DraftCreated,
    TermsAccepted,
    TermsDeclined,
    VerificationPending,
    VerificationConfirmed,
    VerificationRejected,
    PrimaryDeviceConfirmed,
    AccessInstanceCreated,
    Complete,
}

fn validate_id(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    if !s.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_user_id(field: &'static str, u: &UserId) -> Result<(), ContractViolation> {
    let s = u.as_str();
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 128 chars",
        });
    }
    Ok(())
}

fn validate_opt_id(
    field: &'static str,
    s: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = s {
        validate_id(field, v, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbSessionStartDraftRequest {
    pub link_id: LinkId,
    pub prefilled_context_ref: Option<PrefilledContextRef>,
    pub tenant_id: Option<String>,
    pub device_fingerprint: String,
}

impl Validate for OnbSessionStartDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.link_id.validate()?;
        if let Some(r) = &self.prefilled_context_ref {
            r.validate()?;
        }
        validate_opt_id(
            "onb_session_start_draft_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        if self.device_fingerprint.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "onb_session_start_draft_request.device_fingerprint",
                reason: "must not be empty",
            });
        }
        if self.device_fingerprint.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "onb_session_start_draft_request.device_fingerprint",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbSessionStartResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub status: OnboardingStatus,
    pub next_step: OnboardingNextStep,
}

impl OnbSessionStartResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        status: OnboardingStatus,
        next_step: OnboardingNextStep,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            status,
            next_step,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbSessionStartResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_session_start_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbTermsAcceptCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub terms_version_id: String,
    pub accepted: bool,
    pub idempotency_key: String,
}

impl Validate for OnbTermsAcceptCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        validate_id(
            "onb_terms_accept_commit_request.terms_version_id",
            &self.terms_version_id,
            64,
        )?;
        validate_id(
            "onb_terms_accept_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbTermsAcceptResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub terms_status: TermsStatus,
}

impl OnbTermsAcceptResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        terms_status: TermsStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            terms_status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbTermsAcceptResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_terms_accept_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbEmployeePhotoCaptureSendCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub photo_blob_ref: String,
    pub sender_user_id: UserId,
    pub idempotency_key: String,
}

impl Validate for OnbEmployeePhotoCaptureSendCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        validate_id(
            "onb_employee_photo_capture_send_commit_request.photo_blob_ref",
            &self.photo_blob_ref,
            128,
        )?;
        validate_user_id(
            "onb_employee_photo_capture_send_commit_request.sender_user_id",
            &self.sender_user_id,
        )?;
        validate_id(
            "onb_employee_photo_capture_send_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbEmployeePhotoCaptureSendResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub photo_proof_ref: String,
    pub verification_status: VerificationStatus,
}

impl OnbEmployeePhotoCaptureSendResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        photo_proof_ref: String,
        verification_status: VerificationStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            photo_proof_ref,
            verification_status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbEmployeePhotoCaptureSendResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_employee_photo_capture_send_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        validate_id(
            "onb_employee_photo_capture_send_result.photo_proof_ref",
            &self.photo_proof_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbEmployeeSenderVerifyCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub sender_user_id: UserId,
    pub decision: SenderVerifyDecision,
    pub idempotency_key: String,
}

impl Validate for OnbEmployeeSenderVerifyCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        validate_user_id(
            "onb_employee_sender_verify_commit_request.sender_user_id",
            &self.sender_user_id,
        )?;
        validate_id(
            "onb_employee_sender_verify_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbEmployeeSenderVerifyResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub verification_status: VerificationStatus,
}

impl OnbEmployeeSenderVerifyResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        verification_status: VerificationStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            verification_status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbEmployeeSenderVerifyResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_employee_sender_verify_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbPrimaryDeviceConfirmCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub device_id: DeviceId,
    pub proof_type: ProofType,
    pub proof_ok: bool,
    pub idempotency_key: String,
}

impl Validate for OnbPrimaryDeviceConfirmCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        self.device_id.validate()?;
        validate_id(
            "onb_primary_device_confirm_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbPrimaryDeviceConfirmResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub primary_device_confirmed: bool,
}

impl OnbPrimaryDeviceConfirmResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        primary_device_confirmed: bool,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            primary_device_confirmed,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbPrimaryDeviceConfirmResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_primary_device_confirm_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbAccessInstanceCreateCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub user_id: UserId,
    pub tenant_id: Option<String>,
    pub role_id: String,
    pub idempotency_key: String,
}

impl Validate for OnbAccessInstanceCreateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        validate_user_id(
            "onb_access_instance_create_commit_request.user_id",
            &self.user_id,
        )?;
        validate_opt_id(
            "onb_access_instance_create_commit_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_id(
            "onb_access_instance_create_commit_request.role_id",
            &self.role_id,
            64,
        )?;
        validate_id(
            "onb_access_instance_create_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbAccessInstanceCreateResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub access_engine_instance_id: String,
}

impl OnbAccessInstanceCreateResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        access_engine_instance_id: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            access_engine_instance_id,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbAccessInstanceCreateResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_access_instance_create_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        validate_id(
            "onb_access_instance_create_result.access_engine_instance_id",
            &self.access_engine_instance_id,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbCompleteCommitRequest {
    pub onboarding_session_id: OnboardingSessionId,
    pub idempotency_key: String,
}

impl Validate for OnbCompleteCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.onboarding_session_id.validate()?;
        validate_id(
            "onb_complete_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnbCompleteResult {
    pub schema_version: SchemaVersion,
    pub onboarding_session_id: OnboardingSessionId,
    pub onboarding_status: OnboardingStatus,
}

impl OnbCompleteResult {
    pub fn v1(
        onboarding_session_id: OnboardingSessionId,
        onboarding_status: OnboardingStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            onboarding_session_id,
            onboarding_status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for OnbCompleteResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "onb_complete_result.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.onboarding_session_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnbRequest {
    SessionStartDraft(OnbSessionStartDraftRequest),
    TermsAcceptCommit(OnbTermsAcceptCommitRequest),
    EmployeePhotoCaptureSendCommit(OnbEmployeePhotoCaptureSendCommitRequest),
    EmployeeSenderVerifyCommit(OnbEmployeeSenderVerifyCommitRequest),
    PrimaryDeviceConfirmCommit(OnbPrimaryDeviceConfirmCommitRequest),
    AccessInstanceCreateCommit(OnbAccessInstanceCreateCommitRequest),
    CompleteCommit(OnbCompleteCommitRequest),
}

impl OnbRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            OnbRequest::SessionStartDraft(_) => ONB_SESSION_START_DRAFT,
            OnbRequest::TermsAcceptCommit(_) => ONB_TERMS_ACCEPT_COMMIT,
            OnbRequest::EmployeePhotoCaptureSendCommit(_) => ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT,
            OnbRequest::EmployeeSenderVerifyCommit(_) => ONB_EMPLOYEE_SENDER_VERIFY_COMMIT,
            OnbRequest::PrimaryDeviceConfirmCommit(_) => ONB_PRIMARY_DEVICE_CONFIRM_COMMIT,
            OnbRequest::AccessInstanceCreateCommit(_) => ONB_ACCESS_INSTANCE_CREATE_COMMIT,
            OnbRequest::CompleteCommit(_) => ONB_COMPLETE_COMMIT,
        }
    }

    pub fn simulation_type(&self) -> SimulationType {
        match self {
            OnbRequest::SessionStartDraft(_) => SimulationType::Draft,
            OnbRequest::TermsAcceptCommit(_)
            | OnbRequest::EmployeePhotoCaptureSendCommit(_)
            | OnbRequest::EmployeeSenderVerifyCommit(_)
            | OnbRequest::PrimaryDeviceConfirmCommit(_)
            | OnbRequest::AccessInstanceCreateCommit(_)
            | OnbRequest::CompleteCommit(_) => SimulationType::Commit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1OnbRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: SimulationType,
    pub request: OnbRequest,
}

impl Ph1OnbRequest {
    pub fn session_start_draft_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        link_id: LinkId,
        prefilled_context_ref: Option<PrefilledContextRef>,
        tenant_id: Option<String>,
        device_fingerprint: String,
    ) -> Result<Self, ContractViolation> {
        let req = OnbSessionStartDraftRequest {
            link_id,
            prefilled_context_ref,
            tenant_id,
            device_fingerprint,
        };
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: ONB_SESSION_START_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: OnbRequest::SessionStartDraft(req),
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1OnbRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_request.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_request.now",
                reason: "must be > 0",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_request.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_request.simulation_id",
                reason: "must match the request variant's simulation_id",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_request.simulation_type",
                reason: "must match the request variant's simulation_type",
            });
        }
        match &self.request {
            OnbRequest::SessionStartDraft(r) => r.validate(),
            OnbRequest::TermsAcceptCommit(r) => r.validate(),
            OnbRequest::EmployeePhotoCaptureSendCommit(r) => r.validate(),
            OnbRequest::EmployeeSenderVerifyCommit(r) => r.validate(),
            OnbRequest::PrimaryDeviceConfirmCommit(r) => r.validate(),
            OnbRequest::AccessInstanceCreateCommit(r) => r.validate(),
            OnbRequest::CompleteCommit(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1OnbOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub session_start_result: Option<OnbSessionStartResult>,
    pub terms_accept_result: Option<OnbTermsAcceptResult>,
    pub employee_photo_result: Option<OnbEmployeePhotoCaptureSendResult>,
    pub employee_sender_verify_result: Option<OnbEmployeeSenderVerifyResult>,
    pub primary_device_confirm_result: Option<OnbPrimaryDeviceConfirmResult>,
    pub access_instance_create_result: Option<OnbAccessInstanceCreateResult>,
    pub complete_result: Option<OnbCompleteResult>,
}

impl Ph1OnbOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        session_start_result: Option<OnbSessionStartResult>,
        terms_accept_result: Option<OnbTermsAcceptResult>,
        employee_photo_result: Option<OnbEmployeePhotoCaptureSendResult>,
        employee_sender_verify_result: Option<OnbEmployeeSenderVerifyResult>,
        primary_device_confirm_result: Option<OnbPrimaryDeviceConfirmResult>,
        access_instance_create_result: Option<OnbAccessInstanceCreateResult>,
        complete_result: Option<OnbCompleteResult>,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            session_start_result,
            terms_accept_result,
            employee_photo_result,
            employee_sender_verify_result,
            primary_device_confirm_result,
            access_instance_create_result,
            complete_result,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1OnbOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_ok.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_ok.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_ok.simulation_id",
                reason: "must be <= 96 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_ok.reason_code",
                reason: "must be > 0",
            });
        }

        let mut count = 0u8;
        if let Some(r) = &self.session_start_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.terms_accept_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.employee_photo_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.employee_sender_verify_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.primary_device_confirm_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.access_instance_create_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.complete_result {
            r.validate()?;
            count += 1;
        }
        if count != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_ok",
                reason: "must contain exactly one result kind",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1OnbRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1OnbRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ONB_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1OnbRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ONB_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.schema_version",
                reason: "must match PH1ONB_CONTRACT_VERSION",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.simulation_id",
                reason: "must be <= 96 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1onb_refuse.message",
                reason: "must be <= 512 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1OnbResponse {
    Ok(Ph1OnbOk),
    Refuse(Ph1OnbRefuse),
}

impl Validate for Ph1OnbResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1OnbResponse::Ok(o) => o.validate(),
            Ph1OnbResponse::Refuse(r) => r.validate(),
        }
    }
}
