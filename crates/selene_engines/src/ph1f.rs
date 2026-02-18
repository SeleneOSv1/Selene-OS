#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1f::{
    ConversationTurnId, ConversationTurnInput, ConversationTurnRecord,
};
use selene_kernel_contracts::ph1j::CorrelationId;
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.F reason-code namespace. Values are placeholders until global registry lock.
    pub const F_OK_APPEND_CONVERSATION: ReasonCodeId = ReasonCodeId(0x4600_0001);
    pub const F_APPEND_ONLY_VIOLATION: ReasonCodeId = ReasonCodeId(0x4600_00F1);
    pub const F_IDEMPOTENCY_REPLAY: ReasonCodeId = ReasonCodeId(0x4600_00F2);
    pub const F_CONTRACT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4600_00F3);
    pub const F_UNIQUE_CORRELATION_TURN_VIOLATION: ReasonCodeId = ReasonCodeId(0x4600_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1fConfig {
    pub max_conversation_rows: usize,
}

impl Ph1fConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_conversation_rows: 500_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1fRuntime {
    config: Ph1fConfig,
    conversation_rows: Vec<ConversationTurnRecord>,
    next_conversation_turn_id: u64,
    conversation_idempotency_index: BTreeMap<(CorrelationId, String), ConversationTurnId>,
    correlation_turn_index: BTreeSet<(CorrelationId, u64)>,
}

impl Ph1fRuntime {
    pub fn new(config: Ph1fConfig) -> Self {
        Self {
            config,
            conversation_rows: Vec::new(),
            next_conversation_turn_id: 1,
            conversation_idempotency_index: BTreeMap::new(),
            correlation_turn_index: BTreeSet::new(),
        }
    }

    pub fn append_conversation_row(
        &mut self,
        input: ConversationTurnInput,
    ) -> Result<ConversationTurnId, ContractViolation> {
        input.validate()?;
        if self.conversation_rows.len() >= self.config.max_conversation_rows {
            return Err(ContractViolation::InvalidValue {
                field: "ph1f_runtime.conversation_rows",
                reason: "max_conversation_rows exceeded",
            });
        }

        if let Some(idempotency_key) = &input.idempotency_key {
            if let Some(existing_id) = self
                .conversation_idempotency_index
                .get(&(input.correlation_id, idempotency_key.clone()))
            {
                return Ok(*existing_id);
            }
        }

        let unique_key = (input.correlation_id, input.turn_id.0);
        if self.correlation_turn_index.contains(&unique_key) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1f_runtime.conversation_rows",
                reason: "unique (correlation_id, turn_id) violated",
            });
        }

        let conversation_turn_id = ConversationTurnId(self.next_conversation_turn_id);
        self.next_conversation_turn_id = self.next_conversation_turn_id.saturating_add(1);

        let record = ConversationTurnRecord::from_input_v1(conversation_turn_id, input.clone())?;

        if let Some(idempotency_key) = &input.idempotency_key {
            self.conversation_idempotency_index.insert(
                (input.correlation_id, idempotency_key.clone()),
                conversation_turn_id,
            );
        }
        self.correlation_turn_index.insert(unique_key);
        self.conversation_rows.push(record);
        Ok(conversation_turn_id)
    }

    pub fn conversation_rows(&self) -> &[ConversationTurnRecord] {
        &self.conversation_rows
    }

    pub fn conversation_rows_by_correlation(
        &self,
        correlation_id: CorrelationId,
    ) -> Vec<ConversationTurnRecord> {
        self.conversation_rows
            .iter()
            .filter(|row| row.correlation_id == correlation_id)
            .cloned()
            .collect()
    }

    pub fn attempt_overwrite_conversation_turn(
        &self,
        _conversation_turn_id: ConversationTurnId,
    ) -> Result<(), ContractViolation> {
        Err(ContractViolation::InvalidValue {
            field: "conversation.conversation_ledger",
            reason: "append-only; overwrite is forbidden",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1f::{ConversationRole, ConversationSource, PrivacyScope};
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::MonotonicTimeNs;

    fn user_id() -> UserId {
        UserId::new("f_user_1").unwrap()
    }

    fn base_input(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        idempotency_key: Option<&str>,
    ) -> ConversationTurnInput {
        ConversationTurnInput::v1(
            MonotonicTimeNs(10),
            correlation_id,
            turn_id,
            None,
            user_id(),
            None,
            ConversationRole::User,
            ConversationSource::TypedText,
            "hello".to_string(),
            format!("hash_{}", turn_id.0),
            PrivacyScope::PublicChat,
            idempotency_key.map(ToString::to_string),
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_f_01_append_only_enforced() {
        let mut rt = Ph1fRuntime::new(Ph1fConfig::mvp_v1());
        let id = rt
            .append_conversation_row(base_input(CorrelationId(9401), TurnId(1), None))
            .unwrap();
        assert!(matches!(
            rt.attempt_overwrite_conversation_turn(id),
            Err(ContractViolation::InvalidValue { .. })
        ));
    }

    #[test]
    fn at_f_02_idempotency_dedupe_works() {
        let mut rt = Ph1fRuntime::new(Ph1fConfig::mvp_v1());
        let a = rt
            .append_conversation_row(base_input(CorrelationId(9402), TurnId(1), Some("conv_dup")))
            .unwrap();
        let b = rt
            .append_conversation_row(base_input(CorrelationId(9402), TurnId(1), Some("conv_dup")))
            .unwrap();

        assert_eq!(a, b);
        assert_eq!(rt.conversation_rows().len(), 1);
    }

    #[test]
    fn at_f_03_unique_correlation_turn_is_enforced() {
        let mut rt = Ph1fRuntime::new(Ph1fConfig::mvp_v1());
        let _ = rt
            .append_conversation_row(base_input(CorrelationId(9403), TurnId(1), None))
            .unwrap();
        let duplicate =
            rt.append_conversation_row(base_input(CorrelationId(9403), TurnId(1), None));
        assert!(duplicate.is_err());
    }

    #[test]
    fn at_f_04_correlation_query_returns_scoped_rows() {
        let mut rt = Ph1fRuntime::new(Ph1fConfig::mvp_v1());
        let _ = rt
            .append_conversation_row(base_input(CorrelationId(9404), TurnId(1), None))
            .unwrap();
        let _ = rt
            .append_conversation_row(base_input(CorrelationId(9404), TurnId(2), None))
            .unwrap();
        let _ = rt
            .append_conversation_row(base_input(CorrelationId(9405), TurnId(1), None))
            .unwrap();

        let rows = rt.conversation_rows_by_correlation(CorrelationId(9404));
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.correlation_id == CorrelationId(9404)));
    }
}
