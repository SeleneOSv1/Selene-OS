#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::multihop::hop_budget::DEFAULT_MAX_HOPS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const HOP_PLAN_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HopMode {
    Web,
    News,
    Structured,
    UrlFetch,
}

impl HopMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::News => "news",
            Self::Structured => "structured",
            Self::UrlFetch => "url_fetch",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "web" => Ok(Self::Web),
            "news" => Ok(Self::News),
            "structured" => Ok(Self::Structured),
            "url_fetch" => Ok(Self::UrlFetch),
            other => Err(format!("unsupported hop mode {}", other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopCondition {
    ProviderResultsExhausted,
    SufficientEvidence,
    BudgetExhausted,
    HopFailed,
}

impl StopCondition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderResultsExhausted => "provider_results_exhausted",
            Self::SufficientEvidence => "sufficient_evidence",
            Self::BudgetExhausted => "budget_exhausted",
            Self::HopFailed => "hop_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedOutput {
    Sources,
    ContentChunks,
    StructuredRows,
}

impl ExpectedOutput {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sources => "sources",
            Self::ContentChunks => "content_chunks",
            Self::StructuredRows => "structured_rows",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hop {
    pub hop_index: usize,
    pub sub_query: String,
    pub mode: HopMode,
    pub stop_condition: StopCondition,
    pub expected_outputs: Vec<ExpectedOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopPlan {
    pub plan_id: String,
    pub plan_version: String,
    pub root_query: String,
    pub hops: Vec<Hop>,
    pub max_hops: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopPlanInput {
    pub root_query: String,
    pub mode: HopMode,
    pub requested_sub_queries: Vec<String>,
    pub max_hops: usize,
}

pub fn build_hop_plan(input: &HopPlanInput) -> Result<HopPlan, String> {
    let normalized_root = normalize_query(input.root_query.as_str());
    if normalized_root.is_empty() {
        return Err("root query must not be empty".to_string());
    }

    let max_hops = if input.max_hops == 0 {
        DEFAULT_MAX_HOPS
    } else {
        input.max_hops
    };

    let candidate_queries = build_candidate_queries(&normalized_root, &input.requested_sub_queries);
    let mut deduped = Vec::new();
    let mut seen = BTreeSet::new();
    for query in candidate_queries {
        if !query.is_empty() && seen.insert(query.clone()) {
            deduped.push(query);
        }
    }
    deduped.truncate(max_hops);

    if deduped.is_empty() {
        return Err("hop plan requires at least one canonical sub_query".to_string());
    }

    let expected_outputs = expected_outputs_for_mode(input.mode);
    let hops = deduped
        .iter()
        .enumerate()
        .map(|(hop_index, sub_query)| Hop {
            hop_index,
            sub_query: sub_query.clone(),
            mode: input.mode,
            stop_condition: if hop_index + 1 == deduped.len() {
                StopCondition::SufficientEvidence
            } else {
                StopCondition::ProviderResultsExhausted
            },
            expected_outputs: expected_outputs.clone(),
        })
        .collect::<Vec<Hop>>();

    let plan_id = derive_plan_id(
        normalized_root.as_str(),
        input.mode,
        max_hops,
        deduped.as_slice(),
    );

    Ok(HopPlan {
        plan_id,
        plan_version: HOP_PLAN_VERSION.to_string(),
        root_query: normalized_root,
        hops,
        max_hops,
    })
}

fn build_candidate_queries(root_query: &str, requested_sub_queries: &[String]) -> Vec<String> {
    let mut candidates = vec![root_query.to_string()];

    if requested_sub_queries.is_empty() {
        candidates.extend(split_root_query(root_query));
    } else {
        candidates.extend(
            requested_sub_queries
                .iter()
                .map(|entry| normalize_query(entry.as_str()))
                .filter(|entry| !entry.is_empty()),
        );
    }

    candidates
}

fn split_root_query(root_query: &str) -> Vec<String> {
    let separator_applied = root_query
        .replace(" -> ", "|")
        .replace(';', "|")
        .replace(" and ", "|")
        .replace(" then ", "|");

    separator_applied
        .split('|')
        .map(normalize_query)
        .filter(|piece| !piece.is_empty() && piece != root_query)
        .collect()
}

fn expected_outputs_for_mode(mode: HopMode) -> Vec<ExpectedOutput> {
    match mode {
        HopMode::Structured => vec![ExpectedOutput::Sources, ExpectedOutput::StructuredRows],
        HopMode::Web | HopMode::News | HopMode::UrlFetch => {
            vec![ExpectedOutput::Sources, ExpectedOutput::ContentChunks]
        }
    }
}

pub fn normalize_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn derive_plan_id(root_query: &str, mode: HopMode, max_hops: usize, sub_queries: &[String]) -> String {
    let serialized_sub_queries = sub_queries.join("\x1e");
    let material = format!(
        "hop_plan_version={}\x1fmode={}\x1fmax_hops={}\x1froot_query={}\x1fsub_queries={}",
        HOP_PLAN_VERSION,
        mode.as_str(),
        max_hops,
        root_query,
        serialized_sub_queries
    );
    sha256_hex(material.as_bytes())
}
