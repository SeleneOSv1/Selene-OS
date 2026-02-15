#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1access::{
    AccessCompiledLineageRef, AccessOverlayId, AccessProfileId, AccessProfileVersionRef,
};
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::ph1position::{
    PositionId, PositionLifecycleState, PositionRecord, PositionScheduleType, TenantId,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};
use selene_storage::ph1f::{
    AccessBoardVoteValue, AccessDecision, AccessDeviceTrustLevel, AccessLifecycleState, AccessMode,
    AccessOverrideStatus, AccessOverrideType, AccessSchemaEventAction, AccessSchemaScope,
    AccessVerificationLevel, DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord,
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

fn ensure_global_ap_active(
    store: &mut Ph1fStore,
    now: u64,
    actor: UserId,
    profile_id: &str,
    version_id: &str,
) {
    store
        .ph1access_ap_schema_lifecycle_commit_row(
            MonotonicTimeNs(now),
            None,
            profile_id.to_string(),
            version_id.to_string(),
            AccessSchemaScope::Global,
            AccessSchemaEventAction::CreateDraft,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            ReasonCodeId(0x4143_3001),
            actor.clone(),
            format!("idem-global-draft-{}-{}", profile_id, version_id),
        )
        .unwrap();
    store
        .ph1access_ap_schema_lifecycle_commit_row(
            MonotonicTimeNs(now + 1),
            None,
            profile_id.to_string(),
            version_id.to_string(),
            AccessSchemaScope::Global,
            AccessSchemaEventAction::Activate,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            ReasonCodeId(0x4143_3002),
            actor,
            format!("idem-global-activate-{}-{}", profile_id, version_id),
        )
        .unwrap();
}

fn ensure_tenant_ap_active(
    store: &mut Ph1fStore,
    now: u64,
    actor: UserId,
    tenant_id: &str,
    profile_id: &str,
    version_id: &str,
) {
    store
        .ph1access_ap_schema_lifecycle_commit_row(
            MonotonicTimeNs(now),
            Some(tenant_id.to_string()),
            profile_id.to_string(),
            version_id.to_string(),
            AccessSchemaScope::Tenant,
            AccessSchemaEventAction::CreateDraft,
            "{\"allow\":[\"LINK_INVITE\",\"CAPREQ_MANAGE\"]}".to_string(),
            ReasonCodeId(0x4143_3011),
            actor.clone(),
            format!(
                "idem-tenant-draft-{}-{}-{}",
                tenant_id, profile_id, version_id
            ),
        )
        .unwrap();
    store
        .ph1access_ap_schema_lifecycle_commit_row(
            MonotonicTimeNs(now + 1),
            Some(tenant_id.to_string()),
            profile_id.to_string(),
            version_id.to_string(),
            AccessSchemaScope::Tenant,
            AccessSchemaEventAction::Activate,
            "{\"allow\":[\"LINK_INVITE\",\"CAPREQ_MANAGE\"]}".to_string(),
            ReasonCodeId(0x4143_3012),
            actor,
            format!(
                "idem-tenant-activate-{}-{}-{}",
                tenant_id, profile_id, version_id
            ),
        )
        .unwrap();
}

fn ensure_overlay_active(
    store: &mut Ph1fStore,
    now: u64,
    actor: UserId,
    tenant_id: &str,
    overlay_id: &str,
    overlay_version_id: &str,
) {
    store
        .ph1access_ap_overlay_update_commit_row(
            MonotonicTimeNs(now),
            tenant_id.to_string(),
            overlay_id.to_string(),
            overlay_version_id.to_string(),
            AccessSchemaEventAction::CreateDraft,
            "{\"ops\":[{\"op\":\"REMOVE_PERMISSION\",\"capability_id\":\"CAPREQ_MANAGE\"}]}"
                .to_string(),
            ReasonCodeId(0x4143_3021),
            actor.clone(),
            format!(
                "idem-overlay-draft-{}-{}-{}",
                tenant_id, overlay_id, overlay_version_id
            ),
        )
        .unwrap();
    store
        .ph1access_ap_overlay_update_commit_row(
            MonotonicTimeNs(now + 1),
            tenant_id.to_string(),
            overlay_id.to_string(),
            overlay_version_id.to_string(),
            AccessSchemaEventAction::Activate,
            "{\"ops\":[{\"op\":\"REMOVE_PERMISSION\",\"capability_id\":\"CAPREQ_MANAGE\"}]}"
                .to_string(),
            ReasonCodeId(0x4143_3022),
            actor,
            format!(
                "idem-overlay-activate-{}-{}-{}",
                tenant_id, overlay_id, overlay_version_id
            ),
        )
        .unwrap();
}

fn ensure_board_policy_active(
    store: &mut Ph1fStore,
    now: u64,
    actor: UserId,
    tenant_id: &str,
    board_policy_id: &str,
    policy_version_id: &str,
    payload_json: &str,
) {
    store
        .ph1access_board_policy_update_commit_row(
            MonotonicTimeNs(now),
            tenant_id.to_string(),
            board_policy_id.to_string(),
            policy_version_id.to_string(),
            AccessSchemaEventAction::CreateDraft,
            payload_json.to_string(),
            ReasonCodeId(0x4143_3031),
            actor.clone(),
            format!(
                "idem-policy-draft-{}-{}-{}",
                tenant_id, board_policy_id, policy_version_id
            ),
        )
        .unwrap();
    store
        .ph1access_board_policy_update_commit_row(
            MonotonicTimeNs(now + 1),
            tenant_id.to_string(),
            board_policy_id.to_string(),
            policy_version_id.to_string(),
            AccessSchemaEventAction::Activate,
            payload_json.to_string(),
            ReasonCodeId(0x4143_3032),
            actor,
            format!(
                "idem-policy-activate-{}-{}-{}",
                tenant_id, board_policy_id, policy_version_id
            ),
        )
        .unwrap();
}

fn baseline_lineage(
    global_profile_id: &str,
    global_version: &str,
    tenant_profile_id: Option<&str>,
    tenant_version: Option<&str>,
    overlay_ids: Vec<&str>,
    position_id: Option<&str>,
) -> AccessCompiledLineageRef {
    AccessCompiledLineageRef {
        global_profile_version: AccessProfileVersionRef {
            access_profile_id: AccessProfileId::new(global_profile_id).unwrap(),
            schema_version_id: global_version.to_string(),
        },
        tenant_profile_version: tenant_profile_id.map(|id| AccessProfileVersionRef {
            access_profile_id: AccessProfileId::new(id).unwrap(),
            schema_version_id: tenant_version.unwrap_or("v1").to_string(),
        }),
        overlay_version_ids: overlay_ids
            .into_iter()
            .map(|id| AccessOverlayId::new(id).unwrap())
            .collect(),
        position_id: position_id.map(str::to_string),
    }
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
    assert!(gate
        .restriction_flags
        .iter()
        .any(|flag| flag == "INSTANCE_MISSING"));
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
    assert!(gate
        .restriction_flags
        .iter()
        .any(|flag| flag == "ACTION_NOT_ALLOWED"));
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
    assert!(gate
        .restriction_flags
        .iter()
        .any(|flag| flag == "MODE_UPGRADE_REQUIRED"));
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
    assert!(gate
        .restriction_flags
        .iter()
        .any(|flag| flag == "USER_SCOPE_MISMATCH"));
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

#[test]
fn at_access_db_10_deny_by_default_when_schema_chain_missing_global_ap() {
    let s = Ph1fStore::new_in_memory();
    let err = s
        .ph1access_read_schema_chain_row("tenant_a", "ap_missing", &[], None)
        .unwrap_err();
    assert!(matches!(err, StorageError::ForeignKeyViolation { .. }));
}

#[test]
fn at_access_db_11_ap_version_pin_and_replay_determinism() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:access_admin_1");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_admin_1"));

    ensure_global_ap_active(&mut s, 1000, actor.clone(), "ap_driver_global", "v1");
    ensure_tenant_ap_active(
        &mut s,
        1010,
        actor.clone(),
        "tenant_a",
        "ap_driver_tenant",
        "v3",
    );

    let compiled_first = s
        .ph1access_instance_compile_commit_row(
            MonotonicTimeNs(1020),
            "tenant_a".to_string(),
            actor.clone(),
            "employee_driver".to_string(),
            AccessMode::A,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::Biometric,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_snapshot_access_v3".to_string(),
            baseline_lineage(
                "ap_driver_global",
                "v1",
                Some("ap_driver_tenant"),
                Some("v3"),
                Vec::new(),
                None,
            ),
            Some("idem-compile-v3".to_string()),
        )
        .unwrap();
    let compiled_second = s
        .ph1access_instance_compile_commit_row(
            MonotonicTimeNs(1021),
            "tenant_a".to_string(),
            actor.clone(),
            "employee_driver_changed".to_string(),
            AccessMode::R,
            "{\"allow\":[\"CAPREQ_MANAGE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl3,
            AccessLifecycleState::Restricted,
            "policy_snapshot_access_mutated".to_string(),
            baseline_lineage(
                "ap_driver_global",
                "v1",
                Some("ap_driver_tenant"),
                Some("v3"),
                Vec::new(),
                None,
            ),
            Some("idem-compile-v3".to_string()),
        )
        .unwrap();

    assert_eq!(
        compiled_first.access_instance_id,
        compiled_second.access_instance_id
    );
    assert_eq!(
        compiled_first.compiled_global_profile_version.as_deref(),
        Some("v1")
    );
    assert_eq!(
        compiled_first.compiled_tenant_profile_version.as_deref(),
        Some("v3")
    );
    assert_eq!(compiled_first.role_template_id, "employee_driver");
    assert_eq!(compiled_second.role_template_id, "employee_driver");
}

#[test]
fn at_access_db_12_overlay_merge_deterministic() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:access_admin_overlay");
    seed_identity_device(
        &mut s,
        actor.clone(),
        device("tenant_a_device_admin_overlay"),
    );

    ensure_global_ap_active(&mut s, 1100, actor.clone(), "ap_clerk_global", "v1");
    ensure_tenant_ap_active(
        &mut s,
        1110,
        actor.clone(),
        "tenant_a",
        "ap_clerk_tenant",
        "v2",
    );
    ensure_overlay_active(
        &mut s,
        1120,
        actor.clone(),
        "tenant_a",
        "overlay_b",
        "ov_b_v1",
    );
    ensure_overlay_active(
        &mut s,
        1130,
        actor.clone(),
        "tenant_a",
        "overlay_a",
        "ov_a_v1",
    );

    let compiled = s
        .ph1access_instance_compile_commit_row(
            MonotonicTimeNs(1140),
            "tenant_a".to_string(),
            actor,
            "employee_clerk".to_string(),
            AccessMode::W,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_snapshot_overlay".to_string(),
            baseline_lineage(
                "ap_clerk_global",
                "v1",
                Some("ap_clerk_tenant"),
                Some("v2"),
                vec!["ov_b_v1", "ov_a_v1"],
                None,
            ),
            Some("idem-overlay-compile".to_string()),
        )
        .unwrap();

    assert_eq!(
        compiled.compiled_overlay_set_ref.as_deref(),
        Some("ov_a_v1,ov_b_v1")
    );
}

#[test]
fn at_access_db_13_position_binding_required_for_compile() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:access_admin_position");
    seed_identity_device(
        &mut s,
        actor.clone(),
        device("tenant_a_device_admin_position"),
    );

    ensure_global_ap_active(&mut s, 1200, actor.clone(), "ap_ops_global", "v1");

    let missing_position_err = s
        .ph1access_instance_compile_commit_row(
            MonotonicTimeNs(1210),
            "tenant_a".to_string(),
            actor.clone(),
            "employee_ops".to_string(),
            AccessMode::W,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_snapshot_position_missing".to_string(),
            baseline_lineage(
                "ap_ops_global",
                "v1",
                None,
                None,
                Vec::new(),
                Some("position_missing"),
            ),
            Some("idem-position-missing".to_string()),
        )
        .unwrap_err();
    assert!(matches!(
        missing_position_err,
        StorageError::ForeignKeyViolation { .. }
    ));

    let tenant_id = TenantId::new("tenant_a").unwrap();
    s.ph1tenant_company_upsert(TenantCompanyRecord {
        schema_version: SchemaVersion(1),
        tenant_id: tenant_id.clone(),
        company_id: "company_a".to_string(),
        legal_name: "Company A".to_string(),
        jurisdiction: "US-CA".to_string(),
        lifecycle_state: TenantCompanyLifecycleState::Active,
        created_at: MonotonicTimeNs(1220),
        updated_at: MonotonicTimeNs(1220),
    })
    .unwrap();
    s.ph1position_upsert(
        PositionRecord::v1(
            tenant_id,
            "company_a".to_string(),
            PositionId::new("position_ops_1").unwrap(),
            "Operations".to_string(),
            "Ops".to_string(),
            "US-CA".to_string(),
            PositionScheduleType::FullTime,
            "ap_ops_global".to_string(),
            "band_1".to_string(),
            PositionLifecycleState::Active,
            MonotonicTimeNs(1221),
            MonotonicTimeNs(1221),
        )
        .unwrap(),
    )
    .unwrap();

    let compiled = s
        .ph1access_instance_compile_commit_row(
            MonotonicTimeNs(1230),
            "tenant_a".to_string(),
            actor,
            "employee_ops".to_string(),
            AccessMode::W,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_snapshot_position_bound".to_string(),
            baseline_lineage(
                "ap_ops_global",
                "v1",
                None,
                None,
                Vec::new(),
                Some("position_ops_1"),
            ),
            Some("idem-position-bound".to_string()),
        )
        .unwrap();
    assert_eq!(
        compiled.compiled_position_id.as_deref(),
        Some("position_ops_1")
    );
}

#[test]
fn at_access_db_14_escalation_n_of_m_and_board_quorum_vote_paths() {
    let mut s = Ph1fStore::new_in_memory();
    let admin = user("tenant_a:board_admin");
    let voter_1 = user("tenant_a:board_voter_1");
    let voter_2 = user("tenant_a:board_voter_2");
    seed_identity_device(&mut s, admin.clone(), device("tenant_a_device_board_admin"));
    seed_identity_device(
        &mut s,
        voter_1.clone(),
        device("tenant_a_device_board_voter_1"),
    );
    seed_identity_device(
        &mut s,
        voter_2.clone(),
        device("tenant_a_device_board_voter_2"),
    );

    ensure_board_policy_active(
        &mut s,
        1300,
        admin.clone(),
        "tenant_a",
        "policy_n_of_m",
        "v1",
        "{\"primitive\":\"N_OF_M\",\"required_approvals\":2,\"approver_pool_size\":3}",
    );
    ensure_board_policy_active(
        &mut s,
        1310,
        admin,
        "tenant_a",
        "policy_quorum",
        "v1",
        "{\"primitive\":\"BOARD_QUORUM_PERCENT\",\"board_quorum_percent\":70}",
    );

    let vote_1 = s
        .ph1access_board_vote_commit_row(
            MonotonicTimeNs(1320),
            "tenant_a".to_string(),
            "esc_case_1".to_string(),
            "policy_n_of_m".to_string(),
            voter_1.clone(),
            AccessBoardVoteValue::Approve,
            ReasonCodeId(0x4143_3041),
            "idem-vote-1".to_string(),
        )
        .unwrap();
    let vote_1_replay = s
        .ph1access_board_vote_commit_row(
            MonotonicTimeNs(1321),
            "tenant_a".to_string(),
            "esc_case_1".to_string(),
            "policy_n_of_m".to_string(),
            voter_1,
            AccessBoardVoteValue::Approve,
            ReasonCodeId(0x4143_3041),
            "idem-vote-1".to_string(),
        )
        .unwrap();
    let vote_2 = s
        .ph1access_board_vote_commit_row(
            MonotonicTimeNs(1322),
            "tenant_a".to_string(),
            "esc_case_1".to_string(),
            "policy_n_of_m".to_string(),
            voter_2,
            AccessBoardVoteValue::Approve,
            ReasonCodeId(0x4143_3042),
            "idem-vote-2".to_string(),
        )
        .unwrap();

    assert_eq!(vote_1.vote_row_id, vote_1_replay.vote_row_id);
    assert_ne!(vote_1.vote_row_id, vote_2.vote_row_id);
    assert!(s
        .ph1access_board_policy_current_rows()
        .contains_key(&("tenant_a".to_string(), "policy_n_of_m".to_string())));
    assert!(s
        .ph1access_board_policy_current_rows()
        .contains_key(&("tenant_a".to_string(), "policy_quorum".to_string())));
}

#[test]
fn at_access_db_15_override_lifecycle_types_persist() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:override_admin");
    seed_identity_device(
        &mut s,
        actor.clone(),
        device("tenant_a_device_override_admin"),
    );

    let inst = s
        .ph2access_upsert_instance_commit_row(
            MonotonicTimeNs(1400),
            "tenant_a".to_string(),
            actor.clone(),
            "employee_override".to_string(),
            AccessMode::R,
            "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
            true,
            AccessVerificationLevel::PasscodeTime,
            AccessDeviceTrustLevel::Dtl4,
            AccessLifecycleState::Active,
            "policy_v1".to_string(),
            Some("idem-override-base".to_string()),
        )
        .unwrap();

    let one_shot = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(1401),
            "tenant_a".to_string(),
            inst.access_instance_id.clone(),
            AccessOverrideType::OneShot,
            "{\"grant_mode\":\"W\",\"module\":\"module_one_shot\"}".to_string(),
            actor.clone(),
            "ACCESS_OVERRIDE_ONE_SHOT_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_3051),
            MonotonicTimeNs(1401),
            Some(MonotonicTimeNs(1410)),
            "idem-override-one-shot".to_string(),
        )
        .unwrap();
    let temporary = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(1402),
            "tenant_a".to_string(),
            inst.access_instance_id.clone(),
            AccessOverrideType::Temporary,
            "{\"grant_mode\":\"A\",\"module\":\"module_temporary\"}".to_string(),
            actor.clone(),
            "ACCESS_OVERRIDE_TEMP_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_3052),
            MonotonicTimeNs(1402),
            Some(MonotonicTimeNs(1500)),
            "idem-override-temporary".to_string(),
        )
        .unwrap();
    let permanent = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(1403),
            "tenant_a".to_string(),
            inst.access_instance_id.clone(),
            AccessOverrideType::Permanent,
            "{\"grant_mode\":\"X\",\"module\":\"module_permanent\"}".to_string(),
            actor.clone(),
            "ACCESS_OVERRIDE_PERM_GRANT_COMMIT".to_string(),
            ReasonCodeId(0x4143_3053),
            MonotonicTimeNs(1403),
            None,
            "idem-override-permanent".to_string(),
        )
        .unwrap();
    let revoke = s
        .ph2access_apply_override_commit_row(
            MonotonicTimeNs(1404),
            "tenant_a".to_string(),
            inst.access_instance_id.clone(),
            AccessOverrideType::Revoke,
            "{\"grant_mode\":\"R\",\"module\":\"module_revoke\"}".to_string(),
            actor,
            "ACCESS_OVERRIDE_REVOKE_COMMIT".to_string(),
            ReasonCodeId(0x4143_3054),
            MonotonicTimeNs(1404),
            None,
            "idem-override-revoke".to_string(),
        )
        .unwrap();

    assert_eq!(one_shot.status, AccessOverrideStatus::Active);
    assert_eq!(temporary.status, AccessOverrideStatus::Active);
    assert_eq!(permanent.status, AccessOverrideStatus::Active);
    assert_eq!(revoke.status, AccessOverrideStatus::Revoked);
    assert_eq!(
        s.ph2access_override_rows_for_instance(&inst.access_instance_id)
            .len(),
        4
    );
}

#[test]
fn at_access_db_16_tenant_isolation_enforced_for_schema_chain_reads() {
    let mut s = Ph1fStore::new_in_memory();
    let actor = user("tenant_a:access_admin_scope");
    seed_identity_device(&mut s, actor.clone(), device("tenant_a_device_admin_scope"));
    ensure_global_ap_active(&mut s, 1500, actor.clone(), "ap_scope_global", "v1");
    ensure_overlay_active(
        &mut s,
        1510,
        actor.clone(),
        "tenant_a",
        "overlay_scope",
        "ov_scope_v1",
    );
    ensure_board_policy_active(
        &mut s,
        1520,
        actor,
        "tenant_a",
        "policy_scope",
        "v1",
        "{\"primitive\":\"N_OF_M\",\"required_approvals\":2,\"approver_pool_size\":3}",
    );

    let chain_err = s
        .ph1access_read_schema_chain_row(
            "tenant_b",
            "ap_scope_global",
            &["overlay_scope".to_string()],
            Some("policy_scope"),
        )
        .unwrap_err();
    assert!(matches!(
        chain_err,
        StorageError::ForeignKeyViolation { .. }
    ));
}
