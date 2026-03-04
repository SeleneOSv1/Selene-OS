#![forbid(unsafe_code)]

use selene_os::web_search_plan::contract_hash::sha256_hex;
use selene_os::web_search_plan::vision::packet_builder::{
    BoundingBox, DetectedObject, KeyframeEntry, KeyframeIndexResult, ObjectDetectionResult,
    OcrResult, OcrTextBlock, TranscriptSegment, VideoTranscriptResult,
};
use selene_os::web_search_plan::vision::{
    run_vision_tool, run_vision_tool_with_providers, VisionError, VisionProviderSet,
    VisionRuntimeConfig, VisionToolRequest,
};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const FIXTURE_NOW_MS: i64 = 1_762_463_000_000;

#[derive(Debug, Clone)]
struct CliArgs {
    fixture_mode: bool,
    mode: String,
    asset: String,
}

#[derive(Debug, Clone)]
struct FixtureAsset {
    relative_path: String,
    mime_type: String,
}

#[derive(Debug, Clone)]
struct FixtureProviders {
    ocr: OcrResult,
    objects: ObjectDetectionResult,
    transcript: VideoTranscriptResult,
    keyframes: KeyframeIndexResult,
}

impl FixtureProviders {
    fn load() -> Result<Self, String> {
        Ok(Self {
            ocr: parse_ocr(expected_path("ocr_expected.json")?)?,
            objects: parse_objects(expected_path("objects_expected.json")?)?,
            transcript: parse_transcript(expected_path("transcript_expected.json")?)?,
            keyframes: parse_keyframes(expected_path("keyframes_expected.json")?)?,
        })
    }
}

impl VisionProviderSet for FixtureProviders {
    fn run_ocr(
        &self,
        _asset: &selene_os::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<OcrResult, VisionError> {
        Ok(self.ocr.clone())
    }

    fn run_objects(
        &self,
        _asset: &selene_os::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<ObjectDetectionResult, VisionError> {
        Ok(self.objects.clone())
    }

    fn run_video_stt(
        &self,
        _asset: &selene_os::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<VideoTranscriptResult, VisionError> {
        Ok(self.transcript.clone())
    }

    fn run_keyframes(
        &self,
        _asset: &selene_os::web_search_plan::vision::download::LoadedAsset,
        _request: &VisionToolRequest,
        _config: &VisionRuntimeConfig,
    ) -> Result<KeyframeIndexResult, VisionError> {
        Ok(self.keyframes.clone())
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{}", message);
            eprintln!(
                "usage: web_search_vision_turn [--fixture] --mode <image_ocr|image_objects|video_transcribe|video_keyframes> --asset <fixture_key_or_path>"
            );
            return ExitCode::from(64);
        }
    };

    let packet = match build_request_packet(&args) {
        Ok(packet) => packet,
        Err(message) => {
            eprintln!("request_build_error={}", message);
            return ExitCode::from(64);
        }
    };

    let result = if args.fixture_mode {
        let providers = match FixtureProviders::load() {
            Ok(providers) => providers,
            Err(message) => {
                eprintln!("fixture_provider_error={}", message);
                return ExitCode::from(64);
            }
        };
        run_vision_tool_with_providers(
            &packet,
            FIXTURE_NOW_MS,
            &VisionRuntimeConfig::default(),
            &providers,
        )
    } else {
        run_vision_tool(&packet, FIXTURE_NOW_MS, &VisionRuntimeConfig::default())
    };

    match result {
        Ok(value) => {
            let (block_count, object_count, segment_count, keyframe_count) = count_outputs(&value);
            let output_hash = value
                .get("output_hash")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            println!("mode={}", args.mode);
            println!("ocr_blocks={}", block_count);
            println!("objects={}", object_count);
            println!("segments={}", segment_count);
            println!("keyframes={}", keyframe_count);
            println!("output_hash={}", output_hash);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("FAIL_CLOSED_REASON={}", error.reason_code());
            eprintln!("FAIL_CLOSED_MESSAGE={}", error.message);
            ExitCode::from(2)
        }
    }
}

fn parse_args() -> Result<CliArgs, String> {
    let mut fixture_mode = false;
    let mut mode: Option<String> = None;
    let mut asset: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixture" => fixture_mode = true,
            "--mode" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--mode requires a value".to_string())?;
                let normalized = value.trim().to_ascii_lowercase();
                if !matches!(
                    normalized.as_str(),
                    "image_ocr" | "image_objects" | "video_transcribe" | "video_keyframes"
                ) {
                    return Err(format!("unsupported --mode {}", value));
                }
                mode = Some(normalized);
            }
            "--asset" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--asset requires a value".to_string())?;
                if value.trim().is_empty() {
                    return Err("--asset cannot be empty".to_string());
                }
                asset = Some(value);
            }
            other => return Err(format!("unsupported argument {}", other)),
        }
    }

    Ok(CliArgs {
        fixture_mode,
        mode: mode.ok_or_else(|| "--mode is required".to_string())?,
        asset: asset.ok_or_else(|| "--asset is required".to_string())?,
    })
}

fn build_request_packet(args: &CliArgs) -> Result<Value, String> {
    let fixture_asset = if args.fixture_mode {
        fixture_asset_from_key(args.asset.as_str(), args.mode.as_str())?
    } else {
        FixtureAsset {
            relative_path: args.asset.clone(),
            mime_type: mime_from_path(args.asset.as_str())?,
        }
    };

    let asset_path = if args.fixture_mode {
        fixture_root().join(fixture_asset.relative_path.as_str())
    } else {
        PathBuf::from(fixture_asset.relative_path.as_str())
    };

    let bytes = fs::read(&asset_path)
        .map_err(|error| format!("failed to read asset {}: {}", asset_path.display(), error))?;
    let asset_hash = sha256_hex(bytes.as_slice());

    Ok(json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E", "PH1.D", "PH1.J"],
        "created_at_ms": FIXTURE_NOW_MS,
        "trace_id": format!("trace_vision_cli_{}", args.mode),
        "mode": args.mode,
        "asset_ref": {
            "asset_hash": asset_hash,
            "locator": asset_path.to_string_lossy(),
            "mime_type": fixture_asset.mime_type,
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
    }))
}

fn fixture_asset_from_key(key: &str, mode: &str) -> Result<FixtureAsset, String> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "text_heavy" => Ok(FixtureAsset {
            relative_path: "images/text_heavy.png".to_string(),
            mime_type: "image/png".to_string(),
        }),
        "object_heavy" => Ok(FixtureAsset {
            relative_path: "images/object_heavy.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
        }),
        "blank" => Ok(FixtureAsset {
            relative_path: "images/blank.png".to_string(),
            mime_type: "image/png".to_string(),
        }),
        "clear_speech" => Ok(FixtureAsset {
            relative_path: "videos/clear_speech.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
        }),
        "noisy_speech" => Ok(FixtureAsset {
            relative_path: "videos/noisy_speech.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
        }),
        "silent" => Ok(FixtureAsset {
            relative_path: "videos/silent.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
        }),
        _ => {
            if mode.starts_with("image_") {
                Err(format!(
                    "unsupported fixture --asset {} for {} (expected text_heavy|object_heavy|blank)",
                    key, mode
                ))
            } else {
                Err(format!(
                    "unsupported fixture --asset {} for {} (expected clear_speech|noisy_speech|silent)",
                    key, mode
                ))
            }
        }
    }
}

fn mime_from_path(path: &str) -> Result<String, String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "--asset path must include a file extension".to_string())?;

    match extension.as_str() {
        "png" => Ok("image/png".to_string()),
        "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
        "webp" => Ok("image/webp".to_string()),
        "mp4" => Ok("video/mp4".to_string()),
        "mov" => Ok("video/quicktime".to_string()),
        _ => Err(format!("unsupported asset extension {}", extension)),
    }
}

fn count_outputs(packet: &Value) -> (usize, usize, usize, usize) {
    let mut blocks = 0usize;
    let mut objects = 0usize;
    let mut segments = 0usize;
    let mut keyframes = 0usize;

    if let Some(outputs) = packet.get("outputs").and_then(Value::as_array) {
        for output in outputs {
            blocks += output
                .get("text_blocks")
                .and_then(Value::as_array)
                .map(|entries| entries.len())
                .unwrap_or(0);
            objects += output
                .get("objects")
                .and_then(Value::as_array)
                .map(|entries| entries.len())
                .unwrap_or(0);
            segments += output
                .get("segments")
                .and_then(Value::as_array)
                .map(|entries| entries.len())
                .unwrap_or(0);
            keyframes += output
                .get("keyframes")
                .and_then(Value::as_array)
                .map(|entries| entries.len())
                .unwrap_or(0);
        }
    }

    (blocks, objects, segments, keyframes)
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/vision_fixtures")
}

fn expected_path(name: &str) -> Result<PathBuf, String> {
    Ok(fixture_root().join("expected").join(name))
}

fn load_json(path: PathBuf) -> Result<Value, String> {
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to load fixture {}: {}", path.display(), error))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse fixture {}: {}", path.display(), error))
}

fn parse_ocr(path: PathBuf) -> Result<OcrResult, String> {
    let value = load_json(path)?;
    Ok(OcrResult {
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
    })
}

fn parse_objects(path: PathBuf) -> Result<ObjectDetectionResult, String> {
    let value = load_json(path)?;
    Ok(ObjectDetectionResult {
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
    })
}

fn parse_transcript(path: PathBuf) -> Result<VideoTranscriptResult, String> {
    let value = load_json(path)?;
    Ok(VideoTranscriptResult {
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
    })
}

fn parse_keyframes(path: PathBuf) -> Result<KeyframeIndexResult, String> {
    let value = load_json(path)?;
    Ok(KeyframeIndexResult {
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
    })
}
