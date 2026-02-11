#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1m::{
    MemoryConfidence, MemoryConsent, MemoryKey, MemoryLayer, MemoryLedgerEvent,
    MemoryLedgerEventKind, MemoryProvenance, MemorySensitivityFlag, MemoryUsePolicy, MemoryValue,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{IdentityRecord, IdentityStatus, Ph1fStore, StorageError};
use selene_storage::repo::{Ph1MRepo, Ph1fFoundationRepo};

fn user(id: &str) -> UserId {
    UserId::new(id).unwrap()
}

fn seed_identity(store: &mut Ph1fStore, user_id: UserId) {
    store
        .insert_identity_row(IdentityRecord::v1(
            user_id,
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .unwrap();
}

fn memory_event(
    kind: MemoryLedgerEventKind,
    t_event: u64,
    key: &str,
    value: Option<&str>,
) -> MemoryLedgerEvent {
    MemoryLedgerEvent::v1(
        kind,
        MonotonicTimeNs(t_event),
        MemoryKey::new(key).unwrap(),
        value.map(|v| MemoryValue::v1(v.to_string(), None).unwrap()),
        Some("memory-evidence".to_string()),
        MemoryProvenance::v1(None, None).unwrap(),
        MemoryLayer::LongTerm,
        MemorySensitivityFlag::Low,
        MemoryConfidence::High,
        MemoryConsent::NotRequested,
        ReasonCodeId(0x4d00_0001),
    )
    .unwrap()
}

#[test]
fn at_m_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    seed_identity(&mut s, user_a.clone());
    seed_identity(&mut s, user_b.clone());

    // Same idempotency key is allowed across different users (user-scoped dedupe).
    let row_a = s
        .ph1m_append_ledger_row(
            &user_a,
            memory_event(
                MemoryLedgerEventKind::Stored,
                10,
                "profile:preferred_name",
                Some("Alice"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("same-idempotency".to_string()),
        )
        .unwrap();
    let row_b = s
        .ph1m_append_ledger_row(
            &user_b,
            memory_event(
                MemoryLedgerEventKind::Stored,
                11,
                "profile:preferred_name",
                Some("Bob"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("same-idempotency".to_string()),
        )
        .unwrap();
    assert_ne!(row_a, row_b);

    let key = MemoryKey::new("profile:preferred_name").unwrap();
    let current_a = s.ph1m_memory_current_row(&user_a, &key).unwrap();
    let current_b = s.ph1m_memory_current_row(&user_b, &key).unwrap();
    assert_eq!(
        current_a.memory_value.as_ref().unwrap().verbatim,
        "Alice".to_string()
    );
    assert_eq!(
        current_b.memory_value.as_ref().unwrap().verbatim,
        "Bob".to_string()
    );
    assert_eq!(s.ph1m_memory_ledger_rows().len(), 2);
}

#[test]
fn at_m_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let row_id = s
        .ph1m_append_ledger_row(
            &user_id,
            memory_event(
                MemoryLedgerEventKind::Stored,
                20,
                "profile:timezone",
                Some("America/Los_Angeles"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            None,
        )
        .unwrap();

    assert!(matches!(
        s.ph1m_attempt_overwrite_memory_ledger_row(row_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_m_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let first = s
        .ph1m_append_ledger_row(
            &user_id,
            memory_event(
                MemoryLedgerEventKind::Stored,
                30,
                "profile:language",
                Some("en-US"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("mem-idem-language".to_string()),
        )
        .unwrap();
    let second = s
        .ph1m_append_ledger_row(
            &user_id,
            memory_event(
                MemoryLedgerEventKind::Stored,
                31,
                "profile:language",
                Some("en-US"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("mem-idem-language".to_string()),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(s.ph1m_memory_ledger_rows().len(), 1);
}

#[test]
fn at_m_db_04_rebuild_current_from_ledger() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    s.ph1m_append_ledger_row(
        &user_id,
        memory_event(
            MemoryLedgerEventKind::Stored,
            40,
            "profile:preferred_name",
            Some("Ana"),
        ),
        MemoryUsePolicy::AlwaysUsable,
        None,
        None,
    )
    .unwrap();
    s.ph1m_append_ledger_row(
        &user_id,
        memory_event(
            MemoryLedgerEventKind::Updated,
            41,
            "profile:preferred_name",
            Some("Ana P"),
        ),
        MemoryUsePolicy::AlwaysUsable,
        None,
        None,
    )
    .unwrap();
    let lang_row = s
        .ph1m_append_ledger_row(
            &user_id,
            memory_event(
                MemoryLedgerEventKind::Stored,
                42,
                "profile:language",
                Some("es-MX"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("mem-rebuild-lang".to_string()),
        )
        .unwrap();

    let before = s.ph1m_memory_current_rows().clone();
    s.ph1m_rebuild_current_from_ledger();
    let after = s.ph1m_memory_current_rows().clone();
    assert_eq!(before, after);

    // Idempotency index must still dedupe after rebuild.
    let retry = s
        .ph1m_append_ledger_row(
            &user_id,
            memory_event(
                MemoryLedgerEventKind::Stored,
                43,
                "profile:language",
                Some("es-MX"),
            ),
            MemoryUsePolicy::AlwaysUsable,
            None,
            Some("mem-rebuild-lang".to_string()),
        )
        .unwrap();
    assert_eq!(retry, lang_row);
    assert_eq!(s.ph1m_memory_ledger_rows().len(), 3);
}
