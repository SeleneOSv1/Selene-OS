#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1health::{
    HealthIssueTimelineReadRequest, HealthRefuse, HealthReportQueryReadRequest,
    HealthSnapshotReadRequest, HealthUnresolvedSummaryReadRequest, Ph1HealthRequest,
    Ph1HealthResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.HEALTH OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_HEALTH_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4845_0101);
    pub const PH1_HEALTH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4845_01F1);
    pub const PH1_HEALTH_RESPONSE_CAPABILITY_MISMATCH: ReasonCodeId = ReasonCodeId(0x4845_01F2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1HealthWiringConfig {
    pub health_enabled: bool,
    pub max_issue_rows: u16,
    pub max_timeline_rows: u16,
}

impl Ph1HealthWiringConfig {
    pub fn mvp_v1(health_enabled: bool) -> Self {
        Self {
            health_enabled,
            max_issue_rows: 128,
            max_timeline_rows: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthWiringOutcome {
    NotInvokedDisabled,
    Refused(HealthRefuse),
    Forwarded(Ph1HealthResponse),
}

pub trait Ph1HealthEngine {
    fn run(&self, req: &Ph1HealthRequest) -> Ph1HealthResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1HealthWiring<E>
where
    E: Ph1HealthEngine,
{
    config: Ph1HealthWiringConfig,
    engine: E,
}

impl<E> Ph1HealthWiring<E>
where
    E: Ph1HealthEngine,
{
    pub fn new(config: Ph1HealthWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_issue_rows == 0 || config.max_issue_rows > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1health_wiring_config.max_issue_rows",
                reason: "must be within 1..=512",
            });
        }
        if config.max_timeline_rows == 0 || config.max_timeline_rows > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1health_wiring_config.max_timeline_rows",
                reason: "must be within 1..=2048",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_read(
        &self,
        req: &Ph1HealthRequest,
    ) -> Result<HealthWiringOutcome, ContractViolation> {
        if !self.config.health_enabled {
            return Ok(HealthWiringOutcome::NotInvokedDisabled);
        }

        if req.validate().is_err() {
            return Ok(HealthWiringOutcome::Refused(HealthRefuse::v1(
                req.capability_id(),
                reason_codes::PH1_HEALTH_INPUT_SCHEMA_INVALID,
                "health read request failed contract validation".to_string(),
            )?));
        }

        let bounded_req = self.bounded_request(req);
        let response = self.engine.run(&bounded_req);
        if response.validate().is_err() {
            return Ok(HealthWiringOutcome::Refused(HealthRefuse::v1(
                bounded_req.capability_id(),
                reason_codes::PH1_HEALTH_INTERNAL_PIPELINE_ERROR,
                "health engine response failed contract validation".to_string(),
            )?));
        }

        if !response_matches_request(&bounded_req, &response) {
            return Ok(HealthWiringOutcome::Refused(HealthRefuse::v1(
                bounded_req.capability_id(),
                reason_codes::PH1_HEALTH_RESPONSE_CAPABILITY_MISMATCH,
                "health engine response capability mismatched request".to_string(),
            )?));
        }

        match response {
            Ph1HealthResponse::Refuse(refuse) => Ok(HealthWiringOutcome::Refused(refuse)),
            out => Ok(HealthWiringOutcome::Forwarded(out)),
        }
    }

    fn bounded_request(&self, req: &Ph1HealthRequest) -> Ph1HealthRequest {
        match req {
            Ph1HealthRequest::HealthSnapshotRead(snapshot) => {
                let mut bounded: HealthSnapshotReadRequest = snapshot.clone();
                bounded.max_issue_rows = min(bounded.max_issue_rows, self.config.max_issue_rows);
                Ph1HealthRequest::HealthSnapshotRead(bounded)
            }
            Ph1HealthRequest::HealthIssueTimelineRead(timeline) => {
                let mut bounded: HealthIssueTimelineReadRequest = timeline.clone();
                bounded.max_timeline_rows =
                    min(bounded.max_timeline_rows, self.config.max_timeline_rows);
                Ph1HealthRequest::HealthIssueTimelineRead(bounded)
            }
            Ph1HealthRequest::HealthUnresolvedSummaryRead(unresolved) => {
                let mut bounded: HealthUnresolvedSummaryReadRequest = unresolved.clone();
                bounded.max_issue_rows = min(bounded.max_issue_rows, self.config.max_issue_rows);
                Ph1HealthRequest::HealthUnresolvedSummaryRead(bounded)
            }
            Ph1HealthRequest::HealthReportQueryRead(report_query) => {
                let mut bounded: HealthReportQueryReadRequest = report_query.clone();
                bounded.page_size = min(bounded.page_size, self.config.max_issue_rows);
                Ph1HealthRequest::HealthReportQueryRead(bounded)
            }
        }
    }
}

fn response_matches_request(req: &Ph1HealthRequest, response: &Ph1HealthResponse) -> bool {
    match (req, response) {
        (Ph1HealthRequest::HealthSnapshotRead(_), Ph1HealthResponse::HealthSnapshotReadOk(_)) => {
            true
        }
        (
            Ph1HealthRequest::HealthIssueTimelineRead(_),
            Ph1HealthResponse::HealthIssueTimelineReadOk(_),
        ) => true,
        (
            Ph1HealthRequest::HealthUnresolvedSummaryRead(_),
            Ph1HealthResponse::HealthUnresolvedSummaryReadOk(_),
        ) => true,
        (
            Ph1HealthRequest::HealthReportQueryRead(_),
            Ph1HealthResponse::HealthReportQueryReadOk(_),
        ) => true,
        (_, Ph1HealthResponse::Refuse(refuse)) => refuse.capability_id == req.capability_id(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1health::{
        HealthAckState, HealthActionResult, HealthCapabilityId, HealthIssueEvent,
        HealthIssueStatus, HealthIssueTimelineEntry, HealthIssueTimelineMetadata,
        HealthIssueTimelineReadOk, HealthIssueTimelineReadRequest, HealthReadEnvelope,
        HealthSeverity, HealthSnapshotReadOk, HealthSnapshotReadRequest,
        HealthUnresolvedSummaryReadRequest,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

    fn tenant(id: &str) -> TenantId {
        TenantId::new(id.to_string()).unwrap()
    }

    fn envelope() -> HealthReadEnvelope {
        HealthReadEnvelope::v1(CorrelationId(501), TurnId(71), MonotonicTimeNs(100)).unwrap()
    }

    fn event(issue: &str, attempt: u16, started: u64, bcast: Option<&str>) -> HealthIssueEvent {
        HealthIssueEvent::v1(
            tenant("tenant_a"),
            issue.to_string(),
            "PH1.C".to_string(),
            HealthSeverity::Warn,
            if bcast.is_some() {
                HealthIssueStatus::Escalated
            } else {
                HealthIssueStatus::Open
            },
            format!("ACT_{attempt}"),
            HealthActionResult::Fail,
            attempt,
            ReasonCodeId(8000 + attempt as u32),
            MonotonicTimeNs(started),
            None,
            Some(MonotonicTimeNs(120)),
            bcast.map(|v| v.to_string()),
            Some(HealthAckState::Waiting),
        )
        .unwrap()
    }

    struct DeterministicHealthEngine;

    impl Ph1HealthEngine for DeterministicHealthEngine {
        fn run(&self, req: &Ph1HealthRequest) -> Ph1HealthResponse {
            match req {
                Ph1HealthRequest::HealthSnapshotRead(_r) => {
                    Ph1HealthResponse::HealthSnapshotReadOk(
                        HealthSnapshotReadOk::v1(ReasonCodeId(8101), 1, 0, 0, 0, 0, vec![], true)
                            .unwrap(),
                    )
                }
                Ph1HealthRequest::HealthIssueTimelineRead(r) => {
                    let latest = r
                        .issue_events
                        .iter()
                        .max_by_key(|event| (event.started_at.0, event.attempt_no))
                        .cloned()
                        .unwrap();
                    let metadata = HealthIssueTimelineMetadata::v1(
                        latest.issue_id.clone(),
                        latest.owner_engine_id.clone(),
                        latest.severity,
                        latest.status,
                        latest.reason_code,
                        latest.unresolved_deadline_at,
                        latest.bcast_id.clone(),
                        latest.ack_state,
                    )
                    .unwrap();
                    let mut timeline_entries = r
                        .issue_events
                        .iter()
                        .cloned()
                        .map(|event| {
                            HealthIssueTimelineEntry::v1(
                                event.attempt_no,
                                event.action_id,
                                event.action_result,
                                event.reason_code,
                                event.started_at,
                                event.completed_at,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    timeline_entries.sort_by_key(|entry| entry.attempt_no);
                    Ph1HealthResponse::HealthIssueTimelineReadOk(
                        HealthIssueTimelineReadOk::v1(
                            ReasonCodeId(8102),
                            metadata,
                            timeline_entries,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1HealthRequest::HealthUnresolvedSummaryRead(_r) => {
                    Ph1HealthResponse::HealthUnresolvedSummaryReadOk(
                        selene_kernel_contracts::ph1health::HealthUnresolvedSummaryReadOk::v1(
                            ReasonCodeId(8103),
                            0,
                            0,
                            0,
                            vec![],
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1HealthRequest::HealthReportQueryRead(_r) => {
                    Ph1HealthResponse::HealthReportQueryReadOk(
                        selene_kernel_contracts::ph1health::HealthReportQueryReadOk::v1(
                            ReasonCodeId(8104),
                            "ctx_os_health".to_string(),
                            1,
                            "report query".to_string(),
                            vec![],
                            selene_kernel_contracts::ph1health::HealthReportQueryPaging::v1(
                                false, false, None, None,
                            )
                            .unwrap(),
                            None,
                            Some(
                                "Where do you want this report displayed: desktop or phone?"
                                    .to_string(),
                            ),
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct CapabilityDriftEngine;

    impl Ph1HealthEngine for CapabilityDriftEngine {
        fn run(&self, _req: &Ph1HealthRequest) -> Ph1HealthResponse {
            Ph1HealthResponse::HealthIssueTimelineReadOk(
                HealthIssueTimelineReadOk::v1(
                    ReasonCodeId(8202),
                    HealthIssueTimelineMetadata::v1(
                        "issue_x".to_string(),
                        "PH1.C".to_string(),
                        HealthSeverity::Warn,
                        HealthIssueStatus::Open,
                        ReasonCodeId(8201),
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                    vec![HealthIssueTimelineEntry::v1(
                        1,
                        "ACT_1".to_string(),
                        HealthActionResult::Fail,
                        ReasonCodeId(8201),
                        MonotonicTimeNs(10),
                        None,
                    )
                    .unwrap()],
                    true,
                )
                .unwrap(),
            )
        }
    }

    #[test]
    fn at_health_01_os_snapshot_read_forwards_schema_valid_response() {
        let wiring = Ph1HealthWiring::new(
            Ph1HealthWiringConfig::mvp_v1(true),
            DeterministicHealthEngine,
        )
        .unwrap();

        let req = Ph1HealthRequest::HealthSnapshotRead(
            HealthSnapshotReadRequest::v1(
                envelope(),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                None,
                None,
                false,
                64,
                vec![event("issue_a", 1, 10, None)],
            )
            .unwrap(),
        );

        let out = wiring.run_read(&req).unwrap();
        match out {
            HealthWiringOutcome::Forwarded(response) => {
                assert!(response.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_health_02_os_timeline_read_preserves_engine_entry_order() {
        let wiring = Ph1HealthWiring::new(
            Ph1HealthWiringConfig::mvp_v1(true),
            DeterministicHealthEngine,
        )
        .unwrap();

        let req = Ph1HealthRequest::HealthIssueTimelineRead(
            HealthIssueTimelineReadRequest::v1(
                envelope(),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                "issue_a".to_string(),
                64,
                vec![
                    event("issue_a", 2, 20, Some("bcast_01")),
                    event("issue_a", 1, 10, None),
                ],
            )
            .unwrap(),
        );

        let out = wiring.run_read(&req).unwrap();
        match out {
            HealthWiringOutcome::Forwarded(Ph1HealthResponse::HealthIssueTimelineReadOk(ok)) => {
                let attempt_order = ok
                    .timeline_entries
                    .iter()
                    .map(|entry| entry.attempt_no)
                    .collect::<Vec<_>>();
                assert_eq!(attempt_order, vec![1, 2]);
            }
            _ => panic!("expected HealthIssueTimelineReadOk"),
        }
    }

    #[test]
    fn at_health_03_os_not_invoked_when_disabled() {
        let wiring = Ph1HealthWiring::new(
            Ph1HealthWiringConfig::mvp_v1(false),
            DeterministicHealthEngine,
        )
        .unwrap();

        let req = Ph1HealthRequest::HealthUnresolvedSummaryRead(
            HealthUnresolvedSummaryReadRequest::v1(
                envelope(),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                64,
                vec![event("issue_a", 1, 10, None)],
            )
            .unwrap(),
        );

        let out = wiring.run_read(&req).unwrap();
        assert_eq!(out, HealthWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_health_04_os_fails_closed_on_response_capability_drift() {
        let wiring =
            Ph1HealthWiring::new(Ph1HealthWiringConfig::mvp_v1(true), CapabilityDriftEngine)
                .unwrap();

        let req = Ph1HealthRequest::HealthSnapshotRead(
            HealthSnapshotReadRequest::v1(
                envelope(),
                tenant("tenant_a"),
                "viewer_01".to_string(),
                false,
                None,
                None,
                false,
                64,
                vec![event("issue_a", 1, 10, None)],
            )
            .unwrap(),
        );

        let out = wiring.run_read(&req).unwrap();
        match out {
            HealthWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_HEALTH_RESPONSE_CAPABILITY_MISMATCH
                );
                assert_eq!(refuse.capability_id, HealthCapabilityId::HealthSnapshotRead);
            }
            _ => panic!("expected Refused"),
        }
    }
}
