#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::vision::ocr::BoundingBox;
use crate::web_search_plan::vision::thresholds::allow_object;
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode, VisionRuntimeNetworkConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub const LABEL_MAP_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedObject {
    pub label: String,
    pub bbox: BoundingBox,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectDetectionResult {
    pub frame_index: Option<u32>,
    pub timestamp_ms: Option<u64>,
    pub model_id: String,
    pub objects: Vec<DetectedObject>,
}

#[derive(Debug, Clone)]
pub struct ObjectRequest {
    pub asset_hash: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub timeout_ms: u64,
}

pub trait ObjectBackend {
    fn detect_objects(
        &self,
        request: &ObjectRequest,
    ) -> Result<ObjectDetectionResult, VisionProviderError>;
}

#[derive(Debug, Clone)]
pub struct HttpObjectBackend {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub user_agent: String,
    pub network: VisionRuntimeNetworkConfig,
}

impl HttpObjectBackend {
    pub fn from_env(network: VisionRuntimeNetworkConfig) -> Self {
        Self {
            endpoint: std::env::var("SELENE_OBJECTS_ENDPOINT")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            api_key: std::env::var("SELENE_OBJECTS_API_KEY")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| {
                    selene_engines::device_vault::resolve_secret(
                        selene_kernel_contracts::provider_secrets::ProviderSecretId::OpenAIApiKey
                            .as_str(),
                    )
                    .ok()
                    .flatten()
                }),
            user_agent: "selene-vision-objects/1.0".to_string(),
            network,
        }
    }
}

impl ObjectBackend for HttpObjectBackend {
    fn detect_objects(
        &self,
        request: &ObjectRequest,
    ) -> Result<ObjectDetectionResult, VisionProviderError> {
        let start = Instant::now();

        let Some(endpoint) = self.endpoint.as_deref() else {
            return Err(VisionProviderError::new(
                "vision_objects",
                "objects",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "Objects endpoint is not configured",
                0,
            ));
        };

        let Some(api_key) = self.api_key.as_deref() else {
            return Err(VisionProviderError::new(
                "vision_objects",
                "objects",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "Objects API key is not configured",
                0,
            ));
        };

        let mut builder = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(request.timeout_ms))
            .timeout_read(Duration::from_millis(request.timeout_ms))
            .timeout_write(Duration::from_millis(request.timeout_ms))
            .user_agent(&self.user_agent)
            .try_proxy_from_env(false);

        if let Err(check) = run_startup_self_check(&self.network.proxy_config) {
            if check.error_kind == ProxyErrorKind::ProxyMisconfigured
                && check.severity.as_str() == "critical"
            {
                return Err(VisionProviderError::new(
                    "vision_objects",
                    "objects",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    &check.details,
                    start.elapsed().as_millis() as u64,
                ));
            }
        }

        if let Some(proxy_raw) = self.network.proxy_url_for(endpoint) {
            let _ = redact_proxy_url(proxy_raw).map_err(|_| {
                VisionProviderError::new(
                    "vision_objects",
                    "objects",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    "proxy redaction failed",
                    start.elapsed().as_millis() as u64,
                )
            })?;
            let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| {
                VisionProviderError::new(
                    "vision_objects",
                    "objects",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    "invalid proxy url",
                    start.elapsed().as_millis() as u64,
                )
            })?;
            builder = builder.proxy(proxy);
        }

        let payload = json!({
            "asset_hash": request.asset_hash,
            "mime_type": request.mime_type,
            "bytes_hex": hex_encode(&request.bytes),
        });

        let response = builder
            .build()
            .post(endpoint)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", api_key))
            .send_string(&payload.to_string())
            .map_err(|err| {
                map_transport_error(
                    "vision_objects",
                    "objects",
                    err,
                    start.elapsed().as_millis() as u64,
                )
            })?;

        let parsed: Value = serde_json::from_reader(response.into_reader()).map_err(|_| {
            VisionProviderError::new(
                "vision_objects",
                "objects",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "object response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let model_id = parsed
            .get("model_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown_object_model")
            .trim()
            .to_string();

        let mut objects = parsed
            .get("objects")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(parse_object)
            .filter(|item| allow_object(item.confidence))
            .collect::<Vec<DetectedObject>>();

        objects.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.bbox
                        .y
                        .partial_cmp(&b.bbox.y)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(
                    a.bbox
                        .x
                        .partial_cmp(&b.bbox.x)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.label.cmp(&b.label))
        });

        Ok(ObjectDetectionResult {
            frame_index: None,
            timestamp_ms: None,
            model_id,
            objects,
        })
    }
}

fn parse_object(value: Value) -> Option<DetectedObject> {
    let obj = value.as_object()?;
    let bbox_obj = obj.get("bbox")?.as_object()?;
    let label = canonicalize_label(obj.get("label")?.as_str()?);
    let confidence = obj.get("confidence")?.as_f64()?;

    Some(DetectedObject {
        label,
        bbox: BoundingBox {
            x: bbox_obj.get("x")?.as_f64()?,
            y: bbox_obj.get("y")?.as_f64()?,
            w: bbox_obj.get("w")?.as_f64()?,
            h: bbox_obj.get("h")?.as_f64()?,
        },
        confidence,
    })
}

fn canonicalize_label(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase().replace(' ', "_");
    match normalized.as_str() {
        "human" => "person".to_string(),
        "automobile" => "car".to_string(),
        _ => normalized,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn map_transport_error(
    provider_id: &str,
    endpoint: &str,
    err: ureq::Error,
    latency_ms: u64,
) -> VisionProviderError {
    match err {
        ureq::Error::Status(status, _) => VisionProviderError::new(
            provider_id,
            endpoint,
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            &format!("HTTP status {}", status),
            latency_ms,
        ),
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            if combined.contains("timeout") {
                VisionProviderError::new(
                    provider_id,
                    endpoint,
                    VisionProviderErrorKind::TimeoutExceeded,
                    VisionReasonCode::TimeoutExceeded,
                    "transport timeout",
                    latency_ms,
                )
            } else {
                VisionProviderError::new(
                    provider_id,
                    endpoint,
                    VisionProviderErrorKind::ProviderUpstreamFailed,
                    VisionReasonCode::ProviderUpstreamFailed,
                    "transport failure",
                    latency_ms,
                )
            }
        }
    }
}
