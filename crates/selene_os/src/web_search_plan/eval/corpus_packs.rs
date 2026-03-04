#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::docs_dir;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_CORPUS_PACK_FILES: &[&str] = &[
    "core.json",
    "web.json",
    "news.json",
    "url_fetch.json",
    "structured.json",
    "compliance.json",
    "realtime.json",
    "competitive.json",
    "risk.json",
    "temporal.json",
    "merge.json",
];

const EVAL_DIR: &str = "eval";
const CORPUS_PACKS_DIR: &str = "corpus_packs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCase {
    pub case_id: String,
    pub fixture_case_id: String,
    pub mode: String,
    pub importance_tier: String,
    pub expected_outcome: String,
    pub fixed_time_ms: i64,
    pub fixed_policy_snapshot_id: String,
    pub expected_snapshot_hash: String,
    #[serde(default)]
    pub expect_stale_refusal: bool,
    #[serde(default)]
    pub expect_conflict_flag: bool,
    #[serde(default)]
    pub expect_trust_filtering: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCorpusPack {
    pub pack_id: String,
    pub pack_version: String,
    pub cases: Vec<EvalCase>,
}

pub fn eval_docs_dir() -> PathBuf {
    docs_dir().join(EVAL_DIR)
}

pub fn corpus_pack_path(file_name: &str) -> PathBuf {
    eval_docs_dir().join(CORPUS_PACKS_DIR).join(file_name)
}

pub fn load_corpus_pack(file_name: &str) -> Result<EvalCorpusPack, String> {
    let full_path = corpus_pack_path(file_name);
    let text = fs::read_to_string(&full_path)
        .map_err(|e| format!("failed to read {}: {}", full_path.display(), e))?;
    serde_json::from_str::<EvalCorpusPack>(&text)
        .map_err(|e| format!("invalid corpus pack JSON {}: {}", full_path.display(), e))
}

pub fn load_default_corpus_packs() -> Result<Vec<EvalCorpusPack>, String> {
    let mut packs = Vec::new();
    for file_name in DEFAULT_CORPUS_PACK_FILES {
        let pack = load_corpus_pack(file_name)?;
        validate_corpus_pack(&pack)?;
        packs.push(pack);
    }
    Ok(packs)
}

pub fn validate_corpus_pack(pack: &EvalCorpusPack) -> Result<(), String> {
    if pack.pack_id.trim().is_empty() {
        return Err("eval pack_id must not be empty".to_string());
    }
    if pack.pack_version.trim().is_empty() {
        return Err(format!(
            "eval pack {} pack_version must not be empty",
            pack.pack_id
        ));
    }
    if pack.cases.is_empty() {
        return Err(format!(
            "eval pack {} must include at least one case",
            pack.pack_id
        ));
    }

    let mut seen_case_ids = BTreeSet::new();
    let mut sorted_case_ids = pack
        .cases
        .iter()
        .map(|case| case.case_id.clone())
        .collect::<Vec<String>>();
    sorted_case_ids.sort();

    for case in &pack.cases {
        if case.case_id.trim().is_empty() {
            return Err(format!("eval pack {} contains empty case_id", pack.pack_id));
        }
        if case.fixture_case_id.trim().is_empty() {
            return Err(format!(
                "eval case {} fixture_case_id must not be empty",
                case.case_id
            ));
        }
        if !seen_case_ids.insert(case.case_id.clone()) {
            return Err(format!(
                "eval pack {} has duplicate case_id {}",
                pack.pack_id, case.case_id
            ));
        }

        validate_mode(case.case_id.as_str(), case.mode.as_str())?;
        validate_tier(case.case_id.as_str(), case.importance_tier.as_str())?;
        validate_expected_outcome(case.case_id.as_str(), case.expected_outcome.as_str())?;

        if case.fixed_policy_snapshot_id.trim().is_empty() {
            return Err(format!(
                "eval case {} fixed_policy_snapshot_id must not be empty",
                case.case_id
            ));
        }

        if !is_hex_sha256(case.expected_snapshot_hash.as_str()) {
            return Err(format!(
                "eval case {} expected_snapshot_hash must be a 64-char lowercase hex sha256",
                case.case_id
            ));
        }
    }

    let case_ids = pack
        .cases
        .iter()
        .map(|case| case.case_id.clone())
        .collect::<Vec<String>>();
    if case_ids != sorted_case_ids {
        return Err(format!(
            "eval pack {} case ordering must be lexical by case_id",
            pack.pack_id
        ));
    }

    Ok(())
}

pub fn merge_cases(packs: &[EvalCorpusPack]) -> Result<Vec<EvalCase>, String> {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();

    for pack in packs {
        for case in &pack.cases {
            if !seen.insert(case.case_id.clone()) {
                return Err(format!(
                    "duplicate case_id across packs is not allowed: {}",
                    case.case_id
                ));
            }
            merged.push(case.clone());
        }
    }

    merged.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(merged)
}

fn validate_mode(case_id: &str, mode: &str) -> Result<(), String> {
    if [
        "web",
        "news",
        "url_fetch",
        "structured",
        "compliance",
        "realtime",
        "competitive",
        "risk",
        "temporal",
        "merge",
    ]
    .contains(&mode)
    {
        Ok(())
    } else {
        Err(format!("eval case {} has unsupported mode {}", case_id, mode))
    }
}

fn validate_tier(case_id: &str, tier: &str) -> Result<(), String> {
    if ["low", "medium", "high"].contains(&tier) {
        Ok(())
    } else {
        Err(format!(
            "eval case {} has unsupported importance_tier {}",
            case_id, tier
        ))
    }
}

fn validate_expected_outcome(case_id: &str, expected_outcome: &str) -> Result<(), String> {
    if ["answer", "refusal"].contains(&expected_outcome) {
        Ok(())
    } else {
        Err(format!(
            "eval case {} has unsupported expected_outcome {}",
            case_id, expected_outcome
        ))
    }
}

fn is_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}
