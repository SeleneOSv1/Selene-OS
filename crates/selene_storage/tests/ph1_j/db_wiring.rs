#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity,
    BenchmarkComparisonOutcome, BenchmarkResultPacket, BenchmarkTargetPacket,
    BenchmarkTargetStatus, CanonicalProofRecordInput, CorrelationId, DeviceId, PayloadKey,
    PayloadValue, ProofProtectedActionClass, ProofRetentionClass, ProofSignerIdentityMetadata,
    ProofWriteOutcome, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1simcat::{SimulationId, SimulationVersion};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SessionState};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, SessionRecord, StorageError,
};
use selene_storage::repo::{BenchmarkResultRepo, Ph1jAuditRepo, Ph1jProofRepo};

fn user() -> UserId {
    UserId::new("dbw_j_user_1").unwrap()
}

fn device() -> DeviceId {
    DeviceId::new("dbw_j_device_1").unwrap()
}

fn store_with_identity_device_session() -> Ph1fStore {
    let mut s = Ph1fStore::new_in_memory();
    s.insert_identity(IdentityRecord::v1(
        user(),
        None,
        None,
        MonotonicTimeNs(1),
        IdentityStatus::Active,
    ))
    .unwrap();
    s.insert_device(
        DeviceRecord::v1(
            device(),
            user(),
            "mobile".to_string(),
            MonotonicTimeNs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    s.insert_session(
        SessionRecord::v1(
            SessionId(1),
            user(),
            device(),
            SessionState::Open,
            MonotonicTimeNs(1),
            MonotonicTimeNs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    s
}

fn payload_with_gate(gate: &str) -> AuditPayloadMin {
    AuditPayloadMin::v1(BTreeMap::from([(
        PayloadKey::new("gate").unwrap(),
        PayloadValue::new(gate).unwrap(),
    )]))
    .unwrap()
}

fn payload_with_entry_count(entry_count: usize) -> AuditPayloadMin {
    let mut entries = BTreeMap::new();
    for i in 0..entry_count {
        entries.insert(
            PayloadKey::new(format!("k{i}")).unwrap(),
            PayloadValue::new("v").unwrap(),
        );
    }
    AuditPayloadMin::v1(entries).unwrap()
}

fn proof_input(
    request_id: &str,
    trace_id: &str,
    turn_id: u64,
    execution_outcome: &str,
) -> CanonicalProofRecordInput {
    CanonicalProofRecordInput::v1(
        request_id.to_string(),
        trace_id.to_string(),
        Some(SessionId(1)),
        Some(TurnId(turn_id)),
        Some("user_scope:dbw_j_user_1".to_string()),
        Some(device()),
        "node_a".to_string(),
        "runtime_a".to_string(),
        "test".to_string(),
        "build_v1".to_string(),
        "deadbeef".to_string(),
        ProofProtectedActionClass::VoiceTurnExecution,
        Some("authority:allowed".to_string()),
        vec!["RG-PROOF-001".to_string()],
        Some("2026.03.08.v1".to_string()),
        Some(SimulationId::new("SIM_TEST".to_string()).unwrap()),
        Some(SimulationVersion(1)),
        Some("CERTIFIED_ACTIVE".to_string()),
        execution_outcome.to_string(),
        None,
        vec![ReasonCodeId(0x4A10_0001)],
        MonotonicTimeNs(50 + turn_id),
        MonotonicTimeNs(60 + turn_id),
        ProofSignerIdentityMetadata::v1(
            "ph1j_signer".to_string(),
            "ph1j_key".to_string(),
            "SHA256_KEYED_DIGEST".to_string(),
        )
        .unwrap(),
        ProofRetentionClass::ComplianceRetention,
        selene_kernel_contracts::ph1j::ProofVerificationPosture::VerificationReady,
        selene_kernel_contracts::ph1j::TimestampTrustPosture::RuntimeMonotonic,
        Some(format!("request:{request_id}")),
    )
    .unwrap()
}

#[test]
fn at_j_db_01_tenant_isolation_enforced() {
    let mut s = store_with_identity_device_session();

    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(10),
            Some("tenant_a".to_string()),
            Some("wo_a".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(0x4A00_0001),
            AuditSeverity::Info,
            CorrelationId(100),
            TurnId(1),
            payload_with_gate("wake"),
            None,
            Some("idem_a_1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(11),
            Some("tenant_b".to_string()),
            Some("wo_b".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(0x4A00_0002),
            AuditSeverity::Info,
            CorrelationId(100),
            TurnId(1),
            payload_with_gate("wake"),
            None,
            Some("idem_b_1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    let a = s.audit_rows_by_tenant("tenant_a");
    let b = s.audit_rows_by_tenant("tenant_b");
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].tenant_id.as_deref(), Some("tenant_a"));
    assert_eq!(b[0].tenant_id.as_deref(), Some("tenant_b"));
}

#[test]
fn at_j_db_02_append_only_enforced() {
    let mut s = store_with_identity_device_session();
    let id = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(20),
                Some("tenant_a".to_string()),
                Some("wo_a".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::Other,
                ReasonCodeId(0x4A00_0010),
                AuditSeverity::Info,
                CorrelationId(101),
                TurnId(1),
                AuditPayloadMin::empty_v1(),
                None,
                Some("idem_append_only".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_j_db_benchmark_envelope_is_replay_safe_and_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let target = BenchmarkTargetPacket::v1(
        "bench_target_stage2a_runtime".to_string(),
        "foundation".to_string(),
        "Stage 2A".to_string(),
        "runtime proof law benchmark envelope".to_string(),
        "documented".to_string(),
        BenchmarkTargetStatus::CertificationTargetPassed,
        None,
        Some("replay:stage2a".to_string()),
        Some("cert:stage2a".to_string()),
        MonotonicTimeNs(1),
    )
    .unwrap();

    let row_id = s
        .append_benchmark_target_row(target.clone(), Some("idem_bench_target_1".to_string()))
        .unwrap();
    let retry_row_id = s
        .append_benchmark_target_row(target.clone(), Some("idem_bench_target_1".to_string()))
        .unwrap();
    assert_eq!(row_id, retry_row_id);
    assert_eq!(s.benchmark_target_rows().len(), 1);

    let result = BenchmarkResultPacket::v1(
        "bench_result_stage2a_runtime".to_string(),
        target.benchmark_target_id.clone(),
        "run_stage2a_1".to_string(),
        Some("passed".to_string()),
        BenchmarkComparisonOutcome::Passed,
        BenchmarkTargetStatus::CertificationTargetPassed,
        None,
        Some("replay_artifact:stage2a:runtime".to_string()),
        Some("a".repeat(64)),
        None,
        None,
        MonotonicTimeNs(2),
    )
    .unwrap();
    let result_row_id = s
        .append_benchmark_result_row(result.clone(), Some("idem_bench_result_1".to_string()))
        .unwrap();
    let retry_result_row_id = s
        .append_benchmark_result_row(result.clone(), Some("idem_bench_result_1".to_string()))
        .unwrap();

    assert_eq!(result_row_id, retry_result_row_id);
    assert_eq!(s.benchmark_result_rows().len(), 1);
    assert_eq!(
        s.benchmark_results_by_target(&target.benchmark_target_id)
            .len(),
        1
    );
    let latest = s
        .latest_benchmark_result_for_target(&target.benchmark_target_id)
        .expect("latest benchmark result must exist");
    assert!(latest.certifies_target(&target));
}

#[test]
fn at_j_db_03_idempotency_dedupe_works() {
    let mut s = store_with_identity_device_session();

    let a1 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(30),
                Some("tenant_a".to_string()),
                Some("wo_123".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::StateTransition,
                ReasonCodeId(0x4A00_0020),
                AuditSeverity::Info,
                CorrelationId(200),
                TurnId(1),
                AuditPayloadMin::v1(BTreeMap::from([
                    (
                        PayloadKey::new("state_from").unwrap(),
                        PayloadValue::new("A").unwrap(),
                    ),
                    (
                        PayloadKey::new("state_to").unwrap(),
                        PayloadValue::new("B").unwrap(),
                    ),
                ]))
                .unwrap(),
                None,
                Some("idem_same".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    let a2 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(31),
                Some("tenant_a".to_string()),
                Some("wo_123".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::StateTransition,
                ReasonCodeId(0x4A00_0020),
                AuditSeverity::Info,
                CorrelationId(201),
                TurnId(2),
                AuditPayloadMin::v1(BTreeMap::from([
                    (
                        PayloadKey::new("state_from").unwrap(),
                        PayloadValue::new("A").unwrap(),
                    ),
                    (
                        PayloadKey::new("state_to").unwrap(),
                        PayloadValue::new("B").unwrap(),
                    ),
                ]))
                .unwrap(),
                None,
                Some("idem_same".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(a1, a2);
    assert_eq!(s.audit_rows().len(), 1);
}

#[test]
fn at_j_db_04_ledger_only_no_current_rebuild_required() {
    let mut s = store_with_identity_device_session();
    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(40),
            Some("tenant_a".to_string()),
            Some("wo_x".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::Other,
            ReasonCodeId(0x4A00_0030),
            AuditSeverity::Info,
            CorrelationId(300),
            TurnId(1),
            AuditPayloadMin::empty_v1(),
            None,
            Some("idem_ledger_only".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    // PH1.J has no current projection table in this slice; proof is append-only row presence.
    assert_eq!(s.audit_rows().len(), 1);
}

#[test]
fn at_j_db_07_append_audit_row_accepts_twenty_four_entry_payload() {
    let mut s = store_with_identity_device_session();
    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(41),
            Some("tenant_a".to_string()),
            Some("wo_24".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::Other,
            ReasonCodeId(0x4A00_0031),
            AuditSeverity::Info,
            CorrelationId(301),
            TurnId(1),
            payload_with_entry_count(24),
            None,
            Some("idem_payload_24".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(s.audit_rows().len(), 1);
    assert_eq!(s.audit_rows()[0].payload_min.entries.len(), 24);
}

#[test]
fn at_j_db_05_canonical_proof_record_is_append_only_and_hash_chained() {
    let mut s = store_with_identity_device_session();
    let receipt_1 = s
        .append_proof_row(
            proof_input("req_proof_1", "trace_proof_1", 1, "ALLOW"),
            Some("proof_idem_1".to_string()),
        )
        .unwrap();
    let receipt_2 = s
        .append_proof_row(
            proof_input("req_proof_2", "trace_proof_2", 2, "DISPATCH"),
            Some("proof_idem_2".to_string()),
        )
        .unwrap();

    assert_eq!(receipt_1.proof_write_outcome, ProofWriteOutcome::Written);
    assert_eq!(receipt_2.proof_write_outcome, ProofWriteOutcome::Written);
    assert_eq!(s.proof_rows().len(), 2);
    assert_eq!(
        s.proof_rows()[1].previous_event_hash.as_deref(),
        Some(s.proof_rows()[0].current_event_hash.as_str())
    );
    assert!(matches!(
        s.attempt_overwrite_proof_row(s.proof_rows()[0].proof_event_id),
        Err(StorageError::AppendOnlyViolation {
            table: "proof_ledger"
        })
    ));
}

#[test]
fn at_j_db_06_bounded_proof_reconstruction_by_request_and_session_turn() {
    let mut s = store_with_identity_device_session();
    s.append_proof_row(
        proof_input("req_reconstruct", "trace_reconstruct_a", 1, "ALLOW"),
        Some("proof_reconstruct_1".to_string()),
    )
    .unwrap();
    s.append_proof_row(
        proof_input("req_reconstruct", "trace_reconstruct_b", 1, "DISPATCH"),
        Some("proof_reconstruct_2".to_string()),
    )
    .unwrap();
    s.append_proof_row(
        proof_input("req_reconstruct_other", "trace_reconstruct_c", 2, "ALLOW"),
        Some("proof_reconstruct_3".to_string()),
    )
    .unwrap();

    let by_request = s
        .proof_rows_by_request_id_bounded("req_reconstruct", 1)
        .unwrap();
    assert_eq!(by_request.len(), 1);
    assert_eq!(by_request[0].request_id, "req_reconstruct");

    let by_session_turn = s
        .proof_rows_by_session_turn_bounded(SessionId(1), TurnId(1), 2)
        .unwrap();
    assert_eq!(by_session_turn.len(), 2);
    assert!(by_session_turn
        .iter()
        .all(|row| row.session_id == Some(SessionId(1))));
    assert!(by_session_turn
        .iter()
        .all(|row| row.turn_id == Some(TurnId(1))));
}

#[test]
fn at_j_db_07_proof_verification_recomputes_hash_and_signature() {
    let mut s = store_with_identity_device_session();
    let receipt = s
        .append_proof_row(
            proof_input("req_verify", "trace_verify", 7, "DISPATCH"),
            Some("proof_verify_1".to_string()),
        )
        .unwrap();
    assert_eq!(receipt.proof_write_outcome, ProofWriteOutcome::Written);

    let results = s
        .verify_proof_rows_by_request_id_bounded("req_verify", 4)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].verified);
    assert_eq!(results[0].failure_class, None);
}

#[test]
fn at_j_db_08_proof_idempotency_reuses_existing_record() {
    let mut s = store_with_identity_device_session();
    let receipt_1 = s
        .append_proof_row(
            proof_input("req_reuse", "trace_reuse", 9, "ALLOW"),
            Some("proof_reuse_1".to_string()),
        )
        .unwrap();
    let receipt_2 = s
        .append_proof_row(
            proof_input("req_reuse_duplicate", "trace_reuse_dup", 10, "DISPATCH"),
            Some("proof_reuse_1".to_string()),
        )
        .unwrap();
    assert_eq!(receipt_1.proof_event_id, receipt_2.proof_event_id);
    assert_eq!(
        receipt_2.proof_write_outcome,
        ProofWriteOutcome::ReusedExisting
    );
    assert_eq!(s.proof_rows().len(), 1);
}
