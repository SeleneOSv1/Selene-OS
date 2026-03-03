#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySnapshot {
    pub case_id: String,
    pub evidence_hash: String,
    pub synthesis_hash: Option<String>,
    pub write_hash: Option<String>,
    pub audit_hash: Option<String>,
    pub stop_reason: String,
    pub reason_codes: Vec<String>,
}

pub fn build_snapshot(
    case_id: &str,
    stop_reason: &str,
    evidence_packet: &Value,
    synthesis_packet: Option<&Value>,
    write_packet: Option<&Value>,
    audit_packet: Option<&Value>,
) -> Result<ReplaySnapshot, String> {
    let evidence_hash = hash_canonical_json(evidence_packet)?;
    let synthesis_hash = synthesis_packet.map(hash_canonical_json).transpose()?;
    let write_hash = write_packet.map(hash_canonical_json).transpose()?;
    let audit_hash = audit_packet.map(hash_canonical_json).transpose()?;

    let reason_codes = collect_reason_codes(synthesis_packet, audit_packet)?;

    Ok(ReplaySnapshot {
        case_id: case_id.to_string(),
        evidence_hash,
        synthesis_hash,
        write_hash,
        audit_hash,
        stop_reason: stop_reason.to_string(),
        reason_codes,
    })
}

pub fn hash_canonical_json(value: &Value) -> Result<String, String> {
    let canonical = canonicalize_value(value);
    let encoded =
        serde_json::to_string(&canonical).map_err(|e| format!("canonical json encode failed: {}", e))?;
    Ok(sha256_hex(encoded.as_bytes()))
}

fn collect_reason_codes(
    synthesis_packet: Option<&Value>,
    audit_packet: Option<&Value>,
) -> Result<Vec<String>, String> {
    let mut deduped = BTreeSet::new();

    if let Some(packet) = synthesis_packet {
        if let Some(codes) = packet.get("reason_codes") {
            let parsed = parse_reason_codes(codes)?;
            for code in parsed {
                deduped.insert(code);
            }
        }
    }

    if let Some(packet) = audit_packet {
        if let Some(codes) = packet.get("reason_codes") {
            let parsed = parse_reason_codes(codes)?;
            for code in parsed {
                deduped.insert(code);
            }
        }
    }

    Ok(deduped.into_iter().collect())
}

fn parse_reason_codes(value: &Value) -> Result<Vec<String>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "reason_codes must be an array".to_string())?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| "reason_codes entry must be a string".to_string())
                .map(ToString::to_string)
        })
        .collect()
}

pub fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<String>>();
            keys.sort();
            let mut out = Map::new();
            for key in keys {
                if let Some(nested) = map.get(&key) {
                    out.insert(key, canonicalize_value(nested));
                }
            }
            Value::Object(out)
        }
        Value::Array(entries) => Value::Array(entries.iter().map(canonicalize_value).collect()),
        _ => value.clone(),
    }
}
