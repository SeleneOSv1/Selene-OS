#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1lang::{
    LangCapabilityId, LangMultipleDetectOk, LangMultipleDetectRequest, LangPlanScope, LangRefuse,
    LangResponseMode, LangResponsePlanItem, LangSegment, LangSegmentResponseMapOk,
    LangSegmentResponseMapRequest, LangValidationStatus, Ph1LangRequest, Ph1LangResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LANG reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_LANG_OK_MULTIPLE_DETECT: ReasonCodeId = ReasonCodeId(0x4C41_0001);
    pub const PH1_LANG_OK_SEGMENT_RESPONSE_MAP: ReasonCodeId = ReasonCodeId(0x4C41_0002);

    pub const PH1_LANG_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4C41_00F1);
    pub const PH1_LANG_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4C41_00F2);
    pub const PH1_LANG_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4C41_00F3);
    pub const PH1_LANG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C41_00F4);
    pub const PH1_LANG_SEGMENTATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C41_00F5);
    pub const PH1_LANG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C41_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LangConfig {
    pub max_transcript_chars: usize,
    pub max_segments: u8,
    pub max_diagnostics: u8,
}

impl Ph1LangConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_transcript_chars: 2048,
            max_segments: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1LangRuntime {
    config: Ph1LangConfig,
}

impl Ph1LangRuntime {
    pub fn new(config: Ph1LangConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1LangRequest) -> Ph1LangResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_LANG_INPUT_SCHEMA_INVALID,
                "lang request failed contract validation",
            );
        }

        match req {
            Ph1LangRequest::LangMultipleDetect(r) => self.run_multiple_detect(r),
            Ph1LangRequest::LangSegmentResponseMap(r) => self.run_segment_response_map(r),
        }
    }

    fn run_multiple_detect(&self, req: &LangMultipleDetectRequest) -> Ph1LangResponse {
        if req.transcript_text.trim().is_empty() {
            return self.refuse(
                LangCapabilityId::LangMultipleDetect,
                reason_codes::PH1_LANG_UPSTREAM_INPUT_MISSING,
                "transcript text is empty",
            );
        }

        if req.transcript_text.len() > self.config.max_transcript_chars {
            return self.refuse(
                LangCapabilityId::LangMultipleDetect,
                reason_codes::PH1_LANG_BUDGET_EXCEEDED,
                "transcript exceeds configured budget",
            );
        }

        let segment_budget = min(req.envelope.max_segments, self.config.max_segments) as usize;
        if segment_budget == 0 {
            return self.refuse(
                LangCapabilityId::LangMultipleDetect,
                reason_codes::PH1_LANG_BUDGET_EXCEEDED,
                "segment budget exceeded",
            );
        }

        let inferred = infer_segments(
            &req.transcript_text,
            req.locale_hint.as_deref(),
            segment_budget,
        );

        let (detected_languages, segment_spans, dominant_language_tag) = match inferred {
            Ok(v) => v,
            Err(reason_code) => {
                return self.refuse(
                    LangCapabilityId::LangMultipleDetect,
                    reason_code,
                    "failed to infer language segments",
                )
            }
        };

        match LangMultipleDetectOk::v1(
            reason_codes::PH1_LANG_OK_MULTIPLE_DETECT,
            detected_languages,
            segment_spans,
            dominant_language_tag,
            true,
        ) {
            Ok(ok) => Ph1LangResponse::LangMultipleDetectOk(ok),
            Err(_) => self.refuse(
                LangCapabilityId::LangMultipleDetect,
                reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR,
                "failed to build multiple-detect output",
            ),
        }
    }

    fn run_segment_response_map(&self, req: &LangSegmentResponseMapRequest) -> Ph1LangResponse {
        if req.detected_languages.is_empty() || req.segment_spans.is_empty() {
            return self.refuse(
                LangCapabilityId::LangSegmentResponseMap,
                reason_codes::PH1_LANG_UPSTREAM_INPUT_MISSING,
                "missing detected languages or segment spans",
            );
        }

        if req.detected_languages.len() > self.config.max_segments as usize
            || req.segment_spans.len() > self.config.max_segments as usize
        {
            return self.refuse(
                LangCapabilityId::LangSegmentResponseMap,
                reason_codes::PH1_LANG_BUDGET_EXCEEDED,
                "language segmentation exceeds configured budget",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();
        let mut detected_set: BTreeSet<String> = BTreeSet::new();
        for language_tag in &req.detected_languages {
            detected_set.insert(language_tag.to_ascii_lowercase());
        }

        for (idx, segment) in req.segment_spans.iter().enumerate() {
            if !detected_set.contains(&segment.language_tag.to_ascii_lowercase()) {
                diagnostics.push(format!("segment_{idx}_language_not_in_detected_set"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        if !req.user_language_preferences.is_empty()
            && !req
                .user_language_preferences
                .iter()
                .any(|pref| detected_set.contains(&pref.to_ascii_lowercase()))
            && diagnostics.len() < self.config.max_diagnostics as usize
        {
            diagnostics.push("user_language_preferences_no_match".to_string());
        }

        let default_response_language = preferred_or_dominant_language(
            &req.user_language_preferences,
            &req.detected_languages,
            &req.segment_spans,
        );

        let response_language_plan = match build_response_plan(
            req.response_mode,
            &default_response_language,
            &req.segment_spans,
        ) {
            Ok(plan) => plan,
            Err(reason_code) => {
                return self.refuse(
                    LangCapabilityId::LangSegmentResponseMap,
                    reason_code,
                    "failed to build response language plan",
                )
            }
        };

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                LangValidationStatus::Ok,
                reason_codes::PH1_LANG_OK_SEGMENT_RESPONSE_MAP,
            )
        } else {
            (
                LangValidationStatus::Fail,
                reason_codes::PH1_LANG_VALIDATION_FAILED,
            )
        };

        match LangSegmentResponseMapOk::v1(
            reason_code,
            validation_status,
            response_language_plan,
            default_response_language,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1LangResponse::LangSegmentResponseMapOk(ok),
            Err(_) => self.refuse(
                LangCapabilityId::LangSegmentResponseMap,
                reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR,
                "failed to build segment-response-map output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: LangCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1LangResponse {
        let r = LangRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("LangRefuse::v1 must construct for static message");
        Ph1LangResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1LangRequest) -> LangCapabilityId {
    match req {
        Ph1LangRequest::LangMultipleDetect(_) => LangCapabilityId::LangMultipleDetect,
        Ph1LangRequest::LangSegmentResponseMap(_) => LangCapabilityId::LangSegmentResponseMap,
    }
}

fn infer_segments(
    transcript_text: &str,
    locale_hint: Option<&str>,
    segment_budget: usize,
) -> Result<(Vec<String>, Vec<LangSegment>, String), ReasonCodeId> {
    let mut boundaries: Vec<(u32, u32, String)> = Vec::new();

    let mut current_start: Option<usize> = None;
    let mut current_tag: Option<String> = None;

    for (idx, ch) in transcript_text.char_indices() {
        let inferred = infer_char_language_tag(ch);

        match (&current_tag, inferred) {
            (None, Some(tag)) => {
                current_start = Some(idx);
                current_tag = Some(tag.to_string());
            }
            (Some(active), Some(tag)) if active.eq_ignore_ascii_case(tag) => {
                // Continue the active segment.
            }
            (Some(active), Some(tag)) if !active.eq_ignore_ascii_case(tag) => {
                let start = current_start.ok_or(reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR)?;
                boundaries.push((start as u32, idx as u32, active.clone()));
                current_start = Some(idx);
                current_tag = Some(tag.to_string());
            }
            (Some(_), None) => {
                // Keep punctuation/spacing attached to current segment.
            }
            (None, None) => {
                // Skip non-language chars until a language-bearing char is found.
            }
            _ => {}
        }
    }

    if let (Some(start), Some(tag)) = (current_start, current_tag) {
        boundaries.push((start as u32, transcript_text.len() as u32, tag));
    }

    if boundaries.is_empty() {
        let fallback_tag = locale_hint_to_language(locale_hint).unwrap_or_else(|| "en".to_string());
        boundaries.push((0, transcript_text.len() as u32, fallback_tag));
    }

    if boundaries.len() > segment_budget {
        return Err(reason_codes::PH1_LANG_BUDGET_EXCEEDED);
    }

    let mut detected_languages: Vec<String> = Vec::new();
    let mut segments: Vec<LangSegment> = Vec::new();
    let mut byte_totals: BTreeMap<String, u32> = BTreeMap::new();

    for (idx, (start, end, language_tag)) in boundaries.into_iter().enumerate() {
        if end <= start {
            continue;
        }

        let text = transcript_text
            .get(start as usize..end as usize)
            .ok_or(reason_codes::PH1_LANG_SEGMENTATION_FAILED)?
            .to_string();

        let segment = LangSegment::v1(
            format!("seg_{idx:03}"),
            start,
            end,
            language_tag.clone(),
            text,
        )
        .map_err(|_| reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR)?;

        if !detected_languages
            .iter()
            .any(|l| l.eq_ignore_ascii_case(&language_tag))
        {
            detected_languages.push(language_tag.clone());
        }

        let entry = byte_totals.entry(language_tag).or_insert(0);
        *entry += end - start;
        segments.push(segment);
    }

    if segments.is_empty() {
        return Err(reason_codes::PH1_LANG_SEGMENTATION_FAILED);
    }

    let dominant_language_tag = byte_totals
        .iter()
        .max_by_key(|(_, bytes)| *bytes)
        .map(|(tag, _)| tag.clone())
        .ok_or(reason_codes::PH1_LANG_SEGMENTATION_FAILED)?;

    Ok((detected_languages, segments, dominant_language_tag))
}

fn preferred_or_dominant_language(
    user_language_preferences: &[String],
    detected_languages: &[String],
    segment_spans: &[LangSegment],
) -> String {
    for preferred in user_language_preferences {
        if detected_languages
            .iter()
            .any(|detected| detected.eq_ignore_ascii_case(preferred))
        {
            return preferred.clone();
        }
    }

    let mut byte_totals: BTreeMap<String, u32> = BTreeMap::new();
    for segment in segment_spans {
        let entry = byte_totals.entry(segment.language_tag.clone()).or_insert(0);
        *entry += segment.end_byte - segment.start_byte;
    }

    byte_totals
        .iter()
        .max_by_key(|(_, bytes)| *bytes)
        .map(|(tag, _)| tag.clone())
        .unwrap_or_else(|| detected_languages[0].clone())
}

fn build_response_plan(
    response_mode: LangResponseMode,
    default_response_language: &str,
    segment_spans: &[LangSegment],
) -> Result<Vec<LangResponsePlanItem>, ReasonCodeId> {
    match response_mode {
        LangResponseMode::Voice => LangResponsePlanItem::v1(
            LangPlanScope::Turn,
            None,
            default_response_language.to_string(),
        )
        .map(|item| vec![item])
        .map_err(|_| reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR),
        LangResponseMode::Text => {
            let mut plan: Vec<LangResponsePlanItem> = Vec::new();
            for segment in segment_spans {
                let item = LangResponsePlanItem::v1(
                    LangPlanScope::Segment,
                    Some(segment.segment_id.clone()),
                    segment.language_tag.clone(),
                )
                .map_err(|_| reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR)?;
                plan.push(item);
            }
            if plan.is_empty() {
                return Err(reason_codes::PH1_LANG_SEGMENTATION_FAILED);
            }
            Ok(plan)
        }
    }
}

fn locale_hint_to_language(locale_hint: Option<&str>) -> Option<String> {
    locale_hint
        .and_then(|hint| hint.split('-').next())
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
}

fn infer_char_language_tag(ch: char) -> Option<&'static str> {
    if ch.is_ascii_alphabetic() {
        return Some("en");
    }

    let code = ch as u32;
    match code {
        // CJK Unified Ideographs
        0x4E00..=0x9FFF | 0x3400..=0x4DBF => Some("zh"),
        // Hiragana + Katakana
        0x3040..=0x30FF => Some("ja"),
        // Hangul
        0xAC00..=0xD7AF => Some("ko"),
        // Arabic
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => Some("ar"),
        // Cyrillic
        0x0400..=0x04FF => Some("ru"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1lang::{
        LangRequestEnvelope, LangSourceModality, Ph1LangRequest,
    };

    fn envelope(max_segments: u8) -> LangRequestEnvelope {
        LangRequestEnvelope::v1(CorrelationId(901), TurnId(51), max_segments).unwrap()
    }

    #[test]
    fn at_lang_01_mixed_language_detection_returns_ordered_segments() {
        let runtime = Ph1LangRuntime::new(Ph1LangConfig::mvp_v1());
        let req = Ph1LangRequest::LangMultipleDetect(
            LangMultipleDetectRequest::v1(
                envelope(8),
                "hello 世界".to_string(),
                Some("en-US".to_string()),
                LangSourceModality::Voice,
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1LangResponse::LangMultipleDetectOk(ok) => {
                assert!(ok.validate().is_ok());
                assert!(ok
                    .detected_languages
                    .iter()
                    .any(|lang| lang.eq_ignore_ascii_case("en")));
                assert!(ok
                    .detected_languages
                    .iter()
                    .any(|lang| lang.eq_ignore_ascii_case("zh")));
                assert!(ok.segment_spans.len() >= 2);
            }
            _ => panic!("expected LangMultipleDetectOk"),
        }
    }

    #[test]
    fn at_lang_02_segment_response_map_aligns_with_text_mode_segments() {
        let runtime = Ph1LangRuntime::new(Ph1LangConfig::mvp_v1());
        let detect_req = Ph1LangRequest::LangMultipleDetect(
            LangMultipleDetectRequest::v1(
                envelope(8),
                "hello 世界".to_string(),
                Some("en-US".to_string()),
                LangSourceModality::Voice,
            )
            .unwrap(),
        );

        let detect_ok = match runtime.run(&detect_req) {
            Ph1LangResponse::LangMultipleDetectOk(ok) => ok,
            _ => panic!("expected detect ok"),
        };

        let map_req = Ph1LangRequest::LangSegmentResponseMap(
            LangSegmentResponseMapRequest::v1(
                envelope(8),
                "hello 世界".to_string(),
                Some("en-US".to_string()),
                LangSourceModality::Voice,
                detect_ok.detected_languages.clone(),
                detect_ok.segment_spans.clone(),
                vec!["en".to_string()],
                LangResponseMode::Text,
            )
            .unwrap(),
        );

        let out = runtime.run(&map_req);
        match out {
            Ph1LangResponse::LangSegmentResponseMapOk(ok) => {
                assert!(ok.validate().is_ok());
                assert_eq!(ok.validation_status, LangValidationStatus::Ok);
                assert_eq!(
                    ok.response_language_plan.len(),
                    detect_ok.segment_spans.len()
                );
                assert!(ok
                    .response_language_plan
                    .iter()
                    .all(|item| item.scope == LangPlanScope::Segment));
            }
            _ => panic!("expected LangSegmentResponseMapOk"),
        }
    }

    #[test]
    fn at_lang_03_fragmented_input_still_produces_schema_valid_detection() {
        let runtime = Ph1LangRuntime::new(Ph1LangConfig::mvp_v1());
        let req = Ph1LangRequest::LangMultipleDetect(
            LangMultipleDetectRequest::v1(
                envelope(8),
                "...hola...你好??".to_string(),
                Some("es-ES".to_string()),
                LangSourceModality::Text,
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1LangResponse::LangMultipleDetectOk(ok) => {
                assert!(ok.validate().is_ok());
                assert!(!ok.segment_spans.is_empty());
            }
            _ => panic!("expected LangMultipleDetectOk"),
        }
    }

    #[test]
    fn at_lang_04_budget_overflow_fails_closed() {
        let runtime = Ph1LangRuntime::new(Ph1LangConfig::mvp_v1());
        let req = Ph1LangRequest::LangMultipleDetect(
            LangMultipleDetectRequest::v1(
                envelope(2),
                "hello 世界 hello 世界".to_string(),
                Some("en-US".to_string()),
                LangSourceModality::Voice,
            )
            .unwrap(),
        );

        let out = runtime.run(&req);
        match out {
            Ph1LangResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_LANG_BUDGET_EXCEEDED);
            }
            _ => panic!("expected Refuse"),
        }
    }
}
