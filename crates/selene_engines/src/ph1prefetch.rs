#![forbid(unsafe_code)]

use std::cmp::{min, Reverse};
use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1prefetch::{
    Ph1PrefetchRequest, Ph1PrefetchResponse, PrefetchCandidate, PrefetchCapabilityId,
    PrefetchPlanBuildOk, PrefetchPlanBuildRequest, PrefetchPrioritizeOk, PrefetchPrioritizeRequest,
    PrefetchRefuse, PrefetchToolKind, PrefetchValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PREFETCH reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PREFETCH_OK_PLAN_BUILD: ReasonCodeId = ReasonCodeId(0x5052_4601);
    pub const PH1_PREFETCH_OK_PRIORITIZE: ReasonCodeId = ReasonCodeId(0x5052_4602);

    pub const PH1_PREFETCH_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5052_46F1);
    pub const PH1_PREFETCH_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5052_46F2);
    pub const PH1_PREFETCH_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5052_46F3);
    pub const PH1_PREFETCH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_46F4);
    pub const PH1_PREFETCH_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_46F5);
    pub const PH1_PREFETCH_POLICY_DISABLED: ReasonCodeId = ReasonCodeId(0x5052_46F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PrefetchConfig {
    pub max_candidates: u8,
    pub max_diagnostics: u8,
}

impl Ph1PrefetchConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_candidates: 4,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PrefetchRuntime {
    config: Ph1PrefetchConfig,
}

impl Ph1PrefetchRuntime {
    pub fn new(config: Ph1PrefetchConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PrefetchRequest) -> Ph1PrefetchResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PREFETCH_INPUT_SCHEMA_INVALID,
                "prefetch request failed contract validation",
            );
        }

        match req {
            Ph1PrefetchRequest::PrefetchPlanBuild(r) => self.run_plan_build(r),
            Ph1PrefetchRequest::PrefetchPrioritize(r) => self.run_prioritize(r),
        }
    }

    fn run_plan_build(&self, req: &PrefetchPlanBuildRequest) -> Ph1PrefetchResponse {
        if !req.policy_prefetch_enabled {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPlanBuild,
                reason_codes::PH1_PREFETCH_POLICY_DISABLED,
                "prefetch policy is disabled",
            );
        }
        if req.intent_type.trim().is_empty() {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPlanBuild,
                reason_codes::PH1_PREFETCH_UPSTREAM_INPUT_MISSING,
                "intent_type is empty",
            );
        }

        let max_candidates = min(
            req.envelope.max_candidates as usize,
            self.config.max_candidates as usize,
        );
        let candidates = match build_prefetch_candidates(
            req.intent_type.as_str(),
            req.search_query_hints.as_slice(),
            req.privacy_mode,
            req.envelope.turn_id.0,
            max_candidates,
        ) {
            Ok(candidates) => candidates,
            Err(_) => {
                return self.refuse(
                    PrefetchCapabilityId::PrefetchPlanBuild,
                    reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                    "failed to build prefetch candidates",
                )
            }
        };

        if candidates.is_empty() {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPlanBuild,
                reason_codes::PH1_PREFETCH_UPSTREAM_INPUT_MISSING,
                "no prefetch candidates could be produced",
            );
        }
        if candidates.len() > max_candidates {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPlanBuild,
                reason_codes::PH1_PREFETCH_BUDGET_EXCEEDED,
                "prefetch candidates exceed budget",
            );
        }

        match PrefetchPlanBuildOk::v1(reason_codes::PH1_PREFETCH_OK_PLAN_BUILD, candidates, true) {
            Ok(ok) => Ph1PrefetchResponse::PrefetchPlanBuildOk(ok),
            Err(_) => self.refuse(
                PrefetchCapabilityId::PrefetchPlanBuild,
                reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                "failed to construct prefetch plan output",
            ),
        }
    }

    fn run_prioritize(&self, req: &PrefetchPrioritizeRequest) -> Ph1PrefetchResponse {
        if !req.policy_prefetch_enabled {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPrioritize,
                reason_codes::PH1_PREFETCH_POLICY_DISABLED,
                "prefetch policy is disabled",
            );
        }

        let max_candidates = min(
            req.envelope.max_candidates as usize,
            self.config.max_candidates as usize,
        );
        if req.candidates.len() > max_candidates {
            return self.refuse(
                PrefetchCapabilityId::PrefetchPrioritize,
                reason_codes::PH1_PREFETCH_BUDGET_EXCEEDED,
                "prefetch candidates exceed budget",
            );
        }

        let expected = match build_prefetch_candidates(
            req.intent_type.as_str(),
            req.search_query_hints.as_slice(),
            req.privacy_mode,
            req.envelope.turn_id.0,
            max_candidates,
        ) {
            Ok(candidates) => candidates,
            Err(_) => {
                return self.refuse(
                    PrefetchCapabilityId::PrefetchPrioritize,
                    reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                    "failed to rebuild expected prefetch candidates",
                )
            }
        };

        let mut diagnostics: Vec<String> = Vec::new();
        if req.candidates.len() != expected.len() {
            diagnostics.push("candidates_len_mismatch".to_string());
        }

        let expected_by_id: BTreeMap<&str, &PrefetchCandidate> = expected
            .iter()
            .map(|candidate| (candidate.candidate_id.as_str(), candidate))
            .collect();
        let actual_by_id: BTreeMap<&str, &PrefetchCandidate> = req
            .candidates
            .iter()
            .map(|candidate| (candidate.candidate_id.as_str(), candidate))
            .collect();

        for (candidate_id, expected_candidate) in &expected_by_id {
            match actual_by_id.get(candidate_id) {
                Some(actual_candidate) => {
                    collect_candidate_diagnostics(
                        actual_candidate,
                        expected_candidate,
                        &mut diagnostics,
                    );
                }
                None => diagnostics.push(format!("{candidate_id}_missing")),
            }
            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }
        }

        for candidate_id in actual_by_id.keys() {
            if !expected_by_id.contains_key(candidate_id) {
                diagnostics.push(format!("{candidate_id}_unexpected"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let prioritized_candidate_ids = prioritized_ids(req.candidates.as_slice());
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                PrefetchValidationStatus::Ok,
                reason_codes::PH1_PREFETCH_OK_PRIORITIZE,
            )
        } else {
            (
                PrefetchValidationStatus::Fail,
                reason_codes::PH1_PREFETCH_VALIDATION_FAILED,
            )
        };

        match PrefetchPrioritizeOk::v1(
            reason_code,
            validation_status,
            prioritized_candidate_ids,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1PrefetchResponse::PrefetchPrioritizeOk(ok),
            Err(_) => self.refuse(
                PrefetchCapabilityId::PrefetchPrioritize,
                reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                "failed to construct prefetch prioritize output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PrefetchCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PrefetchResponse {
        let r = PrefetchRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PrefetchRefuse::v1 must construct for static message");
        Ph1PrefetchResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1PrefetchRequest) -> PrefetchCapabilityId {
    match req {
        Ph1PrefetchRequest::PrefetchPlanBuild(_) => PrefetchCapabilityId::PrefetchPlanBuild,
        Ph1PrefetchRequest::PrefetchPrioritize(_) => PrefetchCapabilityId::PrefetchPrioritize,
    }
}

fn build_prefetch_candidates(
    intent_type: &str,
    search_query_hints: &[String],
    privacy_mode: bool,
    turn_id: u64,
    max_candidates: usize,
) -> Result<Vec<PrefetchCandidate>, selene_kernel_contracts::ContractViolation> {
    if max_candidates == 0 {
        return Ok(Vec::new());
    }

    let normalized_intent = intent_type.to_ascii_uppercase();
    let hint = search_query_hints
        .iter()
        .find(|h| !h.trim().is_empty())
        .map(|h| collapse_ws(h))
        .unwrap_or_default();

    let mut planned_tools: Vec<PrefetchToolKind> = Vec::new();
    if normalized_intent.contains("WEATHER") {
        planned_tools.push(PrefetchToolKind::Weather);
        planned_tools.push(PrefetchToolKind::Time);
    } else if normalized_intent.contains("TIME") || normalized_intent.contains("DATE") {
        planned_tools.push(PrefetchToolKind::Time);
    } else if normalized_intent.contains("NEWS") {
        planned_tools.push(PrefetchToolKind::News);
        planned_tools.push(PrefetchToolKind::WebSearch);
    } else if normalized_intent.contains("GENERAL_FACT")
        || normalized_intent.contains("DEFINITION")
        || normalized_intent.contains("STATUS")
        || normalized_intent.contains("QUERY_GENERAL")
    {
        planned_tools.push(PrefetchToolKind::WebSearch);
    }

    if !hint.is_empty()
        && !planned_tools
            .iter()
            .any(|tool| *tool == PrefetchToolKind::WebSearch)
    {
        planned_tools.push(PrefetchToolKind::WebSearch);
    }

    if privacy_mode {
        planned_tools
            .retain(|tool| *tool != PrefetchToolKind::News && *tool != PrefetchToolKind::WebSearch);
    }

    let mut dedup: BTreeSet<PrefetchToolKind> = BTreeSet::new();
    planned_tools.retain(|tool| dedup.insert(*tool));
    planned_tools.truncate(max_candidates);

    let mut candidates: Vec<PrefetchCandidate> = Vec::new();
    for (idx, tool_kind) in planned_tools.iter().enumerate() {
        let query_text = query_for_tool(*tool_kind, hint.as_str());
        let ttl_seconds = ttl_for_tool(*tool_kind);
        let rank_weight_bp = rank_weight_for_tool(*tool_kind, normalized_intent.as_str());
        let candidate_id = format!("pf_{idx:02}_{}", tool_kind.as_str().to_ascii_lowercase());
        let idempotency_dedupe_key = format!(
            "pf_t{}_{}_{}",
            turn_id,
            idx,
            tool_kind.as_str().to_ascii_lowercase()
        );

        let candidate = PrefetchCandidate::v1(
            candidate_id,
            *tool_kind,
            truncate_to_char_boundary(query_text.as_str(), 256),
            ttl_seconds,
            rank_weight_bp,
            idempotency_dedupe_key,
        )?;
        candidates.push(candidate);
    }

    Ok(candidates)
}

fn query_for_tool(tool_kind: PrefetchToolKind, hint: &str) -> String {
    match tool_kind {
        PrefetchToolKind::Time => "current local time".to_string(),
        PrefetchToolKind::Weather => {
            if hint.is_empty() {
                "weather forecast today".to_string()
            } else {
                hint.to_string()
            }
        }
        PrefetchToolKind::News => {
            if hint.is_empty() {
                "latest headlines".to_string()
            } else {
                hint.to_string()
            }
        }
        PrefetchToolKind::WebSearch => {
            if hint.is_empty() {
                "latest updates".to_string()
            } else {
                hint.to_string()
            }
        }
    }
}

fn ttl_for_tool(tool_kind: PrefetchToolKind) -> u16 {
    match tool_kind {
        PrefetchToolKind::Time => 30,
        PrefetchToolKind::Weather => 300,
        PrefetchToolKind::News => 120,
        PrefetchToolKind::WebSearch => 120,
    }
}

fn rank_weight_for_tool(tool_kind: PrefetchToolKind, normalized_intent: &str) -> u16 {
    let primary = match tool_kind {
        PrefetchToolKind::Time => {
            normalized_intent.contains("TIME") || normalized_intent.contains("DATE")
        }
        PrefetchToolKind::Weather => normalized_intent.contains("WEATHER"),
        PrefetchToolKind::News => normalized_intent.contains("NEWS"),
        PrefetchToolKind::WebSearch => {
            normalized_intent.contains("GENERAL_FACT")
                || normalized_intent.contains("DEFINITION")
                || normalized_intent.contains("QUERY_GENERAL")
        }
    };
    if primary {
        9000
    } else {
        7000
    }
}

fn prioritized_ids(candidates: &[PrefetchCandidate]) -> Vec<String> {
    let mut ranked = candidates.iter().collect::<Vec<_>>();
    ranked.sort_by_key(|candidate| {
        (
            Reverse(candidate.rank_weight_bp),
            candidate.candidate_id.as_str(),
        )
    });
    ranked
        .into_iter()
        .map(|candidate| candidate.candidate_id.clone())
        .collect::<Vec<_>>()
}

fn collect_candidate_diagnostics(
    actual: &PrefetchCandidate,
    expected: &PrefetchCandidate,
    diagnostics: &mut Vec<String>,
) {
    let prefix = actual.candidate_id.as_str();
    if actual.tool_kind != expected.tool_kind {
        diagnostics.push(format!("{prefix}_tool_kind_mismatch"));
    }
    if actual.query_text != expected.query_text {
        diagnostics.push(format!("{prefix}_query_text_mismatch"));
    }
    if actual.ttl_seconds != expected.ttl_seconds {
        diagnostics.push(format!("{prefix}_ttl_seconds_mismatch"));
    }
    if actual.rank_weight_bp != expected.rank_weight_bp {
        diagnostics.push(format!("{prefix}_rank_weight_bp_mismatch"));
    }
    if actual.idempotency_dedupe_key != expected.idempotency_dedupe_key {
        diagnostics.push(format!("{prefix}_idempotency_key_mismatch"));
    }
}

fn collapse_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_to_char_boundary(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1prefetch::{
        PrefetchPlanBuildRequest, PrefetchPrioritizeRequest, PrefetchRequestEnvelope,
    };

    fn runtime() -> Ph1PrefetchRuntime {
        Ph1PrefetchRuntime::new(Ph1PrefetchConfig::mvp_v1())
    }

    fn envelope(max_candidates: u8) -> PrefetchRequestEnvelope {
        PrefetchRequestEnvelope::v1(CorrelationId(1601), TurnId(121), max_candidates).unwrap()
    }

    #[test]
    fn at_prefetch_01_plan_build_output_is_schema_valid() {
        let req = Ph1PrefetchRequest::PrefetchPlanBuild(
            PrefetchPlanBuildRequest::v1(
                envelope(4),
                "QUERY_WEATHER".to_string(),
                Some("en-US".to_string()),
                vec!["weather in singapore".to_string()],
                true,
                false,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1PrefetchResponse::PrefetchPlanBuildOk(ok) => {
                assert!(!ok.candidates.is_empty());
                assert!(ok.read_only_only);
            }
            _ => panic!("expected PrefetchPlanBuildOk"),
        }
    }

    #[test]
    fn at_prefetch_02_deterministic_order_is_preserved() {
        let req = Ph1PrefetchRequest::PrefetchPlanBuild(
            PrefetchPlanBuildRequest::v1(
                envelope(4),
                "QUERY_NEWS".to_string(),
                Some("en-US".to_string()),
                vec!["tech headlines".to_string()],
                true,
                false,
            )
            .unwrap(),
        );

        let runtime = runtime();
        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);
        let ids1 = match out1 {
            Ph1PrefetchResponse::PrefetchPlanBuildOk(ok) => ok
                .candidates
                .iter()
                .map(|candidate| candidate.candidate_id.clone())
                .collect::<Vec<_>>(),
            _ => panic!("expected PrefetchPlanBuildOk"),
        };
        let ids2 = match out2 {
            Ph1PrefetchResponse::PrefetchPlanBuildOk(ok) => ok
                .candidates
                .iter()
                .map(|candidate| candidate.candidate_id.clone())
                .collect::<Vec<_>>(),
            _ => panic!("expected PrefetchPlanBuildOk"),
        };
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn at_prefetch_03_candidate_budget_and_privacy_mode_are_enforced() {
        let req = Ph1PrefetchRequest::PrefetchPlanBuild(
            PrefetchPlanBuildRequest::v1(
                envelope(1),
                "QUERY_NEWS".to_string(),
                Some("en-US".to_string()),
                vec!["global updates".to_string()],
                true,
                true,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PrefetchResponse::Refuse(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::PH1_PREFETCH_UPSTREAM_INPUT_MISSING
                );
            }
            Ph1PrefetchResponse::PrefetchPlanBuildOk(ok) => {
                assert!(ok.candidates.len() <= 1);
                assert!(ok
                    .candidates
                    .iter()
                    .all(|candidate| candidate.tool_kind != PrefetchToolKind::News));
                assert!(ok
                    .candidates
                    .iter()
                    .all(|candidate| candidate.tool_kind != PrefetchToolKind::WebSearch));
            }
            _ => panic!("unexpected response"),
        }
    }

    #[test]
    fn at_prefetch_04_prioritize_fails_for_drifted_candidate() {
        let drifted = vec![PrefetchCandidate::v1(
            "pf_00_weather".to_string(),
            PrefetchToolKind::Weather,
            "weather in singapore".to_string(),
            120,
            9000,
            "pf_t121_0_weather".to_string(),
        )
        .unwrap()];

        let req = Ph1PrefetchRequest::PrefetchPrioritize(
            PrefetchPrioritizeRequest::v1(
                envelope(4),
                "QUERY_WEATHER".to_string(),
                Some("en-US".to_string()),
                vec!["weather in singapore".to_string()],
                true,
                false,
                drifted,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PrefetchResponse::PrefetchPrioritizeOk(ok) => {
                assert_eq!(ok.validation_status, PrefetchValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_PREFETCH_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|diag| diag == "pf_00_weather_ttl_seconds_mismatch"));
            }
            _ => panic!("expected PrefetchPrioritizeOk"),
        }
    }
}
