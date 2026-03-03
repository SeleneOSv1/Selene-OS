#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::{CacheKey, CacheMode};
use crate::web_search_plan::cache::l1::L1Cache;
use crate::web_search_plan::cache::l2;
use crate::web_search_plan::cache::{clear_l1_end_of_turn, lookup_typed, store_typed, CacheLayer};
use crate::web_search_plan::cache::ttl::ttl_ms_for;
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard, OnceLock};

static CACHE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_cache_tests() -> MutexGuard<'static, ()> {
    CACHE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cache test lock must not be poisoned")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SamplePayload {
    title: String,
    score: i32,
}

fn sample_key(policy_snapshot_id: &str) -> CacheKey {
    CacheKey::new(
        CacheMode::Web,
        "  Mixed   CASE query ",
        Some("https://example.com/a"),
        Some("brave_web_search"),
        ImportanceTier::Medium,
        Some(policy_snapshot_id),
    )
}

#[test]
fn test_t1_cache_key_determinism_same_inputs_same_key() {
    let _guard = lock_cache_tests();
    let key_a = sample_key("policy-a");
    let key_b = sample_key("policy-a");

    assert_eq!(key_a.stable_key_string(), key_b.stable_key_string());
    assert_eq!(key_a.stable_key_hash(), key_b.stable_key_hash());
}

#[test]
fn test_t2_l1_hit_and_clear_after_turn() {
    let _guard = lock_cache_tests();
    l2::clear_all_for_tests();
    let key = sample_key("policy-a");
    let mut l1 = L1Cache::default();

    let payload = SamplePayload {
        title: "from-cache".to_string(),
        score: 7,
    };

    let _ = store_typed(
        &mut l1,
        &key,
        &payload,
        "1.0.0",
        1_000,
        ttl_ms_for(CacheMode::Web, ImportanceTier::Medium),
        "policy-a",
        900,
    )
    .expect("store should succeed");

    let hit_l1 = lookup_typed::<SamplePayload>(&mut l1, &key, 1_100, "1.0.0", "policy-a")
        .expect("lookup should succeed")
        .expect("l1 hit expected");
    assert_eq!(hit_l1.layer, CacheLayer::L1);
    assert_eq!(hit_l1.value, payload);

    clear_l1_end_of_turn(&mut l1);
    assert!(l1.is_empty());

    let hit_l2 = lookup_typed::<SamplePayload>(&mut l1, &key, 1_200, "1.0.0", "policy-a")
        .expect("lookup should succeed")
        .expect("l2 hit expected");
    assert_eq!(hit_l2.layer, CacheLayer::L2);
    assert_eq!(hit_l2.value, payload);
}

#[test]
fn test_t3_l2_ttl_expiry_blocks_stale_return() {
    let _guard = lock_cache_tests();
    l2::clear_all_for_tests();
    let key = sample_key("policy-a");
    let mut l1 = L1Cache::default();

    let _ = store_typed(
        &mut l1,
        &key,
        &SamplePayload {
            title: "stale".to_string(),
            score: 1,
        },
        "1.0.0",
        10,
        5,
        "policy-a",
        10,
    )
    .expect("store should succeed");

    clear_l1_end_of_turn(&mut l1);
    let miss = lookup_typed::<SamplePayload>(&mut l1, &key, 20, "1.0.0", "policy-a")
        .expect("expired entry should not fail lookup");
    assert!(miss.is_none(), "expired L2 entry must not be returned");
}

#[test]
fn test_t4_cache_blocks_cross_tier_or_cross_policy_mismatch() {
    let _guard = lock_cache_tests();
    l2::clear_all_for_tests();
    let mut l1 = L1Cache::default();

    let key_medium = CacheKey::new(
        CacheMode::Web,
        "same query",
        Some("https://example.com/a"),
        Some("brave_web_search"),
        ImportanceTier::Medium,
        Some("policy-a"),
    );
    let key_low = CacheKey::new(
        CacheMode::Web,
        "same query",
        Some("https://example.com/a"),
        Some("brave_web_search"),
        ImportanceTier::Low,
        Some("policy-a"),
    );

    let _ = store_typed(
        &mut l1,
        &key_medium,
        &SamplePayload {
            title: "payload".to_string(),
            score: 9,
        },
        "1.0.0",
        1_000,
        1_000,
        "policy-a",
        1_000,
    )
    .expect("store should succeed");

    clear_l1_end_of_turn(&mut l1);

    let tier_miss = lookup_typed::<SamplePayload>(&mut l1, &key_low, 1_010, "1.0.0", "policy-a")
        .expect("lookup should succeed");
    assert!(tier_miss.is_none(), "different tier key must not hit cache");

    let policy_err = lookup_typed::<SamplePayload>(&mut l1, &key_medium, 1_010, "1.0.0", "policy-b")
        .expect_err("policy mismatch must fail closed");
    assert!(policy_err.contains("policy_snapshot_id mismatch"));
}

#[test]
fn test_t5_cache_preserves_retrieved_at_ms_provenance() {
    let _guard = lock_cache_tests();
    l2::clear_all_for_tests();
    let key = sample_key("policy-a");
    let mut l1 = L1Cache::default();

    let _ = store_typed(
        &mut l1,
        &key,
        &SamplePayload {
            title: "provenance".to_string(),
            score: 3,
        },
        "1.0.0",
        5_000,
        1_000,
        "policy-a",
        4_321,
    )
    .expect("store should succeed");

    let hit = lookup_typed::<SamplePayload>(&mut l1, &key, 5_100, "1.0.0", "policy-a")
        .expect("lookup should succeed")
        .expect("hit expected");
    assert_eq!(hit.retrieved_at_ms, 4_321);
}
