#![forbid(unsafe_code)]

use crate::web_search_plan::vision::download::LoadedAsset;
use crate::web_search_plan::vision::packet_builder::{
    deterministic_join_lines, sort_ocr_blocks, BoundingBox, OcrResult, OcrTextBlock,
};
use crate::web_search_plan::vision::thresholds::meets_ocr_threshold;
use crate::web_search_plan::vision::{VisionError, VisionErrorKind, VisionToolRequest};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value;

pub fn run_ocr_runtime(
    asset: &LoadedAsset,
    request: &VisionToolRequest,
    ocr_endpoint: Option<&str>,
    ocr_api_key: Option<&str>,
) -> Result<OcrResult, VisionError> {
    let endpoint = ocr_endpoint.ok_or_else(|| {
        VisionError::new(
            VisionErrorKind::ProviderUnconfigured,
            "OCR backend is not configured",
        )
    })?;

    let payload = serde_json::json!({
        "mime_type": asset.mime_type,
        "asset_base64": BASE64.encode(asset.bytes.as_slice()),
        "language_hint": request.options.language_hint,
    });

    let mut req = ureq::post(endpoint).set("User-Agent", "selene-vision-ocr/1.0");
    if let Some(key) = ocr_api_key {
        req = req.set("Authorization", format!("Bearer {}", key).as_str());
    }

    let response = req.send_json(payload).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("OCR provider request failed: {}", error),
        )
    })?;

    if response.status() != 200 {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("OCR provider non-200 status {}", response.status()),
        ));
    }

    let value: Value = response.into_json().map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("OCR provider response parse failed: {}", error),
        )
    })?;

    parse_ocr_response(&value, request)
}

pub fn parse_ocr_response(
    value: &Value,
    request: &VisionToolRequest,
) -> Result<OcrResult, VisionError> {
    let ocr_engine_id = value
        .get("ocr_engine_id")
        .and_then(Value::as_str)
        .unwrap_or("vision_ocr")
        .trim()
        .to_string();
    let language = value
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or_else(|| request.options.language_hint.as_deref().unwrap_or("en-US"))
        .trim()
        .to_string();

    let blocks = value
        .get("blocks")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                "OCR response missing blocks",
            )
        })?;

    let mut text_blocks = Vec::new();
    for entry in blocks {
        let text = normalize_text(
            entry
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        if text.is_empty() {
            continue;
        }
        let confidence = entry
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if !meets_ocr_threshold(confidence) {
            continue;
        }
        let bbox = parse_bbox(entry.get("bbox")).ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                "OCR bbox is invalid",
            )
        })?;

        text_blocks.push(OcrTextBlock {
            bbox,
            text: text.clone(),
            confidence,
            pii_suspected: if request.options.safe_mode {
                Some(detect_pii(&text))
            } else {
                None
            },
        });
    }

    sort_ocr_blocks(&mut text_blocks);

    if text_blocks.is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::InsufficientEvidence,
            "OCR returned no blocks above deterministic confidence threshold",
        ));
    }

    let lines = text_blocks
        .iter()
        .map(|block| block.text.clone())
        .collect::<Vec<String>>();

    Ok(OcrResult {
        page_or_frame_index: 0,
        timestamp_ms: None,
        ocr_engine_id,
        language,
        text_blocks,
        full_text: deterministic_join_lines(lines.as_slice()),
    })
}

fn parse_bbox(value: Option<&Value>) -> Option<BoundingBox> {
    let map = value?.as_object()?;
    Some(BoundingBox {
        x: map.get("x")?.as_f64()?,
        y: map.get("y")?.as_f64()?,
        w: map.get("w")?.as_f64()?,
        h: map.get("h")?.as_f64()?,
    })
}

fn normalize_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

fn detect_pii(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains('@') {
        return true;
    }
    let digits = text.chars().filter(|ch| ch.is_ascii_digit()).count();
    digits >= 8
}
