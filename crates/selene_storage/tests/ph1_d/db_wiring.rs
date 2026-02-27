#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1d::{RequestId, SchemaHash};
use selene_kernel_contracts::ph1j::{AuditEngine, CorrelationId, DeviceId, PayloadKey, TurnId};
use selene_kernel_contracts::ph1n::TranscriptHash;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1dCommitEnvelope, Ph1fStore, StorageError,
};
use selene_storage::repo::{Ph1dRouterRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

fn user(id: &str) -> UserId {
    UserId::new(id).unwrap()
}

fn device(id: &str) -> DeviceId {
    DeviceId::new(id).unwrap()
}

fn seed_identity_device(store: &mut Ph1fStore, user_id: UserId, device_id: DeviceId) {
    store
        .insert_identity_row(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();

    store
        .insert_device_row(
            DeviceRecord::v1(
                device_id,
                user_id,
                "desktop".to_string(),
                MonotonicTimeNs(1),
                None,
            )
            .unwrap(),
        )
        .unwrap();
}

fn envelope() -> Ph1dCommitEnvelope {
    Ph1dCommitEnvelope::v1(
        RequestId(9_001),
        selene_kernel_contracts::SchemaVersion(1),
        SchemaHash(8_001),
        SchemaHash(8_002),
        SchemaHash(8_003),
        TranscriptHash(8_004),
        "model.router.v1".to_string(),
        "PRIMARY".to_string(),
        0,
        256,
    )
    .unwrap()
}

#[test]
fn at_d_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.ph1d_chat_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(11001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        envelope(),
        ReasonCodeId(0x4400_1001),
        "d-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1d_intent_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(12001),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        envelope(),
        "SET_REMINDER".to_string(),
        ReasonCodeId(0x4400_1002),
        "d-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1d_clarify_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(13001),
        TurnId(1),
        None,
        user_a,
        device_a,
        envelope(),
        "what_is_missing".to_string(),
        ReasonCodeId(0x4400_1003),
        "d-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert_eq!(s.audit_rows_by_tenant("tenant_a").len(), 1);
    assert_eq!(s.audit_rows_by_tenant("tenant_b").len(), 1);
}

#[test]
fn at_d_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let event_id = s
        .ph1d_analysis_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(14001),
            TurnId(1),
            None,
            u,
            d,
            envelope(),
            "ROUTE_SANITY_CHECK".to_string(),
            ReasonCodeId(0x4400_2001),
            "d-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_d_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(15001);
    let first = s
        .ph1d_clarify_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            envelope(),
            "time".to_string(),
            ReasonCodeId(0x4400_3001),
            "d-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1d_fail_closed_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(2),
            None,
            u,
            d,
            envelope(),
            "D_FAIL_INVALID_SCHEMA".to_string(),
            ReasonCodeId(0x4400_3002),
            "d-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(s.ph1d_audit_rows(corr).len(), 1);
}

#[test]
fn at_d_db_04_no_current_table_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(16001);
    s.ph1d_chat_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u.clone(),
        d.clone(),
        envelope(),
        ReasonCodeId(0x4400_4001),
        "d-current-chat".to_string(),
    )
    .unwrap();

    s.ph1d_intent_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        corr,
        TurnId(2),
        None,
        u.clone(),
        d.clone(),
        envelope(),
        "BOOK_TABLE".to_string(),
        ReasonCodeId(0x4400_4002),
        "d-current-intent".to_string(),
    )
    .unwrap();

    s.ph1d_analysis_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        corr,
        TurnId(3),
        None,
        u.clone(),
        d.clone(),
        envelope(),
        "PROMPT_VALIDATION".to_string(),
        ReasonCodeId(0x4400_4003),
        "d-current-analysis".to_string(),
    )
    .unwrap();

    s.ph1d_fail_closed_commit_row(
        MonotonicTimeNs(403),
        "tenant_a".to_string(),
        corr,
        TurnId(4),
        None,
        u,
        d,
        envelope(),
        "D_FAIL_FORBIDDEN_OUTPUT".to_string(),
        ReasonCodeId(0x4400_4004),
        "d-current-fail".to_string(),
    )
    .unwrap();

    // Row 15 is ledger-only on `audit_events`; no PH1.D-owned current table exists.
    assert_eq!(s.ph1d_audit_rows(corr).len(), 4);
}

#[test]
fn at_d_db_05_payload_includes_required_request_and_model_keys() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(17001);
    s.ph1d_chat_commit_row(
        MonotonicTimeNs(500),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u,
        d,
        envelope(),
        ReasonCodeId(0x4400_5001),
        "d-required-payload".to_string(),
    )
    .unwrap();

    let row = s
        .audit_events_by_tenant("tenant_a")
        .into_iter()
        .find(|event| event.engine == AuditEngine::Ph1D && event.correlation_id == corr)
        .expect("PH1.D row must exist");
    let entries = &row.payload_min.entries;
    for key in [
        "decision",
        "output_mode",
        "request_id",
        "prompt_template_version",
        "output_schema_hash",
        "tool_catalog_hash",
        "policy_context_hash",
        "transcript_hash",
        "model_id",
        "model_route_class",
        "temperature_bp",
        "max_tokens",
    ] {
        assert!(
            entries.contains_key(&PayloadKey::new(key).unwrap()),
            "missing payload key: {key}"
        );
    }
}
