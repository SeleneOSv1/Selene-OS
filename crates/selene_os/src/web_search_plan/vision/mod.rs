#![forbid(unsafe_code)]

pub mod asset_ref;
pub mod download;
pub mod keyframes_ffmpeg;
pub mod objects;
pub mod ocr;
pub mod packet_builder;
pub mod redaction;
pub mod stt_google;
pub mod thresholds;
pub mod video;

use crate::web_search_plan::proxy::proxy_config::{ProxyConfig, SystemEnvProvider};
use crate::web_search_plan::proxy::ProxyMode;
use crate::web_search_plan::vision::asset_ref::{is_image_mime, is_video_mime, validate_asset_ref};
use crate::web_search_plan::vision::download::load_asset;
use crate::web_search_plan::vision::packet_builder::{
    build_vision_evidence_packet, deterministic_join_lines, sort_detected_objects, sort_keyframes,
    sort_ocr_blocks, sort_transcript_segments, to_json_value, KeyframeIndexResult,
    ObjectDetectionResult, OcrResult, VideoTranscriptResult, VisionOutput, VisionProviderRun,
};
use crate::web_search_plan::vision::redaction::{redact_locator, redact_secrets};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VisionMode {
    ImageOcr,
    ImageObjects,
    ImageAnalyze,
    VideoTranscribe,
    VideoKeyframes,
    VideoAnalyze,
}

impl VisionMode {
    pub fn parse(value: &str) -> Result<Self, VisionError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "image_ocr" => Ok(Self::ImageOcr),
            "image_objects" => Ok(Self::ImageObjects),
            "image_analyze" => Ok(Self::ImageAnalyze),
            "video_transcribe" => Ok(Self::VideoTranscribe),
            "video_keyframes" => Ok(Self::VideoKeyframes),
            "video_analyze" => Ok(Self::VideoAnalyze),
            _ => Err(VisionError::new(
                VisionErrorKind::PolicyViolation,
                format!("unsupported vision mode {}", value),
            )),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImageOcr => "image_ocr",
            Self::ImageObjects => "image_objects",
            Self::ImageAnalyze => "image_analyze",
            Self::VideoTranscribe => "video_transcribe",
            Self::VideoKeyframes => "video_keyframes",
            Self::VideoAnalyze => "video_analyze",
        }
    }

    pub const fn requires_video(self) -> bool {
        matches!(
            self,
            Self::VideoTranscribe | Self::VideoKeyframes | Self::VideoAnalyze
        )
    }

    pub const fn requires_image(self) -> bool {
        matches!(
            self,
            Self::ImageOcr | Self::ImageObjects | Self::ImageAnalyze
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionAssetRef {
    pub asset_hash: String,
    pub locator: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_frames: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_stride_ms: Option<u32>,
    pub safe_mode: bool,
    #[serde(default)]
    pub analyze_url: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionBudgets {
    pub timeout_ms: u64,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionToolRequest {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub mode: VisionMode,
    pub asset_ref: VisionAssetRef,
    pub options: VisionOptions,
    pub budgets: VisionBudgets,
    pub policy_snapshot_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionErrorKind {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    InsufficientEvidence,
    PolicyViolation,
}

impl VisionErrorKind {
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::PolicyViolation => "policy_violation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionError {
    pub kind: VisionErrorKind,
    pub message: String,
}

impl VisionError {
    pub fn new(kind: VisionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: redact_secrets(message.into().as_str()),
        }
    }

    pub const fn reason_code(&self) -> &'static str {
        self.kind.reason_code()
    }
}

#[derive(Debug, Clone)]
pub struct VisionRuntimeConfig {
    pub max_timeout_ms: u64,
    pub ocr_endpoint: Option<String>,
    pub ocr_api_key: Option<String>,
    pub objects_endpoint: Option<String>,
    pub objects_api_key: Option<String>,
    pub google_stt_api_key: Option<String>,
    pub google_stt_endpoint: Option<String>,
    pub proxy_config: ProxyConfig,
}

impl VisionRuntimeConfig {
    pub fn proxy_url_for_asset_mime(&self, mime_type: &str) -> Option<String> {
        let is_https = mime_type.to_ascii_lowercase().starts_with("video/");
        if is_https {
            self.proxy_config
                .https_proxy_url
                .clone()
                .or_else(|| self.proxy_config.http_proxy_url.clone())
        } else {
            self.proxy_config
                .http_proxy_url
                .clone()
                .or_else(|| self.proxy_config.https_proxy_url.clone())
        }
    }
}

impl Default for VisionRuntimeConfig {
    fn default() -> Self {
        let env = SystemEnvProvider;
        let proxy_mode_raw =
            std::env::var("SELENE_VISION_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
        let proxy_mode = ProxyMode::parse(proxy_mode_raw.as_str()).unwrap_or(ProxyMode::Off);

        Self {
            max_timeout_ms: 15_000,
            ocr_endpoint: std::env::var("SELENE_VISION_OCR_ENDPOINT").ok(),
            ocr_api_key: std::env::var("SELENE_VISION_OCR_API_KEY").ok(),
            objects_endpoint: std::env::var("SELENE_VISION_OBJECTS_ENDPOINT").ok(),
            objects_api_key: std::env::var("SELENE_VISION_OBJECTS_API_KEY").ok(),
            google_stt_api_key: std::env::var("GOOGLE_STT_API_KEY").ok(),
            google_stt_endpoint: std::env::var("SELENE_GOOGLE_STT_ENDPOINT").ok(),
            proxy_config: ProxyConfig::from_env(proxy_mode, &env),
        }
    }
}

pub trait VisionProviderSet {
    fn run_ocr(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<OcrResult, VisionError>;

    fn run_objects(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<ObjectDetectionResult, VisionError>;

    fn run_video_stt(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<VideoTranscriptResult, VisionError>;

    fn run_keyframes(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<KeyframeIndexResult, VisionError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeVisionProviders;

impl VisionProviderSet for RuntimeVisionProviders {
    fn run_ocr(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<OcrResult, VisionError> {
        ocr::run_ocr_runtime(
            asset,
            request,
            config.ocr_endpoint.as_deref(),
            config.ocr_api_key.as_deref(),
        )
    }

    fn run_objects(
        &self,
        asset: &download::LoadedAsset,
        _request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<ObjectDetectionResult, VisionError> {
        objects::run_objects_runtime(
            asset,
            config.objects_endpoint.as_deref(),
            config.objects_api_key.as_deref(),
        )
    }

    fn run_video_stt(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        config: &VisionRuntimeConfig,
    ) -> Result<VideoTranscriptResult, VisionError> {
        let audio = video::extract_audio_wav_16k_mono(asset)?;
        stt_google::run_google_stt_runtime(
            audio.as_slice(),
            request,
            config.google_stt_api_key.as_deref(),
            config.google_stt_endpoint.as_deref(),
        )
    }

    fn run_keyframes(
        &self,
        asset: &download::LoadedAsset,
        request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<KeyframeIndexResult, VisionError> {
        keyframes_ffmpeg::extract_keyframes_runtime(asset, request)
    }
}

pub fn run_vision_tool(
    request_packet: &Value,
    now_ms: i64,
    config: &VisionRuntimeConfig,
) -> Result<Value, VisionError> {
    run_vision_tool_with_providers(
        request_packet,
        now_ms,
        config,
        &RuntimeVisionProviders::default(),
    )
}

pub fn run_vision_tool_with_providers(
    request_packet: &Value,
    now_ms: i64,
    config: &VisionRuntimeConfig,
    providers: &impl VisionProviderSet,
) -> Result<Value, VisionError> {
    let request = parse_vision_tool_request_packet(request_packet)?;
    validate_asset_ref(&request.asset_ref)?;
    validate_mode_mime_compatibility(&request)?;

    let asset = load_asset(&request, config)?;

    let mut provider_runs = vec![VisionProviderRun {
        provider_id: "vision_download".to_string(),
        endpoint: "download".to_string(),
        latency_ms: asset.latency_ms,
        error: None,
    }];
    let mut outputs = Vec::new();
    let mut reason_codes = BTreeSet::new();

    match request.mode {
        VisionMode::ImageOcr => {
            let result = run_endpoint("vision_ocr", "ocr", &mut provider_runs, || {
                providers.run_ocr(&asset, &request, config)
            })?;
            outputs.push(VisionOutput::OcrResult(canonicalize_ocr_result(result)));
        }
        VisionMode::ImageObjects => {
            let result = run_endpoint("vision_objects", "objects", &mut provider_runs, || {
                providers.run_objects(&asset, &request, config)
            })?;
            outputs.push(VisionOutput::ObjectDetectionResult(
                canonicalize_object_result(result),
            ));
        }
        VisionMode::ImageAnalyze => {
            let ocr_result = run_endpoint("vision_ocr", "ocr", &mut provider_runs, || {
                providers.run_ocr(&asset, &request, config)
            })?;
            outputs.push(VisionOutput::OcrResult(canonicalize_ocr_result(ocr_result)));

            let objects_result =
                run_endpoint("vision_objects", "objects", &mut provider_runs, || {
                    providers.run_objects(&asset, &request, config)
                })?;
            outputs.push(VisionOutput::ObjectDetectionResult(
                canonicalize_object_result(objects_result),
            ));
        }
        VisionMode::VideoTranscribe => {
            let transcript = run_endpoint("vision_stt", "stt", &mut provider_runs, || {
                providers.run_video_stt(&asset, &request, config)
            })?;
            outputs.push(VisionOutput::VideoTranscriptResult(
                canonicalize_transcript_result(transcript),
            ));
        }
        VisionMode::VideoKeyframes => {
            let keyframes =
                run_endpoint("vision_keyframes", "keyframes", &mut provider_runs, || {
                    providers.run_keyframes(&asset, &request, config)
                })?;
            outputs.push(VisionOutput::KeyframeIndexResult(
                canonicalize_keyframes_result(keyframes),
            ));
        }
        VisionMode::VideoAnalyze => {
            let transcript = run_endpoint("vision_stt", "stt", &mut provider_runs, || {
                providers.run_video_stt(&asset, &request, config)
            })?;
            outputs.push(VisionOutput::VideoTranscriptResult(
                canonicalize_transcript_result(transcript),
            ));

            let keyframes =
                run_endpoint("vision_keyframes", "keyframes", &mut provider_runs, || {
                    providers.run_keyframes(&asset, &request, config)
                })?;
            outputs.push(VisionOutput::KeyframeIndexResult(
                canonicalize_keyframes_result(keyframes),
            ));
        }
    }

    if outputs.is_empty() {
        reason_codes.insert("insufficient_evidence".to_string());
    }

    let packet = build_vision_evidence_packet(
        &request,
        now_ms,
        provider_runs,
        outputs,
        &reason_codes.into_iter().collect::<Vec<String>>(),
    )?;

    to_json_value(&packet)
}

fn run_endpoint<T>(
    provider_id: &str,
    endpoint: &str,
    provider_runs: &mut Vec<VisionProviderRun>,
    mut runner: impl FnMut() -> Result<T, VisionError>,
) -> Result<T, VisionError> {
    let start = Instant::now();
    match runner() {
        Ok(value) => {
            provider_runs.push(VisionProviderRun {
                provider_id: provider_id.to_string(),
                endpoint: endpoint.to_string(),
                latency_ms: start.elapsed().as_millis() as u64,
                error: None,
            });
            Ok(value)
        }
        Err(error) => {
            provider_runs.push(VisionProviderRun {
                provider_id: provider_id.to_string(),
                endpoint: endpoint.to_string(),
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some(redact_secrets(error.message.as_str())),
            });
            Err(error)
        }
    }
}

fn parse_vision_tool_request_packet(packet: &Value) -> Result<VisionToolRequest, VisionError> {
    let obj = packet.as_object().ok_or_else(|| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            "vision tool request must be object",
        )
    })?;

    let schema_version = required_string(obj.get("schema_version"), "schema_version")?;
    let produced_by = required_string(obj.get("produced_by"), "produced_by")?;
    let trace_id = required_string(obj.get("trace_id"), "trace_id")?;
    let mode_raw = required_string(obj.get("mode"), "mode")?;
    let mode = VisionMode::parse(mode_raw.as_str())?;

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::PolicyViolation,
                "created_at_ms is required",
            )
        })?;

    let intended_consumers = obj
        .get("intended_consumers")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
        .filter(|entries| !entries.is_empty())
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::PolicyViolation,
                "intended_consumers must be a non-empty array_string",
            )
        })?;

    let asset = obj
        .get("asset_ref")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            VisionError::new(VisionErrorKind::PolicyViolation, "asset_ref is required")
        })?;

    let asset_ref = VisionAssetRef {
        asset_hash: required_string(asset.get("asset_hash"), "asset_ref.asset_hash")?,
        locator: required_string(asset.get("locator"), "asset_ref.locator")?,
        mime_type: required_string(asset.get("mime_type"), "asset_ref.mime_type")?,
        size_bytes: asset
            .get("size_bytes")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                VisionError::new(
                    VisionErrorKind::PolicyViolation,
                    "asset_ref.size_bytes is required",
                )
            })?,
    };

    let options_obj = obj
        .get("options")
        .and_then(Value::as_object)
        .ok_or_else(|| VisionError::new(VisionErrorKind::PolicyViolation, "options is required"))?;
    let options = VisionOptions {
        language_hint: options_obj
            .get("language_hint")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
        max_frames: options_obj
            .get("max_frames")
            .and_then(Value::as_u64)
            .map(|value| value as u32),
        frame_stride_ms: options_obj
            .get("frame_stride_ms")
            .and_then(Value::as_u64)
            .map(|value| value as u32),
        safe_mode: options_obj
            .get("safe_mode")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                VisionError::new(
                    VisionErrorKind::PolicyViolation,
                    "options.safe_mode is required",
                )
            })?,
        analyze_url: options_obj
            .get("analyze_url")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };

    let budgets_obj = obj
        .get("budgets")
        .and_then(Value::as_object)
        .ok_or_else(|| VisionError::new(VisionErrorKind::PolicyViolation, "budgets is required"))?;
    let budgets = VisionBudgets {
        timeout_ms: budgets_obj
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                VisionError::new(
                    VisionErrorKind::PolicyViolation,
                    "budgets.timeout_ms is required",
                )
            })?,
        max_bytes: budgets_obj
            .get("max_bytes")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                VisionError::new(
                    VisionErrorKind::PolicyViolation,
                    "budgets.max_bytes is required",
                )
            })?,
    };

    let policy_snapshot_id = required_string(obj.get("policy_snapshot_id"), "policy_snapshot_id")?;

    Ok(VisionToolRequest {
        schema_version,
        produced_by,
        intended_consumers,
        created_at_ms,
        trace_id,
        mode,
        asset_ref,
        options,
        budgets,
        policy_snapshot_id,
    })
}

fn required_string(field: Option<&Value>, name: &str) -> Result<String, VisionError> {
    field
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::PolicyViolation,
                format!("{} is required", name),
            )
        })
}

fn validate_mode_mime_compatibility(request: &VisionToolRequest) -> Result<(), VisionError> {
    let mime = request.asset_ref.mime_type.to_ascii_lowercase();
    if request.mode.requires_image() && !is_image_mime(&mime) {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!(
                "mode {} requires image mime_type, got {}",
                request.mode.as_str(),
                request.asset_ref.mime_type
            ),
        ));
    }
    if request.mode.requires_video() && !is_video_mime(&mime) {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!(
                "mode {} requires video mime_type, got {}",
                request.mode.as_str(),
                request.asset_ref.mime_type
            ),
        ));
    }

    if request.asset_ref.locator.starts_with("http://")
        || request.asset_ref.locator.starts_with("https://")
    {
        let _ = redact_locator(request.asset_ref.locator.as_str());
    }

    Ok(())
}

fn canonicalize_ocr_result(mut result: OcrResult) -> OcrResult {
    sort_ocr_blocks(&mut result.text_blocks);
    result.full_text = deterministic_join_lines(
        &result
            .text_blocks
            .iter()
            .map(|block| block.text.clone())
            .collect::<Vec<String>>(),
    );
    result
}

fn canonicalize_object_result(mut result: ObjectDetectionResult) -> ObjectDetectionResult {
    sort_detected_objects(&mut result.objects);
    result
}

fn canonicalize_transcript_result(mut result: VideoTranscriptResult) -> VideoTranscriptResult {
    sort_transcript_segments(&mut result.segments);
    result.full_transcript = deterministic_join_lines(
        &result
            .segments
            .iter()
            .map(|segment| segment.text.clone())
            .collect::<Vec<String>>(),
    );
    result
}

fn canonicalize_keyframes_result(mut result: KeyframeIndexResult) -> KeyframeIndexResult {
    sort_keyframes(&mut result.keyframes);
    result
}

#[cfg(test)]
pub mod vision_tests;
