#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::{run_numeric_consensus, structured_rows_from_evidence};
use crate::web_search_plan::competitive::schema::CompetitiveRequest;
use crate::web_search_plan::competitive::{parse_computation_packet, run_competitive_mode};
use crate::web_search_plan::document::{execute_document_pipeline_from_tool_request, DocumentRuntimeConfig};
use crate::web_search_plan::enterprise::consistency::{validate_cross_mode_consistency, ConsistencyInputs};
use crate::web_search_plan::enterprise::enterprise_request::EnterpriseRequest;
use crate::web_search_plan::enterprise::mode_router::EnterpriseMode;
use crate::web_search_plan::enterprise::provenance::{build_enterprise_provenance, EnterpriseProvenance};
use crate::web_search_plan::merge::{run_internal_external_merge, MergeRequest};
use crate::web_search_plan::multihop::{
    build_hop_plan, execute_hop_plan, HopBudget, HopExecutionOutput, HopExecutor, HopMode, HopPlanInput,
    ProviderRunSummary,
};
use crate::web_search_plan::realtime::{execute_realtime_from_tool_request, RealtimeRuntimeConfig};
use crate::web_search_plan::regulatory::apply_regulatory_mode;
use crate::web_search_plan::risk::{build_risk_packet, RiskRequest};
use crate::web_search_plan::structured::types::{StructuredRow, StructuredRuntimeConfig};
use crate::web_search_plan::structured::execute_structured_from_tool_request;
use crate::web_search_plan::temporal::{build_temporal_comparison_packet, TemporalRequest};
use crate::web_search_plan::trust::enrich_evidence_sources;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterprisePipelineOutput {
    pub mode: String,
    pub stage_trace: Vec<String>,
    pub evidence_packet: Value,
    pub computation_packet: Option<Value>,
    pub competitive_packet: Option<Value>,
    pub temporal_packet: Option<Value>,
    pub risk_packet: Option<Value>,
    pub merge_packet: Option<Value>,
    pub report_packet: Option<Value>,
    pub multihop_packet: Option<Value>,
    pub provenance: EnterpriseProvenance,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterprisePipelineError {
    pub reason_code: &'static str,
    pub message: String,
}

impl EnterprisePipelineError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

pub fn run_enterprise_pipeline(
    request: &EnterpriseRequest,
    now_ms: i64,
) -> Result<EnterprisePipelineOutput, EnterprisePipelineError> {
    let mut stage_trace = Vec::new();
    let mut reason_codes = BTreeSet::new();

    let mut base_rows: Option<Vec<StructuredRow>> = None;
    let mut evidence_packet = if let Some(existing) = request.evidence_packet.as_ref() {
        stage_trace.push("base_evidence:provided".to_string());
        existing.clone()
    } else {
        let tool_request = request
            .tool_request_packet
            .as_ref()
            .ok_or_else(|| EnterprisePipelineError::new(
                "insufficient_evidence",
                "enterprise pipeline requires either evidence_packet or tool_request_packet",
            ))?;
        match request.mode {
            EnterpriseMode::Structured => {
                stage_trace.push("base_evidence:structured".to_string());
                let result = execute_structured_from_tool_request(
                    tool_request,
                    now_ms,
                    &StructuredRuntimeConfig::default(),
                )
                .map_err(|error| {
                    EnterprisePipelineError::new(error.reason_code(), error.message)
                })?;
                base_rows = Some(result.extraction.rows);
                result.evidence_packet
            }
            EnterpriseMode::Document => {
                stage_trace.push("base_evidence:document".to_string());
                let result = execute_document_pipeline_from_tool_request(
                    tool_request,
                    now_ms,
                    &DocumentRuntimeConfig::default(),
                )
                .map_err(|error| EnterprisePipelineError::new(error.reason_code(), error.message))?;
                base_rows = Some(result.extraction.rows);
                result.evidence_packet
            }
            EnterpriseMode::Realtime => {
                stage_trace.push("base_evidence:realtime".to_string());
                let result = execute_realtime_from_tool_request(
                    tool_request,
                    now_ms,
                    &RealtimeRuntimeConfig::default(),
                )
                .map_err(|error| EnterprisePipelineError::new(error.reason_code(), error.message))?;
                result.evidence_packet
            }
            _ => {
                return Err(EnterprisePipelineError::new(
                    "insufficient_evidence",
                    "non-retrieval enterprise modes require prebuilt evidence_packet",
                ))
            }
        }
    };

    stage_trace.push("trust_enrichment".to_string());
    let trust_applied = enrich_evidence_sources(&evidence_packet, now_ms).map_err(|error| {
        EnterprisePipelineError::new(error.reason_code(), error.message)
    })?;
    evidence_packet = trust_applied.evidence_packet;

    if should_apply_regulatory(request) {
        stage_trace.push("regulatory_filter".to_string());
        let regulatory_request = request.to_regulatory_tool_request();
        let regulatory_result = apply_regulatory_mode(&regulatory_request, &evidence_packet)
            .map_err(|error| EnterprisePipelineError::new(error.reason_code(), error.message))?;
        for code in &regulatory_result.reason_codes {
            reason_codes.insert(code.clone());
        }
        evidence_packet = regulatory_result.evidence_packet;
    }

    let resolved_rows = resolve_structured_rows(request, base_rows, &evidence_packet)?;
    if !resolved_rows.is_empty() {
        stage_trace.push("structured_rows:resolved".to_string());
    }

    let computation_packet = resolve_computation_packet(
        request,
        now_ms,
        &evidence_packet,
        resolved_rows.as_slice(),
        &mut stage_trace,
        &mut reason_codes,
    )?;

    let mut competitive_packet = None;
    let mut temporal_packet = None;
    let mut risk_packet = None;
    let mut merge_packet = None;
    let mut report_packet = None;
    let mut multihop_packet = None;

    match request.mode {
        EnterpriseMode::Competitive => {
            stage_trace.push("mode_output:competitive".to_string());
            competitive_packet = Some(run_competitive(
                request,
                &evidence_packet,
                resolved_rows.as_slice(),
                computation_packet.as_ref(),
            )?);
        }
        EnterpriseMode::Temporal => {
            stage_trace.push("mode_output:temporal".to_string());
            temporal_packet = Some(run_temporal(
                request,
                now_ms,
                &evidence_packet,
                resolved_rows.as_slice(),
            )?);
        }
        EnterpriseMode::Risk => {
            stage_trace.push("mode_output:risk".to_string());
            risk_packet = Some(run_risk(
                request,
                &evidence_packet,
                computation_packet.as_ref(),
            )?);
        }
        EnterpriseMode::Merge => {
            stage_trace.push("mode_output:merge".to_string());
            merge_packet = Some(run_merge(request, &evidence_packet)?);
        }
        EnterpriseMode::Multihop => {
            stage_trace.push("mode_output:multihop".to_string());
            multihop_packet = Some(run_multihop(request, &evidence_packet)?);
        }
        EnterpriseMode::Report => {
            stage_trace.push("mode_output:report.compose".to_string());
            if !resolved_rows.is_empty() {
                competitive_packet = Some(run_competitive(
                    request,
                    &evidence_packet,
                    resolved_rows.as_slice(),
                    computation_packet.as_ref(),
                )?);
                temporal_packet = Some(run_temporal(
                    request,
                    now_ms,
                    &evidence_packet,
                    resolved_rows.as_slice(),
                )?);
            }
            risk_packet = Some(run_risk(
                request,
                &evidence_packet,
                computation_packet.as_ref(),
            )?);
            merge_packet = Some(run_merge(request, &evidence_packet)?);
            report_packet = Some(build_report_packet(
                request,
                now_ms,
                &evidence_packet,
                competitive_packet.as_ref(),
                temporal_packet.as_ref(),
                risk_packet.as_ref(),
                merge_packet.as_ref(),
            ));
        }
        EnterpriseMode::Structured
        | EnterpriseMode::Document
        | EnterpriseMode::Realtime
        | EnterpriseMode::Regulatory
        | EnterpriseMode::Trust => {
            stage_trace.push("mode_output:evidence_only".to_string());
            report_packet = Some(build_report_packet(
                request,
                now_ms,
                &evidence_packet,
                None,
                None,
                None,
                None,
            ));
        }
    }

    stage_trace.push("consistency_validation".to_string());
    validate_cross_mode_consistency(ConsistencyInputs {
        evidence_packet: &evidence_packet,
        competitive_packet: competitive_packet.as_ref(),
        temporal_packet: temporal_packet.as_ref(),
        risk_packet: risk_packet.as_ref(),
        merge_packet: merge_packet.as_ref(),
        report_packet: report_packet.as_ref(),
    })
    .map_err(|error| EnterprisePipelineError::new(error.reason_code, error.message))?;

    let mut output_pairs = Vec::new();
    if let Some(packet) = competitive_packet.as_ref() {
        output_pairs.push(("competitive", packet));
    }
    if let Some(packet) = temporal_packet.as_ref() {
        output_pairs.push(("temporal", packet));
    }
    if let Some(packet) = risk_packet.as_ref() {
        output_pairs.push(("risk", packet));
    }
    if let Some(packet) = merge_packet.as_ref() {
        output_pairs.push(("merge", packet));
    }
    if let Some(packet) = report_packet.as_ref() {
        output_pairs.push(("report", packet));
    }
    if let Some(packet) = multihop_packet.as_ref() {
        output_pairs.push(("multihop", packet));
    }

    let reason_codes_vec = reason_codes.into_iter().collect::<Vec<String>>();
    let provenance = build_enterprise_provenance(
        request.mode,
        &evidence_packet,
        computation_packet.as_ref(),
        output_pairs.as_slice(),
        &reason_codes_vec,
    )
    .map_err(|error| EnterprisePipelineError::new("policy_violation", error))?;

    Ok(EnterprisePipelineOutput {
        mode: request.mode.as_str().to_string(),
        stage_trace,
        evidence_packet,
        computation_packet,
        competitive_packet,
        temporal_packet,
        risk_packet,
        merge_packet,
        report_packet,
        multihop_packet,
        provenance,
        reason_codes: reason_codes_vec,
    })
}

fn should_apply_regulatory(request: &EnterpriseRequest) -> bool {
    matches!(request.mode, EnterpriseMode::Regulatory)
        || request.constraints.require_regulatory_filter
        || request.jurisdiction.is_some()
}

fn resolve_structured_rows(
    request: &EnterpriseRequest,
    base_rows: Option<Vec<StructuredRow>>,
    evidence_packet: &Value,
) -> Result<Vec<StructuredRow>, EnterprisePipelineError> {
    if let Some(rows) = request.structured_rows.as_ref() {
        return Ok(rows.clone());
    }
    if let Some(rows) = base_rows {
        return Ok(rows);
    }
    match structured_rows_from_evidence(evidence_packet) {
        Ok(rows) => Ok(rows),
        Err(_) => Ok(Vec::new()),
    }
}

fn resolve_computation_packet(
    request: &EnterpriseRequest,
    now_ms: i64,
    evidence_packet: &Value,
    structured_rows: &[StructuredRow],
    stage_trace: &mut Vec<String>,
    reason_codes: &mut BTreeSet<String>,
) -> Result<Option<Value>, EnterprisePipelineError> {
    if let Some(packet) = request.computation_packet.as_ref() {
        stage_trace.push("computation:provided".to_string());
        return Ok(Some(packet.clone()));
    }
    if !needs_computation(request.mode) || structured_rows.is_empty() {
        return Ok(None);
    }
    stage_trace.push("computation:run_numeric_consensus".to_string());
    match run_numeric_consensus(
        request.trace_id.clone(),
        request.created_at_ms.max(now_ms),
        request.policy_snapshot_id.clone(),
        request.as_of_to_ms,
        evidence_packet.clone(),
        structured_rows.to_vec(),
    ) {
        Ok(packet) => serde_json::to_value(packet)
            .map(Some)
            .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string())),
        Err(error) => {
            reason_codes.insert(error.reason_code().to_string());
            Ok(None)
        }
    }
}

fn needs_computation(mode: EnterpriseMode) -> bool {
    matches!(
        mode,
        EnterpriseMode::Competitive | EnterpriseMode::Temporal | EnterpriseMode::Risk | EnterpriseMode::Report
    )
}

fn run_competitive(
    request: &EnterpriseRequest,
    evidence_packet: &Value,
    structured_rows: &[StructuredRow],
    computation_packet: Option<&Value>,
) -> Result<Value, EnterprisePipelineError> {
    let target_entity = target_entity_for_request(request)?;
    let parsed_computation = parse_computation_packet(computation_packet).map_err(|error| {
        EnterprisePipelineError::new(error.reason_code(), error.message)
    })?;
    let packet = run_competitive_mode(CompetitiveRequest {
        trace_id: request.trace_id.clone(),
        created_at_ms: request.created_at_ms,
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        target_entity,
        evidence_packet: evidence_packet.clone(),
        structured_rows: structured_rows.to_vec(),
        computation_packet: parsed_computation,
    })
    .map_err(|error| EnterprisePipelineError::new(error.reason_code(), error.message))?;

    serde_json::to_value(packet)
        .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string()))
}

fn run_temporal(
    request: &EnterpriseRequest,
    now_ms: i64,
    evidence_packet: &Value,
    structured_rows: &[StructuredRow],
) -> Result<Value, EnterprisePipelineError> {
    if structured_rows.is_empty() {
        return Err(EnterprisePipelineError::new(
            "insufficient_evidence",
            "temporal mode requires structured rows",
        ));
    }

    let temporal_request = TemporalRequest {
        trace_id: request.trace_id.clone(),
        created_at_ms: request.created_at_ms,
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        now_ms,
        baseline_from_ms: request.as_of_from_ms,
        baseline_to_ms: request.as_of_to_ms,
        compare_from_ms: None,
        compare_to_ms: None,
        allow_default_windows: request.as_of_from_ms.is_none() || request.as_of_to_ms.is_none(),
        policy_snapshot_id: request.policy_snapshot_id.clone(),
    };
    let output = build_temporal_comparison_packet(&temporal_request, evidence_packet, structured_rows)
        .map_err(|error| EnterprisePipelineError::new(error.reason_code, error.message))?;
    serde_json::to_value(output.packet)
        .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string()))
}

fn run_risk(
    request: &EnterpriseRequest,
    evidence_packet: &Value,
    computation_packet: Option<&Value>,
) -> Result<Value, EnterprisePipelineError> {
    let packet = build_risk_packet(&RiskRequest {
        trace_id: request.trace_id.clone(),
        created_at_ms: request.created_at_ms,
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        evidence_packet: evidence_packet.clone(),
        computation_packet: computation_packet.cloned(),
    })
    .map_err(|error| EnterprisePipelineError::new(error.reason_code, error.message))?;

    serde_json::to_value(packet)
        .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string()))
}

fn run_merge(request: &EnterpriseRequest, evidence_packet: &Value) -> Result<Value, EnterprisePipelineError> {
    let packet = run_internal_external_merge(MergeRequest {
        trace_id: request.trace_id.clone(),
        created_at_ms: request.created_at_ms,
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        policy_snapshot_id: request.policy_snapshot_id.clone(),
        evidence_packet: evidence_packet.clone(),
        internal_context: request.internal_context.clone(),
    })
    .map_err(|error| EnterprisePipelineError::new(error.reason_code, error.message))?;

    serde_json::to_value(packet)
        .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string()))
}

fn run_multihop(
    request: &EnterpriseRequest,
    evidence_packet: &Value,
) -> Result<Value, EnterprisePipelineError> {
    let plan = build_hop_plan(&HopPlanInput {
        root_query: request.query.clone(),
        mode: HopMode::Web,
        requested_sub_queries: Vec::new(),
        max_hops: request.constraints.max_hops,
    })
    .map_err(|error| EnterprisePipelineError::new("policy_violation", error))?;

    let mut executor = EvidenceBackedHopExecutor::new(evidence_packet);
    let result = execute_hop_plan(
        &plan,
        HopBudget {
            policy_version: crate::web_search_plan::multihop::hop_budget::HOP_BUDGET_POLICY_VERSION,
            max_hops: request.constraints.max_hops,
            max_total_time_ms: request.constraints.max_total_time_ms,
            max_time_per_hop_ms: request.constraints.max_total_time_ms / (request.constraints.max_hops.max(1) as u64),
            max_provider_calls_total: request.constraints.max_provider_calls_total,
            max_url_opens_total: request.constraints.max_url_opens_total,
        },
        &mut executor,
    )
    .map_err(|error| EnterprisePipelineError::new("policy_violation", error))?;

    serde_json::to_value(result)
        .map_err(|error| EnterprisePipelineError::new("policy_violation", error.to_string()))
}

fn build_report_packet(
    request: &EnterpriseRequest,
    now_ms: i64,
    evidence_packet: &Value,
    competitive_packet: Option<&Value>,
    temporal_packet: Option<&Value>,
    risk_packet: Option<&Value>,
    merge_packet: Option<&Value>,
) -> Value {
    let fallback_citation = collect_fallback_citation(evidence_packet);
    let mut claims = Vec::new();

    if let Some(packet) = competitive_packet {
        let citations = read_string_array(packet.get("source_refs")).unwrap_or_else(|_| fallback_citation.clone());
        claims.push(json!({
            "text": "Competitive comparison output generated from evidence-bound inputs.",
            "citations": citations,
        }));
    }
    if let Some(packet) = temporal_packet {
        let citations = packet
            .pointer("/changes/0/citations_new")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<String>>())
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| fallback_citation.clone());
        claims.push(json!({
            "text": "Temporal change analysis output generated from evidence-bound inputs.",
            "citations": citations,
        }));
    }
    if let Some(packet) = risk_packet {
        let citations = read_string_array(packet.get("evidence_refs")).unwrap_or_else(|_| fallback_citation.clone());
        claims.push(json!({
            "text": "Risk scoring output generated from evidence-bound inputs.",
            "citations": citations,
        }));
    }
    if let Some(packet) = merge_packet {
        let citations = packet
            .pointer("/delta/changes_since_last_time/0/citations")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<String>>())
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| fallback_citation.clone());
        claims.push(json!({
            "text": "Merge delta output generated with external evidence supremacy.",
            "citations": citations,
        }));
    }

    if claims.is_empty() {
        claims.push(json!({
            "text": "Enterprise evidence packet is available for downstream grounded synthesis.",
            "citations": fallback_citation,
        }));
    }

    json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.ENTERPRISE",
        "trace_id": request.trace_id,
        "created_at_ms": request.created_at_ms.max(now_ms),
        "mode": request.mode.as_str(),
        "claims": claims,
    })
}

fn collect_fallback_citation(evidence_packet: &Value) -> Vec<String> {
    let mut citations = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|source| {
            source
                .get("url")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<String>>();
    citations.sort();
    citations.dedup();
    if citations.is_empty() {
        citations.push("unknown".to_string());
    }
    citations
}

fn read_string_array(raw: Option<&Value>) -> Result<Vec<String>, String> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let array = raw
        .as_array()
        .ok_or_else(|| "expected array of strings".to_string())?;
    let mut out = Vec::new();
    for entry in array {
        let value = entry
            .as_str()
            .ok_or_else(|| "expected string array entry".to_string())?;
        let value = value.trim();
        if !value.is_empty() {
            out.push(value.to_string());
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn target_entity_for_request(request: &EnterpriseRequest) -> Result<String, EnterprisePipelineError> {
    if let Some(target) = request.target_entity.as_ref() {
        return Ok(target.clone());
    }
    let guess = request
        .query
        .split_whitespace()
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string();
    if guess.is_empty() {
        Err(EnterprisePipelineError::new(
            "insufficient_evidence",
            "competitive mode requires target_entity",
        ))
    } else {
        Ok(guess)
    }
}

struct EvidenceBackedHopExecutor {
    source_urls: Vec<String>,
}

impl EvidenceBackedHopExecutor {
    fn new(evidence_packet: &Value) -> Self {
        let mut source_urls = evidence_packet
            .get("sources")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|source| {
                source
                    .get("url")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .collect::<Vec<String>>();
        source_urls.sort();
        source_urls.dedup();
        Self { source_urls }
    }
}

impl HopExecutor for EvidenceBackedHopExecutor {
    fn execute(
        &mut self,
        hop: &crate::web_search_plan::multihop::Hop,
    ) -> Result<HopExecutionOutput, crate::web_search_plan::multihop::HopExecutionError> {
        let endpoint = match hop.mode {
            HopMode::Web => "web",
            HopMode::News => "news",
            HopMode::Structured => "structured",
            HopMode::UrlFetch => "url_fetch",
        };
        Ok(HopExecutionOutput {
            provider_runs: vec![ProviderRunSummary {
                provider_id: "enterprise_evidence_replay".to_string(),
                endpoint: endpoint.to_string(),
                success: true,
            }],
            source_urls: self.source_urls.clone(),
            elapsed_ms: 1,
            provider_calls: 1,
            url_opens: 0,
            reason_code: None,
        })
    }
}
