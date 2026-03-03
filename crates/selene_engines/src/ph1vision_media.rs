#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1vision::{
    VisionAnalyzeMediaOk, VisionAnalyzeMediaRequest, VisionAssetRef, VisionBoundingBox,
    VisionConfidenceSummary, VisionDetectedObject, VisionKeyframeEntry, VisionKeyframeIndexResult,
    VisionMediaMode, VisionMediaOutput, VisionOcrResult, VisionOcrTextBlock, VisionPacketHashes,
    VisionProviderErrorRecord, VisionProviderRun, VisionTranscriptSegment,
    VisionVideoTranscriptResult,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ALLOWED_MEDIA_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "video/mp4",
    "video/mov",
];

const OCR_BLOCK_MIN_CONFIDENCE: f64 = 0.60;
const OBJECT_MIN_CONFIDENCE: f64 = 0.55;
const TRANSCRIPT_SEGMENT_MIN_CONFIDENCE: f64 = 0.50;
const DEFAULT_MAX_FRAMES: u32 = 12;
const DEFAULT_FRAME_STRIDE_MS: u32 = 1000;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoadedAsset {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub size_bytes: u64,
    pub redacted_locator: String,
}

#[derive(Debug, Clone)]
pub struct ProviderFailure {
    pub provider_id: String,
    pub endpoint: String,
    pub error_kind: String,
    pub reason_code: String,
    pub message: String,
    pub latency_ms: u64,
}

impl ProviderFailure {
    pub fn new(
        provider_id: &str,
        endpoint: &str,
        error_kind: &str,
        reason_code: &str,
        message: &str,
        latency_ms: u64,
    ) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            endpoint: endpoint.to_string(),
            error_kind: error_kind.to_string(),
            reason_code: reason_code.to_string(),
            message: redact_error_message(message),
            latency_ms,
        }
    }

    fn to_provider_run(&self, schema_version: SchemaVersion) -> VisionProviderRun {
        VisionProviderRun {
            schema_version,
            provider_id: self.provider_id.clone(),
            endpoint: self.endpoint.clone(),
            latency_ms: self.latency_ms,
            error: Some(VisionProviderErrorRecord {
                schema_version,
                error_kind: self.error_kind.clone(),
                reason_code: self.reason_code.clone(),
                message: self.message.clone(),
            }),
        }
    }
}

pub trait VisionMediaProviders {
    fn now_ms(&self) -> i64;

    fn load_asset(
        &self,
        request: &VisionAnalyzeMediaRequest,
    ) -> Result<LoadedAsset, ProviderFailure>;

    fn extract_ocr(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionOcrResult, ProviderFailure>;

    fn detect_objects(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<selene_kernel_contracts::ph1vision::VisionObjectDetectionResult, ProviderFailure>;

    fn transcribe_video(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionVideoTranscriptResult, ProviderFailure>;

    fn extract_keyframes(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionKeyframeIndexResult, ProviderFailure>;
}

#[derive(Debug, Clone, Default)]
pub struct ProductionVisionMediaProviders;

impl VisionMediaProviders for ProductionVisionMediaProviders {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or(0)
    }

    fn load_asset(
        &self,
        request: &VisionAnalyzeMediaRequest,
    ) -> Result<LoadedAsset, ProviderFailure> {
        let start = Instant::now();
        let asset_ref = &request.asset_ref;
        let mime = normalize_mime(&asset_ref.mime_type);

        if !is_supported_mime(&mime) {
            return Err(ProviderFailure::new(
                "vision_download",
                "download",
                "unsupported_media_type",
                "policy_violation",
                "unsupported media type",
                0,
            ));
        }

        if asset_ref.size_bytes > request.budgets.max_bytes {
            return Err(ProviderFailure::new(
                "vision_download",
                "download",
                "policy_violation",
                "policy_violation",
                "asset size exceeds budget cap",
                0,
            ));
        }

        let redacted_locator = redact_locator(&asset_ref.locator);
        if is_http_locator(&asset_ref.locator) {
            load_remote_asset(
                asset_ref,
                &mime,
                request.budgets.timeout_ms,
                request.budgets.max_bytes,
                redacted_locator,
                start,
            )
        } else {
            load_local_asset(
                asset_ref,
                &mime,
                request.budgets.max_bytes,
                redacted_locator,
                start,
            )
        }
    }

    fn extract_ocr(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionOcrResult, ProviderFailure> {
        let start = Instant::now();
        let endpoint = std::env::var("SELENE_OCR_ENDPOINT")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProviderFailure::new(
                    "vision_ocr",
                    "ocr",
                    "provider_unconfigured",
                    "provider_unconfigured",
                    "OCR endpoint is not configured",
                    0,
                )
            })?;

        let api_key = std::env::var("SELENE_OCR_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                crate::device_vault::resolve_secret(
                    selene_kernel_contracts::provider_secrets::ProviderSecretId::OpenAIApiKey
                        .as_str(),
                )
                .ok()
                .flatten()
            })
            .ok_or_else(|| {
                ProviderFailure::new(
                    "vision_ocr",
                    "ocr",
                    "provider_unconfigured",
                    "provider_unconfigured",
                    "OCR API key is not configured",
                    0,
                )
            })?;

        let payload = serde_json::json!({
            "asset_hash": request.asset_ref.asset_hash,
            "mime_type": asset.mime_type,
            "language_hint": request.options.language_hint,
            "safe_mode": request.options.safe_mode,
            "bytes_hex": hex_encode(&asset.bytes),
        });

        let response = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_read(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_write(Duration::from_millis(request.budgets.timeout_ms))
            .user_agent("selene-ph1vision-ocr/1.0")
            .try_proxy_from_env(false)
            .build()
            .post(&endpoint)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", api_key))
            .send_string(&payload.to_string())
            .map_err(|err| {
                map_transport_error("vision_ocr", "ocr", err, start.elapsed().as_millis() as u64)
            })?;

        let parsed: Value = serde_json::from_reader(response.into_reader()).map_err(|_| {
            ProviderFailure::new(
                "vision_ocr",
                "ocr",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "OCR response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let mut text_blocks = parsed
            .get("text_blocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|block| {
                parse_ocr_block(&block, request.options.safe_mode, request.schema_version)
            })
            .filter(|block| block.confidence >= OCR_BLOCK_MIN_CONFIDENCE)
            .collect::<Vec<VisionOcrTextBlock>>();

        text_blocks.sort_by(|a, b| {
            cmp_f64(a.bbox.y, b.bbox.y)
                .then(cmp_f64(a.bbox.x, b.bbox.x))
                .then(cmp_f64(b.confidence, a.confidence))
                .then(a.text.cmp(&b.text))
        });

        let full_text = text_blocks
            .iter()
            .map(|block| block.text.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let result = VisionOcrResult {
            schema_version: request.schema_version,
            page_or_frame_index: 0,
            timestamp_ms: None,
            ocr_engine_id: parsed
                .get("ocr_engine_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_ocr_engine")
                .trim()
                .to_string(),
            language: parsed
                .get("language")
                .and_then(Value::as_str)
                .or(request.options.language_hint.as_deref())
                .unwrap_or("und")
                .trim()
                .to_string(),
            text_blocks,
            full_text,
        };
        result.validate().map_err(|_| {
            ProviderFailure::new(
                "vision_ocr",
                "ocr",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "OCR normalized output failed contract",
                start.elapsed().as_millis() as u64,
            )
        })?;
        Ok(result)
    }

    fn detect_objects(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<selene_kernel_contracts::ph1vision::VisionObjectDetectionResult, ProviderFailure>
    {
        let start = Instant::now();
        let endpoint = std::env::var("SELENE_OBJECTS_ENDPOINT")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ProviderFailure::new(
                    "vision_objects",
                    "objects",
                    "provider_unconfigured",
                    "provider_unconfigured",
                    "Objects endpoint is not configured",
                    0,
                )
            })?;

        let api_key = std::env::var("SELENE_OBJECTS_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                crate::device_vault::resolve_secret(
                    selene_kernel_contracts::provider_secrets::ProviderSecretId::OpenAIApiKey
                        .as_str(),
                )
                .ok()
                .flatten()
            })
            .ok_or_else(|| {
                ProviderFailure::new(
                    "vision_objects",
                    "objects",
                    "provider_unconfigured",
                    "provider_unconfigured",
                    "Objects API key is not configured",
                    0,
                )
            })?;

        let payload = serde_json::json!({
            "asset_hash": request.asset_ref.asset_hash,
            "mime_type": asset.mime_type,
            "bytes_hex": hex_encode(&asset.bytes),
        });

        let response = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_read(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_write(Duration::from_millis(request.budgets.timeout_ms))
            .user_agent("selene-ph1vision-objects/1.0")
            .try_proxy_from_env(false)
            .build()
            .post(&endpoint)
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
            ProviderFailure::new(
                "vision_objects",
                "objects",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "objects response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let mut objects = parsed
            .get("objects")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| parse_object(&value, request.schema_version))
            .filter(|item| item.confidence >= OBJECT_MIN_CONFIDENCE)
            .collect::<Vec<VisionDetectedObject>>();

        objects.sort_by(|a, b| {
            cmp_f64(b.confidence, a.confidence)
                .then(cmp_f64(a.bbox.y, b.bbox.y))
                .then(cmp_f64(a.bbox.x, b.bbox.x))
                .then(a.label.cmp(&b.label))
        });

        let result = selene_kernel_contracts::ph1vision::VisionObjectDetectionResult {
            schema_version: request.schema_version,
            frame_index: None,
            timestamp_ms: None,
            model_id: parsed
                .get("model_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_object_model")
                .trim()
                .to_string(),
            objects,
        };

        result.validate().map_err(|_| {
            ProviderFailure::new(
                "vision_objects",
                "objects",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "objects normalized output failed contract",
                start.elapsed().as_millis() as u64,
            )
        })?;
        Ok(result)
    }

    fn transcribe_video(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionVideoTranscriptResult, ProviderFailure> {
        let start = Instant::now();
        let audio_wav_bytes = extract_audio_linear16_mono_16k(
            &request.asset_ref.asset_hash,
            &asset.mime_type,
            &asset.bytes,
        )?;

        let endpoint = std::env::var("SELENE_GOOGLE_STT_ENDPOINT")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "https://speech.googleapis.com/v1/speech:recognize".to_string());

        let api_key = std::env::var("GOOGLE_STT_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                crate::device_vault::resolve_secret(
                    selene_kernel_contracts::provider_secrets::ProviderSecretId::GoogleSttApiKey
                        .as_str(),
                )
                .ok()
                .flatten()
            })
            .ok_or_else(|| {
                ProviderFailure::new(
                    "google_stt",
                    "stt",
                    "provider_unconfigured",
                    "provider_unconfigured",
                    "Google STT API key is not configured",
                    0,
                )
            })?;

        let language = request
            .options
            .language_hint
            .clone()
            .unwrap_or_else(|| "en-US".to_string());

        let payload = serde_json::json!({
            "config": {
                "encoding": "LINEAR16",
                "sampleRateHertz": 16000,
                "languageCode": language,
                "enableWordTimeOffsets": true,
                "enableAutomaticPunctuation": false
            },
            "audio": {
                "content": base64_encode(&audio_wav_bytes)
            }
        });

        let endpoint_with_key = format!("{}?key={}", endpoint, api_key);
        let response = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_read(Duration::from_millis(request.budgets.timeout_ms))
            .timeout_write(Duration::from_millis(request.budgets.timeout_ms))
            .user_agent("selene-ph1vision-google-stt/1.0")
            .try_proxy_from_env(false)
            .build()
            .post(&endpoint_with_key)
            .set("Accept", "application/json")
            .send_string(&payload.to_string())
            .map_err(|err| {
                map_transport_error("google_stt", "stt", err, start.elapsed().as_millis() as u64)
            })?;

        let parsed: Value = serde_json::from_reader(response.into_reader()).map_err(|_| {
            ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "Google STT response parse failed",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let mut segments = parse_transcript_segments(&parsed, request.schema_version)
            .into_iter()
            .filter(|segment| {
                (segment.confidence as f64 / 100.0) >= TRANSCRIPT_SEGMENT_MIN_CONFIDENCE
            })
            .collect::<Vec<VisionTranscriptSegment>>();

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

        let result = VisionVideoTranscriptResult {
            schema_version: request.schema_version,
            stt_provider_id: "GOOGLE_STT".to_string(),
            language,
            segments,
            full_transcript,
        };
        result.validate().map_err(|_| {
            ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "transcript normalized output failed contract",
                start.elapsed().as_millis() as u64,
            )
        })?;

        Ok(result)
    }

    fn extract_keyframes(
        &self,
        request: &VisionAnalyzeMediaRequest,
        asset: &LoadedAsset,
    ) -> Result<VisionKeyframeIndexResult, ProviderFailure> {
        let start = Instant::now();
        let keyframes = extract_keyframes_ffmpeg(
            &request.asset_ref.asset_hash,
            &asset.mime_type,
            &asset.bytes,
            request.options.max_frames.unwrap_or(DEFAULT_MAX_FRAMES),
            request
                .options
                .frame_stride_ms
                .unwrap_or(DEFAULT_FRAME_STRIDE_MS),
            request.schema_version,
        )
        .map_err(|mut err| {
            err.latency_ms = start.elapsed().as_millis() as u64;
            err
        })?;
        Ok(keyframes)
    }
}

pub fn run_media_analyze_with_providers<P: VisionMediaProviders>(
    request: &VisionAnalyzeMediaRequest,
    providers: &P,
    success_reason_code: ReasonCodeId,
    degraded_reason_code: ReasonCodeId,
) -> Result<VisionAnalyzeMediaOk, ContractViolation> {
    request.validate()?;

    let mut provider_runs: Vec<VisionProviderRun> = Vec::new();
    let mut outputs: Vec<VisionMediaOutput> = Vec::new();
    let mut reason_codes: BTreeSet<String> = BTreeSet::new();

    let loaded_asset = match providers.load_asset(request) {
        Ok(asset) => asset,
        Err(error) => {
            reason_codes.insert(error.reason_code.clone());
            provider_runs.push(error.to_provider_run(request.schema_version));
            return finalize_packet(
                request,
                provider_runs,
                outputs,
                reason_codes,
                degraded_reason_code,
                providers.now_ms(),
            );
        }
    };

    if !verify_hash(&request.asset_ref.asset_hash, &loaded_asset.bytes) {
        let error = ProviderFailure::new(
            "vision_download",
            "download",
            "policy_violation",
            "policy_violation",
            "asset hash mismatch",
            0,
        );
        reason_codes.insert(error.reason_code.clone());
        provider_runs.push(error.to_provider_run(request.schema_version));
        return finalize_packet(
            request,
            provider_runs,
            outputs,
            reason_codes,
            degraded_reason_code,
            providers.now_ms(),
        );
    }

    provider_runs.push(VisionProviderRun {
        schema_version: request.schema_version,
        provider_id: "vision_download".to_string(),
        endpoint: "download".to_string(),
        latency_ms: 0,
        error: None,
    });

    let normalized_mime = normalize_mime(&loaded_asset.mime_type);
    if is_image_mode(request.mode) && !normalized_mime.starts_with("image/") {
        reason_codes.insert("policy_violation".to_string());
        return finalize_packet(
            request,
            provider_runs,
            outputs,
            reason_codes,
            degraded_reason_code,
            providers.now_ms(),
        );
    }

    if is_video_mode(request.mode) && !normalized_mime.starts_with("video/") {
        reason_codes.insert("policy_violation".to_string());
        return finalize_packet(
            request,
            provider_runs,
            outputs,
            reason_codes,
            degraded_reason_code,
            providers.now_ms(),
        );
    }

    match request.mode {
        VisionMediaMode::ImageOcr => run_ocr(
            request,
            providers,
            &loaded_asset,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMediaMode::ImageObjects => run_objects(
            request,
            providers,
            &loaded_asset,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMediaMode::ImageAnalyze => {
            run_ocr(
                request,
                providers,
                &loaded_asset,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
            run_objects(
                request,
                providers,
                &loaded_asset,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
        }
        VisionMediaMode::VideoTranscribe => run_transcript(
            request,
            providers,
            &loaded_asset,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMediaMode::VideoKeyframes => run_keyframes(
            request,
            providers,
            &loaded_asset,
            &mut provider_runs,
            &mut outputs,
            &mut reason_codes,
        ),
        VisionMediaMode::VideoAnalyze => {
            run_keyframes(
                request,
                providers,
                &loaded_asset,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
            run_transcript(
                request,
                providers,
                &loaded_asset,
                &mut provider_runs,
                &mut outputs,
                &mut reason_codes,
            );
        }
    }

    sort_outputs(&mut outputs);
    if outputs.is_empty() {
        reason_codes.insert("insufficient_evidence".to_string());
    }

    let reason_code = if reason_codes.is_empty() {
        success_reason_code
    } else {
        degraded_reason_code
    };

    finalize_packet(
        request,
        provider_runs,
        outputs,
        reason_codes,
        reason_code,
        providers.now_ms(),
    )
}

fn run_ocr<P: VisionMediaProviders>(
    request: &VisionAnalyzeMediaRequest,
    providers: &P,
    loaded_asset: &LoadedAsset,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionMediaOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    match providers.extract_ocr(request, loaded_asset) {
        Ok(mut result) => {
            result
                .text_blocks
                .retain(|block| block.confidence >= OCR_BLOCK_MIN_CONFIDENCE);
            result.text_blocks.sort_by(|a, b| {
                cmp_f64(a.bbox.y, b.bbox.y)
                    .then(cmp_f64(a.bbox.x, b.bbox.x))
                    .then(cmp_f64(b.confidence, a.confidence))
                    .then(a.text.cmp(&b.text))
            });
            result.full_text = result
                .text_blocks
                .iter()
                .map(|block| block.text.clone())
                .collect::<Vec<String>>()
                .join("\n");
            if result.text_blocks.is_empty() {
                reason_codes.insert("insufficient_evidence".to_string());
            }
            provider_runs.push(VisionProviderRun {
                schema_version: request.schema_version,
                provider_id: "vision_ocr".to_string(),
                endpoint: "ocr".to_string(),
                latency_ms: 0,
                error: None,
            });
            outputs.push(VisionMediaOutput::OCRResult(result));
        }
        Err(error) => {
            reason_codes.insert(error.reason_code.clone());
            provider_runs.push(error.to_provider_run(request.schema_version));
        }
    }
}

fn run_objects<P: VisionMediaProviders>(
    request: &VisionAnalyzeMediaRequest,
    providers: &P,
    loaded_asset: &LoadedAsset,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionMediaOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    match providers.detect_objects(request, loaded_asset) {
        Ok(mut result) => {
            result
                .objects
                .retain(|object| object.confidence >= OBJECT_MIN_CONFIDENCE);
            result.objects.sort_by(|a, b| {
                cmp_f64(b.confidence, a.confidence)
                    .then(cmp_f64(a.bbox.y, b.bbox.y))
                    .then(cmp_f64(a.bbox.x, b.bbox.x))
                    .then(a.label.cmp(&b.label))
            });
            if result.objects.is_empty() {
                reason_codes.insert("insufficient_evidence".to_string());
            }
            provider_runs.push(VisionProviderRun {
                schema_version: request.schema_version,
                provider_id: "vision_objects".to_string(),
                endpoint: "objects".to_string(),
                latency_ms: 0,
                error: None,
            });
            outputs.push(VisionMediaOutput::ObjectDetectionResult(result));
        }
        Err(error) => {
            reason_codes.insert(error.reason_code.clone());
            provider_runs.push(error.to_provider_run(request.schema_version));
        }
    }
}

fn run_transcript<P: VisionMediaProviders>(
    request: &VisionAnalyzeMediaRequest,
    providers: &P,
    loaded_asset: &LoadedAsset,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionMediaOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    match providers.transcribe_video(request, loaded_asset) {
        Ok(mut result) => {
            result.segments.retain(|segment| {
                (segment.confidence as f64 / 100.0) >= TRANSCRIPT_SEGMENT_MIN_CONFIDENCE
            });
            result.segments.sort_by(|a, b| {
                a.start_ms
                    .cmp(&b.start_ms)
                    .then(a.end_ms.cmp(&b.end_ms))
                    .then(a.text.cmp(&b.text))
            });
            result.full_transcript = result
                .segments
                .iter()
                .map(|segment| segment.text.clone())
                .collect::<Vec<String>>()
                .join("\n");
            if result.segments.is_empty() {
                reason_codes.insert("insufficient_evidence".to_string());
            }
            provider_runs.push(VisionProviderRun {
                schema_version: request.schema_version,
                provider_id: "google_stt".to_string(),
                endpoint: "stt".to_string(),
                latency_ms: 0,
                error: None,
            });
            outputs.push(VisionMediaOutput::VideoTranscriptResult(result));
        }
        Err(error) => {
            reason_codes.insert(error.reason_code.clone());
            provider_runs.push(error.to_provider_run(request.schema_version));
        }
    }
}

fn run_keyframes<P: VisionMediaProviders>(
    request: &VisionAnalyzeMediaRequest,
    providers: &P,
    loaded_asset: &LoadedAsset,
    provider_runs: &mut Vec<VisionProviderRun>,
    outputs: &mut Vec<VisionMediaOutput>,
    reason_codes: &mut BTreeSet<String>,
) {
    match providers.extract_keyframes(request, loaded_asset) {
        Ok(mut result) => {
            result.keyframes.sort_by(|a, b| {
                a.timestamp_ms
                    .cmp(&b.timestamp_ms)
                    .then(a.frame_index.cmp(&b.frame_index))
                    .then(a.frame_hash.cmp(&b.frame_hash))
            });
            if result.keyframes.is_empty() {
                reason_codes.insert("insufficient_evidence".to_string());
            }
            provider_runs.push(VisionProviderRun {
                schema_version: request.schema_version,
                provider_id: "vision_keyframes".to_string(),
                endpoint: "keyframes".to_string(),
                latency_ms: 0,
                error: None,
            });
            outputs.push(VisionMediaOutput::KeyframeIndexResult(result));
        }
        Err(error) => {
            reason_codes.insert(error.reason_code.clone());
            provider_runs.push(error.to_provider_run(request.schema_version));
        }
    }
}

fn finalize_packet(
    request: &VisionAnalyzeMediaRequest,
    provider_runs: Vec<VisionProviderRun>,
    outputs: Vec<VisionMediaOutput>,
    reason_codes: BTreeSet<String>,
    reason_code: ReasonCodeId,
    _retrieved_at_ms: i64,
) -> Result<VisionAnalyzeMediaOk, ContractViolation> {
    let output_hash = sha256_hex(canonical_outputs(&outputs).as_bytes());
    let provider_runs_hash = sha256_hex(canonical_provider_runs(&provider_runs).as_bytes());

    let confidence_summary = build_confidence_summary(request.schema_version, &outputs);
    let packet_hashes = VisionPacketHashes {
        schema_version: request.schema_version,
        asset_hash: request.asset_ref.asset_hash.clone(),
        provider_runs_hash,
        outputs_hash: output_hash.clone(),
    };

    let ok = VisionAnalyzeMediaOk::v1(
        reason_code,
        VisionAssetRef {
            schema_version: request.schema_version,
            asset_hash: request.asset_ref.asset_hash.clone(),
            locator: request.asset_ref.locator.clone(),
            mime_type: request.asset_ref.mime_type.clone(),
            size_bytes: request.asset_ref.size_bytes,
        },
        provider_runs,
        outputs,
        confidence_summary,
        reason_codes.into_iter().collect(),
        packet_hashes,
        output_hash,
    )?;

    Ok(ok)
}

fn build_confidence_summary(
    schema_version: SchemaVersion,
    outputs: &[VisionMediaOutput],
) -> VisionConfidenceSummary {
    let mut score_total = 0.0f64;
    let mut score_count = 0u64;
    let mut ocr_blocks = 0u64;
    let mut object_count = 0u64;
    let mut transcript_segments = 0u64;

    for output in outputs {
        match output {
            VisionMediaOutput::OCRResult(result) => {
                for block in &result.text_blocks {
                    score_total += block.confidence;
                    score_count += 1;
                    ocr_blocks += 1;
                }
            }
            VisionMediaOutput::ObjectDetectionResult(result) => {
                for object in &result.objects {
                    score_total += object.confidence;
                    score_count += 1;
                    object_count += 1;
                }
            }
            VisionMediaOutput::VideoTranscriptResult(result) => {
                for segment in &result.segments {
                    score_total += segment.confidence as f64 / 100.0;
                    score_count += 1;
                    transcript_segments += 1;
                }
            }
            VisionMediaOutput::KeyframeIndexResult(result) => {
                if !result.keyframes.is_empty() {
                    score_total += 1.0;
                    score_count += 1;
                }
            }
        }
    }

    let mean_confidence = if score_count == 0 {
        0.0
    } else {
        score_total / score_count as f64
    };

    VisionConfidenceSummary {
        schema_version,
        mean_confidence,
        ocr_blocks_retained: ocr_blocks,
        objects_retained: object_count,
        transcript_segments_retained: transcript_segments,
        output_count: outputs.len() as u64,
    }
}

fn sort_outputs(outputs: &mut [VisionMediaOutput]) {
    outputs.sort_by(|a, b| output_sort_key(a).cmp(&output_sort_key(b)));
}

fn output_sort_key(output: &VisionMediaOutput) -> (u8, u64, u64, String) {
    match output {
        VisionMediaOutput::OCRResult(result) => (
            0,
            result.page_or_frame_index as u64,
            result.timestamp_ms.unwrap_or(0),
            result.ocr_engine_id.clone(),
        ),
        VisionMediaOutput::ObjectDetectionResult(result) => (
            1,
            result.frame_index.unwrap_or(0) as u64,
            result.timestamp_ms.unwrap_or(0),
            result.model_id.clone(),
        ),
        VisionMediaOutput::VideoTranscriptResult(result) => (
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
        VisionMediaOutput::KeyframeIndexResult(result) => (
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

fn is_image_mode(mode: VisionMediaMode) -> bool {
    matches!(
        mode,
        VisionMediaMode::ImageOcr | VisionMediaMode::ImageObjects | VisionMediaMode::ImageAnalyze
    )
}

fn is_video_mode(mode: VisionMediaMode) -> bool {
    matches!(
        mode,
        VisionMediaMode::VideoTranscribe
            | VisionMediaMode::VideoKeyframes
            | VisionMediaMode::VideoAnalyze
    )
}

fn normalize_mime(mime_type: &str) -> String {
    mime_type.trim().to_ascii_lowercase()
}

fn is_supported_mime(mime_type: &str) -> bool {
    ALLOWED_MEDIA_MIME_TYPES
        .iter()
        .any(|mime| *mime == mime_type)
}

fn verify_hash(expected_sha256: &str, bytes: &[u8]) -> bool {
    sha256_hex(bytes) == expected_sha256.trim().to_ascii_lowercase()
}

fn parse_ocr_block(
    value: &Value,
    safe_mode: bool,
    schema_version: SchemaVersion,
) -> Option<VisionOcrTextBlock> {
    let bbox_obj = value.get("bbox")?.as_object()?;
    let bbox = VisionBoundingBox::new(
        bbox_obj.get("x")?.as_f64()?,
        bbox_obj.get("y")?.as_f64()?,
        bbox_obj.get("w")?.as_f64()?,
        bbox_obj.get("h")?.as_f64()?,
    )
    .ok()?;

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

    let block = VisionOcrTextBlock {
        schema_version,
        bbox,
        text,
        confidence,
        pii_suspected,
    };
    block.validate().ok()?;
    Some(block)
}

fn parse_object(value: &Value, schema_version: SchemaVersion) -> Option<VisionDetectedObject> {
    let object = value.as_object()?;
    let bbox_obj = object.get("bbox")?.as_object()?;
    let label = canonicalize_label(object.get("label")?.as_str()?);
    let confidence = object.get("confidence")?.as_f64()?;
    let parsed = VisionDetectedObject {
        schema_version,
        label,
        bbox: VisionBoundingBox::new(
            bbox_obj.get("x")?.as_f64()?,
            bbox_obj.get("y")?.as_f64()?,
            bbox_obj.get("w")?.as_f64()?,
            bbox_obj.get("h")?.as_f64()?,
        )
        .ok()?,
        confidence,
    };
    parsed.validate().ok()?;
    Some(parsed)
}

fn parse_transcript_segments(
    payload: &Value,
    schema_version: SchemaVersion,
) -> Vec<VisionTranscriptSegment> {
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
        let segment = VisionTranscriptSegment {
            schema_version,
            start_ms,
            end_ms,
            text,
            confidence: (confidence * 100.0).round().clamp(0.0, 100.0) as u32,
        };
        if segment.validate().is_ok() {
            segments.push(segment);
        }
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

fn canonicalize_label(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase().replace(' ', "_");
    match normalized.as_str() {
        "human" => "person".to_string(),
        "automobile" => "car".to_string(),
        _ => normalized,
    }
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn looks_like_pii(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains('@') {
        return true;
    }
    lower.chars().filter(|ch| ch.is_ascii_digit()).count() >= 9
}

fn cmp_f64(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

fn load_local_asset(
    asset_ref: &VisionAssetRef,
    mime_type: &str,
    max_bytes: u64,
    redacted_locator: String,
    start: Instant,
) -> Result<LoadedAsset, ProviderFailure> {
    let path = local_path_from_locator(&asset_ref.locator).ok_or_else(|| {
        ProviderFailure::new(
            "vision_download",
            "download",
            "policy_violation",
            "policy_violation",
            "invalid local asset locator",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let metadata = fs::metadata(&path).map_err(|_| {
        ProviderFailure::new(
            "vision_download",
            "download",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "failed to read local asset metadata",
            start.elapsed().as_millis() as u64,
        )
    })?;
    if metadata.len() > max_bytes {
        return Err(ProviderFailure::new(
            "vision_download",
            "download",
            "policy_violation",
            "policy_violation",
            "local asset exceeds max_bytes",
            start.elapsed().as_millis() as u64,
        ));
    }

    let bytes = fs::read(path).map_err(|_| {
        ProviderFailure::new(
            "vision_download",
            "download",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "failed to read local asset",
            start.elapsed().as_millis() as u64,
        )
    })?;

    Ok(LoadedAsset {
        bytes,
        mime_type: mime_type.to_string(),
        size_bytes: metadata.len(),
        redacted_locator,
    })
}

fn load_remote_asset(
    asset_ref: &VisionAssetRef,
    fallback_mime_type: &str,
    timeout_ms: u64,
    max_bytes: u64,
    redacted_locator: String,
    start: Instant,
) -> Result<LoadedAsset, ProviderFailure> {
    let response = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .user_agent("selene-ph1vision-download/1.0")
        .try_proxy_from_env(false)
        .build()
        .get(&asset_ref.locator)
        .set("Accept", &ALLOWED_MEDIA_MIME_TYPES.join(","))
        .call()
        .map_err(|err| {
            map_transport_error(
                "vision_download",
                "download",
                err,
                start.elapsed().as_millis() as u64,
            )
        })?;

    let response_mime = response
        .header("Content-Type")
        .and_then(|raw| raw.split(';').next())
        .map(normalize_mime)
        .unwrap_or_else(|| fallback_mime_type.to_string());

    if !is_supported_mime(&response_mime) {
        return Err(ProviderFailure::new(
            "vision_download",
            "download",
            "unsupported_media_type",
            "policy_violation",
            "remote MIME type is not allowed",
            start.elapsed().as_millis() as u64,
        ));
    }

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = reader.read(&mut buffer).map_err(|_| {
            ProviderFailure::new(
                "vision_download",
                "download",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "failed while streaming asset bytes",
                start.elapsed().as_millis() as u64,
            )
        })?;

        if read == 0 {
            break;
        }

        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() as u64 > max_bytes {
            return Err(ProviderFailure::new(
                "vision_download",
                "download",
                "policy_violation",
                "policy_violation",
                "remote asset exceeded max_bytes",
                start.elapsed().as_millis() as u64,
            ));
        }
    }

    Ok(LoadedAsset {
        size_bytes: bytes.len() as u64,
        bytes,
        mime_type: response_mime,
        redacted_locator,
    })
}

fn extract_audio_linear16_mono_16k(
    asset_hash: &str,
    mime_type: &str,
    video_bytes: &[u8],
) -> Result<Vec<u8>, ProviderFailure> {
    let start = Instant::now();
    let input_path = write_temp_asset(asset_hash, mime_type, video_bytes).map_err(|_| {
        ProviderFailure::new(
            "vision_video",
            "stt",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "failed to stage video asset",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let output_path = temp_root().join(format!("{}_audio.wav", asset_hash));

    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(&input_path)
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-f")
        .arg("wav")
        .arg(&output_path)
        .status();

    let result = match status {
        Ok(exit) if exit.success() => fs::read(&output_path).map_err(|_| {
            ProviderFailure::new(
                "vision_video",
                "stt",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "failed to read extracted audio",
                start.elapsed().as_millis() as u64,
            )
        }),
        Ok(_) => Err(ProviderFailure::new(
            "vision_video",
            "stt",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "ffmpeg audio extraction failed",
            start.elapsed().as_millis() as u64,
        )),
        Err(_) => Err(ProviderFailure::new(
            "vision_video",
            "stt",
            "provider_unconfigured",
            "provider_unconfigured",
            "ffmpeg is not available",
            start.elapsed().as_millis() as u64,
        )),
    };

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path);
    result
}

fn extract_keyframes_ffmpeg(
    asset_hash: &str,
    mime_type: &str,
    video_bytes: &[u8],
    max_frames: u32,
    frame_stride_ms: u32,
    schema_version: SchemaVersion,
) -> Result<VisionKeyframeIndexResult, ProviderFailure> {
    let start = Instant::now();
    let input_path = write_temp_asset(asset_hash, mime_type, video_bytes).map_err(|_| {
        ProviderFailure::new(
            "vision_keyframes",
            "keyframes",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "failed to stage video asset",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let frames_dir = temp_root().join(format!("{}_frames", asset_hash));
    let _ = fs::remove_dir_all(&frames_dir);
    fs::create_dir_all(&frames_dir).map_err(|_| {
        ProviderFailure::new(
            "vision_keyframes",
            "keyframes",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "failed to create keyframe directory",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let stride_ms = frame_stride_ms.max(1);
    let fps = format!("fps=1/{}", stride_ms as f64 / 1000.0);
    let output_pattern = frames_dir.join("frame_%06d.png");

    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(&input_path)
        .arg("-vf")
        .arg(format!("{},scale=640:-1:flags=lanczos,format=rgb24", fps))
        .arg("-frames:v")
        .arg(max_frames.max(1).to_string())
        .arg(&output_pattern)
        .status();

    if let Err(_) = status {
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_dir_all(&frames_dir);
        return Err(ProviderFailure::new(
            "vision_keyframes",
            "keyframes",
            "provider_unconfigured",
            "provider_unconfigured",
            "ffmpeg is not available",
            start.elapsed().as_millis() as u64,
        ));
    }

    if !status.expect("checked above").success() {
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_dir_all(&frames_dir);
        return Err(ProviderFailure::new(
            "vision_keyframes",
            "keyframes",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "ffmpeg keyframe extraction failed",
            start.elapsed().as_millis() as u64,
        ));
    }

    let mut frame_paths: Vec<PathBuf> = fs::read_dir(&frames_dir)
        .map_err(|_| {
            ProviderFailure::new(
                "vision_keyframes",
                "keyframes",
                "provider_upstream_failed",
                "provider_upstream_failed",
                "failed to read keyframe directory",
                start.elapsed().as_millis() as u64,
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
        .collect();

    frame_paths.sort();

    let keyframes = frame_paths
        .iter()
        .enumerate()
        .map(|(idx, path)| {
            let bytes = fs::read(path).map_err(|_| {
                ProviderFailure::new(
                    "vision_keyframes",
                    "keyframes",
                    "provider_upstream_failed",
                    "provider_upstream_failed",
                    "failed to read keyframe bytes",
                    start.elapsed().as_millis() as u64,
                )
            })?;

            Ok(VisionKeyframeEntry {
                schema_version,
                timestamp_ms: idx as u64 * stride_ms as u64,
                frame_index: idx as u32,
                frame_hash: sha256_hex(&bytes),
            })
        })
        .collect::<Result<Vec<VisionKeyframeEntry>, ProviderFailure>>()?;

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_dir_all(&frames_dir);

    let result = VisionKeyframeIndexResult {
        schema_version,
        keyframes,
    };
    result.validate().map_err(|_| {
        ProviderFailure::new(
            "vision_keyframes",
            "keyframes",
            "provider_upstream_failed",
            "provider_upstream_failed",
            "keyframe normalized output failed contract",
            start.elapsed().as_millis() as u64,
        )
    })?;
    Ok(result)
}

fn local_path_from_locator(locator: &str) -> Option<PathBuf> {
    if locator.trim().is_empty() {
        return None;
    }

    if let Some(stripped) = locator.strip_prefix("file://") {
        return Some(PathBuf::from(stripped));
    }

    Some(PathBuf::from(locator))
}

fn is_http_locator(locator: &str) -> bool {
    let lower = locator.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn redact_locator(locator: &str) -> String {
    let trimmed = locator.trim();
    if trimmed.is_empty() {
        return "asset://redacted/empty".to_string();
    }

    if is_http_locator(trimmed) {
        let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        let without_userinfo = if let Some((scheme, rest)) = without_query.split_once("://") {
            if let Some(at) = rest.find('@') {
                format!("{}://{}", scheme, &rest[(at + 1)..])
            } else {
                without_query.to_string()
            }
        } else {
            "asset://redacted/invalid-url".to_string()
        };
        return without_userinfo;
    }

    let locator_hash = sha256_hex(trimmed.as_bytes());
    format!("asset://local/{}", &locator_hash[..16])
}

pub fn redact_error_message(message: &str) -> String {
    let mut out = message.replace("api_key", "[redacted_key]");
    out = out.replace("authorization", "[redacted_auth]");

    if let Some(index) = out.to_ascii_lowercase().find("sk-") {
        out.replace_range(index.., "[redacted_secret]");
    }

    out
}

fn write_temp_asset(asset_hash: &str, mime_type: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    let dir = temp_root();
    fs::create_dir_all(&dir)?;
    let extension = file_extension_for_mime(mime_type);
    let path = dir.join(format!("{}.{}", asset_hash, extension));
    fs::write(&path, bytes)?;
    Ok(path)
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join("selene_ph1vision_media")
}

fn file_extension_for_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/mov" => "mov",
        _ => "bin",
    }
}

fn map_transport_error(
    provider_id: &str,
    endpoint: &str,
    err: ureq::Error,
    latency_ms: u64,
) -> ProviderFailure {
    match err {
        ureq::Error::Status(status, _) => ProviderFailure::new(
            provider_id,
            endpoint,
            "provider_upstream_failed",
            "provider_upstream_failed",
            &format!("HTTP status {}", status),
            latency_ms,
        ),
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            if combined.contains("timeout") {
                ProviderFailure::new(
                    provider_id,
                    endpoint,
                    "timeout_exceeded",
                    "timeout_exceeded",
                    "transport timeout",
                    latency_ms,
                )
            } else {
                ProviderFailure::new(
                    provider_id,
                    endpoint,
                    "provider_upstream_failed",
                    "provider_upstream_failed",
                    "transport failure",
                    latency_ms,
                )
            }
        }
    }
}

fn canonical_provider_runs(provider_runs: &[VisionProviderRun]) -> String {
    let mut out = String::new();
    for run in provider_runs {
        out.push_str(&format!(
            "p:{}|e:{}|l:{}|",
            run.provider_id, run.endpoint, run.latency_ms
        ));
        if let Some(error) = &run.error {
            out.push_str(&format!(
                "ek:{}|rc:{}|m:{}|",
                error.error_kind, error.reason_code, error.message
            ));
        }
    }
    out
}

fn canonical_outputs(outputs: &[VisionMediaOutput]) -> String {
    let mut out = String::new();
    for output in outputs {
        match output {
            VisionMediaOutput::OCRResult(result) => {
                out.push_str(&format!(
                    "ocr:{}:{}:{}:{}|",
                    result.page_or_frame_index,
                    result.timestamp_ms.unwrap_or(0),
                    result.ocr_engine_id,
                    result.language
                ));
                for block in &result.text_blocks {
                    out.push_str(&format!(
                        "b:{:.6}:{:.6}:{:.6}:{:.6}:{}:{:.6}:{}|",
                        block.bbox.x,
                        block.bbox.y,
                        block.bbox.w,
                        block.bbox.h,
                        block.text,
                        block.confidence,
                        block.pii_suspected.unwrap_or(false)
                    ));
                }
                out.push_str(&format!("full:{}|", result.full_text));
            }
            VisionMediaOutput::ObjectDetectionResult(result) => {
                out.push_str(&format!(
                    "obj:{}:{}:{}|",
                    result.frame_index.unwrap_or(0),
                    result.timestamp_ms.unwrap_or(0),
                    result.model_id
                ));
                for object in &result.objects {
                    out.push_str(&format!(
                        "o:{}:{:.6}:{:.6}:{:.6}:{:.6}:{:.6}|",
                        object.label,
                        object.bbox.x,
                        object.bbox.y,
                        object.bbox.w,
                        object.bbox.h,
                        object.confidence
                    ));
                }
            }
            VisionMediaOutput::VideoTranscriptResult(result) => {
                out.push_str(&format!(
                    "stt:{}:{}|",
                    result.stt_provider_id, result.language
                ));
                for segment in &result.segments {
                    out.push_str(&format!(
                        "s:{}:{}:{}:{}|",
                        segment.start_ms, segment.end_ms, segment.text, segment.confidence
                    ));
                }
                out.push_str(&format!("full:{}|", result.full_transcript));
            }
            VisionMediaOutput::KeyframeIndexResult(result) => {
                out.push_str("kf|");
                for frame in &result.keyframes {
                    out.push_str(&format!(
                        "k:{}:{}:{}|",
                        frame.timestamp_ms, frame.frame_index, frame.frame_hash
                    ));
                }
            }
        }
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
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

#[cfg(test)]
pub mod tests_support {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct FixedClockProviders {
        pub now_ms_value: i64,
        pub load_result: Result<LoadedAsset, ProviderFailure>,
        pub ocr_result: Result<VisionOcrResult, ProviderFailure>,
        pub objects_result: Result<
            selene_kernel_contracts::ph1vision::VisionObjectDetectionResult,
            ProviderFailure,
        >,
        pub transcript_result: Result<VisionVideoTranscriptResult, ProviderFailure>,
        pub keyframes_result: Result<VisionKeyframeIndexResult, ProviderFailure>,
    }

    impl VisionMediaProviders for FixedClockProviders {
        fn now_ms(&self) -> i64 {
            self.now_ms_value
        }

        fn load_asset(
            &self,
            _request: &VisionAnalyzeMediaRequest,
        ) -> Result<LoadedAsset, ProviderFailure> {
            self.load_result.clone()
        }

        fn extract_ocr(
            &self,
            _request: &VisionAnalyzeMediaRequest,
            _asset: &LoadedAsset,
        ) -> Result<VisionOcrResult, ProviderFailure> {
            self.ocr_result.clone()
        }

        fn detect_objects(
            &self,
            _request: &VisionAnalyzeMediaRequest,
            _asset: &LoadedAsset,
        ) -> Result<selene_kernel_contracts::ph1vision::VisionObjectDetectionResult, ProviderFailure>
        {
            self.objects_result.clone()
        }

        fn transcribe_video(
            &self,
            _request: &VisionAnalyzeMediaRequest,
            _asset: &LoadedAsset,
        ) -> Result<VisionVideoTranscriptResult, ProviderFailure> {
            self.transcript_result.clone()
        }

        fn extract_keyframes(
            &self,
            _request: &VisionAnalyzeMediaRequest,
            _asset: &LoadedAsset,
        ) -> Result<VisionKeyframeIndexResult, ProviderFailure> {
            self.keyframes_result.clone()
        }
    }

    pub fn sha256_for_tests(bytes: &[u8]) -> String {
        sha256_hex(bytes)
    }

    pub fn redact_error_message_for_tests(message: &str) -> String {
        redact_error_message(message)
    }
}
