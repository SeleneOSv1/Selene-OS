#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::multihop::hop_plan::HopPlan;
use crate::web_search_plan::multihop::hop_runner::HopExecutionRecord;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopProof {
    pub hop_index: usize,
    pub sub_query_hash: String,
    pub evidence_hash: String,
    pub provider_runs_count: usize,
    pub sources_count: usize,
    pub success: bool,
    pub reason_code: Option<String>,
    pub time_spent_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopProofChain {
    pub plan_id: String,
    pub hops_planned: usize,
    pub hops_executed: usize,
    pub hop_proofs: Vec<HopProof>,
    pub stop_reason: String,
    pub reason_codes: Vec<String>,
    pub cycle_detected: bool,
    pub complete: bool,
    pub proof_chain_hash: String,
}

pub fn build_hop_proof_chain(
    plan: &HopPlan,
    hop_records: &[HopExecutionRecord],
    stop_reason: &str,
    reason_codes: &[String],
    cycle_detected: bool,
) -> Result<HopProofChain, String> {
    let mut hop_proofs = hop_records
        .iter()
        .map(|record| HopProof {
            hop_index: record.hop_index,
            sub_query_hash: sha256_hex(normalize_query(record.query.as_str()).as_bytes()),
            evidence_hash: record.evidence_hash.clone(),
            provider_runs_count: record.provider_runs.len(),
            sources_count: record.source_urls.len(),
            success: record.success,
            reason_code: record.reason_code.clone(),
            time_spent_ms: record.time_spent_ms,
        })
        .collect::<Vec<HopProof>>();
    hop_proofs.sort_by_key(|proof| proof.hop_index);

    let complete =
        stop_reason == "success" && !cycle_detected && hop_records.len() == plan.hops.len();

    let unsigned_payload = json!({
        "plan_id": plan.plan_id,
        "hops_planned": plan.hops.len(),
        "hops_executed": hop_records.len(),
        "hop_proofs": hop_proofs,
        "stop_reason": stop_reason,
        "reason_codes": reason_codes,
        "cycle_detected": cycle_detected,
        "complete": complete
    });
    let proof_chain_hash = hash_canonical_json(&unsigned_payload)?;

    Ok(HopProofChain {
        plan_id: plan.plan_id.clone(),
        hops_planned: plan.hops.len(),
        hops_executed: hop_records.len(),
        hop_proofs,
        stop_reason: stop_reason.to_string(),
        reason_codes: reason_codes.to_vec(),
        cycle_detected,
        complete,
        proof_chain_hash,
    })
}

pub fn can_mark_complete(chain: &HopProofChain) -> bool {
    chain.complete
        && chain.stop_reason == "success"
        && chain.hops_executed == chain.hops_planned
        && !chain.cycle_detected
}

pub fn append_hop_proof_chain_to_turn_state_transition(
    transition: &Value,
    chain: &HopProofChain,
) -> Value {
    let mut payload = match transition {
        Value::Object(map) => map.clone(),
        Value::String(raw) => {
            let mut map = Map::new();
            map.insert("state_path".to_string(), Value::String(raw.clone()));
            map
        }
        _ => {
            let mut map = Map::new();
            map.insert("state_path".to_string(), transition.clone());
            map
        }
    };

    payload.insert(
        "hop_proof_chain".to_string(),
        serde_json::to_value(chain).unwrap_or_else(|_| json!({"invalid_hop_proof_chain": true})),
    );
    Value::Object(payload)
}

fn normalize_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn hash_canonical_json(value: &Value) -> Result<String, String> {
    let canonical = canonicalize_value(value);
    let encoded =
        serde_json::to_string(&canonical).map_err(|e| format!("canonical serialization failed: {}", e))?;
    Ok(sha256_hex(encoded.as_bytes()))
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort();
            let mut out = Map::new();
            for key in keys {
                if let Some(inner) = map.get(&key) {
                    out.insert(key, canonicalize_value(inner));
                }
            }
            Value::Object(out)
        }
        Value::Array(entries) => Value::Array(entries.iter().map(canonicalize_value).collect()),
        _ => value.clone(),
    }
}
