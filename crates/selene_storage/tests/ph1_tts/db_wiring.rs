#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1TtsRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

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
fn at_tts_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.ph1tts_started_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(41001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        "answer_a".to_string(),
        ReasonCodeId(0x5454_1001),
        "tts-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1tts_render_summary_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(42001),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        "ON_DEVICE".to_string(),
        "SHADOW".to_string(),
        "voice_b".to_string(),
        ReasonCodeId(0x5454_1002),
        "tts-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1tts_failed_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(43001),
        TurnId(1),
        None,
        user_a,
        device_a,
        "answer_a".to_string(),
        "TTS_FAIL_POLICY_RESTRICTED".to_string(),
        ReasonCodeId(0x5454_1003),
        "tts-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert_eq!(s.audit_rows_by_tenant("tenant_a").len(), 1);
    assert_eq!(s.audit_rows_by_tenant("tenant_b").len(), 1);
}

#[test]
fn at_tts_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let event_id = s
        .ph1tts_canceled_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(44001),
            TurnId(1),
            None,
            u,
            d,
            "answer_1".to_string(),
            "BARGE_IN".to_string(),
            ReasonCodeId(0x5454_2001),
            "tts-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_tts_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(45001);
    let first = s
        .ph1tts_started_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            "answer_2".to_string(),
            ReasonCodeId(0x5454_3001),
            "tts-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1tts_failed_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(2),
            None,
            u,
            d,
            "answer_2".to_string(),
            "TTS_FAIL_PROVIDER_TIMEOUT".to_string(),
            ReasonCodeId(0x5454_3002),
            "tts-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(s.ph1tts_audit_rows(corr).len(), 1);
}

#[test]
fn at_tts_db_04_no_current_table_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(46001);
    s.ph1tts_render_summary_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u.clone(),
        d.clone(),
        "ON_DEVICE".to_string(),
        "ASSIST".to_string(),
        "voice_1".to_string(),
        ReasonCodeId(0x5454_4001),
        "tts-current-summary".to_string(),
    )
    .unwrap();

    s.ph1tts_started_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        corr,
        TurnId(2),
        None,
        u.clone(),
        d.clone(),
        "answer_3".to_string(),
        ReasonCodeId(0x5454_4002),
        "tts-current-start".to_string(),
    )
    .unwrap();

    s.ph1tts_canceled_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        corr,
        TurnId(3),
        None,
        u.clone(),
        d.clone(),
        "answer_3".to_string(),
        "BARGE_IN".to_string(),
        ReasonCodeId(0x5454_4003),
        "tts-current-cancel".to_string(),
    )
    .unwrap();

    s.ph1tts_failed_commit_row(
        MonotonicTimeNs(403),
        "tenant_a".to_string(),
        corr,
        TurnId(4),
        None,
        u,
        d,
        "answer_4".to_string(),
        "TTS_FAIL_NETWORK_UNAVAILABLE".to_string(),
        ReasonCodeId(0x5454_4004),
        "tts-current-fail".to_string(),
    )
    .unwrap();

    // Row 18 is ledger-only on `audit_events`; no PH1.TTS-owned current table exists.
    assert_eq!(s.ph1tts_audit_rows(corr).len(), 4);
}
