#![forbid(unsafe_code)]

use crate::web_search_plan::vision::download::LoadedAsset;
use crate::web_search_plan::vision::packet_builder::hash_bytes;
use crate::web_search_plan::vision::{VisionError, VisionErrorKind};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn extract_audio_wav_16k_mono(asset: &LoadedAsset) -> Result<Vec<u8>, VisionError> {
    let ffmpeg = std::env::var("SELENE_FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".to_string());
    let work_dir = prepare_temp_dir()?;

    let input_path = work_dir.join("input_video.bin");
    let output_path = work_dir.join("audio.wav");

    fs::write(&input_path, &asset.bytes).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed writing temp video file: {}", error),
        )
    })?;

    let status = Command::new(ffmpeg)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input_path.as_os_str())
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-f")
        .arg("wav")
        .arg(output_path.as_os_str())
        .status()
        .map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUnconfigured,
                format!("ffmpeg unavailable for audio extraction: {}", error),
            )
        })?;

    if !status.success() {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            "ffmpeg failed extracting audio",
        ));
    }

    fs::read(output_path).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed reading extracted audio: {}", error),
        )
    })
}

pub fn asset_temp_dir_name(asset: &LoadedAsset) -> String {
    format!("selene_vision_{}", hash_bytes(asset.bytes.as_slice()))
}

fn prepare_temp_dir() -> Result<PathBuf, VisionError> {
    let mut dir = std::env::temp_dir();
    dir.push("selene_vision_runtime");
    fs::create_dir_all(&dir).map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("failed creating temp dir: {}", error),
        )
    })?;
    Ok(dir)
}
