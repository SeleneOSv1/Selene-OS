#![forbid(unsafe_code)]

use crate::web_search_plan::vision::download::LoadedAsset;
use crate::web_search_plan::vision::packet_builder::{
    sort_detected_objects, BoundingBox, DetectedObject, ObjectDetectionResult,
};
use crate::web_search_plan::vision::thresholds::meets_object_threshold;
use crate::web_search_plan::vision::{VisionError, VisionErrorKind};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value;

pub const LABEL_MAP_VERSION: &str = "v1";

pub fn run_objects_runtime(
    asset: &LoadedAsset,
    objects_endpoint: Option<&str>,
    objects_api_key: Option<&str>,
) -> Result<ObjectDetectionResult, VisionError> {
    let endpoint = objects_endpoint.ok_or_else(|| {
        VisionError::new(
            VisionErrorKind::ProviderUnconfigured,
            "object detection backend is not configured",
        )
    })?;

    let payload = serde_json::json!({
        "mime_type": asset.mime_type,
        "asset_base64": BASE64.encode(asset.bytes.as_slice()),
        "label_map_version": LABEL_MAP_VERSION,
    });

    let mut req = ureq::post(endpoint).set("User-Agent", "selene-vision-objects/1.0");
    if let Some(key) = objects_api_key {
        req = req.set("Authorization", format!("Bearer {}", key).as_str());
    }

    let response = req.send_json(payload).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("objects provider request failed: {}", error),
        )
    })?;

    if response.status() != 200 {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("objects provider non-200 status {}", response.status()),
        ));
    }

    let value: Value = response.into_json().map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("objects provider response parse failed: {}", error),
        )
    })?;

    parse_objects_response(&value)
}

pub fn parse_objects_response(value: &Value) -> Result<ObjectDetectionResult, VisionError> {
    let model_id = value
        .get("model_id")
        .and_then(Value::as_str)
        .unwrap_or("vision_objects_model")
        .trim()
        .to_string();

    let objects = value
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                "objects response missing objects",
            )
        })?;

    let mut detected = Vec::new();
    for entry in objects {
        let confidence = entry
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if !meets_object_threshold(confidence) {
            continue;
        }
        let label = canonical_label(
            entry
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        if label.is_empty() {
            continue;
        }
        let bbox = parse_bbox(entry.get("bbox")).ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                "object bbox is invalid",
            )
        })?;

        detected.push(DetectedObject {
            label,
            bbox,
            confidence,
        });
    }

    sort_detected_objects(&mut detected);
    if detected.is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::InsufficientEvidence,
            "object detection returned no objects above threshold",
        ));
    }

    Ok(ObjectDetectionResult {
        frame_index: None,
        timestamp_ms: None,
        model_id,
        objects: detected,
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

fn canonical_label(label: &str) -> String {
    let normalized = label.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "human" => "person".to_string(),
        "automobile" => "car".to_string(),
        _ => normalized,
    }
}
