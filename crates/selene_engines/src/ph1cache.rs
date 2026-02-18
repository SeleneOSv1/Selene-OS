#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeMap;

use selene_kernel_contracts::ph1cache::{
    CacheCapabilityId, CacheDeliveryHint, CacheHintSnapshotReadOk, CacheHintSnapshotReadRequest,
    CacheHintSnapshotRefreshOk, CacheHintSnapshotRefreshRequest, CacheMoveKind, CachePlanSkeleton,
    CacheRefuse, CacheRouteHint, CacheValidationStatus, Ph1CacheRequest, Ph1CacheResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CACHE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_CACHE_OK_HINT_SNAPSHOT_READ: ReasonCodeId = ReasonCodeId(0x4348_0001);
    pub const PH1_CACHE_OK_HINT_SNAPSHOT_REFRESH: ReasonCodeId = ReasonCodeId(0x4348_0002);

    pub const PH1_CACHE_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4348_00F1);
    pub const PH1_CACHE_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4348_00F2);
    pub const PH1_CACHE_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4348_00F3);
    pub const PH1_CACHE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4348_00F4);
    pub const PH1_CACHE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4348_00F5);
    pub const PH1_CACHE_POLICY_DISABLED: ReasonCodeId = ReasonCodeId(0x4348_00F6);
    pub const PH1_CACHE_UNGOVERNED_ARTIFACT: ReasonCodeId = ReasonCodeId(0x4348_00F7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1CacheConfig {
    pub max_skeletons: u8,
    pub max_diagnostics: u8,
    pub default_ttl_seconds: u16,
    pub max_ttl_seconds: u16,
}

impl Ph1CacheConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_skeletons: 4,
            max_diagnostics: 8,
            default_ttl_seconds: 300,
            max_ttl_seconds: 900,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1CacheRuntime {
    config: Ph1CacheConfig,
}

impl Ph1CacheRuntime {
    pub fn new(config: Ph1CacheConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1CacheRequest) -> Ph1CacheResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_CACHE_INPUT_SCHEMA_INVALID,
                "cache request failed contract validation",
            );
        }

        match req {
            Ph1CacheRequest::CacheHintSnapshotRead(r) => self.run_snapshot_read(r),
            Ph1CacheRequest::CacheHintSnapshotRefresh(r) => self.run_snapshot_refresh(r),
        }
    }

    fn run_snapshot_read(&self, req: &CacheHintSnapshotReadRequest) -> Ph1CacheResponse {
        if !req.policy_cache_enabled {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRead,
                reason_codes::PH1_CACHE_POLICY_DISABLED,
                "cache policy is disabled",
            );
        }
        if req.intent_type.trim().is_empty() || req.environment_profile_ref.trim().is_empty() {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRead,
                reason_codes::PH1_CACHE_UPSTREAM_INPUT_MISSING,
                "cache intent/environment input is missing",
            );
        }

        let budget = min(
            req.envelope.max_skeletons as usize,
            self.config.max_skeletons as usize,
        );

        let skeletons = match build_cache_skeletons(
            req.intent_type.as_str(),
            req.environment_profile_ref.as_str(),
            req.persona_profile_ref.as_deref(),
            req.route_budget_hint,
            req.cache_policy_pack_id.as_deref(),
            req.privacy_mode,
            budget,
            self.config.default_ttl_seconds,
            self.config.max_ttl_seconds,
        ) {
            Ok(skeletons) => skeletons,
            Err(_) => {
                return self.refuse(
                    CacheCapabilityId::CacheHintSnapshotRead,
                    reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                    "failed to build cache hint skeletons",
                )
            }
        };

        if skeletons.is_empty() {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRead,
                reason_codes::PH1_CACHE_UPSTREAM_INPUT_MISSING,
                "no cache skeletons produced",
            );
        }
        if skeletons.len() > budget {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRead,
                reason_codes::PH1_CACHE_BUDGET_EXCEEDED,
                "cache skeletons exceed configured budget",
            );
        }

        let selected_skeleton_id = skeletons[0].skeleton_id.clone();
        match CacheHintSnapshotReadOk::v1(
            reason_codes::PH1_CACHE_OK_HINT_SNAPSHOT_READ,
            selected_skeleton_id,
            skeletons,
            true,
            true,
        ) {
            Ok(ok) => Ph1CacheResponse::CacheHintSnapshotReadOk(ok),
            Err(_) => self.refuse(
                CacheCapabilityId::CacheHintSnapshotRead,
                reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                "failed to construct cache read output",
            ),
        }
    }

    fn run_snapshot_refresh(&self, req: &CacheHintSnapshotRefreshRequest) -> Ph1CacheResponse {
        if !req.policy_cache_enabled {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRefresh,
                reason_codes::PH1_CACHE_POLICY_DISABLED,
                "cache policy is disabled",
            );
        }
        if req.intent_type.trim().is_empty() || req.environment_profile_ref.trim().is_empty() {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRefresh,
                reason_codes::PH1_CACHE_UPSTREAM_INPUT_MISSING,
                "cache intent/environment input is missing",
            );
        }

        let budget = min(
            req.envelope.max_skeletons as usize,
            self.config.max_skeletons as usize,
        );
        if req.ordered_skeletons.len() > budget {
            return self.refuse(
                CacheCapabilityId::CacheHintSnapshotRefresh,
                reason_codes::PH1_CACHE_BUDGET_EXCEEDED,
                "cache skeletons exceed configured budget",
            );
        }

        let expected = match build_cache_skeletons(
            req.intent_type.as_str(),
            req.environment_profile_ref.as_str(),
            req.persona_profile_ref.as_deref(),
            req.route_budget_hint,
            req.cache_policy_pack_id.as_deref(),
            req.privacy_mode,
            budget,
            self.config.default_ttl_seconds,
            self.config.max_ttl_seconds,
        ) {
            Ok(skeletons) => skeletons,
            Err(_) => {
                return self.refuse(
                    CacheCapabilityId::CacheHintSnapshotRefresh,
                    reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                    "failed to rebuild expected cache skeletons",
                )
            }
        };

        let mut diagnostics = Vec::new();
        if req.contains_ungoverned_artifacts {
            diagnostics.push("ungoverned_artifact_detected".to_string());
        }

        if req.selected_skeleton_id != req.ordered_skeletons[0].skeleton_id {
            diagnostics.push("selected_not_first".to_string());
        }

        if req.ordered_skeletons.len() != expected.len() {
            diagnostics.push("skeleton_count_mismatch".to_string());
        }

        let expected_by_id: BTreeMap<&str, &CachePlanSkeleton> = expected
            .iter()
            .map(|skeleton| (skeleton.skeleton_id.as_str(), skeleton))
            .collect();
        let actual_by_id: BTreeMap<&str, &CachePlanSkeleton> = req
            .ordered_skeletons
            .iter()
            .map(|skeleton| (skeleton.skeleton_id.as_str(), skeleton))
            .collect();

        for (skeleton_id, expected_skeleton) in &expected_by_id {
            match actual_by_id.get(skeleton_id) {
                Some(actual_skeleton) => {
                    if actual_skeleton != expected_skeleton {
                        diagnostics.push(format!("{}_payload_mismatch", skeleton_id));
                    }
                }
                None => diagnostics.push(format!("{}_missing", skeleton_id)),
            }
            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }
        }

        for skeleton_id in actual_by_id.keys() {
            if !expected_by_id.contains_key(skeleton_id) {
                diagnostics.push(format!("{}_unexpected", skeleton_id));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        diagnostics.truncate(min(
            self.config.max_diagnostics as usize,
            req.envelope.max_diagnostics as usize,
        ));

        let all_artifacts_governed_active = !req.contains_ungoverned_artifacts;
        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                CacheValidationStatus::Ok,
                reason_codes::PH1_CACHE_OK_HINT_SNAPSHOT_REFRESH,
            )
        } else if req.contains_ungoverned_artifacts {
            (
                CacheValidationStatus::Fail,
                reason_codes::PH1_CACHE_UNGOVERNED_ARTIFACT,
            )
        } else {
            (
                CacheValidationStatus::Fail,
                reason_codes::PH1_CACHE_VALIDATION_FAILED,
            )
        };

        match CacheHintSnapshotRefreshOk::v1(
            reason_code,
            validation_status,
            diagnostics,
            all_artifacts_governed_active,
            true,
            true,
        ) {
            Ok(ok) => Ph1CacheResponse::CacheHintSnapshotRefreshOk(ok),
            Err(_) => self.refuse(
                CacheCapabilityId::CacheHintSnapshotRefresh,
                reason_codes::PH1_CACHE_INTERNAL_PIPELINE_ERROR,
                "failed to construct cache refresh output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: CacheCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1CacheResponse {
        let refuse = CacheRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("CacheRefuse::v1 must construct for static message");
        Ph1CacheResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1CacheRequest) -> CacheCapabilityId {
    match req {
        Ph1CacheRequest::CacheHintSnapshotRead(_) => CacheCapabilityId::CacheHintSnapshotRead,
        Ph1CacheRequest::CacheHintSnapshotRefresh(_) => CacheCapabilityId::CacheHintSnapshotRefresh,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_cache_skeletons(
    intent_type: &str,
    environment_profile_ref: &str,
    persona_profile_ref: Option<&str>,
    route_budget_hint: Option<CacheRouteHint>,
    cache_policy_pack_id: Option<&str>,
    privacy_mode: bool,
    max_skeletons: usize,
    default_ttl_seconds: u16,
    max_ttl_seconds: u16,
) -> Result<Vec<CachePlanSkeleton>, selene_kernel_contracts::ContractViolation> {
    if max_skeletons == 0 {
        return Ok(Vec::new());
    }

    let normalized_intent = normalize(intent_type);
    let normalized_environment = normalize(environment_profile_ref);
    let persona = persona_profile_ref.map(collapse_ws);

    let environment_text_restricted = normalized_environment.contains("meeting")
        || normalized_environment.contains("quiet")
        || normalized_environment.contains("shared");
    let delivery_hint = if privacy_mode || environment_text_restricted {
        CacheDeliveryHint::TextRequired
    } else {
        CacheDeliveryHint::VoiceAllowed
    };

    let primary_move = if intent_looks_read_only(&normalized_intent) {
        CacheMoveKind::DispatchReadOnlyTool
    } else if intent_looks_actionable(&normalized_intent) {
        CacheMoveKind::Confirm
    } else if intent_looks_ambiguous(&normalized_intent) {
        CacheMoveKind::ClarifyOneQuestion
    } else {
        CacheMoveKind::Respond
    };

    let mut primary_route = route_budget_hint.unwrap_or(CacheRouteHint::Standard);
    if normalized_intent.contains("cost") || normalized_intent.contains("budget") {
        primary_route = CacheRouteHint::CostSaver;
    }
    if normalized_environment.contains("high_load") || normalized_environment.contains("noisy") {
        primary_route = CacheRouteHint::FastTrack;
    }

    let prefetch_hint_enabled = matches!(primary_move, CacheMoveKind::DispatchReadOnlyTool)
        && !privacy_mode
        && !matches!(primary_route, CacheRouteHint::CostSaver);

    let ttl_seconds = min(
        max_ttl_seconds,
        match primary_route {
            CacheRouteHint::FastTrack => default_ttl_seconds,
            CacheRouteHint::Standard => default_ttl_seconds.saturating_add(120),
            CacheRouteHint::CostSaver => default_ttl_seconds.saturating_add(240),
        },
    );

    let intent_bucket = id_bucket(&normalized_intent);
    let env_bucket = id_bucket(&normalized_environment);
    let policy_pack = cache_policy_pack_id.map(collapse_ws);
    let evidence_prefix = format!("cache:evidence:{}:{}", intent_bucket, env_bucket);

    let mut skeletons = vec![CachePlanSkeleton::v1(
        format!("cache:primary:{}:{}", intent_bucket, env_bucket),
        collapse_ws(intent_type),
        collapse_ws(environment_profile_ref),
        persona.clone(),
        primary_move,
        primary_route,
        delivery_hint,
        prefetch_hint_enabled,
        true,
        true,
        ttl_seconds,
        policy_pack.clone(),
        format!("{}:primary", evidence_prefix),
    )?];

    let secondary_move = if matches!(primary_move, CacheMoveKind::ClarifyOneQuestion) {
        CacheMoveKind::Respond
    } else {
        CacheMoveKind::ClarifyOneQuestion
    };
    skeletons.push(CachePlanSkeleton::v1(
        format!("cache:fallback:{}:{}", intent_bucket, env_bucket),
        collapse_ws(intent_type),
        collapse_ws(environment_profile_ref),
        persona.clone(),
        secondary_move,
        CacheRouteHint::Standard,
        delivery_hint,
        false,
        true,
        true,
        min(max_ttl_seconds, ttl_seconds.saturating_add(60)),
        policy_pack.clone(),
        format!("{}:fallback", evidence_prefix),
    )?);

    skeletons.push(CachePlanSkeleton::v1(
        format!("cache:safety:{}:{}", intent_bucket, env_bucket),
        collapse_ws(intent_type),
        collapse_ws(environment_profile_ref),
        persona,
        CacheMoveKind::Wait,
        CacheRouteHint::CostSaver,
        delivery_hint,
        false,
        true,
        true,
        min(max_ttl_seconds, ttl_seconds.saturating_add(120)),
        policy_pack,
        format!("{}:safety", evidence_prefix),
    )?);

    skeletons.truncate(max_skeletons);
    Ok(skeletons)
}

fn intent_looks_read_only(intent: &str) -> bool {
    [
        "QUERY_",
        "WEATHER",
        "TIME",
        "DATE",
        "NEWS",
        "GENERAL_FACT",
        "DEFINITION",
        "STATUS",
    ]
    .iter()
    .any(|hint| intent.contains(hint))
}

fn intent_looks_actionable(intent: &str) -> bool {
    [
        "SEND",
        "BOOK",
        "SCHEDULE",
        "PAY",
        "REMINDER_CREATE",
        "TASK_CREATE",
    ]
    .iter()
    .any(|hint| intent.contains(hint))
}

fn intent_looks_ambiguous(intent: &str) -> bool {
    ["UNKNOWN", "AMBIGUOUS", "CLARIFY", "MISSING"]
        .iter()
        .any(|hint| intent.contains(hint))
}

fn normalize(input: &str) -> String {
    collapse_ws(input)
        .to_ascii_uppercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn collapse_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn id_bucket(input: &str) -> String {
    let bucket = input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if c == '_' || c == '-' || c == ' ' {
                Some('_')
            } else {
                None
            }
        })
        .collect::<String>();

    let collapsed = bucket
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if collapsed.is_empty() {
        "unknown".to_string()
    } else {
        collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1cache::{
        CacheHintSnapshotReadRequest, CacheHintSnapshotRefreshRequest, CacheRequestEnvelope,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1CacheRuntime {
        Ph1CacheRuntime::new(Ph1CacheConfig::mvp_v1())
    }

    fn envelope(max_skeletons: u8, max_diagnostics: u8) -> CacheRequestEnvelope {
        CacheRequestEnvelope::v1(
            CorrelationId(8201),
            TurnId(341),
            max_skeletons,
            max_diagnostics,
        )
        .unwrap()
    }

    fn read_request() -> CacheHintSnapshotReadRequest {
        CacheHintSnapshotReadRequest::v1(
            envelope(4, 8),
            "QUERY_WEATHER".to_string(),
            "office_quiet".to_string(),
            Some("persona_brief".to_string()),
            Some(CacheRouteHint::Standard),
            Some("artifact_cache_pack_v1".to_string()),
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_cache_01_read_output_is_schema_valid() {
        let req = Ph1CacheRequest::CacheHintSnapshotRead(read_request());
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1CacheResponse::CacheHintSnapshotReadOk(ok) => {
                assert!(!ok.ordered_skeletons.is_empty());
                assert_eq!(
                    ok.ordered_skeletons[0].suggested_move,
                    CacheMoveKind::DispatchReadOnlyTool
                );
            }
            _ => panic!("expected CacheHintSnapshotReadOk"),
        }
    }

    #[test]
    fn at_cache_02_read_ordering_is_deterministic() {
        let req = Ph1CacheRequest::CacheHintSnapshotRead(read_request());
        let runtime = runtime();

        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        match (out1, out2) {
            (
                Ph1CacheResponse::CacheHintSnapshotReadOk(a),
                Ph1CacheResponse::CacheHintSnapshotReadOk(b),
            ) => {
                assert_eq!(a.selected_skeleton_id, b.selected_skeleton_id);
                assert_eq!(a.ordered_skeletons, b.ordered_skeletons);
            }
            _ => panic!("expected CacheHintSnapshotReadOk outputs"),
        }
    }

    #[test]
    fn at_cache_03_budget_is_enforced() {
        let runtime = Ph1CacheRuntime::new(Ph1CacheConfig {
            max_skeletons: 1,
            max_diagnostics: 8,
            default_ttl_seconds: 300,
            max_ttl_seconds: 600,
        });

        let read_out = runtime.run(&Ph1CacheRequest::CacheHintSnapshotRead(read_request()));
        let read_ok = match read_out {
            Ph1CacheResponse::CacheHintSnapshotReadOk(ok) => ok,
            _ => panic!("expected read output"),
        };

        let mut over_budget = read_ok.ordered_skeletons.clone();
        let mut extra = read_ok.ordered_skeletons[0].clone();
        extra.skeleton_id = "cache:synthetic:over_budget".to_string();
        extra.evidence_ref = "cache:evidence:synthetic:over_budget".to_string();
        over_budget.push(extra);

        let refresh_req = Ph1CacheRequest::CacheHintSnapshotRefresh(
            CacheHintSnapshotRefreshRequest::v1(
                envelope(4, 8),
                "QUERY_WEATHER".to_string(),
                "office_quiet".to_string(),
                Some("persona_brief".to_string()),
                Some(CacheRouteHint::Standard),
                Some("artifact_cache_pack_v1".to_string()),
                true,
                true,
                read_ok.selected_skeleton_id,
                over_budget,
                false,
            )
            .unwrap(),
        );

        let out = runtime.run(&refresh_req);
        match out {
            Ph1CacheResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_CACHE_BUDGET_EXCEEDED);
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_cache_04_refresh_drift_fails_closed() {
        let read_out = runtime().run(&Ph1CacheRequest::CacheHintSnapshotRead(read_request()));
        let read_ok = match read_out {
            Ph1CacheResponse::CacheHintSnapshotReadOk(ok) => ok,
            _ => panic!("expected CacheHintSnapshotReadOk"),
        };

        let mut drifted = read_ok.ordered_skeletons.clone();
        drifted[0].ttl_seconds = drifted[0].ttl_seconds.saturating_add(30);

        let refresh_req = Ph1CacheRequest::CacheHintSnapshotRefresh(
            CacheHintSnapshotRefreshRequest::v1(
                envelope(4, 8),
                "QUERY_WEATHER".to_string(),
                "office_quiet".to_string(),
                Some("persona_brief".to_string()),
                Some(CacheRouteHint::Standard),
                Some("artifact_cache_pack_v1".to_string()),
                true,
                true,
                read_ok.selected_skeleton_id,
                drifted,
                false,
            )
            .unwrap(),
        );

        let out = runtime().run(&refresh_req);
        match out {
            Ph1CacheResponse::CacheHintSnapshotRefreshOk(ok) => {
                assert_eq!(ok.validation_status, CacheValidationStatus::Fail);
            }
            _ => panic!("expected CacheHintSnapshotRefreshOk"),
        }
    }
}
