#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1REM_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1REM_ENGINE_ID: &str = "PH1.REM";
pub const PH1REM_IMPLEMENTATION_ID: &str = "PH1.REM.001";
pub const PH1REM_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1REM_IMPLEMENTATION_ID];

pub const REMINDER_SCHEDULE_COMMIT: &str = "REMINDER_SCHEDULE_COMMIT";
pub const REMINDER_UPDATE_COMMIT: &str = "REMINDER_UPDATE_COMMIT";
pub const REMINDER_CANCEL_COMMIT: &str = "REMINDER_CANCEL_COMMIT";
pub const REMINDER_SNOOZE_COMMIT: &str = "REMINDER_SNOOZE_COMMIT";
pub const REMINDER_DELIVER_PRE_COMMIT: &str = "REMINDER_DELIVER_PRE_COMMIT";
pub const REMINDER_DELIVER_DUE_COMMIT: &str = "REMINDER_DELIVER_DUE_COMMIT";
pub const REMINDER_FOLLOWUP_SCHEDULE_COMMIT: &str = "REMINDER_FOLLOWUP_SCHEDULE_COMMIT";
pub const REMINDER_ESCALATE_COMMIT: &str = "REMINDER_ESCALATE_COMMIT";
pub const REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT: &str = "REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT";
pub const REMINDER_MARK_COMPLETED_COMMIT: &str = "REMINDER_MARK_COMPLETED_COMMIT";
pub const REMINDER_MARK_FAILED_COMMIT: &str = "REMINDER_MARK_FAILED_COMMIT";

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
pub enum ReminderLocalTimeMode {
    FixedTimezone,
    LocalTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderPriorityLevel {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReminderType {
    Task,
    Meeting,
    Timer,
    Medical,
    Custom,
    BcastMhpFollowup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderChannel {
    Voice,
    Push,
    Text,
    Email,
    PhoneApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderState {
    Scheduled,
    Snoozed,
    FollowupPending,
    Canceled,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderDeliveryStatus {
    Delivered,
    DeferredQuietHours,
    RetryScheduled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReminderAckSource {
    Voice,
    Text,
    Ui,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReminderId(String);

impl ReminderId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for ReminderId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("reminder_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReminderOccurrenceId(String);

impl ReminderOccurrenceId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for ReminderOccurrenceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("occurrence_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReminderDeliveryAttemptId(String);

impl ReminderDeliveryAttemptId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for ReminderDeliveryAttemptId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("delivery_attempt_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderScheduleCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub device_id: Option<DeviceId>,
    pub reminder_type: ReminderType,
    pub reminder_request_text: String,
    pub desired_time: String,
    pub user_timezone: String,
    pub local_time_mode: ReminderLocalTimeMode,
    pub priority_level: ReminderPriorityLevel,
    pub recurrence_rule: Option<String>,
    pub channel_preferences: Vec<ReminderChannel>,
    pub idempotency_key: String,
}

impl Validate for ReminderScheduleCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_schedule.user_id", self.user_id.as_str(), 128)?;
        if let Some(device_id) = &self.device_id {
            device_id.validate()?;
        }
        validate_token(
            "rem_schedule.reminder_request_text",
            &self.reminder_request_text,
            512,
        )?;
        validate_token("rem_schedule.desired_time", &self.desired_time, 128)?;
        validate_token("rem_schedule.user_timezone", &self.user_timezone, 64)?;
        if let Some(rule) = &self.recurrence_rule {
            validate_token("rem_schedule.recurrence_rule", rule, 256)?;
        }
        if self.channel_preferences.is_empty() || self.channel_preferences.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "rem_schedule.channel_preferences",
                reason: "must contain 1..=8 channels",
            });
        }
        validate_token("rem_schedule.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderUpdateFields {
    pub reminder_request_text: Option<String>,
    pub desired_time: Option<String>,
    pub user_timezone: Option<String>,
    pub local_time_mode: Option<ReminderLocalTimeMode>,
    pub priority_level: Option<ReminderPriorityLevel>,
    pub recurrence_rule: Option<Option<String>>,
    pub channel_preferences: Option<Vec<ReminderChannel>>,
}

impl ReminderUpdateFields {
    pub fn empty() -> Self {
        Self {
            reminder_request_text: None,
            desired_time: None,
            user_timezone: None,
            local_time_mode: None,
            priority_level: None,
            recurrence_rule: None,
            channel_preferences: None,
        }
    }
}

impl Validate for ReminderUpdateFields {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_opt_token(
            "rem_update.fields.reminder_request_text",
            &self.reminder_request_text,
            512,
        )?;
        validate_opt_token("rem_update.fields.desired_time", &self.desired_time, 128)?;
        validate_opt_token("rem_update.fields.user_timezone", &self.user_timezone, 64)?;
        if let Some(recurrence_rule) = &self.recurrence_rule {
            validate_opt_token("rem_update.fields.recurrence_rule", recurrence_rule, 256)?;
        }
        if let Some(channel_preferences) = &self.channel_preferences {
            if channel_preferences.is_empty() || channel_preferences.len() > 8 {
                return Err(ContractViolation::InvalidValue {
                    field: "rem_update.fields.channel_preferences",
                    reason: "must contain 1..=8 channels when present",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderUpdateCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub updated_fields: ReminderUpdateFields,
    pub idempotency_key: String,
}

impl Validate for ReminderUpdateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_update.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.updated_fields.validate()?;
        validate_token("rem_update.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderCancelCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub cancel_reason: Option<String>,
    pub idempotency_key: String,
}

impl Validate for ReminderCancelCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_cancel.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        validate_opt_token("rem_cancel.cancel_reason", &self.cancel_reason, 256)?;
        validate_token("rem_cancel.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderSnoozeCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub snooze_duration_ms: u32,
    pub idempotency_key: String,
}

impl Validate for ReminderSnoozeCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_snooze.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        if self.snooze_duration_ms == 0 || self.snooze_duration_ms > 86_400_000 {
            return Err(ContractViolation::InvalidValue {
                field: "rem_snooze.snooze_duration_ms",
                reason: "must be within 1..=86400000",
            });
        }
        validate_token("rem_snooze.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderFollowupScheduleCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub followup_delay_ms: u32,
    pub idempotency_key: String,
}

impl Validate for ReminderFollowupScheduleCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_followup.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        if self.followup_delay_ms == 0 || self.followup_delay_ms > 604_800_000 {
            return Err(ContractViolation::InvalidValue {
                field: "rem_followup.followup_delay_ms",
                reason: "must be within 1..=604800000",
            });
        }
        validate_token("rem_followup.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderDeliveryRetryScheduleCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub retry_time: MonotonicTimeNs,
    pub idempotency_key: String,
}

impl Validate for ReminderDeliveryRetryScheduleCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_retry.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        if self.retry_time.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rem_retry.retry_time",
                reason: "must be > 0",
            });
        }
        validate_token("rem_retry.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderDeliverPreCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub delivery_channel: ReminderChannel,
    pub delivery_attempt_id: ReminderDeliveryAttemptId,
    pub idempotency_key: String,
}

impl Validate for ReminderDeliverPreCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_deliver_pre.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        self.delivery_attempt_id.validate()?;
        validate_token(
            "rem_deliver_pre.idempotency_key",
            &self.idempotency_key,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderDeliverDueCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub delivery_channel: ReminderChannel,
    pub delivery_attempt_id: ReminderDeliveryAttemptId,
    pub offline_state: bool,
    pub idempotency_key: String,
}

impl Validate for ReminderDeliverDueCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_deliver_due.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        self.delivery_attempt_id.validate()?;
        validate_token(
            "rem_deliver_due.idempotency_key",
            &self.idempotency_key,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderEscalateCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub from_channel: ReminderChannel,
    pub to_channel: ReminderChannel,
    pub delivery_attempt_id: ReminderDeliveryAttemptId,
    pub idempotency_key: String,
}

impl Validate for ReminderEscalateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_escalate.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        self.delivery_attempt_id.validate()?;
        validate_token("rem_escalate.idempotency_key", &self.idempotency_key, 128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderMarkCompletedCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub ack_source: ReminderAckSource,
    pub idempotency_key: String,
}

impl Validate for ReminderMarkCompletedCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_mark_completed.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        validate_token(
            "rem_mark_completed.idempotency_key",
            &self.idempotency_key,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderMarkFailedCommitRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub reminder_id: ReminderId,
    pub occurrence_id: ReminderOccurrenceId,
    pub failure_reason: String,
    pub idempotency_key: String,
}

impl Validate for ReminderMarkFailedCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("rem_mark_failed.user_id", self.user_id.as_str(), 128)?;
        self.reminder_id.validate()?;
        self.occurrence_id.validate()?;
        validate_token("rem_mark_failed.failure_reason", &self.failure_reason, 256)?;
        validate_token(
            "rem_mark_failed.idempotency_key",
            &self.idempotency_key,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderRequest {
    ScheduleCommit(ReminderScheduleCommitRequest),
    UpdateCommit(ReminderUpdateCommitRequest),
    CancelCommit(ReminderCancelCommitRequest),
    SnoozeCommit(ReminderSnoozeCommitRequest),
    DeliverPreCommit(ReminderDeliverPreCommitRequest),
    DeliverDueCommit(ReminderDeliverDueCommitRequest),
    FollowupScheduleCommit(ReminderFollowupScheduleCommitRequest),
    EscalateCommit(ReminderEscalateCommitRequest),
    DeliveryRetryScheduleCommit(ReminderDeliveryRetryScheduleCommitRequest),
    MarkCompletedCommit(ReminderMarkCompletedCommitRequest),
    MarkFailedCommit(ReminderMarkFailedCommitRequest),
}

impl ReminderRequest {
    pub fn expected_simulation_id(&self) -> &'static str {
        match self {
            ReminderRequest::ScheduleCommit(_) => REMINDER_SCHEDULE_COMMIT,
            ReminderRequest::UpdateCommit(_) => REMINDER_UPDATE_COMMIT,
            ReminderRequest::CancelCommit(_) => REMINDER_CANCEL_COMMIT,
            ReminderRequest::SnoozeCommit(_) => REMINDER_SNOOZE_COMMIT,
            ReminderRequest::DeliverPreCommit(_) => REMINDER_DELIVER_PRE_COMMIT,
            ReminderRequest::DeliverDueCommit(_) => REMINDER_DELIVER_DUE_COMMIT,
            ReminderRequest::FollowupScheduleCommit(_) => REMINDER_FOLLOWUP_SCHEDULE_COMMIT,
            ReminderRequest::EscalateCommit(_) => REMINDER_ESCALATE_COMMIT,
            ReminderRequest::DeliveryRetryScheduleCommit(_) => {
                REMINDER_DELIVERY_RETRY_SCHEDULE_COMMIT
            }
            ReminderRequest::MarkCompletedCommit(_) => REMINDER_MARK_COMPLETED_COMMIT,
            ReminderRequest::MarkFailedCommit(_) => REMINDER_MARK_FAILED_COMMIT,
        }
    }
}

impl Validate for ReminderRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            ReminderRequest::ScheduleCommit(v) => v.validate(),
            ReminderRequest::UpdateCommit(v) => v.validate(),
            ReminderRequest::CancelCommit(v) => v.validate(),
            ReminderRequest::SnoozeCommit(v) => v.validate(),
            ReminderRequest::DeliverPreCommit(v) => v.validate(),
            ReminderRequest::DeliverDueCommit(v) => v.validate(),
            ReminderRequest::FollowupScheduleCommit(v) => v.validate(),
            ReminderRequest::EscalateCommit(v) => v.validate(),
            ReminderRequest::DeliveryRetryScheduleCommit(v) => v.validate(),
            ReminderRequest::MarkCompletedCommit(v) => v.validate(),
            ReminderRequest::MarkFailedCommit(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
}

impl ReminderRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        simulation_id: String,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1REM_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for ReminderRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1REM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rem_request_envelope.schema_version",
                reason: "must match PH1REM_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rem_request_envelope.now",
                reason: "must be > 0",
            });
        }
        validate_token(
            "rem_request_envelope.simulation_id",
            &self.simulation_id,
            128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1RemRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ReminderRequestEnvelope,
    pub request: ReminderRequest,
}

impl Ph1RemRequest {
    pub fn v1(
        envelope: ReminderRequestEnvelope,
        request: ReminderRequest,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1REM_CONTRACT_VERSION,
            envelope,
            request,
        };
        v.validate()?;
        Ok(v)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn schedule_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        tenant_id: TenantId,
        user_id: UserId,
        device_id: Option<DeviceId>,
        reminder_type: ReminderType,
        reminder_request_text: String,
        desired_time: String,
        user_timezone: String,
        local_time_mode: ReminderLocalTimeMode,
        priority_level: ReminderPriorityLevel,
        recurrence_rule: Option<String>,
        channel_preferences: Vec<ReminderChannel>,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            ReminderRequestEnvelope::v1(
                correlation_id,
                turn_id,
                now,
                REMINDER_SCHEDULE_COMMIT.to_string(),
            )?,
            ReminderRequest::ScheduleCommit(ReminderScheduleCommitRequest {
                tenant_id,
                user_id,
                device_id,
                reminder_type,
                reminder_request_text,
                desired_time,
                user_timezone,
                local_time_mode,
                priority_level,
                recurrence_rule,
                channel_preferences,
                idempotency_key,
            }),
        )
    }
}

impl Validate for Ph1RemRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1REM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_request.schema_version",
                reason: "must match PH1REM_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.request.validate()?;
        if self.envelope.simulation_id != self.request.expected_simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_request.envelope.simulation_id",
                reason: "does not match request type",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1RemOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub reminder_id: ReminderId,
    pub occurrence_id: Option<ReminderOccurrenceId>,
    pub state: ReminderState,
    pub scheduled_time: Option<MonotonicTimeNs>,
    pub delivery_status: Option<ReminderDeliveryStatus>,
    pub delivery_attempt_id: Option<ReminderDeliveryAttemptId>,
    pub delivery_proof_ref: Option<String>,
    pub escalation_level: Option<u8>,
}

impl Ph1RemOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        reminder_id: ReminderId,
        occurrence_id: Option<ReminderOccurrenceId>,
        state: ReminderState,
        scheduled_time: Option<MonotonicTimeNs>,
        delivery_status: Option<ReminderDeliveryStatus>,
        delivery_attempt_id: Option<ReminderDeliveryAttemptId>,
        delivery_proof_ref: Option<String>,
        escalation_level: Option<u8>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1REM_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            reminder_id,
            occurrence_id,
            state,
            scheduled_time,
            delivery_status,
            delivery_attempt_id,
            delivery_proof_ref,
            escalation_level,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1RemOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1REM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_ok.schema_version",
                reason: "must match PH1REM_CONTRACT_VERSION",
            });
        }
        validate_token("ph1rem_ok.simulation_id", &self.simulation_id, 128)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.reminder_id.validate()?;
        if let Some(occurrence_id) = &self.occurrence_id {
            occurrence_id.validate()?;
        }
        if let Some(delivery_attempt_id) = &self.delivery_attempt_id {
            delivery_attempt_id.validate()?;
        }
        validate_opt_token(
            "ph1rem_ok.delivery_proof_ref",
            &self.delivery_proof_ref,
            256,
        )?;
        if self.delivery_status.is_some() && self.delivery_attempt_id.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_ok.delivery_attempt_id",
                reason: "required when delivery_status is present",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1RemRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1RemRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1REM_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1RemRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1REM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_refuse.schema_version",
                reason: "must match PH1REM_CONTRACT_VERSION",
            });
        }
        validate_token("ph1rem_refuse.simulation_id", &self.simulation_id, 128)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rem_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_token("ph1rem_refuse.message", &self.message, 256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1RemResponse {
    Ok(Ph1RemOk),
    Refuse(Ph1RemRefuse),
}

impl Validate for Ph1RemResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1RemResponse::Ok(v) => v.validate(),
            Ph1RemResponse::Refuse(v) => v.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_rem_contract_01_schedule_simulation_id_must_match() {
        let req = Ph1RemRequest::v1(
            ReminderRequestEnvelope::v1(
                CorrelationId(11),
                TurnId(22),
                MonotonicTimeNs(33),
                REMINDER_CANCEL_COMMIT.to_string(),
            )
            .unwrap(),
            ReminderRequest::ScheduleCommit(ReminderScheduleCommitRequest {
                tenant_id: TenantId::new("tenant_demo").unwrap(),
                user_id: UserId::new("user_demo").unwrap(),
                device_id: None,
                reminder_type: ReminderType::Task,
                reminder_request_text: "payroll cutoff".to_string(),
                desired_time: "in 2 hours".to_string(),
                user_timezone: "America/Los_Angeles".to_string(),
                local_time_mode: ReminderLocalTimeMode::LocalTime,
                priority_level: ReminderPriorityLevel::Normal,
                recurrence_rule: None,
                channel_preferences: vec![ReminderChannel::Text],
                idempotency_key: "idem_1".to_string(),
            }),
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_rem_contract_02_ok_delivery_requires_attempt_id() {
        let ok = Ph1RemOk::v1(
            REMINDER_DELIVER_DUE_COMMIT.to_string(),
            ReasonCodeId(1),
            ReminderId::new("rem_1").unwrap(),
            Some(ReminderOccurrenceId::new("occ_1").unwrap()),
            ReminderState::Scheduled,
            Some(MonotonicTimeNs(100)),
            Some(ReminderDeliveryStatus::Delivered),
            None,
            Some("proof_1".to_string()),
            None,
        );

        assert!(ok.is_err());
    }
}
