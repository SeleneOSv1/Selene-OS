#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::hasher::HashedChunk;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkHashCollision {
    pub chunk_id: String,
    pub first_index: usize,
    pub second_index: usize,
}

pub fn detect_chunk_hash_collisions(chunks: &[HashedChunk]) -> Result<(), ChunkHashCollision> {
    let mut seen: BTreeMap<&str, (&str, usize)> = BTreeMap::new();

    for chunk in chunks {
        if let Some((existing_text, existing_index)) = seen.get(chunk.chunk_id.as_str()) {
            if *existing_text != chunk.normalized_text {
                return Err(ChunkHashCollision {
                    chunk_id: chunk.chunk_id.clone(),
                    first_index: *existing_index,
                    second_index: chunk.chunk_index,
                });
            }
        } else {
            seen.insert(
                chunk.chunk_id.as_str(),
                (chunk.normalized_text.as_str(), chunk.chunk_index),
            );
        }
    }

    Ok(())
}
