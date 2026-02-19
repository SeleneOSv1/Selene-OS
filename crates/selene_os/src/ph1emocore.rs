#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1emocore::{
    Ph1EmoCoreOk, Ph1EmoCoreRefuse, Ph1EmoCoreRequest, Ph1EmoCoreResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EMO.CORE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_EMO_CORE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4543_0101);
    pub const PH1_EMO_CORE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4543_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EmoCoreWiringConfig {
    pub emo_core_enabled: bool,
}

impl Ph1EmoCoreWiringConfig {
    pub fn mvp_v1(emo_core_enabled: bool) -> Self {
        Self { emo_core_enabled }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EmoCoreWiringOutcome {
    NotInvokedDisabled,
    Refused(Ph1EmoCoreRefuse),
    Forwarded(Ph1EmoCoreOk),
}

pub trait Ph1EmoCoreEngine {
    fn run(&self, req: &Ph1EmoCoreRequest) -> Ph1EmoCoreResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1EmoCoreWiring<E>
where
    E: Ph1EmoCoreEngine,
{
    config: Ph1EmoCoreWiringConfig,
    engine: E,
}

impl<E> Ph1EmoCoreWiring<E>
where
    E: Ph1EmoCoreEngine,
{
    pub fn new(config: Ph1EmoCoreWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_request(
        &self,
        req: &Ph1EmoCoreRequest,
    ) -> Result<Ph1EmoCoreWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.emo_core_enabled {
            return Ok(Ph1EmoCoreWiringOutcome::NotInvokedDisabled);
        }

        let resp = self.engine.run(req);
        resp.validate()?;

        match resp {
            Ph1EmoCoreResponse::Ok(ok) => {
                if ok.simulation_id != req.simulation_id {
                    return Ok(Ph1EmoCoreWiringOutcome::Refused(Ph1EmoCoreRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_EMO_CORE_INTERNAL_PIPELINE_ERROR,
                        "simulation id drift detected in ph1emo core output".to_string(),
                    )?));
                }
                if ok.capability_id != req.request.capability_id() {
                    return Ok(Ph1EmoCoreWiringOutcome::Refused(Ph1EmoCoreRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_EMO_CORE_INTERNAL_PIPELINE_ERROR,
                        "capability drift detected in ph1emo core output".to_string(),
                    )?));
                }
                Ok(Ph1EmoCoreWiringOutcome::Forwarded(ok))
            }
            Ph1EmoCoreResponse::Refuse(r) => Ok(Ph1EmoCoreWiringOutcome::Refused(r)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1emocore::{
        EmoAuditEventResult, EmoAuditEventStatus, EmoCoreCapabilityId, EmoCoreOutcome,
        EmoCoreRequest, EmoCoreSimulationType, EMO_SIM_006, PH1EMOCORE_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::MonotonicTimeNs;
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct StubEngine {
        response: Ph1EmoCoreResponse,
    }

    impl Ph1EmoCoreEngine for StubEngine {
        fn run(&self, _req: &Ph1EmoCoreRequest) -> Ph1EmoCoreResponse {
            self.response.clone()
        }
    }

    fn req() -> Ph1EmoCoreRequest {
        Ph1EmoCoreRequest {
            schema_version: PH1EMOCORE_CONTRACT_VERSION,
            correlation_id: CorrelationId(901),
            turn_id: TurnId(44),
            now: MonotonicTimeNs(200),
            simulation_id: EMO_SIM_006.to_string(),
            simulation_type: EmoCoreSimulationType::Commit,
            request: EmoCoreRequest::AuditEventCommit(
                selene_kernel_contracts::ph1emocore::EmoAuditEventCommitRequest {
                    tenant_id: selene_kernel_contracts::ph1position::TenantId::new("tenant_w")
                        .unwrap(),
                    requester_user_id: UserId::new("user_w").unwrap(),
                    session_id: Some("session_w".to_string()),
                    event_type: "EMO_EVENT".to_string(),
                    reason_codes: vec![ReasonCodeId(1)],
                    idempotency_key: "idem_w".to_string(),
                },
            ),
        }
    }

    #[test]
    fn at_emo_core_wiring_01_forwards_schema_valid_output() {
        let out_ok = Ph1EmoCoreOk::v1(
            EmoCoreCapabilityId::AuditEventCommit,
            EMO_SIM_006.to_string(),
            ReasonCodeId(10),
            EmoCoreOutcome::AuditEvent(EmoAuditEventResult {
                event_id: "emo_evt_1".to_string(),
                status: EmoAuditEventStatus::Recorded,
            }),
            true,
            true,
            true,
        )
        .unwrap();
        let wiring = Ph1EmoCoreWiring::new(
            Ph1EmoCoreWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1EmoCoreResponse::Ok(out_ok),
            },
        )
        .unwrap();

        match wiring.run_request(&req()).unwrap() {
            Ph1EmoCoreWiringOutcome::Forwarded(ok) => {
                assert_eq!(ok.capability_id, EmoCoreCapabilityId::AuditEventCommit)
            }
            _ => panic!("expected forwarded output"),
        }
    }

    #[test]
    fn at_emo_core_wiring_02_disabled_wiring_returns_not_invoked() {
        let wiring = Ph1EmoCoreWiring::new(
            Ph1EmoCoreWiringConfig::mvp_v1(false),
            StubEngine {
                response: Ph1EmoCoreResponse::Refuse(
                    Ph1EmoCoreRefuse::v1(
                        EmoCoreCapabilityId::AuditEventCommit,
                        EMO_SIM_006.to_string(),
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
            Ph1EmoCoreWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_emo_core_wiring_03_simulation_id_drift_fails_closed() {
        let drift_ok = Ph1EmoCoreOk::v1(
            EmoCoreCapabilityId::AuditEventCommit,
            "EMO_SIM_999".to_string(),
            ReasonCodeId(12),
            EmoCoreOutcome::AuditEvent(EmoAuditEventResult {
                event_id: "emo_evt_2".to_string(),
                status: EmoAuditEventStatus::Recorded,
            }),
            true,
            true,
            true,
        )
        .unwrap();
        let wiring = Ph1EmoCoreWiring::new(
            Ph1EmoCoreWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1EmoCoreResponse::Ok(drift_ok),
            },
        )
        .unwrap();
        match wiring.run_request(&req()).unwrap() {
            Ph1EmoCoreWiringOutcome::Refused(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_EMO_CORE_INTERNAL_PIPELINE_ERROR
                );
            }
            _ => panic!("expected refused drift output"),
        }
    }
}
