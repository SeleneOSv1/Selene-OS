#![forbid(unsafe_code)]

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::grpc_api::{
    voice_ingress_server::{VoiceIngress, VoiceIngressServer},
    RunVoiceTurnAudioCaptureRef, RunVoiceTurnRequest, RunVoiceTurnResponse,
    RunVoiceTurnVisualInputRef, RunVoiceTurnVisualTokenRef, UiHealthReportPaging,
    UiHealthReportQueryRequest, UiHealthReportQueryResponse, UiHealthReportRow,
};
use selene_adapter::{
    AdapterRuntime, UiHealthReportQueryRequest as AdapterUiHealthReportQueryRequest,
    UiHealthReportQueryResponse as AdapterUiHealthReportQueryResponse, VoiceTurnAdapterRequest,
    VoiceTurnAudioCaptureRef, VoiceTurnVisualInputRef, VoiceTurnVisualTokenRef,
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
        let adapter_request = map_run_voice_turn_request(request.into_inner());

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

fn map_audio_capture_ref(
    capture: Option<RunVoiceTurnAudioCaptureRef>,
) -> Option<VoiceTurnAudioCaptureRef> {
    capture.map(|capture| VoiceTurnAudioCaptureRef {
        stream_id: ((capture.stream_id_hi as u128) << 64) | capture.stream_id_lo as u128,
        pre_roll_buffer_id: capture.pre_roll_buffer_id,
        t_start_ns: capture.t_start_ns,
        t_end_ns: capture.t_end_ns,
        t_candidate_start_ns: capture.t_candidate_start_ns,
        t_confirmed_ns: capture.t_confirmed_ns,
        locale_tag: capture.locale_tag,
        device_route: capture.device_route,
        selected_mic: capture.selected_mic,
        selected_speaker: capture.selected_speaker,
        tts_playback_active: capture.tts_playback_active,
        detection_text: capture.detection_text,
        detection_confidence_bp: capture
            .detection_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        vad_confidence_bp: capture
            .vad_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        acoustic_confidence_bp: capture
            .acoustic_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        prosody_confidence_bp: capture
            .prosody_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        speech_likeness_bp: capture
            .speech_likeness_bp
            .and_then(|v| u16::try_from(v).ok()),
        echo_safe_confidence_bp: capture
            .echo_safe_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        nearfield_confidence_bp: capture
            .nearfield_confidence_bp
            .and_then(|v| u16::try_from(v).ok()),
        capture_degraded: capture.capture_degraded,
        stream_gap_detected: capture.stream_gap_detected,
        aec_unstable: capture.aec_unstable,
        device_changed: capture.device_changed,
        snr_db_milli: capture.snr_db_milli,
        clipping_ratio_bp: capture
            .clipping_ratio_bp
            .and_then(|v| u16::try_from(v).ok()),
        echo_delay_ms_milli: capture.echo_delay_ms_milli,
        packet_loss_bp: capture.packet_loss_bp.and_then(|v| u16::try_from(v).ok()),
        double_talk_bp: capture.double_talk_bp.and_then(|v| u16::try_from(v).ok()),
        erle_db_milli: capture.erle_db_milli,
        device_failures_24h: capture.device_failures_24h,
        device_recoveries_24h: capture.device_recoveries_24h,
        device_mean_recovery_ms: capture.device_mean_recovery_ms,
        device_reliability_bp: capture
            .device_reliability_bp
            .and_then(|v| u16::try_from(v).ok()),
        timing_jitter_ms_milli: capture.timing_jitter_ms_milli,
        timing_drift_ppm_milli: capture.timing_drift_ppm_milli,
        timing_buffer_depth_ms_milli: capture.timing_buffer_depth_ms_milli,
        timing_underruns: capture.timing_underruns,
        timing_overruns: capture.timing_overruns,
    })
}

fn map_visual_token_ref(token: RunVoiceTurnVisualTokenRef) -> VoiceTurnVisualTokenRef {
    VoiceTurnVisualTokenRef {
        token: token.token,
        x: token.x,
        y: token.y,
        w: token.w,
        h: token.h,
    }
}

fn map_visual_input_ref(visual: Option<RunVoiceTurnVisualInputRef>) -> Option<VoiceTurnVisualInputRef> {
    visual.map(|visual| VoiceTurnVisualInputRef {
        turn_opt_in_enabled: visual.turn_opt_in_enabled.unwrap_or(false),
        source_id: optional_string(visual.source_id),
        source_kind: optional_string(visual.source_kind),
        image_ref: optional_string(visual.image_ref),
        blob_ref: optional_string(visual.blob_ref),
        visible_tokens: visual
            .visible_tokens
            .into_iter()
            .map(map_visual_token_ref)
            .collect(),
    })
}

fn map_run_voice_turn_request(req: RunVoiceTurnRequest) -> VoiceTurnAdapterRequest {
    VoiceTurnAdapterRequest {
        correlation_id: req.correlation_id,
        turn_id: req.turn_id,
        app_platform: req.app_platform,
        trigger: req.trigger,
        actor_user_id: req.actor_user_id,
        tenant_id: optional_string(req.tenant_id),
        device_id: optional_string(req.device_id),
        now_ns: non_zero_u64(req.now_ns),
        user_text_partial: None,
        user_text_final: None,
        selene_text_partial: None,
        selene_text_final: None,
        audio_capture_ref: map_audio_capture_ref(req.audio_capture_ref),
        visual_input_ref: map_visual_input_ref(req.visual_input_ref),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use std::time::{Duration, Instant};

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<String>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => {
                    env::set_var(self.key, value);
                }
                None => {
                    env::remove_var(self.key);
                }
            }
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn stt_mock_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        let handle = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                        let mut req_buf = [0_u8; 4096];
                        let bytes = stream.read(&mut req_buf).unwrap_or(0);
                        let req_text = String::from_utf8_lossy(&req_buf[..bytes]);
                        let response_json = if req_text.contains("\"task\":\"speech:recognize\"") {
                            r#"{"task":"speech:recognize","transcript":"hello from grpc live","lang":"en-US","confidence_bp":9500,"is_final":true}"#
                        } else {
                            r#"{"task":"stt.transcribe","text":"hello from grpc live","language":"en-US","confidence_bp":9500,"stable":true}"#
                        };
                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nx-request-id: grpc_test_req_1\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            response_json.len(),
                            response_json
                        );
                        stream
                            .write_all(response.as_bytes())
                            .expect("write response");
                        let _ = stream.flush();
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        (format!("http://{addr}/stt"), handle)
    }

    #[test]
    fn at_grpc_01_run_voice_turn_audio_capture_maps_to_adapter_contract() {
        let req = RunVoiceTurnRequest {
            correlation_id: 42,
            turn_id: 7,
            app_platform: "IOS".to_string(),
            trigger: "EXPLICIT".to_string(),
            actor_user_id: "tenant_a:user_1".to_string(),
            tenant_id: "tenant_a".to_string(),
            device_id: "device_ios_1".to_string(),
            now_ns: 99,
            audio_capture_ref: Some(RunVoiceTurnAudioCaptureRef {
                stream_id_hi: 0x1122_3344_5566_7788,
                stream_id_lo: 0x99AA_BBCC_DDEE_FF00,
                pre_roll_buffer_id: 9,
                t_start_ns: 10,
                t_end_ns: 20,
                t_candidate_start_ns: 12,
                t_confirmed_ns: 19,
                locale_tag: Some("en-US".to_string()),
                device_route: Some("BUILT_IN".to_string()),
                selected_mic: Some("mic_ios_1".to_string()),
                selected_speaker: Some("spk_ios_1".to_string()),
                tts_playback_active: Some(true),
                detection_text: Some("stop".to_string()),
                detection_confidence_bp: Some(9_700),
                vad_confidence_bp: Some(9_500),
                acoustic_confidence_bp: Some(9_400),
                prosody_confidence_bp: Some(9_300),
                speech_likeness_bp: Some(9_600),
                echo_safe_confidence_bp: Some(9_200),
                nearfield_confidence_bp: Some(9_000),
                capture_degraded: Some(false),
                stream_gap_detected: Some(false),
                aec_unstable: Some(false),
                device_changed: Some(false),
                snr_db_milli: Some(24_000),
                clipping_ratio_bp: Some(120),
                echo_delay_ms_milli: Some(32_000),
                packet_loss_bp: Some(35),
                double_talk_bp: Some(800),
                erle_db_milli: Some(18_000),
                device_failures_24h: Some(0),
                device_recoveries_24h: Some(0),
                device_mean_recovery_ms: Some(100),
                device_reliability_bp: Some(9_800),
                timing_jitter_ms_milli: Some(8_000),
                timing_drift_ppm_milli: Some(4_000),
                timing_buffer_depth_ms_milli: Some(42_000),
                timing_underruns: Some(0),
                timing_overruns: Some(0),
            }),
            visual_input_ref: Some(RunVoiceTurnVisualInputRef {
                turn_opt_in_enabled: Some(true),
                source_id: "vision_src_1".to_string(),
                source_kind: "IMAGE".to_string(),
                image_ref: "image://capture_001".to_string(),
                blob_ref: "blob://capture/blob_001".to_string(),
                visible_tokens: vec![RunVoiceTurnVisualTokenRef {
                    token: "Invoice".to_string(),
                    x: Some(1),
                    y: Some(2),
                    w: Some(10),
                    h: Some(4),
                }],
            }),
        };

        let mapped = map_run_voice_turn_request(req);
        let capture = mapped
            .audio_capture_ref
            .expect("audio capture ref must be mapped");
        assert_eq!(
            capture.stream_id,
            (0x1122_3344_5566_7788_u128 << 64) | 0x99AA_BBCC_DDEE_FF00_u128
        );
        assert_eq!(capture.pre_roll_buffer_id, 9);
        assert_eq!(capture.t_start_ns, 10);
        assert_eq!(capture.t_end_ns, 20);
        assert_eq!(capture.t_candidate_start_ns, 12);
        assert_eq!(capture.t_confirmed_ns, 19);
        assert_eq!(capture.locale_tag.as_deref(), Some("en-US"));
        assert_eq!(capture.device_route.as_deref(), Some("BUILT_IN"));
        assert_eq!(capture.vad_confidence_bp, Some(9_500));
        assert_eq!(capture.packet_loss_bp, Some(35));
        let visual = mapped
            .visual_input_ref
            .expect("visual input ref must be mapped");
        assert!(visual.turn_opt_in_enabled);
        assert_eq!(visual.source_id.as_deref(), Some("vision_src_1"));
        assert_eq!(visual.source_kind.as_deref(), Some("IMAGE"));
        assert_eq!(visual.image_ref.as_deref(), Some("image://capture_001"));
        assert_eq!(visual.blob_ref.as_deref(), Some("blob://capture/blob_001"));
        assert_eq!(visual.visible_tokens.len(), 1);
    }

    #[tokio::test]
    async fn at_grpc_02_live_stt_flow_accepts_audio_capture_ref_and_writes_transcript() {
        let _guard = env_lock().lock().expect("env lock");
        let (mock_stt_endpoint, server_handle) = stt_mock_server();
        let env_vars = [
            ScopedEnvVar::set("SELENE_PH1D_LIVE_ADAPTER_ENABLED", "1"),
            ScopedEnvVar::set("SELENE_PH1C_LIVE_ENABLED", "1"),
            ScopedEnvVar::set("SELENE_PH1C_STREAMING_ENABLED", "0"),
            ScopedEnvVar::set("PH1D_OPENAI_API_KEY", "grpc_test_openai_key"),
            ScopedEnvVar::set("PH1D_GOOGLE_API_KEY", "grpc_test_google_key"),
            ScopedEnvVar::set("PH1D_OPENAI_STT_ENDPOINT", &mock_stt_endpoint),
            ScopedEnvVar::set("PH1D_GOOGLE_STT_ENDPOINT", &mock_stt_endpoint),
        ];

        let service = GrpcVoiceIngress {
            runtime: Arc::new(Mutex::new(AdapterRuntime::default())),
        };
        let request = RunVoiceTurnRequest {
            correlation_id: 9_001,
            turn_id: 9_101,
            app_platform: "IOS".to_string(),
            trigger: "EXPLICIT".to_string(),
            actor_user_id: "tenant_a:user_grpc_live".to_string(),
            tenant_id: "tenant_a".to_string(),
            device_id: "device_ios_grpc_live".to_string(),
            now_ns: 5,
            audio_capture_ref: Some(RunVoiceTurnAudioCaptureRef {
                stream_id_hi: 0,
                stream_id_lo: 777,
                pre_roll_buffer_id: 1,
                t_start_ns: 1,
                t_end_ns: 5,
                t_candidate_start_ns: 2,
                t_confirmed_ns: 4,
                locale_tag: Some("en-US".to_string()),
                device_route: Some("BUILT_IN".to_string()),
                selected_mic: Some("mic_ios_grpc_live".to_string()),
                selected_speaker: Some("spk_ios_grpc_live".to_string()),
                tts_playback_active: Some(true),
                detection_text: Some("stop".to_string()),
                detection_confidence_bp: Some(9_600),
                vad_confidence_bp: Some(9_400),
                acoustic_confidence_bp: Some(9_300),
                prosody_confidence_bp: Some(9_200),
                speech_likeness_bp: Some(9_300),
                echo_safe_confidence_bp: Some(9_100),
                nearfield_confidence_bp: Some(8_900),
                capture_degraded: Some(false),
                stream_gap_detected: Some(false),
                aec_unstable: Some(false),
                device_changed: Some(false),
                snr_db_milli: Some(21_500),
                clipping_ratio_bp: Some(80),
                echo_delay_ms_milli: Some(28_000),
                packet_loss_bp: Some(20),
                double_talk_bp: Some(400),
                erle_db_milli: Some(20_500),
                device_failures_24h: Some(0),
                device_recoveries_24h: Some(0),
                device_mean_recovery_ms: Some(90),
                device_reliability_bp: Some(9_900),
                timing_jitter_ms_milli: Some(7_000),
                timing_drift_ppm_milli: Some(3_000),
                timing_buffer_depth_ms_milli: Some(35_000),
                timing_underruns: Some(0),
                timing_overruns: Some(0),
            }),
            visual_input_ref: None,
        };

        let out = service
            .run_voice_turn(Request::new(request))
            .await
            .expect("grpc run_voice_turn should succeed")
            .into_inner();
        assert_eq!(out.status, "ok");
        assert_eq!(out.outcome, "FORWARDED");

        let transcript = service
            .runtime
            .lock()
            .expect("runtime lock")
            .ui_chat_transcript_report(Some(6));
        assert!(transcript.messages.iter().any(|message| {
            message.role == "USER"
                && message.source == "PH1.C"
                && message.finalized
                && message.text == "hello from grpc live"
        }));

        drop(env_vars);
        server_handle.join().expect("mock server join");
    }
}
