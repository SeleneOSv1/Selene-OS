#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    AuditEngine, AuditEventInput, AuditEventType, AuditPayloadMin, AuditSeverity, CorrelationId,
    DeviceId, PayloadKey, PayloadValue, TurnId,
};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SessionState};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, SessionRecord, StorageError,
};
use selene_storage::repo::Ph1jAuditRepo;

fn user() -> UserId {
    UserId::new("dbw_j_user_1").unwrap()
}

fn device() -> DeviceId {
    DeviceId::new("dbw_j_device_1").unwrap()
}

fn store_with_identity_device_session() -> Ph1fStore {
    let mut s = Ph1fStore::new_in_memory();
    s.insert_identity(IdentityRecord::v1(
        user(),
        None,
        None,
        MonotonicTimeNs(1),
        IdentityStatus::Active,
    ))
    .unwrap();
    s.insert_device(
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
    s.insert_session(
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

fn payload_with_gate(gate: &str) -> AuditPayloadMin {
    AuditPayloadMin::v1(BTreeMap::from([(
        PayloadKey::new("gate").unwrap(),
        PayloadValue::new(gate).unwrap(),
    )]))
    .unwrap()
}

#[test]
fn at_j_db_01_tenant_isolation_enforced() {
    let mut s = store_with_identity_device_session();

    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(10),
            Some("tenant_a".to_string()),
            Some("wo_a".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(0x4A00_0001),
            AuditSeverity::Info,
            CorrelationId(100),
            TurnId(1),
            payload_with_gate("wake"),
            None,
            Some("idem_a_1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(11),
            Some("tenant_b".to_string()),
            Some("wo_b".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(0x4A00_0002),
            AuditSeverity::Info,
            CorrelationId(100),
            TurnId(1),
            payload_with_gate("wake"),
            None,
            Some("idem_b_1".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    let a = s.audit_rows_by_tenant("tenant_a");
    let b = s.audit_rows_by_tenant("tenant_b");
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].tenant_id.as_deref(), Some("tenant_a"));
    assert_eq!(b[0].tenant_id.as_deref(), Some("tenant_b"));
}

#[test]
fn at_j_db_02_append_only_enforced() {
    let mut s = store_with_identity_device_session();
    let id = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(20),
                Some("tenant_a".to_string()),
                Some("wo_a".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::Other,
                ReasonCodeId(0x4A00_0010),
                AuditSeverity::Info,
                CorrelationId(101),
                TurnId(1),
                AuditPayloadMin::empty_v1(),
                None,
                Some("idem_append_only".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_audit_event(id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_j_db_03_idempotency_dedupe_works() {
    let mut s = store_with_identity_device_session();

    let a1 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(30),
                Some("tenant_a".to_string()),
                Some("wo_123".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::StateTransition,
                ReasonCodeId(0x4A00_0020),
                AuditSeverity::Info,
                CorrelationId(200),
                TurnId(1),
                AuditPayloadMin::v1(BTreeMap::from([
                    (
                        PayloadKey::new("state_from").unwrap(),
                        PayloadValue::new("A").unwrap(),
                    ),
                    (
                        PayloadKey::new("state_to").unwrap(),
                        PayloadValue::new("B").unwrap(),
                    ),
                ]))
                .unwrap(),
                None,
                Some("idem_same".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    let a2 = s
        .append_audit_row(
            AuditEventInput::v1(
                MonotonicTimeNs(31),
                Some("tenant_a".to_string()),
                Some("wo_123".to_string()),
                Some(SessionId(1)),
                Some(user()),
                Some(device()),
                AuditEngine::Ph1J,
                AuditEventType::StateTransition,
                ReasonCodeId(0x4A00_0020),
                AuditSeverity::Info,
                CorrelationId(201),
                TurnId(2),
                AuditPayloadMin::v1(BTreeMap::from([
                    (
                        PayloadKey::new("state_from").unwrap(),
                        PayloadValue::new("A").unwrap(),
                    ),
                    (
                        PayloadKey::new("state_to").unwrap(),
                        PayloadValue::new("B").unwrap(),
                    ),
                ]))
                .unwrap(),
                None,
                Some("idem_same".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(a1, a2);
    assert_eq!(s.audit_rows().len(), 1);
}

#[test]
fn at_j_db_04_ledger_only_no_current_rebuild_required() {
    let mut s = store_with_identity_device_session();
    s.append_audit_row(
        AuditEventInput::v1(
            MonotonicTimeNs(40),
            Some("tenant_a".to_string()),
            Some("wo_x".to_string()),
            Some(SessionId(1)),
            Some(user()),
            Some(device()),
            AuditEngine::Ph1J,
            AuditEventType::Other,
            ReasonCodeId(0x4A00_0030),
            AuditSeverity::Info,
            CorrelationId(300),
            TurnId(1),
            AuditPayloadMin::empty_v1(),
            None,
            Some("idem_ledger_only".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    // PH1.J has no current projection table in this slice; proof is append-only row presence.
    assert_eq!(s.audit_rows().len(), 1);
}
