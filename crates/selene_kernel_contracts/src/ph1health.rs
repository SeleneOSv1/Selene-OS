#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1HEALTH_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1HEALTH_ENGINE_ID: &str = "PH1.HEALTH";

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
            reason: "exceeds max length",
        });
    }
    if !value.is_ascii() || value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be printable ASCII",
        });
    }
    Ok(())
}

fn validate_opt_ascii_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_ascii_token(field, v, max_len)?;
    }
    Ok(())
}

fn validate_text(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthCapabilityId {
    HealthSnapshotRead,
    HealthIssueTimelineRead,
    HealthUnresolvedSummaryRead,
}

impl HealthCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            HealthCapabilityId::HealthSnapshotRead => "HEALTH_SNAPSHOT_READ",
            HealthCapabilityId::HealthIssueTimelineRead => "HEALTH_ISSUE_TIMELINE_READ",
            HealthCapabilityId::HealthUnresolvedSummaryRead => "HEALTH_UNRESOLVED_SUMMARY_READ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthSeverity {
    Info,
    Warn,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthIssueStatus {
    Open,
    Resolved,
    Escalated,
}

impl HealthIssueStatus {
    pub fn unresolved(self) -> bool {
        !matches!(self, Self::Resolved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthActionResult {
    Pass,
    Fail,
    Retry,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthAckState {
    Waiting,
    Acknowledged,
    FollowupPending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReadEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub as_of: MonotonicTimeNs,
}

impl HealthReadEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        as_of: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            as_of,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for HealthReadEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_read_envelope.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.as_of.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_read_envelope.as_of",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueEvent {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: HealthSeverity,
    pub status: HealthIssueStatus,
    pub action_id: String,
    pub action_result: HealthActionResult,
    pub attempt_no: u16,
    pub reason_code: ReasonCodeId,
    pub started_at: MonotonicTimeNs,
    pub completed_at: Option<MonotonicTimeNs>,
    pub unresolved_deadline_at: Option<MonotonicTimeNs>,
    pub bcast_id: Option<String>,
    pub ack_state: Option<HealthAckState>,
}

impl HealthIssueEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        issue_id: String,
        owner_engine_id: String,
        severity: HealthSeverity,
        status: HealthIssueStatus,
        action_id: String,
        action_result: HealthActionResult,
        attempt_no: u16,
        reason_code: ReasonCodeId,
        started_at: MonotonicTimeNs,
        completed_at: Option<MonotonicTimeNs>,
        unresolved_deadline_at: Option<MonotonicTimeNs>,
        bcast_id: Option<String>,
        ack_state: Option<HealthAckState>,
    ) -> Result<Self, ContractViolation> {
        let ev = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            tenant_id,
            issue_id,
            owner_engine_id,
            severity,
            status,
            action_id,
            action_result,
            attempt_no,
            reason_code,
            started_at,
            completed_at,
            unresolved_deadline_at,
            bcast_id,
            ack_state,
        };
        ev.validate()?;
        Ok(ev)
    }
}

impl Validate for HealthIssueEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_event.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        validate_ascii_token("health_issue_event.issue_id", &self.issue_id, 128)?;
        validate_ascii_token(
            "health_issue_event.owner_engine_id",
            &self.owner_engine_id,
            64,
        )?;
        validate_ascii_token("health_issue_event.action_id", &self.action_id, 128)?;
        if self.attempt_no == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_event.attempt_no",
                reason: "must be > 0",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_event.reason_code",
                reason: "must be > 0",
            });
        }
        if self.started_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_event.started_at",
                reason: "must be > 0",
            });
        }
        if let Some(completed_at) = self.completed_at {
            if completed_at.0 < self.started_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.completed_at",
                    reason: "must be >= started_at",
                });
            }
        }
        if let Some(deadline_at) = self.unresolved_deadline_at {
            if deadline_at.0 < self.started_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.unresolved_deadline_at",
                    reason: "must be >= started_at",
                });
            }
        }
        validate_opt_ascii_token("health_issue_event.bcast_id", &self.bcast_id, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshotReadRequest {
    pub schema_version: SchemaVersion,
    pub envelope: HealthReadEnvelope,
    pub tenant_id: TenantId,
    pub viewer_user_id: String,
    pub open_only: bool,
    pub severity_filter: Option<HealthSeverity>,
    pub engine_owner_filter: Option<String>,
    pub escalated_only: bool,
    pub max_issue_rows: u16,
    pub issue_events: Vec<HealthIssueEvent>,
}

impl HealthSnapshotReadRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: HealthReadEnvelope,
        tenant_id: TenantId,
        viewer_user_id: String,
        open_only: bool,
        severity_filter: Option<HealthSeverity>,
        engine_owner_filter: Option<String>,
        escalated_only: bool,
        max_issue_rows: u16,
        issue_events: Vec<HealthIssueEvent>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            envelope,
            tenant_id,
            viewer_user_id,
            open_only,
            severity_filter,
            engine_owner_filter,
            escalated_only,
            max_issue_rows,
            issue_events,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for HealthSnapshotReadRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_request.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        validate_ascii_token(
            "health_snapshot_read_request.viewer_user_id",
            &self.viewer_user_id,
            128,
        )?;
        validate_opt_ascii_token(
            "health_snapshot_read_request.engine_owner_filter",
            &self.engine_owner_filter,
            64,
        )?;
        if self.max_issue_rows == 0 || self.max_issue_rows > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_request.max_issue_rows",
                reason: "must be within 1..=512",
            });
        }
        if self.issue_events.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_request.issue_events",
                reason: "must be <= 4096",
            });
        }
        for event in &self.issue_events {
            event.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueTimelineReadRequest {
    pub schema_version: SchemaVersion,
    pub envelope: HealthReadEnvelope,
    pub tenant_id: TenantId,
    pub viewer_user_id: String,
    pub issue_id: String,
    pub max_timeline_rows: u16,
    pub issue_events: Vec<HealthIssueEvent>,
}

impl HealthIssueTimelineReadRequest {
    pub fn v1(
        envelope: HealthReadEnvelope,
        tenant_id: TenantId,
        viewer_user_id: String,
        issue_id: String,
        max_timeline_rows: u16,
        issue_events: Vec<HealthIssueEvent>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            envelope,
            tenant_id,
            viewer_user_id,
            issue_id,
            max_timeline_rows,
            issue_events,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for HealthIssueTimelineReadRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_request.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        validate_ascii_token(
            "health_issue_timeline_read_request.viewer_user_id",
            &self.viewer_user_id,
            128,
        )?;
        validate_ascii_token(
            "health_issue_timeline_read_request.issue_id",
            &self.issue_id,
            128,
        )?;
        if self.max_timeline_rows == 0 || self.max_timeline_rows > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_request.max_timeline_rows",
                reason: "must be within 1..=2048",
            });
        }
        if self.issue_events.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_request.issue_events",
                reason: "must be <= 4096",
            });
        }
        for event in &self.issue_events {
            event.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthUnresolvedSummaryReadRequest {
    pub schema_version: SchemaVersion,
    pub envelope: HealthReadEnvelope,
    pub tenant_id: TenantId,
    pub viewer_user_id: String,
    pub breach_only: bool,
    pub max_issue_rows: u16,
    pub issue_events: Vec<HealthIssueEvent>,
}

impl HealthUnresolvedSummaryReadRequest {
    pub fn v1(
        envelope: HealthReadEnvelope,
        tenant_id: TenantId,
        viewer_user_id: String,
        breach_only: bool,
        max_issue_rows: u16,
        issue_events: Vec<HealthIssueEvent>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            envelope,
            tenant_id,
            viewer_user_id,
            breach_only,
            max_issue_rows,
            issue_events,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for HealthUnresolvedSummaryReadRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_request.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        validate_ascii_token(
            "health_unresolved_summary_read_request.viewer_user_id",
            &self.viewer_user_id,
            128,
        )?;
        if self.max_issue_rows == 0 || self.max_issue_rows > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_request.max_issue_rows",
                reason: "must be within 1..=512",
            });
        }
        if self.issue_events.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_request.issue_events",
                reason: "must be <= 4096",
            });
        }
        for event in &self.issue_events {
            event.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1HealthRequest {
    HealthSnapshotRead(HealthSnapshotReadRequest),
    HealthIssueTimelineRead(HealthIssueTimelineReadRequest),
    HealthUnresolvedSummaryRead(HealthUnresolvedSummaryReadRequest),
}

impl Ph1HealthRequest {
    pub fn capability_id(&self) -> HealthCapabilityId {
        match self {
            Ph1HealthRequest::HealthSnapshotRead(_) => HealthCapabilityId::HealthSnapshotRead,
            Ph1HealthRequest::HealthIssueTimelineRead(_) => {
                HealthCapabilityId::HealthIssueTimelineRead
            }
            Ph1HealthRequest::HealthUnresolvedSummaryRead(_) => {
                HealthCapabilityId::HealthUnresolvedSummaryRead
            }
        }
    }
}

impl Validate for Ph1HealthRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1HealthRequest::HealthSnapshotRead(req) => req.validate(),
            Ph1HealthRequest::HealthIssueTimelineRead(req) => req.validate(),
            Ph1HealthRequest::HealthUnresolvedSummaryRead(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueSnapshotRow {
    pub schema_version: SchemaVersion,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: HealthSeverity,
    pub status: HealthIssueStatus,
    pub latest_reason_code: ReasonCodeId,
    pub latest_started_at: MonotonicTimeNs,
    pub unresolved_deadline_at: Option<MonotonicTimeNs>,
    pub bcast_id: Option<String>,
    pub ack_state: Option<HealthAckState>,
}

impl HealthIssueSnapshotRow {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        issue_id: String,
        owner_engine_id: String,
        severity: HealthSeverity,
        status: HealthIssueStatus,
        latest_reason_code: ReasonCodeId,
        latest_started_at: MonotonicTimeNs,
        unresolved_deadline_at: Option<MonotonicTimeNs>,
        bcast_id: Option<String>,
        ack_state: Option<HealthAckState>,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            issue_id,
            owner_engine_id,
            severity,
            status,
            latest_reason_code,
            latest_started_at,
            unresolved_deadline_at,
            bcast_id,
            ack_state,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for HealthIssueSnapshotRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_snapshot_row.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        validate_ascii_token("health_issue_snapshot_row.issue_id", &self.issue_id, 128)?;
        validate_ascii_token(
            "health_issue_snapshot_row.owner_engine_id",
            &self.owner_engine_id,
            64,
        )?;
        if self.latest_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_snapshot_row.latest_reason_code",
                reason: "must be > 0",
            });
        }
        if self.latest_started_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_snapshot_row.latest_started_at",
                reason: "must be > 0",
            });
        }
        validate_opt_ascii_token("health_issue_snapshot_row.bcast_id", &self.bcast_id, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueTimelineEntry {
    pub schema_version: SchemaVersion,
    pub attempt_no: u16,
    pub action_id: String,
    pub action_result: HealthActionResult,
    pub reason_code: ReasonCodeId,
    pub started_at: MonotonicTimeNs,
    pub completed_at: Option<MonotonicTimeNs>,
}

impl HealthIssueTimelineEntry {
    pub fn v1(
        attempt_no: u16,
        action_id: String,
        action_result: HealthActionResult,
        reason_code: ReasonCodeId,
        started_at: MonotonicTimeNs,
        completed_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            attempt_no,
            action_id,
            action_result,
            reason_code,
            started_at,
            completed_at,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for HealthIssueTimelineEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_entry.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.attempt_no == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_entry.attempt_no",
                reason: "must be > 0",
            });
        }
        validate_ascii_token(
            "health_issue_timeline_entry.action_id",
            &self.action_id,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_entry.reason_code",
                reason: "must be > 0",
            });
        }
        if self.started_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_entry.started_at",
                reason: "must be > 0",
            });
        }
        if let Some(completed_at) = self.completed_at {
            if completed_at.0 < self.started_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_timeline_entry.completed_at",
                    reason: "must be >= started_at",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueTimelineMetadata {
    pub schema_version: SchemaVersion,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: HealthSeverity,
    pub status: HealthIssueStatus,
    pub latest_reason_code: ReasonCodeId,
    pub unresolved_deadline_at: Option<MonotonicTimeNs>,
    pub bcast_id: Option<String>,
    pub ack_state: Option<HealthAckState>,
}

impl HealthIssueTimelineMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        issue_id: String,
        owner_engine_id: String,
        severity: HealthSeverity,
        status: HealthIssueStatus,
        latest_reason_code: ReasonCodeId,
        unresolved_deadline_at: Option<MonotonicTimeNs>,
        bcast_id: Option<String>,
        ack_state: Option<HealthAckState>,
    ) -> Result<Self, ContractViolation> {
        let m = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            issue_id,
            owner_engine_id,
            severity,
            status,
            latest_reason_code,
            unresolved_deadline_at,
            bcast_id,
            ack_state,
        };
        m.validate()?;
        Ok(m)
    }
}

impl Validate for HealthIssueTimelineMetadata {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_metadata.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        validate_ascii_token(
            "health_issue_timeline_metadata.issue_id",
            &self.issue_id,
            128,
        )?;
        validate_ascii_token(
            "health_issue_timeline_metadata.owner_engine_id",
            &self.owner_engine_id,
            64,
        )?;
        if self.latest_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_metadata.latest_reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_ascii_token(
            "health_issue_timeline_metadata.bcast_id",
            &self.bcast_id,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthUnresolvedSummaryRow {
    pub schema_version: SchemaVersion,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: HealthSeverity,
    pub latest_reason_code: ReasonCodeId,
    pub sla_breached: bool,
    pub escalated: bool,
    pub unresolved_deadline_at: Option<MonotonicTimeNs>,
    pub bcast_id: Option<String>,
    pub ack_state: Option<HealthAckState>,
}

impl HealthUnresolvedSummaryRow {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        issue_id: String,
        owner_engine_id: String,
        severity: HealthSeverity,
        latest_reason_code: ReasonCodeId,
        sla_breached: bool,
        escalated: bool,
        unresolved_deadline_at: Option<MonotonicTimeNs>,
        bcast_id: Option<String>,
        ack_state: Option<HealthAckState>,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            issue_id,
            owner_engine_id,
            severity,
            latest_reason_code,
            sla_breached,
            escalated,
            unresolved_deadline_at,
            bcast_id,
            ack_state,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for HealthUnresolvedSummaryRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_row.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        validate_ascii_token(
            "health_unresolved_summary_row.issue_id",
            &self.issue_id,
            128,
        )?;
        validate_ascii_token(
            "health_unresolved_summary_row.owner_engine_id",
            &self.owner_engine_id,
            64,
        )?;
        if self.latest_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_row.latest_reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_ascii_token(
            "health_unresolved_summary_row.bcast_id",
            &self.bcast_id,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshotReadOk {
    pub schema_version: SchemaVersion,
    pub capability_id: HealthCapabilityId,
    pub reason_code: ReasonCodeId,
    pub open_issue_count: u32,
    pub critical_open_count: u32,
    pub resolved_24h_count: u32,
    pub escalated_24h_count: u32,
    pub mttr_minutes: u32,
    pub issue_rows: Vec<HealthIssueSnapshotRow>,
    pub no_authority_mutation: bool,
}

impl HealthSnapshotReadOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        open_issue_count: u32,
        critical_open_count: u32,
        resolved_24h_count: u32,
        escalated_24h_count: u32,
        mttr_minutes: u32,
        issue_rows: Vec<HealthIssueSnapshotRow>,
        no_authority_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            capability_id: HealthCapabilityId::HealthSnapshotRead,
            reason_code,
            open_issue_count,
            critical_open_count,
            resolved_24h_count,
            escalated_24h_count,
            mttr_minutes,
            issue_rows,
            no_authority_mutation,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for HealthSnapshotReadOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_ok.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != HealthCapabilityId::HealthSnapshotRead {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_ok.capability_id",
                reason: "must be HEALTH_SNAPSHOT_READ",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_ok.reason_code",
                reason: "must be > 0",
            });
        }
        if self.issue_rows.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_ok.issue_rows",
                reason: "must be <= 512",
            });
        }
        for row in &self.issue_rows {
            row.validate()?;
        }
        if !self.no_authority_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "health_snapshot_read_ok.no_authority_mutation",
                reason: "must remain true for display-only engine",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthIssueTimelineReadOk {
    pub schema_version: SchemaVersion,
    pub capability_id: HealthCapabilityId,
    pub reason_code: ReasonCodeId,
    pub issue_metadata: HealthIssueTimelineMetadata,
    pub timeline_entries: Vec<HealthIssueTimelineEntry>,
    pub no_authority_mutation: bool,
}

impl HealthIssueTimelineReadOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        issue_metadata: HealthIssueTimelineMetadata,
        timeline_entries: Vec<HealthIssueTimelineEntry>,
        no_authority_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            capability_id: HealthCapabilityId::HealthIssueTimelineRead,
            reason_code,
            issue_metadata,
            timeline_entries,
            no_authority_mutation,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for HealthIssueTimelineReadOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_ok.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != HealthCapabilityId::HealthIssueTimelineRead {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_ok.capability_id",
                reason: "must be HEALTH_ISSUE_TIMELINE_READ",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.issue_metadata.validate()?;
        if self.timeline_entries.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_ok.timeline_entries",
                reason: "must be <= 2048",
            });
        }
        for entry in &self.timeline_entries {
            entry.validate()?;
        }
        if !self.no_authority_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_timeline_read_ok.no_authority_mutation",
                reason: "must remain true for display-only engine",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthUnresolvedSummaryReadOk {
    pub schema_version: SchemaVersion,
    pub capability_id: HealthCapabilityId,
    pub reason_code: ReasonCodeId,
    pub unresolved_issue_count: u32,
    pub sla_breach_issue_count: u32,
    pub escalated_issue_count: u32,
    pub issue_rows: Vec<HealthUnresolvedSummaryRow>,
    pub no_authority_mutation: bool,
}

impl HealthUnresolvedSummaryReadOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        unresolved_issue_count: u32,
        sla_breach_issue_count: u32,
        escalated_issue_count: u32,
        issue_rows: Vec<HealthUnresolvedSummaryRow>,
        no_authority_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            capability_id: HealthCapabilityId::HealthUnresolvedSummaryRead,
            reason_code,
            unresolved_issue_count,
            sla_breach_issue_count,
            escalated_issue_count,
            issue_rows,
            no_authority_mutation,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for HealthUnresolvedSummaryReadOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_ok.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != HealthCapabilityId::HealthUnresolvedSummaryRead {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_ok.capability_id",
                reason: "must be HEALTH_UNRESOLVED_SUMMARY_READ",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_ok.reason_code",
                reason: "must be > 0",
            });
        }
        if self.issue_rows.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_ok.issue_rows",
                reason: "must be <= 512",
            });
        }
        for row in &self.issue_rows {
            row.validate()?;
        }
        if !self.no_authority_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "health_unresolved_summary_read_ok.no_authority_mutation",
                reason: "must remain true for display-only engine",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: HealthCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
    pub no_authority_mutation: bool,
}

impl HealthRefuse {
    pub fn v1(
        capability_id: HealthCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
            no_authority_mutation: true,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for HealthRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_refuse.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        validate_text("health_refuse.message", &self.message, 240)?;
        if !self.no_authority_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "health_refuse.no_authority_mutation",
                reason: "must remain true for display-only engine",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1HealthResponse {
    HealthSnapshotReadOk(HealthSnapshotReadOk),
    HealthIssueTimelineReadOk(HealthIssueTimelineReadOk),
    HealthUnresolvedSummaryReadOk(HealthUnresolvedSummaryReadOk),
    Refuse(HealthRefuse),
}

impl Validate for Ph1HealthResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1HealthResponse::HealthSnapshotReadOk(out) => out.validate(),
            Ph1HealthResponse::HealthIssueTimelineReadOk(out) => out.validate(),
            Ph1HealthResponse::HealthUnresolvedSummaryReadOk(out) => out.validate(),
            Ph1HealthResponse::Refuse(out) => out.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant(v: &str) -> TenantId {
        TenantId::new(v.to_string()).unwrap()
    }

    fn envelope() -> HealthReadEnvelope {
        HealthReadEnvelope::v1(CorrelationId(1), TurnId(1), MonotonicTimeNs(10)).unwrap()
    }

    fn event() -> HealthIssueEvent {
        HealthIssueEvent::v1(
            tenant("tenant_a"),
            "issue_001".to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Warn,
            HealthIssueStatus::Open,
            "ACTION_A".to_string(),
            HealthActionResult::Fail,
            1,
            ReasonCodeId(1001),
            MonotonicTimeNs(5),
            None,
            Some(MonotonicTimeNs(25)),
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_health_contract_01_snapshot_request_is_schema_valid() {
        let req = Ph1HealthRequest::HealthSnapshotRead(
            HealthSnapshotReadRequest::v1(
                envelope(),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                None,
                None,
                false,
                100,
                vec![event()],
            )
            .unwrap(),
        );
        assert!(req.validate().is_ok());
    }

    #[test]
    fn at_health_contract_02_issue_event_rejects_backwards_complete_time() {
        let bad = HealthIssueEvent::v1(
            tenant("tenant_a"),
            "issue_001".to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Warn,
            HealthIssueStatus::Resolved,
            "ACTION_A".to_string(),
            HealthActionResult::Pass,
            1,
            ReasonCodeId(1001),
            MonotonicTimeNs(20),
            Some(MonotonicTimeNs(10)),
            None,
            None,
            None,
        );
        assert!(bad.is_err());
    }

    #[test]
    fn at_health_contract_03_timeline_ok_requires_matching_capability_id() {
        let mut out = HealthIssueTimelineReadOk::v1(
            ReasonCodeId(1002),
            HealthIssueTimelineMetadata::v1(
                "issue_001".to_string(),
                "PH1.C".to_string(),
                HealthSeverity::Warn,
                HealthIssueStatus::Open,
                ReasonCodeId(1001),
                None,
                None,
                None,
            )
            .unwrap(),
            vec![HealthIssueTimelineEntry::v1(
                1,
                "ACTION_A".to_string(),
                HealthActionResult::Fail,
                ReasonCodeId(1001),
                MonotonicTimeNs(10),
                None,
            )
            .unwrap()],
            true,
        )
        .unwrap();

        out.capability_id = HealthCapabilityId::HealthSnapshotRead;
        assert!(out.validate().is_err());
    }

    #[test]
    fn at_health_contract_04_refuse_is_schema_valid() {
        let refuse =
            HealthRefuse::v1(HealthCapabilityId::HealthSnapshotRead, ReasonCodeId(1099),
                "input invalid".to_string())
                .unwrap();
        assert!(refuse.validate().is_ok());
    }
}
