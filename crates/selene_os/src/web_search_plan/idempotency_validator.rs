#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::IdempotencyRegistry;
use std::collections::BTreeSet;

pub fn validate_idempotency_registry(registry: &IdempotencyRegistry) -> Result<(), String> {
    if registry.registry_version.trim().is_empty() {
        return Err("idempotency registry_version must not be empty".to_string());
    }

    let required_write_paths = [
        "audit_append",
        "evidence_persist",
        "policy_snapshot_persist",
    ];
    let mut seen_paths = BTreeSet::new();

    for write_path in &registry.write_paths {
        if write_path.write_path_name.trim().is_empty() {
            return Err("write_path_name must not be empty".to_string());
        }
        if !seen_paths.insert(write_path.write_path_name.clone()) {
            return Err(format!(
                "duplicate write_path_name {}",
                write_path.write_path_name
            ));
        }
        if write_path
            .idempotency_key_recipe
            .canonical_fields
            .is_empty()
        {
            return Err(format!(
                "write path {} canonical_fields must not be empty",
                write_path.write_path_name
            ));
        }
        if write_path
            .idempotency_key_recipe
            .description
            .trim()
            .is_empty()
        {
            return Err(format!(
                "write path {} recipe description must not be empty",
                write_path.write_path_name
            ));
        }
        validate_enum(
            "uniqueness_scope",
            &write_path.uniqueness_scope,
            &["global", "per_session", "per_entity", "per_tenant"],
        )?;
        validate_enum(
            "duplicate_behavior",
            &write_path.duplicate_behavior,
            &[
                "return_existing",
                "no_op_success",
                "hard_conflict",
                "merge_if_safe",
            ],
        )?;

        for field in &write_path.idempotency_key_recipe.canonical_fields {
            let normalized = field.to_ascii_lowercase();
            if normalized.contains("timestamp")
                || normalized.contains("random")
                || normalized.contains("nonce")
                || normalized == "created_at_ms"
            {
                return Err(format!(
                    "write path {} uses non-deterministic field {}",
                    write_path.write_path_name, field
                ));
            }
        }
    }

    for required in required_write_paths {
        if !seen_paths.contains(required) {
            return Err(format!("missing required write path {}", required));
        }
    }

    for index in &registry.expected_unique_indexes {
        if !seen_paths.contains(index.write_path_name.as_str()) {
            return Err(format!(
                "expected_unique_index references unknown write_path_name {}",
                index.write_path_name
            ));
        }
        if index.index_name.trim().is_empty() || index.columns.is_empty() {
            return Err(format!(
                "expected_unique_index for {} is incomplete",
                index.write_path_name
            ));
        }
    }

    Ok(())
}

fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.iter().any(|entry| value == *entry) {
        Ok(())
    } else {
        Err(format!("invalid {} value: {}", field, value))
    }
}
