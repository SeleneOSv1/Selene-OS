#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1simcat::{
    SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType, SimulationVersion,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::repo::SimulationCatalogTablesRepo;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id).unwrap()
}

fn sim(id: &str) -> SimulationId {
    SimulationId::new(id).unwrap()
}

fn ev(
    t: u64,
    tenant_id: TenantId,
    simulation_id: SimulationId,
    simulation_version: u32,
    simulation_type: SimulationType,
    status: SimulationStatus,
    idempotency_key: Option<&str>,
) -> SimulationCatalogEventInput {
    SimulationCatalogEventInput::v1(
        MonotonicTimeNs(t),
        tenant_id,
        simulation_id,
        SimulationVersion(simulation_version),
        simulation_type,
        status,
        "Link".to_string(),
        "reads_hash_v1".to_string(),
        "writes_hash_v1".to_string(),
        ReasonCodeId(0x5343_0001),
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_simcat_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let a = tenant("tenant_a");
    let b = tenant("tenant_b");
    let sim_id = sim("LINK_INVITE_SEND_COMMIT");

    s.append_simulation_catalog_row(ev(
        10,
        a.clone(),
        sim_id.clone(),
        1,
        SimulationType::Commit,
        SimulationStatus::Active,
        Some("idem_a"),
    ))
    .unwrap();
    s.append_simulation_catalog_row(ev(
        11,
        b.clone(),
        sim_id.clone(),
        1,
        SimulationType::Commit,
        SimulationStatus::Active,
        Some("idem_b"),
    ))
    .unwrap();

    let ra = s.simulation_catalog_current_row(&a, &sim_id).unwrap();
    let rb = s.simulation_catalog_current_row(&b, &sim_id).unwrap();
    assert_eq!(ra.tenant_id, a);
    assert_eq!(rb.tenant_id, b);
    assert_eq!(s.simulation_catalog_current_rows().len(), 2);
}

#[test]
fn at_simcat_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let event_id = s
        .append_simulation_catalog_row(ev(
            20,
            tenant("tenant_a"),
            sim("ONB_COMPLETE_COMMIT"),
            1,
            SimulationType::Commit,
            SimulationStatus::Draft,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_simulation_catalog_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_simcat_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let sim_id = sim("POSITION_SIM_004_ACTIVATE_COMMIT");

    let e1 = s
        .append_simulation_catalog_row(ev(
            30,
            t.clone(),
            sim_id.clone(),
            1,
            SimulationType::Commit,
            SimulationStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();
    let e2 = s
        .append_simulation_catalog_row(ev(
            31,
            t.clone(),
            sim_id.clone(),
            1,
            SimulationType::Commit,
            SimulationStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(e1, e2);
    assert_eq!(s.simulation_catalog_rows().len(), 1);
}

#[test]
fn at_simcat_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let sim_id = sim("LINK_INVITE_SEND_COMMIT");

    s.append_simulation_catalog_row(ev(
        40,
        t.clone(),
        sim_id.clone(),
        1,
        SimulationType::Commit,
        SimulationStatus::Draft,
        Some("idem_1"),
    ))
    .unwrap();
    s.append_simulation_catalog_row(ev(
        41,
        t.clone(),
        sim_id.clone(),
        2,
        SimulationType::Commit,
        SimulationStatus::Active,
        Some("idem_2"),
    ))
    .unwrap();

    let before = s.simulation_catalog_current_rows().clone();
    s.rebuild_simulation_catalog_current_rows().unwrap();
    let after = s.simulation_catalog_current_rows().clone();
    assert_eq!(before, after);

    let current = s.simulation_catalog_current_row(&t, &sim_id).unwrap();
    assert_eq!(current.simulation_version, SimulationVersion(2));
    assert_eq!(current.status, SimulationStatus::Active);
}
