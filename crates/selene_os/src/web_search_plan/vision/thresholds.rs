#![forbid(unsafe_code)]

pub const OCR_BLOCK_MIN_CONFIDENCE: f64 = 0.65;
pub const OBJECT_MIN_CONFIDENCE: f64 = 0.60;
pub const TRANSCRIPT_SEGMENT_MIN_CONFIDENCE: f64 = 0.60;
pub const CAPTION_MIN_CONFIDENCE: f64 = 0.60;

pub fn normalize_confidence(value: f64) -> f64 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

pub fn meets_ocr_threshold(confidence: f64) -> bool {
    normalize_confidence(confidence) >= OCR_BLOCK_MIN_CONFIDENCE
}

pub fn meets_object_threshold(confidence: f64) -> bool {
    normalize_confidence(confidence) >= OBJECT_MIN_CONFIDENCE
}

pub fn meets_transcript_threshold(confidence: f64) -> bool {
    normalize_confidence(confidence) >= TRANSCRIPT_SEGMENT_MIN_CONFIDENCE
}
