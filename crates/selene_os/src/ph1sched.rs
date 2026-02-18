#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1sched::{
    Ph1SchedRequest, Ph1SchedResponse, SchedCapabilityId, SchedDecisionAction,
    SchedDecisionComputeOk, SchedDecisionComputeRequest, SchedPolicyEvaluateOk,
    SchedPolicyEvaluateRequest, SchedRefuse, SchedRequestEnvelope,
};
use selene_kernel_contracts::ph1work::WorkOrderId;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SCHED OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_SCHED_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5343_0101);
    pub const PH1_SCHED_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5343_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SchedWiringConfig {
    pub sched_enabled: bool,
    pub max_retryable_reason_codes: u8,
    pub max_diagnostics: u8,
    pub max_backoff_ms: u32,
}

impl Ph1SchedWiringConfig {
    pub fn mvp_v1(sched_enabled: bool) -> Self {
        Self {
            sched_enabled,
            max_retryable_reason_codes: 16,
            max_diagnostics: 8,
            max_backoff_ms: 300_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub step_id: String,
    pub now_ns: MonotonicTimeNs,
    pub step_started_at_ns: MonotonicTimeNs,
    pub timeout_ms: u32,
    pub max_retries: u16,
    pub retry_backoff_ms: u32,
    pub attempt_index: u16,
    pub last_failure_reason_code: Option<ReasonCodeId>,
    pub retryable_reason_codes: Vec<ReasonCodeId>,
    pub wait_is_pause_only: bool,
}

impl SchedTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        step_id: String,
        now_ns: MonotonicTimeNs,
        step_started_at_ns: MonotonicTimeNs,
        timeout_ms: u32,
        max_retries: u16,
        retry_backoff_ms: u32,
        attempt_index: u16,
        last_failure_reason_code: Option<ReasonCodeId>,
        retryable_reason_codes: Vec<ReasonCodeId>,
        wait_is_pause_only: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            work_order_id,
            step_id,
            now_ns,
            step_started_at_ns,
            timeout_ms,
            max_retries,
            retry_backoff_ms,
            attempt_index,
            last_failure_reason_code,
            retryable_reason_codes,
            wait_is_pause_only,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for SchedTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token("sched_turn_input.step_id", &self.step_id, 96)?;
        if self.now_ns.0 == 0 || self.step_started_at_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.now_ns",
                reason: "time values must be > 0",
            });
        }
        if self.now_ns.0 < self.step_started_at_ns.0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.now_ns",
                reason: "must be >= step_started_at_ns",
            });
        }
        if self.timeout_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.timeout_ms",
                reason: "must be > 0",
            });
        }
        if self.retry_backoff_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.retry_backoff_ms",
                reason: "must be > 0",
            });
        }
        if self.retryable_reason_codes.is_empty() || self.retryable_reason_codes.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.retryable_reason_codes",
                reason: "must contain 1..=32 reason codes",
            });
        }
        for reason in &self.retryable_reason_codes {
            if reason.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_turn_input.retryable_reason_codes",
                    reason: "must contain non-zero reason codes",
                });
            }
        }
        if !self.wait_is_pause_only {
            return Err(ContractViolation::InvalidValue {
                field: "sched_turn_input.wait_is_pause_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: SchedPolicyEvaluateOk,
    pub decision_compute: SchedDecisionComputeOk,
}

impl SchedForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: SchedPolicyEvaluateOk,
        decision_compute: SchedDecisionComputeOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            policy_evaluate,
            decision_compute,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for SchedForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;
        if self.decision_compute.action == SchedDecisionAction::Wait
            && self.decision_compute.attempt_next_index != self.decision_compute.attempt_index
        {
            return Err(ContractViolation::InvalidValue {
                field: "sched_forward_bundle.decision_compute.attempt_next_index",
                reason: "WAIT must not advance attempt index",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedWiringOutcome {
    NotInvokedDisabled,
    Refused(SchedRefuse),
    Forwarded(SchedForwardBundle),
}

pub trait Ph1SchedEngine {
    fn run(&self, req: &Ph1SchedRequest) -> Ph1SchedResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1SchedWiring<E>
where
    E: Ph1SchedEngine,
{
    config: Ph1SchedWiringConfig,
    engine: E,
}

impl<E> Ph1SchedWiring<E>
where
    E: Ph1SchedEngine,
{
    pub fn new(config: Ph1SchedWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_retryable_reason_codes == 0 || config.max_retryable_reason_codes > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1sched_wiring_config.max_retryable_reason_codes",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1sched_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if config.max_backoff_ms == 0 || config.max_backoff_ms > 86_400_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1sched_wiring_config.max_backoff_ms",
                reason: "must be within 1..=86_400_000",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &SchedTurnInput,
    ) -> Result<SchedWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.sched_enabled {
            return Ok(SchedWiringOutcome::NotInvokedDisabled);
        }

        let envelope = SchedRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_retryable_reason_codes, 32),
            min(self.config.max_diagnostics, 16),
            min(self.config.max_backoff_ms, 86_400_000),
        )?;

        let policy_req = Ph1SchedRequest::SchedPolicyEvaluate(SchedPolicyEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.work_order_id.clone(),
            input.step_id.clone(),
            input.now_ns,
            input.step_started_at_ns,
            input.timeout_ms,
            input.max_retries,
            input.retry_backoff_ms,
            input.attempt_index,
            input.last_failure_reason_code,
            input.retryable_reason_codes.clone(),
            input.wait_is_pause_only,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        if policy_resp.validate().is_err() {
            return Ok(SchedWiringOutcome::Refused(SchedRefuse::v1(
                SchedCapabilityId::SchedPolicyEvaluate,
                reason_codes::PH1_SCHED_VALIDATION_FAILED,
                "invalid sched policy response contract".to_string(),
            )?));
        }

        let policy_ok = match policy_resp {
            Ph1SchedResponse::Refuse(refuse) => return Ok(SchedWiringOutcome::Refused(refuse)),
            Ph1SchedResponse::SchedPolicyEvaluateOk(ok) => ok,
            Ph1SchedResponse::SchedDecisionComputeOk(_) => {
                return Ok(SchedWiringOutcome::Refused(SchedRefuse::v1(
                    SchedCapabilityId::SchedPolicyEvaluate,
                    reason_codes::PH1_SCHED_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1SchedRequest::SchedDecisionCompute(SchedDecisionComputeRequest::v1(
            envelope,
            policy_ok.tenant_id.clone(),
            policy_ok.work_order_id.clone(),
            policy_ok.step_id.clone(),
            input.now_ns,
            policy_ok.attempt_index,
            policy_ok.next_attempt_index,
            policy_ok.retry_backoff_ms,
            policy_ok.timeout_exceeded,
            policy_ok.max_retries_reached,
            policy_ok.retry_allowed,
            policy_ok.wait_is_pause_only,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        if decision_resp.validate().is_err() {
            return Ok(SchedWiringOutcome::Refused(SchedRefuse::v1(
                SchedCapabilityId::SchedDecisionCompute,
                reason_codes::PH1_SCHED_VALIDATION_FAILED,
                "invalid sched decision response contract".to_string(),
            )?));
        }

        let decision_ok = match decision_resp {
            Ph1SchedResponse::Refuse(refuse) => return Ok(SchedWiringOutcome::Refused(refuse)),
            Ph1SchedResponse::SchedDecisionComputeOk(ok) => ok,
            Ph1SchedResponse::SchedPolicyEvaluateOk(_) => {
                return Ok(SchedWiringOutcome::Refused(SchedRefuse::v1(
                    SchedCapabilityId::SchedDecisionCompute,
                    reason_codes::PH1_SCHED_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision request".to_string(),
                )?));
            }
        };

        if decision_ok.action == SchedDecisionAction::Wait
            && decision_ok.attempt_next_index != decision_ok.attempt_index
        {
            return Ok(SchedWiringOutcome::Refused(SchedRefuse::v1(
                SchedCapabilityId::SchedDecisionCompute,
                reason_codes::PH1_SCHED_VALIDATION_FAILED,
                "WAIT action advanced attempt index".to_string(),
            )?));
        }

        let bundle =
            SchedForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(SchedWiringOutcome::Forwarded(bundle))
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct DeterministicSchedEngine {
        force_invalid_wait_advance: bool,
    }

    impl Ph1SchedEngine for DeterministicSchedEngine {
        fn run(&self, req: &Ph1SchedRequest) -> Ph1SchedResponse {
            match req {
                Ph1SchedRequest::SchedPolicyEvaluate(r) => Ph1SchedResponse::SchedPolicyEvaluateOk(
                    SchedPolicyEvaluateOk::v1(
                        ReasonCodeId(51),
                        r.tenant_id.clone(),
                        r.work_order_id.clone(),
                        r.step_id.clone(),
                        r.attempt_index,
                        if r.last_failure_reason_code == Some(ReasonCodeId(11))
                            && r.attempt_index < r.max_retries
                        {
                            r.attempt_index + 1
                        } else {
                            r.attempt_index
                        },
                        false,
                        r.attempt_index >= r.max_retries,
                        r.last_failure_reason_code == Some(ReasonCodeId(11))
                            && r.attempt_index < r.max_retries,
                        r.retry_backoff_ms,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
                Ph1SchedRequest::SchedDecisionCompute(r) => {
                    let (action, next_due_at_ns, attempt_next_index) = if r.retry_allowed {
                        (
                            SchedDecisionAction::RetryAt,
                            Some(MonotonicTimeNs(
                                r.now_ns
                                    .0
                                    .saturating_add(u64::from(r.retry_backoff_ms) * 1_000_000),
                            )),
                            r.next_attempt_index,
                        )
                    } else if r.max_retries_reached {
                        (SchedDecisionAction::Fail, None, r.attempt_index)
                    } else {
                        (
                            SchedDecisionAction::Wait,
                            None,
                            if self.force_invalid_wait_advance {
                                r.attempt_index.saturating_add(1)
                            } else {
                                r.attempt_index
                            },
                        )
                    };
                    if self.force_invalid_wait_advance && action == SchedDecisionAction::Wait {
                        Ph1SchedResponse::SchedDecisionComputeOk(SchedDecisionComputeOk {
                            schema_version:
                                selene_kernel_contracts::ph1sched::PH1SCHED_CONTRACT_VERSION,
                            capability_id: SchedCapabilityId::SchedDecisionCompute,
                            reason_code: ReasonCodeId(52),
                            action,
                            attempt_index: r.attempt_index,
                            attempt_next_index: r.attempt_index.saturating_add(1),
                            next_due_at_ns: None,
                            wait_is_pause_only: true,
                            deterministic: true,
                        })
                    } else {
                        Ph1SchedResponse::SchedDecisionComputeOk(
                            SchedDecisionComputeOk::v1(
                                ReasonCodeId(52),
                                action,
                                r.attempt_index,
                                attempt_next_index,
                                next_due_at_ns,
                                true,
                                true,
                            )
                            .unwrap(),
                        )
                    }
                }
            }
        }
    }

    fn sample_input(
        last_failure_reason_code: Option<ReasonCodeId>,
        attempt_index: u16,
    ) -> SchedTurnInput {
        SchedTurnInput::v1(
            CorrelationId(9801),
            TurnId(2801),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_123").unwrap(),
            "step_1".to_string(),
            MonotonicTimeNs(2_000_000_000),
            MonotonicTimeNs(1_999_000_000),
            2_000,
            3,
            1_000,
            attempt_index,
            last_failure_reason_code,
            vec![ReasonCodeId(11), ReasonCodeId(12)],
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_sched_01_os_invokes_and_returns_retry_or_fail_or_wait() {
        let wiring = Ph1SchedWiring::new(
            Ph1SchedWiringConfig::mvp_v1(true),
            DeterministicSchedEngine {
                force_invalid_wait_advance: false,
            },
        )
        .unwrap();
        let out = wiring
            .run_turn(&sample_input(Some(ReasonCodeId(11)), 0))
            .unwrap();
        match out {
            SchedWiringOutcome::Forwarded(bundle) => {
                assert_eq!(bundle.decision_compute.action, SchedDecisionAction::RetryAt);
            }
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_sched_02_disabled_returns_not_invoked() {
        let wiring = Ph1SchedWiring::new(
            Ph1SchedWiringConfig::mvp_v1(false),
            DeterministicSchedEngine {
                force_invalid_wait_advance: false,
            },
        )
        .unwrap();
        let out = wiring
            .run_turn(&sample_input(Some(ReasonCodeId(11)), 0))
            .unwrap();
        assert_eq!(out, SchedWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_sched_03_wait_does_not_advance_and_is_forwarded() {
        let wiring = Ph1SchedWiring::new(
            Ph1SchedWiringConfig::mvp_v1(true),
            DeterministicSchedEngine {
                force_invalid_wait_advance: false,
            },
        )
        .unwrap();
        let out = wiring
            .run_turn(&sample_input(Some(ReasonCodeId(99)), 1))
            .unwrap();
        match out {
            SchedWiringOutcome::Forwarded(bundle) => {
                assert_eq!(bundle.decision_compute.action, SchedDecisionAction::Wait);
                assert_eq!(
                    bundle.decision_compute.attempt_next_index,
                    bundle.decision_compute.attempt_index
                );
            }
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_sched_04_wait_advance_is_refused() {
        let wiring = Ph1SchedWiring::new(
            Ph1SchedWiringConfig::mvp_v1(true),
            DeterministicSchedEngine {
                force_invalid_wait_advance: true,
            },
        )
        .unwrap();
        let out = wiring
            .run_turn(&sample_input(Some(ReasonCodeId(99)), 1))
            .unwrap();
        match out {
            SchedWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_SCHED_VALIDATION_FAILED
                );
                assert_eq!(
                    refuse.capability_id,
                    SchedCapabilityId::SchedDecisionCompute
                );
            }
            _ => panic!("expected refused outcome"),
        }
    }
}
