#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::vision::thresholds::allow_ocr_block;
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode, VisionRuntimeNetworkConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrTextBlock {
    pub bbox: BoundingBox,
    pub text: String,
    pub confidence: f64,
    pub pii_suspected: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    pub page_or_frame_index: u32,
    pub timestamp_ms: Option<u64>,
    pub ocr_engine_id: String,
    pub language: String,
    pub text_blocks: Vec<OcrTextBlock>,
    pub full_text: String,
}

#[derive(Debug, Clone)]
pub struct OcrRequest {
    pub asset_hash: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub language_hint: Option<String>,
    pub safe_mode: bool,
    pub timeout_ms: u64,
}

pub trait OcrBackend {
    fn extract_text(&self, request: &OcrRequest) -> Result<OcrResult, VisionProviderError>;
}

#[derive(Debug, Clone)]
pub struct HttpOcrBackend {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub user_agent: String,
    pub network: VisionRuntimeNetworkConfig,
}

impl HttpOcrBackend {
    pub fn from_env(network: VisionRuntimeNetworkConfig) -> Self {
        Self {
            endpoint: std::env::var("SELENE_OCR_ENDPOINT")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            api_key: std::env::var("SELENE_OCR_API_KEY")
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
            user_agent: "selene-vision-ocr/1.0".to_string(),
            network,
        }
    }
}

impl OcrBackend for HttpOcrBackend {
    fn extract_text(&self, request: &OcrRequest) -> Result<OcrResult, VisionProviderError> {
        let start = Instant::now();

        let Some(endpoint) = self.endpoint.as_deref() else {
            return Err(VisionProviderError::new(
                "vision_ocr",
                "ocr",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "OCR endpoint is not configured",
                0,
            ));
        };

        let Some(api_key) = self.api_key.as_deref() else {
            return Err(VisionProviderError::new(
                "vision_ocr",
                "ocr",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "OCR API key is not configured",
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
                    "vision_ocr",
                    "ocr",
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
                    "vision_ocr",
                    "ocr",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    "proxy redaction failed",
                    start.elapsed().as_millis() as u64,
                )
            })?;
            let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| {
                VisionProviderError::new(
                    "vision_ocr",
                    "ocr",
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
            "language_hint": request.language_hint,
            "safe_mode": request.safe_mode,
            "bytes_hex": hex_encode(&request.bytes),
        });

        let response = builder
            .build()
            .post(endpoint)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", api_key))
            .send_string(&payload.to_string())
            .map_err(|err| {
                map_transport_error("vision_ocr", "ocr", err, start.elapsed().as_millis() as u64)
            })?;

        let parsed: Value = serde_json::from_reader(response.into_reader()).map_err(|_| {
            VisionProviderError::new(
                "vision_ocr",
                "ocr",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "OCR response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let engine_id = parsed
            .get("ocr_engine_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown_ocr_engine")
            .trim()
            .to_string();
        let language = parsed
            .get("language")
            .and_then(Value::as_str)
            .or(request.language_hint.as_deref())
            .unwrap_or("und")
            .trim()
            .to_string();

        let mut blocks = parsed
            .get("text_blocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|block| parse_block(&block, request.safe_mode))
            .filter(|block| allow_ocr_block(block.confidence))
            .collect::<Vec<OcrTextBlock>>();

        blocks.sort_by(|a, b| {
            a.bbox
                .y
                .partial_cmp(&b.bbox.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.bbox
                        .x
                        .partial_cmp(&b.bbox.x)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.text.cmp(&b.text))
        });

        let full_text = blocks
            .iter()
            .map(|block| normalize_text(&block.text))
            .filter(|text| !text.is_empty())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(OcrResult {
            page_or_frame_index: 0,
            timestamp_ms: None,
            ocr_engine_id: engine_id,
            language,
            text_blocks: blocks,
            full_text,
        })
    }
}

fn parse_block(value: &Value, safe_mode: bool) -> Option<OcrTextBlock> {
    let bbox_obj = value.get("bbox")?.as_object()?;
    let bbox = BoundingBox {
        x: bbox_obj.get("x")?.as_f64()?,
        y: bbox_obj.get("y")?.as_f64()?,
        w: bbox_obj.get("w")?.as_f64()?,
        h: bbox_obj.get("h")?.as_f64()?,
    };

    let text = normalize_text(value.get("text")?.as_str()?);
    if text.is_empty() {
        return None;
    }

    let confidence = value.get("confidence")?.as_f64()?;
    let pii_suspected = if safe_mode {
        Some(looks_like_pii(&text))
    } else {
        None
    };

    Some(OcrTextBlock {
        bbox,
        text,
        confidence,
        pii_suspected,
    })
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn looks_like_pii(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains('@') {
        return true;
    }

    let digits = lower.chars().filter(|ch| ch.is_ascii_digit()).count();
    digits >= 9
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
