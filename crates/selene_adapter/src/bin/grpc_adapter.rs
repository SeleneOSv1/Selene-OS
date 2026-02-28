#![forbid(unsafe_code)]

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::grpc_api::{
    voice_ingress_server::{VoiceIngress, VoiceIngressServer},
    RunVoiceTurnProvenance, RunVoiceTurnRequest, RunVoiceTurnResponse, RunVoiceTurnSourceRef,
    UiHealthReportPaging, UiHealthReportQueryRequest, UiHealthReportQueryResponse,
    UiHealthReportRow,
};
use selene_adapter::{
    AdapterRuntime, UiHealthReportQueryRequest as AdapterUiHealthReportQueryRequest,
    UiHealthReportQueryResponse as AdapterUiHealthReportQueryResponse, VoiceTurnAdapterRequest,
    VoiceTurnThreadPolicyFlags as AdapterVoiceTurnThreadPolicyFlags,
};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct GrpcVoiceIngress {
    runtime: Arc<Mutex<AdapterRuntime>>,
}

#[tonic::async_trait]
impl VoiceIngress for GrpcVoiceIngress {
    async fn run_voice_turn(
        &self,
        request: Request<RunVoiceTurnRequest>,
    ) -> Result<Response<RunVoiceTurnResponse>, Status> {
        let req = request.into_inner();
        let adapter_request = VoiceTurnAdapterRequest {
            correlation_id: req.correlation_id,
            turn_id: req.turn_id,
            app_platform: req.app_platform,
            trigger: req.trigger,
            actor_user_id: req.actor_user_id,
            tenant_id: if req.tenant_id.trim().is_empty() {
                None
            } else {
                Some(req.tenant_id)
            },
            device_id: if req.device_id.trim().is_empty() {
                None
            } else {
                Some(req.device_id)
            },
            now_ns: if req.now_ns == 0 {
                None
            } else {
                Some(req.now_ns)
            },
            thread_key: if req.thread_key.trim().is_empty() {
                None
            } else {
                Some(req.thread_key)
            },
            project_id: if req.project_id.trim().is_empty() {
                None
            } else {
                Some(req.project_id)
            },
            pinned_context_refs: if req.pinned_context_refs.is_empty() {
                None
            } else {
                Some(req.pinned_context_refs)
            },
            thread_policy_flags: req.thread_policy_flags.map(|flags| {
                AdapterVoiceTurnThreadPolicyFlags {
                    privacy_mode: flags.privacy_mode,
                    do_not_disturb: flags.do_not_disturb,
                    strict_safety: flags.strict_safety,
                }
            }),
            user_text_partial: None,
            user_text_final: None,
            selene_text_partial: None,
            selene_text_final: None,
            audio_capture_ref: None,
            visual_input_ref: None,
        };

        let runtime = self
            .runtime
            .lock()
            .map_err(|_| Status::internal("adapter runtime lock poisoned"))?;
        match runtime.run_voice_turn(adapter_request) {
            Ok(out) => Ok(Response::new(RunVoiceTurnResponse {
                status: out.status,
                outcome: out.outcome,
                reason: out.reason.unwrap_or_default(),
                next_move: out.next_move,
                response_text: out.response_text,
                reason_code: out.reason_code,
                provenance: out.provenance.map(|p| RunVoiceTurnProvenance {
                    sources: p
                        .sources
                        .into_iter()
                        .map(|s| RunVoiceTurnSourceRef {
                            title: s.title,
                            url: s.url,
                        })
                        .collect(),
                    retrieved_at: p.retrieved_at,
                    cache_status: p.cache_status,
                }),
            })),
            Err(reason) => Err(Status::invalid_argument(reason)),
        }
    }

    async fn ui_health_report_query(
        &self,
        request: Request<UiHealthReportQueryRequest>,
    ) -> Result<Response<UiHealthReportQueryResponse>, Status> {
        let req = request.into_inner();
        let company_ids = if req.company_ids.is_empty() {
            None
        } else {
            Some(req.company_ids)
        };
        let country_codes = if req.country_codes.is_empty() {
            None
        } else {
            Some(req.country_codes)
        };
        let adapter_request = AdapterUiHealthReportQueryRequest {
            correlation_id: non_zero_u64(req.correlation_id),
            turn_id: non_zero_u64(req.turn_id),
            tenant_id: optional_string(req.tenant_id),
            viewer_user_id: optional_string(req.viewer_user_id),
            report_kind: optional_string(req.report_kind),
            from_utc_ns: non_zero_u64(req.from_utc_ns),
            to_utc_ns: non_zero_u64(req.to_utc_ns),
            engine_owner_filter: optional_string(req.engine_owner_filter),
            company_scope: optional_string(req.company_scope),
            company_ids,
            country_codes,
            escalated_only: Some(req.escalated_only),
            unresolved_only: Some(req.unresolved_only),
            display_target: optional_string(req.display_target),
            page_action: optional_string(req.page_action),
            page_cursor: optional_string(req.page_cursor),
            report_context_id: optional_string(req.report_context_id),
            page_size: if req.page_size == 0 {
                None
            } else {
                Some(req.page_size.min(u16::MAX as u32) as u16)
            },
        };

        let runtime = self
            .runtime
            .lock()
            .map_err(|_| Status::internal("adapter runtime lock poisoned"))?;
        let out = runtime.ui_health_report_query(adapter_request, None);
        Ok(Response::new(map_ui_health_report_query_response(out)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = env::var("SELENE_GRPC_BIND").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    let addr = bind.parse()?;
    let sync_worker_enabled = parse_sync_worker_enabled_from_env();
    let sync_worker_interval_ms = parse_sync_worker_interval_ms_from_env();
    let service = GrpcVoiceIngress {
        runtime: Arc::new(Mutex::new(AdapterRuntime::default_from_env()?)),
    };
    if sync_worker_enabled {
        let runtime_for_worker = service.runtime.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(sync_worker_interval_ms));
            loop {
                ticker.tick().await;
                let pass_result = match runtime_for_worker.lock() {
                    Ok(runtime) => runtime.run_device_artifact_sync_worker_pass(None),
                    Err(_) => Err("adapter runtime lock poisoned".to_string()),
                };
                if let Err(err) = pass_result {
                    eprintln!("selene_adapter_grpc sync worker pass failed: {err}");
                }
            }
        });
    }

    println!(
        "selene_adapter_grpc listening on {addr} (sync_worker_enabled={sync_worker_enabled} interval_ms={sync_worker_interval_ms})"
    );
    Server::builder()
        .add_service(VoiceIngressServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

fn parse_sync_worker_enabled_from_env() -> bool {
    match env::var("SELENE_ADAPTER_SYNC_WORKER_ENABLED") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

fn parse_sync_worker_interval_ms_from_env() -> u64 {
    env::var("SELENE_ADAPTER_SYNC_WORKER_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| (100..=60_000).contains(v))
        .unwrap_or(1_000)
}

fn optional_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn non_zero_u64(value: u64) -> Option<u64> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

fn map_ui_health_report_query_response(
    response: AdapterUiHealthReportQueryResponse,
) -> UiHealthReportQueryResponse {
    UiHealthReportQueryResponse {
        status: response.status,
        generated_at_ns: response.generated_at_ns,
        reason_code: response.reason_code,
        report_context_id: response.report_context_id.unwrap_or_default(),
        report_revision: response.report_revision.unwrap_or_default(),
        normalized_query: response.normalized_query.unwrap_or_default(),
        rows: response
            .rows
            .into_iter()
            .map(|row| UiHealthReportRow {
                tenant_id: row.tenant_id,
                issue_id: row.issue_id,
                owner_engine_id: row.owner_engine_id,
                severity: row.severity,
                status: row.status,
                latest_reason_code: row.latest_reason_code,
                last_seen_at_ns: row.last_seen_at_ns,
                bcast_id: row.bcast_id.unwrap_or_default(),
                ack_state: row.ack_state.unwrap_or_default(),
                issue_fingerprint: row.issue_fingerprint.unwrap_or_default(),
                recurrence_observed: row.recurrence_observed,
                impact_summary: row.impact_summary.unwrap_or_default(),
                attempted_fix_actions: row.attempted_fix_actions,
                current_monitoring_evidence: row.current_monitoring_evidence.unwrap_or_default(),
                unresolved_reason_exact: row.unresolved_reason_exact.unwrap_or_default(),
            })
            .collect(),
        paging: Some(UiHealthReportPaging {
            has_next: response.paging.has_next,
            has_prev: response.paging.has_prev,
            next_cursor: response.paging.next_cursor.unwrap_or_default(),
            prev_cursor: response.paging.prev_cursor.unwrap_or_default(),
        }),
        display_target_applied: response.display_target_applied.unwrap_or_default(),
        remembered_display_target: response.remembered_display_target.unwrap_or_default(),
        requires_clarification: response.requires_clarification.unwrap_or_default(),
    }
}
