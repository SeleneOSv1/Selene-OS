#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1pron::{
    Ph1PronRequest, Ph1PronResponse, PronApplyValidateOk, PronApplyValidateRequest,
    PronCapabilityId, PronLexiconEntry, PronLexiconPackBuildOk, PronLexiconPackBuildRequest,
    PronRefuse, PronRequestEnvelope, PronScope, PronTargetEngine, PronValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PRON OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PRON_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_5001);
    pub const PH1_PRON_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_50F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PronWiringConfig {
    pub pron_enabled: bool,
    pub max_entries: u8,
}

impl Ph1PronWiringConfig {
    pub fn mvp_v1(pron_enabled: bool) -> Self {
        Self {
            pron_enabled,
            max_entries: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub scope: PronScope,
    pub consent_asserted: bool,
    pub locale_tag: String,
    pub target_engine: PronTargetEngine,
    pub source_entries: Vec<PronLexiconEntry>,
}

impl PronTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        user_id: Option<String>,
        scope: PronScope,
        consent_asserted: bool,
        locale_tag: String,
        target_engine: PronTargetEngine,
        source_entries: Vec<PronLexiconEntry>,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            tenant_id,
            user_id,
            scope,
            consent_asserted,
            locale_tag,
            target_engine,
            source_entries,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for PronTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;

        validate_ascii_token("pron_turn_input.tenant_id", &self.tenant_id, 64)?;
        validate_locale_tag("pron_turn_input.locale_tag", &self.locale_tag)?;

        if self.scope == PronScope::User {
            let user_id = self
                .user_id
                .as_ref()
                .ok_or(ContractViolation::InvalidValue {
                    field: "pron_turn_input.user_id",
                    reason: "must be present when scope=USER",
                })?;
            validate_ascii_token("pron_turn_input.user_id", user_id, 64)?;
        }

        if self.source_entries.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pron_turn_input.source_entries",
                reason: "must be <= 64",
            });
        }
        for entry in &self.source_entries {
            entry.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub pack_build: PronLexiconPackBuildOk,
    pub apply_validate: PronApplyValidateOk,
}

impl PronForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        pack_build: PronLexiconPackBuildOk,
        apply_validate: PronApplyValidateOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            pack_build,
            apply_validate,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for PronForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.pack_build.validate()?;
        self.apply_validate.validate()?;

        if self.apply_validate.validation_status != PronValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "pron_forward_bundle.apply_validate.validation_status",
                reason: "must be OK",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PronWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoPronInput,
    Refused(PronRefuse),
    Forwarded(PronForwardBundle),
}

pub trait Ph1PronEngine {
    fn run(&self, req: &Ph1PronRequest) -> Ph1PronResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PronWiring<E>
where
    E: Ph1PronEngine,
{
    config: Ph1PronWiringConfig,
    engine: E,
}

impl<E> Ph1PronWiring<E>
where
    E: Ph1PronEngine,
{
    pub fn new(config: Ph1PronWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_entries == 0 || config.max_entries > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pron_wiring_config.max_entries",
                reason: "must be within 1..=64",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &PronTurnInput) -> Result<PronWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.pron_enabled {
            return Ok(PronWiringOutcome::NotInvokedDisabled);
        }
        if input.source_entries.is_empty() {
            return Ok(PronWiringOutcome::NotInvokedNoPronInput);
        }

        let envelope = PronRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_entries, 64),
        )?;

        let build_req = Ph1PronRequest::PronLexiconPackBuild(PronLexiconPackBuildRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.user_id.clone(),
            input.scope,
            input.consent_asserted,
            input.source_entries.clone(),
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1PronResponse::Refuse(r) => return Ok(PronWiringOutcome::Refused(r)),
            Ph1PronResponse::PronLexiconPackBuildOk(ok) => ok,
            Ph1PronResponse::PronApplyValidateOk(_) => {
                return Ok(PronWiringOutcome::Refused(PronRefuse::v1(
                    PronCapabilityId::PronLexiconPackBuild,
                    reason_codes::PH1_PRON_INTERNAL_PIPELINE_ERROR,
                    "unexpected apply-validate response for pack-build request".to_string(),
                )?))
            }
        };

        if !build_ok.target_engines.contains(&input.target_engine) {
            return Ok(PronWiringOutcome::Refused(PronRefuse::v1(
                PronCapabilityId::PronApplyValidate,
                reason_codes::PH1_PRON_INTERNAL_PIPELINE_ERROR,
                "target engine missing from built pronunciation pack".to_string(),
            )?));
        }

        let validate_req = Ph1PronRequest::PronApplyValidate(PronApplyValidateRequest::v1(
            envelope,
            build_ok.pack_id.clone(),
            input.target_engine,
            input.locale_tag.clone(),
            build_ok.entries.clone(),
        )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1PronResponse::Refuse(r) => return Ok(PronWiringOutcome::Refused(r)),
            Ph1PronResponse::PronApplyValidateOk(ok) => ok,
            Ph1PronResponse::PronLexiconPackBuildOk(_) => {
                return Ok(PronWiringOutcome::Refused(PronRefuse::v1(
                    PronCapabilityId::PronApplyValidate,
                    reason_codes::PH1_PRON_INTERNAL_PIPELINE_ERROR,
                    "unexpected pack-build response for apply-validate request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != PronValidationStatus::Ok {
            return Ok(PronWiringOutcome::Refused(PronRefuse::v1(
                PronCapabilityId::PronApplyValidate,
                reason_codes::PH1_PRON_VALIDATION_FAILED,
                "pronunciation apply validation failed".to_string(),
            )?));
        }

        let bundle =
            PronForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(PronWiringOutcome::Forwarded(bundle))
    }
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
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
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
    use selene_kernel_contracts::ph1pron::{PronApplyValidateOk, PronLexiconPackBuildOk};
    use selene_kernel_contracts::ReasonCodeId;

    fn entry(id: &str, grapheme: &str, phoneme: &str, locale: &str) -> PronLexiconEntry {
        PronLexiconEntry::v1(
            id.to_string(),
            grapheme.to_string(),
            phoneme.to_string(),
            locale.to_string(),
        )
        .unwrap()
    }

    struct DeterministicPronEngine;

    impl Ph1PronEngine for DeterministicPronEngine {
        fn run(&self, req: &Ph1PronRequest) -> Ph1PronResponse {
            match req {
                Ph1PronRequest::PronLexiconPackBuild(r) => Ph1PronResponse::PronLexiconPackBuildOk(
                    PronLexiconPackBuildOk::v1(
                        ReasonCodeId(1),
                        "pron.pack.tenant_a.tenant.none.01020304".to_string(),
                        r.scope,
                        vec![
                            PronTargetEngine::Tts,
                            PronTargetEngine::VoiceId,
                            PronTargetEngine::Wake,
                        ],
                        r.entries.clone(),
                        true,
                        r.scope == PronScope::User,
                        true,
                    )
                    .unwrap(),
                ),
                Ph1PronRequest::PronApplyValidate(r) => Ph1PronResponse::PronApplyValidateOk(
                    PronApplyValidateOk::v1(
                        ReasonCodeId(2),
                        PronValidationStatus::Ok,
                        r.pack_id.clone(),
                        r.target_engine,
                        r.locale_tag.clone(),
                        vec![],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftPronEngine;

    impl Ph1PronEngine for DriftPronEngine {
        fn run(&self, req: &Ph1PronRequest) -> Ph1PronResponse {
            match req {
                Ph1PronRequest::PronLexiconPackBuild(r) => Ph1PronResponse::PronLexiconPackBuildOk(
                    PronLexiconPackBuildOk::v1(
                        ReasonCodeId(10),
                        "pron.pack.tenant_a.tenant.none.01020304".to_string(),
                        r.scope,
                        vec![PronTargetEngine::Tts, PronTargetEngine::Wake],
                        r.entries.clone(),
                        true,
                        false,
                        true,
                    )
                    .unwrap(),
                ),
                Ph1PronRequest::PronApplyValidate(r) => Ph1PronResponse::PronApplyValidateOk(
                    PronApplyValidateOk::v1(
                        ReasonCodeId(11),
                        PronValidationStatus::Fail,
                        r.pack_id.clone(),
                        r.target_engine,
                        r.locale_tag.clone(),
                        vec!["entry_0_locale_mismatch".to_string()],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    #[test]
    fn at_pron_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1PronWiring::new(Ph1PronWiringConfig::mvp_v1(true), DeterministicPronEngine).unwrap();

        let input = PronTurnInput::v1(
            CorrelationId(1401),
            TurnId(101),
            "tenant_a".to_string(),
            None,
            PronScope::Tenant,
            false,
            "en".to_string(),
            PronTargetEngine::Tts,
            vec![entry("e1", "selene", "suh-leen", "en")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PronWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.apply_validate.validation_status,
                    PronValidationStatus::Ok
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_pron_02_os_preserves_pack_id_and_target_engine_for_downstream() {
        let wiring =
            Ph1PronWiring::new(Ph1PronWiringConfig::mvp_v1(true), DeterministicPronEngine).unwrap();

        let input = PronTurnInput::v1(
            CorrelationId(1402),
            TurnId(102),
            "tenant_a".to_string(),
            None,
            PronScope::Tenant,
            false,
            "en".to_string(),
            PronTargetEngine::VoiceId,
            vec![entry("e1", "selene", "suh-leen", "en")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PronWiringOutcome::Forwarded(bundle) => {
                assert_eq!(bundle.pack_build.pack_id, bundle.apply_validate.pack_id);
                assert_eq!(
                    bundle.apply_validate.target_engine,
                    PronTargetEngine::VoiceId
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_pron_03_os_does_not_invoke_when_disabled_or_no_input() {
        let disabled =
            Ph1PronWiring::new(Ph1PronWiringConfig::mvp_v1(false), DeterministicPronEngine)
                .unwrap();

        let input = PronTurnInput::v1(
            CorrelationId(1403),
            TurnId(103),
            "tenant_a".to_string(),
            None,
            PronScope::Tenant,
            false,
            "en".to_string(),
            PronTargetEngine::Tts,
            vec![entry("e1", "selene", "suh-leen", "en")],
        )
        .unwrap();
        assert_eq!(
            disabled.run_turn(&input).unwrap(),
            PronWiringOutcome::NotInvokedDisabled
        );

        let enabled =
            Ph1PronWiring::new(Ph1PronWiringConfig::mvp_v1(true), DeterministicPronEngine).unwrap();
        let empty_input = PronTurnInput::v1(
            CorrelationId(1404),
            TurnId(104),
            "tenant_a".to_string(),
            None,
            PronScope::Tenant,
            false,
            "en".to_string(),
            PronTargetEngine::Tts,
            vec![],
        )
        .unwrap();

        assert_eq!(
            enabled.run_turn(&empty_input).unwrap(),
            PronWiringOutcome::NotInvokedNoPronInput
        );
    }

    #[test]
    fn at_pron_04_os_fails_closed_on_validation_drift() {
        let wiring =
            Ph1PronWiring::new(Ph1PronWiringConfig::mvp_v1(true), DriftPronEngine).unwrap();

        let input = PronTurnInput::v1(
            CorrelationId(1405),
            TurnId(105),
            "tenant_a".to_string(),
            None,
            PronScope::Tenant,
            false,
            "en".to_string(),
            PronTargetEngine::Tts,
            vec![entry("e1", "selene", "suh-leen", "en")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PronWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_PRON_VALIDATION_FAILED);
            }
            _ => panic!("expected Refused"),
        }
    }
}
