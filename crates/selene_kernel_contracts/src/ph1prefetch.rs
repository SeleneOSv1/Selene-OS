#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PREFETCH_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrefetchCapabilityId {
    PrefetchPlanBuild,
    PrefetchPrioritize,
}

impl PrefetchCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PrefetchCapabilityId::PrefetchPlanBuild => "PREFETCH_PLAN_BUILD",
            PrefetchCapabilityId::PrefetchPrioritize => "PREFETCH_PRIORITIZE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrefetchToolKind {
    Time,
    Weather,
    News,
    WebSearch,
}

impl PrefetchToolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PrefetchToolKind::Time => "TIME",
            PrefetchToolKind::Weather => "WEATHER",
            PrefetchToolKind::News => "NEWS",
            PrefetchToolKind::WebSearch => "WEB_SEARCH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrefetchValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_candidates: u8,
}

impl PrefetchRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_candidates: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_candidates,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PrefetchRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_request_envelope.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_candidates == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_request_envelope.max_candidates",
                reason: "must be > 0",
            });
        }
        if self.max_candidates > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_request_envelope.max_candidates",
                reason: "must be <= 8",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchPlanBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PrefetchRequestEnvelope,
    pub intent_type: String,
    pub locale: Option<String>,
    pub search_query_hints: Vec<String>,
    pub policy_prefetch_enabled: bool,
    pub privacy_mode: bool,
}

impl PrefetchPlanBuildRequest {
    pub fn v1(
        envelope: PrefetchRequestEnvelope,
        intent_type: String,
        locale: Option<String>,
        search_query_hints: Vec<String>,
        policy_prefetch_enabled: bool,
        privacy_mode: bool,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            envelope,
            intent_type,
            locale,
            search_query_hints,
            policy_prefetch_enabled,
            privacy_mode,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PrefetchPlanBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_plan_build_request.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "prefetch_plan_build_request.intent_type",
            &self.intent_type,
            96,
        )?;
        if let Some(locale) = &self.locale {
            validate_text("prefetch_plan_build_request.locale", locale, 32)?;
        }
        if self.search_query_hints.len() > self.envelope.max_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_plan_build_request.search_query_hints",
                reason: "must be <= envelope.max_candidates",
            });
        }
        for hint in &self.search_query_hints {
            validate_text("prefetch_plan_build_request.search_query_hints", hint, 256)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchPrioritizeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PrefetchRequestEnvelope,
    pub intent_type: String,
    pub locale: Option<String>,
    pub search_query_hints: Vec<String>,
    pub policy_prefetch_enabled: bool,
    pub privacy_mode: bool,
    pub candidates: Vec<PrefetchCandidate>,
}

impl PrefetchPrioritizeRequest {
    pub fn v1(
        envelope: PrefetchRequestEnvelope,
        intent_type: String,
        locale: Option<String>,
        search_query_hints: Vec<String>,
        policy_prefetch_enabled: bool,
        privacy_mode: bool,
        candidates: Vec<PrefetchCandidate>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            envelope,
            intent_type,
            locale,
            search_query_hints,
            policy_prefetch_enabled,
            privacy_mode,
            candidates,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PrefetchPrioritizeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_request.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "prefetch_prioritize_request.intent_type",
            &self.intent_type,
            96,
        )?;
        if let Some(locale) = &self.locale {
            validate_text("prefetch_prioritize_request.locale", locale, 32)?;
        }
        if self.search_query_hints.len() > self.envelope.max_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_request.search_query_hints",
                reason: "must be <= envelope.max_candidates",
            });
        }
        for hint in &self.search_query_hints {
            validate_text("prefetch_prioritize_request.search_query_hints", hint, 256)?;
        }
        validate_candidates(
            "prefetch_prioritize_request.candidates",
            &self.candidates,
            self.envelope.max_candidates as usize,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PrefetchRequest {
    PrefetchPlanBuild(PrefetchPlanBuildRequest),
    PrefetchPrioritize(PrefetchPrioritizeRequest),
}

impl Validate for Ph1PrefetchRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PrefetchRequest::PrefetchPlanBuild(r) => r.validate(),
            Ph1PrefetchRequest::PrefetchPrioritize(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchCandidate {
    pub schema_version: SchemaVersion,
    pub candidate_id: String,
    pub tool_kind: PrefetchToolKind,
    pub query_text: String,
    pub ttl_seconds: u16,
    pub rank_weight_bp: u16,
    pub idempotency_dedupe_key: String,
}

impl PrefetchCandidate {
    pub fn v1(
        candidate_id: String,
        tool_kind: PrefetchToolKind,
        query_text: String,
        ttl_seconds: u16,
        rank_weight_bp: u16,
        idempotency_dedupe_key: String,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            candidate_id,
            tool_kind,
            query_text,
            ttl_seconds,
            rank_weight_bp,
            idempotency_dedupe_key,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for PrefetchCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_candidate.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        validate_text("prefetch_candidate.candidate_id", &self.candidate_id, 64)?;
        validate_text("prefetch_candidate.query_text", &self.query_text, 256)?;
        if self.ttl_seconds == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_candidate.ttl_seconds",
                reason: "must be > 0",
            });
        }
        if self.ttl_seconds > 900 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_candidate.ttl_seconds",
                reason: "must be <= 900",
            });
        }
        if self.rank_weight_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_candidate.rank_weight_bp",
                reason: "must be <= 10000",
            });
        }
        validate_text(
            "prefetch_candidate.idempotency_dedupe_key",
            &self.idempotency_dedupe_key,
            96,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchPlanBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PrefetchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub candidates: Vec<PrefetchCandidate>,
    pub read_only_only: bool,
}

impl PrefetchPlanBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        candidates: Vec<PrefetchCandidate>,
        read_only_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            capability_id: PrefetchCapabilityId::PrefetchPlanBuild,
            reason_code,
            candidates,
            read_only_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for PrefetchPlanBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_plan_build_ok.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PrefetchCapabilityId::PrefetchPlanBuild {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_plan_build_ok.capability_id",
                reason: "must be PREFETCH_PLAN_BUILD",
            });
        }
        validate_candidates("prefetch_plan_build_ok.candidates", &self.candidates, 8)?;
        if !self.read_only_only {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_plan_build_ok.read_only_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchPrioritizeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PrefetchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PrefetchValidationStatus,
    pub prioritized_candidate_ids: Vec<String>,
    pub diagnostics: Vec<String>,
    pub read_only_only: bool,
}

impl PrefetchPrioritizeOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PrefetchValidationStatus,
        prioritized_candidate_ids: Vec<String>,
        diagnostics: Vec<String>,
        read_only_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            capability_id: PrefetchCapabilityId::PrefetchPrioritize,
            reason_code,
            validation_status,
            prioritized_candidate_ids,
            diagnostics,
            read_only_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for PrefetchPrioritizeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PrefetchCapabilityId::PrefetchPrioritize {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.capability_id",
                reason: "must be PREFETCH_PRIORITIZE",
            });
        }
        if self.prioritized_candidate_ids.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.prioritized_candidate_ids",
                reason: "must not be empty",
            });
        }
        if self.prioritized_candidate_ids.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.prioritized_candidate_ids",
                reason: "must be <= 8",
            });
        }
        let mut ids: BTreeSet<&str> = BTreeSet::new();
        for candidate_id in &self.prioritized_candidate_ids {
            validate_text(
                "prefetch_prioritize_ok.prioritized_candidate_ids",
                candidate_id,
                64,
            )?;
            if !ids.insert(candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "prefetch_prioritize_ok.prioritized_candidate_ids",
                    reason: "must be unique",
                });
            }
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("prefetch_prioritize_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == PrefetchValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.read_only_only {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_prioritize_ok.read_only_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PrefetchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PrefetchRefuse {
    pub fn v1(
        capability_id: PrefetchCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PREFETCH_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for PrefetchRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PREFETCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_refuse.schema_version",
                reason: "must match PH1PREFETCH_CONTRACT_VERSION",
            });
        }
        validate_text("prefetch_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PrefetchResponse {
    PrefetchPlanBuildOk(PrefetchPlanBuildOk),
    PrefetchPrioritizeOk(PrefetchPrioritizeOk),
    Refuse(PrefetchRefuse),
}

impl Validate for Ph1PrefetchResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PrefetchResponse::PrefetchPlanBuildOk(o) => o.validate(),
            Ph1PrefetchResponse::PrefetchPrioritizeOk(o) => o.validate(),
            Ph1PrefetchResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_candidates(
    field: &'static str,
    candidates: &[PrefetchCandidate],
    max_entries: usize,
) -> Result<(), ContractViolation> {
    if candidates.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if candidates.len() > max_entries {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max entries",
        });
    }
    let mut candidate_ids: BTreeSet<&str> = BTreeSet::new();
    for candidate in candidates {
        candidate.validate()?;
        if !candidate_ids.insert(candidate.candidate_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "candidate_id entries must be unique",
            });
        }
    }
    Ok(())
}

fn validate_text(field: &'static str, text: &str, max_len: usize) -> Result<(), ContractViolation> {
    if text.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if text.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if text.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_candidates: u8) -> PrefetchRequestEnvelope {
        PrefetchRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_candidates).unwrap()
    }

    fn candidate(tool_kind: PrefetchToolKind, id: &str) -> PrefetchCandidate {
        PrefetchCandidate::v1(
            id.to_string(),
            tool_kind,
            "weather in singapore".to_string(),
            300,
            8000,
            format!("pf-1-{id}"),
        )
        .unwrap()
    }

    #[test]
    fn prefetch_candidate_rejects_zero_ttl() {
        let out = PrefetchCandidate::v1(
            "c0".to_string(),
            PrefetchToolKind::Weather,
            "weather now".to_string(),
            0,
            5000,
            "pf-0".to_string(),
        );
        assert!(out.is_err());
    }

    #[test]
    fn prefetch_plan_build_ok_requires_read_only_only_true() {
        let out = PrefetchPlanBuildOk::v1(
            ReasonCodeId(1),
            vec![candidate(PrefetchToolKind::Weather, "c0")],
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn prefetch_prioritize_ok_requires_diagnostics_when_fail() {
        let out = PrefetchPrioritizeOk::v1(
            ReasonCodeId(1),
            PrefetchValidationStatus::Fail,
            vec!["c0".to_string()],
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn prefetch_plan_build_request_rejects_too_many_hints() {
        let req = PrefetchPlanBuildRequest::v1(
            envelope(2),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec![
                "weather".to_string(),
                "today".to_string(),
                "singapore".to_string(),
            ],
            true,
            false,
        );
        assert!(req.is_err());
    }

    #[test]
    fn prefetch_prioritize_request_validates_candidates() {
        let req = PrefetchPrioritizeRequest::v1(
            envelope(4),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec!["weather singapore".to_string()],
            true,
            false,
            vec![
                candidate(PrefetchToolKind::Weather, "c0"),
                candidate(PrefetchToolKind::Time, "c1"),
            ],
        );
        assert!(req.is_ok());
    }
}
