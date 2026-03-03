#![forbid(unsafe_code)]

use crate::web_search_plan::vision::asset_ref::file_extension_for_mime;
use crate::web_search_plan::vision::{
    VisionProviderError, VisionProviderErrorKind, VisionReasonCode,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub trait AudioExtractor {
    fn extract_audio(
        &self,
        asset_hash: &str,
        mime_type: &str,
        video_bytes: &[u8],
    ) -> Result<Vec<u8>, VisionProviderError>;
}

#[derive(Debug, Clone, Default)]
pub struct FfmpegAudioExtractor;

impl AudioExtractor for FfmpegAudioExtractor {
    fn extract_audio(
        &self,
        asset_hash: &str,
        mime_type: &str,
        video_bytes: &[u8],
    ) -> Result<Vec<u8>, VisionProviderError> {
        extract_audio_linear16_mono_16k(asset_hash, mime_type, video_bytes)
    }
}

pub fn extract_audio_linear16_mono_16k(
    asset_hash: &str,
    mime_type: &str,
    video_bytes: &[u8],
) -> Result<Vec<u8>, VisionProviderError> {
    let start = Instant::now();
    let input_path = write_temp_asset(asset_hash, mime_type, video_bytes).map_err(|_| {
        VisionProviderError::new(
            "vision_video",
            "stt",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            "failed to stage video asset",
            start.elapsed().as_millis() as u64,
        )
    })?;

    let output_path = temp_root().join(format!("{}_audio.wav", asset_hash));

    let status = Command::new("ffmpeg")
        .arg("-nostdin")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(&input_path)
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-f")
        .arg("wav")
        .arg(&output_path)
        .status();

    let result = match status {
        Ok(exit) if exit.success() => fs::read(&output_path).map_err(|_| {
            VisionProviderError::new(
                "vision_video",
                "stt",
                VisionProviderErrorKind::ProviderUpstreamFailed,
                VisionReasonCode::ProviderUpstreamFailed,
                "failed to read extracted audio",
                start.elapsed().as_millis() as u64,
            )
        }),
        Ok(_) => Err(VisionProviderError::new(
            "vision_video",
            "stt",
            VisionProviderErrorKind::ProviderUpstreamFailed,
            VisionReasonCode::ProviderUpstreamFailed,
            "ffmpeg audio extraction failed",
            start.elapsed().as_millis() as u64,
        )),
        Err(_) => Err(VisionProviderError::new(
            "vision_video",
            "stt",
            VisionProviderErrorKind::ProviderUnconfigured,
            VisionReasonCode::ProviderUnconfigured,
            "ffmpeg is not available",
            start.elapsed().as_millis() as u64,
        )),
    };

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path);

    result
}

pub fn write_temp_asset(
    asset_hash: &str,
    mime_type: &str,
    bytes: &[u8],
) -> std::io::Result<PathBuf> {
    let dir = temp_root();
    fs::create_dir_all(&dir)?;

    let ext = file_extension_for_mime(mime_type);
    let path = dir.join(format!("{}.{}", asset_hash, ext));
    fs::write(&path, bytes)?;
    Ok(path)
}

pub fn temp_root() -> PathBuf {
    std::env::temp_dir().join("selene_web_search_plan_vision")
}

pub fn read_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}
