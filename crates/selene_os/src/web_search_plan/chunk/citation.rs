#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::hasher::HashedChunk;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationAnchor {
    pub chunk_id: String,
    pub source_url: String,
}

pub fn build_citation_anchors(chunks: &[HashedChunk]) -> Vec<CitationAnchor> {
    chunks
        .iter()
        .map(|chunk| CitationAnchor {
            chunk_id: chunk.chunk_id.clone(),
            source_url: chunk.source_url.clone(),
        })
        .collect()
}

pub fn validate_citation_anchors(
    anchors: &[CitationAnchor],
    chunks: &[HashedChunk],
) -> Result<(), String> {
    let chunk_ids: BTreeSet<&str> = chunks.iter().map(|chunk| chunk.chunk_id.as_str()).collect();

    for anchor in anchors {
        if !chunk_ids.contains(anchor.chunk_id.as_str()) {
            return Err(format!(
                "citation anchor references unknown chunk_id {}",
                anchor.chunk_id
            ));
        }
        if anchor.source_url.trim().is_empty() {
            return Err("citation anchor source_url must not be empty".to_string());
        }
    }

    Ok(())
}
