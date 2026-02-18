#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1f::{
    ConversationTurnId, ConversationTurnInput, ConversationTurnRecord,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.F OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_F_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4600_0101);
    pub const PH1_F_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4600_01F1);
    pub const PH1_F_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4600_01F2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1fWiringConfig {
    pub foundation_enabled: bool,
    pub max_query_rows: u16,
}

impl Ph1fWiringConfig {
    pub fn mvp_v1(foundation_enabled: bool) -> Self {
        Self {
            foundation_enabled,
            max_query_rows: 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FoundationOperation {
    AppendConversation(ConversationTurnInput),
    QueryConversationByCorrelation { correlation_id: CorrelationId },
}

impl Validate for FoundationOperation {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            FoundationOperation::AppendConversation(input) => input.validate(),
            FoundationOperation::QueryConversationByCorrelation { correlation_id } => {
                correlation_id.validate()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FoundationTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub operation: FoundationOperation,
}

impl FoundationTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        operation: FoundationOperation,
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

impl Validate for FoundationTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.operation.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FoundationTurnOutput {
    AppendedConversation {
        conversation_turn_id: ConversationTurnId,
    },
    QueryConversationRows {
        rows: Vec<ConversationTurnRecord>,
    },
}

impl Validate for FoundationTurnOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            FoundationTurnOutput::AppendedConversation {
                conversation_turn_id,
            } => conversation_turn_id.validate(),
            FoundationTurnOutput::QueryConversationRows { rows } => {
                for row in rows {
                    row.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FoundationForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub output: FoundationTurnOutput,
}

impl FoundationForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        output: FoundationTurnOutput,
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

impl Validate for FoundationForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.output.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationWiringRefuse {
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl FoundationWiringRefuse {
    pub fn v1(reason_code: ReasonCodeId, message: String) -> Result<Self, ContractViolation> {
        let refuse = Self {
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for FoundationWiringRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.message.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "foundation_wiring_refuse.message",
                reason: "must be non-empty",
            });
        }
        if self.message.len() > 192 {
            return Err(ContractViolation::InvalidValue {
                field: "foundation_wiring_refuse.message",
                reason: "must be <= 192 chars",
            });
        }
        if !self.message.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "foundation_wiring_refuse.message",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FoundationWiringOutcome {
    NotInvokedDisabled,
    Refused(FoundationWiringRefuse),
    Forwarded(FoundationForwardBundle),
}

pub trait Ph1FoundationEngine {
    fn append_conversation_row(
        &mut self,
        input: ConversationTurnInput,
    ) -> Result<ConversationTurnId, ContractViolation>;
    fn conversation_rows_by_correlation(
        &self,
        correlation_id: CorrelationId,
    ) -> Result<Vec<ConversationTurnRecord>, ContractViolation>;
}

#[derive(Debug, Clone)]
pub struct Ph1fWiring<E>
where
    E: Ph1FoundationEngine,
{
    config: Ph1fWiringConfig,
    engine: E,
}

impl<E> Ph1fWiring<E>
where
    E: Ph1FoundationEngine,
{
    pub fn new(config: Ph1fWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_query_rows == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1f_wiring_config.max_query_rows",
                reason: "must be > 0",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &mut self,
        input: &FoundationTurnInput,
    ) -> Result<FoundationWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.foundation_enabled {
            return Ok(FoundationWiringOutcome::NotInvokedDisabled);
        }

        match &input.operation {
            FoundationOperation::AppendConversation(turn_input) => {
                let conversation_turn_id =
                    match self.engine.append_conversation_row(turn_input.clone()) {
                        Ok(conversation_turn_id) => conversation_turn_id,
                        Err(_) => {
                            return Ok(FoundationWiringOutcome::Refused(
                                FoundationWiringRefuse::v1(
                                    reason_codes::PH1_F_INTERNAL_PIPELINE_ERROR,
                                    "ph1f append pipeline failed".to_string(),
                                )?,
                            ));
                        }
                    };
                let bundle = FoundationForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    FoundationTurnOutput::AppendedConversation {
                        conversation_turn_id,
                    },
                )?;
                Ok(FoundationWiringOutcome::Forwarded(bundle))
            }
            FoundationOperation::QueryConversationByCorrelation { correlation_id } => {
                let rows = match self
                    .engine
                    .conversation_rows_by_correlation(*correlation_id)
                {
                    Ok(rows) => rows,
                    Err(_) => {
                        return Ok(FoundationWiringOutcome::Refused(
                            FoundationWiringRefuse::v1(
                                reason_codes::PH1_F_INTERNAL_PIPELINE_ERROR,
                                "ph1f query pipeline failed".to_string(),
                            )?,
                        ));
                    }
                };
                if rows.len() > self.config.max_query_rows as usize {
                    return Ok(FoundationWiringOutcome::Refused(
                        FoundationWiringRefuse::v1(
                            reason_codes::PH1_F_BUDGET_EXCEEDED,
                            "ph1f query row budget exceeded".to_string(),
                        )?,
                    ));
                }
                for row in &rows {
                    if row.validate().is_err() {
                        return Ok(FoundationWiringOutcome::Refused(
                            FoundationWiringRefuse::v1(
                                reason_codes::PH1_F_VALIDATION_FAILED,
                                "invalid ph1f row returned by engine".to_string(),
                            )?,
                        ));
                    }
                }
                let bundle = FoundationForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    FoundationTurnOutput::QueryConversationRows { rows },
                )?;
                Ok(FoundationWiringOutcome::Forwarded(bundle))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1f::{ConversationRole, ConversationSource, PrivacyScope};
    use selene_kernel_contracts::MonotonicTimeNs;

    #[derive(Debug, Clone)]
    struct MockFoundationEngine {
        append_result: Result<ConversationTurnId, ContractViolation>,
        query_result: Result<Vec<ConversationTurnRecord>, ContractViolation>,
    }

    impl Ph1FoundationEngine for MockFoundationEngine {
        fn append_conversation_row(
            &mut self,
            _input: ConversationTurnInput,
        ) -> Result<ConversationTurnId, ContractViolation> {
            self.append_result.clone()
        }

        fn conversation_rows_by_correlation(
            &self,
            _correlation_id: CorrelationId,
        ) -> Result<Vec<ConversationTurnRecord>, ContractViolation> {
            self.query_result.clone()
        }
    }

    fn conversation_input(correlation_id: CorrelationId, turn_id: TurnId) -> ConversationTurnInput {
        ConversationTurnInput::v1(
            MonotonicTimeNs(1_000),
            correlation_id,
            turn_id,
            None,
            UserId::new("f_os_user_1").unwrap(),
            None,
            ConversationRole::User,
            ConversationSource::TypedText,
            "hello".to_string(),
            format!("hash_{}", turn_id.0),
            PrivacyScope::PublicChat,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn conversation_record(
        id: u64,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> ConversationTurnRecord {
        ConversationTurnRecord::from_input_v1(
            ConversationTurnId(id),
            conversation_input(correlation_id, turn_id),
        )
        .unwrap()
    }

    #[test]
    fn at_f_05_wiring_disabled() {
        let mut wiring = Ph1fWiring::new(
            Ph1fWiringConfig::mvp_v1(false),
            MockFoundationEngine {
                append_result: Ok(ConversationTurnId(1)),
                query_result: Ok(vec![]),
            },
        )
        .unwrap();
        let input = FoundationTurnInput::v1(
            CorrelationId(9501),
            TurnId(1),
            FoundationOperation::AppendConversation(conversation_input(
                CorrelationId(9501),
                TurnId(1),
            )),
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, FoundationWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_f_06_append_forwarded() {
        let mut wiring = Ph1fWiring::new(
            Ph1fWiringConfig::mvp_v1(true),
            MockFoundationEngine {
                append_result: Ok(ConversationTurnId(42)),
                query_result: Ok(vec![]),
            },
        )
        .unwrap();
        let input = FoundationTurnInput::v1(
            CorrelationId(9502),
            TurnId(2),
            FoundationOperation::AppendConversation(conversation_input(
                CorrelationId(9502),
                TurnId(2),
            )),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FoundationWiringOutcome::Forwarded(bundle) => match bundle.output {
                FoundationTurnOutput::AppendedConversation {
                    conversation_turn_id,
                } => assert_eq!(conversation_turn_id, ConversationTurnId(42)),
                _ => panic!("expected appended output"),
            },
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_f_07_query_budget_exceeded_fails_closed() {
        let rows = vec![
            conversation_record(1, CorrelationId(9503), TurnId(1)),
            conversation_record(2, CorrelationId(9503), TurnId(2)),
            conversation_record(3, CorrelationId(9503), TurnId(3)),
        ];
        let mut wiring = Ph1fWiring::new(
            Ph1fWiringConfig {
                foundation_enabled: true,
                max_query_rows: 2,
            },
            MockFoundationEngine {
                append_result: Ok(ConversationTurnId(1)),
                query_result: Ok(rows),
            },
        )
        .unwrap();
        let input = FoundationTurnInput::v1(
            CorrelationId(9503),
            TurnId(3),
            FoundationOperation::QueryConversationByCorrelation {
                correlation_id: CorrelationId(9503),
            },
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FoundationWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_F_BUDGET_EXCEEDED);
            }
            _ => panic!("expected refused outcome"),
        }
    }

    #[test]
    fn at_f_08_query_forwarded() {
        let rows = vec![
            conversation_record(1, CorrelationId(9504), TurnId(1)),
            conversation_record(2, CorrelationId(9504), TurnId(2)),
        ];
        let mut wiring = Ph1fWiring::new(
            Ph1fWiringConfig::mvp_v1(true),
            MockFoundationEngine {
                append_result: Ok(ConversationTurnId(1)),
                query_result: Ok(rows),
            },
        )
        .unwrap();
        let input = FoundationTurnInput::v1(
            CorrelationId(9504),
            TurnId(4),
            FoundationOperation::QueryConversationByCorrelation {
                correlation_id: CorrelationId(9504),
            },
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FoundationWiringOutcome::Forwarded(bundle) => match bundle.output {
                FoundationTurnOutput::QueryConversationRows { rows } => {
                    assert_eq!(rows.len(), 2);
                    assert!(rows.iter().all(|r| r.correlation_id == CorrelationId(9504)));
                }
                _ => panic!("expected query rows output"),
            },
            _ => panic!("expected forwarded outcome"),
        }
    }
}
