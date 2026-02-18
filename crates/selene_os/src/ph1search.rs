#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1search::{
    Ph1SearchRequest, Ph1SearchResponse, SearchCapabilityId, SearchPlanBuildOk,
    SearchPlanBuildRequest, SearchQueryRewriteOk, SearchQueryRewriteRequest, SearchRefuse,
    SearchRequestEnvelope, SearchValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SEARCH OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_SEARCH_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5348_0101);
    pub const PH1_SEARCH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5348_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SearchWiringConfig {
    pub search_enabled: bool,
    pub max_plan_queries: u8,
}

impl Ph1SearchWiringConfig {
    pub fn mvp_v1(search_enabled: bool) -> Self {
        Self {
            search_enabled,
            max_plan_queries: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub raw_query: String,
    pub language_hint: Option<String>,
}

impl SearchTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        raw_query: String,
        language_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            raw_query,
            language_hint,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for SearchTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.raw_query.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "search_turn_input.raw_query",
                reason: "must be <= 512 chars",
            });
        }
        if self.raw_query.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "search_turn_input.raw_query",
                reason: "must not contain control characters",
            });
        }
        if let Some(language_hint) = &self.language_hint {
            if language_hint.len() > 32 || language_hint.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "search_turn_input.language_hint",
                    reason: "must be <= 32 chars and contain no control characters",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub plan_build: SearchPlanBuildOk,
    pub query_rewrite: SearchQueryRewriteOk,
}

impl SearchForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        plan_build: SearchPlanBuildOk,
        query_rewrite: SearchQueryRewriteOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            plan_build,
            query_rewrite,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for SearchForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.plan_build.validate()?;
        self.query_rewrite.validate()?;
        if self.query_rewrite.validation_status != SearchValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "search_forward_bundle.query_rewrite.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSearchInput,
    Refused(SearchRefuse),
    Forwarded(SearchForwardBundle),
}

pub trait Ph1SearchEngine {
    fn run(&self, req: &Ph1SearchRequest) -> Ph1SearchResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1SearchWiring<E>
where
    E: Ph1SearchEngine,
{
    config: Ph1SearchWiringConfig,
    engine: E,
}

impl<E> Ph1SearchWiring<E>
where
    E: Ph1SearchEngine,
{
    pub fn new(config: Ph1SearchWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_plan_queries == 0 || config.max_plan_queries > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1search_wiring_config.max_plan_queries",
                reason: "must be within 1..=8",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &SearchTurnInput,
    ) -> Result<SearchWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.search_enabled {
            return Ok(SearchWiringOutcome::NotInvokedDisabled);
        }

        if input.raw_query.trim().is_empty() {
            return Ok(SearchWiringOutcome::NotInvokedNoSearchInput);
        }

        let envelope = SearchRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_plan_queries, 8),
        )?;

        let plan_req = Ph1SearchRequest::SearchPlanBuild(SearchPlanBuildRequest::v1(
            envelope.clone(),
            input.raw_query.clone(),
            input.language_hint.clone(),
        )?);
        let plan_resp = self.engine.run(&plan_req);
        plan_resp.validate()?;

        let plan_ok = match plan_resp {
            Ph1SearchResponse::Refuse(r) => return Ok(SearchWiringOutcome::Refused(r)),
            Ph1SearchResponse::SearchPlanBuildOk(ok) => ok,
            Ph1SearchResponse::SearchQueryRewriteOk(_) => {
                return Ok(SearchWiringOutcome::Refused(SearchRefuse::v1(
                    SearchCapabilityId::SearchPlanBuild,
                    reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                    "unexpected query-rewrite response for plan-build request".to_string(),
                )?))
            }
        };

        let rewrite_req = Ph1SearchRequest::SearchQueryRewrite(SearchQueryRewriteRequest::v1(
            envelope,
            input.raw_query.clone(),
            plan_ok.planned_queries.clone(),
        )?);
        let rewrite_resp = self.engine.run(&rewrite_req);
        rewrite_resp.validate()?;

        let rewrite_ok = match rewrite_resp {
            Ph1SearchResponse::Refuse(r) => return Ok(SearchWiringOutcome::Refused(r)),
            Ph1SearchResponse::SearchQueryRewriteOk(ok) => ok,
            Ph1SearchResponse::SearchPlanBuildOk(_) => {
                return Ok(SearchWiringOutcome::Refused(SearchRefuse::v1(
                    SearchCapabilityId::SearchQueryRewrite,
                    reason_codes::PH1_SEARCH_INTERNAL_PIPELINE_ERROR,
                    "unexpected plan-build response for query-rewrite request".to_string(),
                )?))
            }
        };

        if rewrite_ok.validation_status != SearchValidationStatus::Ok {
            return Ok(SearchWiringOutcome::Refused(SearchRefuse::v1(
                SearchCapabilityId::SearchQueryRewrite,
                reason_codes::PH1_SEARCH_VALIDATION_FAILED,
                "search query rewrite validation failed".to_string(),
            )?));
        }

        let bundle =
            SearchForwardBundle::v1(input.correlation_id, input.turn_id, plan_ok, rewrite_ok)?;
        Ok(SearchWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1search::{
        SearchPlanQuery, SearchQueryId, SearchQueryRewriteOk,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicSearchEngine;

    impl Ph1SearchEngine for DeterministicSearchEngine {
        fn run(&self, req: &Ph1SearchRequest) -> Ph1SearchResponse {
            match req {
                Ph1SearchRequest::SearchPlanBuild(r) => {
                    let queries = vec![SearchPlanQuery::v1(
                        SearchQueryId::new("q0").unwrap(),
                        r.raw_query.clone(),
                        r.language_hint.clone(),
                    )
                    .unwrap()];
                    Ph1SearchResponse::SearchPlanBuildOk(
                        SearchPlanBuildOk::v1(ReasonCodeId(1), queries, true).unwrap(),
                    )
                }
                Ph1SearchRequest::SearchQueryRewrite(r) => Ph1SearchResponse::SearchQueryRewriteOk(
                    SearchQueryRewriteOk::v1(
                        ReasonCodeId(2),
                        SearchValidationStatus::Ok,
                        r.planned_queries.clone(),
                        vec![],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftSearchEngine;

    impl Ph1SearchEngine for DriftSearchEngine {
        fn run(&self, req: &Ph1SearchRequest) -> Ph1SearchResponse {
            match req {
                Ph1SearchRequest::SearchPlanBuild(r) => {
                    let queries = vec![SearchPlanQuery::v1(
                        SearchQueryId::new("q0").unwrap(),
                        r.raw_query.clone(),
                        r.language_hint.clone(),
                    )
                    .unwrap()];
                    Ph1SearchResponse::SearchPlanBuildOk(
                        SearchPlanBuildOk::v1(ReasonCodeId(10), queries, true).unwrap(),
                    )
                }
                Ph1SearchRequest::SearchQueryRewrite(r) => Ph1SearchResponse::SearchQueryRewriteOk(
                    SearchQueryRewriteOk::v1(
                        ReasonCodeId(11),
                        SearchValidationStatus::Fail,
                        r.planned_queries.clone(),
                        vec!["query_0_not_intent_anchored".to_string()],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    #[test]
    fn at_search_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1SearchWiring::new(
            Ph1SearchWiringConfig::mvp_v1(true),
            DeterministicSearchEngine,
        )
        .unwrap();

        let input = SearchTurnInput::v1(
            CorrelationId(1101),
            TurnId(71),
            "weather in sydney tomorrow".to_string(),
            Some("en".to_string()),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SearchWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.query_rewrite.validation_status,
                    SearchValidationStatus::Ok
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_search_02_os_preserves_query_order_for_ph1e_handoff() {
        let wiring = Ph1SearchWiring::new(
            Ph1SearchWiringConfig::mvp_v1(true),
            DeterministicSearchEngine,
        )
        .unwrap();

        let input = SearchTurnInput::v1(
            CorrelationId(1102),
            TurnId(72),
            "news about rust language".to_string(),
            Some("en".to_string()),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SearchWiringOutcome::Forwarded(bundle) => {
                let texts = bundle
                    .query_rewrite
                    .rewritten_queries
                    .iter()
                    .map(|q| q.query_text.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(texts, vec!["news about rust language"]);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_search_03_os_does_not_invoke_when_search_is_disabled() {
        let wiring = Ph1SearchWiring::new(
            Ph1SearchWiringConfig::mvp_v1(false),
            DeterministicSearchEngine,
        )
        .unwrap();

        let input = SearchTurnInput::v1(
            CorrelationId(1103),
            TurnId(73),
            "time in tokyo".to_string(),
            Some("en".to_string()),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, SearchWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_search_04_os_fails_closed_on_query_rewrite_drift() {
        let wiring =
            Ph1SearchWiring::new(Ph1SearchWiringConfig::mvp_v1(true), DriftSearchEngine).unwrap();

        let input = SearchTurnInput::v1(
            CorrelationId(1104),
            TurnId(74),
            "weather in tokyo".to_string(),
            Some("en".to_string()),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            SearchWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_SEARCH_VALIDATION_FAILED);
                assert_eq!(r.capability_id, SearchCapabilityId::SearchQueryRewrite);
            }
            _ => panic!("expected Refused"),
        }
    }
}
