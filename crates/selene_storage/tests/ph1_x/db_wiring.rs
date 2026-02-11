#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1jAuditRepo, Ph1xConversationRepo};

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

#[test]
fn at_x_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.ph1x_confirm_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(21001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        "COMMIT_POINT".to_string(),
        ReasonCodeId(0x5800_1001),
        "x-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1x_dispatch_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(22001),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        "TOOL_REQUEST".to_string(),
        ReasonCodeId(0x5800_1002),
        "x-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1x_wait_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(23001),
        TurnId(1),
        None,
        user_a,
        device_a,
        "BARGE_IN_CANCEL".to_string(),
        ReasonCodeId(0x5800_1003),
        "x-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert_eq!(s.audit_rows_by_tenant("tenant_a").len(), 1);
    assert_eq!(s.audit_rows_by_tenant("tenant_b").len(), 1);
}

#[test]
fn at_x_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let event_id = s
        .ph1x_dispatch_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(24001),
            TurnId(1),
            None,
            u,
            d,
            "SIMULATION_CANDIDATE".to_string(),
            ReasonCodeId(0x5800_2001),
            "x-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_x_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(25001);
    let first = s
        .ph1x_clarify_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            "location".to_string(),
            ReasonCodeId(0x5800_3001),
            "x-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1x_respond_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(2),
            None,
            u,
            d,
            "ACKNOWLEDGE_ABORT".to_string(),
            ReasonCodeId(0x5800_3002),
            "x-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(s.ph1x_audit_rows(corr).len(), 1);
}

#[test]
fn at_x_db_04_no_current_table_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(26001);
    s.ph1x_confirm_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u.clone(),
        d.clone(),
        "HIGH_IMPACT_ACTION".to_string(),
        ReasonCodeId(0x5800_4001),
        "x-current-confirm".to_string(),
    )
    .unwrap();

    s.ph1x_clarify_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        corr,
        TurnId(2),
        None,
        u.clone(),
        d.clone(),
        "recipient".to_string(),
        ReasonCodeId(0x5800_4002),
        "x-current-clarify".to_string(),
    )
    .unwrap();

    s.ph1x_respond_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        corr,
        TurnId(3),
        None,
        u.clone(),
        d.clone(),
        "TOOL_RESULT_OK".to_string(),
        ReasonCodeId(0x5800_4003),
        "x-current-respond".to_string(),
    )
    .unwrap();

    s.ph1x_dispatch_commit_row(
        MonotonicTimeNs(403),
        "tenant_a".to_string(),
        corr,
        TurnId(4),
        None,
        u.clone(),
        d.clone(),
        "TOOL_REQUEST".to_string(),
        ReasonCodeId(0x5800_4004),
        "x-current-dispatch".to_string(),
    )
    .unwrap();

    s.ph1x_wait_commit_row(
        MonotonicTimeNs(404),
        "tenant_a".to_string(),
        corr,
        TurnId(5),
        None,
        u,
        d,
        "BARGE_IN_CANCEL".to_string(),
        ReasonCodeId(0x5800_4005),
        "x-current-wait".to_string(),
    )
    .unwrap();

    // Row 16 is ledger-only on `audit_events`; no PH1.X-owned current table exists.
    assert_eq!(s.ph1x_audit_rows(corr).len(), 5);
}
