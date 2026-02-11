#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError, WakeEnrollStatus,
    WakeSampleResult,
};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1wWakeRepo};

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
fn at_w_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    let started_a = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(100),
            user_a.clone(),
            device_a.clone(),
            None,
            5,
            12,
            300_000,
            "w-tenant-a".to_string(),
        )
        .unwrap();
    let started_b = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(101),
            user_b.clone(),
            device_b.clone(),
            None,
            5,
            12,
            300_000,
            "w-tenant-b".to_string(),
        )
        .unwrap();

    assert_ne!(
        started_a.wake_enrollment_session_id,
        started_b.wake_enrollment_session_id
    );
    assert_eq!(started_a.device_id, device_a);
    assert_eq!(started_b.device_id, device_b);

    let cross_user_device = s.ph1w_enroll_start_draft_row(
        MonotonicTimeNs(102),
        user_a.clone(),
        device_b.clone(),
        None,
        5,
        12,
        300_000,
        "w-tenant-cross".to_string(),
    );
    assert!(matches!(
        cross_user_device,
        Err(StorageError::ContractViolation(_))
    ));

    let duplicate_in_progress = s.ph1w_enroll_start_draft_row(
        MonotonicTimeNs(103),
        user_a,
        device_a,
        None,
        5,
        12,
        300_000,
        "w-tenant-dup-in-progress".to_string(),
    );
    assert!(matches!(
        duplicate_in_progress,
        Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_w_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let started = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(200),
            u.clone(),
            d.clone(),
            None,
            5,
            12,
            300_000,
            "w-append-start".to_string(),
        )
        .unwrap();

    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(201),
        started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -17.0,
        -42.0,
        -3.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "w-append-sample".to_string(),
    )
    .unwrap();

    let runtime = s
        .ph1w_runtime_event_commit_row(
            MonotonicTimeNs(202),
            "wake_event_append_1".to_string(),
            None,
            Some(u),
            d,
            false,
            ReasonCodeId(0x5700_4001),
            None,
            true,
            false,
            Some(ReasonCodeId(0x5700_5001)),
            "w-append-runtime".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_wake_enrollment_sample_row(&started.wake_enrollment_session_id, 1),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
    assert!(matches!(
        s.attempt_overwrite_wake_runtime_event_row(&runtime.wake_event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_w_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let started = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(300),
            u.clone(),
            d.clone(),
            None,
            3,
            12,
            300_000,
            "w-idem-start".to_string(),
        )
        .unwrap();
    let started_dup = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(301),
            u.clone(),
            d.clone(),
            None,
            3,
            12,
            300_000,
            "w-idem-start".to_string(),
        )
        .unwrap();
    assert_eq!(
        started.wake_enrollment_session_id,
        started_dup.wake_enrollment_session_id
    );

    let sample_1 = s
        .ph1w_enroll_sample_commit_row(
            MonotonicTimeNs(302),
            started.wake_enrollment_session_id.clone(),
            850,
            0.90,
            14.0,
            0.01,
            -18.0,
            -43.0,
            -4.0,
            0.0,
            WakeSampleResult::Pass,
            None,
            "w-idem-sample-1".to_string(),
        )
        .unwrap();
    let sample_1_dup = s
        .ph1w_enroll_sample_commit_row(
            MonotonicTimeNs(303),
            started.wake_enrollment_session_id.clone(),
            850,
            0.90,
            14.0,
            0.01,
            -18.0,
            -43.0,
            -4.0,
            0.0,
            WakeSampleResult::Pass,
            None,
            "w-idem-sample-1".to_string(),
        )
        .unwrap();
    assert_eq!(sample_1.attempt_count, sample_1_dup.attempt_count);
    assert_eq!(
        s.ph1w_enrollment_sample_rows(&started.wake_enrollment_session_id)
            .len(),
        1
    );

    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(304),
        started.wake_enrollment_session_id.clone(),
        860,
        0.91,
        14.5,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "w-idem-sample-2".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(305),
        started.wake_enrollment_session_id.clone(),
        870,
        0.92,
        15.0,
        0.01,
        -17.5,
        -42.5,
        -3.5,
        0.0,
        WakeSampleResult::Pass,
        None,
        "w-idem-sample-3".to_string(),
    )
    .unwrap();

    let runtime_1 = s
        .ph1w_runtime_event_commit_row(
            MonotonicTimeNs(306),
            "wake_event_idem_1".to_string(),
            None,
            Some(u.clone()),
            d.clone(),
            true,
            ReasonCodeId(0x5700_4101),
            None,
            false,
            false,
            None,
            "w-idem-runtime".to_string(),
        )
        .unwrap();
    let runtime_1_dup = s
        .ph1w_runtime_event_commit_row(
            MonotonicTimeNs(307),
            "wake_event_idem_1_retry".to_string(),
            None,
            Some(u.clone()),
            d.clone(),
            true,
            ReasonCodeId(0x5700_4101),
            None,
            false,
            false,
            None,
            "w-idem-runtime".to_string(),
        )
        .unwrap();
    assert_eq!(runtime_1.wake_event_id, runtime_1_dup.wake_event_id);
    assert_eq!(s.ph1w_runtime_event_rows().len(), 1);

    let complete = s
        .ph1w_enroll_complete_commit_row(
            MonotonicTimeNs(308),
            started.wake_enrollment_session_id.clone(),
            "wake_profile_tenant_a_v1".to_string(),
            "w-idem-complete".to_string(),
        )
        .unwrap();
    let complete_dup = s
        .ph1w_enroll_complete_commit_row(
            MonotonicTimeNs(309),
            started.wake_enrollment_session_id.clone(),
            "wake_profile_tenant_a_v1".to_string(),
            "w-idem-complete".to_string(),
        )
        .unwrap();
    assert_eq!(complete.wake_profile_id, complete_dup.wake_profile_id);
    assert_eq!(
        s.ph1w_active_wake_profile(&u, &d),
        Some("wake_profile_tenant_a_v1")
    );
}

#[test]
fn at_w_db_04_current_table_consistency_with_enrollment_and_runtime_ledger() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let started = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(400),
            u.clone(),
            d.clone(),
            None,
            3,
            12,
            300_000,
            "w-current-start".to_string(),
        )
        .unwrap();

    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(401),
        started.wake_enrollment_session_id.clone(),
        820,
        0.70,
        8.0,
        0.05,
        -22.0,
        -48.0,
        -6.0,
        0.1,
        WakeSampleResult::Fail,
        Some(ReasonCodeId(0x5700_4201)),
        "w-current-sample-1".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(402),
        started.wake_enrollment_session_id.clone(),
        830,
        0.90,
        14.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "w-current-sample-2".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(403),
        started.wake_enrollment_session_id.clone(),
        840,
        0.91,
        14.5,
        0.01,
        -17.5,
        -42.5,
        -3.5,
        0.0,
        WakeSampleResult::Pass,
        None,
        "w-current-sample-3".to_string(),
    )
    .unwrap();
    let completed_by_samples = s
        .ph1w_enroll_sample_commit_row(
            MonotonicTimeNs(404),
            started.wake_enrollment_session_id.clone(),
            850,
            0.92,
            15.0,
            0.01,
            -17.0,
            -42.0,
            -3.0,
            0.0,
            WakeSampleResult::Pass,
            None,
            "w-current-sample-4".to_string(),
        )
        .unwrap();
    assert_eq!(
        completed_by_samples.wake_enroll_status,
        WakeEnrollStatus::Complete
    );

    let completed = s
        .ph1w_enroll_complete_commit_row(
            MonotonicTimeNs(405),
            started.wake_enrollment_session_id.clone(),
            "wake_profile_current_v1".to_string(),
            "w-current-complete".to_string(),
        )
        .unwrap();
    assert_eq!(
        completed.wake_profile_id.as_deref(),
        Some("wake_profile_current_v1")
    );
    assert_eq!(
        s.ph1w_active_wake_profile(&u, &d),
        Some("wake_profile_current_v1")
    );

    s.ph1w_runtime_event_commit_row(
        MonotonicTimeNs(406),
        "wake_event_current_1".to_string(),
        None,
        Some(u.clone()),
        d.clone(),
        false,
        ReasonCodeId(0x5700_4301),
        Some("wake_profile_current_v1".to_string()),
        true,
        false,
        Some(ReasonCodeId(0x5700_5301)),
        "w-current-runtime-1".to_string(),
    )
    .unwrap();
    s.ph1w_runtime_event_commit_row(
        MonotonicTimeNs(407),
        "wake_event_current_2".to_string(),
        None,
        Some(u),
        d,
        true,
        ReasonCodeId(0x5700_4302),
        Some("wake_profile_current_v1".to_string()),
        false,
        false,
        None,
        "w-current-runtime-2".to_string(),
    )
    .unwrap();

    let session = s
        .ph1w_enrollment_session_row(&started.wake_enrollment_session_id)
        .unwrap();
    assert_eq!(session.wake_enroll_status, WakeEnrollStatus::Complete);
    assert_eq!(session.attempt_count, 4);
    assert_eq!(session.pass_count, 3);
    assert_eq!(
        s.ph1w_enrollment_sample_rows(&started.wake_enrollment_session_id)
            .len(),
        4
    );
    assert_eq!(s.ph1w_runtime_event_rows().len(), 2);
}
