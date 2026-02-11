#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, Ph1kDeviceHealth,
    Ph1kRuntimeEventKind, StorageError,
};
use selene_storage::repo::{Ph1fFoundationRepo, Ph1kVoiceRuntimeRepo};

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
fn at_k_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    let device_a = device("tenant_a_device_1");
    let device_b = device("tenant_b_device_1");
    seed_identity_device(&mut s, user_a, device_a.clone());
    seed_identity_device(&mut s, user_b, device_b.clone());

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(100),
        "tenant_a".to_string(),
        device_a.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(1001),
        None,
        Some(501),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-a".to_string(),
    )
    .unwrap();

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(101),
        "tenant_b".to_string(),
        device_b.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(2001),
        None,
        Some(601),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-b".to_string(),
    )
    .unwrap();

    let tenant_mismatch = s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(102),
        "tenant_b".to_string(),
        device_a.clone(),
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(0),
        Some(0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-tenant-mismatch".to_string(),
    );
    assert!(matches!(
        tenant_mismatch,
        Err(StorageError::ContractViolation(_))
    ));

    assert!(s.ph1k_runtime_current_row("tenant_a", &device_a).is_some());
    assert!(s.ph1k_runtime_current_row("tenant_b", &device_b).is_some());
    assert!(s.ph1k_runtime_current_row("tenant_b", &device_a).is_none());
}

#[test]
fn at_k_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let ev = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(200),
            "tenant_a".to_string(),
            d,
            None,
            Ph1kRuntimeEventKind::InterruptCandidate,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(11),
            Some("stop".to_string()),
            Some(ReasonCodeId(0x4B00_1001)),
            None,
            None,
            None,
            None,
            None,
            "k-append-only".to_string(),
        )
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_ph1k_runtime_event_row(ev.event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_k_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    let first = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(300),
            "tenant_a".to_string(),
            d.clone(),
            None,
            Ph1kRuntimeEventKind::TimingStats,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(1.5),
            Some(2.5),
            Some(3.5),
            Some(1),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "k-idem".to_string(),
        )
        .unwrap();

    let second = s
        .ph1k_runtime_event_commit_row(
            MonotonicTimeNs(301),
            "tenant_a".to_string(),
            d.clone(),
            None,
            Ph1kRuntimeEventKind::TimingStats,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(9.9),
            Some(9.9),
            Some(9.9),
            Some(99),
            Some(99),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "k-idem".to_string(),
        )
        .unwrap();

    assert_eq!(first.event_id, second.event_id);
    assert_eq!(s.ph1k_runtime_event_rows().len(), 1);
    assert_eq!(
        s.ph1k_runtime_current_row("tenant_a", &d)
            .unwrap()
            .jitter_ms,
        Some(1500)
    );
}

#[test]
fn at_k_db_04_current_table_rebuild_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();

    let u = user("tenant_a:user_1");
    let d = device("tenant_a_device_1");
    seed_identity_device(&mut s, u, d.clone());

    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(400),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::StreamRefs,
        Some(7001),
        Some(7002),
        Some(333),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-stream".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(401),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DeviceState,
        None,
        None,
        None,
        Some("mic_primary".to_string()),
        Some("spk_primary".to_string()),
        Some(Ph1kDeviceHealth::Healthy),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-device".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(402),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TimingStats,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5),
        Some(6),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-timing".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(403),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::DegradationFlags,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        Some(false),
        Some(true),
        Some(true),
        "k-rebuild-degrade".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(404),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::InterruptCandidate,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(99),
        Some("hold on".to_string()),
        Some(ReasonCodeId(0x4B00_2001)),
        None,
        None,
        None,
        None,
        None,
        "k-rebuild-interrupt".to_string(),
    )
    .unwrap();
    s.ph1k_runtime_event_commit_row(
        MonotonicTimeNs(405),
        "tenant_a".to_string(),
        d.clone(),
        None,
        Ph1kRuntimeEventKind::TtsPlaybackActive,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        "k-rebuild-tts".to_string(),
    )
    .unwrap();

    let before = s.ph1k_runtime_current_row("tenant_a", &d).unwrap().clone();
    s.rebuild_ph1k_runtime_current_rows();
    let after = s.ph1k_runtime_current_row("tenant_a", &d).unwrap().clone();

    assert_eq!(before, after);
    assert_eq!(after.tts_playback_active, true);
    assert_eq!(after.capture_degraded, true);
    assert_eq!(after.stream_gap_detected, true);
    assert_eq!(after.last_interrupt_phrase.as_deref(), Some("hold on"));
    assert_eq!(
        after.last_event_id,
        s.ph1k_runtime_event_rows().last().unwrap().event_id
    );
}
