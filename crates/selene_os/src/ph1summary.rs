#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1summary::{
    Ph1SummaryRequest, Ph1SummaryResponse, SummaryBuildOk, SummaryBuildRequest,
    SummaryCitationValidateOk, SummaryCitationValidateRequest, SummaryEvidenceItem, SummaryRefuse,
    SummaryRequestEnvelope, SummaryValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SUMMARY OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_SUMMARY_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5355_0101);
    pub const PH1_SUMMARY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5355_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SummaryWiringConfig {
    pub summary_enabled: bool,
    pub max_summary_bullets: u8,
}

impl Ph1SummaryWiringConfig {
    pub fn mvp_v1(summary_enabled: bool) -> Self {
        Self {
            summary_enabled,
            max_summary_bullets: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub evidence_items: Vec<SummaryEvidenceItem>,
}

impl SummaryTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        evidence_items: Vec<SummaryEvidenceItem>,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            evidence_items,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for SummaryTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.evidence_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_turn_input.evidence_items",
                reason: "must be <= 128 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub summary_build: SummaryBuildOk,
    pub summary_validation: SummaryCitationValidateOk,
}

impl SummaryForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        summary_build: SummaryBuildOk,
        summary_validation: SummaryCitationValidateOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            summary_build,
            summary_validation,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for SummaryForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.summary_build.validate()?;
        self.summary_validation.validate()?;
        if self.summary_validation.validation_status != SummaryValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "summary_forward_bundle.summary_validation.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoEvidence,
    Refused(SummaryRefuse),
    Forwarded(SummaryForwardBundle),
}

pub trait Ph1SummaryEngine {
    fn run(&self, req: &Ph1SummaryRequest) -> Ph1SummaryResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1SummaryWiring<E>
where
    E: Ph1SummaryEngine,
{
    config: Ph1SummaryWiringConfig,
    engine: E,
}

impl<E> Ph1SummaryWiring<E>
where
    E: Ph1SummaryEngine,
{
    pub fn new(config: Ph1SummaryWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_summary_bullets == 0 || config.max_summary_bullets > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1summary_wiring_config.max_summary_bullets",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &SummaryTurnInput,
    ) -> Result<SummaryWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.summary_enabled {
            return Ok(SummaryWiringOutcome::NotInvokedDisabled);
        }

        if input.evidence_items.is_empty() {
            return Ok(SummaryWiringOutcome::NotInvokedNoEvidence);
        }

        let envelope = SummaryRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_summary_bullets, 16),
        )?;

        let build_req = Ph1SummaryRequest::SummaryBuild(SummaryBuildRequest::v1(
            envelope.clone(),
            input.evidence_items.clone(),
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1SummaryResponse::Refuse(r) => return Ok(SummaryWiringOutcome::Refused(r)),
            Ph1SummaryResponse::SummaryBuildOk(ok) => ok,
            Ph1SummaryResponse::SummaryCitationValidateOk(_) => {
                return Ok(SummaryWiringOutcome::Refused(SummaryRefuse::v1(
                    selene_kernel_contracts::ph1summary::SummaryCapabilityId::SummaryBuild,
                    reason_codes::PH1_SUMMARY_INTERNAL_PIPELINE_ERROR,
                    "unexpected validation response for summary build request".to_string(),
                )?))
            }
        };

        let validate_req =
            Ph1SummaryRequest::SummaryCitationValidate(SummaryCitationValidateRequest::v1(
                envelope,
                input.evidence_items.clone(),
                build_ok.summary_bullets.clone(),
            )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1SummaryResponse::Refuse(r) => return Ok(SummaryWiringOutcome::Refused(r)),
            Ph1SummaryResponse::SummaryCitationValidateOk(ok) => ok,
            Ph1SummaryResponse::SummaryBuildOk(_) => {
                return Ok(SummaryWiringOutcome::Refused(SummaryRefuse::v1(
                    selene_kernel_contracts::ph1summary::SummaryCapabilityId::SummaryCitationValidate,
                    reason_codes::PH1_SUMMARY_INTERNAL_PIPELINE_ERROR,
                    "unexpected summary build response for citation validation request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != SummaryValidationStatus::Ok {
            return Ok(SummaryWiringOutcome::Refused(SummaryRefuse::v1(
                selene_kernel_contracts::ph1summary::SummaryCapabilityId::SummaryCitationValidate,
                reason_codes::PH1_SUMMARY_VALIDATION_FAILED,
                "summary citation validation failed".to_string(),
            )?));
        }

        let bundle =
            SummaryForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(SummaryWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1summary::{
        SummaryBuildOk, SummaryBullet, SummaryCapabilityId, SummaryCitationValidateOk,
        SummaryEvidenceId,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicSummaryEngine;

    impl Ph1SummaryEngine for DeterministicSummaryEngine {
        fn run(&self, req: &Ph1SummaryRequest) -> Ph1SummaryResponse {
            match req {
                Ph1SummaryRequest::SummaryBuild(r) => {
                    let bullets = r
                        .evidence_items
                        .iter()
                        .take(r.envelope.max_summary_bullets as usize)
                        .map(|e| {
                            SummaryBullet::v1(e.text.clone(), vec![e.evidence_id.clone()]).unwrap()
                        })
                        .collect::<Vec<_>>();

                    Ph1SummaryResponse::SummaryBuildOk(
                        SummaryBuildOk::v1(ReasonCodeId(1), bullets, true).unwrap(),
                    )
                }
                Ph1SummaryRequest::SummaryCitationValidate(r) => {
                    let evidence = r
                        .evidence_items
                        .iter()
                        .map(|e| e.text.to_ascii_lowercase())
                        .collect::<Vec<_>>();
                    let mut diagnostics = vec![];
                    for (idx, b) in r.summary_bullets.iter().enumerate() {
                        if !evidence
                            .iter()
                            .any(|e| e.contains(&b.text.to_ascii_lowercase()))
                        {
                            diagnostics.push(format!("bullet_{idx}_not_evidence_backed"));
                        }
                    }
                    let (status, reason) = if diagnostics.is_empty() {
                        (SummaryValidationStatus::Ok, ReasonCodeId(2))
                    } else {
                        (SummaryValidationStatus::Fail, ReasonCodeId(3))
                    };
                    Ph1SummaryResponse::SummaryCitationValidateOk(
                        SummaryCitationValidateOk::v1(reason, status, diagnostics, true).unwrap(),
                    )
                }
            }
        }
    }

    struct DriftSummaryEngine;

    impl Ph1SummaryEngine for DriftSummaryEngine {
        fn run(&self, req: &Ph1SummaryRequest) -> Ph1SummaryResponse {
            match req {
                Ph1SummaryRequest::SummaryBuild(r) => {
                    let mut bullets = r
                        .evidence_items
                        .iter()
                        .map(|e| {
                            SummaryBullet::v1(e.text.clone(), vec![e.evidence_id.clone()]).unwrap()
                        })
                        .collect::<Vec<_>>();
                    bullets.push(
                        SummaryBullet::v1(
                            "Inferred hidden decision".to_string(),
                            vec![SummaryEvidenceId::new("missing").unwrap()],
                        )
                        .unwrap(),
                    );
                    Ph1SummaryResponse::SummaryBuildOk(
                        SummaryBuildOk::v1(ReasonCodeId(10), bullets, true).unwrap(),
                    )
                }
                Ph1SummaryRequest::SummaryCitationValidate(_r) => {
                    Ph1SummaryResponse::SummaryCitationValidateOk(
                        SummaryCitationValidateOk::v1(
                            ReasonCodeId(11),
                            SummaryValidationStatus::Fail,
                            vec!["bullet_2_not_evidence_backed".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn evidence(id: &str, text: &str) -> SummaryEvidenceItem {
        SummaryEvidenceItem::v1(SummaryEvidenceId::new(id).unwrap(), text.to_string()).unwrap()
    }

    #[test]
    fn at_summary_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1SummaryWiring::new(
            Ph1SummaryWiringConfig::mvp_v1(true),
            DeterministicSummaryEngine,
        )
        .unwrap();

        let input = SummaryTurnInput::v1(
            CorrelationId(701),
            TurnId(31),
            vec![
                evidence("e1", "Doc section one"),
                evidence("e2", "Doc section two"),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SummaryWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_summary_02_os_preserves_summary_order_for_downstream_consumers() {
        let wiring = Ph1SummaryWiring::new(
            Ph1SummaryWiringConfig::mvp_v1(true),
            DeterministicSummaryEngine,
        )
        .unwrap();

        let input = SummaryTurnInput::v1(
            CorrelationId(702),
            TurnId(32),
            vec![
                evidence("e1", "First summary item"),
                evidence("e2", "Second summary item"),
                evidence("e3", "Third summary item"),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SummaryWiringOutcome::Forwarded(bundle) => {
                let items = bundle
                    .summary_build
                    .summary_bullets
                    .into_iter()
                    .map(|b| b.text)
                    .collect::<Vec<_>>();
                assert_eq!(
                    items,
                    vec![
                        "First summary item",
                        "Second summary item",
                        "Third summary item"
                    ]
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_summary_03_os_does_not_invoke_when_summary_disabled() {
        let wiring = Ph1SummaryWiring::new(
            Ph1SummaryWiringConfig::mvp_v1(false),
            DeterministicSummaryEngine,
        )
        .unwrap();

        let input = SummaryTurnInput::v1(
            CorrelationId(703),
            TurnId(33),
            vec![evidence("e1", "Any evidence")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, SummaryWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_summary_04_os_fails_closed_on_citation_validation_drift() {
        let wiring =
            Ph1SummaryWiring::new(Ph1SummaryWiringConfig::mvp_v1(true), DriftSummaryEngine)
                .unwrap();

        let input = SummaryTurnInput::v1(
            CorrelationId(704),
            TurnId(34),
            vec![
                evidence("e1", "Visible evidence one"),
                evidence("e2", "Visible evidence two"),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SummaryWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_SUMMARY_VALIDATION_FAILED);
                assert_eq!(
                    r.capability_id,
                    SummaryCapabilityId::SummaryCitationValidate
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
