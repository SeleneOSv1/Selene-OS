#![forbid(unsafe_code)]

use std::cmp::{min, Reverse};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

use selene_kernel_contracts::ph1health::{
    HealthCapabilityId, HealthCompanyScope, HealthDisplayTarget, HealthIssueEvent,
    HealthIssueSnapshotRow, HealthIssueStatus, HealthIssueTimelineEntry,
    HealthIssueTimelineMetadata, HealthIssueTimelineReadOk, HealthIssueTimelineReadRequest,
    HealthPageAction, HealthRefuse, HealthReportQueryPaging, HealthReportQueryReadOk,
    HealthReportQueryReadRequest, HealthReportQueryRow, HealthSeverity, HealthSnapshotReadOk,
    HealthSnapshotReadRequest, HealthUnresolvedSummaryReadOk, HealthUnresolvedSummaryReadRequest,
    HealthUnresolvedSummaryRow, Ph1HealthRequest, Ph1HealthResponse,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};

const WINDOW_24H_NS: u64 = 86_400_000_000_000;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.HEALTH reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_HEALTH_OK_SNAPSHOT_READ: ReasonCodeId = ReasonCodeId(0x4845_0001);
    pub const PH1_HEALTH_OK_ISSUE_TIMELINE_READ: ReasonCodeId = ReasonCodeId(0x4845_0002);
    pub const PH1_HEALTH_OK_UNRESOLVED_SUMMARY_READ: ReasonCodeId = ReasonCodeId(0x4845_0003);
    pub const PH1_HEALTH_OK_REPORT_QUERY_READ: ReasonCodeId = ReasonCodeId(0x4845_0004);

    pub const PH1_HEALTH_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F1);
    pub const PH1_HEALTH_TENANT_SCOPE_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F2);
    pub const PH1_HEALTH_ISSUE_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4845_00F3);
    pub const PH1_HEALTH_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4845_00F4);
    pub const PH1_HEALTH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4845_00F5);
    pub const PH1_HEALTH_DISPLAY_TARGET_REQUIRED: ReasonCodeId = ReasonCodeId(0x4845_00F6);
    pub const PH1_HEALTH_DATE_RANGE_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F7);
    pub const PH1_HEALTH_COUNTRY_FILTER_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F8);
    pub const PH1_HEALTH_CROSS_TENANT_UNAUTHORIZED: ReasonCodeId = ReasonCodeId(0x4845_00F9);
    pub const PH1_HEALTH_REPORT_CONTEXT_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4845_00FA);
    pub const PH1_HEALTH_PAGE_CURSOR_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00FB);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1HealthConfig {
    pub max_input_events: usize,
    pub max_snapshot_rows: u16,
    pub max_timeline_rows: u16,
    pub max_unresolved_rows: u16,
    pub max_report_rows: u16,
}

impl Ph1HealthConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_input_events: 4096,
            max_snapshot_rows: 128,
            max_timeline_rows: 256,
            max_unresolved_rows: 128,
            max_report_rows: 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1HealthRuntime {
    config: Ph1HealthConfig,
}

#[derive(Debug, Clone)]
struct IssueRollup {
    latest_event: HealthIssueEvent,
    first_unresolved_started_at: Option<MonotonicTimeNs>,
    latest_resolved_completed_at: Option<MonotonicTimeNs>,
    escalated_within_window: bool,
}

impl Ph1HealthRuntime {
    pub fn new(config: Ph1HealthConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1HealthRequest) -> Ph1HealthResponse {
        if req.validate().is_err() {
            return self.refuse(
                req.capability_id(),
                reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID,
                "health request failed contract validation",
            );
        }

        match req {
            Ph1HealthRequest::HealthSnapshotRead(r) => self.run_snapshot(r),
            Ph1HealthRequest::HealthIssueTimelineRead(r) => self.run_issue_timeline(r),
            Ph1HealthRequest::HealthUnresolvedSummaryRead(r) => self.run_unresolved_summary(r),
            Ph1HealthRequest::HealthReportQueryRead(r) => self.run_report_query(r),
        }
    }

    fn run_report_query(&self, req: &HealthReportQueryReadRequest) -> Ph1HealthResponse {
        if req.issue_events.len() > self.config.max_input_events {
            return self.refuse(
                HealthCapabilityId::HealthReportQueryRead,
                reason_codes::PH1_HEALTH_BUDGET_EXCEEDED,
                "health report query input event budget exceeded",
            );
        }

        if req.time_range.from_utc.0 > req.time_range.to_utc.0 {
            return self.refuse(
                HealthCapabilityId::HealthReportQueryRead,
                reason_codes::PH1_HEALTH_DATE_RANGE_INVALID,
                "health report query date range is invalid",
            );
        }

        if req
            .country_codes
            .iter()
            .any(|code| !code.chars().all(|c| c.is_ascii_uppercase()))
        {
            return self.refuse(
                HealthCapabilityId::HealthReportQueryRead,
                reason_codes::PH1_HEALTH_COUNTRY_FILTER_INVALID,
                "health report query country filter is invalid",
            );
        }

        let Some(display_target) = req.display_target else {
            let paging = match HealthReportQueryPaging::v1(false, false, None, None) {
                Ok(v) => v,
                Err(_) => {
                    return self.refuse(
                        HealthCapabilityId::HealthReportQueryRead,
                        reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                        "failed to construct health report query clarification paging",
                    )
                }
            };
            let out = HealthReportQueryReadOk::v1(
                reason_codes::PH1_HEALTH_DISPLAY_TARGET_REQUIRED,
                "ctx_missing_target".to_string(),
                req.envelope.as_of.0,
                "display_target_required".to_string(),
                Vec::new(),
                paging,
                None,
                Some("Where do you want this report displayed: desktop or phone?".to_string()),
                true,
            );
            return match out {
                Ok(v) => Ph1HealthResponse::HealthReportQueryReadOk(v),
                Err(_) => self.refuse(
                    HealthCapabilityId::HealthReportQueryRead,
                    reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                    "failed to construct health report query clarification response",
                ),
            };
        };

        let mut allowed_tenants: BTreeSet<String> = BTreeSet::new();
        match req.company_scope {
            HealthCompanyScope::TenantOnly => {
                allowed_tenants.insert(req.tenant_id.as_str().to_string());
            }
            HealthCompanyScope::CrossTenantTenantRows => {
                if req.company_ids.is_empty() {
                    return self.refuse(
                        HealthCapabilityId::HealthReportQueryRead,
                        reason_codes::PH1_HEALTH_CROSS_TENANT_UNAUTHORIZED,
                        "cross-tenant report query requires explicit tenant scope",
                    );
                }
                for tenant_id in &req.company_ids {
                    allowed_tenants.insert(tenant_id.as_str().to_string());
                }
            }
        }

        let normalized_query = normalize_query(req, display_target);
        let context_id = build_context_id(req, &normalized_query);
        if let Some(requested_context_id) = &req.report_context_id {
            if requested_context_id != &context_id {
                return self.refuse(
                    HealthCapabilityId::HealthReportQueryRead,
                    reason_codes::PH1_HEALTH_REPORT_CONTEXT_NOT_FOUND,
                    "health report context id was not found",
                );
            }
        }

        let mut rows_by_issue = BTreeMap::<(String, String), HealthIssueEvent>::new();
        for event in &req.issue_events {
            if !allowed_tenants.contains(event.tenant_id.as_str()) {
                continue;
            }
            if event.started_at.0 < req.time_range.from_utc.0
                || event.started_at.0 > req.time_range.to_utc.0
            {
                continue;
            }
            if let Some(owner) = &req.engine_owner_filter {
                if &event.owner_engine_id != owner {
                    continue;
                }
            }
            if req.escalated_only
                && !(event.status == HealthIssueStatus::Escalated || event.bcast_id.is_some())
            {
                continue;
            }
            if req.unresolved_only && !event.status.unresolved() {
                continue;
            }
            if !tenant_matches_country_filter(event.tenant_id.as_str(), &req.country_codes) {
                continue;
            }

            let key = (event.tenant_id.as_str().to_string(), event.issue_id.clone());
            let replace = rows_by_issue
                .get(&key)
                .map(|existing| {
                    (event.started_at.0, event.attempt_no, &event.action_id)
                        >= (
                            existing.started_at.0,
                            existing.attempt_no,
                            &existing.action_id,
                        )
                })
                .unwrap_or(true);
            if replace {
                rows_by_issue.insert(key, event.clone());
            }
        }

        let mut rows = Vec::<HealthReportQueryRow>::new();
        for ((_tenant, _issue), event) in rows_by_issue {
            let row = HealthReportQueryRow {
                schema_version: event.schema_version,
                tenant_id: event.tenant_id.clone(),
                issue_id: event.issue_id.clone(),
                owner_engine_id: event.owner_engine_id.clone(),
                severity: event.severity,
                status: event.status,
                latest_reason_code: event.reason_code,
                last_seen_at: event.started_at,
                bcast_id: event.bcast_id.clone(),
                ack_state: event.ack_state,
                issue_fingerprint: event.issue_fingerprint.clone(),
                verification_window_start_at: event.verification_window_start_at,
                verification_window_end_at: event.verification_window_end_at,
                recurrence_observed: event.recurrence_observed.unwrap_or(false),
                impact_summary: event.impact_summary.clone(),
                attempted_fix_actions: event.attempted_fix_actions.clone(),
                current_monitoring_evidence: event.current_monitoring_evidence.clone(),
                unresolved_reason_exact: event.unresolved_reason_exact.clone(),
            };
            if row.validate().is_err() {
                return self.refuse(
                    HealthCapabilityId::HealthReportQueryRead,
                    reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                    "failed to build health report query row",
                );
            }
            rows.push(row);
        }

        rows.sort_by_key(|row| {
            (
                row.tenant_id.as_str().to_string(),
                row.issue_id.clone(),
                Reverse(row.last_seen_at.0),
            )
        });

        let page_size = min(req.page_size, self.config.max_report_rows) as usize;
        let start_index =
            match read_page_start(req.page_action, req.page_cursor.as_deref(), page_size) {
                Ok(v) => v,
                Err(_) => {
                    return self.refuse(
                        HealthCapabilityId::HealthReportQueryRead,
                        reason_codes::PH1_HEALTH_PAGE_CURSOR_INVALID,
                        "health report query page cursor is invalid",
                    )
                }
            };
        let total = rows.len();
        let clamped_start = min(start_index, total);
        let end = min(clamped_start.saturating_add(page_size), total);
        let paged_rows = rows[clamped_start..end].to_vec();
        let has_prev = clamped_start > 0;
        let has_next = end < total;
        let next_cursor = if has_next {
            Some(format!("idx:{end}"))
        } else {
            None
        };
        let prev_cursor = if has_prev {
            Some(format!("idx:{}", clamped_start.saturating_sub(page_size)))
        } else {
            None
        };
        let paging = match HealthReportQueryPaging::v1(has_next, has_prev, next_cursor, prev_cursor)
        {
            Ok(v) => v,
            Err(_) => {
                return self.refuse(
                    HealthCapabilityId::HealthReportQueryRead,
                    reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                    "failed to construct health report query paging",
                )
            }
        };
        let out = HealthReportQueryReadOk::v1(
            reason_codes::PH1_HEALTH_OK_REPORT_QUERY_READ,
            context_id,
            req.envelope.as_of.0,
            normalized_query,
            paged_rows,
            paging,
            Some(display_target),
            None,
            true,
        );
        match out {
            Ok(v) => Ph1HealthResponse::HealthReportQueryReadOk(v),
            Err(_) => self.refuse(
                HealthCapabilityId::HealthReportQueryRead,
                reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                "failed to construct health report query response",
            ),
        }
    }

    fn run_snapshot(&self, req: &HealthSnapshotReadRequest) -> Ph1HealthResponse {
        if req.issue_events.len() > self.config.max_input_events {
            return self.refuse(
                HealthCapabilityId::HealthSnapshotRead,
                reason_codes::PH1_HEALTH_BUDGET_EXCEEDED,
                "health snapshot input event budget exceeded",
            );
        }
        if !tenant_scope_valid(&req.tenant_id, &req.issue_events) {
            return self.refuse(
                HealthCapabilityId::HealthSnapshotRead,
                reason_codes::PH1_HEALTH_TENANT_SCOPE_INVALID,
                "health snapshot tenant scope mismatch",
            );
        }

        let rollups = rollups_by_issue(&req.issue_events, req.envelope.as_of);

        let mut rows = Vec::new();
        let mut open_issue_count: u32 = 0;
        let mut critical_open_count: u32 = 0;
        let mut resolved_24h_count: u32 = 0;
        let mut escalated_24h_count: u32 = 0;
        let mut mttr_total_ns: u128 = 0;
        let mut mttr_samples: u32 = 0;

        for rollup in rollups.values() {
            let latest = &rollup.latest_event;
            let escalated_state =
                latest.status == HealthIssueStatus::Escalated || latest.bcast_id.is_some();

            if req.open_only && !latest.status.unresolved() {
                continue;
            }
            if req.escalated_only && !escalated_state {
                continue;
            }
            if let Some(severity_filter) = req.severity_filter {
                if latest.severity != severity_filter {
                    continue;
                }
            }
            if let Some(engine_owner_filter) = &req.engine_owner_filter {
                if latest.owner_engine_id != *engine_owner_filter {
                    continue;
                }
            }

            if latest.status.unresolved() {
                open_issue_count = open_issue_count.saturating_add(1);
                if latest.severity == HealthSeverity::Critical {
                    critical_open_count = critical_open_count.saturating_add(1);
                }
            }

            if latest.status == HealthIssueStatus::Resolved {
                if let Some(completed_at) = latest.completed_at {
                    if within_24h(req.envelope.as_of, completed_at) {
                        resolved_24h_count = resolved_24h_count.saturating_add(1);
                    }
                }
                if let (Some(first_unresolved_started_at), Some(latest_resolved_completed_at)) = (
                    rollup.first_unresolved_started_at,
                    rollup.latest_resolved_completed_at,
                ) {
                    if latest_resolved_completed_at.0 >= first_unresolved_started_at.0 {
                        mttr_total_ns = mttr_total_ns.saturating_add(
                            (latest_resolved_completed_at.0 - first_unresolved_started_at.0)
                                as u128,
                        );
                        mttr_samples = mttr_samples.saturating_add(1);
                    }
                }
            }

            if rollup.escalated_within_window {
                escalated_24h_count = escalated_24h_count.saturating_add(1);
            }

            match HealthIssueSnapshotRow::v1(
                latest.issue_id.clone(),
                latest.owner_engine_id.clone(),
                latest.severity,
                latest.status,
                latest.reason_code,
                latest.started_at,
                latest.unresolved_deadline_at,
                latest.bcast_id.clone(),
                latest.ack_state,
            ) {
                Ok(row) => rows.push(row),
                Err(_) => {
                    return self.refuse(
                        HealthCapabilityId::HealthSnapshotRead,
                        reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                        "failed to build health snapshot row",
                    )
                }
            }
        }

        rows.sort_by_key(|row| (Reverse(row.latest_started_at.0), row.issue_id.clone()));
        rows.truncate(min(
            req.max_issue_rows as usize,
            self.config.max_snapshot_rows as usize,
        ));

        let mttr_minutes = if mttr_samples == 0 {
            0
        } else {
            let avg_ns = mttr_total_ns / mttr_samples as u128;
            (avg_ns / 60_000_000_000) as u32
        };

        match HealthSnapshotReadOk::v1(
            reason_codes::PH1_HEALTH_OK_SNAPSHOT_READ,
            open_issue_count,
            critical_open_count,
            resolved_24h_count,
            escalated_24h_count,
            mttr_minutes,
            rows,
            true,
        ) {
            Ok(out) => Ph1HealthResponse::HealthSnapshotReadOk(out),
            Err(_) => self.refuse(
                HealthCapabilityId::HealthSnapshotRead,
                reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                "failed to construct health snapshot response",
            ),
        }
    }

    fn run_issue_timeline(&self, req: &HealthIssueTimelineReadRequest) -> Ph1HealthResponse {
        if req.issue_events.len() > self.config.max_input_events {
            return self.refuse(
                HealthCapabilityId::HealthIssueTimelineRead,
                reason_codes::PH1_HEALTH_BUDGET_EXCEEDED,
                "health timeline input event budget exceeded",
            );
        }
        if !tenant_scope_valid(&req.tenant_id, &req.issue_events) {
            return self.refuse(
                HealthCapabilityId::HealthIssueTimelineRead,
                reason_codes::PH1_HEALTH_TENANT_SCOPE_INVALID,
                "health timeline tenant scope mismatch",
            );
        }

        let mut events = req
            .issue_events
            .iter()
            .filter(|event| event.issue_id == req.issue_id)
            .cloned()
            .collect::<Vec<_>>();

        if events.is_empty() {
            return self.refuse(
                HealthCapabilityId::HealthIssueTimelineRead,
                reason_codes::PH1_HEALTH_ISSUE_NOT_FOUND,
                "health issue timeline target was not found",
            );
        }

        events.sort_by_key(|event| {
            (
                event.started_at.0,
                event.attempt_no,
                event.action_id.clone(),
            )
        });

        let latest = events
            .last()
            .cloned()
            .expect("events is non-empty after guard");

        let metadata = match HealthIssueTimelineMetadata::v1(
            latest.issue_id.clone(),
            latest.owner_engine_id.clone(),
            latest.severity,
            latest.status,
            latest.reason_code,
            latest.unresolved_deadline_at,
            latest.bcast_id.clone(),
            latest.ack_state,
        ) {
            Ok(m) => m,
            Err(_) => {
                return self.refuse(
                    HealthCapabilityId::HealthIssueTimelineRead,
                    reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                    "failed to build health issue timeline metadata",
                )
            }
        };

        let mut timeline_entries = Vec::new();
        for event in events.into_iter().take(min(
            req.max_timeline_rows as usize,
            self.config.max_timeline_rows as usize,
        )) {
            match HealthIssueTimelineEntry::v1(
                event.attempt_no,
                event.action_id,
                event.action_result,
                event.reason_code,
                event.started_at,
                event.completed_at,
            ) {
                Ok(entry) => timeline_entries.push(entry),
                Err(_) => {
                    return self.refuse(
                        HealthCapabilityId::HealthIssueTimelineRead,
                        reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                        "failed to build health issue timeline entry",
                    )
                }
            }
        }

        match HealthIssueTimelineReadOk::v1(
            reason_codes::PH1_HEALTH_OK_ISSUE_TIMELINE_READ,
            metadata,
            timeline_entries,
            true,
        ) {
            Ok(out) => Ph1HealthResponse::HealthIssueTimelineReadOk(out),
            Err(_) => self.refuse(
                HealthCapabilityId::HealthIssueTimelineRead,
                reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                "failed to construct health timeline response",
            ),
        }
    }

    fn run_unresolved_summary(
        &self,
        req: &HealthUnresolvedSummaryReadRequest,
    ) -> Ph1HealthResponse {
        if req.issue_events.len() > self.config.max_input_events {
            return self.refuse(
                HealthCapabilityId::HealthUnresolvedSummaryRead,
                reason_codes::PH1_HEALTH_BUDGET_EXCEEDED,
                "health unresolved summary input event budget exceeded",
            );
        }
        if !tenant_scope_valid(&req.tenant_id, &req.issue_events) {
            return self.refuse(
                HealthCapabilityId::HealthUnresolvedSummaryRead,
                reason_codes::PH1_HEALTH_TENANT_SCOPE_INVALID,
                "health unresolved summary tenant scope mismatch",
            );
        }

        let rollups = rollups_by_issue(&req.issue_events, req.envelope.as_of);

        let mut rows = Vec::new();
        let mut unresolved_issue_count: u32 = 0;
        let mut sla_breach_issue_count: u32 = 0;
        let mut escalated_issue_count: u32 = 0;

        for rollup in rollups.values() {
            let latest = &rollup.latest_event;
            if !latest.status.unresolved() {
                continue;
            }

            let sla_breached = latest
                .unresolved_deadline_at
                .map(|deadline| deadline.0 <= req.envelope.as_of.0)
                .unwrap_or(false);
            let escalated =
                latest.status == HealthIssueStatus::Escalated || latest.bcast_id.is_some();

            if req.breach_only && !sla_breached {
                continue;
            }

            unresolved_issue_count = unresolved_issue_count.saturating_add(1);
            if sla_breached {
                sla_breach_issue_count = sla_breach_issue_count.saturating_add(1);
            }
            if escalated {
                escalated_issue_count = escalated_issue_count.saturating_add(1);
            }

            match HealthUnresolvedSummaryRow::v1(
                latest.issue_id.clone(),
                latest.owner_engine_id.clone(),
                latest.severity,
                latest.reason_code,
                sla_breached,
                escalated,
                latest.unresolved_deadline_at,
                latest.bcast_id.clone(),
                latest.ack_state,
            ) {
                Ok(row) => rows.push(row),
                Err(_) => {
                    return self.refuse(
                        HealthCapabilityId::HealthUnresolvedSummaryRead,
                        reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                        "failed to build health unresolved summary row",
                    )
                }
            }
        }

        rows.sort_by_key(|row| {
            (
                Reverse(row.sla_breached),
                row.unresolved_deadline_at.map(|v| v.0).unwrap_or(u64::MAX),
                row.issue_id.clone(),
            )
        });
        rows.truncate(min(
            req.max_issue_rows as usize,
            self.config.max_unresolved_rows as usize,
        ));

        match HealthUnresolvedSummaryReadOk::v1(
            reason_codes::PH1_HEALTH_OK_UNRESOLVED_SUMMARY_READ,
            unresolved_issue_count,
            sla_breach_issue_count,
            escalated_issue_count,
            rows,
            true,
        ) {
            Ok(out) => Ph1HealthResponse::HealthUnresolvedSummaryReadOk(out),
            Err(_) => self.refuse(
                HealthCapabilityId::HealthUnresolvedSummaryRead,
                reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                "failed to construct health unresolved summary response",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: HealthCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1HealthResponse {
        let refuse = HealthRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("HealthRefuse::v1 must construct for static message");
        Ph1HealthResponse::Refuse(refuse)
    }
}

fn tenant_scope_valid(
    tenant_id: &selene_kernel_contracts::ph1position::TenantId,
    events: &[HealthIssueEvent],
) -> bool {
    events
        .iter()
        .all(|event| event.tenant_id.as_str() == tenant_id.as_str())
}

fn within_24h(as_of: MonotonicTimeNs, ts: MonotonicTimeNs) -> bool {
    if as_of.0 < ts.0 {
        return false;
    }
    as_of.0 - ts.0 <= WINDOW_24H_NS
}

fn tenant_matches_country_filter(tenant_id: &str, country_codes: &[String]) -> bool {
    if country_codes.is_empty() {
        return true;
    }
    let mut tenant_country = String::new();
    for sep in ['_', '-'] {
        if let Some((_, suffix)) = tenant_id.rsplit_once(sep) {
            tenant_country = suffix.to_ascii_uppercase();
            break;
        }
    }
    if tenant_country.is_empty() {
        return false;
    }
    country_codes.iter().any(|code| code == &tenant_country)
}

fn normalize_query(
    req: &HealthReportQueryReadRequest,
    display_target: HealthDisplayTarget,
) -> String {
    let owner = req.engine_owner_filter.as_deref().unwrap_or("ALL_ENGINES");
    let company_scope = match req.company_scope {
        HealthCompanyScope::TenantOnly => "TENANT_ONLY",
        HealthCompanyScope::CrossTenantTenantRows => "CROSS_TENANT_TENANT_ROWS",
    };
    let display = match display_target {
        HealthDisplayTarget::Desktop => "DESKTOP",
        HealthDisplayTarget::Phone => "PHONE",
    };
    format!(
        "kind={:?};from={};to={};owner={};scope={};escalated_only={};unresolved_only={};target={display}",
        req.report_kind,
        req.time_range.from_utc.0,
        req.time_range.to_utc.0,
        owner,
        company_scope,
        req.escalated_only,
        req.unresolved_only,
    )
}

fn build_context_id(req: &HealthReportQueryReadRequest, normalized_query: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    req.tenant_id.as_str().hash(&mut hasher);
    req.viewer_user_id.hash(&mut hasher);
    normalized_query.hash(&mut hasher);
    format!("ctx_{:016x}", hasher.finish())
}

fn read_page_start(
    action: HealthPageAction,
    cursor: Option<&str>,
    page_size: usize,
) -> Result<usize, ()> {
    match action {
        HealthPageAction::First => Ok(0),
        HealthPageAction::Refresh => parse_cursor(cursor).or(Ok(0)),
        HealthPageAction::Next => parse_cursor(cursor),
        HealthPageAction::Prev => {
            let idx = parse_cursor(cursor)?;
            Ok(idx.saturating_sub(page_size))
        }
    }
}

fn parse_cursor(cursor: Option<&str>) -> Result<usize, ()> {
    let cursor = cursor.ok_or(())?;
    let (_, value) = cursor.split_once(':').ok_or(())?;
    value.parse::<usize>().map_err(|_| ())
}

fn rollups_by_issue(
    events: &[HealthIssueEvent],
    as_of: MonotonicTimeNs,
) -> BTreeMap<String, IssueRollup> {
    let mut ordered = events.to_vec();
    ordered.sort_by_key(|event| {
        (
            event.started_at.0,
            event.attempt_no,
            event.action_id.clone(),
        )
    });

    let mut out = BTreeMap::<String, IssueRollup>::new();
    for event in ordered {
        let entry = out
            .entry(event.issue_id.clone())
            .or_insert_with(|| IssueRollup {
                latest_event: event.clone(),
                first_unresolved_started_at: None,
                latest_resolved_completed_at: None,
                escalated_within_window: false,
            });

        if event.status.unresolved() && entry.first_unresolved_started_at.is_none() {
            entry.first_unresolved_started_at = Some(event.started_at);
        }
        if event.status == HealthIssueStatus::Resolved {
            if let Some(completed_at) = event.completed_at {
                let should_set_latest = entry
                    .latest_resolved_completed_at
                    .map(|existing| completed_at.0 >= existing.0)
                    .unwrap_or(true);
                if should_set_latest {
                    entry.latest_resolved_completed_at = Some(completed_at);
                }
            }
        }
        if event.status == HealthIssueStatus::Escalated && within_24h(as_of, event.started_at) {
            entry.escalated_within_window = true;
        }

        entry.latest_event = event;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1health::{
        HealthAckState, HealthActionResult, HealthCompanyScope, HealthDisplayTarget,
        HealthPageAction, HealthReadEnvelope, HealthReportKind, HealthReportTimeRange,
        HealthSeverity, Ph1HealthRequest,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;

    fn tenant(id: &str) -> TenantId {
        TenantId::new(id.to_string()).unwrap()
    }

    fn envelope(as_of: u64) -> HealthReadEnvelope {
        HealthReadEnvelope::v1(CorrelationId(1001), TurnId(51), MonotonicTimeNs(as_of)).unwrap()
    }

    fn event(
        tenant_id: &str,
        issue_id: &str,
        owner_engine_id: &str,
        severity: HealthSeverity,
        status: HealthIssueStatus,
        action_id: &str,
        action_result: HealthActionResult,
        attempt_no: u16,
        reason_code: u32,
        started_at: u64,
        completed_at: Option<u64>,
        unresolved_deadline_at: Option<u64>,
        bcast_id: Option<&str>,
        ack_state: Option<HealthAckState>,
    ) -> HealthIssueEvent {
        let seed_status = if status == HealthIssueStatus::Escalated {
            HealthIssueStatus::Open
        } else {
            status
        };
        let mut base = HealthIssueEvent::v1(
            tenant(tenant_id),
            issue_id.to_string(),
            owner_engine_id.to_string(),
            severity,
            seed_status,
            action_id.to_string(),
            action_result,
            attempt_no,
            ReasonCodeId(reason_code),
            MonotonicTimeNs(started_at),
            completed_at.map(MonotonicTimeNs),
            unresolved_deadline_at.map(MonotonicTimeNs),
            None,
            None,
        )
        .unwrap();
        base.status = status;
        base.bcast_id = bcast_id.map(|v| v.to_string());
        base.ack_state = ack_state;
        if status == HealthIssueStatus::Escalated || bcast_id.is_some() {
            base.with_escalation_payload(
                Some(format!("{issue_id} impact summary")),
                vec![format!("{issue_id} attempted fix")],
                Some(format!("{issue_id} monitoring evidence")),
                Some(format!("{issue_id} unresolved reason")),
            )
            .unwrap()
        } else {
            base
        }
    }

    fn sample_events() -> Vec<HealthIssueEvent> {
        vec![
            event(
                "tenant_a",
                "issue_a",
                "PH1.C",
                HealthSeverity::Warn,
                HealthIssueStatus::Open,
                "A1",
                HealthActionResult::Fail,
                1,
                2001,
                10,
                None,
                Some(120),
                None,
                None,
            ),
            event(
                "tenant_a",
                "issue_a",
                "PH1.C",
                HealthSeverity::Warn,
                HealthIssueStatus::Escalated,
                "A2",
                HealthActionResult::Retry,
                2,
                2002,
                20,
                None,
                Some(120),
                Some("bcast_001"),
                Some(HealthAckState::Waiting),
            ),
            event(
                "tenant_a",
                "issue_b",
                "PH1.DELIVERY",
                HealthSeverity::Info,
                HealthIssueStatus::Open,
                "B1",
                HealthActionResult::Fail,
                1,
                2101,
                30,
                None,
                Some(200),
                None,
                None,
            ),
            event(
                "tenant_a",
                "issue_b",
                "PH1.DELIVERY",
                HealthSeverity::Info,
                HealthIssueStatus::Resolved,
                "B2",
                HealthActionResult::Pass,
                2,
                2102,
                70,
                Some(80),
                None,
                None,
                Some(HealthAckState::Acknowledged),
            ),
            event(
                "tenant_a",
                "issue_c",
                "PH1.VOICE.ID",
                HealthSeverity::Critical,
                HealthIssueStatus::Open,
                "C1",
                HealthActionResult::Fail,
                1,
                2201,
                90,
                None,
                Some(95),
                None,
                None,
            ),
        ]
    }

    #[test]
    fn at_health_01_snapshot_read_is_schema_valid_and_display_only() {
        let req = Ph1HealthRequest::HealthSnapshotRead(
            HealthSnapshotReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                None,
                None,
                false,
                100,
                sample_events(),
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        assert!(out.validate().is_ok());

        match out {
            Ph1HealthResponse::HealthSnapshotReadOk(ok) => {
                assert!(ok.no_authority_mutation);
                assert!(ok.issue_rows.len() >= 2);
            }
            _ => panic!("expected HealthSnapshotReadOk"),
        }
    }

    #[test]
    fn at_health_02_unresolved_summary_includes_owner_and_reason_code() {
        let req = Ph1HealthRequest::HealthUnresolvedSummaryRead(
            HealthUnresolvedSummaryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                100,
                sample_events(),
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        match out {
            Ph1HealthResponse::HealthUnresolvedSummaryReadOk(ok) => {
                assert!(ok.unresolved_issue_count >= 1);
                assert!(ok
                    .issue_rows
                    .iter()
                    .any(|row| row.owner_engine_id == "PH1.C"
                        || row.owner_engine_id == "PH1.VOICE.ID"));
                assert!(ok.issue_rows.iter().all(|row| row.latest_reason_code.0 > 0));
            }
            _ => panic!("expected HealthUnresolvedSummaryReadOk"),
        }
    }

    #[test]
    fn at_health_03_issue_timeline_exposes_bcast_reference_when_escalated() {
        let req = Ph1HealthRequest::HealthIssueTimelineRead(
            HealthIssueTimelineReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                "issue_a".to_string(),
                100,
                sample_events(),
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        match out {
            Ph1HealthResponse::HealthIssueTimelineReadOk(ok) => {
                assert_eq!(ok.issue_metadata.issue_id, "issue_a");
                assert_eq!(ok.issue_metadata.bcast_id.as_deref(), Some("bcast_001"));
                assert!(!ok.timeline_entries.is_empty());
            }
            _ => panic!("expected HealthIssueTimelineReadOk"),
        }
    }

    #[test]
    fn at_health_04_refuses_tenant_scope_mismatch() {
        let mut events = sample_events();
        events.push(event(
            "tenant_b",
            "issue_z",
            "PH1.C",
            HealthSeverity::Warn,
            HealthIssueStatus::Open,
            "Z1",
            HealthActionResult::Fail,
            1,
            2999,
            30,
            None,
            None,
            None,
            None,
        ));

        let req = Ph1HealthRequest::HealthSnapshotRead(
            HealthSnapshotReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                None,
                None,
                false,
                100,
                events,
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        match out {
            Ph1HealthResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_HEALTH_TENANT_SCOPE_INVALID
                );
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_health_05_report_query_missing_display_target_returns_clarify() {
        let req = Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::UnresolvedEscalated,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                Some("PH1.C".to_string()),
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                true,
                true,
                None,
                HealthPageAction::First,
                None,
                None,
                25,
                sample_events(),
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        match out {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => {
                assert_eq!(
                    ok.reason_code,
                    reason_codes::PH1_HEALTH_DISPLAY_TARGET_REQUIRED
                );
                assert!(ok.display_target_applied.is_none());
                assert!(ok.requires_clarification.is_some());
            }
            _ => panic!("expected HealthReportQueryReadOk clarification"),
        }
    }

    #[test]
    fn at_health_06_report_query_paging_is_deterministic() {
        let req_first = Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::UnresolvedEscalated,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                None,
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                false,
                false,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::First,
                None,
                None,
                1,
                sample_events(),
            )
            .unwrap(),
        );

        let runtime = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1());
        let out_first = runtime.run(&req_first);
        let next_cursor = match out_first {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => ok.paging.next_cursor,
            _ => panic!("expected report query ok"),
        };
        let req_next = Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::UnresolvedEscalated,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                None,
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                false,
                false,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::Next,
                next_cursor,
                None,
                1,
                sample_events(),
            )
            .unwrap(),
        );
        let out_next = runtime.run(&req_next);
        match out_next {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => {
                assert_eq!(
                    ok.display_target_applied,
                    Some(HealthDisplayTarget::Desktop)
                );
                assert!(ok.rows.len() <= 1);
            }
            _ => panic!("expected report query next page ok"),
        }
    }

    #[test]
    fn at_health_07_report_query_cross_tenant_requires_scope() {
        let req = Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::IssueStatus,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                None,
                HealthCompanyScope::CrossTenantTenantRows,
                vec![],
                vec![],
                false,
                false,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::First,
                None,
                None,
                25,
                sample_events(),
            )
            .unwrap(),
        );

        let out = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1()).run(&req);
        match out {
            Ph1HealthResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_HEALTH_CROSS_TENANT_UNAUTHORIZED
                );
            }
            _ => panic!("expected cross-tenant unauthorized refuse"),
        }
    }

    #[test]
    fn at_health_08_report_query_context_survives_follow_up_turn_time() {
        let runtime = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1());
        let first = runtime.run(&Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(100),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::UnresolvedEscalated,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                None,
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                false,
                false,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::First,
                None,
                None,
                25,
                sample_events(),
            )
            .unwrap(),
        ));
        let context_id = match first {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => ok.report_context_id,
            _ => panic!("expected first report query read ok"),
        };

        let follow_up = runtime.run(&Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(120),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::UnresolvedEscalated,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(100)).unwrap(),
                None,
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                false,
                false,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::Refresh,
                None,
                Some(context_id.clone()),
                25,
                sample_events(),
            )
            .unwrap(),
        ));
        match follow_up {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => {
                assert_eq!(ok.report_context_id, context_id);
            }
            _ => panic!("expected follow-up report query read ok"),
        }
    }

    #[test]
    fn at_health_08b_report_query_surfaces_ph1x_interrupt_outcome_by_engine_and_topic() {
        let runtime = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1());
        let mut interrupt_event = event(
            "tenant_a",
            "issue_interrupt_project_status",
            "PH1.X",
            HealthSeverity::Warn,
            HealthIssueStatus::Open,
            "X1",
            HealthActionResult::Retry,
            1,
            0x5800_001D,
            180,
            None,
            Some(260),
            None,
            None,
        );
        interrupt_event.issue_fingerprint = Some("topic:project_status".to_string());

        let req = Ph1HealthRequest::HealthReportQueryRead(
            HealthReportQueryReadRequest::v1(
                envelope(200),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                HealthReportKind::IssueStatus,
                HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(220)).unwrap(),
                Some("PH1.X".to_string()),
                HealthCompanyScope::TenantOnly,
                vec![],
                vec![],
                false,
                true,
                Some(HealthDisplayTarget::Desktop),
                HealthPageAction::First,
                None,
                None,
                25,
                vec![interrupt_event],
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1HealthResponse::HealthReportQueryReadOk(ok) => {
                assert_eq!(ok.rows.len(), 1);
                let row = &ok.rows[0];
                assert_eq!(row.owner_engine_id, "PH1.X");
                assert_eq!(row.latest_reason_code, ReasonCodeId(0x5800_001D));
                assert_eq!(
                    row.issue_fingerprint.as_deref(),
                    Some("topic:project_status")
                );
            }
            _ => panic!("expected report query row for PH1.X interruption outcome"),
        }
    }

    #[test]
    fn at_health_09_recurrence_true_post_fix_keeps_issue_unresolved() {
        let runtime = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1());
        let mut req = HealthReportQueryReadRequest::v1(
            envelope(140),
            tenant("tenant_a"),
            "viewer_01".to_string(),
            HealthReportKind::UnresolvedEscalated,
            HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(200)).unwrap(),
            None,
            HealthCompanyScope::TenantOnly,
            vec![],
            vec![],
            false,
            false,
            Some(HealthDisplayTarget::Desktop),
            HealthPageAction::First,
            None,
            None,
            25,
            sample_events(),
        )
        .unwrap();
        let mut invalid_event = event(
            "tenant_a",
            "issue_recurring",
            "PH1.STT",
            HealthSeverity::Warn,
            HealthIssueStatus::Resolved,
            "R1",
            HealthActionResult::Pass,
            1,
            2601,
            150,
            Some(160),
            None,
            None,
            None,
        );
        invalid_event.issue_fingerprint = Some("stt_recurring_fingerprint".to_string());
        invalid_event.verification_window_start_at = Some(MonotonicTimeNs(140));
        invalid_event.verification_window_end_at = Some(MonotonicTimeNs(150));
        invalid_event.recurrence_observed = Some(true);
        invalid_event.current_monitoring_evidence =
            Some("same fingerprint still emitted post-fix".to_string());
        invalid_event.unresolved_reason_exact =
            Some("recurrence still present after deployment".to_string());
        req.issue_events = vec![invalid_event];

        let out = runtime.run(&Ph1HealthRequest::HealthReportQueryRead(req));
        match out {
            Ph1HealthResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID
                );
            }
            _ => panic!("expected fail-closed refuse for recurring resolved issue"),
        }
    }

    #[test]
    fn at_health_10_escalated_issue_requires_minimum_payload() {
        let runtime = Ph1HealthRuntime::new(Ph1HealthConfig::mvp_v1());
        let mut req = HealthReportQueryReadRequest::v1(
            envelope(150),
            tenant("tenant_a"),
            "viewer_01".to_string(),
            HealthReportKind::UnresolvedEscalated,
            HealthReportTimeRange::v1(MonotonicTimeNs(1), MonotonicTimeNs(200)).unwrap(),
            None,
            HealthCompanyScope::TenantOnly,
            vec![],
            vec![],
            true,
            true,
            Some(HealthDisplayTarget::Desktop),
            HealthPageAction::First,
            None,
            None,
            25,
            sample_events(),
        )
        .unwrap();
        let mut invalid_event = event(
            "tenant_a",
            "issue_missing_payload",
            "PH1.STT",
            HealthSeverity::Critical,
            HealthIssueStatus::Open,
            "E1",
            HealthActionResult::Retry,
            2,
            2602,
            170,
            None,
            Some(200),
            None,
            None,
        );
        invalid_event.status = HealthIssueStatus::Escalated;
        invalid_event.bcast_id = Some("bcast_missing_payload".to_string());
        invalid_event.ack_state = Some(HealthAckState::Waiting);
        invalid_event.impact_summary = None;
        invalid_event.attempted_fix_actions = Vec::new();
        invalid_event.current_monitoring_evidence = None;
        invalid_event.unresolved_reason_exact = None;
        req.issue_events = vec![invalid_event];

        let out = runtime.run(&Ph1HealthRequest::HealthReportQueryRead(req));
        match out {
            Ph1HealthResponse::Refuse(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID
                );
            }
            _ => panic!("expected fail-closed refuse for incomplete escalation payload"),
        }
    }
}
