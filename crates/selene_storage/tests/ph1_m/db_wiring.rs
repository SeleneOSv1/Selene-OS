#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1f::ConversationTurnInput;
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1m::{
    MemoryConfidence, MemoryConsent, MemoryGraphEdgeInput, MemoryGraphEdgeKind,
    MemoryGraphNodeInput, MemoryGraphNodeKind, MemoryKey, MemoryLayer, MemoryLedgerEvent,
    MemoryLedgerEventKind, MemoryMetricPayload, MemoryProvenance, MemoryRetentionMode,
    MemorySensitivityFlag, MemorySuppressionRule, MemorySuppressionRuleKind,
    MemorySuppressionTargetType, MemoryThreadDigest, MemoryUsePolicy, MemoryValue,
    MemoryEmotionalThreadState,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
use selene_storage::ph1f::{IdentityRecord, IdentityStatus, MemoryThreadEventKind, Ph1fStore, StorageError};
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

fn suppression_rule(
    target_id: &str,
    rule_kind: MemorySuppressionRuleKind,
    active: bool,
    updated_at: u64,
    reason_code: ReasonCodeId,
) -> MemorySuppressionRule {
    MemorySuppressionRule::v1(
        MemorySuppressionTargetType::TopicKey,
        target_id.to_string(),
        rule_kind,
        active,
        reason_code,
        MonotonicTimeNs(updated_at),
    )
    .unwrap()
}

fn emotional_state(thread_key: &str, tag: &str, updated_at: u64) -> MemoryEmotionalThreadState {
    MemoryEmotionalThreadState::v1(
        thread_key.to_string(),
        vec![tag.to_string()],
        Some(format!("summary-{tag}")),
        MonotonicTimeNs(updated_at),
    )
    .unwrap()
}

fn metric_payload() -> MemoryMetricPayload {
    MemoryMetricPayload::v1(64, 2, 1, 1, 0, 0, 0, 0, 0, 0).unwrap()
}

fn thread_digest(
    thread_id: &str,
    title: &str,
    unresolved: bool,
    updated_at: u64,
) -> MemoryThreadDigest {
    MemoryThreadDigest::v1(
        thread_id.to_string(),
        title.to_string(),
        vec!["b1".to_string()],
        false,
        unresolved,
        MonotonicTimeNs(updated_at),
        1,
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

#[test]
fn at_m_db_05_suppression_rules_idempotent_and_user_scoped() {
    let mut s = Ph1fStore::new_in_memory();
    let user_a = user("tenant_a:user_1");
    let user_b = user("tenant_b:user_1");
    seed_identity(&mut s, user_a.clone());
    seed_identity(&mut s, user_b.clone());

    let rule_a = suppression_rule(
        "topic:japan_trip",
        MemorySuppressionRuleKind::DoNotMention,
        true,
        50,
        ReasonCodeId(0x4d00_0101),
    );
    let changed_a_first = s
        .ph1m_set_suppression_rule_row(
            &user_a,
            rule_a.clone(),
            MonotonicTimeNs(50),
            "sup_idem_1".to_string(),
        )
        .unwrap();
    let changed_a_retry = s
        .ph1m_set_suppression_rule_row(
            &user_a,
            suppression_rule(
                "topic:japan_trip",
                MemorySuppressionRuleKind::DoNotMention,
                true,
                51,
                ReasonCodeId(0x4d00_0102),
            ),
            MonotonicTimeNs(51),
            "sup_idem_1".to_string(),
        )
        .unwrap();

    let changed_b_first = s
        .ph1m_set_suppression_rule_row(
            &user_b,
            suppression_rule(
                "topic:japan_trip",
                MemorySuppressionRuleKind::DoNotMention,
                true,
                52,
                ReasonCodeId(0x4d00_0103),
            ),
            MonotonicTimeNs(52),
            "sup_idem_1".to_string(),
        )
        .unwrap();

    assert!(changed_a_first);
    assert!(changed_a_retry);
    assert!(changed_b_first);

    let row_a = s
        .ph1m_suppression_rule_row(
            &user_a,
            MemorySuppressionTargetType::TopicKey,
            "topic:japan_trip",
            MemorySuppressionRuleKind::DoNotMention,
        )
        .unwrap();
    let row_b = s
        .ph1m_suppression_rule_row(
            &user_b,
            MemorySuppressionTargetType::TopicKey,
            "topic:japan_trip",
            MemorySuppressionRuleKind::DoNotMention,
        )
        .unwrap();

    assert_eq!(row_a.rule.reason_code, ReasonCodeId(0x4d00_0101));
    assert_eq!(row_b.rule.reason_code, ReasonCodeId(0x4d00_0103));
    assert_eq!(s.ph1m_suppression_rule_rows().len(), 2);
}

#[test]
fn at_m_db_06_emotional_thread_commit_append_only_and_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let first = s
        .ph1m_emotional_thread_update_commit_row(
            &user_id,
            emotional_state("thread:japan_trip", "calm", 60),
            ReasonCodeId(0x4d00_0201),
            "emo_idem_1".to_string(),
        )
        .unwrap();
    let retry = s
        .ph1m_emotional_thread_update_commit_row(
            &user_id,
            emotional_state("thread:japan_trip", "focused", 61),
            ReasonCodeId(0x4d00_0202),
            "emo_idem_1".to_string(),
        )
        .unwrap();

    assert_eq!(first, retry);
    assert_eq!(s.ph1m_emotional_thread_ledger_rows().len(), 1);
    let current = s
        .ph1m_emotional_thread_current_row(&user_id, "thread:japan_trip")
        .unwrap();
    assert_eq!(current.state.tone_tags, vec!["calm".to_string()]);
    assert!(matches!(
        s.ph1m_attempt_overwrite_emotional_thread_ledger_row(first),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_m_db_07_metrics_commit_append_only_and_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let first = s
        .ph1m_metrics_emit_commit_row(
            &user_id,
            metric_payload(),
            ReasonCodeId(0x4d00_0301),
            MonotonicTimeNs(70),
            "metrics_idem_1".to_string(),
        )
        .unwrap();
    let retry = s
        .ph1m_metrics_emit_commit_row(
            &user_id,
            metric_payload(),
            ReasonCodeId(0x4d00_0301),
            MonotonicTimeNs(71),
            "metrics_idem_1".to_string(),
        )
        .unwrap();

    assert_eq!(first, retry);
    assert_eq!(s.ph1m_metrics_ledger_rows().len(), 1);
    assert!(matches!(
        s.ph1m_attempt_overwrite_metrics_ledger_row(first),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_m_db_08_thread_archive_fk_and_append_only() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let turn_id = s
        .append_conversation_row(
            ConversationTurnInput::v1(
                MonotonicTimeNs(80),
                CorrelationId(8080),
                TurnId(1),
                None,
                user_id.clone(),
                None,
                selene_kernel_contracts::ph1f::ConversationRole::User,
                selene_kernel_contracts::ph1f::ConversationSource::TypedText,
                "trip planning".to_string(),
                "hash_trip_planning".to_string(),
                selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                Some("ph1m_conv_idem_1".to_string()),
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap();

    let (event_id, stored) = s
        .ph1m_thread_digest_upsert_commit_row(
            &user_id,
            MemoryRetentionMode::Default,
            thread_digest("thread:japan_trip", "Japan trip", true, 81),
            MemoryThreadEventKind::ThreadDigestUpsert,
            ReasonCodeId(0x4d00_0401),
            "thread_idem_1".to_string(),
        )
        .unwrap();
    assert!(stored);

    let ref_count = s
        .ph1m_upsert_thread_refs(
            &user_id,
            "thread:japan_trip",
            vec![turn_id.0],
            MonotonicTimeNs(82),
        )
        .unwrap();
    assert_eq!(ref_count, 1);
    assert_eq!(
        s.ph1m_thread_ref_rows_for_thread(&user_id, "thread:japan_trip")
            .len(),
        1
    );

    s.ph1m_archive_index_upsert_row(
        &user_id,
        "archive:japan_trip:1".to_string(),
        Some("thread:japan_trip".to_string()),
        Some(turn_id.0),
        Some(7),
        MonotonicTimeNs(83),
    )
    .unwrap();

    assert!(matches!(
        s.ph1m_archive_index_upsert_row(
            &user_id,
            "archive:japan_trip:2".to_string(),
            Some("thread:japan_trip".to_string()),
            Some(turn_id.0.saturating_add(999)),
            Some(9),
            MonotonicTimeNs(84),
        ),
        Err(StorageError::ForeignKeyViolation {
            table: "memory_archive_index.conversation_turn_id",
            ..
        })
    ));

    assert!(matches!(
        s.ph1m_attempt_overwrite_thread_ledger_row(event_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_m_db_09_graph_upsert_unique_edge_and_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let node_a = MemoryGraphNodeInput::v1(
        "node:project:japan".to_string(),
        MemoryGraphNodeKind::Project,
        MemoryConfidence::High,
        MonotonicTimeNs(90),
        1,
    )
    .unwrap();
    let node_b = MemoryGraphNodeInput::v1(
        "node:vendor:niseko".to_string(),
        MemoryGraphNodeKind::Vendor,
        MemoryConfidence::High,
        MonotonicTimeNs(90),
        1,
    )
    .unwrap();
    let edge_1 = MemoryGraphEdgeInput::v1(
        "edge:1".to_string(),
        "node:project:japan".to_string(),
        "node:vendor:niseko".to_string(),
        MemoryGraphEdgeKind::DependsOn,
        MemoryConfidence::High,
        MonotonicTimeNs(90),
        1,
    )
    .unwrap();

    let count_first = s
        .ph1m_graph_upsert_commit_row(
            &user_id,
            vec![node_a.clone(), node_b.clone()],
            vec![edge_1.clone()],
            MonotonicTimeNs(90),
            "graph_idem_1".to_string(),
        )
        .unwrap();
    let count_retry = s
        .ph1m_graph_upsert_commit_row(
            &user_id,
            vec![node_a, node_b],
            vec![edge_1],
            MonotonicTimeNs(91),
            "graph_idem_1".to_string(),
        )
        .unwrap();

    assert_eq!(count_first, 3);
    assert_eq!(count_retry, 3);

    let edge_2 = MemoryGraphEdgeInput::v1(
        "edge:2".to_string(),
        "node:project:japan".to_string(),
        "node:vendor:niseko".to_string(),
        MemoryGraphEdgeKind::DependsOn,
        MemoryConfidence::High,
        MonotonicTimeNs(92),
        1,
    )
    .unwrap();
    let count_second = s
        .ph1m_graph_upsert_commit_row(
            &user_id,
            vec![],
            vec![edge_2],
            MonotonicTimeNs(92),
            "graph_idem_2".to_string(),
        )
        .unwrap();

    assert_eq!(count_second, 1);
    assert_eq!(s.ph1m_graph_node_rows_for_user(&user_id).len(), 2);
    let edges = s.ph1m_graph_edge_rows_for_user(&user_id);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge.edge_id, "edge:2".to_string());
}

#[test]
fn at_m_db_10_retention_mode_commit_idempotent() {
    let mut s = Ph1fStore::new_in_memory();
    let user_id = user("tenant_a:user_1");
    seed_identity(&mut s, user_id.clone());

    let first_effective_at = s
        .ph1m_retention_mode_set_commit_row(
            &user_id,
            MemoryRetentionMode::RememberEverything,
            MonotonicTimeNs(100),
            ReasonCodeId(0x4d00_0601),
            "ret_idem_1".to_string(),
        )
        .unwrap();
    let retry_effective_at = s
        .ph1m_retention_mode_set_commit_row(
            &user_id,
            MemoryRetentionMode::Default,
            MonotonicTimeNs(101),
            ReasonCodeId(0x4d00_0602),
            "ret_idem_1".to_string(),
        )
        .unwrap();

    assert_eq!(first_effective_at, MonotonicTimeNs(100));
    assert_eq!(retry_effective_at, MonotonicTimeNs(100));
    let pref = s.ph1m_retention_preference_row(&user_id).unwrap();
    assert_eq!(
        pref.memory_retention_mode,
        MemoryRetentionMode::RememberEverything
    );

    let second_effective_at = s
        .ph1m_retention_mode_set_commit_row(
            &user_id,
            MemoryRetentionMode::Default,
            MonotonicTimeNs(102),
            ReasonCodeId(0x4d00_0603),
            "ret_idem_2".to_string(),
        )
        .unwrap();
    assert_eq!(second_effective_at, MonotonicTimeNs(102));
    let pref_after = s.ph1m_retention_preference_row(&user_id).unwrap();
    assert_eq!(pref_after.memory_retention_mode, MemoryRetentionMode::Default);
}
