#![forbid(unsafe_code)]

use crate::web_search_plan::vision::redaction::redact_locator;
use crate::web_search_plan::vision::{VisionAssetRef, VisionError, VisionErrorKind};

pub const ALLOWED_IMAGE_MIME: &[&str] = &["image/jpeg", "image/png", "image/webp"];
pub const ALLOWED_VIDEO_MIME: &[&str] = &["video/mp4", "video/quicktime"];

pub fn validate_asset_ref(asset_ref: &VisionAssetRef) -> Result<(), VisionError> {
    if asset_ref.asset_hash.trim().is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset_ref.asset_hash is required",
        ));
    }
    if asset_ref.locator.trim().is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset_ref.locator is required",
        ));
    }
    if asset_ref.mime_type.trim().is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset_ref.mime_type is required",
        ));
    }
    if asset_ref.size_bytes == 0 {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset_ref.size_bytes must be > 0",
        ));
    }

    let mime = asset_ref.mime_type.to_ascii_lowercase();
    let allowed = ALLOWED_IMAGE_MIME.iter().chain(ALLOWED_VIDEO_MIME.iter());
    if !allowed.clone().any(|allowed| *allowed == mime) {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("unsupported media mime_type {}", asset_ref.mime_type),
        ));
    }

    Ok(())
}

pub fn is_image_mime(mime_type: &str) -> bool {
    let value = mime_type.to_ascii_lowercase();
    ALLOWED_IMAGE_MIME.iter().any(|entry| *entry == value)
}

pub fn is_video_mime(mime_type: &str) -> bool {
    let value = mime_type.to_ascii_lowercase();
    ALLOWED_VIDEO_MIME.iter().any(|entry| *entry == value)
}

pub fn redacted_asset_locator(asset_ref: &VisionAssetRef) -> String {
    redact_locator(asset_ref.locator.as_str())
}
