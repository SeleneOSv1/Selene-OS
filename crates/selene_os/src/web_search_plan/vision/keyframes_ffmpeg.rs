#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use crate::web_search_plan::vision::video::{read_bytes, temp_root, write_temp_asset};
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeEntry {
    pub timestamp_ms: u64,
    pub frame_index: u32,
    pub frame_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeIndexResult {
    pub keyframes: Vec<KeyframeEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyframeRequest {
    pub asset_hash: String,
    pub mime_type: String,
    pub video_bytes: Vec<u8>,
    pub max_frames: u32,
    pub frame_stride_ms: u32,
}

pub trait KeyframeExtractor {
    fn extract_keyframes(
        &self,
        request: &KeyframeRequest,
    ) -> Result<KeyframeIndexResult, VisionProviderError>;
}

#[derive(Debug, Clone, Default)]
pub struct FfmpegKeyframeExtractor;

impl KeyframeExtractor for FfmpegKeyframeExtractor {
    fn extract_keyframes(
        &self,
        request: &KeyframeRequest,
    ) -> Result<KeyframeIndexResult, VisionProviderError> {
        let start = Instant::now();
        let input_path = write_temp_asset(
            &request.asset_hash,
            &request.mime_type,
            &request.video_bytes,
        )
        .map_err(|_| {
            VisionProviderError::new(
                "vision_keyframes",
                "keyframes",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "failed to stage video asset",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let frames_dir = temp_root().join(format!("{}_frames", request.asset_hash));
        let _ = fs::remove_dir_all(&frames_dir);
        fs::create_dir_all(&frames_dir).map_err(|_| {
            VisionProviderError::new(
                "vision_keyframes",
                "keyframes",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "failed to create keyframe directory",
                start.elapsed().as_millis() as u64,
            )
        })?;

        let stride_ms = request.frame_stride_ms.max(1);
        let fps = format!("fps=1/{}", stride_ms as f64 / 1000.0);
        let output_pattern = frames_dir.join("frame_%06d.png");

        let status = Command::new("ffmpeg")
            .arg("-nostdin")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-i")
            .arg(&input_path)
            .arg("-vf")
            .arg(format!("{},scale=640:-1:flags=lanczos,format=rgb24", fps))
            .arg("-frames:v")
            .arg(request.max_frames.max(1).to_string())
            .arg(&output_pattern)
            .status();

        if let Err(_) = status {
            let _ = fs::remove_file(&input_path);
            let _ = fs::remove_dir_all(&frames_dir);
            return Err(VisionProviderError::new(
                "vision_keyframes",
                "keyframes",
                VisionProviderErrorKind::ProviderUnconfigured,
                VisionReasonCode::ProviderUnconfigured,
                "ffmpeg is not available",
                start.elapsed().as_millis() as u64,
            ));
        }

        if !status.expect("checked above").success() {
            let _ = fs::remove_file(&input_path);
            let _ = fs::remove_dir_all(&frames_dir);
            return Err(VisionProviderError::new(
                "vision_keyframes",
                "keyframes",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "ffmpeg keyframe extraction failed",
                start.elapsed().as_millis() as u64,
            ));
        }

        let mut frame_paths: Vec<PathBuf> = fs::read_dir(&frames_dir)
            .map_err(|_| {
                VisionProviderError::new(
                    "vision_keyframes",
                    "keyframes",
                    VisionProviderErrorKind::ProviderUpstreamFailed,
                    VisionReasonCode::ProviderUpstreamFailed,
                    "failed to read keyframe directory",
                    start.elapsed().as_millis() as u64,
                )
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
            .collect();

        frame_paths.sort();

        let keyframes = frame_paths
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let bytes = read_bytes(path).map_err(|_| {
                    VisionProviderError::new(
                        "vision_keyframes",
                        "keyframes",
                        VisionProviderErrorKind::ProviderUpstreamFailed,
                        VisionReasonCode::ProviderUpstreamFailed,
                        "failed to read keyframe bytes",
                        start.elapsed().as_millis() as u64,
                    )
                })?;

                Ok(KeyframeEntry {
                    timestamp_ms: idx as u64 * stride_ms as u64,
                    frame_index: idx as u32,
                    frame_hash: sha256_hex(&bytes),
                })
            })
            .collect::<Result<Vec<KeyframeEntry>, VisionProviderError>>()?;

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_dir_all(&frames_dir);

        Ok(KeyframeIndexResult { keyframes })
    }
}
