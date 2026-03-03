#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use serde::{Deserialize, Serialize};

pub const ALLOWED_MEDIA_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "video/mp4",
    "video/mov",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionAssetRef {
    pub asset_hash: String,
    pub locator: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

impl VisionAssetRef {
    pub fn normalized_mime_type(&self) -> String {
        self.mime_type.trim().to_ascii_lowercase()
    }

    pub fn is_supported_mime_type(&self) -> bool {
        ALLOWED_MEDIA_MIME_TYPES
            .iter()
            .any(|mime| *mime == self.normalized_mime_type())
    }

    pub fn is_image(&self) -> bool {
        self.normalized_mime_type().starts_with("image/")
    }

    pub fn is_video(&self) -> bool {
        self.normalized_mime_type().starts_with("video/")
    }

    pub fn verify_hash(&self, bytes: &[u8]) -> bool {
        let actual = sha256_hex(bytes);
        actual == self.asset_hash.trim().to_ascii_lowercase()
    }
}

pub fn file_extension_for_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/mov" => "mov",
        _ => "bin",
    }
}
