#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1position::{
    PositionLifecycleState, PositionScheduleType, TenantId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord,
};
use selene_storage::repo::{Ph1PositionRepo, Ph1fFoundationRepo};

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
