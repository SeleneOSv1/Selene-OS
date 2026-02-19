#![forbid(unsafe_code)]

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::grpc_api::{
    voice_ingress_server::{VoiceIngress, VoiceIngressServer},
    RunVoiceTurnRequest, RunVoiceTurnResponse,
};
use selene_adapter::{AdapterRuntime, VoiceTurnAdapterRequest};
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
            })),
            Err(reason) => Err(Status::invalid_argument(reason)),
        }
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
