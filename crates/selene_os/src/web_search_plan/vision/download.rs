#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::vision::asset_ref::{
    is_image_mime, is_video_mime, redacted_asset_locator,
};
use crate::web_search_plan::vision::redaction::redact_url;
use crate::web_search_plan::vision::{
    VisionError, VisionErrorKind, VisionRuntimeConfig, VisionToolRequest,
};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct LoadedAsset {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub redacted_locator: String,
    pub latency_ms: u64,
}

pub fn load_asset(
    request: &VisionToolRequest,
    config: &VisionRuntimeConfig,
) -> Result<LoadedAsset, VisionError> {
    if request.asset_ref.size_bytes > request.budgets.max_bytes {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset size exceeds request max_bytes budget",
        ));
    }

    let mime = request.asset_ref.mime_type.to_ascii_lowercase();
    if !(is_image_mime(&mime) || is_video_mime(&mime)) {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            format!("unsupported media_type {}", request.asset_ref.mime_type),
        ));
    }

    let start = Instant::now();
    let bytes = if request.asset_ref.locator.starts_with("http://")
        || request.asset_ref.locator.starts_with("https://")
    {
        if !request.options.analyze_url {
            return Err(VisionError::new(
                VisionErrorKind::PolicyViolation,
                "URL locator requires options.analyze_url=true",
            ));
        }
        fetch_url_bytes(request, config)?
    } else {
        read_local_bytes(request)?
    };

    if bytes.len() as u64 > request.budgets.max_bytes {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "downloaded bytes exceed max_bytes budget",
        ));
    }

    let hash = sha256_hex(bytes.as_slice());
    if hash != request.asset_ref.asset_hash {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "asset_hash mismatch for loaded asset",
        ));
    }

    Ok(LoadedAsset {
        bytes,
        mime_type: request.asset_ref.mime_type.to_ascii_lowercase(),
        redacted_locator: redacted_asset_locator(&request.asset_ref),
        latency_ms: start.elapsed().as_millis() as u64,
    })
}

fn fetch_url_bytes(
    request: &VisionToolRequest,
    config: &VisionRuntimeConfig,
) -> Result<Vec<u8>, VisionError> {
    let mut builder = ureq::AgentBuilder::new().timeout(std::time::Duration::from_millis(
        request.budgets.timeout_ms.min(config.max_timeout_ms),
    ));

    if let Some(proxy_url) = config.proxy_url_for_asset_mime(&request.asset_ref.mime_type) {
        let proxy = ureq::Proxy::new(proxy_url.as_str()).map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                format!("invalid proxy configuration: {}", error),
            )
        })?;
        builder = builder.proxy(proxy);
    }

    let agent = builder.build();
    let response = agent
        .get(request.asset_ref.locator.as_str())
        .set("User-Agent", "selene-vision-download/1.0")
        .call()
        .map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                format!(
                    "asset download failed for {}: {}",
                    redact_url(request.asset_ref.locator.as_str()),
                    error
                ),
            )
        })?;

    if response.status() != 200 {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!(
                "asset download non-200 status={} for {}",
                response.status(),
                redact_url(request.asset_ref.locator.as_str())
            ),
        ));
    }

    let mut reader = response.into_reader();
    let mut limited = reader.by_ref().take(request.budgets.max_bytes + 1);
    let mut bytes = Vec::new();
    limited.read_to_end(&mut bytes).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed reading downloaded asset bytes: {}", error),
        )
    })?;

    Ok(bytes)
}

fn read_local_bytes(request: &VisionToolRequest) -> Result<Vec<u8>, VisionError> {
    let path = resolve_local_locator(request.asset_ref.locator.as_str())?;
    let metadata = fs::metadata(&path).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed reading local asset metadata: {}", error),
        )
    })?;

    if metadata.len() > request.budgets.max_bytes {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "local asset size exceeds max_bytes budget",
        ));
    }

    fs::read(&path).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed reading local asset bytes: {}", error),
        )
    })
}

fn resolve_local_locator(locator: &str) -> Result<PathBuf, VisionError> {
    if let Some(stripped) = locator.strip_prefix("file://") {
        return Ok(PathBuf::from(stripped));
    }
    if locator.starts_with("asset://") {
        let local = locator.trim_start_matches("asset://");
        return Ok(PathBuf::from(local));
    }
    if locator.starts_with("http://") || locator.starts_with("https://") {
        return Err(VisionError::new(
            VisionErrorKind::PolicyViolation,
            "HTTP locator must be fetched via URL path",
        ));
    }
    Ok(PathBuf::from(locator))
}
