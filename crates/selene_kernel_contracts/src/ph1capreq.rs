#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1CAPREQ_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

fn validate_id(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    Ok(())
}

fn validate_opt_id(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_id(field, v, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapreqId(String);

impl CapreqId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for CapreqId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("capreq_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityRequestAction {
    CreateDraft,
    SubmitForApproval,
    Approve,
    Reject,
    Fulfill,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityRequestStatus {
    Draft,
    PendingApproval,
    Approved,
    Rejected,
    Fulfilled,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequestLedgerEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub requester_user_id: UserId,
    pub action: CapabilityRequestAction,
    pub status: CapabilityRequestStatus,
    pub reason_code: ReasonCodeId,
    pub payload_hash: String,
    pub idempotency_key: Option<String>,
}

impl CapabilityRequestLedgerEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        requester_user_id: UserId,
        action: CapabilityRequestAction,
        status: CapabilityRequestStatus,
        reason_code: ReasonCodeId,
        payload_hash: String,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            created_at,
            tenant_id,
            capreq_id,
            requester_user_id,
            action,
            status,
            reason_code,
            payload_hash,
            idempotency_key,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for CapabilityRequestLedgerEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event_input.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event_input.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_ledger_event_input.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        validate_id(
            "capreq_ledger_event_input.payload_hash",
            &self.payload_hash,
            128,
        )?;
        validate_opt_id(
            "capreq_ledger_event_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequestLedgerEvent {
    pub schema_version: SchemaVersion,
    pub capreq_event_id: u64,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub requester_user_id: UserId,
    pub action: CapabilityRequestAction,
    pub status: CapabilityRequestStatus,
    pub reason_code: ReasonCodeId,
    pub payload_hash: String,
    pub idempotency_key: Option<String>,
}

impl CapabilityRequestLedgerEvent {
    pub fn from_input_v1(
        capreq_event_id: u64,
        input: CapabilityRequestLedgerEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let row = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            capreq_event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            capreq_id: input.capreq_id,
            requester_user_id: input.requester_user_id,
            action: input.action,
            status: input.status,
            reason_code: input.reason_code,
            payload_hash: input.payload_hash,
            idempotency_key: input.idempotency_key,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for CapabilityRequestLedgerEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        if self.capreq_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event.capreq_event_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_ledger_event.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_ledger_event.reason_code",
                reason: "must be > 0",
            });
        }
        validate_id("capreq_ledger_event.payload_hash", &self.payload_hash, 128)?;
        validate_opt_id(
            "capreq_ledger_event.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequestCurrentRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub requester_user_id: UserId,
    pub status: CapabilityRequestStatus,
    pub last_action: CapabilityRequestAction,
    pub payload_hash: String,
    pub source_event_id: u64,
    pub updated_at: MonotonicTimeNs,
    pub last_reason_code: ReasonCodeId,
}

impl CapabilityRequestCurrentRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        capreq_id: CapreqId,
        requester_user_id: UserId,
        status: CapabilityRequestStatus,
        last_action: CapabilityRequestAction,
        payload_hash: String,
        source_event_id: u64,
        updated_at: MonotonicTimeNs,
        last_reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            tenant_id,
            capreq_id,
            requester_user_id,
            status,
            last_action,
            payload_hash,
            source_event_id,
            updated_at,
            last_reason_code,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for CapabilityRequestCurrentRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_current_record.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_current_record.requester_user_id",
            self.requester_user_id.as_str(),
            128,
        )?;
        if self.source_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_current_record.source_event_id",
                reason: "must be > 0",
            });
        }
        if self.updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_current_record.updated_at",
                reason: "must be > 0",
            });
        }
        if self.last_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_current_record.last_reason_code",
                reason: "must be > 0",
            });
        }
        validate_id(
            "capreq_current_record.payload_hash",
            &self.payload_hash,
            128,
        )?;
        Ok(())
    }
}

// Simulation IDs (authoritative strings; must match docs/08_SIMULATION_CATALOG.md).
pub const CAPREQ_CREATE_DRAFT: &str = "CAPREQ_CREATE_DRAFT";
pub const CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT: &str = "CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT";
pub const CAPREQ_APPROVE_COMMIT: &str = "CAPREQ_APPROVE_COMMIT";
pub const CAPREQ_REJECT_COMMIT: &str = "CAPREQ_REJECT_COMMIT";
pub const CAPREQ_FULFILL_COMMIT: &str = "CAPREQ_FULFILL_COMMIT";
pub const CAPREQ_CANCEL_REVOKE: &str = "CAPREQ_CANCEL_REVOKE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapreqSimulationType {
    Draft,
    Commit,
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqCreateDraftRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub requested_capability_id: String,
    pub target_scope_ref: String,
    pub justification: String,
    pub idempotency_key: String,
}

impl Validate for CapreqCreateDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_create_draft_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        validate_id(
            "capreq_create_draft_request.requested_capability_id",
            &self.requested_capability_id,
            128,
        )?;
        validate_id(
            "capreq_create_draft_request.target_scope_ref",
            &self.target_scope_ref,
            128,
        )?;
        validate_id(
            "capreq_create_draft_request.justification",
            &self.justification,
            512,
        )?;
        validate_id(
            "capreq_create_draft_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqSubmitForApprovalCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub idempotency_key: String,
}

impl Validate for CapreqSubmitForApprovalCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_submit_for_approval_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_submit_for_approval_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqApproveCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub idempotency_key: String,
}

impl Validate for CapreqApproveCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_approve_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_approve_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqRejectCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub idempotency_key: String,
}

impl Validate for CapreqRejectCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_reject_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_reject_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqFulfillCommitRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub idempotency_key: String,
}

impl Validate for CapreqFulfillCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_fulfill_commit_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_fulfill_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqCancelRevokeRequest {
    pub actor_user_id: UserId,
    pub tenant_id: TenantId,
    pub capreq_id: CapreqId,
    pub idempotency_key: String,
}

impl Validate for CapreqCancelRevokeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "capreq_cancel_revoke_request.actor_user_id",
            self.actor_user_id.as_str(),
            128,
        )?;
        self.tenant_id.validate()?;
        self.capreq_id.validate()?;
        validate_id(
            "capreq_cancel_revoke_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapreqRequest {
    CreateDraft(CapreqCreateDraftRequest),
    SubmitForApprovalCommit(CapreqSubmitForApprovalCommitRequest),
    ApproveCommit(CapreqApproveCommitRequest),
    RejectCommit(CapreqRejectCommitRequest),
    FulfillCommit(CapreqFulfillCommitRequest),
    CancelRevoke(CapreqCancelRevokeRequest),
}

impl CapreqRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            CapreqRequest::CreateDraft(_) => CAPREQ_CREATE_DRAFT,
            CapreqRequest::SubmitForApprovalCommit(_) => CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT,
            CapreqRequest::ApproveCommit(_) => CAPREQ_APPROVE_COMMIT,
            CapreqRequest::RejectCommit(_) => CAPREQ_REJECT_COMMIT,
            CapreqRequest::FulfillCommit(_) => CAPREQ_FULFILL_COMMIT,
            CapreqRequest::CancelRevoke(_) => CAPREQ_CANCEL_REVOKE,
        }
    }

    pub fn simulation_type(&self) -> CapreqSimulationType {
        match self {
            CapreqRequest::CreateDraft(_) => CapreqSimulationType::Draft,
            CapreqRequest::SubmitForApprovalCommit(_)
            | CapreqRequest::ApproveCommit(_)
            | CapreqRequest::RejectCommit(_)
            | CapreqRequest::FulfillCommit(_) => CapreqSimulationType::Commit,
            CapreqRequest::CancelRevoke(_) => CapreqSimulationType::Revoke,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1CapreqRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: CapreqSimulationType,
    pub request: CapreqRequest,
}

impl Ph1CapreqRequest {
    pub fn create_draft_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        requested_capability_id: String,
        target_scope_ref: String,
        justification: String,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request = CapreqRequest::CreateDraft(CapreqCreateDraftRequest {
            actor_user_id,
            tenant_id,
            requested_capability_id,
            target_scope_ref,
            justification,
            idempotency_key,
        });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_CREATE_DRAFT.to_string(),
            simulation_type: CapreqSimulationType::Draft,
            request,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn submit_for_approval_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request =
            CapreqRequest::SubmitForApprovalCommit(CapreqSubmitForApprovalCommitRequest {
                actor_user_id,
                tenant_id,
                capreq_id,
                idempotency_key,
            });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_SUBMIT_FOR_APPROVAL_COMMIT.to_string(),
            simulation_type: CapreqSimulationType::Commit,
            request,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn approve_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request = CapreqRequest::ApproveCommit(CapreqApproveCommitRequest {
            actor_user_id,
            tenant_id,
            capreq_id,
            idempotency_key,
        });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_APPROVE_COMMIT.to_string(),
            simulation_type: CapreqSimulationType::Commit,
            request,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn reject_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request = CapreqRequest::RejectCommit(CapreqRejectCommitRequest {
            actor_user_id,
            tenant_id,
            capreq_id,
            idempotency_key,
        });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_REJECT_COMMIT.to_string(),
            simulation_type: CapreqSimulationType::Commit,
            request,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn fulfill_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request = CapreqRequest::FulfillCommit(CapreqFulfillCommitRequest {
            actor_user_id,
            tenant_id,
            capreq_id,
            idempotency_key,
        });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_FULFILL_COMMIT.to_string(),
            simulation_type: CapreqSimulationType::Commit,
            request,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn cancel_revoke_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        actor_user_id: UserId,
        tenant_id: TenantId,
        capreq_id: CapreqId,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let request = CapreqRequest::CancelRevoke(CapreqCancelRevokeRequest {
            actor_user_id,
            tenant_id,
            capreq_id,
            idempotency_key,
        });
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: CAPREQ_CANCEL_REVOKE.to_string(),
            simulation_type: CapreqSimulationType::Revoke,
            request,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1CapreqRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_request.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_request.now",
                reason: "must be > 0",
            });
        }
        validate_id("ph1capreq_request.simulation_id", &self.simulation_id, 96)?;
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_request.simulation_id",
                reason: "must match request variant simulation_id",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_request.simulation_type",
                reason: "must match request variant simulation_type",
            });
        }
        match &self.request {
            CapreqRequest::CreateDraft(r) => r.validate(),
            CapreqRequest::SubmitForApprovalCommit(r) => r.validate(),
            CapreqRequest::ApproveCommit(r) => r.validate(),
            CapreqRequest::RejectCommit(r) => r.validate(),
            CapreqRequest::FulfillCommit(r) => r.validate(),
            CapreqRequest::CancelRevoke(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapreqLifecycleResult {
    pub schema_version: SchemaVersion,
    pub capreq_id: CapreqId,
    pub capreq_event_id: u64,
    pub action: CapabilityRequestAction,
    pub status: CapabilityRequestStatus,
}

impl CapreqLifecycleResult {
    pub fn v1(
        capreq_id: CapreqId,
        capreq_event_id: u64,
        action: CapabilityRequestAction,
        status: CapabilityRequestStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            capreq_id,
            capreq_event_id,
            action,
            status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for CapreqLifecycleResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_lifecycle_result.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        self.capreq_id.validate()?;
        if self.capreq_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capreq_lifecycle_result.capreq_event_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1CapreqOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub lifecycle_result: CapreqLifecycleResult,
}

impl Ph1CapreqOk {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        lifecycle_result: CapreqLifecycleResult,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            lifecycle_result,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1CapreqOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_ok.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        validate_id("ph1capreq_ok.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.lifecycle_result.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1CapreqRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1CapreqRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1CapreqRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CAPREQ_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_refuse.schema_version",
                reason: "must match PH1CAPREQ_CONTRACT_VERSION",
            });
        }
        validate_id("ph1capreq_refuse.simulation_id", &self.simulation_id, 96)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1capreq_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_id("ph1capreq_refuse.message", &self.message, 512)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1CapreqResponse {
    Ok(Ph1CapreqOk),
    Refuse(Ph1CapreqRefuse),
}

impl Validate for Ph1CapreqResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1CapreqResponse::Ok(o) => o.validate(),
            Ph1CapreqResponse::Refuse(r) => r.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_simulation_id_and_type_must_match_variant() {
        let req = Ph1CapreqRequest {
            schema_version: PH1CAPREQ_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(2),
            now: MonotonicTimeNs(3),
            simulation_id: CAPREQ_APPROVE_COMMIT.to_string(),
            simulation_type: CapreqSimulationType::Commit,
            request: CapreqRequest::ApproveCommit(CapreqApproveCommitRequest {
                actor_user_id: UserId::new("actor_1").unwrap(),
                tenant_id: TenantId::new("tenant_1").unwrap(),
                capreq_id: CapreqId::new("capreq_1").unwrap(),
                idempotency_key: "idem_1".to_string(),
            }),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn ok_response_requires_valid_lifecycle_payload() {
        let res = Ph1CapreqOk::v1(
            CAPREQ_CREATE_DRAFT.to_string(),
            ReasonCodeId(1),
            CapreqLifecycleResult::v1(
                CapreqId::new("capreq_1").unwrap(),
                1,
                CapabilityRequestAction::CreateDraft,
                CapabilityRequestStatus::Draft,
            )
            .unwrap(),
        );
        assert!(res.is_ok());
    }
}
