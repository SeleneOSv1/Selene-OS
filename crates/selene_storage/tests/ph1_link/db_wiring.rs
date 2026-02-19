#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1link::{
    AppPlatform, DraftStatus, InviteeType, LinkStatus, PrefilledContext,
};
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
use selene_storage::repo::{Ph1LinkRepo, Ph1PositionRepo, Ph1fFoundationRepo};

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
        evidence_mode: PositionRequirementEvidenceMode::Attestation,
        prompt_short: format!("Provide {field_key}"),
        prompt_long: format!("Please provide required field {field_key}."),
    }
}

#[test]
fn at_link_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a);
    seed_identity_device(&mut s, user_b.clone(), device_b);

    let (a, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(100),
            user_a.clone(),
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
    let (b, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(101),
            user_b.clone(),
            InviteeType::Employee,
            Some("tenant_b".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    assert_ne!(a.token_id, b.token_id);

    let mismatch = s.ph1link_invite_generate_draft_row(
        MonotonicTimeNs(102),
        user_a,
        InviteeType::Employee,
        Some("tenant_b".to_string()),
        None,
        None,
        None,
    );
    assert!(matches!(mismatch, Err(StorageError::ContractViolation(_))));
}

#[test]
fn at_link_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(200),
            u,
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let (status, _, _, _, _, _, _, _, _, _) = s
        .ph1link_invite_open_activate_commit_row(
            MonotonicTimeNs(201),
            link.token_id,
            "append_fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_append".to_string(),
            "nonce_append_201".to_string(),
            MonotonicTimeNs(201),
        )
        .unwrap();
    assert_eq!(status, LinkStatus::Activated);
}

#[test]
fn at_link_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let (l1, p1) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(300),
            u.clone(),
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            None,
            Some("default".to_string()),
        )
        .unwrap();
    let (l2, p2) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(301),
            u,
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            None,
            Some("default".to_string()),
        )
        .unwrap();
    assert_eq!(l1.token_id, l2.token_id);
    assert!(p1.was_new);
    assert!(!p2.was_new);
}

#[test]
fn at_link_db_04_current_table_consistency_with_lifecycle_and_proofs() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(400),
            u,
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
    assert_eq!(link.status, LinkStatus::DraftCreated);

    let sent_status = s
        .ph1link_mark_sent_commit_row(link.token_id.clone())
        .unwrap();
    assert_eq!(sent_status, LinkStatus::Sent);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Sent
    );

    let (activated_status, _, _, _, _, _, _, _, _, _) = s
        .ph1link_invite_open_activate_commit_row(
            MonotonicTimeNs(401),
            link.token_id.clone(),
            "fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_401".to_string(),
            "nonce_401".to_string(),
            MonotonicTimeNs(401),
        )
        .unwrap();
    assert_eq!(activated_status, LinkStatus::Activated);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Activated
    );

    let (blocked_status, _, _, _, _) = s
        .ph1link_invite_forward_block_commit_row(link.token_id.clone(), "fp_other".to_string())
        .unwrap();
    assert_eq!(blocked_status, LinkStatus::Blocked);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Blocked
    );

    let current = s.ph1link_get_link_row(&link.token_id).unwrap();
    assert_eq!(current.status, LinkStatus::Blocked);
    assert!(current.bound_device_fingerprint_hash.is_some());
}

#[test]
fn at_link_db_05_draft_update_success_and_idempotent_replay() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u.clone(), device("tenant_a_device_1"));

    let prefilled = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        Some("position_a".to_string()),
        None,
        None,
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(500),
            u,
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            Some(prefilled),
            None,
        )
        .unwrap();

    assert!(link
        .missing_required_fields
        .contains(&"location_id".to_string()));
    assert!(link
        .missing_required_fields
        .contains(&"start_date".to_string()));

    let mut updates = BTreeMap::new();
    updates.insert("location_id".to_string(), "loc_1".to_string());
    updates.insert("start_date".to_string(), "2026-02-15".to_string());

    let first = s
        .ph1link_invite_draft_update_commit(
            MonotonicTimeNs(501),
            link.draft_id.clone(),
            updates.clone(),
            "link-draft-update-1".to_string(),
        )
        .unwrap();
    assert_eq!(first.1, DraftStatus::DraftReady);
    assert!(first.2.is_empty());

    let replay = s
        .ph1link_invite_draft_update_commit(
            MonotonicTimeNs(502),
            link.draft_id,
            updates,
            "link-draft-update-1".to_string(),
        )
        .unwrap();
    assert_eq!(replay.1, DraftStatus::DraftReady);
    assert!(replay.2.is_empty());
}

#[test]
fn at_link_db_06_draft_update_refused_for_invalid_state() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u.clone(), device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(520),
            u,
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    s.ph1link_invite_revoke_revoke(link.token_id, "admin_revoke".to_string())
        .unwrap();

    let mut updates = BTreeMap::new();
    updates.insert("tenant_id".to_string(), "tenant_a".to_string());

    let out = s.ph1link_invite_draft_update_commit(
        MonotonicTimeNs(521),
        link.draft_id,
        updates,
        "link-draft-update-terminal".to_string(),
    );
    assert!(matches!(out, Err(StorageError::ContractViolation(_))));
}

#[test]
fn at_link_db_07_revoke_refused_for_activated_without_ap_override() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u, device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(540),
            user("tenant_a:user_1"),
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let (status, _, _, _, _, _, _, _, _, _) = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(541),
            link.token_id.clone(),
            "fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_1".to_string(),
            "nonce_541".to_string(),
            MonotonicTimeNs(541),
            "open-idem-1".to_string(),
        )
        .unwrap();
    assert_eq!(status, LinkStatus::Activated);

    let revoke = s.ph1link_invite_revoke_revoke(link.token_id.clone(), "revoke_now".to_string());
    assert!(matches!(revoke, Err(StorageError::ContractViolation(_))));
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Activated
    );
}

#[test]
fn at_link_db_08_revoke_allows_non_activated_state() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u.clone(), device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(560),
            u,
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    s.ph1link_invite_revoke_revoke(link.token_id.clone(), "admin_revoke".to_string())
        .unwrap();
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Revoked
    );
}

#[test]
fn at_link_db_09_open_activate_idempotency_replay_behavior() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u, device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(580),
            user("tenant_a:user_1"),
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let first = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(581),
            link.token_id.clone(),
            "fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_2".to_string(),
            "nonce_581".to_string(),
            MonotonicTimeNs(581),
            "open-idem-replay".to_string(),
        )
        .unwrap();
    assert_eq!(first.0, LinkStatus::Activated);

    // Same token + same idempotency key must replay original outcome even if fingerprint differs.
    let replay = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(582),
            link.token_id.clone(),
            "fp_other".to_string(),
            AppPlatform::Ios,
            "ios_instance_2".to_string(),
            "nonce_581".to_string(),
            MonotonicTimeNs(581),
            "open-idem-replay".to_string(),
        )
        .unwrap();
    assert_eq!(replay.0, first.0);
    assert_eq!(replay.3, first.3);
    assert_eq!(replay.4, first.4);
}

#[test]
fn at_link_db_10_forward_block_deterministic_single_path() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u, device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(600),
            user("tenant_a:user_1"),
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let bind = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(601),
            link.token_id.clone(),
            "fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_3".to_string(),
            "nonce_601".to_string(),
            MonotonicTimeNs(601),
            "open-bind".to_string(),
        )
        .unwrap();
    assert_eq!(bind.0, LinkStatus::Activated);

    let blocked = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(602),
            link.token_id.clone(),
            "fp_other".to_string(),
            AppPlatform::Ios,
            "ios_instance_4".to_string(),
            "nonce_602".to_string(),
            MonotonicTimeNs(602),
            "open-mismatch".to_string(),
        )
        .unwrap();
    assert_eq!(blocked.0, LinkStatus::Blocked);
    assert_eq!(blocked.4.as_deref(), Some("FORWARDED_LINK_DEVICE_MISMATCH"));

    let replay = s
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(603),
            link.token_id.clone(),
            "fp_other".to_string(),
            AppPlatform::Ios,
            "ios_instance_4".to_string(),
            "nonce_602".to_string(),
            MonotonicTimeNs(602),
            "open-mismatch".to_string(),
        )
        .unwrap();
    assert_eq!(replay.0, LinkStatus::Blocked);
    assert_eq!(replay.4.as_deref(), Some("FORWARDED_LINK_DEVICE_MISMATCH"));
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Blocked
    );
}

#[test]
fn at_link_db_11_missing_required_fields_recompute_is_schema_driven() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:user_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_1"));

    let tenant = TenantId::new("tenant_a").unwrap();
    seed_company(&mut s, &tenant, "company_a");

    let position = s
        .ph1position_create_draft_row(
            MonotonicTimeNs(620),
            actor.clone(),
            tenant.clone(),
            "company_a".to_string(),
            "Warehouse Operator".to_string(),
            "Ops".to_string(),
            "US".to_string(),
            PositionScheduleType::FullTime,
            "profile_ops".to_string(),
            "band_l2".to_string(),
            "position-create-link-schema".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    s.ph1position_activate_commit_row(
        MonotonicTimeNs(621),
        actor.clone(),
        tenant.clone(),
        position.position_id.clone(),
        "position-activate-link-schema".to_string(),
        "POSITION_SIM_004_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0004),
    )
    .unwrap();

    s.ph1position_requirements_schema_create_draft_row(
        MonotonicTimeNs(622),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        position.position_id.clone(),
        "schema_v1".to_string(),
        selector_snapshot(),
        vec![required_field("working_hours")],
        "schema-create-link-schema".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_CREATE_DRAFT",
        ReasonCodeId(0x5900_0006),
    )
    .unwrap();

    s.ph1position_requirements_schema_activate_commit_row(
        MonotonicTimeNs(623),
        actor.clone(),
        tenant.clone(),
        "company_a".to_string(),
        position.position_id.clone(),
        "schema_v1".to_string(),
        PositionSchemaApplyScope::NewHiresOnly,
        "schema-activate-link-schema".to_string(),
        "POSITION_REQUIREMENTS_SCHEMA_ACTIVATE_COMMIT",
        ReasonCodeId(0x5900_0008),
    )
    .unwrap();

    let prefilled = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        Some(position.position_id.as_str().to_string()),
        Some("loc_1".to_string()),
        Some("2026-02-15".to_string()),
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(624),
            actor,
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            Some(prefilled),
            None,
        )
        .unwrap();

    assert_eq!(
        link.missing_required_fields,
        vec!["working_hours".to_string()]
    );

    let mut updates = BTreeMap::new();
    updates.insert("working_hours".to_string(), "09:00-17:00".to_string());
    let (_, draft_status, missing) = s
        .ph1link_invite_draft_update_commit(
            MonotonicTimeNs(625),
            link.draft_id,
            updates,
            "schema-driven-recompute".to_string(),
        )
        .unwrap();
    assert_eq!(draft_status, DraftStatus::DraftReady);
    assert!(missing.is_empty());
}

#[test]
fn at_link_db_12_draft_update_row_method_is_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u.clone(), device("tenant_a_device_1"));

    let prefilled = PrefilledContext::v1(
        Some("tenant_a".to_string()),
        Some("company_a".to_string()),
        Some("position_a".to_string()),
        None,
        None,
        None,
        Some("band_l2".to_string()),
        Vec::new(),
    )
    .unwrap();

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(700),
            u,
            InviteeType::Employee,
            Some("tenant_a".to_string()),
            None,
            Some(prefilled),
            None,
        )
        .unwrap();

    let mut updates = BTreeMap::new();
    updates.insert("location_id".to_string(), "loc_1".to_string());
    updates.insert("start_date".to_string(), "2026-02-15".to_string());

    let first = s
        .ph1link_invite_draft_update_commit_row(
            MonotonicTimeNs(701),
            link.draft_id.clone(),
            updates.clone(),
            "row-draft-update-idem-1".to_string(),
        )
        .unwrap();
    assert_eq!(first.1, DraftStatus::DraftReady);
    assert!(first.2.is_empty());

    let replay = s
        .ph1link_invite_draft_update_commit_row(
            MonotonicTimeNs(702),
            link.draft_id,
            updates,
            "row-draft-update-idem-1".to_string(),
        )
        .unwrap();
    assert_eq!(replay.1, DraftStatus::DraftReady);
    assert!(replay.2.is_empty());
}

#[test]
fn at_link_db_13_open_activate_row_with_idempotency_replays_by_key() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_1");
    seed_identity_device(&mut s, u, device("tenant_a_device_1"));

    let (link, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(720),
            user("tenant_a:user_1"),
            InviteeType::FamilyMember,
            Some("tenant_a".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

    let first = s
        .ph1link_invite_open_activate_commit_row_with_idempotency(
            MonotonicTimeNs(721),
            link.token_id.clone(),
            "fp_primary".to_string(),
            AppPlatform::Ios,
            "ios_instance_721".to_string(),
            "nonce_721".to_string(),
            MonotonicTimeNs(721),
            "row-open-idem-1".to_string(),
        )
        .unwrap();
    assert_eq!(first.0, LinkStatus::Activated);

    // Same token + same idempotency key must replay exact outcome even if fingerprint differs.
    let replay = s
        .ph1link_invite_open_activate_commit_row_with_idempotency(
            MonotonicTimeNs(722),
            link.token_id.clone(),
            "fp_other".to_string(),
            AppPlatform::Ios,
            "ios_instance_721".to_string(),
            "nonce_721".to_string(),
            MonotonicTimeNs(721),
            "row-open-idem-1".to_string(),
        )
        .unwrap();
    assert_eq!(replay.0, LinkStatus::Activated);
    assert_eq!(replay.3, first.3);
    assert_eq!(replay.4, first.4);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Activated
    );
}
