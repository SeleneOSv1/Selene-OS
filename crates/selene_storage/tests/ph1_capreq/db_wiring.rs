#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1capreq::{
    CapabilityRequestAction, CapabilityRequestLedgerEventInput, CapabilityRequestStatus, CapreqId,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::Ph1CapreqRepo;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id).unwrap()
}

fn capreq(id: &str) -> CapreqId {
    CapreqId::new(id).unwrap()
}

fn seed_identity(store: &mut Ph1fStore, user_id: &str) -> UserId {
    let user_id = UserId::new(user_id).unwrap();
    store
        .insert_identity(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();
    user_id
}

fn ev(
    t: u64,
    tenant_id: TenantId,
    capreq_id: CapreqId,
    requester_user_id: UserId,
    action: CapabilityRequestAction,
    status: CapabilityRequestStatus,
    idempotency_key: Option<&str>,
) -> CapabilityRequestLedgerEventInput {
    CapabilityRequestLedgerEventInput::v1(
        MonotonicTimeNs(t),
        tenant_id,
        capreq_id,
        requester_user_id,
        action,
        status,
        ReasonCodeId(0x4352_0001),
        "payload_hash_v1".to_string(),
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_capreq_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let a = tenant("tenant_a");
    let b = tenant("tenant_b");
    let capreq_id = capreq("capreq_upgrade_01");
    let user_a = seed_identity(&mut s, "tenant_a:user_alice");
    let user_b = seed_identity(&mut s, "tenant_b:user_bob");

    s.append_capreq_row(ev(
        10,
        a.clone(),
        capreq_id.clone(),
        user_a,
        CapabilityRequestAction::CreateDraft,
        CapabilityRequestStatus::Draft,
        Some("idem_a"),
    ))
    .unwrap();
    s.append_capreq_row(ev(
        11,
        b.clone(),
        capreq_id.clone(),
        user_b,
        CapabilityRequestAction::CreateDraft,
        CapabilityRequestStatus::Draft,
        Some("idem_b"),
    ))
    .unwrap();

    let ra = s.capreq_current_row(&a, &capreq_id).unwrap();
    let rb = s.capreq_current_row(&b, &capreq_id).unwrap();
    assert_eq!(ra.tenant_id, a);
    assert_eq!(rb.tenant_id, b);
    assert_eq!(s.capreq_current_rows().len(), 2);
}

#[test]
fn at_capreq_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let user = seed_identity(&mut s, "tenant_a:user_alice");

    let event_id = s
        .append_capreq_row(ev(
            20,
            tenant("tenant_a"),
            capreq("capreq_upgrade_02"),
            user,
            CapabilityRequestAction::CreateDraft,
            CapabilityRequestStatus::Draft,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_capreq_event(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_capreq_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let r = capreq("capreq_upgrade_03");
    let user = seed_identity(&mut s, "tenant_a:user_alice");

    let e1 = s
        .append_capreq_row(ev(
            30,
            t.clone(),
            r.clone(),
            user.clone(),
            CapabilityRequestAction::CreateDraft,
            CapabilityRequestStatus::Draft,
            Some("idem_same"),
        ))
        .unwrap();
    let e2 = s
        .append_capreq_row(ev(
            31,
            t.clone(),
            r.clone(),
            user,
            CapabilityRequestAction::SubmitForApproval,
            CapabilityRequestStatus::PendingApproval,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(e1, e2);
    assert_eq!(s.capreq_rows().len(), 1);
}

#[test]
fn at_capreq_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let t = tenant("tenant_a");
    let r = capreq("capreq_upgrade_04");
    let user = seed_identity(&mut s, "tenant_a:user_alice");

    let first = s
        .append_capreq_row(ev(
            40,
            t.clone(),
            r.clone(),
            user.clone(),
            CapabilityRequestAction::CreateDraft,
            CapabilityRequestStatus::Draft,
            Some("idem_1"),
        ))
        .unwrap();
    let second = s
        .append_capreq_row(ev(
            41,
            t.clone(),
            r.clone(),
            user,
            CapabilityRequestAction::Approve,
            CapabilityRequestStatus::Approved,
            Some("idem_2"),
        ))
        .unwrap();

    assert!(second > first);

    let before = s.capreq_current_rows().clone();
    s.rebuild_capreq_current_rows();
    let after = s.capreq_current_rows().clone();
    assert_eq!(before, after);

    let current = s.capreq_current_row(&t, &r).unwrap();
    assert_eq!(current.status, CapabilityRequestStatus::Approved);
    assert_eq!(current.last_action, CapabilityRequestAction::Approve);
    assert_eq!(current.source_event_id, second);
}
