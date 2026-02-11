#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1pbs::{
    BlueprintStatus, BlueprintVersion, IntentType, ProcessBlueprintEventInput, ProcessId,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::repo::PbsTablesRepo;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id).unwrap()
}

fn intent(v: &str) -> IntentType {
    IntentType::new(v).unwrap()
}

fn process(v: &str) -> ProcessId {
    ProcessId::new(v).unwrap()
}

fn ev(
    t: u64,
    tenant_id: TenantId,
    process_id: ProcessId,
    blueprint_version: u32,
    intent_type: IntentType,
    status: BlueprintStatus,
    idempotency_key: Option<&str>,
) -> ProcessBlueprintEventInput {
    ProcessBlueprintEventInput::v1(
        MonotonicTimeNs(t),
        tenant_id,
        process_id,
        BlueprintVersion(blueprint_version),
        intent_type,
        status,
        8,
        2,
        "sim_hash_v1".to_string(),
        ReasonCodeId(0x5042_0001),
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_pbs_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let a = tenant("tenant_a");
    let b = tenant("tenant_b");
    let i = intent("LINK_INVITE");

    s.append_process_blueprint_row(ev(
        10,
        a.clone(),
        process("LINK_INVITE_V1"),
        1,
        i.clone(),
        BlueprintStatus::Active,
        Some("idem_a"),
    ))
    .unwrap();
    s.append_process_blueprint_row(ev(
        11,
        b.clone(),
        process("LINK_INVITE_V1"),
        1,
        i.clone(),
        BlueprintStatus::Active,
        Some("idem_b"),
    ))
    .unwrap();

    let ra = s.blueprint_registry_row(&a, &i).unwrap();
    let rb = s.blueprint_registry_row(&b, &i).unwrap();
    assert_eq!(ra.tenant_id, a);
    assert_eq!(rb.tenant_id, b);
    assert_eq!(s.blueprint_registry_rows().len(), 2);
}

#[test]
fn at_pbs_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let event_id = s
        .append_process_blueprint_row(ev(
            20,
            tenant("tenant_a"),
            process("ONB_INVITED_V1"),
            1,
            intent("ONB_INVITED"),
            BlueprintStatus::Draft,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_process_blueprint_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_pbs_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let p = process("POSITION_MANAGE_V1");
    let i = intent("POSITION_MANAGE");

    let e1 = s
        .append_process_blueprint_row(ev(
            30,
            t.clone(),
            p.clone(),
            1,
            i.clone(),
            BlueprintStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();
    let e2 = s
        .append_process_blueprint_row(ev(
            31,
            t.clone(),
            p.clone(),
            1,
            i.clone(),
            BlueprintStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(e1, e2);
    assert_eq!(s.process_blueprint_rows().len(), 1);
}

#[test]
fn at_pbs_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let i = intent("LINK_INVITE");

    s.append_process_blueprint_row(ev(
        40,
        t.clone(),
        process("LINK_INVITE_V1"),
        1,
        i.clone(),
        BlueprintStatus::Active,
        Some("idem_1"),
    ))
    .unwrap();
    s.append_process_blueprint_row(ev(
        41,
        t.clone(),
        process("LINK_INVITE_V2"),
        2,
        i.clone(),
        BlueprintStatus::Active,
        Some("idem_2"),
    ))
    .unwrap();

    let before = s.blueprint_registry_rows().clone();
    s.rebuild_blueprint_registry_rows();
    let after = s.blueprint_registry_rows().clone();
    assert_eq!(before, after);

    let current = s.blueprint_registry_row(&t, &i).unwrap();
    assert_eq!(current.process_id, process("LINK_INVITE_V2"));
    assert_eq!(current.blueprint_version, BlueprintVersion(2));
    assert_eq!(current.status, BlueprintStatus::Active);
}
