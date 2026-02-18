#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SRL_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SrlCapabilityId {
    SrlFrameBuild,
    SrlArgumentNormalize,
}

impl SrlCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            SrlCapabilityId::SrlFrameBuild => "SRL_FRAME_BUILD",
            SrlCapabilityId::SrlArgumentNormalize => "SRL_ARGUMENT_NORMALIZE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SrlRoleLabel {
    Action,
    Target,
    Time,
    Date,
    Amount,
    Recipient,
    Reference,
    Filler,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SrlValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_spans: u8,
    pub max_notes: u8,
    pub max_ambiguities: u8,
    pub max_diagnostics: u8,
}

impl SrlRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_spans: u8,
        max_notes: u8,
        max_ambiguities: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_spans,
            max_notes,
            max_ambiguities,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for SrlRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_request_envelope.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_spans == 0 || self.max_spans > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_request_envelope.max_spans",
                reason: "must be within 1..=64",
            });
        }
        if self.max_notes == 0 || self.max_notes > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_request_envelope.max_notes",
                reason: "must be within 1..=64",
            });
        }
        if self.max_ambiguities == 0 || self.max_ambiguities > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_request_envelope.max_ambiguities",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlUncertainSpan {
    pub schema_version: SchemaVersion,
    pub span_id: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub field_hint: Option<String>,
}

impl SrlUncertainSpan {
    pub fn v1(
        span_id: String,
        start_byte: u32,
        end_byte: u32,
        field_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let span = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            span_id,
            start_byte,
            end_byte,
            field_hint,
        };
        span.validate()?;
        Ok(span)
    }
}

impl Validate for SrlUncertainSpan {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_uncertain_span.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        validate_token("srl_uncertain_span.span_id", &self.span_id, 64)?;
        if self.start_byte >= self.end_byte {
            return Err(ContractViolation::InvalidValue {
                field: "srl_uncertain_span.start_byte",
                reason: "must be < end_byte",
            });
        }
        if let Some(field_hint) = &self.field_hint {
            validate_field_key("srl_uncertain_span.field_hint", field_hint)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlFrameSpan {
    pub schema_version: SchemaVersion,
    pub span_id: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub raw_text: String,
    pub normalized_text: String,
    pub language_tag: String,
    pub role_label: SrlRoleLabel,
}

impl SrlFrameSpan {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        span_id: String,
        start_byte: u32,
        end_byte: u32,
        raw_text: String,
        normalized_text: String,
        language_tag: String,
        role_label: SrlRoleLabel,
    ) -> Result<Self, ContractViolation> {
        let span = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            span_id,
            start_byte,
            end_byte,
            raw_text,
            normalized_text,
            language_tag,
            role_label,
        };
        span.validate()?;
        Ok(span)
    }
}

impl Validate for SrlFrameSpan {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_span.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        validate_token("srl_frame_span.span_id", &self.span_id, 64)?;
        if self.start_byte >= self.end_byte {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_span.start_byte",
                reason: "must be < end_byte",
            });
        }
        validate_text("srl_frame_span.raw_text", &self.raw_text, 256)?;
        validate_text("srl_frame_span.normalized_text", &self.normalized_text, 256)?;
        validate_language_tag("srl_frame_span.language_tag", &self.language_tag)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlRepairNote {
    pub schema_version: SchemaVersion,
    pub note_id: String,
    pub note_kind: String,
    pub note_message: String,
    pub evidence_ref: String,
}

impl SrlRepairNote {
    pub fn v1(
        note_id: String,
        note_kind: String,
        note_message: String,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let note = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            note_id,
            note_kind,
            note_message,
            evidence_ref,
        };
        note.validate()?;
        Ok(note)
    }
}

impl Validate for SrlRepairNote {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_repair_note.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        validate_token("srl_repair_note.note_id", &self.note_id, 64)?;
        validate_token("srl_repair_note.note_kind", &self.note_kind, 64)?;
        validate_text("srl_repair_note.note_message", &self.note_message, 256)?;
        validate_token("srl_repair_note.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlFrameBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SrlRequestEnvelope,
    pub transcript_hash: String,
    pub transcript_text: String,
    pub language_tag: String,
    pub uncertain_spans: Vec<SrlUncertainSpan>,
    pub know_dictionary_hints: Vec<String>,
    pub normalize_shorthand: bool,
    pub preserve_code_switch: bool,
    pub no_translate: bool,
}

impl SrlFrameBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: SrlRequestEnvelope,
        transcript_hash: String,
        transcript_text: String,
        language_tag: String,
        uncertain_spans: Vec<SrlUncertainSpan>,
        know_dictionary_hints: Vec<String>,
        normalize_shorthand: bool,
        preserve_code_switch: bool,
        no_translate: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            envelope,
            transcript_hash,
            transcript_text,
            language_tag,
            uncertain_spans,
            know_dictionary_hints,
            normalize_shorthand,
            preserve_code_switch,
            no_translate,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for SrlFrameBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_hash(
            "srl_frame_build_request.transcript_hash",
            &self.transcript_hash,
        )?;
        validate_text(
            "srl_frame_build_request.transcript_text",
            &self.transcript_text,
            4096,
        )?;
        validate_language_tag("srl_frame_build_request.language_tag", &self.language_tag)?;
        if self.uncertain_spans.len() > self.envelope.max_ambiguities as usize {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.uncertain_spans",
                reason: "must be <= envelope.max_ambiguities",
            });
        }
        for span in &self.uncertain_spans {
            span.validate()?;
            if span.end_byte > self.transcript_text.len() as u32 {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_frame_build_request.uncertain_spans",
                    reason: "span end_byte must be <= transcript length",
                });
            }
        }

        if self.know_dictionary_hints.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.know_dictionary_hints",
                reason: "must be <= 64",
            });
        }
        let mut hints = BTreeSet::new();
        for hint in &self.know_dictionary_hints {
            validate_token("srl_frame_build_request.know_dictionary_hints", hint, 96)?;
            if !hints.insert(hint.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_frame_build_request.know_dictionary_hints",
                    reason: "must be unique",
                });
            }
        }
        if !self.normalize_shorthand {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.normalize_shorthand",
                reason: "must be true",
            });
        }
        if !self.preserve_code_switch {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.preserve_code_switch",
                reason: "must be true",
            });
        }
        if !self.no_translate {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_request.no_translate",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlArgumentNormalizeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SrlRequestEnvelope,
    pub transcript_hash: String,
    pub repaired_transcript_text: String,
    pub frame_spans: Vec<SrlFrameSpan>,
    pub repair_notes: Vec<SrlRepairNote>,
    pub ambiguity_flags: Vec<String>,
    pub no_intent_change_required: bool,
    pub no_fact_invention_required: bool,
    pub preserve_code_switch: bool,
    pub no_translate: bool,
}

impl SrlArgumentNormalizeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: SrlRequestEnvelope,
        transcript_hash: String,
        repaired_transcript_text: String,
        frame_spans: Vec<SrlFrameSpan>,
        repair_notes: Vec<SrlRepairNote>,
        ambiguity_flags: Vec<String>,
        no_intent_change_required: bool,
        no_fact_invention_required: bool,
        preserve_code_switch: bool,
        no_translate: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            envelope,
            transcript_hash,
            repaired_transcript_text,
            frame_spans,
            repair_notes,
            ambiguity_flags,
            no_intent_change_required,
            no_fact_invention_required,
            preserve_code_switch,
            no_translate,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for SrlArgumentNormalizeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_hash(
            "srl_argument_normalize_request.transcript_hash",
            &self.transcript_hash,
        )?;
        validate_text(
            "srl_argument_normalize_request.repaired_transcript_text",
            &self.repaired_transcript_text,
            4096,
        )?;

        if self.frame_spans.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.frame_spans",
                reason: "must be non-empty",
            });
        }
        if self.frame_spans.len() > self.envelope.max_spans as usize {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.frame_spans",
                reason: "must be <= envelope.max_spans",
            });
        }

        let mut span_ids = BTreeSet::new();
        for span in &self.frame_spans {
            span.validate()?;
            if !span_ids.insert(span.span_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_argument_normalize_request.frame_spans",
                    reason: "span_id must be unique",
                });
            }
        }

        if self.repair_notes.len() > self.envelope.max_notes as usize {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.repair_notes",
                reason: "must be <= envelope.max_notes",
            });
        }
        let mut note_ids = BTreeSet::new();
        for note in &self.repair_notes {
            note.validate()?;
            if !note_ids.insert(note.note_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_argument_normalize_request.repair_notes",
                    reason: "note_id must be unique",
                });
            }
        }

        if self.ambiguity_flags.len() > self.envelope.max_ambiguities as usize {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.ambiguity_flags",
                reason: "must be <= envelope.max_ambiguities",
            });
        }
        let mut ambiguity_set = BTreeSet::new();
        for flag in &self.ambiguity_flags {
            validate_field_key("srl_argument_normalize_request.ambiguity_flags", flag)?;
            if !ambiguity_set.insert(flag.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_argument_normalize_request.ambiguity_flags",
                    reason: "must be unique",
                });
            }
        }

        if !self.no_intent_change_required {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.no_intent_change_required",
                reason: "must be true",
            });
        }
        if !self.no_fact_invention_required {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.no_fact_invention_required",
                reason: "must be true",
            });
        }
        if !self.preserve_code_switch {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.preserve_code_switch",
                reason: "must be true",
            });
        }
        if !self.no_translate {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_request.no_translate",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SrlRequest {
    SrlFrameBuild(SrlFrameBuildRequest),
    SrlArgumentNormalize(SrlArgumentNormalizeRequest),
}

impl Validate for Ph1SrlRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SrlRequest::SrlFrameBuild(req) => req.validate(),
            Ph1SrlRequest::SrlArgumentNormalize(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlFrameBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SrlCapabilityId,
    pub reason_code: ReasonCodeId,
    pub repaired_transcript_text: String,
    pub frame_spans: Vec<SrlFrameSpan>,
    pub repair_notes: Vec<SrlRepairNote>,
    pub ambiguity_flags: Vec<String>,
    pub preserve_code_switch: bool,
    pub no_new_facts: bool,
    pub no_translation_performed: bool,
}

impl SrlFrameBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        repaired_transcript_text: String,
        frame_spans: Vec<SrlFrameSpan>,
        repair_notes: Vec<SrlRepairNote>,
        ambiguity_flags: Vec<String>,
        preserve_code_switch: bool,
        no_new_facts: bool,
        no_translation_performed: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            capability_id: SrlCapabilityId::SrlFrameBuild,
            reason_code,
            repaired_transcript_text,
            frame_spans,
            repair_notes,
            ambiguity_flags,
            preserve_code_switch,
            no_new_facts,
            no_translation_performed,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SrlFrameBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SrlCapabilityId::SrlFrameBuild {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.capability_id",
                reason: "must be SRL_FRAME_BUILD",
            });
        }
        validate_text(
            "srl_frame_build_ok.repaired_transcript_text",
            &self.repaired_transcript_text,
            4096,
        )?;

        if self.frame_spans.is_empty() || self.frame_spans.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.frame_spans",
                reason: "must be within 1..=64",
            });
        }
        let mut span_ids = BTreeSet::new();
        for span in &self.frame_spans {
            span.validate()?;
            if !span_ids.insert(span.span_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_frame_build_ok.frame_spans",
                    reason: "span_id must be unique",
                });
            }
        }

        if self.repair_notes.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.repair_notes",
                reason: "must be <= 64",
            });
        }
        let mut note_ids = BTreeSet::new();
        for note in &self.repair_notes {
            note.validate()?;
            if !note_ids.insert(note.note_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_frame_build_ok.repair_notes",
                    reason: "note_id must be unique",
                });
            }
        }

        if self.ambiguity_flags.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.ambiguity_flags",
                reason: "must be <= 32",
            });
        }
        let mut ambiguity_set = BTreeSet::new();
        for flag in &self.ambiguity_flags {
            validate_field_key("srl_frame_build_ok.ambiguity_flags", flag)?;
            if !ambiguity_set.insert(flag.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_frame_build_ok.ambiguity_flags",
                    reason: "must be unique",
                });
            }
        }

        if !self.preserve_code_switch {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.preserve_code_switch",
                reason: "must be true",
            });
        }
        if !self.no_new_facts {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.no_new_facts",
                reason: "must be true",
            });
        }
        if !self.no_translation_performed {
            return Err(ContractViolation::InvalidValue {
                field: "srl_frame_build_ok.no_translation_performed",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlArgumentNormalizeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SrlCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: SrlValidationStatus,
    pub diagnostics: Vec<String>,
    pub normalized_frame_spans: Vec<SrlFrameSpan>,
    pub ambiguity_flags: Vec<String>,
    pub clarify_required: bool,
    pub preserve_code_switch: bool,
    pub no_new_facts: bool,
    pub no_translation_performed: bool,
    pub no_intent_change: bool,
}

impl SrlArgumentNormalizeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: SrlValidationStatus,
        diagnostics: Vec<String>,
        normalized_frame_spans: Vec<SrlFrameSpan>,
        ambiguity_flags: Vec<String>,
        clarify_required: bool,
        preserve_code_switch: bool,
        no_new_facts: bool,
        no_translation_performed: bool,
        no_intent_change: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            capability_id: SrlCapabilityId::SrlArgumentNormalize,
            reason_code,
            validation_status,
            diagnostics,
            normalized_frame_spans,
            ambiguity_flags,
            clarify_required,
            preserve_code_switch,
            no_new_facts,
            no_translation_performed,
            no_intent_change,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SrlArgumentNormalizeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SrlCapabilityId::SrlArgumentNormalize {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.capability_id",
                reason: "must be SRL_ARGUMENT_NORMALIZE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("srl_argument_normalize_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == SrlValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }

        if self.normalized_frame_spans.is_empty() || self.normalized_frame_spans.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.normalized_frame_spans",
                reason: "must be within 1..=64",
            });
        }
        let mut span_ids = BTreeSet::new();
        for span in &self.normalized_frame_spans {
            span.validate()?;
            if !span_ids.insert(span.span_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_argument_normalize_ok.normalized_frame_spans",
                    reason: "span_id must be unique",
                });
            }
        }

        if self.ambiguity_flags.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.ambiguity_flags",
                reason: "must be <= 32",
            });
        }
        let mut ambiguity_set = BTreeSet::new();
        for flag in &self.ambiguity_flags {
            validate_field_key("srl_argument_normalize_ok.ambiguity_flags", flag)?;
            if !ambiguity_set.insert(flag.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "srl_argument_normalize_ok.ambiguity_flags",
                    reason: "must be unique",
                });
            }
        }
        if !self.ambiguity_flags.is_empty() && !self.clarify_required {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok.clarify_required",
                reason: "must be true when ambiguity_flags is non-empty",
            });
        }

        if self.validation_status == SrlValidationStatus::Ok
            && (!self.preserve_code_switch
                || !self.no_new_facts
                || !self.no_translation_performed
                || !self.no_intent_change)
        {
            return Err(ContractViolation::InvalidValue {
                field: "srl_argument_normalize_ok",
                reason: "OK status requires all boundary flags true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: SrlCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl SrlRefuse {
    pub fn v1(
        capability_id: SrlCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SRL_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SrlRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SRL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "srl_refuse.schema_version",
                reason: "must match PH1SRL_CONTRACT_VERSION",
            });
        }
        validate_text("srl_refuse.message", &self.message, 192)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SrlResponse {
    SrlFrameBuildOk(SrlFrameBuildOk),
    SrlArgumentNormalizeOk(SrlArgumentNormalizeOk),
    Refuse(SrlRefuse),
}

impl Validate for Ph1SrlResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SrlResponse::SrlFrameBuildOk(out) => out.validate(),
            Ph1SrlResponse::SrlArgumentNormalizeOk(out) => out.validate(),
            Ph1SrlResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_hash(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() < 16 || value.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 16..=128 chars",
        });
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be hex",
        });
    }
    Ok(())
}

fn validate_language_tag(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() || value.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 1..=16 chars",
        });
    }
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric/hyphen",
        });
    }
    Ok(())
}

fn validate_field_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() || value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 1..=64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII snake_case",
        });
    }
    Ok(())
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(char::is_control) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control chars",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> SrlRequestEnvelope {
        SrlRequestEnvelope::v1(CorrelationId(4401), TurnId(421), 16, 12, 8, 8).unwrap()
    }

    #[test]
    fn at_srl_01_frame_build_contract_is_schema_valid() {
        let req = SrlFrameBuildRequest::v1(
            envelope(),
            "0123456789abcdef0123456789abcdef".to_string(),
            "Selene tmr remind me to call 妈妈".to_string(),
            "en".to_string(),
            vec![SrlUncertainSpan::v1("u1".to_string(), 7, 10, Some("when".to_string())).unwrap()],
            vec!["mama".to_string()],
            true,
            true,
            true,
        )
        .unwrap();

        assert!(req.validate().is_ok());

        let span1 = SrlFrameSpan::v1(
            "s1".to_string(),
            0,
            6,
            "Selene".to_string(),
            "Selene".to_string(),
            "en".to_string(),
            SrlRoleLabel::Action,
        )
        .unwrap();
        let span2 = SrlFrameSpan::v1(
            "s2".to_string(),
            27,
            33,
            "妈妈".to_string(),
            "妈妈".to_string(),
            "zh".to_string(),
            SrlRoleLabel::Target,
        )
        .unwrap();

        let out = SrlFrameBuildOk::v1(
            ReasonCodeId(801),
            "Selene tomorrow remind me to call 妈妈".to_string(),
            vec![span1, span2],
            vec![],
            vec!["when_ambiguous".to_string()],
            true,
            true,
            true,
        )
        .unwrap();

        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_srl_02_code_switch_preserved_in_output_contract() {
        let span = SrlFrameSpan::v1(
            "s1".to_string(),
            0,
            6,
            "妈妈".to_string(),
            "妈妈".to_string(),
            "zh".to_string(),
            SrlRoleLabel::Target,
        )
        .unwrap();

        let out = SrlArgumentNormalizeOk::v1(
            ReasonCodeId(802),
            SrlValidationStatus::Ok,
            vec![],
            vec![span],
            vec![],
            false,
            true,
            true,
            true,
            true,
        );

        assert!(out.is_ok());
    }

    #[test]
    fn at_srl_03_ambiguity_requires_clarify() {
        let span = SrlFrameSpan::v1(
            "s1".to_string(),
            0,
            5,
            "later".to_string(),
            "later".to_string(),
            "en".to_string(),
            SrlRoleLabel::Date,
        )
        .unwrap();

        let out = SrlArgumentNormalizeOk::v1(
            ReasonCodeId(803),
            SrlValidationStatus::Ok,
            vec![],
            vec![span],
            vec!["date_ambiguous".to_string()],
            false,
            true,
            true,
            true,
            true,
        );

        assert!(out.is_err());
    }
}
