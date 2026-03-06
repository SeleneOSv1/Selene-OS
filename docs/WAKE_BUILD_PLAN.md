# Selene Wake System - Current Build Plan (Aligned With Repo)

## Section 1: Purpose of Wake

Wake has one job only.

Wake exists to open or resume a Selene session so the user can start speaking.

Wake does not process commands, reasoning, or responses.

Flow:
Trigger -> Open/Resume Session -> Wake job finished -> Voice pipeline continues.

After session opens the system continues:
Voice ID -> STT -> PH1.X -> tools -> response.

## Section 2: Where Wake Runs

Wake behavior depends on the platform.

Desktop

Uses wake phrase ("Selene").

Always listening runtime required.

First time desktop is used -> wake training must run.

Android

Uses wake phrase.

Must run in low-power always-listening mode.

Wake training required during onboarding.

iPhone

Wake phrase is not used.

Uses side button trigger only.

iOS onboarding requires "ios_side_button_configured" receipt.

All triggers result in the same action:

Trigger -> Session Open/Resume -> Voice pipeline starts.

Client/server trigger contract (TARGET / OPEN / NOT BUILT YET):

Transport and auth:
- HTTPS `POST /v1/voice/turn`
- `Authorization: Bearer <device_token>`
- `Content-Type: application/json`

Request payload (VoiceTurnRequestV1):
- `request_id` (UUID)
- `idempotency_key` (UUID)
- `device_id` (string)
- `app_platform` (`DESKTOP | ANDROID | IOS`)
- `trigger` (`EXPLICIT | WAKE_WORD`)
- `timestamp_ms` (u64)
- `session_id` (optional string)
- `processed_stream_ref` (optional string)
- `pre_roll_buffer_ref` (optional string)
- `audio_capture_ref` (optional string)

Response payload (VoiceTurnResponseV1):
- `session_id` (string)
- `turn_id` (string)
- `trigger_accepted` (bool)
- `next_state` (`STREAMING | REJECTED | RETRYABLE`)
- `wake_decision_ref` (optional string)
- `error_code` (optional string)

Response semantics:
- `200`: session opened/resumed and turn accepted.
- `409`: duplicate `idempotency_key`; return prior `session_id` and `turn_id`.
- `422`: gating failure (missing required onboarding receipt).
- `401/403`: auth failure.

Ingress security (required):
- Validate token subject == actor_user_id and token device claim == device_id
- Enforce signed nonce + timestamp replay window (deterministic stale/replay rejects)
- Enforce per-token/per-device quotas with deterministic 429 + Retry-After
- Remove / forbid auto-provisioning of identity/device inside /v1/voice/turn

Platform runtime budgets (hard requirements):

Desktop budgets:
- Wake-to-session-open latency p95 <= 350 ms.
- Listener CPU <= 2.0% average (5 minute window).
- Listener memory RSS <= 80 MB.
- Battery drain on laptops <= 1.5% per hour screen-off equivalent.

Android budgets:
- Wake-to-session-open latency p95 <= 500 ms.
- Listener CPU <= 3.0% average (5 minute window).
- Listener memory RSS <= 64 MB.
- Battery drain <= 2.0% per hour in always-listening mode.

Hard fail thresholds:
- Desktop CPU > 4.0% for 5 minutes or RSS > 120 MB.
- Android CPU > 5.0% for 5 minutes or RSS > 96 MB.
- Battery drain above target by > 50% for two consecutive 30 minute windows.

## Section 2A: Universal Invite Link Architecture (TARGET / OPEN / NOT BUILT YET)

A) Canonical invite link

- One universal invite link format: `https://<selene_host>/invite/<token>`.
- One invite token source of truth: token resolution is authoritative for inviter, invitee, tenant, expiry, platform permissions, and ownership mode.
- Token resolution must return deterministic app-open context used by `LINK_INVITE_OPEN_ACTIVATE_COMMIT` and `ONB_SESSION_START_DRAFT`.

B) Selene intro step

- First page introduces Selene and identifies who sent the invite.
- User must explicitly approve continue before install/open routing.
- Intro step detects platform and device class (`enterprise-managed iPhone`, `personal iPhone`, `Desktop`) and selects install/open route.
- Invite token is preserved unchanged through intro and routing.

C) Routing matrix

1. Enterprise-managed iPhone
- Route through enterprise-managed install/assignment path (MDM-owned).
- After install/open, app re-enters with invite token and calls `/v1/invite/click`.
- Continue canonical onboarding through `/v1/onboarding/continue`.
- iPhone remains `EXPLICIT` only; `WAKE_WORD` is blocked.

2. Personal iPhone
- Route to App Store/TestFlight install path.
- After install/open, app re-enters through universal/deep link carrying invite token.
- Continue canonical onboarding through `/v1/invite/click` then `/v1/onboarding/continue`.
- iPhone remains `EXPLICIT` only; `WAKE_WORD` is blocked.

3. Desktop
- Route to desktop app open/installer path.
- If app is already installed, deep-link directly with invite token.
- Continue canonical onboarding through `/v1/invite/click` then `/v1/onboarding/continue`.
- Desktop wake enrollment remains required before `WAKE_WORD` is enabled.

D) Unified onboarding rule

- All invite paths converge into one canonical onboarding engine.
- Platform-specific receipts and gating are enforced only inside the canonical onboarding engine.
- No duplicate onboarding systems allowed.

E) Ownership model

- `enterprise-managed device`: enterprise controls install, assignment, and policy envelope before onboarding continue.
- `personal device`: user-controlled install path, then token-bound onboarding under tenant policy.
- `desktop device`: direct app open/install path, then token-bound onboarding with desktop wake enrollment requirement.

F) Hard rules

- one universal invite link only
- one invite token source of truth only
- one onboarding engine only
- install path may differ, onboarding engine must not fork
- iPhone wake word remains blocked
- desktop wake training remains required

Copy-ready flow blocks:

Enterprise-managed iPhone:
Invite link -> Selene intro -> enterprise approval -> MDM install/assign -> app opens -> onboarding -> ios_side_button_configured -> ready

Personal iPhone:
Invite link -> Selene intro -> App Store/TestFlight -> app install -> app opens -> onboarding -> ios_side_button_configured -> ready

Desktop:
Invite link -> Selene intro -> desktop app open/install -> onboarding -> wake enrollment -> ready

## Section 3: Current Repo Reality

From the current repository state:

What already exists:

- Full onboarding state machine (PH1.ONB)
- Wake enrollment storage and tables
- Wake sample quality metrics (SNR, VAD coverage, clipping)
- Wake enrollment sessions and samples
- Wake artifact generation and binding
- Artifact ledger and versioning
- Reliable mobile artifact outbox
- Cloud sync worker

What is now implemented in runtime:

- Live `WAKE_WORD` evaluation runs PH1.W inference in adapter turn flow using PH1.K capture refs.
- Synthetic adapter wake acceptance is removed; reject decisions fail-closed before session open.

## Section 4: Wake Training During Onboarding

Wake training is already wired in the database layer.

Tables:

- `wake_enrollment_sessions`
- `wake_enrollment_samples`
- `wake_runtime_events`
- `wake_profile_bindings`

Training flow:

Onboarding starts wake enrollment session.

User records multiple samples.

Sample quality metrics are recorded.

Enrollment completes.

Wake artifact receipt is generated.

Artifact queued to mobile sync outbox.

Onboarding gate matrix (target):

Current enforcement now matches the strict Android/Desktop wake completion gate and iOS side-button-only gate (Gap 6 closed in e8da92e).

- Desktop:
  - Required receipts: `wake_enrollment_completed`, `wake_artifact_sync_receipt`.
  - `WAKE_WORD` blocked until receipts are present.
- Android:
  - Required receipts: `wake_enrollment_completed`, `wake_artifact_sync_receipt`.
  - `WAKE_WORD` blocked until receipts are present.
- iOS:
  - Required receipt: `ios_side_button_configured`.
  - `WAKE_WORD` disabled.

Bypass policy:

- Temporary bypass allowed only via signed receipt `wake_training_bypass_v1`.
- Bypass TTL: 24 hours.
- Bypass scope: `EXPLICIT` trigger only; `WAKE_WORD` remains disabled.

Recovery path for failed enrollment:

- Set onboarding state to `WakeEnrollPending`.
- Keep session onboarding active until receipt is recovered.
- Re-enter wake enrollment step automatically on next onboarding continue.
- Allow explicit trigger path for rescue flow while wake remains gated.

Privacy and retention for onboarding capture:

- Store quality features and bounded enrollment windows.
- Raw enrollment audio retention TTL: 30 days max.
- Feature/embedding retention TTL: 365 days max.
- Encryption at rest for stored audio/features is mandatory.
- User delete request must remove enrollment samples, bindings, and derived wake profile artifacts.

Retention & deletion execution (required):
- Purge workers (cadence defined) for TTL cleanup
- User-delete execution flow for wake enrollment/runtime/artifacts
- Deletion receipts + audit verification across all wake tables and artifact payload refs

## Section 5: Wake Artifact System

Wake models are stored as artifacts.

Artifacts contain:

- `artifact_version`
- `model_abi`
- `package_hash`
- `payload_ref`

WakePack manifest (TARGET / OPEN / NOT BUILT YET):

- `model_version`
- `model_abi`
- `feature_config_id`
- `threshold_profile_id`
- `artifact_version`
- `package_hash`
- `payload_ref`
- `provenance_ref`
- `train_data_snapshot_id`
- `eval_metrics_summary`
- `rollback_to_artifact_version`

Artifacts are synced through the Mobile Artifact Sync system.

Outbox states:

Queued -> InFlight -> Acked -> DeadLetter.

Sync worker pushes artifacts to cloud via HTTP endpoint.

Artifact compatibility contract:

- Runtime loads artifact only if `model_abi.major == runtime_abi.major`.
- Runtime loads artifact only if `model_abi.minor <= runtime_abi.minor`.
- `package_hash` must match downloaded payload hash before activation.
- On compatibility failure, artifact is rejected and previous active artifact is retained.

Rollout policy:

- Shadow rollout: 5% devices.
- Canary rollout: 20% devices.
- Full rollout: 100% after canary gates pass.

Rollback triggers:

- FAR regression > 30% vs control in 24 hour window.
- Miss rate regression > 20% vs control in 24 hour window.
- Runtime crash increase > 0.10% absolute in 24 hour window.

Downgrade semantics:

- Revert to last known good artifact with compatible ABI.
- Mark failed artifact version as blocked for 24 hours minimum.
- Preserve idempotency and receipt lineage across downgrade.

Promotion + rollback hardening (required):

- Replay-safe canary cohorts must use deterministic cohort assignment and stable cohort keys.
- Failed versions must enter a blocked-version list before retrying activation.
- A rollback drill must pass before full promotion.
- Full promotion must verify `last_known_good` pointer integrity for every activated device.

Artifact privacy and deletion:

- Artifact packages must be encrypted in transit and at rest.
- Artifact metadata may be retained for audit; payloads follow data retention policy.
- Delete path must remove artifact payloads linked to deleted wake profiles.

## Section 6: Wake Runtime Events

Wake runtime stores events including:

- wake accepted
- wake rejected
- rejection reason codes

Examples:

- `W_FAIL_G1_NOISE`
- `W_FAIL_G3A_REPLAY_SUSPECTED`
- `W_WAKE_REJECTED_TIMEOUT`

Events are committed through:

`ph1w_runtime_event_commit`.

Runtime event required fields:

- `event_id` (UUID)
- `idempotency_key` (UUID)
- `device_id`
- `profile_id` (optional)
- `model_version`
- `decision` (`ACCEPT | REJECT`)
- `score`
- `threshold_used`
- `reason_code` (optional)
- `detected_at_ms`
- `audio_window_start_ms`
- `audio_window_end_ms`

- Introduce wake_window_id propagated through runtime events, learn signals, outbox records, and cloud ACKs for deterministic traceability.

## Section 7: Wake Failure Learning

Learning pipeline exists and is expanded with explicit wake taxonomy.

Required wake learning event taxonomy:

- `WakeAccepted`
- `WakeRejected`
- `FalseWake`
- `MissedWake`
- `LowConfidenceWake`
- `NoisyEnvironment`

Deterministic generation rule (required):
- Map PH1.W reason_code + trigger/policy context -> WakeLearnSignalV1.event_type + required fields
- Emit exactly one learn signal per wake decision window

Wake learn signal wire format (`WakeLearnSignalV1`):

- `signal_id` (UUID)
- `idempotency_key` (UUID)
- `event_type` (enum above)
- `device_id`
- `session_id` (optional)
- `trigger` (`WAKE_WORD | EXPLICIT`)
- `model_version`
- `score` (optional)
- `threshold` (optional)
- `reason_code` (optional)
- `snr_db` (optional)
- `vad_coverage` (optional)
- `timestamp_ms`

Outbox contract for learning signals:

- Outbox key is `device_id + signal_id`.
- Duplicate inserts with same key are no-op idempotent commits.
- Signals must be written before enqueue.

Retry/backoff policy:

- Retry delays: 1s, 5s, 15s, 60s, 5m, 15m.
- After max retries, move item to `DeadLetter`.
- Dead letter requires explicit replay command.

Cloud ACK/NACK contract (`WakeLearnAckV1`):

- `ACK`: accepted and persisted; outbox item transitions to `Acked`.
- `NACK_RETRYABLE`: include `retry_after_ms`; outbox item stays retryable.
- `NACK_FATAL`: include `error_code`; outbox item transitions to `DeadLetter`.

## Section 8: Wake Outbox Reliability

The device outbox is already implemented.

Features:

- Offline first
- Retry scheduling
- Idempotency keys
- Dead letter queue
- Batch dequeue
- HTTP sync worker

This system is currently used to send wake artifacts, device sync events, and wake learning signals.
Wake learning signal outbox/ACK-NACK is CLOSED in e8dc612.

## Section 9: Real Wake Detection (Live Route Implemented)

`WAKE_WORD` calls `Ph1wRuntime` inference; `ACCEPT` continues and `REJECT` prevents session open.

Strict PH1.W runtime contract (required):

Input packet (`WakeInferenceRequestV1`):

- `request_id` (UUID)
- `idempotency_key` (UUID)
- `device_id`
- `app_platform`
- `trigger`
- `pcm_encoding` (`S16LE`)
- `sample_rate_hz` (`16000`)
- `channels` (`1`)
- `frame_ms` (`20`)
- `window_ms` (`1500`)
- `hop_ms` (`200`)
- `pre_roll_ms` (`1200`)
- `processed_stream_ref`
- `pre_roll_buffer_ref`
- `model_version`
- `threshold_profile_ref`
- `captured_at_ms`

Output packet (`WakeDecisionV1`):

- `decision` (`ACCEPT | REJECT`)
- `score` (f32)
- `threshold_used` (f32)
- `reason_code` (optional)
- `model_version`
- `detected_at_ms`
- `audio_window_start_ms`
- `audio_window_end_ms`
- `idempotency_key`

Reject reason code requirements:

- Every reject must include one stable reason code.
- Timeout rejects must use `W_WAKE_REJECTED_TIMEOUT`.
- Unknown internal errors must map to `W_WAKE_REJECTED_INTERNAL`.

## Section 10: Audio Runtime

Wake requires a real microphone runtime.

Components required:

- microphone capture
- ring buffer
- pre-roll buffer
- audio feature extraction

Current repo runtime state:

- Desktop real microphone runtime is implemented (`desktop_mic_producer`) and provides capture refs for live wake (`WAKE_WORD`) evaluation.
- Android microphone runtime remains OPEN / NOT BUILT YET.

Audio runtime implementation requirements:

- Capture format: PCM 16-bit signed little-endian, mono, 16 kHz.
- Ring buffer minimum capacity: 3 seconds.
- Pre-roll minimum retained audio: 1.2 seconds.
- Post-wake capture minimum: 1.0 second.
- Stream clock drift correction required for sessions > 5 minutes.
- VAD must provide frame-level speech probability for wake quality metrics.

Desktop/Android PH1.K producer contract:
- Owns microphone lifecycle and capture thread(s)
- Owns ring buffer + pre-roll fill guarantees
- Emits health telemetry (clipping, dropout, SNR estimates where available)
- Must fail-closed on incomplete/invalid capture bundles before WAKE_WORD inference

## Section 11: Wake Detection Model

Wake detection must use a small keyword spotting model.

Design:

Audio window -> feature extraction -> wake model -> score -> threshold.

Wake fires when threshold passes.

Low-power requirement:

Use DSP / NPU / optimized inference where available.

Detection model requirements:

- Input features: log-mel or MFCC with fixed frame/hop parameters.
- Model output score range must be normalized to [0.0, 1.0].
- Threshold profile must be versioned and tied to `model_version`.
- Threshold updates require artifact ledger commit and rollout policy compliance.

Wake training dataset builder contract (TARGET / OPEN / NOT BUILT YET):

Inputs:
- onboarding wake enrollment samples
- wake runtime events
- wake learn signals
- curated non-wake speech/background negatives

Dataset construction rules:
- positives = completed wake enrollment `PASS` samples + `WakeAccepted` windows.
- negatives = `WakeRejected` / `NoisyEnvironment` windows + curated non-wake speech.
- `FalseWake` / `MissedWake` adjudications join once verified evidence exists.
- train/val/test must enforce no user leakage.
- train/val/test must enforce no device leakage where possible.
- train/val/test must enforce no time-adjacent leakage.

Split policy:
- global corpus for shared wake robustness.
- per-user adaptation corpus for personalized thresholds/features.
- per-platform validation slices (`DESKTOP`, `ANDROID`) with policy-equivalent trigger context.

Feature extraction lock (TARGET / OPEN / NOT BUILT YET):

Locked default:
- `sample_rate = 16000`
- mono audio
- `frame_ms = 25`
- `hop_ms = 10`
- `mel_bins = 40`
- primary feature = log-mel
- fallback feature = MFCC-13 + deltas (only if needed)

Normalization rules:
- gain normalization to bounded target RMS before feature extraction.
- per-utterance CMVN is mandatory.
- `feature_config_id` must be embedded in WakePack manifests and runtime telemetry.

Model architecture decision (TARGET / OPEN / NOT BUILT YET):

Primary model:
- DS-CNN (depthwise separable CNN), INT8-ready.

Fallback model:
- compact CNN-LSTM.

Architecture rationale:
- DS-CNN is primary for low-latency and low-power always-listening suitability on desktop now and Android next.
- CNN-LSTM is fallback when temporal robustness gains justify higher latency/power cost.

Expected tradeoffs:
- latency: DS-CNN lower, CNN-LSTM higher.
- power: DS-CNN lower, CNN-LSTM higher.
- robustness: CNN-LSTM can improve temporal/noise resilience.
- platform suitability: DS-CNN preferred for mobile/edge constraints; CNN-LSTM limited to gated fallback profiles.

## Section 12: Wake Learning Loop

Continuous learning pipeline:

Device detects wake
-> failures logged
-> signals stored
-> signals sent through outbox
-> cloud aggregates signals
-> improved wake artifact produced
-> artifact synced back to device.

Cloud-side integration contract:

- Ingest endpoint must accept `WakeLearnSignalV1` batches with idempotency keys.
- Aggregation output must emit a versioned wake artifact package.
- Promotion from shadow/canary to active must pass rollout gates.
- Device pull must verify package hash and ABI compatibility before apply.

Device apply/activation state machine (required):
- Staged -> Shadow -> Active -> RolledBack
- Persist last_known_good pointer
- Automatic rollback triggers on quality regressions and crash regressions

Offline training boundary (hard rule):

- Training and evaluation are `OFFLINE_PIPELINE_ONLY`.
- Runtime never retrains in-turn.
- Runtime only consumes verified active WakePack artifacts.
- Runtime emits events/signals; offline pipeline converts them into candidate artifacts.

Training pipeline role split (TARGET / OPEN / NOT BUILT YET):

- PH1.LEARN aggregates signals.
- Training job builds candidate WakePack artifacts.
- Builder/governance validates candidate artifacts and rollout eligibility.
- Artifact ledger commit records approved candidates and lineage.
- Promotion path executes `Shadow -> Canary -> Active` with deterministic gates.
- Device pull/apply activates approved artifacts with rollback safety.

## Section 13: Wake Accuracy Stabilization

Wake quality is measured by explicit metrics and windows.

Training candidate required evaluation metrics (TARGET / OPEN / NOT BUILT YET):

- FAR (false accepts per listening hour).
- FRR (false rejects for true wake attempts).
- `miss_rate` for true wake opportunities.
- wake decision latency proxy.
- threshold calibration quality.
- reject reason distribution.

Evaluation protocol:

- Fixed held-out validation sets are mandatory and excluded from training.
- Platform-specific held-out slices are mandatory (`DESKTOP`, `ANDROID`).
- Promotion is blocked if any required evaluation metric regresses beyond configured thresholds.
- Threshold calibration outputs must be versioned and tied to `threshold_profile_id`.

Per-device rolling 7 day metrics:

- FAR (false accepts per listening hour) <= 0.05.
- FRR (false rejects for true wake attempts) <= 2.0%.
- Miss rate (missed wake events) <= 1.5%.
- Wake-to-session-open latency p95 <= platform budget.

Global rolling 30 day metrics:

- FAR <= 0.08.
- FRR <= 2.5%.
- Miss rate <= 2.0%.

Stabilization criteria:

- Personal wake profile considered stable after 14 consecutive days meeting per-device targets.
- After stabilization, only global model improvements are auto-applied unless local quality regresses.

## Section 14: Final Runtime Flow

Final target runtime sequence:

PH1.K -> PH1.W -> PH1.L -> PH1.VOICE.ID -> PH1.C -> PH1.X

Wake's job ends after PH1.L session open.

For WAKE_WORD turns, PH1.W decision is evaluated before PH1.L.
If REJECT, return wake_rejected and do not open/resume session.
If ACCEPT, pass WakeDecision into PH1.L to open/resume session.
After PH1.L emits session state, wake processing ends and PH1.VOICE.ID/PH1.C/PH1.X own the turn.

Runtime state machine for wake/session handoff:

States:

- `Idle`
- `Listening`
- `WakeDetected`
- `SessionOpening`
- `Streaming`

Transitions:

- `Idle -> Listening`: microphone runtime ready.
- `Listening -> WakeDetected`: PH1.W emits `WakeDecisionV1(decision=ACCEPT)`.
- `WakeDetected -> SessionOpening`: open/resume session request dispatched.
- `SessionOpening -> Streaming`: session open/resume succeeds and turn starts.
- `SessionOpening -> Listening`: session open timeout or hard reject.
- `Streaming -> Listening`: turn completed, canceled, or session soft-close.

Duplicate trigger handling:

- If same `idempotency_key` arrives within 5 seconds, do not create a second session/turn.
- Return prior `session_id` and `turn_id`.

Timeout behavior:

- PH1.W inference timeout: 250 ms max per decision window.
- Session open/resume timeout: 2000 ms max.
- Timeout must emit runtime event with stable reason code.

## Section 15: Global Wake Quality Target

Wake must achieve:

- extremely low false triggers
- extremely low missed triggers
- instant response
- minimal power usage
- reliable operation everywhere

Operational acceptance criteria:

- Metrics in Section 13 pass for both per-device and global windows.
- Platform budgets in Section 2 are within limits.
- No open P0 wake defects at release cut.

## Section 16: Codex Development Rule - Clean Tree

Every Codex task must follow this rule:

Start with clean repo
-> perform work
-> run CI gate checks
-> commit changes
-> push changes
-> end with clean repo.

Commands:

Start:
- `git status --short`

CI gate checks (required before commit):
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- wake perf/power smoke checks from Section 18

End:
- commit + push -> verify clean tree with `git status --short`

This rule is mandatory.

## Section 17: Next Build Steps (Phased Milestones)

Phase 1 - Runtime wake path wiring

Owner: PH1.W runtime team

Dependencies:
- Section 9 PH1.W contract finalized.
- Section 10 microphone runtime requirements accepted.

Scope:
- Wire `WAKE_WORD` trigger to real `Ph1wRuntime` inference.
- Implement desktop always-on microphone runtime.
- Implement Android wake runtime.

Done criteria:
- End-to-end wake-to-session flow works on desktop and Android.
- Duplicate/timeout behavior matches Section 14.

Phase 2 - Learning and feedback completeness

Owner: PH1 feedback and storage team

Dependencies:
- Phase 1 runtime events available.

Scope:
- Add explicit `FalseWake`, `MissedWake`, `LowConfidenceWake` signals.
- Enforce Section 7 wire format and outbox behavior.

Done criteria:
- Learning signals are generated, persisted, queued, synced, and ACK/NACK handled correctly.

Phase 3 - Artifact improvement and safe rollout

Owner: artifact and cloud sync team

Dependencies:
- Phase 2 signals available in cloud ingest.

Scope:
- Enable artifact improvement loop through cloud.
- Enforce compatibility, canary/shadow rollout, rollback rules.

Done criteria:
- Improved artifacts are produced, promoted, downloaded, verified, and applied with rollback safety.

## Section 18: Required Test Matrix and Release Gates

Mandatory test matrix:

- Unit tests:
  - PH1.W request/response schema validation.
  - Threshold and reason-code mapping.
  - Onboarding gating matrix validation.
- Integration tests:
  - PH1.K -> PH1.W -> PH1.L handoff.
  - Duplicate trigger idempotency behavior.
  - Outbox enqueue/dequeue/retry/dead-letter transitions.
- End-to-end tests:
  - Desktop wake phrase -> session open -> first turn.
  - Android wake phrase -> session open -> first turn.
  - iOS side button trigger -> session open -> first turn.
- Performance tests:
  - Wake-to-session-open latency p50/p95/p99 by platform.
  - Inference timeout compliance.
- Power tests:
  - 30 minute and 2 hour battery drain checks in always-listening mode.
  - CPU/RSS budget compliance checks.

Release gate pass criteria:

- 100% pass on mandatory unit/integration/e2e suites.
- Performance and power metrics within Section 2 and Section 13 limits.
- No P0/P1 wake defects open.
- Rollback drill executed successfully for current candidate artifact.

## Section 19: Current Repo Gap Closure Matrix (Mandatory)

This section records the concrete implementation gaps that must be closed before the wake plan can be considered end-to-end complete.

Gap 1 - Live wake trigger currently synthesizes acceptance: CLOSED in b54a3e512317a308930a0021f62889e3bbb219ae - Live `WAKE_WORD` decisions now come from `Ph1wRuntime` inference output in adapter turn flow.
- Repo anchor: `crates/selene_adapter/src/lib.rs::run_voice_turn_internal`, `evaluate_wake_for_turn`.

Gap 2 - PH1.W runtime exists but is not wired into live turn route: CLOSED in b54a3e512317a308930a0021f62889e3bbb219ae - PH1.W is invoked in live adapter turn flow before PH1.L session-open handling.
- Repo anchor: `crates/selene_engines/src/ph1w.rs::Ph1wRuntime`, `crates/selene_adapter/src/lib.rs::run_voice_turn_internal`.

Gap 3 - Platform microphone runtime parity: PARTIAL (desktop closed, Android open):
- Desktop path CLOSED in 7a1f03d4f5b3bc457a6b7fd12d8919fb7c845ca3 and b3040e636d58ffcf8f73ddff7430fe6a9769a5f2 (`desktop_mic_producer` + live wake decision window tuning).
- Current state: adapter still expects `audio_capture_ref` on live voice turn ingress; Android producer remains OPEN.
- Repo anchor: `crates/selene_adapter/src/desktop_mic_producer.rs`, `crates/selene_adapter/src/lib.rs::VoiceTurnAudioCaptureRef`, `build_ph1k_live_signal_bundle`.
- Required change: implement Android microphone runtime producer with parity to desktop capture contract.

Gap 4 - iOS side-button-only policy: CLOSED in b54a3e512317a308930a0021f62889e3bbb219ae - iOS + `WAKE_WORD` is rejected fail-closed (`ios_wake_disabled`); iOS uses side-button `EXPLICIT` only.
- Repo anchor: `crates/selene_adapter/src/lib.rs::evaluate_wake_for_turn`, adapter wake parity tests.

Gap 5 - Onboarding wake-enrollment action wiring: CLOSED in e8da92e - onboarding continue now supports wake-enroll start/sample/complete/defer actions and routes them through PH1.W runtime.
- Repo anchor: `crates/selene_os/src/app_ingress.rs::AppOnboardingContinueAction`, `run_onboarding_continue`, `crates/selene_adapter/src/lib.rs::parse_onboarding_continue_action`.

Gap 6 - Wake onboarding gate matrix differs from target strict matrix: CLOSED in e8da92e - Android/Desktop completion now requires completed wake enrollment plus matching wake sync receipt; iOS remains side-button-only.
- Repo anchor: `crates/selene_storage/src/ph1f.rs::ph1onb_complete_commit`, `crates/selene_os/src/app_ingress.rs::ensure_wake_enrollment_completed_for_platform`.

Gap 7 - `/v1/voice/turn` API contract is below target:
- Current state: request/response do not expose plan-required idempotency/session fields and status semantics.
- Repo anchor: `crates/selene_adapter/src/lib.rs::VoiceTurnAdapterRequest`, `VoiceTurnAdapterResponse`, `crates/selene_adapter/src/bin/http_adapter.rs::run_voice_turn`.
- Required change: add contract fields (`idempotency_key`, `session_id`, `turn_id`, decision refs) and status mapping (`200/409/422/401/403`).

Gap 8 - Wake runtime event commit wiring: CLOSED in b54a3e512317a308930a0021f62889e3bbb219ae - Live adapter path now commits wake runtime events for both accepted and rejected decisions.
- Repo anchor: `crates/selene_storage/src/ph1f.rs::ph1w_runtime_event_commit`, adapter live flow in `run_voice_turn_internal`.

Gap 9 - Wake learning taxonomy in contracts: CLOSED in e8dc612 - wake learn taxonomy now includes `WakeAccepted`, `WakeRejected`, `FalseWake`, `MissedWake`, `LowConfidenceWake`, and `NoisyEnvironment`.
- Repo anchor: `crates/selene_kernel_contracts/src/ph1learn.rs::LearnSignalType`, `WakeLearnSignalV1`.

Gap 10 - Learning-signal outbox/cloud ACK-NACK contract: CLOSED in e8dc612 - wake learn signals are persisted, queued idempotently, and handled via ACK/NACK_RETRYABLE/NACK_FATAL with retry/dead-letter behavior.
- Repo anchor: `crates/selene_storage/src/ph1f.rs::wake_learn_signal_commit_and_enqueue`, `crates/selene_os/src/device_artifact_sync.rs::handle_wake_learn_sync_http_response`.

Gap 11 - Artifact ABI compatibility fields/checks are missing:
- Current state: artifact ledger has `artifact_version/package_hash/payload_ref` but no wake runtime ABI negotiation fields.
- Repo anchor: `crates/selene_kernel_contracts/src/ph1art.rs::ArtifactLedgerRowInput`, `ArtifactLedgerRow`.
- Required change: add wake model ABI metadata and enforce load-time compatibility checks.

Gap 12 - Device-side pull/apply loop for improved wake artifacts: CLOSED in 343db5c - device sync worker now executes wake artifact pull, hash-verified stage/apply, and rollback with last-known-good pointer preservation.
- Repo anchor: `crates/selene_os/src/device_artifact_sync.rs::run_device_artifact_pull_apply_pass_internal`, `apply_wake_artifact_update`; `crates/selene_storage/src/ph1f.rs::wake_artifact_activate_commit`, `wake_artifact_rollback_commit`.

Gap 13 - Required perf/power release gate coverage is missing:
- Current state: wake tests exist for engine/runtime behavior, but no enforced perf/power gate harness for always-listening runtime.
- Repo anchor: wake unit/wiring tests in `crates/selene_engines/src/ph1w.rs`, `crates/selene_os/src/ph1w.rs`, adapter parity tests in `crates/selene_adapter/src/lib.rs`.
- Required change: add deterministic perf/power CI gates and block rollout on budget violations.

Gap 14 - `/v1/voice/turn` authentication enforcement: CLOSED in 36db4ff - bearer auth is enforced with deterministic `401/403` mapping before runtime execution.
- Repo anchor: `crates/selene_adapter/src/bin/http_adapter.rs::run_voice_turn`, `enforce_ingress_security`.

Gap 15 - Wake runtime event schema telemetry: CLOSED in b54a3e512317a308930a0021f62889e3bbb219ae - Wake runtime events now persist score, threshold, model version, and audio window timing boundaries.
- Repo anchor: `crates/selene_storage/src/ph1f.rs::WakeRuntimeEventRecord`.

Gap 16 - Internal wake reject code path is incomplete:
- Current state: timeout reject code exists, but no explicit internal-failure reject reason code required by runtime contract.
- Repo anchor: `crates/selene_engines/src/ph1w.rs::reason_codes`.
- Required change: add internal reject reason code mapping (for unknown/internal PH1.W failures) and persist it.

Gap 17 - Platform+trigger policy is not enforced at OS trigger model:
- Current state: trigger model is generic (`WakeWord|Explicit`) and wake-stage requirement is trigger-only, not platform-gated.
- Repo anchor: `crates/selene_os/src/ph1os.rs::OsVoiceTrigger`, `wake_stage_required`.
- Required change: enforce platform-aware trigger policy (`IOS => Explicit only`) in OS-level voice context validation.

Gap 18 - Builder does not explicitly bind wake artifact rollout:
- Current state: rollout/canary/rollback control exists in builder, but no explicit wake artifact binding path is defined.
- Repo anchor: `crates/selene_os/src/ph1builder.rs`.
- Required change: add wake artifact target/binding path to builder governed ingest + staged rollout flow.

Gap 19 - Native mobile app implementation is not present in this repository:
- Current state: no iOS/Android source files are present in this repo.
- Repo anchor: repository file scan (`.swift/.kt/.java/.xcodeproj/Gradle/Podfile/Info.plist`).
- Required change: define and maintain separate app-repo contract checklist and CI contract tests against this server API.

Gap 20 - Wake-specific retention and deletion lifecycle is missing:
- Current state: wake tables persist enrollment/runtime data, but no dedicated wake TTL/purge/delete flows are implemented.
- Repo anchor: `crates/selene_storage/src/ph1f.rs` wake enrollment/runtime records.
- Required change: add wake data retention TTL, purge jobs, and user-driven deletion paths for wake enrollment/runtime artifacts.

Gap 21 - Plan contract symbol names are docs-only and not codified:
- Current state: types like `WakeInferenceRequestV1`, `WakeDecisionV1`, `WakeLearnSignalV1`, `WakeLearnAckV1` are not implemented as concrete code contracts.
- Repo anchor: `docs/WAKE_BUILD_PLAN.md` (plan contract names), missing in `crates/**`.
- Required change: add these canonical contract types to kernel contracts and wire their validators through runtime paths.

Gap 22 - Token-subject and device-binding authorization on `/v1/voice/turn`: CLOSED in 36db4ff - ingress now validates bearer subject/device claims against request actor/device and fails closed on mismatch.
- Repo anchor: `crates/selene_adapter/src/bin/http_adapter.rs::enforce_ingress_security`, `run_voice_turn`; adapter ingress runtime uses `allow_identity_auto_provision=false`.

Gap 23 - Anti-replay ingress request security: CLOSED in 36db4ff - timestamp windows + nonce/request/idempotency replay cache now reject stale/replayed requests deterministically.
- Repo anchor: `crates/selene_adapter/src/bin/http_adapter.rs::required_header_u64`, `enforce_ingress_security`.

Gap 24 - HTTP ingress abuse controls (`rate/quota/429`): CLOSED in 36db4ff - per-token/per-device quota checks now enforce deterministic `429` responses with `Retry-After`.
- Repo anchor: `crates/selene_adapter/src/bin/http_adapter.rs::enforce_quota_counter`, `SecurityReject::too_many_requests`.

Gap 25 - Capture-bundle attestation boundary is missing:
- Current state: PH1.K live bundle consumes raw client-supplied capture metadata directly.
- Repo anchor: `crates/selene_adapter/src/lib.rs::VoiceTurnAudioCaptureRef`, `build_ph1k_live_signal_bundle`.
- Required change: require signed/attested capture bundle provenance and verify tamper-evidence before PH1.K/PH1.W/PH1.L decisions use capture fields.

Gap 26 - Artifact authenticity verification is missing (hash-only today):
- Current state: artifact contracts persist `package_hash/payload_ref/provenance_ref` but do not define signature trust-root verification flow.
- Repo anchor: `crates/selene_kernel_contracts/src/ph1art.rs::ArtifactLedgerRowInput`, `crates/selene_os/src/device_artifact_sync.rs`, `crates/selene_os/src/ph1builder.rs`.
- Required change: add cryptographic signature verification (artifact envelope + trust roots) at artifact ingest, pull/apply, and activation boundaries.
