#![forbid(unsafe_code)]

use crate::web_search_plan::diag::debug_packet::{DebugPacketContext, DebugStatus};
use crate::web_search_plan::diag::state_trace::{
    MonotonicClock, StateTraceRecorder, TurnStateTransition,
};
use crate::web_search_plan::diag::try_build_debug_packet;
use crate::web_search_plan::learn::failure_signature::{
    compute_signature_id, FailureEvent, LearningLane,
};
use crate::web_search_plan::news::{
    append_news_audit_fields, execute_news_provider_ladder_from_tool_request, NewsAuditMetrics,
    NewsRuntimeConfig,
};
use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::parallel::scheduler::{schedule_deterministically, RetrievalTask};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::planning::open_selector::UrlOpenContext;
use crate::web_search_plan::planning::{
    execute_search_topk_pipeline_with_url_fetch, planning_input_from_tool_request, PlanningPolicy,
    SearchCandidate,
};
use crate::web_search_plan::proxy::proxy_config::ProxyConfig;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::synthesis::insufficiency_gate::EvidenceSufficiencyPolicy;
use crate::web_search_plan::synthesis::{
    append_synthesis_audit_fields, synthesize_evidence_bound, SynthesisPolicy,
};
use crate::web_search_plan::url::{fetch_url_to_evidence_packet, UrlFetchPolicy, UrlFetchRequest};
use crate::web_search_plan::web_provider::health_state::ProviderHealthTracker;
use crate::web_search_plan::web_provider::{
    append_web_provider_audit_fields, execute_web_provider_ladder_from_tool_request,
    WebProviderAuditMetrics, WebProviderRuntimeConfig,
};
use crate::web_search_plan::write::{
    append_write_audit_fields, render_write_packet, WriteFormatMode,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

pub type TurnInputPacket = Value;
pub type SearchAssistPacket = Value;
pub type ToolRequestPacket = Value;
pub type EvidencePacket = Value;
pub type SynthesisPacket = Value;
pub type WritePacket = Value;
pub type AuditPacket = Value;
pub type ReasonCodeId = String;

const RUNTIME_ENGINE_ID: &str = "PH1.OS";

#[derive(Debug, Default)]
pub struct RuntimeServiceTrace {
    learn_observe_calls: AtomicU64,
    parallel_plan_calls: AtomicU64,
}

impl RuntimeServiceTrace {
    pub fn learn_observe_calls(&self) -> u64 {
        self.learn_observe_calls.load(Ordering::SeqCst)
    }

    pub fn parallel_plan_calls(&self) -> u64 {
        self.parallel_plan_calls.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeDependencies {
    pub web_runtime_config: WebProviderRuntimeConfig,
    pub news_runtime_config: NewsRuntimeConfig,
    pub planning_policy: PlanningPolicy,
    pub write_format_mode: WriteFormatMode,
    pub learn_observation_enabled: bool,
    pub service_trace: Option<Arc<RuntimeServiceTrace>>,
}

impl Default for RuntimeDependencies {
    fn default() -> Self {
        Self {
            web_runtime_config: WebProviderRuntimeConfig::default(),
            news_runtime_config: NewsRuntimeConfig::default(),
            planning_policy: PlanningPolicy::default(),
            write_format_mode: WriteFormatMode::Standard,
            learn_observation_enabled: false,
            service_trace: None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeExecutionArtifacts {
    pub evidence_packet: EvidencePacket,
    pub synthesis_packet: SynthesisPacket,
    pub write_packet: WritePacket,
    pub audit_packet: AuditPacket,
    pub transitions: Vec<TurnStateTransition>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RuntimeExecutionError {
    pub reason_code: ReasonCodeId,
    pub transitions: Vec<TurnStateTransition>,
    pub debug_packet: Option<Value>,
}

#[derive(Debug)]
struct RuntimeContext {
    trace_id: String,
    created_at_ms: i64,
    query: String,
    importance_tier: String,
    mode: String,
}

#[derive(Debug)]
struct DeterministicClock {
    next_ms: AtomicI64,
}

impl DeterministicClock {
    fn new(start_ms: i64) -> Self {
        Self {
            next_ms: AtomicI64::new(start_ms),
        }
    }
}

impl MonotonicClock for DeterministicClock {
    fn now_ms(&self) -> i64 {
        self.next_ms.fetch_add(1, Ordering::SeqCst)
    }
}

pub fn execute_web_search_turn(
    turn_input: TurnInputPacket,
    search_assist: SearchAssistPacket,
    tool_request: ToolRequestPacket,
    policy_snapshot_id: String,
) -> Result<(EvidencePacket, SynthesisPacket, WritePacket, AuditPacket), ReasonCodeId> {
    let mut deps = RuntimeDependencies::default();
    execute_web_search_turn_with_dependencies(
        turn_input,
        search_assist,
        tool_request,
        policy_snapshot_id,
        &mut deps,
    )
    .map(|out| {
        (
            out.evidence_packet,
            out.synthesis_packet,
            out.write_packet,
            out.audit_packet,
        )
    })
    .map_err(|err| err.reason_code)
}

pub(crate) fn execute_web_search_turn_with_dependencies(
    turn_input: TurnInputPacket,
    search_assist: SearchAssistPacket,
    tool_request: ToolRequestPacket,
    policy_snapshot_id: String,
    deps: &mut RuntimeDependencies,
) -> Result<RuntimeExecutionArtifacts, RuntimeExecutionError> {
    let context = build_runtime_context(&turn_input, &tool_request).map_err(|reason| {
        RuntimeExecutionError {
            reason_code: reason,
            transitions: vec![
                TurnStateTransition {
                    from: "TURN_ACCEPTED".to_string(),
                    to: "INPUT_PARSED".to_string(),
                    at_ms: 0,
                },
                TurnStateTransition {
                    from: "INPUT_PARSED".to_string(),
                    to: "TURN_FAILED_CLOSED".to_string(),
                    at_ms: 1,
                },
            ],
            debug_packet: None,
        }
    })?;

    let mut recorder = StateTraceRecorder::new(
        DeterministicClock::new(context.created_at_ms),
        "TURN_ACCEPTED",
    )
    .map_err(|_| RuntimeExecutionError {
        reason_code: "policy_violation".to_string(),
        transitions: Vec::new(),
        debug_packet: None,
    })?;
    let mut runtime = RuntimeOrchestrator::new(
        context,
        policy_snapshot_id,
        turn_input,
        search_assist,
        tool_request,
        deps,
        &mut recorder,
    );
    runtime.run()
}

struct RuntimeOrchestrator<'a, C: MonotonicClock> {
    context: RuntimeContext,
    policy_snapshot_id: String,
    turn_input: TurnInputPacket,
    search_assist: SearchAssistPacket,
    tool_request: ToolRequestPacket,
    deps: &'a mut RuntimeDependencies,
    recorder: &'a mut StateTraceRecorder<C>,
}

impl<'a, C: MonotonicClock> RuntimeOrchestrator<'a, C> {
    fn new(
        context: RuntimeContext,
        policy_snapshot_id: String,
        turn_input: TurnInputPacket,
        search_assist: SearchAssistPacket,
        tool_request: ToolRequestPacket,
        deps: &'a mut RuntimeDependencies,
        recorder: &'a mut StateTraceRecorder<C>,
    ) -> Self {
        Self {
            context,
            policy_snapshot_id,
            turn_input,
            search_assist,
            tool_request,
            deps,
            recorder,
        }
    }

    fn run(&mut self) -> Result<RuntimeExecutionArtifacts, RuntimeExecutionError> {
        self.transition_or_fail("INPUT_PARSED")?;
        self.validate_input_packets()?;

        if !self
            .search_assist
            .get("search_required")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            return self.fail_closed(
                "insufficient_evidence",
                "SearchAssist",
                "insufficient_evidence",
            );
        }

        self.transition_or_fail("INTENT_CLASSIFIED")?;
        self.transition_or_fail("PLAN_SELECTED")?;

        let mut web_audit_metrics: Option<WebProviderAuditMetrics> = None;
        let mut news_audit_metrics: Option<NewsAuditMetrics> = None;
        let evidence_packet = match self.context.mode.as_str() {
            "web" | "deep_research" => {
                let now_ms = self.context.created_at_ms;
                let mut health_tracker = ProviderHealthTracker::default();
                let provider_tool_request = if self.context.mode == "deep_research" {
                    tool_request_with_mode(&self.tool_request, "web")
                } else {
                    self.tool_request.clone()
                };
                let web_result = execute_web_provider_ladder_from_tool_request(
                    &provider_tool_request,
                    now_ms,
                    &mut health_tracker,
                    &self.deps.web_runtime_config,
                )
                .map_err(|error| {
                    self.fail_closed_from_provider(
                        error.reason_code(),
                        "BraveWebSearch/OpenAI_WebSearch",
                        error.kind.as_str(),
                        Some(error.message.as_str()),
                    )
                    .expect_err("fail_closed_from_provider always returns Err")
                })?;
                web_audit_metrics = Some(web_result.audit_metrics.clone());

                let candidates = candidates_from_web_sources(&web_result.evidence_packet);
                if candidates.is_empty() {
                    return self.fail_closed(
                        "insufficient_evidence",
                        "WebProvider",
                        "empty_results",
                    );
                }
                let candidates = self.schedule_web_candidates(candidates);

                let retrieved_at_ms = web_result
                    .evidence_packet
                    .get("retrieved_at_ms")
                    .and_then(Value::as_i64)
                    .unwrap_or(self.context.created_at_ms);

                let planning_input = planning_input_from_tool_request(
                    &provider_tool_request,
                    retrieved_at_ms,
                    candidates,
                )
                .map_err(|_| {
                    self.fail_closed("policy_violation", "Planning", "input_unparseable")
                        .expect_err("fail_closed always errors")
                })?;

                let tier = self.context.importance_tier.clone();
                let url_open_cap = match tier.as_str() {
                    "low" => 1,
                    "medium" => 2,
                    "high" => 3,
                    _ => 2,
                };

                let mut planning_policy = self.deps.planning_policy.clone();
                planning_policy.policy_snapshot_id = self.policy_snapshot_id.clone();
                let open_context = UrlOpenContext {
                    trace_id: self.context.trace_id.clone(),
                    query: self.context.query.clone(),
                    importance_tier: tier,
                    created_at_ms: self.context.created_at_ms,
                    retrieved_at_ms,
                    produced_by: "PH1.SEARCH".to_string(),
                    intended_consumers: vec![
                        "PH1.D".to_string(),
                        "PH1.WRITE".to_string(),
                        "PH1.J".to_string(),
                    ],
                    proxy_config: self.deps.web_runtime_config.proxy_config.clone(),
                    url_fetch_policy: UrlFetchPolicy::default(),
                    url_open_cap,
                    cache_enabled: true,
                    cache_policy_snapshot_id: self.policy_snapshot_id.clone(),
                };

                let planning_result = execute_search_topk_pipeline_with_url_fetch(
                    &planning_input,
                    &planning_policy,
                    &open_context,
                )
                .map_err(|_| {
                    self.fail_closed("provider_upstream_failed", "UrlFetch", "transport_failed")
                        .expect_err("fail_closed always errors")
                })?;
                let mut evidence_packet = planning_result.evidence_packet;
                if self.context.mode == "deep_research" {
                    append_deep_research_metadata(
                        &mut evidence_packet,
                        self.context.query.as_str(),
                        self.context.created_at_ms,
                    );
                }
                evidence_packet
            }
            "news" => {
                let now_ms = self.context.created_at_ms;
                let mut health_tracker = ProviderHealthTracker::default();
                let news_result = execute_news_provider_ladder_from_tool_request(
                    &self.tool_request,
                    now_ms,
                    &mut health_tracker,
                    &self.deps.news_runtime_config,
                )
                .map_err(|error| {
                    self.fail_closed_from_provider(
                        error.reason_code(),
                        "BraveNewsSearch/GDELT",
                        error.kind.as_str(),
                        Some(error.message.as_str()),
                    )
                    .expect_err("fail_closed_from_provider always returns Err")
                })?;
                news_audit_metrics = Some(news_result.audit_metrics.clone());
                news_result.evidence_packet
            }
            "url_fetch" => {
                let request = UrlFetchRequest {
                    trace_id: self.context.trace_id.clone(),
                    query: self.context.query.clone(),
                    requested_url: self.context.query.clone(),
                    importance_tier: self.context.importance_tier.clone(),
                    url_open_ordinal: 0,
                    url_open_cap: Some(1),
                    created_at_ms: self.context.created_at_ms,
                    retrieved_at_ms: self.context.created_at_ms,
                    produced_by: "PH1.E".to_string(),
                    intended_consumers: vec![
                        "PH1.D".to_string(),
                        "PH1.WRITE".to_string(),
                        "PH1.J".to_string(),
                    ],
                    proxy_config: default_proxy_config(),
                    policy: UrlFetchPolicy::default(),
                };
                let success = fetch_url_to_evidence_packet(&request).map_err(|failure| {
                    self.fail_closed_from_provider(
                        failure.reason_code,
                        "UrlFetch",
                        failure.error_kind.as_str(),
                        Some(failure.message.as_str()),
                    )
                    .expect_err("fail_closed_from_provider always returns Err")
                })?;
                success.evidence_packet
            }
            _ => {
                return self.fail_closed("policy_violation", "RuntimeRouter", "mode_not_supported");
            }
        };

        self.transition_or_fail("RETRIEVAL_EXECUTED")?;
        self.validate_packet_or_fail("EvidencePacket", &evidence_packet)?;
        self.transition_or_fail("EVIDENCE_LOCKED")?;

        let synthesis_policy = if self.context.mode == "deep_research" {
            SynthesisPolicy {
                sufficiency: EvidenceSufficiencyPolicy {
                    min_distinct_sources: 1,
                    min_chunk_support: 1,
                },
                max_claims: 8,
            }
        } else {
            SynthesisPolicy::default()
        };

        let synthesis = synthesize_evidence_bound(
            self.context.query.as_str(),
            &evidence_packet,
            self.context.created_at_ms.saturating_add(100),
            self.context.trace_id.as_str(),
            synthesis_policy,
            None,
        )
        .map_err(|error| match error {
            crate::web_search_plan::synthesis::SynthesisError::EvidenceBoundaryViolation(_) => self
                .fail_closed("policy_violation", "Synthesis", "policy_violation")
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::synthesis::SynthesisError::InvalidEvidence(_) => self
                .fail_closed(
                    "insufficient_evidence",
                    "Synthesis",
                    "insufficient_evidence",
                )
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::synthesis::SynthesisError::CitationMismatch(_) => self
                .fail_closed("citation_mismatch", "Synthesis", "citation_mismatch")
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::synthesis::SynthesisError::UnsupportedClaim(_) => self
                .fail_closed("policy_violation", "Synthesis", "unsupported_claim")
                .expect_err("fail_closed always errors"),
        })?;

        if synthesis
            .synthesis_packet
            .get("reason_codes")
            .and_then(Value::as_array)
            .map(|codes| {
                codes
                    .iter()
                    .any(|code| code.as_str() == Some("insufficient_evidence"))
            })
            .unwrap_or(false)
        {
            return self.fail_closed(
                "insufficient_evidence",
                "Synthesis",
                "insufficient_evidence",
            );
        }

        self.transition_or_fail("SYNTHESIS_READY")?;
        self.validate_packet_or_fail("SynthesisPacket", &synthesis.synthesis_packet)?;

        let write = render_write_packet(
            &synthesis.synthesis_packet,
            self.context.created_at_ms.saturating_add(200),
            self.context.trace_id.as_str(),
            self.deps.write_format_mode,
        )
        .map_err(|error| match error {
            crate::web_search_plan::write::WriteError::InvalidSynthesis(_) => self
                .fail_closed("input_unparseable", "Write", "input_unparseable")
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::write::WriteError::CitationMismatch(_) => self
                .fail_closed("citation_mismatch", "Write", "citation_mismatch")
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::write::WriteError::UnsupportedClaim(_) => self
                .fail_closed("policy_violation", "Write", "unsupported_claim")
                .expect_err("fail_closed always errors"),
            crate::web_search_plan::write::WriteError::StyleGuardViolation(_) => self
                .fail_closed("policy_violation", "Write", "policy_violation")
                .expect_err("fail_closed always errors"),
        })?;

        self.transition_or_fail("OUTPUT_RENDERED")?;
        self.validate_packet_or_fail("WritePacket", &write.write_packet)?;

        self.transition_or_fail("AUDIT_COMMITTED")?;
        self.transition_or_fail("TURN_COMPLETED")?;
        let transitions = self.recorder.transitions().to_vec();

        let mut audit_packet = build_audit_packet(
            self.context.trace_id.as_str(),
            self.context.created_at_ms.saturating_add(300),
            self.policy_snapshot_id.as_str(),
            &evidence_packet,
            &synthesis.synthesis_packet,
            &write.write_packet,
            transitions.as_slice(),
        )
        .map_err(|_| {
            self.fail_closed("policy_violation", "Audit", "policy_violation")
                .expect_err("fail_closed always errors")
        })?;

        if let Some(metrics) = web_audit_metrics.as_ref() {
            let _ = append_web_provider_audit_fields(&mut audit_packet, metrics);
        }
        if let Some(metrics) = news_audit_metrics.as_ref() {
            let _ = append_news_audit_fields(&mut audit_packet, metrics);
        }
        let _ = append_synthesis_audit_fields(&mut audit_packet, &synthesis.audit_metrics);
        let _ = append_write_audit_fields(&mut audit_packet, &write.audit_metrics);
        self.validate_packet_or_fail("AuditPacket", &audit_packet)?;

        Ok(RuntimeExecutionArtifacts {
            evidence_packet,
            synthesis_packet: synthesis.synthesis_packet,
            write_packet: write.write_packet,
            audit_packet,
            transitions,
        })
    }

    fn transition_or_fail(&mut self, state: &str) -> Result<(), RuntimeExecutionError> {
        self.recorder
            .transition(state)
            .map_err(|_| RuntimeExecutionError {
                reason_code: "policy_violation".to_string(),
                transitions: self.recorder.transitions().to_vec(),
                debug_packet: None,
            })
    }

    fn validate_input_packets(&mut self) -> Result<(), RuntimeExecutionError> {
        let registry = load_packet_schema_registry().map_err(|_| RuntimeExecutionError {
            reason_code: "policy_violation".to_string(),
            transitions: self.recorder.transitions().to_vec(),
            debug_packet: None,
        })?;
        validate_packet("TurnInputPacket", &self.turn_input, &registry).map_err(|_| {
            self.fail_closed("policy_violation", "RuntimeInput", "invalid_turn_input")
                .expect_err("fail_closed always errors")
        })?;
        validate_packet("SearchAssistPacket", &self.search_assist, &registry).map_err(|_| {
            self.fail_closed("policy_violation", "RuntimeInput", "invalid_search_assist")
                .expect_err("fail_closed always errors")
        })?;
        validate_packet("ToolRequestPacket", &self.tool_request, &registry).map_err(|_| {
            self.fail_closed("policy_violation", "RuntimeInput", "invalid_tool_request")
                .expect_err("fail_closed always errors")
        })?;
        Ok(())
    }

    fn validate_packet_or_fail(
        &mut self,
        packet_name: &str,
        packet: &Value,
    ) -> Result<(), RuntimeExecutionError> {
        let registry = load_packet_schema_registry().map_err(|_| RuntimeExecutionError {
            reason_code: "policy_violation".to_string(),
            transitions: self.recorder.transitions().to_vec(),
            debug_packet: None,
        })?;
        validate_packet(packet_name, packet, &registry).map_err(|_| {
            self.fail_closed(
                "policy_violation",
                "RuntimeValidator",
                "packet_validation_failed",
            )
            .expect_err("fail_closed always errors")
        })
    }

    fn fail_closed_from_provider(
        &mut self,
        reason_code: &str,
        provider: &str,
        error_kind: &str,
        debug_hint: Option<&str>,
    ) -> Result<RuntimeExecutionArtifacts, RuntimeExecutionError> {
        self.fail_closed_with_hint(reason_code, provider, error_kind, debug_hint)
    }

    fn fail_closed(
        &mut self,
        reason_code: &str,
        provider: &str,
        error_kind: &str,
    ) -> Result<RuntimeExecutionArtifacts, RuntimeExecutionError> {
        self.fail_closed_with_hint(reason_code, provider, error_kind, None)
    }

    fn fail_closed_with_hint(
        &mut self,
        reason_code: &str,
        provider: &str,
        error_kind: &str,
        debug_hint: Option<&str>,
    ) -> Result<RuntimeExecutionArtifacts, RuntimeExecutionError> {
        let failure_signature_id =
            self.observe_failure_signature(reason_code, provider, error_kind);
        let debug_hint_owned =
            append_failure_signature_debug_hint(debug_hint, failure_signature_id.as_str());
        let _ = self.recorder.transition("TURN_FAILED_CLOSED");
        let transitions = self.recorder.transitions().to_vec();
        let debug_packet = try_build_debug_packet(DebugPacketContext {
            trace_id: self.context.trace_id.as_str(),
            status: DebugStatus::Failed,
            provider,
            error_kind,
            reason_code,
            proxy_mode: None,
            source_url: None,
            created_at_ms: self.context.created_at_ms,
            turn_state_transitions: transitions.as_slice(),
            debug_hint: Some(debug_hint_owned.as_str()),
            fallback_used: None,
            health_status_before_fallback: None,
        })
        .ok()
        .and_then(|packet| serde_json::to_value(packet).ok());

        Err(RuntimeExecutionError {
            reason_code: reason_code.to_string(),
            transitions,
            debug_packet,
        })
    }

    fn schedule_web_candidates(
        &mut self,
        candidates: Vec<SearchCandidate>,
    ) -> Vec<SearchCandidate> {
        let mut candidates_by_task_id = BTreeMap::new();
        let tasks = candidates
            .into_iter()
            .enumerate()
            .map(|(index, candidate)| {
                let task_id = format!("runtime-plan-{:06}", index);
                candidates_by_task_id.insert(task_id.clone(), candidate.clone());
                RetrievalTask {
                    task_id,
                    priority: candidate.provider_rank as u32,
                    canonical_url: candidate.canonical_url,
                    provider_id: candidate.provider_id,
                    task_type: "url_fetch".to_string(),
                }
            })
            .collect::<Vec<RetrievalTask>>();
        let scheduled_tasks = schedule_deterministically(tasks);

        if let Some(trace) = self.deps.service_trace.as_ref() {
            trace.parallel_plan_calls.fetch_add(1, Ordering::SeqCst);
        }

        scheduled_tasks
            .into_iter()
            .filter_map(|task| candidates_by_task_id.remove(task.task_id.as_str()))
            .collect()
    }

    fn observe_failure_signature(
        &mut self,
        reason_code: &str,
        provider: &str,
        error_kind: &str,
    ) -> String {
        if let Some(trace) = self.deps.service_trace.as_ref() {
            trace.learn_observe_calls.fetch_add(1, Ordering::SeqCst);
        }

        let event = FailureEvent {
            lane: lane_from_mode(self.context.mode.as_str()),
            provider_id: Some(provider.to_string()),
            error_kind: error_kind.to_string(),
            reason_code_id: reason_code.to_string(),
            importance_tier: ImportanceTier::parse_or_default(
                self.context.importance_tier.as_str(),
            ),
            canonical_url: None,
            occurred_at_ms: self.context.created_at_ms,
            ttl_ms: 0,
        };
        compute_signature_id(&event)
    }
}

fn build_runtime_context(
    turn_input: &TurnInputPacket,
    tool_request: &ToolRequestPacket,
) -> Result<RuntimeContext, ReasonCodeId> {
    let trace_id = tool_request
        .get("trace_id")
        .and_then(Value::as_str)
        .or_else(|| turn_input.get("trace_id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "policy_violation".to_string())?
        .to_string();
    let created_at_ms = tool_request
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .or_else(|| turn_input.get("created_at_ms").and_then(Value::as_i64))
        .ok_or_else(|| "policy_violation".to_string())?;
    let query = tool_request
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "policy_violation".to_string())?
        .to_string();
    let importance_tier = tool_request
        .get("importance_tier")
        .and_then(Value::as_str)
        .unwrap_or("medium")
        .trim()
        .to_ascii_lowercase();
    let mode = tool_request
        .get("mode")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "policy_violation".to_string())?
        .to_ascii_lowercase();

    Ok(RuntimeContext {
        trace_id,
        created_at_ms,
        query,
        importance_tier,
        mode,
    })
}

fn build_audit_packet(
    trace_id: &str,
    created_at_ms: i64,
    policy_snapshot_id: &str,
    evidence_packet: &Value,
    synthesis_packet: &Value,
    write_packet: &Value,
    transitions: &[TurnStateTransition],
) -> Result<Value, String> {
    let evidence_hash = hash_canonical_json(evidence_packet)?;
    let synthesis_hash = hash_canonical_json(synthesis_packet)?;
    let write_hash = hash_canonical_json(write_packet)?;
    let reason_codes = synthesis_packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(json!({
        "schema_version": "1.0.0",
        "produced_by": RUNTIME_ENGINE_ID,
        "intended_consumers": ["PH1.J"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "turn_state_transition": {
            "state": "TURN_COMPLETED",
            "transitions": transitions,
        },
        "packet_hashes": {
            "evidence": evidence_hash,
            "synthesis": synthesis_hash,
            "write": write_hash,
        },
        "evidence_hash": evidence_hash,
        "response_hash": write_hash,
        "reason_codes": reason_codes,
        "policy_snapshot_id": policy_snapshot_id,
    }))
}

fn candidates_from_web_sources(evidence_packet: &Value) -> Vec<SearchCandidate> {
    let mut candidates = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|source| {
            let title = source
                .get("title")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            let url = source
                .get("url")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            let canonical_url = source
                .get("canonical_url")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(url.as_str())
                .to_string();
            if title.is_empty() || url.is_empty() || canonical_url.is_empty() {
                return None;
            }

            let rank = source
                .get("rank")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(1);
            let snippet = source
                .get("snippet")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let trust_tier = source
                .get("trust_tier_score")
                .and_then(Value::as_i64)
                .or_else(|| {
                    source
                        .get("trust_score")
                        .and_then(Value::as_f64)
                        .map(|v| v as i64)
                })
                .unwrap_or(50) as i32;
            let freshness_score = source
                .get("freshness_score")
                .and_then(Value::as_f64)
                .map(|value| (value * 100.0).round() as i32)
                .unwrap_or(50);

            Some(SearchCandidate {
                title,
                url,
                snippet,
                canonical_url,
                provider_id: source
                    .get("provider_id")
                    .and_then(Value::as_str)
                    .unwrap_or("web_provider_ladder")
                    .to_string(),
                provider_rank: rank,
                relevance: (100i32.saturating_sub(rank as i32)).max(1),
                trust_tier,
                freshness_score,
                corroboration_count: 1,
                spam_risk: 0,
            })
        })
        .collect::<Vec<SearchCandidate>>();

    candidates.sort_by(|left, right| {
        left.provider_rank
            .cmp(&right.provider_rank)
            .then_with(|| left.canonical_url.cmp(&right.canonical_url))
    });
    candidates
}

fn tool_request_with_mode(tool_request: &Value, mode: &str) -> Value {
    let mut cloned = tool_request.clone();
    if let Some(obj) = cloned.as_object_mut() {
        obj.insert("mode".to_string(), json!(mode));
    }
    cloned
}

fn append_deep_research_metadata(evidence_packet: &mut Value, query: &str, created_at_ms: i64) {
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let content_chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let source_chips = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            let url = source.get("url").and_then(Value::as_str)?.trim();
            let title = source.get("title").and_then(Value::as_str).unwrap_or(url);
            if url.is_empty() {
                return None;
            }
            Some(json!({
                "chip_id": format!("source_chip_{:03}", index + 1),
                "claim_id": format!("claim_{:03}", index + 1),
                "citation_ids": [format!("citation_card_{:03}", index + 1)],
                "display_label": source_chip_label(title, url),
                "primary_domain": domain_from_url(url),
                "additional_source_count": 0,
                "trust_tier": source
                    .get("trust_tier")
                    .or_else(|| source.get("trust_tier_score"))
                    .cloned()
                    .unwrap_or_else(|| json!("UNKNOWN")),
                "freshness_tier": source
                    .get("freshness_tier")
                    .cloned()
                    .unwrap_or_else(|| json!("STABLE_REFERENCE_ACCEPTABLE")),
                "display_position": "after_claim",
                "source_urls": [url],
                "display_safe": true
            }))
        })
        .collect::<Vec<Value>>();

    let citation_cards = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            let url = source.get("url").and_then(Value::as_str)?.trim();
            if url.is_empty() {
                return None;
            }
            let title = source
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Source");
            let evidence_excerpt = content_chunks
                .iter()
                .find(|chunk| {
                    chunk
                        .get("source_url")
                        .and_then(Value::as_str)
                        .map(|source_url| source_url == url)
                        .unwrap_or(false)
                })
                .and_then(|chunk| {
                    chunk
                        .get("text_excerpt")
                        .or_else(|| chunk.get("excerpt"))
                        .and_then(Value::as_str)
                })
                .unwrap_or(title);
            Some(json!({
                "citation_id": format!("citation_card_{:03}", index + 1),
                "title": title,
                "domain": domain_from_url(url),
                "publisher": source.get("publisher").cloned().unwrap_or(Value::Null),
                "published_at": source.get("published_at").cloned().unwrap_or(Value::Null),
                "retrieved_at": evidence_packet.get("retrieved_at_ms").cloned().unwrap_or_else(|| json!(created_at_ms)),
                "trust_tier": source
                    .get("trust_tier")
                    .or_else(|| source.get("trust_tier_score"))
                    .cloned()
                    .unwrap_or_else(|| json!("UNKNOWN")),
                "freshness_tier": source
                    .get("freshness_tier")
                    .cloned()
                    .unwrap_or_else(|| json!("STABLE_REFERENCE_ACCEPTABLE")),
                "evidence_excerpt": truncate_chars(evidence_excerpt, 240),
                "supports_claim_ids": [format!("claim_{:03}", index + 1)],
                "conflict_marker": Value::Null,
                "source_url": url,
                "display_safe": true
            }))
        })
        .collect::<Vec<Value>>();

    if let Some(obj) = evidence_packet.as_object_mut() {
        let trust_metadata = obj
            .entry("trust_metadata".to_string())
            .or_insert_with(|| json!({}));
        if let Some(trust_obj) = trust_metadata.as_object_mut() {
            trust_obj.insert(
                "deep_research".to_string(),
                json!({
                    "research_plan": {
                        "research_goal": query,
                        "planned_queries": [query],
                        "planned_fetches": content_chunks.len(),
                        "max_steps": 3,
                        "max_sources": sources.len(),
                        "expected_output_format": "markdown",
                        "protected_risk_detected": false,
                        "confirmation_required": false,
                        "reason_codes": ["DEEP_RESEARCH_PLAN_PASS", "PH1_SEARCH_DEEP_RESEARCH_PLANNING_PASS"]
                    },
                    "source_scope": {
                        "allowed_domains": [],
                        "blocked_domains": [],
                        "preferred_domains": [],
                        "official_source_domains": [],
                        "source_type_preferences": ["PRIMARY_OFFICIAL", "REPUTABLE_NEWS", "DOCUMENTATION"],
                        "user_requested_domains": [],
                        "admin_policy_domains": [],
                        "final_effective_scope": "public_web_only"
                    },
                    "multihop": {
                        "max_hops": 3,
                        "executed_hops": 1,
                        "hop_strategy": "bounded_public_web_evidence_then_source_open"
                    },
                    "source_chips": source_chips,
                    "citation_cards": citation_cards,
                    "image_source_cards": [],
                    "image_source_card_status": "WEB_IMAGE_SOURCE_CARD_DEFERRED",
                    "citation_correction_loop": {
                        "status": "session_metadata_ready",
                        "historical_audit_rewrite_allowed": false
                    },
                    "retention_class": "AUDIT_METADATA_ONLY"
                }),
            );
        }
    }
}

fn domain_from_url(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase()
}

fn source_chip_label(title: &str, url: &str) -> String {
    let domain = domain_from_url(url);
    let title = title.trim();
    if title.is_empty() {
        domain
    } else {
        truncate_chars(title, 48)
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in value.chars().take(max_chars) {
        out.push(ch);
    }
    out
}

fn default_proxy_config() -> ProxyConfig {
    ProxyConfig {
        mode: crate::web_search_plan::proxy::ProxyMode::Off,
        http_proxy_url: None,
        https_proxy_url: None,
    }
}

fn lane_from_mode(mode: &str) -> LearningLane {
    match mode {
        "news" => LearningLane::News,
        "url_fetch" => LearningLane::UrlFetch,
        "synthesis" => LearningLane::Synthesis,
        "write" => LearningLane::Write,
        "images" => LearningLane::Images,
        "video" => LearningLane::Video,
        _ => LearningLane::Web,
    }
}

fn append_failure_signature_debug_hint(base_hint: Option<&str>, signature_id: &str) -> String {
    let signature_fragment = format!("failure_signature_id:{}", signature_id);
    match base_hint.map(str::trim).filter(|hint| !hint.is_empty()) {
        Some(hint) => format!("{} | {}", hint, signature_fragment),
        None => signature_fragment,
    }
}
