#![forbid(unsafe_code)]

pub mod cache_key;
pub mod cache_safety;
pub mod l1;
pub mod l2;
pub mod ttl;

use crate::web_search_plan::cache::cache_key::CacheKey;
use crate::web_search_plan::cache::cache_safety::validate_cache_entry;
use crate::web_search_plan::cache::l1::L1Cache;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLayer {
    L1,
    L2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub cache_key_hash: String,
    pub payload: Value,
    pub payload_hash: String,
    pub schema_version: String,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub policy_snapshot_id: String,
    pub retrieved_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CacheLookupHit<T> {
    pub value: T,
    pub layer: CacheLayer,
    pub retrieved_at_ms: i64,
}

pub fn lookup_typed<T: DeserializeOwned>(
    l1_cache: &mut L1Cache,
    key: &CacheKey,
    now_ms: i64,
    expected_schema_version: &str,
    expected_policy_snapshot_id: &str,
) -> Result<Option<CacheLookupHit<T>>, String> {
    if let Some(entry) = l1_cache.get(key).cloned() {
        match validate_cache_entry(
            key,
            &entry,
            expected_schema_version,
            expected_policy_snapshot_id,
            now_ms,
        ) {
            Ok(()) => {
                let value = parse_entry_payload::<T>(&entry)?;
                return Ok(Some(CacheLookupHit {
                    value,
                    layer: CacheLayer::L1,
                    retrieved_at_ms: entry.retrieved_at_ms,
                }));
            }
            Err(_) => {
                l1_cache.clear_end_of_turn();
            }
        }
    }

    let Some(entry) = l2::get(key) else {
        return Ok(None);
    };

    if let Err(err) = validate_cache_entry(
        key,
        &entry,
        expected_schema_version,
        expected_policy_snapshot_id,
        now_ms,
    ) {
        if err.contains("expired") {
            l2::remove(key);
            return Ok(None);
        }
        return Err(format!("cache safety violation: {}", err));
    }

    let value = parse_entry_payload::<T>(&entry)?;
    l1_cache.put(key, entry.clone());

    Ok(Some(CacheLookupHit {
        value,
        layer: CacheLayer::L2,
        retrieved_at_ms: entry.retrieved_at_ms,
    }))
}

pub fn store_typed<T: Serialize>(
    l1_cache: &mut L1Cache,
    key: &CacheKey,
    value: &T,
    schema_version: &str,
    now_ms: i64,
    ttl_ms: i64,
    policy_snapshot_id: &str,
    retrieved_at_ms: i64,
) -> Result<CacheEntry, String> {
    if ttl_ms <= 0 {
        return Err("cache ttl_ms must be > 0".to_string());
    }

    let payload = serde_json::to_value(value).map_err(|e| format!("cache serialize failed: {}", e))?;
    let payload_hash = hash_payload(&payload)?;

    let entry = CacheEntry {
        cache_key_hash: key.stable_key_hash(),
        payload,
        payload_hash,
        schema_version: schema_version.to_string(),
        created_at_ms: now_ms,
        expires_at_ms: now_ms.saturating_add(ttl_ms),
        policy_snapshot_id: if policy_snapshot_id.trim().is_empty() {
            "none".to_string()
        } else {
            policy_snapshot_id.to_string()
        },
        retrieved_at_ms,
    };

    l1_cache.put(key, entry.clone());
    l2::put(key, entry.clone());
    Ok(entry)
}

pub fn clear_l1_end_of_turn(l1_cache: &mut L1Cache) {
    l1_cache.clear_end_of_turn();
}

fn parse_entry_payload<T: DeserializeOwned>(entry: &CacheEntry) -> Result<T, String> {
    let hash = hash_payload(&entry.payload)?;
    if hash != entry.payload_hash {
        return Err("cache payload_hash mismatch".to_string());
    }
    serde_json::from_value(entry.payload.clone())
        .map_err(|e| format!("cache payload deserialize failed: {}", e))
}

fn hash_payload(payload: &Value) -> Result<String, String> {
    let canonical = serde_json::to_string(payload)
        .map_err(|e| format!("cache canonical json serialization failed: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
pub mod cache_tests;
