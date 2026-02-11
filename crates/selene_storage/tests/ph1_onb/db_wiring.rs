#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1link::{DeliveryMethod, InviteeType, PrefilledContext};
use selene_kernel_contracts::ph1onb::{OnboardingStatus, ProofType, TermsStatus};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1LinkRepo, Ph1OnbRepo, Ph1fFoundationRepo, Ph1jAuditRepo};

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
    recipient_contact: &str,
    tenant_id: Option<String>,
    prefilled_context: Option<PrefilledContext>,
) -> selene_kernel_contracts::ph1link::LinkId {
    let (link, _) = store
        .ph1link_invite_generate_draft_row(
            MonotonicTimeNs(now),
            inviter_user_id,
            invitee_type,
            recipient_contact.to_string(),
            DeliveryMethod::Email,
            tenant_id,
            prefilled_context,
            None,
        )
        .unwrap();

    store
        .ph1link_invite_send_commit_row(
            MonotonicTimeNs(now + 1),
            link.link_id.clone(),
            DeliveryMethod::Email,
            recipient_contact.to_string(),
            format!("onb-link-send-{now}"),
        )
        .unwrap();

    store
        .ph1link_invite_open_activate_commit_row(
            MonotonicTimeNs(now + 2),
            link.link_id.clone(),
            format!("fp_{now}"),
        )
        .unwrap();

    link.link_id
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
        "a@example.com",
        Some("tenant_a".to_string()),
        Some(prefilled_a),
    );
    let link_b = seed_activated_link(
        &mut s,
        200,
        user_b,
        InviteeType::Employee,
        "b@example.com",
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

    let link_id = seed_activated_link(
        &mut s,
        500,
        u.clone(),
        InviteeType::Household,
        "idem@example.com",
        Some("tenant_a".to_string()),
        None,
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(503),
            link_id,
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

    let link_id = seed_activated_link(
        &mut s,
        600,
        u.clone(),
        InviteeType::Household,
        "flow@example.com",
        Some("tenant_a".to_string()),
        None,
    );

    let started = s
        .ph1onb_session_start_draft_row(
            MonotonicTimeNs(603),
            link_id,
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
            "household_member".to_string(),
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
