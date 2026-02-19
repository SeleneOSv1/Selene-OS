#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1art::{
    ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
};
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1link::{AppPlatform, InviteeType, LinkStatus};
use selene_kernel_contracts::ph1onb::OnboardingSessionId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, MobileArtifactSyncKind, MobileArtifactSyncState,
    OnboardingVoiceRuntimeMode, Ph1fStore, StorageError, VoiceEnrollStatus,
};
use selene_storage::repo::{Ph1VidEnrollmentRepo, Ph1fFoundationRepo};

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

fn seed_onboarding_session(
    store: &mut Ph1fStore,
    inviter_user_id: UserId,
    device_fingerprint: &str,
    now_base: u64,
) -> OnboardingSessionId {
    let (link, _) = store
        .ph1link_invite_generate_draft(
            MonotonicTimeNs(now_base),
            inviter_user_id,
            InviteeType::FamilyMember,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    let (status, _, _, _, _, _, _, _, _, _) = store
        .ph1link_invite_open_activate_commit(
            MonotonicTimeNs(now_base + 1),
            link.token_id.clone(),
            device_fingerprint.to_string(),
        )
        .unwrap();
    assert_eq!(status, LinkStatus::Activated);

    store
        .ph1onb_session_start_draft(
            MonotonicTimeNs(now_base + 2),
            link.token_id,
            None,
            None,
            device_fingerprint.to_string(),
            AppPlatform::Ios,
            "legacy_app_instance".to_string(),
            format!("legacy_nonce_{}", now_base + 1),
            MonotonicTimeNs(now_base + 1),
        )
        .unwrap()
        .onboarding_session_id
}

#[test]
fn at_vid_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");

    seed_identity_device(&mut s, user_a.clone(), device_a.clone());
    seed_identity_device(&mut s, user_b.clone(), device_b.clone());

    let onb_a = seed_onboarding_session(&mut s, user_a, "fp_tenant_a", 100);
    let onb_b = seed_onboarding_session(&mut s, user_b, "fp_tenant_b", 200);

    let a = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(300),
            onb_a,
            device_a.clone(),
            true,
            8,
            120_000,
            3,
        )
        .unwrap();
    let b = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(301),
            onb_b,
            device_b.clone(),
            true,
            8,
            120_000,
            3,
        )
        .unwrap();

    assert_ne!(a.voice_enrollment_session_id, b.voice_enrollment_session_id);
    assert_eq!(a.device_id, device_a);
    assert_eq!(b.device_id, device_b);
}

#[test]
fn at_vid_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_append_only", 400);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(500), onb, d, true, 8, 120_000, 3)
        .unwrap();

    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(510),
        started.voice_enrollment_session_id.clone(),
        "sample_ref_append_1".to_string(),
        1,
        1_400,
        0.93,
        18.0,
        0.4,
        0.0,
        "vid-append-1".to_string(),
    )
    .unwrap();

    assert!(matches!(
        s.attempt_overwrite_voice_enrollment_sample_row(&started.voice_enrollment_session_id, 1),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_vid_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_idem", 600);

    let started = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(700),
            onb.clone(),
            d.clone(),
            true,
            5,
            60_000,
            2,
        )
        .unwrap();
    let started_dup = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(701), onb, d, true, 5, 60_000, 2)
        .unwrap();
    assert_eq!(
        started.voice_enrollment_session_id,
        started_dup.voice_enrollment_session_id
    );

    let s1 = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(710),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_idem_1".to_string(),
            1,
            1_350,
            0.91,
            17.0,
            0.5,
            0.0,
            "vid-idem-sample-1".to_string(),
        )
        .unwrap();
    let s1_dup = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(711),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_idem_1".to_string(),
            1,
            1_350,
            0.91,
            17.0,
            0.5,
            0.0,
            "vid-idem-sample-1".to_string(),
        )
        .unwrap();

    assert_eq!(s1.attempt_count, s1_dup.attempt_count);
    assert_eq!(
        s.ph1vid_enrollment_sample_rows(&started.voice_enrollment_session_id)
            .len(),
        1
    );

    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(712),
        started.voice_enrollment_session_id.clone(),
        "sample_ref_idem_2".to_string(),
        2,
        1_360,
        0.92,
        17.2,
        0.4,
        0.0,
        "vid-idem-sample-2".to_string(),
    )
    .unwrap();

    let c1 = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(720),
            started.voice_enrollment_session_id.clone(),
            "vid-idem-complete".to_string(),
        )
        .unwrap();
    let c1_dup = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(721),
            started.voice_enrollment_session_id.clone(),
            "vid-idem-complete".to_string(),
        )
        .unwrap();

    assert_eq!(c1.voice_profile_id, c1_dup.voice_profile_id);
}

#[test]
fn at_vid_db_04_current_table_consistency_with_sample_ledger() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_current", 800);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(900), onb, d, true, 8, 120_000, 3)
        .unwrap();

    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(910),
        started.voice_enrollment_session_id.clone(),
        "sample_ref_current_1".to_string(),
        1,
        850,
        0.62,
        9.0,
        4.2,
        0.0,
        "vid-current-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(911),
        started.voice_enrollment_session_id.clone(),
        "sample_ref_current_2".to_string(),
        2,
        1_300,
        0.90,
        16.0,
        0.6,
        0.0,
        "vid-current-sample-2".to_string(),
    )
    .unwrap();
    let locked = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(912),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_current_3".to_string(),
            3,
            1_280,
            0.91,
            16.4,
            0.6,
            0.0,
            "vid-current-sample-3".to_string(),
        )
        .unwrap();

    // lock_after_consecutive_passes=3 is not yet reached after FAIL,PASS,PASS.
    assert_eq!(locked.consecutive_passes, 2);

    let locked = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(913),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_current_4".to_string(),
            4,
            1_290,
            0.92,
            16.5,
            0.5,
            0.0,
            "vid-current-sample-4".to_string(),
        )
        .unwrap();
    assert_eq!(locked.voice_enroll_status, VoiceEnrollStatus::Locked);

    let completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(920),
            started.voice_enrollment_session_id.clone(),
            "vid-current-complete".to_string(),
        )
        .unwrap();

    let profile_id = completed.voice_profile_id.clone().unwrap();
    assert!(s.ph1vid_voice_profile_row(&profile_id).is_some());
    assert_eq!(
        s.ph1vid_enrollment_sample_rows(&started.voice_enrollment_session_id)
            .len(),
        4
    );
}

#[test]
fn at_vid_db_05_complete_commit_enqueues_mobile_sync_row() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_sync_1");
    let d = device("tenant_a_device_sync_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_sync", 900);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(910), onb, d, true, 8, 120_000, 2)
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(911),
        started.voice_enrollment_session_id.clone(),
        "sample_sync_1".to_string(),
        1,
        1_350,
        0.92,
        18.0,
        0.5,
        0.0,
        "vid-sync-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(912),
        started.voice_enrollment_session_id.clone(),
        "sample_sync_2".to_string(),
        2,
        1_360,
        0.92,
        18.2,
        0.4,
        0.0,
        "vid-sync-sample-2".to_string(),
    )
    .unwrap();
    let completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(913),
            started.voice_enrollment_session_id,
            "vid-sync-complete-1".to_string(),
        )
        .unwrap();

    let receipt = completed
        .voice_artifact_sync_receipt_ref
        .clone()
        .expect("voice receipt must exist");
    let sync_row = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("voice sync queue row must exist");
    assert_eq!(sync_row.sync_kind, MobileArtifactSyncKind::VoiceProfile);
}

#[test]
fn at_vid_db_06_sample_grading_is_runtime_scored_from_quality_metrics() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_quality_1");
    let d = device("tenant_a_device_quality_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_quality", 950);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(960), onb, d, true, 8, 120_000, 3)
        .unwrap();

    let st = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(961),
            started.voice_enrollment_session_id.clone(),
            "sample_quality_bad".to_string(),
            1,
            780,
            0.55,
            7.0,
            4.5,
            0.0,
            "vid-quality-sample-1".to_string(),
        )
        .unwrap();
    assert_eq!(st.consecutive_passes, 0);

    let sample_row = s
        .ph1vid_get_sample_for_attempt_and_idempotency(
            &started.voice_enrollment_session_id,
            1,
            "vid-quality-sample-1",
        )
        .expect("sample row should be present");
    assert!(sample_row.reason_code.is_some());
}

#[test]
fn at_vid_db_06b_enroll_start_persists_consent_scope_binding() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_consent_1");
    let d = device("tenant_a_device_consent_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_consent_scope", 965);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(966), onb, d, true, 8, 120_000, 2)
        .unwrap();
    assert!(started.consent_asserted);
    assert!(started
        .consent_scope_ref
        .starts_with("voice_enroll_consent:"));
}

#[test]
fn at_vid_db_06c_lock_criteria_enforce_min_duration_and_pending_mode_stays_limited() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_lock_criteria_1");
    let d = device("tenant_a_device_lock_criteria_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u.clone(), "fp_lock_criteria", 970);

    let started = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(971),
            onb.clone(),
            d.clone(),
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    assert_eq!(
        s.ph1onb_voice_runtime_mode(&onb),
        OnboardingVoiceRuntimeMode::Limited
    );
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(972),
        started.voice_enrollment_session_id.clone(),
        "sample_lock_gate_1".to_string(),
        1,
        1_200,
        0.92,
        17.0,
        0.4,
        0.0,
        "vid-lock-gate-sample-1".to_string(),
    )
    .unwrap();
    let second = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(973),
            started.voice_enrollment_session_id.clone(),
            "sample_lock_gate_2".to_string(),
            2,
            1_200,
            0.91,
            17.1,
            0.5,
            0.0,
            "vid-lock-gate-sample-2".to_string(),
        )
        .unwrap();
    assert_eq!(second.voice_enroll_status, VoiceEnrollStatus::Pending);
    assert_eq!(second.reason_code, Some(ReasonCodeId(0x5649_0307)));
    assert_eq!(
        s.ph1onb_voice_runtime_mode(&onb),
        OnboardingVoiceRuntimeMode::Limited
    );

    let u2 = user("tenant_a:user_lock_criteria_2");
    let d2 = device("tenant_a_device_lock_criteria_2");
    seed_identity_device(&mut s, u2.clone(), d2.clone());
    let onb_full = seed_onboarding_session(&mut s, u2, "fp_lock_full", 980);
    let started_full = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(981),
            onb_full.clone(),
            d2,
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(982),
        started_full.voice_enrollment_session_id.clone(),
        "sample_lock_full_1".to_string(),
        1,
        1_300,
        0.92,
        17.3,
        0.4,
        0.0,
        "vid-lock-full-sample-1".to_string(),
    )
    .unwrap();
    let locked = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(983),
            started_full.voice_enrollment_session_id,
            "sample_lock_full_2".to_string(),
            2,
            1_310,
            0.92,
            17.4,
            0.3,
            0.0,
            "vid-lock-full-sample-2".to_string(),
        )
        .unwrap();
    assert_eq!(locked.voice_enroll_status, VoiceEnrollStatus::Locked);
    assert_eq!(
        s.ph1onb_voice_runtime_mode(&onb_full),
        OnboardingVoiceRuntimeMode::Full
    );
}

#[test]
fn at_vid_db_07_mobile_sync_queue_dequeue_replay_ack_lifecycle() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_sync_worker_1");
    let d = device("tenant_a_device_sync_worker_1");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_sync_worker", 980);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(990), onb, d, true, 8, 120_000, 2)
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(991),
        started.voice_enrollment_session_id.clone(),
        "sample_worker_1".to_string(),
        1,
        1_340,
        0.92,
        17.4,
        0.4,
        0.0,
        "vid-worker-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(992),
        started.voice_enrollment_session_id.clone(),
        "sample_worker_2".to_string(),
        2,
        1_350,
        0.93,
        17.8,
        0.3,
        0.0,
        "vid-worker-sample-2".to_string(),
    )
    .unwrap();
    let completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(993),
            started.voice_enrollment_session_id,
            "vid-worker-complete".to_string(),
        )
        .unwrap();
    let receipt = completed
        .voice_artifact_sync_receipt_ref
        .clone()
        .expect("voice receipt must exist");
    let sync_job_id = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("sync queue row must exist")
        .sync_job_id
        .clone();

    let dequeued = s
        .mobile_artifact_sync_dequeue_batch(
            MonotonicTimeNs(994),
            1,
            1_000,
            "worker_vid_1".to_string(),
        )
        .unwrap();
    assert_eq!(dequeued.len(), 1);
    assert_eq!(dequeued[0].sync_job_id, sync_job_id);
    assert_eq!(dequeued[0].state, MobileArtifactSyncState::InFlight);
    assert_eq!(dequeued[0].attempt_count, 1);

    assert!(s
        .mobile_artifact_sync_replay_due_rows(MonotonicTimeNs(995))
        .is_empty());
    assert_eq!(
        s.mobile_artifact_sync_replay_due_rows(MonotonicTimeNs(2_000_000_000))
            .len(),
        1
    );

    let replay = s
        .mobile_artifact_sync_dequeue_batch(
            MonotonicTimeNs(2_000_000_001),
            1,
            1_000,
            "worker_vid_2".to_string(),
        )
        .unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].attempt_count, 2);

    s.mobile_artifact_sync_ack_commit(
        MonotonicTimeNs(2_000_000_002),
        &sync_job_id,
        Some("worker_vid_2"),
    )
    .unwrap();
    let row = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("sync queue row should remain queryable after ack");
    assert_eq!(row.state, MobileArtifactSyncState::Acked);
    assert!(row.acked_at.is_some());

    let none_left = s
        .mobile_artifact_sync_dequeue_batch(
            MonotonicTimeNs(2_000_000_100),
            1,
            1_000,
            "worker_vid_3".to_string(),
        )
        .unwrap();
    assert!(none_left.is_empty());
}

#[test]
fn at_vid_db_08_mobile_sync_ack_rejects_worker_mismatch() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_sync_worker_mismatch");
    let d = device("tenant_a_device_sync_worker_mismatch");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_sync_mismatch", 1_020);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(1_030), onb, d, true, 8, 120_000, 2)
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1_031),
        started.voice_enrollment_session_id.clone(),
        "sample_mismatch_1".to_string(),
        1,
        1_350,
        0.94,
        18.0,
        0.2,
        0.0,
        "vid-worker-mismatch-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1_032),
        started.voice_enrollment_session_id.clone(),
        "sample_mismatch_2".to_string(),
        2,
        1_360,
        0.93,
        17.9,
        0.3,
        0.0,
        "vid-worker-mismatch-sample-2".to_string(),
    )
    .unwrap();
    let completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(1_033),
            started.voice_enrollment_session_id,
            "vid-worker-mismatch-complete".to_string(),
        )
        .unwrap();
    let receipt = completed
        .voice_artifact_sync_receipt_ref
        .expect("voice receipt must exist");
    let sync_job_id = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("sync queue row must exist")
        .sync_job_id
        .clone();

    s.mobile_artifact_sync_dequeue_batch(
        MonotonicTimeNs(1_034),
        1,
        1_000,
        "worker_vid_match".to_string(),
    )
    .unwrap();

    let ack_mismatch = s.mobile_artifact_sync_ack_commit(
        MonotonicTimeNs(1_035),
        &sync_job_id,
        Some("worker_other"),
    );
    assert!(matches!(
        ack_mismatch,
        Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_vid_db_09_mobile_sync_fail_commit_records_error_and_retry_window() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_sync_fail");
    let d = device("tenant_a_device_sync_fail");
    seed_identity_device(&mut s, u.clone(), d.clone());
    let onb = seed_onboarding_session(&mut s, u, "fp_sync_fail", 1_100);

    let started = s
        .ph1vid_enroll_start_draft_row(MonotonicTimeNs(1_110), onb, d, true, 8, 120_000, 2)
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1_111),
        started.voice_enrollment_session_id.clone(),
        "sample_fail_1".to_string(),
        1,
        1_340,
        0.92,
        17.4,
        0.4,
        0.0,
        "vid-fail-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1_112),
        started.voice_enrollment_session_id.clone(),
        "sample_fail_2".to_string(),
        2,
        1_350,
        0.93,
        17.8,
        0.3,
        0.0,
        "vid-fail-sample-2".to_string(),
    )
    .unwrap();
    let completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(1_113),
            started.voice_enrollment_session_id,
            "vid-fail-complete".to_string(),
        )
        .unwrap();
    let receipt = completed
        .voice_artifact_sync_receipt_ref
        .expect("voice receipt must exist");
    let sync_job_id = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("sync queue row must exist")
        .sync_job_id
        .clone();

    s.mobile_artifact_sync_dequeue_batch(
        MonotonicTimeNs(1_114),
        1,
        30_000,
        "worker_vid_fail".to_string(),
    )
    .unwrap();
    s.mobile_artifact_sync_fail_commit(
        MonotonicTimeNs(1_115),
        &sync_job_id,
        Some("worker_vid_fail"),
        "network timeout".to_string(),
        5_000,
    )
    .unwrap();

    let row = s
        .mobile_artifact_sync_queue_row_for_receipt(&receipt)
        .expect("sync queue row should still exist");
    assert_eq!(row.state, MobileArtifactSyncState::InFlight);
    assert_eq!(row.last_error.as_deref(), Some("network timeout"));
    assert_eq!(row.worker_id.as_deref(), Some("worker_vid_fail"));
    assert_eq!(row.acked_at, None);
    assert_eq!(
        row.lease_expires_at,
        Some(MonotonicTimeNs(1_115 + 5_000_000_000))
    );
}

#[test]
fn at_vid_db_10_voice_artifact_manifest_changes_enqueue_sync_rows() {
    let mut s = Ph1fStore::new_in_memory();
    let tenant_id = "tenant_a".to_string();

    let artifact_types = [
        ArtifactType::VoiceIdThresholdPack,
        ArtifactType::VoiceIdConfusionPairPack,
        ArtifactType::VoiceIdSpoofPolicyPack,
        ArtifactType::VoiceIdProfileDeltaPack,
    ];

    for (idx, artifact_type) in artifact_types.into_iter().enumerate() {
        s.ph1learn_artifact_commit(
            MonotonicTimeNs(2_000 + idx as u64),
            tenant_id.clone(),
            ArtifactScopeType::Tenant,
            tenant_id.clone(),
            artifact_type,
            ArtifactVersion((idx + 1) as u32),
            format!("pkg_hash_voice_manifest_{idx}"),
            format!("payload_ref_voice_manifest_{idx}"),
            format!("prov_voice_manifest_{idx}"),
            ArtifactStatus::Active,
            format!("idem_voice_manifest_{idx}"),
        )
        .unwrap();
    }

    let queue_rows = s.mobile_artifact_sync_queue_rows();
    assert_eq!(queue_rows.len(), 4);
    for row in queue_rows {
        assert_eq!(row.sync_kind, MobileArtifactSyncKind::VoiceArtifactManifest);
    }
}

#[test]
fn at_vid_db_11_non_voice_artifact_does_not_enqueue_voice_manifest_sync() {
    let mut s = Ph1fStore::new_in_memory();
    let tenant_id = "tenant_a".to_string();

    s.ph1learn_artifact_commit(
        MonotonicTimeNs(3_000),
        tenant_id.clone(),
        ArtifactScopeType::Tenant,
        tenant_id.clone(),
        ArtifactType::SttRoutingPolicyPack,
        ArtifactVersion(1),
        "pkg_hash_non_voice".to_string(),
        "payload_ref_non_voice".to_string(),
        "prov_non_voice".to_string(),
        ArtifactStatus::Active,
        "idem_non_voice".to_string(),
    )
    .unwrap();

    assert!(s.mobile_artifact_sync_queue_rows().is_empty());
}

#[test]
fn at_vid_db_12_wake_artifact_manifest_changes_enqueue_sync_rows() {
    let mut s = Ph1fStore::new_in_memory();
    let tenant_id = "tenant_a".to_string();

    s.ph1learn_artifact_commit(
        MonotonicTimeNs(3_100),
        tenant_id.clone(),
        ArtifactScopeType::Tenant,
        tenant_id.clone(),
        ArtifactType::WakePack,
        ArtifactVersion(1),
        "pkg_hash_wake_manifest".to_string(),
        "payload_ref_wake_manifest".to_string(),
        "prov_wake_manifest".to_string(),
        ArtifactStatus::Active,
        "idem_wake_manifest".to_string(),
    )
    .unwrap();

    let queue_rows = s.mobile_artifact_sync_queue_rows();
    assert_eq!(queue_rows.len(), 1);
    assert_eq!(
        queue_rows[0].sync_kind,
        MobileArtifactSyncKind::WakeArtifactManifest
    );
}

#[test]
fn at_vid_db_13_emo_artifact_manifest_changes_enqueue_sync_rows() {
    let mut s = Ph1fStore::new_in_memory();
    let tenant_id = "tenant_a".to_string();

    for (idx, artifact_type) in [ArtifactType::EmoAffectPack, ArtifactType::EmoPolicyPack]
        .into_iter()
        .enumerate()
    {
        s.ph1learn_artifact_commit(
            MonotonicTimeNs(3_200 + idx as u64),
            tenant_id.clone(),
            ArtifactScopeType::Tenant,
            tenant_id.clone(),
            artifact_type,
            ArtifactVersion((idx + 1) as u32),
            format!("pkg_hash_emo_manifest_{idx}"),
            format!("payload_ref_emo_manifest_{idx}"),
            format!("prov_emo_manifest_{idx}"),
            ArtifactStatus::Active,
            format!("idem_emo_manifest_{idx}"),
        )
        .unwrap();
    }

    let queue_rows = s.mobile_artifact_sync_queue_rows();
    assert_eq!(queue_rows.len(), 2);
    for row in queue_rows {
        assert_eq!(row.sync_kind, MobileArtifactSyncKind::EmoArtifactManifest);
    }
}
