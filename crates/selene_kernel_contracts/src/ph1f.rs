#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1l::SessionId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1F_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConversationTurnId(pub u64);

impl Validate for ConversationTurnId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversationRole {
    User,
    Selene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversationSource {
    VoiceTranscript,
    TypedText,
    SeleneOutput,
    Tombstone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyScope {
    PublicChat,
    PrivateDelivery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationTurnInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub device_id: Option<DeviceId>,
    pub role: ConversationRole,
    pub source: ConversationSource,
    pub text: String,
    pub text_hash: String,
    pub privacy_scope: PrivacyScope,
    /// Optional key to dedupe storage writes on retries (PH1.F invariant).
    pub idempotency_key: Option<String>,
    /// Required when source=Tombstone: references the original conversation_turn_id.
    pub tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
    /// Required when source=Tombstone.
    pub tombstone_reason_code: Option<ReasonCodeId>,
}

impl ConversationTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: Option<DeviceId>,
        role: ConversationRole,
        source: ConversationSource,
        text: String,
        text_hash: String,
        privacy_scope: PrivacyScope,
        idempotency_key: Option<String>,
        tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
        tombstone_reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let t = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            created_at,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            role,
            source,
            text,
            text_hash,
            privacy_scope,
            idempotency_key,
            tombstone_of_conversation_turn_id,
            tombstone_reason_code,
        };
        t.validate()?;
        Ok(t)
    }
}

impl Validate for ConversationTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1F_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.schema_version",
                reason: "must match PH1F_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.created_at",
                reason: "must be > 0",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if let Some(s) = self.session_id {
            if s.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if self.user_id.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.user_id",
                reason: "must not be empty",
            });
        }
        if let Some(d) = &self.device_id {
            d.validate()?;
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 8192 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                reason: "must be <= 8192 chars",
            });
        }
        if self.text_hash.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text_hash",
                reason: "must not be empty",
            });
        }
        if self.text_hash.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text_hash",
                reason: "must be <= 128 chars",
            });
        }
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }

        match self.source {
            ConversationSource::VoiceTranscript | ConversationSource::TypedText => {
                if self.role != ConversationRole::User {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be USER for voice_transcript/typed_text",
                    });
                }
                if self.tombstone_of_conversation_turn_id.is_some()
                    || self.tombstone_reason_code.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input",
                        reason: "tombstone fields must be None unless source=Tombstone",
                    });
                }
            }
            ConversationSource::SeleneOutput => {
                if self.role != ConversationRole::Selene {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be SELENE for selene_output",
                    });
                }
                if self.tombstone_of_conversation_turn_id.is_some()
                    || self.tombstone_reason_code.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input",
                        reason: "tombstone fields must be None unless source=Tombstone",
                    });
                }
            }
            ConversationSource::Tombstone => {
                if self.role != ConversationRole::Selene {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be SELENE for tombstone",
                    });
                }
                if self.text != "[REDACTED]" {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.text",
                        reason: "tombstone text must be the fixed placeholder [REDACTED]",
                    });
                }
                let Some(id) = self.tombstone_of_conversation_turn_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_of_conversation_turn_id",
                        reason: "required for tombstone",
                    });
                };
                id.validate()?;
                let Some(rc) = self.tombstone_reason_code else {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_reason_code",
                        reason: "required for tombstone",
                    });
                };
                if rc.0 == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_reason_code",
                        reason: "must be > 0",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationTurnRecord {
    pub schema_version: SchemaVersion,
    pub conversation_turn_id: ConversationTurnId,
    pub created_at: MonotonicTimeNs,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub device_id: Option<DeviceId>,
    pub role: ConversationRole,
    pub source: ConversationSource,
    pub text: String,
    pub text_hash: String,
    pub privacy_scope: PrivacyScope,
    pub idempotency_key: Option<String>,
    pub tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
    pub tombstone_reason_code: Option<ReasonCodeId>,
}

impl ConversationTurnRecord {
    pub fn from_input_v1(
        conversation_turn_id: ConversationTurnId,
        input: ConversationTurnInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        conversation_turn_id.validate()?;
        let r = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            conversation_turn_id,
            created_at: input.created_at,
            correlation_id: input.correlation_id,
            turn_id: input.turn_id,
            session_id: input.session_id,
            user_id: input.user_id,
            device_id: input.device_id,
            role: input.role,
            source: input.source,
            text: input.text,
            text_hash: input.text_hash,
            privacy_scope: input.privacy_scope,
            idempotency_key: input.idempotency_key,
            tombstone_of_conversation_turn_id: input.tombstone_of_conversation_turn_id,
            tombstone_reason_code: input.tombstone_reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ConversationTurnRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1F_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_record.schema_version",
                reason: "must match PH1F_CONTRACT_VERSION",
            });
        }
        self.conversation_turn_id.validate()?;
        // Reuse input validation for the rest of the rules.
        ConversationTurnInput {
            schema_version: self.schema_version,
            created_at: self.created_at,
            correlation_id: self.correlation_id,
            turn_id: self.turn_id,
            session_id: self.session_id,
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            role: self.role,
            source: self.source,
            text: self.text.clone(),
            text_hash: self.text_hash.clone(),
            privacy_scope: self.privacy_scope,
            idempotency_key: self.idempotency_key.clone(),
            tombstone_of_conversation_turn_id: self.tombstone_of_conversation_turn_id,
            tombstone_reason_code: self.tombstone_reason_code,
        }
        .validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> ConversationTurnInput {
        ConversationTurnInput::v1(
            MonotonicTimeNs(1),
            CorrelationId(9601),
            TurnId(1),
            None,
            UserId::new("f_contract_user_1").unwrap(),
            None,
            ConversationRole::User,
            ConversationSource::TypedText,
            "hello".to_string(),
            "hash_1".to_string(),
            PrivacyScope::PublicChat,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_f_contract_01_typed_text_requires_user_role() {
        let mut input = base_input();
        input.role = ConversationRole::Selene;
        assert!(matches!(
            input.validate(),
            Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.role",
                ..
            })
        ));
    }

    #[test]
    fn at_f_contract_02_tombstone_requires_placeholder_and_reason() {
        let mut input = base_input();
        input.role = ConversationRole::Selene;
        input.source = ConversationSource::Tombstone;
        input.text = "not redacted".to_string();
        input.tombstone_of_conversation_turn_id = Some(ConversationTurnId(10));
        input.tombstone_reason_code = Some(ReasonCodeId(1));

        assert!(matches!(
            input.validate(),
            Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                ..
            })
        ));
    }

    #[test]
    fn at_f_contract_03_record_from_input_roundtrip_is_valid() {
        let input = base_input();
        let record = ConversationTurnRecord::from_input_v1(ConversationTurnId(1), input).unwrap();
        assert_eq!(record.conversation_turn_id, ConversationTurnId(1));
        assert_eq!(record.source, ConversationSource::TypedText);
        assert!(record.validate().is_ok());
    }
}
