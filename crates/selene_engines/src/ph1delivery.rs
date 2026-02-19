#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1delivery::{
    DeliveryCapabilityId, DeliveryChannel, DeliveryLatencyBucket, DeliveryOutcome,
    DeliveryProviderHealthCheckResult, DeliveryProviderHealthState, DeliveryRequest,
    DeliveryStatus, DeliveryStatusResult, Ph1DeliveryOk, Ph1DeliveryRefuse, Ph1DeliveryRequest,
    Ph1DeliveryResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DELIVERY reason-code namespace. Values are placeholders until registry lock.
    pub const DELIVERY_OK_SEND: ReasonCodeId = ReasonCodeId(0x4445_0001);
    pub const DELIVERY_OK_STATUS: ReasonCodeId = ReasonCodeId(0x4445_0002);
    pub const DELIVERY_OK_CANCEL: ReasonCodeId = ReasonCodeId(0x4445_0003);
    pub const DELIVERY_OK_HEALTH: ReasonCodeId = ReasonCodeId(0x4445_0004);

    pub const DELIVERY_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4445_00F1);
    pub const DELIVERY_SIMULATION_CONTEXT_MISSING: ReasonCodeId = ReasonCodeId(0x4445_00F2);
    pub const DELIVERY_CHANNEL_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4445_00F3);
    pub const DELIVERY_PROVIDER_SEND_FAILED: ReasonCodeId = ReasonCodeId(0x4445_00F4);
    pub const DELIVERY_ATTEMPT_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4445_00F5);
    pub const DELIVERY_PROVIDER_STATUS_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4445_00F6);
    pub const DELIVERY_CANCEL_NOT_SUPPORTED: ReasonCodeId = ReasonCodeId(0x4445_00F7);
    pub const DELIVERY_PROVIDER_CANCEL_FAILED: ReasonCodeId = ReasonCodeId(0x4445_00F8);
    pub const DELIVERY_PROVIDER_HEALTH_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4445_00F9);
    pub const DELIVERY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4445_00FF);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryProviderBinding {
    pub provider_ref: String,
    pub kms_handle_ref: String,
    pub available: bool,
    pub cancel_supported: bool,
}

impl DeliveryProviderBinding {
    pub fn new(
        provider_ref: impl Into<String>,
        kms_handle_ref: impl Into<String>,
        available: bool,
        cancel_supported: bool,
    ) -> Self {
        Self {
            provider_ref: provider_ref.into(),
            kms_handle_ref: kms_handle_ref.into(),
            available,
            cancel_supported,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1DeliveryConfig {
    pub sms: DeliveryProviderBinding,
    pub email: DeliveryProviderBinding,
    pub whatsapp: DeliveryProviderBinding,
    pub wechat: DeliveryProviderBinding,
    pub app_push: DeliveryProviderBinding,
}

impl Ph1DeliveryConfig {
    pub fn mvp_v1() -> Self {
        Self {
            sms: DeliveryProviderBinding::new(
                "provider:sms/default",
                "kms://delivery/sms/default",
                true,
                true,
            ),
            email: DeliveryProviderBinding::new(
                "provider:email/default",
                "kms://delivery/email/default",
                true,
                true,
            ),
            whatsapp: DeliveryProviderBinding::new(
                "provider:whatsapp/default",
                "kms://delivery/whatsapp/default",
                true,
                false,
            ),
            wechat: DeliveryProviderBinding::new(
                "provider:wechat/default",
                "kms://delivery/wechat/default",
                true,
                false,
            ),
            app_push: DeliveryProviderBinding::new(
                "provider:app_push/default",
                "kms://delivery/app_push/default",
                true,
                false,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1DeliveryRuntime {
    config: Ph1DeliveryConfig,
}

impl Default for Ph1DeliveryRuntime {
    fn default() -> Self {
        Self::new(Ph1DeliveryConfig::mvp_v1())
    }
}

impl Ph1DeliveryRuntime {
    pub fn new(config: Ph1DeliveryConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1DeliveryRequest) -> Ph1DeliveryResponse {
        if req.validate().is_err() {
            return refuse(
                req.request.capability_id(),
                req.simulation_id.clone(),
                reason_codes::DELIVERY_INPUT_SCHEMA_INVALID,
                "delivery request failed contract validation",
            );
        }

        match &req.request {
            DeliveryRequest::Send(v) => {
                if v.simulation_context.trim().is_empty() {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_SIMULATION_CONTEXT_MISSING,
                        "simulation_context is required for delivery send",
                    );
                }
                let binding = self.binding_for_channel(v.channel);
                if !binding.available || v.provider_ref != binding.provider_ref {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_CHANNEL_UNAVAILABLE,
                        "provider unavailable for requested delivery channel",
                    );
                }

                if v.payload_ref.to_ascii_lowercase().contains("provider_fail") {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_PROVIDER_SEND_FAILED,
                        "provider send failed in deterministic simulation path",
                    );
                }

                let attempt_id = build_attempt_id(v);
                let proof_ref = format!(
                    "delivery_proof:{}:{}",
                    attempt_id,
                    binding.kms_handle_ref.as_str()
                );
                let provider_msg = Some(format!("provider_msg:{attempt_id}"));
                let out = DeliveryOutcome::Send(
                    selene_kernel_contracts::ph1delivery::DeliverySendResult {
                        delivery_attempt_id: attempt_id.clone(),
                        delivery_proof_ref: proof_ref,
                        delivery_status: DeliveryStatus::Sent,
                        provider_message_ref: provider_msg,
                        reason_code: reason_codes::DELIVERY_OK_SEND,
                    },
                );
                ok(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::DELIVERY_OK_SEND,
                    out,
                )
            }
            DeliveryRequest::Status(v) => {
                if v.delivery_attempt_id
                    .to_ascii_lowercase()
                    .contains("unknown")
                {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_ATTEMPT_NOT_FOUND,
                        "delivery attempt not found",
                    );
                }
                if v.provider_message_ref
                    .to_ascii_lowercase()
                    .contains("status_unavailable")
                {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_PROVIDER_STATUS_UNAVAILABLE,
                        "provider status unavailable",
                    );
                }
                let normalized = if v.provider_message_ref.to_ascii_lowercase().contains("fail") {
                    DeliveryStatus::Failed
                } else if v
                    .provider_message_ref
                    .to_ascii_lowercase()
                    .contains("pending")
                {
                    DeliveryStatus::Pending
                } else {
                    DeliveryStatus::Sent
                };
                let out = DeliveryOutcome::Status(DeliveryStatusResult {
                    normalized_status: normalized,
                    provider_status_raw: "provider_status:deterministic".to_string(),
                    reason_code: reason_codes::DELIVERY_OK_STATUS,
                });
                ok(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::DELIVERY_OK_STATUS,
                    out,
                )
            }
            DeliveryRequest::Cancel(v) => {
                if v.simulation_context.trim().is_empty() {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_SIMULATION_CONTEXT_MISSING,
                        "simulation_context is required for delivery cancel",
                    );
                }
                let binding = self.binding_for_provider_ref(&v.provider_ref);
                let Some(binding) = binding else {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_CANCEL_NOT_SUPPORTED,
                        "provider does not support cancel",
                    );
                };
                if !binding.cancel_supported {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_CANCEL_NOT_SUPPORTED,
                        "provider does not support cancel",
                    );
                }
                if v.delivery_attempt_id
                    .to_ascii_lowercase()
                    .contains("cancel_fail")
                {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_PROVIDER_CANCEL_FAILED,
                        "provider cancel failed in deterministic simulation path",
                    );
                }
                let out = DeliveryOutcome::Cancel(
                    selene_kernel_contracts::ph1delivery::DeliveryCancelResult {
                        canceled: true,
                        reason_code: reason_codes::DELIVERY_OK_CANCEL,
                    },
                );
                ok(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::DELIVERY_OK_CANCEL,
                    out,
                )
            }
            DeliveryRequest::ProviderHealthCheck(v) => {
                let state = if v.provider_ref.contains("wechat") {
                    if v.region_hint
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains("china")
                    {
                        DeliveryProviderHealthState::Healthy
                    } else {
                        DeliveryProviderHealthState::Degraded
                    }
                } else if self.binding_for_provider_ref(&v.provider_ref).is_some() {
                    DeliveryProviderHealthState::Healthy
                } else {
                    DeliveryProviderHealthState::Unavailable
                };

                if state == DeliveryProviderHealthState::Unavailable {
                    return refuse(
                        req.request.capability_id(),
                        req.simulation_id.clone(),
                        reason_codes::DELIVERY_PROVIDER_HEALTH_UNAVAILABLE,
                        "provider health unavailable",
                    );
                }

                let latency_bucket = if state == DeliveryProviderHealthState::Degraded {
                    DeliveryLatencyBucket::High
                } else {
                    DeliveryLatencyBucket::Low
                };
                let out = DeliveryOutcome::ProviderHealthCheck(DeliveryProviderHealthCheckResult {
                    provider_health_state: state,
                    latency_bucket,
                    reason_code: reason_codes::DELIVERY_OK_HEALTH,
                });
                ok(
                    req.request.capability_id(),
                    req.simulation_id.clone(),
                    reason_codes::DELIVERY_OK_HEALTH,
                    out,
                )
            }
        }
    }

    pub fn default_provider_ref_for_channel(channel: DeliveryChannel) -> &'static str {
        match channel {
            DeliveryChannel::Sms => "provider:sms/default",
            DeliveryChannel::Email => "provider:email/default",
            DeliveryChannel::Whatsapp => "provider:whatsapp/default",
            DeliveryChannel::Wechat => "provider:wechat/default",
            DeliveryChannel::AppPush => "provider:app_push/default",
        }
    }

    fn binding_for_channel(&self, channel: DeliveryChannel) -> &DeliveryProviderBinding {
        match channel {
            DeliveryChannel::Sms => &self.config.sms,
            DeliveryChannel::Email => &self.config.email,
            DeliveryChannel::Whatsapp => &self.config.whatsapp,
            DeliveryChannel::Wechat => &self.config.wechat,
            DeliveryChannel::AppPush => &self.config.app_push,
        }
    }

    fn binding_for_provider_ref(&self, provider_ref: &str) -> Option<&DeliveryProviderBinding> {
        let bindings = [
            &self.config.sms,
            &self.config.email,
            &self.config.whatsapp,
            &self.config.wechat,
            &self.config.app_push,
        ];
        bindings
            .into_iter()
            .find(|binding| binding.provider_ref == provider_ref)
    }
}

fn ok(
    capability_id: DeliveryCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    outcome: DeliveryOutcome,
) -> Ph1DeliveryResponse {
    match Ph1DeliveryOk::v1(
        capability_id,
        simulation_id.clone(),
        reason_code,
        outcome,
        true,
    ) {
        Ok(v) => Ph1DeliveryResponse::Ok(v),
        Err(_) => refuse(
            capability_id,
            simulation_id,
            reason_codes::DELIVERY_INTERNAL_PIPELINE_ERROR,
            "failed to build delivery ok response",
        ),
    }
}

fn refuse(
    capability_id: DeliveryCapabilityId,
    simulation_id: String,
    reason_code: ReasonCodeId,
    reason: &str,
) -> Ph1DeliveryResponse {
    match Ph1DeliveryRefuse::v1(
        capability_id,
        simulation_id,
        reason_code,
        reason.to_string(),
    ) {
        Ok(v) => Ph1DeliveryResponse::Refuse(v),
        Err(_) => {
            let fallback = Ph1DeliveryRefuse {
                capability_id,
                simulation_id: "delivery_refuse_fallback".to_string(),
                reason_code: reason_codes::DELIVERY_INTERNAL_PIPELINE_ERROR,
                reason: "failed to build delivery refuse response".to_string(),
            };
            Ph1DeliveryResponse::Refuse(fallback)
        }
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn build_attempt_id(v: &selene_kernel_contracts::ph1delivery::DeliverySendRequest) -> String {
    let key = format!(
        "{}|{}|{}|{:?}|{}",
        v.tenant_id.as_str(),
        v.message_id,
        v.recipient,
        v.channel,
        v.idempotency_key
    );
    format!("delivery_{:016x}", fnv1a64(key.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1delivery::{
        DeliveryRequest, DeliverySimulationType, DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT,
        PH1DELIVERY_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::MonotonicTimeNs;

    fn send_req(channel: DeliveryChannel, payload_ref: &str) -> Ph1DeliveryRequest {
        Ph1DeliveryRequest::send_commit_v1(
            CorrelationId(31),
            TurnId(41),
            MonotonicTimeNs(51),
            TenantId::new("tenant_delivery").unwrap(),
            "message_delivery".to_string(),
            "recipient_delivery".to_string(),
            channel,
            payload_ref.to_string(),
            Ph1DeliveryRuntime::default_provider_ref_for_channel(channel).to_string(),
            "sim_ctx_delivery".to_string(),
            "idem_delivery".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_delivery_01_send_sms_returns_ok() {
        let rt = Ph1DeliveryRuntime::default();
        let out = rt.run(&send_req(DeliveryChannel::Sms, "payload_ok"));
        match out {
            Ph1DeliveryResponse::Ok(ok) => match ok.outcome {
                DeliveryOutcome::Send(v) => {
                    assert_eq!(v.delivery_status, DeliveryStatus::Sent);
                    assert!(v.delivery_proof_ref.contains("kms://delivery/sms/default"));
                }
                _ => panic!("expected send outcome"),
            },
            _ => panic!("expected ok response"),
        }
    }

    #[test]
    fn at_delivery_02_send_provider_fail_fails_closed() {
        let rt = Ph1DeliveryRuntime::default();
        let out = rt.run(&send_req(DeliveryChannel::Email, "provider_fail_payload"));
        match out {
            Ph1DeliveryResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::DELIVERY_PROVIDER_SEND_FAILED)
            }
            _ => panic!("expected refuse response"),
        }
    }

    #[test]
    fn at_delivery_03_cancel_unsupported_for_wechat() {
        let rt = Ph1DeliveryRuntime::default();
        let req = Ph1DeliveryRequest {
            schema_version: PH1DELIVERY_CONTRACT_VERSION,
            correlation_id: CorrelationId(61),
            turn_id: TurnId(71),
            now: MonotonicTimeNs(81),
            simulation_id: selene_kernel_contracts::ph1delivery::DELIVERY_CANCEL_COMMIT.to_string(),
            simulation_type: DeliverySimulationType::Commit,
            request: DeliveryRequest::Cancel(
                selene_kernel_contracts::ph1delivery::DeliveryCancelRequest {
                    delivery_attempt_id: "attempt_1".to_string(),
                    provider_ref: "provider:wechat/default".to_string(),
                    simulation_context: "sim_ctx".to_string(),
                    idempotency_key: "idem_cancel".to_string(),
                },
            ),
        };
        let out = rt.run(&req);
        match out {
            Ph1DeliveryResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::DELIVERY_CANCEL_NOT_SUPPORTED)
            }
            _ => panic!("expected refuse response"),
        }
    }

    #[test]
    fn at_delivery_04_health_check_unknown_provider_fails_closed() {
        let rt = Ph1DeliveryRuntime::default();
        let req = Ph1DeliveryRequest {
            schema_version: PH1DELIVERY_CONTRACT_VERSION,
            correlation_id: CorrelationId(91),
            turn_id: TurnId(101),
            now: MonotonicTimeNs(111),
            simulation_id: DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT.to_string(),
            simulation_type: DeliverySimulationType::Draft,
            request: DeliveryRequest::ProviderHealthCheck(
                selene_kernel_contracts::ph1delivery::DeliveryProviderHealthCheckRequest {
                    provider_ref: "provider:unknown".to_string(),
                    region_hint: None,
                },
            ),
        };
        let out = rt.run(&req);
        match out {
            Ph1DeliveryResponse::Refuse(r) => assert_eq!(
                r.reason_code,
                reason_codes::DELIVERY_PROVIDER_HEALTH_UNAVAILABLE
            ),
            _ => panic!("expected refuse response"),
        }
    }
}
