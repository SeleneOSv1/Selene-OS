#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::replay::snapshot::{canonicalize_value, hash_canonical_json};
use crate::web_search_plan::vision::{
    VisionAssetRef, VisionError, VisionErrorKind, VisionToolRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pii_suspected: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    pub page_or_frame_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_ms: Option<u64>,
    pub ocr_engine_id: String,
    pub language: String,
    pub text_blocks: Vec<OcrTextBlock>,
    pub full_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedObject {
    pub label: String,
    pub bbox: BoundingBox,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectDetectionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_ms: Option<u64>,
    pub model_id: String,
    pub objects: Vec<DetectedObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoTranscriptResult {
    pub stt_provider_id: String,
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
    pub full_transcript: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeEntry {
    pub timestamp_ms: u64,
    pub frame_index: u32,
    pub frame_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeIndexResult {
    pub keyframes: Vec<KeyframeEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "output_type")]
pub enum VisionOutput {
    #[serde(rename = "OCRResult")]
    OcrResult(OcrResult),
    #[serde(rename = "ObjectDetectionResult")]
    ObjectDetectionResult(ObjectDetectionResult),
    #[serde(rename = "VideoTranscriptResult")]
    VideoTranscriptResult(VideoTranscriptResult),
    #[serde(rename = "KeyframeIndexResult")]
    KeyframeIndexResult(KeyframeIndexResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionProviderRun {
    pub provider_id: String,
    pub endpoint: String,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceSummary {
    pub mean_confidence: f64,
    pub confident_item_count: u32,
    pub total_item_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHashes {
    pub asset_hash: String,
    pub provider_runs_hash: String,
    pub outputs_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisionEvidencePacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub asset_ref: VisionAssetRef,
    pub retrieved_at_ms: i64,
    pub provider_runs: Vec<VisionProviderRun>,
    pub outputs: Vec<VisionOutput>,
    pub confidence_summary: ConfidenceSummary,
    pub reason_codes: Vec<String>,
    pub packet_hashes: PacketHashes,
    pub output_hash: String,
}

pub fn sort_ocr_blocks(blocks: &mut [OcrTextBlock]) {
    blocks.sort_by(|left, right| {
        (
            ordered_f64(left.bbox.y),
            ordered_f64(left.bbox.x),
            rev_f64(left.confidence),
        )
            .cmp(&(
                ordered_f64(right.bbox.y),
                ordered_f64(right.bbox.x),
                rev_f64(right.confidence),
            ))
    });
}

pub fn sort_detected_objects(objects: &mut [DetectedObject]) {
    objects.sort_by(|left, right| {
        (
            rev_f64(left.confidence),
            ordered_f64(left.bbox.y),
            ordered_f64(left.bbox.x),
        )
            .cmp(&(
                rev_f64(right.confidence),
                ordered_f64(right.bbox.y),
                ordered_f64(right.bbox.x),
            ))
    });
}

pub fn sort_transcript_segments(segments: &mut [TranscriptSegment]) {
    segments.sort_by(|left, right| {
        (left.start_ms, left.end_ms, left.text.as_str()).cmp(&(
            right.start_ms,
            right.end_ms,
            right.text.as_str(),
        ))
    });
}

pub fn sort_keyframes(keyframes: &mut [KeyframeEntry]) {
    keyframes.sort_by(|left, right| {
        (
            left.timestamp_ms,
            left.frame_index,
            left.frame_hash.as_str(),
        )
            .cmp(&(
                right.timestamp_ms,
                right.frame_index,
                right.frame_hash.as_str(),
            ))
    });
}

pub fn sort_outputs(outputs: &mut [VisionOutput]) {
    outputs.sort_by(|left, right| output_order_key(left).cmp(&output_order_key(right)));
}

fn output_order_key(output: &VisionOutput) -> (u8, u64, u32) {
    match output {
        VisionOutput::OcrResult(result) => (
            0,
            result
                .timestamp_ms
                .unwrap_or(result.page_or_frame_index as u64),
            result.page_or_frame_index,
        ),
        VisionOutput::ObjectDetectionResult(result) => (
            1,
            result
                .timestamp_ms
                .unwrap_or(result.frame_index.unwrap_or(0) as u64),
            result.frame_index.unwrap_or(0),
        ),
        VisionOutput::VideoTranscriptResult(result) => (
            2,
            result
                .segments
                .first()
                .map(|segment| segment.start_ms)
                .unwrap_or(0),
            0,
        ),
        VisionOutput::KeyframeIndexResult(result) => (
            3,
            result
                .keyframes
                .first()
                .map(|entry| entry.timestamp_ms)
                .unwrap_or(0),
            result
                .keyframes
                .first()
                .map(|entry| entry.frame_index)
                .unwrap_or(0),
        ),
    }
}

pub fn build_vision_evidence_packet(
    request: &VisionToolRequest,
    retrieved_at_ms: i64,
    mut provider_runs: Vec<VisionProviderRun>,
    mut outputs: Vec<VisionOutput>,
    reason_codes: &[String],
) -> Result<VisionEvidencePacket, VisionError> {
    provider_runs.sort_by(|left, right| {
        (
            left.endpoint.as_str(),
            left.provider_id.as_str(),
            left.latency_ms,
        )
            .cmp(&(
                right.endpoint.as_str(),
                right.provider_id.as_str(),
                right.latency_ms,
            ))
    });
    sort_outputs(&mut outputs);

    let outputs_value = serde_json::to_value(&outputs).map_err(|error| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("failed to serialize vision outputs: {}", error),
        )
    })?;
    let canonical_outputs = canonicalize_value(&outputs_value);
    let output_hash = hash_canonical_json(&canonical_outputs).map_err(|error| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("failed to hash vision outputs: {}", error),
        )
    })?;

    let provider_runs_value = serde_json::to_value(&provider_runs).map_err(|error| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("failed to serialize provider runs: {}", error),
        )
    })?;
    let provider_runs_hash = hash_canonical_json(&provider_runs_value).map_err(|error| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("failed to hash provider runs: {}", error),
        )
    })?;

    let confidence_summary = compute_confidence_summary(&outputs);

    let mut dedup = BTreeSet::new();
    for reason_code in reason_codes {
        dedup.insert(reason_code.clone());
    }

    Ok(VisionEvidencePacket {
        schema_version: "1.0.0".to_string(),
        produced_by: "PH1.E".to_string(),
        intended_consumers: request.intended_consumers.clone(),
        created_at_ms: request.created_at_ms,
        trace_id: request.trace_id.clone(),
        asset_ref: request.asset_ref.clone(),
        retrieved_at_ms,
        provider_runs,
        outputs,
        confidence_summary,
        reason_codes: dedup.into_iter().collect(),
        packet_hashes: PacketHashes {
            asset_hash: request.asset_ref.asset_hash.clone(),
            provider_runs_hash,
            outputs_hash: output_hash.clone(),
        },
        output_hash,
    })
}

fn compute_confidence_summary(outputs: &[VisionOutput]) -> ConfidenceSummary {
    let mut confidences = Vec::new();
    for output in outputs {
        match output {
            VisionOutput::OcrResult(result) => {
                for block in &result.text_blocks {
                    confidences.push(normalize_confidence(block.confidence));
                }
            }
            VisionOutput::ObjectDetectionResult(result) => {
                for object in &result.objects {
                    confidences.push(normalize_confidence(object.confidence));
                }
            }
            VisionOutput::VideoTranscriptResult(result) => {
                for segment in &result.segments {
                    confidences.push(normalize_confidence(segment.confidence));
                }
            }
            VisionOutput::KeyframeIndexResult(result) => {
                for _ in &result.keyframes {
                    confidences.push(1.0);
                }
            }
        }
    }

    if confidences.is_empty() {
        return ConfidenceSummary {
            mean_confidence: 0.0,
            confident_item_count: 0,
            total_item_count: 0,
        };
    }

    let total_item_count = confidences.len() as u32;
    let confident_item_count = confidences
        .iter()
        .filter(|confidence| **confidence >= 0.8)
        .count() as u32;
    let sum = confidences.iter().copied().sum::<f64>();

    ConfidenceSummary {
        mean_confidence: round4(sum / confidences.len() as f64),
        confident_item_count,
        total_item_count,
    }
}

fn normalize_confidence(confidence: f64) -> f64 {
    if confidence > 1.0 {
        (confidence / 100.0).clamp(0.0, 1.0)
    } else {
        confidence.clamp(0.0, 1.0)
    }
}

fn round4(value: f64) -> f64 {
    ((value * 10_000.0).round()) / 10_000.0
}

pub fn deterministic_join_lines(lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

pub fn to_json_value(packet: &VisionEvidencePacket) -> Result<Value, VisionError> {
    serde_json::to_value(packet).map_err(|error| {
        VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("failed to serialize vision evidence packet: {}", error),
        )
    })
}

pub fn redacted_error_payload(error: &str) -> Value {
    json!({ "message": error })
}

fn ordered_f64(value: f64) -> i64 {
    (value * 10_000.0).round() as i64
}

fn rev_f64(value: f64) -> i64 {
    -(value * 10_000.0).round() as i64
}
