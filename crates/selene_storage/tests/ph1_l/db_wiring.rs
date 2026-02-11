#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SessionState};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, SessionRecord, StorageError,
};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1jAuditRepo, Ph1lSessionLifecycleRepo};

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

fn session_record(
    session_id: u128,
    user_id: UserId,
    device_id: DeviceId,
    session_state: SessionState,
    opened_at: u64,
    last_activity_at: u64,
    closed_at: Option<u64>,
) -> SessionRecord {
    SessionRecord::v1(
        SessionId(session_id),
        user_id,
        device_id,
        session_state,
        MonotonicTimeNs(opened_at),
        MonotonicTimeNs(last_activity_at),
        closed_at.map(MonotonicTimeNs),
    )
    .unwrap()
}

#[test]
fn at_l_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("device_a");
    let device_b = device("device_b");

    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.upsert_session_lifecycle_row(
        session_record(
            1001,
            user_a.clone(),
            device_a,
            SessionState::Open,
            10,
            10,
            None,
        ),
        Some("idem_l_a".to_string()),
    )
    .unwrap();
    s.upsert_session_lifecycle_row(
        session_record(
            2001,
            user_b.clone(),
            device_b,
            SessionState::Open,
            11,
            11,
            None,
        ),
        Some("idem_l_b".to_string()),
    )
    .unwrap();

    let a = s.session_row(&SessionId(1001)).unwrap();
    let b = s.session_row(&SessionId(2001)).unwrap();
    assert_eq!(a.user_id, user_a);
    assert_eq!(b.user_id, user_b);
    assert_eq!(s.session_rows().len(), 2);
}

#[test]
fn at_l_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let device_a = device("device_a");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    s.upsert_session_lifecycle_row(
        session_record(
            3001,
            user_a.clone(),
            device_a.clone(),
            SessionState::Open,
            20,
            20,
            None,
        ),
        Some("idem_l_append".to_string()),
    )
    .unwrap();

    let audit_id = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(21),
                None,
                None,
                Some(SessionId(3001)),
                Some(user_a),
                Some(device_a),
                AuditEngine::Ph1L,
                AuditEventType::SessionOpen,
                ReasonCodeId(0x4C00_0001),
                AuditSeverity::Info,
                CorrelationId(701),
                TurnId(1),
                AuditPayloadMin::v1(BTreeMap::from([(
                    PayloadKey::new("session_state").unwrap(),
                    PayloadValue::new("OPEN").unwrap(),
                )]))
                .unwrap(),
                None,
                Some("idem_l_audit_append".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(audit_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_l_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("device_a");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let sid = s
        .upsert_session_lifecycle_row(
            session_record(4001, u.clone(), d.clone(), SessionState::Open, 30, 30, None),
            Some("idem_l_same".to_string()),
        )
        .unwrap();

    let sid_dup = s
        .upsert_session_lifecycle_row(
            session_record(4001, u, d, SessionState::Active, 30, 35, None),
            Some("idem_l_same".to_string()),
        )
        .unwrap();

    assert_eq!(sid, sid_dup);
    let current = s.session_row(&SessionId(4001)).unwrap();
    assert_eq!(current.session_state, SessionState::Open);
    assert_eq!(current.last_activity_at, MonotonicTimeNs(30));
}

#[test]
fn at_l_db_04_current_table_no_ledger_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("device_a");
    seed_identity_device(&mut s, u.clone(), d.clone());

    s.upsert_session_lifecycle_row(
        session_record(5001, u, d, SessionState::Active, 40, 45, None),
        Some("idem_l_current_only".to_string()),
    )
    .unwrap();

    // Row 8 is scoped to current `sessions` persistence; no PH1.L-owned session ledger is in-scope.
    assert_eq!(s.session_rows().len(), 1);
    assert_eq!(
        s.session_row(&SessionId(5001)).unwrap().session_state,
        SessionState::Active
    );
}
