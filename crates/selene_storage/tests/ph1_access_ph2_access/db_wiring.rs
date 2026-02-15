#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    AccessDecision, AccessDeviceTrustLevel, AccessLifecycleState, AccessMode, AccessOverrideType,
    AccessVerificationLevel, DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
};
use selene_storage::repo::{Ph1AccessPh2AccessRepo, Ph1fFoundationRepo};

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
fn at_access_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a.clone(), device_a);
    seed_identity_device(&mut s, user_b.clone(), device_b);

    let a = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(100),
            "tenant_a".to_string(),
            user_a.clone(),
            "employee_reader".to_string(),
            AccessMode::R,
            "{\"financial_auth\":false}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-a-idem".to_string()),
        )
        .unwrap();
    let b = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(101),
            "tenant_b".to_string(),
            user_b.clone(),
            "employee_writer".to_string(),
            AccessMode::W,
            "{\"financial_auth\":false}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-b-idem".to_string()),
        )
        .unwrap();

    assert_ne!(a.access_instance_id, b.access_instance_id);
    assert_eq!(
        s.ph2access_instance_row_by_tenant_user("tenant_a", &user_a)
            .unwrap()
            .access_instance_id,
        a.access_instance_id
    );
    assert_eq!(
        s.ph2access_instance_row_by_tenant_user("tenant_b", &user_b)
            .unwrap()
            .access_instance_id,
        b.access_instance_id
    );

    let tenant_mismatch = s.ph2access_apply_override_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        a.access_instance_id,
        AccessOverrideType::Temporary,
        "{\"grant_mode\":\"W\",\"module\":\"broadcast\"}".to_string(),
        user_b,
        "ACCESS_OVERRIDE_TEMP_GRANT_COMMIT".to_string(),
        ReasonCodeId(0x4143_1001),
        MonotonicTimeNs(102),
        Some(MonotonicTimeNs(302)),
        "ovr-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_access_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            u.clone(),
            "employee_reader".to_string(),
            AccessMode::R,
            "{\"financial_auth\":false}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-append-idem".to_string()),
        )
        .unwrap();

    let ovr = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(201),
            "tenant_a".to_string(),
            inst.access_instance_id,
            AccessOverrideType::Temporary,
            "{\"grant_mode\":\"W\",\"module\":\"position\"}".to_string(),
            u,
            "ACCESS_OVERRIDE_TEMP_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_1002),
            MonotonicTimeNs(201),
            Some(MonotonicTimeNs(401)),
            "ovr-append-idem".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_access_override_row(&ovr.override_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_access_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let first = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            u.clone(),
            "employee_reader".to_string(),
            AccessMode::R,
            "{\"financial_auth\":false}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-idem-same".to_string()),
        )
        .unwrap();
    let second = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            u.clone(),
            "employee_admin".to_string(),
            AccessMode::X,
            "{\"financial_auth\":true}".to_string(),
            true,
            AccessVerificationLevel::Biometric,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v2".to_string(),
            Some("access-idem-same".to_string()),
        )
        .unwrap();

    assert_eq!(first.access_instance_id, second.access_instance_id);
    assert_eq!(first.effective_access_mode, AccessMode::R);
    assert_eq!(
        s.ph2access_instance_row_by_id(&first.access_instance_id)
            .unwrap()
            .effective_access_mode,
        AccessMode::R
    );

    let o1 = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(302),
            "tenant_a".to_string(),
            first.access_instance_id.clone(),
            AccessOverrideType::Temporary,
            "{\"grant_mode\":\"W\",\"module\":\"broadcast\"}".to_string(),
            u.clone(),
            "ACCESS_OVERRIDE_TEMP_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_1003),
            MonotonicTimeNs(302),
            Some(MonotonicTimeNs(502)),
            "ovr-idem-same".to_string(),
        )
        .unwrap();
    let o2 = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(303),
            "tenant_a".to_string(),
            first.access_instance_id.clone(),
            AccessOverrideType::Temporary,
            "{\"grant_mode\":\"W\",\"module\":\"broadcast\"}".to_string(),
            u,
            "ACCESS_OVERRIDE_TEMP_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_1003),
            MonotonicTimeNs(302),
            Some(MonotonicTimeNs(502)),
            "ovr-idem-same".to_string(),
        )
        .unwrap();

    assert_eq!(o1.override_id, o2.override_id);
    assert_eq!(
        s.ph2access_override_rows_for_instance(&first.access_instance_id)
            .len(),
        1
    );
}

#[test]
fn at_access_db_04_current_table_no_ledger_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u.clone(), d);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(400),
            "tenant_a".to_string(),
            u.clone(),
            "employee_writer".to_string(),
            AccessMode::W,
            "{\"financial_auth\":true}".to_string(),
            true,
            AccessVerificationLevel::Biometric,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-current-only".to_string()),
        )
        .unwrap();

    let overrides_before = s.ph2access_override_rows().len();
    let gate = s
        .ph1access_gate_decide_row(
            u,
            inst.access_instance_id.clone(),
            "position.update".to_string(),
            AccessMode::W,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(401),
        )
        .unwrap();
    let overrides_after = s.ph2access_override_rows().len();

    assert_eq!(gate.access_decision, AccessDecision::Allow);
    assert_eq!(overrides_before, overrides_after);
    assert_eq!(s.ph2access_instance_rows().len(), 1);
}

#[test]
fn at_access_db_05_gate_missing_instance_fails_closed() {
    let mut s = Ph1fStore::new_in_memory();
    let u = user("tenant_a:user_missing");
    let d = device("tenant_a_device_missing");
    seed_identity_device(&mut s, u.clone(), d);

    let gate = s
        .ph1access_gate_decide_row(
            u,
            "missing_access_instance".to_string(),
            "LINK_INVITE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(500),
        )
        .unwrap();

    assert_eq!(gate.access_decision, AccessDecision::Deny);
    assert!(
        gate.restriction_flags
            .iter()
            .any(|flag| flag == "INSTANCE_MISSING")
    );
}

#[test]
fn at_access_db_06_gate_deny_when_requested_action_not_allowed() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_action_deny");
    let d = device("tenant_a_device_action_deny");
    seed_identity_device(&mut s, u.clone(), d);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(600),
            "tenant_a".to_string(),
            u.clone(),
            "employee_link_inviter".to_string(),
            AccessMode::A,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-action-deny".to_string()),
        )
        .unwrap();

    let gate = s
        .ph1access_gate_decide_row(
            u,
            inst.access_instance_id,
            "CAPREQ_MANAGE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(601),
        )
        .unwrap();

    assert_eq!(gate.access_decision, AccessDecision::Deny);
    assert!(
        gate.restriction_flags
            .iter()
            .any(|flag| flag == "ACTION_NOT_ALLOWED")
    );
}

#[test]
fn at_access_db_07_gate_escalate_on_mode_upgrade_required() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_mode_escalate");
    let d = device("tenant_a_device_mode_escalate");
    seed_identity_device(&mut s, u.clone(), d);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(700),
            "tenant_a".to_string(),
            u.clone(),
            "employee_link_reader".to_string(),
            AccessMode::R,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-mode-escalate".to_string()),
        )
        .unwrap();

    let gate = s
        .ph1access_gate_decide_row(
            u,
            inst.access_instance_id,
            "LINK_INVITE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(701),
        )
        .unwrap();

    assert_eq!(gate.access_decision, AccessDecision::Escalate);
    assert!(
        gate.restriction_flags
            .iter()
            .any(|flag| flag == "MODE_UPGRADE_REQUIRED")
    );
}

#[test]
fn at_access_db_08_gate_user_scope_mismatch_fails_closed() {
    let mut s = Ph1fStore::new_in_memory();

    let owner = user("tenant_a:user_scope_owner");
    let other = user("tenant_a:user_scope_other");
    let d_owner = device("tenant_a_device_scope_owner");
    let d_other = device("tenant_a_device_scope_other");
    seed_identity_device(&mut s, owner.clone(), d_owner);
    seed_identity_device(&mut s, other.clone(), d_other);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(800),
            "tenant_a".to_string(),
            owner,
            "employee_link_inviter".to_string(),
            AccessMode::A,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-scope-owner".to_string()),
        )
        .unwrap();

    let gate = s
        .ph1access_gate_decide_row(
            other,
            inst.access_instance_id,
            "LINK_INVITE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(801),
        )
        .unwrap();

    assert_eq!(gate.access_decision, AccessDecision::Deny);
    assert!(
        gate.restriction_flags
            .iter()
            .any(|flag| flag == "USER_SCOPE_MISMATCH")
    );
}

#[test]
fn at_access_db_09_gate_allow_path_is_deterministic_across_retries() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_allow_retry");
    let d = device("tenant_a_device_allow_retry");
    seed_identity_device(&mut s, u.clone(), d);

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(900),
            "tenant_a".to_string(),
            u.clone(),
            "employee_link_inviter".to_string(),
            AccessMode::A,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("access-allow-retry".to_string()),
        )
        .unwrap();

    let first = s
        .ph1access_gate_decide_row(
            u.clone(),
            inst.access_instance_id.clone(),
            "LINK_INVITE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(901),
        )
        .unwrap();
    let second = s
        .ph1access_gate_decide_row(
            u,
            inst.access_instance_id,
            "LINK_INVITE".to_string(),
            AccessMode::A,
            AccessDeviceTrustLevel::Dtl4,
            false,
            MonotonicTimeNs(901),
        )
        .unwrap();

    assert_eq!(first.access_decision, AccessDecision::Allow);
    assert_eq!(first, second);
}
