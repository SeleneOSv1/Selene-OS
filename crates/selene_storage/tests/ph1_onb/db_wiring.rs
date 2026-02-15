#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1link::{InviteeType, LinkStatus, PrefilledContext};
use selene_kernel_contracts::ph1onb::{OnboardingStatus, ProofType, TermsStatus};
use selene_kernel_contracts::ph1position::{
    PositionRequirementEvidenceMode, PositionRequirementExposureRule, PositionRequirementFieldSpec,
    PositionRequirementFieldType, PositionRequirementRuleType, PositionRequirementSensitivity,
    PositionScheduleType, PositionSchemaApplyScope, PositionSchemaSelectorSnapshot, TenantId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord,
};
use selene_storage::repo::{Ph1LinkRepo, Ph1OnbRepo, Ph1PositionRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

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

#[allow(clippy::too_many_arguments)]
fn seed_activated_link(
    store: &mut Ph1fStore,
    now: u64,
    inviter_user_id: UserId,
    invitee_type: InviteeType,
    tenant_id: Option<String>,
    prefilled_context: Option<PrefilledContext>,
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
        )
        .unwrap();

    link.token_id
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
    )
    .unwrap();
    s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(301),
        link_b,
        None,
        Some("tenant_b".to_string()),
        "fp_onb_b".to_string(),
    )
    .unwrap();

    let mismatch = s.ph1onb_session_start_draft_row(
        MonotonicTimeNs(302),
        link_a,
        None,
        Some("tenant_b".to_string()),
        "fp_onb_mismatch".to_string(),
    );
    assert!(matches!(mismatch, Err(StorageError::ContractViolation(_))));
    assert_eq!(s.ph1onb_session_rows().len(), 2);
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
        d,
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

    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(607),
            started.onboarding_session_id.clone(),
            "onb-flow-complete".to_string(),
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
        )
        .unwrap();
    let expected_schema_id = format!("position:{position_id}");
    assert_eq!(started.pinned_schema_id.as_deref(), Some(expected_schema_id.as_str()));
    assert_eq!(started.pinned_schema_version.as_deref(), Some("schema_v1"));
    assert_eq!(
        started.pinned_overlay_set_id.as_deref(),
        Some("position_requirements_active")
    );
    assert!(
        started
            .pinned_selector_snapshot_ref
            .as_deref()
            .unwrap_or("")
            .starts_with(&format!("selector:{tenant_id}:{position_id}:"))
    );
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
    assert_eq!(current.pinned_schema_id.as_deref(), Some(expected_schema_id.as_str()));
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
        inviter_device,
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
    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(909),
            started.onboarding_session_id,
            "onb-required-complete-ok".to_string(),
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
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
        inviter_device,
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
    let completed = s
        .ph1onb_complete_commit_row(
            MonotonicTimeNs(1007),
            started.onboarding_session_id,
            "onb-no-required-complete".to_string(),
        )
        .unwrap();
    assert_eq!(completed.onboarding_status, OnboardingStatus::Complete);
}
