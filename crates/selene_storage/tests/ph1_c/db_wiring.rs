#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1c::{
    ConfidenceBucket as Ph1cConfidenceBucket, LanguageTag, RetryAdvice as Ph1cRetryAdvice,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1cSttRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

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
fn at_c_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    let lang = LanguageTag::new("en-US").unwrap();
    s.ph1c_transcript_ok_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(1001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        "schedule meeting".to_string(),
        "hash_tenant_a".to_string(),
        lang.clone(),
        Ph1cConfidenceBucket::High,
        "c-tenant-a".to_string(),
    )
    .unwrap();
    s.ph1c_transcript_ok_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(2001),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        "book table".to_string(),
        "hash_tenant_b".to_string(),
        lang,
        Ph1cConfidenceBucket::High,
        "c-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1c_transcript_ok_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(3001),
        TurnId(1),
        None,
        user_a,
        device_a,
        "should fail".to_string(),
        "hash_mismatch".to_string(),
        LanguageTag::new("en-US").unwrap(),
        Ph1cConfidenceBucket::High,
        "c-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert_eq!(s.audit_rows_by_tenant("tenant_a").len(), 2);
    assert_eq!(s.audit_rows_by_tenant("tenant_b").len(), 2);
}

#[test]
fn at_c_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let committed = s
        .ph1c_transcript_ok_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(4001),
            TurnId(1),
            None,
            u,
            d,
            "open dashboard".to_string(),
            "hash_append".to_string(),
            LanguageTag::new("en-US").unwrap(),
            Ph1cConfidenceBucket::High,
            "c-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_conversation_turn(committed.conversation_turn_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
    assert!(matches!(
        s.attempt_overwrite_audit_event(committed.transcript_audit_event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_c_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(5001);
    let first = s
        .ph1c_transcript_ok_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            "start timer five minutes".to_string(),
            "hash_idem_first".to_string(),
            LanguageTag::new("en-US").unwrap(),
            Ph1cConfidenceBucket::High,
            "c-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1c_transcript_ok_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u,
            d,
            "different text ignored by dedupe".to_string(),
            "hash_idem_second".to_string(),
            LanguageTag::new("en-US").unwrap(),
            Ph1cConfidenceBucket::Low,
            "c-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first.conversation_turn_id, second.conversation_turn_id);
    assert_eq!(
        first.transcript_audit_event_id,
        second.transcript_audit_event_id
    );
    assert_eq!(
        first.candidate_eval_audit_event_id,
        second.candidate_eval_audit_event_id
    );
    assert_eq!(s.ph1c_voice_transcript_rows(corr).len(), 1);
    assert_eq!(s.audit_rows_by_correlation(corr).len(), 2);
}

#[test]
fn at_c_db_04_no_current_table_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(6001);
    s.ph1c_transcript_ok_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        u.clone(),
        d.clone(),
        "what is the weather".to_string(),
        "hash_current_ok".to_string(),
        LanguageTag::new("en-US").unwrap(),
        Ph1cConfidenceBucket::High,
        "c-current-ok".to_string(),
    )
    .unwrap();

    s.ph1c_transcript_reject_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        corr,
        TurnId(2),
        None,
        u,
        d,
        ReasonCodeId(0x4300_0004),
        Ph1cRetryAdvice::Repeat,
        Some("hash_current_reject".to_string()),
        "c-current-reject".to_string(),
    )
    .unwrap();

    // PH1.C row 13 is ledger-only on `conversation_ledger` + `audit_events`; no PH1.C current table exists.
    assert_eq!(s.ph1c_voice_transcript_rows(corr).len(), 1);
    assert_eq!(s.audit_rows_by_correlation(corr).len(), 4);
}
