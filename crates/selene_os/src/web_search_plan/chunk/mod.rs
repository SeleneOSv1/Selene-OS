#![forbid(unsafe_code)]

pub mod chunker;
pub mod citation;
pub mod collision;
pub mod hasher;
pub mod normalize;

use crate::web_search_plan::chunk::chunker::{chunk_document, ChunkPolicy, TextChunk};
use crate::web_search_plan::chunk::citation::{build_citation_anchors, validate_citation_anchors};
use crate::web_search_plan::chunk::collision::detect_chunk_hash_collisions;
use crate::web_search_plan::chunk::hasher::{
    hash_chunks, hash_chunks_with_hasher, ChunkHasher, HashedChunk,
};
use crate::web_search_plan::chunk::normalize::normalize_document_for_chunking;

pub const EVIDENCE_TRUNCATED_REASON_CODE: &str = "evidence_truncated";
pub const HASH_COLLISION_REASON_CODE: &str = "hash_collision_detected";

#[derive(Debug, Clone)]
pub struct ChunkBuildOutput {
    pub chunks: Vec<HashedChunk>,
    pub truncated: bool,
    pub reason_codes: Vec<&'static str>,
    pub normalized_document: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkBuildError {
    HashCollisionDetected {
        chunk_id: String,
        first_index: usize,
        second_index: usize,
    },
    CitationAnchorInvalid(String),
}

pub fn build_hashed_chunks_for_document(
    canonical_url: &str,
    source_url: &str,
    raw_text: &str,
    policy: ChunkPolicy,
) -> Result<ChunkBuildOutput, ChunkBuildError> {
    build_hashed_chunks_with_default_hasher(canonical_url, source_url, raw_text, policy)
}

pub(crate) fn build_hashed_chunks_with_default_hasher(
    canonical_url: &str,
    source_url: &str,
    raw_text: &str,
    policy: ChunkPolicy,
) -> Result<ChunkBuildOutput, ChunkBuildError> {
    let normalized_document = normalize_document_for_chunking(raw_text);
    let chunked = chunk_document(&normalized_document, policy);
    let hashed = hash_chunks(canonical_url, source_url, &chunked.chunks);
    finalize_chunk_output(normalized_document, chunked.truncated, hashed)
}

pub(crate) fn build_hashed_chunks_with_custom_hasher(
    canonical_url: &str,
    source_url: &str,
    raw_text: &str,
    policy: ChunkPolicy,
    hasher: &impl ChunkHasher,
) -> Result<ChunkBuildOutput, ChunkBuildError> {
    let normalized_document = normalize_document_for_chunking(raw_text);
    let chunked = chunk_document(&normalized_document, policy);
    let hashed = hash_chunks_with_hasher(canonical_url, source_url, &chunked.chunks, hasher);
    finalize_chunk_output(normalized_document, chunked.truncated, hashed)
}

fn finalize_chunk_output(
    normalized_document: String,
    truncated: bool,
    chunks: Vec<HashedChunk>,
) -> Result<ChunkBuildOutput, ChunkBuildError> {
    if let Err(collision) = detect_chunk_hash_collisions(&chunks) {
        return Err(ChunkBuildError::HashCollisionDetected {
            chunk_id: collision.chunk_id,
            first_index: collision.first_index,
            second_index: collision.second_index,
        });
    }

    let anchors = build_citation_anchors(&chunks);
    if let Err(err) = validate_citation_anchors(&anchors, &chunks) {
        return Err(ChunkBuildError::CitationAnchorInvalid(err));
    }

    let mut reason_codes = Vec::new();
    if truncated {
        reason_codes.push(EVIDENCE_TRUNCATED_REASON_CODE);
    }

    Ok(ChunkBuildOutput {
        chunks,
        truncated,
        reason_codes,
        normalized_document,
    })
}

pub fn bounded_excerpt(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

pub fn to_text_chunks(output: &ChunkBuildOutput) -> Vec<TextChunk> {
    output
        .chunks
        .iter()
        .map(|chunk| TextChunk {
            chunk_index: chunk.chunk_index,
            normalized_text: chunk.normalized_text.clone(),
            norm_version: chunk.norm_version,
            chunk_version: chunk.chunk_version,
        })
        .collect()
}

#[cfg(test)]
pub mod chunk_tests;
