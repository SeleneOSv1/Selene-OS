#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1work::{WorkOrderId, WorkOrderLedgerEventInput, WorkOrderStatus};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use selene_storage::repo::SeleneOsWorkOrderRepo;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id).unwrap()
}

fn work_order(id: &str) -> WorkOrderId {
    WorkOrderId::new(id).unwrap()
}

fn ev(
    t: u64,
    tenant_id: TenantId,
    work_order_id: WorkOrderId,
    status: WorkOrderStatus,
    reason_code: u32,
    idempotency_key: Option<&str>,
) -> WorkOrderLedgerEventInput {
    WorkOrderLedgerEventInput::v1(
        MonotonicTimeNs(t),
        tenant_id,
        work_order_id,
        CorrelationId(500),
        TurnId(1),
        status,
        ReasonCodeId(reason_code),
        Some("step_dispatch".to_string()),
        Some("input_hash_1".to_string()),
        None,
        None,
        None,
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_os_core_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let t1 = tenant("tenant_a");
    let t2 = tenant("tenant_b");
    let wo = work_order("wo_1");

    s.append_work_order_ledger_row(ev(
        10,
        t1.clone(),
        wo.clone(),
        WorkOrderStatus::Draft,
        0x2300_0001,
        Some("idem_t1"),
    ))
    .unwrap();

    s.append_work_order_ledger_row(ev(
        11,
        t2.clone(),
        wo.clone(),
        WorkOrderStatus::Draft,
        0x2300_0002,
        Some("idem_t2"),
    ))
    .unwrap();

    assert!(s.work_order_current_row(&t1, &wo).is_some());
    assert!(s.work_order_current_row(&t2, &wo).is_some());
    assert_eq!(s.work_orders_current_rows().len(), 2);
}

#[test]
fn at_os_core_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let id = s
        .append_work_order_ledger_row(ev(
            20,
            tenant("tenant_a"),
            work_order("wo_2"),
            WorkOrderStatus::Draft,
            0x2300_0010,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_work_order_ledger_event(id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_os_core_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let wo = work_order("wo_3");

    let e1 = s
        .append_work_order_ledger_row(ev(
            30,
            t.clone(),
            wo.clone(),
            WorkOrderStatus::Executing,
            0x2300_0020,
            Some("idem_same"),
        ))
        .unwrap();
    let e2 = s
        .append_work_order_ledger_row(ev(
            31,
            t.clone(),
            wo.clone(),
            WorkOrderStatus::Executing,
            0x2300_0020,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(e1, e2);
    assert_eq!(s.work_order_ledger_rows().len(), 1);
}

#[test]
fn at_os_core_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let wo = work_order("wo_4");

    s.append_work_order_ledger_row(ev(
        40,
        t.clone(),
        wo.clone(),
        WorkOrderStatus::Draft,
        0x2300_0030,
        Some("idem_1"),
    ))
    .unwrap();

    s.append_work_order_ledger_row(ev(
        41,
        t.clone(),
        wo.clone(),
        WorkOrderStatus::Done,
        0x2300_0031,
        Some("idem_2"),
    ))
    .unwrap();

    let before = s.work_orders_current_rows().clone();
    s.rebuild_work_orders_current_rows();
    let after = s.work_orders_current_rows().clone();
    assert_eq!(before, after);

    let current = s.work_order_current_row(&t, &wo).unwrap();
    assert_eq!(current.work_order_status, WorkOrderStatus::Done);
}
