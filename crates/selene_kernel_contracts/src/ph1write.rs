#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1l::SessionId;
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1WRITE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1WRITE_ENGINE_ID: &str = "PH1.WRITE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WriteFormatMode {
    FormattedText,
    FallbackOriginal,
}

impl WriteFormatMode {
    pub fn as_str(self) -> &'static str {
        match self {
            WriteFormatMode::FormattedText => "FORMATTED_TEXT",
            WriteFormatMode::FallbackOriginal => "FALLBACK_ORIGINAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WriteRenderStyle {
    Professional,
    Preserve,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CriticalToken(String);

impl CriticalToken {
    pub fn new(token: impl Into<String>) -> Result<Self, ContractViolation> {
        let token = token.into();
        let t = Self(token);
        t.validate()?;
        Ok(t)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for CriticalToken {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "critical_token",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "critical_token",
                reason: "must be <= 64 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "critical_token",
                reason: "must not contain control characters",
            });
        }
        if self.0.chars().any(|c| c.is_whitespace()) {
            return Err(ContractViolation::InvalidValue {
                field: "critical_token",
                reason: "must not contain whitespace",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1WriteRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub response_text: String,
    pub render_style: WriteRenderStyle,
    pub critical_tokens: Vec<CriticalToken>,
    pub is_refusal_or_policy_text: bool,
    pub idempotency_key: String,
}

impl Ph1WriteRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        now: MonotonicTimeNs,
        tenant_id: TenantId,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: DeviceId,
        response_text: String,
        render_style: WriteRenderStyle,
        critical_tokens: Vec<CriticalToken>,
        is_refusal_or_policy_text: bool,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1WRITE_CONTRACT_VERSION,
            now,
            tenant_id,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            response_text,
            render_style,
            critical_tokens,
            is_refusal_or_policy_text,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1WriteRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WRITE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.schema_version",
                reason: "must match PH1WRITE_CONTRACT_VERSION",
            });
        }
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.now",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if let Some(s) = self.session_id {
            if s.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1write_request.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if self.user_id.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.user_id",
                reason: "must not be empty",
            });
        }
        if self.user_id.as_str().len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.user_id",
                reason: "must be <= 128 chars",
            });
        }
        self.device_id.validate()?;

        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.response_text",
                reason: "must not be empty",
            });
        }
        if self.response_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.response_text",
                reason: "must be <= 32768 chars",
            });
        }

        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }

        if self.critical_tokens.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_request.critical_tokens",
                reason: "must be <= 32 entries",
            });
        }
        let mut seen = BTreeSet::new();
        for token in &self.critical_tokens {
            token.validate()?;
            if !self.response_text.contains(token.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1write_request.critical_tokens",
                    reason: "every token must exist in response_text",
                });
            }
            if !seen.insert(token.as_str().to_string()) {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1write_request.critical_tokens",
                    reason: "must not contain duplicates",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1WriteOk {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub formatted_text: String,
    pub formatted_text_hash: String,
    pub format_mode: WriteFormatMode,
    pub reason_code: ReasonCodeId,
    pub critical_tokens_preserved: bool,
}

impl Ph1WriteOk {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        formatted_text: String,
        format_mode: WriteFormatMode,
        reason_code: ReasonCodeId,
        critical_tokens_preserved: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1WRITE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            formatted_text_hash: deterministic_text_hash_64_hex(&formatted_text),
            formatted_text,
            format_mode,
            reason_code,
            critical_tokens_preserved,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1WriteOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WRITE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.schema_version",
                reason: "must match PH1WRITE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.formatted_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.formatted_text",
                reason: "must not be empty",
            });
        }
        if self.formatted_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.formatted_text",
                reason: "must be <= 32768 chars",
            });
        }
        if self.formatted_text_hash.len() != 16
            || !self
                .formatted_text_hash
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.formatted_text_hash",
                reason: "must be 16 hex chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.reason_code",
                reason: "must be > 0",
            });
        }
        if !self.critical_tokens_preserved {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.critical_tokens_preserved",
                reason: "must be true for OK output",
            });
        }
        if self.formatted_text_hash != deterministic_text_hash_64_hex(&self.formatted_text) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_ok.formatted_text_hash",
                reason: "must equal deterministic hash(formatted_text)",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1WriteRefuse {
    pub schema_version: SchemaVersion,
    pub reason_code: ReasonCodeId,
    pub refusal_text: String,
}

impl Ph1WriteRefuse {
    pub fn v1(reason_code: ReasonCodeId, refusal_text: String) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1WRITE_CONTRACT_VERSION,
            reason_code,
            refusal_text,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1WriteRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WRITE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_refuse.schema_version",
                reason: "must match PH1WRITE_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.refusal_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_refuse.refusal_text",
                reason: "must not be empty",
            });
        }
        if self.refusal_text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1write_refuse.refusal_text",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1WriteResponse {
    Ok(Ph1WriteOk),
    Refuse(Ph1WriteRefuse),
}

impl Validate for Ph1WriteResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1WriteResponse::Ok(v) => v.validate(),
            Ph1WriteResponse::Refuse(v) => v.validate(),
        }
    }
}

pub fn deterministic_text_hash_64_hex(input: &str) -> String {
    // FNV-1a 64-bit: deterministic and platform-independent.
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in input.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    if h == 0 {
        h = 1;
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1j::{CorrelationId, DeviceId, TurnId};

    fn req_base() -> Ph1WriteRequest {
        Ph1WriteRequest::v1(
            MonotonicTimeNs(10),
            TenantId::new("tenant_a").unwrap(),
            CorrelationId(1001),
            TurnId(2),
            None,
            UserId::new("tenant_a:user_1").unwrap(),
            DeviceId::new("tenant_a_device_1").unwrap(),
            "John owes $1200 on 2026-03-01 at 3:00pm.".to_string(),
            WriteRenderStyle::Professional,
            vec![
                CriticalToken::new("John").unwrap(),
                CriticalToken::new("$1200").unwrap(),
                CriticalToken::new("2026-03-01").unwrap(),
                CriticalToken::new("3:00pm").unwrap(),
            ],
            false,
            "write-contract-1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_write_contract_01_critical_tokens_must_exist_in_source_text() {
        let req = Ph1WriteRequest::v1(
            MonotonicTimeNs(10),
            TenantId::new("tenant_a").unwrap(),
            CorrelationId(1001),
            TurnId(2),
            None,
            UserId::new("tenant_a:user_1").unwrap(),
            DeviceId::new("tenant_a_device_1").unwrap(),
            "John owes $1200 on 2026-03-01 at 3:00pm.".to_string(),
            WriteRenderStyle::Professional,
            vec![CriticalToken::new("missing_token").unwrap()],
            false,
            "write-contract-1".to_string(),
        );

        assert!(matches!(
            req,
            Err(ContractViolation::InvalidValue {
                field: "ph1write_request.critical_tokens",
                ..
            })
        ));
    }

    #[test]
    fn at_write_contract_02_ok_hash_is_deterministic_and_validated() {
        let req = req_base();
        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            "John owes $1200 on 2026-03-01 at 3:00pm.".to_string(),
            WriteFormatMode::FormattedText,
            ReasonCodeId(0x5752_0001),
            true,
        )
        .unwrap();
        assert_eq!(
            ok.formatted_text_hash,
            deterministic_text_hash_64_hex(&ok.formatted_text)
        );
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn at_write_contract_03_response_enum_is_schema_valid() {
        let req = req_base();
        let ok = Ph1WriteOk::v1(
            req.correlation_id,
            req.turn_id,
            req.response_text.clone(),
            WriteFormatMode::FallbackOriginal,
            ReasonCodeId(0x5752_0002),
            true,
        )
        .unwrap();
        assert!(Ph1WriteResponse::Ok(ok).validate().is_ok());

        let refuse = Ph1WriteRefuse::v1(
            ReasonCodeId(0x5752_00F1),
            "invalid ph1write input".to_string(),
        )
        .unwrap();
        assert!(Ph1WriteResponse::Refuse(refuse).validate().is_ok());
    }
}
