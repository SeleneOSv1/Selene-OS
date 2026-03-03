#![forbid(unsafe_code)]

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct SchemaPolicy {
    pub default_additional_properties: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketVersionSchema {
    pub schema_version: String,
    #[serde(default)]
    pub additional_properties: bool,
    pub required_fields: Vec<String>,
    pub field_types: BTreeMap<String, String>,
    #[serde(default)]
    pub field_enums: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketSchemaDefinition {
    pub packet_name: String,
    pub consumer_expected_version: String,
    pub versions: Vec<PacketVersionSchema>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketSchemaRegistry {
    pub registry_version: String,
    pub schema_policy: SchemaPolicy,
    pub packets: Vec<PacketSchemaDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReasonCodeEntry {
    pub code_id: String,
    pub owner_engine: String,
    pub category: String,
    pub severity: String,
    pub retry_class: String,
    pub user_message_template: String,
    pub introduced_in_version: String,
    pub audit_required_fields: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReasonCodeWordingContract {
    pub forbidden_substrings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReasonCodeRegistry {
    pub registry_version: String,
    pub wording_contract: ReasonCodeWordingContract,
    pub reason_codes: Vec<ReasonCodeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdempotencyKeyRecipe {
    pub description: String,
    pub canonical_fields: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdempotencyWritePath {
    pub write_path_name: String,
    pub owning_engine: String,
    pub idempotency_key_recipe: IdempotencyKeyRecipe,
    pub uniqueness_scope: String,
    pub persistence_table: String,
    pub duplicate_behavior: String,
    pub introduced_in_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedUniqueIndex {
    pub write_path_name: String,
    pub index_name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdempotencyRegistry {
    pub registry_version: String,
    pub write_paths: Vec<IdempotencyWritePath>,
    pub expected_unique_indexes: Vec<ExpectedUniqueIndex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurnStateMachine {
    pub registry_version: String,
    pub states: Vec<String>,
    pub allowed_transitions: Vec<StateTransition>,
    pub failure_state: String,
    pub gate_order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HandoffEntry {
    pub producer_engine: String,
    pub packet_type: String,
    pub consumer_engine: String,
    pub rule: String,
    pub authority: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HandoffMap {
    pub registry_version: String,
    pub handoffs: Vec<HandoffEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OwnershipEntry {
    pub engine_id: String,
    pub authority: String,
    pub allowed_actions: Vec<String>,
    pub must_not_do: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OwnershipMatrix {
    pub registry_version: String,
    pub engines: Vec<OwnershipEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractHashValues {
    pub packet_schema_hash: String,
    pub reason_code_registry_hash: String,
    pub idempotency_registry_hash: String,
    pub turn_state_machine_hash: String,
    pub handoff_map_hash: String,
    pub ownership_matrix_hash: String,
    pub compat_matrix_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractHashManifest {
    pub manifest_version: String,
    pub hash_algorithm: String,
    pub hashes: ContractHashValues,
}

pub fn docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/web_search_plan")
        .components()
        .collect()
}

pub fn fixtures_dir(kind: &str) -> PathBuf {
    docs_dir().join("fixtures").join(kind)
}

pub fn read_text(relative_path: &str) -> Result<String, String> {
    let full = docs_dir().join(relative_path);
    fs::read_to_string(&full).map_err(|e| format!("failed to read {}: {}", full.display(), e))
}

pub fn read_json_value(relative_path: &str) -> Result<Value, String> {
    let text = read_text(relative_path)?;
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON in {}: {}", relative_path, e))
}

pub fn parse_json_file<T: DeserializeOwned>(relative_path: &str) -> Result<T, String> {
    let text = read_text(relative_path)?;
    serde_json::from_str::<T>(&text)
        .map_err(|e| format!("failed to parse {}: {}", relative_path, e))
}

pub fn load_packet_schema_registry() -> Result<PacketSchemaRegistry, String> {
    parse_json_file("PACKET_SCHEMAS.json")
}

pub fn load_reason_code_registry() -> Result<ReasonCodeRegistry, String> {
    parse_json_file("REASON_CODES.json")
}

pub fn load_idempotency_registry() -> Result<IdempotencyRegistry, String> {
    parse_json_file("IDEMPOTENCY_KEYS.json")
}

pub fn load_turn_state_machine() -> Result<TurnStateMachine, String> {
    parse_json_file("TURN_STATE_MACHINE.json")
}

pub fn load_handoff_map() -> Result<HandoffMap, String> {
    parse_json_file("HANDOFF_MAP.json")
}

pub fn load_ownership_matrix() -> Result<OwnershipMatrix, String> {
    parse_json_file("OWNERSHIP_MATRIX.json")
}

pub fn load_contract_hash_manifest() -> Result<ContractHashManifest, String> {
    parse_json_file("CONTRACT_HASH_MANIFEST.json")
}
