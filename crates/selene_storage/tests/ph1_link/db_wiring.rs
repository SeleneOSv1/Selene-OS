#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1link::{DeliveryMethod, InviteeType, LinkStatus};
use selene_kernel_contracts::MonotonicTimeNs;
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1LinkRepo, Ph1fFoundationRepo};

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
            "a@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_a".to_string()),
            None,
            None,
        )
        .unwrap();
    let (b, _) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(101),
            user_b.clone(),
            InviteeType::Employee,
            "b@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_b".to_string()),
            None,
            None,
        )
        .unwrap();

    assert_ne!(a.token_id, b.token_id);

    let mismatch = s.ph1link_invite_generate_draft_row(
        MonotonicTimeNs(102),
        user_a,
        InviteeType::Employee,
        "mismatch@example.com".to_string(),
        DeliveryMethod::Email,
        Some("tenant_b".to_string()),
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
            InviteeType::Household,
            "append@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_a".to_string()),
            None,
            None,
        )
        .unwrap();

    let proof = s
        .ph1link_invite_send_commit_row(
            MonotonicTimeNs(201),
            link.token_id,
            DeliveryMethod::Email,
            "append@example.com".to_string(),
            "link-append".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_link_delivery_proof_row(&proof.delivery_proof_ref),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
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
            "idem@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_a".to_string()),
            None,
            Some("default".to_string()),
        )
        .unwrap();
    let (l2, p2) = s
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(301),
            u,
            InviteeType::Employee,
            "idem@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_a".to_string()),
            None,
            Some("default".to_string()),
        )
        .unwrap();
    assert_eq!(l1.token_id, l2.token_id);
    assert!(p1.was_new);
    assert!(!p2.was_new);

    let s1 = s
        .ph1link_invite_send_commit_row(
            MonotonicTimeNs(302),
            l1.token_id.clone(),
            DeliveryMethod::Email,
            "idem@example.com".to_string(),
            "link-send-idem".to_string(),
        )
        .unwrap();
    let s2 = s
        .ph1link_invite_send_commit_row(
            MonotonicTimeNs(303),
            l1.token_id.clone(),
            DeliveryMethod::Email,
            "idem@example.com".to_string(),
            "link-send-idem".to_string(),
        )
        .unwrap();
    assert_eq!(s1.delivery_proof_ref, s2.delivery_proof_ref);
    assert_eq!(
        s.ph1link_delivery_proofs_for_link_row(&l1.token_id).len(),
        1
    );
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
            InviteeType::Household,
            "life@example.com".to_string(),
            DeliveryMethod::Email,
            Some("tenant_a".to_string()),
            None,
            None,
        )
        .unwrap();
    assert_eq!(link.status, LinkStatus::DraftCreated);

    s.ph1link_invite_send_commit_row(
        MonotonicTimeNs(401),
        link.token_id.clone(),
        DeliveryMethod::Email,
        "life@example.com".to_string(),
        "link-life-send".to_string(),
    )
    .unwrap();
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Sent
    );

    let (status, _, _) = s
        .ph1link_invite_open_activate_commit_row(
            MonotonicTimeNs(402),
            link.token_id.clone(),
            "fp_primary".to_string(),
        )
        .unwrap();
    assert_eq!(status, LinkStatus::Activated);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Activated
    );

    let (blocked_status, _) = s
        .ph1link_invite_forward_block_commit_row(link.token_id.clone(), "fp_other".to_string())
        .unwrap();
    assert_eq!(blocked_status, LinkStatus::Blocked);
    assert_eq!(
        s.ph1link_get_link_row(&link.token_id).unwrap().status,
        LinkStatus::Blocked
    );

    // Row 20 uses current-state `links` plus append-only `link_delivery_proofs`; no separate
    // rebuildable current projection exists for this slice.
    assert_eq!(
        s.ph1link_delivery_proofs_for_link_row(&link.token_id).len(),
        1
    );
}
