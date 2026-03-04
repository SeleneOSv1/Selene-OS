#![forbid(unsafe_code)]

use crate::web_search_plan::enterprise::mode_router::EnterpriseMode;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterpriseProvenance {
    pub evidence_hash: String,
    pub computation_hash: Option<String>,
    pub output_packet_hashes: BTreeMap<String, String>,
    pub reason_codes: Vec<String>,
    pub mode_selected: String,
}

pub fn build_enterprise_provenance(
    mode: EnterpriseMode,
    evidence_packet: &Value,
    computation_packet: Option<&Value>,
    outputs: &[(&str, &Value)],
    reason_codes: &[String],
) -> Result<EnterpriseProvenance, String> {
    let evidence_hash = hash_canonical_json(evidence_packet)?;
    let computation_hash = computation_packet
        .map(hash_canonical_json)
        .transpose()?;

    let mut output_packet_hashes = BTreeMap::new();
    for (name, packet) in outputs {
        output_packet_hashes.insert((*name).to_string(), hash_canonical_json(packet)?);
    }

    let mut deduped_reason_codes = reason_codes.to_vec();
    deduped_reason_codes.sort();
    deduped_reason_codes.dedup();

    Ok(EnterpriseProvenance {
        evidence_hash,
        computation_hash,
        output_packet_hashes,
        reason_codes: deduped_reason_codes,
        mode_selected: mode.as_str().to_string(),
    })
}
