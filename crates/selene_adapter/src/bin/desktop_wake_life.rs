#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    let run_seed = monotonic_seed();

    println!(
        "usage: cargo run -p selene_adapter --bin desktop_wake_life -- [--device <substring>] [--seconds <n>]"
    );

    let actor_user_id = UserId::new(format!(
        "tenant_1:desktop_life_actor_{}",
        run_seed % 1_000_000
    ))
    .map_err(|err| format!("invalid actor_user_id: {err:?}"))?;
    let device_id = DeviceId::new(format!("desktop_life_device_{}", run_seed % 1_000_000))
        .map_err(|err| format!("invalid device_id: {err:?}"))?;

    let mut store = Ph1fStore::new_in_memory();
    seed_identity_and_device(&mut store, &actor_user_id, &device_id)?;
    seed_wake_profile(&mut store, &actor_user_id, &device_id)?;

    let store = Arc::new(Mutex::new(store));
    let runtime = AdapterRuntime::new_with_persistence(
        AppServerIngressRuntime::default(),
        store.clone(),
        life_test_journal_path(run_seed),
        true,
    )
    .map_err(|err| format!("desktop wake life runtime bootstrap failed: {err}"))?;

    let mut config = DesktopMicProducerConfig::default();
    config.input_device_name_substring = cli.preferred_device_substring;
    let capture_start_instant = Instant::now();
    let producer = DesktopMicProducer::start(config)?;
    let sample_rate_hz = producer.source_sample_rate_hz()?;
    let channels = producer.source_channels()?;
    let wake_loop = resolve_desktop_wake_loop_config();
    println!(
        "wake loop config: window_ms={} hop_ms={} max_steps={} (override: SELENE_PH1W_LIVE_WINDOW_MS/SELENE_PH1W_LIVE_HOP_MS/SELENE_PH1W_LIVE_MAX_STEPS)",
        wake_loop.window_ms, wake_loop.hop_ms, wake_loop.max_steps
    );
    producer.wait_until_pre_roll_ready(Duration::from_secs(8))?;
    let capture_start_to_preroll_ready_ms = capture_start_instant.elapsed().as_millis() as u64;
    if cli.capture_seconds > 0 {
        println!(
            "capture window active: {}s (speak wake phrase during this window)",
            cli.capture_seconds
        );
        std::thread::sleep(Duration::from_secs(cli.capture_seconds));
    }

    let capture_ref_before = producer.build_capture_ref()?;
    let now_ns = capture_ref_before.t_end_ns.max(1);
    let selected_mic = capture_ref_before
        .selected_mic
        .clone()
        .unwrap_or_else(|| "unknown_mic".to_string());
    let pre_roll_ms = capture_ref_before
        .t_end_ns
        .saturating_sub(capture_ref_before.t_start_ns)
        .saturating_div(1_000_000);
    println!(
        "mic selected: {} sample_rate_hz={} channels={}",
        selected_mic, sample_rate_hz, channels
    );
    println!("pre-roll ready: {}ms (>=1200ms required)", pre_roll_ms);
    println!(
        "capture metrics: vad_confidence_bp={} clipping_ratio_bp={} timing_jitter_ms_milli={} snr_db_milli={}",
        capture_ref_before.vad_confidence_bp.unwrap_or(0),
        capture_ref_before.clipping_ratio_bp.unwrap_or(0),
        capture_ref_before.timing_jitter_ms_milli.unwrap_or(0),
        capture_ref_before.snr_db_milli.unwrap_or(0)
    );
    println!(
        "capture timing: drift_ppm_milli={} underruns={} overruns={} buffer_depth_ms_milli={}",
        capture_ref_before.timing_drift_ppm_milli.unwrap_or(0),
        capture_ref_before.timing_underruns.unwrap_or(0),
        capture_ref_before.timing_overruns.unwrap_or(0),
        capture_ref_before.timing_buffer_depth_ms_milli.unwrap_or(0)
    );
    println!(
        "capture scoring: acoustic_confidence_bp={} prosody_confidence_bp={} speech_likeness_bp={} detection_present={}",
        capture_ref_before.acoustic_confidence_bp.unwrap_or(0),
        capture_ref_before.prosody_confidence_bp.unwrap_or(0),
        capture_ref_before.speech_likeness_bp.unwrap_or(0),
        capture_ref_before
            .detection_text
            .as_deref()
            .is_some_and(|v| !v.trim().is_empty())
    );

    let session_count_before = {
        let guard = store
            .lock()
            .map_err(|_| "store lock poisoned while counting baseline sessions")?;
        guard
            .session_rows()
            .values()
            .filter(|row| row.user_id == actor_user_id && row.device_id == device_id)
            .count()
    };
    let wake_event_count_before = {
        let guard = store
            .lock()
            .map_err(|_| "store lock poisoned while counting baseline wake events")?;
        guard.ph1w_get_runtime_events().len()
    };

    let request = VoiceTurnAdapterRequest {
        correlation_id: (run_seed % 9_000_000).saturating_add(80_000),
        turn_id: (run_seed % 9_000_000).saturating_add(80_000),
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
        audio_capture_ref: Some(capture_ref_before.clone()),
        visual_input_ref: None,
    };

    let process_before = sample_process_stats();
    let decision_start = Instant::now();
    let run_result = runtime.run_voice_turn(request);
    let decision_elapsed_ms = decision_start.elapsed().as_millis() as u64;
    let process_after = sample_process_stats();
    let capture_ref_after = producer.build_capture_ref()?;

    let (wake_summary, session_summary, wake_event_emitted, wake_accepted, session_opened) = {
        let guard = store
            .lock()
            .map_err(|_| "store lock poisoned while building life-test summary")?;

        let wake_event_emitted = guard.ph1w_get_runtime_events().len() > wake_event_count_before;
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
                    event.model_version.as_deref().unwrap_or("unknown_model")
                )
            })
            .unwrap_or_else(|| "reject reason_code=NO_WAKE_RUNTIME_EVENT".to_string());

        let wake_accepted = guard
            .ph1w_get_runtime_events()
            .last()
            .map(|event| event.accepted)
            .unwrap_or(false);

        let session_count_after = guard
            .session_rows()
            .values()
            .filter(|row| row.user_id == actor_user_id && row.device_id == device_id)
            .count();
        let session_opened = session_count_after > session_count_before;

        let session_summary = guard
            .session_rows()
            .values()
            .filter(|row| row.user_id == actor_user_id && row.device_id == device_id)
            .max_by_key(|row| row.last_activity_at.0)
            .map(|row| {
                format!(
                    "session_id={} state={:?}",
                    row.session_id.0, row.session_state
                )
            })
            .unwrap_or_else(|| "session_id=NONE state=Closed".to_string());

        (
            wake_summary,
            session_summary,
            wake_event_emitted,
            wake_accepted,
            session_opened,
        )
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

    let peak_buffer_depth_ms = [
        capture_ref_before.timing_buffer_depth_ms_milli,
        capture_ref_after.timing_buffer_depth_ms_milli,
    ]
    .into_iter()
    .flatten()
    .map(|v| (v as u64).saturating_div(1_000))
    .max();

    let metrics = DesktopWakeRunMetrics {
        capture_start_to_preroll_ready_ms,
        pre_roll_ms,
        wake_decision_latency_ms: if wake_event_emitted {
            Some(decision_elapsed_ms)
        } else {
            None
        },
        session_open_latency_ms: if session_opened {
            Some(decision_elapsed_ms)
        } else {
            None
        },
        peak_buffer_depth_ms,
        timing_jitter_ms_milli: capture_ref_after.timing_jitter_ms_milli.unwrap_or(0),
        vad_confidence_bp: capture_ref_after.vad_confidence_bp.unwrap_or(0),
        snr_db_milli: capture_ref_after.snr_db_milli.unwrap_or(0),
        clipping_ratio_bp: capture_ref_after.clipping_ratio_bp.unwrap_or(0),
        wake_event_emitted,
        wake_accepted,
        session_opened,
        process_rss_mb_peak: peak_f64(process_before.rss_mb, process_after.rss_mb),
        process_cpu_percent_snapshot: process_after.cpu_percent,
    };

    let gate_config = DesktopWakeReleaseGateConfig::default();
    let gate_results = evaluate_desktop_release_gates(&metrics, &gate_config);
    for line in render_metric_summary_lines(&metrics) {
        println!("{line}");
    }
    for line in render_gate_summary_lines(&gate_results) {
        println!("{line}");
    }

    producer.stop();
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct ProcessStatsSnapshot {
    rss_mb: Option<f64>,
    cpu_percent: Option<f64>,
}

fn sample_process_stats() -> ProcessStatsSnapshot {
    let pid = std::process::id().to_string();
    let rss_kb = sample_ps_value(&pid, "rss=");
    let cpu_percent = sample_ps_value(&pid, "%cpu=");
    ProcessStatsSnapshot {
        rss_mb: rss_kb.map(|kb| kb / 1024.0),
        cpu_percent,
    }
}

fn sample_ps_value(pid: &str, column: &str) -> Option<f64> {
    let output = Command::new("ps")
        .args(["-o", column, "-p", pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let value = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        .replace(',', ".");
    value.parse::<f64>().ok()
}

fn peak_f64(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[derive(Debug, Clone)]
struct DesktopWakeRunMetrics {
    capture_start_to_preroll_ready_ms: u64,
    pre_roll_ms: u64,
    wake_decision_latency_ms: Option<u64>,
    session_open_latency_ms: Option<u64>,
    peak_buffer_depth_ms: Option<u64>,
    timing_jitter_ms_milli: u32,
    vad_confidence_bp: u16,
    snr_db_milli: i32,
    clipping_ratio_bp: u16,
    wake_event_emitted: bool,
    wake_accepted: bool,
    session_opened: bool,
    process_rss_mb_peak: Option<f64>,
    process_cpu_percent_snapshot: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct DesktopWakeReleaseGateConfig {
    min_pre_roll_ms: u64,
    max_wake_to_session_open_latency_ms: u64,
    max_cpu_percent: f64,
    max_rss_mb: f64,
}

impl Default for DesktopWakeReleaseGateConfig {
    fn default() -> Self {
        Self {
            min_pre_roll_ms: 1_200,
            max_wake_to_session_open_latency_ms: 350,
            max_cpu_percent: 4.0,
            max_rss_mb: 120.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateStatus {
    Pass,
    Fail,
    Open,
}

impl GateStatus {
    fn as_str(self) -> &'static str {
        match self {
            GateStatus::Pass => "PASS",
            GateStatus::Fail => "FAIL",
            GateStatus::Open => "OPEN",
        }
    }
}

#[derive(Debug, Clone)]
struct GateResult {
    name: &'static str,
    status: GateStatus,
    detail: String,
}

fn evaluate_desktop_release_gates(
    metrics: &DesktopWakeRunMetrics,
    config: &DesktopWakeReleaseGateConfig,
) -> Vec<GateResult> {
    let mut gates = Vec::with_capacity(7);

    gates.push(GateResult {
        name: "pre_roll_ready",
        status: if metrics.pre_roll_ms >= config.min_pre_roll_ms {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        },
        detail: format!(
            "pre_roll_ms={} required_min={}",
            metrics.pre_roll_ms, config.min_pre_roll_ms
        ),
    });

    gates.push(GateResult {
        name: "wake_decision_emitted",
        status: if metrics.wake_event_emitted {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        },
        detail: format!(
            "wake_event_emitted={} wake_accepted={}",
            metrics.wake_event_emitted, metrics.wake_accepted
        ),
    });

    let speech_metrics_ok = metrics.vad_confidence_bp > 0 && metrics.snr_db_milli != 0;
    gates.push(GateResult {
        name: "speech_metrics_nonzero",
        status: if speech_metrics_ok {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        },
        detail: format!(
            "vad_confidence_bp={} snr_db_milli={}",
            metrics.vad_confidence_bp, metrics.snr_db_milli
        ),
    });

    let session_latency_gate = match metrics.session_open_latency_ms {
        Some(latency_ms) => GateResult {
            name: "wake_to_session_open_latency",
            status: if latency_ms <= config.max_wake_to_session_open_latency_ms {
                GateStatus::Pass
            } else {
                GateStatus::Fail
            },
            detail: format!(
                "latency_ms={} max_allowed_ms={}",
                latency_ms, config.max_wake_to_session_open_latency_ms
            ),
        },
        None => GateResult {
            name: "wake_to_session_open_latency",
            status: GateStatus::Fail,
            detail: "session_open_latency_ms=NONE (session not opened)".to_string(),
        },
    };
    gates.push(session_latency_gate);

    let cpu_gate = match metrics.process_cpu_percent_snapshot {
        Some(cpu_percent) => GateResult {
            name: "cpu_budget",
            status: if cpu_percent <= config.max_cpu_percent {
                GateStatus::Pass
            } else {
                GateStatus::Fail
            },
            detail: format!(
                "cpu_percent={} max_allowed={}",
                format_float_2(cpu_percent),
                format_float_2(config.max_cpu_percent)
            ),
        },
        None => GateResult {
            name: "cpu_budget",
            status: GateStatus::Open,
            detail: "OPEN / NOT MEASURED YET".to_string(),
        },
    };
    gates.push(cpu_gate);

    let rss_gate = match metrics.process_rss_mb_peak {
        Some(rss_mb) => GateResult {
            name: "rss_budget",
            status: if rss_mb <= config.max_rss_mb {
                GateStatus::Pass
            } else {
                GateStatus::Fail
            },
            detail: format!(
                "rss_mb_peak={} max_allowed={}",
                format_float_2(rss_mb),
                format_float_2(config.max_rss_mb)
            ),
        },
        None => GateResult {
            name: "rss_budget",
            status: GateStatus::Open,
            detail: "OPEN / NOT MEASURED YET".to_string(),
        },
    };
    gates.push(rss_gate);

    gates
}

fn render_metric_summary_lines(metrics: &DesktopWakeRunMetrics) -> Vec<String> {
    vec![
        format!(
            "metric.capture_start_to_preroll_ready_ms={}",
            metrics.capture_start_to_preroll_ready_ms
        ),
        format!("metric.pre_roll_ms={}", metrics.pre_roll_ms),
        format!(
            "metric.wake_decision_latency_ms={}",
            optional_u64(metrics.wake_decision_latency_ms)
        ),
        format!(
            "metric.session_open_latency_ms={}",
            optional_u64(metrics.session_open_latency_ms)
        ),
        format!(
            "metric.peak_buffer_depth_ms={}",
            optional_u64(metrics.peak_buffer_depth_ms)
        ),
        format!(
            "metric.timing_jitter_ms_milli={}",
            metrics.timing_jitter_ms_milli
        ),
        format!("metric.vad_confidence_bp={}", metrics.vad_confidence_bp),
        format!("metric.snr_db_milli={}", metrics.snr_db_milli),
        format!("metric.clipping_ratio_bp={}", metrics.clipping_ratio_bp),
        format!("metric.wake_event_emitted={}", metrics.wake_event_emitted),
        format!("metric.wake_accepted={}", metrics.wake_accepted),
        format!("metric.session_opened={}", metrics.session_opened),
        format!(
            "metric.process_rss_mb_peak={}",
            optional_f64(metrics.process_rss_mb_peak)
        ),
        format!(
            "metric.process_cpu_percent_snapshot={}",
            optional_f64(metrics.process_cpu_percent_snapshot)
        ),
    ]
}

fn render_gate_summary_lines(gates: &[GateResult]) -> Vec<String> {
    let mut lines = Vec::with_capacity(gates.len() + 1);
    let mut pass_count = 0_u32;
    let mut fail_count = 0_u32;
    let mut open_count = 0_u32;
    for gate in gates {
        match gate.status {
            GateStatus::Pass => pass_count = pass_count.saturating_add(1),
            GateStatus::Fail => fail_count = fail_count.saturating_add(1),
            GateStatus::Open => open_count = open_count.saturating_add(1),
        }
        lines.push(format!(
            "gate.{}={} detail={}",
            gate.name,
            gate.status.as_str(),
            gate.detail
        ));
    }
    let overall = if fail_count > 0 {
        GateStatus::Fail
    } else if open_count > 0 {
        GateStatus::Open
    } else {
        GateStatus::Pass
    };
    lines.push(format!(
        "gate.summary={} pass={} fail={} open={}",
        overall.as_str(),
        pass_count,
        fail_count,
        open_count
    ));
    lines
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "OPEN_NOT_MEASURED_YET".to_string())
}

fn optional_f64(value: Option<f64>) -> String {
    value
        .map(format_float_2)
        .unwrap_or_else(|| "OPEN_NOT_MEASURED_YET".to_string())
}

fn format_float_2(value: f64) -> String {
    format!("{value:.2}")
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

fn seed_wake_profile(
    store: &mut Ph1fStore,
    user_id: &UserId,
    device_id: &DeviceId,
) -> Result<(), String> {
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

fn monotonic_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        .max(1)
}

fn life_test_journal_path(seed: u64) -> PathBuf {
    std::env::temp_dir().join(format!("selene_desktop_wake_life_{seed}.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics() -> DesktopWakeRunMetrics {
        DesktopWakeRunMetrics {
            capture_start_to_preroll_ready_ms: 1320,
            pre_roll_ms: 1500,
            wake_decision_latency_ms: Some(210),
            session_open_latency_ms: Some(210),
            peak_buffer_depth_ms: Some(1800),
            timing_jitter_ms_milli: 2200,
            vad_confidence_bp: 7400,
            snr_db_milli: 12000,
            clipping_ratio_bp: 250,
            wake_event_emitted: true,
            wake_accepted: true,
            session_opened: true,
            process_rss_mb_peak: Some(85.5),
            process_cpu_percent_snapshot: Some(2.5),
        }
    }

    #[test]
    fn desktop_wake_summary_formatter_includes_required_metric_labels() {
        let lines = render_metric_summary_lines(&sample_metrics());
        let text = lines.join("\n");
        for label in [
            "metric.capture_start_to_preroll_ready_ms=",
            "metric.wake_decision_latency_ms=",
            "metric.session_open_latency_ms=",
            "metric.peak_buffer_depth_ms=",
            "metric.timing_jitter_ms_milli=",
            "metric.vad_confidence_bp=",
            "metric.snr_db_milli=",
            "metric.clipping_ratio_bp=",
        ] {
            assert!(
                text.contains(label),
                "summary should include required label: {label}"
            );
        }
    }

    #[test]
    fn desktop_release_gates_pass_and_fail_as_expected() {
        let cfg = DesktopWakeReleaseGateConfig::default();
        let pass = evaluate_desktop_release_gates(&sample_metrics(), &cfg);
        assert!(
            pass.iter().all(|g| g.status == GateStatus::Pass),
            "all synthetic pass metrics should pass"
        );

        let mut fail_metrics = sample_metrics();
        fail_metrics.pre_roll_ms = 800;
        fail_metrics.session_open_latency_ms = None;
        fail_metrics.process_cpu_percent_snapshot = Some(7.0);
        let fail = evaluate_desktop_release_gates(&fail_metrics, &cfg);
        assert!(
            fail.iter().any(|g| g.status == GateStatus::Fail),
            "fail metrics should produce at least one FAIL gate"
        );
    }
}
