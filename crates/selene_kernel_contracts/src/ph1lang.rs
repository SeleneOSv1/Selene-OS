#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1LANG_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangCapabilityId {
    LangMultipleDetect,
    LangSegmentResponseMap,
}

impl LangCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            LangCapabilityId::LangMultipleDetect => "LANG_MULTIPLE_DETECT",
            LangCapabilityId::LangSegmentResponseMap => "LANG_SEGMENT_RESPONSE_MAP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangSourceModality {
    Voice,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangResponseMode {
    Voice,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangPlanScope {
    Turn,
    Segment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_segments: u8,
}

impl LangRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_segments: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_segments,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for LangRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_request_envelope.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_segments == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_request_envelope.max_segments",
                reason: "must be > 0",
            });
        }
        if self.max_segments > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_request_envelope.max_segments",
                reason: "must be <= 16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangSegment {
    pub schema_version: SchemaVersion,
    pub segment_id: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub language_tag: String,
    pub segment_text: String,
}

impl LangSegment {
    pub fn v1(
        segment_id: String,
        start_byte: u32,
        end_byte: u32,
        language_tag: String,
        segment_text: String,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            segment_id,
            start_byte,
            end_byte,
            language_tag,
            segment_text,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for LangSegment {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        validate_text("lang_segment.segment_id", &self.segment_id, 64)?;
        if self.start_byte >= self.end_byte {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment.start_byte",
                reason: "must be < end_byte",
            });
        }
        validate_language_tag("lang_segment.language_tag", &self.language_tag)?;
        validate_text("lang_segment.segment_text", &self.segment_text, 512)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangResponsePlanItem {
    pub schema_version: SchemaVersion,
    pub scope: LangPlanScope,
    pub segment_id: Option<String>,
    pub language_tag: String,
}

impl LangResponsePlanItem {
    pub fn v1(
        scope: LangPlanScope,
        segment_id: Option<String>,
        language_tag: String,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            scope,
            segment_id,
            language_tag,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for LangResponsePlanItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_response_plan_item.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        match (&self.scope, &self.segment_id) {
            (LangPlanScope::Turn, None) => {}
            (LangPlanScope::Turn, Some(_)) => {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_response_plan_item.segment_id",
                    reason: "must be None when scope=TURN",
                })
            }
            (LangPlanScope::Segment, Some(segment_id)) => {
                validate_text("lang_response_plan_item.segment_id", segment_id, 64)?;
            }
            (LangPlanScope::Segment, None) => {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_response_plan_item.segment_id",
                    reason: "must be present when scope=SEGMENT",
                })
            }
        }
        validate_language_tag("lang_response_plan_item.language_tag", &self.language_tag)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangMultipleDetectRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LangRequestEnvelope,
    pub transcript_text: String,
    pub locale_hint: Option<String>,
    pub source_modality: LangSourceModality,
}

impl LangMultipleDetectRequest {
    pub fn v1(
        envelope: LangRequestEnvelope,
        transcript_text: String,
        locale_hint: Option<String>,
        source_modality: LangSourceModality,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            envelope,
            transcript_text,
            locale_hint,
            source_modality,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LangMultipleDetectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_request.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "lang_multiple_detect_request.transcript_text",
            &self.transcript_text,
            2048,
        )?;
        if let Some(locale_hint) = &self.locale_hint {
            validate_locale_hint("lang_multiple_detect_request.locale_hint", locale_hint)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangSegmentResponseMapRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LangRequestEnvelope,
    pub transcript_text: String,
    pub locale_hint: Option<String>,
    pub source_modality: LangSourceModality,
    pub detected_languages: Vec<String>,
    pub segment_spans: Vec<LangSegment>,
    pub user_language_preferences: Vec<String>,
    pub response_mode: LangResponseMode,
}

impl LangSegmentResponseMapRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: LangRequestEnvelope,
        transcript_text: String,
        locale_hint: Option<String>,
        source_modality: LangSourceModality,
        detected_languages: Vec<String>,
        segment_spans: Vec<LangSegment>,
        user_language_preferences: Vec<String>,
        response_mode: LangResponseMode,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            envelope,
            transcript_text,
            locale_hint,
            source_modality,
            detected_languages,
            segment_spans,
            user_language_preferences,
            response_mode,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LangSegmentResponseMapRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_request.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "lang_segment_response_map_request.transcript_text",
            &self.transcript_text,
            2048,
        )?;
        if let Some(locale_hint) = &self.locale_hint {
            validate_locale_hint("lang_segment_response_map_request.locale_hint", locale_hint)?;
        }
        if self.detected_languages.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_request.detected_languages",
                reason: "must not be empty",
            });
        }
        if self.detected_languages.len() > self.envelope.max_segments as usize {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_request.detected_languages",
                reason: "must be <= envelope.max_segments",
            });
        }
        let mut language_set: BTreeSet<&str> = BTreeSet::new();
        for language_tag in &self.detected_languages {
            validate_language_tag(
                "lang_segment_response_map_request.detected_languages",
                language_tag,
            )?;
            if !language_set.insert(language_tag.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_segment_response_map_request.detected_languages",
                    reason: "language tags must be unique",
                });
            }
        }
        validate_segments_against_transcript(
            "lang_segment_response_map_request.segment_spans",
            &self.segment_spans,
            &self.transcript_text,
            self.envelope.max_segments as usize,
        )?;
        if self.user_language_preferences.len() > self.envelope.max_segments as usize {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_request.user_language_preferences",
                reason: "must be <= envelope.max_segments",
            });
        }
        for language_tag in &self.user_language_preferences {
            validate_language_tag(
                "lang_segment_response_map_request.user_language_preferences",
                language_tag,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LangRequest {
    LangMultipleDetect(LangMultipleDetectRequest),
    LangSegmentResponseMap(LangSegmentResponseMapRequest),
}

impl Validate for Ph1LangRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LangRequest::LangMultipleDetect(r) => r.validate(),
            Ph1LangRequest::LangSegmentResponseMap(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangMultipleDetectOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LangCapabilityId,
    pub reason_code: ReasonCodeId,
    pub detected_languages: Vec<String>,
    pub segment_spans: Vec<LangSegment>,
    pub dominant_language_tag: String,
    pub no_translation_performed: bool,
}

impl LangMultipleDetectOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        detected_languages: Vec<String>,
        segment_spans: Vec<LangSegment>,
        dominant_language_tag: String,
        no_translation_performed: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            capability_id: LangCapabilityId::LangMultipleDetect,
            reason_code,
            detected_languages,
            segment_spans,
            dominant_language_tag,
            no_translation_performed,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for LangMultipleDetectOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LangCapabilityId::LangMultipleDetect {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.capability_id",
                reason: "must be LANG_MULTIPLE_DETECT",
            });
        }
        if self.detected_languages.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.detected_languages",
                reason: "must not be empty",
            });
        }
        let mut language_set: BTreeSet<&str> = BTreeSet::new();
        for language_tag in &self.detected_languages {
            validate_language_tag("lang_multiple_detect_ok.detected_languages", language_tag)?;
            if !language_set.insert(language_tag.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_multiple_detect_ok.detected_languages",
                    reason: "language tags must be unique",
                });
            }
        }
        if self.segment_spans.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.segment_spans",
                reason: "must not be empty",
            });
        }
        if self.segment_spans.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.segment_spans",
                reason: "must be <= 16",
            });
        }
        for segment in &self.segment_spans {
            segment.validate()?;
        }
        validate_language_tag(
            "lang_multiple_detect_ok.dominant_language_tag",
            &self.dominant_language_tag,
        )?;
        if !self.no_translation_performed {
            return Err(ContractViolation::InvalidValue {
                field: "lang_multiple_detect_ok.no_translation_performed",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangSegmentResponseMapOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LangCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: LangValidationStatus,
    pub response_language_plan: Vec<LangResponsePlanItem>,
    pub default_response_language: String,
    pub diagnostics: Vec<String>,
    pub no_translation_performed: bool,
}

impl LangSegmentResponseMapOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: LangValidationStatus,
        response_language_plan: Vec<LangResponsePlanItem>,
        default_response_language: String,
        diagnostics: Vec<String>,
        no_translation_performed: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            capability_id: LangCapabilityId::LangSegmentResponseMap,
            reason_code,
            validation_status,
            response_language_plan,
            default_response_language,
            diagnostics,
            no_translation_performed,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for LangSegmentResponseMapOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LangCapabilityId::LangSegmentResponseMap {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.capability_id",
                reason: "must be LANG_SEGMENT_RESPONSE_MAP",
            });
        }
        if self.response_language_plan.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.response_language_plan",
                reason: "must not be empty",
            });
        }
        if self.response_language_plan.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.response_language_plan",
                reason: "must be <= 16",
            });
        }
        for plan_item in &self.response_language_plan {
            plan_item.validate()?;
        }
        validate_language_tag(
            "lang_segment_response_map_ok.default_response_language",
            &self.default_response_language,
        )?;
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("lang_segment_response_map_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == LangValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_translation_performed {
            return Err(ContractViolation::InvalidValue {
                field: "lang_segment_response_map_ok.no_translation_performed",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: LangCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl LangRefuse {
    pub fn v1(
        capability_id: LangCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LANG_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LangRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LANG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lang_refuse.schema_version",
                reason: "must match PH1LANG_CONTRACT_VERSION",
            });
        }
        validate_text("lang_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LangResponse {
    LangMultipleDetectOk(LangMultipleDetectOk),
    LangSegmentResponseMapOk(LangSegmentResponseMapOk),
    Refuse(LangRefuse),
}

impl Validate for Ph1LangResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LangResponse::LangMultipleDetectOk(o) => o.validate(),
            Ph1LangResponse::LangSegmentResponseMapOk(o) => o.validate(),
            Ph1LangResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_segments_against_transcript(
    field: &'static str,
    segment_spans: &[LangSegment],
    transcript_text: &str,
    max_segments: usize,
) -> Result<(), ContractViolation> {
    if segment_spans.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if segment_spans.len() > max_segments {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= envelope.max_segments",
        });
    }

    let mut segment_ids: BTreeSet<&str> = BTreeSet::new();
    let mut previous_end: u32 = 0;
    let transcript_len = transcript_text.len() as u32;
    for segment in segment_spans {
        segment.validate()?;
        if !segment_ids.insert(segment.segment_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "segment_id entries must be unique",
            });
        }
        if segment.end_byte > transcript_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "segment end exceeds transcript length",
            });
        }
        if segment.start_byte < previous_end {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "segments must be non-overlapping and ordered",
            });
        }
        if !transcript_text.is_char_boundary(segment.start_byte as usize)
            || !transcript_text.is_char_boundary(segment.end_byte as usize)
        {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "segment boundaries must be UTF-8 char boundaries",
            });
        }
        previous_end = segment.end_byte;
    }
    Ok(())
}

fn validate_locale_hint(field: &'static str, locale_hint: &str) -> Result<(), ContractViolation> {
    if locale_hint.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if locale_hint.len() > 32 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 32 chars",
        });
    }
    if !locale_hint
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric and hyphen",
        });
    }
    Ok(())
}

fn validate_language_tag(field: &'static str, language_tag: &str) -> Result<(), ContractViolation> {
    if language_tag.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if language_tag.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 16 chars",
        });
    }
    if !language_tag
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric and hyphen",
        });
    }
    Ok(())
}

fn validate_text(field: &'static str, text: &str, max_len: usize) -> Result<(), ContractViolation> {
    if text.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if text.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if text.chars().any(|character| character.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_segments: u8) -> LangRequestEnvelope {
        LangRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_segments).unwrap()
    }

    #[test]
    fn lang_multiple_detect_request_rejects_empty_transcript() {
        let req = LangMultipleDetectRequest::v1(
            envelope(4),
            "".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Voice,
        );
        assert!(req.is_err());
    }

    #[test]
    fn lang_segment_response_map_request_rejects_overlapping_segments() {
        let segments = vec![
            LangSegment::v1(
                "seg_0".to_string(),
                0,
                5,
                "en".to_string(),
                "hello".to_string(),
            )
            .unwrap(),
            LangSegment::v1(
                "seg_1".to_string(),
                4,
                9,
                "en".to_string(),
                "there".to_string(),
            )
            .unwrap(),
        ];
        let req = LangSegmentResponseMapRequest::v1(
            envelope(4),
            "hello there".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Voice,
            vec!["en".to_string()],
            segments,
            vec!["en".to_string()],
            LangResponseMode::Text,
        );
        assert!(req.is_err());
    }

    #[test]
    fn lang_segment_response_map_ok_requires_diagnostics_when_fail() {
        let out = LangSegmentResponseMapOk::v1(
            ReasonCodeId(1),
            LangValidationStatus::Fail,
            vec![LangResponsePlanItem::v1(LangPlanScope::Turn, None, "en".to_string()).unwrap()],
            "en".to_string(),
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn lang_multiple_detect_ok_requires_no_translation_performed_true() {
        let out = LangMultipleDetectOk::v1(
            ReasonCodeId(1),
            vec!["en".to_string()],
            vec![LangSegment::v1(
                "seg_0".to_string(),
                0,
                5,
                "en".to_string(),
                "hello".to_string(),
            )
            .unwrap()],
            "en".to_string(),
            false,
        );
        assert!(out.is_err());
    }
}
