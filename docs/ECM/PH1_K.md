# PH1.K ECM Spec

## Engine Header
- `engine_id`: `PH1.K`
- `implementation_id`: `PH1.K.001`
- `active_implementation_ids`: `[PH1.K.001]`
- `purpose`: Persist deterministic voice runtime substrate facts (stream refs, device/timing/interruption/degradation signals) as append-only events plus rebuildable current state.
- `data_owned`: `audio_runtime_events`, `audio_runtime_current`, `conversation_ledger` (PH1.K VAD markers only)
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1K_RUNTIME_EVENT_COMMIT_ROW`
- `name`: Commit one PH1.K runtime event row and project current state
- `input_schema`: `(now, tenant_id, device_id, session_id?, event_kind, stream/device/timing/interrupt/degradation fields, idempotency_key)` where `INTERRUPT_CANDIDATE` rows include `interrupt_policy_profile_id`, `interrupt_tenant_profile_id`, `interrupt_locale_tag`, `trigger_phrase_id`, `trigger_locale`, `candidate_confidence_band`, `vad_decision_confidence_band`, `risk_context_class`, `degradation_context`, `quality_metrics` (`snr_db`, `clipping_ratio`, `echo_delay_ms`, `packet_loss_pct`, `double_talk_score`, `erle_db`), `timing_markers`, `speech_window_metrics`, `subject_relation_confidence_bundle`, and adaptive-threshold inputs (`device_route`, `noise_class`, `capture_to_handoff_latency_ms`, `timing_stats`, `device_reliability_score`).
- `output_schema`: `Result<Ph1kRuntimeEventRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1K_RUNTIME_EVENT_COMMIT_ROW_EXTENDED`
- `name`: Commit one PH1.K runtime event row using typed extended interrupt payload surface
- `input_schema`: `(now, tenant_id, device_id, session_id?, event_kind, stream/device/timing fields, phrase/reason fields, interrupt_extended?, degradation fields, tts_playback_active?, idempotency_key)` where `interrupt_extended` is allowed only for `INTERRUPT_CANDIDATE` and must satisfy strict bounded validators.
- `output_schema`: `Result<Ph1kRuntimeEventRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1K_VAD_MARKER_COMMIT_ROW`
- `name`: Commit one bounded PH1.K VAD marker row to conversation ledger
- `input_schema`: `(now, correlation_id, turn_id, session_id?, user_id, device_id?, vad_state, idempotency_key)`
- `output_schema`: `Result<ConversationTurnId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1K_READ_RUNTIME_EVENT_ROWS`
- `name`: Read PH1.K runtime ledger rows
- `input_schema`: `none`
- `output_schema`: `Ph1kRuntimeEventRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_READ_RUNTIME_CURRENT_ROWS`
- `name`: Read PH1.K current projection map
- `input_schema`: `none`
- `output_schema`: `Map<(tenant_id, device_id), Ph1kRuntimeCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_READ_RUNTIME_CURRENT_ROW`
- `name`: Read one `(tenant_id, device_id)` current projection row
- `input_schema`: `(tenant_id, device_id)`
- `output_schema`: `Option<Ph1kRuntimeCurrentRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1K_REBUILD_RUNTIME_CURRENT_ROWS`
- `name`: Rebuild PH1.K current projection from append-only ledger
- `input_schema`: `none`
- `output_schema`: `unit`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE_CURRENT_PROJECTION)`

### `PH1K_APPEND_ONLY_GUARD`
- `name`: Guard against overwrite of PH1.K runtime ledger rows
- `input_schema`: `event_id`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- unknown implementation dispatch: `K_FAIL_UNKNOWN_IMPLEMENTATION`
- unknown interrupt policy profile binding: `K_FAIL_INTERRUPT_POLICY_PROFILE_UNKNOWN`
- contract/field validation failure: `K_FAIL_EVENT_FIELDS_INVALID`
- invalid or missing scope binding: `K_FAIL_TENANT_SCOPE_MISMATCH`
- invalid session reference: `K_FAIL_SESSION_INVALID`
- device binding failure: `K_FAIL_DEVICE_UNBOUND`
- invalid VAD marker scope/content: `K_FAIL_VAD_MARKER_SCOPE_INVALID`
- idempotency replay/no-op: `K_IDEMPOTENCY_REPLAY`
- interrupt lexical rejection: `K_INTERRUPT_LEXICAL_TRIGGER_REJECTED`
- interrupt noise gate rejection: `K_INTERRUPT_NOISE_GATE_REJECTED`
- interrupt candidate emitted (banded): `K_INTERRUPT_CANDIDATE_EMITTED_HIGH | K_INTERRUPT_CANDIDATE_EMITTED_MEDIUM | K_INTERRUPT_CANDIDATE_EMITTED_LOW`
- interrupt feedback wiring:
  - `K_INTERRUPT_FEEDBACK_FALSE_INTERRUPT`
  - `K_INTERRUPT_FEEDBACK_MISSED_INTERRUPT`
  - `K_INTERRUPT_FEEDBACK_WRONG_DEGRADATION_CLASSIFICATION`
  - `K_INTERRUPT_FEEDBACK_BAD_FAILOVER_SELECTION`

## Audit Emission Requirements Per Capability
- `PH1K_RUNTIME_EVENT_COMMIT_ROW` must emit PH1.J with bounded payload and reason code for each event class:
  - `AuditEventType::PerceptionSignalEmitted` with allowlisted PH1.K payload keys only.
  - `K_STREAM_REFS_COMMIT`
  - `K_DEVICE_STATE_COMMIT`
  - `K_TIMING_STATS_COMMIT`
  - `K_INTERRUPT_CANDIDATE_COMMIT`
  - `K_VAD_EVENT_MARKER_COMMIT`
  - `K_DEGRADATION_FLAGS_COMMIT`
  - `K_TTS_PLAYBACK_ACTIVE_COMMIT`
- `PH1K_REBUILD_RUNTIME_CURRENT_ROWS` emits audit only in replay/diagnostic mode.
- Read/guard capabilities emit audit only when explicitly run under verification traces.

## Runtime Guardrails (Voice Substrate Boundary)
- Unknown `implementation_id` must fail closed at runtime dispatch (`ph1_k.implementation_id`).
- Unknown interrupt policy profile binding must fail closed at lexical candidate gating (`interrupt_input.lexicon_policy_binding.policy_profile_id`).
- Unknown adaptive threshold policy/tenant profile combination must fail closed before gate evaluation.
- Interrupt candidate emission must pass mandatory noise-safe gates:
  - lexical phrase match from approved policy-bound phrase set
  - hybrid lexical+acoustic+prosody safeguards (`phrase_confidence`, `acoustic_confidence`, `prosody_confidence`, `vad_confidence`, `speech_likeness`, voiced window, echo-safe, optional nearfield)
  - deterministic adaptive-threshold selection (`device_route + noise/degradation class + tenant-approved policy profile`)
  - jitter/clock recovery policy budgets (`max_jitter_ms`, `max_abs_drift_ppm`, `max_handoff_latency_ms`)
  - degradation gate (candidate blocked when degraded)
- Noise-only signals must never emit interrupt candidates.
- interrupt candidate handoff payload must include bounded subject-handoff hints:
  - `timing_markers`
  - `speech_window_metrics`
  - `subject_relation_confidence_bundle`
- PH1.X snapshot integrity is fail-closed when cursor/snapshot/timing invariants drift.
- PH1.K interruption output is candidate-only (`InterruptCandidate`); cancellation policy remains in PH1.X.
- PH1.K remains non-authoritative for identity/authority/execution decisions.
- Locale-tagged interrupt phrase matching must use Unicode-safe normalization and must not rely on ASCII-only pathing.

## Feedback/Learning Sink Contract
- PH1.K interruption diagnostics must emit PH1.FEEDBACK route events through:
  - `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`
- PH1.K Step-13 captures must include clustering fingerprints in PH1.FEEDBACK payload metadata:
  - `cluster_primary_fingerprint`
  - `cluster_secondary_fingerprint`
- PH1.K Step-14 promotion routing lock:
  - PH1.K feedback capture must emit a PH1.LEARN bundle and a PH1.PAE ladder decision/audit in the same deterministic commit path.
  - PH1.PAE mode transitions must remain one-step (`SHADOW <-> ASSIST <-> LEAD`); direct `SHADOW -> LEAD` jump is forbidden.
  - regression auto-demotion and rollback trigger are mandatory when:
    - `false_interrupt_rate_milli_per_hour > 300`
    - or quality-regression flags are raised (`capture_degraded|aec_unstable|device_changed|stream_gap_detected`).
- PH1.K does not self-authorize threshold promotions; PH1.PAE remains route/policy promotion owner.

## Round-2 Test-Lock References (Step 15)
- Kernel-contract fail-closed + Unicode normalization locks:
  - `normalize_interrupt_phrase_strips_controls_and_collapses_whitespace`
  - `normalize_interrupt_phrase_rejects_control_only_input_fail_closed`
- Runtime confidence/profile determinism locks:
  - `at_k_interrupt_13_confidence_band_and_reason_code_mapping_boundaries_are_locked`
  - `at_k_interrupt_14_threshold_profile_selection_is_deterministic_by_route_and_noise`
  - `at_k_interrupt_15_threshold_profile_selection_fails_closed_on_unknown_tenant_profile`
- Proof commands:
  - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
  - `cargo test -p selene_engines ph1k -- --nocapture`

## Round-2 Runtime/Replay/Integration Locks (Step 16)
- Runtime recovery replay lock:
  - `at_k_runtime_16_noisy_environment_recovery_replay_is_deterministic`
- Overlap-speech interruption replay lock:
  - `at_k_runtime_17_overlap_speech_interrupt_decision_is_replay_deterministic`
- Failover cooldown/stability determinism lock:
  - `at_k_runtime_18_failover_cooldown_stability_windows_are_deterministic`
- PH1.K -> PH1.C / PH1.K -> PH1.X envelope compatibility lock:
  - `at_k_runtime_19_ph1c_and_ph1x_handoff_envelopes_are_compatible`
- Proof commands:
  - `cargo test -p selene_engines ph1k::tests::at_k_runtime_ -- --nocapture`
  - `cargo test -p selene_engines ph1k -- --nocapture`
  - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`

## Round-2 Benchmark/Eval Harness Lock (Step 17)
- Canonical harness:
  - `scripts/check_ph1k_round2_eval_snapshot.sh`
- Canonical persisted snapshot:
  - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
- Required coverage dimensions:
  - locale: `en-US`, `es-ES`, `zh-CN`, `tr-TR`
  - device route: `BUILT_IN`, `BLUETOOTH`, `USB`, `VIRTUAL`
  - noise class: `CLEAN`, `ELEVATED`, `SEVERE`
  - overlap speech: both `0` and `1`
- Harness outputs:
  - per-row benchmark lines (`PH1K_EVAL_ROW`)
  - aggregated summary (`PH1K_EVAL_SUMMARY`)
  - worst-case summary (`PH1K_EVAL_WORST`)
  - file-order trend delta (`PH1K_EVAL_TREND`)
- Proof command:
  - `bash scripts/check_ph1k_round2_eval_snapshot.sh docs/fixtures/ph1k_round2_eval_snapshot.csv`

## Round-2 Global-Standard Release Gate Lock (Step 18)
- Canonical release gate:
  - `scripts/check_ph1k_release_gate.sh`
- Canonical input snapshot:
  - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
- Required pass thresholds:
  - false interrupt rate <= `0.3/hour`
  - missed interrupt rate <= `2%`
  - end-of-speech p95 <= `180ms`
  - PH1.K -> PH1.C handoff p95 <= `120ms`
  - device failover recovery p95 <= `1500ms`
  - noisy auto-recovery success >= `97%`
  - multilingual interrupt recall >= `95%`
  - audit completeness = `100%`
  - tenant isolation = `100%`
- Release-gate outputs:
  - per-row metric lines (`PH1K_RELEASE_ROW`)
  - worst-case threshold summary (`PH1K_RELEASE_WORST`)
- Proof command:
  - `bash scripts/check_ph1k_release_gate.sh docs/fixtures/ph1k_round2_eval_snapshot.csv`

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1kVoiceRuntimeRepo`)
- `docs/DB_WIRING/PH1_K.md`

## Related Engine Boundary (`PH1.ENDPOINT`)
- PH1.K runtime capabilities expose VAD/timing substrate signals that Selene OS may pass to PH1.ENDPOINT for optional boundary refinement.
- PH1.K does not depend on PH1.ENDPOINT for runtime integrity; endpointing is an optional assist path only.

## Related Engine Boundary (`PH1.C`)
- PH1.K exposes deterministic handoff builder output for PH1.C strategy alignment using bounded fields only:
  - `interrupt_confidence_band`
  - `vad_confidence_band`
  - `quality_metrics`
  - `degradation_class_bundle`
- If no interrupt candidate is emitted, PH1.K must derive handoff classes from current degradation flags and quality metrics without non-deterministic inference.
- PH1.C remains authoritative for STT route execution and final transcript gating.

## Related Engine Boundary (`PH1.X`)
- PH1.K exposes deterministic interrupt-risk handoff builder output for PH1.X using bounded fields only:
  - `candidate_confidence_band`
  - `gate_confidences`
  - `degradation_context`
  - `risk_context_class`
- PH1.K handoff projection contract is `Ph1kToPh1xInterruptHandoff`, produced from validated `InterruptCandidate` output.
- If no interrupt candidate is emitted, PH1.K returns `None` for PH1.X handoff (no inferred interruption action).
- PH1.X remains authoritative for interruption branch/action selection; PH1.K remains advisory substrate only.

## FDX Design Lock (Section 5F)
- PH1.K is the always-on duplex substrate owner in voice sessions.
- PH1.K must continuously emit bounded duplex capture state (`DuplexFrame` equivalent) while Selene is speaking and listening.
- Barge-in trigger source is lexical-only (approved words/phrases); non-lexical-only signals (`noise`, `acoustic-only`, `prosody-only`) are forbidden as interruption triggers.
- PH1.K interruption output remains candidate-only (`InterruptCandidate`); PH1.K cannot cancel TTS or choose response branches.
- PH1.K must emit bounded timing markers required for p95 latency gates (`barge-in detect -> cancel` path, capture-to-handoff path).
- PH1.K must provide deterministic handoff envelopes to:
  - PH1.C for partial transcript strategy alignment.
  - PH1.X for interruption branch evaluation.
- PH1.K must emit duplex miss/false/missed-interrupt signals into `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`.
