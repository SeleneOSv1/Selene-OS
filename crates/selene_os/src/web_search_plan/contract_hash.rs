#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::{
    load_contract_hash_manifest, read_text, ContractHashManifest, ContractHashValues,
};
use sha2::{Digest, Sha256};

const HASH_TARGETS: &[(&str, &str)] = &[
    ("packet_schema_hash", "PACKET_SCHEMAS.json"),
    ("reason_code_registry_hash", "REASON_CODES.json"),
    ("idempotency_registry_hash", "IDEMPOTENCY_KEYS.json"),
    ("turn_state_machine_hash", "TURN_STATE_MACHINE.json"),
    ("handoff_map_hash", "HANDOFF_MAP.json"),
    ("ownership_matrix_hash", "OWNERSHIP_MATRIX.json"),
    ("compat_matrix_hash", "BACKWARD_COMPAT_MATRIX.md"),
];

pub fn compute_contract_hash_values() -> Result<ContractHashValues, String> {
    let mut packet_schema_hash = String::new();
    let mut reason_code_registry_hash = String::new();
    let mut idempotency_registry_hash = String::new();
    let mut turn_state_machine_hash = String::new();
    let mut handoff_map_hash = String::new();
    let mut ownership_matrix_hash = String::new();
    let mut compat_matrix_hash = String::new();

    for (key, relative) in HASH_TARGETS {
        let text = read_text(relative)?;
        let hash = sha256_hex(text.as_bytes());
        match *key {
            "packet_schema_hash" => packet_schema_hash = hash,
            "reason_code_registry_hash" => reason_code_registry_hash = hash,
            "idempotency_registry_hash" => idempotency_registry_hash = hash,
            "turn_state_machine_hash" => turn_state_machine_hash = hash,
            "handoff_map_hash" => handoff_map_hash = hash,
            "ownership_matrix_hash" => ownership_matrix_hash = hash,
            "compat_matrix_hash" => compat_matrix_hash = hash,
            _ => return Err(format!("unexpected hash key {}", key)),
        }
    }

    Ok(ContractHashValues {
        packet_schema_hash,
        reason_code_registry_hash,
        idempotency_registry_hash,
        turn_state_machine_hash,
        handoff_map_hash,
        ownership_matrix_hash,
        compat_matrix_hash,
    })
}

pub fn computed_manifest() -> Result<ContractHashManifest, String> {
    Ok(ContractHashManifest {
        manifest_version: "1.0.0".to_string(),
        hash_algorithm: "sha256".to_string(),
        hashes: compute_contract_hash_values()?,
    })
}

pub fn verify_contract_hash_manifest() -> Result<(), String> {
    let expected = load_contract_hash_manifest()?;
    let actual = computed_manifest()?;

    if expected.manifest_version != actual.manifest_version {
        return Err(format!(
            "manifest_version mismatch expected={} actual={}",
            expected.manifest_version, actual.manifest_version
        ));
    }
    if expected.hash_algorithm != actual.hash_algorithm {
        return Err(format!(
            "hash_algorithm mismatch expected={} actual={}",
            expected.hash_algorithm, actual.hash_algorithm
        ));
    }
    if expected.hashes.packet_schema_hash != actual.hashes.packet_schema_hash
        || expected.hashes.reason_code_registry_hash != actual.hashes.reason_code_registry_hash
        || expected.hashes.idempotency_registry_hash != actual.hashes.idempotency_registry_hash
        || expected.hashes.turn_state_machine_hash != actual.hashes.turn_state_machine_hash
        || expected.hashes.handoff_map_hash != actual.hashes.handoff_map_hash
        || expected.hashes.ownership_matrix_hash != actual.hashes.ownership_matrix_hash
        || expected.hashes.compat_matrix_hash != actual.hashes.compat_matrix_hash
    {
        return Err(format!(
            "contract hash manifest mismatch expected={:?} actual={:?}",
            expected.hashes, actual.hashes
        ));
    }

    Ok(())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
