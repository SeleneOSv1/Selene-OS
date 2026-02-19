#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1onb::OnboardingSessionId;
use selene_kernel_contracts::ph1w::{
    Ph1wOk, Ph1wRequest, Ph1wResponse, WakeEnrollCompleteResult, WakeEnrollDeferResult,
    WakeEnrollSampleResult, WakeEnrollStartResult, WakeEnrollStatus as ContractWakeEnrollStatus,
    WakeEnrollmentSessionId, WakeRequest, WakeSampleResult as ContractWakeSampleResult,
    PH1W_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{
    Ph1fStore, StorageError, WakeEnrollStatus as StoreWakeEnrollStatus,
    WakeSampleResult as StoreWakeSampleResult,
};
use selene_storage::ph1j::Ph1jRuntime;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.W reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const WAKE_OK_ENROLL_START_DRAFT: ReasonCodeId = ReasonCodeId(0x5700_0001);
    pub const WAKE_OK_ENROLL_SAMPLE_COMMIT: ReasonCodeId = ReasonCodeId(0x5700_0002);
    pub const WAKE_OK_ENROLL_COMPLETE_COMMIT: ReasonCodeId = ReasonCodeId(0x5700_0003);
    pub const WAKE_OK_ENROLL_DEFER_COMMIT: ReasonCodeId = ReasonCodeId(0x5700_0004);
}

pub const PH1_W_ENGINE_ID: &str = "PH1.W";
pub const PH1_W_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1W_IMPLEMENTATION_ID];

#[derive(Debug, Default, Clone)]
pub struct Ph1wRuntime;

impl Ph1wRuntime {
    pub fn run(
        &self,
        store: &mut Ph1fStore,
        req: &Ph1wRequest,
    ) -> Result<Ph1wResponse, StorageError> {
        self.run_for_implementation(store, PH1W_IMPLEMENTATION_ID, req)
    }

    pub fn run_for_implementation(
        &self,
        store: &mut Ph1fStore,
        implementation_id: &str,
        req: &Ph1wRequest,
    ) -> Result<Ph1wResponse, StorageError> {
        if implementation_id != PH1W_IMPLEMENTATION_ID {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1_w.implementation_id",
                    reason: "unknown implementation_id",
                },
            ));
        }
        req.validate().map_err(StorageError::ContractViolation)?;

        match &req.request {
            WakeRequest::EnrollStartDraft(r) => {
                let onboarding_session_id = r
                    .onboarding_session_id
                    .clone()
                    .map(OnboardingSessionId::new)
                    .transpose()
                    .map_err(StorageError::ContractViolation)?;

                let rec = store.ph1w_enroll_start_draft_with_ios_override(
                    req.now,
                    r.user_id.clone(),
                    r.device_id.clone(),
                    onboarding_session_id,
                    r.allow_ios_wake_override,
                    r.pass_target,
                    r.max_attempts,
                    r.enrollment_timeout_ms,
                    r.idempotency_key.clone(),
                )?;

                let out = WakeEnrollStartResult::v1(
                    WakeEnrollmentSessionId::new(rec.wake_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.wake_enroll_status),
                    rec.pass_target,
                    rec.max_attempts,
                    rec.enrollment_timeout_ms,
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "NONE",
                    status_label(rec.wake_enroll_status),
                    reason_codes::WAKE_OK_ENROLL_START_DRAFT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1wResponse::Ok(
                    Ph1wOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::WAKE_OK_ENROLL_START_DRAFT,
                        Some(out),
                        None,
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            WakeRequest::EnrollSampleCommit(r) => {
                let rec = store.ph1w_enroll_sample_commit(
                    req.now,
                    r.wake_enrollment_session_id.as_str().to_string(),
                    r.sample_duration_ms,
                    r.vad_coverage,
                    r.snr_db,
                    r.clipping_pct,
                    r.rms_dbfs,
                    r.noise_floor_dbfs,
                    r.peak_dbfs,
                    r.overlap_ratio,
                    map_sample_result(r.result),
                    r.reason_code,
                    r.idempotency_key.clone(),
                )?;

                let out = WakeEnrollSampleResult::v1(
                    WakeEnrollmentSessionId::new(rec.wake_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.wake_enroll_status),
                    rec.pass_count,
                    rec.attempt_count,
                    rec.reason_code,
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "WAKE_ENROLL_SAMPLE",
                    status_label(rec.wake_enroll_status),
                    reason_codes::WAKE_OK_ENROLL_SAMPLE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1wResponse::Ok(
                    Ph1wOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::WAKE_OK_ENROLL_SAMPLE_COMMIT,
                        None,
                        Some(out),
                        None,
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            WakeRequest::EnrollCompleteCommit(r) => {
                let rec = store.ph1w_enroll_complete_commit(
                    req.now,
                    r.wake_enrollment_session_id.as_str().to_string(),
                    r.wake_profile_id.clone(),
                    r.idempotency_key.clone(),
                )?;

                let wake_profile_id =
                    rec.wake_profile_id
                        .clone()
                        .ok_or(StorageError::ContractViolation(
                            ContractViolation::InvalidValue {
                                field: "ph1w_runtime.wake_profile_id",
                                reason: "must be present after complete commit",
                            },
                        ))?;

                let out = WakeEnrollCompleteResult::v1_with_sync_receipt(
                    WakeEnrollmentSessionId::new(rec.wake_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.wake_enroll_status),
                    wake_profile_id,
                    rec.wake_artifact_sync_receipt_ref.clone(),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "WAKE_ENROLL_IN_PROGRESS",
                    status_label(rec.wake_enroll_status),
                    reason_codes::WAKE_OK_ENROLL_COMPLETE_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1wResponse::Ok(
                    Ph1wOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::WAKE_OK_ENROLL_COMPLETE_COMMIT,
                        None,
                        None,
                        Some(out),
                        None,
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
            WakeRequest::EnrollDeferCommit(r) => {
                let rec = store.ph1w_enroll_defer_commit(
                    req.now,
                    r.wake_enrollment_session_id.as_str().to_string(),
                    r.deferred_until,
                    r.reason_code,
                    r.idempotency_key.clone(),
                )?;

                let out = WakeEnrollDeferResult::v1(
                    WakeEnrollmentSessionId::new(rec.wake_enrollment_session_id.clone())
                        .map_err(StorageError::ContractViolation)?,
                    map_status(rec.wake_enroll_status),
                    rec.deferred_until,
                    rec.reason_code.unwrap_or(r.reason_code),
                )
                .map_err(StorageError::ContractViolation)?;

                self.audit_transition(
                    store,
                    req.now,
                    req.correlation_id,
                    req.turn_id,
                    "WAKE_ENROLL_IN_PROGRESS",
                    status_label(rec.wake_enroll_status),
                    reason_codes::WAKE_OK_ENROLL_DEFER_COMMIT,
                    Some(r.idempotency_key.clone()),
                )?;

                Ok(Ph1wResponse::Ok(
                    Ph1wOk::v1(
                        req.simulation_id.clone(),
                        reason_codes::WAKE_OK_ENROLL_DEFER_COMMIT,
                        None,
                        None,
                        None,
                        Some(out),
                    )
                    .map_err(StorageError::ContractViolation)?,
                ))
            }
        }
    }

    fn audit_transition(
        &self,
        store: &mut Ph1fStore,
        now: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        state_from: &'static str,
        state_to: &'static str,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<(), StorageError> {
        let mut entries: BTreeMap<PayloadKey, PayloadValue> = BTreeMap::new();
        entries.insert(
            PayloadKey::new("state_from").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_from).map_err(StorageError::ContractViolation)?,
        );
        entries.insert(
            PayloadKey::new("state_to").map_err(StorageError::ContractViolation)?,
            PayloadValue::new(state_to).map_err(StorageError::ContractViolation)?,
        );
        let payload_min = AuditPayloadMin::v1(entries).map_err(StorageError::ContractViolation)?;

        // Use Other("ph1_w") until the global audit engine enum is updated.
        let engine = AuditEngine::Other("ph1_w".to_string());

        let ev = AuditEventInput::v1(
            now,
            None,
            None,
            None,
            None,
            None,
            engine,
            AuditEventType::StateTransition,
            reason_code,
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            payload_min,
            None,
            idempotency_key,
        )
        .map_err(StorageError::ContractViolation)?;

        Ph1jRuntime::emit(store, ev)?;
        Ok(())
    }
}

fn map_status(v: StoreWakeEnrollStatus) -> ContractWakeEnrollStatus {
    match v {
        StoreWakeEnrollStatus::InProgress => ContractWakeEnrollStatus::InProgress,
        StoreWakeEnrollStatus::Pending => ContractWakeEnrollStatus::Pending,
        StoreWakeEnrollStatus::Complete => ContractWakeEnrollStatus::Complete,
        StoreWakeEnrollStatus::Declined => ContractWakeEnrollStatus::Declined,
    }
}

fn status_label(v: StoreWakeEnrollStatus) -> &'static str {
    match v {
        StoreWakeEnrollStatus::InProgress => "WAKE_ENROLL_IN_PROGRESS",
        StoreWakeEnrollStatus::Pending => "WAKE_ENROLL_PENDING",
        StoreWakeEnrollStatus::Complete => "WAKE_ENROLL_COMPLETE",
        StoreWakeEnrollStatus::Declined => "WAKE_ENROLL_DECLINED",
    }
}

fn map_sample_result(v: ContractWakeSampleResult) -> StoreWakeSampleResult {
    match v {
        ContractWakeSampleResult::Pass => StoreWakeSampleResult::Pass,
        ContractWakeSampleResult::Fail => StoreWakeSampleResult::Fail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1w::{
        Ph1wResponse, WakeEnrollCompleteCommitRequest, WakeEnrollDeferCommitRequest,
        WakeEnrollSampleCommitRequest, WakeEnrollStartDraftRequest, WakeRequest,
        WakeSampleResult as ContractWakeSampleResult, WakeSimulationType, PH1W_CONTRACT_VERSION,
        WAKE_ENROLL_COMPLETE_COMMIT, WAKE_ENROLL_DEFER_COMMIT, WAKE_ENROLL_SAMPLE_COMMIT,
        WAKE_ENROLL_START_DRAFT,
    };
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus};

    fn now() -> MonotonicTimeNs {
        MonotonicTimeNs(1_000_000_000)
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).unwrap()
    }

    fn setup_store(uid: &UserId, did: &DeviceId) -> Ph1fStore {
        let mut s = Ph1fStore::new_in_memory();
        s.insert_identity(IdentityRecord::v1(
            uid.clone(),
            None,
            None,
            now(),
            IdentityStatus::Active,
        ))
        .unwrap();
        s.insert_device(
            DeviceRecord::v1(
                did.clone(),
                uid.clone(),
                "phone".to_string(),
                now(),
                Some("audio_prof_1".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        s
    }

    #[test]
    fn ph1w_happy_path_start_sample_complete() {
        let rt = Ph1wRuntime;
        let uid = user("wake-user-1");
        let did = device("wake-device-1");
        let mut store = setup_store(&uid, &did);

        let start = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(100),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(now().0 + 1),
            simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
            simulation_type: WakeSimulationType::Draft,
            request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                user_id: uid.clone(),
                device_id: did.clone(),
                onboarding_session_id: None,
                allow_ios_wake_override: false,
                pass_target: 3,
                max_attempts: 12,
                enrollment_timeout_ms: 300_000,
                idempotency_key: "wake-start-1".to_string(),
            }),
        };

        let start_out = rt.run(&mut store, &start).unwrap();
        let wake_session_id = match start_out {
            Ph1wResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, WAKE_ENROLL_START_DRAFT);
                ok.enroll_start_result.unwrap().wake_enrollment_session_id
            }
            Ph1wResponse::Refuse(_) => panic!("expected ok"),
        };

        for i in 0..3 {
            let sample = Ph1wRequest {
                schema_version: PH1W_CONTRACT_VERSION,
                correlation_id: CorrelationId(100),
                turn_id: TurnId((i + 2) as u64),
                now: MonotonicTimeNs(now().0 + 10 + i as u64),
                simulation_id: WAKE_ENROLL_SAMPLE_COMMIT.to_string(),
                simulation_type: WakeSimulationType::Commit,
                request: WakeRequest::EnrollSampleCommit(WakeEnrollSampleCommitRequest {
                    wake_enrollment_session_id: wake_session_id.clone(),
                    sample_duration_ms: 900,
                    vad_coverage: 0.7,
                    snr_db: 14.0,
                    clipping_pct: 0.5,
                    rms_dbfs: -20.0,
                    noise_floor_dbfs: -48.0,
                    peak_dbfs: -8.0,
                    overlap_ratio: 0.0,
                    result: ContractWakeSampleResult::Pass,
                    reason_code: None,
                    idempotency_key: format!("wake-sample-{i}"),
                }),
            };
            let sample_out = rt.run(&mut store, &sample).unwrap();
            match sample_out {
                Ph1wResponse::Ok(ok) => {
                    assert_eq!(ok.simulation_id, WAKE_ENROLL_SAMPLE_COMMIT);
                    let sample = ok.enroll_sample_result.unwrap();
                    assert_eq!(sample.attempt_count, (i + 1) as u8);
                    assert_eq!(sample.pass_count, (i + 1) as u8);
                }
                Ph1wResponse::Refuse(_) => panic!("expected ok"),
            }
        }

        let complete = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(100),
            turn_id: TurnId(9),
            now: MonotonicTimeNs(now().0 + 50),
            simulation_id: WAKE_ENROLL_COMPLETE_COMMIT.to_string(),
            simulation_type: WakeSimulationType::Commit,
            request: WakeRequest::EnrollCompleteCommit(WakeEnrollCompleteCommitRequest {
                wake_enrollment_session_id: wake_session_id.clone(),
                wake_profile_id: "wake_profile_u1_d1".to_string(),
                idempotency_key: "wake-complete-1".to_string(),
            }),
        };
        let complete_out = rt.run(&mut store, &complete).unwrap();
        match complete_out {
            Ph1wResponse::Ok(ok) => {
                assert_eq!(ok.simulation_id, WAKE_ENROLL_COMPLETE_COMMIT);
                let out = ok.enroll_complete_result.unwrap();
                assert_eq!(out.wake_profile_id, "wake_profile_u1_d1");
                assert_eq!(out.wake_enroll_status, ContractWakeEnrollStatus::Complete);
            }
            Ph1wResponse::Refuse(_) => panic!("expected ok"),
        }

        assert_eq!(
            store.ph1w_get_active_wake_profile(&uid, &did),
            Some("wake_profile_u1_d1")
        );
    }

    #[test]
    fn ph1w_defer_is_idempotent() {
        let rt = Ph1wRuntime;
        let uid = user("wake-user-2");
        let did = device("wake-device-2");
        let mut store = setup_store(&uid, &did);

        let start = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(200),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(now().0 + 1),
            simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
            simulation_type: WakeSimulationType::Draft,
            request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                user_id: uid,
                device_id: did,
                onboarding_session_id: None,
                allow_ios_wake_override: false,
                pass_target: 5,
                max_attempts: 12,
                enrollment_timeout_ms: 300_000,
                idempotency_key: "wake-start-2".to_string(),
            }),
        };

        let wake_session_id = match rt.run(&mut store, &start).unwrap() {
            Ph1wResponse::Ok(ok) => ok.enroll_start_result.unwrap().wake_enrollment_session_id,
            Ph1wResponse::Refuse(_) => panic!("expected ok"),
        };

        let defer = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(200),
            turn_id: TurnId(2),
            now: MonotonicTimeNs(now().0 + 2),
            simulation_id: WAKE_ENROLL_DEFER_COMMIT.to_string(),
            simulation_type: WakeSimulationType::Commit,
            request: WakeRequest::EnrollDeferCommit(WakeEnrollDeferCommitRequest {
                wake_enrollment_session_id: wake_session_id,
                deferred_until: Some(MonotonicTimeNs(now().0 + 10_000)),
                reason_code: ReasonCodeId(0x5700_0203),
                idempotency_key: "wake-defer-1".to_string(),
            }),
        };

        let first = rt.run(&mut store, &defer).unwrap();
        let second = rt.run(&mut store, &defer).unwrap();

        match (first, second) {
            (Ph1wResponse::Ok(a), Ph1wResponse::Ok(b)) => {
                let a = a.enroll_defer_result.unwrap();
                let b = b.enroll_defer_result.unwrap();
                assert_eq!(a.wake_enroll_status, ContractWakeEnrollStatus::Pending);
                assert_eq!(a.wake_enroll_status, b.wake_enroll_status);
                assert_eq!(a.deferred_until, b.deferred_until);
                assert_eq!(a.reason_code, b.reason_code);
            }
            _ => panic!("expected ok"),
        }
    }

    #[test]
    fn at_w_wiring_01_unknown_implementation_fails_closed() {
        let rt = Ph1wRuntime;
        let uid = user("wake-user-3");
        let did = device("wake-device-3");
        let mut store = setup_store(&uid, &did);

        let req = Ph1wRequest {
            schema_version: PH1W_CONTRACT_VERSION,
            correlation_id: CorrelationId(300),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(now().0 + 1),
            simulation_id: WAKE_ENROLL_START_DRAFT.to_string(),
            simulation_type: WakeSimulationType::Draft,
            request: WakeRequest::EnrollStartDraft(WakeEnrollStartDraftRequest {
                user_id: uid,
                device_id: did,
                onboarding_session_id: None,
                allow_ios_wake_override: false,
                pass_target: 3,
                max_attempts: 12,
                enrollment_timeout_ms: 300_000,
                idempotency_key: "wake-start-3".to_string(),
            }),
        };

        let out = rt.run_for_implementation(&mut store, "PH1.W.999", &req);
        assert!(matches!(
            out,
            Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field: "ph1_w.implementation_id",
                    reason: "unknown implementation_id",
                }
            ))
        ));
    }

    #[test]
    fn at_w_wiring_02_active_implementation_list_is_locked() {
        assert_eq!(PH1_W_ENGINE_ID, "PH1.W");
        assert_eq!(PH1_W_ACTIVE_IMPLEMENTATION_IDS, &["PH1.W.001"]);
    }
}
