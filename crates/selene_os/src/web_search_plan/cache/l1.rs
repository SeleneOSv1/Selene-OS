#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::CacheKey;
use crate::web_search_plan::cache::CacheEntry;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct L1Cache {
    entries: BTreeMap<String, CacheEntry>,
}

impl L1Cache {
    pub fn get(&self, key: &CacheKey) -> Option<&CacheEntry> {
        self.entries.get(&key.stable_key_hash())
    }

    pub fn put(&mut self, key: &CacheKey, entry: CacheEntry) {
        self.entries.insert(key.stable_key_hash(), entry);
    }

    pub fn clear_end_of_turn(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
