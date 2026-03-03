#![forbid(unsafe_code)]

use crate::web_search_plan::chunk::normalize::{split_paragraphs, NORM_VERSION};

pub const CHUNK_VERSION: &str = "1.0.0";
pub const MAX_CHUNK_CHARS_DEFAULT: usize = 1_200;
pub const MIN_CHUNK_CHARS_DEFAULT: usize = 200;
pub const MAX_CHUNKS_PER_DOCUMENT_DEFAULT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPolicy {
    pub max_chunk_chars: usize,
    pub min_chunk_chars: usize,
    pub max_chunks_per_document: usize,
}

impl Default for ChunkPolicy {
    fn default() -> Self {
        Self {
            max_chunk_chars: MAX_CHUNK_CHARS_DEFAULT,
            min_chunk_chars: MIN_CHUNK_CHARS_DEFAULT,
            max_chunks_per_document: MAX_CHUNKS_PER_DOCUMENT_DEFAULT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub chunk_index: usize,
    pub normalized_text: String,
    pub norm_version: &'static str,
    pub chunk_version: &'static str,
}

#[derive(Debug, Clone)]
pub struct ChunkedDocument {
    pub chunks: Vec<TextChunk>,
    pub truncated: bool,
}

pub fn chunk_document(normalized_document: &str, policy: ChunkPolicy) -> ChunkedDocument {
    let paragraphs = split_paragraphs(normalized_document);
    let expanded = expand_long_paragraphs(&paragraphs, policy.max_chunk_chars);
    let combined = combine_paragraphs_into_chunks(&expanded, policy.max_chunk_chars);
    let min_adjusted = enforce_minimum_chunk_size(&combined, policy);

    let truncated = min_adjusted.len() > policy.max_chunks_per_document;
    let mut limited = min_adjusted;
    if truncated {
        limited.truncate(policy.max_chunks_per_document);
    }

    let chunks = limited
        .into_iter()
        .enumerate()
        .map(|(chunk_index, normalized_text)| TextChunk {
            chunk_index,
            normalized_text,
            norm_version: NORM_VERSION,
            chunk_version: CHUNK_VERSION,
        })
        .collect();

    ChunkedDocument { chunks, truncated }
}

fn expand_long_paragraphs(paragraphs: &[String], max_chunk_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in paragraphs {
        if paragraph.chars().count() <= max_chunk_chars {
            out.push(paragraph.clone());
            continue;
        }
        out.extend(split_long_segment(paragraph, max_chunk_chars));
    }
    out
}

fn split_long_segment(segment: &str, max_chunk_chars: usize) -> Vec<String> {
    let chars: Vec<char> = segment.chars().collect();
    let mut out = Vec::new();
    let mut start = 0usize;

    while start < chars.len() {
        let mut end = (start + max_chunk_chars).min(chars.len());
        if end < chars.len() {
            let mut candidate = end;
            while candidate > start && !chars[candidate - 1].is_whitespace() {
                candidate = candidate.saturating_sub(1);
            }
            if candidate > start {
                end = candidate;
            }
        }

        let chunk: String = chars[start..end].iter().collect();
        let trimmed = chunk.split_whitespace().collect::<Vec<&str>>().join(" ");
        if !trimmed.is_empty() {
            out.push(trimmed);
        }

        start = end;
        while start < chars.len() && chars[start].is_whitespace() {
            start = start.saturating_add(1);
        }
    }

    out
}

fn combine_paragraphs_into_chunks(paragraphs: &[String], max_chunk_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in paragraphs {
        if paragraph.is_empty() {
            continue;
        }
        if current.is_empty() {
            current = paragraph.clone();
            continue;
        }

        let next_len = current.chars().count() + 2 + paragraph.chars().count();
        if next_len <= max_chunk_chars {
            current.push_str("\n\n");
            current.push_str(paragraph);
        } else {
            chunks.push(current);
            current = paragraph.clone();
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn enforce_minimum_chunk_size(chunks: &[String], policy: ChunkPolicy) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = 0usize;

    while index < chunks.len() {
        let current = chunks[index].clone();
        let current_len = current.chars().count();

        if current_len < policy.min_chunk_chars && index + 1 < chunks.len() {
            let next = chunks[index + 1].clone();
            let combined_len = current_len + 2 + next.chars().count();
            if combined_len <= policy.max_chunk_chars {
                out.push(format!("{}\n\n{}", current, next));
                index = index.saturating_add(2);
                continue;
            }
        }

        if current_len < policy.min_chunk_chars {
            if let Some(last) = out.last_mut() {
                let merged_len = last.chars().count() + 2 + current_len;
                if merged_len <= policy.max_chunk_chars {
                    last.push_str("\n\n");
                    last.push_str(&current);
                    index = index.saturating_add(1);
                    continue;
                }
            }
        }

        out.push(current);
        index = index.saturating_add(1);
    }

    out
}
