#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1link::{AppPlatform, InviteeType, LinkStatus, PrefilledContext};
use selene_kernel_contracts::ph1onb::{
    BackfillCampaignState, BackfillRolloutScope, BackfillTargetStatus, OnboardingStatus, ProofType,
    TermsStatus, VerificationStatus,
};
use selene_kernel_contracts::ph1position::{
    PositionRequirementEvidenceMode, PositionRequirementExposureRule, PositionRequirementFieldSpec,
    PositionRequirementFieldType, PositionRequirementRuleType, PositionRequirementSensitivity,
    PositionScheduleType, PositionSchemaApplyScope, PositionSchemaSelectorSnapshot, TenantId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord, WakeSampleResult,
};
use selene_storage::repo::{
    Ph1LinkRepo, Ph1OnbRepo, Ph1PositionRepo, Ph1VidEnrollmentRepo, Ph1fFoundationRepo,
    Ph1jAuditRepo, Ph1wWakeRepo,
};

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

fn seed_identity_only(store: &mut Ph1fStore, user_id: UserId) {
    store
        .insert_identity_row(IdentityRecord::v1(
            user_id,
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();
}

#[allow(clippy::too_many_arguments)]
fn seed_activated_link(
    store: &mut Ph1fStore,
    now: u64,
    inviter_user_id: UserId,
    invitee_type: InviteeType,
    tenant_id: Option<String>,
    prefilled_context: Option<PrefilledContext>,
) -> selene_kernel_contracts::ph1link::TokenId {
    seed_activated_link_with_platform(
        store,
        now,
        inviter_user_id,
        invitee_type,
        tenant_id,
        prefilled_context,
        AppPlatform::Ios,
        "ios_instance_onb_test",
    )
}

#[allow(clippy::too_many_arguments)]
fn seed_activated_link_with_platform(
    store: &mut Ph1fStore,
    now: u64,
    inviter_user_id: UserId,
    invitee_type: InviteeType,
    tenant_id: Option<String>,
    prefilled_context: Option<PrefilledContext>,
    app_platform: AppPlatform,
    app_instance_id: &str,
) -> selene_kernel_contracts::ph1link::TokenId {
    let (link, _) = store
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(now),
            inviter_user_id,
            invitee_type,
            tenant_id,
            None,
            prefilled_context,
            None,
        )
        .unwrap();

    store
        .ph1link_invite_open_activate_commit_row(
            MonotonicTimeNs(now + 1),
            link.token_id.clone(),
            format!("fp_{now}"),
            app_platform,
            app_instance_id.to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    link.token_id
}

fn complete_locked_voice_enrollment(
    store: &mut Ph1fStore,
    onboarding_session_id: selene_kernel_contracts::ph1onb::OnboardingSessionId,
    device_id: DeviceId,
    now_base: u64,
    id_prefix: &str,
) -> String {
    let started = store
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(now_base),
            onboarding_session_id,
            device_id,
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    store
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(now_base + 1),
            started.voice_enrollment_session_id.clone(),
            format!("audio:{id_prefix}:voice:1"),
            1,
            1_420,
            0.93,
            18.2,
            0.4,
            0.0,
            format!("{id_prefix}-voice-sample-1"),
        )
        .unwrap();
    store
        .ph1vid_enroll_sample_commit_row(
            MonotonicTimeNs(now_base + 2),
            started.voice_enrollment_session_id.clone(),
            format!("audio:{id_prefix}:voice:2"),
            2,
            1_390,
            0.92,
            17.9,
            0.3,
            0.0,
            format!("{id_prefix}-voice-sample-2"),
        )
        .unwrap();
    let completed = store
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(now_base + 3),
            started.voice_enrollment_session_id,
            format!("{id_prefix}-voice-complete"),
        )
        .unwrap();
    completed
        .voice_artifact_sync_receipt_ref
        .expect("voice sync receipt should exist after complete")
}

fn seed_company(store: &mut Ph1fStore, tenant_id: &TenantId, company_id: &str) {
    store
        .ph1tenant_company_upsert_row(TenantCompanyRecord {
            schema_version: SchemaVersion(1),
            tenant_id: tenant_id.clone(),
            company_id: company_id.to_string(),
            legal_name: format!("{company_id} LLC"),
            jurisdiction: "US".to_string(),
            lifecycle_state: TenantCompanyLifecycleState::Active,
            created_at: MonotonicTimeNs(1),
            updated_at: MonotonicTimeNs(1),
        })
        .unwrap();
}

fn selector_snapshot() -> PositionSchemaSelectorSnapshot {
    PositionSchemaSelectorSnapshot {
        company_size: Some("SMALL".to_string()),
        industry_code: Some("LOGISTICS".to_string()),
        jurisdiction: Some("US".to_string()),
        position_family: Some("DRIVER".to_string()),
    }
}

fn required_doc_field(field_key: &str) -> PositionRequirementFieldSpec {
    PositionRequirementFieldSpec {
        field_key: field_key.to_string(),
        field_type: PositionRequirementFieldType::String,
        required_rule: PositionRequirementRuleType::Always,
        required_predicate_ref: None,
        validation_ref: None,
        sensitivity: PositionRequirementSensitivity::Private,
        exposure_rule: PositionRequirementExposureRule::InternalOnly,
        evidence_mode: PositionRequirementEvidenceMode::DocRequired,
        prompt_short: format!("Provide {field_key}"),
        prompt_long: format!("Please provide required field {field_key}."),
    }
}

fn seed_employee_position_schema_requiring_verification(
    store: &mut Ph1fStore,
    actor: UserId,
) -> (String, String, String) {
    let tenant = TenantId::new("tenant_a").unwrap();
    let company_id = "company_a".to_string();
    seed_company(store, &tenant, &company_id);

    let draft = store
        .ph1position_create_draft_row(
            MonotonicTimeNs(700),
            actor.clone(),
            tenant.clone(),
            company_id.clone(),
            "Driver".to_string(),
            "Logistics".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_driver".to_string(),
            "band_l2".to_string(),
            "onb-pos-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();
    let position_id = draft.position_id.as_str().to_string();

    store
        .ph1position_activate_commit_row(
            MonotonicTimeNs(701),
            actor.clone(),
            tenant.clone(),
            draft.position_id.clone(),
            "onb-pos-activate".to_string(),
            "POSITION_SIM_004_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0004),
        )
        .unwrap();

    store
        .ph1position_requirements_schema_create_draft_row(
            MonotonicTimeNs(702),
            actor.clone(),
            tenant.clone(),
            company_id.clone(),
            draft.position_id.clone(),
            "schema_v1".to_string(),
            selector_snapshot(),
            vec![required_doc_field("driver_license_doc_ref")],
            "onb-schema-create".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
            ReasonCodeId(0x5900_0006),
        )
        .unwrap();
    store
        .ph1position_requirements_schema_activate_commit_row(
            MonotonicTimeNs(703),
            actor,
            tenant.clone(),
            company_id.clone(),
            draft.position_id,
            "schema_v1".to_string(),
            PositionSchemaApplyScope::NewHiresOnly,
            "onb-schema-activate".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0008),
        )
        .unwrap();

    (tenant.as_str().to_string(), company_id, position_id)
}

#[test]
fn at_onb_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a);
    seed_identity_device(&mut s, user_b.clone(), device_b);

    let prefilled_a = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        None,
        None,
        None,
        None,
        None,
        Vec::new(),
    )
    .unwrap();
    let prefilled_b = PrefilledContext::v1(
        Some("tenant_b".to_string()),
        Some("company_b".to_string()),
        None,
        None,
        None,
        None,
        None,
        Vec::new(),
    )
    .unwrap();

    let link_a = seed_activated_link(
        &mut s,
        100,
        user_a,
        InviteeType::Employee,
        Some("tenant_a".to_string()),
        Some(prefilled_a),
    );
    let link_b = seed_activated_link(
        &mut s,
        200,
        user_b,
        InviteeType::Employee,
        Some("tenant_b".to_string()),
        Some(prefilled_b),
    );

    s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(300),
        link_a.clone(),
        None,
        Some("tenant_a".to_string()),
        "fp_onb_a".to_string(),
        AppPlatform::Ios,
        "ios_instance_onb_test".to_string(),
        "nonce_onb_test".to_string(),
        MonotonicTimeNs(1),
    )
    .unwrap();
    s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(301),
        link_b,
        None,
        Some("tenant_b".to_string()),
        "fp_onb_b".to_string(),
        AppPlatform::Ios,
        "ios_instance_onb_test".to_string(),
        "nonce_onb_test".to_string(),
        MonotonicTimeNs(1),
    )
    .unwrap();

    let mismatch = s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(302),
        link_a,
        None,
        Some("tenant_b".to_string()),
        "fp_onb_mismatch".to_string(),
        AppPlatform::Ios,
        "ios_instance_onb_test".to_string(),
        "nonce_onb_test".to_string(),
        MonotonicTimeNs(1),
    );
    assert!(matches!(mismatch, Err(StorageError::ContractViolation(_))));
    assert_eq!(s.ph1onb_session_rows().len(), 2);
}

#[test]
fn at_onb_db_01b_phone_first_start_requires_exact_link_open_activate_handoff_context() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_handoff_lock");
    let inviter_device = device("tenant_a_device_handoff_lock");
    seed_identity_device(&mut s, inviter.clone(), inviter_device);

    let token_id = seed_activated_link_with_platform(
        &mut s,
        260,
        inviter,
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Ios,
        "ios_instance_handoff",
    );

    let wrong_instance = s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(261),
        token_id.clone(),
        None,
        Some("tenant_a".to_string()),
        "fp_handoff_lock".to_string(),
        AppPlatform::Ios,
        "ios_instance_other".to_string(),
        "nonce_onb_test".to_string(),
        MonotonicTimeNs(1),
    );
    assert!(matches!(
        wrong_instance,
        Err(StorageError::ContractViolation(_))
    ));

    let wrong_nonce = s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(262),
        token_id.clone(),
        None,
        Some("tenant_a".to_string()),
        "fp_handoff_lock".to_string(),
        AppPlatform::Ios,
        "ios_instance_handoff".to_string(),
        "nonce_other".to_string(),
        MonotonicTimeNs(1),
    );
    assert!(matches!(
        wrong_nonce,
        Err(StorageError::ContractViolation(_))
    ));

    let wrong_link_opened_at = s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(263),
        token_id.clone(),
        None,
        Some("tenant_a".to_string()),
        "fp_handoff_lock".to_string(),
        AppPlatform::Ios,
        "ios_instance_handoff".to_string(),
        "nonce_onb_test".to_string(),
        MonotonicTimeNs(2),
    );
    assert!(matches!(
        wrong_link_opened_at,
        Err(StorageError::ContractViolation(_))
    ));

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(264),
            token_id.clone(),
            None,
            Some("tenant_a".to_string()),
            "fp_handoff_lock".to_string(),
            AppPlatform::Ios,
            "ios_instance_handoff".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();
    let replay = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(265),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_handoff_lock_replay".to_string(),
            AppPlatform::Ios,
            "ios_instance_handoff".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();
    assert_eq!(replay.onboarding_session_id, started.onboarding_session_id);
}

#[test]
fn at_onb_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let device_a = device("tenant_a_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a.clone());

    let audit_id = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(400),
                Some("tenant_a".to_string()),
                Some("wo_onb_1".to_string()),
                None,
                Some(user_a),
                Some(device_a),
                AuditEngine::Other("PH1.ONB".to_string()),
                AuditEventType::StateTransition,
                ReasonCodeId(0x4F4E_4201),
                AuditSeverity::Info,
                CorrelationId(61001),
                TurnId(1),
                AuditPayloadMin::v1(BTreeMap::from([
                    (
                        PayloadKey::new("state_from").unwrap(),
                        PayloadValue::new("NONE").unwrap(),
                    ),
                    (
                        PayloadKey::new("state_to").unwrap(),
                        PayloadValue::new("DRAFT_CREATED").unwrap(),
                    ),
                ]))
                .unwrap(),
                None,
                Some("onb-audit-append".to_string()),
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
fn at_onb_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let token_id = seed_activated_link(
        &mut s,
        500,
        u.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(503),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_idem".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    let first_terms = s
        .ph1onb_terms_accept_commit_row(
            MonotonicTimeNs(504),
            started.onboarding_session_id.clone(),
            "terms_v1".to_string(),
            true,
            "onb-terms-idem".to_string(),
        )
        .unwrap();
    let second_terms = s
        .ph1onb_terms_accept_commit_row(
            MonotonicTimeNs(505),
            started.onboarding_session_id.clone(),
            "terms_v2_should_be_ignored".to_string(),
            false,
            "onb-terms-idem".to_string(),
        )
        .unwrap();
    assert_eq!(first_terms.terms_status, TermsStatus::Accepted);
    assert_eq!(second_terms.terms_status, TermsStatus::Accepted);

    let first_device = s
        .ph1onb_primary_device_confirm_commit_row(
            MonotonicTimeNs(506),
            started.onboarding_session_id.clone(),
            d.clone(),
            ProofType::Passcode,
            true,
            "onb-device-idem".to_string(),
        )
        .unwrap();
    let second_device = s
        .ph1onb_primary_device_confirm_commit_row(
            MonotonicTimeNs(507),
            started.onboarding_session_id.clone(),
            d,
            ProofType::Biometric,
            false,
            "onb-device-idem".to_string(),
        )
        .unwrap();
    assert!(first_device.primary_device_confirmed);
    assert!(second_device.primary_device_confirmed);

    let current = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(current.terms_status, Some(TermsStatus::Accepted));
    assert!(current.primary_device_confirmed);
}

#[test]
fn at_onb_db_04_current_table_no_ledger_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d.clone());

    let token_id = seed_activated_link(
        &mut s,
        600,
        u.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(603),
            token_id.clone(),
            None,
            Some("tenant_a".to_string()),
            "fp_onb_flow".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(604),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-flow-terms".to_string(),
    )
    .unwrap();

    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(605),
        started.onboarding_session_id.clone(),
        d.clone(),
        ProofType::Passcode,
        true,
        "onb-flow-device".to_string(),
    )
    .unwrap();

    let access_created = s
        .ph1onb_access_instance_create_commit_row(
            MonotonicTimeNs(606),
            started.onboarding_session_id.clone(),
            u,
            Some("tenant_a".to_string()),
            "family_member".to_string(),
            "onb-flow-access".to_string(),
        )
        .unwrap();

    let voice_receipt = complete_locked_voice_enrollment(
        &mut s,
        started.onboarding_session_id.clone(),
        d,
        607,
        "onb-flow",
    );

    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(611),
            started.onboarding_session_id.clone(),
            "onb-flow-complete".to_string(),
            Some(voice_receipt),
            None,
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
    let link_after = s.ph1link_get_link_row(&token_id).unwrap();
    assert_eq!(link_after.status, LinkStatus::Consumed);

    let current = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(current.status, OnboardingStatus::Complete);
    assert_eq!(
        current.access_engine_instance_id.as_deref(),
        Some(access_created.access_engine_instance_id.as_str())
    );

    // Row 21 is scoped to current `onboarding_sessions` persistence; no PH1.ONB-owned session
    // ledger table is in-scope for rebuild in this lock slice.
    assert_eq!(s.ph1onb_session_rows().len(), 1);
}

#[test]
fn at_onb_db_05_session_start_pins_schema_context_and_required_gates() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter");
    let inviter_device = device("tenant_a_device_inviter");
    seed_identity_device(&mut s, inviter.clone(), inviter_device);

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, inviter.clone());
    let prefilled = PrefilledContext::v1(
        Some(tenant_id.clone()),
        Some(company_id),
        Some(position_id.clone()),
        Some("loc_1".to_string()),
        Some("2026-02-15".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let token_id = seed_activated_link(
        &mut s,
        800,
        inviter,
        InviteeType::Employee,
        Some(tenant_id.clone()),
        Some(prefilled),
    );
    let replay_token_id = token_id.clone();

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(801),
            token_id,
            None,
            Some(tenant_id.clone()),
            "fp_onb_schema_gated".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();
    let expected_schema_id = format!("position:{position_id}");
    assert_eq!(
        started.pinned_schema_id.as_deref(),
        Some(expected_schema_id.as_str())
    );
    assert_eq!(started.pinned_schema_version.as_deref(), Some("schema_v1"));
    assert_eq!(
        started.pinned_overlay_set_id.as_deref(),
        Some("position_requirements_active")
    );
    assert!(started
        .pinned_selector_snapshot_ref
        .as_deref()
        .unwrap_or("")
        .starts_with(&format!("selector:{tenant_id}:{position_id}:")));
    assert!(started
        .required_verification_gates
        .iter()
        .any(|g| g == "PHOTO_EVIDENCE_CAPTURE"));
    assert!(started
        .required_verification_gates
        .iter()
        .any(|g| g == "SENDER_CONFIRMATION"));

    let replay = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(802),
            replay_token_id,
            None,
            Some(tenant_id.clone()),
            "fp_onb_schema_gated_replay".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();
    assert_eq!(replay.onboarding_session_id, started.onboarding_session_id);
    assert_eq!(replay.pinned_schema_id, started.pinned_schema_id);
    assert_eq!(replay.pinned_schema_version, started.pinned_schema_version);
    assert_eq!(
        replay.required_verification_gates,
        started.required_verification_gates
    );

    let current = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(
        current.pinned_schema_id.as_deref(),
        Some(expected_schema_id.as_str())
    );
    assert_eq!(current.pinned_schema_version.as_deref(), Some("schema_v1"));
    assert_eq!(
        current.pinned_overlay_set_id.as_deref(),
        Some("position_requirements_active")
    );
    assert!(current
        .required_verification_gates
        .iter()
        .any(|g| g == "PHOTO_EVIDENCE_CAPTURE"));
    assert!(current
        .required_verification_gates
        .iter()
        .any(|g| g == "SENDER_CONFIRMATION"));
}

#[test]
fn at_onb_db_06_required_sender_verification_blocks_access_and_complete_until_confirmed() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter");
    let inviter_device = device("tenant_a_device_inviter");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, inviter.clone());
    let prefilled = PrefilledContext::v1(
        Some(tenant_id.clone()),
        Some(company_id),
        Some(position_id),
        Some("loc_1".to_string()),
        Some("2026-02-15".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let token_id = seed_activated_link(
        &mut s,
        900,
        inviter.clone(),
        InviteeType::Employee,
        Some(tenant_id.clone()),
        Some(prefilled),
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(901),
            token_id,
            None,
            Some(tenant_id.clone()),
            "fp_onb_required_verify".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(902),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-required-terms".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(903),
        started.onboarding_session_id.clone(),
        inviter_device.clone(),
        ProofType::Passcode,
        true,
        "onb-required-device".to_string(),
    )
    .unwrap();

    let access_blocked = s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(904),
        started.onboarding_session_id.clone(),
        user("tenant_a:hire_1"),
        Some(tenant_id.clone()),
        "employee".to_string(),
        "onb-required-access-blocked".to_string(),
    );
    assert!(matches!(
        access_blocked,
        Err(StorageError::ContractViolation(_))
    ));

    let complete_blocked = s.ph1onb_complete_commit_row(
        MonotonicTimeNs(905),
        started.onboarding_session_id.clone(),
        "onb-required-complete-blocked".to_string(),
        None,
        None,
    );
    assert!(matches!(
        complete_blocked,
        Err(StorageError::ContractViolation(_))
    ));

    s.ph1onb_employee_photo_capture_send_commit_row(
        MonotonicTimeNs(906),
        started.onboarding_session_id.clone(),
        "blob://required-photo".to_string(),
        inviter.clone(),
        "onb-required-photo".to_string(),
    )
    .unwrap();
    s.ph1onb_employee_sender_verify_commit_row(
        MonotonicTimeNs(907),
        started.onboarding_session_id.clone(),
        inviter,
        selene_kernel_contracts::ph1onb::SenderVerifyDecision::Confirm,
        "onb-required-verify".to_string(),
    )
    .unwrap();

    s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(908),
        started.onboarding_session_id.clone(),
        user("tenant_a:hire_1"),
        Some(tenant_id),
        "employee".to_string(),
        "onb-required-access-ok".to_string(),
    )
    .unwrap();
    let voice_receipt = complete_locked_voice_enrollment(
        &mut s,
        started.onboarding_session_id.clone(),
        inviter_device,
        909,
        "onb-required",
    );
    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(913),
            started.onboarding_session_id,
            "onb-required-complete-ok".to_string(),
            Some(voice_receipt),
            None,
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
}

#[test]
fn at_onb_db_12_required_verification_commit_idempotency_replays_deterministically() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter");
    let inviter_device = device("tenant_a_device_inviter");
    seed_identity_device(&mut s, inviter.clone(), inviter_device);

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, inviter.clone());
    let prefilled = PrefilledContext::v1(
        Some(tenant_id.clone()),
        Some(company_id),
        Some(position_id),
        Some("loc_1".to_string()),
        Some("2026-02-15".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let token_id = seed_activated_link(
        &mut s,
        950,
        inviter.clone(),
        InviteeType::Employee,
        Some(tenant_id.clone()),
        Some(prefilled),
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(951),
            token_id,
            None,
            Some(tenant_id.clone()),
            "fp_onb_required_verify_idem".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(952),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-required-idem-terms".to_string(),
    )
    .unwrap();

    let first_photo = s
        .ph1onb_employee_photo_capture_send_commit_row(
            MonotonicTimeNs(953),
            started.onboarding_session_id.clone(),
            "blob://required-photo-first".to_string(),
            inviter.clone(),
            "onb-required-photo-idem".to_string(),
        )
        .unwrap();
    let replay_photo = s
        .ph1onb_employee_photo_capture_send_commit_row(
            MonotonicTimeNs(954),
            started.onboarding_session_id.clone(),
            "blob://required-photo-should-be-ignored".to_string(),
            inviter.clone(),
            "onb-required-photo-idem".to_string(),
        )
        .unwrap();
    assert_eq!(replay_photo.photo_proof_ref, first_photo.photo_proof_ref);
    assert_eq!(
        replay_photo.verification_status,
        VerificationStatus::Pending
    );

    let after_photo = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(
        after_photo.photo_blob_ref.as_deref(),
        Some("blob://required-photo-first")
    );

    let first_verify = s
        .ph1onb_employee_sender_verify_commit_row(
            MonotonicTimeNs(955),
            started.onboarding_session_id.clone(),
            inviter.clone(),
            selene_kernel_contracts::ph1onb::SenderVerifyDecision::Confirm,
            "onb-required-verify-idem".to_string(),
        )
        .unwrap();
    let replay_verify = s
        .ph1onb_employee_sender_verify_commit_row(
            MonotonicTimeNs(956),
            started.onboarding_session_id.clone(),
            inviter,
            selene_kernel_contracts::ph1onb::SenderVerifyDecision::Reject,
            "onb-required-verify-idem".to_string(),
        )
        .unwrap();
    assert_eq!(
        first_verify.verification_status,
        VerificationStatus::Confirmed
    );
    assert_eq!(
        replay_verify.verification_status,
        VerificationStatus::Confirmed
    );

    let after_verify = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(
        after_verify.verification_status,
        Some(VerificationStatus::Confirmed)
    );
}

#[test]
fn at_onb_db_07_photo_sender_commits_refuse_when_schema_gate_not_required() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter");
    let inviter_device = device("tenant_a_device_inviter");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link(
        &mut s,
        1000,
        inviter.clone(),
        InviteeType::Employee,
        Some("tenant_a".to_string()),
        None,
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1001),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_no_required_verify".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();
    assert!(started.required_verification_gates.is_empty());

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(1002),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-no-required-terms".to_string(),
    )
    .unwrap();

    let photo_refused = s.ph1onb_employee_photo_capture_send_commit_row(
        MonotonicTimeNs(1003),
        started.onboarding_session_id.clone(),
        "blob://not-required-photo".to_string(),
        inviter.clone(),
        "onb-no-required-photo".to_string(),
    );
    assert!(matches!(
        photo_refused,
        Err(StorageError::ContractViolation(_))
    ));

    let sender_refused = s.ph1onb_employee_sender_verify_commit_row(
        MonotonicTimeNs(1004),
        started.onboarding_session_id.clone(),
        inviter,
        selene_kernel_contracts::ph1onb::SenderVerifyDecision::Confirm,
        "onb-no-required-verify".to_string(),
    );
    assert!(matches!(
        sender_refused,
        Err(StorageError::ContractViolation(_))
    ));

    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(1005),
        started.onboarding_session_id.clone(),
        inviter_device.clone(),
        ProofType::Passcode,
        true,
        "onb-no-required-device".to_string(),
    )
    .unwrap();
    s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(1006),
        started.onboarding_session_id.clone(),
        user("tenant_a:hire_optional"),
        Some("tenant_a".to_string()),
        "employee".to_string(),
        "onb-no-required-access".to_string(),
    )
    .unwrap();
    let voice_receipt = complete_locked_voice_enrollment(
        &mut s,
        started.onboarding_session_id.clone(),
        inviter_device,
        1007,
        "onb-no-required",
    );
    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(1011),
            started.onboarding_session_id,
            "onb-no-required-complete".to_string(),
            Some(voice_receipt),
            None,
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
}

#[test]
fn at_onb_db_13_ios_complete_allows_missing_wake_receipt_when_voice_locked() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter_ios");
    let inviter_device = device("tenant_a_device_inviter_ios");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link_with_platform(
        &mut s,
        1010,
        inviter.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Ios,
        "ios_instance_onb_test",
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1011),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_ios_no_wake".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(1012),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-ios-terms".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(1013),
        started.onboarding_session_id.clone(),
        inviter_device.clone(),
        ProofType::Passcode,
        true,
        "onb-ios-device".to_string(),
    )
    .unwrap();
    s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(1014),
        started.onboarding_session_id.clone(),
        user("tenant_a:invitee_ios_no_wake"),
        Some("tenant_a".to_string()),
        "family_member".to_string(),
        "onb-ios-access".to_string(),
    )
    .unwrap();

    let voice_started = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(1015),
            started.onboarding_session_id.clone(),
            inviter_device,
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1016),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:ios:voice:1".to_string(),
        1,
        1_420,
        0.93,
        18.4,
        0.5,
        0.0,
        "onb-ios-voice-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1017),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:ios:voice:2".to_string(),
        2,
        1_390,
        0.92,
        18.0,
        0.4,
        0.0,
        "onb-ios-voice-sample-2".to_string(),
    )
    .unwrap();
    let voice_completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(1018),
            voice_started.voice_enrollment_session_id,
            "onb-ios-voice-complete".to_string(),
        )
        .unwrap();
    let voice_receipt = voice_completed
        .voice_artifact_sync_receipt_ref
        .clone()
        .expect("voice sync receipt should exist");

    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(1019),
            started.onboarding_session_id.clone(),
            "onb-ios-complete".to_string(),
            Some(voice_receipt),
            None,
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);

    let current = s
        .ph1onb_session_row(&started.onboarding_session_id)
        .unwrap();
    assert_eq!(current.status, OnboardingStatus::Complete);
    assert!(current.wake_artifact_sync_receipt_ref.is_none());
}

#[test]
fn at_onb_db_14_android_complete_requires_wake_receipt_when_wake_is_complete() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter_android");
    let inviter_device = device("tenant_a_device_inviter_android");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link_with_platform(
        &mut s,
        1020,
        inviter.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Android,
        "android_instance_onb_test",
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1021),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_android_wake".to_string(),
            AppPlatform::Android,
            "android_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(1022),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-android-terms".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(1023),
        started.onboarding_session_id.clone(),
        inviter_device.clone(),
        ProofType::Passcode,
        true,
        "onb-android-device".to_string(),
    )
    .unwrap();
    s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(1024),
        started.onboarding_session_id.clone(),
        user("tenant_a:invitee_android_wake"),
        Some("tenant_a".to_string()),
        "family_member".to_string(),
        "onb-android-access".to_string(),
    )
    .unwrap();

    let voice_started = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(1025),
            started.onboarding_session_id.clone(),
            inviter_device.clone(),
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1026),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:android:voice:1".to_string(),
        1,
        1_410,
        0.92,
        17.8,
        0.5,
        0.0,
        "onb-android-voice-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1027),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:android:voice:2".to_string(),
        2,
        1_380,
        0.91,
        17.6,
        0.4,
        0.0,
        "onb-android-voice-sample-2".to_string(),
    )
    .unwrap();
    let voice_completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(1028),
            voice_started.voice_enrollment_session_id,
            "onb-android-voice-complete".to_string(),
        )
        .unwrap();
    let voice_receipt = voice_completed
        .voice_artifact_sync_receipt_ref
        .clone()
        .expect("voice sync receipt should exist");

    let wake_started = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(1029),
            inviter.clone(),
            inviter_device.clone(),
            Some(started.onboarding_session_id.clone()),
            3,
            12,
            300_000,
            "onb-android-wake-start".to_string(),
        )
        .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1030),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-android-wake-sample-1".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1031),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-android-wake-sample-2".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1032),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-android-wake-sample-3".to_string(),
    )
    .unwrap();
    let wake_completed = s
        .ph1w_enroll_complete_commit_row(
            MonotonicTimeNs(1033),
            wake_started.wake_enrollment_session_id,
            "wake_profile_android_v1".to_string(),
            "onb-android-wake-complete".to_string(),
        )
        .unwrap();
    let wake_receipt = wake_completed
        .wake_artifact_sync_receipt_ref
        .clone()
        .expect("wake sync receipt should exist");

    let missing_wake_receipt = s.ph1onb_complete_commit_row(
        MonotonicTimeNs(1034),
        started.onboarding_session_id.clone(),
        "onb-android-complete-missing-wake".to_string(),
        Some(voice_receipt.clone()),
        None,
    );
    assert!(matches!(
        missing_wake_receipt,
        Err(StorageError::ContractViolation(_))
    ));

    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(1035),
            started.onboarding_session_id,
            "onb-android-complete-with-wake".to_string(),
            Some(voice_receipt),
            Some(wake_receipt),
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
}

#[test]
fn at_onb_db_14b_desktop_complete_requires_wake_receipt_when_wake_is_complete() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter_desktop");
    let inviter_device = device("tenant_a_device_inviter_desktop");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link_with_platform(
        &mut s,
        1060,
        inviter.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Desktop,
        "desktop_instance_onb_test",
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1061),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_desktop_wake".to_string(),
            AppPlatform::Desktop,
            "desktop_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(1062),
        started.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-desktop-terms".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(1063),
        started.onboarding_session_id.clone(),
        inviter_device.clone(),
        ProofType::Passcode,
        true,
        "onb-desktop-device".to_string(),
    )
    .unwrap();
    s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(1064),
        started.onboarding_session_id.clone(),
        user("tenant_a:invitee_desktop_wake"),
        Some("tenant_a".to_string()),
        "family_member".to_string(),
        "onb-desktop-access".to_string(),
    )
    .unwrap();

    let voice_started = s
        .ph1vid_enroll_start_draft_row(
            MonotonicTimeNs(1065),
            started.onboarding_session_id.clone(),
            inviter_device.clone(),
            true,
            8,
            120_000,
            2,
        )
        .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1066),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:desktop:voice:1".to_string(),
        1,
        1_410,
        0.92,
        17.8,
        0.5,
        0.0,
        "onb-desktop-voice-sample-1".to_string(),
    )
    .unwrap();
    s.ph1vid_enroll_sample_commit_row(
        MonotonicTimeNs(1067),
        voice_started.voice_enrollment_session_id.clone(),
        "audio:desktop:voice:2".to_string(),
        2,
        1_380,
        0.90,
        16.9,
        0.6,
        0.0,
        "onb-desktop-voice-sample-2".to_string(),
    )
    .unwrap();
    let voice_completed = s
        .ph1vid_enroll_complete_commit_row(
            MonotonicTimeNs(1068),
            voice_started.voice_enrollment_session_id,
            "onb-desktop-voice-complete".to_string(),
        )
        .unwrap();
    let voice_receipt = voice_completed
        .voice_artifact_sync_receipt_ref
        .clone()
        .expect("voice sync receipt should exist");

    let wake_started = s
        .ph1w_enroll_start_draft_row(
            MonotonicTimeNs(1069),
            inviter,
            inviter_device,
            Some(started.onboarding_session_id.clone()),
            3,
            12,
            300_000,
            "onb-desktop-wake-start".to_string(),
        )
        .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1070),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-desktop-wake-sample-1".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1071),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-desktop-wake-sample-2".to_string(),
    )
    .unwrap();
    s.ph1w_enroll_sample_commit_row(
        MonotonicTimeNs(1072),
        wake_started.wake_enrollment_session_id.clone(),
        900,
        0.92,
        15.0,
        0.01,
        -18.0,
        -43.0,
        -4.0,
        0.0,
        WakeSampleResult::Pass,
        None,
        "onb-desktop-wake-sample-3".to_string(),
    )
    .unwrap();
    let wake_completed = s
        .ph1w_enroll_complete_commit_row(
            MonotonicTimeNs(1073),
            wake_started.wake_enrollment_session_id,
            "wake_profile_desktop_v1".to_string(),
            "onb-desktop-wake-complete".to_string(),
        )
        .unwrap();
    let wake_receipt = wake_completed
        .wake_artifact_sync_receipt_ref
        .clone()
        .expect("wake sync receipt should exist");

    let missing_wake_receipt = s.ph1onb_complete_commit_row(
        MonotonicTimeNs(1074),
        started.onboarding_session_id.clone(),
        "onb-desktop-complete-missing-wake".to_string(),
        Some(voice_receipt.clone()),
        None,
    );
    assert!(matches!(
        missing_wake_receipt,
        Err(StorageError::ContractViolation(_))
    ));

    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(1075),
            started.onboarding_session_id,
            "onb-desktop-complete-with-wake".to_string(),
            Some(voice_receipt),
            Some(wake_receipt),
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
}

#[test]
fn at_onb_db_15_wake_start_refused_for_ios_onboarding_session_default_policy() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter_ios_wake");
    let inviter_device = device("tenant_a_device_inviter_ios_wake");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link_with_platform(
        &mut s,
        1040,
        inviter.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Ios,
        "ios_instance_onb_test",
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1041),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_ios_wake_refuse".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    let wake_start = s.ph1w_enroll_start_draft_row(
        MonotonicTimeNs(1042),
        inviter,
        inviter_device,
        Some(started.onboarding_session_id),
        3,
        12,
        300_000,
        "onb-ios-wake-start-should-refuse".to_string(),
    );
    assert!(matches!(
        wake_start,
        Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_onb_db_16_wake_start_allows_ios_override_flag() {
    let mut s = Ph1fStore::new_in_memory();
    let inviter = user("tenant_a:user_inviter_ios_wake_override");
    let inviter_device = device("tenant_a_device_inviter_ios_wake_override");
    seed_identity_device(&mut s, inviter.clone(), inviter_device.clone());

    let token_id = seed_activated_link_with_platform(
        &mut s,
        1050,
        inviter.clone(),
        InviteeType::FamilyMember,
        Some("tenant_a".to_string()),
        None,
        AppPlatform::Ios,
        "ios_instance_onb_test",
    );
    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(1051),
            token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_ios_wake_refuse".to_string(),
            AppPlatform::Ios,
            "ios_instance_onb_test".to_string(),
            "nonce_onb_test".to_string(),
            MonotonicTimeNs(1),
        )
        .unwrap();

    let wake_start = s.ph1w_enroll_start_draft_with_ios_override(
        MonotonicTimeNs(1052),
        inviter,
        inviter_device,
        Some(started.onboarding_session_id),
        true,
        3,
        12,
        300_000,
        "onb-ios-wake-start-allow-override".to_string(),
    );
    assert!(wake_start.is_ok());
}

#[test]
fn at_onb_db_08_backfill_new_hires_only_is_refused_no_campaign_started() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:user_inviter");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_inviter"));

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, actor.clone());

    let out = s.ph1onb_requirement_backfill_start_draft_row(
        MonotonicTimeNs(1100),
        actor,
        tenant_id,
        company_id,
        position_id,
        "schema_v2".to_string(),
        BackfillRolloutScope::NewHiresOnly,
        "bf-new-hires-only".to_string(),
        "ONB_REQUIREMENT_BACKFILL_START_DRAFT",
        ReasonCodeId(0x4F00_0008),
    );

    assert!(matches!(out, Err(StorageError::ContractViolation(_))));
}

#[test]
fn at_onb_db_09_backfill_current_and_new_creates_campaign_with_deterministic_snapshot() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:user_inviter");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_inviter"));
    seed_identity_only(&mut s, user("tenant_a:worker_1"));
    seed_identity_only(&mut s, user("tenant_a:worker_2"));

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, actor.clone());

    let started = s
        .ph1onb_requirement_backfill_start_draft_row(
            MonotonicTimeNs(1200),
            actor.clone(),
            tenant_id.clone(),
            company_id.clone(),
            position_id.clone(),
            "schema_v2".to_string(),
            BackfillRolloutScope::CurrentAndNew,
            "bf-current-and-new".to_string(),
            "ONB_REQUIREMENT_BACKFILL_START_DRAFT",
            ReasonCodeId(0x4F00_0008),
        )
        .unwrap();
    assert_eq!(started.state, BackfillCampaignState::Running);
    assert!(started.pending_target_count >= 3);

    let replay = s
        .ph1onb_requirement_backfill_start_draft_row(
            MonotonicTimeNs(1201),
            actor,
            tenant_id,
            company_id,
            position_id,
            "schema_v2".to_string(),
            BackfillRolloutScope::CurrentAndNew,
            "bf-current-and-new".to_string(),
            "ONB_REQUIREMENT_BACKFILL_START_DRAFT",
            ReasonCodeId(0x4F00_0008),
        )
        .unwrap();

    assert_eq!(replay.campaign_id, started.campaign_id);
    assert_eq!(replay.state, started.state);
    assert_eq!(replay.pending_target_count, started.pending_target_count);
}

#[test]
fn at_onb_db_10_backfill_notify_loop_and_complete_are_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:user_inviter");
    let target = user("tenant_a:worker_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_inviter"));
    seed_identity_only(&mut s, target.clone());

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, actor);

    let started = s
        .ph1onb_requirement_backfill_start_draft_row(
            MonotonicTimeNs(1300),
            user("tenant_a:user_inviter"),
            tenant_id.clone(),
            company_id,
            position_id,
            "schema_v2".to_string(),
            BackfillRolloutScope::CurrentAndNew,
            "bf-loop-start".to_string(),
            "ONB_REQUIREMENT_BACKFILL_START_DRAFT",
            ReasonCodeId(0x4F00_0008),
        )
        .unwrap();

    let notify_first = s
        .ph1onb_requirement_backfill_notify_commit_row(
            MonotonicTimeNs(1301),
            started.campaign_id.clone(),
            tenant_id.clone(),
            target.clone(),
            "bf-loop-notify-1".to_string(),
            "ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT",
            ReasonCodeId(0x4F00_0009),
        )
        .unwrap();
    assert_eq!(notify_first.target_status, BackfillTargetStatus::Requested);

    let notify_replay = s
        .ph1onb_requirement_backfill_notify_commit_row(
            MonotonicTimeNs(1302),
            started.campaign_id.clone(),
            tenant_id.clone(),
            target.clone(),
            "bf-loop-notify-1".to_string(),
            "ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT",
            ReasonCodeId(0x4F00_0009),
        )
        .unwrap();
    assert_eq!(notify_replay.target_status, BackfillTargetStatus::Requested);

    let notify_reminder = s
        .ph1onb_requirement_backfill_notify_commit_row(
            MonotonicTimeNs(1303),
            started.campaign_id.clone(),
            tenant_id.clone(),
            target,
            "bf-loop-notify-2".to_string(),
            "ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT",
            ReasonCodeId(0x4F00_0009),
        )
        .unwrap();
    assert_eq!(
        notify_reminder.target_status,
        BackfillTargetStatus::Requested
    );

    let complete_first = s
        .ph1onb_requirement_backfill_complete_commit_row(
            MonotonicTimeNs(1304),
            started.campaign_id.clone(),
            tenant_id.clone(),
            "bf-loop-complete-1".to_string(),
            "ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT",
            ReasonCodeId(0x4F00_000A),
        )
        .unwrap();
    assert_eq!(complete_first.state, BackfillCampaignState::Completed);
    assert_eq!(complete_first.completed_target_count, 0);
    assert_eq!(
        complete_first.total_target_count,
        started.pending_target_count
    );

    let complete_replay = s
        .ph1onb_requirement_backfill_complete_commit_row(
            MonotonicTimeNs(1305),
            started.campaign_id,
            tenant_id,
            "bf-loop-complete-1".to_string(),
            "ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT",
            ReasonCodeId(0x4F00_000A),
        )
        .unwrap();
    assert_eq!(complete_replay.state, BackfillCampaignState::Completed);
    assert_eq!(
        complete_replay.completed_target_count,
        complete_first.completed_target_count
    );
    assert_eq!(
        complete_replay.total_target_count,
        complete_first.total_target_count
    );
}

#[test]
fn at_onb_db_11_backfill_fail_closed_on_tenant_scope_and_missing_target() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:user_inviter");
    let target = user("tenant_a:worker_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_inviter"));
    seed_identity_only(&mut s, target.clone());

    let (tenant_id, company_id, position_id) =
        seed_employee_position_schema_requiring_verification(&mut s, actor);

    let started = s
        .ph1onb_requirement_backfill_start_draft_row(
            MonotonicTimeNs(1400),
            user("tenant_a:user_inviter"),
            tenant_id.clone(),
            company_id,
            position_id,
            "schema_v2".to_string(),
            BackfillRolloutScope::CurrentAndNew,
            "bf-fail-closed-start".to_string(),
            "ONB_REQUIREMENT_BACKFILL_START_DRAFT",
            ReasonCodeId(0x4F00_0008),
        )
        .unwrap();

    let notify_wrong_tenant = s.ph1onb_requirement_backfill_notify_commit_row(
        MonotonicTimeNs(1401),
        started.campaign_id.clone(),
        "tenant_wrong".to_string(),
        target,
        "bf-fail-closed-notify-tenant".to_string(),
        "ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT",
        ReasonCodeId(0x4F00_0009),
    );
    assert!(matches!(
        notify_wrong_tenant,
        Err(StorageError::ContractViolation(_))
    ));

    let notify_missing_target = s.ph1onb_requirement_backfill_notify_commit_row(
        MonotonicTimeNs(1402),
        started.campaign_id.clone(),
        tenant_id.clone(),
        user("tenant_a:missing_target"),
        "bf-fail-closed-notify-missing-target".to_string(),
        "ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT",
        ReasonCodeId(0x4F00_0009),
    );
    assert!(matches!(
        notify_missing_target,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    let complete_wrong_tenant = s.ph1onb_requirement_backfill_complete_commit_row(
        MonotonicTimeNs(1403),
        started.campaign_id,
        "tenant_wrong".to_string(),
        "bf-fail-closed-complete-tenant".to_string(),
        "ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT",
        ReasonCodeId(0x4F00_000A),
    );
    assert!(matches!(
        complete_wrong_tenant,
        Err(StorageError::ContractViolation(_))
    ));
}
