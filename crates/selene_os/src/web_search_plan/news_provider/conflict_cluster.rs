#![forbid(unsafe_code)]

use crate::web_search_plan::news_provider::NormalizedNewsResult;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictCluster {
    pub group_id: String,
    pub topic_key: String,
    pub source_urls: Vec<String>,
    pub claims: Vec<String>,
}

pub fn build_conflict_clusters(results: &[NormalizedNewsResult]) -> Vec<ConflictCluster> {
    let mut grouped: BTreeMap<String, Vec<&NormalizedNewsResult>> = BTreeMap::new();

    for result in results {
        let key = topic_key(result);
        grouped.entry(key).or_default().push(result);
    }

    let mut out = Vec::new();
    let mut cluster_index = 1usize;

    for (topic_key, group) in grouped {
        let mut has_positive = false;
        let mut has_negative = false;
        let mut source_urls = BTreeSet::new();
        let mut claims = BTreeSet::new();

        for result in group {
            let polarity = polarity(result);
            has_positive |= polarity >= 0;
            has_negative |= polarity <= 0;
            source_urls.insert(result.url.clone());
            claims.insert(result.snippet.clone());
        }

        if !(has_positive && has_negative) {
            continue;
        }
        if source_urls.len() < 2 {
            continue;
        }

        out.push(ConflictCluster {
            group_id: format!("news_conflict_{:03}", cluster_index),
            topic_key,
            source_urls: source_urls.into_iter().collect(),
            claims: claims.into_iter().collect(),
        });
        cluster_index = cluster_index.saturating_add(1);
    }

    out
}

pub fn cluster_lookup_by_canonical_url(
    results: &[NormalizedNewsResult],
    clusters: &[ConflictCluster],
) -> BTreeMap<String, String> {
    let mut by_topic = BTreeMap::new();
    for cluster in clusters {
        by_topic.insert(cluster.topic_key.clone(), cluster.group_id.clone());
    }

    let mut out = BTreeMap::new();
    for result in results {
        let key = topic_key(result);
        if let Some(group_id) = by_topic.get(&key) {
            out.insert(result.canonical_url.clone(), group_id.clone());
        }
    }

    out
}

fn topic_key(result: &NormalizedNewsResult) -> String {
    let mut cleaned = String::new();
    for ch in result.title.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() {
            cleaned.push(ch.to_ascii_lowercase());
        } else {
            cleaned.push(' ');
        }
    }

    cleaned
        .split_whitespace()
        .take(5)
        .collect::<Vec<&str>>()
        .join(" ")
}

fn polarity(result: &NormalizedNewsResult) -> i8 {
    let text = format!("{} {}", result.title, result.snippet).to_ascii_lowercase();
    if contains_negation(&text) {
        -1
    } else {
        1
    }
}

fn contains_negation(text: &str) -> bool {
    const TOKENS: &[&str] = &[
        " not ",
        " no ",
        " deny ",
        " denied",
        " denies",
        " reject",
        " false",
        " inaccurate",
        " didn't",
        " wont",
        " won't",
    ];

    let padded = format!(" {} ", text);
    TOKENS.iter().any(|token| padded.contains(token))
}
