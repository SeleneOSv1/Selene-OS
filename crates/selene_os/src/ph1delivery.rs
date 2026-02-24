#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1delivery::{
    Ph1DeliveryRefuse, Ph1DeliveryRequest, Ph1DeliveryResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DELIVERY OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_DELIVERY_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4444_0101);
    pub const PH1_DELIVERY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4444_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1DeliveryWiringConfig {
    pub delivery_enabled: bool,
}

impl Ph1DeliveryWiringConfig {
    pub fn mvp_v1(delivery_enabled: bool) -> Self {
        Self { delivery_enabled }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DeliveryWiringOutcome {
    NotInvokedDisabled,
    Refused(Ph1DeliveryRefuse),
    Forwarded(selene_kernel_contracts::ph1delivery::Ph1DeliveryOk),
}

pub trait Ph1DeliveryEngine {
    fn run(&self, req: &Ph1DeliveryRequest) -> Ph1DeliveryResponse;
}

impl Ph1DeliveryEngine for selene_engines::ph1delivery::Ph1DeliveryRuntime {
    fn run(&self, req: &Ph1DeliveryRequest) -> Ph1DeliveryResponse {
        self.run(req)
    }
}

#[derive(Debug, Clone)]
pub struct Ph1DeliveryWiring<E>
where
    E: Ph1DeliveryEngine,
{
    config: Ph1DeliveryWiringConfig,
    engine: E,
}

impl<E> Ph1DeliveryWiring<E>
where
    E: Ph1DeliveryEngine,
{
    pub fn new(config: Ph1DeliveryWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_request(
        &self,
        req: &Ph1DeliveryRequest,
    ) -> Result<Ph1DeliveryWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.delivery_enabled {
            return Ok(Ph1DeliveryWiringOutcome::NotInvokedDisabled);
        }

        let resp = self.engine.run(req);
        resp.validate()?;

        match resp {
            Ph1DeliveryResponse::Ok(ok) => {
                if ok.simulation_id != req.simulation_id {
                    return Ok(Ph1DeliveryWiringOutcome::Refused(Ph1DeliveryRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_DELIVERY_INTERNAL_PIPELINE_ERROR,
                        "simulation id drift detected in ph1delivery output".to_string(),
                    )?));
                }
                if ok.capability_id != req.request.capability_id() {
                    return Ok(Ph1DeliveryWiringOutcome::Refused(Ph1DeliveryRefuse::v1(
                        ok.capability_id,
                        req.simulation_id.clone(),
                        reason_codes::PH1_DELIVERY_INTERNAL_PIPELINE_ERROR,
                        "capability drift detected in ph1delivery output".to_string(),
                    )?));
                }
                Ok(Ph1DeliveryWiringOutcome::Forwarded(ok))
            }
            Ph1DeliveryResponse::Refuse(r) => Ok(Ph1DeliveryWiringOutcome::Refused(r)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1delivery::{
        DeliveryCapabilityId, DeliveryChannel, DeliveryOutcome, DeliverySimulationType,
        DeliveryStatus, DeliveryStatusResult, Ph1DeliveryOk, DELIVERY_SEND_COMMIT,
        PH1DELIVERY_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

    #[derive(Debug, Clone)]
    struct StubEngine {
        response: Ph1DeliveryResponse,
    }

    impl Ph1DeliveryEngine for StubEngine {
        fn run(&self, _req: &Ph1DeliveryRequest) -> Ph1DeliveryResponse {
            self.response.clone()
        }
    }

    fn req() -> Ph1DeliveryRequest {
        Ph1DeliveryRequest::send_commit_v1(
            CorrelationId(101),
            TurnId(201),
            MonotonicTimeNs(301),
            TenantId::new("tenant_delivery_wiring").unwrap(),
            "message_1".to_string(),
            "recipient_1".to_string(),
            DeliveryChannel::Sms,
            "payload_ref_1".to_string(),
            "provider:sms/default".to_string(),
            "sim_ctx_1".to_string(),
            "idem_1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_delivery_wiring_01_forwards_schema_valid_output() {
        let out_ok = Ph1DeliveryOk::v1(
            DeliveryCapabilityId::Send,
            DELIVERY_SEND_COMMIT.to_string(),
            ReasonCodeId(1),
            DeliveryOutcome::Send(selene_kernel_contracts::ph1delivery::DeliverySendResult {
                delivery_attempt_id: "attempt_1".to_string(),
                delivery_proof_ref: "proof_1".to_string(),
                delivery_status: DeliveryStatus::Sent,
                provider_message_ref: Some("provider_msg_1".to_string()),
                reason_code: ReasonCodeId(1),
            }),
            true,
        )
        .unwrap();
        let wiring = Ph1DeliveryWiring::new(
            Ph1DeliveryWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1DeliveryResponse::Ok(out_ok),
            },
        )
        .unwrap();
        match wiring.run_request(&req()).unwrap() {
            Ph1DeliveryWiringOutcome::Forwarded(ok) => {
                assert_eq!(ok.capability_id, DeliveryCapabilityId::Send)
            }
            _ => panic!("expected forwarded output"),
        }
    }

    #[test]
    fn at_delivery_wiring_02_disabled_returns_not_invoked() {
        let wiring = Ph1DeliveryWiring::new(
            Ph1DeliveryWiringConfig::mvp_v1(false),
            StubEngine {
                response: Ph1DeliveryResponse::Refuse(
                    Ph1DeliveryRefuse::v1(
                        DeliveryCapabilityId::Send,
                        DELIVERY_SEND_COMMIT.to_string(),
                        ReasonCodeId(2),
                        "disabled".to_string(),
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap();
        assert_eq!(
            wiring.run_request(&req()).unwrap(),
            Ph1DeliveryWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_delivery_wiring_03_simulation_id_drift_fails_closed() {
        let drift_ok = Ph1DeliveryOk::v1(
            DeliveryCapabilityId::Send,
            "DELIVERY_SEND_COMMIT_DRIFT".to_string(),
            ReasonCodeId(3),
            DeliveryOutcome::Send(selene_kernel_contracts::ph1delivery::DeliverySendResult {
                delivery_attempt_id: "attempt_2".to_string(),
                delivery_proof_ref: "proof_2".to_string(),
                delivery_status: DeliveryStatus::Sent,
                provider_message_ref: Some("provider_msg_2".to_string()),
                reason_code: ReasonCodeId(3),
            }),
            true,
        )
        .unwrap();
        let wiring = Ph1DeliveryWiring::new(
            Ph1DeliveryWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1DeliveryResponse::Ok(drift_ok),
            },
        )
        .unwrap();
        match wiring.run_request(&req()).unwrap() {
            Ph1DeliveryWiringOutcome::Refused(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_DELIVERY_INTERNAL_PIPELINE_ERROR
                );
            }
            _ => panic!("expected refused drift output"),
        }
    }

    #[test]
    fn at_delivery_wiring_04_capability_drift_fails_closed() {
        let capability_drift = Ph1DeliveryOk::v1(
            DeliveryCapabilityId::Status,
            DELIVERY_SEND_COMMIT.to_string(),
            ReasonCodeId(4),
            DeliveryOutcome::Status(DeliveryStatusResult {
                normalized_status: DeliveryStatus::Pending,
                provider_status_raw: "pending".to_string(),
                reason_code: ReasonCodeId(4),
            }),
            true,
        )
        .unwrap();
        let wiring = Ph1DeliveryWiring::new(
            Ph1DeliveryWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1DeliveryResponse::Ok(capability_drift),
            },
        )
        .unwrap();
        match wiring.run_request(&req()).unwrap() {
            Ph1DeliveryWiringOutcome::Refused(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_DELIVERY_INTERNAL_PIPELINE_ERROR
                );
            }
            _ => panic!("expected refused capability drift output"),
        }
    }

    #[test]
    fn at_delivery_wiring_05_request_contract_validation_error_bubbles() {
        let mut bad_req = req();
        bad_req.simulation_type = DeliverySimulationType::Draft;
        bad_req.schema_version = PH1DELIVERY_CONTRACT_VERSION;
        let wiring = Ph1DeliveryWiring::new(
            Ph1DeliveryWiringConfig::mvp_v1(true),
            StubEngine {
                response: Ph1DeliveryResponse::Refuse(
                    Ph1DeliveryRefuse::v1(
                        DeliveryCapabilityId::Send,
                        DELIVERY_SEND_COMMIT.to_string(),
                        ReasonCodeId(5),
                        "noop".to_string(),
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap();
        assert!(wiring.run_request(&bad_req).is_err());
    }
}
