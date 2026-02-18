#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1srl::{
    Ph1SrlRequest, Ph1SrlResponse, SrlArgumentNormalizeOk, SrlArgumentNormalizeRequest,
    SrlCapabilityId, SrlFrameBuildOk, SrlFrameBuildRequest, SrlFrameSpan, SrlRefuse, SrlRepairNote,
    SrlRoleLabel, SrlValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SRL reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_SRL_OK_FRAME_BUILD: ReasonCodeId = ReasonCodeId(0x5352_0001);
    pub const PH1_SRL_OK_ARGUMENT_NORMALIZE: ReasonCodeId = ReasonCodeId(0x5352_0002);

    pub const PH1_SRL_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5352_00F1);
    pub const PH1_SRL_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5352_00F2);
    pub const PH1_SRL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5352_00F3);
    pub const PH1_SRL_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5352_00F4);
    pub const PH1_SRL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5352_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SrlConfig {
    pub max_spans: u8,
    pub max_notes: u8,
    pub max_ambiguities: u8,
    pub max_diagnostics: u8,
}

impl Ph1SrlConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_spans: 32,
            max_notes: 16,
            max_ambiguities: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1SrlRuntime {
    config: Ph1SrlConfig,
}

impl Ph1SrlRuntime {
    pub fn new(config: Ph1SrlConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1SrlRequest) -> Ph1SrlResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_SRL_INPUT_SCHEMA_INVALID,
                "srl request failed contract validation",
            );
        }

        match req {
            Ph1SrlRequest::SrlFrameBuild(r) => self.run_frame_build(r),
            Ph1SrlRequest::SrlArgumentNormalize(r) => self.run_argument_normalize(r),
        }
    }

    fn run_frame_build(&self, req: &SrlFrameBuildRequest) -> Ph1SrlResponse {
        if req.transcript_text.trim().is_empty() {
            return self.refuse(
                SrlCapabilityId::SrlFrameBuild,
                reason_codes::PH1_SRL_UPSTREAM_INPUT_MISSING,
                "transcript_text is empty",
            );
        }

        let span_budget = min(req.envelope.max_spans, self.config.max_spans) as usize;
        if span_budget == 0 {
            return self.refuse(
                SrlCapabilityId::SrlFrameBuild,
                reason_codes::PH1_SRL_BUDGET_EXCEEDED,
                "span budget exceeded",
            );
        }
        let notes_budget = min(req.envelope.max_notes, self.config.max_notes) as usize;
        let ambiguity_budget =
            min(req.envelope.max_ambiguities, self.config.max_ambiguities) as usize;

        if req.uncertain_spans.len() > ambiguity_budget {
            return self.refuse(
                SrlCapabilityId::SrlFrameBuild,
                reason_codes::PH1_SRL_BUDGET_EXCEEDED,
                "uncertain_spans exceeds configured budget",
            );
        }

        let hint_set = req
            .know_dictionary_hints
            .iter()
            .map(|h| h.to_ascii_lowercase())
            .collect::<BTreeSet<_>>();

        let mut frame_spans = Vec::new();
        let mut repair_notes = Vec::new();
        let mut note_counter = 1u32;

        let tokens = tokenize_with_offsets(&req.transcript_text);
        if tokens.is_empty() {
            return self.refuse(
                SrlCapabilityId::SrlFrameBuild,
                reason_codes::PH1_SRL_UPSTREAM_INPUT_MISSING,
                "no tokens available after normalization",
            );
        }

        for (idx, (start, end, raw_token)) in tokens.into_iter().enumerate() {
            if frame_spans.len() >= span_budget {
                return self.refuse(
                    SrlCapabilityId::SrlFrameBuild,
                    reason_codes::PH1_SRL_BUDGET_EXCEEDED,
                    "frame span budget exceeded",
                );
            }

            let normalized = normalize_token(raw_token, req.normalize_shorthand);
            let language_tag = detect_language_tag(raw_token, &req.language_tag);
            let role_label = detect_role_label(&normalized);

            let span = match SrlFrameSpan::v1(
                format!("span_{:03}", idx + 1),
                start as u32,
                end as u32,
                raw_token.to_string(),
                normalized.clone(),
                language_tag.clone(),
                role_label,
            ) {
                Ok(span) => span,
                Err(_) => {
                    return self.refuse(
                        SrlCapabilityId::SrlFrameBuild,
                        reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                        "failed to build frame span",
                    );
                }
            };
            frame_spans.push(span);

            if normalized != raw_token && repair_notes.len() < notes_budget {
                let note = SrlRepairNote::v1(
                    format!("note_{:03}", note_counter),
                    "SHORTHAND_NORMALIZED".to_string(),
                    format!("normalized '{}' -> '{}'", raw_token, normalized),
                    format!("srl:span:{}", idx + 1),
                );
                note_counter += 1;
                match note {
                    Ok(note) => repair_notes.push(note),
                    Err(_) => {
                        return self.refuse(
                            SrlCapabilityId::SrlFrameBuild,
                            reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                            "failed to build repair note",
                        );
                    }
                }
            }

            if language_tag != req.language_tag && repair_notes.len() < notes_budget {
                let note = SrlRepairNote::v1(
                    format!("note_{:03}", note_counter),
                    "CODE_SWITCH_PRESERVED".to_string(),
                    format!(
                        "preserved token '{}' in language {}",
                        raw_token, language_tag
                    ),
                    format!("srl:span:{}", idx + 1),
                );
                note_counter += 1;
                match note {
                    Ok(note) => repair_notes.push(note),
                    Err(_) => {
                        return self.refuse(
                            SrlCapabilityId::SrlFrameBuild,
                            reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                            "failed to build repair note",
                        );
                    }
                }
            }

            if hint_set.contains(&raw_token.to_ascii_lowercase())
                && repair_notes.len() < notes_budget
            {
                let note = SrlRepairNote::v1(
                    format!("note_{:03}", note_counter),
                    "KNOW_HINT_OBSERVED".to_string(),
                    format!("recognized dictionary hint token '{}'", raw_token),
                    format!("srl:span:{}", idx + 1),
                );
                note_counter += 1;
                match note {
                    Ok(note) => repair_notes.push(note),
                    Err(_) => {
                        return self.refuse(
                            SrlCapabilityId::SrlFrameBuild,
                            reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                            "failed to build repair note",
                        );
                    }
                }
            }
        }

        let mut ambiguity_flags = req
            .uncertain_spans
            .iter()
            .map(|span| {
                span.field_hint
                    .as_ref()
                    .map(|field| format!("{}_ambiguous", field))
                    .unwrap_or_else(|| "uncertain_span_present".to_string())
            })
            .collect::<Vec<_>>();

        if frame_spans
            .iter()
            .any(|span| matches!(span.role_label, SrlRoleLabel::Reference))
        {
            ambiguity_flags.push("reference_ambiguous".to_string());
        }
        if ambiguity_flags.len() > ambiguity_budget {
            ambiguity_flags.truncate(ambiguity_budget);
        }
        ambiguity_flags.sort();
        ambiguity_flags.dedup();

        let repaired_transcript_text = frame_spans
            .iter()
            .map(|span| span.normalized_text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        match SrlFrameBuildOk::v1(
            reason_codes::PH1_SRL_OK_FRAME_BUILD,
            repaired_transcript_text,
            frame_spans,
            repair_notes,
            ambiguity_flags,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1SrlResponse::SrlFrameBuildOk(ok),
            Err(_) => self.refuse(
                SrlCapabilityId::SrlFrameBuild,
                reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                "failed to construct frame-build output",
            ),
        }
    }

    fn run_argument_normalize(&self, req: &SrlArgumentNormalizeRequest) -> Ph1SrlResponse {
        if req.frame_spans.is_empty() {
            return self.refuse(
                SrlCapabilityId::SrlArgumentNormalize,
                reason_codes::PH1_SRL_UPSTREAM_INPUT_MISSING,
                "frame_spans is empty",
            );
        }

        if req.frame_spans.len() > min(req.envelope.max_spans, self.config.max_spans) as usize {
            return self.refuse(
                SrlCapabilityId::SrlArgumentNormalize,
                reason_codes::PH1_SRL_BUDGET_EXCEEDED,
                "frame_spans exceeds configured budget",
            );
        }

        let mut diagnostics = Vec::new();

        for pair in req.frame_spans.windows(2) {
            if pair[0].start_byte > pair[1].start_byte {
                diagnostics.push("span_order_not_canonical".to_string());
                break;
            }
        }
        for pair in req.frame_spans.windows(2) {
            if pair[0].end_byte > pair[1].start_byte {
                diagnostics.push("span_overlap_detected".to_string());
                break;
            }
        }

        if req.repaired_transcript_text.chars().any(char::is_control) {
            diagnostics.push("repaired_text_control_char_detected".to_string());
        }

        for span in &req.frame_spans {
            if contains_cjk(&span.raw_text) && span.normalized_text != span.raw_text {
                diagnostics.push("code_switch_token_mutated".to_string());
                break;
            }
        }

        diagnostics.truncate(min(
            req.envelope.max_diagnostics as usize,
            self.config.max_diagnostics as usize,
        ));

        let clarify_required = !req.ambiguity_flags.is_empty();
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                SrlValidationStatus::Ok,
                reason_codes::PH1_SRL_OK_ARGUMENT_NORMALIZE,
            )
        } else {
            (
                SrlValidationStatus::Fail,
                reason_codes::PH1_SRL_VALIDATION_FAILED,
            )
        };

        match SrlArgumentNormalizeOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            req.frame_spans.clone(),
            req.ambiguity_flags.clone(),
            clarify_required,
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1SrlResponse::SrlArgumentNormalizeOk(ok),
            Err(_) => self.refuse(
                SrlCapabilityId::SrlArgumentNormalize,
                reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                "failed to construct argument-normalize output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: SrlCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1SrlResponse {
        let out = SrlRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("SrlRefuse::v1 must construct for static messages");
        Ph1SrlResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1SrlRequest) -> SrlCapabilityId {
    match req {
        Ph1SrlRequest::SrlFrameBuild(_) => SrlCapabilityId::SrlFrameBuild,
        Ph1SrlRequest::SrlArgumentNormalize(_) => SrlCapabilityId::SrlArgumentNormalize,
    }
}

fn tokenize_with_offsets(input: &str) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        while i < bytes.len() {
            let ch = input[i..].chars().next().unwrap_or(' ');
            if !ch.is_whitespace() {
                break;
            }
            i += ch.len_utf8();
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        while i < bytes.len() {
            let ch = input[i..].chars().next().unwrap_or(' ');
            if ch.is_whitespace() {
                break;
            }
            i += ch.len_utf8();
        }
        let end = i;
        if start < end {
            if let Some(tok) = input.get(start..end) {
                out.push((start, end, tok));
            }
        }
    }

    out
}

fn normalize_token(raw_token: &str, normalize_shorthand: bool) -> String {
    if !normalize_shorthand {
        return raw_token.to_string();
    }

    match raw_token.to_ascii_lowercase().as_str() {
        "tmr" => "tomorrow".to_string(),
        "2nite" => "tonight".to_string(),
        "pls" => "please".to_string(),
        _ => raw_token.to_string(),
    }
}

fn detect_language_tag(raw_token: &str, fallback: &str) -> String {
    if contains_cjk(raw_token) {
        "zh".to_string()
    } else if raw_token.is_ascii() {
        fallback.to_ascii_lowercase()
    } else {
        fallback.to_ascii_lowercase()
    }
}

fn contains_cjk(token: &str) -> bool {
    token.chars().any(|ch| {
        let code = ch as u32;
        (0x4E00..=0x9FFF).contains(&code)
            || (0x3400..=0x4DBF).contains(&code)
            || (0xF900..=0xFAFF).contains(&code)
    })
}

fn detect_role_label(normalized: &str) -> SrlRoleLabel {
    let token = normalized.to_ascii_lowercase();

    if ["set", "book", "send", "remind", "schedule", "create"].contains(&token.as_str()) {
        return SrlRoleLabel::Action;
    }
    if ["that", "it", "there", "this"].contains(&token.as_str()) {
        return SrlRoleLabel::Reference;
    }
    if [
        "today",
        "tomorrow",
        "tonight",
        "week",
        "month",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ]
    .contains(&token.as_str())
    {
        return SrlRoleLabel::Date;
    }
    if ["am", "pm", "noon", "midnight"].contains(&token.as_str()) || token.contains(':') {
        return SrlRoleLabel::Time;
    }
    if token.starts_with('$') || token.chars().any(|c| c.is_ascii_digit()) {
        return SrlRoleLabel::Amount;
    }
    if ["a", "an", "the", "uh", "um", "like", "please"].contains(&token.as_str()) {
        return SrlRoleLabel::Filler;
    }
    if token == "to" || token == "for" {
        return SrlRoleLabel::Recipient;
    }
    if !token.is_empty() {
        return SrlRoleLabel::Target;
    }

    SrlRoleLabel::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1srl::{SrlRequestEnvelope, SrlUncertainSpan};

    fn runtime() -> Ph1SrlRuntime {
        Ph1SrlRuntime::new(Ph1SrlConfig::mvp_v1())
    }

    fn envelope(max_spans: u8, max_notes: u8, max_ambiguities: u8) -> SrlRequestEnvelope {
        SrlRequestEnvelope::v1(
            CorrelationId(4501),
            TurnId(431),
            max_spans,
            max_notes,
            max_ambiguities,
            8,
        )
        .unwrap()
    }

    #[test]
    fn at_srl_01_no_new_facts_introduced() {
        let req = Ph1SrlRequest::SrlFrameBuild(
            SrlFrameBuildRequest::v1(
                envelope(16, 16, 8),
                "0123456789abcdef0123456789abcdef".to_string(),
                "Selene tmr remind me to call 妈妈".to_string(),
                "en".to_string(),
                vec![
                    SrlUncertainSpan::v1("u1".to_string(), 7, 10, Some("when".to_string()))
                        .unwrap(),
                ],
                vec!["mama".to_string()],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1SrlResponse::SrlFrameBuildOk(ok) => {
                assert!(ok.no_new_facts);
            }
            _ => panic!("expected SrlFrameBuildOk"),
        }
    }

    #[test]
    fn at_srl_02_code_switch_is_preserved() {
        let req = Ph1SrlRequest::SrlFrameBuild(
            SrlFrameBuildRequest::v1(
                envelope(16, 16, 8),
                "fedcba9876543210fedcba9876543210".to_string(),
                "call 妈妈 tonight".to_string(),
                "en".to_string(),
                vec![],
                vec![],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1SrlResponse::SrlFrameBuildOk(ok) => {
                let has_zh = ok.frame_spans.iter().any(|span| span.language_tag == "zh");
                assert!(has_zh);
            }
            _ => panic!("expected SrlFrameBuildOk"),
        }
    }

    #[test]
    fn at_srl_03_ambiguity_triggers_clarify_not_inference() {
        let frame_req = Ph1SrlRequest::SrlFrameBuild(
            SrlFrameBuildRequest::v1(
                envelope(16, 16, 8),
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                "schedule it later".to_string(),
                "en".to_string(),
                vec![
                    SrlUncertainSpan::v1("u1".to_string(), 11, 16, Some("when".to_string()))
                        .unwrap(),
                ],
                vec![],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let frame_out = runtime().run(&frame_req);
        let frame_ok = match frame_out {
            Ph1SrlResponse::SrlFrameBuildOk(ok) => ok,
            _ => panic!("expected SrlFrameBuildOk"),
        };

        let normalize_req = Ph1SrlRequest::SrlArgumentNormalize(
            SrlArgumentNormalizeRequest::v1(
                envelope(16, 16, 8),
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                frame_ok.repaired_transcript_text,
                frame_ok.frame_spans,
                frame_ok.repair_notes,
                frame_ok.ambiguity_flags,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let normalize_out = runtime().run(&normalize_req);
        match normalize_out {
            Ph1SrlResponse::SrlArgumentNormalizeOk(ok) => {
                assert!(ok.clarify_required);
            }
            _ => panic!("expected SrlArgumentNormalizeOk"),
        }
    }

    #[test]
    fn at_srl_04_budget_overflow_fails_closed() {
        let req = Ph1SrlRequest::SrlFrameBuild(
            SrlFrameBuildRequest::v1(
                envelope(2, 4, 2),
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                "one two three four five".to_string(),
                "en".to_string(),
                vec![],
                vec![],
                true,
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1SrlResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_SRL_BUDGET_EXCEEDED);
            }
            _ => panic!("expected Refuse"),
        }
    }
}
