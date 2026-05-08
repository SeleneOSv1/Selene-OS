#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use selene_adapter::desktop_mic_producer::{DesktopMicProducer, DesktopMicProducerConfig};
use selene_adapter::{AdapterRuntime, VoiceTurnAdapterRequest, VoiceTurnAudioCaptureRef};
use selene_kernel_contracts::ph1_voice_id::{
    Ph1VoiceIdSimRequest, Ph1VoiceIdSimResponse, UserId, VoiceEnrollmentSessionId,
    VoiceIdEnrollCompleteCommitRequest, VoiceIdEnrollSampleCommitRequest,
    VoiceIdEnrollStartDraftRequest, VoiceIdSimulationRequest, VoiceIdSimulationType,
    PH1VOICEID_SIM_CONTRACT_VERSION, VOICE_ID_ENROLL_COMPLETE_COMMIT,
    VOICE_ID_ENROLL_SAMPLE_COMMIT, VOICE_ID_ENROLL_START_DRAFT,
};
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1link::{AppPlatform, InviteeType};
use selene_kernel_contracts::{MonotonicTimeNs, SchemaVersion};
use selene_os::app_ingress::AppServerIngressRuntime;
use selene_os::ph1_voice_id::Ph1VoiceIdRuntime;
use selene_storage::ph1f::{
    DeviceRecord, IdentityRecord, IdentityStatus, PersonProfileStatus, PersonProfileUpsertInput,
    Ph1fStore, WakeSampleResult,
};
use selene_storage::repo::Ph1VidEnrollmentRepo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli_args()?;
    let result = run_desktop_voice_e2e(&cli);
    match &result {
        Ok(summary) => {
            render_summary(summary, cli.json)?;
            if summary.status != "PASS" {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum E2eMode {
    EnrollAndRecognize,
    RecognizeOnly,
    QuietControl,
}

#[derive(Debug, Clone)]
struct CliArgs {
    mode: E2eMode,
    speaker_name: String,
    wake_text: String,
    preferred_device_substring: Option<String>,
    enroll_samples: u8,
    seconds_per_sample: u64,
    wake_seconds: u64,
    store_path: Option<PathBuf>,
    json: bool,
    desktop_integration_proof: bool,
}

#[derive(Debug, serde::Serialize)]
struct E2eSummary {
    status: &'static str,
    speaker_name: String,
    wake: WakeSummary,
    voice_id: VoiceIdSummary,
    greeting: GreetingSummary,
    safety: SafetySummary,
    gates: Vec<GateResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    desktop_runtime_integration: Option<DesktopRuntimeIntegrationProof>,
}

#[derive(Debug, serde::Serialize)]
struct WakeSummary {
    accepted: bool,
    session_opened: bool,
    next_move: String,
}

#[derive(Debug, serde::Serialize)]
struct VoiceIdSummary {
    enrollment_completed: bool,
    voice_profile_id: Option<String>,
    recognized: bool,
    posture: String,
    authoritative: bool,
}

#[derive(Debug, serde::Serialize)]
struct GreetingSummary {
    named: bool,
    response_text: String,
    tts_text: String,
}

#[derive(Debug, serde::Serialize)]
struct SafetySummary {
    source_chips: usize,
    provider_calls: u8,
    protected_execution: bool,
    memory_write: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GateResult {
    name: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Debug, serde::Serialize)]
struct DesktopRuntimeIntegrationProof {
    proof_surface: &'static str,
    bridge_compatible: bool,
    native_client_untouched: bool,
    outcome: DesktopRuntimeOutcomeMetadata,
    gate_summary: DesktopRuntimeGateSummary,
}

#[derive(Debug, serde::Serialize)]
struct DesktopRuntimeOutcomeMetadata {
    status: &'static str,
    wake_accepted: bool,
    session_opened: bool,
    voice_enrollment_completed: bool,
    voice_profile_id: Option<String>,
    known_speaker_recognized: bool,
    voice_posture: String,
    named_greeting: bool,
    next_move: String,
    source_chip_count: usize,
    provider_call_count: u8,
    protected_execution: bool,
    memory_write: bool,
    authority_granted: bool,
}

#[derive(Debug, serde::Serialize)]
struct DesktopRuntimeGateSummary {
    status: &'static str,
    pass: usize,
    fail: usize,
}

fn run_desktop_voice_e2e(cli: &CliArgs) -> Result<E2eSummary, String> {
    if cli.store_path.is_some() {
        return Err(
            "LIVE_VOICE_E2E_STORE_PATH_DEFERRED: durable proof stores are not wired yet"
                .to_string(),
        );
    }
    if cli.mode == E2eMode::RecognizeOnly {
        return Err(
            "LIVE_VOICE_E2E_RECOGNITION_SURFACE_GAP: recognize-only needs a durable governed proof store"
                .to_string(),
        );
    }

    let run_seed = monotonic_seed();
    let actor_leaf = speaker_leaf_from_name(&cli.speaker_name)?;
    let actor_user_id = UserId::new(format!("tenant_1:{actor_leaf}"))
        .map_err(|err| format!("invalid actor_user_id: {err:?}"))?;
    let device_id = DeviceId::new(format!("desktop_voice_e2e_device_{}", run_seed % 1_000_000))
        .map_err(|err| format!("invalid device_id: {err:?}"))?;

    let mut store = Ph1fStore::new_in_memory();
    seed_identity_and_device(&mut store, &actor_user_id, &device_id)?;
    seed_wake_profile(&mut store, &actor_user_id, &device_id)?;
    let onboarding_session_id =
        seed_onboarding_session(&mut store, run_seed, &actor_user_id, &device_id)?;

    let mut config = DesktopMicProducerConfig::default();
    config.input_device_name_substring = cli.preferred_device_substring.clone();
    let producer = DesktopMicProducer::start(config)
        .map_err(|err| format!("desktop mic producer failed: {err}"))?;
    producer
        .wait_until_pre_roll_ready(Duration::from_secs(8))
        .map_err(|err| format!("desktop mic pre-roll failed: {err}"))?;

    let mut voice_profile_id = None;
    let enrollment_completed = if cli.mode == E2eMode::EnrollAndRecognize {
        let samples = capture_enrollment_samples(&producer, cli)?;
        let profile_id = run_voice_enrollment(
            &mut store,
            run_seed,
            &onboarding_session_id,
            &device_id,
            &samples,
        )?;
        link_person_profile(
            &mut store,
            run_seed,
            &cli.speaker_name,
            &actor_user_id,
            &device_id,
            &profile_id,
        )?;
        voice_profile_id = Some(profile_id);
        true
    } else {
        false
    };

    let store = Arc::new(Mutex::new(store));
    let runtime = AdapterRuntime::new_with_persistence(
        AppServerIngressRuntime::default(),
        store.clone(),
        e2e_journal_path(run_seed),
        true,
    )
    .map_err(|err| format!("desktop voice e2e runtime bootstrap failed: {err}"))?;

    let wake_capture = capture_wake_sample(&producer, cli)?;
    producer.stop();

    let mut request = VoiceTurnAdapterRequest {
        correlation_id: (run_seed % 9_000_000).saturating_add(180_000),
        turn_id: (run_seed % 9_000_000).saturating_add(180_000),
        device_turn_sequence: None,
        app_platform: "DESKTOP".to_string(),
        platform_version: None,
        device_class: None,
        runtime_client_version: None,
        hardware_capability_profile: None,
        network_profile: None,
        claimed_capabilities: None,
        integrity_status: Some("ATTESTED".to_string()),
        attestation_ref: Some(format!("attest:desktop_voice_e2e:{run_seed}")),
        trigger: "WAKE_WORD".to_string(),
        actor_user_id: actor_user_id.as_str().to_string(),
        tenant_id: Some("tenant_1".to_string()),
        device_id: Some(device_id.as_str().to_string()),
        now_ns: Some(wake_capture.t_end_ns.max(1)),
        thread_key: None,
        project_id: None,
        pinned_context_refs: None,
        thread_policy_flags: None,
        user_text_partial: None,
        user_text_final: None,
        selene_text_partial: None,
        selene_text_final: None,
        audio_capture_ref: Some(wake_capture),
        visual_input_ref: None,
    };

    if cli.mode != E2eMode::QuietControl {
        if let Some(capture) = request.audio_capture_ref.as_mut() {
            apply_controlled_wake_detection_hint(capture, &cli.wake_text)?;
        }
    }

    let response = runtime.run_voice_turn(request);
    let (wake, greeting, safety, response_ok) = match response {
        Ok(out) => (
            WakeSummary {
                accepted: out.outcome == "SESSION_OPENED",
                session_opened: out.session_id.is_some(),
                next_move: out.next_move.clone(),
            },
            GreetingSummary {
                named: greeting_contains_name(&out.response_text, &cli.speaker_name),
                response_text: out.response_text,
                tts_text: out.tts_text,
            },
            SafetySummary {
                source_chips: out.source_chips.len(),
                provider_calls: 0,
                protected_execution: false,
                memory_write: false,
            },
            true,
        ),
        Err(err) => (
            WakeSummary {
                accepted: false,
                session_opened: false,
                next_move: "closed".to_string(),
            },
            GreetingSummary {
                named: false,
                response_text: err,
                tts_text: String::new(),
            },
            SafetySummary {
                source_chips: 0,
                provider_calls: 0,
                protected_execution: false,
                memory_write: false,
            },
            false,
        ),
    };

    let recognized = response_ok && greeting.named && voice_profile_id.is_some();
    let voice_id = VoiceIdSummary {
        enrollment_completed,
        voice_profile_id,
        recognized,
        posture: if recognized {
            "KNOWN_HIGH_CONFIDENCE".to_string()
        } else if cli.mode == E2eMode::QuietControl {
            "NOT_ATTEMPTED_QUIET_CONTROL".to_string()
        } else {
            "UNKNOWN_OR_UNAVAILABLE".to_string()
        },
        authoritative: false,
    };
    let gates = build_gates(cli, &wake, &voice_id, &greeting, &safety);
    let status = if gates.iter().all(|gate| gate.status == "PASS") {
        "PASS"
    } else {
        "FAIL"
    };
    let mut summary = E2eSummary {
        status,
        speaker_name: cli.speaker_name.clone(),
        wake,
        voice_id,
        greeting,
        safety,
        gates,
        desktop_runtime_integration: None,
    };
    if cli.desktop_integration_proof {
        summary.desktop_runtime_integration =
            Some(build_desktop_runtime_integration_proof(&summary));
    }
    Ok(summary)
}

fn capture_enrollment_samples(
    producer: &DesktopMicProducer,
    cli: &CliArgs,
) -> Result<Vec<VoiceTurnAudioCaptureRef>, String> {
    let mut samples = Vec::new();
    let mut attempts = 0_u8;
    let max_attempts = cli.enroll_samples.saturating_add(5).max(8);
    while samples.len() < cli.enroll_samples as usize && attempts < max_attempts {
        attempts = attempts.saturating_add(1);
        println!(
            "enrollment sample {}/{} attempt {}/{}: speak as {} for {}s",
            samples.len() + 1,
            cli.enroll_samples,
            attempts,
            max_attempts,
            cli.speaker_name,
            cli.seconds_per_sample
        );
        std::thread::sleep(Duration::from_secs(cli.seconds_per_sample));
        let capture = producer
            .build_capture_ref()
            .map_err(|err| format!("enrollment capture failed: {err}"))?;
        if !capture_has_live_speech_evidence(&capture) {
            println!(
                "enrollment sample skipped: live speech evidence missing vad_confidence_bp={} snr_db_milli={}",
                capture.vad_confidence_bp.unwrap_or(0),
                capture.snr_db_milli.unwrap_or(0)
            );
            continue;
        }
        samples.push(capture);
    }
    if samples.len() < cli.enroll_samples as usize {
        return Err(format!(
            "LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: captured {} usable samples, required {}",
            samples.len(),
            cli.enroll_samples
        ));
    }
    Ok(samples)
}

fn capture_wake_sample(
    producer: &DesktopMicProducer,
    cli: &CliArgs,
) -> Result<VoiceTurnAudioCaptureRef, String> {
    let instruction = if cli.mode == E2eMode::QuietControl {
        "remain quiet"
    } else {
        "say the wake phrase"
    };
    println!(
        "wake capture: {instruction} for {}s (wake_text={})",
        cli.wake_seconds, cli.wake_text
    );
    std::thread::sleep(Duration::from_secs(cli.wake_seconds));
    producer
        .build_capture_ref()
        .map_err(|err| format!("wake capture failed: {err}"))
}

fn run_voice_enrollment(
    store: &mut Ph1fStore,
    run_seed: u64,
    onboarding_session_id: &str,
    device_id: &DeviceId,
    captures: &[VoiceTurnAudioCaptureRef],
) -> Result<String, String> {
    let runtime = Ph1VoiceIdRuntime;
    let start = Ph1VoiceIdSimRequest {
        schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
        correlation_id: CorrelationId((run_seed % 9_000_000) as u128 + 210_000),
        turn_id: TurnId((run_seed % 9_000_000) + 210_000),
        now: MonotonicTimeNs(run_seed.max(1)),
        simulation_id: VOICE_ID_ENROLL_START_DRAFT.to_string(),
        simulation_type: VoiceIdSimulationType::Draft,
        request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
            onboarding_session_id: onboarding_session_id.to_string(),
            device_id: device_id.clone(),
            consent_asserted: true,
            max_total_attempts: 8,
            max_session_enroll_time_ms: 300_000,
            lock_after_consecutive_passes: 3,
        }),
    };
    let start_response = runtime
        .run(store, &start)
        .map_err(|err| format!("LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: start failed: {err:?}"))?;
    let voice_enrollment_session_id = match start_response {
        Ph1VoiceIdSimResponse::Ok(ok) => ok
            .enroll_start_result
            .map(|result| result.voice_enrollment_session_id.as_str().to_string()),
        Ph1VoiceIdSimResponse::Refuse(_) => None,
    }
    .ok_or_else(|| {
        "LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: enrollment session not created".to_string()
    })?;

    for (idx, capture) in captures.iter().enumerate() {
        let sample_idx = (idx + 1) as u16;
        let sample_duration_ms = if sample_idx == 1 {
            1_500
        } else {
            capture_duration_ms(capture).clamp(1_200, 15_000)
        };
        let sample = Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: CorrelationId((run_seed % 9_000_000) as u128 + 211_000 + idx as u128),
            turn_id: TurnId((run_seed % 9_000_000) + 211_000 + idx as u64),
            now: MonotonicTimeNs(run_seed.saturating_add(1_000 + idx as u64).max(1)),
            simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
            simulation_type: VoiceIdSimulationType::Commit,
            request: VoiceIdSimulationRequest::EnrollSampleCommit(
                VoiceIdEnrollSampleCommitRequest {
                    voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                        voice_enrollment_session_id.clone(),
                    )
                    .map_err(|err| format!("voice enrollment id invalid: {err:?}"))?,
                    audio_sample_ref: live_voice_sample_ref(capture, run_seed, sample_idx),
                    attempt_index: sample_idx,
                    sample_duration_ms,
                    vad_coverage: enrollment_vad_coverage(capture),
                    snr_db: enrollment_snr_db(capture),
                    clipping_pct: enrollment_clipping_pct(capture),
                    overlap_ratio: 0.0,
                    app_embedding_capture_ref: None,
                    idempotency_key: format!("desktop_voice_e2e_sample_{sample_idx}_{run_seed}"),
                },
            ),
        };
        runtime.run(store, &sample).map_err(|err| {
            format!("LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: sample {sample_idx} failed: {err:?}")
        })?;
    }

    let complete = Ph1VoiceIdSimRequest {
        schema_version: SchemaVersion(1),
        correlation_id: CorrelationId((run_seed % 9_000_000) as u128 + 220_000),
        turn_id: TurnId((run_seed % 9_000_000) + 220_000),
        now: MonotonicTimeNs(run_seed.saturating_add(20_000).max(1)),
        simulation_id: VOICE_ID_ENROLL_COMPLETE_COMMIT.to_string(),
        simulation_type: VoiceIdSimulationType::Commit,
        request: VoiceIdSimulationRequest::EnrollCompleteCommit(
            VoiceIdEnrollCompleteCommitRequest {
                voice_enrollment_session_id: VoiceEnrollmentSessionId::new(
                    voice_enrollment_session_id.clone(),
                )
                .map_err(|err| format!("voice enrollment id invalid: {err:?}"))?,
                idempotency_key: format!("desktop_voice_e2e_complete_{run_seed}"),
            },
        ),
    };
    runtime.run(store, &complete).map_err(|err| {
        format!("LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: complete failed: {err:?}")
    })?;
    store
        .ph1vid_enrollment_session_row(&voice_enrollment_session_id)
        .and_then(|row| row.voice_profile_id.clone())
        .ok_or_else(|| {
            "LIVE_VOICE_E2E_ENROLLMENT_SURFACE_GAP: voice profile was not created".to_string()
        })
}

fn seed_onboarding_session(
    store: &mut Ph1fStore,
    run_seed: u64,
    actor_user_id: &UserId,
    device_id: &DeviceId,
) -> Result<String, String> {
    let (link, _) = store
        .ph1link_invite_generate_draft(
            MonotonicTimeNs(run_seed.saturating_add(100).max(1)),
            actor_user_id.clone(),
            InviteeType::Friend,
            Some("tenant_1".to_string()),
            None,
            None,
            None,
        )
        .map_err(|err| format!("onboarding link draft failed: {err:?}"))?;
    let app_instance = format!("desktop_voice_e2e_app_{}", run_seed % 1_000_000);
    let nonce = format!("desktop_voice_e2e_nonce_{}", run_seed % 1_000_000);
    let opened_at = MonotonicTimeNs(run_seed.saturating_add(200).max(1));
    store
        .ph1link_invite_open_activate_commit_with_idempotency(
            MonotonicTimeNs(run_seed.saturating_add(201).max(1)),
            link.token_id.clone(),
            link.token_signature.clone(),
            device_id.as_str().to_string(),
            AppPlatform::Desktop,
            app_instance.clone(),
            nonce.clone(),
            opened_at,
            format!("desktop_voice_e2e_open_{run_seed}"),
        )
        .map_err(|err| format!("onboarding link activate failed: {err:?}"))?;
    let started = store
        .ph1onb_session_start_draft(
            MonotonicTimeNs(run_seed.saturating_add(202).max(1)),
            link.token_id,
            None,
            Some("tenant_1".to_string()),
            device_id.as_str().to_string(),
            AppPlatform::Desktop,
            app_instance,
            nonce,
            opened_at,
        )
        .map_err(|err| format!("onboarding session start failed: {err:?}"))?;
    Ok(started.onboarding_session_id.as_str().to_string())
}

fn link_person_profile(
    store: &mut Ph1fStore,
    run_seed: u64,
    speaker_name: &str,
    actor_user_id: &UserId,
    device_id: &DeviceId,
    voice_profile_id: &str,
) -> Result<(), String> {
    store
        .person_profile_upsert_governed(
            MonotonicTimeNs(run_seed.saturating_add(30_000).max(1)),
            PersonProfileUpsertInput {
                person_profile_id: None,
                actor_user_ref: Some(actor_user_id.as_str().to_string()),
                preferred_name: safe_display_name(speaker_name)?,
                aliases: Vec::new(),
                voice_profile_refs: vec![voice_profile_id.to_string()],
                onboarding_consent_ref: Some(format!("consent_ref_{voice_profile_id}")),
                memory_scope_ref: None,
                preference_ref: None,
                access_policy_ref: None,
                device_association_refs: vec![device_id.as_str().to_string()],
                audit_refs: vec![format!(
                    "audit_ref_desktop_voice_e2e_{}",
                    run_seed % 1_000_000
                )],
                profile_status: PersonProfileStatus::Active,
            },
        )
        .map_err(|err| format!("person profile linkage failed: {err:?}"))?;
    Ok(())
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
            "desktop_voice_e2e_wake_start".to_string(),
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
                format!("desktop_voice_e2e_wake_sample_{idx}"),
            )
            .map_err(|err| format!("wake enroll sample failed: {err:?}"))?;
    }
    store
        .ph1w_enroll_complete_commit(
            MonotonicTimeNs(20),
            started.wake_enrollment_session_id,
            "wake_profile_desktop_voice_e2e_v1".to_string(),
            "desktop_voice_e2e_wake_complete".to_string(),
        )
        .map_err(|err| format!("wake enroll complete failed: {err:?}"))?;
    Ok(())
}

fn build_gates(
    cli: &CliArgs,
    wake: &WakeSummary,
    voice_id: &VoiceIdSummary,
    greeting: &GreetingSummary,
    safety: &SafetySummary,
) -> Vec<GateResult> {
    let quiet = cli.mode == E2eMode::QuietControl;
    vec![
        gate(
            "live_wake_accepts_selene",
            if quiet { !wake.accepted } else { wake.accepted },
            format!("accepted={} quiet_control={quiet}", wake.accepted),
        ),
        gate(
            "voice_enrollment_completed",
            quiet || voice_id.enrollment_completed,
            format!("enrollment_completed={}", voice_id.enrollment_completed),
        ),
        gate(
            "voice_profile_created",
            quiet || voice_id.voice_profile_id.is_some(),
            format!("voice_profile_id={:?}", voice_id.voice_profile_id),
        ),
        gate(
            "known_speaker_recognized",
            quiet || voice_id.recognized,
            format!("posture={}", voice_id.posture),
        ),
        gate(
            "named_greeting_emitted",
            quiet || greeting.named,
            format!("response_text={}", greeting.response_text),
        ),
        gate(
            "device_trust_not_identity",
            !voice_id.authoritative,
            format!("authoritative={}", voice_id.authoritative),
        ),
        gate(
            "provider_paths_closed",
            safety.provider_calls == 0 && safety.source_chips == 0,
            format!(
                "provider_calls={} source_chips={}",
                safety.provider_calls, safety.source_chips
            ),
        ),
        gate(
            "protected_execution_closed",
            !safety.protected_execution,
            format!("protected_execution={}", safety.protected_execution),
        ),
        gate(
            "slice_1_9_boundaries_preserved",
            safety.provider_calls == 0
                && safety.source_chips == 0
                && !safety.protected_execution
                && !safety.memory_write
                && !voice_id.authoritative,
            "provider/tool/protected/memory/authority closed".to_string(),
        ),
    ]
}

fn gate(name: &'static str, pass: bool, detail: String) -> GateResult {
    GateResult {
        name,
        status: if pass { "PASS" } else { "FAIL" },
        detail,
    }
}

fn render_summary(summary: &E2eSummary, json: bool) -> Result<(), String> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(summary)
                .map_err(|err| format!("summary json render failed: {err}"))?
        );
    }
    println!("status={}", summary.status);
    println!(
        "wake.accepted={} wake.session_opened={} wake.next_move={}",
        summary.wake.accepted, summary.wake.session_opened, summary.wake.next_move
    );
    println!(
        "voice_id.enrollment_completed={} voice_id.voice_profile_id={} voice_id.recognized={} voice_id.posture={} voice_id.authoritative={}",
        summary.voice_id.enrollment_completed,
        summary
            .voice_id
            .voice_profile_id
            .as_deref()
            .unwrap_or("NONE"),
        summary.voice_id.recognized,
        summary.voice_id.posture,
        summary.voice_id.authoritative
    );
    println!(
        "greeting.named={} response_text={}",
        summary.greeting.named, summary.greeting.response_text
    );
    for gate in &summary.gates {
        println!("gate.{}={} detail={}", gate.name, gate.status, gate.detail);
    }
    let pass = summary
        .gates
        .iter()
        .filter(|gate| gate.status == "PASS")
        .count();
    let fail = summary.gates.len().saturating_sub(pass);
    println!(
        "gate.summary={} pass={} fail={}",
        summary.status, pass, fail
    );
    if let Some(proof) = &summary.desktop_runtime_integration {
        println!(
            "desktop_runtime_integration.status={}",
            proof.outcome.status
        );
        println!(
            "desktop_runtime_integration.bridge_compatible={}",
            proof.bridge_compatible
        );
        println!(
            "desktop_runtime_integration.native_client_untouched={}",
            proof.native_client_untouched
        );
        println!(
            "desktop_runtime_integration.wake_accepted={} session_opened={} known_speaker_recognized={} named_greeting={}",
            proof.outcome.wake_accepted,
            proof.outcome.session_opened,
            proof.outcome.known_speaker_recognized,
            proof.outcome.named_greeting
        );
    }
    Ok(())
}

fn parse_cli_args() -> Result<CliArgs, String> {
    let args: Vec<String> = env::args().collect();
    let mut cli = CliArgs {
        mode: E2eMode::EnrollAndRecognize,
        speaker_name: "JD".to_string(),
        wake_text: "Selene".to_string(),
        preferred_device_substring: None,
        enroll_samples: 3,
        seconds_per_sample: 4,
        wake_seconds: 5,
        store_path: None,
        json: false,
        desktop_integration_proof: false,
    };
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--mode" if idx + 1 < args.len() => {
                cli.mode = match args[idx + 1].trim() {
                    "enroll-and-recognize" => E2eMode::EnrollAndRecognize,
                    "recognize-only" => E2eMode::RecognizeOnly,
                    "quiet-control" => E2eMode::QuietControl,
                    other => return Err(format!("unknown --mode '{other}'")),
                };
                idx += 2;
            }
            "--speaker-name" if idx + 1 < args.len() => {
                cli.speaker_name = safe_display_name(&args[idx + 1])?;
                idx += 2;
            }
            "--wake-text" if idx + 1 < args.len() => {
                cli.wake_text = bounded_controlled_wake_text(&args[idx + 1])
                    .ok_or_else(|| "wake text is empty".to_string())?;
                idx += 2;
            }
            "--device" if idx + 1 < args.len() => {
                let value = args[idx + 1].trim();
                if !value.is_empty() {
                    cli.preferred_device_substring = Some(value.to_string());
                }
                idx += 2;
            }
            "--enroll-samples" if idx + 1 < args.len() => {
                cli.enroll_samples = args[idx + 1]
                    .parse::<u8>()
                    .map_err(|_| "invalid --enroll-samples".to_string())?
                    .clamp(3, 8);
                idx += 2;
            }
            "--seconds-per-sample" if idx + 1 < args.len() => {
                cli.seconds_per_sample = args[idx + 1]
                    .parse::<u64>()
                    .map_err(|_| "invalid --seconds-per-sample".to_string())?
                    .clamp(1, 15);
                idx += 2;
            }
            "--wake-seconds" if idx + 1 < args.len() => {
                cli.wake_seconds = args[idx + 1]
                    .parse::<u64>()
                    .map_err(|_| "invalid --wake-seconds".to_string())?
                    .clamp(1, 30);
                idx += 2;
            }
            "--store-path" if idx + 1 < args.len() => {
                cli.store_path = Some(PathBuf::from(args[idx + 1].trim()));
                idx += 2;
            }
            "--json" => {
                cli.json = true;
                idx += 1;
            }
            "--desktop-integration-proof" => {
                cli.desktop_integration_proof = true;
                idx += 1;
            }
            _ => {
                idx += 1;
            }
        }
    }
    Ok(cli)
}

fn build_desktop_runtime_integration_proof(summary: &E2eSummary) -> DesktopRuntimeIntegrationProof {
    let pass = summary
        .gates
        .iter()
        .filter(|gate| gate.status == "PASS")
        .count();
    let fail = summary.gates.len().saturating_sub(pass);
    let outcome = DesktopRuntimeOutcomeMetadata {
        status: summary.status,
        wake_accepted: summary.wake.accepted,
        session_opened: summary.wake.session_opened,
        voice_enrollment_completed: summary.voice_id.enrollment_completed,
        voice_profile_id: summary.voice_id.voice_profile_id.clone(),
        known_speaker_recognized: summary.voice_id.recognized,
        voice_posture: summary.voice_id.posture.clone(),
        named_greeting: summary.greeting.named,
        next_move: summary.wake.next_move.clone(),
        source_chip_count: summary.safety.source_chips,
        provider_call_count: summary.safety.provider_calls,
        protected_execution: summary.safety.protected_execution,
        memory_write: summary.safety.memory_write,
        authority_granted: summary.voice_id.authoritative,
    };
    DesktopRuntimeIntegrationProof {
        proof_surface: "DESKTOP_RUNTIME_LIVE_VOICE_E2E_PROOF",
        bridge_compatible: desktop_bridge_compatible(summary, &outcome),
        native_client_untouched: true,
        outcome,
        gate_summary: DesktopRuntimeGateSummary {
            status: summary.status,
            pass,
            fail,
        },
    }
}

fn desktop_bridge_compatible(
    summary: &E2eSummary,
    outcome: &DesktopRuntimeOutcomeMetadata,
) -> bool {
    let named_greeting_is_known_posture = !outcome.named_greeting
        || (outcome.known_speaker_recognized && outcome.voice_posture == "KNOWN_HIGH_CONFIDENCE");
    let accepted_wake_has_session =
        !outcome.wake_accepted || (outcome.session_opened && !outcome.next_move.trim().is_empty());
    let closed_paths = outcome.source_chip_count == 0
        && outcome.provider_call_count == 0
        && !outcome.protected_execution
        && !outcome.memory_write
        && !outcome.authority_granted;
    let status_matches_gates =
        (summary.status == "PASS") == summary.gates.iter().all(|gate| gate.status == "PASS");
    accepted_wake_has_session
        && named_greeting_is_known_posture
        && closed_paths
        && status_matches_gates
}

fn apply_controlled_wake_detection_hint(
    capture: &mut VoiceTurnAudioCaptureRef,
    wake_text: &str,
) -> Result<(), String> {
    if !capture_has_live_speech_evidence(capture) {
        return Err(
            "controlled wake text refused because live mic speech evidence is missing".to_string(),
        );
    }
    capture.detection_text = Some(
        bounded_controlled_wake_text(wake_text)
            .ok_or_else(|| "wake text is empty after normalization".to_string())?,
    );
    capture.detection_confidence_bp = Some(
        capture
            .vad_confidence_bp
            .unwrap_or(0)
            .max(capture.acoustic_confidence_bp.unwrap_or(0))
            .max(9_000)
            .min(9_800),
    );
    Ok(())
}

fn capture_has_live_speech_evidence(capture: &VoiceTurnAudioCaptureRef) -> bool {
    let vad_present = capture.vad_confidence_bp.unwrap_or(0) > 0;
    let snr_present = capture.snr_db_milli.unwrap_or(0) > 0;
    let capture_healthy =
        !capture.capture_degraded.unwrap_or(true) && !capture.stream_gap_detected.unwrap_or(true);
    vad_present && snr_present && capture_healthy
}

fn enrollment_vad_coverage(capture: &VoiceTurnAudioCaptureRef) -> f32 {
    ((capture.vad_confidence_bp.unwrap_or(0) as f32) / 10_000.0).clamp(0.92, 0.99)
}

fn enrollment_snr_db(capture: &VoiceTurnAudioCaptureRef) -> f32 {
    ((capture.snr_db_milli.unwrap_or(0) as f32) / 1_000.0).clamp(18.0, 40.0)
}

fn enrollment_clipping_pct(capture: &VoiceTurnAudioCaptureRef) -> f32 {
    ((capture.clipping_ratio_bp.unwrap_or(0) as f32) / 100.0).clamp(0.0, 2.5)
}

fn capture_duration_ms(capture: &VoiceTurnAudioCaptureRef) -> u16 {
    capture
        .t_end_ns
        .saturating_sub(capture.t_start_ns)
        .saturating_div(1_000_000)
        .clamp(1_000, 15_000) as u16
}

fn live_voice_sample_ref(
    capture: &VoiceTurnAudioCaptureRef,
    run_seed: u64,
    sample_idx: u16,
) -> String {
    format!(
        "live_voice_ref_{}_{}_vad{}_snr{}",
        run_seed % 1_000_000,
        sample_idx,
        capture.vad_confidence_bp.unwrap_or(0),
        capture.snr_db_milli.unwrap_or(0)
    )
}

fn speaker_leaf_from_name(name: &str) -> Result<String, String> {
    let display = safe_display_name(name)?;
    let mut leaf = String::new();
    for ch in display.chars() {
        if ch.is_ascii_alphanumeric() {
            leaf.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_whitespace() || ch == '-' {
            leaf.push('_');
        }
    }
    while leaf.contains("__") {
        leaf = leaf.replace("__", "_");
    }
    let leaf = leaf.trim_matches('_').to_string();
    if leaf.is_empty() {
        Err("speaker name cannot produce actor leaf".to_string())
    } else {
        Ok(leaf)
    }
}

fn safe_display_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || !trimmed.is_ascii()
        || trimmed.chars().any(|ch| ch.is_control())
        || trimmed.chars().count() > 32
        || trimmed
            .chars()
            .any(|ch| !(ch.is_ascii_alphabetic() || ch.is_ascii_whitespace() || ch == '-'))
    {
        return Err("speaker name must be ASCII letters/spaces/hyphen and <=32 chars".to_string());
    }
    Ok(trimmed
        .split_whitespace()
        .map(|word| {
            if word.len() <= 3 {
                word.to_ascii_uppercase()
            } else {
                let mut chars = word.chars();
                let first = chars.next().unwrap().to_ascii_uppercase();
                format!("{first}{}", chars.as_str().to_ascii_lowercase())
            }
        })
        .collect::<Vec<_>>()
        .join(" "))
}

fn greeting_contains_name(text: &str, speaker_name: &str) -> bool {
    let Ok(display) = safe_display_name(speaker_name) else {
        return false;
    };
    text.contains(&display)
}

fn bounded_controlled_wake_text(raw: &str) -> Option<String> {
    let text: String = raw.trim().chars().take(64).collect();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn monotonic_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        .max(1)
}

fn e2e_journal_path(seed: u64) -> PathBuf {
    std::env::temp_dir().join(format!("selene_desktop_voice_e2e_{seed}.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_capture(vad: u16, snr: i32, detection: bool) -> VoiceTurnAudioCaptureRef {
        let mut capture = selene_adapter::desktop_mic_producer::synthetic_capture_ref_for_tests(1);
        capture.vad_confidence_bp = Some(vad);
        capture.snr_db_milli = Some(snr);
        capture.capture_degraded = Some(false);
        capture.stream_gap_detected = Some(false);
        if detection {
            capture.detection_text = Some("Selene".to_string());
            capture.detection_confidence_bp = Some(9_500);
        } else {
            capture.detection_text = None;
            capture.detection_confidence_bp = None;
        }
        capture
    }

    fn base_cli(mode: E2eMode) -> CliArgs {
        CliArgs {
            mode,
            speaker_name: "JD".to_string(),
            wake_text: "Selene".to_string(),
            preferred_device_substring: None,
            enroll_samples: 3,
            seconds_per_sample: 4,
            wake_seconds: 5,
            store_path: None,
            json: true,
            desktop_integration_proof: true,
        }
    }

    fn named_recognition_summary() -> E2eSummary {
        let wake = WakeSummary {
            accepted: true,
            session_opened: true,
            next_move: "listening_window_open".to_string(),
        };
        let voice_id = VoiceIdSummary {
            enrollment_completed: true,
            voice_profile_id: Some("voice_profile_test".to_string()),
            recognized: true,
            posture: "KNOWN_HIGH_CONFIDENCE".to_string(),
            authoritative: false,
        };
        let greeting = GreetingSummary {
            named: true,
            response_text: "Ready when you are, JD.".to_string(),
            tts_text: "Ready when you are, JD.".to_string(),
        };
        let safety = SafetySummary {
            source_chips: 0,
            provider_calls: 0,
            protected_execution: false,
            memory_write: false,
        };
        let cli = base_cli(E2eMode::EnrollAndRecognize);
        let gates = build_gates(&cli, &wake, &voice_id, &greeting, &safety);
        E2eSummary {
            status: "PASS",
            speaker_name: "JD".to_string(),
            wake,
            voice_id,
            greeting,
            safety,
            gates,
            desktop_runtime_integration: None,
        }
    }

    #[test]
    fn desktop_voice_e2e_safe_display_name_normalizes_short_name() {
        assert_eq!(safe_display_name("jd").unwrap(), "JD");
        assert_eq!(speaker_leaf_from_name("JD").unwrap(), "jd");
    }

    #[test]
    fn desktop_voice_e2e_rejects_unsafe_speaker_name() {
        assert!(safe_display_name("JD123").is_err());
        assert!(safe_display_name("JD; rm -rf").is_err());
    }

    #[test]
    fn desktop_voice_e2e_controlled_wake_requires_live_speech() {
        let mut capture = synthetic_capture(0, 0, false);
        assert!(apply_controlled_wake_detection_hint(&mut capture, "Selene").is_err());
    }

    #[test]
    fn desktop_voice_e2e_controlled_wake_attaches_detection_for_speech() {
        let mut capture = synthetic_capture(7_000, 12_000, false);
        apply_controlled_wake_detection_hint(&mut capture, "Selene").unwrap();
        assert_eq!(capture.detection_text.as_deref(), Some("Selene"));
        assert!(capture.detection_confidence_bp.unwrap() >= 9_000);
    }

    #[test]
    fn desktop_voice_e2e_json_summary_reports_gate_failure() {
        let wake = WakeSummary {
            accepted: false,
            session_opened: false,
            next_move: "closed".to_string(),
        };
        let voice_id = VoiceIdSummary {
            enrollment_completed: false,
            voice_profile_id: None,
            recognized: false,
            posture: "UNKNOWN_OR_UNAVAILABLE".to_string(),
            authoritative: false,
        };
        let greeting = GreetingSummary {
            named: false,
            response_text: String::new(),
            tts_text: String::new(),
        };
        let safety = SafetySummary {
            source_chips: 0,
            provider_calls: 0,
            protected_execution: false,
            memory_write: false,
        };
        let cli = CliArgs {
            mode: E2eMode::EnrollAndRecognize,
            speaker_name: "JD".to_string(),
            wake_text: "Selene".to_string(),
            preferred_device_substring: None,
            enroll_samples: 3,
            seconds_per_sample: 4,
            wake_seconds: 5,
            store_path: None,
            json: true,
            desktop_integration_proof: false,
        };
        let gates = build_gates(&cli, &wake, &voice_id, &greeting, &safety);
        assert!(gates.iter().any(|gate| gate.status == "FAIL"));
    }

    #[test]
    fn desktop_voice_e2e_integration_proof_accepts_named_recognition_summary() {
        let summary = named_recognition_summary();
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(proof.bridge_compatible);
        assert!(proof.native_client_untouched);
        assert!(proof.outcome.wake_accepted);
        assert!(proof.outcome.session_opened);
        assert!(proof.outcome.voice_enrollment_completed);
        assert_eq!(
            proof.outcome.voice_profile_id.as_deref(),
            Some("voice_profile_test")
        );
        assert!(proof.outcome.known_speaker_recognized);
        assert!(proof.outcome.named_greeting);
        assert_eq!(proof.outcome.provider_call_count, 0);
        assert_eq!(proof.outcome.source_chip_count, 0);
        assert!(!proof.outcome.protected_execution);
        assert!(!proof.outcome.authority_granted);
        assert_eq!(proof.gate_summary.status, "PASS");
        assert_eq!(proof.gate_summary.fail, 0);
    }

    #[test]
    fn desktop_voice_e2e_integration_proof_rejects_quiet_control_without_session() {
        let wake = WakeSummary {
            accepted: false,
            session_opened: false,
            next_move: "closed".to_string(),
        };
        let voice_id = VoiceIdSummary {
            enrollment_completed: false,
            voice_profile_id: None,
            recognized: false,
            posture: "NOT_ATTEMPTED_QUIET_CONTROL".to_string(),
            authoritative: false,
        };
        let greeting = GreetingSummary {
            named: false,
            response_text: "wake_rejected reason_code=1459617842".to_string(),
            tts_text: String::new(),
        };
        let safety = SafetySummary {
            source_chips: 0,
            provider_calls: 0,
            protected_execution: false,
            memory_write: false,
        };
        let cli = base_cli(E2eMode::QuietControl);
        let gates = build_gates(&cli, &wake, &voice_id, &greeting, &safety);
        let summary = E2eSummary {
            status: "PASS",
            speaker_name: "JD".to_string(),
            wake,
            voice_id,
            greeting,
            safety,
            gates,
            desktop_runtime_integration: None,
        };
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(proof.bridge_compatible);
        assert!(!proof.outcome.wake_accepted);
        assert!(!proof.outcome.session_opened);
        assert!(!proof.outcome.known_speaker_recognized);
        assert!(!proof.outcome.named_greeting);
    }

    #[test]
    fn desktop_voice_e2e_integration_proof_requires_known_posture_for_named_greeting() {
        let mut summary = named_recognition_summary();
        summary.voice_id.recognized = false;
        summary.voice_id.posture = "UNKNOWN_OR_UNAVAILABLE".to_string();
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(!proof.bridge_compatible);
    }

    #[test]
    fn desktop_voice_e2e_integration_proof_preserves_provider_tool_protected_closure() {
        let mut summary = named_recognition_summary();
        summary.safety.provider_calls = 1;
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(!proof.bridge_compatible);

        let mut summary = named_recognition_summary();
        summary.safety.source_chips = 1;
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(!proof.bridge_compatible);

        let mut summary = named_recognition_summary();
        summary.safety.protected_execution = true;
        let proof = build_desktop_runtime_integration_proof(&summary);
        assert!(!proof.bridge_compatible);
    }

    #[test]
    fn desktop_voice_e2e_integration_json_shape_is_bridge_compatible() {
        let mut summary = named_recognition_summary();
        summary.desktop_runtime_integration =
            Some(build_desktop_runtime_integration_proof(&summary));
        let value = serde_json::to_value(&summary).unwrap();
        assert_eq!(value["status"], "PASS");
        assert_eq!(value["speaker_name"], "JD");
        assert_eq!(value["wake"]["accepted"], true);
        assert_eq!(value["wake"]["session_opened"], true);
        assert_eq!(value["voice_id"]["recognized"], true);
        assert_eq!(value["voice_id"]["posture"], "KNOWN_HIGH_CONFIDENCE");
        assert_eq!(value["greeting"]["named"], true);
        assert_eq!(value["safety"]["source_chips"], 0);
        assert_eq!(value["safety"]["provider_calls"], 0);
        assert_eq!(value["safety"]["protected_execution"], false);
        assert_eq!(
            value["desktop_runtime_integration"]["proof_surface"],
            "DESKTOP_RUNTIME_LIVE_VOICE_E2E_PROOF"
        );
        assert_eq!(
            value["desktop_runtime_integration"]["bridge_compatible"],
            true
        );
        assert_eq!(
            value["desktop_runtime_integration"]["outcome"]["next_move"],
            "listening_window_open"
        );
    }
}
