#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1ecm::{
    AllowedCallers, CapabilityId, CapabilityMapStatus, CapabilityMapVersion,
    EngineCapabilityMapEventInput, EngineId, SideEffectsMode,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::repo::EngineCapabilityMapsTablesRepo;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id).unwrap()
}

fn engine(id: &str) -> EngineId {
    EngineId::new(id).unwrap()
}

fn capability(id: &str) -> CapabilityId {
    CapabilityId::new(id).unwrap()
}

fn ev(
    t: u64,
    tenant_id: TenantId,
    engine_id: EngineId,
    capability_id: CapabilityId,
    capability_map_version: u32,
    map_status: CapabilityMapStatus,
    idempotency_key: Option<&str>,
) -> EngineCapabilityMapEventInput {
    EngineCapabilityMapEventInput::v1(
        MonotonicTimeNs(t),
        tenant_id,
        engine_id,
        capability_id,
        CapabilityMapVersion(capability_map_version),
        map_status,
        "Governance".to_string(),
        "capability_name_v1".to_string(),
        AllowedCallers::OsAndSimulation,
        SideEffectsMode::Declared,
        "reads_hash_v1".to_string(),
        "writes_hash_v1".to_string(),
        ReasonCodeId(0x4543_0001),
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_ecm_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let a = tenant("tenant_a");
    let b = tenant("tenant_b");
    let engine_id = engine("PH1.LINK");
    let capability_id = capability("LINK_INVITE_SEND_COMMIT");

    s.append_engine_capability_map_row(ev(
        10,
        a.clone(),
        engine_id.clone(),
        capability_id.clone(),
        1,
        CapabilityMapStatus::Active,
        Some("idem_a"),
    ))
    .unwrap();
    s.append_engine_capability_map_row(ev(
        11,
        b.clone(),
        engine_id.clone(),
        capability_id.clone(),
        1,
        CapabilityMapStatus::Active,
        Some("idem_b"),
    ))
    .unwrap();

    let ra = s
        .engine_capability_maps_current_row(&a, &engine_id, &capability_id)
        .unwrap();
    let rb = s
        .engine_capability_maps_current_row(&b, &engine_id, &capability_id)
        .unwrap();
    assert_eq!(ra.tenant_id, a);
    assert_eq!(rb.tenant_id, b);
    assert_eq!(s.engine_capability_maps_current_rows().len(), 2);
}

#[test]
fn at_ecm_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let event_id = s
        .append_engine_capability_map_row(ev(
            20,
            tenant("tenant_a"),
            engine("PH1.X"),
            capability("X_DISPATCH"),
            1,
            CapabilityMapStatus::Draft,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_engine_capability_map_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_ecm_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let e = engine("PH1.E");
    let c = capability("E_WEB_SEARCH");

    let e1 = s
        .append_engine_capability_map_row(ev(
            30,
            t.clone(),
            e.clone(),
            c.clone(),
            1,
            CapabilityMapStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();
    let e2 = s
        .append_engine_capability_map_row(ev(
            31,
            t.clone(),
            e.clone(),
            c.clone(),
            1,
            CapabilityMapStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(e1, e2);
    assert_eq!(s.engine_capability_map_rows().len(), 1);
}

#[test]
fn at_ecm_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let e = engine("PH1.LINK");
    let c = capability("LINK_INVITE_SEND_COMMIT");

    s.append_engine_capability_map_row(ev(
        40,
        t.clone(),
        e.clone(),
        c.clone(),
        1,
        CapabilityMapStatus::Draft,
        Some("idem_1"),
    ))
    .unwrap();
    s.append_engine_capability_map_row(ev(
        41,
        t.clone(),
        e.clone(),
        c.clone(),
        2,
        CapabilityMapStatus::Active,
        Some("idem_2"),
    ))
    .unwrap();

    let before = s.engine_capability_maps_current_rows().clone();
    s.rebuild_engine_capability_maps_current_rows();
    let after = s.engine_capability_maps_current_rows().clone();
    assert_eq!(before, after);

    let current = s.engine_capability_maps_current_row(&t, &e, &c).unwrap();
    assert_eq!(current.capability_map_version, CapabilityMapVersion(2));
    assert_eq!(current.map_status, CapabilityMapStatus::Active);
}
