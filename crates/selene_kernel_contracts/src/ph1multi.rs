#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1MULTI_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultiCapabilityId {
    MultiBundleCompose,
    MultiSignalAlign,
}

impl MultiCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            MultiCapabilityId::MultiBundleCompose => "MULTI_BUNDLE_COMPOSE",
            MultiCapabilityId::MultiSignalAlign => "MULTI_SIGNAL_ALIGN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultiModality {
    Voice,
    Text,
    Vision,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultiValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signals: u8,
    pub max_bundle_items: u8,
    pub privacy_scope_required: bool,
}

impl MultiRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signals: u8,
        max_bundle_items: u8,
        privacy_scope_required: bool,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signals,
            max_bundle_items,
            privacy_scope_required,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for MultiRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signals == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.max_signals",
                reason: "must be > 0",
            });
        }
        if self.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.max_signals",
                reason: "must be <= 64",
            });
        }
        if self.max_bundle_items == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.max_bundle_items",
                reason: "must be > 0",
            });
        }
        if self.max_bundle_items > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.max_bundle_items",
                reason: "must be <= 32",
            });
        }
        if !self.privacy_scope_required {
            return Err(ContractViolation::InvalidValue {
                field: "multi_request_envelope.privacy_scope_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSourceSignal {
    pub schema_version: SchemaVersion,
    pub signal_id: String,
    pub source_engine: String,
    pub modality: MultiModality,
    pub hint_key: String,
    pub hint_value: String,
    pub evidence_ref: Option<String>,
    pub confidence_pct: u8,
    pub privacy_scoped: bool,
}

impl MultiSourceSignal {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        signal_id: String,
        source_engine: String,
        modality: MultiModality,
        hint_key: String,
        hint_value: String,
        evidence_ref: Option<String>,
        confidence_pct: u8,
        privacy_scoped: bool,
    ) -> Result<Self, ContractViolation> {
        let signal = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            signal_id,
            source_engine,
            modality,
            hint_key,
            hint_value,
            evidence_ref,
            confidence_pct,
            privacy_scoped,
        };
        signal.validate()?;
        Ok(signal)
    }
}

impl Validate for MultiSourceSignal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_source_signal.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        validate_token("multi_source_signal.signal_id", &self.signal_id, 64)?;
        validate_engine_id("multi_source_signal.source_engine", &self.source_engine)?;
        validate_field_key("multi_source_signal.hint_key", &self.hint_key)?;
        validate_token("multi_source_signal.hint_value", &self.hint_value, 160)?;
        if let Some(evidence_ref) = &self.evidence_ref {
            validate_token("multi_source_signal.evidence_ref", evidence_ref, 128)?;
        }
        if !self.privacy_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "multi_source_signal.privacy_scoped",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiBundleComposeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: MultiRequestEnvelope,
    pub signals: Vec<MultiSourceSignal>,
    pub include_vision: bool,
}

impl MultiBundleComposeRequest {
    pub fn v1(
        envelope: MultiRequestEnvelope,
        signals: Vec<MultiSourceSignal>,
        include_vision: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            envelope,
            signals,
            include_vision,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for MultiBundleComposeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_request.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.signals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_request.signals",
                reason: "must not be empty",
            });
        }
        if self.signals.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_request.signals",
                reason: "must be <= 64",
            });
        }

        let mut signal_ids = BTreeSet::new();
        for signal in &self.signals {
            signal.validate()?;
            if !signal_ids.insert(signal.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "multi_bundle_compose_request.signals",
                    reason: "signal_id must be unique",
                });
            }
        }

        if self.include_vision
            && !self
                .signals
                .iter()
                .any(|signal| signal.modality == MultiModality::Vision)
        {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_request.include_vision",
                reason: "requires at least one vision modality signal",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiBundleItem {
    pub schema_version: SchemaVersion,
    pub signal_id: String,
    pub source_engine: String,
    pub modality: MultiModality,
    pub fused_rank: u8,
    pub confidence_pct: u8,
    pub evidence_ref: Option<String>,
}

impl MultiBundleItem {
    pub fn v1(
        signal_id: String,
        source_engine: String,
        modality: MultiModality,
        fused_rank: u8,
        confidence_pct: u8,
        evidence_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let item = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            signal_id,
            source_engine,
            modality,
            fused_rank,
            confidence_pct,
            evidence_ref,
        };
        item.validate()?;
        Ok(item)
    }
}

impl Validate for MultiBundleItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_item.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        validate_token("multi_bundle_item.signal_id", &self.signal_id, 64)?;
        validate_engine_id("multi_bundle_item.source_engine", &self.source_engine)?;
        if self.fused_rank == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_item.fused_rank",
                reason: "must be > 0",
            });
        }
        if let Some(evidence_ref) = &self.evidence_ref {
            validate_token("multi_bundle_item.evidence_ref", evidence_ref, 128)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSignalAlignRequest {
    pub schema_version: SchemaVersion,
    pub envelope: MultiRequestEnvelope,
    pub selected_signal_id: String,
    pub ordered_bundle_items: Vec<MultiBundleItem>,
}

impl MultiSignalAlignRequest {
    pub fn v1(
        envelope: MultiRequestEnvelope,
        selected_signal_id: String,
        ordered_bundle_items: Vec<MultiBundleItem>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            envelope,
            selected_signal_id,
            ordered_bundle_items,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for MultiSignalAlignRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_request.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "multi_signal_align_request.selected_signal_id",
            &self.selected_signal_id,
            64,
        )?;
        if self.ordered_bundle_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_request.ordered_bundle_items",
                reason: "must not be empty",
            });
        }
        if self.ordered_bundle_items.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_request.ordered_bundle_items",
                reason: "must be <= 32",
            });
        }

        let mut signal_ids = BTreeSet::new();
        for item in &self.ordered_bundle_items {
            item.validate()?;
            if !signal_ids.insert(item.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "multi_signal_align_request.ordered_bundle_items",
                    reason: "signal_id must be unique",
                });
            }
        }

        if !self
            .ordered_bundle_items
            .iter()
            .any(|item| item.signal_id == self.selected_signal_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_request.selected_signal_id",
                reason: "must exist in ordered_bundle_items",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1MultiRequest {
    MultiBundleCompose(MultiBundleComposeRequest),
    MultiSignalAlign(MultiSignalAlignRequest),
}

impl Validate for Ph1MultiRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1MultiRequest::MultiBundleCompose(req) => req.validate(),
            Ph1MultiRequest::MultiSignalAlign(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiBundleComposeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: MultiCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_signal_id: String,
    pub ordered_bundle_items: Vec<MultiBundleItem>,
    pub evidence_backed: bool,
    pub privacy_scoped: bool,
    pub no_execution_authority: bool,
}

impl MultiBundleComposeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_signal_id: String,
        ordered_bundle_items: Vec<MultiBundleItem>,
        evidence_backed: bool,
        privacy_scoped: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            capability_id: MultiCapabilityId::MultiBundleCompose,
            reason_code,
            selected_signal_id,
            ordered_bundle_items,
            evidence_backed,
            privacy_scoped,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for MultiBundleComposeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        if self.capability_id != MultiCapabilityId::MultiBundleCompose {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.capability_id",
                reason: "must be MULTI_BUNDLE_COMPOSE",
            });
        }
        validate_token(
            "multi_bundle_compose_ok.selected_signal_id",
            &self.selected_signal_id,
            64,
        )?;
        if self.ordered_bundle_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.ordered_bundle_items",
                reason: "must not be empty",
            });
        }
        if self.ordered_bundle_items.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.ordered_bundle_items",
                reason: "must be <= 32",
            });
        }

        let mut signal_ids = BTreeSet::new();
        for item in &self.ordered_bundle_items {
            item.validate()?;
            if !signal_ids.insert(item.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "multi_bundle_compose_ok.ordered_bundle_items",
                    reason: "signal_id must be unique",
                });
            }
        }

        if !self
            .ordered_bundle_items
            .iter()
            .any(|item| item.signal_id == self.selected_signal_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.selected_signal_id",
                reason: "must exist in ordered_bundle_items",
            });
        }

        if !self.evidence_backed {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.evidence_backed",
                reason: "must be true",
            });
        }
        if !self.privacy_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.privacy_scoped",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "multi_bundle_compose_ok.no_execution_authority",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSignalAlignOk {
    pub schema_version: SchemaVersion,
    pub capability_id: MultiCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: MultiValidationStatus,
    pub diagnostics: Vec<String>,
    pub evidence_backed: bool,
    pub privacy_scoped: bool,
    pub no_execution_authority: bool,
}

impl MultiSignalAlignOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: MultiValidationStatus,
        diagnostics: Vec<String>,
        evidence_backed: bool,
        privacy_scoped: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            capability_id: MultiCapabilityId::MultiSignalAlign,
            reason_code,
            validation_status,
            diagnostics,
            evidence_backed,
            privacy_scoped,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for MultiSignalAlignOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        if self.capability_id != MultiCapabilityId::MultiSignalAlign {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.capability_id",
                reason: "must be MULTI_SIGNAL_ALIGN",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("multi_signal_align_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == MultiValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }

        if !self.evidence_backed {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.evidence_backed",
                reason: "must be true",
            });
        }
        if !self.privacy_scoped {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.privacy_scoped",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "multi_signal_align_ok.no_execution_authority",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: MultiCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl MultiRefuse {
    pub fn v1(
        capability_id: MultiCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1MULTI_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for MultiRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1MULTI_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "multi_refuse.schema_version",
                reason: "must match PH1MULTI_CONTRACT_VERSION",
            });
        }
        validate_token("multi_refuse.message", &self.message, 256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1MultiResponse {
    MultiBundleComposeOk(MultiBundleComposeOk),
    MultiSignalAlignOk(MultiSignalAlignOk),
    Refuse(MultiRefuse),
}

impl Validate for Ph1MultiResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1MultiResponse::MultiBundleComposeOk(out) => out.validate(),
            Ph1MultiResponse::MultiSignalAlignOk(out) => out.validate(),
            Ph1MultiResponse::Refuse(out) => out.validate(),
        }
    }
}

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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_field_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII snake_case",
        });
    }
    Ok(())
}

fn validate_engine_id(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '.' || c == '_' || c.is_ascii_digit())
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be uppercase engine token",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> MultiRequestEnvelope {
        MultiRequestEnvelope::v1(CorrelationId(2101), TurnId(181), 8, 4, true).unwrap()
    }

    fn signal(
        signal_id: &str,
        source_engine: &str,
        modality: MultiModality,
        evidence_ref: Option<&str>,
    ) -> MultiSourceSignal {
        MultiSourceSignal::v1(
            signal_id.to_string(),
            source_engine.to_string(),
            modality,
            "context_hint".to_string(),
            "value".to_string(),
            evidence_ref.map(|v| v.to_string()),
            82,
            true,
        )
        .unwrap()
    }

    #[test]
    fn multi_bundle_compose_request_is_schema_valid() {
        let req = MultiBundleComposeRequest::v1(
            envelope(),
            vec![
                signal("s_1", "PH1.LISTEN", MultiModality::Voice, None),
                signal("s_2", "PH1.VISION", MultiModality::Vision, Some("vision:1")),
            ],
            true,
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn multi_source_signal_requires_privacy_scope() {
        let out = MultiSourceSignal::v1(
            "s_1".to_string(),
            "PH1.LISTEN".to_string(),
            MultiModality::Voice,
            "context_hint".to_string(),
            "value".to_string(),
            None,
            50,
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn multi_bundle_compose_ok_rejects_missing_selected_signal() {
        let item = MultiBundleItem::v1(
            "s_1".to_string(),
            "PH1.LISTEN".to_string(),
            MultiModality::Voice,
            1,
            88,
            None,
        )
        .unwrap();
        let out = MultiBundleComposeOk::v1(
            ReasonCodeId(1),
            "missing".to_string(),
            vec![item],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn multi_signal_align_ok_fail_requires_diagnostics() {
        let out = MultiSignalAlignOk::v1(
            ReasonCodeId(2),
            MultiValidationStatus::Fail,
            vec![],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }
}
