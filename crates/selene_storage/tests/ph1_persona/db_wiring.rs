#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1PersonaRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

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
fn at_pers_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.ph1persona_profile_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(24001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        "DOMINANT".to_string(),
        "VOICE_ALLOWED".to_string(),
        "prefs_v1".to_string(),
        ReasonCodeId(0x5000_1001),
        "pers-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1persona_profile_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(24002),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        "GENTLE".to_string(),
        "TEXT_ONLY".to_string(),
        "prefs_v2".to_string(),
        ReasonCodeId(0x5000_1002),
        "pers-tenant-b".to_string(),
    )
    .unwrap();

    let mismatch = s.ph1persona_profile_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(24003),
        TurnId(1),
        None,
        user_a,
        device_a,
        "NEUTRAL".to_string(),
        "VOICE_ALLOWED".to_string(),
        "prefs_mismatch".to_string(),
        ReasonCodeId(0x5000_1003),
        "pers-tenant-mismatch".to_string(),
    );
    assert!(matches!(mismatch, Err(StorageError::ContractViolation(_))));

    assert_eq!(s.audit_rows_by_tenant("tenant_a").len(), 1);
    assert_eq!(s.audit_rows_by_tenant("tenant_b").len(), 1);
}

#[test]
fn at_pers_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let event_id = s
        .ph1persona_profile_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(25001),
            TurnId(1),
            None,
            u,
            d,
            "DOMINANT".to_string(),
            "VOICE_ALLOWED".to_string(),
            "prefs_append".to_string(),
            ReasonCodeId(0x5000_2001),
            "pers-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_pers_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(26001);
    let first = s
        .ph1persona_profile_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            "GENTLE".to_string(),
            "TEXT_ONLY".to_string(),
            "prefs_idem_1".to_string(),
            ReasonCodeId(0x5000_3001),
            "pers-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1persona_profile_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(2),
            None,
            u,
            d,
            "DOMINANT".to_string(),
            "VOICE_ALLOWED".to_string(),
            "prefs_idem_2".to_string(),
            ReasonCodeId(0x5000_3002),
            "pers-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(s.ph1persona_audit_rows(corr).len(), 1);
}

#[test]
fn at_pers_db_04_no_current_table_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(27001);
    s.ph1persona_profile_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u.clone(),
        d.clone(),
        "NEUTRAL".to_string(),
        "VOICE_ALLOWED".to_string(),
        "prefs_current_1".to_string(),
        ReasonCodeId(0x5000_4001),
        "pers-current-1".to_string(),
    )
    .unwrap();

    s.ph1persona_profile_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        corr,
        TurnId(2),
        None,
        u.clone(),
        d.clone(),
        "DOMINANT".to_string(),
        "TEXT_ONLY".to_string(),
        "prefs_current_2".to_string(),
        ReasonCodeId(0x5000_4002),
        "pers-current-2".to_string(),
    )
    .unwrap();

    // Row 24 is ledger-only on `audit_events`; no PH1.PERSONA-owned current table exists.
    assert_eq!(s.ph1persona_audit_rows(corr).len(), 2);
}
