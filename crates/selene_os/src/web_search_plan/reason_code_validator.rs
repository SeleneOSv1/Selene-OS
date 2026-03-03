#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::ReasonCodeRegistry;
use std::collections::BTreeSet;

pub fn validate_reason_code_registry(registry: &ReasonCodeRegistry) -> Result<(), String> {
    if registry.registry_version.trim().is_empty() {
        return Err("reason code registry_version must not be empty".to_string());
    }
    let mut seen = BTreeSet::new();
    for entry in &registry.reason_codes {
        if entry.code_id.trim().is_empty() {
            return Err("reason code_id must not be empty".to_string());
        }
        if !seen.insert(entry.code_id.clone()) {
            return Err(format!("duplicate reason code_id: {}", entry.code_id));
        }
        validate_enum(
            "category",
            &entry.category,
            &[
                "input",
                "policy",
                "retrieval",
                "evidence",
                "synthesis",
                "quota",
                "compliance",
                "system",
            ],
        )?;
        validate_enum(
            "severity",
            &entry.severity,
            &["info", "warn", "error", "critical"],
        )?;
        validate_enum(
            "retry_class",
            &entry.retry_class,
            &[
                "never",
                "immediate_allowed",
                "cooldown_required",
                "builder_action_required",
            ],
        )?;
        if entry.user_message_template.trim().is_empty() {
            return Err(format!(
                "reason code {} has empty user_message_template",
                entry.code_id
            ));
        }
        let lower = entry.user_message_template.to_ascii_lowercase();
        for forbidden in &registry.wording_contract.forbidden_substrings {
            if !forbidden.trim().is_empty() && lower.contains(&forbidden.to_ascii_lowercase()) {
                return Err(format!(
                    "reason code {} template contains forbidden substring {}",
                    entry.code_id, forbidden
                ));
            }
        }
        if entry.audit_required_fields.is_empty() {
            return Err(format!(
                "reason code {} must declare audit_required_fields",
                entry.code_id
            ));
        }
        if entry.owner_engine.trim().is_empty() || entry.introduced_in_version.trim().is_empty() {
            return Err(format!(
                "reason code {} missing owner_engine or introduced_in_version",
                entry.code_id
            ));
        }
    }
    Ok(())
}

pub fn validate_reason_codes_registered(
    reason_codes: &[String],
    registry: &ReasonCodeRegistry,
) -> Result<(), String> {
    let known: BTreeSet<&str> = registry
        .reason_codes
        .iter()
        .map(|entry| entry.code_id.as_str())
        .collect();
    for code in reason_codes {
        if !known.contains(code.as_str()) {
            return Err(format!("unknown reason code: {}", code));
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
