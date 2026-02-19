#![forbid(unsafe_code)]

use std::cmp::{min, Reverse};
use std::collections::BTreeMap;

use selene_kernel_contracts::ph1health::{
    HealthCapabilityId, HealthIssueEvent, HealthIssueSnapshotRow, HealthIssueStatus,
    HealthIssueTimelineEntry, HealthIssueTimelineMetadata, HealthIssueTimelineReadOk,
    HealthIssueTimelineReadRequest, HealthRefuse, HealthSeverity, HealthSnapshotReadOk,
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

    pub const PH1_HEALTH_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F1);
    pub const PH1_HEALTH_TENANT_SCOPE_INVALID: ReasonCodeId = ReasonCodeId(0x4845_00F2);
    pub const PH1_HEALTH_ISSUE_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4845_00F3);
    pub const PH1_HEALTH_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4845_00F4);
    pub const PH1_HEALTH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4845_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1HealthConfig {
    pub max_input_events: usize,
    pub max_snapshot_rows: u16,
    pub max_timeline_rows: u16,
    pub max_unresolved_rows: u16,
}

impl Ph1HealthConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_input_events: 4096,
            max_snapshot_rows: 128,
            max_timeline_rows: 256,
            max_unresolved_rows: 128,
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
            let escalated_state = latest.status == HealthIssueStatus::Escalated || latest.bcast_id.is_some();

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

        events.sort_by_key(|event| (event.started_at.0, event.attempt_no, event.action_id.clone()));

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
        for event in events
            .into_iter()
            .take(min(req.max_timeline_rows as usize, self.config.max_timeline_rows as usize))
        {
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

    fn run_unresolved_summary(&self, req: &HealthUnresolvedSummaryReadRequest) -> Ph1HealthResponse {
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
            let escalated = latest.status == HealthIssueStatus::Escalated || latest.bcast_id.is_some();

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

fn rollups_by_issue(
    events: &[HealthIssueEvent],
    as_of: MonotonicTimeNs,
) -> BTreeMap<String, IssueRollup> {
    let mut ordered = events.to_vec();
    ordered.sort_by_key(|event| (event.started_at.0, event.attempt_no, event.action_id.clone()));

    let mut out = BTreeMap::<String, IssueRollup>::new();
    for event in ordered {
        let entry = out.entry(event.issue_id.clone()).or_insert_with(|| IssueRollup {
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
        HealthAckState, HealthActionResult, HealthReadEnvelope, HealthSeverity, Ph1HealthRequest,
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
        HealthIssueEvent::v1(
            tenant(tenant_id),
            issue_id.to_string(),
            owner_engine_id.to_string(),
            severity,
            status,
            action_id.to_string(),
            action_result,
            attempt_no,
            ReasonCodeId(reason_code),
            MonotonicTimeNs(started_at),
            completed_at.map(MonotonicTimeNs),
            unresolved_deadline_at.map(MonotonicTimeNs),
            bcast_id.map(|v| v.to_string()),
            ack_state,
        )
        .unwrap()
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
                    .any(|row| row.owner_engine_id == "PH1.C" || row.owner_engine_id == "PH1.VOICE.ID"));
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
}
