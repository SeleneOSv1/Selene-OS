#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1bcast::{
    Ph1BcastOk, Ph1BcastRefuse, Ph1BcastRequest, Ph1BcastResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.BCAST.001 OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_BCAST_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4243_0101);
    pub const PH1_BCAST_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4243_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1BcastWiringConfig {
    pub bcast_enabled: bool,
}

impl Ph1BcastWiringConfig {
    pub fn mvp_v1(bcast_enabled: bool) -> Self {
        Self { bcast_enabled }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1BcastWiringOutcome {
    NotInvokedDisabled,
    Refused(Ph1BcastRefuse),
    Forwarded(Ph1BcastOk),
}

pub trait Ph1BcastEngine {
    fn run(&self, req: &Ph1BcastRequest) -> Ph1BcastResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1BcastWiring<E>
where
    E: Ph1BcastEngine,
{
    config: Ph1BcastWiringConfig,
    engine: E,
}

impl<E> Ph1BcastWiring<E>
where
    E: Ph1BcastEngine,
{
    pub fn new(config: Ph1BcastWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_request(
        &self,
        req: &Ph1BcastRequest,
    ) -> Result<Ph1BcastWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.bcast_enabled {
            return Ok(Ph1BcastWiringOutcome::NotInvokedDisabled);
        }

        let resp = self.engine.run(req);
        resp.validate()?;

        match resp {
            Ph1BcastResponse::Ok(ok) => {
                if ok.simulation_id != req.simulation_id {
                    return Ok(Ph1BcastWiringOutcome::Refused(Ph1BcastRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_BCAST_INTERNAL_PIPELINE_ERROR,
                        "simulation id drift detected in ph1bcast output".to_string(),
                    )?));
                }
                if ok.capability_id != req.request.capability_id() {
                    return Ok(Ph1BcastWiringOutcome::Refused(Ph1BcastRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_BCAST_INTERNAL_PIPELINE_ERROR,
                        "capability drift detected in ph1bcast output".to_string(),
                    )?));
                }
                Ok(Ph1BcastWiringOutcome::Forwarded(ok))
            }
            Ph1BcastResponse::Refuse(r) => Ok(Ph1BcastWiringOutcome::Refused(r)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1bcast::{
        BcastCapabilityId, BcastDraftCreateRequest, BcastDraftCreateResult, BcastOutcome,
        BcastRecipientState, BcastRequest, BcastSimulationType, BCAST_CREATE_DRAFT,
        PH1BCAST_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

    #[derive(Debug, Clone)]
    struct StubEngine {
        response: Ph1BcastResponse,
    }

    impl Ph1BcastEngine for StubEngine {
        fn run(&self, _req: &Ph1BcastRequest) -> Ph1BcastResponse {
            self.response.clone()
        }
    }

    fn req() -> Ph1BcastRequest {
        Ph1BcastRequest {
            schema_version: PH1BCAST_CONTRACT_VERSION,
            correlation_id: CorrelationId(991),
            turn_id: TurnId(55),
            now: MonotonicTimeNs(500),
            simulation_id: BCAST_CREATE_DRAFT.to_string(),
            simulation_type: BcastSimulationType::Draft,
            request: BcastRequest::DraftCreate(BcastDraftCreateRequest {
                tenant_id: TenantId::new("tenant_w").unwrap(),
                sender_user_id: UserId::new("user_w").unwrap(),
                audience_spec: "jd".to_string(),
                classification:
                    selene_kernel_contracts::ph1bcast::BroadcastClassification::Priority,
                content_payload_ref: "payload_w".to_string(),
                prompt_dedupe_key: Some("dedupe_w".to_string()),
                idempotency_key: "idem_w".to_string(),
            }),
        }
    }

    #[test]
    fn at_bcast_wiring_01_forwards_schema_valid_output() {
        let out_ok = Ph1BcastOk::v1(
            BcastCapabilityId::DraftCreate,
            BCAST_CREATE_DRAFT.to_string(),
            ReasonCodeId(10),
            BcastOutcome::DraftCreate(BcastDraftCreateResult {
                broadcast_id: selene_kernel_contracts::ph1bcast::BroadcastId::new("bcast_1")
                    .unwrap(),
                state: BcastRecipientState::DraftCreated,
                reason_code: ReasonCodeId(10),
            }),
            true,
            true,
        )
        .unwrap();
        let wiring = Ph1BcastWiring::new(
            Ph1BcastWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1BcastResponse::Ok(out_ok),
            },
        )
        .unwrap();

        match wiring.run_request(&req()).unwrap() {
            Ph1BcastWiringOutcome::Forwarded(ok) => {
                assert_eq!(ok.capability_id, BcastCapabilityId::DraftCreate)
            }
            _ => panic!("expected forwarded output"),
        }
    }

    #[test]
    fn at_bcast_wiring_02_disabled_wiring_returns_not_invoked() {
        let wiring = Ph1BcastWiring::new(
            Ph1BcastWiringConfig::mvp_v1(false),
            StubEngine {
                response: Ph1BcastResponse::Refuse(
                    Ph1BcastRefuse::v1(
                        BcastCapabilityId::DraftCreate,
                        BCAST_CREATE_DRAFT.to_string(),
                        ReasonCodeId(11),
                        "blocked".to_string(),
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap();
        assert_eq!(
            wiring.run_request(&req()).unwrap(),
            Ph1BcastWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_bcast_wiring_03_simulation_id_drift_fails_closed() {
        let drift_ok = Ph1BcastOk::v1(
            BcastCapabilityId::DraftCreate,
            "BCAST_CREATE_DRAFT_DRIFT".to_string(),
            ReasonCodeId(12),
            BcastOutcome::DraftCreate(BcastDraftCreateResult {
                broadcast_id: selene_kernel_contracts::ph1bcast::BroadcastId::new("bcast_2")
                    .unwrap(),
                state: BcastRecipientState::DraftCreated,
                reason_code: ReasonCodeId(12),
            }),
            true,
            true,
        )
        .unwrap();
        let wiring = Ph1BcastWiring::new(
            Ph1BcastWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1BcastResponse::Ok(drift_ok),
            },
        )
        .unwrap();
        match wiring.run_request(&req()).unwrap() {
            Ph1BcastWiringOutcome::Refused(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_BCAST_INTERNAL_PIPELINE_ERROR
                );
            }
            _ => panic!("expected refused drift output"),
        }
    }
}
