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

Client/server trigger contract (wake lives inside the Selene App):

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

What does not yet exist in runtime:

- Real always-on wake detection in the voice route
- Local device wake engine implementation

Currently the adapter synthesizes wake acceptance instead of running real wake inference.

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

Onboarding gate matrix (enforced):

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

## Section 5: Wake Artifact System

Wake models are stored as artifacts.

Artifacts contain:

- `artifact_version`
- `model_abi`
- `package_hash`
- `payload_ref`

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

## Section 7: Wake Failure Learning

Learning pipeline exists and is expanded with explicit wake taxonomy.

Required wake learning event taxonomy:

- `WakeAccepted`
- `WakeRejected`
- `FalseWake`
- `MissedWake`
- `LowConfidenceWake`
- `NoisyEnvironment`

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

This system is used to send:

- wake artifacts
- learning signals
- device sync events

## Section 9: Real Wake Detection (Missing)

The repository currently does not run real wake detection in the live voice route.

Current state:

Adapter builds a synthetic `WakeDecision::accept`.

Required change:

`WAKE_WORD` trigger must call `Ph1wRuntime` inference using the audio buffer.

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

Current repo only passes:

- `processed_stream_ref`
- `pre_roll_buffer_ref`

Actual microphone runtime must still be implemented.

Audio runtime implementation requirements:

- Capture format: PCM 16-bit signed little-endian, mono, 16 kHz.
- Ring buffer minimum capacity: 3 seconds.
- Pre-roll minimum retained audio: 1.2 seconds.
- Post-wake capture minimum: 1.0 second.
- Stream clock drift correction required for sessions > 5 minutes.
- VAD must provide frame-level speech probability for wake quality metrics.

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

## Section 13: Wake Accuracy Stabilization

Wake quality is measured by explicit metrics and windows.

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
