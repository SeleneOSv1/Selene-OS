#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SEARCH_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchCapabilityId {
    SearchPlanBuild,
    SearchQueryRewrite,
}

impl SearchCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            SearchCapabilityId::SearchPlanBuild => "SEARCH_PLAN_BUILD",
            SearchCapabilityId::SearchQueryRewrite => "SEARCH_QUERY_REWRITE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_plan_queries: u8,
}

impl SearchRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_plan_queries: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_plan_queries,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for SearchRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_request_envelope.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_plan_queries == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "search_request_envelope.max_plan_queries",
                reason: "must be > 0",
            });
        }
        if self.max_plan_queries > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "search_request_envelope.max_plan_queries",
                reason: "must be <= 8",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SearchQueryId(String);

impl SearchQueryId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for SearchQueryId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlanQuery {
    pub schema_version: SchemaVersion,
    pub query_id: SearchQueryId,
    pub query_text: String,
    pub language_hint: Option<String>,
}

impl SearchPlanQuery {
    pub fn v1(
        query_id: SearchQueryId,
        query_text: String,
        language_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let q = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            query_id,
            query_text,
            language_hint,
        };
        q.validate()?;
        Ok(q)
    }
}

impl Validate for SearchPlanQuery {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_query.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        self.query_id.validate()?;
        validate_bounded_text("search_plan_query.query_text", &self.query_text, 256)?;
        if let Some(language_hint) = &self.language_hint {
            validate_bounded_text("search_plan_query.language_hint", language_hint, 32)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlanBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SearchRequestEnvelope,
    pub raw_query: String,
    pub language_hint: Option<String>,
}

impl SearchPlanBuildRequest {
    pub fn v1(
        envelope: SearchRequestEnvelope,
        raw_query: String,
        language_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            envelope,
            raw_query,
            language_hint,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SearchPlanBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_request.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_bounded_text("search_plan_build_request.raw_query", &self.raw_query, 512)?;
        if let Some(language_hint) = &self.language_hint {
            validate_bounded_text("search_plan_build_request.language_hint", language_hint, 32)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQueryRewriteRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SearchRequestEnvelope,
    pub raw_query: String,
    pub planned_queries: Vec<SearchPlanQuery>,
}

impl SearchQueryRewriteRequest {
    pub fn v1(
        envelope: SearchRequestEnvelope,
        raw_query: String,
        planned_queries: Vec<SearchPlanQuery>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            envelope,
            raw_query,
            planned_queries,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SearchQueryRewriteRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_request.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_bounded_text(
            "search_query_rewrite_request.raw_query",
            &self.raw_query,
            512,
        )?;
        if self.planned_queries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_request.planned_queries",
                reason: "must not be empty",
            });
        }
        if self.planned_queries.len() > self.envelope.max_plan_queries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_request.planned_queries",
                reason: "must be <= envelope.max_plan_queries",
            });
        }
        for query in &self.planned_queries {
            query.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SearchRequest {
    SearchPlanBuild(SearchPlanBuildRequest),
    SearchQueryRewrite(SearchQueryRewriteRequest),
}

impl Validate for Ph1SearchRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SearchRequest::SearchPlanBuild(r) => r.validate(),
            Ph1SearchRequest::SearchQueryRewrite(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlanBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SearchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub planned_queries: Vec<SearchPlanQuery>,
    pub no_intent_drift: bool,
}

impl SearchPlanBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        planned_queries: Vec<SearchPlanQuery>,
        no_intent_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            capability_id: SearchCapabilityId::SearchPlanBuild,
            reason_code,
            planned_queries,
            no_intent_drift,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for SearchPlanBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_ok.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SearchCapabilityId::SearchPlanBuild {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_ok.capability_id",
                reason: "must be SEARCH_PLAN_BUILD",
            });
        }
        if self.planned_queries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_ok.planned_queries",
                reason: "must not be empty",
            });
        }
        if self.planned_queries.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_ok.planned_queries",
                reason: "must be <= 8",
            });
        }
        for query in &self.planned_queries {
            query.validate()?;
        }
        if !self.no_intent_drift {
            return Err(ContractViolation::InvalidValue {
                field: "search_plan_build_ok.no_intent_drift",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQueryRewriteOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SearchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: SearchValidationStatus,
    pub rewritten_queries: Vec<SearchPlanQuery>,
    pub diagnostics: Vec<String>,
    pub no_intent_drift: bool,
}

impl SearchQueryRewriteOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: SearchValidationStatus,
        rewritten_queries: Vec<SearchPlanQuery>,
        diagnostics: Vec<String>,
        no_intent_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            capability_id: SearchCapabilityId::SearchQueryRewrite,
            reason_code,
            validation_status,
            rewritten_queries,
            diagnostics,
            no_intent_drift,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for SearchQueryRewriteOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SearchCapabilityId::SearchQueryRewrite {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.capability_id",
                reason: "must be SEARCH_QUERY_REWRITE",
            });
        }
        if self.rewritten_queries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.rewritten_queries",
                reason: "must not be empty",
            });
        }
        if self.rewritten_queries.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.rewritten_queries",
                reason: "must be <= 8",
            });
        }
        for query in &self.rewritten_queries {
            query.validate()?;
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_bounded_text("search_query_rewrite_ok.diagnostics", diagnostic, 128)?;
        }
        if self.validation_status == SearchValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_intent_drift {
            return Err(ContractViolation::InvalidValue {
                field: "search_query_rewrite_ok.no_intent_drift",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: SearchCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl SearchRefuse {
    pub fn v1(
        capability_id: SearchCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SEARCH_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SearchRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SEARCH_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "search_refuse.schema_version",
                reason: "must match PH1SEARCH_CONTRACT_VERSION",
            });
        }
        validate_bounded_text("search_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SearchResponse {
    SearchPlanBuildOk(SearchPlanBuildOk),
    SearchQueryRewriteOk(SearchQueryRewriteOk),
    Refuse(SearchRefuse),
}

impl Validate for Ph1SearchResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SearchResponse::SearchPlanBuildOk(o) => o.validate(),
            Ph1SearchResponse::SearchQueryRewriteOk(o) => o.validate(),
            Ph1SearchResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_bounded_text(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_queries: u8) -> SearchRequestEnvelope {
        SearchRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_queries).unwrap()
    }

    fn query(id: &str, text: &str) -> SearchPlanQuery {
        SearchPlanQuery::v1(
            SearchQueryId::new(id).unwrap(),
            text.to_string(),
            Some("en".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn search_plan_build_request_rejects_empty_query() {
        let req = SearchPlanBuildRequest::v1(envelope(4), "".to_string(), None);
        assert!(req.is_err());
    }

    #[test]
    fn search_query_rewrite_request_rejects_empty_planned_queries() {
        let req = SearchQueryRewriteRequest::v1(envelope(4), "weather today".to_string(), vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn search_query_rewrite_ok_requires_diagnostic_when_status_fail() {
        let out = SearchQueryRewriteOk::v1(
            ReasonCodeId(1),
            SearchValidationStatus::Fail,
            vec![query("q1", "weather today")],
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn search_plan_build_ok_requires_no_intent_drift_true() {
        let out = SearchPlanBuildOk::v1(ReasonCodeId(1), vec![query("q1", "weather today")], false);
        assert!(out.is_err());
    }
}
