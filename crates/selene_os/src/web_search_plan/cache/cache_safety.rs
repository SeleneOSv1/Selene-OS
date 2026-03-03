#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::CacheKey;
use crate::web_search_plan::cache::CacheEntry;

pub fn validate_cache_entry(
    key: &CacheKey,
    entry: &CacheEntry,
    expected_schema_version: &str,
    expected_policy_snapshot_id: &str,
    now_ms: i64,
) -> Result<(), String> {
    if entry.expires_at_ms <= now_ms {
        return Err("cache entry expired".to_string());
    }

    if entry.schema_version != expected_schema_version {
        return Err("cache schema_version mismatch".to_string());
    }

    let expected_policy = if expected_policy_snapshot_id.trim().is_empty() {
        "none"
    } else {
        expected_policy_snapshot_id
    };

    if entry.policy_snapshot_id != expected_policy {
        return Err("cache policy_snapshot_id mismatch".to_string());
    }

    if entry.cache_key_hash != key.stable_key_hash() {
        return Err("cache key hash mismatch".to_string());
    }

    Ok(())
}
