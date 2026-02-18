#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PATTERN_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternCapabilityId {
    PatternMineOffline,
    PatternProposalEmit,
}

impl PatternCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PatternCapabilityId::PatternMineOffline => "PATTERN_MINE_OFFLINE",
            PatternCapabilityId::PatternProposalEmit => "PATTERN_PROPOSAL_EMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternProposalTarget {
    PaeProviderRoutingWeights,
    PruneClarificationOrdering,
    CachePrefetchHeuristics,
    ContextRetrievalScoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signals: u8,
    pub max_proposals: u8,
    pub offline_pipeline_only: bool,
}

impl PatternRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signals: u8,
        max_proposals: u8,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signals,
            max_proposals,
            offline_pipeline_only,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PatternRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signals == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.max_signals",
                reason: "must be > 0",
            });
        }
        if self.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.max_signals",
                reason: "must be <= 64",
            });
        }
        if self.max_proposals == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.max_proposals",
                reason: "must be > 0",
            });
        }
        if self.max_proposals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.max_proposals",
                reason: "must be <= 32",
            });
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_request_envelope.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSignal {
    pub schema_version: SchemaVersion,
    pub signal_id: String,
    pub source_engine: String,
    pub metric_key: String,
    pub metric_value_bp: i16,
    pub occurrence_count: u32,
    pub evidence_ref: String,
}

impl PatternSignal {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        signal_id: String,
        source_engine: String,
        metric_key: String,
        metric_value_bp: i16,
        occurrence_count: u32,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let signal = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            signal_id,
            source_engine,
            metric_key,
            metric_value_bp,
            occurrence_count,
            evidence_ref,
        };
        signal.validate()?;
        Ok(signal)
    }
}

impl Validate for PatternSignal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_signal.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        validate_token("pattern_signal.signal_id", &self.signal_id, 64)?;
        validate_engine_id("pattern_signal.source_engine", &self.source_engine)?;
        validate_metric_key("pattern_signal.metric_key", &self.metric_key)?;
        if self.metric_value_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_signal.metric_value_bp",
                reason: "must be within -20000..=20000 basis points",
            });
        }
        if self.occurrence_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_signal.occurrence_count",
                reason: "must be > 0",
            });
        }
        validate_token("pattern_signal.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternMineOfflineRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PatternRequestEnvelope,
    pub signals: Vec<PatternSignal>,
    pub analysis_window_days: u16,
}

impl PatternMineOfflineRequest {
    pub fn v1(
        envelope: PatternRequestEnvelope,
        signals: Vec<PatternSignal>,
        analysis_window_days: u16,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            envelope,
            signals,
            analysis_window_days,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PatternMineOfflineRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_request.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.signals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_request.signals",
                reason: "must not be empty",
            });
        }
        if self.signals.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_request.signals",
                reason: "must be <= 64",
            });
        }
        let mut signal_ids = BTreeSet::new();
        for signal in &self.signals {
            signal.validate()?;
            if !signal_ids.insert(signal.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pattern_mine_offline_request.signals",
                    reason: "signal_id must be unique",
                });
            }
        }
        if self.analysis_window_days == 0 || self.analysis_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_request.analysis_window_days",
                reason: "must be within 1..=365",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternProposalItem {
    pub schema_version: SchemaVersion,
    pub proposal_id: String,
    pub target: PatternProposalTarget,
    pub rank: u8,
    pub confidence_pct: u8,
    pub approval_tier: u8,
    pub evidence_ref: String,
}

impl PatternProposalItem {
    pub fn v1(
        proposal_id: String,
        target: PatternProposalTarget,
        rank: u8,
        confidence_pct: u8,
        approval_tier: u8,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let item = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            proposal_id,
            target,
            rank,
            confidence_pct,
            approval_tier,
            evidence_ref,
        };
        item.validate()?;
        Ok(item)
    }
}

impl Validate for PatternProposalItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_item.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        validate_token("pattern_proposal_item.proposal_id", &self.proposal_id, 96)?;
        if self.rank == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_item.rank",
                reason: "must be > 0",
            });
        }
        if self.approval_tier > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_item.approval_tier",
                reason: "must be within 0..=3",
            });
        }
        validate_token(
            "pattern_proposal_item.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternProposalEmitRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PatternRequestEnvelope,
    pub selected_proposal_id: String,
    pub ordered_proposals: Vec<PatternProposalItem>,
}

impl PatternProposalEmitRequest {
    pub fn v1(
        envelope: PatternRequestEnvelope,
        selected_proposal_id: String,
        ordered_proposals: Vec<PatternProposalItem>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            envelope,
            selected_proposal_id,
            ordered_proposals,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PatternProposalEmitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_request.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "pattern_proposal_emit_request.selected_proposal_id",
            &self.selected_proposal_id,
            96,
        )?;
        if self.ordered_proposals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_request.ordered_proposals",
                reason: "must not be empty",
            });
        }
        if self.ordered_proposals.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_request.ordered_proposals",
                reason: "must be <= 32",
            });
        }
        let mut proposal_ids = BTreeSet::new();
        for item in &self.ordered_proposals {
            item.validate()?;
            if !proposal_ids.insert(item.proposal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pattern_proposal_emit_request.ordered_proposals",
                    reason: "proposal_id must be unique",
                });
            }
        }
        if !self
            .ordered_proposals
            .iter()
            .any(|item| item.proposal_id == self.selected_proposal_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_request.selected_proposal_id",
                reason: "must exist in ordered_proposals",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PatternRequest {
    PatternMineOffline(PatternMineOfflineRequest),
    PatternProposalEmit(PatternProposalEmitRequest),
}

impl Validate for Ph1PatternRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PatternRequest::PatternMineOffline(req) => req.validate(),
            Ph1PatternRequest::PatternProposalEmit(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternMineOfflineOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PatternCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_proposal_id: String,
    pub ordered_proposals: Vec<PatternProposalItem>,
    pub offline_only: bool,
    pub no_execution_authority: bool,
}

impl PatternMineOfflineOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_proposal_id: String,
        ordered_proposals: Vec<PatternProposalItem>,
        offline_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            capability_id: PatternCapabilityId::PatternMineOffline,
            reason_code,
            selected_proposal_id,
            ordered_proposals,
            offline_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PatternMineOfflineOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PatternCapabilityId::PatternMineOffline {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.capability_id",
                reason: "must be PATTERN_MINE_OFFLINE",
            });
        }
        validate_token(
            "pattern_mine_offline_ok.selected_proposal_id",
            &self.selected_proposal_id,
            96,
        )?;
        if self.ordered_proposals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.ordered_proposals",
                reason: "must not be empty",
            });
        }
        if self.ordered_proposals.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.ordered_proposals",
                reason: "must be <= 32",
            });
        }
        let mut proposal_ids = BTreeSet::new();
        for item in &self.ordered_proposals {
            item.validate()?;
            if !proposal_ids.insert(item.proposal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pattern_mine_offline_ok.ordered_proposals",
                    reason: "proposal_id must be unique",
                });
            }
        }
        if !self
            .ordered_proposals
            .iter()
            .any(|item| item.proposal_id == self.selected_proposal_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.selected_proposal_id",
                reason: "must exist in ordered_proposals",
            });
        }
        if !self.offline_only {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.offline_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_mine_offline_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternProposalEmitOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PatternCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PatternValidationStatus,
    pub diagnostics: Vec<String>,
    pub offline_only: bool,
    pub no_execution_authority: bool,
}

impl PatternProposalEmitOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PatternValidationStatus,
        diagnostics: Vec<String>,
        offline_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            capability_id: PatternCapabilityId::PatternProposalEmit,
            reason_code,
            validation_status,
            diagnostics,
            offline_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PatternProposalEmitOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PatternCapabilityId::PatternProposalEmit {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.capability_id",
                reason: "must be PATTERN_PROPOSAL_EMIT",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("pattern_proposal_emit_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == PatternValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.offline_only {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.offline_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_proposal_emit_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PatternCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PatternRefuse {
    pub fn v1(
        capability_id: PatternCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PATTERN_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PatternRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PATTERN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_refuse.schema_version",
                reason: "must match PH1PATTERN_CONTRACT_VERSION",
            });
        }
        validate_token("pattern_refuse.message", &self.message, 256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PatternResponse {
    PatternMineOfflineOk(PatternMineOfflineOk),
    PatternProposalEmitOk(PatternProposalEmitOk),
    Refuse(PatternRefuse),
}

impl Validate for Ph1PatternResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PatternResponse::PatternMineOfflineOk(out) => out.validate(),
            Ph1PatternResponse::PatternProposalEmitOk(out) => out.validate(),
            Ph1PatternResponse::Refuse(out) => out.validate(),
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

fn validate_metric_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
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

    fn envelope() -> PatternRequestEnvelope {
        PatternRequestEnvelope::v1(CorrelationId(2701), TurnId(241), 8, 4, true).unwrap()
    }

    fn signal(signal_id: &str, metric_key: &str) -> PatternSignal {
        PatternSignal::v1(
            signal_id.to_string(),
            "PH1.J".to_string(),
            metric_key.to_string(),
            180,
            20,
            "evidence:pattern:1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn pattern_mine_request_is_schema_valid() {
        let req = PatternMineOfflineRequest::v1(
            envelope(),
            vec![
                signal("sig_a", "clarify_loop_rate"),
                signal("sig_b", "tool_timeout_rate"),
            ],
            30,
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn pattern_envelope_requires_offline_only_true() {
        let out = PatternRequestEnvelope::v1(CorrelationId(1), TurnId(1), 4, 4, false);
        assert!(out.is_err());
    }

    #[test]
    fn pattern_mine_ok_rejects_missing_selected_proposal() {
        let item = PatternProposalItem::v1(
            "proposal_a".to_string(),
            PatternProposalTarget::PaeProviderRoutingWeights,
            1,
            81,
            3,
            "evidence:pattern:1".to_string(),
        )
        .unwrap();
        let out = PatternMineOfflineOk::v1(
            ReasonCodeId(1),
            "missing".to_string(),
            vec![item],
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn pattern_proposal_emit_ok_fail_requires_diagnostics() {
        let out = PatternProposalEmitOk::v1(
            ReasonCodeId(2),
            PatternValidationStatus::Fail,
            vec![],
            true,
            true,
        );
        assert!(out.is_err());
    }
}
