#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::chunker::TextChunk;
use sha2::{Digest, Sha256};

pub const HASH_VERSION: &str = "sha256-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedChunk {
    pub chunk_id: String,
    pub hash_version: &'static str,
    pub chunk_index: usize,
    pub norm_version: &'static str,
    pub chunk_version: &'static str,
    pub source_url: String,
    pub canonical_url: String,
    pub normalized_text: String,
    pub text_len_chars: usize,
}

pub trait ChunkHasher {
    fn hash_hex(&self, input: &[u8]) -> String;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256ChunkHasher;

impl ChunkHasher for Sha256ChunkHasher {
    fn hash_hex(&self, input: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}

pub fn hash_chunks(
    canonical_url: &str,
    source_url: &str,
    chunks: &[TextChunk],
) -> Vec<HashedChunk> {
    hash_chunks_with_hasher(canonical_url, source_url, chunks, &Sha256ChunkHasher)
}

pub fn hash_chunks_with_hasher(
    canonical_url: &str,
    source_url: &str,
    chunks: &[TextChunk],
    hasher: &impl ChunkHasher,
) -> Vec<HashedChunk> {
    chunks
        .iter()
        .map(|chunk| {
            let id = derive_chunk_id(canonical_url, chunk, hasher);
            HashedChunk {
                chunk_id: id,
                hash_version: HASH_VERSION,
                chunk_index: chunk.chunk_index,
                norm_version: chunk.norm_version,
                chunk_version: chunk.chunk_version,
                source_url: source_url.to_string(),
                canonical_url: canonical_url.to_string(),
                text_len_chars: chunk.normalized_text.chars().count(),
                normalized_text: chunk.normalized_text.clone(),
            }
        })
        .collect()
}

pub fn derive_chunk_id(
    canonical_url: &str,
    chunk: &TextChunk,
    hasher: &impl ChunkHasher,
) -> String {
    let material = format!(
        "hash_version={}\x1fcanonical_url={}\x1fnorm_version={}\x1fchunk_version={}\x1fchunk_index={}\x1fnormalized_text={}",
        HASH_VERSION,
        canonical_url,
        chunk.norm_version,
        chunk.chunk_version,
        chunk.chunk_index,
        chunk.normalized_text
    );
    hasher.hash_hex(material.as_bytes())
}
