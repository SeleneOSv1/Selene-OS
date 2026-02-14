#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1link::{InviteeType, LinkStatus};
use selene_kernel_contracts::ph1onb::OnboardingSessionId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError, VoiceEnrollStatus,
    VoiceSampleResult,
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

    let (status, _, _, _, _, _) = store
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
        VoiceSampleResult::Pass,
        None,
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
            VoiceSampleResult::Pass,
            None,
            "vid-idem-sample-1".to_string(),
        )
        .unwrap();
    let s1_dup = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(711),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_idem_1".to_string(),
            1,
            VoiceSampleResult::Pass,
            None,
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
        VoiceSampleResult::Pass,
        None,
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
        VoiceSampleResult::Fail,
        Some(ReasonCodeId(0x5649_3001)),
        "vid-current-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(911),
        started.voice_enrollment_session_id.clone(),
        "sample_ref_current_2".to_string(),
        2,
        VoiceSampleResult::Pass,
        None,
        "vid-current-sample-2".to_string(),
    )
    .unwrap();
    let locked = s
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(912),
            started.voice_enrollment_session_id.clone(),
            "sample_ref_current_3".to_string(),
            3,
            VoiceSampleResult::Pass,
            None,
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
            VoiceSampleResult::Pass,
            None,
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
