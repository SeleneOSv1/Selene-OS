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
use crate::web_search_plan::vision::download::{AssetLoader, DefaultAssetLoader, LoadedAsset};
use crate::web_search_plan::vision::keyframes_ffmpeg::{
    FfmpegKeyframeExtractor, KeyframeExtractor, KeyframeIndexResult, KeyframeRequest,
};
use crate::web_search_plan::vision::objects::{
    HttpObjectBackend, ObjectBackend, ObjectDetectionResult, ObjectRequest,
};
use crate::web_search_plan::vision::ocr::{HttpOcrBackend, OcrBackend, OcrRequest, OcrResult};
use crate::web_search_plan::vision::packet_builder::build_vision_evidence_packet;
use crate::web_search_plan::vision::stt_google::{
    GoogleSttBackend, SttBackend, SttRequest, VideoTranscriptResult,
};
use crate::web_search_plan::vision::thresholds::{
    allow_object, allow_ocr_block, allow_transcript_segment,
};
use crate::web_search_plan::vision::video::{AudioExtractor, FfmpegAudioExtractor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub const VISION_ENGINE_ID: &str = "PH1.E";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisionMode {
    ImageOcr,
    ImageObjects,
    ImageAnalyze,
    VideoTranscribe,
    VideoKeyframes,
    VideoAnalyze,
}

impl VisionMode {
    pub fn is_image_mode(self) -> bool {
        matches!(
            self,
            Self::ImageOcr | Self::ImageObjects | Self::ImageAnalyze
        )
    }

    pub fn is_video_mode(self) -> bool {
        matches!(
            self,
            Self::VideoTranscribe | Self::VideoKeyframes | Self::VideoAnalyze
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionReasonCode {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    InsufficientEvidence,
    PolicyViolation,
}

impl VisionReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::PolicyViolation => "policy_violation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionProviderErrorKind {
    ProviderUnconfigured,
    ProviderUpstreamFailed,
    TimeoutExceeded,
    PolicyViolation,
    ProxyMisconfigured,
    UnsupportedMediaType,
}

impl VisionProviderErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderUnconfigured => "provider_unconfigured",
            Self::ProviderUpstreamFailed => "provider_upstream_failed",
            Self::TimeoutExceeded => "timeout_exceeded",
            Self::PolicyViolation => "policy_violation",
            Self::ProxyMisconfigured => "proxy_misconfigured",
            Self::UnsupportedMediaType => "unsupported_media_type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionProviderErrorRecord {
    pub error_kind: String,
    pub reason_code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct VisionProviderError {
    pub provider_id: String,
    pub endpoint: String,
    pub kind: VisionProviderErrorKind,
    pub reason_code: VisionReasonCode,
    pub message: String,
    pub latency_ms: u64,
}

impl VisionProviderError {
    pub fn new(
        provider_id: &str,
        endpoint: &str,
        kind: VisionProviderErrorKind,
        reason_code: VisionReasonCode,
        message: &str,
        latency_ms: u64,
    ) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            endpoint: endpoint.to_string(),
            kind,
            reason_code,
            message: redaction::redact_error_message(message),
            latency_ms,
        }
    }

    pub fn to_record(&self) -> VisionProviderErrorRecord {
        VisionProviderErrorRecord {
            error_kind: self.kind.as_str().to_string(),
            reason_code: self.reason_code.as_str().to_string(),
            message: self.message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionProviderRun {
    pub provider_id: String,
    pub endpoint: String,
    pub latency_ms: u64,
    pub error: Option<VisionProviderErrorRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionOptions {
    pub language_hint: Option<String>,
    pub max_frames: Option<u32>,
    pub frame_stride_ms: Option<u32>,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionBudgets {
    pub timeout_ms: u64,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionToolRequestPacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub mode: VisionMode,
    pub asset_ref: asset_ref::VisionAssetRef,
    pub options: VisionOptions,
    pub budgets: VisionBudgets,
    pub policy_snapshot_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "output_type")]
pub enum VisionOutput {
    OCRResult(OcrResult),
    ObjectDetectionResult(ObjectDetectionResult),
    VideoTranscriptResult(VideoTranscriptResult),
    KeyframeIndexResult(KeyframeIndexResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceSummary {
    pub mean_confidence: f64,
    pub ocr_blocks_retained: u64,
    pub objects_retained: u64,
    pub transcript_segments_retained: u64,
    pub output_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisionEvidencePacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub asset_ref: asset_ref::VisionAssetRef,
    pub retrieved_at_ms: i64,
    pub provider_runs: Vec<VisionProviderRun>,
    pub outputs: Vec<VisionOutput>,
    pub confidence_summary: ConfidenceSummary,
    pub reason_codes: Vec<String>,
    pub packet_hashes: Value,
    pub output_hash: String,
}

pub trait VisionClock {
    fn now_ms(&self) -> i64;
}

#[derive(Debug, Clone, Default)]
pub struct SystemVisionClock;

impl VisionClock for SystemVisionClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct VisionRuntimeNetworkConfig {
    pub proxy_config: ProxyConfig,
}

impl VisionRuntimeNetworkConfig {
    pub fn from_env() -> Self {
        let env = SystemEnvProvider;
        let proxy_mode_raw =
            std::env::var("SELENE_WEB_PROXY_MODE").unwrap_or_else(|_| "off".to_string());
        let proxy_mode = ProxyMode::parse(&proxy_mode_raw).unwrap_or(ProxyMode::Off);

        Self {
            proxy_config: ProxyConfig::from_env(proxy_mode, &env),
        }
    }

    pub fn proxy_url_for<'a>(&'a self, endpoint: &str) -> Option<&'a str> {
        let is_https = endpoint.trim().to_ascii_lowercase().starts_with("https://");
        if is_https {
            self.proxy_config.https_proxy_url.as_deref()
        } else {
            self.proxy_config.http_proxy_url.as_deref()
        }
    }
}

pub struct VisionRuntime {
    pub asset_loader: Box<dyn AssetLoader>,
    pub ocr_backend: Box<dyn OcrBackend>,
    pub object_backend: Box<dyn ObjectBackend>,
    pub stt_backend: Box<dyn SttBackend>,
    pub keyframe_extractor: Box<dyn KeyframeExtractor>,
    pub audio_extractor: Box<dyn AudioExtractor>,
    pub clock: Box<dyn VisionClock>,
}

impl VisionRuntime {
    pub fn production() -> Self {
        let network = VisionRuntimeNetworkConfig::from_env();
        Self {
            asset_loader: Box::new(DefaultAssetLoader::from_env(network.clone())),
            ocr_backend: Box::new(HttpOcrBackend::from_env(network.clone())),
            object_backend: Box::new(HttpObjectBackend::from_env(network.clone())),
            stt_backend: Box::new(GoogleSttBackend::from_env(network.clone())),
            keyframe_extractor: Box::new(FfmpegKeyframeExtractor),
            audio_extractor: Box::new(FfmpegAudioExtractor),
            clock: Box::new(SystemVisionClock),
        }
    }
}

pub fn run_vision_tool(request: VisionToolRequestPacket) -> VisionEvidencePacket {
    let runtime = VisionRuntime::production();
    run_vision_tool_with_runtime(request, &runtime)
}

pub fn run_vision_tool_with_runtime(
    request: VisionToolRequestPacket,
    runtime: &VisionRuntime,
) -> VisionEvidencePacket {
    let retrieved_at_ms = runtime.clock.now_ms();
    let mut provider_runs: Vec<VisionProviderRun> = Vec::new();
    let mut outputs: Vec<VisionOutput> = Vec::new();
    let mut reason_codes: BTreeSet<String> = BTreeSet::new();

    let loaded_asset = match runtime.asset_loader.load(&request) {
        Ok(asset) => asset,
        Err(err) => {
            reason_codes.insert(err.reason_code.as_str().to_string());
            provider_runs.push(error_run(&err));
            return build_vision_evidence_packet(
                &request,
                retrieved_at_ms,
                provider_runs,
                outputs,
                reason_codes,
            );
        }
    };

    if !request.asset_ref.verify_hash(&loaded_asset.bytes) {
        let error = VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::PolicyViolation,
            VisionReasonCode::PolicyViolation,
            "asset hash mismatch",
            0,
        );
        reason_codes.insert(error.reason_code.as_str().to_string());
        provider_runs.push(error_run(&error));
        return build_vision_evidence_packet(
            &request,
            retrieved_at_ms,
            provider_runs,
            outputs,
            reason_codes,
        );
    }

    provider_runs.push(VisionProviderRun {
        provider_id: "vision_download".to_string(),
        endpoint: "download".to_string(),
        latency_ms: 0,
        error: None,
    });

    if request.mode.is_image_mode() && !loaded_asset.mime_type.starts_with("image/") {
        reason_codes.insert(VisionReasonCode::PolicyViolation.as_str().to_string());
        return build_vision_evidence_packet(
            &request,
            retrieved_at_ms,
            provider_runs,
            outputs,
            reason_codes,
        );
    }

    if request.mode.is_video_mode() && !loaded_asset.mime_type.starts_with("video/") {
        reason_codes.insert(VisionReasonCode::PolicyViolation.as_str().to_string());
        return build_vision_evidence_packet(
            &request,
            retrieved_at_ms,
            provider_runs,
            outputs,
            reason_codes,
        );
    }

    match request.mode {
        VisionMode::ImageOcr => run_ocr(
            &request,
            &loaded_asset,
            runtime,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMode::ImageObjects => run_objects(
            &request,
            &loaded_asset,
            runtime,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMode::ImageAnalyze => {
            run_ocr(
                &request,
                &loaded_asset,
                runtime,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
            run_objects(
                &request,
                &loaded_asset,
                runtime,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
        }
        VisionMode::VideoTranscribe => run_video_transcription(
            &request,
            &loaded_asset,
            runtime,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMode::VideoKeyframes => run_keyframes(
            &request,
            &loaded_asset,
            runtime,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMode::VideoAnalyze => {
            run_keyframes(
                &request,
                &loaded_asset,
                runtime,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
            run_video_transcription(
                &request,
                &loaded_asset,
                runtime,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
        }
    }

    sort_outputs(&mut outputs);
    if outputs.is_empty() {
        reason_codes.insert(VisionReasonCode::InsufficientEvidence.as_str().to_string());
    }

    build_vision_evidence_packet(
        &request,
        retrieved_at_ms,
        provider_runs,
        outputs,
        reason_codes,
    )
}

fn run_ocr(
    request: &VisionToolRequestPacket,
    asset: &LoadedAsset,
    runtime: &VisionRuntime,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    let ocr_request = OcrRequest {
        asset_hash: request.asset_ref.asset_hash.clone(),
        mime_type: asset.mime_type.clone(),
        bytes: asset.bytes.clone(),
        language_hint: request.options.language_hint.clone(),
        safe_mode: request.options.safe_mode,
        timeout_ms: request.budgets.timeout_ms,
    };

    match runtime.ocr_backend.extract_text(&ocr_request) {
        Ok(result) => {
            let mut normalized = result;
            normalized
                .text_blocks
                .retain(|block| allow_ocr_block(block.confidence));
            normalized.text_blocks.sort_by(|a, b| {
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
            normalized.full_text = normalized
                .text_blocks
                .iter()
                .map(|block| block.text.clone())
                .collect::<Vec<String>>()
                .join("\n");

            provider_runs.push(VisionProviderRun {
                provider_id: "vision_ocr".to_string(),
                endpoint: "ocr".to_string(),
                latency_ms: 0,
                error: None,
            });
            if normalized.text_blocks.is_empty() {
                reason_codes.insert(VisionReasonCode::InsufficientEvidence.as_str().to_string());
            }
            outputs.push(VisionOutput::OCRResult(normalized));
        }
        Err(err) => {
            reason_codes.insert(err.reason_code.as_str().to_string());
            provider_runs.push(error_run(&err));
        }
    }
}

fn run_objects(
    request: &VisionToolRequestPacket,
    asset: &LoadedAsset,
    runtime: &VisionRuntime,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    let object_request = ObjectRequest {
        asset_hash: request.asset_ref.asset_hash.clone(),
        mime_type: asset.mime_type.clone(),
        bytes: asset.bytes.clone(),
        timeout_ms: request.budgets.timeout_ms,
    };

    match runtime.object_backend.detect_objects(&object_request) {
        Ok(result) => {
            let mut normalized = result;
            normalized
                .objects
                .retain(|object| allow_object(object.confidence));
            normalized.objects.sort_by(|a, b| {
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

            provider_runs.push(VisionProviderRun {
                provider_id: "vision_objects".to_string(),
                endpoint: "objects".to_string(),
                latency_ms: 0,
                error: None,
            });
            if normalized.objects.is_empty() {
                reason_codes.insert(VisionReasonCode::InsufficientEvidence.as_str().to_string());
            }
            outputs.push(VisionOutput::ObjectDetectionResult(normalized));
        }
        Err(err) => {
            reason_codes.insert(err.reason_code.as_str().to_string());
            provider_runs.push(error_run(&err));
        }
    }
}

fn run_video_transcription(
    request: &VisionToolRequestPacket,
    asset: &LoadedAsset,
    runtime: &VisionRuntime,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    match runtime.audio_extractor.extract_audio(
        &request.asset_ref.asset_hash,
        &asset.mime_type,
        &asset.bytes,
    ) {
        Ok(audio_bytes) => {
            let stt_request = SttRequest {
                language: request
                    .options
                    .language_hint
                    .clone()
                    .unwrap_or_else(|| "en-US".to_string()),
                audio_wav_bytes: audio_bytes,
                timeout_ms: request.budgets.timeout_ms,
            };
            match runtime.stt_backend.transcribe(&stt_request) {
                Ok(result) => {
                    let mut normalized: VideoTranscriptResult = result;
                    normalized.segments.retain(|segment| {
                        allow_transcript_segment(segment.confidence as f64 / 100.0)
                    });
                    normalized.segments.sort_by(|a, b| {
                        a.start_ms
                            .cmp(&b.start_ms)
                            .then(a.end_ms.cmp(&b.end_ms))
                            .then(a.text.cmp(&b.text))
                    });
                    normalized.full_transcript = normalized
                        .segments
                        .iter()
                        .map(|segment| segment.text.clone())
                        .collect::<Vec<String>>()
                        .join("\n");

                    provider_runs.push(VisionProviderRun {
                        provider_id: "google_stt".to_string(),
                        endpoint: "stt".to_string(),
                        latency_ms: 0,
                        error: None,
                    });
                    if normalized.segments.is_empty() {
                        reason_codes
                            .insert(VisionReasonCode::InsufficientEvidence.as_str().to_string());
                    }
                    outputs.push(VisionOutput::VideoTranscriptResult(normalized));
                }
                Err(err) => {
                    reason_codes.insert(err.reason_code.as_str().to_string());
                    provider_runs.push(error_run(&err));
                }
            }
        }
        Err(err) => {
            reason_codes.insert(err.reason_code.as_str().to_string());
            provider_runs.push(error_run(&err));
        }
    }
}

fn run_keyframes(
    request: &VisionToolRequestPacket,
    asset: &LoadedAsset,
    runtime: &VisionRuntime,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    let keyframe_request = KeyframeRequest {
        asset_hash: request.asset_ref.asset_hash.clone(),
        mime_type: asset.mime_type.clone(),
        video_bytes: asset.bytes.clone(),
        max_frames: request.options.max_frames.unwrap_or(12),
        frame_stride_ms: request.options.frame_stride_ms.unwrap_or(1000),
    };

    match runtime
        .keyframe_extractor
        .extract_keyframes(&keyframe_request)
    {
        Ok(result) => {
            let mut normalized = result;
            normalized.keyframes.sort_by(|a, b| {
                a.timestamp_ms
                    .cmp(&b.timestamp_ms)
                    .then(a.frame_index.cmp(&b.frame_index))
                    .then(a.frame_hash.cmp(&b.frame_hash))
            });

            provider_runs.push(VisionProviderRun {
                provider_id: "vision_keyframes".to_string(),
                endpoint: "keyframes".to_string(),
                latency_ms: 0,
                error: None,
            });
            if normalized.keyframes.is_empty() {
                reason_codes.insert(VisionReasonCode::InsufficientEvidence.as_str().to_string());
            }
            outputs.push(VisionOutput::KeyframeIndexResult(normalized));
        }
        Err(err) => {
            reason_codes.insert(err.reason_code.as_str().to_string());
            provider_runs.push(error_run(&err));
        }
    }
}

fn error_run(error: &VisionProviderError) -> VisionProviderRun {
    VisionProviderRun {
        provider_id: error.provider_id.clone(),
        endpoint: error.endpoint.clone(),
        latency_ms: error.latency_ms,
        error: Some(error.to_record()),
    }
}

fn sort_outputs(outputs: &mut [VisionOutput]) {
    outputs.sort_by(|a, b| output_sort_key(a).cmp(&output_sort_key(b)));
}

fn output_sort_key(output: &VisionOutput) -> (u8, u64, u64, String) {
    match output {
        VisionOutput::OCRResult(result) => (
            0,
            result.page_or_frame_index as u64,
            result.timestamp_ms.unwrap_or(0),
            result.ocr_engine_id.clone(),
        ),
        VisionOutput::ObjectDetectionResult(result) => (
            1,
            result.frame_index.unwrap_or(0) as u64,
            result.timestamp_ms.unwrap_or(0),
            result.model_id.clone(),
        ),
        VisionOutput::VideoTranscriptResult(result) => (
            2,
            result
                .segments
                .first()
                .map(|segment| segment.start_ms)
                .unwrap_or(0),
            result
                .segments
                .first()
                .map(|segment| segment.end_ms)
                .unwrap_or(0),
            result.stt_provider_id.clone(),
        ),
        VisionOutput::KeyframeIndexResult(result) => (
            3,
            result
                .keyframes
                .first()
                .map(|frame| frame.timestamp_ms)
                .unwrap_or(0),
            result
                .keyframes
                .first()
                .map(|frame| frame.frame_index as u64)
                .unwrap_or(0),
            result
                .keyframes
                .first()
                .map(|frame| frame.frame_hash.clone())
                .unwrap_or_default(),
        ),
    }
}

#[cfg(test)]
pub mod vision_tests;
