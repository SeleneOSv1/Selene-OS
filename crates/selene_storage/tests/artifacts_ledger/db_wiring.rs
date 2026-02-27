#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1art::{
    ArtifactLedgerRowInput, ArtifactScopeType, ArtifactStatus, ArtifactType, ArtifactVersion,
    ToolCacheRowInput,
};
use selene_kernel_contracts::MonotonicTimeNs;
use selene_storage::ph1f::{Ph1fStore, StorageError};

fn artifact_ev(
    t: u64,
    scope_type: ArtifactScopeType,
    scope_id: &str,
    artifact_type: ArtifactType,
    artifact_version: u32,
    status: ArtifactStatus,
    idempotency_key: Option<&str>,
) -> ArtifactLedgerRowInput {
    ArtifactLedgerRowInput::v1(
        MonotonicTimeNs(t),
        scope_type,
        scope_id.to_string(),
        artifact_type,
        ArtifactVersion(artifact_version),
        "sha256_pkg_hash_v1".to_string(),
        "blob://artifact_ref_v1".to_string(),
        "PH1.BUILDER".to_string(),
        "corr:123".to_string(),
        status,
        idempotency_key.map(ToString::to_string),
    )
    .unwrap()
}

#[test]
fn at_art_db_01_tenant_isolation_enforced() {
    let mut s = Ph1fStore::new_in_memory();

    s.append_artifact_ledger_row(artifact_ev(
        10,
        ArtifactScopeType::Tenant,
        "tenant_a",
        ArtifactType::WakePack,
        1,
        ArtifactStatus::Active,
        Some("idem_a"),
    ))
    .unwrap();
    s.append_artifact_ledger_row(artifact_ev(
        11,
        ArtifactScopeType::Tenant,
        "tenant_b",
        ArtifactType::WakePack,
        1,
        ArtifactStatus::Active,
        Some("idem_b"),
    ))
    .unwrap();

    let a = s
        .artifact_ledger_row(
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::WakePack,
            ArtifactVersion(1),
        )
        .unwrap();
    let b = s
        .artifact_ledger_row(
            ArtifactScopeType::Tenant,
            "tenant_b",
            ArtifactType::WakePack,
            ArtifactVersion(1),
        )
        .unwrap();
    assert_eq!(a.scope_id, "tenant_a");
    assert_eq!(b.scope_id, "tenant_b");
    assert_eq!(s.artifacts_ledger_rows().len(), 2);
}

#[test]
fn at_art_db_02_append_only_enforced() {
    let mut s = Ph1fStore::new_in_memory();
    let artifact_id = s
        .append_artifact_ledger_row(artifact_ev(
            20,
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::SttRoutingPolicyPack,
            1,
            ArtifactStatus::Active,
            Some("idem_append"),
        ))
        .unwrap();

    assert!(matches!(
        s.attempt_overwrite_artifact_ledger_row(artifact_id),
        Err(StorageError::AppendOnlyViolation { .. })
    ));
}

#[test]
fn at_art_db_03_idempotency_dedupe_works() {
    let mut s = Ph1fStore::new_in_memory();
    let ev1 = s
        .append_artifact_ledger_row(artifact_ev(
            30,
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::TtsRoutingPolicyPack,
            1,
            ArtifactStatus::Active,
            Some("idem_same"),
        ))
        .unwrap();
    let ev2 = s
        .append_artifact_ledger_row(artifact_ev(
            31,
            ArtifactScopeType::Tenant,
            "tenant_a",
            ArtifactType::TtsRoutingPolicyPack,
            1,
            ArtifactStatus::Active,
            Some("idem_same"),
        ))
        .unwrap();

    assert_eq!(ev1, ev2);
    assert_eq!(s.artifacts_ledger_rows().len(), 1);
}

#[test]
fn at_art_db_04_ledger_only_no_current_rebuild_required() {
    let mut s = Ph1fStore::new_in_memory();
    s.append_artifact_ledger_row(artifact_ev(
        40,
        ArtifactScopeType::Tenant,
        "tenant_a",
        ArtifactType::SttVocabPack,
        1,
        ArtifactStatus::Active,
        Some("idem_ledger_only"),
    ))
    .unwrap();

    // Row 7 has no current projection table in this slice; proof is append-only row presence.
    assert_eq!(s.artifacts_ledger_rows().len(), 1);
}

#[test]
fn at_art_db_05_tool_cache_upsert_and_ttl_read() {
    let mut s = Ph1fStore::new_in_memory();

    let c1 = s
        .upsert_tool_cache_row(
            ToolCacheRowInput::v1(
                "weather".to_string(),
                "qhash_1".to_string(),
                "en-US".to_string(),
                "{\"temp_c\":20}".to_string(),
                MonotonicTimeNs(200),
            )
            .unwrap(),
        )
        .unwrap();
    let c2 = s
        .upsert_tool_cache_row(
            ToolCacheRowInput::v1(
                "weather".to_string(),
                "qhash_1".to_string(),
                "en-US".to_string(),
                "{\"temp_c\":21}".to_string(),
                MonotonicTimeNs(300),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(c1, c2);
    assert_eq!(s.tool_cache_rows().len(), 1);
    let hit = s.tool_cache_hit("weather", "qhash_1", "en-US", MonotonicTimeNs(250));
    assert!(hit.is_some());
    let miss = s.tool_cache_hit("weather", "qhash_1", "en-US", MonotonicTimeNs(350));
    assert!(miss.is_none());
}
