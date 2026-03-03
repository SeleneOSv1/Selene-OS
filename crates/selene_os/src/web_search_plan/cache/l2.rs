#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::CacheKey;
use crate::web_search_plan::cache::CacheEntry;
use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

static L2_CACHE: OnceLock<RwLock<BTreeMap<String, CacheEntry>>> = OnceLock::new();

fn l2_map() -> &'static RwLock<BTreeMap<String, CacheEntry>> {
    L2_CACHE.get_or_init(|| RwLock::new(BTreeMap::new()))
}

pub fn get(key: &CacheKey) -> Option<CacheEntry> {
    let guard = l2_map().read().ok()?;
    guard.get(&key.stable_key_hash()).cloned()
}

pub fn put(key: &CacheKey, entry: CacheEntry) {
    if let Ok(mut guard) = l2_map().write() {
        guard.insert(key.stable_key_hash(), entry);
    }
}

pub fn remove(key: &CacheKey) {
    if let Ok(mut guard) = l2_map().write() {
        guard.remove(&key.stable_key_hash());
    }
}

pub fn prune_expired(now_ms: i64) {
    if let Ok(mut guard) = l2_map().write() {
        guard.retain(|_, value| value.expires_at_ms > now_ms);
    }
}

#[cfg(test)]
pub fn clear_all_for_tests() {
    if let Ok(mut guard) = l2_map().write() {
        guard.clear();
    }
}
