#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1BCAST_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1BCAST_ENGINE_ID: &str = "PH1.BCAST";
pub const PH1BCAST_IMPLEMENTATION_ID: &str = "PH1.BCAST.001";
pub const PH1BCAST_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1BCAST_IMPLEMENTATION_ID];

pub const BCAST_CREATE_DRAFT: &str = "BCAST_CREATE_DRAFT";
pub const BCAST_DELIVER_COMMIT: &str = "BCAST_DELIVER_COMMIT";
pub const BCAST_DEFER_COMMIT: &str = "BCAST_DEFER_COMMIT";
pub const BCAST_REMINDER_FIRED_COMMIT: &str = "BCAST_REMINDER_FIRED_COMMIT";
pub const BCAST_ACK_COMMIT: &str = "BCAST_ACK_COMMIT";
pub const BCAST_ESCALATE_COMMIT: &str = "BCAST_ESCALATE_COMMIT";
pub const BCAST_EXPIRE_COMMIT: &str = "BCAST_EXPIRE_COMMIT";
pub const BCAST_CANCEL_COMMIT: &str = "BCAST_CANCEL_COMMIT";
pub const BCAST_NON_URGENT_FOLLOWUP_WINDOW_NS: u64 = 300_000_000_000;

fn validate_token(
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
            reason: "exceeds max length",
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

fn validate_opt_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_token(field, v, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastSimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastCapabilityId {
    DraftCreate,
    DeliverCommit,
    DeferAndScheduleRetryCommit,
    ReminderFiredCommit,
    AckRecordCommit,
    EscalateToSenderCommit,
    ExpireCommit,
    CancelCommit,
}

impl BcastCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            BcastCapabilityId::DraftCreate => "BCAST_DRAFT_CREATE",
            BcastCapabilityId::DeliverCommit => "BCAST_DELIVER_COMMIT",
            BcastCapabilityId::DeferAndScheduleRetryCommit => "BCAST_DEFER_AND_SCHEDULE_RETRY",
            BcastCapabilityId::ReminderFiredCommit => "BCAST_REMINDER_FIRED",
            BcastCapabilityId::AckRecordCommit => "BCAST_ACK_RECORD",
            BcastCapabilityId::EscalateToSenderCommit => "BCAST_ESCALATE_TO_SENDER",
            BcastCapabilityId::ExpireCommit => "BCAST_EXPIRE",
            BcastCapabilityId::CancelCommit => "BCAST_CANCEL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BroadcastClassification {
    Simple,
    Priority,
    Emergency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastDeliveryMethod {
    SeleneApp,
    Sms,
    Whatsapp,
    Wechat,
    Email,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastRecipientRegion {
    Global,
    China,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastRecipientState {
    DraftCreated,
    Waiting,
    Followup,
    ReminderSet,
    ReminderFired,
    Deferred,
    Concluded,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcastAckStatus {
    Received,
    ActionConfirmed,
    Declined,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BroadcastId(String);

impl BroadcastId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for BroadcastId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("broadcast_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BroadcastRecipientId(String);

impl BroadcastRecipientId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for BroadcastRecipientId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("broadcast_recipient_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDraftCreateRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub audience_spec: String,
    pub classification: BroadcastClassification,
    pub content_payload_ref: String,
    pub prompt_dedupe_key: Option<String>,
    pub idempotency_key: String,
}

impl Validate for BcastDraftCreateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_draft_create.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        validate_token("bcast_draft_create.audience_spec", &self.audience_spec, 256)?;
        validate_token(
            "bcast_draft_create.content_payload_ref",
            &self.content_payload_ref,
            256,
        )?;
        validate_opt_token(
            "bcast_draft_create.prompt_dedupe_key",
            &self.prompt_dedupe_key,
            128,
        )?;
        validate_token(
            "bcast_draft_create.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDeliverCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub delivery_method: BcastDeliveryMethod,
    pub recipient_region: BcastRecipientRegion,
    pub app_unavailable: bool,
    pub delivery_plan_ref: String,
    pub simulation_context: String,
    pub idempotency_key: String,
}

impl Validate for BcastDeliverCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_deliver.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_deliver.delivery_plan_ref",
            &self.delivery_plan_ref,
            256,
        )?;
        validate_token(
            "bcast_deliver.simulation_context",
            &self.simulation_context,
            256,
        )?;
        validate_token("bcast_deliver.idempotency_key", &self.idempotency_key, 128)?;
        if self.delivery_method == BcastDeliveryMethod::SeleneApp && self.app_unavailable {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_deliver.app_unavailable",
                reason: "must be false when delivery_method is SeleneApp",
            });
        }
        if self.delivery_method != BcastDeliveryMethod::SeleneApp && !self.app_unavailable {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_deliver.app_unavailable",
                reason: "fallback delivery requires app_unavailable=true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDeferCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub defer_until: MonotonicTimeNs,
    pub handoff_to_reminder: bool,
    pub idempotency_key: String,
}

impl Validate for BcastDeferCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_defer.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        if self.defer_until.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_defer.defer_until",
                reason: "must be > 0",
            });
        }
        validate_token("bcast_defer.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastReminderFiredCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub reminder_ref: String,
    pub idempotency_key: String,
}

impl Validate for BcastReminderFiredCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_reminder_fired.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_reminder_fired.reminder_ref",
            &self.reminder_ref,
            256,
        )?;
        validate_token(
            "bcast_reminder_fired.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastAckCommitRequest {
    pub tenant_id: TenantId,
    pub recipient_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub ack_status: BcastAckStatus,
    pub idempotency_key: String,
}

impl Validate for BcastAckCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_ack.recipient_user_id",
            self.recipient_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token("bcast_ack.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastEscalateCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub escalation_reason: String,
    pub idempotency_key: String,
}

impl Validate for BcastEscalateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_escalate.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_escalate.escalation_reason",
            &self.escalation_reason,
            256,
        )?;
        validate_token("bcast_escalate.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastExpireCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub expiry_reason: String,
    pub idempotency_key: String,
}

impl Validate for BcastExpireCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_expire.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        validate_token("bcast_expire.expiry_reason", &self.expiry_reason, 256)?;
        validate_token("bcast_expire.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastCancelCommitRequest {
    pub tenant_id: TenantId,
    pub sender_user_id: UserId,
    pub broadcast_id: BroadcastId,
    pub cancel_reason: String,
    pub idempotency_key: String,
}

impl Validate for BcastCancelCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "bcast_cancel.sender_user_id",
            self.sender_user_id.as_str(),
            128,
        )?;
        self.broadcast_id.validate()?;
        validate_token("bcast_cancel.cancel_reason", &self.cancel_reason, 256)?;
        validate_token("bcast_cancel.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcastRequest {
    DraftCreate(BcastDraftCreateRequest),
    DeliverCommit(BcastDeliverCommitRequest),
    DeferCommit(BcastDeferCommitRequest),
    ReminderFiredCommit(BcastReminderFiredCommitRequest),
    AckCommit(BcastAckCommitRequest),
    EscalateCommit(BcastEscalateCommitRequest),
    ExpireCommit(BcastExpireCommitRequest),
    CancelCommit(BcastCancelCommitRequest),
}

impl BcastRequest {
    pub fn capability_id(&self) -> BcastCapabilityId {
        match self {
            BcastRequest::DraftCreate(_) => BcastCapabilityId::DraftCreate,
            BcastRequest::DeliverCommit(_) => BcastCapabilityId::DeliverCommit,
            BcastRequest::DeferCommit(_) => BcastCapabilityId::DeferAndScheduleRetryCommit,
            BcastRequest::ReminderFiredCommit(_) => BcastCapabilityId::ReminderFiredCommit,
            BcastRequest::AckCommit(_) => BcastCapabilityId::AckRecordCommit,
            BcastRequest::EscalateCommit(_) => BcastCapabilityId::EscalateToSenderCommit,
            BcastRequest::ExpireCommit(_) => BcastCapabilityId::ExpireCommit,
            BcastRequest::CancelCommit(_) => BcastCapabilityId::CancelCommit,
        }
    }

    pub fn expected_simulation_id(&self) -> &'static str {
        match self {
            BcastRequest::DraftCreate(_) => BCAST_CREATE_DRAFT,
            BcastRequest::DeliverCommit(_) => BCAST_DELIVER_COMMIT,
            BcastRequest::DeferCommit(_) => BCAST_DEFER_COMMIT,
            BcastRequest::ReminderFiredCommit(_) => BCAST_REMINDER_FIRED_COMMIT,
            BcastRequest::AckCommit(_) => BCAST_ACK_COMMIT,
            BcastRequest::EscalateCommit(_) => BCAST_ESCALATE_COMMIT,
            BcastRequest::ExpireCommit(_) => BCAST_EXPIRE_COMMIT,
            BcastRequest::CancelCommit(_) => BCAST_CANCEL_COMMIT,
        }
    }

    pub fn expected_simulation_type(&self) -> BcastSimulationType {
        match self {
            BcastRequest::DraftCreate(_) => BcastSimulationType::Draft,
            BcastRequest::DeliverCommit(_)
            | BcastRequest::DeferCommit(_)
            | BcastRequest::ReminderFiredCommit(_)
            | BcastRequest::AckCommit(_)
            | BcastRequest::EscalateCommit(_)
            | BcastRequest::ExpireCommit(_)
            | BcastRequest::CancelCommit(_) => BcastSimulationType::Commit,
        }
    }
}

impl Validate for BcastRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            BcastRequest::DraftCreate(v) => v.validate(),
            BcastRequest::DeliverCommit(v) => v.validate(),
            BcastRequest::DeferCommit(v) => v.validate(),
            BcastRequest::ReminderFiredCommit(v) => v.validate(),
            BcastRequest::AckCommit(v) => v.validate(),
            BcastRequest::EscalateCommit(v) => v.validate(),
            BcastRequest::ExpireCommit(v) => v.validate(),
            BcastRequest::CancelCommit(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1BcastRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: BcastSimulationType,
    pub request: BcastRequest,
}

impl Validate for Ph1BcastRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BCAST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_request.schema_version",
                reason: "must match PH1BCAST_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_request.now",
                reason: "must be > 0",
            });
        }
        validate_token("ph1bcast_request.simulation_id", &self.simulation_id, 128)?;
        self.request.validate()?;

        if self.simulation_id != self.request.expected_simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_request.simulation_id",
                reason: "must match request variant simulation id",
            });
        }
        if self.simulation_type != self.request.expected_simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_request.simulation_type",
                reason: "must match request variant simulation type",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDraftCreateResult {
    pub broadcast_id: BroadcastId,
    pub state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastDraftCreateResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_draft_create_result.reason_code",
                reason: "must be > 0",
            });
        }
        if self.state != BcastRecipientState::DraftCreated {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_draft_create_result.state",
                reason: "must be DraftCreated",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDeliverCommitResult {
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub delivery_request_ref: String,
    pub recipient_state: BcastRecipientState,
    pub followup_immediate: bool,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastDeliverCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_deliver_commit_result.delivery_request_ref",
            &self.delivery_request_ref,
            256,
        )?;
        let expected_state = if self.followup_immediate {
            BcastRecipientState::Followup
        } else {
            BcastRecipientState::Waiting
        };
        if self.recipient_state != expected_state {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_deliver_commit_result.recipient_state",
                reason: "recipient_state does not match followup_immediate mode",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_deliver_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastDeferCommitResult {
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub retry_at: MonotonicTimeNs,
    pub recipient_state: BcastRecipientState,
    pub handoff_to_reminder: bool,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastDeferCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        if self.retry_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_defer_commit_result.retry_at",
                reason: "must be > 0",
            });
        }
        let expected_state = if self.handoff_to_reminder {
            BcastRecipientState::ReminderSet
        } else {
            BcastRecipientState::Deferred
        };
        if self.recipient_state != expected_state {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_defer_commit_result.recipient_state",
                reason: "recipient_state does not match handoff_to_reminder mode",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_defer_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastReminderFiredCommitResult {
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub reminder_ref: String,
    pub recipient_state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastReminderFiredCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_reminder_fired_commit_result.reminder_ref",
            &self.reminder_ref,
            256,
        )?;
        if self.recipient_state != BcastRecipientState::ReminderFired {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_reminder_fired_commit_result.recipient_state",
                reason: "must be ReminderFired",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_reminder_fired_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastAckCommitResult {
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub ack_status: BcastAckStatus,
    pub recipient_state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastAckCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        if self.recipient_state != BcastRecipientState::Concluded {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_ack_commit_result.recipient_state",
                reason: "must be Concluded",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_ack_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastEscalateCommitResult {
    pub broadcast_id: BroadcastId,
    pub recipient_id: BroadcastRecipientId,
    pub sender_notice_ref: String,
    pub recipient_state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastEscalateCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        self.recipient_id.validate()?;
        validate_token(
            "bcast_escalate_commit_result.sender_notice_ref",
            &self.sender_notice_ref,
            256,
        )?;
        if self.recipient_state != BcastRecipientState::Followup {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_escalate_commit_result.recipient_state",
                reason: "must be Followup",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_escalate_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastExpireCommitResult {
    pub broadcast_id: BroadcastId,
    pub state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastExpireCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        if self.state != BcastRecipientState::Expired {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_expire_commit_result.state",
                reason: "must be Expired",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_expire_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcastCancelCommitResult {
    pub broadcast_id: BroadcastId,
    pub state: BcastRecipientState,
    pub reason_code: ReasonCodeId,
}

impl Validate for BcastCancelCommitResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.broadcast_id.validate()?;
        if self.state != BcastRecipientState::Canceled {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_cancel_commit_result.state",
                reason: "must be Canceled",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bcast_cancel_commit_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BcastOutcome {
    DraftCreate(BcastDraftCreateResult),
    DeliverCommit(BcastDeliverCommitResult),
    DeferCommit(BcastDeferCommitResult),
    ReminderFiredCommit(BcastReminderFiredCommitResult),
    AckCommit(BcastAckCommitResult),
    EscalateCommit(BcastEscalateCommitResult),
    ExpireCommit(BcastExpireCommitResult),
    CancelCommit(BcastCancelCommitResult),
}

impl Validate for BcastOutcome {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            BcastOutcome::DraftCreate(v) => v.validate(),
            BcastOutcome::DeliverCommit(v) => v.validate(),
            BcastOutcome::DeferCommit(v) => v.validate(),
            BcastOutcome::ReminderFiredCommit(v) => v.validate(),
            BcastOutcome::AckCommit(v) => v.validate(),
            BcastOutcome::EscalateCommit(v) => v.validate(),
            BcastOutcome::ExpireCommit(v) => v.validate(),
            BcastOutcome::CancelCommit(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1BcastOk {
    pub schema_version: SchemaVersion,
    pub capability_id: BcastCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub outcome: BcastOutcome,
    pub no_authority_grant: bool,
    pub simulation_gated: bool,
}

impl Ph1BcastOk {
    pub fn v1(
        capability_id: BcastCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        outcome: BcastOutcome,
        no_authority_grant: bool,
        simulation_gated: bool,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            capability_id,
            simulation_id,
            reason_code,
            outcome,
            no_authority_grant,
            simulation_gated,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1BcastOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BCAST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_ok.schema_version",
                reason: "must match PH1BCAST_CONTRACT_VERSION",
            });
        }
        validate_token("ph1bcast_ok.simulation_id", &self.simulation_id, 128)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.outcome.validate()?;
        if !self.no_authority_grant || !self.simulation_gated {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_ok",
                reason: "no_authority_grant and simulation_gated must both be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1BcastRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: BcastCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1BcastRefuse {
    pub fn v1(
        capability_id: BcastCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            capability_id,
            simulation_id,
            reason_code,
            message,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1BcastRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BCAST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_refuse.schema_version",
                reason: "must match PH1BCAST_CONTRACT_VERSION",
            });
        }
        validate_token("ph1bcast_refuse.simulation_id", &self.simulation_id, 128)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1bcast_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_token("ph1bcast_refuse.message", &self.message, 512)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1BcastResponse {
    Ok(Ph1BcastOk),
    Refuse(Ph1BcastRefuse),
}

impl Validate for Ph1BcastResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1BcastResponse::Ok(v) => v.validate(),
            Ph1BcastResponse::Refuse(v) => v.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantId {
        TenantId::new("tenant_bcast").unwrap()
    }

    fn sender() -> UserId {
        UserId::new("user_sender").unwrap()
    }

    fn recipient() -> UserId {
        UserId::new("user_recipient").unwrap()
    }

    fn broadcast_id() -> BroadcastId {
        BroadcastId::new("bcast_100").unwrap()
    }

    fn recipient_id() -> BroadcastRecipientId {
        BroadcastRecipientId::new("recipient_1").unwrap()
    }

    #[test]
    fn at_bcast_contract_01_simulation_id_must_match_variant() {
        let req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id: CorrelationId(800),
            turn_id: TurnId(41),
            now: MonotonicTimeNs(1),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                audience_spec: "wife".to_string(),
                classification: BroadcastClassification::Priority,
                content_payload_ref: "payload_1".to_string(),
                prompt_dedupe_key: Some("dedupe_1".to_string()),
                idempotency_key: "idem_1".to_string(),
            }),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn at_bcast_contract_02_deliver_requires_simulation_context() {
        let req = Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id: CorrelationId(801),
            turn_id: TurnId(42),
            now: MonotonicTimeNs(2),
            simulation_id: BCAST_DELIVER_COMMIT.to_string(),
            simulation_type: BcastSimulationType::Commit,
            request: BcastRequest::DeliverCommit(BcastDeliverCommitRequest {
                tenant_id: tenant(),
                sender_user_id: sender(),
                broadcast_id: broadcast_id(),
                recipient_id: recipient_id(),
                delivery_method: BcastDeliveryMethod::SeleneApp,
                recipient_region: BcastRecipientRegion::Global,
                app_unavailable: false,
                delivery_plan_ref: "plan_1".to_string(),
                simulation_context: "".to_string(),
                idempotency_key: "idem_2".to_string(),
            }),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn at_bcast_contract_02a_fallback_requires_app_unavailable_flag() {
        let req = BcastDeliverCommitRequest {
            tenant_id: tenant(),
            sender_user_id: sender(),
            broadcast_id: broadcast_id(),
            recipient_id: recipient_id(),
            delivery_method: BcastDeliveryMethod::Sms,
            recipient_region: BcastRecipientRegion::Global,
            app_unavailable: false,
            delivery_plan_ref: "plan_fallback".to_string(),
            simulation_context: "ctx".to_string(),
            idempotency_key: "idem_2a".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn at_bcast_contract_03_ok_requires_safety_flags_true() {
        let out = Ph1BcastOk::v1(
            BcastCapabilityId::AckRecordCommit,
            BCAST_ACK_COMMIT.to_string(),
            ReasonCodeId(100),
            BcastOutcome::AckCommit(BcastAckCommitResult {
                broadcast_id: broadcast_id(),
                recipient_id: recipient_id(),
                ack_status: BcastAckStatus::Received,
                recipient_state: BcastRecipientState::Concluded,
                reason_code: ReasonCodeId(101),
            }),
            true,
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_bcast_contract_04_ack_commit_request_validates() {
        let req = BcastAckCommitRequest {
            tenant_id: tenant(),
            recipient_user_id: recipient(),
            broadcast_id: broadcast_id(),
            recipient_id: recipient_id(),
            ack_status: BcastAckStatus::ActionConfirmed,
            idempotency_key: "idem_ack".to_string(),
        };
        assert!(req.validate().is_ok());
    }
}
