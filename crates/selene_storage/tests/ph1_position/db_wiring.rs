#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1link::{InviteeType, PrefilledContext};
use selene_kernel_contracts::ph1onb::{ProofType, SenderVerifyDecision};
use selene_kernel_contracts::ph1position::{
    PositionLifecycleState, PositionRequirementEvidenceMode, PositionRequirementExposureRule,
    PositionRequirementFieldSpec, PositionRequirementFieldType, PositionRequirementRuleType,
    PositionRequirementSensitivity, PositionScheduleType, PositionSchemaApplyScope,
    PositionSchemaSelectorSnapshot, TenantId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord,
};
use selene_storage::repo::{Ph1LinkRepo, Ph1OnbRepo, Ph1PositionRepo, Ph1fFoundationRepo};

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
        industry_code: Some("RETAIL".to_string()),
        jurisdiction: Some("US".to_string()),
        position_family: Some("WAREHOUSE".to_string()),
    }
}

fn required_field(field_key: &str) -> PositionRequirementFieldSpec {
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

#[test]
fn at_position_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    seed_identity_device(&mut s, user_a.clone(), device("tenant_a_device_1"));
    seed_identity_device(&mut s, user_b.clone(), device("tenant_b_device_1"));

    let tenant_a = TenantId::new("tenant_a").unwrap();
    let tenant_b = TenantId::new("tenant_b").unwrap();
    seed_company(&mut s, &tenant_a, "company_a");
    seed_company(&mut s, &tenant_b, "company_b");

    let pos_a = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(100),
            user_a,
            tenant_a.clone(),
            "company_a".to_string(),
            "Manager".to_string(),
            "Ops".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_manager".to_string(),
            "band_l3".to_string(),
            "position-create-tenant-a".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    assert!(s.ph1position_row(&tenant_a, &pos_a.position_id).is_some());
    assert!(s.ph1position_row(&tenant_b, &pos_a.position_id).is_none());

    let cross_tenant_activate = s.ph1position_activate_commit_row(
        MonotonicTimeNs(101),
        user_b,
        tenant_b.clone(),
        pos_a.position_id.clone(),
        "position-activate-cross-tenant".to_string(),
        "POSITION_SIM_004_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0004),
    );
    assert!(matches!(
        cross_tenant_activate,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    assert_eq!(
        s.ph1position_lifecycle_rows_for_position(&tenant_a, &pos_a.position_id)
            .len(),
        1
    );
    assert_eq!(
        s.ph1position_lifecycle_rows_for_position(&tenant_b, &pos_a.position_id)
            .len(),
        0
    );
}

#[test]
fn at_position_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let actor = user("tenant_a:user_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_1"));
    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let pos = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(200),
            actor,
            tenant.clone(),
            "company_a".to_string(),
            "Supervisor".to_string(),
            "Retail".to_string(),
            "US".to_string(),
            PositionScheduleType::PartTime,
            "profile_supervisor".to_string(),
            "band_l2".to_string(),
            "position-append-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    let events = s.ph1position_lifecycle_rows_for_position(&tenant, &pos.position_id);
    assert_eq!(events.len(), 1);
    let event_id = events[0].event_id;

    assert!(matches!(
        s.attempt_overwrite_position_lifecycle_event_row(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_position_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let actor = user("tenant_a:user_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_1"));
    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let first = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(300),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Cashier".to_string(),
            "Retail".to_string(),
            "US".to_string(),
            PositionScheduleType::Shift,
            "profile_cashier".to_string(),
            "band_l1".to_string(),
            "position-idem-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();
    let second = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(301),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Cashier".to_string(),
            "Retail".to_string(),
            "US".to_string(),
            PositionScheduleType::Shift,
            "profile_cashier".to_string(),
            "band_l1".to_string(),
            "position-idem-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();
    assert_eq!(first.position_id, second.position_id);

    let activated_first = s
        .ph1position_activate_commit_row(
            MonotonicTimeNs(302),
            actor.clone(),
            tenant.clone(),
            first.position_id.clone(),
            "position-idem-activate".to_string(),
            "POSITION_SIM_004_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0004),
        )
        .unwrap();
    let activated_second = s
        .ph1position_activate_commit_row(
            MonotonicTimeNs(303),
            actor,
            tenant.clone(),
            first.position_id.clone(),
            "position-idem-activate".to_string(),
            "POSITION_SIM_004_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0004),
        )
        .unwrap();
    assert_eq!(
        activated_first.lifecycle_state,
        PositionLifecycleState::Active
    );
    assert_eq!(
        activated_second.lifecycle_state,
        PositionLifecycleState::Active
    );

    // Exactly one create + one activate event despite retries.
    assert_eq!(
        s.ph1position_lifecycle_rows_for_position(&tenant, &first.position_id)
            .len(),
        2
    );
}

#[test]
fn at_position_db_04_current_table_consistency_with_lifecycle_ledger() {
    let mut s = Ph1fStore::new_in_memory();

    let actor = user("tenant_a:user_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_1"));
    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let draft = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(400),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Stock Lead".to_string(),
            "Warehouse".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_stock_lead".to_string(),
            "band_l3".to_string(),
            "position-flow-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();
    s.ph1position_activate_commit_row(
        MonotonicTimeNs(401),
        actor.clone(),
        tenant.clone(),
        draft.position_id.clone(),
        "position-flow-activate".to_string(),
        "POSITION_SIM_004_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0004),
    )
    .unwrap();
    let suspended = s
        .ph1position_retire_or_suspend_commit_row(
            MonotonicTimeNs(402),
            actor,
            tenant.clone(),
            draft.position_id.clone(),
            PositionLifecycleState::Suspended,
            "position-flow-suspend".to_string(),
            "POSITION_SIM_005_RETIRE_OR_SUSPEND_COMMIT",
            ReasonCodeId(0x5900_0005),
        )
        .unwrap();

    let current = s.ph1position_row(&tenant, &draft.position_id).unwrap();
    assert_eq!(current.lifecycle_state, PositionLifecycleState::Suspended);
    assert_eq!(suspended.lifecycle_state, PositionLifecycleState::Suspended);

    let events = s.ph1position_lifecycle_rows_for_position(&tenant, &draft.position_id);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events.last().map(|e| e.to_state),
        Some(PositionLifecycleState::Suspended)
    );
}

#[test]
fn at_position_db_05_requirements_schema_activation_monotonic() {
    let mut s = Ph1fStore::new_in_memory();

    let actor = user("tenant_a:user_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_1"));
    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let draft = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(500),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Driver".to_string(),
            "Logistics".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_driver".to_string(),
            "band_l2".to_string(),
            "position-schema-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    s.ph1position_activate_commit_row(
        MonotonicTimeNs(501),
        actor.clone(),
        tenant.clone(),
        draft.position_id.clone(),
        "position-schema-activate".to_string(),
        "POSITION_SIM_004_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0004),
    )
    .unwrap();

    let missing_schema = s.ph1position_requirements_schema_activate_commit_row(
        MonotonicTimeNs(502),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        draft.position_id.clone(),
        "schema_v_missing".to_string(),
        PositionSchemaApplyScope::NewHiresOnly,
        "schema-activate-missing".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0008),
    );
    assert!(matches!(
        missing_schema,
        Err(StorageError::ForeignKeyViolation {
            table: "position_requirements_schema_ledger.schema_version_id",
            ..
        })
    ));

    let draft_v1 = s
        .ph1position_requirements_schema_create_draft_row(
            MonotonicTimeNs(503),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            draft.position_id.clone(),
            "schema_v1".to_string(),
            selector_snapshot(),
            vec![required_field("sender_verification")],
            "schema-v1-create".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
            ReasonCodeId(0x5900_0006),
        )
        .unwrap();
    assert_eq!(draft_v1.field_count, 1);

    let activated_v1 = s
        .ph1position_requirements_schema_activate_commit_row(
            MonotonicTimeNs(504),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            draft.position_id.clone(),
            "schema_v1".to_string(),
            PositionSchemaApplyScope::NewHiresOnly,
            "schema-v1-activate".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0008),
        )
        .unwrap();
    assert_eq!(
        activated_v1.apply_scope_result,
        PositionSchemaApplyScope::NewHiresOnly
    );
    assert!(!activated_v1.backfill_handoff_required);

    let replay_same_idempotency = s
        .ph1position_requirements_schema_activate_commit_row(
            MonotonicTimeNs(505),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            draft.position_id.clone(),
            "schema_v1".to_string(),
            PositionSchemaApplyScope::CurrentAndNew,
            "schema-v1-activate".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0008),
        )
        .unwrap();
    assert_eq!(
        replay_same_idempotency.apply_scope_result,
        PositionSchemaApplyScope::NewHiresOnly
    );
    assert!(!replay_same_idempotency.backfill_handoff_required);

    s.ph1position_requirements_schema_create_draft_row(
        MonotonicTimeNs(506),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        draft.position_id.clone(),
        "schema_v2".to_string(),
        selector_snapshot(),
        vec![required_field("sender_verification"), required_field("employee_photo")],
        "schema-v2-create".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
        ReasonCodeId(0x5900_0006),
    )
    .unwrap();

    let activated_v2 = s
        .ph1position_requirements_schema_activate_commit_row(
            MonotonicTimeNs(507),
            actor,
            tenant,
            "company_a".to_string(),
            draft.position_id,
            "schema_v2".to_string(),
            PositionSchemaApplyScope::CurrentAndNew,
            "schema-v2-activate".to_string(),
            "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
            ReasonCodeId(0x5900_0008),
        )
        .unwrap();
    assert_eq!(
        activated_v2.apply_scope_result,
        PositionSchemaApplyScope::CurrentAndNew
    );
    assert!(activated_v2.backfill_handoff_required);
}

#[test]
fn at_position_db_06_onb_read_only_schema_boundary() {
    let mut s = Ph1fStore::new_in_memory();

    let actor = user("tenant_a:user_1");
    let device_a = device("tenant_a_device_1");
    seed_identity_device(&mut s, actor.clone(), device_a.clone());
    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let draft = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(600),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Driver".to_string(),
            "Logistics".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_driver".to_string(),
            "band_l2".to_string(),
            "position-onb-create".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    s.ph1position_activate_commit_row(
        MonotonicTimeNs(601),
        actor.clone(),
        tenant.clone(),
        draft.position_id.clone(),
        "position-onb-activate".to_string(),
        "POSITION_SIM_004_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0004),
    )
    .unwrap();

    s.ph1position_requirements_schema_create_draft_row(
        MonotonicTimeNs(602),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        draft.position_id.clone(),
        "schema_v1".to_string(),
        selector_snapshot(),
        vec![required_field("sender_verification")],
        "schema-onb-create".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
        ReasonCodeId(0x5900_0006),
    )
    .unwrap();
    s.ph1position_requirements_schema_activate_commit_row(
        MonotonicTimeNs(603),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        draft.position_id.clone(),
        "schema_v1".to_string(),
        PositionSchemaApplyScope::NewHiresOnly,
        "schema-onb-activate".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0008),
    )
    .unwrap();

    let prefilled = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        Some(draft.position_id.as_str().to_string()),
        Some("loc_1".to_string()),
        Some("2026-02-15".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let (link_1, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(604),
            actor.clone(),
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            Some(prefilled.clone()),
            None,
        )
        .unwrap();
    s.ph1link_invite_open_activate_commit_row(
        MonotonicTimeNs(605),
        link_1.token_id.clone(),
        "fp_employee_1".to_string(),
    )
    .unwrap();

    let session_1 = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(606),
            link_1.token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_1".to_string(),
        )
        .unwrap();
    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(607),
        session_1.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-terms-1".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(608),
        session_1.onboarding_session_id.clone(),
        device_a.clone(),
        ProofType::Passcode,
        true,
        "onb-device-1".to_string(),
    )
    .unwrap();

    let access_without_verification = s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(609),
        session_1.onboarding_session_id.clone(),
        user("tenant_a:hire_1"),
        Some("tenant_a".to_string()),
        "employee".to_string(),
        "onb-access-1-fail".to_string(),
    );
    assert!(matches!(
        access_without_verification,
        Err(StorageError::ContractViolation(_))
    ));

    s.ph1onb_employee_photo_capture_send_commit_row(
        MonotonicTimeNs(610),
        session_1.onboarding_session_id.clone(),
        "blob://photo_1".to_string(),
        actor.clone(),
        "onb-photo-1".to_string(),
    )
    .unwrap();
    s.ph1onb_employee_sender_verify_commit_row(
        MonotonicTimeNs(611),
        session_1.onboarding_session_id.clone(),
        actor.clone(),
        SenderVerifyDecision::Confirm,
        "onb-verify-1".to_string(),
    )
    .unwrap();

    let access_after_verification = s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(612),
        session_1.onboarding_session_id,
        user("tenant_a:hire_1"),
        Some("tenant_a".to_string()),
        "employee".to_string(),
        "onb-access-1-ok".to_string(),
    );
    assert!(access_after_verification.is_ok());

    // Start a fresh onboarding session for the same position to prove ONB did not mutate
    // requirements schema truth.
    let prefilled_second = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        Some(draft.position_id.as_str().to_string()),
        Some("loc_1".to_string()),
        Some("2026-03-01".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let (link_2, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(613),
            actor.clone(),
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            Some(prefilled_second),
            None,
        )
        .unwrap();
    s.ph1link_invite_open_activate_commit_row(
        MonotonicTimeNs(614),
        link_2.token_id.clone(),
        "fp_employee_2".to_string(),
    )
    .unwrap();

    let session_2 = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(615),
            link_2.token_id,
            None,
            Some("tenant_a".to_string()),
            "fp_onb_2".to_string(),
        )
        .unwrap();
    s.ph1onb_terms_accept_commit_row(
        MonotonicTimeNs(616),
        session_2.onboarding_session_id.clone(),
        "terms_v1".to_string(),
        true,
        "onb-terms-2".to_string(),
    )
    .unwrap();
    s.ph1onb_primary_device_confirm_commit_row(
        MonotonicTimeNs(617),
        session_2.onboarding_session_id.clone(),
        device_a,
        ProofType::Passcode,
        true,
        "onb-device-2".to_string(),
    )
    .unwrap();

    let access_session_2_without_verification = s.ph1onb_access_instance_create_commit_row(
        MonotonicTimeNs(618),
        session_2.onboarding_session_id,
        user("tenant_a:hire_2"),
        Some("tenant_a".to_string()),
        "employee".to_string(),
        "onb-access-2-fail".to_string(),
    );
    assert!(matches!(
        access_session_2_without_verification,
        Err(StorageError::ContractViolation(_))
    ));
}
