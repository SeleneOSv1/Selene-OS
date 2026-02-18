#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1CONTEXT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextCapabilityId {
    ContextBundleBuild,
    ContextBundleTrim,
}

impl ContextCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            ContextCapabilityId::ContextBundleBuild => "CONTEXT_BUNDLE_BUILD",
            ContextCapabilityId::ContextBundleTrim => "CONTEXT_BUNDLE_TRIM",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextSourceKind {
    Memory,
    ConversationState,
    ClarificationHistory,
    AttentionHint,
    MultiHint,
    DocEvidence,
    SummaryEvidence,
    VisionEvidence,
    WebEvidence,
    CacheHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_items: u8,
    pub max_diagnostics: u8,
    pub evidence_required: bool,
}

impl ContextRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_items: u8,
        max_diagnostics: u8,
        evidence_required: bool,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_items,
            max_diagnostics,
            evidence_required,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for ContextRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_items == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.max_items",
                reason: "must be > 0",
            });
        }
        if self.max_items > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.max_items",
                reason: "must be <= 32",
            });
        }
        if self.max_diagnostics == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.max_diagnostics",
                reason: "must be > 0",
            });
        }
        if self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.max_diagnostics",
                reason: "must be <= 16",
            });
        }
        if !self.evidence_required {
            return Err(ContractViolation::InvalidValue {
                field: "context_request_envelope.evidence_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSourceItem {
    pub schema_version: SchemaVersion,
    pub item_id: String,
    pub source_engine: String,
    pub source_kind: ContextSourceKind,
    pub rank_score_bp: i16,
    pub content_ref: String,
    pub evidence_ref: String,
    pub sensitivity_private: bool,
}

impl ContextSourceItem {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        item_id: String,
        source_engine: String,
        source_kind: ContextSourceKind,
        rank_score_bp: i16,
        content_ref: String,
        evidence_ref: String,
        sensitivity_private: bool,
    ) -> Result<Self, ContractViolation> {
        let item = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            item_id,
            source_engine,
            source_kind,
            rank_score_bp,
            content_ref,
            evidence_ref,
            sensitivity_private,
        };
        item.validate()?;
        Ok(item)
    }
}

impl Validate for ContextSourceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_source_item.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        validate_token("context_source_item.item_id", &self.item_id, 96)?;
        validate_engine_id("context_source_item.source_engine", &self.source_engine)?;
        if !(-20_000..=20_000).contains(&self.rank_score_bp) {
            return Err(ContractViolation::InvalidValue {
                field: "context_source_item.rank_score_bp",
                reason: "must be within -20000..=20000",
            });
        }
        validate_token("context_source_item.content_ref", &self.content_ref, 160)?;
        validate_token("context_source_item.evidence_ref", &self.evidence_ref, 160)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ContextRequestEnvelope,
    pub intent_type: String,
    pub privacy_mode: bool,
    pub source_items: Vec<ContextSourceItem>,
    pub multi_signal_align_ok: bool,
    pub cache_hint_refresh_ok: bool,
}

impl ContextBundleBuildRequest {
    pub fn v1(
        envelope: ContextRequestEnvelope,
        intent_type: String,
        privacy_mode: bool,
        source_items: Vec<ContextSourceItem>,
        multi_signal_align_ok: bool,
        cache_hint_refresh_ok: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            envelope,
            intent_type,
            privacy_mode,
            source_items,
            multi_signal_align_ok,
            cache_hint_refresh_ok,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ContextBundleBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_request.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "context_bundle_build_request.intent_type",
            &self.intent_type,
            96,
        )?;
        if self.source_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_request.source_items",
                reason: "must be non-empty",
            });
        }
        if self.source_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_request.source_items",
                reason: "must be <= 128",
            });
        }
        let mut seen = BTreeSet::new();
        for item in &self.source_items {
            item.validate()?;
            if !seen.insert(item.item_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "context_bundle_build_request.source_items",
                    reason: "item_id values must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleItem {
    pub schema_version: SchemaVersion,
    pub item_id: String,
    pub source_engine: String,
    pub source_kind: ContextSourceKind,
    pub bundle_rank: u8,
    pub content_ref: String,
    pub evidence_ref: String,
    pub sensitivity_private: bool,
}

impl ContextBundleItem {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        item_id: String,
        source_engine: String,
        source_kind: ContextSourceKind,
        bundle_rank: u8,
        content_ref: String,
        evidence_ref: String,
        sensitivity_private: bool,
    ) -> Result<Self, ContractViolation> {
        let item = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            item_id,
            source_engine,
            source_kind,
            bundle_rank,
            content_ref,
            evidence_ref,
            sensitivity_private,
        };
        item.validate()?;
        Ok(item)
    }
}

impl Validate for ContextBundleItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_item.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        validate_token("context_bundle_item.item_id", &self.item_id, 96)?;
        validate_engine_id("context_bundle_item.source_engine", &self.source_engine)?;
        if self.bundle_rank == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_item.bundle_rank",
                reason: "must be > 0",
            });
        }
        validate_token("context_bundle_item.content_ref", &self.content_ref, 160)?;
        validate_token("context_bundle_item.evidence_ref", &self.evidence_ref, 160)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ContextCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_item_ids: Vec<String>,
    pub ordered_bundle_items: Vec<ContextBundleItem>,
    pub preserved_evidence_refs: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl ContextBundleBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_item_ids: Vec<String>,
        ordered_bundle_items: Vec<ContextBundleItem>,
        preserved_evidence_refs: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            capability_id: ContextCapabilityId::ContextBundleBuild,
            reason_code,
            selected_item_ids,
            ordered_bundle_items,
            preserved_evidence_refs,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for ContextBundleBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ContextCapabilityId::ContextBundleBuild {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.capability_id",
                reason: "must be CONTEXT_BUNDLE_BUILD",
            });
        }
        if self.selected_item_ids.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.selected_item_ids",
                reason: "must be non-empty",
            });
        }
        if self.selected_item_ids.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.selected_item_ids",
                reason: "must be <= 8",
            });
        }
        if self.ordered_bundle_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.ordered_bundle_items",
                reason: "must be non-empty",
            });
        }
        if self.ordered_bundle_items.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.ordered_bundle_items",
                reason: "must be <= 32",
            });
        }
        let mut seen_ids = BTreeSet::new();
        for item in &self.ordered_bundle_items {
            item.validate()?;
            if !seen_ids.insert(item.item_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "context_bundle_build_ok.ordered_bundle_items",
                    reason: "item_id values must be unique",
                });
            }
        }
        for selected_item_id in &self.selected_item_ids {
            validate_token(
                "context_bundle_build_ok.selected_item_ids",
                selected_item_id,
                96,
            )?;
            if !seen_ids.contains(selected_item_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "context_bundle_build_ok.selected_item_ids",
                    reason: "must reference ordered_bundle_items ids",
                });
            }
        }
        if !self.preserved_evidence_refs {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.preserved_evidence_refs",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleTrimRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ContextRequestEnvelope,
    pub intent_type: String,
    pub privacy_mode: bool,
    pub selected_item_ids: Vec<String>,
    pub ordered_bundle_items: Vec<ContextBundleItem>,
    pub multi_signal_align_ok: bool,
    pub cache_hint_refresh_ok: bool,
}

impl ContextBundleTrimRequest {
    pub fn v1(
        envelope: ContextRequestEnvelope,
        intent_type: String,
        privacy_mode: bool,
        selected_item_ids: Vec<String>,
        ordered_bundle_items: Vec<ContextBundleItem>,
        multi_signal_align_ok: bool,
        cache_hint_refresh_ok: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            envelope,
            intent_type,
            privacy_mode,
            selected_item_ids,
            ordered_bundle_items,
            multi_signal_align_ok,
            cache_hint_refresh_ok,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ContextBundleTrimRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_request.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "context_bundle_trim_request.intent_type",
            &self.intent_type,
            96,
        )?;
        if self.selected_item_ids.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_request.selected_item_ids",
                reason: "must be non-empty",
            });
        }
        if self.ordered_bundle_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_request.ordered_bundle_items",
                reason: "must be non-empty",
            });
        }
        if self.ordered_bundle_items.len() > self.envelope.max_items as usize {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_request.ordered_bundle_items",
                reason: "must be <= envelope.max_items",
            });
        }

        let mut seen = BTreeSet::new();
        for item in &self.ordered_bundle_items {
            item.validate()?;
            if !seen.insert(item.item_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "context_bundle_trim_request.ordered_bundle_items",
                    reason: "item_id values must be unique",
                });
            }
        }
        for selected_item_id in &self.selected_item_ids {
            validate_token(
                "context_bundle_trim_request.selected_item_ids",
                selected_item_id,
                96,
            )?;
            if !seen.contains(selected_item_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "context_bundle_trim_request.selected_item_ids",
                    reason: "must reference ordered_bundle_items ids",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundleTrimOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ContextCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: ContextValidationStatus,
    pub diagnostics: Vec<String>,
    pub preserved_ranked_source_order: bool,
    pub preserved_evidence_refs: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl ContextBundleTrimOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: ContextValidationStatus,
        diagnostics: Vec<String>,
        preserved_ranked_source_order: bool,
        preserved_evidence_refs: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            capability_id: ContextCapabilityId::ContextBundleTrim,
            reason_code,
            validation_status,
            diagnostics,
            preserved_ranked_source_order,
            preserved_evidence_refs,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for ContextBundleTrimOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ContextCapabilityId::ContextBundleTrim {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.capability_id",
                reason: "must be CONTEXT_BUNDLE_TRIM",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("context_bundle_trim_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == ContextValidationStatus::Ok
            && (!self.preserved_ranked_source_order || !self.preserved_evidence_refs)
        {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok",
                reason: "OK status requires preserved order and evidence refs",
            });
        }
        if self.validation_status == ContextValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "context_bundle_trim_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: ContextCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl ContextRefuse {
    pub fn v1(
        capability_id: ContextCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1CONTEXT_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for ContextRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1CONTEXT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "context_refuse.schema_version",
                reason: "must match PH1CONTEXT_CONTRACT_VERSION",
            });
        }
        validate_text("context_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ContextRequest {
    ContextBundleBuild(ContextBundleBuildRequest),
    ContextBundleTrim(ContextBundleTrimRequest),
}

impl Validate for Ph1ContextRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ContextRequest::ContextBundleBuild(r) => r.validate(),
            Ph1ContextRequest::ContextBundleTrim(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ContextResponse {
    ContextBundleBuildOk(ContextBundleBuildOk),
    ContextBundleTrimOk(ContextBundleTrimOk),
    Refuse(ContextRefuse),
}

impl Validate for Ph1ContextResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ContextResponse::ContextBundleBuildOk(ok) => ok.validate(),
            Ph1ContextResponse::ContextBundleTrimOk(ok) => ok.validate(),
            Ph1ContextResponse::Refuse(r) => r.validate(),
        }
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

fn validate_text(
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_engine_id(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if value
        .chars()
        .any(|c| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '.'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain uppercase engine id characters only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> ContextRequestEnvelope {
        ContextRequestEnvelope::v1(CorrelationId(9101), TurnId(501), 8, 6, true).unwrap()
    }

    fn source_item(
        id: &str,
        source_kind: ContextSourceKind,
        rank_score_bp: i16,
    ) -> ContextSourceItem {
        ContextSourceItem::v1(
            id.to_string(),
            "PH1.SUMMARY".to_string(),
            source_kind,
            rank_score_bp,
            format!("context:content:{}", id),
            format!("context:evidence:{}", id),
            false,
        )
        .unwrap()
    }

    #[test]
    fn at_context_contract_01_build_request_is_schema_valid() {
        let req = ContextBundleBuildRequest::v1(
            envelope(),
            "QUERY_WEATHER".to_string(),
            false,
            vec![source_item(
                "ctx_1",
                ContextSourceKind::SummaryEvidence,
                1200,
            )],
            true,
            true,
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn at_context_contract_02_evidence_required_enforced() {
        let out = ContextSourceItem::v1(
            "ctx_bad".to_string(),
            "PH1.SUMMARY".to_string(),
            ContextSourceKind::SummaryEvidence,
            100,
            "context:content:bad".to_string(),
            "".to_string(),
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_context_contract_03_trim_ok_fail_requires_diagnostics() {
        let out = ContextBundleTrimOk::v1(
            ReasonCodeId(1),
            ContextValidationStatus::Fail,
            vec![],
            false,
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_context_contract_04_trim_request_selected_ids_must_exist() {
        let req = ContextBundleTrimRequest::v1(
            envelope(),
            "QUERY_WEATHER".to_string(),
            false,
            vec!["ctx_missing".to_string()],
            vec![ContextBundleItem::v1(
                "ctx_1".to_string(),
                "PH1.SUMMARY".to_string(),
                ContextSourceKind::SummaryEvidence,
                1,
                "context:content:ctx_1".to_string(),
                "context:evidence:ctx_1".to_string(),
                false,
            )
            .unwrap()],
            true,
            true,
        );
        assert!(req.is_err());
    }
}
