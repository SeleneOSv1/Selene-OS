#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::docs_dir;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub const REPLAY_CORPUS_FILE: &str = "replay_corpus.json";
pub const REPLAY_EXPECTED_FILE: &str = "replay_expected.json";
pub const REPLAY_FIXTURE_DIR: &str = "replay_fixtures";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedInputs {
    pub fixed_time_ms: i64,
    pub fixed_policy_snapshot_id: String,
    pub fixed_provider_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCase {
    pub case_id: String,
    pub mode: String,
    pub query: String,
    pub importance_tier: String,
    pub expected_outcome: String,
    pub fixed_inputs: FixedInputs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCorpus {
    pub corpus_version: String,
    pub cases: Vec<ReplayCase>,
}

pub fn replay_docs_dir() -> PathBuf {
    docs_dir()
}

pub fn replay_fixture_path(case_id: &str) -> PathBuf {
    replay_docs_dir()
        .join(REPLAY_FIXTURE_DIR)
        .join(format!("{}.json", case_id))
}

pub fn load_replay_corpus() -> Result<ReplayCorpus, String> {
    let full_path = replay_docs_dir().join(REPLAY_CORPUS_FILE);
    let text = fs::read_to_string(&full_path)
        .map_err(|e| format!("failed to read {}: {}", full_path.display(), e))?;
    serde_json::from_str::<ReplayCorpus>(&text)
        .map_err(|e| format!("invalid replay corpus JSON {}: {}", full_path.display(), e))
}

pub fn validate_replay_corpus(corpus: &ReplayCorpus) -> Result<(), String> {
    if corpus.corpus_version.trim().is_empty() {
        return Err("replay corpus_version must not be empty".to_string());
    }
    if corpus.cases.is_empty() {
        return Err("replay corpus must include at least one case".to_string());
    }

    let mut seen_case_ids = BTreeSet::new();
    for case in &corpus.cases {
        if case.case_id.trim().is_empty() {
            return Err("replay case_id must not be empty".to_string());
        }
        if !seen_case_ids.insert(case.case_id.clone()) {
            return Err(format!("duplicate replay case_id {}", case.case_id));
        }
        if case.query.trim().is_empty() {
            return Err(format!("replay case {} query must not be empty", case.case_id));
        }

        validate_mode(case.case_id.as_str(), case.mode.as_str())?;
        validate_tier(case.case_id.as_str(), case.importance_tier.as_str())?;
        validate_expected_outcome(case.case_id.as_str(), case.expected_outcome.as_str())?;

        if case.fixed_inputs.fixed_provider_mode != "fixture" {
            return Err(format!(
                "replay case {} fixed_provider_mode must be fixture",
                case.case_id
            ));
        }
        if case.fixed_inputs.fixed_policy_snapshot_id.trim().is_empty() {
            return Err(format!(
                "replay case {} fixed_policy_snapshot_id must not be empty",
                case.case_id
            ));
        }
    }

    Ok(())
}

fn validate_mode(case_id: &str, mode: &str) -> Result<(), String> {
    if ["web", "news", "url_fetch", "images", "video"].contains(&mode) {
        Ok(())
    } else {
        Err(format!("replay case {} has unsupported mode {}", case_id, mode))
    }
}

fn validate_tier(case_id: &str, tier: &str) -> Result<(), String> {
    if ["low", "medium", "high"].contains(&tier) {
        Ok(())
    } else {
        Err(format!(
            "replay case {} has unsupported importance_tier {}",
            case_id, tier
        ))
    }
}

fn validate_expected_outcome(case_id: &str, expected_outcome: &str) -> Result<(), String> {
    if ["answer", "refusal"].contains(&expected_outcome) {
        Ok(())
    } else {
        Err(format!(
            "replay case {} has unsupported expected_outcome {}",
            case_id, expected_outcome
        ))
    }
}
