#![forbid(unsafe_code)]

use crate::web_search_plan::competitive::schema::{
    CompetitiveEntity, COMPETITIVE_NORM_VERSION,
};
use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::structured::types::StructuredRow;
use std::collections::{BTreeMap, BTreeSet};
use url::Url;

#[derive(Debug, Clone)]
pub struct EntityIndex {
    pub entities: Vec<CompetitiveEntity>,
    target_canonical: String,
    canonical_to_id: BTreeMap<String, String>,
}

impl EntityIndex {
    pub fn entity_id_for_name(&self, name: &str) -> Option<&str> {
        let canonical = canonicalize_entity_name(name);
        self.canonical_to_id.get(&canonical).map(String::as_str)
    }

    pub fn target_entity_id(&self) -> Option<&str> {
        self.canonical_to_id
            .get(&self.target_canonical)
            .map(String::as_str)
    }

    pub fn target_canonical(&self) -> &str {
        self.target_canonical.as_str()
    }
}

pub fn build_entity_index(rows: &[StructuredRow], target_entity: &str) -> EntityIndex {
    #[derive(Debug, Default)]
    struct Accumulator {
        display_name: String,
        max_as_of_ms: Option<i64>,
        sources: BTreeSet<String>,
        domains: BTreeSet<String>,
    }

    let target_canonical = canonicalize_entity_name(target_entity);
    let mut by_canonical: BTreeMap<String, Accumulator> = BTreeMap::new();

    for row in rows {
        let canonical = canonicalize_entity_name(&row.entity);
        if canonical.is_empty() {
            continue;
        }
        let entry = by_canonical.entry(canonical.clone()).or_default();
        if entry.display_name.is_empty() {
            entry.display_name = canonical.clone();
        }
        entry.max_as_of_ms = match (entry.max_as_of_ms, row.as_of_ms) {
            (Some(current), Some(next)) => Some(current.max(next)),
            (Some(current), None) => Some(current),
            (None, Some(next)) => Some(next),
            (None, None) => None,
        };

        let source_ref = row.source_ref.trim();
        if !source_ref.is_empty() {
            entry.sources.insert(source_ref.to_string());
        } else if !row.source_url.trim().is_empty() {
            entry.sources.insert(row.source_url.trim().to_string());
        }

        if let Some(host) = host_from_url(&row.source_url) {
            entry.domains.insert(host);
        }
    }

    if target_canonical.is_empty() {
        return EntityIndex {
            entities: vec![],
            target_canonical,
            canonical_to_id: BTreeMap::new(),
        };
    }

    by_canonical
        .entry(target_canonical.clone())
        .or_insert_with(Accumulator::default);

    let mut entities = Vec::with_capacity(by_canonical.len());
    let mut canonical_to_id = BTreeMap::new();

    for (canonical, acc) in by_canonical {
        let chosen_domain = acc.domains.iter().next().cloned().unwrap_or_default();
        let entity_id = entity_id_for(canonical.as_str(), chosen_domain.as_str());
        canonical_to_id.insert(canonical.clone(), entity_id.clone());
        entities.push(CompetitiveEntity {
            entity_id,
            name: acc.display_name,
            region: None,
            as_of_ms: acc.max_as_of_ms,
            sources: acc.sources.into_iter().collect(),
        });
    }
    entities.sort_by(|left, right| left.name.cmp(&right.name));

    EntityIndex {
        entities,
        target_canonical,
        canonical_to_id,
    }
}

pub fn canonicalize_entity_name(raw: &str) -> String {
    raw.split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

pub fn entity_id_for(canonical_name: &str, domain_hint: &str) -> String {
    let digest = sha256_hex(
        format!(
            "{}|{}|{}",
            canonical_name.trim().to_lowercase(),
            domain_hint.trim().to_lowercase(),
            COMPETITIVE_NORM_VERSION
        )
        .as_bytes(),
    );
    format!("entity_{}", &digest[..16])
}

fn host_from_url(raw_url: &str) -> Option<String> {
    let parsed = Url::parse(raw_url).ok()?;
    parsed.host_str().map(|host| host.to_ascii_lowercase())
}
