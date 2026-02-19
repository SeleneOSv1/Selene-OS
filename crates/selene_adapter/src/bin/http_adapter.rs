#![forbid(unsafe_code)]

use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use selene_adapter::{
    AdapterHealthResponse, AdapterRuntime, AdapterSyncHealth, VoiceTurnAdapterRequest,
    VoiceTurnAdapterResponse,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = env::var("SELENE_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = bind.parse()?;
    let sync_worker_enabled = parse_sync_worker_enabled_from_env();
    let sync_worker_interval_ms = parse_sync_worker_interval_ms_from_env();

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
        .route("/healthz", get(healthz))
        .route("/v1/voice/turn", post(run_voice_turn))
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
            }),
        ),
    }
}
