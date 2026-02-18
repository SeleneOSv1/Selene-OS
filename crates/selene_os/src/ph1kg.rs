#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1kg::{
    KgCapabilityId, KgEntityCandidate, KgEntityLinkOk, KgEntityLinkRequest, KgFactBundleSelectOk,
    KgFactBundleSelectRequest, KgRefuse, KgRelationType, KgRequestEnvelope, KgValidationStatus,
    Ph1KgRequest, Ph1KgResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KG OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_KG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B47_0101);
    pub const PH1_KG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B47_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KgWiringConfig {
    pub kg_enabled: bool,
    pub max_entity_candidates: u8,
    pub max_fact_candidates: u8,
    pub max_diagnostics: u8,
}

impl Ph1KgWiringConfig {
    pub fn mvp_v1(kg_enabled: bool) -> Self {
        Self {
            kg_enabled,
            max_entity_candidates: 24,
            max_fact_candidates: 12,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub entity_candidates: Vec<KgEntityCandidate>,
    pub relation_type_hints: Vec<KgRelationType>,
}

impl KgTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        entity_candidates: Vec<KgEntityCandidate>,
        relation_type_hints: Vec<KgRelationType>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            entity_candidates,
            relation_type_hints,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for KgTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.tenant_id.is_empty() || self.tenant_id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_turn_input.tenant_id",
                reason: "must be non-empty and <= 64 chars",
            });
        }
        if self.entity_candidates.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_turn_input.entity_candidates",
                reason: "must be <= 64",
            });
        }
        if self.relation_type_hints.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "kg_turn_input.relation_type_hints",
                reason: "must be <= 8",
            });
        }
        for entity in &self.entity_candidates {
            entity.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KgForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub entity_link: KgEntityLinkOk,
    pub fact_bundle_select: KgFactBundleSelectOk,
}

impl KgForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        entity_link: KgEntityLinkOk,
        fact_bundle_select: KgFactBundleSelectOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            entity_link,
            fact_bundle_select,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for KgForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.entity_link.validate()?;
        self.fact_bundle_select.validate()?;
        if self.fact_bundle_select.validation_status != KgValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "kg_forward_bundle.fact_bundle_select.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KgWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoEntities,
    Refused(KgRefuse),
    Forwarded(KgForwardBundle),
}

pub trait Ph1KgEngine {
    fn run(&self, req: &Ph1KgRequest) -> Ph1KgResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1KgWiring<E>
where
    E: Ph1KgEngine,
{
    config: Ph1KgWiringConfig,
    engine: E,
}

impl<E> Ph1KgWiring<E>
where
    E: Ph1KgEngine,
{
    pub fn new(config: Ph1KgWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_entity_candidates == 0 || config.max_entity_candidates > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1kg_wiring_config.max_entity_candidates",
                reason: "must be within 1..=64",
            });
        }
        if config.max_fact_candidates == 0 || config.max_fact_candidates > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1kg_wiring_config.max_fact_candidates",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1kg_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &KgTurnInput) -> Result<KgWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.kg_enabled {
            return Ok(KgWiringOutcome::NotInvokedDisabled);
        }
        if input.entity_candidates.is_empty() || input.relation_type_hints.is_empty() {
            return Ok(KgWiringOutcome::NotInvokedNoEntities);
        }

        let envelope = KgRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_entity_candidates, 64),
            min(self.config.max_fact_candidates, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let link_req = Ph1KgRequest::KgEntityLink(KgEntityLinkRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.entity_candidates.clone(),
            input.relation_type_hints.clone(),
        )?);
        let link_resp = self.engine.run(&link_req);
        link_resp.validate()?;

        let link_ok = match link_resp {
            Ph1KgResponse::Refuse(refuse) => return Ok(KgWiringOutcome::Refused(refuse)),
            Ph1KgResponse::KgEntityLinkOk(ok) => ok,
            Ph1KgResponse::KgFactBundleSelectOk(_) => {
                return Ok(KgWiringOutcome::Refused(KgRefuse::v1(
                    KgCapabilityId::KgEntityLink,
                    reason_codes::PH1_KG_INTERNAL_PIPELINE_ERROR,
                    "unexpected fact-bundle-select response for entity-link request".to_string(),
                )?));
            }
        };

        let select_req = Ph1KgRequest::KgFactBundleSelect(KgFactBundleSelectRequest::v1(
            envelope,
            input.tenant_id.clone(),
            link_ok.selected_fact_id.clone(),
            link_ok.ordered_fact_candidates.clone(),
            true,
            true,
            true,
        )?);
        let select_resp = self.engine.run(&select_req);
        select_resp.validate()?;

        let select_ok = match select_resp {
            Ph1KgResponse::Refuse(refuse) => return Ok(KgWiringOutcome::Refused(refuse)),
            Ph1KgResponse::KgFactBundleSelectOk(ok) => ok,
            Ph1KgResponse::KgEntityLinkOk(_) => {
                return Ok(KgWiringOutcome::Refused(KgRefuse::v1(
                    KgCapabilityId::KgFactBundleSelect,
                    reason_codes::PH1_KG_INTERNAL_PIPELINE_ERROR,
                    "unexpected entity-link response for fact-bundle-select request".to_string(),
                )?));
            }
        };

        if select_ok.validation_status != KgValidationStatus::Ok {
            return Ok(KgWiringOutcome::Refused(KgRefuse::v1(
                KgCapabilityId::KgFactBundleSelect,
                reason_codes::PH1_KG_VALIDATION_FAILED,
                "kg fact-bundle-select validation failed".to_string(),
            )?));
        }

        let bundle = KgForwardBundle::v1(input.correlation_id, input.turn_id, link_ok, select_ok)?;
        Ok(KgWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1kg::{
        KgEntityLinkOk, KgEntityType, KgFactBundleSelectOk, KgFactCandidate, KgRelationType,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicKgEngine;

    impl Ph1KgEngine for DeterministicKgEngine {
        fn run(&self, req: &Ph1KgRequest) -> Ph1KgResponse {
            match req {
                Ph1KgRequest::KgEntityLink(r) => {
                    let mut facts = r
                        .entity_candidates
                        .iter()
                        .filter(|entity| entity.entity_type == KgEntityType::Person)
                        .flat_map(|person| {
                            r.entity_candidates
                                .iter()
                                .filter(move |entity| entity.entity_type == KgEntityType::Role)
                                .map(move |role| {
                                    KgFactCandidate::v1(
                                        format!(
                                            "fact:{}:{}",
                                            person.candidate_id, role.candidate_id
                                        ),
                                        r.tenant_id.clone(),
                                        KgRelationType::PersonHasRole,
                                        person.candidate_id.clone(),
                                        role.candidate_id.clone(),
                                        1200,
                                        person.evidence_ref.clone(),
                                    )
                                    .unwrap()
                                })
                        })
                        .collect::<Vec<_>>();
                    facts.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));

                    Ph1KgResponse::KgEntityLinkOk(
                        KgEntityLinkOk::v1(
                            ReasonCodeId(301),
                            facts[0].fact_id.clone(),
                            facts,
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1KgRequest::KgFactBundleSelect(_r) => Ph1KgResponse::KgFactBundleSelectOk(
                    KgFactBundleSelectOk::v1(
                        ReasonCodeId(302),
                        KgValidationStatus::Ok,
                        vec![],
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

    struct DriftKgEngine;

    impl Ph1KgEngine for DriftKgEngine {
        fn run(&self, req: &Ph1KgRequest) -> Ph1KgResponse {
            match req {
                Ph1KgRequest::KgEntityLink(r) => {
                    let fact = KgFactCandidate::v1(
                        format!(
                            "fact:{}:{}",
                            r.entity_candidates[0].candidate_id,
                            r.entity_candidates[1].candidate_id
                        ),
                        r.tenant_id.clone(),
                        KgRelationType::PersonHasRole,
                        r.entity_candidates[0].candidate_id.clone(),
                        r.entity_candidates[1].candidate_id.clone(),
                        1000,
                        r.entity_candidates[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1KgResponse::KgEntityLinkOk(
                        KgEntityLinkOk::v1(
                            ReasonCodeId(311),
                            fact.fact_id.clone(),
                            vec![fact],
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1KgRequest::KgFactBundleSelect(_r) => Ph1KgResponse::KgFactBundleSelectOk(
                    KgFactBundleSelectOk::v1(
                        ReasonCodeId(312),
                        KgValidationStatus::Fail,
                        vec!["selected_not_first_in_ordered_fact_candidates".to_string()],
                        false,
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

    fn entity(
        candidate_id: &str,
        entity_type: KgEntityType,
        confidence_bp: u16,
    ) -> KgEntityCandidate {
        KgEntityCandidate::v1(
            candidate_id.to_string(),
            "tenant_1".to_string(),
            entity_type,
            format!("key:{}", candidate_id),
            format!("Label {}", candidate_id),
            confidence_bp,
            format!("kg:evidence:{}", candidate_id),
        )
        .unwrap()
    }

    fn input() -> KgTurnInput {
        KgTurnInput::v1(
            CorrelationId(3701),
            TurnId(341),
            "tenant_1".to_string(),
            vec![
                entity("person_1", KgEntityType::Person, 8900),
                entity("role_1", KgEntityType::Role, 8500),
            ],
            vec![KgRelationType::PersonHasRole],
        )
        .unwrap()
    }

    #[test]
    fn at_kg_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1KgWiring::new(Ph1KgWiringConfig::mvp_v1(true), DeterministicKgEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            KgWiringOutcome::Forwarded(bundle) => assert!(bundle.validate().is_ok()),
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_kg_02_os_output_is_deterministic() {
        let wiring =
            Ph1KgWiring::new(Ph1KgWiringConfig::mvp_v1(true), DeterministicKgEngine).unwrap();

        let out1 = wiring.run_turn(&input()).unwrap();
        let out2 = wiring.run_turn(&input()).unwrap();

        match (out1, out2) {
            (KgWiringOutcome::Forwarded(a), KgWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.entity_link, b.entity_link);
                assert_eq!(a.fact_bundle_select, b.fact_bundle_select);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_kg_03_os_does_not_invoke_when_kg_is_disabled() {
        let wiring =
            Ph1KgWiring::new(Ph1KgWiringConfig::mvp_v1(false), DeterministicKgEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, KgWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_kg_04_os_fails_closed_on_fact_bundle_validation_drift() {
        let wiring = Ph1KgWiring::new(Ph1KgWiringConfig::mvp_v1(true), DriftKgEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            KgWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_KG_VALIDATION_FAILED)
            }
            _ => panic!("expected Refused"),
        }
    }
}
