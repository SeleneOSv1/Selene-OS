#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::vision::thresholds::allow_transcript_segment;
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode, VisionRuntimeNetworkConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub const GOOGLE_STT_PROVIDER_ID: &str = "GOOGLE_STT";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub confidence: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoTranscriptResult {
    pub stt_provider_id: String,
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
    pub full_transcript: String,
}

#[derive(Debug, Clone)]
pub struct SttRequest {
    pub language: String,
    pub audio_wav_bytes: Vec<u8>,
    pub timeout_ms: u64,
}

pub trait SttBackend {
    fn transcribe(
        &self,
        request: &SttRequest,
    ) -> Result<VideoTranscriptResult, VisionProviderError>;
}

#[derive(Debug, Clone)]
pub struct GoogleSttBackend {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub user_agent: String,
    pub network: VisionRuntimeNetworkConfig,
}

impl GoogleSttBackend {
    pub fn from_env(network: VisionRuntimeNetworkConfig) -> Self {
        Self {
            endpoint: std::env::var("SELENE_GOOGLE_STT_ENDPOINT")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| "https://speech.googleapis.com/v1/speech:recognize".to_string()),
            api_key: std::env::var("GOOGLE_STT_API_KEY")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .or_else(|| {
                    selene_engines::device_vault::resolve_secret(
                        selene_kernel_contracts::provider_secrets::ProviderSecretId::GoogleSttApiKey
                            .as_str(),
                    )
                    .ok()
                    .flatten()
                }),
            user_agent: "selene-vision-google-stt/1.0".to_string(),
            network,
        }
    }
}

impl SttBackend for GoogleSttBackend {
    fn transcribe(
        &self,
        request: &SttRequest,
    ) -> Result<VideoTranscriptResult, VisionProviderError> {
        let start = Instant::now();
        let Some(api_key) = self.api_key.as_deref() else {
            return Err(VisionProviderError::new(
                "google_stt",
                "stt",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "Google STT API key is not configured",
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
                    "google_stt",
                    "stt",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    &check.details,
                    start.elapsed().as_millis() as u64,
                ));
            }
        }

        if let Some(proxy_raw) = self.network.proxy_url_for(&self.endpoint) {
            let _ = redact_proxy_url(proxy_raw).map_err(|_| {
                VisionProviderError::new(
                    "google_stt",
                    "stt",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    "proxy redaction failed",
                    start.elapsed().as_millis() as u64,
                )
            })?;
            let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| {
                VisionProviderError::new(
                    "google_stt",
                    "stt",
                    VisionProviderErrorKind::ProxyMisconfigured,
                    VisionReasonCode::PolicyViolation,
                    "invalid proxy url",
                    start.elapsed().as_millis() as u64,
                )
            })?;
            builder = builder.proxy(proxy);
        }

        let payload = json!({
            "config": {
                "encoding": "LINEAR16",
                "sampleRateHertz": 16000,
                "languageCode": request.language,
                "enableWordTimeOffsets": true,
                "enableAutomaticPunctuation": false
            },
            "audio": {
                "content": base64_encode(&request.audio_wav_bytes)
            }
        });

        let endpoint = format!("{}?key={}", self.endpoint, api_key);
        let response = builder
            .build()
            .post(&endpoint)
            .set("Accept", "application/json")
            .send_string(&payload.to_string())
            .map_err(|err| map_transport_error(err, start.elapsed().as_millis() as u64))?;

        let parsed: Value = serde_json::from_reader(response.into_reader()).map_err(|_| {
            VisionProviderError::new(
                "google_stt",
                "stt",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "Google STT response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let mut segments = parse_segments(&parsed)
            .into_iter()
            .filter(|segment| allow_transcript_segment(segment.confidence as f64 / 100.0))
            .collect::<Vec<TranscriptSegment>>();

        segments.sort_by(|a, b| {
            a.start_ms
                .cmp(&b.start_ms)
                .then(a.end_ms.cmp(&b.end_ms))
                .then(a.text.cmp(&b.text))
        });

        let full_transcript = segments
            .iter()
            .map(|segment| segment.text.clone())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(VideoTranscriptResult {
            stt_provider_id: GOOGLE_STT_PROVIDER_ID.to_string(),
            language: request.language.clone(),
            segments,
            full_transcript,
        })
    }
}

fn parse_segments(payload: &Value) -> Vec<TranscriptSegment> {
    let mut segments = Vec::new();

    let Some(results) = payload.get("results").and_then(Value::as_array) else {
        return segments;
    };

    for result in results {
        let Some(alternative) = result
            .get("alternatives")
            .and_then(Value::as_array)
            .and_then(|alts| alts.first())
        else {
            continue;
        };

        let text = alternative
            .get("transcript")
            .and_then(Value::as_str)
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        if text.is_empty() {
            continue;
        }

        let confidence = alternative
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        let (start_ms, end_ms) = parse_word_offsets(alternative).unwrap_or((0, 0));
        segments.push(TranscriptSegment {
            start_ms,
            end_ms,
            text,
            confidence: (confidence * 100.0).round().clamp(0.0, 100.0) as u32,
        });
    }

    segments
}

fn parse_word_offsets(alternative: &Value) -> Option<(u64, u64)> {
    let words = alternative.get("words")?.as_array()?;
    let first = words.first()?.get("startTime")?.as_str()?;
    let last = words.last()?.get("endTime")?.as_str()?;
    Some((
        parse_google_duration_ms(first),
        parse_google_duration_ms(last),
    ))
}

fn parse_google_duration_ms(raw: &str) -> u64 {
    let clean = raw.trim().trim_end_matches('s');
    if clean.is_empty() {
        return 0;
    }

    let value = clean.parse::<f64>().unwrap_or(0.0);
    if value <= 0.0 {
        0
    } else {
        (value * 1000.0).round() as u64
    }
}

fn map_transport_error(err: ureq::Error, latency_ms: u64) -> VisionProviderError {
    match err {
        ureq::Error::Status(status, _) => VisionProviderError::new(
            "google_stt",
            "stt",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            &format!("HTTP status {}", status),
            latency_ms,
        ),
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            if combined.contains("timeout") {
                VisionProviderError::new(
                    "google_stt",
                    "stt",
                    VisionProviderErrorKind::TimeoutExceeded,
                    VisionReasonCode::TimeoutExceeded,
                    "transport timeout",
                    latency_ms,
                )
            } else {
                VisionProviderError::new(
                    "google_stt",
                    "stt",
                    VisionProviderErrorKind::ProviderUpstreamFailed,
                    VisionReasonCode::ProviderUpstreamFailed,
                    "transport failure",
                    latency_ms,
                )
            }
        }
    }
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::new();
    let mut index = 0usize;
    while index < input.len() {
        let b0 = input[index];
        let b1 = if index + 1 < input.len() {
            input[index + 1]
        } else {
            0
        };
        let b2 = if index + 2 < input.len() {
            input[index + 2]
        } else {
            0
        };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);

        if index + 1 < input.len() {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }

        if index + 2 < input.len() {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }

        index += 3;
    }

    out
}
