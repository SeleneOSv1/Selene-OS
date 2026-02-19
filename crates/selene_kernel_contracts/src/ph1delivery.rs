#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1DELIVERY_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1DELIVERY_ENGINE_ID: &str = "PH1.DELIVERY";

pub const DELIVERY_SEND_COMMIT: &str = "DELIVERY_SEND_COMMIT";
pub const DELIVERY_CANCEL_COMMIT: &str = "DELIVERY_CANCEL_COMMIT";
pub const DELIVERY_STATUS_DRAFT: &str = "DELIVERY_STATUS_DRAFT";
pub const DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT: &str = "DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT";

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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_opt_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_token(field, v, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliverySimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryCapabilityId {
    Send,
    Status,
    Cancel,
    ProviderHealthCheck,
}

impl DeliveryCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            DeliveryCapabilityId::Send => "DELIVERY_SEND",
            DeliveryCapabilityId::Status => "DELIVERY_STATUS",
            DeliveryCapabilityId::Cancel => "DELIVERY_CANCEL",
            DeliveryCapabilityId::ProviderHealthCheck => "DELIVERY_PROVIDER_HEALTH_CHECK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryChannel {
    Sms,
    Email,
    Whatsapp,
    Wechat,
    AppPush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryStatus {
    Sent,
    Pending,
    Failed,
    Canceled,
    NotSupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryProviderHealthState {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryLatencyBucket {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliverySendRequest {
    pub tenant_id: TenantId,
    pub message_id: String,
    pub recipient: String,
    pub channel: DeliveryChannel,
    pub payload_ref: String,
    pub provider_ref: String,
    pub simulation_context: String,
    pub idempotency_key: String,
}

impl Validate for DeliverySendRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token("delivery_send.message_id", &self.message_id, 128)?;
        validate_token("delivery_send.recipient", &self.recipient, 256)?;
        validate_token("delivery_send.payload_ref", &self.payload_ref, 256)?;
        validate_token("delivery_send.provider_ref", &self.provider_ref, 128)?;
        validate_token(
            "delivery_send.simulation_context",
            &self.simulation_context,
            256,
        )?;
        validate_token("delivery_send.idempotency_key", &self.idempotency_key, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryStatusRequest {
    pub delivery_attempt_id: String,
    pub provider_ref: String,
    pub provider_message_ref: String,
}

impl Validate for DeliveryStatusRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "delivery_status.delivery_attempt_id",
            &self.delivery_attempt_id,
            128,
        )?;
        validate_token("delivery_status.provider_ref", &self.provider_ref, 128)?;
        validate_token(
            "delivery_status.provider_message_ref",
            &self.provider_message_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryCancelRequest {
    pub delivery_attempt_id: String,
    pub provider_ref: String,
    pub simulation_context: String,
    pub idempotency_key: String,
}

impl Validate for DeliveryCancelRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "delivery_cancel.delivery_attempt_id",
            &self.delivery_attempt_id,
            128,
        )?;
        validate_token("delivery_cancel.provider_ref", &self.provider_ref, 128)?;
        validate_token(
            "delivery_cancel.simulation_context",
            &self.simulation_context,
            256,
        )?;
        validate_token(
            "delivery_cancel.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryProviderHealthCheckRequest {
    pub provider_ref: String,
    pub region_hint: Option<String>,
}

impl Validate for DeliveryProviderHealthCheckRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "delivery_provider_health_check.provider_ref",
            &self.provider_ref,
            128,
        )?;
        validate_opt_token(
            "delivery_provider_health_check.region_hint",
            &self.region_hint,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryRequest {
    Send(DeliverySendRequest),
    Status(DeliveryStatusRequest),
    Cancel(DeliveryCancelRequest),
    ProviderHealthCheck(DeliveryProviderHealthCheckRequest),
}

impl DeliveryRequest {
    pub fn capability_id(&self) -> DeliveryCapabilityId {
        match self {
            DeliveryRequest::Send(_) => DeliveryCapabilityId::Send,
            DeliveryRequest::Status(_) => DeliveryCapabilityId::Status,
            DeliveryRequest::Cancel(_) => DeliveryCapabilityId::Cancel,
            DeliveryRequest::ProviderHealthCheck(_) => DeliveryCapabilityId::ProviderHealthCheck,
        }
    }

    pub fn expected_simulation_id(&self) -> &'static str {
        match self {
            DeliveryRequest::Send(_) => DELIVERY_SEND_COMMIT,
            DeliveryRequest::Status(_) => DELIVERY_STATUS_DRAFT,
            DeliveryRequest::Cancel(_) => DELIVERY_CANCEL_COMMIT,
            DeliveryRequest::ProviderHealthCheck(_) => DELIVERY_PROVIDER_HEALTH_CHECK_DRAFT,
        }
    }

    pub fn expected_simulation_type(&self) -> DeliverySimulationType {
        match self {
            DeliveryRequest::Send(_) | DeliveryRequest::Cancel(_) => DeliverySimulationType::Commit,
            DeliveryRequest::Status(_) | DeliveryRequest::ProviderHealthCheck(_) => {
                DeliverySimulationType::Draft
            }
        }
    }
}

impl Validate for DeliveryRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            DeliveryRequest::Send(v) => v.validate(),
            DeliveryRequest::Status(v) => v.validate(),
            DeliveryRequest::Cancel(v) => v.validate(),
            DeliveryRequest::ProviderHealthCheck(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1DeliveryRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: DeliverySimulationType,
    pub request: DeliveryRequest,
}

impl Ph1DeliveryRequest {
    pub fn send_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        tenant_id: TenantId,
        message_id: String,
        recipient: String,
        channel: DeliveryChannel,
        payload_ref: String,
        provider_ref: String,
        simulation_context: String,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1DELIVERY_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: DELIVERY_SEND_COMMIT.to_string(),
            simulation_type: DeliverySimulationType::Commit,
            request: DeliveryRequest::Send(DeliverySendRequest {
                tenant_id,
                message_id,
                recipient,
                channel,
                payload_ref,
                provider_ref,
                simulation_context,
                idempotency_key,
            }),
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for Ph1DeliveryRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DELIVERY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_request.schema_version",
                reason: "must match PH1DELIVERY_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_request.now",
                reason: "must be > 0",
            });
        }
        validate_token(
            "ph1delivery_request.simulation_id",
            &self.simulation_id,
            128,
        )?;
        self.request.validate()?;
        if self.simulation_id != self.request.expected_simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_request.simulation_id",
                reason: "must match request variant simulation id",
            });
        }
        if self.simulation_type != self.request.expected_simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_request.simulation_type",
                reason: "must match request variant simulation type",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliverySendResult {
    pub delivery_attempt_id: String,
    pub delivery_proof_ref: String,
    pub delivery_status: DeliveryStatus,
    pub provider_message_ref: Option<String>,
    pub reason_code: ReasonCodeId,
}

impl Validate for DeliverySendResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "delivery_send_result.delivery_attempt_id",
            &self.delivery_attempt_id,
            128,
        )?;
        validate_token(
            "delivery_send_result.delivery_proof_ref",
            &self.delivery_proof_ref,
            256,
        )?;
        validate_opt_token(
            "delivery_send_result.provider_message_ref",
            &self.provider_message_ref,
            256,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "delivery_send_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryStatusResult {
    pub normalized_status: DeliveryStatus,
    pub provider_status_raw: String,
    pub reason_code: ReasonCodeId,
}

impl Validate for DeliveryStatusResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token(
            "delivery_status_result.provider_status_raw",
            &self.provider_status_raw,
            256,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "delivery_status_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryCancelResult {
    pub canceled: bool,
    pub reason_code: ReasonCodeId,
}

impl Validate for DeliveryCancelResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "delivery_cancel_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryProviderHealthCheckResult {
    pub provider_health_state: DeliveryProviderHealthState,
    pub latency_bucket: DeliveryLatencyBucket,
    pub reason_code: ReasonCodeId,
}

impl Validate for DeliveryProviderHealthCheckResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "delivery_provider_health_check_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Send(DeliverySendResult),
    Status(DeliveryStatusResult),
    Cancel(DeliveryCancelResult),
    ProviderHealthCheck(DeliveryProviderHealthCheckResult),
}

impl DeliveryOutcome {
    pub fn capability_id(&self) -> DeliveryCapabilityId {
        match self {
            DeliveryOutcome::Send(_) => DeliveryCapabilityId::Send,
            DeliveryOutcome::Status(_) => DeliveryCapabilityId::Status,
            DeliveryOutcome::Cancel(_) => DeliveryCapabilityId::Cancel,
            DeliveryOutcome::ProviderHealthCheck(_) => DeliveryCapabilityId::ProviderHealthCheck,
        }
    }
}

impl Validate for DeliveryOutcome {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            DeliveryOutcome::Send(v) => v.validate(),
            DeliveryOutcome::Status(v) => v.validate(),
            DeliveryOutcome::Cancel(v) => v.validate(),
            DeliveryOutcome::ProviderHealthCheck(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1DeliveryOk {
    pub capability_id: DeliveryCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub outcome: DeliveryOutcome,
    pub simulation_gate_passed: bool,
}

impl Ph1DeliveryOk {
    pub fn v1(
        capability_id: DeliveryCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        outcome: DeliveryOutcome,
        simulation_gate_passed: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            capability_id,
            simulation_id,
            reason_code,
            outcome,
            simulation_gate_passed,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for Ph1DeliveryOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("ph1delivery_ok.simulation_id", &self.simulation_id, 128)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_ok.reason_code",
                reason: "must be > 0",
            });
        }
        self.outcome.validate()?;
        if self.outcome.capability_id() != self.capability_id {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_ok.outcome",
                reason: "outcome capability does not match capability_id",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1DeliveryRefuse {
    pub capability_id: DeliveryCapabilityId,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub reason: String,
}

impl Ph1DeliveryRefuse {
    pub fn v1(
        capability_id: DeliveryCapabilityId,
        simulation_id: String,
        reason_code: ReasonCodeId,
        reason: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            capability_id,
            simulation_id,
            reason_code,
            reason,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1DeliveryRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("ph1delivery_refuse.simulation_id", &self.simulation_id, 128)?;
        validate_token("ph1delivery_refuse.reason", &self.reason, 512)?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1delivery_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DeliveryResponse {
    Ok(Ph1DeliveryOk),
    Refuse(Ph1DeliveryRefuse),
}

impl Validate for Ph1DeliveryResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1DeliveryResponse::Ok(v) => v.validate(),
            Ph1DeliveryResponse::Refuse(v) => v.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send_req() -> Ph1DeliveryRequest {
        Ph1DeliveryRequest::send_commit_v1(
            CorrelationId(11),
            TurnId(22),
            MonotonicTimeNs(33),
            TenantId::new("tenant_a").unwrap(),
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
    fn at_delivery_contract_01_send_commit_constructor_is_valid() {
        assert!(send_req().validate().is_ok());
    }

    #[test]
    fn at_delivery_contract_02_simulation_id_mismatch_fails_closed() {
        let mut req = send_req();
        req.simulation_id = DELIVERY_CANCEL_COMMIT.to_string();
        assert!(matches!(
            req.validate(),
            Err(ContractViolation::InvalidValue {
                field: "ph1delivery_request.simulation_id",
                ..
            })
        ));
    }

    #[test]
    fn at_delivery_contract_03_outcome_capability_mismatch_fails_closed() {
        let bad = Ph1DeliveryOk {
            capability_id: DeliveryCapabilityId::Status,
            simulation_id: DELIVERY_SEND_COMMIT.to_string(),
            reason_code: ReasonCodeId(1),
            outcome: DeliveryOutcome::Send(DeliverySendResult {
                delivery_attempt_id: "attempt_1".to_string(),
                delivery_proof_ref: "proof_1".to_string(),
                delivery_status: DeliveryStatus::Sent,
                provider_message_ref: Some("provider_msg_1".to_string()),
                reason_code: ReasonCodeId(1),
            }),
            simulation_gate_passed: true,
        };
        assert!(matches!(
            bad.validate(),
            Err(ContractViolation::InvalidValue {
                field: "ph1delivery_ok.outcome",
                ..
            })
        ));
    }
}
