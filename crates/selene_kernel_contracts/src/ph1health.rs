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

fn validate_text(
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
    HealthReportQueryRead,
}

impl HealthCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            HealthCapabilityId::HealthSnapshotRead => "HEALTH_SNAPSHOT_READ",
            HealthCapabilityId::HealthIssueTimelineRead => "HEALTH_ISSUE_TIMELINE_READ",
            HealthCapabilityId::HealthUnresolvedSummaryRead => "HEALTH_UNRESOLVED_SUMMARY_READ",
            HealthCapabilityId::HealthReportQueryRead => "HEALTH_REPORT_QUERY_READ",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthReportKind {
    MissedStt,
    UnresolvedEscalated,
    IssueStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthCompanyScope {
    TenantOnly,
    CrossTenantTenantRows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthDisplayTarget {
    Desktop,
    Phone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthPageAction {
    First,
    Next,
    Prev,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReportTimeRange {
    pub schema_version: SchemaVersion,
    pub from_utc: MonotonicTimeNs,
    pub to_utc: MonotonicTimeNs,
}

impl HealthReportTimeRange {
    pub fn v1(
        from_utc: MonotonicTimeNs,
        to_utc: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let range = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            from_utc,
            to_utc,
        };
        range.validate()?;
        Ok(range)
    }
}

impl Validate for HealthReportTimeRange {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_time_range.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.from_utc.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_time_range.from_utc",
                reason: "must be > 0",
            });
        }
        if self.to_utc.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_time_range.to_utc",
                reason: "must be > 0",
            });
        }
        if self.from_utc.0 > self.to_utc.0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_time_range",
                reason: "from_utc must be <= to_utc",
            });
        }
        Ok(())
    }
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
    pub issue_fingerprint: Option<String>,
    pub verification_window_start_at: Option<MonotonicTimeNs>,
    pub verification_window_end_at: Option<MonotonicTimeNs>,
    pub recurrence_observed: Option<bool>,
    pub impact_summary: Option<String>,
    pub attempted_fix_actions: Vec<String>,
    pub current_monitoring_evidence: Option<String>,
    pub unresolved_reason_exact: Option<String>,
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
            issue_fingerprint: None,
            verification_window_start_at: None,
            verification_window_end_at: None,
            recurrence_observed: None,
            impact_summary: None,
            attempted_fix_actions: Vec::new(),
            current_monitoring_evidence: None,
            unresolved_reason_exact: None,
        };
        ev.validate()?;
        Ok(ev)
    }

    pub fn with_resolution_proof(
        mut self,
        issue_fingerprint: Option<String>,
        verification_window_start_at: Option<MonotonicTimeNs>,
        verification_window_end_at: Option<MonotonicTimeNs>,
        recurrence_observed: Option<bool>,
    ) -> Result<Self, ContractViolation> {
        self.issue_fingerprint = issue_fingerprint;
        self.verification_window_start_at = verification_window_start_at;
        self.verification_window_end_at = verification_window_end_at;
        self.recurrence_observed = recurrence_observed;
        self.validate()?;
        Ok(self)
    }

    pub fn with_escalation_payload(
        mut self,
        impact_summary: Option<String>,
        attempted_fix_actions: Vec<String>,
        current_monitoring_evidence: Option<String>,
        unresolved_reason_exact: Option<String>,
    ) -> Result<Self, ContractViolation> {
        self.impact_summary = impact_summary;
        self.attempted_fix_actions = attempted_fix_actions;
        self.current_monitoring_evidence = current_monitoring_evidence;
        self.unresolved_reason_exact = unresolved_reason_exact;
        self.validate()?;
        Ok(self)
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
        validate_opt_ascii_token(
            "health_issue_event.issue_fingerprint",
            &self.issue_fingerprint,
            128,
        )?;
        if let (Some(start), Some(end)) = (
            self.verification_window_start_at,
            self.verification_window_end_at,
        ) {
            if end.0 < start.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.verification_window",
                    reason: "verification_window_end_at must be >= verification_window_start_at",
                });
            }
        }
        let escalation_dispatched =
            self.status == HealthIssueStatus::Escalated || self.bcast_id.is_some();
        if escalation_dispatched {
            if self.bcast_id.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.bcast_id",
                    reason: "must be present when escalation is dispatched",
                });
            }
            let Some(summary) = &self.impact_summary else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.impact_summary",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text("health_issue_event.impact_summary", summary, 512)?;
            let Some(evidence) = &self.current_monitoring_evidence else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.current_monitoring_evidence",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text(
                "health_issue_event.current_monitoring_evidence",
                evidence,
                512,
            )?;
            let Some(reason) = &self.unresolved_reason_exact else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.unresolved_reason_exact",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text("health_issue_event.unresolved_reason_exact", reason, 512)?;
            if self.attempted_fix_actions.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.attempted_fix_actions",
                    reason: "must include at least one attempted fix when escalation is dispatched",
                });
            }
        } else if let Some(summary) = &self.impact_summary {
            validate_text("health_issue_event.impact_summary", summary, 512)?;
        }
        if self.recurrence_observed.unwrap_or(false) {
            if !self.status.unresolved() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.status",
                    reason: "must remain unresolved when recurrence_observed is true",
                });
            }
            if self.issue_fingerprint.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.issue_fingerprint",
                    reason: "must be present when recurrence_observed is true",
                });
            }
            if self.current_monitoring_evidence.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.current_monitoring_evidence",
                    reason: "must be present when recurrence_observed is true",
                });
            }
            if self.unresolved_reason_exact.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_issue_event.unresolved_reason_exact",
                    reason: "must be present when recurrence_observed is true",
                });
            }
        }
        if self.attempted_fix_actions.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "health_issue_event.attempted_fix_actions",
                reason: "must be <= 32 entries",
            });
        }
        for action in &self.attempted_fix_actions {
            validate_text("health_issue_event.attempted_fix_actions[]", action, 160)?;
        }
        if !escalation_dispatched {
            if let Some(evidence) = &self.current_monitoring_evidence {
                validate_text(
                    "health_issue_event.current_monitoring_evidence",
                    evidence,
                    512,
                )?;
            }
        }
        if !escalation_dispatched {
            if let Some(reason) = &self.unresolved_reason_exact {
                validate_text("health_issue_event.unresolved_reason_exact", reason, 512)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReportQueryReadRequest {
    pub schema_version: SchemaVersion,
    pub envelope: HealthReadEnvelope,
    pub tenant_id: TenantId,
    pub viewer_user_id: String,
    pub report_kind: HealthReportKind,
    pub time_range: HealthReportTimeRange,
    pub engine_owner_filter: Option<String>,
    pub company_scope: HealthCompanyScope,
    pub company_ids: Vec<TenantId>,
    pub country_codes: Vec<String>,
    pub escalated_only: bool,
    pub unresolved_only: bool,
    pub display_target: Option<HealthDisplayTarget>,
    pub page_action: HealthPageAction,
    pub page_cursor: Option<String>,
    pub report_context_id: Option<String>,
    pub page_size: u16,
    pub issue_events: Vec<HealthIssueEvent>,
}

impl HealthReportQueryReadRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: HealthReadEnvelope,
        tenant_id: TenantId,
        viewer_user_id: String,
        report_kind: HealthReportKind,
        time_range: HealthReportTimeRange,
        engine_owner_filter: Option<String>,
        company_scope: HealthCompanyScope,
        company_ids: Vec<TenantId>,
        country_codes: Vec<String>,
        escalated_only: bool,
        unresolved_only: bool,
        display_target: Option<HealthDisplayTarget>,
        page_action: HealthPageAction,
        page_cursor: Option<String>,
        report_context_id: Option<String>,
        page_size: u16,
        issue_events: Vec<HealthIssueEvent>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            envelope,
            tenant_id,
            viewer_user_id,
            report_kind,
            time_range,
            engine_owner_filter,
            company_scope,
            company_ids,
            country_codes,
            escalated_only,
            unresolved_only,
            display_target,
            page_action,
            page_cursor,
            report_context_id,
            page_size,
            issue_events,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for HealthReportQueryReadRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.time_range.validate()?;
        validate_ascii_token(
            "health_report_query_read_request.viewer_user_id",
            &self.viewer_user_id,
            128,
        )?;
        validate_opt_ascii_token(
            "health_report_query_read_request.engine_owner_filter",
            &self.engine_owner_filter,
            64,
        )?;
        if self.company_ids.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.company_ids",
                reason: "must be <= 256",
            });
        }
        for company_id in &self.company_ids {
            company_id.validate()?;
        }
        if self.company_scope == HealthCompanyScope::TenantOnly
            && self
                .company_ids
                .iter()
                .any(|id| id.as_str() != self.tenant_id.as_str())
        {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.company_scope",
                reason: "TENANT_ONLY cannot include foreign tenant ids",
            });
        }
        if self.country_codes.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.country_codes",
                reason: "must be <= 32",
            });
        }
        for code in &self.country_codes {
            validate_ascii_token("health_report_query_read_request.country_codes[]", code, 3)?;
            if !code.chars().all(|c| c.is_ascii_uppercase()) {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_read_request.country_codes[]",
                    reason: "must be uppercase ASCII country code",
                });
            }
        }
        validate_opt_ascii_token(
            "health_report_query_read_request.page_cursor",
            &self.page_cursor,
            128,
        )?;
        validate_opt_ascii_token(
            "health_report_query_read_request.report_context_id",
            &self.report_context_id,
            128,
        )?;
        if self.page_size == 0 || self.page_size > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.page_size",
                reason: "must be within 1..=512",
            });
        }
        if self.issue_events.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_request.issue_events",
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
    HealthReportQueryRead(HealthReportQueryReadRequest),
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
            Ph1HealthRequest::HealthReportQueryRead(_) => HealthCapabilityId::HealthReportQueryRead,
        }
    }
}

impl Validate for Ph1HealthRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1HealthRequest::HealthSnapshotRead(req) => req.validate(),
            Ph1HealthRequest::HealthIssueTimelineRead(req) => req.validate(),
            Ph1HealthRequest::HealthUnresolvedSummaryRead(req) => req.validate(),
            Ph1HealthRequest::HealthReportQueryRead(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReportQueryRow {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub issue_id: String,
    pub owner_engine_id: String,
    pub severity: HealthSeverity,
    pub status: HealthIssueStatus,
    pub latest_reason_code: ReasonCodeId,
    pub last_seen_at: MonotonicTimeNs,
    pub bcast_id: Option<String>,
    pub ack_state: Option<HealthAckState>,
    pub issue_fingerprint: Option<String>,
    pub verification_window_start_at: Option<MonotonicTimeNs>,
    pub verification_window_end_at: Option<MonotonicTimeNs>,
    pub recurrence_observed: bool,
    pub impact_summary: Option<String>,
    pub attempted_fix_actions: Vec<String>,
    pub current_monitoring_evidence: Option<String>,
    pub unresolved_reason_exact: Option<String>,
}

impl Validate for HealthReportQueryRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_row.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        validate_ascii_token("health_report_query_row.issue_id", &self.issue_id, 128)?;
        validate_ascii_token(
            "health_report_query_row.owner_engine_id",
            &self.owner_engine_id,
            64,
        )?;
        if self.latest_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_row.latest_reason_code",
                reason: "must be > 0",
            });
        }
        if self.last_seen_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_row.last_seen_at",
                reason: "must be > 0",
            });
        }
        validate_opt_ascii_token("health_report_query_row.bcast_id", &self.bcast_id, 128)?;
        validate_opt_ascii_token(
            "health_report_query_row.issue_fingerprint",
            &self.issue_fingerprint,
            128,
        )?;
        if let (Some(start), Some(end)) = (
            self.verification_window_start_at,
            self.verification_window_end_at,
        ) {
            if end.0 < start.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.verification_window",
                    reason: "verification window end must be >= start",
                });
            }
        }
        let escalation_dispatched =
            self.status == HealthIssueStatus::Escalated || self.bcast_id.is_some();
        if escalation_dispatched {
            if self.bcast_id.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.bcast_id",
                    reason: "must be present when escalation is dispatched",
                });
            }
            let Some(summary) = &self.impact_summary else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.impact_summary",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text("health_report_query_row.impact_summary", summary, 512)?;
            let Some(evidence) = &self.current_monitoring_evidence else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.current_monitoring_evidence",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text(
                "health_report_query_row.current_monitoring_evidence",
                evidence,
                512,
            )?;
            let Some(reason) = &self.unresolved_reason_exact else {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.unresolved_reason_exact",
                    reason: "must be present when escalation is dispatched",
                });
            };
            validate_text(
                "health_report_query_row.unresolved_reason_exact",
                reason,
                512,
            )?;
            if self.attempted_fix_actions.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.attempted_fix_actions",
                    reason: "must include at least one attempted fix when escalation is dispatched",
                });
            }
        } else if let Some(summary) = &self.impact_summary {
            validate_text("health_report_query_row.impact_summary", summary, 512)?;
        }
        if self.recurrence_observed {
            if !self.status.unresolved() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.status",
                    reason: "must remain unresolved when recurrence_observed is true",
                });
            }
            if self.issue_fingerprint.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.issue_fingerprint",
                    reason: "must be present when recurrence_observed is true",
                });
            }
            if self.current_monitoring_evidence.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.current_monitoring_evidence",
                    reason: "must be present when recurrence_observed is true",
                });
            }
            if self.unresolved_reason_exact.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "health_report_query_row.unresolved_reason_exact",
                    reason: "must be present when recurrence_observed is true",
                });
            }
        }
        if self.attempted_fix_actions.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_row.attempted_fix_actions",
                reason: "must be <= 32 entries",
            });
        }
        for action in &self.attempted_fix_actions {
            validate_text(
                "health_report_query_row.attempted_fix_actions[]",
                action,
                160,
            )?;
        }
        if !escalation_dispatched {
            if let Some(evidence) = &self.current_monitoring_evidence {
                validate_text(
                    "health_report_query_row.current_monitoring_evidence",
                    evidence,
                    512,
                )?;
            }
            if let Some(reason) = &self.unresolved_reason_exact {
                validate_text(
                    "health_report_query_row.unresolved_reason_exact",
                    reason,
                    512,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReportQueryPaging {
    pub schema_version: SchemaVersion,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

impl HealthReportQueryPaging {
    pub fn v1(
        has_next: bool,
        has_prev: bool,
        next_cursor: Option<String>,
        prev_cursor: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let paging = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            has_next,
            has_prev,
            next_cursor,
            prev_cursor,
        };
        paging.validate()?;
        Ok(paging)
    }
}

impl Validate for HealthReportQueryPaging {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_paging.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        validate_opt_ascii_token(
            "health_report_query_paging.next_cursor",
            &self.next_cursor,
            128,
        )?;
        validate_opt_ascii_token(
            "health_report_query_paging.prev_cursor",
            &self.prev_cursor,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReportQueryReadOk {
    pub schema_version: SchemaVersion,
    pub capability_id: HealthCapabilityId,
    pub reason_code: ReasonCodeId,
    pub report_context_id: String,
    pub report_revision: u64,
    pub normalized_query: String,
    pub rows: Vec<HealthReportQueryRow>,
    pub paging: HealthReportQueryPaging,
    pub display_target_applied: Option<HealthDisplayTarget>,
    pub requires_clarification: Option<String>,
    pub no_authority_mutation: bool,
}

impl HealthReportQueryReadOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        report_context_id: String,
        report_revision: u64,
        normalized_query: String,
        rows: Vec<HealthReportQueryRow>,
        paging: HealthReportQueryPaging,
        display_target_applied: Option<HealthDisplayTarget>,
        requires_clarification: Option<String>,
        no_authority_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            capability_id: HealthCapabilityId::HealthReportQueryRead,
            reason_code,
            report_context_id,
            report_revision,
            normalized_query,
            rows,
            paging,
            display_target_applied,
            requires_clarification,
            no_authority_mutation,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for HealthReportQueryReadOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1HEALTH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_ok.schema_version",
                reason: "must match PH1HEALTH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != HealthCapabilityId::HealthReportQueryRead {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_ok.capability_id",
                reason: "must be HEALTH_REPORT_QUERY_READ",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_ok.reason_code",
                reason: "must be > 0",
            });
        }
        validate_ascii_token(
            "health_report_query_read_ok.report_context_id",
            &self.report_context_id,
            128,
        )?;
        validate_text(
            "health_report_query_read_ok.normalized_query",
            &self.normalized_query,
            512,
        )?;
        if self.rows.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_ok.rows",
                reason: "must be <= 1024",
            });
        }
        for row in &self.rows {
            row.validate()?;
        }
        self.paging.validate()?;
        if let Some(clarify) = &self.requires_clarification {
            validate_text(
                "health_report_query_read_ok.requires_clarification",
                clarify,
                240,
            )?;
        }
        if !self.no_authority_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "health_report_query_read_ok.no_authority_mutation",
                reason: "must remain true for display-only engine",
            });
        }
        Ok(())
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
    HealthReportQueryReadOk(HealthReportQueryReadOk),
    Refuse(HealthRefuse),
}

impl Validate for Ph1HealthResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1HealthResponse::HealthSnapshotReadOk(out) => out.validate(),
            Ph1HealthResponse::HealthIssueTimelineReadOk(out) => out.validate(),
            Ph1HealthResponse::HealthUnresolvedSummaryReadOk(out) => out.validate(),
            Ph1HealthResponse::HealthReportQueryReadOk(out) => out.validate(),
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
        let refuse = HealthRefuse::v1(
            HealthCapabilityId::HealthSnapshotRead,
            ReasonCodeId(1099),
            "input invalid".to_string(),
        )
        .unwrap();
        assert!(refuse.validate().is_ok());
    }

    #[test]
    fn at_health_contract_05_report_query_request_rejects_backwards_date_range() {
        let bad_range = HealthReportTimeRange::v1(MonotonicTimeNs(200), MonotonicTimeNs(100));
        assert!(bad_range.is_err());
    }

    #[test]
    fn at_health_contract_06_report_query_response_requires_matching_capability_id() {
        let row = HealthReportQueryRow {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            tenant_id: tenant("tenant_a"),
            issue_id: "issue_001".to_string(),
            owner_engine_id: "PH1.C".to_string(),
            severity: HealthSeverity::Warn,
            status: HealthIssueStatus::Open,
            latest_reason_code: ReasonCodeId(1001),
            last_seen_at: MonotonicTimeNs(100),
            bcast_id: None,
            ack_state: None,
            issue_fingerprint: Some("stt_missed_phrase".to_string()),
            verification_window_start_at: Some(MonotonicTimeNs(80)),
            verification_window_end_at: Some(MonotonicTimeNs(100)),
            recurrence_observed: true,
            impact_summary: Some("missed STT outputs in payroll queue".to_string()),
            attempted_fix_actions: vec!["restart stt route".to_string()],
            current_monitoring_evidence: Some("same fingerprint seen in last 5m".to_string()),
            unresolved_reason_exact: Some("fingerprint recurrence still present".to_string()),
        };
        let mut out = HealthReportQueryReadOk::v1(
            ReasonCodeId(1201),
            "ctx_001".to_string(),
            1,
            "missed stt june by tenant".to_string(),
            vec![row],
            HealthReportQueryPaging::v1(false, false, None, None).unwrap(),
            Some(HealthDisplayTarget::Desktop),
            None,
            true,
        )
        .unwrap();
        out.capability_id = HealthCapabilityId::HealthSnapshotRead;
        assert!(out.validate().is_err());
    }

    #[test]
    fn at_health_contract_07_issue_event_supports_escalation_payload_fields() {
        let mut event = HealthIssueEvent::v1(
            tenant("tenant_a"),
            "issue_901".to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Critical,
            HealthIssueStatus::Open,
            "ACTION_ESCALATE".to_string(),
            HealthActionResult::Retry,
            3,
            ReasonCodeId(1901),
            MonotonicTimeNs(100),
            None,
            Some(MonotonicTimeNs(120)),
            None,
            None,
        )
        .unwrap();
        event.status = HealthIssueStatus::Escalated;
        event.bcast_id = Some("bcast_901".to_string());
        event.ack_state = Some(HealthAckState::Waiting);
        let event = event
            .with_escalation_payload(
                Some("critical stt misses for payroll intake".to_string()),
                vec![
                    "restarted stt route".to_string(),
                    "rotated decoder".to_string(),
                ],
                Some("same fingerprint detected in live stream".to_string()),
                Some("recurrence still observed".to_string()),
            )
            .unwrap()
            .with_resolution_proof(
                Some("stt_missing_token_june".to_string()),
                Some(MonotonicTimeNs(100)),
                Some(MonotonicTimeNs(130)),
                Some(true),
            )
            .unwrap();
        assert!(event.validate().is_ok());
    }

    #[test]
    fn at_health_contract_08_recurrence_true_cannot_be_marked_resolved() {
        let mut event = HealthIssueEvent::v1(
            tenant("tenant_a"),
            "issue_902".to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Warn,
            HealthIssueStatus::Resolved,
            "ACTION_VERIFY".to_string(),
            HealthActionResult::Pass,
            2,
            ReasonCodeId(1902),
            MonotonicTimeNs(200),
            Some(MonotonicTimeNs(210)),
            None,
            None,
            Some(HealthAckState::Acknowledged),
        )
        .unwrap();
        event.issue_fingerprint = Some("stt_missing_phrase".to_string());
        event.verification_window_start_at = Some(MonotonicTimeNs(200));
        event.verification_window_end_at = Some(MonotonicTimeNs(260));
        event.recurrence_observed = Some(true);
        event.current_monitoring_evidence =
            Some("same fingerprint detected after deployment".to_string());
        event.unresolved_reason_exact =
            Some("recurrence observed in live verification window".to_string());
        assert!(event.validate().is_err());
    }

    #[test]
    fn at_health_contract_09_escalated_event_requires_minimum_payload_fields() {
        let mut missing_payload = HealthIssueEvent::v1(
            tenant("tenant_a"),
            "issue_903".to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Critical,
            HealthIssueStatus::Open,
            "ACTION_ESCALATE".to_string(),
            HealthActionResult::Retry,
            3,
            ReasonCodeId(1903),
            MonotonicTimeNs(300),
            None,
            Some(MonotonicTimeNs(360)),
            None,
            None,
        )
        .unwrap();
        missing_payload.status = HealthIssueStatus::Escalated;
        missing_payload.bcast_id = Some("bcast_903".to_string());
        missing_payload.ack_state = Some(HealthAckState::Waiting);
        assert!(missing_payload.validate().is_err());

        let complete_payload = missing_payload
            .with_escalation_payload(
                Some("critical customer impact".to_string()),
                vec!["restart route".to_string()],
                Some("fingerprint still detected in live stream".to_string()),
                Some("cannot prove fix in production yet".to_string()),
            )
            .unwrap();
        assert!(complete_payload.validate().is_ok());
    }

    #[test]
    fn at_health_contract_10_escalated_report_row_requires_minimum_payload_fields() {
        let row = HealthReportQueryRow {
            schema_version: PH1HEALTH_CONTRACT_VERSION,
            tenant_id: tenant("tenant_a"),
            issue_id: "issue_904".to_string(),
            owner_engine_id: "PH1.C".to_string(),
            severity: HealthSeverity::Critical,
            status: HealthIssueStatus::Escalated,
            latest_reason_code: ReasonCodeId(1904),
            last_seen_at: MonotonicTimeNs(400),
            bcast_id: Some("bcast_904".to_string()),
            ack_state: Some(HealthAckState::Waiting),
            issue_fingerprint: Some("sync_dead_letter_fingerprint".to_string()),
            verification_window_start_at: Some(MonotonicTimeNs(380)),
            verification_window_end_at: Some(MonotonicTimeNs(400)),
            recurrence_observed: true,
            impact_summary: None,
            attempted_fix_actions: Vec::new(),
            current_monitoring_evidence: None,
            unresolved_reason_exact: None,
        };
        assert!(row.validate().is_err());
    }
}
