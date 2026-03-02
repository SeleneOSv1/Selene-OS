#![forbid(unsafe_code)]

use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use selene_engines::ph1e::startup_outbound_self_check_logs;
use selene_adapter::{
    app_ui_assets, AdapterHealthResponse, AdapterRuntime, AdapterSyncHealth,
    InviteLinkOpenAdapterRequest, InviteLinkOpenAdapterResponse, OnboardingContinueAdapterRequest,
    OnboardingContinueAdapterResponse, UiChatTranscriptResponse, UiHealthChecksResponse,
    UiHealthDetailFilter, UiHealthDetailResponse, UiHealthReportQueryRequest,
    UiHealthReportQueryResponse, UiHealthSummary, UiHealthTimelinePaging, VoiceTurnAdapterRequest,
    VoiceTurnAdapterResponse,
};

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct UiHealthDetailQueryParams {
    issue_query: Option<String>,
    engine_owner: Option<String>,
    open_only: Option<bool>,
    critical_only: Option<bool>,
    escalated_only: Option<bool>,
    from_utc_ns: Option<u64>,
    to_utc_ns: Option<u64>,
    selected_issue_id: Option<String>,
    timeline_page_size: Option<u16>,
    timeline_cursor: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = env::var("SELENE_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse()?;
    let sync_worker_enabled = parse_sync_worker_enabled_from_env();
    let sync_worker_interval_ms = parse_sync_worker_interval_ms_from_env();

    for line in startup_outbound_self_check_logs() {
        eprintln!("{line}");
    }

    let runtime = Arc::new(Mutex::new(AdapterRuntime::default_from_env()?));
    if sync_worker_enabled {
        let runtime_for_worker = runtime.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(sync_worker_interval_ms));
            loop {
                ticker.tick().await;
                let pass_result = match runtime_for_worker.lock() {
                    Ok(runtime) => runtime.run_device_artifact_sync_worker_pass(None),
                    Err(_) => Err("adapter runtime lock poisoned".to_string()),
                };
                if let Err(err) = pass_result {
                    eprintln!("selene_adapter_http sync worker pass failed: {err}");
                }
            }
        });
    }
    let app = Router::new()
        .route("/", get(app_root))
        .route("/app", get(app_root))
        .route("/app/health", get(app_root))
        .route("/app.css", get(app_css))
        .route("/app.js", get(app_js))
        .route("/healthz", get(healthz))
        .route("/v1/ui/health/checks", get(ui_health_checks))
        .route("/v1/ui/health/detail/:check_id", get(ui_health_detail))
        .route("/v1/ui/health/report/query", post(ui_health_report_query))
        .route("/v1/ui/chat/transcript", get(ui_chat_transcript))
        .route("/v1/voice/turn", post(run_voice_turn))
        .route("/v1/invite/click", post(run_invite_click))
        .route("/v1/onboarding/continue", post(run_onboarding_continue))
        .with_state(runtime);

    println!(
        "selene_adapter_http listening on http://{addr} (sync_worker_enabled={sync_worker_enabled} interval_ms={sync_worker_interval_ms})"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
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

async fn app_root() -> Html<&'static str> {
    Html(app_ui_assets::APP_HTML)
}

async fn app_css() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        app_ui_assets::APP_CSS,
    )
}

async fn app_js() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        app_ui_assets::APP_JS,
    )
}

async fn healthz(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
) -> (StatusCode, Json<AdapterHealthResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(health_error_response(
                    "adapter runtime lock poisoned".to_string(),
                )),
            );
        }
    };
    match runtime.health_report(None) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(health_error_response(reason)),
        ),
    }
}

fn health_error_response(reason: String) -> AdapterHealthResponse {
    AdapterHealthResponse {
        status: "error".to_string(),
        outcome: "UNHEALTHY".to_string(),
        reason: Some(reason),
        sync: AdapterSyncHealth::default(),
    }
}

async fn ui_health_checks(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
) -> (StatusCode, Json<UiHealthChecksResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ui_health_checks_error_response(
                    "adapter runtime lock poisoned".to_string(),
                )),
            );
        }
    };
    match runtime.ui_health_checks_report(None) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ui_health_checks_error_response(reason)),
        ),
    }
}

fn ui_health_checks_error_response(reason: String) -> UiHealthChecksResponse {
    UiHealthChecksResponse {
        status: format!("error: {reason}"),
        generated_at_ns: 0,
        checks: Vec::new(),
    }
}

async fn ui_health_detail(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
    Path(check_id): Path<String>,
    Query(params): Query<UiHealthDetailQueryParams>,
) -> (StatusCode, Json<UiHealthDetailResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ui_health_detail_error_response(
                    check_id,
                    "adapter runtime lock poisoned".to_string(),
                )),
            );
        }
    };
    let filter = UiHealthDetailFilter {
        issue_query: params.issue_query,
        engine_owner: params.engine_owner,
        open_only: params.open_only.unwrap_or(false),
        critical_only: params.critical_only.unwrap_or(false),
        escalated_only: params.escalated_only.unwrap_or(false),
        from_utc_ns: params.from_utc_ns,
        to_utc_ns: params.to_utc_ns,
        selected_issue_id: params.selected_issue_id,
        timeline_page_size: params.timeline_page_size,
        timeline_cursor: params.timeline_cursor,
    };
    match runtime.ui_health_detail_report_filtered(&check_id, filter, None) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::BAD_REQUEST,
            Json(ui_health_detail_error_response(check_id, reason)),
        ),
    }
}

fn ui_health_detail_error_response(check_id: String, reason: String) -> UiHealthDetailResponse {
    UiHealthDetailResponse {
        status: format!("error: {reason}"),
        generated_at_ns: 0,
        selected_check_id: check_id,
        selected_check_label: "Unknown".to_string(),
        summary: UiHealthSummary {
            open_issues: 0,
            critical_open_count: 0,
            auto_resolved_24h_count: 0,
            escalated_24h_count: 0,
            mttr_ms: None,
        },
        issues: Vec::new(),
        active_issue_id: None,
        timeline: Vec::new(),
        timeline_paging: UiHealthTimelinePaging {
            has_next: false,
            next_cursor: None,
            total_entries: 0,
            visible_entries: 0,
        },
    }
}

async fn ui_chat_transcript(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
) -> (StatusCode, Json<UiChatTranscriptResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UiChatTranscriptResponse {
                    status: "error".to_string(),
                    generated_at_ns: 0,
                    note: Some("adapter runtime lock poisoned".to_string()),
                    messages: Vec::new(),
                }),
            );
        }
    };
    (
        StatusCode::OK,
        Json(runtime.ui_chat_transcript_report(None)),
    )
}

async fn ui_health_report_query(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
    Json(request): Json<UiHealthReportQueryRequest>,
) -> (StatusCode, Json<UiHealthReportQueryResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UiHealthReportQueryResponse {
                    status: "error".to_string(),
                    generated_at_ns: 0,
                    reason_code: "adapter_runtime_lock_poisoned".to_string(),
                    report_context_id: None,
                    report_revision: None,
                    normalized_query: None,
                    rows: Vec::new(),
                    paging: selene_adapter::UiHealthReportPaging {
                        has_next: false,
                        has_prev: false,
                        next_cursor: None,
                        prev_cursor: None,
                    },
                    display_target_applied: None,
                    remembered_display_target: None,
                    requires_clarification: Some("adapter runtime lock poisoned".to_string()),
                }),
            );
        }
    };
    let response = runtime.ui_health_report_query(request, None);
    let status = if response.status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    (status, Json(response))
}

async fn run_voice_turn(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
    Json(request): Json<VoiceTurnAdapterRequest>,
) -> (StatusCode, Json<VoiceTurnAdapterResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(VoiceTurnAdapterResponse {
                    status: "error".to_string(),
                    outcome: "REJECTED".to_string(),
                    reason: Some("adapter runtime lock poisoned".to_string()),
                    next_move: "respond".to_string(),
                    response_text: String::new(),
                    reason_code: "0".to_string(),
                    provenance: None,
                }),
            )
        }
    };
    match runtime.run_voice_turn(request) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::BAD_REQUEST,
            Json(VoiceTurnAdapterResponse {
                status: "error".to_string(),
                outcome: "REJECTED".to_string(),
                reason: Some(reason),
                next_move: "respond".to_string(),
                response_text: String::new(),
                reason_code: "0".to_string(),
                provenance: None,
            }),
        ),
    }
}

async fn run_invite_click(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
    Json(request): Json<InviteLinkOpenAdapterRequest>,
) -> (StatusCode, Json<InviteLinkOpenAdapterResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InviteLinkOpenAdapterResponse {
                    status: "error".to_string(),
                    outcome: "REJECTED".to_string(),
                    reason: Some("adapter runtime lock poisoned".to_string()),
                    onboarding_session_id: None,
                    next_step: None,
                    required_fields: Vec::new(),
                    required_verification_gates: Vec::new(),
                }),
            )
        }
    };
    match runtime.run_invite_link_open_and_start_onboarding(request) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::BAD_REQUEST,
            Json(InviteLinkOpenAdapterResponse {
                status: "error".to_string(),
                outcome: "REJECTED".to_string(),
                reason: Some(reason),
                onboarding_session_id: None,
                next_step: None,
                required_fields: Vec::new(),
                required_verification_gates: Vec::new(),
            }),
        ),
    }
}

async fn run_onboarding_continue(
    State(runtime): State<Arc<Mutex<AdapterRuntime>>>,
    Json(request): Json<OnboardingContinueAdapterRequest>,
) -> (StatusCode, Json<OnboardingContinueAdapterResponse>) {
    let runtime = match runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OnboardingContinueAdapterResponse {
                    status: "error".to_string(),
                    outcome: "REJECTED".to_string(),
                    reason: Some("adapter runtime lock poisoned".to_string()),
                    onboarding_session_id: None,
                    next_step: None,
                    blocking_field: None,
                    blocking_question: None,
                    remaining_missing_fields: Vec::new(),
                    remaining_platform_receipt_kinds: Vec::new(),
                    voice_artifact_sync_receipt_ref: None,
                    access_engine_instance_id: None,
                    onboarding_status: None,
                }),
            )
        }
    };
    match runtime.run_onboarding_continue(request) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(reason) => (
            StatusCode::BAD_REQUEST,
            Json(OnboardingContinueAdapterResponse {
                status: "error".to_string(),
                outcome: "REJECTED".to_string(),
                reason: Some(reason),
                onboarding_session_id: None,
                next_step: None,
                blocking_field: None,
                blocking_question: None,
                remaining_missing_fields: Vec::new(),
                remaining_platform_receipt_kinds: Vec::new(),
                voice_artifact_sync_receipt_ref: None,
                access_engine_instance_id: None,
                onboarding_status: None,
            }),
        ),
    }
}
