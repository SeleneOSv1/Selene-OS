#![forbid(unsafe_code)]

use crate::web_search_plan::structured::adapters::fetch_json_with_caps;
use crate::web_search_plan::structured::schema::StructuredSchemaId;
use crate::web_search_plan::structured::types::{
    StructuredAdapterOutput, StructuredAdapterRequest, StructuredConnectorError,
    StructuredErrorKind, StructuredRow, StructuredRuntimeConfig, StructuredValue,
    STRUCTURED_SCHEMA_VERSION,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use serde_json::{json, Value};

pub const ADAPTER_ID: &str = "gov_dataset";

pub fn execute(
    request: &StructuredAdapterRequest,
    config: &StructuredRuntimeConfig,
) -> Result<StructuredAdapterOutput, StructuredConnectorError> {
    let api_key = resolve_api_key(config).ok_or_else(|| {
        StructuredConnectorError::new(
            ADAPTER_ID,
            StructuredErrorKind::ProviderUnconfigured,
            None,
            "Not Implemented: no provider credentials configured",
            0,
        )
    })?;

    let query_encoded = percent_encode(request.query.as_str());
    let requested_url = format!(
        "{}?api_key={}&school.name={}&per_page=5&fields=id,school.name,latest.student.size,latest.cost.avg_net_price.overall,latest.completion.rate",
        config.gov_dataset_endpoint, api_key, query_encoded
    );
    let source_url = format!(
        "{}?school.name={}&per_page=5",
        config.gov_dataset_endpoint, query_encoded
    );

    let (payload, latency_ms) =
        fetch_json_with_caps(ADAPTER_ID, request, config, requested_url.as_str(), &[])?;

    let results = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            StructuredConnectorError::new(
                ADAPTER_ID,
                StructuredErrorKind::ProviderUpstreamFailed,
                None,
                "gov_dataset response missing results array",
                latency_ms,
            )
        })?;

    let mut rows = Vec::new();
    for record in results {
        if let Some(row_bundle) = rows_from_record(record, source_url.as_str()) {
            rows.extend(row_bundle);
        }
    }

    if rows.is_empty() {
        return Err(StructuredConnectorError::new(
            ADAPTER_ID,
            StructuredErrorKind::EmptyResults,
            None,
            "gov_dataset returned zero parsable rows",
            latency_ms,
        ));
    }

    Ok(StructuredAdapterOutput {
        schema_id: StructuredSchemaId::GovDatasetV1.as_str().to_string(),
        rows,
        provider_runs: vec![json!({
            "provider_id": ADAPTER_ID,
            "endpoint": "structured",
            "latency_ms": latency_ms,
            "results_count": results.len(),
            "error": Value::Null,
        })],
        sources: vec![json!({
            "title": "US College Scorecard",
            "url": source_url,
            "snippet": "structured connector fetched government dataset records",
            "media_type": "structured",
            "provider_id": ADAPTER_ID,
            "rank": 1,
            "canonical_url": source_url.to_ascii_lowercase(),
        })],
        errors: Vec::new(),
    })
}

fn rows_from_record(record: &Value, source_url: &str) -> Option<Vec<StructuredRow>> {
    let id = record.get("id").and_then(Value::as_i64)?;
    let entity = record
        .get("school.name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown_school")
        .to_string();

    let mut rows = vec![StructuredRow {
        entity: entity.clone(),
        attribute: "dataset_id".to_string(),
        value: StructuredValue::Int { value: id },
        unit: None,
        as_of_ms: None,
        source_url: source_url.to_string(),
        source_ref: source_url.to_string(),
        confidence: None,
        schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
    }];

    if let Some(size) = record.get("latest.student.size").and_then(Value::as_i64) {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "value".to_string(),
            value: StructuredValue::Int { value: size },
            unit: Some("students".to_string()),
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some(net_price) = record
        .get("latest.cost.avg_net_price.overall")
        .and_then(Value::as_f64)
    {
        rows.push(StructuredRow {
            entity: entity.clone(),
            attribute: "net_price".to_string(),
            value: StructuredValue::Currency {
                amount: net_price,
                currency_code: "USD".to_string(),
            },
            unit: None,
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    if let Some(rate) = record.get("latest.completion.rate").and_then(Value::as_f64) {
        let percent = if rate <= 1.0 { rate * 100.0 } else { rate };
        rows.push(StructuredRow {
            entity,
            attribute: "completion_rate".to_string(),
            value: StructuredValue::Percent { value: percent },
            unit: Some("percent".to_string()),
            as_of_ms: None,
            source_url: source_url.to_string(),
            source_ref: source_url.to_string(),
            confidence: None,
            schema_version: STRUCTURED_SCHEMA_VERSION.to_string(),
        });
    }

    Some(rows)
}

fn resolve_api_key(config: &StructuredRuntimeConfig) -> Option<String> {
    if let Some(override_key) = &config.gov_dataset_api_key_override {
        let trimmed = override_key.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Ok(env_key) = std::env::var("SELENE_GOV_DATASET_API_KEY") {
        let trimmed = env_key.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let secret_id = config
        .gov_dataset_vault_secret_id_override
        .as_deref()
        .and_then(ProviderSecretId::parse);
    let secret_id = secret_id.or(Some(ProviderSecretId::OpenAIApiKey))?;

    match selene_engines::device_vault::resolve_secret(secret_id.as_str()) {
        Ok(Some(secret)) => {
            let trimmed = secret.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => None,
    }
}

fn percent_encode(input: &str) -> String {
    url::form_urlencoded::byte_serialize(input.as_bytes()).collect::<String>()
}
