#![forbid(unsafe_code)]

use crate::web_search_plan::vision::packet_builder::{
    deterministic_join_lines, sort_transcript_segments, TranscriptSegment, VideoTranscriptResult,
};
use crate::web_search_plan::vision::thresholds::meets_transcript_threshold;
use crate::web_search_plan::vision::{VisionError, VisionErrorKind, VisionToolRequest};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::Value;

const GOOGLE_STT_ENDPOINT: &str = "https://speech.googleapis.com/v1/speech:recognize";

pub fn run_google_stt_runtime(
    audio_bytes: &[u8],
    request: &VisionToolRequest,
    google_api_key: Option<&str>,
    endpoint_override: Option<&str>,
) -> Result<VideoTranscriptResult, VisionError> {
    let api_key = google_api_key.ok_or_else(|| {
        VisionError::new(
            VisionErrorKind::ProviderUnconfigured,
            "Google STT API key is not configured",
        )
    })?;

    let endpoint = endpoint_override.unwrap_or(GOOGLE_STT_ENDPOINT);
    let language = request
        .options
        .language_hint
        .clone()
        .unwrap_or_else(|| "en-US".to_string());

    let payload = serde_json::json!({
        "config": {
            "encoding": "LINEAR16",
            "sampleRateHertz": 16000,
            "languageCode": language,
            "enableWordTimeOffsets": true,
            "enableWordConfidence": true
        },
        "audio": {
            "content": BASE64.encode(audio_bytes)
        }
    });

    let url = format!("{}?key={}", endpoint, api_key);
    let response = ureq::post(url.as_str())
        .set("User-Agent", "selene-vision-google-stt/1.0")
        .send_json(payload)
        .map_err(|error| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                format!("google stt request failed: {}", error),
            )
        })?;

    if response.status() != 200 {
        return Err(VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("google stt non-200 status {}", response.status()),
        ));
    }

    let value: Value = response.into_json().map_err(|error| {
        VisionError::new(
            VisionErrorKind::ProviderUpstreamFailed,
            format!("google stt parse failed: {}", error),
        )
    })?;

    parse_google_stt_response(&value)
}

pub fn parse_google_stt_response(value: &Value) -> Result<VideoTranscriptResult, VisionError> {
    let results = value
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VisionError::new(
                VisionErrorKind::ProviderUpstreamFailed,
                "stt response missing results",
            )
        })?;

    let mut segments = Vec::new();
    for (index, result) in results.iter().enumerate() {
        let alternative = result
            .get("alternatives")
            .and_then(Value::as_array)
            .and_then(|alternatives| alternatives.first())
            .ok_or_else(|| {
                VisionError::new(
                    VisionErrorKind::ProviderUpstreamFailed,
                    "stt response missing alternatives",
                )
            })?;

        let text = alternative
            .get("transcript")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }

        let confidence = alternative
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        if !meets_transcript_threshold(confidence) {
            continue;
        }

        let (start_ms, end_ms) = parse_word_timing(alternative, index as u64);
        segments.push(TranscriptSegment {
            start_ms,
            end_ms,
            text,
            confidence,
        });
    }

    sort_transcript_segments(&mut segments);
    if segments.is_empty() {
        return Err(VisionError::new(
            VisionErrorKind::InsufficientEvidence,
            "stt produced no segments above threshold",
        ));
    }

    let full_text = deterministic_join_lines(
        &segments
            .iter()
            .map(|segment| segment.text.clone())
            .collect::<Vec<String>>(),
    );

    Ok(VideoTranscriptResult {
        stt_provider_id: "GOOGLE_STT".to_string(),
        language: "en-US".to_string(),
        segments,
        full_transcript: full_text,
    })
}

fn parse_word_timing(alternative: &Value, fallback_index: u64) -> (u64, u64) {
    let words = alternative
        .get("words")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if words.is_empty() {
        let start = fallback_index * 1_000;
        return (start, start + 1_000);
    }

    let start = words
        .first()
        .and_then(|word| word.get("startTime").and_then(Value::as_str))
        .and_then(parse_duration_ms)
        .unwrap_or(fallback_index * 1_000);
    let end = words
        .last()
        .and_then(|word| word.get("endTime").and_then(Value::as_str))
        .and_then(parse_duration_ms)
        .unwrap_or(start + 1_000);

    (start, end.max(start))
}

fn parse_duration_ms(value: &str) -> Option<u64> {
    let trimmed = value.trim().trim_end_matches('s');
    if trimmed.is_empty() {
        return None;
    }
    let seconds = trimmed.parse::<f64>().ok()?;
    Some((seconds * 1000.0).round().max(0.0) as u64)
}
