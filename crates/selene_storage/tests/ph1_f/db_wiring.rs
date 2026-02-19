#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1builder::{
    required_approvals_for_change_class, BuilderApprovalState, BuilderApprovalStateStatus,
    BuilderChangeClass, BuilderExpectedEffect, BuilderLearningContext, BuilderMetricsSnapshot,
    BuilderPatchProposal, BuilderPostDeployDecisionAction, BuilderPostDeployJudgeResult,
    BuilderProposalStatus, BuilderReleaseStage, BuilderReleaseState, BuilderReleaseStateStatus,
    BuilderSignalWindow, BuilderValidationGateId, BuilderValidationGateResult,
    BuilderValidationRun, BuilderValidationRunStatus,
};
use selene_kernel_contracts::ph1f::ConversationTurnInput;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1m::{
    MemoryConfidence, MemoryConsent, MemoryKey, MemoryLayer, MemoryLedgerEvent,
    MemoryLedgerEventKind, MemoryProvenance, MemorySensitivityFlag, MemoryUsePolicy, MemoryValue,
};
use selene_kernel_contracts::ph1position::{PositionScheduleType, TenantId};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState};
use selene_storage::ph1f::{
    BuilderProposalLedgerRowInput, DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore,
    SessionRecord, StorageError, TenantCompanyLifecycleState, TenantCompanyRecord,
};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1jAuditRepo};

fn user() -> UserId {
    UserId::new("dbw_user_1").unwrap()
}

fn device() -> DeviceId {
    DeviceId::new("dbw_device_1").unwrap()
}

fn store_with_identity_device_session() -> Ph1fStore {
    let mut s = Ph1fStore::new_in_memory();
    s.insert_identity_row(IdentityRecord::v1(
        user(),
        None,
        None,
        MonotonicTimeNs(1),
        IdentityStatus::Active,
    ))
    .unwrap();
    s.insert_device_row(
        DeviceRecord::v1(
            device(),
            user(),
            "mobile".to_string(),
            MonotonicTimeNs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    s.insert_session_row(
        SessionRecord::v1(
            SessionId(1),
            user(),
            device(),
            SessionState::Open,
            MonotonicTimeNs(1),
            MonotonicTimeNs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    s
}

fn mem_event(
    kind: MemoryLedgerEventKind,
    key: &str,
    value: Option<&str>,
    t: u64,
) -> MemoryLedgerEvent {
    MemoryLedgerEvent::v1(
        kind,
        MonotonicTimeNs(t),
        MemoryKey::new(key).unwrap(),
        value.map(|v| MemoryValue::v1(v.to_string(), None).unwrap()),
        Some("evidence_ref".to_string()),
        MemoryProvenance::v1(Some(SessionId(1)), None).unwrap(),
        MemoryLayer::LongTerm,
        MemorySensitivityFlag::Low,
        MemoryConfidence::High,
        MemoryConsent::NotRequested,
        ReasonCodeId(0xF000_0001),
    )
    .unwrap()
}

fn builder_proposal(proposal_id: &str, source_signal_hash: &str) -> BuilderPatchProposal {
    BuilderPatchProposal::v1(
        proposal_id.to_string(),
        MonotonicTimeNs(100),
        BuilderSignalWindow::v1(MonotonicTimeNs(10), MonotonicTimeNs(20), 6).unwrap(),
        source_signal_hash.to_string(),
        vec![
            "crates/selene_os/src/ph1os.rs".to_string(),
            "crates/selene_storage/src/ph1f.rs".to_string(),
        ],
        BuilderChangeClass::ClassB,
        2700,
        BuilderExpectedEffect::v1(-120, -140, 230, 0).unwrap(),
        "compile + tests + guardrails".to_string(),
        "revert to previous patch set".to_string(),
        BuilderProposalStatus::Draft,
    )
    .unwrap()
}

fn builder_proposal_with_learning_context(
    proposal_id: &str,
    source_signal_hash: &str,
) -> BuilderPatchProposal {
    builder_proposal(proposal_id, source_signal_hash)
        .with_learning_context(
            BuilderLearningContext::v1(
                format!("learn_report_{}", proposal_id),
                vec![
                    "PH1.FEEDBACK".to_string(),
                    "PH1.LEARN".to_string(),
                    "PH1.KNOW".to_string(),
                ],
                3,
                vec![
                    "evidence_ref:9200:2:PH1.FEEDBACK:STT_REJECT".to_string(),
                    "evidence_ref:9200:2:PH1.LEARN:CLARIFY_LOOP".to_string(),
                    "evidence_ref:9200:2:PH1.KNOW:VOCAB_MISS".to_string(),
                ],
            )
            .unwrap(),
        )
        .unwrap()
}

fn builder_approval_state(
    approval_state_id: &str,
    proposal_id: &str,
    change_class: BuilderChangeClass,
    status: BuilderApprovalStateStatus,
    idempotency_key: &str,
) -> BuilderApprovalState {
    let required = required_approvals_for_change_class(change_class);
    let (tech_approved, product_security_approved) = match (change_class, status) {
        (BuilderChangeClass::ClassA, _) => (false, false),
        (BuilderChangeClass::ClassB, BuilderApprovalStateStatus::Approved) => (true, false),
        (BuilderChangeClass::ClassC, BuilderApprovalStateStatus::Approved) => (true, true),
        _ => (false, false),
    };
    let approvals_granted = (u8::from(tech_approved)) + (u8::from(product_security_approved));
    let resolved_at = match status {
        BuilderApprovalStateStatus::Pending => None,
        BuilderApprovalStateStatus::Approved | BuilderApprovalStateStatus::Rejected => {
            Some(MonotonicTimeNs(320))
        }
    };
    BuilderApprovalState::v1(
        approval_state_id.to_string(),
        proposal_id.to_string(),
        change_class,
        required,
        approvals_granted,
        tech_approved,
        product_security_approved,
        status,
        ReasonCodeId(0xB1D0_1001),
        MonotonicTimeNs(300),
        resolved_at,
        Some(idempotency_key.to_string()),
    )
    .unwrap()
}

fn builder_release_state(
    release_state_id: &str,
    proposal_id: &str,
    stage: BuilderReleaseStage,
    status: BuilderReleaseStateStatus,
    idempotency_key: &str,
) -> BuilderReleaseState {
    let stage_rollout_pct = match stage {
        BuilderReleaseStage::Staging => 0,
        BuilderReleaseStage::Canary => 5,
        BuilderReleaseStage::Ramp25 => 25,
        BuilderReleaseStage::Ramp50 => 50,
        BuilderReleaseStage::Production => 100,
        BuilderReleaseStage::RolledBack => 0,
    };
    BuilderReleaseState::v1(
        release_state_id.to_string(),
        proposal_id.to_string(),
        stage,
        stage_rollout_pct,
        status,
        "rollback_hook_ref".to_string(),
        true,
        ReasonCodeId(0xB1D0_1002),
        MonotonicTimeNs(400),
        Some(idempotency_key.to_string()),
    )
    .unwrap()
}

fn builder_post_deploy_result(
    judge_result_id: &str,
    proposal_id: &str,
    release_state_id: &str,
    idempotency_key: &str,
) -> BuilderPostDeployJudgeResult {
    BuilderPostDeployJudgeResult::v1(
        judge_result_id.to_string(),
        proposal_id.to_string(),
        release_state_id.to_string(),
        BuilderMetricsSnapshot::v1(180, 260, 40, 0, 30).unwrap(),
        BuilderMetricsSnapshot::v1(184, 266, 45, 10, 30).unwrap(),
        BuilderPostDeployDecisionAction::Accept,
        ReasonCodeId(0xB1D0_1003),
        MonotonicTimeNs(500),
        Some(idempotency_key.to_string()),
    )
    .unwrap()
}

#[test]
fn at_f_db_01_tenant_isolation_enforced() {
    let mut s = store_with_identity_device_session();
    let t1 = TenantId::new("tenant_1").unwrap();
    let t2 = TenantId::new("tenant_2").unwrap();

    s.ph1tenant_company_upsert(TenantCompanyRecord {
        schema_version: SchemaVersion(1),
        tenant_id: t1.clone(),
        company_id: "company_1".to_string(),
        legal_name: "Tenant One".to_string(),
        jurisdiction: "CN".to_string(),
        lifecycle_state: TenantCompanyLifecycleState::Active,
        created_at: MonotonicTimeNs(1),
        updated_at: MonotonicTimeNs(1),
    })
    .unwrap();
    s.ph1tenant_company_upsert(TenantCompanyRecord {
        schema_version: SchemaVersion(1),
        tenant_id: t2.clone(),
        company_id: "company_2".to_string(),
        legal_name: "Tenant Two".to_string(),
        jurisdiction: "US".to_string(),
        lifecycle_state: TenantCompanyLifecycleState::Active,
        created_at: MonotonicTimeNs(1),
        updated_at: MonotonicTimeNs(1),
    })
    .unwrap();

    let created = s
        .ph1position_create_draft(
            MonotonicTimeNs(10),
            user(),
            t1.clone(),
            "company_1".to_string(),
            "Store Manager".to_string(),
            "Ops".to_string(),
            "CN".to_string(),
            PositionScheduleType::FullTime,
            "profile_1".to_string(),
            "band_l3".to_string(),
            "dbw_pos_create_1".to_string(),
            "POSITION_SIM_001_CREATE_DRAFT",
            ReasonCodeId(0x5900_0001),
        )
        .unwrap();

    assert!(s.ph1position_get(&t1, &created.position_id).is_some());
    assert!(s.ph1position_get(&t2, &created.position_id).is_none());
}

#[test]
fn at_f_db_02_append_only_enforced() {
    let mut s = store_with_identity_device_session();

    let mem_id = s
        .append_memory_row(
            &user(),
            mem_event(
                MemoryLedgerEventKind::Stored,
                "preferred_name",
                Some("J"),
                10,
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("dbw_mem_1".to_string()),
        )
        .unwrap();
    assert!(matches!(
        s.attempt_overwrite_memory_ledger_row(mem_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));

    let conv_id = s
        .append_conversation_row(
            ConversationTurnInput::v1(
                MonotonicTimeNs(20),
                CorrelationId(100),
                TurnId(1),
                Some(SessionId(1)),
                user(),
                Some(device()),
                selene_kernel_contracts::ph1f::ConversationRole::User,
                selene_kernel_contracts::ph1f::ConversationSource::TypedText,
                "hello".to_string(),
                "hash_hello".to_string(),
                selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                Some("dbw_conv_1".to_string()),
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        s.attempt_overwrite_conversation_turn(conv_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));

    let ev = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(30),
                None,
                None,
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::Other,
                ReasonCodeId(0xF000_0002),
                AuditSeverity::Info,
                CorrelationId(100),
                TurnId(1),
                AuditPayloadMin::v1(BTreeMap::from([(
                    PayloadKey::new("event").unwrap(),
                    PayloadValue::new("append_only_check").unwrap(),
                )]))
                .unwrap(),
                None,
                Some("dbw_audit_1".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        s.attempt_overwrite_audit_event(ev),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_f_db_03_idempotency_dedupe_works() {
    let mut s = store_with_identity_device_session();

    let m1 = s
        .append_memory_row(
            &user(),
            mem_event(MemoryLedgerEventKind::Stored, "k", Some("v"), 10),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("dbw_mem_dup".to_string()),
        )
        .unwrap();
    let m2 = s
        .append_memory_row(
            &user(),
            mem_event(MemoryLedgerEventKind::Stored, "k", Some("v"), 11),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("dbw_mem_dup".to_string()),
        )
        .unwrap();
    assert_eq!(m1, m2);
    assert_eq!(s.memory_ledger_rows().len(), 1);

    let c1 = s
        .append_conversation_row(
            ConversationTurnInput::v1(
                MonotonicTimeNs(20),
                CorrelationId(101),
                TurnId(1),
                Some(SessionId(1)),
                user(),
                Some(device()),
                selene_kernel_contracts::ph1f::ConversationRole::User,
                selene_kernel_contracts::ph1f::ConversationSource::VoiceTranscript,
                "hi".to_string(),
                "hash_hi".to_string(),
                selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                Some("dbw_conv_dup".to_string()),
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    let c2 = s
        .append_conversation_row(
            ConversationTurnInput::v1(
                MonotonicTimeNs(21),
                CorrelationId(101),
                TurnId(1),
                Some(SessionId(1)),
                user(),
                Some(device()),
                selene_kernel_contracts::ph1f::ConversationRole::User,
                selene_kernel_contracts::ph1f::ConversationSource::VoiceTranscript,
                "hi".to_string(),
                "hash_hi".to_string(),
                selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                Some("dbw_conv_dup".to_string()),
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(c1, c2);
    assert_eq!(s.conversation_rows().len(), 1);

    let a1 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(30),
                None,
                None,
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::Other,
                ReasonCodeId(0xF000_0003),
                AuditSeverity::Info,
                CorrelationId(101),
                TurnId(1),
                AuditPayloadMin::empty_v1(),
                None,
                Some("dbw_audit_dup".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    let a2 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(31),
                None,
                None,
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::Other,
                ReasonCodeId(0xF000_0003),
                AuditSeverity::Info,
                CorrelationId(101),
                TurnId(1),
                AuditPayloadMin::empty_v1(),
                None,
                Some("dbw_audit_dup".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(a1, a2);
    assert_eq!(s.audit_rows().len(), 1);
}

#[test]
fn at_f_db_04_rebuild_current_from_ledger() {
    let mut s = store_with_identity_device_session();
    s.append_memory_row(
        &user(),
        mem_event(
            MemoryLedgerEventKind::Stored,
            "preferred_name",
            Some("John"),
            10,
        ),
        MemoryUsePolicy::AlwaysUsable,
        None,
        None,
    )
    .unwrap();
    s.append_memory_row(
        &user(),
        mem_event(
            MemoryLedgerEventKind::Updated,
            "preferred_name",
            Some("John P."),
            11,
        ),
        MemoryUsePolicy::AlwaysUsable,
        None,
        None,
    )
    .unwrap();

    let before = s.memory_current_rows().clone();
    s.rebuild_memory_current_rows();
    let after = s.memory_current_rows().clone();
    assert_eq!(before, after);
}

#[test]
fn at_f_db_05_builder_proposal_run_result_append_only_with_idempotency() {
    let mut s = store_with_identity_device_session();

    let proposal_row_id = s
        .append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
            proposal: builder_proposal("proposal_dbw_01", "signal_hash_dbw_01"),
            idempotency_key: Some("builder_prop_idem_01".to_string()),
        })
        .unwrap();
    let proposal_row_id_retry = s
        .append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
            proposal: builder_proposal("proposal_dbw_02", "signal_hash_dbw_01"),
            idempotency_key: Some("builder_prop_idem_01".to_string()),
        })
        .unwrap();
    assert_eq!(proposal_row_id, proposal_row_id_retry);
    assert_eq!(s.builder_proposal_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_proposal_ledger_row(proposal_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));

    let run_row_id = s
        .append_builder_validation_run_ledger_row(
            BuilderValidationRun::v1(
                "run_dbw_01".to_string(),
                "proposal_dbw_01".to_string(),
                MonotonicTimeNs(200),
                Some(MonotonicTimeNs(220)),
                BuilderValidationRunStatus::Passed,
                1,
                1,
                Some("builder_run_idem_01".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    let run_row_id_retry = s
        .append_builder_validation_run_ledger_row(
            BuilderValidationRun::v1(
                "run_dbw_02".to_string(),
                "proposal_dbw_01".to_string(),
                MonotonicTimeNs(201),
                Some(MonotonicTimeNs(220)),
                BuilderValidationRunStatus::Passed,
                1,
                1,
                Some("builder_run_idem_01".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(run_row_id, run_row_id_retry);
    assert_eq!(s.builder_validation_run_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_validation_run_ledger_row(run_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));

    let gate_row_id = s
        .append_builder_validation_gate_result_ledger_row(
            BuilderValidationGateResult::v1(
                "run_dbw_01".to_string(),
                "proposal_dbw_01".to_string(),
                BuilderValidationGateId::BldG1,
                true,
                MonotonicTimeNs(221),
                ReasonCodeId(0xB1D0_0001),
                "reproducible diff check passed".to_string(),
                Some("builder_gate_idem_01".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    let gate_row_id_retry = s
        .append_builder_validation_gate_result_ledger_row(
            BuilderValidationGateResult::v1(
                "run_dbw_01".to_string(),
                "proposal_dbw_01".to_string(),
                BuilderValidationGateId::BldG1,
                true,
                MonotonicTimeNs(221),
                ReasonCodeId(0xB1D0_0001),
                "reproducible diff check passed".to_string(),
                Some("builder_gate_idem_01".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(gate_row_id, gate_row_id_retry);
    assert_eq!(s.builder_validation_gate_result_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_validation_gate_result_ledger_row(gate_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_f_db_06_builder_run_result_requires_foreign_keys_and_match() {
    let mut s = store_with_identity_device_session();

    let run_without_proposal = s.append_builder_validation_run_ledger_row(
        BuilderValidationRun::v1(
            "run_fk_bad".to_string(),
            "missing_proposal".to_string(),
            MonotonicTimeNs(10),
            Some(MonotonicTimeNs(20)),
            BuilderValidationRunStatus::Failed,
            1,
            1,
            Some("builder_run_fk_bad".to_string()),
        )
        .unwrap(),
    );
    assert!(matches!(
        run_without_proposal,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    s.append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
        proposal: builder_proposal("proposal_dbw_10", "signal_hash_dbw_10"),
        idempotency_key: Some("builder_prop_10".to_string()),
    })
    .unwrap();
    s.append_builder_validation_run_ledger_row(
        BuilderValidationRun::v1(
            "run_dbw_10".to_string(),
            "proposal_dbw_10".to_string(),
            MonotonicTimeNs(30),
            Some(MonotonicTimeNs(40)),
            BuilderValidationRunStatus::Passed,
            1,
            1,
            Some("builder_run_10".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    let mismatched_result = s.append_builder_validation_gate_result_ledger_row(
        BuilderValidationGateResult::v1(
            "run_dbw_10".to_string(),
            "proposal_dbw_11".to_string(),
            BuilderValidationGateId::BldG2,
            false,
            MonotonicTimeNs(41),
            ReasonCodeId(0xB1D0_0002),
            "compile gate failed".to_string(),
            Some("builder_gate_mismatch".to_string()),
        )
        .unwrap(),
    );
    assert!(matches!(
        mismatched_result,
        Err(StorageError::ForeignKeyViolation { .. }) | Err(StorageError::ContractViolation(_))
    ));
}

#[test]
fn at_f_db_07_builder_approval_release_append_only_with_fk_and_idempotency() {
    let mut s = store_with_identity_device_session();

    let approval_without_proposal =
        s.append_builder_approval_state_ledger_row(builder_approval_state(
            "approval_missing",
            "missing_proposal",
            BuilderChangeClass::ClassB,
            BuilderApprovalStateStatus::Pending,
            "builder_approval_missing",
        ));
    assert!(matches!(
        approval_without_proposal,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    let release_without_proposal =
        s.append_builder_release_state_ledger_row(builder_release_state(
            "release_missing",
            "missing_proposal",
            BuilderReleaseStage::Staging,
            BuilderReleaseStateStatus::Blocked,
            "builder_release_missing",
        ));
    assert!(matches!(
        release_without_proposal,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    s.append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
        proposal: builder_proposal("proposal_dbw_20", "signal_hash_dbw_20"),
        idempotency_key: Some("builder_prop_20".to_string()),
    })
    .unwrap();

    let approval_row_id = s
        .append_builder_approval_state_ledger_row(builder_approval_state(
            "approval_dbw_20",
            "proposal_dbw_20",
            BuilderChangeClass::ClassB,
            BuilderApprovalStateStatus::Pending,
            "builder_approval_idem_20",
        ))
        .unwrap();
    let approval_row_id_retry = s
        .append_builder_approval_state_ledger_row(builder_approval_state(
            "approval_dbw_20_retry",
            "proposal_dbw_20",
            BuilderChangeClass::ClassB,
            BuilderApprovalStateStatus::Pending,
            "builder_approval_idem_20",
        ))
        .unwrap();
    assert_eq!(approval_row_id, approval_row_id_retry);
    assert_eq!(s.builder_approval_state_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_approval_state_ledger_row(approval_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));

    let release_row_id = s
        .append_builder_release_state_ledger_row(builder_release_state(
            "release_dbw_20",
            "proposal_dbw_20",
            BuilderReleaseStage::Staging,
            BuilderReleaseStateStatus::Blocked,
            "builder_release_idem_20",
        ))
        .unwrap();
    let release_row_id_retry = s
        .append_builder_release_state_ledger_row(builder_release_state(
            "release_dbw_20_retry",
            "proposal_dbw_20",
            BuilderReleaseStage::Staging,
            BuilderReleaseStateStatus::Blocked,
            "builder_release_idem_20",
        ))
        .unwrap();
    assert_eq!(release_row_id, release_row_id_retry);
    assert_eq!(s.builder_release_state_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_release_state_ledger_row(release_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_f_db_08_builder_post_deploy_judge_append_only_with_fk_and_idempotency() {
    let mut s = store_with_identity_device_session();

    let judge_without_fk =
        s.append_builder_post_deploy_judge_result_ledger_row(builder_post_deploy_result(
            "judge_missing",
            "missing_proposal",
            "missing_release",
            "builder_judge_missing",
        ));
    assert!(matches!(
        judge_without_fk,
        Err(StorageError::ForeignKeyViolation { .. })
    ));

    s.append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
        proposal: builder_proposal("proposal_dbw_30", "signal_hash_dbw_30"),
        idempotency_key: Some("builder_prop_30".to_string()),
    })
    .unwrap();
    s.append_builder_release_state_ledger_row(builder_release_state(
        "release_dbw_30",
        "proposal_dbw_30",
        BuilderReleaseStage::Production,
        BuilderReleaseStateStatus::Completed,
        "builder_release_30",
    ))
    .unwrap();

    let judge_row_id = s
        .append_builder_post_deploy_judge_result_ledger_row(builder_post_deploy_result(
            "judge_dbw_30",
            "proposal_dbw_30",
            "release_dbw_30",
            "builder_judge_idem_30",
        ))
        .unwrap();
    let judge_row_id_retry = s
        .append_builder_post_deploy_judge_result_ledger_row(builder_post_deploy_result(
            "judge_dbw_30_retry",
            "proposal_dbw_30",
            "release_dbw_30",
            "builder_judge_idem_30",
        ))
        .unwrap();
    assert_eq!(judge_row_id, judge_row_id_retry);
    assert_eq!(s.builder_post_deploy_judge_result_ledger_rows().len(), 1);
    assert!(matches!(
        s.attempt_overwrite_builder_post_deploy_judge_result_ledger_row(judge_row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_f_db_09_builder_learning_context_persists_in_proposal_rows() {
    let mut s = store_with_identity_device_session();
    s.append_builder_proposal_ledger_row(BuilderProposalLedgerRowInput {
        proposal: builder_proposal_with_learning_context("proposal_dbw_40", "signal_hash_dbw_40"),
        idempotency_key: Some("builder_prop_40".to_string()),
    })
    .unwrap();

    let row = s
        .builder_proposal_ledger_rows()
        .iter()
        .find(|r| r.proposal.proposal_id == "proposal_dbw_40")
        .expect("missing proposal row");
    let learning = row
        .proposal
        .learning_context
        .as_ref()
        .expect("missing learning context");
    assert_eq!(learning.learning_report_id, "learn_report_proposal_dbw_40");
    assert_eq!(learning.learning_signal_count, 3);
    assert!(learning
        .source_engines
        .iter()
        .any(|engine| engine == "PH1.FEEDBACK"));
    assert_eq!(learning.evidence_refs.len(), 3);
}
