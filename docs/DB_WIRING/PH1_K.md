# PH1.K DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.K`
- `implementation_id`: `PH1.K.001`
- `active_implementation_ids`: `[PH1.K.001]`
- `purpose`: Persist deterministic voice-runtime substrate outputs (stream refs, device state, timing, interrupt candidates, degradation flags) as an append-only ledger plus a rebuildable current projection.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audio_runtime_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - `event_kind` in `STREAM_REFS | VAD_EVENT | DEVICE_STATE | TIMING_STATS | INTERRUPT_CANDIDATE | DEGRADATION_FLAGS | TTS_PLAYBACK_ACTIVE`
  - `device_health` in `HEALTHY | DEGRADED | FAILED` when present
  - idempotent append dedupe on `(tenant_id, device_id, event_kind, idempotency_key)`
  - append-only; overwrite/delete prohibited
  - one tenant binding per device in PH1.K runtime scope

### `os_core.audio_runtime_current`
- `truth_type`: `CURRENT` (materialized)
- `primary key`: `(tenant_id, device_id)`
- invariants:
  - FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - FK `last_event_id -> audio_runtime_events.event_id`
  - state is rebuildable from `audio_runtime_events` in deterministic event-id order
  - `device_health` in `HEALTHY | DEGRADED | FAILED` when present

### `os_core.conversation_ledger` (PH1.K VAD markers only)
- `truth_type`: `LEDGER`
- `primary key`: `conversation_turn_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - optional FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - PH1.K writes only bounded VAD boundary markers with `source=PH1_K_VAD` and `role=SYSTEM`
  - text content is fixed marker text only (no transcript payload)
  - idempotent append dedupe on `(correlation_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

## 3) Reads (dependencies)

### Device/session FK checks
- reads: `devices.device_id`, `sessions.session_id` (optional), `identities.user_id` (for VAD marker writes)
- keys/joins used: direct FK existence lookups
- required indices:
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
  - `identities(user_id)` (PK)
- scope rules: PH1.K runtime writes are device-scoped and tenant-bound
- why this read is required: fail closed before runtime event persistence

### Runtime state reads
- reads:
  - `audio_runtime_events` by `(tenant_id, device_id)` and event id
  - `audio_runtime_current` by `(tenant_id, device_id)`
  - `conversation_ledger` by `(correlation_id, turn_id)` for VAD marker replay
- keys/joins used: deterministic key lookups and ordered replay
- required indices:
  - `ux_audio_runtime_events_dedupe`
  - `ix_audio_runtime_events_tenant_device_time`
  - `audio_runtime_current(tenant_id, device_id)` (PK)
  - `conversation_ledger(correlation_id, turn_id)`
- scope rules: no cross-tenant reads for one device binding
- why this read is required: deterministic idempotency and current-state rebuild

## 4) Writes (outputs)

### Append PH1.K runtime event
- writes: `audio_runtime_events`
- required fields:
  - `tenant_id`, `device_id`, `event_kind`, `created_at`, `idempotency_key`
  - plus event-kind fields:
    - `STREAM_REFS`: `processed_stream_id`, `pre_roll_buffer_id`
    - `VAD_EVENT`: `vad_state`, `vad_confidence`
    - `DEVICE_STATE`: `selected_mic`, `selected_speaker`, `device_health`
    - `TIMING_STATS`: `jitter_ms`, `drift_ppm`, `buffer_depth_ms`, `underruns`, `overruns`
    - `INTERRUPT_CANDIDATE`: `phrase_id`, `trigger_phrase_id`, `phrase_text`, `trigger_locale`, `candidate_confidence_band`, `vad_decision_confidence_band`, `risk_context_class`, `degradation_context_capture_degraded`, `degradation_context_aec_unstable`, `degradation_context_device_changed`, `degradation_context_stream_gap_detected`, `quality_metrics_snr_db`, `quality_metrics_clipping_ratio`, `quality_metrics_echo_delay_ms`, `quality_metrics_packet_loss_pct`, `quality_metrics_double_talk_score`, `quality_metrics_erle_db`, `timing_markers_window_start_ns`, `timing_markers_window_end_ns`, `speech_window_metrics_voiced_window_ms`, `subject_relation_confidence_bundle_lexical_confidence`, `subject_relation_confidence_bundle_vad_confidence`, `subject_relation_confidence_bundle_speech_likeness`, `subject_relation_confidence_bundle_echo_safe_confidence`, `subject_relation_confidence_bundle_nearfield_confidence?`, `subject_relation_confidence_bundle_combined_confidence`, `interrupt_policy_profile_id`, `interrupt_tenant_profile_id`, `interrupt_locale_tag`, `adaptive_device_route`, `adaptive_noise_class`, `adaptive_capture_to_handoff_latency_ms`, `adaptive_timing_jitter_ms`, `adaptive_timing_drift_ppm`, `adaptive_device_reliability_score`, `reason_code` (`phrase_text` must be one normalized approved phrase from the locale-tagged set bound to `interrupt_policy_profile_id` + `interrupt_tenant_profile_id`)
    - `DEGRADATION_FLAGS`: `capture_degraded`, `aec_unstable`, `device_changed`, `stream_gap_detected`
    - `TTS_PLAYBACK_ACTIVE`: `tts_playback_active`
- ledger event_type (if ledger): `K_RUNTIME_EVENT_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, device_id, event_kind, idempotency_key)`
- failure reason codes (minimum examples):
  - `K_FAIL_DEVICE_UNBOUND`
  - `K_FAIL_SESSION_INVALID`
  - `K_FAIL_TENANT_SCOPE_MISMATCH`
  - `K_FAIL_EVENT_FIELDS_INVALID`
  - `K_FAIL_INTERRUPT_POLICY_PROFILE_UNKNOWN`

### Append PH1.K VAD marker to conversation ledger
- writes: `conversation_ledger` (bounded marker row only)
- required fields:
  - `correlation_id`, `turn_id`, `session_id?`, `user_id`, `device_id`, `created_at`, `idempotency_key`
  - marker payload fields:
    - `source=PH1_K_VAD`
    - `role=SYSTEM`
    - `text="[VAD_EVENT:<state>]"` where `<state>` is one of `SPEECH_START | SPEECH_END | SILENCE_WINDOW`
    - `text_hash`
    - `privacy_scope=INTERNAL_ONLY`
- ledger event_type (if ledger): `K_VAD_EVENT_MARKER_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `K_FAIL_VAD_MARKER_SCOPE_INVALID`
  - `K_FAIL_VAD_MARKER_FIELDS_INVALID`

### Materialize/update PH1.K current runtime state
- writes: `audio_runtime_current`
- required fields:
  - `(tenant_id, device_id)`, `last_event_id`, `updated_at`
  - event-derived current fields (stream refs, device state, timing, interrupt/degradation snapshots)
- ledger event_type (if ledger): n/a (current projection update)
- idempotency_key rule (exact formula):
  - driven by source ledger dedupe key
- failure reason codes (minimum examples):
  - `K_FAIL_REBUILD_INTEGRITY`

## 5) Relations & Keys

FKs:
- `audio_runtime_events.device_id -> devices.device_id`
- `audio_runtime_events.session_id -> sessions.session_id` (nullable)
- `audio_runtime_current.device_id -> devices.device_id`
- `audio_runtime_current.session_id -> sessions.session_id` (nullable)
- `audio_runtime_current.last_event_id -> audio_runtime_events.event_id`
- `conversation_ledger.user_id -> identities.user_id`
- `conversation_ledger.device_id -> devices.device_id` (nullable)
- `conversation_ledger.session_id -> sessions.session_id` (nullable)

Unique constraints:
- `audio_runtime_events(event_id)` (PK)
- `ux_audio_runtime_events_dedupe`
- `audio_runtime_current(tenant_id, device_id)` (PK)

State/boundary constraints:
- `audio_runtime_events` is append-only.
- `audio_runtime_current` must be derivable from `audio_runtime_events` only.
- unknown `implementation_id` is fail-closed at runtime dispatch (`ph1_k.implementation_id`).
- unknown `interrupt_policy_profile_id` is fail-closed at lexical candidate evaluation (`interrupt_input.lexicon_policy_binding.policy_profile_id`).
- unknown/unauthorized adaptive threshold profile inputs are fail-closed (`adaptive_threshold_policy_input` validation + profile selection by tenant policy binding).
- interrupt candidate emission is noise-safe and fail-closed:
  - lexical phrase match from approved phrase set is mandatory.
  - hybrid lexical+acoustic+prosody safeguards are mandatory (`phrase_confidence`, `acoustic_confidence`, `prosody_confidence`, `vad_confidence`, `speech_likeness`, voiced window, echo-safe gate, optional nearfield gate).
  - adaptive thresholds are deterministic by `device_route + noise/degradation class + tenant policy profile`.
  - jitter/clock recovery gate must pass bounded profile budgets before candidate emission.
  - degradation flags block candidate emission.
  - noise-only signals cannot emit interrupt candidates.
- PH1.K persists substrate facts only; it does not persist intent/authority decisions.
- PH1.K must not persist raw transcript text or sensitive phrase content; interrupt rows persist only normalized locale-tagged interrupt phrases (Unicode-safe canonical form) and VAD rows persist bounded markers.

## 6) Audit Emissions (PH1.J)

PH1.K runtime writes must emit PH1.J audit events with:
- `event_type`:
  - `AuditEventType::PerceptionSignalEmitted` with payload key `event_name` set to:
    - `K_STREAM_REFS_COMMIT`
    - `K_DEVICE_STATE_COMMIT`
    - `K_TIMING_STATS_COMMIT`
    - `K_INTERRUPT_CANDIDATE_COMMIT`
    - `K_VAD_EVENT_MARKER_COMMIT`
    - `K_DEGRADATION_FLAGS_COMMIT`
    - `K_TTS_PLAYBACK_ACTIVE_COMMIT`
- `reason_code(s)`:
  - `K_INTERRUPT_LEXICAL_TRIGGER_REJECTED`
  - `K_INTERRUPT_NOISE_GATE_REJECTED`
  - `K_INTERRUPT_CANDIDATE_EMITTED_HIGH`
  - `K_INTERRUPT_CANDIDATE_EMITTED_MEDIUM`
  - `K_INTERRUPT_CANDIDATE_EMITTED_LOW`
  - `K_INTERRUPT_FEEDBACK_FALSE_LEXICAL_TRIGGER`
  - `K_INTERRUPT_FEEDBACK_MISSED_LEXICAL_TRIGGER`
  - `K_INTERRUPT_FEEDBACK_WRONG_CONFIDENCE_BAND`
  - `K_STREAM_GAP_DETECTED`
  - `K_AEC_UNSTABLE`
  - `K_DEVICE_FAILOVER`
  - `K_DEVICE_UNHEALTHY`
- `payload_min` allowlisted keys:
  - `tenant_id`
  - `device_id`
  - `session_id`
  - `event_kind`
  - `processed_stream_id`
  - `pre_roll_buffer_id`
  - `vad_state`
  - `vad_confidence`
  - `device_health`
  - `tts_playback_active`
  - `interrupt_phrase_id`
  - `trigger_phrase_id`
  - `interrupt_phrase_text`
  - `trigger_locale`
  - `candidate_confidence_band`
  - `risk_context_class`
  - `degradation_context`
  - `timing_markers`
  - `speech_window_metrics`
  - `subject_relation_confidence_bundle`
  - `interrupt_policy_profile_id`
  - `interrupt_tenant_profile_id`
  - `interrupt_locale_tag`
  - `degradation_flags`
- PH1.J payload allowlist lock:
  - PH1.K `PerceptionSignalEmitted` rows are restricted to allowlisted PH1.K keys only (unknown keys fail closed at contract validation).

## 6B) Feedback/Learning Sink Wiring

PH1.K interruption diagnostics must emit PH1.FEEDBACK events with deterministic routing:
- `false interrupt` -> `K_INTERRUPT_FEEDBACK_FALSE_INTERRUPT`
- `missed interrupt` -> `K_INTERRUPT_FEEDBACK_MISSED_INTERRUPT`
- `wrong degradation classification` -> `K_INTERRUPT_FEEDBACK_WRONG_DEGRADATION_CLASSIFICATION`
- `bad failover selection` -> `K_INTERRUPT_FEEDBACK_BAD_FAILOVER_SELECTION`

PH1.K Step-13 clustering proof fields (stored in PH1.FEEDBACK payload and PH1.K capture rows):
- `cluster_primary_fingerprint`
- `cluster_secondary_fingerprint`
- `feedback_kind`

Routing contract:
- `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`
- PH1.K remains substrate-only; sink events are advisory for threshold/model route tuning and builder remediation proposals.
- Step-14 governed promotion lock:
  - each `ph1k_feedback_capture_commit` must emit one PH1.LEARN signal bundle and one PH1.PAE ladder audit row.
  - PH1.PAE mode transitions are one-step only (`SHADOW -> ASSIST -> LEAD` for promote, reverse for demote).
  - regression auto-demotion is mandatory when quality-regression flags or false-interrupt-rate breach is detected.
  - strict rollback trigger is mandatory when:
    - `false_interrupt_rate_milli_per_hour > 300` (Step-18 gate limit `0.3/hour`)
    - or quality regression is detected (`capture_degraded|aec_unstable|device_changed|stream_gap_detected`).
  - PH1.K capture rows must persist Step-14 proof fields:
    - `learn_bundle_id`
    - `pae_mode_from`
    - `pae_mode_to`
    - `pae_decision_action`
    - `pae_rollback_triggered`
    - `pae_quality_regression_triggered`
    - `pae_false_interrupt_regression_triggered`
    - `false_interrupt_rate_milli_per_hour`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-K-01` tenant isolation enforced
  - `at_k_db_01_tenant_isolation_enforced`
- `AT-K-02` append-only enforcement for PH1.K runtime ledger
  - `at_k_db_02_append_only_enforced`
- `AT-K-03` idempotency dedupe works
  - `at_k_db_03_idempotency_dedupe_works`
- `AT-K-04` current-table rebuild from runtime ledger is deterministic
  - `at_k_db_04_current_table_rebuild_from_ledger`
- `AT-K-05` extended interrupt metrics/confidence payload persists and projects deterministically
  - `at_k_db_05_interrupt_extended_fields_persist_and_project`
- `AT-K-06` extended interrupt payload validation is fail-closed on invalid bounds/shape
  - `at_k_db_06_interrupt_extended_fields_fail_closed`
- `AT-K-07` every PH1.K runtime commit emits reason-coded PH1.J perception audit row
  - `at_k_db_07_runtime_commits_emit_reason_coded_ph1j_rows`
- `AT-K-08` expanded interrupt audit payload keys are present and bounded
  - `at_k_db_08_interrupt_extended_audit_payload_includes_step12_keys`
- `AT-K-09` PH1.K feedback-capture wiring emits all Step-13 issue classes with fingerprint metadata
  - `at_k_db_09_feedback_capture_wires_issue_kinds_and_fingerprints`
- `AT-K-10` bad-failover feedback capture fails closed without full failover pair
  - `at_k_db_10_feedback_capture_bad_failover_requires_device_pair`
- `AT-K-11` PH1.K feedback capture emits LEARN bundle + governed PAE ladder transitions with regression rollback triggers
  - `at_k_db_11_feedback_capture_routes_to_learn_and_governed_pae_ladder`
- `AT-K-12` Unicode interrupt phrase normalization remains fail-closed and control-safe
  - `normalize_interrupt_phrase_strips_controls_and_collapses_whitespace`
  - `normalize_interrupt_phrase_rejects_control_only_input_fail_closed`
- `AT-K-13` confidence-band mapping to reason codes is deterministic at boundary values
  - `at_k_interrupt_13_confidence_band_and_reason_code_mapping_boundaries_are_locked`
- `AT-K-14` adaptive threshold profile selection is deterministic and fails closed on unknown tenant profile
  - `at_k_interrupt_14_threshold_profile_selection_is_deterministic_by_route_and_noise`
  - `at_k_interrupt_15_threshold_profile_selection_fails_closed_on_unknown_tenant_profile`
- `AT-K-15` noisy-environment recovery replay is deterministic and only returns to healthy after recovery stability window
  - `at_k_runtime_16_noisy_environment_recovery_replay_is_deterministic`
- `AT-K-16` overlap-speech interruption path remains deterministic (replay-stable reject/emit behavior by confidence and noise class)
  - `at_k_runtime_17_overlap_speech_interrupt_decision_is_replay_deterministic`
- `AT-K-17` device failover remains deterministic under stability + cooldown windows
  - `at_k_runtime_18_failover_cooldown_stability_windows_are_deterministic`
- `AT-K-18` PH1.K handoff envelopes remain compatible with PH1.C and PH1.X contracts
  - `at_k_runtime_19_ph1c_and_ph1x_handoff_envelopes_are_compatible`
- `AT-K-19` PH1.K benchmark/eval harness snapshot enforces required scenario coverage and emits trend-ready metrics
  - `check_ph1k_round2_eval_snapshot.sh`
  - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
- `AT-K-20` PH1.K global-standard release gate enforces all Step-18 thresholds on canonical eval snapshot input
  - false interrupt rate <= `0.3/hour`
  - missed interrupt rate <= `2%`
  - end-of-speech p95 <= `180ms`
  - PH1.K -> PH1.C handoff p95 <= `120ms`
  - device failover recovery p95 <= `1500ms`
  - noisy auto-recovery success >= `97%`
  - multilingual interrupt recall >= `95%`
  - audit completeness = `100%`
  - tenant isolation = `100%`
  - `check_ph1k_release_gate.sh`
  - `docs/fixtures/ph1k_round2_eval_snapshot.csv`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0010_ph1k_audio_runtime_tables.sql`
- migration extension: `crates/selene_storage/migrations/0023_ph1k_interrupt_extended_fields.sql`
- tests: `crates/selene_storage/tests/ph1_k/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.ENDPOINT`)

- Selene OS may consume PH1.K VAD windows/timing stats and invoke PH1.ENDPOINT for boundary refinement before PH1.C finalization.
- PH1.K remains owner of runtime substrate events; PH1.ENDPOINT does not write PH1.K-owned tables.
- If PH1.ENDPOINT validation fails, Selene OS must fail closed for endpoint handoff and keep PH1.K runtime truth unchanged.

## 9) Related Engine Boundary (`PH1.C`)

- PH1.K must provide deterministic handoff inputs for PH1.C STT strategy alignment:
  - `interrupt_confidence_band`
  - `vad_confidence_band`
  - `quality_metrics` summary
  - `degradation_class_bundle`
- PH1.K handoff builder must derive `degradation_class_bundle` deterministically from append-only degradation flags when an interrupt candidate is absent, so replay/current rebuild remains stable.
- PH1.C remains STT decision authority; PH1.K handoff is advisory substrate input only.

## 10) Related Engine Boundary (`PH1.X`)

- PH1.K must provide deterministic interrupt-risk handoff inputs for PH1.X:
  - `candidate_confidence_band`
  - `gate_confidences`
  - `degradation_context`
  - `risk_context_class`
- PH1.K handoff projection contract is `Ph1kToPh1xInterruptHandoff`, derived from emitted `InterruptCandidate` rows (`build_ph1k_to_ph1x_handoff(...)`).
- If no interrupt candidate is emitted, PH1.K must return `None` for PH1.X handoff and must not infer interruption action.
- PH1.X remains interruption-action authority (`WAIT | RESPOND | CLARIFY | DISPATCH`); PH1.K handoff remains advisory substrate input only.

## 11) FDX Wiring Lock (Section 5F)

- PH1.K DB wiring remains append-only substrate truth for duplex runtime signals.
- Interruption trigger proof must remain lexical-only:
  - persisted candidate rows must include approved phrase identity/locale linkage.
  - non-lexical-only triggers must never persist accepted interruption rows.
- PH1.K rows must provide deterministic handoff evidence for PH1.C and PH1.X (timing markers + confidence bundles + risk/degradation context).
- Duplex regressions must be emitted as PH1.FEEDBACK-linked reason-coded events for downstream LEARN/PAE governance.
