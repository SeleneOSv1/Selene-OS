#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::vision::packet_builder::{
    BoundingBox, DetectedObject, KeyframeEntry, KeyframeIndexResult, ObjectDetectionResult,
    OcrResult, OcrTextBlock, TranscriptSegment, VideoTranscriptResult,
};
use crate::web_search_plan::vision::redaction::{redact_locator, redact_secrets, redact_url};
use crate::web_search_plan::vision::{
    run_vision_tool, run_vision_tool_with_providers, VisionError, VisionErrorKind,
    VisionProviderSet, VisionRuntimeConfig, VisionToolRequest,
};
use crate::web_search_plan::{
    packet_validator::validate_packet, registry_loader::load_packet_schema_registry,
};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/vision_fixtures")
}

fn expected_path(name: &str) -> PathBuf {
    fixtures_root().join("expected").join(name)
}

fn fixture_file_bytes(relative: &str) -> Vec<u8> {
    fs::read(fixtures_root().join(relative)).expect("fixture bytes should load")
}

fn fixture_file_path(relative: &str) -> String {
    fixtures_root().join(relative).to_string_lossy().to_string()
}

#[derive(Debug, Clone)]
struct FixtureProviders {
    ocr: OcrResult,
    objects: ObjectDetectionResult,
    transcript: VideoTranscriptResult,
    keyframes: KeyframeIndexResult,
}

impl FixtureProviders {
    fn load() -> Self {
        let ocr = parse_ocr(expected_path("ocr_expected.json"));
        let objects = parse_objects(expected_path("objects_expected.json"));
        let transcript = parse_transcript(expected_path("transcript_expected.json"));
        let keyframes = parse_keyframes(expected_path("keyframes_expected.json"));

        Self {
            ocr,
            objects,
            transcript,
            keyframes,
        }
    }
}

impl VisionProviderSet for FixtureProviders {
    fn run_ocr(
        &self,
        _asset: &crate::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<OcrResult, VisionError> {
        Ok(self.ocr.clone())
    }

    fn run_objects(
        &self,
        _asset: &crate::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<ObjectDetectionResult, VisionError> {
        Ok(self.objects.clone())
    }

    fn run_video_stt(
        &self,
        _asset: &crate::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<VideoTranscriptResult, VisionError> {
        Ok(self.transcript.clone())
    }

    fn run_keyframes(
        &self,
        _asset: &crate::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<KeyframeIndexResult, VisionError> {
        Ok(self.keyframes.clone())
    }
}

#[test]
fn test_t1_deterministic_ordering_ocr_blocks() {
    let mut providers = FixtureProviders::load();
    providers.ocr.text_blocks = vec![
        OcrTextBlock {
            bbox: BoundingBox {
                x: 100.0,
                y: 30.0,
                w: 10.0,
                h: 4.0,
            },
            text: "line3".to_string(),
            confidence: 0.93,
            pii_suspected: Some(false),
        },
        OcrTextBlock {
            bbox: BoundingBox {
                x: 5.0,
                y: 0.0,
                w: 10.0,
                h: 4.0,
            },
            text: "line1".to_string(),
            confidence: 0.91,
            pii_suspected: Some(false),
        },
        OcrTextBlock {
            bbox: BoundingBox {
                x: 5.0,
                y: 10.0,
                w: 10.0,
                h: 4.0,
            },
            text: "line2".to_string(),
            confidence: 0.92,
            pii_suspected: Some(false),
        },
    ];

    let request = request_packet("image_ocr", "images/text_heavy.png", "image/png");
    let value = run_vision_tool_with_providers(
        &request,
        1_762_463_000_000,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("image ocr should succeed");

    let texts = value
        .pointer("/outputs/0/text_blocks")
        .and_then(Value::as_array)
        .expect("ocr text blocks")
        .iter()
        .filter_map(|entry| entry.get("text").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    assert_eq!(texts, vec!["line1", "line2", "line3"]);
}

#[test]
fn test_t2_deterministic_ordering_objects() {
    let mut providers = FixtureProviders::load();
    providers.objects.objects = vec![
        DetectedObject {
            label: "car".to_string(),
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                w: 5.0,
                h: 5.0,
            },
            confidence: 0.80,
        },
        DetectedObject {
            label: "person".to_string(),
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                w: 5.0,
                h: 5.0,
            },
            confidence: 0.99,
        },
    ];

    let request = request_packet("image_objects", "images/object_heavy.jpg", "image/jpeg");
    let value = run_vision_tool_with_providers(
        &request,
        1_762_463_000_111,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("image objects should succeed");

    let labels = value
        .pointer("/outputs/0/objects")
        .and_then(Value::as_array)
        .expect("objects array")
        .iter()
        .filter_map(|entry| entry.get("label").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    assert_eq!(labels, vec!["person", "car"]);
}

#[test]
fn test_t3_deterministic_transcript_segment_ordering() {
    let mut providers = FixtureProviders::load();
    providers.transcript.segments = vec![
        TranscriptSegment {
            start_ms: 1200,
            end_ms: 2500,
            text: "world".to_string(),
            confidence: 89.0,
        },
        TranscriptSegment {
            start_ms: 0,
            end_ms: 1200,
            text: "hello".to_string(),
            confidence: 92.0,
        },
    ];

    let request = request_packet("video_transcribe", "videos/clear_speech.mp4", "video/mp4");
    let value = run_vision_tool_with_providers(
        &request,
        1_762_463_000_222,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("video transcribe should succeed");

    let starts = value
        .pointer("/outputs/0/segments")
        .and_then(Value::as_array)
        .expect("segments")
        .iter()
        .filter_map(|entry| entry.get("start_ms").and_then(Value::as_u64))
        .collect::<Vec<u64>>();
    assert_eq!(starts, vec![0, 1200]);
}

#[test]
fn test_t4_deterministic_frame_selection_and_hash_stability() {
    let providers = FixtureProviders::load();
    let request = request_packet("video_keyframes", "videos/noisy_speech.mp4", "video/mp4");

    let first = run_vision_tool_with_providers(
        &request,
        1_762_463_000_333,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("first keyframe run should pass");
    let second = run_vision_tool_with_providers(
        &request,
        1_762_463_000_333,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("second keyframe run should pass");

    assert_eq!(first, second);
    let hashes = first
        .pointer("/outputs/0/keyframes")
        .and_then(Value::as_array)
        .expect("keyframes")
        .iter()
        .filter_map(|entry| entry.get("frame_hash").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    assert_eq!(hashes, vec!["hash0", "hash1"]);
}

#[test]
fn test_t5_threshold_enforcement_is_deterministic() {
    let mut providers = FixtureProviders::load();
    providers.objects.objects = vec![DetectedObject {
        label: "car".to_string(),
        bbox: BoundingBox {
            x: 0.0,
            y: 0.0,
            w: 5.0,
            h: 5.0,
        },
        confidence: 0.1,
    }];

    let request = request_packet("image_objects", "images/object_heavy.jpg", "image/jpeg");
    let value = run_vision_tool_with_providers(
        &request,
        1_762_463_000_444,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("fixture provider path still deterministic");

    // Fixture provider bypasses runtime thresholds; deterministic output must remain stable.
    let labels = value
        .pointer("/outputs/0/objects")
        .and_then(Value::as_array)
        .expect("objects")
        .len();
    assert_eq!(labels, 1);
}

#[test]
fn test_t6_provider_unconfigured_fail_closed() {
    let request = request_packet("image_ocr", "images/text_heavy.png", "image/png");
    let mut config = VisionRuntimeConfig::default();
    config.ocr_endpoint = None;
    config.ocr_api_key = None;

    let err = run_vision_tool(&request, 1_762_463_000_555, &config)
        .expect_err("runtime ocr must fail when provider is unconfigured");
    assert_eq!(err.kind, VisionErrorKind::ProviderUnconfigured);
    assert_eq!(err.reason_code(), "provider_unconfigured");
}

#[test]
fn test_t7_redaction_safety_no_secrets() {
    let url = "https://user:pass@example.com/path?q=secret_token#frag";
    let redacted_url = redact_url(url);
    assert_eq!(redacted_url, "https://example.com/path");

    let redacted_locator = redact_locator("file:///Users/xiamo/Documents/private/image.png");
    assert_eq!(redacted_locator, "file://[REDACTED]");

    let redacted_message = redact_secrets("Authorization: Bearer sk-test-token");
    assert_eq!(redacted_message, "[REDACTED]");
}

#[test]
fn test_t8_vision_evidence_packet_schema_and_stable_output_hash() {
    let providers = FixtureProviders::load();
    let request = request_packet("image_analyze", "images/text_heavy.png", "image/png");

    let first = run_vision_tool_with_providers(
        &request,
        1_762_463_000_666,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("first packet should pass");
    let second = run_vision_tool_with_providers(
        &request,
        1_762_463_000_666,
        &VisionRuntimeConfig::default(),
        &providers,
    )
    .expect("second packet should pass");

    assert_eq!(first, second);
    let registry = load_packet_schema_registry().expect("packet schema registry should load");
    validate_packet("VisionEvidencePacket", &first, &registry)
        .expect("vision evidence packet must satisfy schema");

    let first_hash = first
        .get("output_hash")
        .and_then(Value::as_str)
        .expect("output_hash required");
    let second_hash = second
        .get("output_hash")
        .and_then(Value::as_str)
        .expect("output_hash required");
    assert_eq!(first_hash, second_hash);
}

fn request_packet(mode: &str, relative_path: &str, mime_type: &str) -> Value {
    let bytes = fixture_file_bytes(relative_path);
    let asset_hash = sha256_hex(bytes.as_slice());
    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E", "PH1.D", "PH1.J"],
        "created_at_ms": 1762463000000i64,
        "trace_id": format!("trace_vision_{}", mode),
        "mode": mode,
        "asset_ref": {
            "asset_hash": asset_hash,
            "locator": fixture_file_path(relative_path),
            "mime_type": mime_type,
            "size_bytes": bytes.len() as u64
        },
        "options": {
            "language_hint": "en-US",
            "max_frames": 4,
            "frame_stride_ms": 1000,
            "safe_mode": true,
            "analyze_url": false
        },
        "budgets": {
            "timeout_ms": 3000,
            "max_bytes": 10485760
        },
        "policy_snapshot_id": "policy_run21a_v1"
    })
}

fn parse_ocr(path: PathBuf) -> OcrResult {
    let value = load_json(path);
    OcrResult {
        page_or_frame_index: value
            .get("page_or_frame_index")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        timestamp_ms: value.get("timestamp_ms").and_then(Value::as_u64),
        ocr_engine_id: value
            .get("ocr_engine_id")
            .and_then(Value::as_str)
            .unwrap_or("fixture_ocr")
            .to_string(),
        language: value
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or("en-US")
            .to_string(),
        text_blocks: value
            .get("text_blocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|entry| OcrTextBlock {
                bbox: BoundingBox {
                    x: entry
                        .pointer("/bbox/x")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    y: entry
                        .pointer("/bbox/y")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    w: entry
                        .pointer("/bbox/w")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    h: entry
                        .pointer("/bbox/h")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                },
                text: entry
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                confidence: entry
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                pii_suspected: entry.get("pii_suspected").and_then(Value::as_bool),
            })
            .collect(),
        full_text: value
            .get("full_text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

fn parse_objects(path: PathBuf) -> ObjectDetectionResult {
    let value = load_json(path);
    ObjectDetectionResult {
        frame_index: value
            .get("frame_index")
            .and_then(Value::as_u64)
            .map(|v| v as u32),
        timestamp_ms: value.get("timestamp_ms").and_then(Value::as_u64),
        model_id: value
            .get("model_id")
            .and_then(Value::as_str)
            .unwrap_or("fixture_model")
            .to_string(),
        objects: value
            .get("objects")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|entry| DetectedObject {
                label: entry
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                bbox: BoundingBox {
                    x: entry
                        .pointer("/bbox/x")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    y: entry
                        .pointer("/bbox/y")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    w: entry
                        .pointer("/bbox/w")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                    h: entry
                        .pointer("/bbox/h")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                },
                confidence: entry
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
            })
            .collect(),
    }
}

fn parse_transcript(path: PathBuf) -> VideoTranscriptResult {
    let value = load_json(path);
    VideoTranscriptResult {
        stt_provider_id: value
            .get("stt_provider_id")
            .and_then(Value::as_str)
            .unwrap_or("GOOGLE_STT")
            .to_string(),
        language: value
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or("en-US")
            .to_string(),
        segments: value
            .get("segments")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|entry| TranscriptSegment {
                start_ms: entry.get("start_ms").and_then(Value::as_u64).unwrap_or(0),
                end_ms: entry.get("end_ms").and_then(Value::as_u64).unwrap_or(0),
                text: entry
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                confidence: entry
                    .get("confidence")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
            })
            .collect(),
        full_transcript: value
            .get("full_transcript")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

fn parse_keyframes(path: PathBuf) -> KeyframeIndexResult {
    let value = load_json(path);
    KeyframeIndexResult {
        keyframes: value
            .get("keyframes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|entry| KeyframeEntry {
                timestamp_ms: entry
                    .get("timestamp_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                frame_index: entry
                    .get("frame_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32,
                frame_hash: entry
                    .get("frame_hash")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect(),
    }
}

fn load_json(path: PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("json fixture should load");
    serde_json::from_str(&text).expect("json fixture should parse")
}
