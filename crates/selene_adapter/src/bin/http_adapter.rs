#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    env,
    net::SocketAddr,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use selene_adapter::{
    app_ui_assets, build_runtime_execution_envelope_for_voice_turn_request, AdapterHealthResponse,
    AdapterRuntime, AdapterSyncHealth, InviteLinkOpenAdapterRequest, InviteLinkOpenAdapterResponse,
    OnboardingContinueAdapterRequest, OnboardingContinueAdapterResponse,
    SessionAttachAdapterRequest, SessionAttachAdapterResponse,
    SessionPostureEvidenceAdapterRequest, SessionPostureEvidenceAdapterResponse,
    SessionRecentListAdapterRequest, SessionRecentListAdapterResponse,
    SessionRecoverAdapterRequest, SessionRecoverAdapterResponse, SessionResumeAdapterRequest,
    SessionResumeAdapterResponse, UiChatTranscriptResponse, UiHealthChecksResponse,
    UiHealthDetailFilter, UiHealthDetailResponse, UiHealthReportQueryRequest,
    UiHealthReportQueryResponse, UiHealthSummary, UiHealthTimelinePaging, VoiceTurnAdapterRequest,
    VoiceTurnAdapterResponse, VoiceTurnIngressError, WakeProfileAvailabilityRefreshAdapterRequest,
    WakeProfileAvailabilityRefreshAdapterResponse,
};
use selene_engines::device_vault;
use selene_engines::ph1e::startup_outbound_self_check_logs;
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use selene_kernel_contracts::runtime_execution::{FailureClass, RuntimeExecutionEnvelope};
use sha2::{Digest, Sha256};

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

#[derive(Clone)]
struct HttpAdapterState {
    runtime: Arc<Mutex<AdapterRuntime>>,
    ingress_security: Arc<Mutex<IngressSecurityState>>,
    ingress_security_config: IngressSecurityConfig,
    tts_request_limiter: Arc<Mutex<TtsRequestLimiterState>>,
}

#[derive(Debug, serde::Deserialize)]
struct DesktopRealtimeTranscriptionSessionRequest {
    correlation_id: u64,
    actor_user_id: String,
    device_id: String,
    requested_model: Option<String>,
    feature_flag_name: String,
    feature_flag_enabled: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DesktopRealtimeTranscriptionSessionResponse {
    status: String,
    outcome: String,
    reason: Option<String>,
    session_id: Option<String>,
    client_secret: Option<String>,
    expires_at: Option<u64>,
    websocket_url: String,
    transcription_model: String,
    input_audio_format: String,
    max_session_duration_ms: u64,
    max_silence_duration_ms: u64,
    max_reconnect_attempts: u8,
    retry_count: u8,
}

#[derive(Debug, serde::Deserialize)]
struct DesktopOpenAiTtsSpeechRequest {
    correlation_id: u64,
    actor_user_id: String,
    device_id: String,
    response_text: String,
    session_id: Option<String>,
    turn_id: Option<String>,
    request_id: Option<String>,
    feature_flag_name: String,
    feature_flag_enabled: bool,
    voice: Option<String>,
    format: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DesktopOpenAiTtsSpeechResponse {
    status: String,
    outcome: String,
    ok: bool,
    safe_failure_reason: Option<String>,
    audio_base64: Option<String>,
    audio_mime_type: Option<String>,
    audio_byte_len: u64,
    audio_sha256: Option<String>,
    model: String,
    voice: String,
    answer_text_sha256: Option<String>,
    session_id: Option<String>,
    turn_id: Option<String>,
    request_id: Option<String>,
    fallback_allowed: bool,
    ai_voice_disclosure: String,
}

#[derive(Debug, Clone, Copy)]
struct IngressSecurityConfig {
    max_stale_ms: u64,
    max_future_ms: u64,
    replay_ttl_ms: u64,
    quota_enabled: bool,
    quota_window_ms: u64,
    quota_per_token: u32,
    quota_per_device: u32,
}

impl IngressSecurityConfig {
    fn from_env() -> Self {
        Self {
            max_stale_ms: parse_u64_env("SELENE_INGRESS_MAX_STALE_MS", 300_000, 1_000, 86_400_000),
            max_future_ms: parse_u64_env("SELENE_INGRESS_MAX_FUTURE_MS", 30_000, 1_000, 86_400_000),
            replay_ttl_ms: parse_u64_env(
                "SELENE_INGRESS_REPLAY_TTL_MS",
                600_000,
                5_000,
                86_400_000,
            ),
            quota_enabled: parse_bool_env("SELENE_INGRESS_QUOTA_ENABLED", true),
            quota_window_ms: parse_u64_env(
                "SELENE_INGRESS_QUOTA_WINDOW_MS",
                60_000,
                1_000,
                86_400_000,
            ),
            quota_per_token: parse_u32_env("SELENE_INGRESS_QUOTA_PER_TOKEN", 120, 1, 1_000_000),
            quota_per_device: parse_u32_env("SELENE_INGRESS_QUOTA_PER_DEVICE", 120, 1, 1_000_000),
        }
    }
}

#[derive(Debug, Default)]
struct IngressSecurityState {
    replay_cache: BTreeMap<String, u64>,
    token_quota: BTreeMap<String, QuotaWindowCounter>,
    device_quota: BTreeMap<String, QuotaWindowCounter>,
}

#[derive(Debug, Default)]
struct TtsRequestLimiterState {
    per_turn: BTreeMap<String, u64>,
    per_session: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Copy, Default)]
struct QuotaWindowCounter {
    window_start_ms: u64,
    count: u32,
}

#[derive(Debug, Clone)]
struct ParsedBearerToken {
    token_id: String,
    subject: String,
    device: String,
}

#[derive(Debug, Clone)]
struct EndpointSecurityInput {
    endpoint: &'static str,
    expected_subject: String,
    expected_device: String,
    request_id: String,
    idempotency_key: String,
    timestamp_ms: u64,
    nonce: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecurityRejectKind {
    Unauthorized,
    Forbidden,
    Conflict,
    Unprocessable,
    TooManyRequests,
}

#[derive(Debug, Clone)]
struct SecurityReject {
    kind: SecurityRejectKind,
    reason: String,
    retry_after_secs: Option<u64>,
}

impl SecurityReject {
    fn unauthorized(reason: impl Into<String>) -> Self {
        Self {
            kind: SecurityRejectKind::Unauthorized,
            reason: reason.into(),
            retry_after_secs: None,
        }
    }

    fn forbidden(reason: impl Into<String>) -> Self {
        Self {
            kind: SecurityRejectKind::Forbidden,
            reason: reason.into(),
            retry_after_secs: None,
        }
    }

    fn conflict(reason: impl Into<String>) -> Self {
        Self {
            kind: SecurityRejectKind::Conflict,
            reason: reason.into(),
            retry_after_secs: None,
        }
    }

    fn unprocessable(reason: impl Into<String>) -> Self {
        Self {
            kind: SecurityRejectKind::Unprocessable,
            reason: reason.into(),
            retry_after_secs: None,
        }
    }

    fn too_many_requests(reason: impl Into<String>, retry_after_secs: u64) -> Self {
        Self {
            kind: SecurityRejectKind::TooManyRequests,
            reason: reason.into(),
            retry_after_secs: Some(retry_after_secs.max(1)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = env::var("SELENE_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse()?;
    let sync_worker_enabled = parse_sync_worker_enabled_from_env();
    let sync_worker_interval_ms = parse_sync_worker_interval_ms_from_env();

    let runtime = Arc::new(Mutex::new(AdapterRuntime::default_from_env()?));
    let state = HttpAdapterState {
        runtime: runtime.clone(),
        ingress_security: Arc::new(Mutex::new(IngressSecurityState::default())),
        ingress_security_config: IngressSecurityConfig::from_env(),
        tts_request_limiter: Arc::new(Mutex::new(TtsRequestLimiterState::default())),
    };
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
        .route(
            "/v1/desktop/realtime-transcription/session",
            post(run_desktop_realtime_transcription_session),
        )
        .route(
            "/v1/desktop/openai-tts/speech",
            post(run_desktop_openai_tts_speech),
        )
        .route("/v1/invite/click", post(run_invite_click))
        .route("/v1/onboarding/continue", post(run_onboarding_continue))
        .route("/v1/session/attach", post(run_session_attach))
        .route("/v1/session/resume", post(run_session_resume))
        .route("/v1/session/recover", post(run_session_recover))
        .route("/v1/session/recent", post(run_session_recent_list))
        .route("/v1/session/posture", post(run_session_posture_evidence))
        .route(
            "/v1/wake-profile/availability",
            post(run_wake_profile_availability_refresh),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tokio::task::spawn_blocking(|| {
        for line in startup_outbound_self_check_logs() {
            eprintln!("{line}");
        }
    });
    println!(
        "selene_adapter_http listening on http://{addr} (sync_worker_enabled={sync_worker_enabled} interval_ms={sync_worker_interval_ms})"
    );
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

fn parse_bool_env(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => default,
    }
}

fn parse_u64_env(key: &str, default: u64, min: u64, max: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| (min..=max).contains(value))
        .unwrap_or(default)
}

fn parse_u32_env(key: &str, default: u32, min: u32, max: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .filter(|value| (min..=max).contains(value))
        .unwrap_or(default)
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
    State(state): State<HttpAdapterState>,
) -> (StatusCode, Json<AdapterHealthResponse>) {
    let runtime = match state.runtime.lock() {
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
    State(state): State<HttpAdapterState>,
) -> (StatusCode, Json<UiHealthChecksResponse>) {
    let runtime = match state.runtime.lock() {
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
    State(state): State<HttpAdapterState>,
    Path(check_id): Path<String>,
    Query(params): Query<UiHealthDetailQueryParams>,
) -> (StatusCode, Json<UiHealthDetailResponse>) {
    let runtime = match state.runtime.lock() {
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
    State(state): State<HttpAdapterState>,
) -> (StatusCode, Json<UiChatTranscriptResponse>) {
    let runtime = match state.runtime.lock() {
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
    State(state): State<HttpAdapterState>,
    Json(request): Json<UiHealthReportQueryRequest>,
) -> (StatusCode, Json<UiHealthReportQueryResponse>) {
    let runtime = match state.runtime.lock() {
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
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<VoiceTurnAdapterRequest>,
) -> Response {
    let Some(device_id) = request.device_id.clone() else {
        return voice_turn_ingress_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            VoiceTurnIngressError {
                failure_class: FailureClass::InvalidPayload,
                reason_code: "MISSING_DEVICE_ID".to_string(),
                reason: Some("missing_device_id".to_string()),
                session_id: None,
                turn_id: Some(request.turn_id),
                session_state: None,
            },
        );
    };
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return voice_turn_security_reject_response(reject, Some(request.turn_id)),
    };
    let idempotency_key =
        match required_header_token(&headers, "idempotency-key", "missing_idempotency_key") {
            Ok(v) => v,
            Err(reject) => {
                return voice_turn_security_reject_response(reject, Some(request.turn_id))
            }
        };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return voice_turn_security_reject_response(reject, Some(request.turn_id)),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return voice_turn_security_reject_response(reject, Some(request.turn_id)),
    };
    let runtime_execution_envelope = match runtime_execution_envelope_from_voice_turn_request(
        &request,
        &request_id,
        &idempotency_key,
        &device_id,
    ) {
        Ok(envelope) => envelope,
        Err(err) => {
            return voice_turn_ingress_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                VoiceTurnIngressError {
                    failure_class: FailureClass::InvalidPayload,
                    reason_code: "INVALID_RUNTIME_EXECUTION_ENVELOPE".to_string(),
                    reason: Some(err),
                    session_id: None,
                    turn_id: Some(request.turn_id),
                    session_state: None,
                },
            )
        }
    };
    let request_id_for_security = runtime_execution_envelope.request_id.clone();
    let idempotency_key_for_security = runtime_execution_envelope.idempotency_key.clone();
    let nonce_for_security = nonce.clone();
    let timestamp_ms_for_security = timestamp_ms;
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/voice/turn",
        expected_subject: request.actor_user_id.clone(),
        expected_device: device_id,
        request_id: request_id_for_security,
        idempotency_key: idempotency_key_for_security,
        timestamp_ms: timestamp_ms_for_security,
        nonce: nonce_for_security,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return voice_turn_security_reject_response(reject, Some(request.turn_id));
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return voice_turn_ingress_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                VoiceTurnIngressError {
                    failure_class: FailureClass::RetryableRuntime,
                    reason_code: "ADAPTER_RUNTIME_LOCK_POISONED".to_string(),
                    reason: Some("adapter runtime lock poisoned".to_string()),
                    session_id: None,
                    turn_id: Some(request.turn_id),
                    session_state: None,
                },
            )
        }
    };
    match runtime
        .run_voice_turn_ingress_with_execution_envelope(request, runtime_execution_envelope)
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            let status = status_for_failure_class(error.failure_class);
            voice_turn_ingress_error_response(status, error)
        }
    }
}

async fn run_desktop_realtime_transcription_session(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<DesktopRealtimeTranscriptionSessionRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return desktop_realtime_transcription_security_reject_response(reject),
    };
    let idempotency_key =
        match required_header_token(&headers, "idempotency-key", "missing_idempotency_key") {
            Ok(v) => v,
            Err(reject) => return desktop_realtime_transcription_security_reject_response(reject),
        };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return desktop_realtime_transcription_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return desktop_realtime_transcription_security_reject_response(reject),
    };
    if request.correlation_id == 0
        || request.actor_user_id.trim().is_empty()
        || request.device_id.trim().is_empty()
    {
        return desktop_realtime_transcription_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_realtime_transcription_identity",
        );
    }
    if request.feature_flag_name.trim() != "SELENE_DESKTOP_OPENAI_REALTIME_TRANSCRIPTION_ENABLED"
        || !request.feature_flag_enabled
    {
        return desktop_realtime_transcription_error_response(
            StatusCode::PRECONDITION_FAILED,
            "desktop_realtime_transcription_feature_flag_disabled",
        );
    }

    let security_input = EndpointSecurityInput {
        endpoint: "/v1/desktop/realtime-transcription/session",
        expected_subject: request.actor_user_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key,
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return desktop_realtime_transcription_security_reject_response(reject);
    }

    let model = bounded_realtime_transcription_model(request.requested_model.as_deref());
    let websocket_url = env::var("OPENAI_REALTIME_TRANSCRIPTION_WS_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "wss://api.openai.com/v1/realtime?intent=transcription".to_string());
    let max_session_duration_ms = parse_u64_env(
        "SELENE_DESKTOP_REALTIME_TRANSCRIPTION_MAX_SESSION_MS",
        90_000,
        5_000,
        300_000,
    );
    let max_silence_duration_ms = parse_u64_env(
        "SELENE_DESKTOP_REALTIME_TRANSCRIPTION_MAX_SILENCE_MS",
        1_200,
        300,
        10_000,
    );
    let max_reconnect_attempts = parse_u32_env(
        "SELENE_DESKTOP_REALTIME_TRANSCRIPTION_MAX_RECONNECTS",
        1,
        0,
        5,
    ) as u8;

    match mint_openai_realtime_transcription_session(&model, max_silence_duration_ms) {
        Ok(minted) => {
            let response = DesktopRealtimeTranscriptionSessionResponse {
                status: "ok".to_string(),
                outcome: "REALTIME_TRANSCRIPTION_SESSION_CREATED".to_string(),
                reason: None,
                session_id: minted.session_id,
                client_secret: Some(minted.client_secret),
                expires_at: minted.expires_at,
                websocket_url,
                transcription_model: model,
                input_audio_format: "pcm16_24000_mono".to_string(),
                max_session_duration_ms,
                max_silence_duration_ms,
                max_reconnect_attempts,
                retry_count: 0,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(reason) => desktop_realtime_transcription_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            reason.as_str(),
        ),
    }
}

async fn run_desktop_openai_tts_speech(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<DesktopOpenAiTtsSpeechRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return desktop_openai_tts_security_reject_response(reject, &request),
    };
    let idempotency_key =
        match required_header_token(&headers, "idempotency-key", "missing_idempotency_key") {
            Ok(v) => v,
            Err(reject) => return desktop_openai_tts_security_reject_response(reject, &request),
        };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return desktop_openai_tts_security_reject_response(reject, &request),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return desktop_openai_tts_security_reject_response(reject, &request),
    };
    if request.correlation_id == 0
        || request.actor_user_id.trim().is_empty()
        || request.device_id.trim().is_empty()
    {
        return desktop_openai_tts_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            &request,
            "invalid_openai_tts_identity",
        );
    }
    if request.feature_flag_name.trim() != "SELENE_DESKTOP_OPENAI_TTS_ENABLED"
        || !request.feature_flag_enabled
    {
        return desktop_openai_tts_error_response(
            StatusCode::PRECONDITION_FAILED,
            &request,
            "desktop_openai_tts_feature_flag_disabled",
        );
    }

    let security_input = EndpointSecurityInput {
        endpoint: "/v1/desktop/openai-tts/speech",
        expected_subject: request.actor_user_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key,
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return desktop_openai_tts_security_reject_response(reject, &request);
    }

    let bounded_text = match bounded_openai_tts_input_text(&request.response_text) {
        Ok(text) => text,
        Err(reason) => {
            return desktop_openai_tts_error_response(StatusCode::BAD_REQUEST, &request, &reason)
        }
    };
    if let Err(reason) = speakable_openai_tts_text(&bounded_text) {
        return desktop_openai_tts_error_response(StatusCode::FORBIDDEN, &request, &reason);
    }
    if let Err(reason) = enforce_openai_tts_request_bounds(&state.tts_request_limiter, &request) {
        return desktop_openai_tts_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            &request,
            reason.as_str(),
        );
    }

    let model = bounded_openai_tts_model(None);
    let voice = bounded_openai_tts_voice(request.voice.as_deref());
    let format = bounded_openai_tts_format(request.format.as_deref());
    let answer_text_sha256 = sha256_hex(bounded_text.as_bytes());
    match request_openai_tts_speech(&bounded_text, &model, &voice, &format) {
        Ok(audio) => {
            let audio_byte_len = audio.len() as u64;
            let max_audio_bytes = parse_u64_env(
                "SELENE_DESKTOP_OPENAI_TTS_MAX_AUDIO_BYTES",
                4_000_000,
                8_192,
                20_000_000,
            );
            if audio_byte_len > max_audio_bytes {
                return desktop_openai_tts_error_response(
                    StatusCode::BAD_GATEWAY,
                    &request,
                    "openai_tts_audio_response_too_large",
                );
            }
            let audio_sha256 = sha256_hex(&audio);
            let response = DesktopOpenAiTtsSpeechResponse {
                status: "ok".to_string(),
                outcome: "OPENAI_TTS_SPEECH_CREATED".to_string(),
                ok: true,
                safe_failure_reason: None,
                audio_base64: Some(BASE64_STANDARD.encode(audio)),
                audio_mime_type: Some(audio_mime_type_for_openai_tts_format(&format).to_string()),
                audio_byte_len,
                audio_sha256: Some(audio_sha256),
                model,
                voice,
                answer_text_sha256: Some(answer_text_sha256),
                session_id: request.session_id,
                turn_id: request.turn_id,
                request_id: request.request_id,
                fallback_allowed: false,
                ai_voice_disclosure: "synthetic_ai_voice".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(reason) => desktop_openai_tts_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            &request,
            reason.as_str(),
        ),
    }
}

async fn run_invite_click(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<InviteLinkOpenAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return invite_click_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return invite_click_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return invite_click_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/invite/click",
        expected_subject: request.token_id.clone(),
        expected_device: request.app_instance_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return invite_click_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return invite_click_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_invite_link_open_and_start_onboarding(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => invite_click_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_onboarding_continue(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<OnboardingContinueAdapterRequest>,
) -> Response {
    let Some(device_id) = request.device_id.clone() else {
        return onboarding_continue_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing_device_id".to_string(),
        );
    };
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return onboarding_continue_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return onboarding_continue_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return onboarding_continue_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/onboarding/continue",
        expected_subject: request.onboarding_session_id.clone(),
        expected_device: device_id,
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return onboarding_continue_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return onboarding_continue_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_onboarding_continue(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => onboarding_continue_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_session_attach(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<SessionAttachAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return session_attach_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return session_attach_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return session_attach_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/session/attach",
        expected_subject: request.session_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return session_attach_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return session_attach_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_session_attach(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => session_attach_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_session_resume(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<SessionResumeAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return session_resume_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return session_resume_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return session_resume_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/session/resume",
        expected_subject: request.session_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return session_resume_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return session_resume_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_session_resume(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => session_resume_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_session_recover(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<SessionRecoverAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return session_recover_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return session_recover_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return session_recover_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/session/recover",
        expected_subject: request.session_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return session_recover_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return session_recover_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_session_recover(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => session_recover_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_session_recent_list(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<SessionRecentListAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return session_recent_list_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return session_recent_list_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return session_recent_list_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/session/recent",
        expected_subject: request.device_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return session_recent_list_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return session_recent_list_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_session_recent_list(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => session_recent_list_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_session_posture_evidence(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<SessionPostureEvidenceAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => return session_posture_evidence_security_reject_response(reject),
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => return session_posture_evidence_security_reject_response(reject),
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => return session_posture_evidence_security_reject_response(reject),
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/session/posture",
        expected_subject: request.session_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return session_posture_evidence_security_reject_response(reject);
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return session_posture_evidence_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    match runtime.run_session_posture_evidence(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => session_posture_evidence_error_response(StatusCode::BAD_REQUEST, reason),
    }
}

async fn run_wake_profile_availability_refresh(
    State(state): State<HttpAdapterState>,
    headers: HeaderMap,
    Json(request): Json<WakeProfileAvailabilityRefreshAdapterRequest>,
) -> Response {
    let request_id = match required_header_token(&headers, "x-request-id", "missing_request_id") {
        Ok(v) => v,
        Err(reject) => {
            return wake_profile_availability_security_reject_response(
                reject,
                Some(request.device_id.clone()),
                Some(request.expected_wake_profile_id.clone()),
            )
        }
    };
    let timestamp_ms = match required_header_u64(
        &headers,
        "x-selene-timestamp-ms",
        "missing_timestamp_ms",
        "invalid_timestamp_ms",
    ) {
        Ok(v) => v,
        Err(reject) => {
            return wake_profile_availability_security_reject_response(
                reject,
                Some(request.device_id.clone()),
                Some(request.expected_wake_profile_id.clone()),
            )
        }
    };
    let nonce = match required_header_token(&headers, "x-selene-nonce", "missing_nonce") {
        Ok(v) => v,
        Err(reject) => {
            return wake_profile_availability_security_reject_response(
                reject,
                Some(request.device_id.clone()),
                Some(request.expected_wake_profile_id.clone()),
            )
        }
    };
    let security_input = EndpointSecurityInput {
        endpoint: "/v1/wake-profile/availability",
        expected_subject: request.expected_wake_profile_id.clone(),
        expected_device: request.device_id.clone(),
        request_id,
        idempotency_key: request.idempotency_key.clone(),
        timestamp_ms,
        nonce,
    };
    if let Err(reject) = enforce_ingress_security(
        &headers,
        &state.ingress_security,
        state.ingress_security_config,
        security_input,
    ) {
        return wake_profile_availability_security_reject_response(
            reject,
            Some(request.device_id.clone()),
            Some(request.expected_wake_profile_id.clone()),
        );
    }

    let runtime = match state.runtime.lock() {
        Ok(runtime) => runtime,
        Err(_) => {
            return wake_profile_availability_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(request.device_id.clone()),
                Some(request.expected_wake_profile_id.clone()),
                "adapter runtime lock poisoned".to_string(),
            )
        }
    };
    let request_device_id = request.device_id.clone();
    let request_wake_profile_id = request.expected_wake_profile_id.clone();
    match runtime.run_wake_profile_availability_refresh(request) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(reason) => wake_profile_availability_error_response(
            StatusCode::BAD_REQUEST,
            Some(request_device_id),
            Some(request_wake_profile_id),
            reason,
        ),
    }
}

fn required_header_token(
    headers: &HeaderMap,
    key: &str,
    missing_reason: &'static str,
) -> Result<String, SecurityReject> {
    let value = headers
        .get(key)
        .ok_or_else(|| SecurityReject::unprocessable(missing_reason))?;
    let raw = value
        .to_str()
        .map_err(|_| SecurityReject::unprocessable(format!("invalid_{key}")))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || !trimmed.is_ascii() {
        return Err(SecurityReject::unprocessable(format!("invalid_{key}")));
    }
    Ok(trimmed.to_string())
}

fn required_header_u64(
    headers: &HeaderMap,
    key: &str,
    missing_reason: &'static str,
    invalid_reason: &'static str,
) -> Result<u64, SecurityReject> {
    let value = headers
        .get(key)
        .ok_or_else(|| SecurityReject::unprocessable(missing_reason))?;
    let raw = value
        .to_str()
        .map_err(|_| SecurityReject::unprocessable(invalid_reason))?;
    raw.trim()
        .parse::<u64>()
        .map_err(|_| SecurityReject::unprocessable(invalid_reason))
}

fn enforce_ingress_security(
    headers: &HeaderMap,
    ingress_security: &Arc<Mutex<IngressSecurityState>>,
    config: IngressSecurityConfig,
    input: EndpointSecurityInput,
) -> Result<(), SecurityReject> {
    let auth = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| SecurityReject::unauthorized("missing_bearer_auth"))?
        .to_str()
        .map_err(|_| SecurityReject::unauthorized("invalid_bearer_auth"))?;
    let token = parse_and_verify_bearer(auth)?;
    if token.subject != input.expected_subject {
        return Err(SecurityReject::forbidden("token_subject_mismatch"));
    }
    if token.device != input.expected_device {
        return Err(SecurityReject::forbidden("token_device_mismatch"));
    }
    let now_ms = system_time_now_ms();
    if input.timestamp_ms == 0 {
        return Err(SecurityReject::unprocessable("invalid_timestamp_ms"));
    }
    if input.timestamp_ms.saturating_add(config.max_stale_ms) < now_ms {
        return Err(SecurityReject::unprocessable("stale_request"));
    }
    if input.timestamp_ms > now_ms.saturating_add(config.max_future_ms) {
        return Err(SecurityReject::unprocessable("timestamp_in_future"));
    }

    let replay_key = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        input.endpoint,
        token.token_id,
        token.subject,
        token.device,
        input.request_id,
        input.idempotency_key,
        input.nonce
    );
    let mut security = ingress_security
        .lock()
        .map_err(|_| SecurityReject::unprocessable("ingress_security_lock_poisoned"))?;
    security
        .replay_cache
        .retain(|_, expires_at_ms| *expires_at_ms > now_ms);
    if config.quota_enabled {
        enforce_quota_counter(
            &mut security.token_quota,
            &format!("{}|{}", input.endpoint, token.token_id),
            config.quota_window_ms,
            config.quota_per_token,
            now_ms,
        )?;
        enforce_quota_counter(
            &mut security.device_quota,
            &format!("{}|{}", input.endpoint, token.device),
            config.quota_window_ms,
            config.quota_per_device,
            now_ms,
        )?;
    }
    if security.replay_cache.contains_key(&replay_key) {
        return Err(SecurityReject::conflict("replayed_request"));
    }
    security
        .replay_cache
        .insert(replay_key, now_ms.saturating_add(config.replay_ttl_ms));
    Ok(())
}

fn enforce_quota_counter(
    counters: &mut BTreeMap<String, QuotaWindowCounter>,
    key: &str,
    window_ms: u64,
    max_count: u32,
    now_ms: u64,
) -> Result<(), SecurityReject> {
    let counter = counters
        .entry(key.to_string())
        .or_insert_with(|| QuotaWindowCounter {
            window_start_ms: now_ms,
            count: 0,
        });
    if now_ms.saturating_sub(counter.window_start_ms) >= window_ms {
        counter.window_start_ms = now_ms;
        counter.count = 0;
    }
    if counter.count >= max_count {
        let next_at = counter.window_start_ms.saturating_add(window_ms);
        let retry_after_ms = next_at.saturating_sub(now_ms).max(1);
        let retry_after_secs = retry_after_ms.saturating_add(999) / 1000;
        return Err(SecurityReject::too_many_requests(
            "quota_exceeded",
            retry_after_secs,
        ));
    }
    counter.count = counter.count.saturating_add(1);
    Ok(())
}

const INGRESS_AUTH_SIGNING_KEYS_ENV: &str = "SELENE_INGRESS_AUTH_SIGNING_KEYS";
const DEFAULT_INGRESS_AUTH_KEY_ID: &str = "ingress_kid_v1";
const DEFAULT_INGRESS_AUTH_SECRET: &str = "selene_ingress_local_dev_secret_v1";
const INGRESS_AUTH_VERSION: &str = "v1";

fn parse_and_verify_bearer(value: &str) -> Result<ParsedBearerToken, SecurityReject> {
    let trimmed = value.trim();
    let token = trimmed
        .strip_prefix("Bearer ")
        .or_else(|| trimmed.strip_prefix("bearer "))
        .ok_or_else(|| SecurityReject::unauthorized("missing_bearer_scheme"))?;
    let mut parts = token.split('.');
    let version = parts.next().unwrap_or_default();
    let key_id = parts.next().unwrap_or_default();
    let subject = parts.next().unwrap_or_default();
    let device = parts.next().unwrap_or_default();
    let digest = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || version.is_empty()
        || key_id.is_empty()
        || subject.is_empty()
        || device.is_empty()
        || digest.is_empty()
    {
        return Err(SecurityReject::unauthorized("invalid_bearer_format"));
    }
    if version != INGRESS_AUTH_VERSION {
        return Err(SecurityReject::unauthorized("unsupported_bearer_version"));
    }
    if !subject.is_ascii()
        || !device.is_ascii()
        || !key_id.is_ascii()
        || !digest.is_ascii()
        || subject.len() > 128
        || device.len() > 128
        || key_id.len() > 64
        || digest.len() > 64
    {
        return Err(SecurityReject::unauthorized("invalid_bearer_fields"));
    }
    let keyring = ingress_auth_keyring();
    let secret = keyring
        .get(key_id)
        .ok_or_else(|| SecurityReject::unauthorized("unknown_bearer_key_id"))?;
    let expected = deterministic_bearer_digest(subject, device, key_id, secret.as_str());
    if expected != digest {
        return Err(SecurityReject::unauthorized("invalid_bearer_signature"));
    }
    Ok(ParsedBearerToken {
        token_id: digest.to_string(),
        subject: subject.to_string(),
        device: device.to_string(),
    })
}

fn ingress_auth_keyring() -> BTreeMap<String, String> {
    let mut keyring = BTreeMap::new();
    if let Ok(raw) = env::var(INGRESS_AUTH_SIGNING_KEYS_ENV) {
        for entry in raw.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some((key_id_raw, secret_raw)) = trimmed.split_once(':') else {
                continue;
            };
            let key_id = key_id_raw.trim();
            let secret = secret_raw.trim();
            if key_id.is_empty()
                || key_id.len() > 64
                || !key_id.is_ascii()
                || secret.is_empty()
                || secret.len() > 256
                || !secret.is_ascii()
            {
                continue;
            }
            keyring.insert(key_id.to_string(), secret.to_string());
        }
    }
    if keyring.is_empty() {
        keyring.insert(
            DEFAULT_INGRESS_AUTH_KEY_ID.to_string(),
            DEFAULT_INGRESS_AUTH_SECRET.to_string(),
        );
    }
    keyring
}

fn deterministic_bearer_digest(subject: &str, device: &str, key_id: &str, secret: &str) -> String {
    hash_hex_64(format!("v1|{key_id}|{subject}|{device}|{secret}").as_bytes())
}

fn hash_hex_64(bytes: &[u8]) -> String {
    let mut h = fnv1a64(bytes);
    if h == 0 {
        h = 1;
    }
    format!("{h:016x}")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn system_time_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn status_for_security_reject(kind: SecurityRejectKind) -> StatusCode {
    match kind {
        SecurityRejectKind::Unauthorized => StatusCode::UNAUTHORIZED,
        SecurityRejectKind::Forbidden => StatusCode::FORBIDDEN,
        SecurityRejectKind::Conflict => StatusCode::CONFLICT,
        SecurityRejectKind::Unprocessable => StatusCode::UNPROCESSABLE_ENTITY,
        SecurityRejectKind::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
    }
}

fn status_for_failure_class(failure_class: FailureClass) -> StatusCode {
    match failure_class {
        FailureClass::AuthenticationFailure => StatusCode::UNAUTHORIZED,
        FailureClass::AuthorizationFailure => StatusCode::FORBIDDEN,
        FailureClass::InvalidPayload => StatusCode::UNPROCESSABLE_ENTITY,
        FailureClass::ReplayRejected | FailureClass::SessionConflict => StatusCode::CONFLICT,
        FailureClass::PolicyViolation => StatusCode::FORBIDDEN,
        FailureClass::ExecutionFailure => StatusCode::INTERNAL_SERVER_ERROR,
        FailureClass::RetryableRuntime => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[derive(Debug)]
struct MintedRealtimeTranscriptionSession {
    session_id: Option<String>,
    client_secret: String,
    expires_at: Option<u64>,
}

fn bounded_realtime_transcription_model(value: Option<&str>) -> String {
    let trimmed = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("gpt-4o-transcribe");
    match trimmed {
        "gpt-4o-transcribe" | "gpt-4o-transcribe-latest" | "gpt-4o-mini-transcribe" => {
            trimmed.to_string()
        }
        _ => "gpt-4o-transcribe".to_string(),
    }
}

fn openai_api_key_for_realtime_transcription() -> Result<String, String> {
    env::var("OPENAI_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            device_vault::resolve_secret(ProviderSecretId::OpenAIApiKey.as_str())
                .ok()
                .flatten()
        })
        .ok_or_else(|| "missing_openai_api_key".to_string())
}

fn openai_api_key_for_desktop_tts() -> Result<String, String> {
    openai_api_key_for_realtime_transcription()
}

fn mint_openai_realtime_transcription_session(
    model: &str,
    silence_duration_ms: u64,
) -> Result<MintedRealtimeTranscriptionSession, String> {
    let api_key = openai_api_key_for_realtime_transcription()?;
    let endpoint = env::var("OPENAI_REALTIME_TRANSCRIPTION_SESSION_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://api.openai.com/v1/realtime/transcription_sessions".to_string());
    let timeout_secs = parse_u64_env(
        "SELENE_DESKTOP_REALTIME_TRANSCRIPTION_TOKEN_TIMEOUT_SECS",
        8,
        1,
        30,
    );
    let body = serde_json::json!({
        "input_audio_format": "pcm16",
        "input_audio_transcription": {
            "model": model,
            "prompt": ""
        },
        "turn_detection": {
            "type": "server_vad",
            "threshold": 0.5,
            "prefix_padding_ms": 300,
            "silence_duration_ms": silence_duration_ms.min(10_000).max(300)
        },
        "input_audio_noise_reduction": {
            "type": "near_field"
        },
        "include": []
    });
    let body = serde_json::to_string(&body)
        .map_err(|_| "realtime_transcription_token_payload_encoding_failed".to_string())?;
    let output = Command::new("curl")
        .arg("-sS")
        .arg("--fail-with-body")
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg("-X")
        .arg("POST")
        .arg(&endpoint)
        .arg("-H")
        .arg(format!("Authorization: Bearer {api_key}"))
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg("OpenAI-Beta: realtime=v1")
        .arg("--data")
        .arg(body)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| "realtime_transcription_token_request_spawn_failed".to_string())?;
    if !output.status.success() {
        return Err("realtime_transcription_token_request_failed".to_string());
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| "realtime_transcription_token_response_decode_failed".to_string())?;
    let client_secret = value
        .get("client_secret")
        .and_then(|secret| secret.get("value"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| "realtime_transcription_token_missing_client_secret".to_string())?;
    let session_id = value
        .get("id")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let expires_at = value
        .get("client_secret")
        .and_then(|secret| secret.get("expires_at"))
        .and_then(|value| value.as_u64())
        .or_else(|| value.get("expires_at").and_then(|value| value.as_u64()));
    Ok(MintedRealtimeTranscriptionSession {
        session_id,
        client_secret,
        expires_at,
    })
}

fn bounded_openai_tts_model(value: Option<&str>) -> String {
    let trimmed = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("gpt-4o-mini-tts");
    match trimmed {
        "gpt-4o-mini-tts" => trimmed.to_string(),
        _ => "gpt-4o-mini-tts".to_string(),
    }
}

fn bounded_openai_tts_voice(value: Option<&str>) -> String {
    let trimmed = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("marin");
    match trimmed {
        "alloy" | "ash" | "ballad" | "coral" | "echo" | "fable" | "nova" | "onyx" | "sage"
        | "shimmer" | "verse" | "marin" | "cedar" => trimmed.to_string(),
        _ => "marin".to_string(),
    }
}

fn bounded_openai_tts_format(value: Option<&str>) -> String {
    let trimmed = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("mp3");
    match trimmed {
        "mp3" | "opus" | "aac" | "flac" | "wav" | "pcm" => trimmed.to_string(),
        _ => "mp3".to_string(),
    }
}

fn audio_mime_type_for_openai_tts_format(format: &str) -> &'static str {
    match format {
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "opus" => "audio/ogg",
        "pcm" => "audio/pcm",
        "wav" => "audio/wav",
        _ => "audio/mpeg",
    }
}

fn bounded_openai_tts_input_text(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("openai_tts_empty_response_text".to_string());
    }
    let max_chars =
        parse_u64_env("SELENE_DESKTOP_OPENAI_TTS_MAX_INPUT_CHARS", 4_096, 1, 4_096) as usize;
    if trimmed.chars().count() > max_chars {
        return Err("openai_tts_input_too_large".to_string());
    }
    Ok(trimmed.to_string())
}

fn speakable_openai_tts_text(text: &str) -> Result<(), String> {
    let lowered = text.to_ascii_lowercase();
    let forbidden = [
        "desktop runtime bridge",
        "governance",
        "provider payload",
        "raw provider",
        "debug",
        "reason_code",
        "invalidschema",
        "openai_api_key",
        "internal runtime",
        "stack trace",
    ];
    if forbidden.iter().any(|needle| lowered.contains(needle)) {
        return Err("openai_tts_response_not_speakable".to_string());
    }
    Ok(())
}

fn request_openai_tts_speech(
    response_text: &str,
    model: &str,
    voice: &str,
    format: &str,
) -> Result<Vec<u8>, String> {
    let api_key = openai_api_key_for_desktop_tts()?;
    let endpoint = env::var("OPENAI_AUDIO_SPEECH_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://api.openai.com/v1/audio/speech".to_string());
    let timeout_secs = parse_u64_env("SELENE_DESKTOP_OPENAI_TTS_TIMEOUT_SECS", 12, 1, 60);
    let body = serde_json::json!({
        "model": model,
        "input": response_text,
        "voice": voice,
        "response_format": format
    });
    let body = serde_json::to_string(&body)
        .map_err(|_| "openai_tts_payload_encoding_failed".to_string())?;
    let output = Command::new("curl")
        .arg("-sS")
        .arg("--fail-with-body")
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg("-X")
        .arg("POST")
        .arg(&endpoint)
        .arg("-H")
        .arg(format!("Authorization: Bearer {api_key}"))
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("--data")
        .arg(body)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| "openai_tts_request_spawn_failed".to_string())?;
    if !output.status.success() {
        return Err("openai_tts_request_failed".to_string());
    }
    if output.stdout.is_empty() {
        return Err("openai_tts_empty_audio_response".to_string());
    }
    Ok(output.stdout)
}

fn enforce_openai_tts_request_bounds(
    limiter: &Arc<Mutex<TtsRequestLimiterState>>,
    request: &DesktopOpenAiTtsSpeechRequest,
) -> Result<(), String> {
    let max_per_turn = parse_u64_env("SELENE_DESKTOP_OPENAI_TTS_MAX_REQUESTS_PER_TURN", 1, 1, 5);
    let max_per_session = parse_u64_env(
        "SELENE_DESKTOP_OPENAI_TTS_MAX_REQUESTS_PER_SESSION",
        30,
        1,
        200,
    );
    let turn_key = request
        .turn_id
        .as_ref()
        .map(|turn_id| format!("turn:{turn_id}"))
        .or_else(|| {
            request
                .request_id
                .as_ref()
                .map(|request_id| format!("request:{request_id}"))
        })
        .unwrap_or_else(|| format!("correlation:{}", request.correlation_id));
    let session_key = request
        .session_id
        .as_ref()
        .map(|session_id| format!("session:{session_id}"))
        .unwrap_or_else(|| format!("device:{}", request.device_id));
    let mut limiter = limiter
        .lock()
        .map_err(|_| "openai_tts_request_limiter_lock_poisoned".to_string())?;
    let turn_count = limiter.per_turn.entry(turn_key).or_insert(0);
    if *turn_count >= max_per_turn {
        return Err("openai_tts_max_requests_per_turn_exceeded".to_string());
    }
    *turn_count += 1;
    let session_count = limiter.per_session.entry(session_key).or_insert(0);
    if *session_count >= max_per_session {
        return Err("openai_tts_max_requests_per_session_exceeded".to_string());
    }
    *session_count += 1;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn runtime_execution_envelope_from_voice_turn_request(
    request: &VoiceTurnAdapterRequest,
    request_id: &str,
    idempotency_key: &str,
    device_id: &str,
) -> Result<RuntimeExecutionEnvelope, String> {
    build_runtime_execution_envelope_for_voice_turn_request(
        request,
        request_id,
        idempotency_key,
        device_id,
    )
}

fn failure_class_for_security_reject(kind: SecurityRejectKind) -> FailureClass {
    match kind {
        SecurityRejectKind::Unauthorized => FailureClass::AuthenticationFailure,
        SecurityRejectKind::Forbidden => FailureClass::AuthorizationFailure,
        SecurityRejectKind::Conflict => FailureClass::ReplayRejected,
        SecurityRejectKind::Unprocessable => FailureClass::InvalidPayload,
        SecurityRejectKind::TooManyRequests => FailureClass::RetryableRuntime,
    }
}

fn canonical_reason_code_for_security_reject(reason: &str) -> String {
    let mut out = String::with_capacity(reason.len());
    let mut prev_underscore = false;
    for ch in reason.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            prev_underscore = false;
            ch.to_ascii_uppercase()
        } else {
            if prev_underscore {
                continue;
            }
            prev_underscore = true;
            '_'
        };
        out.push(next);
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "SECURITY_REJECTED".to_string()
    } else {
        out
    }
}

fn voice_turn_security_reject_response(reject: SecurityReject, turn_id: Option<u64>) -> Response {
    let status = status_for_security_reject(reject.kind);
    let reason = reject.reason;
    let response = VoiceTurnAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        session_id: None,
        turn_id,
        session_state: None,
        session_attach_outcome: None,
        failure_class: Some(failure_class_for_security_reject(reject.kind)),
        reason: Some(reason.clone()),
        next_move: "respond".to_string(),
        response_text: String::new(),
        reason_code: canonical_reason_code_for_security_reject(&reason),
        provenance: None,
        deep_research: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn desktop_realtime_transcription_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = DesktopRealtimeTranscriptionSessionResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        session_id: None,
        client_secret: None,
        expires_at: None,
        websocket_url: String::new(),
        transcription_model: "gpt-4o-transcribe".to_string(),
        input_audio_format: "pcm16_24000_mono".to_string(),
        max_session_duration_ms: 0,
        max_silence_duration_ms: 0,
        max_reconnect_attempts: 0,
        retry_count: 0,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn desktop_realtime_transcription_error_response(status: StatusCode, reason: &str) -> Response {
    (
        status,
        Json(DesktopRealtimeTranscriptionSessionResponse {
            status: "error".to_string(),
            outcome: "FAILED_CLOSED".to_string(),
            reason: Some(reason.to_string()),
            session_id: None,
            client_secret: None,
            expires_at: None,
            websocket_url: String::new(),
            transcription_model: "gpt-4o-transcribe".to_string(),
            input_audio_format: "pcm16_24000_mono".to_string(),
            max_session_duration_ms: 0,
            max_silence_duration_ms: 0,
            max_reconnect_attempts: 0,
            retry_count: 0,
        }),
    )
        .into_response()
}

fn desktop_openai_tts_security_reject_response(
    reject: SecurityReject,
    request: &DesktopOpenAiTtsSpeechRequest,
) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = desktop_openai_tts_failure_response(request, "REJECTED", reject.reason);
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn desktop_openai_tts_error_response(
    status: StatusCode,
    request: &DesktopOpenAiTtsSpeechRequest,
    reason: &str,
) -> Response {
    (
        status,
        Json(desktop_openai_tts_failure_response(
            request,
            "FAILED_CLOSED",
            reason.to_string(),
        )),
    )
        .into_response()
}

fn desktop_openai_tts_failure_response(
    request: &DesktopOpenAiTtsSpeechRequest,
    outcome: &str,
    reason: String,
) -> DesktopOpenAiTtsSpeechResponse {
    DesktopOpenAiTtsSpeechResponse {
        status: "error".to_string(),
        outcome: outcome.to_string(),
        ok: false,
        safe_failure_reason: Some(reason),
        audio_base64: None,
        audio_mime_type: None,
        audio_byte_len: 0,
        audio_sha256: None,
        model: "gpt-4o-mini-tts".to_string(),
        voice: bounded_openai_tts_voice(request.voice.as_deref()),
        answer_text_sha256: None,
        session_id: request.session_id.clone(),
        turn_id: request.turn_id.clone(),
        request_id: request.request_id.clone(),
        fallback_allowed: true,
        ai_voice_disclosure: "synthetic_ai_voice".to_string(),
    }
}

fn invite_click_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = InviteLinkOpenAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        onboarding_session_id: None,
        next_step: None,
        required_fields: Vec::new(),
        required_verification_gates: Vec::new(),
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn onboarding_continue_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = OnboardingContinueAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        onboarding_session_id: None,
        next_step: None,
        blocking_field: None,
        blocking_question: None,
        remaining_missing_fields: Vec::new(),
        remaining_platform_receipt_kinds: Vec::new(),
        voice_artifact_sync_receipt_ref: None,
        access_engine_instance_id: None,
        onboarding_status: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn session_attach_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = SessionAttachAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        session_id: None,
        session_state: None,
        session_attach_outcome: None,
        project_id: None,
        pinned_context_refs: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn session_resume_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = SessionResumeAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        session_id: None,
        session_state: None,
        session_attach_outcome: None,
        project_id: None,
        pinned_context_refs: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn session_recover_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = SessionRecoverAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        session_id: None,
        session_state: None,
        session_attach_outcome: None,
        project_id: None,
        pinned_context_refs: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn session_recent_list_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = SessionRecentListAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        sessions: Vec::new(),
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn session_posture_evidence_security_reject_response(reject: SecurityReject) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = SessionPostureEvidenceAdapterResponse {
        status: "error".to_string(),
        outcome: "REJECTED".to_string(),
        reason: Some(reject.reason),
        session_id: None,
        session_state: None,
        last_turn_id: None,
        project_id: None,
        pinned_context_refs: None,
        session_attach_outcome: None,
        selected_thread_id: None,
        selected_thread_title: None,
        pending_work_order_id: None,
        resume_tier: None,
        resume_summary_bullets: None,
        archived_user_turn_text: None,
        archived_selene_turn_text: None,
        recovery_mode: None,
        reconciliation_decision: None,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn wake_profile_availability_security_reject_response(
    reject: SecurityReject,
    device_id: Option<String>,
    wake_profile_id: Option<String>,
) -> Response {
    let status = status_for_security_reject(reject.kind);
    let response = WakeProfileAvailabilityRefreshAdapterResponse {
        status: "error".to_string(),
        outcome: "FAILED_CLOSED".to_string(),
        reason: Some(reject.reason),
        device_id,
        wake_profile_id,
        active_wake_artifact_version: None,
        activated_count: 0,
        noop_count: 0,
        pull_error_count: 0,
    };
    json_response_with_optional_retry_after(status, response, reject.retry_after_secs)
}

fn voice_turn_ingress_error_response(status: StatusCode, error: VoiceTurnIngressError) -> Response {
    (
        status,
        Json(VoiceTurnAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            session_id: error.session_id,
            turn_id: error.turn_id,
            session_state: error.session_state,
            session_attach_outcome: None,
            failure_class: Some(error.failure_class),
            reason: error.reason,
            next_move: "respond".to_string(),
            response_text: String::new(),
            reason_code: error.reason_code,
            provenance: None,
            deep_research: None,
        }),
    )
        .into_response()
}

fn invite_click_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(InviteLinkOpenAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            onboarding_session_id: None,
            next_step: None,
            required_fields: Vec::new(),
            required_verification_gates: Vec::new(),
        }),
    )
        .into_response()
}

fn onboarding_continue_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
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
    )
        .into_response()
}

fn session_resume_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(SessionResumeAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            session_id: None,
            session_state: None,
            session_attach_outcome: None,
            project_id: None,
            pinned_context_refs: None,
        }),
    )
        .into_response()
}

fn session_recover_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(SessionRecoverAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            session_id: None,
            session_state: None,
            session_attach_outcome: None,
            project_id: None,
            pinned_context_refs: None,
        }),
    )
        .into_response()
}

fn session_attach_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(SessionAttachAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            session_id: None,
            session_state: None,
            session_attach_outcome: None,
            project_id: None,
            pinned_context_refs: None,
        }),
    )
        .into_response()
}

fn session_recent_list_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(SessionRecentListAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            sessions: Vec::new(),
        }),
    )
        .into_response()
}

fn session_posture_evidence_error_response(status: StatusCode, reason: String) -> Response {
    (
        status,
        Json(SessionPostureEvidenceAdapterResponse {
            status: "error".to_string(),
            outcome: "REJECTED".to_string(),
            reason: Some(reason),
            session_id: None,
            session_state: None,
            last_turn_id: None,
            project_id: None,
            pinned_context_refs: None,
            session_attach_outcome: None,
            selected_thread_id: None,
            selected_thread_title: None,
            pending_work_order_id: None,
            resume_tier: None,
            resume_summary_bullets: None,
            archived_user_turn_text: None,
            archived_selene_turn_text: None,
            recovery_mode: None,
            reconciliation_decision: None,
        }),
    )
        .into_response()
}

fn wake_profile_availability_error_response(
    status: StatusCode,
    device_id: Option<String>,
    wake_profile_id: Option<String>,
    reason: String,
) -> Response {
    (
        status,
        Json(WakeProfileAvailabilityRefreshAdapterResponse {
            status: "error".to_string(),
            outcome: "FAILED_CLOSED".to_string(),
            reason: Some(reason),
            device_id,
            wake_profile_id,
            active_wake_artifact_version: None,
            activated_count: 0,
            noop_count: 0,
            pull_error_count: 0,
        }),
    )
        .into_response()
}

fn json_response_with_optional_retry_after<T>(
    status: StatusCode,
    body: T,
    retry_after_secs: Option<u64>,
) -> Response
where
    T: serde::Serialize,
{
    let mut response = (status, Json(body)).into_response();
    if let Some(secs) = retry_after_secs {
        if let Ok(value) = HeaderValue::from_str(&secs.max(1).to_string()) {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        ffi::OsString,
        fs,
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc, Mutex, OnceLock,
        },
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use axum::body::to_bytes;
    use axum::http::header::AUTHORIZATION;
    use selene_adapter::VoiceTurnAudioCaptureRef;
    use selene_engines::device_vault::DeviceVault;
    use selene_kernel_contracts::common::SessionState;
    use selene_kernel_contracts::ph1_voice_id::{
        UserId, VoiceEmbeddingCaptureRef, VOICE_ID_ENROLL_COMPLETE_COMMIT,
        VOICE_ID_ENROLL_SAMPLE_COMMIT, VOICE_ID_ENROLL_START_DRAFT,
    };
    use selene_kernel_contracts::ph1art::ArtifactVersion;
    use selene_kernel_contracts::ph1emocore::EMO_SIM_001;
    use selene_kernel_contracts::ph1j::{DeviceId, TurnId};
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::ph1link::{
        InviteeType, LINK_INVITE_DRAFT_UPDATE_COMMIT, LINK_INVITE_OPEN_ACTIVATE_COMMIT,
    };
    use selene_kernel_contracts::ph1onb::{
        ONB_ACCESS_INSTANCE_CREATE_COMMIT, ONB_COMPLETE_COMMIT, ONB_PRIMARY_DEVICE_CONFIRM_COMMIT,
        ONB_SESSION_START_DRAFT, ONB_TERMS_ACCEPT_COMMIT,
    };
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1simcat::{
        SimulationCatalogEventInput, SimulationId, SimulationStatus, SimulationType,
        SimulationVersion,
    };
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
    use selene_os::app_ingress::AppServerIngressRuntime;
    use selene_storage::ph1f::{
        DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, TenantCompanyLifecycleState,
        TenantCompanyRecord, WakeSampleResult,
    };

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_ref() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    struct MockRealtimeSessionServer {
        url: String,
        request_rx: mpsc::Receiver<String>,
        join: thread::JoinHandle<()>,
    }

    fn spawn_mock_realtime_session_server(
        status: u16,
        body: serde_json::Value,
    ) -> MockRealtimeSessionServer {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("mock realtime transcription listener should bind");
        let address = listener.local_addr().expect("mock realtime address");
        let (request_tx, request_rx) = mpsc::channel();
        let body = body.to_string();
        let join = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buffer = [0_u8; 8192];
            let request_len = stream.read(&mut buffer).unwrap_or(0);
            let request_text = String::from_utf8_lossy(&buffer[..request_len]).to_string();
            let _ = request_tx.send(request_text);
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        });
        MockRealtimeSessionServer {
            url: format!("http://{address}/v1/realtime/transcription_sessions"),
            request_rx,
            join,
        }
    }

    struct MockOpenAiSpeechServer {
        url: String,
        request_rx: mpsc::Receiver<String>,
        join: thread::JoinHandle<()>,
    }

    fn spawn_mock_openai_speech_server(
        status: u16,
        content_type: &'static str,
        body: Vec<u8>,
    ) -> MockOpenAiSpeechServer {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("mock OpenAI speech listener should bind");
        let address = listener.local_addr().expect("mock OpenAI speech address");
        let (request_tx, request_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buffer = [0_u8; 8192];
            let request_len = stream.read(&mut buffer).unwrap_or(0);
            let request_text = String::from_utf8_lossy(&buffer[..request_len]).to_string();
            let _ = request_tx.send(request_text);
            let mut response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes();
            response.extend_from_slice(&body);
            let _ = stream.write_all(&response);
        });
        MockOpenAiSpeechServer {
            url: format!("http://{address}/v1/audio/speech"),
            request_rx,
            join,
        }
    }

    fn isolated_realtime_vault_path(label: &str, secrets: &[(&str, &str)]) -> (PathBuf, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time must be monotonic for tests")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("selene-http-realtime-vault-{label}-{nanos}.json"));
        let key_path = path.with_extension("master.key");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&key_path);
        let vault = DeviceVault::for_paths(path.clone(), key_path.clone());
        for (key, value) in secrets {
            vault
                .set_secret(key, value)
                .expect("test vault secret seed should succeed");
        }
        (path, key_path)
    }

    fn realtime_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("realtime transcription env lock poisoned")
    }

    fn test_runtime() -> AdapterRuntime {
        test_runtime_with_store().0
    }

    fn test_persistence_state_path(journal_path: &std::path::Path) -> PathBuf {
        PathBuf::from(format!("{}.state.json", journal_path.display()))
    }

    fn unique_test_journal_path(label: &str) -> PathBuf {
        static NEXT_TEST_JOURNAL_ID: AtomicU64 = AtomicU64::new(1);

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be >= unix epoch")
            .as_nanos();
        let ordinal = NEXT_TEST_JOURNAL_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "selene_ingress_http_test_{label}_{seed}_{ordinal}.jsonl"
        ));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(test_persistence_state_path(&path));
        path
    }

    fn attested_audio_capture_ref() -> VoiceTurnAudioCaptureRef {
        VoiceTurnAudioCaptureRef {
            stream_id: 11,
            pre_roll_buffer_id: 1,
            t_start_ns: 1,
            t_end_ns: 3,
            t_candidate_start_ns: 2,
            t_confirmed_ns: 3,
            locale_tag: Some("en-US".to_string()),
            device_route: Some("BUILT_IN".to_string()),
            selected_mic: Some("tablet_mic_default".to_string()),
            selected_speaker: Some("tablet_speaker_default".to_string()),
            tts_playback_active: Some(true),
            detection_text: Some("stop".to_string()),
            detection_confidence_bp: Some(9_600),
            vad_confidence_bp: Some(9_400),
            acoustic_confidence_bp: Some(9_300),
            prosody_confidence_bp: Some(9_200),
            speech_likeness_bp: Some(9_500),
            echo_safe_confidence_bp: Some(9_100),
            nearfield_confidence_bp: Some(9_000),
            capture_degraded: Some(false),
            stream_gap_detected: Some(false),
            aec_unstable: Some(false),
            device_changed: Some(false),
            snr_db_milli: Some(22_000),
            clipping_ratio_bp: Some(80),
            echo_delay_ms_milli: Some(26_000),
            packet_loss_bp: Some(25),
            double_talk_bp: Some(400),
            erle_db_milli: Some(20_000),
            device_failures_24h: Some(0),
            device_recoveries_24h: Some(0),
            device_mean_recovery_ms: Some(100),
            device_reliability_bp: Some(9_900),
            timing_jitter_ms_milli: Some(7_000),
            timing_drift_ppm_milli: Some(3_000),
            timing_buffer_depth_ms_milli: Some(35_000),
            timing_underruns: Some(0),
            timing_overruns: Some(0),
        }
    }

    fn test_runtime_with_store() -> (AdapterRuntime, Arc<Mutex<Ph1fStore>>) {
        let journal_path = unique_test_journal_path("runtime");
        let store = Arc::new(Mutex::new(Ph1fStore::new_in_memory()));
        let runtime = AdapterRuntime::new_with_persistence(
            AppServerIngressRuntime::default(),
            store.clone(),
            journal_path,
            true,
        )
        .expect("test runtime must bootstrap");
        (runtime, store)
    }

    fn test_state_with_config(config: IngressSecurityConfig) -> HttpAdapterState {
        HttpAdapterState {
            runtime: Arc::new(Mutex::new(test_runtime())),
            ingress_security: Arc::new(Mutex::new(IngressSecurityState::default())),
            ingress_security_config: config,
            tts_request_limiter: Arc::new(Mutex::new(TtsRequestLimiterState::default())),
        }
    }

    fn test_state_with_config_and_store(
        config: IngressSecurityConfig,
    ) -> (HttpAdapterState, Arc<Mutex<Ph1fStore>>) {
        let (runtime, store) = test_runtime_with_store();
        (
            HttpAdapterState {
                runtime: Arc::new(Mutex::new(runtime)),
                ingress_security: Arc::new(Mutex::new(IngressSecurityState::default())),
                ingress_security_config: config,
                tts_request_limiter: Arc::new(Mutex::new(TtsRequestLimiterState::default())),
            },
            store,
        )
    }

    fn base_voice_request() -> VoiceTurnAdapterRequest {
        VoiceTurnAdapterRequest {
            correlation_id: 10_001,
            turn_id: 20_001,
            device_turn_sequence: None,
            app_platform: "DESKTOP".to_string(),
            platform_version: None,
            device_class: None,
            runtime_client_version: None,
            hardware_capability_profile: None,
            network_profile: None,
            claimed_capabilities: None,
            integrity_status: None,
            attestation_ref: None,
            trigger: "EXPLICIT".to_string(),
            actor_user_id: "tenant_a:user_ingress_test".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            device_id: Some("ingress_device_01".to_string()),
            now_ns: Some(1),
            thread_key: None,
            project_id: None,
            pinned_context_refs: None,
            thread_policy_flags: None,
            user_text_partial: None,
            user_text_final: Some("hello".to_string()),
            selene_text_partial: None,
            selene_text_final: None,
            audio_capture_ref: None,
            visual_input_ref: None,
        }
    }

    fn base_invite_request() -> InviteLinkOpenAdapterRequest {
        InviteLinkOpenAdapterRequest {
            correlation_id: 2001,
            idempotency_key: "invite-idem-1".to_string(),
            token_id: "token_ingress_invite_1".to_string(),
            token_signature: "v1.link_kid_v1.fake".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            app_platform: "IOS".to_string(),
            device_fingerprint: "fp_1".to_string(),
            app_instance_id: "ios_device_1".to_string(),
            deep_link_nonce: "deep_link_nonce_1".to_string(),
        }
    }

    fn base_onboarding_request() -> OnboardingContinueAdapterRequest {
        OnboardingContinueAdapterRequest {
            correlation_id: 3001,
            onboarding_session_id: "onb_session_ingress_1".to_string(),
            idempotency_key: "onb-idem-1".to_string(),
            tenant_id: Some("tenant_a".to_string()),
            action: "ASK_PROMPT_COMMIT".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some("ios_device_1".to_string()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        }
    }

    fn base_session_attach_request() -> SessionAttachAdapterRequest {
        SessionAttachAdapterRequest {
            correlation_id: 4000,
            idempotency_key: "session-attach-idem-1".to_string(),
            session_id: "4101".to_string(),
            device_id: "attach_device_1".to_string(),
        }
    }

    fn base_session_resume_request() -> SessionResumeAdapterRequest {
        SessionResumeAdapterRequest {
            correlation_id: 4001,
            idempotency_key: "session-resume-idem-1".to_string(),
            session_id: "4201".to_string(),
            device_id: "resume_device_1".to_string(),
        }
    }

    fn base_session_recover_request() -> SessionRecoverAdapterRequest {
        SessionRecoverAdapterRequest {
            correlation_id: 4002,
            idempotency_key: "session-recover-idem-1".to_string(),
            session_id: "4301".to_string(),
            device_id: "recover_device_1".to_string(),
        }
    }

    fn base_session_recent_list_request() -> SessionRecentListAdapterRequest {
        SessionRecentListAdapterRequest {
            correlation_id: 4003,
            idempotency_key: "session-recent-idem-1".to_string(),
            device_id: "recent_device_1".to_string(),
        }
    }

    fn base_session_posture_request() -> SessionPostureEvidenceAdapterRequest {
        SessionPostureEvidenceAdapterRequest {
            correlation_id: 4004,
            idempotency_key: "session-posture-idem-1".to_string(),
            session_id: "4401".to_string(),
            device_id: "posture_device_1".to_string(),
        }
    }

    fn ios_voice_request(actor_user_id: String, device_id: String) -> VoiceTurnAdapterRequest {
        VoiceTurnAdapterRequest {
            correlation_id: 88_001,
            turn_id: 98_001,
            device_turn_sequence: None,
            app_platform: "IOS".to_string(),
            platform_version: None,
            device_class: None,
            runtime_client_version: None,
            hardware_capability_profile: None,
            network_profile: None,
            claimed_capabilities: None,
            integrity_status: None,
            attestation_ref: None,
            trigger: "EXPLICIT".to_string(),
            actor_user_id,
            tenant_id: Some("tenant_1".to_string()),
            device_id: Some(device_id),
            now_ns: Some(3),
            thread_key: None,
            project_id: None,
            pinned_context_refs: None,
            thread_policy_flags: None,
            user_text_partial: None,
            user_text_final: Some("Selene, are we ready?".to_string()),
            selene_text_partial: None,
            selene_text_final: None,
            audio_capture_ref: Some(VoiceTurnAudioCaptureRef {
                stream_id: 11,
                pre_roll_buffer_id: 1,
                t_start_ns: 1,
                t_end_ns: 3,
                t_candidate_start_ns: 2,
                t_confirmed_ns: 3,
                locale_tag: Some("en-US".to_string()),
                device_route: Some("BUILT_IN".to_string()),
                selected_mic: Some("ios_mic_default".to_string()),
                selected_speaker: Some("ios_speaker_default".to_string()),
                tts_playback_active: Some(true),
                detection_text: Some("stop".to_string()),
                detection_confidence_bp: Some(9_600),
                vad_confidence_bp: Some(9_400),
                acoustic_confidence_bp: Some(9_300),
                prosody_confidence_bp: Some(9_200),
                speech_likeness_bp: Some(9_500),
                echo_safe_confidence_bp: Some(9_100),
                nearfield_confidence_bp: Some(9_000),
                capture_degraded: Some(false),
                stream_gap_detected: Some(false),
                aec_unstable: Some(false),
                device_changed: Some(false),
                snr_db_milli: Some(22_000),
                clipping_ratio_bp: Some(80),
                echo_delay_ms_milli: Some(26_000),
                packet_loss_bp: Some(25),
                double_talk_bp: Some(400),
                erle_db_milli: Some(20_000),
                device_failures_24h: Some(0),
                device_recoveries_24h: Some(0),
                device_mean_recovery_ms: Some(100),
                device_reliability_bp: Some(9_900),
                timing_jitter_ms_milli: Some(7_000),
                timing_drift_ppm_milli: Some(3_000),
                timing_buffer_depth_ms_milli: Some(35_000),
                timing_underruns: Some(0),
                timing_overruns: Some(0),
            }),
            visual_input_ref: None,
        }
    }

    fn seed_identity_and_device(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) {
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn seed_wake_profile_availability_refresh_route_state(
        store: &mut Ph1fStore,
        label: &str,
    ) -> WakeProfileAvailabilityRefreshAdapterRequest {
        let user_id = UserId::new(format!("tenant_1:{label}_wake_route")).unwrap();
        let device_id = DeviceId::new(format!("{label}_wake_route_device")).unwrap();
        seed_identity_and_device(store, &user_id, &device_id);

        let (link, _) = store
            .ph1link_invite_generate_draft(
                MonotonicTimeNs(1),
                user_id.clone(),
                InviteeType::Employee,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        store
            .ph1link_invite_open_activate_commit_with_idempotency(
                MonotonicTimeNs(2),
                link.token_id.clone(),
                link.token_signature.clone(),
                format!("{label}_desktop_fp"),
                AppPlatform::Desktop,
                format!("desktop_instance_{label}"),
                format!("desktop_nonce_{label}"),
                MonotonicTimeNs(2),
                format!("{label}_link_open"),
            )
            .unwrap();
        let onb = store
            .ph1onb_session_start_draft(
                MonotonicTimeNs(3),
                link.token_id,
                None,
                Some("tenant_1".to_string()),
                format!("{label}_desktop_fp"),
                AppPlatform::Desktop,
                format!("desktop_instance_{label}"),
                format!("desktop_nonce_{label}"),
                MonotonicTimeNs(2),
            )
            .unwrap();

        let voice_started = store
            .ph1vid_enroll_start_draft(
                MonotonicTimeNs(4),
                onb.onboarding_session_id.clone(),
                device_id.clone(),
                true,
                8,
                120_000,
                3,
            )
            .unwrap();
        store
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(5),
                voice_started.voice_enrollment_session_id.clone(),
                format!("sample_ref_{label}_1"),
                1,
                1_350,
                0.91,
                17.0,
                0.4,
                0.0,
                Some(
                    VoiceEmbeddingCaptureRef::v1(
                        format!("embed://desktop/voice/{label}/1"),
                        "desktop.voiceid.v1".to_string(),
                        256,
                    )
                    .unwrap(),
                ),
                format!("{label}_voice_sample_1"),
            )
            .unwrap();
        store
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(6),
                voice_started.voice_enrollment_session_id.clone(),
                format!("sample_ref_{label}_2"),
                2,
                1_340,
                0.92,
                17.2,
                0.4,
                0.0,
                None,
                format!("{label}_voice_sample_2"),
            )
            .unwrap();
        store
            .ph1vid_enroll_sample_commit(
                MonotonicTimeNs(7),
                voice_started.voice_enrollment_session_id.clone(),
                format!("sample_ref_{label}_3"),
                3,
                1_360,
                0.93,
                17.5,
                0.3,
                0.0,
                None,
                format!("{label}_voice_sample_3"),
            )
            .unwrap();
        let voice_completed = store
            .ph1vid_enroll_complete_commit(
                MonotonicTimeNs(8),
                voice_started.voice_enrollment_session_id.clone(),
                format!("{label}_voice_complete"),
            )
            .unwrap();
        let voice_artifact_sync_receipt_ref = voice_completed
            .voice_artifact_sync_receipt_ref
            .clone()
            .unwrap();

        let wake_started = store
            .ph1w_enroll_start_draft(
                MonotonicTimeNs(9),
                user_id,
                device_id.clone(),
                Some(onb.onboarding_session_id),
                3,
                12,
                300_000,
                format!("{label}_wake_start"),
            )
            .unwrap();
        for sample_idx in 0..3_u64 {
            store
                .ph1w_enroll_sample_commit(
                    MonotonicTimeNs(10 + sample_idx),
                    wake_started.wake_enrollment_session_id.clone(),
                    900,
                    0.70,
                    14.0,
                    1.0,
                    -24.0,
                    -46.0,
                    -10.0,
                    0.04,
                    WakeSampleResult::Pass,
                    None,
                    format!("{label}_wake_sample_{sample_idx}"),
                )
                .unwrap();
        }
        let wake_profile_id = format!("wake_profile_{label}_route");
        store
            .ph1w_enroll_complete_commit(
                MonotonicTimeNs(20),
                wake_started.wake_enrollment_session_id,
                wake_profile_id.clone(),
                format!("{label}_wake_complete"),
            )
            .unwrap();
        store
            .wake_artifact_stage_commit(
                MonotonicTimeNs(21),
                device_id.clone(),
                ArtifactVersion(7),
                format!("{:064x}", 7),
                format!("cache://wake/{label}/7"),
                Some(format!("local://wake/{label}/7")),
                format!("{label}_wake_stage"),
            )
            .unwrap();
        store
            .wake_artifact_activate_commit(
                MonotonicTimeNs(22),
                device_id.clone(),
                ArtifactVersion(7),
                format!("{label}_wake_activate"),
            )
            .unwrap();

        WakeProfileAvailabilityRefreshAdapterRequest {
            correlation_id: 5001,
            idempotency_key: format!("{label}_wake_profile_availability"),
            device_id: device_id.as_str().to_string(),
            expected_wake_profile_id: wake_profile_id,
            voice_artifact_sync_receipt_ref,
        }
    }

    fn session_project_context_fixture() -> (String, Vec<String>) {
        (
            "proj_q3_planning".to_string(),
            vec![
                "ctx:spec/roadmap".to_string(),
                "ctx:file/launch_checklist".to_string(),
            ],
        )
    }

    fn apply_session_project_context(record: &mut selene_storage::ph1f::SessionRecord) {
        let (project_id, pinned_context_refs) = session_project_context_fixture();
        record.project_id = Some(project_id);
        record.pinned_context_refs = pinned_context_refs;
    }

    fn seed_soft_closed_session_record(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        origin_device_id: &DeviceId,
        attached_devices: &[DeviceId],
        last_attached_device_id: &DeviceId,
        last_turn_id: TurnId,
    ) {
        let mut record = selene_storage::ph1f::SessionRecord::v1(
            session_id,
            user_id.clone(),
            origin_device_id.clone(),
            SessionState::SoftClosed,
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            None,
        )
        .unwrap();
        record.attached_devices = attached_devices.iter().cloned().collect();
        record.last_attached_device_id = last_attached_device_id.clone();
        record.last_turn_id = Some(last_turn_id);
        record.device_turn_sequences = attached_devices
            .iter()
            .cloned()
            .map(|device_id| (device_id, last_turn_id.0))
            .collect();
        apply_session_project_context(&mut record);
        store
            .upsert_session_lifecycle(
                record,
                Some(format!("seed_http_soft_closed_session_{}", session_id.0)),
            )
            .unwrap();
    }

    fn seed_attachable_session_record(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        origin_device_id: &DeviceId,
        attached_devices: &[DeviceId],
        last_attached_device_id: &DeviceId,
        last_turn_id: TurnId,
        session_state: SessionState,
    ) {
        let mut record = selene_storage::ph1f::SessionRecord::v1(
            session_id,
            user_id.clone(),
            origin_device_id.clone(),
            session_state,
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            None,
        )
        .unwrap();
        record.attached_devices = attached_devices.iter().cloned().collect();
        record.last_attached_device_id = last_attached_device_id.clone();
        record.last_turn_id = Some(last_turn_id);
        record.device_turn_sequences = attached_devices
            .iter()
            .cloned()
            .map(|device_id| (device_id, last_turn_id.0))
            .collect();
        apply_session_project_context(&mut record);
        store
            .upsert_session_lifecycle(
                record,
                Some(format!("seed_http_attachable_session_{}", session_id.0)),
            )
            .unwrap();
    }

    fn seed_suspended_session_record(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        origin_device_id: &DeviceId,
        attached_devices: &[DeviceId],
        last_attached_device_id: &DeviceId,
        last_turn_id: TurnId,
    ) {
        let mut record = selene_storage::ph1f::SessionRecord::v1(
            session_id,
            user_id.clone(),
            origin_device_id.clone(),
            SessionState::Suspended,
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            None,
        )
        .unwrap();
        record.attached_devices = attached_devices.iter().cloned().collect();
        record.last_attached_device_id = last_attached_device_id.clone();
        record.last_turn_id = Some(last_turn_id);
        record.device_turn_sequences = attached_devices
            .iter()
            .cloned()
            .map(|device_id| (device_id, last_turn_id.0))
            .collect();
        apply_session_project_context(&mut record);
        store
            .upsert_session_lifecycle(
                record,
                Some(format!("seed_http_suspended_session_{}", session_id.0)),
            )
            .unwrap();
    }

    fn seed_recent_session_record_for_last_attached_device(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        origin_device_id: &DeviceId,
        attached_devices: &[DeviceId],
        last_attached_device_id: &DeviceId,
        session_state: SessionState,
        last_activity_at: MonotonicTimeNs,
        last_turn_id: Option<TurnId>,
    ) {
        if store.get_identity(user_id).is_none() {
            store
                .insert_identity(IdentityRecord::v1(
                    user_id.clone(),
                    None,
                    None,
                    MonotonicTimeNs(1),
                    IdentityStatus::Active,
                ))
                .unwrap();
        }
        for device_id in attached_devices {
            if store.get_device(device_id).is_none() {
                store
                    .insert_device(
                        DeviceRecord::v1(
                            device_id.clone(),
                            user_id.clone(),
                            "desktop".to_string(),
                            MonotonicTimeNs(2),
                            None,
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
        }

        let mut record = selene_storage::ph1f::SessionRecord::v1(
            session_id,
            user_id.clone(),
            origin_device_id.clone(),
            session_state,
            MonotonicTimeNs(10),
            last_activity_at,
            (session_state == SessionState::Closed).then_some(last_activity_at),
        )
        .unwrap();
        record.attached_devices = attached_devices.iter().cloned().collect();
        record.last_attached_device_id = last_attached_device_id.clone();
        record.last_turn_id = last_turn_id;
        if let Some(turn_id) = last_turn_id {
            record.device_turn_sequences = attached_devices
                .iter()
                .cloned()
                .map(|device_id| (device_id, turn_id.0))
                .collect();
        }
        apply_session_project_context(&mut record);
        store
            .upsert_session_lifecycle(
                record,
                Some(format!("seed_http_recent_session_{}", session_id.0)),
            )
            .unwrap();
    }

    fn seed_session_resume_selection_evidence_for_posture(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        device_id: &DeviceId,
        turn_id: TurnId,
    ) {
        store
            .append_conversation_turn(
                selene_kernel_contracts::ph1f::ConversationTurnInput::v1(
                    MonotonicTimeNs(330),
                    selene_kernel_contracts::ph1j::CorrelationId(94_106),
                    turn_id,
                    Some(session_id),
                    user_id.clone(),
                    Some(device_id.clone()),
                    selene_kernel_contracts::ph1f::ConversationRole::User,
                    selene_kernel_contracts::ph1f::ConversationSource::TypedText,
                    "resume this session".to_string(),
                    "hash_http_resume_this_session".to_string(),
                    selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                    Some(format!(
                        "seed_http_session_posture_resume_selection_turn_{}",
                        session_id.0
                    )),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let digest = selene_kernel_contracts::ph1m::MemoryThreadDigest::v1(
            "thread_resume_hot".to_string(),
            "Japan ski trip".to_string(),
            vec![
                "Flights shortlisted".to_string(),
                "Need hotel confirmation".to_string(),
            ],
            false,
            true,
            MonotonicTimeNs(330),
            5,
        )
        .unwrap();
        store
            .ph1m_thread_digest_upsert_commit(
                user_id,
                selene_kernel_contracts::ph1m::MemoryRetentionMode::Default,
                digest,
                selene_storage::ph1f::MemoryThreadEventKind::ThreadDigestUpsert,
                ReasonCodeId(0x4D00_1003),
                format!(
                    "seed_http_session_posture_resume_selection_digest_{}",
                    session_id.0
                ),
            )
            .unwrap();
        store
            .ph1m_upsert_thread_refs_for_user_turn_with_session(
                user_id,
                "thread_resume_hot",
                turn_id,
                MonotonicTimeNs(331),
            )
            .unwrap();
    }

    fn seed_session_archived_recent_slice_evidence_for_posture(
        store: &mut Ph1fStore,
        session_id: SessionId,
        user_id: &UserId,
        device_id: &DeviceId,
        turn_id: TurnId,
    ) {
        store
            .append_conversation_turn(
                selene_kernel_contracts::ph1f::ConversationTurnInput::v1(
                    MonotonicTimeNs(340),
                    selene_kernel_contracts::ph1j::CorrelationId(94_108),
                    turn_id,
                    Some(session_id),
                    user_id.clone(),
                    Some(device_id.clone()),
                    selene_kernel_contracts::ph1f::ConversationRole::User,
                    selene_kernel_contracts::ph1f::ConversationSource::TypedText,
                    "show me the archived session slice".to_string(),
                    format!("hash_http_archived_user_slice_{}", session_id.0),
                    selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                    Some(format!(
                        "seed_http_session_posture_archived_user_turn_{}",
                        session_id.0
                    )),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        store
            .append_conversation_turn(
                selene_kernel_contracts::ph1f::ConversationTurnInput::v1(
                    MonotonicTimeNs(341),
                    selene_kernel_contracts::ph1j::CorrelationId(94_109),
                    turn_id,
                    Some(session_id),
                    user_id.clone(),
                    Some(device_id.clone()),
                    selene_kernel_contracts::ph1f::ConversationRole::Selene,
                    selene_kernel_contracts::ph1f::ConversationSource::SeleneOutput,
                    "Here is the archived session slice summary.".to_string(),
                    format!("hash_http_archived_selene_slice_{}", session_id.0),
                    selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                    Some(format!(
                        "seed_http_session_posture_archived_selene_turn_{}",
                        session_id.0
                    )),
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn seed_simulation_catalog_status(
        store: &mut Ph1fStore,
        tenant: &str,
        simulation_id: &str,
        simulation_type: SimulationType,
        status: SimulationStatus,
    ) {
        let event = SimulationCatalogEventInput::v1(
            MonotonicTimeNs(1),
            TenantId::new(tenant.to_string()).unwrap(),
            SimulationId::new(simulation_id.to_string()).unwrap(),
            SimulationVersion(1),
            simulation_type,
            status,
            "PH1.TEST".to_string(),
            "reads_v1".to_string(),
            "writes_v1".to_string(),
            ReasonCodeId(1),
            None,
        )
        .unwrap();
        store.append_simulation_catalog_event(event).unwrap();
    }

    fn seed_invite_link_for_click(
        store: &mut Ph1fStore,
        inviter_user_id: &UserId,
    ) -> (String, String) {
        let now = MonotonicTimeNs(system_time_now_ms().max(1) * 1_000_000);
        let (link, _) = store
            .ph1link_invite_generate_draft(
                now,
                inviter_user_id.clone(),
                InviteeType::Friend,
                Some("tenant_1".to_string()),
                None,
                None,
                None,
            )
            .unwrap();
        (
            link.token_id.as_str().to_string(),
            link.token_signature.clone(),
        )
    }

    fn seed_company_position_minimum(store: &mut Ph1fStore) {
        let tenant_id = TenantId::new("tenant_1".to_string()).unwrap();
        store
            .ph1tenant_company_upsert(TenantCompanyRecord {
                schema_version: selene_kernel_contracts::SchemaVersion(1),
                tenant_id: tenant_id.clone(),
                company_id: "company_1".to_string(),
                legal_name: "Selene Co".to_string(),
                jurisdiction: "US".to_string(),
                lifecycle_state: TenantCompanyLifecycleState::Active,
                created_at: MonotonicTimeNs(1),
                updated_at: MonotonicTimeNs(1),
            })
            .unwrap();
        let position = selene_kernel_contracts::ph1position::PositionRecord::v1(
            tenant_id,
            "company_1".to_string(),
            selene_kernel_contracts::ph1position::PositionId::new("position_1").unwrap(),
            "Operator".to_string(),
            "Operations".to_string(),
            "US".to_string(),
            selene_kernel_contracts::ph1position::PositionScheduleType::FullTime,
            "profile_ops".to_string(),
            "band_l2".to_string(),
            selene_kernel_contracts::ph1position::PositionLifecycleState::Active,
            MonotonicTimeNs(1),
            MonotonicTimeNs(1),
        )
        .unwrap();
        store.ph1position_upsert(position).unwrap();
    }

    fn ask_missing_value(field_key: &str) -> String {
        match field_key {
            "tenant_id" => "tenant_1",
            "company_id" => "company_1",
            "position_id" => "position_1",
            "location_id" => "loc_1",
            "start_date" => "2026-03-01",
            "working_hours" => "09:00-17:00",
            "compensation_tier_ref" => "band_l2",
            "jurisdiction_tags" => "US,CA",
            _ => "value_1",
        }
        .to_string()
    }

    async fn decode_json_response<T>(response: Response) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body must read");
        serde_json::from_slice(&bytes).expect("response json must parse")
    }

    fn bearer_for(subject: &str, device: &str) -> String {
        let digest = deterministic_bearer_digest(
            subject,
            device,
            DEFAULT_INGRESS_AUTH_KEY_ID,
            DEFAULT_INGRESS_AUTH_SECRET,
        );
        format!(
            "Bearer {}.{}.{}.{}.{}",
            INGRESS_AUTH_VERSION, DEFAULT_INGRESS_AUTH_KEY_ID, subject, device, digest
        )
    }

    fn security_headers(
        bearer: Option<String>,
        request_id: &str,
        idempotency_key: &str,
        timestamp_ms: u64,
        nonce: &str,
    ) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = bearer {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(token.as_str()).expect("authorization header must parse"),
            );
        }
        headers.insert(
            "x-request-id",
            HeaderValue::from_str(request_id).expect("request id header must parse"),
        );
        headers.insert(
            "idempotency-key",
            HeaderValue::from_str(idempotency_key).expect("idempotency header must parse"),
        );
        headers.insert(
            "x-selene-timestamp-ms",
            HeaderValue::from_str(&timestamp_ms.to_string()).expect("timestamp header must parse"),
        );
        headers.insert(
            "x-selene-nonce",
            HeaderValue::from_str(nonce).expect("nonce header must parse"),
        );
        headers
    }

    fn base_realtime_transcription_session_request() -> DesktopRealtimeTranscriptionSessionRequest {
        DesktopRealtimeTranscriptionSessionRequest {
            correlation_id: 44_001,
            actor_user_id: "tenant_a:user_realtime_test".to_string(),
            device_id: "desktop_realtime_device_1".to_string(),
            requested_model: Some("gpt-4o-transcribe".to_string()),
            feature_flag_name: "SELENE_DESKTOP_OPENAI_REALTIME_TRANSCRIPTION_ENABLED".to_string(),
            feature_flag_enabled: true,
        }
    }

    fn base_openai_tts_speech_request() -> DesktopOpenAiTtsSpeechRequest {
        DesktopOpenAiTtsSpeechRequest {
            correlation_id: 55_001,
            actor_user_id: "tenant_a:user_tts_test".to_string(),
            device_id: "desktop_tts_device_1".to_string(),
            response_text: "Selene runtime final answer.".to_string(),
            session_id: Some("session_tts_test".to_string()),
            turn_id: Some("42".to_string()),
            request_id: Some("voice_turn_request_tts_test".to_string()),
            feature_flag_name: "SELENE_DESKTOP_OPENAI_TTS_ENABLED".to_string(),
            feature_flag_enabled: true,
            voice: Some("marin".to_string()),
            format: Some("mp3".to_string()),
        }
    }

    #[tokio::test]
    async fn ingress_realtime_transcription_session_without_bearer_returns_401() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_realtime_transcription_session_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(None, "req-rt-1", "idem-rt-1", now_ms, "nonce-rt-1");
        let response =
            run_desktop_realtime_transcription_session(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingress_realtime_transcription_session_flag_off_fails_closed() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let mut request = base_realtime_transcription_session_request();
        request.feature_flag_enabled = false;
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-rt-2",
            "idem-rt-2",
            now_ms,
            "nonce-rt-2",
        );
        let response =
            run_desktop_realtime_transcription_session(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
        let body: DesktopRealtimeTranscriptionSessionResponse =
            decode_json_response(response).await;
        assert_eq!(body.status, "error");
        assert_eq!(body.outcome, "FAILED_CLOSED");
        assert_eq!(
            body.reason.as_deref(),
            Some("desktop_realtime_transcription_feature_flag_disabled")
        );
        assert!(body.client_secret.is_none());
    }

    #[tokio::test]
    async fn ingress_realtime_transcription_session_missing_openai_key_fails_closed() {
        let _guard = realtime_env_lock();
        let (vault_path, key_path) = isolated_realtime_vault_path("missing-openai-key", &[]);
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_realtime_transcription_session_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-rt-missing-key",
            "idem-rt-missing-key",
            now_ms,
            "nonce-rt-missing-key",
        );
        let response =
            run_desktop_realtime_transcription_session(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: DesktopRealtimeTranscriptionSessionResponse =
            decode_json_response(response).await;
        assert_eq!(body.status, "error");
        assert_eq!(body.outcome, "FAILED_CLOSED");
        assert_eq!(body.reason.as_deref(), Some("missing_openai_api_key"));
        assert!(body.client_secret.is_none());
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        assert!(!serialized.contains("sk-"));
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_realtime_transcription_session_provider_failure_is_redacted() {
        let _guard = realtime_env_lock();
        let permanent_key = "sk-test-permanent-key-never-exposed";
        let mock = spawn_mock_realtime_session_server(
            400,
            serde_json::json!({
                "error": {
                    "message": "Invalid value: ''. Supported values are language codes.",
                    "type": "invalid_request_error",
                    "code": "invalid_value",
                    "param": "input_audio_transcription.language"
                }
            }),
        );
        let (vault_path, key_path) =
            isolated_realtime_vault_path("provider-failure", &[("openai_api_key", permanent_key)]);
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _endpoint_scope =
            ScopedEnvVar::set("OPENAI_REALTIME_TRANSCRIPTION_SESSION_URL", &mock.url);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_realtime_transcription_session_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-rt-provider-failure",
            "idem-rt-provider-failure",
            now_ms,
            "nonce-rt-provider-failure",
        );
        let response =
            run_desktop_realtime_transcription_session(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: DesktopRealtimeTranscriptionSessionResponse =
            decode_json_response(response).await;
        assert_eq!(body.status, "error");
        assert_eq!(body.outcome, "FAILED_CLOSED");
        assert_eq!(
            body.reason.as_deref(),
            Some("realtime_transcription_token_request_failed")
        );
        assert!(body.client_secret.is_none());
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        for forbidden in [
            permanent_key,
            "Invalid value",
            "input_audio_transcription.language",
            "invalid_request_error",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "desktop response leaked provider/key detail: {serialized}"
            );
        }
        let request_text = mock
            .request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock provider should receive request");
        assert!(request_text.contains("Authorization: Bearer "));
        mock.join.join().expect("mock provider thread should join");
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_realtime_transcription_session_success_maps_client_secret() {
        let _guard = realtime_env_lock();
        let permanent_key = "sk-test-permanent-key-never-exposed";
        let mock = spawn_mock_realtime_session_server(
            200,
            serde_json::json!({
                "id": "sess_realtime_transcription_test",
                "object": "realtime.transcription_session",
                "type": "transcription",
                "input_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": "gpt-4o-transcribe"
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 1200
                },
                "client_secret": {
                    "value": "ek_test_realtime_transcription_client_secret",
                    "expires_at": 1770000000
                },
                "expires_at": 1770000000
            }),
        );
        let (vault_path, key_path) =
            isolated_realtime_vault_path("success", &[("openai_api_key", permanent_key)]);
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _endpoint_scope =
            ScopedEnvVar::set("OPENAI_REALTIME_TRANSCRIPTION_SESSION_URL", &mock.url);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_realtime_transcription_session_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-rt-success",
            "idem-rt-success",
            now_ms,
            "nonce-rt-success",
        );
        let response =
            run_desktop_realtime_transcription_session(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: DesktopRealtimeTranscriptionSessionResponse =
            decode_json_response(response).await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "REALTIME_TRANSCRIPTION_SESSION_CREATED");
        assert_eq!(
            body.client_secret.as_deref(),
            Some("ek_test_realtime_transcription_client_secret")
        );
        assert_eq!(
            body.session_id.as_deref(),
            Some("sess_realtime_transcription_test")
        );
        assert_eq!(body.expires_at, Some(1770000000));
        assert_eq!(body.transcription_model, "gpt-4o-transcribe");
        assert_eq!(body.input_audio_format, "pcm16_24000_mono");
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        assert!(!serialized.contains(permanent_key));

        let request_text = mock
            .request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock provider should receive request");
        assert!(request_text.contains("POST /v1/realtime/transcription_sessions"));
        assert!(request_text.contains("\"input_audio_format\":\"pcm16\""));
        assert!(request_text.contains("\"model\":\"gpt-4o-transcribe\""));
        assert!(request_text.contains("\"input_audio_noise_reduction\":{\"type\":\"near_field\"}"));
        assert!(!request_text.contains("\"language\""));
        mock.join.join().expect("mock provider thread should join");
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_openai_tts_without_bearer_returns_401() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_openai_tts_speech_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(None, "req-tts-1", "idem-tts-1", now_ms, "nonce-tts-1");
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingress_openai_tts_flag_off_fails_closed_without_provider_call() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let mut request = base_openai_tts_speech_request();
        request.feature_flag_enabled = false;
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-tts-2",
            "idem-tts-2",
            now_ms,
            "nonce-tts-2",
        );
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(response).await;
        assert_eq!(body.status, "error");
        assert_eq!(body.outcome, "FAILED_CLOSED");
        assert_eq!(
            body.safe_failure_reason.as_deref(),
            Some("desktop_openai_tts_feature_flag_disabled")
        );
        assert!(body.audio_base64.is_none());
    }

    #[tokio::test]
    async fn ingress_openai_tts_missing_key_fails_closed_safely() {
        let _guard = realtime_env_lock();
        let (vault_path, key_path) = isolated_realtime_vault_path("tts-missing-key", &[]);
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_openai_tts_speech_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-tts-missing-key",
            "idem-tts-missing-key",
            now_ms,
            "nonce-tts-missing-key",
        );
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(response).await;
        assert_eq!(body.status, "error");
        assert_eq!(
            body.safe_failure_reason.as_deref(),
            Some("missing_openai_api_key")
        );
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        assert!(!serialized.contains("sk-"));
        assert!(body.audio_base64.is_none());
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_openai_tts_non_speakable_text_does_not_call_provider() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let mut request = base_openai_tts_speech_request();
        request.response_text =
            "Internal runtime reason_code InvalidSchema governance debug text".to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-tts-non-speakable",
            "idem-tts-non-speakable",
            now_ms,
            "nonce-tts-non-speakable",
        );
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(response).await;
        assert_eq!(
            body.safe_failure_reason.as_deref(),
            Some("openai_tts_response_not_speakable")
        );
        assert!(body.audio_base64.is_none());
    }

    #[tokio::test]
    async fn ingress_openai_tts_provider_failure_is_redacted() {
        let _guard = realtime_env_lock();
        let permanent_key = "sk-test-permanent-key-never-exposed";
        let mock = spawn_mock_openai_speech_server(
            400,
            "application/json",
            br#"{"error":{"message":"account quota raw detail","type":"invalid_request_error"}}"#
                .to_vec(),
        );
        let (vault_path, key_path) = isolated_realtime_vault_path(
            "tts-provider-failure",
            &[("openai_api_key", permanent_key)],
        );
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _endpoint_scope = ScopedEnvVar::set("OPENAI_AUDIO_SPEECH_URL", &mock.url);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_openai_tts_speech_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-tts-provider-failure",
            "idem-tts-provider-failure",
            now_ms,
            "nonce-tts-provider-failure",
        );
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(response).await;
        assert_eq!(
            body.safe_failure_reason.as_deref(),
            Some("openai_tts_request_failed")
        );
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        for forbidden in [
            permanent_key,
            "account quota raw detail",
            "invalid_request_error",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "desktop response leaked provider/key detail: {serialized}"
            );
        }
        let request_text = mock
            .request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock provider should receive request");
        assert!(request_text.contains("POST /v1/audio/speech"));
        assert!(request_text.contains("Authorization: Bearer "));
        mock.join.join().expect("mock provider thread should join");
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_openai_tts_success_returns_audio_and_hashes_without_key_leak() {
        let _guard = realtime_env_lock();
        let permanent_key = "sk-test-permanent-key-never-exposed";
        let audio_bytes = b"fake-mp3-audio-bytes-from-openai".to_vec();
        let expected_audio_sha = sha256_hex(&audio_bytes);
        let expected_audio_base64 = BASE64_STANDARD.encode(&audio_bytes);
        let mock = spawn_mock_openai_speech_server(200, "audio/mpeg", audio_bytes.clone());
        let (vault_path, key_path) =
            isolated_realtime_vault_path("tts-success", &[("openai_api_key", permanent_key)]);
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _endpoint_scope = ScopedEnvVar::set("OPENAI_AUDIO_SPEECH_URL", &mock.url);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_openai_tts_speech_request();
        let expected_answer_sha = sha256_hex(request.response_text.as_bytes());
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, &request.device_id)),
            "req-tts-success",
            "idem-tts-success",
            now_ms,
            "nonce-tts-success",
        );
        let response = run_desktop_openai_tts_speech(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(response).await;
        assert!(body.ok);
        assert_eq!(body.outcome, "OPENAI_TTS_SPEECH_CREATED");
        assert_eq!(body.model, "gpt-4o-mini-tts");
        assert_eq!(body.voice, "marin");
        assert_eq!(body.audio_mime_type.as_deref(), Some("audio/mpeg"));
        assert_eq!(body.audio_byte_len, audio_bytes.len() as u64);
        assert_eq!(
            body.audio_sha256.as_deref(),
            Some(expected_audio_sha.as_str())
        );
        assert_eq!(
            body.answer_text_sha256.as_deref(),
            Some(expected_answer_sha.as_str())
        );
        assert_eq!(
            body.audio_base64.as_deref(),
            Some(expected_audio_base64.as_str())
        );
        assert_eq!(body.ai_voice_disclosure, "synthetic_ai_voice");
        assert!(!body.fallback_allowed);
        let serialized = serde_json::to_string(&body).expect("response should serialize");
        assert!(!serialized.contains(permanent_key));

        let request_text = mock
            .request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock provider should receive request");
        assert!(request_text.contains("POST /v1/audio/speech"));
        assert!(request_text.contains("\"model\":\"gpt-4o-mini-tts\""));
        assert!(request_text.contains("\"voice\":\"marin\""));
        assert!(request_text.contains("\"response_format\":\"mp3\""));
        assert!(request_text.contains("\"input\":\"Selene runtime final answer.\""));
        mock.join.join().expect("mock provider thread should join");
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_openai_tts_rejects_more_than_one_request_per_turn() {
        let _guard = realtime_env_lock();
        let permanent_key = "sk-test-permanent-key-never-exposed";
        let mock = spawn_mock_openai_speech_server(200, "audio/mpeg", b"first-turn-audio".to_vec());
        let (vault_path, key_path) = isolated_realtime_vault_path(
            "tts-per-turn-limit",
            &[("openai_api_key", permanent_key)],
        );
        let vault_path_text = vault_path
            .to_str()
            .expect("test vault path should be valid UTF-8")
            .to_string();
        let _unset_openai = ScopedEnvVar::unset("OPENAI_API_KEY");
        let _vault_scope = ScopedEnvVar::set("SELENE_DEVICE_VAULT_PATH", &vault_path_text);
        let _endpoint_scope = ScopedEnvVar::set("OPENAI_AUDIO_SPEECH_URL", &mock.url);

        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let first_request = base_openai_tts_speech_request();
        let now_ms = system_time_now_ms();
        let first_headers = security_headers(
            Some(bearer_for(
                &first_request.actor_user_id,
                &first_request.device_id,
            )),
            "req-tts-limit-1",
            "idem-tts-limit-1",
            now_ms,
            "nonce-tts-limit-1",
        );
        let first_response =
            run_desktop_openai_tts_speech(State(state.clone()), first_headers, Json(first_request))
                .await;
        assert_eq!(first_response.status(), StatusCode::OK);

        let second_request = base_openai_tts_speech_request();
        let second_headers = security_headers(
            Some(bearer_for(
                &second_request.actor_user_id,
                &second_request.device_id,
            )),
            "req-tts-limit-2",
            "idem-tts-limit-2",
            now_ms + 1,
            "nonce-tts-limit-2",
        );
        let second_response =
            run_desktop_openai_tts_speech(State(state), second_headers, Json(second_request)).await;
        assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body: DesktopOpenAiTtsSpeechResponse = decode_json_response(second_response).await;
        assert_eq!(
            body.safe_failure_reason.as_deref(),
            Some("openai_tts_max_requests_per_turn_exceeded")
        );
        mock.join.join().expect("mock provider thread should join");
        let _ = fs::remove_file(vault_path);
        let _ = fs::remove_file(key_path);
    }

    #[tokio::test]
    async fn ingress_voice_turn_without_bearer_returns_401() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(None, "req-1", "idem-1", now_ms, "nonce-1");
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingress_voice_turn_token_device_mismatch_returns_403() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.actor_user_id, "other_device")),
            "req-2",
            "idem-2",
            now_ms,
            "nonce-2",
        );
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn ingress_voice_turn_replay_is_rejected_deterministically() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-3",
            "idem-3",
            now_ms,
            "nonce-3",
        );
        let first =
            run_voice_turn(State(state.clone()), headers.clone(), Json(request.clone())).await;
        assert_ne!(first.status(), StatusCode::CONFLICT);
        let second = run_voice_turn(State(state), headers, Json(request)).await;
        assert_eq!(second.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn ingress_voice_turn_stale_timestamp_is_rejected() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let stale_ms = now_ms.saturating_sub(600_000);
        let headers = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-4",
            "idem-4",
            stale_ms,
            "nonce-4",
        );
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn ingress_voice_turn_quota_exceeded_returns_429_with_retry_after() {
        let mut config = IngressSecurityConfig::from_env();
        config.quota_enabled = true;
        config.quota_window_ms = 60_000;
        config.quota_per_token = 1;
        config.quota_per_device = 1;
        let state = test_state_with_config(config);
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let headers_first = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-5a",
            "idem-5a",
            now_ms,
            "nonce-5a",
        );
        let headers_second = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-5b",
            "idem-5b",
            now_ms,
            "nonce-5b",
        );
        let first =
            run_voice_turn(State(state.clone()), headers_first, Json(request.clone())).await;
        assert_ne!(first.status(), StatusCode::TOO_MANY_REQUESTS);
        let second = run_voice_turn(State(state), headers_second, Json(request)).await;
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(second.headers().contains_key(header::RETRY_AFTER));
    }

    #[tokio::test]
    async fn ingress_voice_turn_valid_security_reaches_runtime_path() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_voice_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-6",
            "idem-6",
            now_ms,
            "nonce-6",
        );
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert!(
            matches!(
                response.status(),
                StatusCode::OK
                    | StatusCode::UNAUTHORIZED
                    | StatusCode::BAD_REQUEST
                    | StatusCode::FORBIDDEN
            ),
            "expected runtime status after ingress pass, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn ingress_voice_turn_tablet_platform_reaches_runtime_path() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let mut request = base_voice_request();
        request.app_platform = "TABLET".to_string();
        request.platform_version = Some("15.2".to_string());
        request.device_class = Some("TABLET".to_string());
        request.runtime_client_version = Some("2.3.4".to_string());
        request.hardware_capability_profile = Some("TABLET_PRO".to_string());
        request.network_profile = Some("STANDARD".to_string());
        request.claimed_capabilities = Some(vec![
            "MICROPHONE".to_string(),
            "CAMERA".to_string(),
            "SPEAKER_OUTPUT".to_string(),
            "WAKE_WORD".to_string(),
            "SENSOR_AVAILABILITY".to_string(),
        ]);
        request.integrity_status = Some("ATTESTED".to_string());
        request.attestation_ref = Some("tablet_attest_http_01".to_string());
        request.audio_capture_ref = Some(attested_audio_capture_ref());
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-tablet-1",
            "idem-tablet-1",
            now_ms,
            "nonce-tablet-1",
        );
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert_ne!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn ingress_voice_turn_invalid_platform_is_rejected_deterministically() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let mut request = base_voice_request();
        request.app_platform = "BLACKBERRY".to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(
                &request.actor_user_id,
                request.device_id.as_deref().unwrap_or_default(),
            )),
            "req-invalid-platform-1",
            "idem-invalid-platform-1",
            now_ms,
            "nonce-invalid-platform-1",
        );
        let response = run_voice_turn(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body: VoiceTurnAdapterResponse = decode_json_response(response).await;
        assert_eq!(body.failure_class, Some(FailureClass::InvalidPayload));
        assert_eq!(body.reason_code, "INVALID_RUNTIME_EXECUTION_ENVELOPE");
    }

    #[tokio::test]
    async fn ingress_invite_click_without_bearer_returns_401() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_invite_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(None, "req-7", "idem-7", now_ms, "nonce-7");
        let response = run_invite_click(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ingress_onboarding_continue_without_bearer_returns_401() {
        let state = test_state_with_config(IngressSecurityConfig::from_env());
        let request = base_onboarding_request();
        let now_ms = system_time_now_ms();
        let headers = security_headers(None, "req-8", "idem-8", now_ms, "nonce-8");
        let response = run_onboarding_continue(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn at_adapter_11_http_session_resume_route_reuses_soft_closed_session() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let actor_user_id = UserId::new("tenant_1:http_session_resume_actor").unwrap();
        let resumed_device_id = DeviceId::new("http_session_resume_device").unwrap();
        let session_id = SessionId(4_201);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_identity_and_device(&mut guard, &actor_user_id, &resumed_device_id);
            seed_soft_closed_session_record(
                &mut guard,
                session_id,
                &actor_user_id,
                &resumed_device_id,
                std::slice::from_ref(&resumed_device_id),
                &resumed_device_id,
                TurnId(77),
            );
        }

        let mut request = base_session_resume_request();
        request.session_id = session_id.0.to_string();
        request.device_id = resumed_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-resume-1",
            "idem-session-resume-1",
            now_ms,
            "nonce-session-resume-1",
        );
        let response = run_session_resume(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionResumeAdapterResponse = decode_json_response(response).await;
        let (expected_project_id, expected_pinned_context_refs) = session_project_context_fixture();
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_RESUMED");
        assert_eq!(body.session_id.as_deref(), Some("4201"));
        assert_eq!(body.session_state.as_deref(), Some("ACTIVE"));
        assert_eq!(
            body.session_attach_outcome.as_deref(),
            Some("EXISTING_SESSION_REUSED")
        );
        assert_eq!(
            body.project_id.as_deref(),
            Some(expected_project_id.as_str())
        );
        assert_eq!(
            body.pinned_context_refs.as_deref(),
            Some(expected_pinned_context_refs.as_slice())
        );

        let guard = store.lock().expect("store lock must succeed");
        let persisted = guard
            .get_session(&session_id)
            .expect("resumed session must remain persisted");
        assert_eq!(persisted.session_state, SessionState::Active);
        assert_eq!(persisted.last_attached_device_id, resumed_device_id);
    }

    #[tokio::test]
    async fn at_adapter_12_http_wake_profile_availability_route_reports_active_version() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let request = {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_wake_profile_availability_refresh_route_state(&mut guard, "http_route")
        };

        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(
                &request.expected_wake_profile_id,
                &request.device_id,
            )),
            "req-wake-profile-availability-1",
            &request.idempotency_key,
            now_ms,
            "nonce-wake-profile-availability-1",
        );
        let response =
            run_wake_profile_availability_refresh(State(state), headers, Json(request.clone()))
                .await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: WakeProfileAvailabilityRefreshAdapterResponse =
            decode_json_response(response).await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.device_id.as_deref(), Some(request.device_id.as_str()));
        assert_eq!(
            body.wake_profile_id.as_deref(),
            Some(request.expected_wake_profile_id.as_str())
        );
        assert_eq!(body.active_wake_artifact_version.as_deref(), Some("7"));
        assert_eq!(body.pull_error_count, 0);
    }

    #[tokio::test]
    async fn at_adapter_13_http_session_recover_route_recovers_suspended_session() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let actor_user_id = UserId::new("tenant_1:http_session_recover_actor").unwrap();
        let recovered_device_id = DeviceId::new("http_session_recover_device").unwrap();
        let session_id = SessionId(4_301);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_identity_and_device(&mut guard, &actor_user_id, &recovered_device_id);
            seed_suspended_session_record(
                &mut guard,
                session_id,
                &actor_user_id,
                &recovered_device_id,
                std::slice::from_ref(&recovered_device_id),
                &recovered_device_id,
                TurnId(81),
            );
        }

        let mut request = base_session_recover_request();
        request.session_id = session_id.0.to_string();
        request.device_id = recovered_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-recover-1",
            "idem-session-recover-1",
            now_ms,
            "nonce-session-recover-1",
        );
        let response = run_session_recover(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionRecoverAdapterResponse = decode_json_response(response).await;
        let (expected_project_id, expected_pinned_context_refs) = session_project_context_fixture();
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_RECOVERED");
        assert_eq!(body.session_id.as_deref(), Some("4301"));
        assert_eq!(body.session_state.as_deref(), Some("ACTIVE"));
        assert_eq!(
            body.session_attach_outcome.as_deref(),
            Some("EXISTING_SESSION_REUSED")
        );
        assert_eq!(
            body.project_id.as_deref(),
            Some(expected_project_id.as_str())
        );
        assert_eq!(
            body.pinned_context_refs.as_deref(),
            Some(expected_pinned_context_refs.as_slice())
        );

        let guard = store.lock().expect("store lock must succeed");
        let persisted = guard
            .get_session(&session_id)
            .expect("recovered session must remain persisted");
        assert_eq!(persisted.session_state, SessionState::Active);
        assert_eq!(persisted.last_attached_device_id, recovered_device_id);
    }

    #[tokio::test]
    async fn at_adapter_14_http_session_attach_route_attaches_visible_session() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let actor_user_id = UserId::new("tenant_1:http_session_attach_actor").unwrap();
        let attached_device_id = DeviceId::new("http_session_attach_device").unwrap();
        let existing_device_id = DeviceId::new("http_session_attach_existing_device").unwrap();
        let session_id = SessionId(4_101);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_identity_and_device(&mut guard, &actor_user_id, &attached_device_id);
            guard
                .insert_device(
                    DeviceRecord::v1(
                        existing_device_id.clone(),
                        actor_user_id.clone(),
                        "desktop".to_string(),
                        MonotonicTimeNs(2),
                        None,
                    )
                    .unwrap(),
                )
                .unwrap();
            seed_attachable_session_record(
                &mut guard,
                session_id,
                &actor_user_id,
                &existing_device_id,
                std::slice::from_ref(&existing_device_id),
                &existing_device_id,
                TurnId(76),
                SessionState::Active,
            );
        }

        let mut request = base_session_attach_request();
        request.session_id = session_id.0.to_string();
        request.device_id = attached_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-attach-1",
            "idem-session-attach-1",
            now_ms,
            "nonce-session-attach-1",
        );
        let response = run_session_attach(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionAttachAdapterResponse = decode_json_response(response).await;
        let (expected_project_id, expected_pinned_context_refs) = session_project_context_fixture();
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_ATTACHED");
        assert_eq!(body.session_id.as_deref(), Some("4101"));
        assert_eq!(body.session_state.as_deref(), Some("ACTIVE"));
        assert_eq!(
            body.session_attach_outcome.as_deref(),
            Some("EXISTING_SESSION_ATTACHED")
        );
        assert_eq!(
            body.project_id.as_deref(),
            Some(expected_project_id.as_str())
        );
        assert_eq!(
            body.pinned_context_refs.as_deref(),
            Some(expected_pinned_context_refs.as_slice())
        );

        let guard = store.lock().expect("store lock must succeed");
        let persisted = guard
            .get_session(&session_id)
            .expect("attached session must remain persisted");
        assert_eq!(persisted.session_state, SessionState::Active);
        assert_eq!(persisted.last_attached_device_id, attached_device_id);
    }

    #[tokio::test]
    async fn http_session_recent_route_returns_recent_nonclosed_sessions_for_last_attached_device()
    {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let current_device_id = DeviceId::new("http_session_recent_device").unwrap();
        let other_device_id = DeviceId::new("http_session_recent_other_device").unwrap();
        let active_user_id = UserId::new("tenant_1:http_session_recent_active_actor").unwrap();
        let soft_closed_user_id =
            UserId::new("tenant_1:http_session_recent_soft_closed_actor").unwrap();
        let suspended_user_id =
            UserId::new("tenant_1:http_session_recent_suspended_actor").unwrap();
        let closed_user_id = UserId::new("tenant_1:http_session_recent_closed_actor").unwrap();
        let other_device_user_id =
            UserId::new("tenant_1:http_session_recent_other_device_actor").unwrap();

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_920),
                &active_user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::Active,
                MonotonicTimeNs(120),
                Some(TurnId(920)),
            );
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_921),
                &soft_closed_user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::SoftClosed,
                MonotonicTimeNs(220),
                Some(TurnId(921)),
            );
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_922),
                &suspended_user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::Suspended,
                MonotonicTimeNs(180),
                Some(TurnId(922)),
            );
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_923),
                &closed_user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::Closed,
                MonotonicTimeNs(260),
                Some(TurnId(923)),
            );
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_924),
                &other_device_user_id,
                &current_device_id,
                &[current_device_id.clone(), other_device_id.clone()],
                &other_device_id,
                SessionState::Active,
                MonotonicTimeNs(320),
                Some(TurnId(924)),
            );
        }

        let mut request = base_session_recent_list_request();
        request.device_id = current_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.device_id, &request.device_id)),
            "req-session-recent-1",
            "idem-session-recent-1",
            now_ms,
            "nonce-session-recent-1",
        );
        let response = run_session_recent_list(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionRecentListAdapterResponse = decode_json_response(response).await;
        let (expected_project_id, expected_pinned_context_refs) = session_project_context_fixture();
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSIONS_LISTED");
        assert_eq!(body.sessions.len(), 3);
        assert_eq!(body.sessions[0].session_id, "4921");
        assert_eq!(body.sessions[1].session_id, "4922");
        assert_eq!(body.sessions[2].session_id, "4920");
        assert_eq!(body.sessions[0].session_state, "SOFT_CLOSED");
        assert_eq!(body.sessions[0].last_activity_at, 220);
        assert_eq!(body.sessions[0].last_turn_id.as_deref(), Some("921"));
        assert_eq!(
            body.sessions[0].project_id.as_deref(),
            Some(expected_project_id.as_str())
        );
        assert_eq!(
            body.sessions[0].pinned_context_refs.as_deref(),
            Some(expected_pinned_context_refs.as_slice())
        );
        assert!(body
            .sessions
            .iter()
            .all(|session| session.session_id != "4923"));
        assert!(body
            .sessions
            .iter()
            .all(|session| session.session_id != "4924"));
    }

    #[tokio::test]
    async fn http_session_posture_route_returns_current_device_fields_for_session() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let current_device_id = DeviceId::new("http_session_posture_device").unwrap();
        let other_device_id = DeviceId::new("http_session_posture_other_device").unwrap();
        let current_user_id = UserId::new("tenant_1:http_session_posture_actor").unwrap();
        let other_user_id = UserId::new("tenant_1:http_session_posture_other_actor").unwrap();
        let session_id = SessionId(4_930);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                session_id,
                &current_user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::SoftClosed,
                MonotonicTimeNs(330),
                Some(TurnId(930)),
            );
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                SessionId(4_931),
                &other_user_id,
                &current_device_id,
                &[current_device_id.clone(), other_device_id.clone()],
                &other_device_id,
                SessionState::Active,
                MonotonicTimeNs(331),
                Some(TurnId(931)),
            );
        }

        let mut request = base_session_posture_request();
        request.session_id = session_id.0.to_string();
        request.device_id = current_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-posture-1",
            "idem-session-posture-1",
            now_ms,
            "nonce-session-posture-1",
        );
        let response = run_session_posture_evidence(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionPostureEvidenceAdapterResponse = decode_json_response(response).await;
        let (expected_project_id, expected_pinned_context_refs) = session_project_context_fixture();
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_POSTURE_EVIDENCE_READ");
        assert_eq!(body.session_id.as_deref(), Some("4930"));
        assert_eq!(body.session_state.as_deref(), Some("SOFT_CLOSED"));
        assert_eq!(body.last_turn_id.as_deref(), Some("930"));
        assert_eq!(
            body.project_id.as_deref(),
            Some(expected_project_id.as_str())
        );
        assert_eq!(
            body.pinned_context_refs.as_deref(),
            Some(expected_pinned_context_refs.as_slice())
        );
        assert_eq!(body.session_attach_outcome, None);
        assert_eq!(body.selected_thread_id, None);
        assert_eq!(body.selected_thread_title, None);
        assert_eq!(body.pending_work_order_id, None);
        assert_eq!(body.resume_tier, None);
        assert_eq!(body.resume_summary_bullets, None);
        assert_eq!(body.archived_user_turn_text, None);
        assert_eq!(body.archived_selene_turn_text, None);
        assert_eq!(body.recovery_mode, None);
        assert_eq!(body.reconciliation_decision, None);
    }

    #[tokio::test]
    async fn http_session_posture_route_returns_resume_selection_fields_when_lawfully_available_for_current_device_session(
    ) {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let current_device_id =
            DeviceId::new("http_session_posture_resume_selection_device").unwrap();
        let user_id = UserId::new("tenant_1:http_session_posture_resume_selection_actor").unwrap();
        let session_id = SessionId(4_932);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                session_id,
                &user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::SoftClosed,
                MonotonicTimeNs(330),
                Some(TurnId(932)),
            );
            seed_session_resume_selection_evidence_for_posture(
                &mut guard,
                session_id,
                &user_id,
                &current_device_id,
                TurnId(932),
            );
        }

        let mut request = base_session_posture_request();
        request.session_id = session_id.0.to_string();
        request.device_id = current_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-posture-2",
            "idem-session-posture-2",
            now_ms,
            "nonce-session-posture-2",
        );
        let response = run_session_posture_evidence(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionPostureEvidenceAdapterResponse = decode_json_response(response).await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_POSTURE_EVIDENCE_READ");
        assert_eq!(body.session_id.as_deref(), Some("4932"));
        assert_eq!(
            body.selected_thread_id.as_deref(),
            Some("thread_resume_hot")
        );
        assert_eq!(
            body.selected_thread_title.as_deref(),
            Some("Japan ski trip")
        );
        assert_eq!(body.pending_work_order_id, None);
        assert_eq!(body.resume_tier.as_deref(), Some("HOT"));
        assert_eq!(
            body.resume_summary_bullets.as_deref(),
            Some(
                &[
                    "Flights shortlisted".to_string(),
                    "Need hotel confirmation".to_string(),
                ][..]
            )
        );
        assert_eq!(body.archived_user_turn_text, None);
        assert_eq!(body.archived_selene_turn_text, None);
    }

    #[tokio::test]
    async fn http_session_posture_route_returns_archived_recent_slice_fields_when_lawfully_available_for_current_device_session(
    ) {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let current_device_id =
            DeviceId::new("http_session_posture_archived_recent_slice_device").unwrap();
        let user_id =
            UserId::new("tenant_1:http_session_posture_archived_recent_slice_actor").unwrap();
        let session_id = SessionId(4_933);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                session_id,
                &user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::SoftClosed,
                MonotonicTimeNs(340),
                Some(TurnId(933)),
            );
            seed_session_archived_recent_slice_evidence_for_posture(
                &mut guard,
                session_id,
                &user_id,
                &current_device_id,
                TurnId(933),
            );
        }

        let mut request = base_session_posture_request();
        request.session_id = session_id.0.to_string();
        request.device_id = current_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-posture-3",
            "idem-session-posture-3",
            now_ms,
            "nonce-session-posture-3",
        );
        let response = run_session_posture_evidence(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionPostureEvidenceAdapterResponse = decode_json_response(response).await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_POSTURE_EVIDENCE_READ");
        assert_eq!(body.session_id.as_deref(), Some("4933"));
        assert_eq!(
            body.archived_user_turn_text.as_deref(),
            Some("show me the archived session slice")
        );
        assert_eq!(
            body.archived_selene_turn_text.as_deref(),
            Some("Here is the archived session slice summary.")
        );
    }

    #[tokio::test]
    async fn http_session_posture_route_keeps_archived_recent_slice_fields_absent_when_pair_is_not_lawfully_available(
    ) {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let current_device_id =
            DeviceId::new("http_session_posture_archived_recent_slice_absent_device").unwrap();
        let user_id =
            UserId::new("tenant_1:http_session_posture_archived_recent_slice_absent_actor")
                .unwrap();
        let session_id = SessionId(4_934);

        {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_recent_session_record_for_last_attached_device(
                &mut guard,
                session_id,
                &user_id,
                &current_device_id,
                std::slice::from_ref(&current_device_id),
                &current_device_id,
                SessionState::SoftClosed,
                MonotonicTimeNs(350),
                Some(TurnId(934)),
            );
            guard
                .append_conversation_turn(
                    selene_kernel_contracts::ph1f::ConversationTurnInput::v1(
                        MonotonicTimeNs(350),
                        selene_kernel_contracts::ph1j::CorrelationId(94_110),
                        TurnId(934),
                        Some(session_id),
                        user_id.clone(),
                        Some(current_device_id.clone()),
                        selene_kernel_contracts::ph1f::ConversationRole::User,
                        selene_kernel_contracts::ph1f::ConversationSource::TypedText,
                        "only the archived user side exists".to_string(),
                        "hash_http_archived_recent_slice_user_only".to_string(),
                        selene_kernel_contracts::ph1f::PrivacyScope::PublicChat,
                        Some(
                            "seed_http_session_posture_archived_recent_slice_user_only".to_string(),
                        ),
                        None,
                        None,
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let mut request = base_session_posture_request();
        request.session_id = session_id.0.to_string();
        request.device_id = current_device_id.as_str().to_string();
        let now_ms = system_time_now_ms();
        let headers = security_headers(
            Some(bearer_for(&request.session_id, &request.device_id)),
            "req-session-posture-4",
            "idem-session-posture-4",
            now_ms,
            "nonce-session-posture-4",
        );
        let response = run_session_posture_evidence(State(state), headers, Json(request)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: SessionPostureEvidenceAdapterResponse = decode_json_response(response).await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.outcome, "SESSION_POSTURE_EVIDENCE_READ");
        assert_eq!(body.session_id.as_deref(), Some("4934"));
        assert_eq!(body.archived_user_turn_text, None);
        assert_eq!(body.archived_selene_turn_text, None);
    }

    #[tokio::test]
    async fn ingress_iphone_invite_onboarding_and_explicit_voice_turn_e2e() {
        let (state, store) = test_state_with_config_and_store(IngressSecurityConfig::from_env());
        let inviter_user_id = UserId::new("tenant_1:iphone_e2e_inviter").unwrap();
        let inviter_device_id = DeviceId::new("iphone_e2e_inviter_device".to_string()).unwrap();
        let iphone_device_id = inviter_device_id.as_str().to_string();

        let (token_id, token_signature) = {
            let mut guard = store.lock().expect("store lock must succeed");
            seed_identity_and_device(&mut guard, &inviter_user_id, &inviter_device_id);
            seed_company_position_minimum(&mut guard);
            for (simulation_id, simulation_type) in [
                (LINK_INVITE_OPEN_ACTIVATE_COMMIT, SimulationType::Commit),
                (ONB_SESSION_START_DRAFT, SimulationType::Draft),
                (LINK_INVITE_DRAFT_UPDATE_COMMIT, SimulationType::Commit),
                (ONB_TERMS_ACCEPT_COMMIT, SimulationType::Commit),
                (ONB_PRIMARY_DEVICE_CONFIRM_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_START_DRAFT, SimulationType::Draft),
                (VOICE_ID_ENROLL_SAMPLE_COMMIT, SimulationType::Commit),
                (VOICE_ID_ENROLL_COMPLETE_COMMIT, SimulationType::Commit),
                (EMO_SIM_001, SimulationType::Commit),
                (ONB_ACCESS_INSTANCE_CREATE_COMMIT, SimulationType::Commit),
                (ONB_COMPLETE_COMMIT, SimulationType::Commit),
            ] {
                seed_simulation_catalog_status(
                    &mut guard,
                    "tenant_1",
                    simulation_id,
                    simulation_type,
                    SimulationStatus::Active,
                );
            }
            seed_invite_link_for_click(&mut guard, &inviter_user_id)
        };

        let invite_request = InviteLinkOpenAdapterRequest {
            correlation_id: 91_001,
            idempotency_key: "iphone-e2e-invite".to_string(),
            token_id: token_id.clone(),
            token_signature,
            tenant_id: Some("tenant_1".to_string()),
            app_platform: "IOS".to_string(),
            device_fingerprint: "iphone-e2e-fingerprint".to_string(),
            app_instance_id: iphone_device_id.clone(),
            deep_link_nonce: "iphone-e2e-deep-link".to_string(),
        };
        let now_ms = system_time_now_ms();
        let invite_headers = security_headers(
            Some(bearer_for(
                &invite_request.token_id,
                &invite_request.app_instance_id,
            )),
            "iphone-e2e-req-invite",
            "iphone-e2e-idem-invite",
            now_ms,
            "iphone-e2e-nonce-invite",
        );
        let invite_response =
            run_invite_click(State(state.clone()), invite_headers, Json(invite_request)).await;
        assert_eq!(invite_response.status(), StatusCode::OK);
        let invite_body: InviteLinkOpenAdapterResponse =
            decode_json_response(invite_response).await;
        assert_eq!(invite_body.status, "ok");
        assert_eq!(invite_body.outcome, "ONBOARDING_STARTED");
        let onboarding_session_id = invite_body
            .onboarding_session_id
            .expect("onboarding session id must be present");

        let onb_bearer = bearer_for(&onboarding_session_id, &iphone_device_id);
        let mut onb_step_request_counter = 0_u64;

        let ask_prompt_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_002,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-ask-prompt".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "ASK_MISSING_SUBMIT".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let ask_prompt_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-ask-prompt",
            "iphone-e2e-idem-ask-prompt",
            now_ms.saturating_add(1),
            "iphone-e2e-nonce-ask-prompt",
        );
        let ask_prompt_response = run_onboarding_continue(
            State(state.clone()),
            ask_prompt_headers,
            Json(ask_prompt_request),
        )
        .await;
        assert_eq!(ask_prompt_response.status(), StatusCode::OK);
        let mut ask_out: OnboardingContinueAdapterResponse =
            decode_json_response(ask_prompt_response).await;

        while ask_out.next_step.as_deref() == Some("ASK_MISSING") {
            let field_key = ask_out
                .blocking_field
                .clone()
                .expect("ASK_MISSING must include blocking_field");
            onb_step_request_counter = onb_step_request_counter.saturating_add(1);
            let request = OnboardingContinueAdapterRequest {
                correlation_id: 91_002 + onb_step_request_counter,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: format!("iphone-e2e-ask-value-{onb_step_request_counter}"),
                tenant_id: Some("tenant_1".to_string()),
                action: "ASK_MISSING_SUBMIT".to_string(),
                field_value: Some(ask_missing_value(field_key.as_str())),
                receipt_kind: None,
                receipt_ref: None,
                signer: None,
                payload_hash: None,
                terms_version_id: None,
                accepted: None,
                device_id: Some(iphone_device_id.clone()),
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            };
            let headers = security_headers(
                Some(onb_bearer.clone()),
                format!("iphone-e2e-req-ask-{onb_step_request_counter}").as_str(),
                format!("iphone-e2e-idem-ask-{onb_step_request_counter}").as_str(),
                now_ms.saturating_add(10 + onb_step_request_counter),
                format!("iphone-e2e-nonce-ask-{onb_step_request_counter}").as_str(),
            );
            let response =
                run_onboarding_continue(State(state.clone()), headers, Json(request)).await;
            assert_eq!(response.status(), StatusCode::OK);
            ask_out = decode_json_response(response).await;
        }

        assert_eq!(ask_out.next_step.as_deref(), Some("PLATFORM_SETUP"));
        assert!(
            ask_out
                .remaining_platform_receipt_kinds
                .iter()
                .any(|kind| kind == "ios_side_button_configured"),
            "iOS onboarding must require ios_side_button_configured"
        );

        let required_receipts = ask_out.remaining_platform_receipt_kinds.clone();
        let mut platform_out = ask_out;
        for (idx, receipt_kind) in required_receipts.iter().enumerate() {
            let request = OnboardingContinueAdapterRequest {
                correlation_id: 91_100 + idx as u64,
                onboarding_session_id: onboarding_session_id.clone(),
                idempotency_key: format!("iphone-e2e-platform-{idx}"),
                tenant_id: Some("tenant_1".to_string()),
                action: "PLATFORM_SETUP_RECEIPT".to_string(),
                field_value: None,
                receipt_kind: Some(receipt_kind.clone()),
                receipt_ref: Some(format!("receipt:iphone-e2e:{receipt_kind}")),
                signer: Some("selene_mobile_app".to_string()),
                payload_hash: Some(format!("{:064x}", idx + 1)),
                terms_version_id: None,
                accepted: None,
                device_id: Some(iphone_device_id.clone()),
                proof_ok: None,
                sample_seed: None,
                photo_blob_ref: None,
                sender_decision: None,
            };
            let headers = security_headers(
                Some(onb_bearer.clone()),
                format!("iphone-e2e-req-platform-{idx}").as_str(),
                format!("iphone-e2e-idem-platform-{idx}").as_str(),
                now_ms.saturating_add(100 + idx as u64),
                format!("iphone-e2e-nonce-platform-{idx}").as_str(),
            );
            let response =
                run_onboarding_continue(State(state.clone()), headers, Json(request)).await;
            assert_eq!(response.status(), StatusCode::OK);
            platform_out = decode_json_response(response).await;
        }
        assert_eq!(platform_out.next_step.as_deref(), Some("TERMS"));

        let terms_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_201,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-terms".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "TERMS_ACCEPT".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: Some("terms_v1".to_string()),
            accepted: Some(true),
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let terms_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-terms",
            "iphone-e2e-idem-terms",
            now_ms.saturating_add(200),
            "iphone-e2e-nonce-terms",
        );
        let terms_response =
            run_onboarding_continue(State(state.clone()), terms_headers, Json(terms_request)).await;
        assert_eq!(terms_response.status(), StatusCode::OK);
        let terms_out: OnboardingContinueAdapterResponse =
            decode_json_response(terms_response).await;
        assert_eq!(
            terms_out.next_step.as_deref(),
            Some("PRIMARY_DEVICE_CONFIRM")
        );

        let device_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_202,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-device".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "PRIMARY_DEVICE_CONFIRM".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: Some(true),
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let device_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-device",
            "iphone-e2e-idem-device",
            now_ms.saturating_add(201),
            "iphone-e2e-nonce-device",
        );
        let device_response =
            run_onboarding_continue(State(state.clone()), device_headers, Json(device_request))
                .await;
        assert_eq!(device_response.status(), StatusCode::OK);
        let device_out: OnboardingContinueAdapterResponse =
            decode_json_response(device_response).await;
        assert_eq!(device_out.next_step.as_deref(), Some("VOICE_ENROLL"));

        let voice_enroll_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_203,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-voice-enroll".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "VOICE_ENROLL_LOCK".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: Some("iphone_e2e_seed".to_string()),
            photo_blob_ref: None,
            sender_decision: None,
        };
        let voice_enroll_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-voice-enroll",
            "iphone-e2e-idem-voice-enroll",
            now_ms.saturating_add(202),
            "iphone-e2e-nonce-voice-enroll",
        );
        let voice_enroll_response = run_onboarding_continue(
            State(state.clone()),
            voice_enroll_headers,
            Json(voice_enroll_request),
        )
        .await;
        assert_eq!(voice_enroll_response.status(), StatusCode::OK);
        let voice_enroll_out: OnboardingContinueAdapterResponse =
            decode_json_response(voice_enroll_response).await;
        assert_eq!(
            voice_enroll_out.next_step.as_deref(),
            Some("EMO_PERSONA_LOCK")
        );

        let emo_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_204,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-emo".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "EMO_PERSONA_LOCK".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let emo_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-emo",
            "iphone-e2e-idem-emo",
            now_ms.saturating_add(203),
            "iphone-e2e-nonce-emo",
        );
        let emo_response =
            run_onboarding_continue(State(state.clone()), emo_headers, Json(emo_request)).await;
        assert_eq!(emo_response.status(), StatusCode::OK);
        let emo_out: OnboardingContinueAdapterResponse = decode_json_response(emo_response).await;
        assert_eq!(emo_out.next_step.as_deref(), Some("ACCESS_PROVISION"));

        let access_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_205,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-access".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "ACCESS_PROVISION_COMMIT".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let access_headers = security_headers(
            Some(onb_bearer.clone()),
            "iphone-e2e-req-access",
            "iphone-e2e-idem-access",
            now_ms.saturating_add(204),
            "iphone-e2e-nonce-access",
        );
        let access_response =
            run_onboarding_continue(State(state.clone()), access_headers, Json(access_request))
                .await;
        assert_eq!(access_response.status(), StatusCode::OK);
        let access_out: OnboardingContinueAdapterResponse =
            decode_json_response(access_response).await;
        assert_eq!(access_out.next_step.as_deref(), Some("COMPLETE"));

        let complete_request = OnboardingContinueAdapterRequest {
            correlation_id: 91_206,
            onboarding_session_id: onboarding_session_id.clone(),
            idempotency_key: "iphone-e2e-complete".to_string(),
            tenant_id: Some("tenant_1".to_string()),
            action: "COMPLETE_COMMIT".to_string(),
            field_value: None,
            receipt_kind: None,
            receipt_ref: None,
            signer: None,
            payload_hash: None,
            terms_version_id: None,
            accepted: None,
            device_id: Some(iphone_device_id.clone()),
            proof_ok: None,
            sample_seed: None,
            photo_blob_ref: None,
            sender_decision: None,
        };
        let complete_headers = security_headers(
            Some(onb_bearer),
            "iphone-e2e-req-complete",
            "iphone-e2e-idem-complete",
            now_ms.saturating_add(205),
            "iphone-e2e-nonce-complete",
        );
        let complete_response = run_onboarding_continue(
            State(state.clone()),
            complete_headers,
            Json(complete_request),
        )
        .await;
        assert_eq!(complete_response.status(), StatusCode::OK);
        let complete_out: OnboardingContinueAdapterResponse =
            decode_json_response(complete_response).await;
        assert_eq!(complete_out.next_step.as_deref(), Some("READY"));
        assert_eq!(complete_out.onboarding_status.as_deref(), Some("COMPLETE"));

        let mut voice_request = ios_voice_request(
            inviter_user_id.as_str().to_string(),
            iphone_device_id.clone(),
        );
        voice_request.correlation_id = 91_300;
        voice_request.turn_id = 91_301;
        voice_request.now_ns = Some(7_000_000_000);
        let voice_headers = security_headers(
            Some(bearer_for(&voice_request.actor_user_id, &iphone_device_id)),
            "iphone-e2e-req-voice",
            "iphone-e2e-idem-voice",
            now_ms.saturating_add(300),
            "iphone-e2e-nonce-voice",
        );
        let voice_response =
            run_voice_turn(State(state.clone()), voice_headers, Json(voice_request)).await;
        let voice_status = voice_response.status();
        let voice_body: VoiceTurnAdapterResponse = decode_json_response(voice_response).await;
        let voice_packet_debug = if voice_status == StatusCode::OK {
            None
        } else {
            state
                .runtime
                .lock()
                .ok()
                .and_then(|runtime| runtime.debug_last_agent_packet_voice_identity_assertion())
                .and_then(|value| if value.is_empty() { None } else { Some(value) })
        };
        assert_eq!(
            voice_status,
            StatusCode::OK,
            "voice turn failed with reason_code={} reason={:?} failure_class={:?} voice_packet_debug={:?}",
            voice_body.reason_code,
            voice_body.reason,
            voice_body.failure_class,
            voice_packet_debug
        );
        assert_eq!(voice_body.status, "ok");

        let actor_user = inviter_user_id.clone();
        let actor_device = DeviceId::new(iphone_device_id.clone()).unwrap();
        let has_open_session = {
            let guard = store.lock().expect("store lock must succeed");
            guard.session_rows().values().any(|row| {
                row.user_id == actor_user
                    && row.device_id == actor_device
                    && row.session_state != SessionState::Closed
            })
        };
        assert!(
            has_open_session,
            "iOS EXPLICIT voice turn should open/resume a non-closed session"
        );
    }
}
