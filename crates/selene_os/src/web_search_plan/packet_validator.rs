#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::{PacketSchemaDefinition, PacketSchemaRegistry};
use serde_json::Value;
use std::collections::BTreeSet;

pub fn validate_packet_schema_registry(registry: &PacketSchemaRegistry) -> Result<(), String> {
    if registry.registry_version.trim().is_empty() {
        return Err("packet schema registry_version must not be empty".to_string());
    }
    for packet in &registry.packets {
        if packet.packet_name.trim().is_empty() {
            return Err("packet_name must not be empty".to_string());
        }
        if packet.versions.is_empty() {
            return Err(format!(
                "packet {} must include at least one schema version",
                packet.packet_name
            ));
        }
        let has_expected = packet
            .versions
            .iter()
            .any(|v| v.schema_version == packet.consumer_expected_version);
        if !has_expected {
            return Err(format!(
                "packet {} consumer_expected_version {} missing from versions",
                packet.packet_name, packet.consumer_expected_version
            ));
        }
        for version in &packet.versions {
            for required in &version.required_fields {
                if !version.field_types.contains_key(required) {
                    return Err(format!(
                        "packet {} schema {} missing field_types entry for required field {}",
                        packet.packet_name, version.schema_version, required
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn validate_packet(
    packet_name: &str,
    packet: &Value,
    registry: &PacketSchemaRegistry,
) -> Result<(), String> {
    let def = registry
        .packets
        .iter()
        .find(|p| p.packet_name == packet_name)
        .ok_or_else(|| format!("unknown packet name: {}", packet_name))?;

    let schema_version = packet
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("packet {} missing schema_version string", packet_name))?;

    let version = def
        .versions
        .iter()
        .find(|v| v.schema_version == schema_version)
        .ok_or_else(|| {
            format!(
                "unknown schema_version {} for packet {}",
                schema_version, packet_name
            )
        })?;

    if schema_version != def.consumer_expected_version {
        return Err(format!(
            "packet {} schema_version {} does not match consumer_expected_version {}",
            packet_name, schema_version, def.consumer_expected_version
        ));
    }

    let obj = packet
        .as_object()
        .ok_or_else(|| format!("packet {} must be a JSON object", packet_name))?;

    for required in &version.required_fields {
        if !obj.contains_key(required) {
            return Err(format!(
                "packet {} missing required field {}",
                packet_name, required
            ));
        }
    }

    let allow_additional = if version.additional_properties {
        true
    } else {
        registry.schema_policy.default_additional_properties
    };

    if !allow_additional {
        let allowed: BTreeSet<&str> = version.field_types.keys().map(String::as_str).collect();
        for field in obj.keys() {
            if !allowed.contains(field.as_str()) {
                return Err(format!(
                    "packet {} contains unknown top-level field {}",
                    packet_name, field
                ));
            }
        }
    }

    for (field, type_name) in &version.field_types {
        if let Some(value) = obj.get(field) {
            validate_field_type(packet_name, field, type_name, value)?;
            if let Some(allowed_values) = version.field_enums.get(field) {
                let raw = value.as_str().ok_or_else(|| {
                    format!(
                        "packet {} field {} must be string for enum validation",
                        packet_name, field
                    )
                })?;
                if !allowed_values.iter().any(|v| v == raw) {
                    return Err(format!(
                        "packet {} field {} has unsupported enum value {}",
                        packet_name, field, raw
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_field_type(
    packet_name: &str,
    field_name: &str,
    type_name: &str,
    value: &Value,
) -> Result<(), String> {
    let ok = match type_name {
        "string" => value.is_string(),
        "int64" => value.as_i64().is_some() || value.as_u64().is_some(),
        "bool" => value.is_boolean(),
        "float" => value.as_f64().is_some(),
        "array_string" => value
            .as_array()
            .map(|a| a.iter().all(Value::is_string))
            .unwrap_or(false),
        "array_object" => value
            .as_array()
            .map(|a| a.iter().all(Value::is_object))
            .unwrap_or(false),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "string_or_object" => value.is_string() || value.is_object(),
        unknown => {
            return Err(format!(
                "packet {} has unknown type {} for field {}",
                packet_name, unknown, field_name
            ))
        }
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "packet {} field {} failed type check for {}",
            packet_name, field_name, type_name
        ))
    }
}

pub fn packet_name_from_fixture_filename(filename: &str) -> Option<&'static str> {
    match filename {
        "turn_input.json" | "turn_input_missing_required.json" => Some("TurnInputPacket"),
        "search_assist.json" => Some("SearchAssistPacket"),
        "tool_request.json" | "tool_request_bad_mode.json" => Some("ToolRequestPacket"),
        "evidence.json" | "evidence_bad_schema_version.json" => Some("EvidencePacket"),
        "synthesis.json" => Some("SynthesisPacket"),
        "write.json" => Some("WritePacket"),
        "audit.json" | "audit_missing_hashes.json" | "unknown_reason_code.json" => {
            Some("AuditPacket")
        }
        _ => None,
    }
}

pub fn packet_definition_by_name<'a>(
    registry: &'a PacketSchemaRegistry,
    packet_name: &str,
) -> Option<&'a PacketSchemaDefinition> {
    registry
        .packets
        .iter()
        .find(|p| p.packet_name == packet_name)
}
