#![forbid(unsafe_code)]

pub const OCR_BLOCK_MIN_CONFIDENCE: f64 = 0.60;
pub const OBJECT_MIN_CONFIDENCE: f64 = 0.55;
pub const TRANSCRIPT_SEGMENT_MIN_CONFIDENCE: f64 = 0.50;

pub fn allow_ocr_block(confidence: f64) -> bool {
    confidence >= OCR_BLOCK_MIN_CONFIDENCE
}

pub fn allow_object(confidence: f64) -> bool {
    confidence >= OBJECT_MIN_CONFIDENCE
}

pub fn allow_transcript_segment(confidence: f64) -> bool {
    confidence >= TRANSCRIPT_SEGMENT_MIN_CONFIDENCE
}
