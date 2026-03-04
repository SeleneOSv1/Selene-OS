#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const MAX_SUB_QUERIES: usize = 4;
pub const MIN_SUB_QUERIES_FOR_COMPLEX: usize = 2;
pub const MULTI_QUERY_VERSION: &str = "run33-multi-query-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiQueryPlan {
    pub is_complex: bool,
    pub sub_queries: Vec<String>,
}

pub fn decompose_query(query: &str, existing_sub_queries: &[String]) -> MultiQueryPlan {
    let normalized_query = normalize_query(query);
    let is_complex = is_complex_query(&normalized_query);

    let mut sub_queries = Vec::new();
    if !existing_sub_queries.is_empty() {
        sub_queries.extend(existing_sub_queries.iter().map(|entry| normalize_query(entry)));
    } else {
        sub_queries.push(normalized_query.clone());
    }

    if is_complex {
        let templates = [
            format!("{} key facts", normalized_query),
            format!("{} source comparison", normalized_query),
            format!("{} latest status", normalized_query),
        ];
        sub_queries.extend(templates);
    }

    if is_complex && sub_queries.len() < MIN_SUB_QUERIES_FOR_COMPLEX {
        sub_queries.push(format!("{} overview", normalized_query));
    }

    let sub_queries = dedup_preserve_order(sub_queries)
        .into_iter()
        .take(MAX_SUB_QUERIES)
        .collect::<Vec<String>>();

    MultiQueryPlan {
        is_complex,
        sub_queries,
    }
}

fn is_complex_query(normalized_query: &str) -> bool {
    const COMPLEX_MARKERS: &[&str] = &[
        " and ",
        " vs ",
        " versus ",
        " compare ",
        " timeline ",
        " impact ",
        " regulation ",
        " compliance ",
        " contradiction ",
    ];

    if normalized_query.split_whitespace().count() >= 10 {
        return true;
    }

    COMPLEX_MARKERS
        .iter()
        .any(|marker| normalized_query.contains(marker))
}

fn normalize_query(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

fn dedup_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for value in values {
        let key = value.to_ascii_lowercase();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        out.push(value);
    }
    out
}
