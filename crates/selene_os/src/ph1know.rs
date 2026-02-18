#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1know::{
    KnowCapabilityId, KnowDictionaryEntry, KnowDictionaryPackBuildOk,
    KnowDictionaryPackBuildRequest, KnowHintBundleSelectOk, KnowHintBundleSelectRequest,
    KnowRefuse, KnowRequestEnvelope, KnowTargetEngine, KnowValidationStatus, Ph1KnowRequest,
    Ph1KnowResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KNOW OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_KNOW_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B4E_0101);
    pub const PH1_KNOW_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B4E_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KnowWiringConfig {
    pub know_enabled: bool,
    pub max_entries: u8,
    pub max_diagnostics: u8,
}

impl Ph1KnowWiringConfig {
    pub fn mvp_v1(know_enabled: bool) -> Self {
        Self {
            know_enabled,
            max_entries: 24,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub entries: Vec<KnowDictionaryEntry>,
    pub user_terms_present: bool,
    pub user_consent_asserted: bool,
    pub hr_org_authorized: bool,
    pub learn_artifact_ref: Option<String>,
    pub requested_targets: Vec<KnowTargetEngine>,
}

impl KnowTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        entries: Vec<KnowDictionaryEntry>,
        user_terms_present: bool,
        user_consent_asserted: bool,
        hr_org_authorized: bool,
        learn_artifact_ref: Option<String>,
        requested_targets: Vec<KnowTargetEngine>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            entries,
            user_terms_present,
            user_consent_asserted,
            hr_org_authorized,
            learn_artifact_ref,
            requested_targets,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for KnowTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("know_turn_input.tenant_id", &self.tenant_id, 64)?;
        if self.entries.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "know_turn_input.entries",
                reason: "must be <= 128",
            });
        }
        for entry in &self.entries {
            entry.validate()?;
        }
        if let Some(learn_artifact_ref) = &self.learn_artifact_ref {
            validate_token(
                "know_turn_input.learn_artifact_ref",
                learn_artifact_ref,
                128,
            )?;
        }
        if self.requested_targets.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "know_turn_input.requested_targets",
                reason: "must be <= 4",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub dictionary_pack_build: KnowDictionaryPackBuildOk,
    pub hint_bundle_select: KnowHintBundleSelectOk,
}

impl KnowForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        dictionary_pack_build: KnowDictionaryPackBuildOk,
        hint_bundle_select: KnowHintBundleSelectOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            dictionary_pack_build,
            hint_bundle_select,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for KnowForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.dictionary_pack_build.validate()?;
        self.hint_bundle_select.validate()?;
        if self.hint_bundle_select.validation_status != KnowValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "know_forward_bundle.hint_bundle_select.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoKnowInput,
    Refused(KnowRefuse),
    Forwarded(KnowForwardBundle),
}

pub trait Ph1KnowEngine {
    fn run(&self, req: &Ph1KnowRequest) -> Ph1KnowResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1KnowWiring<E>
where
    E: Ph1KnowEngine,
{
    config: Ph1KnowWiringConfig,
    engine: E,
}

impl<E> Ph1KnowWiring<E>
where
    E: Ph1KnowEngine,
{
    pub fn new(config: Ph1KnowWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_entries == 0 || config.max_entries > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1know_wiring_config.max_entries",
                reason: "must be within 1..=128",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1know_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &KnowTurnInput) -> Result<KnowWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.know_enabled {
            return Ok(KnowWiringOutcome::NotInvokedDisabled);
        }
        if input.entries.is_empty() {
            return Ok(KnowWiringOutcome::NotInvokedNoKnowInput);
        }

        let envelope = KnowRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_entries, 128),
            min(self.config.max_diagnostics, 16),
        )?;

        let build_req =
            Ph1KnowRequest::KnowDictionaryPackBuild(KnowDictionaryPackBuildRequest::v1(
                envelope.clone(),
                input.tenant_id.clone(),
                input.entries.clone(),
                input.user_terms_present,
                input.user_consent_asserted,
                input.hr_org_authorized,
                input.learn_artifact_ref.clone(),
                true,
                true,
                true,
            )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1KnowResponse::Refuse(refuse) => return Ok(KnowWiringOutcome::Refused(refuse)),
            Ph1KnowResponse::KnowDictionaryPackBuildOk(ok) => ok,
            Ph1KnowResponse::KnowHintBundleSelectOk(_) => {
                return Ok(KnowWiringOutcome::Refused(KnowRefuse::v1(
                    KnowCapabilityId::KnowDictionaryPackBuild,
                    reason_codes::PH1_KNOW_INTERNAL_PIPELINE_ERROR,
                    "unexpected hint-bundle-select response for dictionary-pack-build request"
                        .to_string(),
                )?))
            }
        };

        let target_engines = if input.requested_targets.is_empty() {
            build_ok.target_engines.clone()
        } else {
            input.requested_targets.clone()
        };

        let select_req = Ph1KnowRequest::KnowHintBundleSelect(KnowHintBundleSelectRequest::v1(
            envelope,
            input.tenant_id.clone(),
            build_ok.pack_id.clone(),
            build_ok.ordered_entries.clone(),
            target_engines,
            true,
            true,
            true,
        )?);
        let select_resp = self.engine.run(&select_req);
        select_resp.validate()?;

        let select_ok = match select_resp {
            Ph1KnowResponse::Refuse(refuse) => return Ok(KnowWiringOutcome::Refused(refuse)),
            Ph1KnowResponse::KnowHintBundleSelectOk(ok) => ok,
            Ph1KnowResponse::KnowDictionaryPackBuildOk(_) => {
                return Ok(KnowWiringOutcome::Refused(KnowRefuse::v1(
                    KnowCapabilityId::KnowHintBundleSelect,
                    reason_codes::PH1_KNOW_INTERNAL_PIPELINE_ERROR,
                    "unexpected dictionary-pack-build response for hint-bundle-select request"
                        .to_string(),
                )?))
            }
        };

        if select_ok.validation_status != KnowValidationStatus::Ok {
            return Ok(KnowWiringOutcome::Refused(KnowRefuse::v1(
                KnowCapabilityId::KnowHintBundleSelect,
                reason_codes::PH1_KNOW_VALIDATION_FAILED,
                "knowledge hint bundle selection validation failed".to_string(),
            )?));
        }

        let bundle =
            KnowForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, select_ok)?;
        Ok(KnowWiringOutcome::Forwarded(bundle))
    }
}

fn validate_token(
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
    if value
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1know::{KnowEntryKind, KnowSourceKind, KnowTargetEngine};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicKnowEngine;

    impl Ph1KnowEngine for DeterministicKnowEngine {
        fn run(&self, req: &Ph1KnowRequest) -> Ph1KnowResponse {
            match req {
                Ph1KnowRequest::KnowDictionaryPackBuild(r) => {
                    let mut targets = vec![
                        KnowTargetEngine::C,
                        KnowTargetEngine::Srl,
                        KnowTargetEngine::Nlp,
                    ];
                    if r.entries
                        .iter()
                        .any(|entry| entry.pronunciation_hint.is_some())
                    {
                        targets.push(KnowTargetEngine::Tts);
                    }

                    Ph1KnowResponse::KnowDictionaryPackBuildOk(
                        KnowDictionaryPackBuildOk::v1(
                            ReasonCodeId(501),
                            "know.pack.tenant_1.abc".to_string(),
                            targets,
                            r.entries.clone(),
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1KnowRequest::KnowHintBundleSelect(r) => Ph1KnowResponse::KnowHintBundleSelectOk(
                    KnowHintBundleSelectOk::v1(
                        ReasonCodeId(502),
                        KnowValidationStatus::Ok,
                        vec![],
                        r.target_engines.clone(),
                        true,
                        true,
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftKnowEngine;

    impl Ph1KnowEngine for DriftKnowEngine {
        fn run(&self, req: &Ph1KnowRequest) -> Ph1KnowResponse {
            match req {
                Ph1KnowRequest::KnowDictionaryPackBuild(r) => {
                    Ph1KnowResponse::KnowDictionaryPackBuildOk(
                        KnowDictionaryPackBuildOk::v1(
                            ReasonCodeId(511),
                            "know.pack.tenant_1.xyz".to_string(),
                            vec![KnowTargetEngine::C, KnowTargetEngine::Nlp],
                            r.entries.clone(),
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1KnowRequest::KnowHintBundleSelect(r) => Ph1KnowResponse::KnowHintBundleSelectOk(
                    KnowHintBundleSelectOk::v1(
                        ReasonCodeId(512),
                        KnowValidationStatus::Fail,
                        vec!["entry_order_not_canonical".to_string()],
                        r.target_engines.clone(),
                        false,
                        true,
                        false,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn entry(
        entry_id: &str,
        entry_kind: KnowEntryKind,
        source_kind: KnowSourceKind,
        canonical_term: &str,
        normalized_term: &str,
        pronunciation_hint: Option<&str>,
    ) -> KnowDictionaryEntry {
        KnowDictionaryEntry::v1(
            entry_id.to_string(),
            "tenant_1".to_string(),
            entry_kind,
            source_kind,
            canonical_term.to_string(),
            normalized_term.to_string(),
            "en".to_string(),
            pronunciation_hint.map(|value| value.to_string()),
            format!("know:evidence:{}", entry_id),
        )
        .unwrap()
    }

    fn input() -> KnowTurnInput {
        KnowTurnInput::v1(
            CorrelationId(4101),
            TurnId(391),
            "tenant_1".to_string(),
            vec![
                entry(
                    "entry_1",
                    KnowEntryKind::EmployeeNamePreferred,
                    KnowSourceKind::HrOrgAuthorized,
                    "Jia Li",
                    "jia li",
                    None,
                ),
                entry(
                    "entry_2",
                    KnowEntryKind::PronunciationHint,
                    KnowSourceKind::UserProvidedConsent,
                    "Selene",
                    "selene",
                    Some("seh-leen"),
                ),
            ],
            true,
            true,
            true,
            Some("artifact.learn.v4".to_string()),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn at_know_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1KnowWiring::new(Ph1KnowWiringConfig::mvp_v1(true), DeterministicKnowEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            KnowWiringOutcome::Forwarded(bundle) => assert!(bundle.validate().is_ok()),
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_know_02_os_output_is_deterministic() {
        let wiring =
            Ph1KnowWiring::new(Ph1KnowWiringConfig::mvp_v1(true), DeterministicKnowEngine).unwrap();

        let out_1 = wiring.run_turn(&input()).unwrap();
        let out_2 = wiring.run_turn(&input()).unwrap();

        match (out_1, out_2) {
            (KnowWiringOutcome::Forwarded(a), KnowWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.dictionary_pack_build, b.dictionary_pack_build);
                assert_eq!(a.hint_bundle_select, b.hint_bundle_select);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_know_03_os_does_not_invoke_when_know_is_disabled() {
        let wiring =
            Ph1KnowWiring::new(Ph1KnowWiringConfig::mvp_v1(false), DeterministicKnowEngine)
                .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, KnowWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_know_04_os_fails_closed_on_hint_bundle_select_drift() {
        let wiring =
            Ph1KnowWiring::new(Ph1KnowWiringConfig::mvp_v1(true), DriftKnowEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            KnowWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_KNOW_VALIDATION_FAILED)
            }
            _ => panic!("expected Refused"),
        }
    }
}
