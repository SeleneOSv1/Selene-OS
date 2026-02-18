#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1tts::StyleProfileRef;
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PERSONA_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaCapabilityId {
    PersonaProfileBuild,
    PersonaProfileValidate,
}

impl PersonaCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PersonaCapabilityId::PersonaProfileBuild => "PERSONA_PROFILE_BUILD",
            PersonaCapabilityId::PersonaProfileValidate => "PERSONA_PROFILE_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaPreferenceKey {
    PreferredLanguage,
    BrevityPreference,
    ResponseToneTarget,
    PrivacyPreference,
    ConfirmationPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaDeliveryPolicyRef {
    VoiceAllowed,
    TextOnly,
    Silent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaBrevityRef {
    Brief,
    Balanced,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signals: u8,
    pub max_diagnostics: u8,
}

impl PersonaRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signals: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signals,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PersonaRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_request_envelope.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signals == 0 || self.max_signals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_request_envelope.max_signals",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaPreferenceSignal {
    pub schema_version: SchemaVersion,
    pub key: PersonaPreferenceKey,
    pub value: String,
    pub evidence_ref: String,
}

impl PersonaPreferenceSignal {
    pub fn v1(
        key: PersonaPreferenceKey,
        value: String,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let signal = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            key,
            value,
            evidence_ref,
        };
        signal.validate()?;
        Ok(signal)
    }
}

impl Validate for PersonaPreferenceSignal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_preference_signal.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        validate_text("persona_preference_signal.value", &self.value, 96)?;
        validate_text(
            "persona_preference_signal.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaProfileSnapshot {
    pub schema_version: SchemaVersion,
    pub style_profile_ref: StyleProfileRef,
    pub delivery_policy_ref: PersonaDeliveryPolicyRef,
    pub brevity_ref: PersonaBrevityRef,
    pub preferences_snapshot_ref: String,
}

impl PersonaProfileSnapshot {
    pub fn v1(
        style_profile_ref: StyleProfileRef,
        delivery_policy_ref: PersonaDeliveryPolicyRef,
        brevity_ref: PersonaBrevityRef,
        preferences_snapshot_ref: String,
    ) -> Result<Self, ContractViolation> {
        let snapshot = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            style_profile_ref,
            delivery_policy_ref,
            brevity_ref,
            preferences_snapshot_ref,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

impl Validate for PersonaProfileSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_snapshot.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        validate_text(
            "persona_profile_snapshot.preferences_snapshot_ref",
            &self.preferences_snapshot_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaProfileBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PersonaRequestEnvelope,
    pub verified_user_id: String,
    pub verified_speaker_id: String,
    pub preference_signals: Vec<PersonaPreferenceSignal>,
    pub correction_event_count: u16,
    pub emo_guide_style_profile_ref: Option<StyleProfileRef>,
    pub previous_snapshot_ref: Option<String>,
}

impl PersonaProfileBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PersonaRequestEnvelope,
        verified_user_id: String,
        verified_speaker_id: String,
        preference_signals: Vec<PersonaPreferenceSignal>,
        correction_event_count: u16,
        emo_guide_style_profile_ref: Option<StyleProfileRef>,
        previous_snapshot_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            envelope,
            verified_user_id,
            verified_speaker_id,
            preference_signals,
            correction_event_count,
            emo_guide_style_profile_ref,
            previous_snapshot_ref,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PersonaProfileBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_build_request.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "persona_profile_build_request.verified_user_id",
            &self.verified_user_id,
            128,
        )?;
        validate_text(
            "persona_profile_build_request.verified_speaker_id",
            &self.verified_speaker_id,
            128,
        )?;
        if self.preference_signals.len() > self.envelope.max_signals as usize {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_build_request.preference_signals",
                reason: "must be <= envelope.max_signals",
            });
        }
        for signal in &self.preference_signals {
            signal.validate()?;
        }
        if self.correction_event_count > 1000 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_build_request.correction_event_count",
                reason: "must be <= 1000",
            });
        }
        if let Some(previous_snapshot_ref) = &self.previous_snapshot_ref {
            validate_text(
                "persona_profile_build_request.previous_snapshot_ref",
                previous_snapshot_ref,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaProfileBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PersonaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub profile_snapshot: PersonaProfileSnapshot,
    pub auditable: bool,
    pub tone_only: bool,
    pub no_meaning_drift: bool,
    pub no_execution_authority: bool,
}

impl PersonaProfileBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        profile_snapshot: PersonaProfileSnapshot,
        auditable: bool,
        tone_only: bool,
        no_meaning_drift: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            capability_id: PersonaCapabilityId::PersonaProfileBuild,
            reason_code,
            profile_snapshot,
            auditable,
            tone_only,
            no_meaning_drift,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for PersonaProfileBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_build_ok.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PersonaCapabilityId::PersonaProfileBuild {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_build_ok.capability_id",
                reason: "must be PERSONA_PROFILE_BUILD",
            });
        }
        self.profile_snapshot.validate()?;
        validate_guard_flags(
            self.auditable,
            self.tone_only,
            self.no_meaning_drift,
            self.no_execution_authority,
            "persona_profile_build_ok",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaProfileValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PersonaRequestEnvelope,
    pub verified_user_id: String,
    pub verified_speaker_id: String,
    pub preference_signals: Vec<PersonaPreferenceSignal>,
    pub correction_event_count: u16,
    pub emo_guide_style_profile_ref: Option<StyleProfileRef>,
    pub previous_snapshot_ref: Option<String>,
    pub proposed_profile_snapshot: PersonaProfileSnapshot,
}

impl PersonaProfileValidateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PersonaRequestEnvelope,
        verified_user_id: String,
        verified_speaker_id: String,
        preference_signals: Vec<PersonaPreferenceSignal>,
        correction_event_count: u16,
        emo_guide_style_profile_ref: Option<StyleProfileRef>,
        previous_snapshot_ref: Option<String>,
        proposed_profile_snapshot: PersonaProfileSnapshot,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            envelope,
            verified_user_id,
            verified_speaker_id,
            preference_signals,
            correction_event_count,
            emo_guide_style_profile_ref,
            previous_snapshot_ref,
            proposed_profile_snapshot,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PersonaProfileValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_request.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "persona_profile_validate_request.verified_user_id",
            &self.verified_user_id,
            128,
        )?;
        validate_text(
            "persona_profile_validate_request.verified_speaker_id",
            &self.verified_speaker_id,
            128,
        )?;
        if self.preference_signals.len() > self.envelope.max_signals as usize {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_request.preference_signals",
                reason: "must be <= envelope.max_signals",
            });
        }
        for signal in &self.preference_signals {
            signal.validate()?;
        }
        if self.correction_event_count > 1000 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_request.correction_event_count",
                reason: "must be <= 1000",
            });
        }
        if let Some(previous_snapshot_ref) = &self.previous_snapshot_ref {
            validate_text(
                "persona_profile_validate_request.previous_snapshot_ref",
                previous_snapshot_ref,
                128,
            )?;
        }
        self.proposed_profile_snapshot.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaProfileValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PersonaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PersonaValidationStatus,
    pub diagnostics: Vec<String>,
    pub auditable: bool,
    pub tone_only: bool,
    pub no_meaning_drift: bool,
    pub no_execution_authority: bool,
}

impl PersonaProfileValidateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PersonaValidationStatus,
        diagnostics: Vec<String>,
        auditable: bool,
        tone_only: bool,
        no_meaning_drift: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            capability_id: PersonaCapabilityId::PersonaProfileValidate,
            reason_code,
            validation_status,
            diagnostics,
            auditable,
            tone_only,
            no_meaning_drift,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for PersonaProfileValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_ok.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PersonaCapabilityId::PersonaProfileValidate {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_ok.capability_id",
                reason: "must be PERSONA_PROFILE_VALIDATE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "persona_profile_validate_ok.diagnostics",
                reason: "must include <= 16 diagnostics",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("persona_profile_validate_ok.diagnostics", diagnostic, 96)?;
        }
        validate_guard_flags(
            self.auditable,
            self.tone_only,
            self.no_meaning_drift,
            self.no_execution_authority,
            "persona_profile_validate_ok",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PersonaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PersonaRefuse {
    pub fn v1(
        capability_id: PersonaCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1PERSONA_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for PersonaRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PERSONA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "persona_refuse.schema_version",
                reason: "must match PH1PERSONA_CONTRACT_VERSION",
            });
        }
        validate_text("persona_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PersonaRequest {
    PersonaProfileBuild(PersonaProfileBuildRequest),
    PersonaProfileValidate(PersonaProfileValidateRequest),
}

impl Validate for Ph1PersonaRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PersonaRequest::PersonaProfileBuild(r) => r.validate(),
            Ph1PersonaRequest::PersonaProfileValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PersonaResponse {
    PersonaProfileBuildOk(PersonaProfileBuildOk),
    PersonaProfileValidateOk(PersonaProfileValidateOk),
    Refuse(PersonaRefuse),
}

impl Validate for Ph1PersonaResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PersonaResponse::PersonaProfileBuildOk(r) => r.validate(),
            Ph1PersonaResponse::PersonaProfileValidateOk(r) => r.validate(),
            Ph1PersonaResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_text(
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_guard_flags(
    auditable: bool,
    tone_only: bool,
    no_meaning_drift: bool,
    no_execution_authority: bool,
    field_prefix: &'static str,
) -> Result<(), ContractViolation> {
    if !auditable {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "auditable must be true",
        });
    }
    if !tone_only {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "tone_only must be true",
        });
    }
    if !no_meaning_drift {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "no_meaning_drift must be true",
        });
    }
    if !no_execution_authority {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "no_execution_authority must be true",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> PersonaRequestEnvelope {
        PersonaRequestEnvelope::v1(CorrelationId(1801), TurnId(121), 8, 8).unwrap()
    }

    #[test]
    fn at_pers_01_profile_build_contract_is_schema_valid() {
        let snapshot = PersonaProfileSnapshot::v1(
            StyleProfileRef::Gentle,
            PersonaDeliveryPolicyRef::VoiceAllowed,
            PersonaBrevityRef::Balanced,
            "ps1:gen:va:bal:en:none:c0".to_string(),
        )
        .unwrap();

        let out = Ph1PersonaResponse::PersonaProfileBuildOk(
            PersonaProfileBuildOk::v1(ReasonCodeId(1), snapshot, true, true, true, true).unwrap(),
        );
        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_pers_02_preference_updates_require_evidence_refs() {
        let signal = PersonaPreferenceSignal::v1(
            PersonaPreferenceKey::BrevityPreference,
            "brief".to_string(),
            " ".to_string(),
        );
        assert!(signal.is_err());
    }

    #[test]
    fn at_pers_03_unknown_speaker_rejected_at_contract_boundary() {
        let req = PersonaProfileBuildRequest::v1(
            envelope(),
            "user_1".to_string(),
            " ".to_string(),
            vec![PersonaPreferenceSignal::v1(
                PersonaPreferenceKey::PreferredLanguage,
                "en".to_string(),
                "ev_1".to_string(),
            )
            .unwrap()],
            0,
            None,
            None,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_pers_04_guard_flags_required_on_validate_output() {
        let out = PersonaProfileValidateOk::v1(
            ReasonCodeId(2),
            PersonaValidationStatus::Ok,
            vec![],
            false,
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }
}
