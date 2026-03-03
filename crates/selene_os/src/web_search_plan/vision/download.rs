#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::proxy_redaction::redact_proxy_url;
use crate::web_search_plan::proxy::proxy_self_check::run_startup_self_check;
use crate::web_search_plan::proxy::ProxyErrorKind;
use crate::web_search_plan::vision::asset_ref::{VisionAssetRef, ALLOWED_MEDIA_MIME_TYPES};
use crate::web_search_plan::vision::redaction::{is_http_locator, redact_locator};
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode, VisionRuntimeNetworkConfig,
    VisionToolRequestPacket,
};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LoadedAsset {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub size_bytes: u64,
    pub redacted_locator: String,
}

pub trait AssetLoader {
    fn load(&self, request: &VisionToolRequestPacket) -> Result<LoadedAsset, VisionProviderError>;
}

#[derive(Debug, Clone)]
pub struct DefaultAssetLoader {
    pub network: VisionRuntimeNetworkConfig,
}

impl DefaultAssetLoader {
    pub fn from_env(network: VisionRuntimeNetworkConfig) -> Self {
        Self { network }
    }
}

impl AssetLoader for DefaultAssetLoader {
    fn load(&self, request: &VisionToolRequestPacket) -> Result<LoadedAsset, VisionProviderError> {
        let start = Instant::now();
        let asset_ref = &request.asset_ref;

        if !asset_ref.is_supported_mime_type() {
            return Err(VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::UnsupportedMediaType,
                VisionReasonCode::PolicyViolation,
                "unsupported media type",
                0,
            ));
        }

        if asset_ref.size_bytes > request.budgets.max_bytes {
            return Err(VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::PolicyViolation,
                VisionReasonCode::PolicyViolation,
                "asset size exceeds budget cap",
                0,
            ));
        }

        let redacted_locator = redact_locator(&asset_ref.locator);
        if is_http_locator(&asset_ref.locator) {
            load_remote_asset(
                asset_ref,
                request.budgets.timeout_ms,
                request.budgets.max_bytes,
                &redacted_locator,
                &self.network,
                start,
            )
        } else {
            load_local_asset(
                asset_ref,
                request.budgets.max_bytes,
                redacted_locator,
                start,
            )
        }
    }
}

fn load_local_asset(
    asset_ref: &VisionAssetRef,
    max_bytes: u64,
    redacted_locator: String,
    start: Instant,
) -> Result<LoadedAsset, VisionProviderError> {
    let path = local_path_from_locator(&asset_ref.locator).ok_or_else(|| {
        VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::PolicyViolation,
            VisionReasonCode::PolicyViolation,
            "invalid local asset locator",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let metadata = fs::metadata(&path).map_err(|_| {
        VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            "failed to read local asset metadata",
            start.elapsed().as_millis() as u64,
        )
    })?;
    let len = metadata.len();
    if len > max_bytes {
        return Err(VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::PolicyViolation,
            VisionReasonCode::PolicyViolation,
            "local asset exceeds max_bytes",
            start.elapsed().as_millis() as u64,
        ));
    }

    let bytes = fs::read(path).map_err(|_| {
        VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            "failed to read local asset",
            start.elapsed().as_millis() as u64,
        )
    })?;

    Ok(LoadedAsset {
        bytes,
        mime_type: asset_ref.normalized_mime_type(),
        size_bytes: len,
        redacted_locator,
    })
}

fn load_remote_asset(
    asset_ref: &VisionAssetRef,
    timeout_ms: u64,
    max_bytes: u64,
    redacted_locator: &str,
    network: &VisionRuntimeNetworkConfig,
    start: Instant,
) -> Result<LoadedAsset, VisionProviderError> {
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .user_agent("selene-vision-download/1.0")
        .try_proxy_from_env(false);

    if let Err(check) = run_startup_self_check(&network.proxy_config) {
        if check.error_kind == ProxyErrorKind::ProxyMisconfigured
            && check.severity.as_str() == "critical"
        {
            return Err(VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::ProxyMisconfigured,
                VisionReasonCode::PolicyViolation,
                &check.details,
                start.elapsed().as_millis() as u64,
            ));
        }
    }

    if let Some(proxy_raw) = network.proxy_url_for(&asset_ref.locator) {
        let _ = redact_proxy_url(proxy_raw).map_err(|_| {
            VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::ProxyMisconfigured,
                VisionReasonCode::PolicyViolation,
                "proxy redaction failed",
                start.elapsed().as_millis() as u64,
            )
        })?;
        let proxy = ureq::Proxy::new(proxy_raw).map_err(|_| {
            VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::ProxyMisconfigured,
                VisionReasonCode::PolicyViolation,
                "invalid proxy url",
                start.elapsed().as_millis() as u64,
            )
        })?;
        builder = builder.proxy(proxy);
    }

    let response = builder
        .build()
        .get(&asset_ref.locator)
        .set("Accept", ALLOWED_MEDIA_MIME_TYPES.join(",").as_str())
        .call()
        .map_err(|err| map_transport_error(err, start.elapsed().as_millis() as u64))?;

    let response_mime = response
        .header("Content-Type")
        .and_then(|raw| raw.split(';').next())
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_else(|| asset_ref.normalized_mime_type());

    if !ALLOWED_MEDIA_MIME_TYPES
        .iter()
        .any(|mime| *mime == response_mime)
    {
        return Err(VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::UnsupportedMediaType,
            VisionReasonCode::PolicyViolation,
            "remote MIME type is not allowed",
            start.elapsed().as_millis() as u64,
        ));
    }

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer).map_err(|_| {
            VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "failed while streaming asset bytes",
                start.elapsed().as_millis() as u64,
            )
        })?;

        if count == 0 {
            break;
        }

        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() as u64 > max_bytes {
            return Err(VisionProviderError::new(
                "vision_download",
                "download",
                VisionProviderErrorKind::PolicyViolation,
                VisionReasonCode::PolicyViolation,
                "remote asset exceeded max_bytes",
                start.elapsed().as_millis() as u64,
            ));
        }
    }

    Ok(LoadedAsset {
        size_bytes: bytes.len() as u64,
        bytes,
        mime_type: response_mime,
        redacted_locator: redacted_locator.to_string(),
    })
}

fn local_path_from_locator(locator: &str) -> Option<PathBuf> {
    if locator.trim().is_empty() {
        return None;
    }

    if locator.starts_with("file://") {
        let parsed = url::Url::parse(locator).ok()?;
        return parsed.to_file_path().ok();
    }

    Some(PathBuf::from(locator))
}

fn map_transport_error(err: ureq::Error, latency_ms: u64) -> VisionProviderError {
    match err {
        ureq::Error::Status(status, _) => VisionProviderError::new(
            "vision_download",
            "download",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            &format!("HTTP status {}", status),
            latency_ms,
        ),
        ureq::Error::Transport(transport) => {
            let combined = format!("{:?} {}", transport.kind(), transport).to_ascii_lowercase();
            if combined.contains("timeout") {
                VisionProviderError::new(
                    "vision_download",
                    "download",
                    VisionProviderErrorKind::TimeoutExceeded,
                    VisionReasonCode::TimeoutExceeded,
                    "transport timeout",
                    latency_ms,
                )
            } else {
                VisionProviderError::new(
                    "vision_download",
                    "download",
                    VisionProviderErrorKind::ProviderUpstreamFailed,
                    VisionReasonCode::ProviderUpstreamFailed,
                    "transport failure",
                    latency_ms,
                )
            }
        }
    }
}
