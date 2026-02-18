#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PRON_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PronCapabilityId {
    PronLexiconPackBuild,
    PronApplyValidate,
}

impl PronCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PronCapabilityId::PronLexiconPackBuild => "PRON_LEXICON_PACK_BUILD",
            PronCapabilityId::PronApplyValidate => "PRON_APPLY_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PronScope {
    Tenant,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PronTargetEngine {
    Tts,
    VoiceId,
    Wake,
}

impl PronTargetEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            PronTargetEngine::Tts => "PH1.TTS",
            PronTargetEngine::VoiceId => "PH1.VOICE.ID",
            PronTargetEngine::Wake => "PH1.W",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PronValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_entries: u8,
}

impl PronRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_entries: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_entries,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PronRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_request_envelope.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_entries == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_request_envelope.max_entries",
                reason: "must be > 0",
            });
        }
        if self.max_entries > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_request_envelope.max_entries",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronLexiconEntry {
    pub schema_version: SchemaVersion,
    pub entry_id: String,
    pub grapheme: String,
    pub phoneme: String,
    pub locale_tag: String,
}

impl PronLexiconEntry {
    pub fn v1(
        entry_id: String,
        grapheme: String,
        phoneme: String,
        locale_tag: String,
    ) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            entry_id,
            grapheme,
            phoneme,
            locale_tag,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for PronLexiconEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_entry.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        validate_text("pron_lexicon_entry.entry_id", &self.entry_id, 64)?;
        validate_text("pron_lexicon_entry.grapheme", &self.grapheme, 96)?;
        validate_text("pron_lexicon_entry.phoneme", &self.phoneme, 96)?;
        validate_locale_tag("pron_lexicon_entry.locale_tag", &self.locale_tag)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronLexiconPackBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PronRequestEnvelope,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub scope: PronScope,
    pub consent_asserted: bool,
    pub entries: Vec<PronLexiconEntry>,
}

impl PronLexiconPackBuildRequest {
    pub fn v1(
        envelope: PronRequestEnvelope,
        tenant_id: String,
        user_id: Option<String>,
        scope: PronScope,
        consent_asserted: bool,
        entries: Vec<PronLexiconEntry>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            envelope,
            tenant_id,
            user_id,
            scope,
            consent_asserted,
            entries,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PronLexiconPackBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_request.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_ascii_token(
            "pron_lexicon_pack_build_request.tenant_id",
            &self.tenant_id,
            64,
        )?;

        match self.scope {
            PronScope::Tenant => {}
            PronScope::User => {
                let user_id = self
                    .user_id
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "pron_lexicon_pack_build_request.user_id",
                        reason: "must be present when scope=USER",
                    })?;
                validate_ascii_token("pron_lexicon_pack_build_request.user_id", user_id, 64)?;
            }
        }

        if self.entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_request.entries",
                reason: "must not be empty",
            });
        }
        if self.entries.len() > self.envelope.max_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_request.entries",
                reason: "must be <= envelope.max_entries",
            });
        }
        for entry in &self.entries {
            entry.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronApplyValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PronRequestEnvelope,
    pub pack_id: String,
    pub target_engine: PronTargetEngine,
    pub locale_tag: String,
    pub entries: Vec<PronLexiconEntry>,
}

impl PronApplyValidateRequest {
    pub fn v1(
        envelope: PronRequestEnvelope,
        pack_id: String,
        target_engine: PronTargetEngine,
        locale_tag: String,
        entries: Vec<PronLexiconEntry>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            envelope,
            pack_id,
            target_engine,
            locale_tag,
            entries,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PronApplyValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_request.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_ascii_token("pron_apply_validate_request.pack_id", &self.pack_id, 128)?;
        validate_locale_tag("pron_apply_validate_request.locale_tag", &self.locale_tag)?;

        if self.entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_request.entries",
                reason: "must not be empty",
            });
        }
        if self.entries.len() > self.envelope.max_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_request.entries",
                reason: "must be <= envelope.max_entries",
            });
        }

        for entry in &self.entries {
            entry.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PronRequest {
    PronLexiconPackBuild(PronLexiconPackBuildRequest),
    PronApplyValidate(PronApplyValidateRequest),
}

impl Validate for Ph1PronRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PronRequest::PronLexiconPackBuild(r) => r.validate(),
            Ph1PronRequest::PronApplyValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronLexiconPackBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PronCapabilityId,
    pub reason_code: ReasonCodeId,
    pub pack_id: String,
    pub scope: PronScope,
    pub target_engines: Vec<PronTargetEngine>,
    pub entries: Vec<PronLexiconEntry>,
    pub tenant_scoped: bool,
    pub user_consent_enforced: bool,
    pub no_meaning_drift: bool,
}

impl PronLexiconPackBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        pack_id: String,
        scope: PronScope,
        target_engines: Vec<PronTargetEngine>,
        entries: Vec<PronLexiconEntry>,
        tenant_scoped: bool,
        user_consent_enforced: bool,
        no_meaning_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            capability_id: PronCapabilityId::PronLexiconPackBuild,
            reason_code,
            pack_id,
            scope,
            target_engines,
            entries,
            tenant_scoped,
            user_consent_enforced,
            no_meaning_drift,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for PronLexiconPackBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PronCapabilityId::PronLexiconPackBuild {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.capability_id",
                reason: "must be PRON_LEXICON_PACK_BUILD",
            });
        }
        validate_ascii_token("pron_lexicon_pack_build_ok.pack_id", &self.pack_id, 128)?;

        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.target_engines",
                reason: "must not be empty",
            });
        }
        if self.target_engines.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.target_engines",
                reason: "must be <= 8",
            });
        }
        let mut engine_set: BTreeSet<&'static str> = BTreeSet::new();
        for engine in &self.target_engines {
            if !engine_set.insert(engine.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pron_lexicon_pack_build_ok.target_engines",
                    reason: "target engines must be unique",
                });
            }
        }

        if self.entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.entries",
                reason: "must not be empty",
            });
        }
        if self.entries.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.entries",
                reason: "must be <= 64",
            });
        }
        for entry in &self.entries {
            entry.validate()?;
        }

        if !self.tenant_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.tenant_scoped",
                reason: "must be true",
            });
        }
        if self.scope == PronScope::User && !self.user_consent_enforced {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.user_consent_enforced",
                reason: "must be true when scope=USER",
            });
        }
        if !self.no_meaning_drift {
            return Err(ContractViolation::InvalidValue {
                field: "pron_lexicon_pack_build_ok.no_meaning_drift",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronApplyValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PronCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PronValidationStatus,
    pub pack_id: String,
    pub target_engine: PronTargetEngine,
    pub locale_tag: String,
    pub diagnostics: Vec<String>,
    pub no_meaning_drift: bool,
}

impl PronApplyValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PronValidationStatus,
        pack_id: String,
        target_engine: PronTargetEngine,
        locale_tag: String,
        diagnostics: Vec<String>,
        no_meaning_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            capability_id: PronCapabilityId::PronApplyValidate,
            reason_code,
            validation_status,
            pack_id,
            target_engine,
            locale_tag,
            diagnostics,
            no_meaning_drift,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for PronApplyValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_ok.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PronCapabilityId::PronApplyValidate {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_ok.capability_id",
                reason: "must be PRON_APPLY_VALIDATE",
            });
        }
        validate_ascii_token("pron_apply_validate_ok.pack_id", &self.pack_id, 128)?;
        validate_locale_tag("pron_apply_validate_ok.locale_tag", &self.locale_tag)?;

        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("pron_apply_validate_ok.diagnostics", diagnostic, 96)?;
        }

        if self.validation_status == PronValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }

        if !self.no_meaning_drift {
            return Err(ContractViolation::InvalidValue {
                field: "pron_apply_validate_ok.no_meaning_drift",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PronCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PronRefuse {
    pub fn v1(
        capability_id: PronCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PRON_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PronRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRON_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pron_refuse.schema_version",
                reason: "must match PH1PRON_CONTRACT_VERSION",
            });
        }
        validate_text("pron_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PronResponse {
    PronLexiconPackBuildOk(PronLexiconPackBuildOk),
    PronApplyValidateOk(PronApplyValidateOk),
    Refuse(PronRefuse),
}

impl Validate for Ph1PronResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PronResponse::PronLexiconPackBuildOk(o) => o.validate(),
            Ph1PronResponse::PronApplyValidateOk(o) => o.validate(),
            Ph1PronResponse::Refuse(r) => r.validate(),
        }
    }
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
    if text.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_ascii_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric, underscore, hyphen, or dot",
        });
    }
    Ok(())
}

fn validate_locale_tag(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 16 chars",
        });
    }
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric and hyphen",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_entries: u8) -> PronRequestEnvelope {
        PronRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_entries).unwrap()
    }

    fn sample_entry() -> PronLexiconEntry {
        PronLexiconEntry::v1(
            "entry_001".to_string(),
            "selene".to_string(),
            "suh-leen".to_string(),
            "en".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn pron_build_request_requires_user_id_for_user_scope() {
        let req = PronLexiconPackBuildRequest::v1(
            envelope(8),
            "tenant_a".to_string(),
            None,
            PronScope::User,
            false,
            vec![sample_entry()],
        );
        assert!(req.is_err());
    }

    #[test]
    fn pron_apply_validate_request_rejects_empty_entries() {
        let req = PronApplyValidateRequest::v1(
            envelope(8),
            "pron.pack".to_string(),
            PronTargetEngine::Tts,
            "en".to_string(),
            vec![],
        );
        assert!(req.is_err());
    }

    #[test]
    fn pron_apply_validate_ok_requires_diagnostics_when_fail() {
        let out = PronApplyValidateOk::v1(
            ReasonCodeId(1),
            PronValidationStatus::Fail,
            "pron.pack".to_string(),
            PronTargetEngine::Tts,
            "en".to_string(),
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn pron_build_ok_requires_no_meaning_drift_true() {
        let out = PronLexiconPackBuildOk::v1(
            ReasonCodeId(1),
            "pron.pack".to_string(),
            PronScope::Tenant,
            vec![PronTargetEngine::Tts],
            vec![sample_entry()],
            true,
            false,
            false,
        );
        assert!(out.is_err());
    }
}
