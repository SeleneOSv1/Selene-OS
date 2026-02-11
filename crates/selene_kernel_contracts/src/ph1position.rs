#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1POSITION_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

pub const POSITION_SIM_001_CREATE_DRAFT: &str = "POSITION_SIM_001_CREATE_DRAFT";
pub const POSITION_SIM_002_VALIDATE_AUTH_COMPANY: &str = "POSITION_SIM_002_VALIDATE_AUTH_COMPANY";
pub const POSITION_SIM_003_BAND_POLICY_CHECK: &str = "POSITION_SIM_003_BAND_POLICY_CHECK";
pub const POSITION_SIM_004_ACTIVATE_COMMIT: &str = "POSITION_SIM_004_ACTIVATE_COMMIT";
pub const POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT: &str =
    "POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionSimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionScheduleType {
    FullTime,
    PartTime,
    Contract,
    Shift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PositionLifecycleState {
    Draft,
    Active,
    Suspended,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionLifecycleAction {
    CreateDraft,
    Activate,
    Suspend,
    Retire,
    PolicyOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionRequestedAction {
    Activate,
    Suspend,
    Retire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionPolicyResult {
    Allow,
    Escalate,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TenantId(String);

impl TenantId {
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

impl Validate for TenantId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("tenant_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositionId(String);

impl PositionId {
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

impl Validate for PositionId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("position_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub company_id: String,
    pub position_id: PositionId,
    pub position_title: String,
    pub department: String,
    pub jurisdiction: String,
    pub schedule_type: PositionScheduleType,
    pub permission_profile_ref: String,
    pub compensation_band_ref: String,
    pub lifecycle_state: PositionLifecycleState,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
}

impl PositionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        company_id: String,
        position_id: PositionId,
        position_title: String,
        department: String,
        jurisdiction: String,
        schedule_type: PositionScheduleType,
        permission_profile_ref: String,
        compensation_band_ref: String,
        lifecycle_state: PositionLifecycleState,
        created_at: MonotonicTimeNs,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            tenant_id,
            company_id,
            position_id,
            position_title,
            department,
            jurisdiction,
            schedule_type,
            permission_profile_ref,
            compensation_band_ref,
            lifecycle_state,
            created_at,
            updated_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PositionRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "position_record.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.position_id.validate()?;
        validate_id("position_record.company_id", &self.company_id, 64)?;
        validate_text("position_record.position_title", &self.position_title, 128)?;
        validate_text("position_record.department", &self.department, 128)?;
        validate_text("position_record.jurisdiction", &self.jurisdiction, 64)?;
        validate_text(
            "position_record.permission_profile_ref",
            &self.permission_profile_ref,
            128,
        )?;
        validate_text(
            "position_record.compensation_band_ref",
            &self.compensation_band_ref,
            128,
        )?;
        if self.updated_at.0 < self.created_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "position_record.updated_at",
                reason: "must be >= created_at",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionCreateDraftRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub company_id: String,
    pub position_title: String,
    pub department: String,
    pub jurisdiction: String,
    pub schedule_type: PositionScheduleType,
    pub permission_profile_ref: String,
    pub compensation_band_ref: String,
    pub idempotency_key: String,
}

impl Validate for PositionCreateDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "position_create_draft_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        validate_id(
            "position_create_draft_request.company_id",
            &self.company_id,
            64,
        )?;
        validate_text(
            "position_create_draft_request.position_title",
            &self.position_title,
            128,
        )?;
        validate_text(
            "position_create_draft_request.department",
            &self.department,
            128,
        )?;
        validate_text(
            "position_create_draft_request.jurisdiction",
            &self.jurisdiction,
            64,
        )?;
        validate_text(
            "position_create_draft_request.permission_profile_ref",
            &self.permission_profile_ref,
            128,
        )?;
        validate_text(
            "position_create_draft_request.compensation_band_ref",
            &self.compensation_band_ref,
            128,
        )?;
        validate_id(
            "position_create_draft_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionValidateAuthCompanyRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub company_id: String,
    pub position_id: PositionId,
    pub requested_action: PositionRequestedAction,
    pub idempotency_key: String,
}

impl Validate for PositionValidateAuthCompanyRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "position_validate_auth_company_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        validate_id(
            "position_validate_auth_company_request.company_id",
            &self.company_id,
            64,
        )?;
        self.position_id.validate()?;
        validate_id(
            "position_validate_auth_company_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionBandPolicyCheckRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub position_id: PositionId,
    pub compensation_band_ref: String,
    pub idempotency_key: String,
}

impl Validate for PositionBandPolicyCheckRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "position_band_policy_check_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.position_id.validate()?;
        validate_text(
            "position_band_policy_check_request.compensation_band_ref",
            &self.compensation_band_ref,
            128,
        )?;
        validate_id(
            "position_band_policy_check_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionActivateCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub position_id: PositionId,
    pub idempotency_key: String,
}

impl Validate for PositionActivateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "position_activate_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.position_id.validate()?;
        validate_id(
            "position_activate_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionRetireOrSuspendCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub position_id: PositionId,
    pub requested_state: PositionLifecycleState,
    pub idempotency_key: String,
}

impl Validate for PositionRetireOrSuspendCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "position_retire_or_suspend_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.position_id.validate()?;
        match self.requested_state {
            PositionLifecycleState::Suspended | PositionLifecycleState::Retired => {}
            _ => {
                return Err(ContractViolation::InvalidValue {
                    field: "position_retire_or_suspend_commit_request.requested_state",
                    reason: "must be Suspended or Retired",
                })
            }
        }
        validate_id(
            "position_retire_or_suspend_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionRequest {
    CreateDraft(PositionCreateDraftRequest),
    ValidateAuthCompany(PositionValidateAuthCompanyRequest),
    BandPolicyCheck(PositionBandPolicyCheckRequest),
    ActivateCommit(PositionActivateCommitRequest),
    RetireOrSuspendCommit(PositionRetireOrSuspendCommitRequest),
}

impl PositionRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            PositionRequest::CreateDraft(_) => POSITION_SIM_001_CREATE_DRAFT,
            PositionRequest::ValidateAuthCompany(_) => POSITION_SIM_002_VALIDATE_AUTH_COMPANY,
            PositionRequest::BandPolicyCheck(_) => POSITION_SIM_003_BAND_POLICY_CHECK,
            PositionRequest::ActivateCommit(_) => POSITION_SIM_004_ACTIVATE_COMMIT,
            PositionRequest::RetireOrSuspendCommit(_) => POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT,
        }
    }

    pub fn simulation_type(&self) -> PositionSimulationType {
        match self {
            PositionRequest::CreateDraft(_)
            | PositionRequest::ValidateAuthCompany(_)
            | PositionRequest::BandPolicyCheck(_) => PositionSimulationType::Draft,
            PositionRequest::ActivateCommit(_) | PositionRequest::RetireOrSuspendCommit(_) => {
                PositionSimulationType::Commit
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1PositionRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: PositionSimulationType,
    pub request: PositionRequest,
}

impl Validate for Ph1PositionRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_request.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_request.now",
                reason: "must be > 0",
            });
        }
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_request.simulation_id",
                reason: "must match request variant simulation ID",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_request.simulation_type",
                reason: "must match request variant simulation type",
            });
        }
        match &self.request {
            PositionRequest::CreateDraft(r) => r.validate(),
            PositionRequest::ValidateAuthCompany(r) => r.validate(),
            PositionRequest::BandPolicyCheck(r) => r.validate(),
            PositionRequest::ActivateCommit(r) => r.validate(),
            PositionRequest::RetireOrSuspendCommit(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionCreateDraftResult {
    pub schema_version: SchemaVersion,
    pub position_id: PositionId,
    pub lifecycle_state: PositionLifecycleState,
}

impl PositionCreateDraftResult {
    pub fn v1(
        position_id: PositionId,
        lifecycle_state: PositionLifecycleState,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            position_id,
            lifecycle_state,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PositionCreateDraftResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "position_create_draft_result.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        self.position_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionValidateAuthCompanyResult {
    pub schema_version: SchemaVersion,
    pub validation_status: PositionValidationStatus,
    pub reason_code: ReasonCodeId,
}

impl PositionValidateAuthCompanyResult {
    pub fn v1(
        validation_status: PositionValidationStatus,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            validation_status,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PositionValidateAuthCompanyResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "position_validate_auth_company_result.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "position_validate_auth_company_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionBandPolicyCheckResult {
    pub schema_version: SchemaVersion,
    pub policy_result: PositionPolicyResult,
    pub reason_code: ReasonCodeId,
}

impl PositionBandPolicyCheckResult {
    pub fn v1(
        policy_result: PositionPolicyResult,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            policy_result,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PositionBandPolicyCheckResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "position_band_policy_check_result.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "position_band_policy_check_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionLifecycleResult {
    pub schema_version: SchemaVersion,
    pub position_id: PositionId,
    pub lifecycle_state: PositionLifecycleState,
}

impl PositionLifecycleResult {
    pub fn v1(
        position_id: PositionId,
        lifecycle_state: PositionLifecycleState,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            position_id,
            lifecycle_state,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PositionLifecycleResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "position_lifecycle_result.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        self.position_id.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1PositionOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub create_draft_result: Option<PositionCreateDraftResult>,
    pub validate_auth_company_result: Option<PositionValidateAuthCompanyResult>,
    pub band_policy_check_result: Option<PositionBandPolicyCheckResult>,
    pub lifecycle_result: Option<PositionLifecycleResult>,
}

impl Ph1PositionOk {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        create_draft_result: Option<PositionCreateDraftResult>,
        validate_auth_company_result: Option<PositionValidateAuthCompanyResult>,
        band_policy_check_result: Option<PositionBandPolicyCheckResult>,
        lifecycle_result: Option<PositionLifecycleResult>,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            create_draft_result,
            validate_auth_company_result,
            band_policy_check_result,
            lifecycle_result,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1PositionOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_ok.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        validate_id("ph1position_ok.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_ok.reason_code",
                reason: "must be > 0",
            });
        }

        let mut count = 0u8;
        if let Some(r) = &self.create_draft_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.validate_auth_company_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.band_policy_check_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.lifecycle_result {
            r.validate()?;
            count += 1;
        }
        if count != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_ok",
                reason: "must contain exactly one result kind",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1PositionRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1PositionRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1PositionRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POSITION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_refuse.schema_version",
                reason: "must match PH1POSITION_CONTRACT_VERSION",
            });
        }
        validate_id("ph1position_refuse.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1position_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_text("ph1position_refuse.message", &self.message, 512)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PositionResponse {
    Ok(Ph1PositionOk),
    Refuse(Ph1PositionRefuse),
}

impl Validate for Ph1PositionResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PositionResponse::Ok(o) => o.validate(),
            Ph1PositionResponse::Refuse(r) => r.validate(),
        }
    }
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

fn validate_text(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_simulation_id_and_type_must_match_variant() {
        let req = Ph1PositionRequest {
            schema_version: PH1POSITION_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(2),
            now: MonotonicTimeNs(3),
            simulation_id: POSITION_SIM_004_ACTIVATE_COMMIT.to_string(),
            simulation_type: PositionSimulationType::Commit,
            request: PositionRequest::ActivateCommit(PositionActivateCommitRequest {
                actor_user_id: UserId::new("actor_1").unwrap(),
                tenant_id: TenantId::new("tenant_1").unwrap(),
                position_id: PositionId::new("pos_1").unwrap(),
                idempotency_key: "idem_1".to_string(),
            }),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn ok_response_requires_exactly_one_result_kind() {
        let res = Ph1PositionOk::v1(
            POSITION_SIM_001_CREATE_DRAFT.to_string(),
            ReasonCodeId(1),
            None,
            None,
            None,
            None,
        );
        assert!(res.is_err());
    }
}
