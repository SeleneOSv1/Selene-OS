#![forbid(unsafe_code)]

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::desktop_mic_producer::{DesktopMicProducer, DesktopMicProducerConfig};
use selene_adapter::{AdapterRuntime, VoiceTurnAdapterRequest};
use selene_engines::ph1w::reason_codes as ph1w_reason_codes;
use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::MonotonicTimeNs;
use selene_os::app_ingress::AppServerIngressRuntime;
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, WakeSampleResult,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli_args();

    println!(
        "usage: cargo run -p selene_adapter --bin desktop_wake_life -- [--device <substring>] [--seconds <n>]"
    );

    let actor_user_id = UserId::new("tenant_1:desktop_life_actor".to_string())
        .map_err(|err| format!("invalid actor_user_id: {err:?}"))?;
    let device_id = DeviceId::new("desktop_life_device_1".to_string())
        .map_err(|err| format!("invalid device_id: {err:?}"))?;

    let mut store = Ph1fStore::new_in_memory();
    seed_identity_and_device(&mut store, &actor_user_id, &device_id)?;
    seed_wake_profile(&mut store, &actor_user_id, &device_id)?;

    let store = Arc::new(Mutex::new(store));
    let runtime = AdapterRuntime::new(AppServerIngressRuntime::default(), store.clone());

    let mut config = DesktopMicProducerConfig::default();
    config.input_device_name_substring = cli.preferred_device_substring;
    let producer = DesktopMicProducer::start(config)?;
    let sample_rate_hz = producer.source_sample_rate_hz()?;
    let channels = producer.source_channels()?;
    let wake_loop = resolve_desktop_wake_loop_config();
    println!(
        "wake loop config: window_ms={} hop_ms={} max_steps={} (override: SELENE_PH1W_LIVE_WINDOW_MS/SELENE_PH1W_LIVE_HOP_MS/SELENE_PH1W_LIVE_MAX_STEPS)",
        wake_loop.window_ms, wake_loop.hop_ms, wake_loop.max_steps
    );
    producer.wait_until_pre_roll_ready(Duration::from_secs(8))?;
    if cli.capture_seconds > 0 {
        println!(
            "capture window active: {}s (speak wake phrase during this window)",
            cli.capture_seconds
        );
        std::thread::sleep(Duration::from_secs(cli.capture_seconds));
    }

    let capture_ref = producer.build_capture_ref()?;
    let now_ns = capture_ref.t_end_ns.max(1);
    let selected_mic = capture_ref
        .selected_mic
        .clone()
        .unwrap_or_else(|| "unknown_mic".to_string());
    let pre_roll_ms = capture_ref
        .t_end_ns
        .saturating_sub(capture_ref.t_start_ns)
        .saturating_div(1_000_000);
    println!(
        "mic selected: {} sample_rate_hz={} channels={}",
        selected_mic, sample_rate_hz, channels
    );
    println!("pre-roll ready: {}ms (>=1200ms required)", pre_roll_ms);
    println!(
        "capture metrics: vad_confidence_bp={} clipping_ratio_bp={} timing_jitter_ms_milli={} snr_db_milli={}",
        capture_ref.vad_confidence_bp.unwrap_or(0),
        capture_ref.clipping_ratio_bp.unwrap_or(0),
        capture_ref.timing_jitter_ms_milli.unwrap_or(0),
        capture_ref.snr_db_milli.unwrap_or(0)
    );
    println!(
        "capture timing: drift_ppm_milli={} underruns={} overruns={} buffer_depth_ms_milli={}",
        capture_ref.timing_drift_ppm_milli.unwrap_or(0),
        capture_ref.timing_underruns.unwrap_or(0),
        capture_ref.timing_overruns.unwrap_or(0),
        capture_ref.timing_buffer_depth_ms_milli.unwrap_or(0)
    );
    println!(
        "capture scoring: acoustic_confidence_bp={} prosody_confidence_bp={} speech_likeness_bp={} detection_present={}",
        capture_ref.acoustic_confidence_bp.unwrap_or(0),
        capture_ref.prosody_confidence_bp.unwrap_or(0),
        capture_ref.speech_likeness_bp.unwrap_or(0),
        capture_ref
            .detection_text
            .as_deref()
            .is_some_and(|v| !v.trim().is_empty())
    );

    let request = VoiceTurnAdapterRequest {
        correlation_id: 88_001,
        turn_id: 88_001,
        app_platform: "DESKTOP".to_string(),
        trigger: "WAKE_WORD".to_string(),
        actor_user_id: actor_user_id.as_str().to_string(),
        tenant_id: Some("tenant_1".to_string()),
        device_id: Some(device_id.as_str().to_string()),
        now_ns: Some(now_ns),
        thread_key: None,
        project_id: None,
        pinned_context_refs: None,
        thread_policy_flags: None,
        user_text_partial: None,
        user_text_final: None,
        selene_text_partial: None,
        selene_text_final: None,
        audio_capture_ref: Some(capture_ref),
        visual_input_ref: None,
    };

    let run_result = runtime.run_voice_turn(request);

    let (wake_summary, session_summary) = {
        let guard = store
            .lock()
            .map_err(|_| "store lock poisoned while building life-test summary")?;

        let wake_summary = guard
            .ph1w_get_runtime_events()
            .last()
            .map(|event| {
                let reason_name = wake_reason_name(event.reason_code.0);
                format!(
                    "{} reason_code={} ({}) model_version={}",
                    if event.accepted { "accept" } else { "reject" },
                    event.reason_code.0,
                    reason_name,
                    event
                        .model_version
                        .as_deref()
                        .unwrap_or("unknown_model")
                )
            })
            .unwrap_or_else(|| "reject reason_code=NO_WAKE_RUNTIME_EVENT".to_string());

        let session_summary = guard
            .session_rows()
            .values()
            .filter(|row| row.user_id == actor_user_id && row.device_id == device_id)
            .max_by_key(|row| row.last_activity_at.0)
            .map(|row| {
                format!(
                    "session_id={} state={:?}",
                    row.session_id.0,
                    row.session_state
                )
            })
            .unwrap_or_else(|| "session_id=NONE state=Closed".to_string());

        (wake_summary, session_summary)
    };

    println!("wake decision: {wake_summary}");
    println!("session summary: {session_summary}");

    match run_result {
        Ok(response) => {
            println!(
                "runtime summary: status={} outcome={} next_move={} reason_code={}",
                response.status, response.outcome, response.next_move, response.reason_code
            );
        }
        Err(err) => {
            println!("runtime summary: error={err}");
        }
    }

    producer.stop();
    Ok(())
}

fn wake_reason_name(reason_code: u32) -> &'static str {
    match reason_code {
        c if c == ph1w_reason_codes::W_WAKE_ACCEPTED.0 => "ACCEPTED",
        c if c == ph1w_reason_codes::W_WAKE_REJECTED_TIMEOUT.0 => "TIMEOUT",
        c if c == ph1w_reason_codes::W_FAIL_G1_NOISE.0 => "NOISE",
        c if c == ph1w_reason_codes::W_FAIL_G1A_NOT_UTTERANCE_START.0 => "NO_SPEECH",
        c if c == ph1w_reason_codes::W_FAIL_G2_NOT_WAKE_LIKE.0 => "NOT_WAKE_LIKE",
        c if c == ph1w_reason_codes::W_FAIL_G3_SCORE_LOW.0 => "SCORE_LOW",
        c if c == ph1w_reason_codes::W_FAIL_G3_UNSTABLE_SCORE.0 => "UNSTABLE_SCORE",
        c if c == ph1w_reason_codes::W_FAIL_G3_ALIGNMENT.0 => "ALIGNMENT",
        c if c == ph1w_reason_codes::W_FAIL_G3A_REPLAY_SUSPECTED.0 => "REPLAY_SUSPECTED",
        c if c == ph1w_reason_codes::W_FAIL_G4_USER_MISMATCH.0 => "USER_MISMATCH",
        c if c == ph1w_reason_codes::W_FAIL_G5_POLICY_BLOCKED.0 => "POLICY_BLOCKED",
        _ => "UNKNOWN",
    }
}

#[derive(Debug, Clone, Copy)]
struct WakeLoopConfig {
    window_ms: u32,
    hop_ms: u32,
    max_steps: u64,
}

fn resolve_desktop_wake_loop_config() -> WakeLoopConfig {
    let window_ms = parse_u32_env("SELENE_PH1W_LIVE_WINDOW_MS", 200, 10_000).unwrap_or(1_500);
    let hop_ms = parse_u32_env("SELENE_PH1W_LIVE_HOP_MS", 20, 2_000)
        .unwrap_or(200)
        .min(window_ms.max(20));
    let derived_steps = ((window_ms as u64)
        .saturating_add(hop_ms as u64)
        .saturating_sub(1))
    .saturating_div(hop_ms as u64)
    .saturating_add(1)
    .max(2);
    let max_steps = parse_u32_env("SELENE_PH1W_LIVE_MAX_STEPS", 2, 512)
        .map(u64::from)
        .unwrap_or(derived_steps);
    WakeLoopConfig {
        window_ms,
        hop_ms,
        max_steps,
    }
}

fn parse_u32_env(key: &str, min: u32, max: u32) -> Option<u32> {
    env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .map(|v| v.clamp(min, max))
}

#[derive(Debug, Clone)]
struct CliArgs {
    preferred_device_substring: Option<String>,
    capture_seconds: u64,
}

fn parse_cli_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();
    let mut preferred_device_substring = None;
    let mut capture_seconds = 0_u64;
    let mut idx = 1;
    while idx < args.len() {
        if args[idx] == "--device" && idx + 1 < args.len() {
            let candidate = args[idx + 1].trim().to_string();
            if !candidate.is_empty() {
                preferred_device_substring = Some(candidate);
            }
            idx += 2;
            continue;
        }
        if args[idx] == "--seconds" && idx + 1 < args.len() {
            if let Ok(value) = args[idx + 1].parse::<u64>() {
                capture_seconds = value.min(300);
            }
            idx += 2;
            continue;
        }
        idx += 1;
    }
    CliArgs {
        preferred_device_substring,
        capture_seconds,
    }
}

fn seed_identity_and_device(
    store: &mut Ph1fStore,
    user_id: &UserId,
    device_id: &DeviceId,
) -> Result<(), String> {
    store
        .insert_identity(IdentityRecord::v1(
            user_id.clone(),
            None,
            None,
            MonotonicTimeNs(1),
            IdentityStatus::Active,
        ))
        .map_err(|err| format!("identity seed failed: {err:?}"))?;

    store
        .insert_device(
            DeviceRecord::v1(
                device_id.clone(),
                user_id.clone(),
                "desktop".to_string(),
                MonotonicTimeNs(1),
                None,
            )
            .map_err(|err| format!("device record invalid: {err:?}"))?,
        )
        .map_err(|err| format!("device seed failed: {err:?}"))?;

    Ok(())
}

fn seed_wake_profile(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) -> Result<(), String> {
    let started = store
        .ph1w_enroll_start_draft(
            MonotonicTimeNs(10),
            user_id.clone(),
            device_id.clone(),
            None,
            3,
            8,
            180_000,
            "desktop_life_wake_start".to_string(),
        )
        .map_err(|err| format!("wake enroll start failed: {err:?}"))?;

    for idx in 0..3_u64 {
        store
            .ph1w_enroll_sample_commit(
                MonotonicTimeNs(11 + idx),
                started.wake_enrollment_session_id.clone(),
                1_200,
                0.94,
                18.0,
                0.02,
                -20.0,
                -45.0,
                -6.0,
                0.0,
                WakeSampleResult::Pass,
                None,
                format!("desktop_life_wake_sample_{idx}"),
            )
            .map_err(|err| format!("wake enroll sample failed: {err:?}"))?;
    }

    store
        .ph1w_enroll_complete_commit(
            MonotonicTimeNs(20),
            started.wake_enrollment_session_id,
            "wake_profile_desktop_life_v1".to_string(),
            "desktop_life_wake_complete".to_string(),
        )
        .map_err(|err| format!("wake enroll complete failed: {err:?}"))?;

    Ok(())
}
