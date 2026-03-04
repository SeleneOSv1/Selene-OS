#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::registry_loader::{load_contract_hash_manifest, read_text};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const REPLAY_SNAPSHOT_FILE_PATH: &str = "docs/web_search_plan/replay_expected.json";

#[derive(Debug, Clone)]
pub struct GenerateReleaseEvidenceConfig {
    pub head_commit: String,
    pub branch: String,
    pub run30_timestamp_utc: String,
    pub date_tag: String,
    pub release_lock_results_path: PathBuf,
    pub slo_lock_results_path: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResult {
    pub gate: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHarnessSummary {
    pub pass: bool,
    pub snapshot_hash_file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SloGateStatus {
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SloGateSummary {
    pub citation_coverage: SloGateStatus,
    pub refusal_correctness: SloGateStatus,
    pub freshness_compliance: SloGateStatus,
    pub determinism_replay: SloGateStatus,
    pub overall: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEvidencePack {
    pub head_commit: String,
    pub branch: String,
    pub run30_timestamp_utc: String,
    pub contract_hash_manifest_hash: String,
    pub packet_schema_hash: String,
    pub reason_code_registry_hash: String,
    pub idempotency_registry_hash: String,
    pub turn_state_machine_hash: String,
    pub handoff_map_hash: String,
    pub ownership_matrix_hash: String,
    pub compat_matrix_hash: String,
    pub ci_scripts_executed: Vec<String>,
    pub ci_gate_results: Vec<GateResult>,
    pub replay_harness_summary: ReplayHarnessSummary,
    pub slo_gate_summary: SloGateSummary,
}

pub fn generate_release_evidence_pack(
    config: &GenerateReleaseEvidenceConfig,
) -> Result<PathBuf, String> {
    validate_config(config)?;

    let release_lock = parse_release_lock_results(config.release_lock_results_path.as_path())?;
    if release_lock.overall != "PASS" {
        return Err("release lock results are not PASS".to_string());
    }
    if release_lock.head_commit != config.head_commit {
        return Err(format!(
            "release lock head mismatch expected={} actual={}",
            config.head_commit, release_lock.head_commit
        ));
    }

    let slo_lock = parse_slo_lock_results(config.slo_lock_results_path.as_path())?;
    if slo_lock.overall != "PASS" {
        return Err("slo lock results are not PASS".to_string());
    }

    let manifest = load_contract_hash_manifest()?;
    let manifest_text = read_text("CONTRACT_HASH_MANIFEST.json")?;
    let manifest_hash = sha256_hex(manifest_text.as_bytes());
    if release_lock.contract_hash_manifest_hash != manifest_hash {
        return Err(format!(
            "release lock manifest hash mismatch expected={} actual={}",
            manifest_hash, release_lock.contract_hash_manifest_hash
        ));
    }

    let replay_pass = release_lock
        .gates
        .iter()
        .find(|gate| gate.gate == "scripts/web_search_plan/check_replay_harness.sh")
        .map(|gate| gate.status == "PASS")
        .unwrap_or(false);

    let ci_scripts_executed = release_lock
        .gates
        .iter()
        .map(|gate| gate.gate.clone())
        .collect::<Vec<String>>();

    let pack = ReleaseEvidencePack {
        head_commit: config.head_commit.clone(),
        branch: config.branch.clone(),
        run30_timestamp_utc: config.run30_timestamp_utc.clone(),
        contract_hash_manifest_hash: manifest_hash,
        packet_schema_hash: manifest.hashes.packet_schema_hash,
        reason_code_registry_hash: manifest.hashes.reason_code_registry_hash,
        idempotency_registry_hash: manifest.hashes.idempotency_registry_hash,
        turn_state_machine_hash: manifest.hashes.turn_state_machine_hash,
        handoff_map_hash: manifest.hashes.handoff_map_hash,
        ownership_matrix_hash: manifest.hashes.ownership_matrix_hash,
        compat_matrix_hash: manifest.hashes.compat_matrix_hash,
        ci_scripts_executed,
        ci_gate_results: release_lock.gates,
        replay_harness_summary: ReplayHarnessSummary {
            pass: replay_pass,
            snapshot_hash_file_path: REPLAY_SNAPSHOT_FILE_PATH.to_string(),
        },
        slo_gate_summary: SloGateSummary {
            citation_coverage: slo_lock.citation_coverage,
            refusal_correctness: slo_lock.refusal_correctness,
            freshness_compliance: slo_lock.freshness_compliance,
            determinism_replay: slo_lock.determinism_replay,
            overall: slo_lock.overall,
        },
    };

    fs::create_dir_all(&config.output_dir).map_err(|error| {
        format!(
            "failed creating output dir {}: {}",
            config.output_dir.display(),
            error
        )
    })?;

    let output_path = config.output_dir.join(format!(
        "ReleaseEvidencePack_{}_{}.json",
        config.date_tag, config.head_commit
    ));
    let encoded = serde_json::to_string_pretty(&pack)
        .map_err(|error| format!("failed serializing release evidence pack: {}", error))?;
    fs::write(&output_path, encoded)
        .map_err(|error| format!("failed writing {}: {}", output_path.display(), error))?;

    Ok(output_path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseLockResults {
    head_commit: String,
    contract_hash_manifest_hash: String,
    gates: Vec<GateResult>,
    overall: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SloLockResults {
    citation_coverage: SloGateStatus,
    refusal_correctness: SloGateStatus,
    freshness_compliance: SloGateStatus,
    determinism_replay: SloGateStatus,
    overall: String,
}

fn parse_release_lock_results(path: &Path) -> Result<ReleaseLockResults, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed reading {}: {}", path.display(), error))?;
    let mut head_commit = String::new();
    let mut contract_hash_manifest_hash = String::new();
    let mut gates = Vec::new();
    let mut overall = String::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts = trimmed.split('\t').collect::<Vec<&str>>();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "HEAD_COMMIT" => {
                if parts.len() != 2 {
                    return Err(format!("invalid HEAD_COMMIT line at {}", index + 1));
                }
                head_commit = parts[1].to_string();
            }
            "CONTRACT_HASH_MANIFEST_HASH" => {
                if parts.len() != 2 {
                    return Err(format!(
                        "invalid CONTRACT_HASH_MANIFEST_HASH line at {}",
                        index + 1
                    ));
                }
                contract_hash_manifest_hash = parts[1].to_string();
            }
            "GATE" => {
                if parts.len() != 3 {
                    return Err(format!("invalid GATE line at {}", index + 1));
                }
                gates.push(GateResult {
                    gate: parts[1].to_string(),
                    status: parts[2].to_string(),
                });
            }
            "OVERALL" => {
                if parts.len() != 2 {
                    return Err(format!("invalid OVERALL line at {}", index + 1));
                }
                overall = parts[1].to_string();
            }
            _ => {
                return Err(format!(
                    "unknown token '{}' in release lock file at line {}",
                    parts[0],
                    index + 1
                ));
            }
        }
    }

    if head_commit.is_empty() {
        return Err("release lock results missing HEAD_COMMIT".to_string());
    }
    if contract_hash_manifest_hash.is_empty() {
        return Err("release lock results missing CONTRACT_HASH_MANIFEST_HASH".to_string());
    }
    if gates.is_empty() {
        return Err("release lock results missing GATE entries".to_string());
    }
    if overall.is_empty() {
        return Err("release lock results missing OVERALL".to_string());
    }

    Ok(ReleaseLockResults {
        head_commit,
        contract_hash_manifest_hash,
        gates,
        overall,
    })
}

fn parse_slo_lock_results(path: &Path) -> Result<SloLockResults, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed reading {}: {}", path.display(), error))?;
    let mut citation_coverage = None;
    let mut refusal_correctness = None;
    let mut freshness_compliance = None;
    let mut determinism_replay = None;
    let mut overall = String::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts = trimmed.split('\t').collect::<Vec<&str>>();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "SLO" => {
                if parts.len() != 4 {
                    return Err(format!("invalid SLO line at {}", index + 1));
                }
                let status = SloGateStatus {
                    status: parts[2].to_string(),
                    detail: parts[3].to_string(),
                };
                match parts[1] {
                    "citation_coverage" => citation_coverage = Some(status),
                    "refusal_correctness" => refusal_correctness = Some(status),
                    "freshness_compliance" => freshness_compliance = Some(status),
                    "determinism_replay" => determinism_replay = Some(status),
                    unknown => {
                        return Err(format!(
                            "unknown SLO key '{}' at line {}",
                            unknown,
                            index + 1
                        ))
                    }
                }
            }
            "OVERALL" => {
                if parts.len() != 2 {
                    return Err(format!("invalid OVERALL line at {}", index + 1));
                }
                overall = parts[1].to_string();
            }
            _ => {
                return Err(format!(
                    "unknown token '{}' in slo lock file at line {}",
                    parts[0],
                    index + 1
                ));
            }
        }
    }

    Ok(SloLockResults {
        citation_coverage: citation_coverage
            .ok_or_else(|| "slo lock results missing citation_coverage".to_string())?,
        refusal_correctness: refusal_correctness
            .ok_or_else(|| "slo lock results missing refusal_correctness".to_string())?,
        freshness_compliance: freshness_compliance
            .ok_or_else(|| "slo lock results missing freshness_compliance".to_string())?,
        determinism_replay: determinism_replay
            .ok_or_else(|| "slo lock results missing determinism_replay".to_string())?,
        overall: if overall.is_empty() {
            return Err("slo lock results missing OVERALL".to_string());
        } else {
            overall
        },
    })
}

fn validate_config(config: &GenerateReleaseEvidenceConfig) -> Result<(), String> {
    if config.head_commit.trim().is_empty() {
        return Err("head_commit must not be empty".to_string());
    }
    if config.branch.trim().is_empty() {
        return Err("branch must not be empty".to_string());
    }
    if config.run30_timestamp_utc.trim().is_empty() {
        return Err("run30_timestamp_utc must not be empty".to_string());
    }
    if config.date_tag.trim().is_empty() {
        return Err("date_tag must not be empty".to_string());
    }
    Ok(())
}
