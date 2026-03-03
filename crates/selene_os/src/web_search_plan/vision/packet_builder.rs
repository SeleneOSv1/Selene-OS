#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::vision::{
    ConfidenceSummary, VisionEvidencePacket, VisionOutput, VisionProviderRun,
    VisionToolRequestPacket,
};
use std::collections::BTreeSet;

pub fn build_vision_evidence_packet(
    request: &VisionToolRequestPacket,
    retrieved_at_ms: i64,
    provider_runs: Vec<VisionProviderRun>,
    outputs: Vec<VisionOutput>,
    reason_codes: BTreeSet<String>,
) -> VisionEvidencePacket {
    let outputs_json = serde_json::to_vec(&outputs).unwrap_or_default();
    let output_hash = sha256_hex(&outputs_json);

    let provider_runs_json = serde_json::to_vec(&provider_runs).unwrap_or_default();
    let provider_runs_hash = sha256_hex(&provider_runs_json);

    let confidence_summary = build_confidence_summary(&outputs);

    VisionEvidencePacket {
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
        reason_codes: reason_codes.into_iter().collect(),
        packet_hashes: serde_json::json!({
            "asset_hash": request.asset_ref.asset_hash,
            "provider_runs_hash": provider_runs_hash,
            "outputs_hash": output_hash,
        }),
        output_hash,
    }
}

pub fn build_confidence_summary(outputs: &[VisionOutput]) -> ConfidenceSummary {
    let mut score_total = 0.0f64;
    let mut score_count = 0u64;
    let mut ocr_blocks = 0u64;
    let mut object_count = 0u64;
    let mut transcript_segments = 0u64;

    for output in outputs {
        match output {
            VisionOutput::OCRResult(result) => {
                for block in &result.text_blocks {
                    score_total += block.confidence;
                    score_count += 1;
                    ocr_blocks += 1;
                }
            }
            VisionOutput::ObjectDetectionResult(result) => {
                for object in &result.objects {
                    score_total += object.confidence;
                    score_count += 1;
                    object_count += 1;
                }
            }
            VisionOutput::VideoTranscriptResult(result) => {
                for segment in &result.segments {
                    score_total += segment.confidence as f64 / 100.0;
                    score_count += 1;
                    transcript_segments += 1;
                }
            }
            VisionOutput::KeyframeIndexResult(result) => {
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

    ConfidenceSummary {
        mean_confidence,
        ocr_blocks_retained: ocr_blocks,
        objects_retained: object_count,
        transcript_segments_retained: transcript_segments,
        output_count: outputs.len() as u64,
    }
}
