#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
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
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, SessionRecord, StorageError,
    TenantCompanyLifecycleState, TenantCompanyRecord,
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
