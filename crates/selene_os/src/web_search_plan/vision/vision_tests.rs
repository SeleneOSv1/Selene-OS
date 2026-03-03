#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::packet_validator::{
    packet_name_from_fixture_filename, validate_packet, validate_packet_schema_registry,
};
use crate::web_search_plan::registry_loader::{fixtures_dir, load_packet_schema_registry};
use crate::web_search_plan::vision::asset_ref::VisionAssetRef;
use crate::web_search_plan::vision::download::{AssetLoader, LoadedAsset};
use crate::web_search_plan::vision::keyframes_ffmpeg::{
    KeyframeEntry, KeyframeExtractor, KeyframeIndexResult, KeyframeRequest,
};
use crate::web_search_plan::vision::objects::{
    DetectedObject, ObjectBackend, ObjectDetectionResult, ObjectRequest,
};
use crate::web_search_plan::vision::ocr::{
    BoundingBox, OcrBackend, OcrRequest, OcrResult, OcrTextBlock,
};
use crate::web_search_plan::vision::stt_google::{
    SttBackend, SttRequest, TranscriptSegment, VideoTranscriptResult,
};
use crate::web_search_plan::vision::video::AudioExtractor;
use crate::web_search_plan::vision::{
    redaction, run_vision_tool_with_runtime, VisionBudgets, VisionClock, VisionEvidencePacket,
    VisionMode, VisionOptions, VisionProviderError, VisionProviderErrorKind, VisionReasonCode,
    VisionRuntime, VisionToolRequestPacket,
};
use serde_json::Value;
use std::fs;

#[derive(Debug, Clone)]
struct FixedClock(i64);

impl VisionClock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone)]
struct MockAssetLoader {
    result: Result<LoadedAsset, VisionProviderError>,
}

impl AssetLoader for MockAssetLoader {
    fn load(&self, _request: &VisionToolRequestPacket) -> Result<LoadedAsset, VisionProviderError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct MockOcrBackend {
    result: Result<OcrResult, VisionProviderError>,
}

impl OcrBackend for MockOcrBackend {
    fn extract_text(&self, _request: &OcrRequest) -> Result<OcrResult, VisionProviderError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct MockObjectBackend {
    result: Result<ObjectDetectionResult, VisionProviderError>,
}

impl ObjectBackend for MockObjectBackend {
    fn detect_objects(
        &self,
        _request: &ObjectRequest,
    ) -> Result<ObjectDetectionResult, VisionProviderError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct MockSttBackend {
    result: Result<VideoTranscriptResult, VisionProviderError>,
}

impl SttBackend for MockSttBackend {
    fn transcribe(
        &self,
        _request: &SttRequest,
    ) -> Result<VideoTranscriptResult, VisionProviderError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct MockKeyframeExtractor {
    result: Result<KeyframeIndexResult, VisionProviderError>,
}

impl KeyframeExtractor for MockKeyframeExtractor {
    fn extract_keyframes(
        &self,
        _request: &KeyframeRequest,
    ) -> Result<KeyframeIndexResult, VisionProviderError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct MockAudioExtractor {
    result: Result<Vec<u8>, VisionProviderError>,
}

impl AudioExtractor for MockAudioExtractor {
    fn extract_audio(
        &self,
        _asset_hash: &str,
        _mime_type: &str,
        _video_bytes: &[u8],
    ) -> Result<Vec<u8>, VisionProviderError> {
        self.result.clone()
    }
}

fn provider_error(
    provider_id: &str,
    endpoint: &str,
    kind: VisionProviderErrorKind,
    reason: VisionReasonCode,
    message: &str,
) -> VisionProviderError {
    VisionProviderError::new(provider_id, endpoint, kind, reason, message, 0)
}

fn base_request(mode: VisionMode, mime_type: &str, bytes: &[u8]) -> VisionToolRequestPacket {
    VisionToolRequestPacket {
        schema_version: "1.0.0".to_string(),
        produced_by: "PH1.X".to_string(),
        intended_consumers: vec!["PH1.D".to_string(), "PH1.WRITE".to_string()],
        created_at_ms: 1_778_000_000_000,
        trace_id: "trace-vision-tests".to_string(),
        mode,
        asset_ref: VisionAssetRef {
            asset_hash: sha256_hex(bytes),
            locator: "asset://fixtures/test".to_string(),
            mime_type: mime_type.to_string(),
            size_bytes: bytes.len() as u64,
        },
        options: VisionOptions {
            language_hint: Some("en-US".to_string()),
            max_frames: Some(3),
            frame_stride_ms: Some(1000),
            safe_mode: true,
        },
        budgets: VisionBudgets {
            timeout_ms: 1500,
            max_bytes: 1024 * 1024,
        },
        policy_snapshot_id: "policy-vision-1".to_string(),
    }
}

fn loaded_asset(bytes: &[u8], mime: &str) -> LoadedAsset {
    LoadedAsset {
        bytes: bytes.to_vec(),
        mime_type: mime.to_string(),
        size_bytes: bytes.len() as u64,
        redacted_locator: "asset://local/redacted".to_string(),
    }
}

fn runtime_with_mocks(
    loader: Result<LoadedAsset, VisionProviderError>,
    ocr: Result<OcrResult, VisionProviderError>,
    objects: Result<ObjectDetectionResult, VisionProviderError>,
    stt: Result<VideoTranscriptResult, VisionProviderError>,
    keyframes: Result<KeyframeIndexResult, VisionProviderError>,
    audio: Result<Vec<u8>, VisionProviderError>,
) -> VisionRuntime {
    VisionRuntime {
        asset_loader: Box::new(MockAssetLoader { result: loader }),
        ocr_backend: Box::new(MockOcrBackend { result: ocr }),
        object_backend: Box::new(MockObjectBackend { result: objects }),
        stt_backend: Box::new(MockSttBackend { result: stt }),
        keyframe_extractor: Box::new(MockKeyframeExtractor { result: keyframes }),
        audio_extractor: Box::new(MockAudioExtractor { result: audio }),
        clock: Box::new(FixedClock(1_778_000_000_123)),
    }
}

fn noop_ocr() -> Result<OcrResult, VisionProviderError> {
    Err(provider_error(
        "vision_ocr",
        "ocr",
        VisionProviderErrorKind::ProviderUnconfigured,
        VisionReasonCode::ProviderUnconfigured,
        "ocr unavailable",
    ))
}

fn noop_objects() -> Result<ObjectDetectionResult, VisionProviderError> {
    Err(provider_error(
        "vision_objects",
        "objects",
        VisionProviderErrorKind::ProviderUnconfigured,
        VisionReasonCode::ProviderUnconfigured,
        "objects unavailable",
    ))
}

fn noop_stt() -> Result<VideoTranscriptResult, VisionProviderError> {
    Err(provider_error(
        "google_stt",
        "stt",
        VisionProviderErrorKind::ProviderUnconfigured,
        VisionReasonCode::ProviderUnconfigured,
        "stt unavailable",
    ))
}

fn noop_keyframes() -> Result<KeyframeIndexResult, VisionProviderError> {
    Err(provider_error(
        "vision_keyframes",
        "keyframes",
        VisionProviderErrorKind::ProviderUnconfigured,
        VisionReasonCode::ProviderUnconfigured,
        "keyframes unavailable",
    ))
}

#[test]
fn test_t1_deterministic_ordering_of_ocr_blocks() {
    let bytes = b"image-bytes";
    let request = base_request(VisionMode::ImageOcr, "image/png", bytes);

    let ocr_result = OcrResult {
        page_or_frame_index: 0,
        timestamp_ms: None,
        ocr_engine_id: "ocr-test".to_string(),
        language: "en-US".to_string(),
        text_blocks: vec![
            OcrTextBlock {
                bbox: BoundingBox {
                    x: 30.0,
                    y: 10.0,
                    w: 5.0,
                    h: 5.0,
                },
                text: "beta".to_string(),
                confidence: 0.91,
                pii_suspected: Some(false),
            },
            OcrTextBlock {
                bbox: BoundingBox {
                    x: 10.0,
                    y: 5.0,
                    w: 5.0,
                    h: 5.0,
                },
                text: "alpha".to_string(),
                confidence: 0.80,
                pii_suspected: Some(false),
            },
            OcrTextBlock {
                bbox: BoundingBox {
                    x: 5.0,
                    y: 20.0,
                    w: 5.0,
                    h: 5.0,
                },
                text: "low".to_string(),
                confidence: 0.10,
                pii_suspected: Some(false),
            },
        ],
        full_text: "beta\nalpha\nlow".to_string(),
    };

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "image/png")),
        Ok(ocr_result),
        noop_objects(),
        noop_stt(),
        noop_keyframes(),
        Ok(Vec::new()),
    );

    let first = run_vision_tool_with_runtime(request.clone(), &runtime);
    let second = run_vision_tool_with_runtime(request, &runtime);

    assert_eq!(first.output_hash, second.output_hash);
    assert_eq!(first.outputs, second.outputs);

    let ocr_output = first
        .outputs
        .iter()
        .find_map(|output| match output {
            crate::web_search_plan::vision::VisionOutput::OCRResult(ocr) => Some(ocr),
            _ => None,
        })
        .expect("ocr output should exist");

    assert_eq!(ocr_output.text_blocks.len(), 2);
    assert_eq!(ocr_output.text_blocks[0].text, "alpha");
    assert_eq!(ocr_output.text_blocks[1].text, "beta");
}

#[test]
fn test_t2_deterministic_ordering_of_objects() {
    let bytes = b"image-bytes-objects";
    let request = base_request(VisionMode::ImageObjects, "image/jpeg", bytes);

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "image/jpeg")),
        noop_ocr(),
        Ok(ObjectDetectionResult {
            frame_index: None,
            timestamp_ms: None,
            model_id: "model-a".to_string(),
            objects: vec![
                DetectedObject {
                    label: "car".to_string(),
                    bbox: BoundingBox {
                        x: 20.0,
                        y: 12.0,
                        w: 5.0,
                        h: 5.0,
                    },
                    confidence: 0.6,
                },
                DetectedObject {
                    label: "cat".to_string(),
                    bbox: BoundingBox {
                        x: 5.0,
                        y: 4.0,
                        w: 5.0,
                        h: 5.0,
                    },
                    confidence: 0.95,
                },
                DetectedObject {
                    label: "dog".to_string(),
                    bbox: BoundingBox {
                        x: 0.0,
                        y: 0.0,
                        w: 5.0,
                        h: 5.0,
                    },
                    confidence: 0.1,
                },
            ],
        }),
        noop_stt(),
        noop_keyframes(),
        Ok(Vec::new()),
    );

    let packet = run_vision_tool_with_runtime(request, &runtime);
    let output = packet
        .outputs
        .iter()
        .find_map(|item| match item {
            crate::web_search_plan::vision::VisionOutput::ObjectDetectionResult(objects) => {
                Some(objects)
            }
            _ => None,
        })
        .expect("objects output should exist");

    assert_eq!(output.objects.len(), 2);
    assert_eq!(output.objects[0].label, "cat");
    assert_eq!(output.objects[1].label, "car");
}

#[test]
fn test_t3_deterministic_transcript_segment_ordering() {
    let bytes = b"video-bytes";
    let request = base_request(VisionMode::VideoTranscribe, "video/mp4", bytes);

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "video/mp4")),
        noop_ocr(),
        noop_objects(),
        Ok(VideoTranscriptResult {
            stt_provider_id: "GOOGLE_STT".to_string(),
            language: "en-US".to_string(),
            segments: vec![
                TranscriptSegment {
                    start_ms: 2000,
                    end_ms: 3000,
                    text: "second".to_string(),
                    confidence: 70,
                },
                TranscriptSegment {
                    start_ms: 0,
                    end_ms: 1000,
                    text: "first".to_string(),
                    confidence: 90,
                },
                TranscriptSegment {
                    start_ms: 4000,
                    end_ms: 5000,
                    text: "drop".to_string(),
                    confidence: 10,
                },
            ],
            full_transcript: "second\nfirst\ndrop".to_string(),
        }),
        noop_keyframes(),
        Ok(vec![1, 2, 3]),
    );

    let packet = run_vision_tool_with_runtime(request, &runtime);
    let transcript = packet
        .outputs
        .iter()
        .find_map(|item| match item {
            crate::web_search_plan::vision::VisionOutput::VideoTranscriptResult(result) => {
                Some(result)
            }
            _ => None,
        })
        .expect("transcript output should exist");

    assert_eq!(transcript.segments.len(), 2);
    assert_eq!(transcript.segments[0].text, "first");
    assert_eq!(transcript.segments[1].text, "second");
}

#[test]
fn test_t4_deterministic_frame_selection_and_hash_stability() {
    let bytes = b"video-bytes-keyframes";
    let request = base_request(VisionMode::VideoKeyframes, "video/mp4", bytes);

    let keyframe_result = KeyframeIndexResult {
        keyframes: vec![
            KeyframeEntry {
                timestamp_ms: 2000,
                frame_index: 2,
                frame_hash: "h2".to_string(),
            },
            KeyframeEntry {
                timestamp_ms: 0,
                frame_index: 0,
                frame_hash: "h0".to_string(),
            },
            KeyframeEntry {
                timestamp_ms: 1000,
                frame_index: 1,
                frame_hash: "h1".to_string(),
            },
        ],
    };

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "video/mp4")),
        noop_ocr(),
        noop_objects(),
        noop_stt(),
        Ok(keyframe_result),
        Ok(vec![1, 2, 3]),
    );

    let first = run_vision_tool_with_runtime(request.clone(), &runtime);
    let second = run_vision_tool_with_runtime(request, &runtime);

    assert_eq!(first.output_hash, second.output_hash);
    assert_eq!(first.outputs, second.outputs);

    let keyframes = first
        .outputs
        .iter()
        .find_map(|item| match item {
            crate::web_search_plan::vision::VisionOutput::KeyframeIndexResult(result) => {
                Some(result)
            }
            _ => None,
        })
        .expect("keyframes output should exist");

    assert_eq!(keyframes.keyframes[0].timestamp_ms, 0);
    assert_eq!(keyframes.keyframes[1].timestamp_ms, 1000);
    assert_eq!(keyframes.keyframes[2].timestamp_ms, 2000);
}

#[test]
fn test_t5_threshold_enforcement_omits_low_confidence_deterministically() {
    let bytes = b"image-low-confidence";
    let request = base_request(VisionMode::ImageAnalyze, "image/png", bytes);

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "image/png")),
        Ok(OcrResult {
            page_or_frame_index: 0,
            timestamp_ms: None,
            ocr_engine_id: "ocr".to_string(),
            language: "en-US".to_string(),
            text_blocks: vec![OcrTextBlock {
                bbox: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                text: "too-low".to_string(),
                confidence: 0.2,
                pii_suspected: Some(false),
            }],
            full_text: "too-low".to_string(),
        }),
        Ok(ObjectDetectionResult {
            frame_index: None,
            timestamp_ms: None,
            model_id: "model".to_string(),
            objects: vec![DetectedObject {
                label: "low".to_string(),
                bbox: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                confidence: 0.1,
            }],
        }),
        noop_stt(),
        noop_keyframes(),
        Ok(Vec::new()),
    );

    let packet = run_vision_tool_with_runtime(request, &runtime);
    assert!(packet
        .reason_codes
        .iter()
        .any(|code| code == "insufficient_evidence"));

    let ocr = packet
        .outputs
        .iter()
        .find_map(|output| match output {
            crate::web_search_plan::vision::VisionOutput::OCRResult(value) => Some(value),
            _ => None,
        })
        .expect("ocr output exists");
    assert!(ocr.text_blocks.is_empty());

    let objects = packet
        .outputs
        .iter()
        .find_map(|output| match output {
            crate::web_search_plan::vision::VisionOutput::ObjectDetectionResult(value) => {
                Some(value)
            }
            _ => None,
        })
        .expect("object output exists");
    assert!(objects.objects.is_empty());
}

#[test]
fn test_t6_fail_closed_when_provider_missing() {
    let bytes = b"image-fail-closed";
    let request = base_request(VisionMode::ImageOcr, "image/png", bytes);

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "image/png")),
        Err(provider_error(
            "vision_ocr",
            "ocr",
            VisionProviderErrorKind::ProviderUnconfigured,
            VisionReasonCode::ProviderUnconfigured,
            "missing configuration",
        )),
        noop_objects(),
        noop_stt(),
        noop_keyframes(),
        Ok(Vec::new()),
    );

    let packet = run_vision_tool_with_runtime(request, &runtime);
    assert!(packet
        .reason_codes
        .iter()
        .any(|code| code == "provider_unconfigured"));
    assert!(packet
        .provider_runs
        .iter()
        .any(|run| run.endpoint == "ocr" && run.error.is_some()));
}

#[test]
fn test_t7_redaction_no_secrets_in_debug_strings() {
    let redacted = redaction::redact_locator("https://user:pass@example.com/path?q=token123#frag");
    assert_eq!(redacted, "https://example.com/path");

    let debug = redaction::debug_string_without_secrets(&[
        "Authorization: Bearer sk-secret",
        "api_key=my-secret",
        "/Users/xiamo/private/file.png",
    ]);
    assert!(!debug.contains("sk-secret"));
    assert!(!debug.contains("api_key"));
}

#[test]
fn test_t8_vision_evidence_packet_hash_stable_with_fixed_clock() {
    let bytes = b"image-stable";
    let request = base_request(VisionMode::ImageOcr, "image/png", bytes);

    let runtime = runtime_with_mocks(
        Ok(loaded_asset(bytes, "image/png")),
        Ok(OcrResult {
            page_or_frame_index: 0,
            timestamp_ms: None,
            ocr_engine_id: "ocr".to_string(),
            language: "en-US".to_string(),
            text_blocks: vec![OcrTextBlock {
                bbox: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                text: "stable".to_string(),
                confidence: 0.99,
                pii_suspected: Some(false),
            }],
            full_text: "stable".to_string(),
        }),
        noop_objects(),
        noop_stt(),
        noop_keyframes(),
        Ok(Vec::new()),
    );

    let first: VisionEvidencePacket = run_vision_tool_with_runtime(request.clone(), &runtime);
    let second: VisionEvidencePacket = run_vision_tool_with_runtime(request, &runtime);

    assert_eq!(first.retrieved_at_ms, 1_778_000_000_123);
    assert_eq!(first.output_hash, second.output_hash);
    assert_eq!(first.packet_hashes, second.packet_hashes);
}

#[test]
fn test_vision_packet_fixtures_validate() {
    let packet_registry = load_packet_schema_registry().expect("packet schema registry must load");
    validate_packet_schema_registry(&packet_registry).expect("packet schema must validate");

    for name in ["vision_tool_request.json", "vision_evidence.json"] {
        let path = fixtures_dir("valid").join(name);
        let text = fs::read_to_string(&path).expect("valid vision fixture should be readable");
        let value: Value = serde_json::from_str(&text).expect("valid vision fixture should parse");

        let packet_name = packet_name_from_fixture_filename(name)
            .expect("vision fixture should map to packet name");
        validate_packet(packet_name, &value, &packet_registry)
            .unwrap_or_else(|err| panic!("valid fixture {} failed: {}", name, err));
    }

    for name in [
        "vision_tool_request_missing_asset_ref.json",
        "vision_evidence_missing_outputs.json",
    ] {
        let path = fixtures_dir("invalid").join(name);
        let text = fs::read_to_string(&path).expect("invalid vision fixture should be readable");
        let value: Value =
            serde_json::from_str(&text).expect("invalid vision fixture should parse");

        let packet_name = packet_name_from_fixture_filename(name)
            .expect("vision fixture should map to packet name");
        let result = validate_packet(packet_name, &value, &packet_registry);
        assert!(
            result.is_err(),
            "invalid fixture {} unexpectedly passed",
            name
        );
    }
}
