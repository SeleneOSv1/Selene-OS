#![forbid(unsafe_code)]

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::desktop_mic_producer::{DesktopMicProducer, DesktopMicProducerConfig};
use selene_adapter::{AdapterRuntime, VoiceTurnAdapterRequest};
use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::DeviceId;
use selene_kernel_contracts::MonotonicTimeNs;
use selene_os::app_ingress::AppServerIngressRuntime;
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, WakeSampleResult,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preferred_device_substring = parse_preferred_device_substring();

    println!(
        "usage: cargo run -p selene_adapter --bin desktop_wake_life -- [--device <substring>]"
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
    config.input_device_name_substring = preferred_device_substring;
    let producer = DesktopMicProducer::start(config)?;
    producer.wait_until_pre_roll_ready(Duration::from_secs(8))?;

    let capture_ref = producer.build_capture_ref()?;
    let now_ns = capture_ref.t_end_ns.max(1);

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
                format!(
                    "{} reason_code={} model_version={}",
                    if event.accepted { "accept" } else { "reject" },
                    event.reason_code.0,
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

fn parse_preferred_device_substring() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let mut out = None;
    let mut idx = 1;
    while idx < args.len() {
        if args[idx] == "--device" && idx + 1 < args.len() {
            let candidate = args[idx + 1].trim().to_string();
            if !candidate.is_empty() {
                out = Some(candidate);
            }
            idx += 2;
            continue;
        }
        idx += 1;
    }
    out
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
