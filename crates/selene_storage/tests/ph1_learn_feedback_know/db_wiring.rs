#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1LearnFeedbackKnowRepo, Ph1fFoundationRepo};

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
fn at_learn_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    s.ph1feedback_event_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        CorrelationId(25001),
        TurnId(1),
        None,
        user_a.clone(),
        device_a.clone(),
        "USER_CORRECTION".to_string(),
        "MEDIUM".to_string(),
        ReasonCodeId(0x4C45_0101),
        "fb-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1feedback_event_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        CorrelationId(25002),
        TurnId(1),
        None,
        user_b.clone(),
        device_b.clone(),
        "STT_REJECT".to_string(),
        "HIGH".to_string(),
        ReasonCodeId(0x4C45_0102),
        "fb-tenant-b".to_string(),
    )
    .unwrap();

    let mismatch = s.ph1feedback_event_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        CorrelationId(25003),
        TurnId(1),
        None,
        user_a,
        device_a,
        "TOOL_FAIL".to_string(),
        "LOW".to_string(),
        ReasonCodeId(0x4C45_0103),
        "fb-tenant-mismatch".to_string(),
    );
    assert!(matches!(mismatch, Err(StorageError::ContractViolation(_))));

    s.ph1learn_artifact_commit_row(
        MonotonicTimeNs(103),
        "tenant_a".to_string(),
        ArtifactScopeType::Tenant,
        "tenant_a".to_string(),
        ArtifactType::SttAdaptationProfile,
        ArtifactVersion(1),
        "pkg_hash_learn_a".to_string(),
        "blob://learn_a".to_string(),
        "corr:25001".to_string(),
        ArtifactStatus::Active,
        "learn-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1know_dictionary_pack_commit_row(
        MonotonicTimeNs(104),
        "tenant_b".to_string(),
        ArtifactType::SttVocabPack,
        ArtifactVersion(1),
        "pkg_hash_know_b".to_string(),
        "blob://know_b".to_string(),
        "corr:25002".to_string(),
        "know-tenant-b".to_string(),
    )
    .unwrap();

    assert_eq!(
        s.ph1learn_artifact_rows(
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::SttAdaptationProfile
        )
        .len(),
        1
    );
    assert_eq!(
        s.ph1know_artifact_rows("tenant_b", ArtifactType::SttVocabPack)
            .len(),
        1
    );
}

#[test]
fn at_learn_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let feedback_event_id = s
        .ph1feedback_event_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            CorrelationId(26001),
            TurnId(1),
            None,
            u,
            d,
            "CLARIFY_LOOP".to_string(),
            "HIGH".to_string(),
            ReasonCodeId(0x4C45_0201),
            "fb-append".to_string(),
        )
        .unwrap();

    let learn_artifact_id = s
        .ph1learn_artifact_commit_row(
            MonotonicTimeNs(201),
            "tenant_a".to_string(),
            ArtifactScopeType::Tenant,
            "tenant_a".to_string(),
            ArtifactType::SttRoutingPolicyPack,
            ArtifactVersion(1),
            "pkg_hash_learn_append".to_string(),
            "blob://learn_append".to_string(),
            "corr:26001".to_string(),
            ArtifactStatus::Active,
            "learn-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(feedback_event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
    assert!(matches!(
        s.attempt_overwrite_artifact_ledger_row(learn_artifact_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_learn_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let corr = CorrelationId(27001);
    let fb_first = s
        .ph1feedback_event_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            corr,
            TurnId(1),
            None,
            u.clone(),
            d.clone(),
            "USER_CORRECTION".to_string(),
            "MEDIUM".to_string(),
            ReasonCodeId(0x4C45_0301),
            "fb-idem".to_string(),
        )
        .unwrap();

    let fb_second = s
        .ph1feedback_event_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            corr,
            TurnId(2),
            None,
            u,
            d,
            "STT_RETRY".to_string(),
            "LOW".to_string(),
            ReasonCodeId(0x4C45_0302),
            "fb-idem".to_string(),
        )
        .unwrap();
    assert_eq!(fb_first, fb_second);
    assert_eq!(s.ph1feedback_audit_rows(corr).len(), 1);

    let learn_first = s
        .ph1learn_artifact_commit_row(
            MonotonicTimeNs(302),
            "tenant_a".to_string(),
            ArtifactScopeType::Tenant,
            "tenant_a".to_string(),
            ArtifactType::SttAdaptationProfile,
            ArtifactVersion(1),
            "pkg_hash_learn_idem".to_string(),
            "blob://learn_idem".to_string(),
            "corr:27001".to_string(),
            ArtifactStatus::Active,
            "learn-idem".to_string(),
        )
        .unwrap();

    let learn_second = s
        .ph1learn_artifact_commit_row(
            MonotonicTimeNs(303),
            "tenant_a".to_string(),
            ArtifactScopeType::Tenant,
            "tenant_a".to_string(),
            ArtifactType::SttAdaptationProfile,
            ArtifactVersion(1),
            "pkg_hash_learn_idem_ignored".to_string(),
            "blob://learn_idem_ignored".to_string(),
            "corr:27001b".to_string(),
            ArtifactStatus::Active,
            "learn-idem".to_string(),
        )
        .unwrap();
    assert_eq!(learn_first, learn_second);
    assert_eq!(
        s.ph1learn_artifact_rows(
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::SttAdaptationProfile
        )
        .len(),
        1
    );

    let know_first = s
        .ph1know_dictionary_pack_commit_row(
            MonotonicTimeNs(304),
            "tenant_a".to_string(),
            ArtifactType::TtsPronunciationPack,
            ArtifactVersion(1),
            "pkg_hash_know_idem".to_string(),
            "blob://know_idem".to_string(),
            "corr:27001".to_string(),
            "know-idem".to_string(),
        )
        .unwrap();

    let know_second = s
        .ph1know_dictionary_pack_commit_row(
            MonotonicTimeNs(305),
            "tenant_a".to_string(),
            ArtifactType::TtsPronunciationPack,
            ArtifactVersion(1),
            "pkg_hash_know_idem_ignored".to_string(),
            "blob://know_idem_ignored".to_string(),
            "corr:27001b".to_string(),
            "know-idem".to_string(),
        )
        .unwrap();
    assert_eq!(know_first, know_second);
    assert_eq!(
        s.ph1know_artifact_rows("tenant_a", ArtifactType::TtsPronunciationPack)
            .len(),
        1
    );
}

#[test]
fn at_learn_db_04_ledger_only_no_current_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d);

    let corr = CorrelationId(28001);
    s.ph1feedback_event_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        corr,
        TurnId(1),
        None,
        user("tenant_a:user_1"),
        device("tenant_a_device_1"),
        "DELIVERY_SWITCH".to_string(),
        "MEDIUM".to_string(),
        ReasonCodeId(0x4C45_0401),
        "fb-current-1".to_string(),
    )
    .unwrap();

    s.ph1know_dictionary_pack_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        ArtifactType::SttVocabPack,
        ArtifactVersion(1),
        "pkg_hash_know_current".to_string(),
        "blob://know_current".to_string(),
        "corr:28001".to_string(),
        "know-current-1".to_string(),
    )
    .unwrap();

    let invalid_pack = s.ph1know_dictionary_pack_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        ArtifactType::WakePack,
        ArtifactVersion(1),
        "pkg_hash_invalid".to_string(),
        "blob://invalid".to_string(),
        "corr:28001b".to_string(),
        "know-invalid".to_string(),
    );
    assert!(matches!(
        invalid_pack,
        Err(StorageError::ContractViolation(_))
    ));

    // Row 25 is ledger-only (`audit_events` + `artifacts_ledger`); no LEARN/FEEDBACK/KNOW current projection.
    assert_eq!(s.ph1feedback_audit_rows(corr).len(), 1);
    assert_eq!(
        s.ph1know_artifact_rows("tenant_a", ArtifactType::SttVocabPack)
            .len(),
        1
    );
}
