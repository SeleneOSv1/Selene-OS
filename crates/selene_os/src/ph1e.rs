#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1e::{StrictBudget, ToolRequest, ToolResponse};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.E OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_E_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4501_0101);
    pub const PH1_E_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4501_01F1);
    pub const PH1_E_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4501_01F2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1eWiringConfig {
    pub tool_router_enabled: bool,
    pub max_timeout_ms: u32,
    pub max_results: u8,
}

impl Ph1eWiringConfig {
    pub fn mvp_v1(tool_router_enabled: bool) -> Self {
        Self {
            tool_router_enabled,
            max_timeout_ms: 2_000,
            max_results: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1eWiringOutcome {
    NotInvokedDisabled,
    Refused(ToolResponse),
    Forwarded(ToolResponse),
}

pub trait Ph1eEngine {
    fn run(&self, req: &ToolRequest) -> ToolResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1eWiring<E>
where
    E: Ph1eEngine,
{
    config: Ph1eWiringConfig,
    engine: E,
}

impl<E> Ph1eWiring<E>
where
    E: Ph1eEngine,
{
    pub fn new(config: Ph1eWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_timeout_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1e_wiring_config.max_timeout_ms",
                reason: "must be > 0",
            });
        }
        if config.max_results == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1e_wiring_config.max_results",
                reason: "must be > 0",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_request(&self, req: &ToolRequest) -> Result<Ph1eWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.tool_router_enabled {
            return Ok(Ph1eWiringOutcome::NotInvokedDisabled);
        }

        if budget_exceeded(req.strict_budget, self.config) {
            return Ok(Ph1eWiringOutcome::Refused(fail_response(
                req,
                reason_codes::PH1_E_BUDGET_EXCEEDED,
            )?));
        }

        let out = self.engine.run(req);
        out.validate()?;

        if out.request_id != req.request_id || out.query_hash != req.query_hash {
            return Ok(Ph1eWiringOutcome::Refused(fail_response(
                req,
                reason_codes::PH1_E_INTERNAL_PIPELINE_ERROR,
            )?));
        }

        Ok(Ph1eWiringOutcome::Forwarded(out))
    }
}

fn budget_exceeded(request_budget: StrictBudget, config: Ph1eWiringConfig) -> bool {
    request_budget.timeout_ms > config.max_timeout_ms
        || request_budget.max_results > config.max_results
}

fn fail_response(req: &ToolRequest, code: ReasonCodeId) -> Result<ToolResponse, ContractViolation> {
    ToolResponse::fail_v1(
        req.request_id,
        req.query_hash,
        code,
        selene_kernel_contracts::ph1e::CacheStatus::Bypassed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{
        CacheStatus, SourceMetadata, SourceRef, ToolName, ToolRequestId, ToolRequestOrigin,
        ToolResult, ToolStatus, ToolTextSnippet,
    };

    #[derive(Debug, Clone)]
    struct StubEngine {
        out: ToolResponse,
    }

    impl Ph1eEngine for StubEngine {
        fn run(&self, _req: &ToolRequest) -> ToolResponse {
            self.out.clone()
        }
    }

    fn req() -> ToolRequest {
        ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            ToolName::WebSearch,
            "selene".to_string(),
            Some("en-US".to_string()),
            StrictBudget::new(1000, 3).unwrap(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap()
    }

    fn ok_response_for(req: &ToolRequest) -> ToolResponse {
        ToolResponse::ok_v1(
            req.request_id,
            req.query_hash,
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Result".to_string(),
                    snippet: "Snippet".to_string(),
                    url: "https://example.com/a".to_string(),
                }],
            },
            SourceMetadata {
                schema_version: selene_kernel_contracts::ph1e::PH1E_CONTRACT_VERSION,
                provider_hint: Some("stub".to_string()),
                retrieved_at_unix_ms: 1_700_000_000_000,
                sources: vec![SourceRef {
                    title: "Example".to_string(),
                    url: "https://example.com/a".to_string(),
                }],
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Miss,
        )
        .unwrap()
    }

    #[test]
    fn at_e_wiring_01_disabled_returns_not_invoked() {
        let r = req();
        let w = Ph1eWiring::new(
            Ph1eWiringConfig::mvp_v1(false),
            StubEngine {
                out: ok_response_for(&r),
            },
        )
        .unwrap();
        assert_eq!(
            w.run_request(&r).unwrap(),
            Ph1eWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_e_wiring_02_forwards_valid_response() {
        let r = req();
        let w = Ph1eWiring::new(
            Ph1eWiringConfig::mvp_v1(true),
            StubEngine {
                out: ok_response_for(&r),
            },
        )
        .unwrap();
        match w.run_request(&r).unwrap() {
            Ph1eWiringOutcome::Forwarded(out) => assert_eq!(out.tool_status, ToolStatus::Ok),
            _ => panic!("expected forwarded output"),
        }
    }

    #[test]
    fn at_e_wiring_03_request_id_drift_fails_closed() {
        let r = req();
        let mut out = ok_response_for(&r);
        out.request_id = ToolRequestId(r.request_id.0.saturating_add(1));
        let w = Ph1eWiring::new(Ph1eWiringConfig::mvp_v1(true), StubEngine { out }).unwrap();
        match w.run_request(&r).unwrap() {
            Ph1eWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.tool_status, ToolStatus::Fail);
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_E_INTERNAL_PIPELINE_ERROR
                );
            }
            _ => panic!("expected refused output"),
        }
    }
}
