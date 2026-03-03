#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::chunker::ChunkPolicy;
use crate::web_search_plan::chunk::{
    bounded_excerpt, build_hashed_chunks_for_document, ChunkBuildError,
    EVIDENCE_TRUNCATED_REASON_CODE, HASH_COLLISION_REASON_CODE,
};
use crate::web_search_plan::planning::SearchCandidate;
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct SnippetFallbackOutput {
    pub sources: Vec<Value>,
    pub content_chunks: Vec<Value>,
    pub reason_codes: Vec<String>,
}

pub fn build_snippet_fallback(
    candidates: &[SearchCandidate],
    max_sources: usize,
    min_sources_for_grounded: usize,
) -> SnippetFallbackOutput {
    let mut sources = Vec::new();
    let mut content_chunks = Vec::new();
    let mut reason_codes: BTreeSet<String> = BTreeSet::new();

    for (rank, candidate) in candidates.iter().take(max_sources).enumerate() {
        sources.push(json!({
            "title": candidate.title,
            "url": candidate.url,
            "snippet": candidate.snippet,
            "media_type": "web",
            "provider_id": candidate.provider_id,
            "rank": rank + 1,
            "canonical_url": candidate.canonical_url,
            "snippet_only": true,
        }));

        match build_hashed_chunks_for_document(
            &candidate.canonical_url,
            &candidate.url,
            &candidate.snippet,
            ChunkPolicy::default(),
        ) {
            Ok(chunk_output) => {
                if chunk_output.truncated {
                    reason_codes.insert(EVIDENCE_TRUNCATED_REASON_CODE.to_string());
                }
                for hashed in chunk_output.chunks {
                    let chunk_id = hashed.chunk_id.clone();
                    let source_url = hashed.source_url.clone();
                    content_chunks.push(json!({
                        "chunk_id": chunk_id,
                        "hash_version": hashed.hash_version,
                        "norm_version": hashed.norm_version,
                        "chunk_version": hashed.chunk_version,
                        "source_url": source_url,
                        "canonical_url": hashed.canonical_url,
                        "chunk_index": hashed.chunk_index,
                        "text_excerpt": bounded_excerpt(&hashed.normalized_text, 320),
                        "text_len_chars": hashed.text_len_chars,
                        "snippet_only": true,
                        "citation": {
                            "chunk_id": hashed.chunk_id,
                            "source_url": hashed.source_url,
                        }
                    }));
                }
            }
            Err(ChunkBuildError::HashCollisionDetected { .. }) => {
                reason_codes.insert(HASH_COLLISION_REASON_CODE.to_string());
            }
            Err(ChunkBuildError::CitationAnchorInvalid(_)) => {
                reason_codes.insert("citation_mismatch".to_string());
            }
        }
    }

    if sources.len() < min_sources_for_grounded {
        reason_codes.insert("insufficient_evidence".to_string());
    }

    SnippetFallbackOutput {
        sources,
        content_chunks,
        reason_codes: reason_codes.into_iter().collect(),
    }
}
