#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1j::{
    AuditEvent, AuditEventId, AuditEventInput, CorrelationId, TurnId,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.J OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_J_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4A00_0101);
    pub const PH1_J_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4A00_01F1);
    pub const PH1_J_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4A00_01F2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1jWiringConfig {
    pub audit_enabled: bool,
    pub max_query_rows: u16,
}

impl Ph1jWiringConfig {
    pub fn mvp_v1(audit_enabled: bool) -> Self {
        Self {
            audit_enabled,
            max_query_rows: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuditOperation {
    Append(AuditEventInput),
    QueryByCorrelation { correlation_id: CorrelationId },
    QueryByTenant { tenant_id: String },
}

impl Validate for AuditOperation {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            AuditOperation::Append(input) => input.validate(),
            AuditOperation::QueryByCorrelation { correlation_id } => correlation_id.validate(),
            AuditOperation::QueryByTenant { tenant_id } => validate_tenant_id(tenant_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub operation: AuditOperation,
}

impl AuditTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        operation: AuditOperation,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            operation,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for AuditTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.operation.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuditTurnOutput {
    Appended { event_id: AuditEventId },
    QueryRows { rows: Vec<AuditEvent> },
}

impl Validate for AuditTurnOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            AuditTurnOutput::Appended { event_id } => event_id.validate(),
            AuditTurnOutput::QueryRows { rows } => {
                for row in rows {
                    row.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub output: AuditTurnOutput,
}

impl AuditForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        output: AuditTurnOutput,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            output,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for AuditForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.output.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditWiringRefuse {
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl AuditWiringRefuse {
    pub fn v1(reason_code: ReasonCodeId, message: String) -> Result<Self, ContractViolation> {
        let refuse = Self {
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for AuditWiringRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.message.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "audit_wiring_refuse.message",
                reason: "must be non-empty",
            });
        }
        if self.message.len() > 192 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_wiring_refuse.message",
                reason: "must be <= 192 chars",
            });
        }
        if !self.message.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "audit_wiring_refuse.message",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuditWiringOutcome {
    NotInvokedDisabled,
    Refused(AuditWiringRefuse),
    Forwarded(AuditForwardBundle),
}

pub trait Ph1AuditEngine {
    fn append_audit_row(
        &mut self,
        input: AuditEventInput,
    ) -> Result<AuditEventId, ContractViolation>;
    fn audit_rows_by_correlation(
        &self,
        correlation_id: CorrelationId,
    ) -> Result<Vec<AuditEvent>, ContractViolation>;
    fn audit_rows_by_tenant(&self, tenant_id: &str) -> Result<Vec<AuditEvent>, ContractViolation>;
}

#[derive(Debug, Clone)]
pub struct Ph1jWiring<E>
where
    E: Ph1AuditEngine,
{
    config: Ph1jWiringConfig,
    engine: E,
}

impl<E> Ph1jWiring<E>
where
    E: Ph1AuditEngine,
{
    pub fn new(config: Ph1jWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_query_rows == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1j_wiring_config.max_query_rows",
                reason: "must be > 0",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &mut self,
        input: &AuditTurnInput,
    ) -> Result<AuditWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.audit_enabled {
            return Ok(AuditWiringOutcome::NotInvokedDisabled);
        }

        match &input.operation {
            AuditOperation::Append(event_input) => {
                let event_id = match self.engine.append_audit_row(event_input.clone()) {
                    Ok(event_id) => event_id,
                    Err(_) => {
                        return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                            reason_codes::PH1_J_INTERNAL_PIPELINE_ERROR,
                            "audit append pipeline failed".to_string(),
                        )?));
                    }
                };
                let bundle = AuditForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    AuditTurnOutput::Appended { event_id },
                )?;
                Ok(AuditWiringOutcome::Forwarded(bundle))
            }
            AuditOperation::QueryByCorrelation { correlation_id } => {
                let rows = match self.engine.audit_rows_by_correlation(*correlation_id) {
                    Ok(rows) => rows,
                    Err(_) => {
                        return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                            reason_codes::PH1_J_INTERNAL_PIPELINE_ERROR,
                            "audit correlation query failed".to_string(),
                        )?));
                    }
                };
                if rows.len() > self.config.max_query_rows as usize {
                    return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                        reason_codes::PH1_J_BUDGET_EXCEEDED,
                        "audit query row budget exceeded".to_string(),
                    )?));
                }
                for row in &rows {
                    if row.validate().is_err() {
                        return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                            reason_codes::PH1_J_VALIDATION_FAILED,
                            "invalid audit row returned by engine".to_string(),
                        )?));
                    }
                }
                let bundle = AuditForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    AuditTurnOutput::QueryRows { rows },
                )?;
                Ok(AuditWiringOutcome::Forwarded(bundle))
            }
            AuditOperation::QueryByTenant { tenant_id } => {
                let rows = match self.engine.audit_rows_by_tenant(tenant_id) {
                    Ok(rows) => rows,
                    Err(_) => {
                        return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                            reason_codes::PH1_J_INTERNAL_PIPELINE_ERROR,
                            "audit tenant query failed".to_string(),
                        )?));
                    }
                };
                if rows.len() > self.config.max_query_rows as usize {
                    return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                        reason_codes::PH1_J_BUDGET_EXCEEDED,
                        "audit query row budget exceeded".to_string(),
                    )?));
                }
                for row in &rows {
                    if row.validate().is_err() {
                        return Ok(AuditWiringOutcome::Refused(AuditWiringRefuse::v1(
                            reason_codes::PH1_J_VALIDATION_FAILED,
                            "invalid audit row returned by engine".to_string(),
                        )?));
                    }
                }
                let bundle = AuditForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    AuditTurnOutput::QueryRows { rows },
                )?;
                Ok(AuditWiringOutcome::Forwarded(bundle))
            }
        }
    }
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), ContractViolation> {
    if tenant_id.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "audit_operation.query_by_tenant.tenant_id",
            reason: "must be non-empty",
        });
    }
    if tenant_id.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field: "audit_operation.query_by_tenant.tenant_id",
            reason: "must be <= 64 chars",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use selene_kernel_contracts::ph1j::{
        AuditEngine, AuditEventType, AuditPayloadMin, AuditSeverity, PayloadKey, PayloadValue,
    };
    use selene_kernel_contracts::MonotonicTimeNs;

    #[derive(Debug, Clone)]
    struct MockAuditEngine {
        append_result: Result<AuditEventId, ContractViolation>,
        corr_result: Result<Vec<AuditEvent>, ContractViolation>,
        tenant_result: Result<Vec<AuditEvent>, ContractViolation>,
    }

    impl Ph1AuditEngine for MockAuditEngine {
        fn append_audit_row(
            &mut self,
            _input: AuditEventInput,
        ) -> Result<AuditEventId, ContractViolation> {
            self.append_result.clone()
        }

        fn audit_rows_by_correlation(
            &self,
            _correlation_id: CorrelationId,
        ) -> Result<Vec<AuditEvent>, ContractViolation> {
            self.corr_result.clone()
        }

        fn audit_rows_by_tenant(
            &self,
            _tenant_id: &str,
        ) -> Result<Vec<AuditEvent>, ContractViolation> {
            self.tenant_result.clone()
        }
    }

    fn event_input(correlation_id: CorrelationId, turn_id: TurnId) -> AuditEventInput {
        let mut payload = BTreeMap::new();
        payload.insert(
            PayloadKey::new("gate").unwrap(),
            PayloadValue::new("policy").unwrap(),
        );
        AuditEventInput::v1(
            MonotonicTimeNs(1_000),
            Some("tenant_a".to_string()),
            Some("wo_1".to_string()),
            None,
            None,
            None,
            AuditEngine::Ph1J,
            AuditEventType::GatePass,
            ReasonCodeId(1),
            AuditSeverity::Info,
            correlation_id,
            turn_id,
            AuditPayloadMin::v1(payload).unwrap(),
            None,
            None,
        )
        .unwrap()
    }

    fn audit_event(id: u64, correlation_id: CorrelationId, turn_id: TurnId) -> AuditEvent {
        AuditEvent::from_input_v1(AuditEventId(id), event_input(correlation_id, turn_id)).unwrap()
    }

    #[test]
    fn at_j_06_wiring_disabled() {
        let mut wiring = Ph1jWiring::new(
            Ph1jWiringConfig::mvp_v1(false),
            MockAuditEngine {
                append_result: Ok(AuditEventId(1)),
                corr_result: Ok(vec![]),
                tenant_result: Ok(vec![]),
            },
        )
        .unwrap();
        let input = AuditTurnInput::v1(
            CorrelationId(9301),
            TurnId(1),
            AuditOperation::Append(event_input(CorrelationId(9301), TurnId(1))),
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, AuditWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_j_07_append_forwarded() {
        let mut wiring = Ph1jWiring::new(
            Ph1jWiringConfig::mvp_v1(true),
            MockAuditEngine {
                append_result: Ok(AuditEventId(42)),
                corr_result: Ok(vec![]),
                tenant_result: Ok(vec![]),
            },
        )
        .unwrap();
        let input = AuditTurnInput::v1(
            CorrelationId(9302),
            TurnId(2),
            AuditOperation::Append(event_input(CorrelationId(9302), TurnId(2))),
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        match out {
            AuditWiringOutcome::Forwarded(bundle) => match bundle.output {
                AuditTurnOutput::Appended { event_id } => assert_eq!(event_id, AuditEventId(42)),
                _ => panic!("expected appended output"),
            },
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_j_08_query_budget_exceeded_fails_closed() {
        let mut many_rows = Vec::new();
        for i in 1..=4 {
            many_rows.push(audit_event(i, CorrelationId(9303), TurnId(i as u64)));
        }
        let mut wiring = Ph1jWiring::new(
            Ph1jWiringConfig {
                audit_enabled: true,
                max_query_rows: 3,
            },
            MockAuditEngine {
                append_result: Ok(AuditEventId(1)),
                corr_result: Ok(many_rows),
                tenant_result: Ok(vec![]),
            },
        )
        .unwrap();

        let input = AuditTurnInput::v1(
            CorrelationId(9303),
            TurnId(3),
            AuditOperation::QueryByCorrelation {
                correlation_id: CorrelationId(9303),
            },
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        match out {
            AuditWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_J_BUDGET_EXCEEDED);
            }
            _ => panic!("expected refused outcome"),
        }
    }

    #[test]
    fn at_j_09_query_forwarded_with_valid_rows() {
        let rows = vec![
            audit_event(1, CorrelationId(9304), TurnId(1)),
            audit_event(2, CorrelationId(9304), TurnId(2)),
        ];
        let mut wiring = Ph1jWiring::new(
            Ph1jWiringConfig::mvp_v1(true),
            MockAuditEngine {
                append_result: Ok(AuditEventId(1)),
                corr_result: Ok(rows.clone()),
                tenant_result: Ok(rows.clone()),
            },
        )
        .unwrap();

        let corr_input = AuditTurnInput::v1(
            CorrelationId(9304),
            TurnId(4),
            AuditOperation::QueryByCorrelation {
                correlation_id: CorrelationId(9304),
            },
        )
        .unwrap();
        let corr_out = wiring.run_turn(&corr_input).unwrap();
        match corr_out {
            AuditWiringOutcome::Forwarded(bundle) => match bundle.output {
                AuditTurnOutput::QueryRows { rows } => assert_eq!(rows.len(), 2),
                _ => panic!("expected query rows output"),
            },
            _ => panic!("expected forwarded correlation query"),
        }

        let tenant_input = AuditTurnInput::v1(
            CorrelationId(9305),
            TurnId(5),
            AuditOperation::QueryByTenant {
                tenant_id: "tenant_a".to_string(),
            },
        )
        .unwrap();
        let tenant_out = wiring.run_turn(&tenant_input).unwrap();
        match tenant_out {
            AuditWiringOutcome::Forwarded(bundle) => match bundle.output {
                AuditTurnOutput::QueryRows { rows } => assert_eq!(rows.len(), 2),
                _ => panic!("expected query rows output"),
            },
            _ => panic!("expected forwarded tenant query"),
        }
    }
}
