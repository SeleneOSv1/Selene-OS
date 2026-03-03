#![forbid(unsafe_code)]

use crate::web_search_plan::vision::download::LoadedAsset;
use crate::web_search_plan::vision::packet_builder::{
    hash_bytes, sort_keyframes, KeyframeEntry, KeyframeIndexResult,
};
use crate::web_search_plan::vision::{VisionError, VisionErrorKind, VisionToolRequest};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn extract_keyframes_runtime(
    asset: &LoadedAsset,
    request: &VisionToolRequest,
) -> Result<KeyframeIndexResult, VisionError> {
    let ffmpeg = std::env::var("SELENE_FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".to_string());
    let work_dir = prepare_temp_dir()?;
    let input_path = work_dir.join("input_video.bin");
    let frames_dir = work_dir.join("frames");
    fs::create_dir_all(&frames_dir).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed creating keyframe dir: {}", error),
        )
    })?;

    fs::write(&input_path, &asset.bytes).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed writing temp video: {}", error),
        )
    })?;

    let stride_ms = request.options.frame_stride_ms.unwrap_or(1000).max(100);
    let fps = format!("1/{}", (stride_ms as f64 / 1000.0));
    let mut cmd = Command::new(ffmpeg);
    cmd.arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input_path.as_os_str())
        .arg("-vf")
        .arg(format!(
            "fps={},scale=640:-1:flags=bicubic,format=rgb24",
            fps
        ))
        .arg("-vsync")
        .arg("vfr");

    if let Some(max_frames) = request.options.max_frames {
        cmd.arg("-frames:v").arg(max_frames.to_string());
    }

    let output_pattern = frames_dir.join("frame_%06d.png");
    cmd.arg(output_pattern.as_os_str());

    let status = cmd.status().map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUnconfigured,
            format!("ffmpeg unavailable for keyframe extraction: {}", error),
        )
    })?;

    if !status.success() {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            "ffmpeg keyframe extraction failed",
        ));
    }

    let mut paths = fs::read_dir(&frames_dir)
        .map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                format!("failed listing keyframes: {}", error),
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<PathBuf>>();
    paths.sort();

    let mut keyframes = Vec::new();
    for (index, frame_path) in paths.iter().enumerate() {
        let bytes = fs::read(frame_path).map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                format!("failed reading keyframe bytes: {}", error),
            )
        })?;
        keyframes.push(KeyframeEntry {
            timestamp_ms: (index as u64) * stride_ms as u64,
            frame_index: index as u32,
            frame_hash: hash_bytes(bytes.as_slice()),
        });
    }

    sort_keyframes(&mut keyframes);
    if keyframes.is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::InsufficientEvidence,
            "no keyframes extracted",
        ));
    }

    Ok(KeyframeIndexResult { keyframes })
}

fn prepare_temp_dir() -> Result<PathBuf, VisionError> {
    let mut dir = std::env::temp_dir();
    dir.push("selene_vision_keyframes");
    fs::create_dir_all(&dir).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed creating keyframe temp dir: {}", error),
        )
    })?;
    Ok(dir)
}
