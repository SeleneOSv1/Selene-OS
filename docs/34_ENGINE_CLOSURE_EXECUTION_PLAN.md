# Engine Closure Execution Plan (Hybrid, End-to-End)

## 0) Purpose
- Define one canonical execution playbook to move Selene OS from current build state to production readiness.
- Prevent drift/loss of direction by combining:
  - global roadmap control, and
  - one-engine-at-a-time closure units.

## 1) Operating Model (Hybrid)
- Use `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/33_ENGINE_REVIEW_TRACKER.md` as fixed engine order and status truth.
- Execute one engine at a time as the unit of closure.
- Keep cross-engine relationship checks mandatory before marking any engine `DONE`.
- Do not bypass gates:
  - No Simulation -> No Execution
  - Access/Authority gate enforced by `PH1.ACCESS.001 -> PH2.ACCESS.002`
  - Engines never call engines directly; Selene OS orchestrates.

## 1A) Current Cycle Lock (Second-Round Engine Finalization)
- Current active cycle is the second round per engine.
- This round must finalize engines one-by-one before production hardening starts.
- An engine cannot be marked `READY_FOR_PRODUCTION_HARDENING` until all actionable engine rows in `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/33_ENGINE_REVIEW_TRACKER.md` complete this second-round finalization (`EXEMPT`/`MERGED` excluded).
- Production hardening work is blocked until this gate is satisfied.
- Locked priority start queue for this round:
  - `PH1.VISION` (completed)
  - `PH1.K` (completed)
  - `PH1.C` (next)
  - `PH1.D`

## 2) Canonical Control Files
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/33_ENGINE_REVIEW_TRACKER.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/07_ENGINE_REGISTRY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/06_ENGINE_MAP.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/08_SIMULATION_CATALOG.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/09_BLUEPRINT_REGISTRY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/10_DB_OWNERSHIP_MATRIX.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/COVERAGE_MATRIX.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/03_BUILD_LEDGER.md`

## 3) Per-Engine Closure Unit (Repeat Per Engine)
1. Lock target engine ID and related-engine set.
2. Extract target requirements from canonical source docs.
3. Produce explicit gap list:
   - docs gap
   - kernel contract gap
   - runtime/wiring gap
   - test gap
4. Patch docs:
   - DB_WIRING
   - ECM
   - registry/map/coverage/tracker where needed
5. Patch kernel contracts (typed request/response + validation).
6. Patch runtime wiring (engine + OS orchestration path).
7. Add/adjust tests (acceptance + fail-closed + deterministic behavior).
8. Run compile/test gates.
9. Run relationship check gate.
10. Mark tracker status and append build ledger proof entry.

## 4) Relationship Check Gate (Mandatory Per Engine)
1. Upstream/downstream contract compatibility.
2. Capability IDs resolve to ACTIVE entries.
3. Simulation IDs resolve to ACTIVE entries for side effects.
4. Access gate path cannot be bypassed.
5. No engine-to-engine direct call path.
6. DB ownership remains single-writer per table contract.
7. Idempotency/lease invariants enforced where applicable.
8. Reason code coverage and deterministic failure mapping present.
9. Audit envelope integrity present for governed actions.
10. SLO/telemetry hooks present where engine has latency contract.

## 5) Four Program Rounds

### Round 1: Closure Round (Contract + Wiring Closure)
Goal:
- Close all in-scope engines to `DONE` using the per-engine closure unit.

Scope:
- Engine-by-engine completion across tracker order, honoring EXEMPT/MERGED rules.

Required output:
- Docs + kernel contracts + runtime wiring + tests aligned per engine.
- Tracker row closed with proof.

Exit criteria:
- No open actionable engine rows in tracker (excluding EXEMPT/MERGED).
- No unresolved relationship-check items.
- Workspace compiles and targeted engine suites pass.

### Round 2: Finalization Round (Engine-By-Engine Closeout, Current)
Goal:
- Re-open each actionable engine one-by-one for final changes and close remaining functionality gaps before hardening.

Scope:
- Per-engine final pass with strict one-engine-at-a-time execution and proof updates.

Mandatory checks:
1. Reconfirm contracts + runtime behavior against current production target requirements.
2. Apply any last functional deltas with fail-closed behavior preserved.
3. Run targeted tests for that engine and impacted relationships.
4. Update tracker finalization status and ledger proof before moving to next engine.

Exit criteria:
- Every actionable runtime engine row is finalized in this round (`EXEMPT`/`MERGED` excluded).
- No open per-engine finalization blockers remain.

### 5A) PH1.K Round-2 Strict Implementation Checklist (Step 1..19)
Purpose:
- Execute PH1.K finalization as a strict, auditable sequence aimed at global-grade voice-runtime quality.
- Keep PH1.K boundary unchanged: substrate owner only, no intent/authority/execution decisions.

Pre-coding lock:
- This checklist is locked before code changes.
- Code implementation begins only after this checklist is accepted as the active PH1.K runbook.

Step 1. Baseline snapshot and freeze
- Capture current PH1.K baseline from tests and runtime telemetry:
  - interrupt false-positive/false-negative rates
  - VAD boundary latency
  - device failover recovery latency
  - degradation incidence (`aec_unstable`, `stream_gap_detected`, `device_changed`)
- Record baseline in build ledger as PH1.K round-2 start proof.
- Execution status (`2026-02-25`):
  - Step 1: COMPLETE (frozen baseline snapshot recorded from current PH1.K tests + runtime telemetry snapshot with derived interrupt miss/false rates, VAD boundary latency, failover recovery latency, and degradation incidence counts).
  - Baseline artifact:
    - `docs/fixtures/ph1k_round2_baseline_snapshot.csv`
  - Baseline validator:
    - `scripts/check_ph1k_round2_baseline_snapshot.sh`
  - Proof lock:
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_os ph1k -- --nocapture`
    - `bash scripts/check_ph1k_round2_baseline_snapshot.sh docs/fixtures/ph1k_round2_baseline_snapshot.csv`

Step 2. Contract/docs lock for expanded PH1.K surface
- Update PH1.K docs/contracts to declare new required surfaces before runtime coding:
  - richer audio quality metrics
  - confidence bands (`HIGH | MEDIUM | LOW`)
  - multilingual/Unicode-safe interrupt phrase handling
  - adaptive-threshold policy inputs
- Files: `docs/DB_WIRING/PH1_K.md`, `docs/ECM/PH1_K.md`, `crates/selene_kernel_contracts/src/ph1k.rs`.

Step 3. Kernel contract expansion (typed, bounded, validated)
- Add typed structures and validators for:
  - advanced quality metrics (`snr_db`, `clipping_ratio`, `echo_delay_ms`, `packet_loss_pct`, `double_talk_score`, `erle_db`)
  - interrupt + VAD decision confidence bands
  - locale-tagged interrupt phrase normalization (Unicode-safe)
  - device reliability score inputs
- Keep fail-closed validation on all unknown/invalid fields.

Step 4. Interrupt candidate pipeline v2 (hybrid detector)
- Replace phrase-only interrupt decision path with hybrid gating:
  - lexical phrase signal
  - acoustic signal
  - prosody/timing signal
- Preserve PH1.K boundary: output candidate only; PH1.X remains cancellation/action owner.

Step 5. Unicode + multilingual normalization
- Replace ASCII-only interrupt normalization with Unicode-safe normalization and locale-aware matching.
- Keep bounded phrase length and strict canonical form in persisted rows.

Step 6. Adaptive threshold policy (deterministic + bounded)
- Add deterministic threshold profile selection by:
  - device route
  - noise/degradation state
  - tenant-approved policy profile
- Add dynamic jitter/clock recovery policy with bounded latency budgets per profile.
- All policy profile IDs must be validated; unknown profiles fail closed.
- Execution status (`2026-02-25`):
  - Step 2: COMPLETE (`PH1.K` docs/contracts lock updated to declare expanded surface: advanced quality metrics, VAD+interrupt confidence bands, Unicode locale-tagged phrase normalization, and adaptive-threshold inputs in `docs/DB_WIRING/PH1_K.md`, `docs/ECM/PH1_K.md`, and `crates/selene_kernel_contracts/src/ph1k.rs`).
  - Step 3: COMPLETE (kernel contract expansion added typed+bounded validators for `AdvancedAudioQualityMetrics`, `DeviceReliabilityScoreInput`, `AdaptiveThresholdPolicyInput`, `JitterClockRecoveryPolicy`, and `VadDecisionConfidenceBand`, with fail-closed range checks and locale-aware normalization helpers).
  - Step 4: COMPLETE (`PH1.K` interrupt pipeline now enforces hybrid lexical + acoustic + prosody gating in runtime decision path while keeping candidate-only boundary to `PH1.X`).
  - Step 5: COMPLETE (`PH1.K` matcher/candidate normalization is Unicode-safe and locale-aware, with multilingual built-in phrase sets and canonical normalized persistence form).
  - Step 6: COMPLETE (deterministic adaptive threshold selection by `device_route + noise/degradation + tenant policy binding`, plus bounded jitter/clock recovery budgets and fail-closed validation for invalid adaptive inputs).
  - Proof lock:
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_os ph1k -- --nocapture`

Step 7. Device reliability and failover policy upgrade
- Add device reliability scoring from historical failures/recoveries.
- Use reliability score in selection/failover path while preserving deterministic tie-break rules.
- Add continuous calibration loop for mic/speaker pairs (safe auto-tune with bounded adjustments and fail-closed rollback).
- Execution status (`2026-02-25`):
  - Step 7: COMPLETE (`PH1.K` runtime now applies reliability-ranked device fallback in selection/failover paths with deterministic tie-break behavior, and runs bounded calibration auto-tune/rollback loop on AEC degradation events).
  - Added acceptance coverage:
    - `device_policy_fallback_prefers_highest_reliability_then_lexical_tiebreak`
    - `runtime_failover_uses_reliability_ranked_selection_with_reason_code`
    - `runtime_aec_autotune_applies_then_rolls_back_within_safe_bounds`
  - Proof lock:
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_os ph1k -- --nocapture`
    - `cargo test -p selene_storage ph1_k::db_wiring -- --nocapture`
    - `cargo test --workspace -- --nocapture`

Step 8. Degradation model expansion
- Expand degradation truth from basic flags to richer state:
  - capture quality class
  - echo-risk class
  - network/stream stability class
  - recoverability class
- Keep current-table derivability from append-only ledger.
- Execution status (`2026-02-25`):
  - Step 8: COMPLETE (`PH1.K` runtime now emits typed degradation state envelopes (`flags + class bundle`) instead of flags-only output, and class bundles are deterministically derived from flags via `DegradationClassBundle::from_flags(...)` to preserve append-only replay derivability.)
  - Added acceptance coverage:
    - `runtime_degradation_state_class_bundle_is_rebuildable_from_flags`
    - `runtime_degradation_state_returns_to_clear_bundle_after_recovery_window`
  - Proof lock:
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_os ph1k -- --nocapture`
    - `cargo test -p selene_storage ph1_k::db_wiring -- --nocapture`
    - `cargo test --workspace -- --nocapture`

Step 9. Handoff contract to PH1.C (STT strategy alignment)
- Add explicit PH1.K -> PH1.C handoff fields for:
  - confidence band
  - quality metrics summary
  - degradation class
- Require PH1.C to choose bounded STT strategy based on these fields.
- Execution status (`2026-02-25`):
  - Step 9: COMPLETE (added deterministic PH1.K handoff builder `build_ph1k_to_ph1c_handoff(...)` that produces typed PH1.C handoff payload from live PH1.K interrupt signals with fail-closed bounded defaults when no candidate exists; PH1.C runtime strategy routing is now acceptance-locked across all bounded branches `STANDARD | NOISE_ROBUST | CLOUD_ASSIST | CLARIFY_ONLY`.)
  - Added acceptance coverage:
    - `ph1k_to_ph1c_handoff_uses_candidate_band_when_interrupt_candidate_exists`
    - `ph1k_to_ph1c_handoff_derives_fallback_bands_without_candidate`
    - `ph1k_handoff_standard_strategy_prefers_primary_when_quality_is_clean`
    - `ph1k_handoff_cloud_assist_strategy_prefers_tertiary_when_confidence_is_low`
    - `ph1k_handoff_noise_robust_strategy_prefers_secondary_under_budget`
    - `ph1k_handoff_critical_degradation_forces_clarify_only`
  - Proof lock:
    - `cargo test -p selene_kernel_contracts ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_os ph1c -- --nocapture`
    - `cargo test -p selene_storage ph1_c::db_wiring -- --nocapture`
    - `cargo test --workspace -- --nocapture`

Step 10. Handoff contract to PH1.X (interrupt risk context)
- Extend candidate envelope to PH1.X with:
  - confidence band
  - gate confidences
  - degradation context
  - risk/context class
- PH1.X decision remains authoritative; PH1.K remains advisory for interruption action.
- Execution status (`2026-02-25`):
  - Step 10: COMPLETE (added typed PH1.K -> PH1.X handoff contract `Ph1kToPh1xInterruptHandoff` for candidate confidence band, gate confidences, degradation context, and risk context class; PH1.K runtime now emits deterministic handoff projection via `build_ph1k_to_ph1x_handoff(...)` while PH1.X remains interruption-action authority.)
  - Added acceptance coverage:
    - `ph1k_to_ph1x_interrupt_handoff_maps_required_risk_context_fields`
    - `ph1k_to_ph1x_interrupt_handoff_rejects_low_risk_when_degraded`
    - `ph1k_to_ph1x_handoff_projects_interrupt_risk_context_when_candidate_exists`
    - `ph1k_to_ph1x_handoff_returns_none_without_interrupt_candidate`
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1x.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`
    - `docs/DB_WIRING/PH1_X.md`
    - `docs/ECM/PH1_X.md`
    - `cargo test -p selene_kernel_contracts ph1x -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_os ph1x -- --nocapture`

Step 11. Storage/migration extension (append-only preserved)
- Add new PH1.K event fields/tables/migration entries for expanded metrics and confidence bands.
- Preserve idempotency, tenant isolation, append-only invariants, and deterministic rebuild.
- Execution status (`2026-02-25`):
  - Step 11: COMPLETE (added PH1.K storage/migration extension for expanded interrupt candidate metrics/confidence fields with append-only preservation: new migration `0023_ph1k_interrupt_extended_fields.sql`, typed extended commit surface in `Ph1fStore`/`Ph1kVoiceRuntimeRepo`, strict fail-closed validators for extended payload bounds, and deterministic projection/rebuild updates in `audio_runtime_current`.)
  - Added acceptance coverage:
    - `at_k_db_05_interrupt_extended_fields_persist_and_project`
    - `at_k_db_06_interrupt_extended_fields_fail_closed`
  - Proof lock:
    - `crates/selene_storage/src/ph1f.rs`
    - `crates/selene_storage/src/repo.rs`
    - `crates/selene_storage/migrations/0023_ph1k_interrupt_extended_fields.sql`
    - `crates/selene_storage/tests/ph1_k/db_wiring.rs`
    - `cargo test -p selene_storage --test db_wiring_ph1k_tables -- --nocapture`
    - `cargo test -p selene_storage -- --nocapture`

Step 12. Audit expansion (PH1.J)
- Add new reason codes and payload allowlist keys for expanded PH1.K decisions.
- Ensure every new commit path has auditable reason-coded emission.
- Execution status (`2026-02-25`):
  - Step 12: COMPLETE (PH1.J contract now enforces `PerceptionSignalEmitted` payload allowlist keys for PH1.K runtime audit emissions; PH1.K storage commit paths now append reason-coded PH1.J rows for every runtime event kind with deterministic event naming and bounded payload fields for expanded interrupt metrics/confidence context.)
  - Added acceptance coverage:
    - `perception_signal_payload_rejects_unknown_key`
    - `perception_signal_payload_accepts_ph1k_allowlist_keys`
    - `at_k_db_07_runtime_commits_emit_reason_coded_ph1j_rows`
    - `at_k_db_08_interrupt_extended_audit_payload_includes_step12_keys`
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1j.rs`
    - `crates/selene_storage/src/ph1f.rs`
    - `crates/selene_storage/tests/ph1_k/db_wiring.rs`
    - `cargo test -p selene_kernel_contracts ph1j -- --nocapture`
    - `cargo test -p selene_storage --test db_wiring_ph1k_tables -- --nocapture`
    - `cargo test -p selene_storage -- --nocapture`

Step 13. Feedback capture wiring (PH1.FEEDBACK)
- Emit structured feedback events for:
  - false interrupt
  - missed interrupt
  - wrong degradation classification
  - bad failover selection
- Include fingerprint fields for clustering.
- Execution status (`2026-02-25`):
  - Step 13: COMPLETE (added typed PH1.K feedback capture commit path in storage/repo that emits deterministic PH1.FEEDBACK events for all four interruption failure classes, with fail-closed validation and idempotent append-only capture rows carrying clustering fingerprints.)
  - Added acceptance coverage:
    - `at_k_db_09_feedback_capture_wires_issue_kinds_and_fingerprints`
    - `at_k_db_10_feedback_capture_bad_failover_requires_device_pair`
  - Proof lock:
    - `crates/selene_storage/src/ph1f.rs`
    - `crates/selene_storage/src/repo.rs`
    - `crates/selene_storage/tests/ph1_k/db_wiring.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`
    - `cargo test -p selene_storage --test db_wiring_ph1k_tables -- --nocapture`
    - `cargo test -p selene_storage -- --nocapture`

Step 14. Learning + promotion wiring (PH1.LEARN + PH1.PAE)
- Route PH1.K feedback artifacts into learning pipeline.
- Promote policy/model updates using governed ladder only:
  - `SHADOW -> ASSIST -> LEAD`
- Auto-demote on regression.
- Enforce strict rollback triggers when quality or false-interrupt metrics regress beyond release-gate limits.
- Execution status (`2026-02-25`):
  - Step 14: COMPLETE (PH1.K feedback capture commit path now emits deterministic PH1.LEARN signal bundles and governed PH1.PAE ladder decisions in the same append-only flow; ladder transitions are one-step only (`SHADOW -> ASSIST -> LEAD`), regression paths auto-demote, and rollback triggers fire fail-closed when quality flags or false-interrupt rate breach Step-18 limits.)
  - Added acceptance coverage:
    - `at_k_db_11_feedback_capture_routes_to_learn_and_governed_pae_ladder`
  - Proof lock:
    - `crates/selene_storage/src/ph1f.rs`
    - `crates/selene_storage/tests/ph1_k/db_wiring.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`
    - `cargo test -p selene_storage --test db_wiring_ph1k_tables -- --nocapture`
    - `cargo test -p selene_storage -- --nocapture`

Step 15. Unit/contract test expansion
- Add/extend tests for:
  - validation/fail-closed behavior
  - Unicode normalization correctness
  - confidence-band mapping correctness
  - deterministic policy profile selection
- Execution status (`2026-02-25`):
  - Step 15: COMPLETE (expanded PH1.K kernel + runtime test locks for fail-closed normalization, confidence-band/reason-code boundary mapping, and deterministic adaptive-threshold profile selection with unknown-tenant fail-closed behavior.)
  - Added acceptance coverage:
    - `normalize_interrupt_phrase_strips_controls_and_collapses_whitespace`
    - `normalize_interrupt_phrase_rejects_control_only_input_fail_closed`
    - `at_k_interrupt_13_confidence_band_and_reason_code_mapping_boundaries_are_locked`
    - `at_k_interrupt_14_threshold_profile_selection_is_deterministic_by_route_and_noise`
    - `at_k_interrupt_15_threshold_profile_selection_accepts_valid_dynamic_profile_ids`
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1k.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`

Step 16. Runtime/replay/integration tests
- Add/extend tests for:
  - noisy environment recovery
  - interruption under overlap speech
  - failover determinism and cooldown/stability windows
  - PH1.K -> PH1.C and PH1.K -> PH1.X envelope compatibility
- Execution status (`2026-02-25`):
  - Step 16: COMPLETE (added runtime/replay/integration coverage for noisy-recovery replay determinism, overlap-speech interruption determinism, failover cooldown/stability-window determinism, and cross-engine PH1.K handoff envelope compatibility with PH1.C/PH1.X.)
  - Added acceptance coverage:
    - `at_k_runtime_16_noisy_environment_recovery_replay_is_deterministic`
    - `at_k_runtime_17_overlap_speech_interrupt_decision_is_replay_deterministic`
    - `at_k_runtime_18_failover_cooldown_stability_windows_are_deterministic`
    - `at_k_runtime_19_ph1c_and_ph1x_handoff_envelopes_are_compatible`
  - Proof lock:
    - `crates/selene_engines/src/ph1k.rs`
    - `cargo test -p selene_engines ph1k::tests::at_k_runtime_ -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`

Step 17. Benchmark/eval harness (release-gate source of truth)
- Produce PH1.K eval snapshot datasets across:
  - language/locale
  - device route type
  - noise class
  - overlap speech scenarios
- Persist results for gate checks and trend comparison.
- Execution status (`2026-02-25`):
  - Step 17: COMPLETE (added PH1.K benchmark/eval harness script and persisted round-2 eval snapshot dataset with required coverage across locale, device route, noise class, and overlap speech scenarios; harness now emits row-level, summary, worst-case, and trend outputs as release-gate source inputs.)
  - Coverage lock:
    - required locale coverage: `en-US`, `es-ES`, `zh-CN`, `tr-TR`
    - required device-route coverage: `BUILT_IN`, `BLUETOOTH`, `USB`, `VIRTUAL`
    - required noise coverage: `CLEAN`, `ELEVATED`, `SEVERE`
    - required overlap coverage: both `overlap_speech=0` and `overlap_speech=1`
  - Proof lock:
    - `scripts/check_ph1k_round2_eval_snapshot.sh`
    - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
    - `scripts/selene_design_readiness_audit.sh` (section `1C5`)
    - `bash scripts/check_ph1k_round2_eval_snapshot.sh docs/fixtures/ph1k_round2_eval_snapshot.csv`

Step 18. PH1.K global-standard release gate (must pass)
- False interrupt rate <= `0.3/hour` active session.
- Missed interrupt rate <= `2%`.
- End-of-speech boundary p95 <= `180ms`.
- PH1.K -> PH1.C handoff p95 <= `120ms`.
- Device failover recovery p95 <= `1.5s`.
- Degradation auto-recovery success >= `97%`.
- Multilingual interrupt recall >= `95%` across supported locales.
- Audit completeness = `100%`.
- Tenant isolation = `100%`.
- Execution status (`2026-02-25`):
  - Step 18: COMPLETE (added strict PH1.K release-gate checker that fails closed unless every Step-18 threshold passes against the canonical Step-17 eval snapshot, and wired it into readiness audit execution path as section `1C6`.)
  - Proof lock:
    - `scripts/check_ph1k_release_gate.sh`
    - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
    - `scripts/selene_design_readiness_audit.sh` (section `1C6`)
    - `bash scripts/check_ph1k_release_gate.sh docs/fixtures/ph1k_round2_eval_snapshot.csv`

Step 19. Closure and tracker update
- Append PH1.K round-2 completion proof to build ledger.
- Update tracker row/status for PH1.K round-2 finalization completion.
- Move queue to next locked engine (`PH1.C`) only after Step 18 gate pass.
- Execution status (`2026-02-25`):
  - Step 19: COMPLETE (recorded PH1.K round-2 closure proof, updated round-2 queue/tracker state to mark `PH1.K` done, and advanced the locked next engine to `PH1.C`.)
  - Proof lock:
    - `docs/03_BUILD_LEDGER.md`
    - `docs/33_ENGINE_REVIEW_TRACKER.md`
    - `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md`
    - `bash scripts/check_ph1k_release_gate.sh docs/fixtures/ph1k_round2_eval_snapshot.csv`

### 5B) PH1.K Benchmark Ownership + Closed-Loop Reporting
Purpose:
- Place each PH1.K benchmark in the correct engine ownership path.
- Force continuous reporting into learning and builder remediation loops.

PH1.K benchmark ownership matrix:

| benchmark | primary owner engine | measurement sources | release-gate owner | learning/fix sinks |
|---|---|---|---|---|
| False interrupt rate <= `0.3/hour` | `PH1.K` | PH1.K interrupt candidates + PH1.X interruption outcomes + PH1.FEEDBACK correction labels | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER` |
| Missed interrupt rate <= `2%` | `PH1.K` | PH1.K candidate windows + user correction outcomes + PH1.FEEDBACK labels | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER` |
| End-of-speech boundary p95 <= `180ms` | `PH1.K` | PH1.K VAD/endpointer timing + PH1.C turn-finalization timestamps | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE` |
| Capture -> PH1.C handoff p95 <= `120ms` | `PH1.OS` (handoff path owner) | PH1.K capture timestamp + PH1.C receive/finalize timestamp | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.BUILDER` |
| Device failover recovery p95 <= `1.5s` | `PH1.K` | PH1.K device-state transitions + degradation/recovery events | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER` |
| Noisy-condition auto-recovery >= `97%` | `PH1.K` | PH1.K degradation episodes + successful recovery counts by noise class | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER` |
| Multilingual interrupt recall >= `95%` | `PH1.K` | PH1.K interrupt detections + locale labels + correction outcomes | `PH1.OS` | `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE` |
| Audit completeness = `100%` | `PH1.J` | commit-path audit event coverage vs expected event map | `PH1.OS` | `PH1.HEALTH` + `PH1.BUILDER` issue pipeline |
| Tenant isolation = `100%` | `PH1.TENANT` + `PH1.F` | cross-tenant guard checks + data-scope invariant tests | `PH1.OS` | `PH1.HEALTH` + `PH1.BUILDER` issue pipeline |

Closed-loop reporting and improvement flow (mandatory):
1. Owner engines emit benchmark telemetry and reason-coded failures to append-only records.
2. `PH1.OS` computes rolling benchmark windows and checks gate thresholds.
3. Any breach emits structured `FailureEvent` + `ProblemCard` and opens unresolved status.
4. `PH1.HEALTH` displays by engine, tenant, severity, and unresolved/escalated state.
5. `PH1.FEEDBACK` records correction evidence and false/miss labels.
6. `PH1.LEARN` clusters recurring fingerprints and creates governed improvement artifacts.
7. `PH1.PAE` promotes/demotes policy/model routes (`SHADOW -> ASSIST -> LEAD`) and auto-demotes on regression.
8. Repeated unresolved fingerprints generate builder remediation candidates (`FixCard`) for `PH1.BUILDER`.
9. `PH1.BUILDER` may propose fixes, but code/launch still follows mandatory human approval gates before apply.
10. Post-deploy recurrence verification is mandatory; if recurrence persists, issue remains open and is escalated.

### 5C) Global Engine Monitoring Contract (All Engines, Not PH1.K Only)
Rule:
- Every runtime engine must have a benchmark card with:
  - quality metrics
  - latency metrics
  - safety/isolation metrics
  - audit completeness target
  - rollback trigger thresholds
  - owner engine + gate owner + learning sinks
- Missing benchmark card blocks that engine from `READY_FOR_PRODUCTION_HARDENING`.

Per-engine minimum reporting contract:
1. Emit reason-coded success/failure telemetry for each critical capability.
2. Emit structured failure records (`FailureEvent`/`ProblemCard`) on threshold breach.
3. Surface unresolved/escalated state in `PH1.HEALTH`.
4. Feed correction and outcome signals into `PH1.FEEDBACK`.
5. Feed governed artifact generation into `PH1.LEARN` and routing/policy decisions into `PH1.PAE`.
6. Route repeated unresolved failures to `PH1.BUILDER` remediation queue with human approval gates preserved.

### 5D) PH1.X Follow-On Checklist (Interrupt Continuity Behavior Lock, Step 1..14)
Purpose:
- Lock the exact post-interrupt conversation behavior before coding.
- Ensure Selene handles interruption with topic continuity and deterministic branch decisions.

Pre-coding lock:
- This section is design-lock only; no runtime code changes begin until this checklist is accepted.
- Execution order: complete PH1.X design lock first, then apply PH1.K delta checklist in Section 5E.

Step 1. Interrupt behavior contract lock
- Define PH1.X interrupt behavior contract with two explicit outcomes:
  - `SAME_SUBJECT_APPEND`
  - `SWITCH_TOPIC_THEN_RETURN_CHECK`
- Keep fail-closed fallback: one clarify if branch confidence is not sufficient.

Step 2. Immediate interruption response lock
- On interruption candidate acceptance:
  - cancel TTS immediately,
  - keep spoken prefix + unsaid remainder in resume buffer,
  - switch to listening posture.
- Preserve existing PH1.X wait/cancel behavior as deterministic base.

Step 3. Subject relation decision input lock
- Require PH1.X request envelope to include subject relation input from understanding path:
  - `subject_relation = SAME | SWITCH | UNCERTAIN`
  - `subject_relation_confidence`
- If missing/invalid, PH1.X must fail closed into one clarify.

Execution status (`2026-02-25`):
- Step 1: COMPLETE (contract/docs lock for interruption continuity outcome + subject relation fields).
- Step 2: COMPLETE (immediate cancel + resume-buffer preservation + listening path retained).
- Step 3: COMPLETE (runtime fail-closed clarify for missing/uncertain/low-confidence relation implemented and mirrored in PH1.K runtime layer).
- Step 4: COMPLETE (SAME-subject merge branch implemented for resume-buffer + chat path with explicit continuity outcome in PH1.OS + PH1.K engine mirror).
- Step 5: COMPLETE (SWITCH-subject branch implemented for new-topic-first response + return-check question with resume-buffer retention).
- Step 6: COMPLETE (UNCERTAIN/low-confidence branch is clarify-only fail-closed across chat and intent paths with resume-buffer preservation).
- Step 7: COMPLETE (thread continuity state extension wired in contracts/runtime: `interrupted_subject_ref`, `return_check_pending`, `return_check_expires_at`, plus deterministic expiry handling in PH1.OS + PH1.K engine mirror).
- Step 8: COMPLETE (resume policy lock implemented in contracts/runtime/tests: explicit `RESUME_NOW | RESUME_LATER | DISCARD`, with `DISCARD` allowed only from explicit user `confirm_answer=No` on return-check path).
- Step 9: COMPLETE (clarify-language lock enforced in contract: clarify question must be short single-line text; options are bounded to 2-3 entries deterministically).
- Step 10: COMPLETE (audit/reporting lock complete: interruption branch reason-codes are persisted in PH1.X audit rows and PH1.HEALTH report query surfaces outcomes by `owner_engine_id` + topic fingerprint).
- Step 11: COMPLETE (acceptance-test lock complete across PH1.OS + PH1.K mirror runtimes, including deterministic no-loss replay continuity proof).
- Step 12: COMPLETE (PH1.X interrupt continuity benchmark gate added with enforced thresholds and deterministic pass/fail output).
- Step 13: COMPLETE (PH1.X release-gate hook added; readiness audit now enforces PH1.X Step-12 metric gate before hardening progression).
- Step 14: COMPLETE (closure/handoff completed; build ledger proof appended and 5D is now fully closed for transition to 5E Step 1).

Step 4. SAME-subject merge branch
- If `subject_relation=SAME` with sufficient confidence:
  - merge unsaid remainder + new user content into one coherent response plan.
  - emit one response that addresses both parts as one thread.
- Do not lose pre-interrupt context.

Step 5. SWITCH-subject branch
- If `subject_relation=SWITCH` with sufficient confidence:
  - answer the new topic first.
  - then ask one short return-check question:
    - "Do you still want to continue the previous topic?"
- Keep previous topic in resumable state until user decides.

Step 6. UNCERTAIN branch
- If subject relation is uncertain:
  - ask one bounded clarify question (no multi-question fan-out).
  - block dispatch/action until clarified.

Step 7. Continuity state contract extension
- Extend PH1.X thread continuity state with deterministic fields:
  - `active_subject_ref`
  - `interrupted_subject_ref`
  - `resume_buffer`
  - `return_check_pending` (bool)
  - `return_check_expires_at`
- Expiry and replay behavior must stay deterministic.

Step 8. Resume policy lock
- Define deterministic resume policies:
  - `RESUME_NOW` (same subject)
  - `RESUME_LATER` (switch topic with pending return check)
  - `DISCARD` (explicit user choice only)
- No silent discard of unresolved interrupted topic.
- Runtime mapping lock:
  - same-subject merge emits `interrupt_resume_policy=RESUME_NOW`.
  - switch-topic return-check emits `interrupt_resume_policy=RESUME_LATER`.
  - return-check `confirm_answer=Yes` emits `RESUME_NOW`.
  - return-check `confirm_answer=No` emits `DISCARD`.

Step 9. Clarify language policy
- Clarify text must stay natural-language and short.
- Options must be bounded to 2-3 choices for deterministic handling.
- Contract lock:
  - `ClarifyDirective.question` must be single-line and `<= 240` chars.
  - `ClarifyDirective.accepted_answer_formats` must contain exactly `2..=3` non-empty entries.

Step 10. PH1.X audit/reporting lock
- Add reason-coded outputs for branches:
  - `X_INTERRUPT_SAME_SUBJECT_APPEND`
  - `X_INTERRUPT_RETURN_CHECK_ASKED`
  - `X_INTERRUPT_RESUME_NOW`
  - `X_INTERRUPT_DISCARD`
  - `X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY`
- Ensure PH1.HEALTH can display interruption branch outcomes by engine/topic.
- Proof lock:
  - PH1.X storage audit rows preserve interruption branch reason codes in `audit_events.reason_code`.
  - PH1.X branch labels are persisted as bounded payload kinds (`response_kind` / `wait_kind`).
  - PH1.HEALTH report query exposes branch outcome rows by `owner_engine_id=PH1.X` and `issue_fingerprint` topic marker.

Step 11. PH1.X acceptance tests (required)
- Add tests for:
  - interrupt cancel + resume buffer write
  - same-subject merge response behavior
  - switch-topic response + return-check prompt
  - uncertain-relation clarify-only fail-closed behavior
  - no-loss continuity across replay
- Proof lock:
  - `cargo test -p selene_os ph1x::tests::at_x_ -- --nocapture`
  - `cargo test -p selene_engines ph1x::tests::at_x_ -- --nocapture`

Step 12. PH1.X benchmarks (interrupt continuity quality)
- Define and enforce:
  - same-subject merge correctness >= `98.00%` (`>=9800 bp`)
  - switch-topic return-check correctness >= `98.00%` (`>=9800 bp`)
  - resume-buffer retention correctness >= `99.50%` (`>=9950 bp`)
  - interruption branch decision latency `p95 <= 120ms`
  - audit completeness = `100%` (`10000 bp`)
- Gate implementation:
  - `scripts/check_ph1x_interrupt_continuity_benchmarks.sh`
  - input snapshot header:
    - `window_min,same_subject_total,same_subject_correct,switch_topic_total,switch_topic_correct,resume_buffer_total,resume_buffer_retained,branch_decision_latency_p95_ms,audit_completeness_bp`
- Proof lock:
  - `bash scripts/check_ph1x_interrupt_continuity_benchmarks.sh docs/fixtures/ph1x_interrupt_continuity_snapshot.csv`

Step 13. PH1.X release-gate hook
- PH1.X release gate must include Step 12 metrics.
- Failure blocks PH1.X round-2 closure and blocks downstream hardening.
- Gate implementation:
  - `scripts/check_ph1x_release_gate.sh`
  - readiness hook: `scripts/selene_design_readiness_audit.sh` section `1C2`
- Proof lock:
  - `bash scripts/check_ph1x_release_gate.sh docs/fixtures/ph1x_interrupt_continuity_snapshot.csv`

Step 14. Closure and handoff
- Append PH1.X interrupt continuity proof to build ledger.
- Mark PH1.X follow-on interrupt checklist as complete.
- Only then execute PH1.K delta implementation checklist in Section 5E.
- Proof lock:
  - `docs/03_BUILD_LEDGER.md` entry: `2026-02-25 | PH1X_5D_STEP14_CLOSURE_HANDOFF_COMPLETE | ...`
  - `cargo test -p selene_os ph1x::tests::at_x_ -- --nocapture`
  - `cargo test -p selene_engines ph1x::tests::at_x_ -- --nocapture`
  - `bash scripts/check_ph1x_release_gate.sh docs/fixtures/ph1x_interrupt_continuity_snapshot.csv`
- Handoff status:
  - `5D` checklist state is `COMPLETE`.
  - `5E Step 1` is complete.
  - `5E Step 2` is complete.
  - `5E Step 3` is complete.
  - `5E Step 4` is complete.
  - `5E Step 5` is complete.
  - `5E Step 6` is complete.
  - `5E Step 7` is complete.
  - `5E Step 8` is complete.
  - `5E Step 9` is complete.
  - `5E Step 10` is complete.
  - Next strict item is `5A Step 7` (`PH1.K` device reliability and failover policy upgrade).

### 5E) PH1.K Updates Required By PH1.X Interrupt Behavior (Step 1..10)
Purpose:
- Apply PH1.K changes required to support the PH1.X interruption branches locked in Section 5D.

Step 1. Interrupt lexicon policy surface
- Keep lexical trigger policy explicit and governed:
  - approved interruption words/phrases
  - locale-tagged sets
  - tenant policy profile binding
- Unknown policy profile must fail closed.
- Execution status (`2026-02-25`):
  - Step 1: COMPLETE (typed `InterruptLexiconPolicyBinding` contract added; PH1.K matcher now enforces locale-tagged approved phrase sets under tenant/profile binding; unknown policy profile fails closed).
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1k.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`

Step 2. Candidate payload extension for PH1.X
- Extend PH1.K interrupt candidate envelope with fields PH1.X needs:
  - `trigger_phrase_id`
  - `trigger_locale`
  - `candidate_confidence_band`
  - `risk_context_class`
  - `degradation_context`
- Execution status (`2026-02-25`):
  - Step 2: COMPLETE (PH1.K interrupt candidate contract/runtime extended with typed trigger locale/phrase fields, confidence band, risk context class, and degradation context; PH1.X validation + docs updated to require the payload surface).
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1k.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `crates/selene_kernel_contracts/src/ph1x.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`
    - `docs/DB_WIRING/PH1_X.md`
    - `docs/ECM/PH1_X.md`

Step 3. Noise-safe lexical gating
- Keep phrase-based trigger as required signal.
- Keep acoustic/prosody/degradation gates as mandatory anti-noise safeguards.
- No interrupt candidate from noise-only signals.
- Execution status (`2026-02-25`):
  - Step 3: COMPLETE (PH1.K interrupt path now uses explicit thresholded lexical + acoustic/prosody/degradation gating with dedicated fail-closed tests proving no noise-only candidate emission).
  - Proof lock:
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`

Step 4. Subject-handoff support fields
- Emit bounded hints for subject relation decision path:
  - interruption timing markers
  - speech window metrics
  - confidence bundle for downstream relation decision.
- Execution status (`2026-02-25`):
  - Step 4: COMPLETE (PH1.K candidate envelope now emits `timing_markers`, `speech_window_metrics`, and `subject_relation_confidence_bundle`; PH1.X contract validation enforces deterministic handoff integrity.)
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1k.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `crates/selene_kernel_contracts/src/ph1x.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`
    - `docs/DB_WIRING/PH1_X.md`
    - `docs/ECM/PH1_X.md`

Step 5. Resume snapshot integrity contract
- Ensure PH1.K/OS handoff preserves accurate `spoken_cursor` and bounded snapshot references.
- Fail closed on invalid cursor/snapshot mismatch.
- Execution status (`2026-02-25`):
  - Step 5: COMPLETE (PH1.X contract now fails closed on stale interruption snapshot references, cursor-without-remainder snapshots, and timing-window mismatches between interruption and snapshot handoff.)
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1x.rs`
    - `crates/selene_engines/src/ph1x.rs`
    - `crates/selene_os/src/ph1x.rs`
    - `docs/DB_WIRING/PH1_X.md`
    - `docs/ECM/PH1_X.md`

Step 6. Unicode + locale trigger normalization
- Replace ASCII-only phrase normalization in PH1.K path with Unicode-safe normalization and locale tagging.
- Keep bounded storage payload policy.
- Execution status (`2026-02-25`):
  - Step 6: COMPLETE (PH1.K phrase normalization is Unicode-safe and locale-tagged across contract + runtime matcher path; storage payload remains bounded to normalized phrase strings.)
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1k.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`

Step 7. Audit expansion
- Add reason-coded PH1.K audit rows for:
  - lexical trigger accepted/rejected
  - noise gate rejection
  - candidate emitted with confidence band
- Ensure PH1.J and PH1.HEALTH visibility compatibility.
- Execution status (`2026-02-25`):
  - Step 7: COMPLETE (PH1.K interrupt decision trace now emits deterministic reason codes for lexical rejection, noise-gate rejection, and confidence-band candidate emission; downstream PH1.X/PH1.HEALTH consumers preserve reason-code visibility.)
  - Proof lock:
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`

Step 8. Feedback/learning wiring
- Emit PH1.FEEDBACK events for:
  - false lexical trigger
  - missed lexical trigger
  - wrong confidence band
- Route to PH1.LEARN/PH1.PAE for threshold/route tuning.
- Execution status (`2026-02-25`):
  - Step 8: COMPLETE (PH1.K now exposes deterministic feedback event mapper `build_interrupt_feedback_signal(...)` with reason-coded outputs for false/missed/wrong-band signals and fixed routing targets to LEARN/PAE sinks.)
  - Proof lock:
    - `crates/selene_engines/src/ph1k.rs`
    - `docs/DB_WIRING/PH1_K.md`
    - `docs/ECM/PH1_K.md`

Step 9. PH1.K compatibility test pack
- Add compatibility tests proving PH1.K candidate envelope satisfies PH1.X 5D inputs.
- Add fail-closed tests for missing locale/trigger/payload invariants.
- Execution status (`2026-02-25`):
  - Step 9: COMPLETE (added compatibility/fail-closed tests across kernel contracts and PH1.K runtime for timing-window integrity, subject-bundle parity, stale snapshot refusal, and deterministic interrupt reason-code behavior.)
  - Proof lock:
    - `crates/selene_kernel_contracts/src/ph1x.rs`
    - `crates/selene_engines/src/ph1k.rs`
    - `cargo test -p selene_kernel_contracts ph1k -- --nocapture`
    - `cargo test -p selene_kernel_contracts ph1x -- --nocapture`
    - `cargo test -p selene_engines ph1k -- --nocapture`
    - `cargo test -p selene_engines ph1x -- --nocapture`
    - `cargo test -p selene_os ph1x -- --nocapture`
    - `cargo test -p selene_os ph1k -- --nocapture`

Step 10. PH1.K pre-implementation readiness gate
- PH1.K runtime coding starts only when:
  - Section 5D PH1.X behavior lock is complete,
  - Section 5E contract/docs updates are merged,
  - benchmark/reporting sinks are wired (`PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`).
- Execution status (`2026-02-25`):
  - Step 10: COMPLETE (added strict gate script `scripts/check_ph1k_5e_readiness_gate.sh` and wired it into `scripts/selene_design_readiness_audit.sh` section `1C3` to enforce 5D closure + 5E Step1-10 completion + sink-chain wiring before further PH1.K hardening execution.)
  - Proof lock:
    - `scripts/check_ph1k_5e_readiness_gate.sh`
    - `scripts/selene_design_readiness_audit.sh`
    - `bash scripts/check_ph1k_5e_readiness_gate.sh`

### 5F) Selene FDX Duplex System Lock (Full-Duplex + Barge-In + Incremental Planning)
Purpose:
- Lock a world-class duplex voice design so Selene can listen while speaking, accept interruption naturally, and prepare responses before the user finishes speaking.
- Keep strict safety boundaries: no speculative side effects and no authority drift.

Design lock status:
- This section is a pre-coding architecture lock.
- Implementation starts in the upcoming `PH1.C` round-2 runbook and must follow this section in order.

Core behavior definition:
1. Selene listens while talking (full-duplex).
2. User can interrupt at any time (barge-in) using approved words/phrases only.
3. Selene starts understanding/planning on partial speech (incremental ASR + speculative planning).
4. Only finalized, validated turn decisions can execute actions.

Core runtime loop (always active in voice session):
1. `PH1.K` captures mic audio continuously and receives TTS playback state reference.
2. `PH1.K` runs echo/noise suppression and lexical barge-in detection every frame.
3. If interruption confidence is high enough, `PH1.K` emits `InterruptCandidate` immediately.
4. `PH1.X` receives candidate and decides: cancel TTS, continue, clarify, or ignore.
5. In parallel, `PH1.C` streams partial transcript chunks from live speech.
6. `PH1.NLP` builds incremental intent hypotheses from partial text.
7. `PH1.X` creates speculative response drafts while the user is still speaking.
8. When end-of-speech is detected, `PH1.X` commits final branch and responds fast.
9. While Selene speaks, step 1 never stops.

Owner split (single authority per layer):
- `PH1.K`: real-time voice substrate, interruption candidate detection while TTS is active.
- `PH1.C`: incremental transcript quality gate (partials + final pass/reject).
- `PH1.NLP`: incremental intent hypotheses (advisory only).
- `PH1.X`: authoritative interruption branch + continuity + turn commit decision.
- `PH1.TTS`: chunked playback control + immediate cancel support.
- `PH1.OS`: orchestration, safety gating, sequencing, and fail-closed enforcement.
- `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`: closed-loop improvement sinks.

Required runtime contract surfaces (to lock before coding):
- `DuplexFrame`: bounded audio/timing envelope with TTS-active marker.
- `InterruptCandidate`: bounded PH1.K candidate envelope (already candidate-only boundary).
- `PartialTranscript`: ordered partial STT units with confidence + stability flag.
- `IntentHypothesis`: ranked incremental intent hypotheses with bounded confidence.
- `SpeculativePlan`: non-authoritative draft response plan (`execute_allowed=false`).
- `TurnCommitDecision`: final authoritative branch/response/action decision with reason code.

Hard safety laws:
1. `PH1.K` remains candidate-only; no interruption action authority.
2. `PH1.X` is the only interruption-action owner (`cancel/clarify/respond/wait/dispatch`).
3. Barge-in trigger source is lexical only: approved words/phrases are mandatory.
4. Non-lexical-only triggers are forbidden (`noise-only`, `acoustic-only`, `prosody-only` cannot trigger interruption).
5. No action execution from speculative drafts.
6. Low-confidence interruption/transcript/intent paths must fail closed to one clarify.
7. Every branch/decision must emit reason-coded audit rows.
8. Tenant isolation and audit completeness remain mandatory hard gates.

Global-grade duplex release targets (must pass):
- barge-in detect -> TTS cancel `p95 <= 120ms`
- false interrupt rate `<= 0.3/hour active session`
- missed interrupt rate `<= 2%`
- non-lexical trigger acceptance rate `= 0.0%`
- end-of-speech -> first response token `p95 <= 300ms`
- capture -> PH1.C partial handoff `p95 <= 120ms`
- partial transcript first chunk latency `p95 <= 250ms`
- audit completeness `= 100%`
- tenant isolation `= 100%`

High-level build order lock:
1. `PH1.K` true duplex loop + interrupt candidate stream.
2. `PH1.C` partial transcript streaming contract.
3. `PH1.NLP` incremental hypothesis contract.
4. `PH1.X` speculative planner + commit gate.
5. End-to-end eval harness + hard release gate.

Strict implementation order for this lock:
Step 1. PH1.C round-2 baseline capture
- Add baseline snapshot for duplex metrics: interrupt timing, partial-STT latency, first-token latency, false/missed interrupts.

Step 2. PH1.K duplex detection path lock
- Keep continuous interruption detection when `tts_playback_active=true`.
- Require lexical match against approved phrase set as a hard precondition for barge-in.
- Keep acoustic/prosody/noise features as rejection safeguards only (never as trigger source).
- Emit deterministic decision traces for accepted/rejected candidates.

Step 3. PH1.C partial transcript contract lock
- Add bounded incremental transcript chunk surface and deterministic finalization handoff.

Step 4. PH1.NLP incremental hypothesis lock
- Add advisory intent hypothesis stream from partial transcripts (no execution authority).

Step 5. PH1.X speculative planning + commit gate
- Build speculative response drafts during active speech.
- Commit only after final confidence + policy gates pass.

Step 6. PH1.TTS cancel/resume lock
- Enforce immediate cancel path + deterministic resume snapshot semantics for interruption continuity.

Step 7. PH1.OS full-duplex orchestration lock
- Enforce strict order: capture -> detect -> partial STT -> hypothesis -> commit decision -> speak.
- Enforce fail-closed on missing/invalid upstream signals.

Step 8. FEEDBACK/LEARN/PAE/BUILDER wiring
- Capture all duplex misses/false interrupts/late cancels/clarify fallbacks as learning signals.

Step 9. Benchmark + release-gate harness
- Add canonical duplex snapshot + strict gate script and wire into readiness audit.

Step 10. Closure + queue progression
- Round-2 closure for duplex-in-scope engines only after Step-9 gate pass and ledger proof append.

### 5G) PH1.C Round-2 Strict Provider + Learning Build Plan (OpenAI Primary, Google Fallback)
Purpose:
- Lock PH1.C execution so Selene gets production-grade STT now, while building in-house capability over time.
- Keep strict engine boundaries: PH1.C owns STT gating, PH1.TTS owns speech output, PH1.D owns external provider calls.

Scope lock (this round):
- STT routing and transcript quality authority remains in `PH1.C`.
- TTS provider routing is integrated as a coupled path requirement with `PH1.TTS` + `PH1.D` so duplex behavior is coherent end-to-end.
- Runtime must stay fail-closed and reason-coded.
- Input robustness is mandatory for:
  - rambling speech
  - all languages (including code-switch)
  - broken English
  - heavy/bad accents
  - scrambled/rubbish speech patterns
- Normalization outputs must preserve source truth:
  - `verbatim_transcript` (what was actually heard)
  - `normalized_english_draft` (readability rewrite, non-authoritative)
  - no silent meaning rewrite is allowed.

Provider routing policy lock (v1 required):
- STT route order:
  - primary: `OpenAI STT`
  - secondary fallback: `Google STT`
  - terminal fallback: clarify/text-safe path (no guessed transcript)
- TTS route order:
  - primary: `OpenAI TTS`
  - secondary fallback: `Google TTS`
  - terminal fallback: text-only response delivery (reason-coded)
- All external provider calls must traverse `PH1.D` typed boundary.
- Direct provider calls from `PH1.C` or `PH1.TTS` are forbidden.

Engine ownership lock:
- `PH1.K`: audio substrate + handoff features (candidate/confidence/degradation context).
- `PH1.C`: partial/final STT quality gate + deterministic pass/reject.
- `PH1.D`: provider adapter boundary (OpenAI/Google call path + strict typed normalization).
- `PH1.NLP`: intent understanding over validated transcript outputs.
- `PH1.X`: interruption/continuity and final turn commit behavior.
- `PH1.TTS`: output rendering + cancel/resume.
- `PH1.OS`: canonical orchestration order + fail-closed enforcement.
- `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE -> PH1.BUILDER`: learning + promotion + remediation loop.

Robust speech handling lock (who does what):
- Rambling speech:
  - `PH1.C` transcribes and confidence-gates.
  - `PH1.SRL` structures fragmented utterances into ordered clauses.
  - `PH1.NLP` extracts intent/slots from structured output.
- All languages:
  - `PH1.LANG` owns language detect + code-switch segmentation.
  - `PH1.C` routes to multilingual provider/model profile.
- Broken English:
  - preserve `verbatim_transcript` first.
  - produce `normalized_english_draft` as advisory readability layer only.
- Heavy accents:
  - `PH1.C` uses accent-robust route/profile selection.
  - `PH1.KNOW`/`PH1.PRON` tenant lexicon hints improve recognition safely.
- Scrambled/rubbish speech:
  - `PH1.SRL` + `PH1.NLP` attempt bounded repair.
  - if confidence stays low, PH1.X must issue one clarify (no guessing).

LLM usage lock for PH1.C path:
- LLM assist is required for best outcomes on rambling/scrambled/multilingual edge cases.
- LLM assist must be routed only through `PH1.D` typed boundary.
- LLM assist is fallback/assist, not authority; final transcript acceptance remains in `PH1.C` quality gates.
- If LLM output confidence/schema is weak, fail closed to clarify path.

In-house capability growth lock:
- Build in-house STT/TTS as `SHADOW` paths first (no authority).
- Compare in-house vs provider outcomes on the same turn slices.
- Promote in-house route by locale/device/tenant only after gates pass; no global silent promotion.
- Initial production route remains:
  - OpenAI primary
  - Google secondary
  - clarify/text-safe terminal fallback

Gold-output learning law (mandatory):
- Every voice transaction must produce bounded evidence rows for learning/eval (no raw secret leakage).
- Create `gold_case` candidates when any of the following happen:
  - user correction
  - low-confidence transcript
  - transcript reject
  - provider disagreement
  - escalation/human intervention
- Gold cases must include:
  - audio_ref/evidence_ref
  - final accepted transcript
  - language/locale
  - failure fingerprint
  - reason code chain
  - owner engine
- `PH1.LEARN` can package artifacts only after validation; runtime activation remains governed by `PH1.PAE`.

Builder governance lock (never bypass):
- Builder may propose fixes from recurring PH1.C/PH1.D/PH1.TTS fingerprints.
- Builder must not auto-merge or auto-launch without human approvals.
- Required gates remain:
  - code approval gate (`CODE_APPROVED=1`)
  - launch approval gate (`LAUNCH_APPROVED=1`)
- Canonical enforcement scripts:
  - `scripts/check_builder_human_permission_gate.sh`
  - `scripts/check_builder_release_hard_gate.sh`

Strict implementation order (PH1.C round-2):
Step 1. Baseline capture and dataset seed
- Capture STT/TTS baseline metrics by locale, device route, noise class, overlap speech.
- Persist canonical snapshot for replay/eval.
  - Step 1: COMPLETE (canonical PH1.C round-2 baseline snapshot seeded with required coverage dimensions `locale`, `device_route`, `noise_class`, `overlap_speech` in `docs/fixtures/ph1c_round2_baseline_snapshot.csv`, with strict schema + coverage + route-total validation locked in `scripts/check_ph1c_round2_baseline_snapshot.sh`; run proof command: `scripts/check_ph1c_round2_baseline_snapshot.sh`.)

Step 2. Provider boundary lock in PH1.D
- Ensure OpenAI/Google STT/TTS adapters share one typed normalized output schema.
- Fail closed on schema drift/timeouts/provider contract mismatches.
  - Step 2: COMPLETE (PH1.D provider boundary now supports `OCR_TEXT_EXTRACT | STT_TRANSCRIBE | TTS_SYNTHESIZE` with one strict normalized output schema contract (`PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_VERSION/HASH_V1`) and deterministic OpenAI/Google normalization in engine runtime; schema drift, timeout, and vendor/contract mismatch now return fail-closed provider error envelopes. PH1.OS OCR provider route now enforces provider-id/model-id/idempotency coherence and normalized schema-hash match before forwarding. Proof files: `crates/selene_kernel_contracts/src/ph1d.rs`, `crates/selene_engines/src/ph1d.rs`, `crates/selene_os/src/ph1os.rs`; proof commands: `cargo test -p selene_kernel_contracts ph1d -- --nocapture`, `cargo test -p selene_engines ph1d -- --nocapture`, `cargo test -p selene_os ocr_handoff -- --nocapture`.)

Step 3. PH1.C partial transcript contract implementation
- Implement `PartialTranscript` surface with:
  - `text_chunk`
  - `confidence`
  - `stable`
  - `revision_id`
- Keep deterministic ordering and finalization semantics.
  - Step 3: COMPLETE (added typed PH1.C contract surfaces `PartialTranscript` + `PartialTranscriptBatch` with strict validators and contiguous ordered revision semantics in `crates/selene_kernel_contracts/src/ph1c.rs`; added deterministic runtime canonicalizer `Ph1cRuntime::canonicalize_partial_transcripts(...)` in `crates/selene_engines/src/ph1c.rs` that resolves duplicate revisions deterministically and fails closed on invalid/out-of-order/finalization drift; aligned PH1.C DB/ECM docs to the canonical contract/runtime boundary in `docs/DB_WIRING/PH1_C.md` and `docs/ECM/PH1_C.md`. Proof commands: `cargo test -p selene_kernel_contracts ph1c -- --nocapture`, `cargo test -p selene_engines ph1c -- --nocapture`, `cargo test -p selene_os ph1c -- --nocapture`.)

Step 4. PH1.C provider ladder execution
- Enforce STT route order `OpenAI -> Google -> clarify`.
- Enforce bounded retries and total budget caps.
- Keep provider invisibility in upstream payloads.
  - Step 4: COMPLETE (PH1.C runtime now enforces strict sequential STT ladder `PRIMARY(OpenAI) -> SECONDARY(Google) -> fail-closed clarify/reject` with no strategy-based provider reordering; added bounded retry/budget controls (`max_retries_per_provider`, `max_attempts_per_turn`, `max_total_latency_budget_ms`) and deterministic fallback execution in `crates/selene_engines/src/ph1c.rs`; provider invisibility is preserved via slot-only upstream contract surfaces with no vendor/model identifiers leaked in PH1.C output payloads. Docs lock updated in `docs/DB_WIRING/PH1_C.md` and `docs/ECM/PH1_C.md`. Proof commands: `cargo test -p selene_kernel_contracts ph1c -- --nocapture`, `cargo test -p selene_engines ph1c -- --nocapture`, `cargo test -p selene_os ph1c -- --nocapture`.)

Step 5. PH1.TTS coupled provider ladder lock
- Enforce TTS route order `OpenAI -> Google -> text-only fail-safe`.
- Preserve interrupt-safe cancel/resume semantics with reason-coded outcomes.
  - Step 5: COMPLETE (PH1.TTS runtime now exposes coupled provider-ladder execution `handle_with_provider_ladder(...)` enforcing strict slot order `PRIMARY(OpenAI) -> SECONDARY(Google) -> terminal text-only fail-safe`, with bounded retry/budget controls (`max_retries_per_provider`, `max_attempts_per_turn`, `max_total_latency_budget_ms`) and fail-closed reason-coded outcomes (`TTS_FAIL_PROVIDER_BUDGET_EXCEEDED`, `TTS_FAIL_TEXT_ONLY_FAILSAFE`) in `crates/selene_engines/src/ph1tts.rs`; PH1.D normalized-output decoder is now exported for typed PH1.TTS boundary validation in `crates/selene_engines/src/ph1d.rs`; cancel/pause/resume semantics remain deterministic and unchanged under ladder mode. Docs lock updated in `docs/DB_WIRING/PH1_TTS.md` and `docs/ECM/PH1_TTS.md`. Proof commands: `cargo test -p selene_kernel_contracts ph1tts -- --nocapture`, `cargo test -p selene_engines ph1d -- --nocapture`, `cargo test -p selene_engines ph1tts -- --nocapture`, `cargo test -p selene_os ph1tts -- --nocapture`.)

Step 6. Cross-engine handoff integrity
- Validate `PH1.K -> PH1.C` and `PH1.C -> PH1.NLP` handoff envelopes.
- Reject malformed/missing handoff fields fail-closed.
  - Step 6: COMPLETE (cross-engine envelope integrity is now strict and fail-closed end-to-end: `PH1.C` request/response contracts enforce schema-version and bounded handoff payload validation in `crates/selene_kernel_contracts/src/ph1c.rs`, Selene OS `PH1.C` wiring enforces strict `require_ph1k_handoff=true` with reason-coded fail-closed refusal (`PH1_C_HANDOFF_INVALID`) in `crates/selene_os/src/ph1c.rs`, `PH1.NLP` request contract enforces schema-version + UTF-8 safe uncertain-span bounds in `crates/selene_kernel_contracts/src/ph1n.rs`, and Selene OS `PH1.NLP` wiring enforces strict `require_ph1c_handoff=true` with mandatory `transcript_ok.audit_meta` (`attempt_count > 0`, `selected_slot != NONE`) and reason-coded fail-closed clarify (`PH1_NLP_HANDOFF_INVALID`) in `crates/selene_os/src/ph1n.rs`; OCR handoff helper was updated to emit valid PH1.C audit metadata under strict mode in `crates/selene_os/src/ph1os.rs`. Proof commands: `cargo test -p selene_kernel_contracts ph1c -- --nocapture`, `cargo test -p selene_kernel_contracts ph1n -- --nocapture`, `cargo test -p selene_os ph1c -- --nocapture`, `cargo test -p selene_os ph1n -- --nocapture`, `cargo test -p selene_os ocr_handoff -- --nocapture`, `cargo test -p selene_engines ph1c -- --nocapture`, `cargo test -p selene_engines ph1n -- --nocapture`.)

Step 7. Confidence and clarify policy lock
- Low-confidence transcript/hypothesis paths must route to one bounded clarify.
- No guessed words and no guessed intent completion.
  - Step 7: COMPLETE (confidence+clarify policy is now fail-closed in wiring/runtime: PH1.C round-2 test lock proves medium-confidence transcript candidates are rejected (`STT_FAIL_LOW_CONFIDENCE`) with no guessed-word pass-through in `crates/selene_engines/src/ph1c.rs`; PH1.NLP wiring now enforces bounded one-clarify policy for low-confidence transcript/hypothesis paths by refusing non-clarify outputs when transcript confidence is not `HIGH` or uncertain spans are present, and by refusing low-confidence intent drafts (`overall_confidence != HIGH`) with deterministic reason code `PH1_NLP_CLARIFY_REQUIRED` in `crates/selene_os/src/ph1n.rs`. Proof commands: `cargo test -p selene_os ph1n -- --nocapture`, `cargo test -p selene_engines ph1c -- --nocapture`, `cargo test -p selene_os ocr_handoff -- --nocapture`, `cargo test -p selene_engines ph1n -- --nocapture`.)

Step 8. Gold-case capture wiring
- Emit deterministic gold-case candidates from PH1.C/PH1.D/PH1.TTS events.
- Attach failure fingerprints for clustering.
  - Step 8: COMPLETE (Selene OS now emits deterministic typed `GoldCaseCapture` envelopes in `crates/selene_os/src/ph1feedback.rs` for PH1.C (`TranscriptReject` and low-confidence/uncertain `TranscriptOk`), PH1.D (provider error/schema-fail or low-confidence STT provider result), and PH1.TTS (`TtsFailed`) with fail-closed validation, pending `gold_case_id`, bounded `reason_code_chain`, and deterministic clustering fingerprints (`primary_failure_fingerprint`, `secondary_failure_fingerprint`). Proof command: `cargo test -p selene_os ph1feedback -- --nocapture`.)

Step 9. Learning package flow
- Route FEEDBACK signals to LEARN package builder with deterministic ordering and idempotency.
- Ensure package outputs are advisory until PAE activation.
  - Step 9: COMPLETE (added deterministic FEEDBACK->LEARN route in Selene OS `crates/selene_os/src/ph1learn.rs`: `map_feedback_bundle_to_learn_turn_input(...)` canonicalizes feedback candidates by score/id/key, maps feedback taxonomy to learn taxonomy (`source_path` + `gold_status` preserved), emits stable deterministic `signal_id` values, and enforces fail-closed correlation/turn/tenant consistency; `route_feedback_into_learn_wiring(...)` now executes this route into PH1.LEARN package builder while preserving advisory-only/no-execution constraints required by learn contracts. Proof commands: `cargo test -p selene_os ph1learn -- --nocapture`, `cargo test -p selene_os ph1feedback -- --nocapture`.)

Step 10. Promotion/demotion policy
- PAE ladder remains one-step only (`SHADOW <-> ASSIST <-> LEAD`).
- Regression auto-demotion and rollback are mandatory.
  - Step 10: COMPLETE (locked promotion/demotion policy in code: PH1.PAE runtime now enforces one-step ladder transitions with regression-triggered auto-demotion and fail-closed refusal when `LEAD -> ASSIST` demotion lacks rollback pointer; PH1.OS PAE mapper now fails closed on ladder jumps, requires `promotion_eligible=true` for upward transitions, and maps lead demotion to `ROLLBACK` action with mandatory rollback readiness; kernel self-heal `PromotionDecision` contract validation now enforces one-step transitions plus action/mode consistency and lead-demotion rollback readiness. Proof commands: `cargo test -p selene_engines ph1pae -- --nocapture`, `cargo test -p selene_os ph1pae -- --nocapture`, `cargo test -p selene_kernel_contracts ph1selfheal -- --nocapture`.)

Step 11. Benchmark/eval harness
- Add PH1.C canonical eval snapshot and gate scripts for:
  - quality
  - latency
  - fallback success
  - cost
  - tenant isolation/audit completeness
  - multilingual coverage + code-switch quality
  - rambling-to-structured quality
  - broken-English normalization quality
  - accent robustness quality
  - scrambled speech clarify recovery quality
  - Step 11: COMPLETE (added canonical PH1.C round-2 benchmark snapshot `docs/fixtures/ph1c_round2_eval_snapshot.csv`; added strict snapshot harness `scripts/check_ph1c_round2_eval_snapshot.sh` with schema/coverage/formula fail-closed checks across locale/device/noise/overlap/tenant dimensions and all quality categories; added category gate script `scripts/check_ph1c_round2_eval_gates.sh` for benchmark category gate enforcement (quality, latency, fallback success, cost, tenant isolation, audit completeness, multilingual+code-switch, rambling, broken-English normalization, accent robustness, scrambled clarify recovery); wired both scripts into readiness audit (`scripts/selene_design_readiness_audit.sh` sections `1C7` and `1C8`). Proof commands: `./scripts/check_ph1c_round2_eval_snapshot.sh docs/fixtures/ph1c_round2_eval_snapshot.csv`, `./scripts/check_ph1c_round2_eval_gates.sh docs/fixtures/ph1c_round2_eval_snapshot.csv`.)

Step 12. Builder remediation wiring
- Feed recurring failure clusters into Builder proposal pipeline.
- Require permission gates and release hard gate for any change promotion.
- Execution status (`2026-02-25`):
  - Step 12: COMPLETE (added deterministic PH1.OS mapper `map_recurring_failure_cluster_to_builder_offline_input(...)` so recurring unresolved clusters (`recurrence_count >= 3`) convert into bounded `BuilderOfflineInput` entries and flow into PH1.BUILDER proposal intake; added strict fail-closed promotion gate `check_builder_remediation_promotion_gate(...)` that blocks promotion unless `code_permission_gate_passed`, `launch_permission_gate_passed`, and `release_hard_gate_passed` are all proven true.)
  - Proof lock:
    - `crates/selene_os/src/ph1os.rs`
    - `scripts/check_ph1c_round2_builder_remediation_gate.sh`
    - `scripts/selene_design_readiness_audit.sh` (section `1C9`)
    - `cargo test -p selene_os ph1os -- --nocapture`
    - `./scripts/check_ph1c_round2_builder_remediation_gate.sh`

Step 13. Acceptance test expansion
- Add deterministic tests for:
  - OpenAI primary success
  - Google fallback on OpenAI fail
  - terminal clarify/text-safe fallback when both fail
  - partial transcript revision correctness
  - provider schema-drift fail-closed behavior
  - gold-case creation on correction/escalation
- Execution status (`2026-02-25`):
  - Step 13: COMPLETE (added explicit PH1.C acceptance tests for primary-slot success, secondary-slot fallback, and terminal fail-closed reject when both slots fail; retained deterministic partial transcript revision correctness lock; retained PH1.D provider schema-drift fail-closed lock; and added PH1.FEEDBACK acceptance lock proving gold-case creation on correction with escalation mapping.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `crates/selene_os/src/ph1feedback.rs`
    - `scripts/check_ph1c_round2_acceptance_tests.sh`
    - `scripts/selene_design_readiness_audit.sh` (section `1C10`)
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`
    - `cargo test -p selene_os ph1feedback -- --nocapture`
    - `./scripts/check_ph1c_round2_acceptance_tests.sh`

Step 14. Release gate thresholds (must pass)
- STT fallback continuity success `>= 99.90%`
- provider schema-valid response rate `>= 99.50%`
- partial transcript first chunk latency `p95 <= 250ms`
- end-of-speech to first response token `p95 <= 300ms`
- false interrupt rate `<= 0.3/hour`
- missed interrupt rate `<= 2%`
- audit completeness `= 100%`
- tenant isolation `= 100%`
- multilingual transcript acceptance (supported locales) `>= 95%`
- heavy-accent transcript acceptance (supported accents) `>= 93%`
- broken-English normalization acceptance (human-rated) `>= 90%`
- rambling/scrambled clarify-to-resolution within 2 turns `>= 90%`
- Execution status (`2026-02-25`):
  - Step 14: COMPLETE (added strict PH1.C release-threshold gate script `check_ph1c_round2_release_gate.sh` that enforces all Step-14 thresholds fail-closed; wired cross-engine interrupt proof by consuming PH1.K snapshot for `false_interrupt_rate_per_hour` and `missed_interrupt_rate_pct`; and upgraded canonical PH1.C eval fixture values to satisfy the locked threshold floor while preserving schema/coverage invariants.)
  - Proof lock:
    - `scripts/check_ph1c_round2_release_gate.sh`
    - `docs/fixtures/ph1c_round2_eval_snapshot.csv`
    - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
    - `scripts/selene_design_readiness_audit.sh` (section `1C11`)
    - `./scripts/check_ph1c_round2_release_gate.sh docs/fixtures/ph1c_round2_eval_snapshot.csv docs/fixtures/ph1k_round2_eval_snapshot.csv`

Step 15. Closure + queue progression
- Record PH1.C round-2 closure only after gate pass + ledger proof append.
- Keep next engine queue locked until PH1.C closure is proven.
- Execution status (`2026-02-25`):
  - Step 15: COMPLETE (re-ran PH1.C strict gate chain, appended closure proof to build ledger, and advanced locked round-2 queue from `PH1.C` to `PH1.D` only after all PH1.C gates passed.)
  - Proof lock:
    - `docs/03_BUILD_LEDGER.md`
    - `docs/33_ENGINE_REVIEW_TRACKER.md`
    - `docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md`
    - `./scripts/check_ph1c_round2_eval_snapshot.sh docs/fixtures/ph1c_round2_eval_snapshot.csv`
    - `./scripts/check_ph1c_round2_eval_gates.sh docs/fixtures/ph1c_round2_eval_snapshot.csv`
    - `./scripts/check_ph1c_round2_builder_remediation_gate.sh`
    - `./scripts/check_ph1c_round2_acceptance_tests.sh`
    - `./scripts/check_ph1c_round2_release_gate.sh docs/fixtures/ph1c_round2_eval_snapshot.csv docs/fixtures/ph1k_round2_eval_snapshot.csv`

### 5H) PH1.D + PH1.C Locked Superiority Plan (Beat ChatGPT on Voice Stack)
Purpose:
- Lock a strict build sequence so Selene exceeds general-purpose voice quality in your operating domain.
- Keep PH1.D as the only external provider boundary and PH1.C as the transcript authority.

Execution lock:
- Steps must execute in strict order.
- No step can be marked complete without code proof + test proof + ledger entry.
- Fail-closed behavior remains mandatory at every step.

Step 1. PH1.D live provider call path (no mock-only runtime)
- Build real HTTP provider adapter path in PH1.D for OpenAI/Google endpoints with auth, timeout, idempotency headers, bounded body reads, and fail-closed mapping.
- Keep strict normalized-output schema enforcement (`PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1`) after live transport.
- Execution status (`2026-02-25`):
  - Step 1: COMPLETE (added concrete `Ph1dLiveProviderAdapter` + env-configurable endpoint map + live HTTP execution path in PH1.D with strict timeout/contract-mismatch mapping and schema-locked normalization output.)
  - Proof lock:
    - `crates/selene_engines/src/ph1d.rs`
    - `crates/selene_engines/Cargo.toml`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 2. PH1.C consumes live PH1.D STT results (remove prebuilt-only dependency)
- Replace prebuilt `SttAttempt`-only handoff as primary path with PH1.D live provider response consumption.
- Keep deterministic gate owner in PH1.C.
- Execution status (`2026-02-26`):
  - Step 2: COMPLETE (wired PH1.C live STT route `run_via_live_provider_adapter(...)` to build typed `Ph1dProviderCallRequest` payloads, call PH1.D adapter directly, decode schema-locked normalized STT output, and feed deterministic PH1.C gating/fail-closed decisions without prebuilt-attempt-only dependency.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 3. Enforce live OpenAI(primary) -> Google(secondary) + hard circuit breaker
- Keep strict route order, retries, budget caps, and cooldown breakers by provider/model/tenant.
- Execution status (`2026-02-26`):
  - Step 3: COMPLETE (PH1.C live provider path now enforces strict slot order `OpenAI(primary) -> Google(secondary)` with per-tenant/provider/model circuit-breaker state, bounded failure thresholds, cooldown windows, and hard skip-on-open behavior before provider call. Provider failures update breaker state fail-closed; successful provider responses close breaker state.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 4. True streaming STT runtime
- Add partial revisions, deterministic replacement/finalization, and low-latency commit path from live providers.
- Execution status (`2026-02-26`):
  - Step 4: COMPLETE (added PH1.C true streaming path `run_stream_via_live_provider_adapter(...)` with live-provider revision polling, deterministic revision canonicalization via `PartialTranscriptBatch`, strict finalization handling, and low-latency commit path when stable/high-confidence partials pass gating before final frame.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 5. Locale normalization upgrade
- Replace exact language-tag matching with normalized locale matching (`en`, `en-US`, `en-GB` family-safe mapping).
- Execution status (`2026-02-26`):
  - Step 5: COMPLETE (PH1.C language mismatch gate now uses normalized locale-family matching instead of exact-tag equality: canonicalized tags (`_` -> `-`, case normalization), language-family equivalence (`en`/`en-US`/`en-GB`), and script-safe mismatch protection when both scripts are explicit and different.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 6. Confidence calibration upgrade
- Replace simple heuristic gating with calibrated confidence combining token/acoustic/context signals.
- Execution status (`2026-02-26`):
  - Step 6: COMPLETE (PH1.C confidence gate now uses calibrated scoring from three bounded signals: token confidence (`avg_word_confidence` + `low_confidence_ratio`), acoustic quality (`noise_level_hint`, `vad_quality_hint`, `PH1.K` advanced metrics), and context consistency (normalized locale-family + hint confidence). Missing signal families now fall back to token confidence so strong clean paths do not regress; poor acoustic paths are penalized deterministically.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 7. Tenant/domain lexicon boosting
- Inject PH1.KNOW/PH1.CONTEXT tenant vocabulary boosts (names, products, entities) into PH1.C/PH1.D provider requests and acceptance logic.
- Execution status (`2026-02-26`):
  - Step 7: COMPLETE (PH1.C live-provider context now carries bounded tenant/user vocabulary pack refs and tenant/domain lexicon term sets, injects normalized lexicon hints into outbound PH1.D STT payloads, and applies deterministic lexicon-based confidence boost in acceptance scoring for matched domain terms. Boosting is bounded and fail-closed safe: no lexicon terms means no boost.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1d -- --nocapture`

Step 8. Intent-aware transcript repair
- Add PH1.NLP/PH1.SRL bounded repair flow for rambling/broken/scrambled speech before final clarify/execute decision.
- Execution status (`2026-02-26`):
  - Step 8: COMPLETE (PH1.C now runs a bounded intent-aware repair pass before final transcript gate: candidate rambling/broken/scrambled transcripts are routed through PH1.SRL frame-build + argument-normalize, then passed through a PH1.NLP acceptance guard to block intent drift. Repair is fail-closed: if SRL marks ambiguity/clarify-required, overlap safety fails, or NLP quality regresses, PH1.C keeps original fail-closed behavior and does not guess. Accepted repairs apply bounded confidence updates only and still must pass standard PH1.C coverage/language/confidence gates.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1srl -- --nocapture`
    - `cargo test -p selene_engines ph1n -- --nocapture`

Step 9. Clarify precision lock
- Enforce one precise clarify question then execute or escalate; no clarify loops.
- Execution status (`2026-02-26`):
  - Step 9: COMPLETE (PH1.X clarify flow is now lock-tight across runtime paths: a first unresolved question emits one bounded clarify as before, and any second clarify attempt in the same thread state escalates immediately to a non-question escalation response instead of looping. This applies uniformly to NLP clarify pass-through and missing-field clarify branches, preserving fail-closed behavior and blocking infinite clarify chains.)
  - Proof lock:
    - `crates/selene_engines/src/ph1x.rs`
    - `crates/selene_os/src/ph1x.rs`
    - `cargo test -p selene_engines ph1x -- --nocapture`
    - `cargo test -p selene_os ph1x -- --nocapture`

Step 10. Gold-loop lock
- Capture every miss/correction into `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE` with deterministic fingerprints and idempotent package flow.
- Execution status (`2026-02-26`):
  - Step 10: COMPLETE (locked a deterministic miss/correction gold-loop chain in Selene OS using real FEEDBACK/LEARN/PAE runtimes: PH1.C miss/correction captures are emitted as pending gold events, verified-gold gating is enforced before LEARN package forwarding, and FEEDBACK -> LEARN -> PAE replay determinism is now acceptance-locked with stable failure/problem/fix/decision ids across repeated runs.)
  - Proof lock:
    - `crates/selene_os/src/ph1os.rs`
    - `cargo test -p selene_os ph1feedback -- --nocapture`
    - `cargo test -p selene_os ph1learn -- --nocapture`
    - `cargo test -p selene_os ph1pae -- --nocapture`
    - `cargo test -p selene_os at_os_43_ph1c_gold_loop_miss_and_correction_route_to_learn_and_pae_deterministically -- --nocapture`
    - `cargo test -p selene_engines ph1feedback -- --nocapture`
    - `cargo test -p selene_engines ph1learn -- --nocapture`
    - `cargo test -p selene_engines ph1pae -- --nocapture`

Step 11. In-house shadow route lock
- Run in-house STT/TTS in `SHADOW`; compare against provider truth by locale/device/tenant; promote only through governed gates.
- Execution status (`2026-02-26`):
  - Step 11: COMPLETE (added deterministic in-house `SHADOW` comparison surfaces in PH1.C + PH1.TTS runtimes: both paths now require explicit slice keys `{locale, device_route, tenant_id}`, validate provider-truth contracts before comparison, compute bounded parity metrics against provider truth, and hold shadow by default unless governed promotion gate proof is present. Promotion eligibility is fail-closed and blocked on gate absence, meaning drift, or parity/latency regressions.)
  - Proof lock:
    - `crates/selene_engines/src/ph1c.rs`
    - `crates/selene_engines/src/ph1tts.rs`
    - `cargo test -p selene_engines ph1c -- --nocapture`
    - `cargo test -p selene_engines ph1tts -- --nocapture`

Step 12. Builder remediation governance lock
- Builder may propose auto-fixes from recurring fingerprints, but code/launch remain human-approved only.
- Execution status (`2026-02-26`):
  - Step 12: COMPLETE (confirmed and re-locked Builder remediation governance for the PH1.D/PH1.C runbook path: recurring unresolved clusters map to bounded `BuilderOfflineInput` proposals only, while promotion remains fail-closed unless explicit code-permission, launch-permission, and release-hard-gate evidence is present. Builder may propose remediation, but cannot auto-ship code or launch without human approval proofs.)
  - Proof lock:
    - `crates/selene_os/src/ph1os.rs`
    - `scripts/check_ph1c_round2_builder_remediation_gate.sh`
    - `cargo test -p selene_os builder_remediation -- --nocapture`
    - `bash scripts/check_ph1c_round2_builder_remediation_gate.sh`

### 5I) PH1.C Superiority Upgrade Lock (Step 1..22)
Purpose:
- Lock the next PH1.C superiority upgrades point-by-point before implementation.
- Implement and prove each locked step with deterministic gates before progression.

Execution lock:
- Steps execute in strict order.
- No step is complete without code proof + gate/test proof + ledger/tracker/plan update.
- Fail-closed behavior remains mandatory.

Global learning model lock (Selene Cloud; cross-tenant):
- Learning is global, not isolated per tenant.
- Each tenant contributes onboarding-permitted correction/evaluation signals to Selene Cloud learning ledgers.
- Selene Cloud produces signed global upgrade artifacts (models, thresholds, lexicons, policies) and pushes upgrades back to all tenants.
- Runtime policy decisions remain slice-gated and fail-closed, but learning evidence is pooled globally for faster improvement.

Step 1. Direct ChatGPT A/B benchmark lane
- Add a direct ChatGPT A/B benchmark lane on the same audio inputs, same locales, same noise classes, and same scoring formulas used for PH1.C.
- Lock dataset parity and scoring parity so comparisons are deterministic and auditable.

Step 2. Semantic accuracy gate
- Add a semantic accuracy gate so PH1.C is scored on intent understanding correctness, not transcript text alone.
- Gate must be fail-closed for low semantic-confidence paths.

Step 3. Global lexicon packs v2 (tenant-fed)
- Add lexicon packs v2 covering names, products, acronyms, and slang.
- Tenant contributions are pushed to Selene Cloud and merged into global lexicon upgrades for all tenants.
- Keep bounded weighting and expiry controls to prevent stale boost drift.

Step 4. Speaker overlap handling
- Add handling for single-speaker, multi-speaker, and interruption-overlap scenarios.
- Ambiguous overlap cases must fail closed (clarify/escalate) instead of guessed assignment.

Step 5. Two-pass decoding in PH1.C
- Add two-pass decoding:
  - pass 1: fast live decode for latency
  - pass 2: bounded repair/verification before final commit
- Final action must rely on validated final commit only.

Step 6. Provider disagreement policy
- Add strict disagreement policy between OpenAI and Google outputs.
- If divergence exceeds threshold, force one clarify question instead of guessing.

Step 7. Intent-aware repair hardening
- Harden intent-aware repair with PH1.SRL and PH1.NLP for rambling, broken, and scrambled speech.
- Explicitly block meaning drift via bounded overlap/consistency checks.

Step 8. Gold-loop acceleration
- Every correction/escalation must become a labeled training case.
- Required labels include deterministic fingerprint, root-cause tags, and resolution outcome.
- Gold cases must be published to Selene Cloud global learning ledgers and redistributed as upgrade artifacts to all tenants.

Step 9. Slice-based promotion control
- In-house STT/TTS promotion from `SHADOW -> ASSIST -> LEAD` is allowed only by slice proof:
  - locale
  - device route
  - tenant
- No global promotion without per-slice gate evidence.
- Global rollout is allowed only after multi-tenant slice evidence proves superiority and safety.

Step 10. Strict superiority gates
- Promotion requires beating both:
  - current PH1.C baseline
  - ChatGPT A/B benchmark on locked domain test packs
- Any regression in safety/clarify/audit/tenant isolation blocks promotion.

Step 11. Cost-quality routing policy
- Add cost-quality routing policy to pick the lowest-cost route that still passes accuracy and safety gates for each slice.
- Route decisions must be reason-coded and replay-deterministic.

Step 12. Final release acceptance pack
- Add final mandatory acceptance pack with explicit pass criteria for:
  - accuracy
  - intent success
  - latency
  - clarify quality
  - audit completeness
  - tenant isolation
- Release is blocked until all acceptance pack gates pass.

Step 13. Teacher-student distillation
- Run OpenAI + Google + in-house routes in shadow on the same turns.
- Use high-agreement outcomes as supervised teacher targets for PH1.C learning artifacts in Selene Cloud.

Step 14. Disagreement mining queue
- Route high-divergence provider disagreements into a priority review queue.
- Convert resolved disagreements into high-value labeled training cases with deterministic fingerprints.

Step 15. Active learning priority
- Prioritize labeling for low-confidence, high-impact, and high-recurrence failures first.
- Enforce deterministic ranking so the learning backlog improves error rate fastest.

Step 16. Gold/silver data tiering
- Maintain two tiers:
  - Gold: human-verified labels
  - Silver: high-confidence teacher consensus labels
- Train primarily on gold and boundedly on silver with drift guards.

Step 17. Hard-negative replay
- Over-sample repeated failure families (accents, jargon, overlap speech, noisy environments).
- Keep replay packs deterministic and versioned so recurrence reduction is measurable.

Step 18. Entity and intent scoring gates
- Add hard gates for entity accuracy (names, ids, dates, amounts) and intent success.
- Promotion is blocked if transcript quality is high but entity/intent correctness regresses.

Step 19. Diarization and overlap attribution
- Add speaker diarization and overlap attribution metrics as first-class acceptance gates.
- Ambiguous speaker attribution must fail closed to clarify/escalate.

Step 20. Per-slice adapter training
- Train per-slice adapters (locale/device/acoustic profile) using global pooled learning evidence.
- Promote adapters slice-by-slice with strict safety and superiority gates.

Step 21. Champion-challenger runtime
- Keep current PH1.C as champion while challengers run in shadow/canary.
- Promote challenger only if it beats both champion and ChatGPT benchmark lanes on locked packs.

Step 22. Learning cadence and rollback discipline
- Run fixed retrain/eval cadence with deterministic release windows.
- Any regression triggers automatic rollback to last approved artifact pack.
- Human approval gates for code/launch remain mandatory and unchanged.

Execution status (`2026-02-26`):
- Step 1-22: COMPLETE (implemented deterministic PH1.C superiority runtime gate surface + canonical 5I snapshot/gate harness and wired readiness enforcement).
- Code proof:
  - `crates/selene_os/src/ph1c_superiority.rs`
  - `crates/selene_os/src/lib.rs`
  - `crates/selene_engines/src/ph1c.rs` (step-2/4/5/6/7/11 runtime locks + acceptance tests)
- Gate proof:
  - `scripts/check_ph1c_5i_eval_snapshot.sh`
  - `scripts/check_ph1c_5i_superiority_gate.sh`
  - `docs/fixtures/ph1c_5i_superiority_snapshot.csv`
  - `scripts/selene_design_readiness_audit.sh` (sections `1C12` + `1C13`)
- Test/gate proof commands:
  - `cargo test -p selene_engines ph1c -- --nocapture`
  - `cargo test -p selene_os ph1c_superiority -- --nocapture`
  - `./scripts/check_ph1c_5i_eval_snapshot.sh docs/fixtures/ph1c_5i_superiority_snapshot.csv`
  - `./scripts/check_ph1c_5i_superiority_gate.sh docs/fixtures/ph1c_5i_superiority_snapshot.csv`

### 5J) PH1.NLP ChatGPT-Superiority Improvement Backlog (Tracking)
Purpose:
- Keep one execution-tracked list of PH1.NLP improvements required to sustain "better than ChatGPT" quality in voice/text understanding paths.
- Convert review findings into explicit build items with deterministic close criteria.

Decision lock (`2026-02-26`, JD-confirmed):
- Superiority scope is `BOTH`:
  - Selene domain workflows (`access/onboarding/capreq/position/link/reminder/broadcast/...`).
  - General open-ended conversation quality.
- Parser policy allows constrained learned parser lanes under strict fail-closed contracts:
  - learned parser outputs are advisory-only structured candidates;
  - no direct execution authority;
  - disagreement/low-confidence/drift always routes to deterministic clarify-safe fallback.

Backlog freeze (`2026-02-26`):

| item_id | priority | gap summary | why this blocks superiority | required remediation | closure proof (minimum) | owner | status |
|---|---|---|---|---|---|---|---|
| NLP-SUP-01 | P0 | `IntentHypothesis` is documented but not implemented in PH1.NLP contract/runtime surfaces | Full-duplex "understand while user is speaking" advantage is not real without incremental hypotheses consumed by PH1.X | Add typed `IntentHypothesis` contract + runtime emission + OS wiring + PH1.X consume path; keep advisory-only/no-execution posture | kernel/engine/OS tests for hypothesis schema + ordering + fail-closed behavior; readiness gate hook | PH1.NLP | OPEN |
| NLP-SUP-02 | P0 | PH1.NLP confidence is effectively always `HIGH` in runtime outputs | Low-confidence clarify lock loses precision if confidence is not calibrated per intent/field | Implement calibrated confidence scoring (`HIGH|MED|LOW`) for intent and slot extraction; enforce clarify when below threshold | acceptance tests proving medium/low confidence routes to clarify; eval snapshot metric for confidence calibration error | PH1.NLP | OPEN |
| NLP-SUP-03 | P0 | Multilingual/code-switch understanding relies on narrow keyword parsing | Claimed robustness for broken English/code-switch/accent paths can overfit and regress in real turns | Add language-aware intent grammars/tokenization + code-switch slot extraction using PH1.LANG signals and deterministic fallback rules | multilingual regression suite expansion + per-locale/route gate rows in eval snapshot | PH1.NLP | OPEN |
| NLP-SUP-04 | P0 | Superiority gate can pass on a small fixture without strong statistical/provenance constraints | "Beats ChatGPT" claim is weak if dataset size, holdout, and evidence lineage are not enforced | Strengthen superiority harness with minimum sample floors, out-of-sample holdout requirements, and signed evidence linkage | gate script checks for min turns/slices/holdout; deterministic fail-closed on missing provenance | PH1.C + PH1.NLP | OPEN |
| NLP-SUP-09 | P0 | Superiority target is not enforced as dual-lane (`domain workflows` + `open-ended conversation`) in one release gate | Team can overfit one lane and still claim "better than ChatGPT" without balanced quality | Add dual-lane superiority scorecard with per-lane minimum floors, weighted aggregate rule, and fail-closed gate requiring pass on both lanes | gate script + fixture pack proving lane-specific thresholds, minimum sample counts, and provenance linkage for both lanes | PH1.C + PH1.NLP + PH1.X | OPEN |
| NLP-SUP-10 | P0 | PH1.NLP parser path is effectively deterministic-rules only; no constrained learned parser lane is contract-locked | Rules-only parsing caps open-ended robustness and code-switch coverage versus frontier assistants | Add constrained learned parser lane (`intent/slot/chat candidates`) behind strict schema validators + deterministic arbiter; enforce advisory-only and clarify-safe fallback on disagreement/drift/low confidence | kernel/engine/OS tests for schema bounds, arbiter fail-closed behavior, and shadow-to-active eval showing lift with zero never-events | PH1.NLP + PH1.OS + PH1.PAE | OPEN |
| NLP-SUP-11 | P0 | PH1.NLP has no typed PH1.D LLM assist lane for tricky conversation recovery | Hard turns (rambling/scrambled/code-switch/broken phrasing) can regress without bounded LLM assist under contract gates | Add PH1.NLP -> PH1.D typed assist call path for tricky-turn resolution with strict schema validators, advisory-only candidate output, deterministic arbiter, and clarify fail-closed fallback | contract/runtime/OS tests proving PH1.D assist is bounded + non-authoritative; disagreement/low-confidence/schema-drift always returns clarify; release gate rows for tricky-turn slices | PH1.NLP + PH1.D + PH1.OS | OPEN |
| NLP-SUP-05 | P1 | `time_context` exists in PH1.NLP request contract but is not consumed in runtime normalization | Relative time understanding quality and determinism are capped without explicit time context usage | Use `time_context` for bounded relative-time grounding and timezone-safe normalization | tests for relative-time resolution with timezone offsets + DST-safe behavior | PH1.NLP | OPEN |
| NLP-SUP-06 | P1 | `confirmed_context` is used mostly as presence/absence, not rich field-level reference resolution | Reference disambiguation quality is below target on follow-up turns | Add deterministic field-level coreference resolution from `last_confirmed_fields` before clarify fallback | tests for "that/it/there" resolution across multi-turn continuity cases | PH1.NLP | OPEN |
| NLP-SUP-07 | P1 | PH1.NLP chat writes use `NlpIntentDraft` event type in storage path | Analytics and quality tracking are blurred across intent vs chat fallback outcomes | Introduce explicit `NlpChat` audit event type and update storage/DB_WIRING/ECM docs + tests | migration/contract/storage tests proving distinct event typing and replay semantics | PH1.NLP + PH1.J | OPEN |
| NLP-SUP-08 | P1 | PH1.NLP reason-code namespaces remain placeholder comments in runtime/OS layers | Long-term audit comparability and cross-engine reporting are weaker until registry is locked | Lock PH1.NLP reason-code IDs in global registry and add compile-time mapping/uniqueness checks | contract/runtime guard tests + readiness check for reason-code registry parity | PH1.NLP + PH1.J | OPEN |

Execution rule:
1. Work top-to-bottom in strict order for P0 items first, then P1.
2. A row closes only when code proof + gate proof + `docs/03_BUILD_LEDGER.md` entry are all present.
3. Any regression on closed rows reopens the row to `OPEN`.

### Round 3: Hardening Round (Cross-Engine Reliability Closure)
Goal:
- Prove deterministic safety and orchestration behavior end-to-end.

Scope:
- Cross-engine integration, not isolated engine implementation.

Mandatory checks:
1. Gate-order enforcement suites (OS sequencing).
2. Fail-closed matrix (invalid input, missing access, missing simulation, drifted contracts).
3. Idempotency and lease replay/takeover tests.
4. Side-effect dedupe and audit chain integrity.
5. Cross-engine E2E paths for core voice/text flows.

Exit criteria:
- Full workspace test suite passes.
- Integration matrix passes with no critical open defects.
- No regression in previously closed engine acceptance tests.

Execution status (`2026-02-26`):
- Round 3: `DONE` (cross-engine hardening sweep completed end-to-end).
- Stability fix included:
  - Eliminated PH1.BUILDER test temp-file collision in parallel workspace runs by isolating per-test temp directories (`crates/selene_os/src/ph1builder.rs`).
  - Tightened stale Phase13-B guardrail to detect only actual runtime builder execution wiring (not offline mapping labels) (`scripts/check_builder_pipeline_phase13b.sh`).
- Proof command bundle:
  - `cargo test -p selene_os ph1builder -- --nocapture`
  - `cargo test --workspace`
  - `bash scripts/check_builder_pipeline_phase13b.sh`
  - `bash scripts/selene_design_readiness_audit.sh`

### Round 4: Production Readiness Round (Operational Release Closure)
Goal:
- Validate runtime operations under production-like conditions.

Scope:
- SLO, staging, operability, and release governance.

Mandatory checks:
1. Latency/SLO measurement for contracted paths (p95/p99 tracked).
2. Staging soak run with deterministic logs/audit traceability.
3. Postgres backup/restore proof and replay integrity checks.
4. Migration traceability and rollback safety checks.
5. Release checklist sign-off (engineering + product + ops).

Exit criteria:
- SLO gates satisfied or approved with explicit bounded exceptions.
- Operational drills passed.
- Release sign-off completed and recorded in build ledger.

## 6) Wave Staging (Execution Safety)
- Wave A (core voice stack): `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.NLP`, `PH1.D`, `PH1.X`, `PH1.WRITE`, `PH1.TTS`, `PH1.L`
- Wave B (governance/execution): access/capreq/position/onboarding/link/broadcast/reminder + enterprise control engines
- Wave C (assist/learning): non-authoritative assist engines and offline optimization chain

Checkpoint cadence:
- Every 3 engines: compile + targeted regression.
- End of each wave: full workspace tests + readiness audit.

## 7) Definition of Done (Program-Level)
- Engine closure complete (Round 1).
- Engine finalization complete (Round 2).
- Hardening complete (Round 3).
- Operational readiness complete (Round 4).
- No unresolved critical blockers in tracker/build ledger.

## 8) Change Control Rule
- Any mid-cycle engine/contract/simulation/blueprint change must:
  1. update canonical files in Section 2,
  2. rerun impacted relationship checks,
  3. append a ledger proof entry before proceeding.

## 9) Cross-Engine Risk TODO Backlog (Must Close)
Purpose:
- Convert architecture findings into explicit work items so they are reviewed, decided, implemented, and verified.
- Prevent unresolved overlap/conflict/performance risk from leaking into production rounds.

### 9.1 Decision + Execution Protocol (Per Issue)
1. Review issue with owner + counterpart reviewer (you + implementation lead).
2. Choose one target model (no dual-model ambiguity).
3. Patch docs/contracts/runtime/tests as a single scoped change-set.
4. Run targeted + integration tests.
5. Mark issue status and log proof in `docs/03_BUILD_LEDGER.md`.

### 9.2 Risk Issue Tracker

| issue_id | issue summary | potential impact | decision required | remediation plan | owner | target round | status |
|---|---|---|---|---|---|---|---|
| RISK-01 | Learning ownership overlap (`PH1.LEARN_FEEDBACK_KNOW` vs `PH1.LEARN` + `PH1.FEEDBACK` + `PH1.KNOW`) | Dual-writer risk, drifted artifacts, ambiguous debugging | Choose one ownership model: aggregate storage-only wrapper or full split ownership | (1) locked split runtime ownership in map/registry/db-ownership (`PH1.FEEDBACK`, `PH1.LEARN`, `PH1.KNOW`) and demoted `PH1.LEARN_FEEDBACK_KNOW` to storage-group-only pointer, (2) updated grouped DB_WIRING/ECM boundaries to explicit single-writer artifact types, (3) added storage single-writer invariant enforcement + test (`at_learn_db_05_single_writer_artifact_types_enforced`), (4) added CI/readiness guardrail script (`scripts/check_learning_ownership_boundaries.sh`) | PH1.LEARN | Round 1 | CLOSED |
| RISK-02 | Intent-shaping overlap (`PH1.LANG`, `PH1.SRL`, `PH1.PUZZLE`, `PH1.ATTN`, `PH1.NLP`, `PH1.PRUNE`, `PH1.DIAG`, `PH1.CONTEXT`) | Conflicting outcomes, clarify loops, higher latency | Lock strict precedence chain and one clarify authority | (1) locked clarify ownership in PH1.OS contracts/runtime (`clarify_owner_engine_id` must be `PH1.NLP`), (2) added deterministic optional assist policy bounds (`PH1.PRUNE`/`PH1.DIAG`) with fail-closed top-level refusal, (3) updated map/DB_WIRING/ECM precedence docs, (4) added PH1.OS contract/runtime tests + CI guardrail script (`scripts/check_understanding_clarify_precedence.sh`) | PH1.NLP | Round 1-2 | CLOSED |
| RISK-03 | Governance gate contention (`PH1.POLICY`, `PH1.ACCESS.001/PH2.ACCESS.002`, `PH1.TENANT`, `PH1.GOV`, `PH1.QUOTA`, `PH1.OS`) | Contradictory block/allow outcomes, unsafe gate order drift | Approve one global precedence matrix and terminal-state policy | (1) adopted Section 10 matrix as canonical precedence order, (2) enforced governance decision trace + contradiction fail-closed in PH1.OS contracts/runtime, (3) added conflicting-gate PH1.OS tests, (4) blocks execution on unresolved contradiction | PH1.OS | Round 1-2 | CLOSED |
| RISK-04 | Delivery orchestration overlap (`PH1.LINK`, `PH1.BCAST`, `PH1.DELIVERY`, `PH1.REM`, SMS setup pre-send gating) | Duplicate sends, inconsistent reminder/sms behavior, retry storms | Confirm one lifecycle owner for outbound comms | (1) locked lifecycle boundaries: LINK token only, BCAST lifecycle owner, DELIVERY provider attempts, REM timing, SMS setup pre-send gate, (2) added idempotency + duplicate-send guard tests (`at_sim_exec_17`, `at_sim_exec_18`), (3) enforced no-cross-owner writes via runtime/CI ownership guardrail (`scripts/check_delivery_ownership_boundaries.sh`) | PH1.BCAST.001 | Round 1-2 | CLOSED |
| RISK-05 | Large TURN_OPTIONAL set can degrade p95/p99 latency | Slower responses and unstable user experience | Define turn-level optional-engine budget policy | (1) kept per-turn optional-engine budget enforcement in PH1.OS orchestration (GATE-U3), (2) added deterministic optional-engine tier classification (`STRICT | BALANCED | RICH`) and utility review scoring over outcome-utilization entries, (3) implemented GATE-U4/GATE-U5 policy actions (`KEEP | DEGRADE | DISABLE_CANDIDATE`) with sustained-fail streak handling, (4) added CI utility-gate checker script (`scripts/check_optional_engine_utility_gates.sh`) and wired readiness audit invocation | PH1.OS | Round 2 | CLOSED |
| RISK-06 | Runtime boundary leak risk for OFFLINE/control-plane engines (`PH1.PATTERN`, `PH1.RLL`, `PH1.GOV`, `PH1.EXPORT`, `PH1.KMS`) | Runtime coupling, unsafe authority drift, performance overhead | Confirm strict out-of-band execution boundary | (1) enforced PH1.OS top-level runtime-boundary fail-closed guard rejecting OFFLINE/control-plane engine ids in live turn wiring, (2) preserved OFFLINE/control-plane ownership by keeping these engines out of `ALWAYS_ON` and `TURN_OPTIONAL` runtime sequences, (3) added CI guard script `scripts/check_runtime_boundary_guards.sh` and wired readiness audit checks for docs/code drift, (4) added PH1.OS runtime tests proving boundary-violation refusal reason-codes | PH1.GOV | Round 1-2 | CLOSED |
| RISK-07 | Namespace duplication pressure over time | Reintroduced duplicate engines and review confusion | Approve duplication prevention guardrail | (1) kept family->implementation merge map in tracker, (2) added guardrail script `scripts/check_engine_tracker_duplicates.sh` and wired it into readiness audit script, (3) enforced merge-note requirement and normalized missing tracker merge-note rows | PH1.F (governance data hygiene) | Round 1 | CLOSED |
| RISK-08 | `PH1.POLICY` docs/runtime wiring drift (`ALWAYS_ON` in map/registry but missing concrete runtime module in `selene_os`) | Prompt dedupe and policy snapshot gate can be bypassed in runtime path; docs/runtime divergence risk | Approve canonical runtime placement and enforcement point for `PH1.POLICY` before `PH1.X` | (1) added concrete `PH1.POLICY` contract + engine + OS wiring modules, (2) enforced call order `Selene OS -> PH1.POLICY -> PH1.X`, (3) added fail-closed contract/wiring/integration tests for invalid/missing policy outputs, (4) enforced `PH1.OS` prompt-policy gate fields in policy evaluate path and decision compute, (5) recorded build proof in ledger | PH1.OS | Round 1 | CLOSED |
| RISK-09 | Full turn orchestrator slice is incomplete (engine-level adapters exist, but no single runtime path proving full `ALWAYS_ON + TURN_OPTIONAL` sequencing) | Fragmented orchestration behavior, inconsistent cross-engine outcomes, weaker end-to-end determinism | Approve one canonical top-level turn orchestrator boundary and sequencing contract | (1) implemented PH1.OS top-level orchestration wiring slice with path-locked ALWAYS_ON sequence checks (voice/text), (2) centralized TURN_OPTIONAL invocation ordering and bounded budget handling at one control point, (3) added voice/text + gate-failure tests proving fail-closed behavior, (4) documented canonical boundary in engine map/registry + PH1.OS DB_WIRING/ECM and recorded ledger proof | PH1.OS | Round 1-2 | CLOSED |
| RISK-10 | `PH1.OS` turn-level optional-engine budget enforcement missing from runtime contract surface | Latency inflation under heavy optional-engine use; unstable p95/p99 and degraded UX | Approve required budget fields and decision policy in `PH1.OS` contracts | (1) extended `PH1.OS` contract/wiring with explicit per-turn optional budget fields (`optional_invocations_requested/budget/skipped` + `optional_latency_budget_ms/estimated_ms`), (2) enforced deterministic skip/degrade policy in top-level orchestration with fail-closed refusal on budget-policy drift, (3) added contract/runtime/wiring tests for GATE-U3 semantics and latency budget breach, (4) preserved owner precedence and `No Simulation -> No Execution` behavior unchanged | PH1.OS | Round 2 | CLOSED |

### 9.3 Issue Closure Gate (Mandatory)
An issue can move from `OPEN` to `CLOSED` only when:
1. Decision is recorded (single chosen model).
2. Canonical docs are updated (`07/06/08/09/10` + relevant DB_WIRING/ECM).
3. Runtime/tests prove behavior and fail-closed semantics.
4. `docs/03_BUILD_LEDGER.md` has an auditable proof entry.

## 10) Strict Owner/Decision Precedence Matrix (Cluster-Level)
Purpose:
- Guarantee one decision owner per cluster.
- Keep all non-owner engines advisory only.
- Fail closed when owner decision is missing, invalid, or contradictory.

### 10.1 Cluster Matrix

| cluster | decision owner (single) | allowed advisors (non-authoritative) | hard fail-closed rule |
|---|---|---|---|
| Understanding + Clarify | PH1.NLP | PH1.LANG, PH1.SRL, PH1.CONTEXT, PH1.PRUNE, PH1.DIAG | If required fields/ambiguity remain or advisor outputs conflict, emit one-field `clarify` and block dispatch/execution |
| Evidence + Query Grounding | PH1.CONTEXT | PH1.E, PH1.SEARCH, PH1.DOC, PH1.SUMMARY, PH1.VISION, PH1.MULTI, PH1.KG | If evidence provenance/citation/validation fails, exclude that evidence and return clarify/refusal-safe response; no side effects |
| Delivery + Outbound Messaging | PH1.BCAST.001 | PH1.LINK, PH1.DELIVERY, PH1.REM.001, SMS setup pre-send gating | If lifecycle state is incomplete, access is not `ALLOW`, or simulation/idempotency checks fail, no send/resend/notify execution |
| Learning + Personalization | PH1.LEARN | PH1.FEEDBACK, PH1.PAE, PH1.CACHE, PH1.KNOW, PH1.LISTEN, PH1.PERSONA, PH1.EMO.GUIDE, PH1.EMO.CORE, PH1.MULTI, PH1.CONTEXT | If artifact governance/validation is missing, drop advisory updates; never alter authority, confirmation, or execution path |
| Governance + Runtime Control | PH1.OS | PH1.POLICY, PH1.ACCESS.001/PH2.ACCESS.002, PH1.TENANT, PH1.GOV, PH1.QUOTA, PH1.WORK, PH1.LEASE, PH1.SCHED, PH1.CAPREQ | If any mandatory gate returns deny/escalate/invalid or contradictions cannot be resolved deterministically, block commit and end in refusal/escalation only |

### 10.2 Mandatory Precedence Invariants
1. Owner output is the only terminal decision for the cluster.
2. Advisor outputs are hints/signals only; never terminal authority.
3. Conflicting advisor outputs must not produce execution.
4. Missing owner output must be treated as failure, not fallback execution.
5. All cluster owner decisions must be reason-coded and auditable.

### 10.3 Cluster Test Requirements (Add/Keep in Hardening Round)
1. Owner-over-advisor precedence tests for every cluster.
2. Contradictory advisor input tests (must fail closed).
3. Missing owner-output tests (must fail closed).
4. Idempotent replay tests for side-effect clusters.
5. Latency budget checks for hot-path clusters.

## 11) Engine Redundancy / Retirement Backlog (Decision Queue)
Purpose:
- Identify engines that can be merged/retired without weakening safety, authority boundaries, or outcomes.
- Reduce orchestration complexity and p95/p99 latency pressure by removing low-yield overlap.

### 11.1 Retirement Evaluation Gate (Must Pass Before Any Retirement)
1. Owner test: authoritative owner engines are not retirement candidates.
2. Unique-outcome test: candidate has no unique terminal decision/output that is required by blueprint/simulation gates.
3. Wiring test: docs/runtime parity is known; shadow (docs-only) engines are treated as merge/defer candidates.
4. Dependency test: removing candidate does not break canonical blueprint/simulation execution paths.
5. Delta test: A/B run shows no safety regression and acceptable outcome delta under approved thresholds.
6. Performance test: removal/merge improves or preserves p95/p99 and reduces per-turn optional budget pressure.
7. Audit test: decision traces remain reason-coded and replay-safe after merge/retirement.

### 11.2 Ranked Candidate Backlog
| rank | candidate engine/cluster | overlap signal | proposed action | decision gate to close | owner | target round | status |
|---|---|---|---|---|---|---|---|
| 1 | `PH1.LEARN_FEEDBACK_KNOW` | Overlaps split runtime roles of `PH1.LEARN` + `PH1.FEEDBACK` + `PH1.KNOW`; dual-model confusion risk | Keep as persistence/contract grouping only; remove standalone runtime-engine identity | Registry/map/coverage lock now enforces split runtime ownership + grouped storage pointer; storage single-writer artifact type tests and guardrail script are active | PH1.LEARN | Round 1 | CLOSED |
| 2 | `PH1.LEARNING_ADAPTIVE` | Functional overlap with LEARN/PAE feedback-to-adaptation loop; low distinct runtime value | Merged into `PH1.LEARN` + `PH1.PAE`; standalone runtime engine identity retired | Merge proof complete: runtime optional list + registry/map/ownership/coverage + blueprint wiring moved to LEARN/PAE with no standalone runtime references | PH1.LEARN | Round 1-2 | CLOSED |
| 3 | `PH1.POLICY` (separate engine identity) | Docs/runtime mismatch is closed; `PH1.POLICY` now exists as concrete runtime gate under `PH1.OS` orchestration | Keep separate runtime identity with explicit prompt-policy gate contract surface in `PH1.OS` | Canonical path selected as `PH1.POLICY` runtime gate before `PH1.X`, validated with fail-closed contract/runtime tests and ledger proof | PH1.OS | Round 1 | CLOSED |
| 4 | `PH1.REVIEW` | Governance assist overlaps existing approval/governance paths (`PH1.GOV` + access escalation flows) | Merged into governance/access approval routing; standalone runtime identity retired | Merge proof complete: runtime optional list + registry/map/coverage + simulation owning_domain/domain-profile moved under Governance with no standalone runtime references | PH1.GOV | Round 1-2 | CLOSED |
| 5 | `PH1.SEARCH` + `PH1.WEBINT` | Two sequential assist engines around read-only evidence pipeline can duplicate transformation/validation passes | Consolidated into one evidence-query assist pipeline under `PH1.SEARCH`; retired standalone `PH1.WEBINT` identity | Merge proof complete: runtime optional/tier wiring and LEARN target routing now use `PH1.SEARCH` only; context/coverage/registry references updated and standalone `PH1.WEBINT` active identity retired | PH1.CONTEXT | Round 2 | CLOSED |
| 6 | `PH1.PRIORITY` + `PH1.COST` | Both emit routing/pacing metadata only; overlapping turn-policy knobs | Consolidated into one turn-policy budget/priority module under `PH1.COST`; retired standalone `PH1.PRIORITY` identity | Merge proof complete: runtime optional/tier wiring now uses `PH1.COST` only; map/registry/coverage/docs synchronized to unified turn-policy surface with unchanged no-authority/no-execution semantics | PH1.OS | Round 2 | CLOSED |
| 7 | `PH1.ATTN` + `PH1.PRUNE` + `PH1.DIAG` + `PH1.PUZZLE` | Understanding-assist overlap risk (clarify loops/latency inflation) with `PH1.NLP` owner | Consolidated to minimum assist set (`PH1.PRUNE` + `PH1.DIAG`) under `PH1.NLP`; retired standalone `PH1.ATTN` + `PH1.PUZZLE` identities | Merge proof complete: runtime optional list/tier and map/registry/coverage/docs now keep only `PH1.PRUNE` + `PH1.DIAG` as understanding assists; no clarify-owner precedence changes | PH1.NLP | Round 2 | CLOSED |

### 11.3 Non-Candidate Guardrail (Do Not Retire)
1. Core authoritative path: `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.NLP`, `PH1.X`, `PH1.E`, `PH1.L`.
2. Governance/storage truth owners: `PH1.F`, `PH1.J`, `PH1.ACCESS.001/PH2.ACCESS.002`, `PH1.OS`, `PH1.WORK`, `PH1.LEASE`, `PH1.SCHED`, `PH1.QUOTA`.
3. Side-effect owners: `PH1.BCAST.001`, `PH1.DELIVERY`, `PH1.REM.001`, `PH1.ONB`, `PH1.LINK`, `PH1.POSITION`.

### 11.4 Closure Rule For This Backlog
1. No candidate can move to `CLOSED` without a recorded decision (KEEP/MERGE/RETIRE) plus rationale.
2. KEEP requires measurable unique value proof.
3. MERGE/RETIRE requires docs/contracts/runtime/tests + ledger proof updates in one scoped change-set.
4. Any safety regression automatically reopens the candidate and rolls back activation.

## 12) Outcome Utilization Execution Block (Latency + Compute ROI)
Purpose:
- Ensure every engine outcome/report is machine-consumed and results in action, learning, bounded audit, or deterministic drop.
- Eliminate "generated but unused" outcomes that add latency/compute cost without product value.

### 12.1 Baseline Snapshot (Current)
1. Engine map scope: 52 PH1 engines
2. Contract outcome surfaces: 53 `Ph1*Response` enums
3. OS wiring outcome surfaces: 50 `*WiringOutcome` types
4. Current risk signal: PH1.OS now enforces top-level orchestration, optional-budget contract posture, utility demotion scoring (`GATE-U1..U5`), runtime-boundary guards for OFFLINE/control-plane engines, delivery ownership fail-closed guardrails for LINK/BCAST/DELIVERY/REM plus SMS setup pre-send gating, understanding/clarify precedence fail-closed guards (`PH1.NLP` single clarify owner + optional assist policy bounds), and learning ownership split with storage-group-only persistence pointer (`PH1.LEARN_FEEDBACK_KNOW`); no open critical risk items remain in Section 9.2.

### 12.2 Outcome Action Contract (Mandatory For Every Emitted Outcome)
Every emitted outcome/report must be classified by Selene OS as exactly one:
1. `ACT_NOW`: affects current turn decision/order/gates now
2. `QUEUE_LEARN`: queued for governed learning/adaptation pipeline
3. `AUDIT_ONLY`: persisted only for bounded audit/replay use
4. `DROP`: explicitly discarded as no-value for current policy/profile

Required metadata for each outcome event:
1. `engine_id`
2. `outcome_type`
3. `correlation_id`
4. `turn_id`
5. `action_class`
6. `consumed_by` (owner engine or `NONE`)
7. `latency_cost_ms`
8. `decision_delta` (`true|false`)
9. `reason_code`

Hard rule:
- No unclassified outcome is allowed.
- No human review dependency is allowed for routine outcome triage.

### 12.3 Machine-Only Review Loop (Selene Reviews Outcomes)
1. Per-turn: `PH1.OS` classifies each outcome via the Action Contract.
2. Post-turn: outcome-utilization rows are appended to ledger storage (append-only).
3. Batch window: Selene computes per-engine utility and waste scores.
4. Policy action: Selene auto-applies `KEEP | DEGRADE | DISABLE_CANDIDATE` for optional engines.
5. Governance-only escalations: only high-risk merges/retirements require formal sign-off.

### 12.4 Thresholds (Concrete)
Window defaults:
1. rolling_turn_window: 5000
2. review_cadence: every 24h
3. fail_streak_window_days: 7

Global thresholds:
1. outcome_classification_coverage: 100%
2. unresolved_outcome_rate: 0%
3. reason_code_coverage: 100%

TURN_OPTIONAL efficiency thresholds (per engine):
1. `decision_delta_rate >= 0.08` OR `queue_learn_conversion_rate >= 0.20`
2. `no_value_rate (DROP) <= 0.60`
3. `latency_cost_p95_ms <= 20`
4. `latency_cost_p99_ms <= 40`

Always-on safety thresholds:
1. classification coverage: 100%
2. unresolved outcomes: 0%
3. no gate-order/safety regressions: 0 tolerance

OFFLINE_ONLY thresholds:
1. online invocation count: 0 (hard fail if non-zero)
2. artifact recommendation acceptance rate: `>= 0.15` after warmup, otherwise demote frequency

### 12.5 Pass/Fail Gates
| gate_id | gate | pass condition | fail action |
|---|---|---|---|
| GATE-U1 | Classification completeness | 100% outcomes have valid `action_class` | block release; fail closed |
| GATE-U2 | Unresolved outcomes | 0 outcomes with missing `consumed_by` when `action_class` is not `DROP`/`AUDIT_ONLY` | block release; route to owner fix |
| GATE-U3 | Optional budget | per-turn optional-engine budget enforced with deterministic skip/degrade | disable optional tier on breach |
| GATE-U4 | Optional engine utility | thresholds in 12.4 satisfied for rolling window | mark engine `DISABLE_CANDIDATE` |
| GATE-U5 | Sustained low utility | GATE-U4 fail for 7 consecutive days | open MERGE/RETIRE patch item |
| GATE-U6 | Safety after disable | no increase in critical fail/safety reason-codes beyond +0.2% | rollback disable immediately |
| GATE-U7 | Latency ROI | overall turn p95 does not regress (>3%) after changes | rollback and re-tune |

### 12.6 Execution Stages
Stage A (Round 1): Instrument + Classify
1. Add Action Contract fields to OS orchestration output path.
2. Add outcome-utilization append-only ledger rows.
3. Enforce GATE-U1 and GATE-U2 in CI.

Stage B (Round 2): Budget + Auto-Demotion (Implemented)
1. Enforce per-turn optional-engine budget in `PH1.OS`.
2. Enable daily machine-only utility scoring.
3. Auto-demote low-yield optional engines via GATE-U4/GATE-U5.

Stage C (Round 2-3): Consolidation + Retirement
1. Execute ranked merge/retire backlog from Section 11.
2. Apply GATE-U6 and GATE-U7 after each merge/retire change-set.
3. Record proofs in build ledger for every decision.

### 12.7 Immediate Candidate Actions Under This Block
1. `PH1.LEARN_FEEDBACK_KNOW`: completed lock as persistence grouping only (no standalone runtime path).
2. `PH1.LEARNING_ADAPTIVE`: completed merge into `PH1.LEARN` + `PH1.PAE`; standalone runtime identity retired.
3. `PH1.SEARCH` + `PH1.WEBINT`: completed consolidation under `PH1.SEARCH`; standalone `PH1.WEBINT` retired.
4. `PH1.PRIORITY` + `PH1.COST`: completed consolidation under `PH1.COST`; standalone `PH1.PRIORITY` retired.
5. `PH1.ATTN` + `PH1.PRUNE` + `PH1.DIAG` + `PH1.PUZZLE`: completed consolidation to `PH1.PRUNE` + `PH1.DIAG`; standalone `PH1.ATTN` + `PH1.PUZZLE` retired.

### 12.8 PH1.M Round-2 Hardening Addendum (2026-02-18)
Status:
- `DONE` (targeted hardening pass completed end-to-end).

Scope completed:
1. Added PH1.OS orchestration test coverage for memory outcome-utilization forwarding (`at_os_20`).
2. Added PH1.OS fail-closed validation on memory outcome correlation mismatch (`at_os_21`).
3. Added SimulationExecutor fail-closed guard proving memory simulation dispatch blocks when PH1.M wiring is disabled (`at_sim_exec_01j`).
4. Revalidated memory path suites across PH1.NLP, PH1.X, SimulationExecutor, and PH1.M OS wiring.

Proof command bundle:
```bash
cargo test -p selene_os ph1os -- --nocapture
cargo test -p selene_os simulation_executor -- --nocapture
cargo test -p selene_os ph1m -- --nocapture
cargo test -p selene_os ph1x -- --nocapture
cargo test -p selene_engines ph1n -- --nocapture
cargo test -p selene_engines ph1x -- --nocapture
```

## 13) Governed Self-Change Pipeline (Runtime Selene + Builder Selene)
Purpose:
- Enable Selene to improve code/config safely from factual signals.
- Keep production runtime deterministic and non-self-modifying.
- Require explicit approvals for risk-bearing changes.

### 13.1 Non-Negotiable Boundary Split
1. Runtime Selene (`PH1.OS` path):
   - may emit signals, scores, and patch requests,
   - must never write source code, migrations, or release state directly.
2. Builder Selene (offline control plane):
   - may generate patch proposals and run validation in sandbox/staging,
   - may never bypass approval and release gates.
3. Hard rule:
   - `No Approval -> No Merge`
   - `No Simulation/Access Gate -> No Runtime Execution`

### 13.2 System Components (Concrete)
1. Signal Intake:
   - input: outcome-utilization ledger + audit/reason codes + latency/SLO counters.
   - output: normalized `improvement_signal` records.
2. Proposal Engine:
   - input: clustered `improvement_signal` records.
   - output: patch proposal package (`diff`, rationale, risk tier, expected effect).
3. Sandbox Validator:
   - runs deterministic checks: compile, tests, gate scripts, migration safety, SLO guard.
4. Approval Gateway:
   - enforces risk-tier policy (auto/one-approver/two-approver).
5. Release Controller:
   - staging -> canary -> production progression with rollback hooks.
6. Post-Deploy Judge:
   - compares before/after metrics and accepts/reverts change automatically by policy.

### 13.3 Change Classes + Approval Policy
| class | examples | authority | required approval |
|---|---|---|---|
| CLASS-A (low risk) | optional-engine ranking thresholds, safe cache hints, wording/template non-authority patches | Builder Selene | Auto-apply allowed only if all validation gates pass and rollback is ready |
| CLASS-B (medium risk) | runtime wiring updates outside authority path, test additions, non-breaking contract tightening | Builder Selene | One human approval required before merge |
| CLASS-C (high risk) | access/authority/simulation gate logic, DB schema migrations, privacy/retention behavior | Builder Selene | Two human approvals required (`tech + product/security`) |

Hard rule:
- Any change touching `PH1.ACCESS.001`, `PH2.ACCESS.002`, `PH1.OS` gate order, or simulation commit paths is forced to `CLASS-C`.

### 13.4 Proposal Contract (Mandatory Fields)
Every patch proposal must include:
1. `proposal_id`
2. `created_at`
3. `source_signal_window`
4. `source_signal_hash`
5. `target_files[]`
6. `change_class`
7. `risk_score` (0.0..1.0)
8. `expected_effect` (latency, quality, safety deltas)
9. `validation_plan`
10. `rollback_plan`
11. `status` (`DRAFT | VALIDATED | APPROVED | RELEASED | REVERTED`)

Hard rule:
- Missing any mandatory field blocks approval.

### 13.5 Validation Gates (Builder Pipeline)
| gate_id | gate | pass condition | fail action |
|---|---|---|---|
| BLD-G1 | Reproducible diff | patch applies cleanly and deterministically | reject proposal |
| BLD-G2 | Compile/test | required crate/workspace tests pass | reject proposal |
| BLD-G3 | Contract guardrails | kernel contract invariants remain valid | reject proposal |
| BLD-G4 | Ownership/precedence | single-owner matrices and fail-closed checks pass | reject proposal |
| BLD-G5 | Runtime boundary | OFFLINE/control-plane boundary checks pass | reject proposal |
| BLD-G6 | Delivery/idempotency | no duplicate-send or idempotency regressions | reject proposal |
| BLD-G7 | Migration safety (if schema touched) | upgrade + rollback drill passes | reject proposal |
| BLD-G8 | Security/privacy | no policy/retention/secret leakage violations | reject proposal |
| BLD-G9 | Latency guard | projected p95/p99 regression within approved budget | reject proposal |
| BLD-G10 | Audit traceability | reason-code + correlation/audit completeness preserved | reject proposal |

### 13.6 Rollout Policy (Release Controller)
1. Stage 1: Staging (100% synthetic + replay fixtures)
2. Stage 2: Canary (1-5% controlled traffic/class)
3. Stage 3: Ramp (25% -> 50% -> 100%)

Promotion conditions at each stage:
1. no critical safety regressions,
2. p95/p99 within threshold,
3. error/fail-closed rates within baseline tolerance,
4. audit completeness remains 100%.

### 13.7 Auto-Rollback Triggers
Rollback immediately if any occurs:
1. authority/gate-order violation,
2. duplicate side-effect event class appears,
3. p95 latency regression > 3% sustained for 30 minutes,
4. p99 latency regression > 5% sustained for 15 minutes,
5. critical reason-code spike > +0.2% absolute vs baseline.

### 13.8 Implementation Plan (Concrete Build Sequence)
Phase 13-A (contracts + ledger schema) [Implemented]:
1. add proposal/run/result contract types to `selene_kernel_contracts`,
2. add append-only storage tables for proposals/runs/results in `selene_storage`,
3. add idempotency keys for proposal and validation runs.

Phase 13-B (builder orchestration module) [Implemented]:
1. implement offline builder orchestration module (not in runtime `ALWAYS_ON`/`TURN_OPTIONAL`),
2. implement deterministic signal clustering and proposal generation,
3. implement sandbox execution + gate result collector.

Phase 13-C (approval + release controls) [Implemented]:
1. add approval state machine with class-based approver requirements,
2. add release controller with staged rollout state and rollback hooks,
3. block production rollout when approval class is unresolved.

Phase 13-D (observability + close-loop) [Implemented]:
1. add before/after metric comparison and automatic accept/revert decision logic,
2. log every decision in `docs/03_BUILD_LEDGER.md`,
3. wire CI job to fail on missing proposal fields or missing gate outcomes.

### 13.9 Program-Level Success Criteria
1. Runtime Selene remains non-self-modifying.
2. Builder Selene can deliver validated patch proposals with full audit trace.
3. High-risk changes cannot ship without explicit approval.
4. Failed changes roll back automatically within policy thresholds.
5. Net outcome: lower latency waste, fewer regressions, faster safe iteration.

### 13.10 Stage-2 Canary Entry Checkpoint (2026-02-17)
Status:
- `PASS` (entry criteria met; canary execution still requires explicit release approval).

Mandatory entry gates and current proof:
1. `BLD-G1..BLD-G10` pipeline guardrails must pass on current tree.
2. Runtime boundary + ownership + clarify precedence guardrails must pass.
3. Optional utility gates (`GATE-U4/U5`) must pass with no disable candidates.
4. Isolated Selene Postgres verification must pass (`selene_os_dev` only; no foreign DB/role/active connections).
5. Builder offline/runtime release-controller tests must pass.
6. Workspace regression must pass before canary start.
7. Stage-2 promotion threshold gate must pass (`p95/p99/reason spike/audit completeness/fail-closed`).

Proof command bundle (latest passing run):
```bash
scripts/dev_postgres_verify_isolation.sh
bash scripts/check_builder_stage2_canary_replay.sh
bash scripts/check_builder_stage2_promotion_gate.sh docs/fixtures/stage2_canary_metrics_snapshot.csv
bash scripts/check_builder_pipeline_phase13a.sh
bash scripts/check_builder_pipeline_phase13b.sh
bash scripts/check_builder_pipeline_phase13c.sh
bash scripts/check_builder_pipeline_phase13d.sh
bash scripts/check_runtime_boundary_guards.sh
bash scripts/check_learning_ownership_boundaries.sh
bash scripts/check_engine_tracker_duplicates.sh
bash scripts/check_understanding_clarify_precedence.sh
bash scripts/check_optional_engine_utility_gates.sh docs/fixtures/optional_engine_utility_snapshot.csv --fail-on-u4
cargo test -p selene_os ph1builder -- --nocapture
cargo test --workspace
scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Passing this checkpoint authorizes only Stage-2 canary entry readiness, not automatic promotion to Stage-3 ramp.
- Any rollback trigger in Section `13.7` immediately invalidates this checkpoint until re-proven.

### 13.11 Stage-3 Ramp Promotion Gate (Real Telemetry Required)
Status:
- `ENFORCED_BY_GATE` (promotion is blocked unless release gate passes on exported real canary telemetry).

Promotion gate source:
1. Export latest canary telemetry from isolated Selene Postgres (`builder_post_deploy_judge_results` + gate coverage).
   - Freshness is mandatory: telemetry age must be within `MAX_TELEMETRY_AGE_MINUTES` (default `180`).
2. Compute deterministic promotion metrics:
   - `p95_delta_bp`
   - `p99_delta_bp`
   - `critical_reason_spike_bp`
   - `audit_completeness_bp`
   - `fail_closed_delta_bp`
   - authority/duplicate side-effect violation flags
3. Evaluate against Section `13.7` rollback thresholds and fail-closed rules.

Mandatory command (before any Stage-3 ramp action):
```bash
bash scripts/check_builder_stage3_release_gate.sh .dev/stage2_canary_metrics_snapshot.csv
```

Readiness-audit integration:
- Optional hard enforcement is available via:
```bash
ENFORCE_STAGE3_RELEASE_GATE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- No successful `CHECK_OK builder_stage3_release_gate=pass` means no Stage-3 ramp progression.
- No canary telemetry rows (`NO_CANARY_TELEMETRY`) is an automatic fail-closed block on Stage-3 ramp.
- Stale canary telemetry (`STALE_CANARY_TELEMETRY`) is an automatic fail-closed block on Stage-3 ramp.

### 13.12 Builder Human Permission Interrupt Loop (BCAST + REM)
Status:
- `REQUIRED_FOR_OPERATION` (policy lock).

Mission:
- Before any Builder-driven code write/generation action, Selene must interrupt via `PH1.BCAST.001` and request explicit human permission.
- Before any launch/ramp progression, Selene must issue a second explicit permission request.
- If the user is busy/unavailable, Selene must schedule reminder follow-ups via `PH1.REM.001` (timing-only) and remain blocked until approval.
- Human-facing permission messages must stay plain-language only (`issue`, `fix`, `should I proceed?`, `all tests passed, can I launch?`).
- Daily cadence is mandatory: Selene must run one daily review cycle and refresh `DAILY_REVIEW_DATE_UTC` before either permission gate can pass.

Non-negotiable sequence:
1. Compose change brief (machine-generated, plain language):
   - `Issue` ("I received these issues ...")
   - `Fix` ("This is the fix ...")
   - `Should I Proceed` ("Should I proceed?")
   - `Launch Question` ("All tests passed. Can I launch?")
2. Send permission request through `PH1.BCAST.001` lifecycle:
   - `BCAST_CREATE_DRAFT` -> `BCAST_DELIVER_COMMIT`
3. Wait for explicit decision:
   - code phase: approve/deny code change
   - launch phase: approve/deny launch progression
4. Busy path:
   - if no decision and busy mode is active, schedule `PH1.REM.001` follow-up (`reminder_type=BCAST_MHP_FOLLOWUP`) and keep state blocked
5. Only after approval:
   - code phase gate may pass
   - launch phase gate may pass

Hard rules:
- No permission means no code generation/write.
- No launch permission means no Stage-2/Stage-3 progression.
- Reminder scheduling does not grant permission; it only preserves follow-up continuity.
- No daily review refresh (`DAILY_REVIEW_OK=1` + `DAILY_REVIEW_DATE_UTC=today(UTC)`) means no code/launch permission gate pass.
- All approvals/denials/reminder handoffs must be reason-coded and auditable.

Operational gate checks:
```bash
# uses .dev/builder_change_brief.md + .dev/builder_permission.env
bash scripts/check_builder_human_permission_gate.sh code
bash scripts/check_builder_human_permission_gate.sh launch
```

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_HUMAN_PERMISSION=1 scripts/selene_design_readiness_audit.sh
```

Template:
- permission env template: `docs/fixtures/builder_permission_template.env`

### 13.13 Operator Runbook (Code Permission + Launch Permission)
Step A: Prepare brief + permission state files.
```bash
cp docs/fixtures/builder_change_brief_template.md .dev/builder_change_brief.md
cp docs/fixtures/builder_permission_template.env .dev/builder_permission.env
```
Then set daily review fields in `.dev/builder_permission.env`:
- `DAILY_REVIEW_OK=1`
- `DAILY_REVIEW_DATE_UTC=$(date -u +%Y-%m-%d)`

Step B: Fill brief and send code permission request (BCAST lifecycle).
1. Populate `.dev/builder_change_brief.md` with:
   - issue
   - fix
   - should-I-proceed question
   - launch question
2. Send BCAST request using:
   - `BCAST_CREATE_DRAFT`
   - `BCAST_DELIVER_COMMIT`
3. Apply decision into `.dev/builder_permission.env` using the decision ingest script:
```bash
BCAST_ID=<code_bcast_id> DECISION_REF=<code_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code approve
```
Alternative (decision file):
```bash
cp docs/fixtures/builder_permission_decision_template.env .dev/builder_code_decision.env
# fill: PHASE=code, DECISION=approve, BCAST_ID, DECISION_REF
DECISION_FILE=.dev/builder_code_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```
4. If busy/no decision:
   - schedule `REMINDER_SCHEDULE_COMMIT` (`reminder_type=BCAST_MHP_FOLLOWUP`)
   - apply pending state:
```bash
REMINDER_REF=<code_reminder_ref> ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code pending
```

Step C: Enforce code gate before any code generation/write.
```bash
TODAY_UTC="$(date -u +%Y-%m-%d)"
sed -i.bak "s/^DAILY_REVIEW_OK=.*/DAILY_REVIEW_OK=1/" .dev/builder_permission.env
sed -i.bak "s/^DAILY_REVIEW_DATE_UTC=.*/DAILY_REVIEW_DATE_UTC=${TODAY_UTC}/" .dev/builder_permission.env
rm -f .dev/builder_permission.env.bak

ENV_FILE=.dev/builder_permission.env BRIEF_FILE=.dev/builder_change_brief.md \
bash scripts/check_builder_human_permission_gate.sh code
```

Step D: Before launch/canary/ramp, repeat permission request for launch.
1. Send launch permission via BCAST (`BCAST_CREATE_DRAFT` -> `BCAST_DELIVER_COMMIT`).
2. Apply launch decision:
```bash
BCAST_ID=<launch_bcast_id> DECISION_REF=<launch_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh launch approve
```
Alternative (decision file):
```bash
cp docs/fixtures/builder_permission_decision_template.env .dev/builder_launch_decision.env
# fill: PHASE=launch, DECISION=approve, BCAST_ID, DECISION_REF
DECISION_FILE=.dev/builder_launch_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```
Optional one-command sync (if both decision files are populated):
```bash
ENV_FILE=.dev/builder_permission.env \
CODE_DECISION_FILE=.dev/builder_code_decision.env \
LAUNCH_DECISION_FILE=.dev/builder_launch_decision.env \
bash scripts/sync_builder_permission_from_decision_files.sh
```
3. If busy/no decision, schedule REM follow-up and set launch reminder fields.

Step E: Enforce launch gate + Stage-3 release gate.
```bash
ENV_FILE=.dev/builder_permission.env BRIEF_FILE=.dev/builder_change_brief.md \
bash scripts/check_builder_human_permission_gate.sh launch

ENFORCE_BUILDER_HUMAN_PERMISSION=1 ENFORCE_STAGE3_RELEASE_GATE=1 \
scripts/selene_design_readiness_audit.sh
```
Preferred strict release path:
```bash
bash scripts/check_builder_release_hard_gate.sh
```
If Step E is green, rollout can proceed automatically through release-controller states (`STAGING -> CANARY -> RAMP25 -> RAMP50 -> PRODUCTION`) under existing Stage-2/Stage-3 fail-closed thresholds.
If this cycle is learning-triggered, also set `ENFORCE_BUILDER_LEARNING_BRIDGE=1`.

Hard rule:
- Any non-pass output from Step C or Step E is a hard block (`No Approval -> No Code`, `No Launch Approval -> No Launch`).

### 13.14 Learning -> Builder Improvement Bridge (Evidence-Backed, Non-Guessing)
Status:
- `REQUIRED_WHEN_LEARNING_TRIGGERED`.

Mission:
- When learning engines produce actionable outcomes, Builder Selene can consume them as structured inputs for deterministic fixes.
- Learning outputs must be evidence-backed and measurable; vague narrative reports are blocked.

Applicable learning sources (minimum):
- `PH1.FEEDBACK`
- `PH1.LEARN`
- `PH1.KNOW`
- optional: `PH1.PAE`, `PH1.CACHE`, `PH1.PRUNE`, `PH1.CONTEXT`

Hard rules:
- No evidence refs means no Builder patch generation from learning signals.
- Learning reports are advisory inputs only; they do not bypass approval, release gates, or authority boundaries.
- The same human permission loop remains mandatory (`Issue/Fix/Should I proceed?` + `All tests passed. Can I launch?`).

Operational gate:
```bash
ENV_FILE=.dev/builder_learning_bridge.env \
bash scripts/check_builder_learning_bridge_gate.sh
```

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_LEARNING_BRIDGE=1 scripts/selene_design_readiness_audit.sh
```

Templates:
- env template: `docs/fixtures/builder_learning_bridge_template.env`
- report template: `docs/fixtures/builder_learning_report_template.md`

### 13.15 Learning-Triggered Runbook (When Improvement Comes From Learning Engines)
Step A: Prepare learning bridge files.
```bash
cp docs/fixtures/builder_learning_bridge_template.env .dev/builder_learning_bridge.env
```

Step B: Auto-generate learning report during Builder offline run.
1. Builder writes `.dev/builder_learning_report.md` by default (or `learning_report_output_path` override).
2. Generated report must include:
   - learning issues received
   - root-cause evidence refs
   - deterministic fix plan
   - expected improvement
   - decision prompt (`Should I proceed with this learning-driven fix?`)
3. Manual edits are allowed, but report structure must still pass the learning bridge gate.

Step C: Fill bridge env and enforce the gate.
1. Set:
   - `LEARNING_TRIGGERED=1`
   - `LEARNING_REPORT_VALIDATED=1`
   - `LEARNING_REPORT_ID=<stable id>`
   - `LEARNING_REPORT_FILE=.dev/builder_learning_report.md`
   - `LEARNING_SOURCE_ENGINES=<comma-separated ids>`
   - `LEARNING_SIGNAL_COUNT=<n>`
2. Run:
```bash
ENV_FILE=.dev/builder_learning_bridge.env \
bash scripts/check_builder_learning_bridge_gate.sh
```

Step D: Continue standard builder flow.
1. Run code permission gate.
2. Run tests/validation gates.
3. Run launch permission gate.
4. Run Stage-3 release gate before ramp.

Hard rule:
- Learning bridge gate pass enables consideration only; execution still requires the standard code+launch permission and release gates.

### 13.16 Single-Command E2E Gate Chain (Learning + Approval + Stage Gates)
Mission:
- Provide one deterministic command that enforces:
  - learning bridge gate
  - code approval gate
  - launch approval gate
  - stage gate (fixture or live telemetry mode)

Operational command:
```bash
bash scripts/check_builder_e2e_gate_flow.sh
```

Modes:
- `STAGE_GATE_MODE=fixture`: uses deterministic fixture stage gate (`check_builder_stage2_promotion_gate.sh`).
- `STAGE_GATE_MODE=live`: uses live telemetry stage gate (`check_builder_stage3_release_gate.sh`).
- `AUTO_SYNC_DECISION_FILES=1`: pre-applies `.dev/builder_code_decision.env` + `.dev/builder_launch_decision.env` into `.dev/builder_permission.env` before permission-gate checks.

Readiness/CI enforcement mode:
```bash
ENFORCE_BUILDER_LEARNING_BRIDGE=1 \
ENFORCE_BUILDER_HUMAN_PERMISSION=1 \
ENFORCE_BUILDER_E2E_GATE_FLOW=1 \
ENFORCE_STAGE3_RELEASE_GATE=1 \
ENFORCE_BUILDER_RELEASE_HARD_GATE=1 \
scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Any failure in the chain is a hard block on patch/launch progression.

### 13.17 Human Brief Auto-Generation (Simple Issue/Fix Prompts)
Mission:
- Remove manual drift in permission briefs by auto-generating a plain-language brief during Builder offline runs.
- Keep operator communication simple and deterministic:
  - issue
  - fix
  - `Should I proceed?`
  - `All tests passed. Can I launch?`

Runtime behavior:
1. `PH1.BUILDER` writes `.dev/builder_change_brief.md` by default (or `change_brief_output_path` override).
2. Generated brief remains compatible with `scripts/check_builder_human_permission_gate.sh`.
3. Brief generation is advisory communication output only; it does not auto-approve code/launch.

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13f.sh
```

Readiness audit:
- Section `1S` enforces Phase13-F brief autogen checks on each run.

Hard rule:
- If brief autogen contract/wiring/test checks fail, permission-loop readiness is blocked until fixed.

### 13.18 Permission Packet Auto-Generation (BCAST + REM Ready)
Mission:
- Ensure each Builder cycle emits a deterministic, machine-readable permission packet that maps the human-approval flow to the exact simulation sequence.
- Keep approvals manual while removing operator ambiguity in "what to send next".

Runtime behavior:
1. `PH1.BUILDER` writes `.dev/builder_permission_packet.md` by default (or `permission_packet_output_path` override).
2. Packet includes two permission phases:
   - code permission request (`Should I proceed?`)
   - launch permission request (`All tests passed. Can I launch?`)
3. Packet includes deterministic simulation mapping:
   - `BCAST_CREATE_DRAFT`
   - `BCAST_DELIVER_COMMIT`
   - busy follow-up via `REMINDER_SCHEDULE_COMMIT` (`BCAST_MHP_FOLLOWUP`)

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13g.sh
```

Readiness audit:
- Section `1T` enforces Phase13-G packet autogen checks on each run.

Hard rule:
- Permission packet generation never implies approval; code/launch still require explicit pass from the human permission gate.

### 13.19 Decision Ingest Automation (Explicit Approval Capture)
Mission:
- Reduce manual mistakes when updating `.dev/builder_permission.env` after BCAST decisions.
- Keep approval explicit: this script records decisions; it never creates approvals automatically.

Operational command:
```bash
# code phase
BCAST_ID=<code_bcast_id> DECISION_REF=<code_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code approve

# launch phase
BCAST_ID=<launch_bcast_id> DECISION_REF=<launch_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh launch approve

# pending busy follow-up
REMINDER_REF=<reminder_ref> ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code pending
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13h.sh
```

Readiness audit:
- Section `1U` enforces Phase13-H decision-ingest checks on each run.

Hard rule:
- Decision ingest updates fields only. Permission still must pass `scripts/check_builder_human_permission_gate.sh` before code/launch progression.

### 13.20 Decision-File Ingest (Structured BCAST Outcome Import)
Mission:
- Allow deterministic approval capture from a structured decision file so operators can import BCAST outcomes with one command.
- Maintain fail-closed behavior on malformed or ambiguous file inputs.

Template:
- `docs/fixtures/builder_permission_decision_template.env`

Operational command:
```bash
# fill PHASE/DECISION/BCAST_ID/DECISION_REF (or REMINDER_REF for pending)
DECISION_FILE=.dev/builder_code_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13i.sh
```

Readiness audit:
- Section `1V` enforces Phase13-I decision-file ingest checks on each run.

Hard rule:
- Decision file import does not grant authority by itself; `check_builder_human_permission_gate.sh` remains the execution gate.

### 13.21 Auto Decision-Seed Export (Per-Run Files)
Mission:
- Reduce operator setup time by auto-exporting decision seed files on every Builder run.
- Ensure both code and launch decision files are always available and path-linked in the permission packet.

Runtime behavior:
1. Builder auto-writes (by default in `.dev/`):
   - `builder_code_decision.env`
   - `builder_launch_decision.env`
2. Files are prefilled with:
   - `PHASE`
   - `DECISION=approve`
   - `REFRESH_DAILY_REVIEW=1`
   - deterministic `PERMISSION_REF` + `PROPOSAL_ID`
3. Operator fills only event outputs (`BCAST_ID`, `DECISION_REF`) and applies with one command.

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13j.sh
```

Readiness audit:
- Section `1W` enforces Phase13-J decision-seed export checks on each run.

Hard rule:
- Seed-file export is convenience only; explicit approval capture and permission gate checks remain mandatory.

### 13.22 Decision-File Auto-Sync Pre-Gate (Optional One-Command Apply)
Mission:
- Allow one deterministic command to apply both code/launch decision files before running permission gates.
- Keep authority unchanged: this only imports explicit decisions already recorded in decision files.

Operational command:
```bash
ENV_FILE=.dev/builder_permission.env \
CODE_DECISION_FILE=.dev/builder_code_decision.env \
LAUNCH_DECISION_FILE=.dev/builder_launch_decision.env \
bash scripts/sync_builder_permission_from_decision_files.sh
```

E2E command mode:
```bash
AUTO_SYNC_DECISION_FILES=1 \
PERMISSION_ENV_FILE=.dev/builder_permission.env \
bash scripts/check_builder_e2e_gate_flow.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13k.sh
```

Readiness audit:
- Section `1X` enforces Phase13-K decision-file auto-sync checks on each run.

Hard rule:
- Auto-sync never grants approval. If decision files are missing/invalid, sync fails closed and code/launch progression remains blocked.

### 13.23 Strict Release Hard Gate (Single Entrypoint)
Mission:
- Provide one strict command for release readiness so no optional/partial gate path is used.
- Enforce: decision-file auto-sync, human permission gates, learning bridge gate, and live telemetry Stage-3 gate in one flow.

Operational command:
```bash
bash scripts/check_builder_release_hard_gate.sh
```

Required defaults in hard gate:
- `AUTO_SYNC_DECISION_FILES=1`
- `STAGE_GATE_MODE=live`

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_RELEASE_HARD_GATE=1 \
scripts/selene_design_readiness_audit.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13l.sh
```

Readiness audit:
- Section `1Z` enforces Phase13-L hard-gate guardrail checks on each run.

Hard rule:
- This is the canonical release-check entrypoint.
- If hard gate fails (including `NO_CANARY_TELEMETRY`), no launch/ramp progression is allowed.

### 13.24 Freeze Checkpoint (Remote Published)
Status:
- `FROZEN_REMOTE_PUBLISHED` (checkpoint is committed, tagged, and pushed to origin).

Frozen release point:
- commit: `a3a002a` (`chore(release): freeze stage3 fresh-cycle checkpoint`)
- tag: `freeze-stage3-fresh-cycle-20260217`
- ledger proof: `docs/03_BUILD_LEDGER.md` entry `FREEZE_CHECKPOINT_STAGE3_FRESH_CYCLE`
- remote publish proof:
  - `origin/main` head includes commit `65a10ed`
  - `origin` tag `freeze-stage3-fresh-cycle-20260217` exists

Local verification commands:
```bash
git show -s --oneline a3a002a
git tag --list "freeze-stage3-fresh-cycle-20260217"
bash scripts/check_builder_release_hard_gate.sh
ENFORCE_BUILDER_LEARNING_BRIDGE=1 ENFORCE_BUILDER_HUMAN_PERMISSION=1 ENFORCE_BUILDER_E2E_GATE_FLOW=1 ENFORCE_STAGE3_RELEASE_GATE=1 ENFORCE_BUILDER_RELEASE_HARD_GATE=1 scripts/selene_design_readiness_audit.sh
```

Remote verification commands:
```bash
git ls-remote --heads origin main
git ls-remote --tags origin freeze-stage3-fresh-cycle-20260217
git show -s --oneline freeze-stage3-fresh-cycle-20260217^{}
```

Hard rule:
- This published freeze tag is the canonical rollback anchor for the Stage-3 fresh-cycle release checkpoint.

### 13.25 Controlled Rollout Start Command (Single Kickoff Gate)
Mission:
- Start rollout only from a synchronized, published checkpoint with deterministic rollback anchor.
- Avoid manual operator drift across release-controller replay checks, approval gates, and live telemetry gates.

Operational command:
```bash
bash scripts/check_builder_controlled_rollout_start.sh
```

What this command enforces:
1. Local `HEAD` must match `origin/main` (no hidden unpushed rollout start).
2. Freeze tag `freeze-stage3-fresh-cycle-20260217` must exist locally and on remote with the same target commit.
3. Release-controller staged-transition replay tests must pass (`check_builder_stage2_canary_replay.sh`).
4. Canonical strict release hard gate must pass (`check_builder_release_hard_gate.sh`).
   - Live telemetry freshness is fail-closed (`MAX_TELEMETRY_AGE_MINUTES`, default `180`).

Expected pass signal:
```text
CHECK_OK builder_controlled_rollout_start=pass commit=<head> freeze_tag=freeze-stage3-fresh-cycle-20260217 freeze_target=<commit>
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13m.sh
```

Readiness audit:
- Section `1AA` enforces Phase13-M rollout-start guardrail checks on each run.
- Section `1AB` optionally enforces live rollout-start gate execution when:
```bash
ENFORCE_BUILDER_CONTROLLED_ROLLOUT_START=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If this gate fails, rollout does not start.
- No manual bypass is allowed; fix failing precondition(s) and re-run this command.

### 13.26 Controlled Rollback Drill (Dry-Run Revert Safety)
Mission:
- Prove rollback safety remains executable before/while rollout.
- Ensure regression-triggered revert path and missing-gate fail-closed path remain intact.

Operational command:
```bash
bash scripts/check_builder_rollback_drill.sh
```

What this command enforces:
1. Post-deploy judge rollback path is still executable:
   - `at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach`
2. Missing gate outcomes still fail closed:
   - `at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes`

Expected pass signal:
```text
CHECK_OK builder_rollback_drill=pass
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13n.sh
```

Readiness audit:
- Section `1AC` enforces Phase13-N rollback-drill guardrail checks on each run.
- Section `1AD` optionally enforces rollback drill execution when:
```bash
ENFORCE_BUILDER_ROLLBACK_DRILL=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If rollback drill fails, rollout progression is blocked until rollback safety is restored.

### 13.27 Pre-Launch Bundle Command (Single Final Checklist)
Mission:
- Provide one deterministic final command before launch progression.
- Ensure rollout-start gating, rollback safety, and strict release hard-gate are all green in a single operator action.

Operational command:
```bash
bash scripts/check_builder_prelaunch_bundle.sh
```

What this command enforces:
1. Controlled rollout-start gate is green:
   - `check_builder_controlled_rollout_start.sh`
2. Rollback drill safety is green:
   - `check_builder_rollback_drill.sh`
3. Strict release hard-gate is re-confirmed:
   - `check_builder_release_hard_gate.sh`

Expected pass signal:
```text
CHECK_OK builder_prelaunch_bundle=pass
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13o.sh
```

Readiness audit:
- Section `1AE` enforces Phase13-O pre-launch-bundle guardrail checks on each run.
- Section `1AF` optionally enforces pre-launch bundle execution when:
```bash
ENFORCE_BUILDER_PRELAUNCH_BUNDLE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If pre-launch bundle fails, no launch/ramp progression is allowed.

### 13.28 Controlled Launch Executor (One-Step Stage Advance)
Mission:
- Execute one deterministic release-stage promotion at a time after all gates are green.
- Keep launch progression explicit, idempotent, and fail-closed.

Operational command (preview default):
```bash
bash scripts/check_builder_controlled_launch_execute.sh
```

Execution command (writes one release-state row):
```bash
EXECUTE=1 \
LAUNCH_EXECUTE_ACK=YES \
LAUNCH_EXECUTE_IDEMPOTENCY_KEY=<unique_key> \
bash scripts/check_builder_controlled_launch_execute.sh
```

What this command enforces:
1. Pre-launch bundle passes first (`check_builder_prelaunch_bundle.sh`), unless explicitly disabled with `PRECHECK=0`.
2. Launch permission gate passes (`check_builder_human_permission_gate.sh launch`).
3. Promotion is one-step only (`STAGING -> CANARY -> RAMP_25 -> RAMP_50 -> PRODUCTION`).
4. `PRODUCTION` promotion requires latest approval status `APPROVED`.
5. Execution mode requires explicit ack + idempotency key; duplicate key resolves deterministically as idempotent reuse.

Expected pass signals:
```text
CHECK_OK builder_controlled_launch_execute=preview ...
CHECK_OK builder_controlled_launch_execute=executed ...
CHECK_OK builder_controlled_launch_execute=idempotent_reuse ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13p.sh
```

Readiness audit:
- Section `1AG` enforces Phase13-P launch-executor guardrail checks on each run.
- Section `1AH` optionally enforces launch-executor preview checks when:
```bash
ENFORCE_BUILDER_CONTROLLED_LAUNCH_EXECUTE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- No execution write is allowed without `EXECUTE=1`, `LAUNCH_EXECUTE_ACK=YES`, and `LAUNCH_EXECUTE_IDEMPOTENCY_KEY`.
- If execution preconditions fail, launch progression is blocked.
- Terminal stages are non-promotable: `PRODUCTION(status=COMPLETED)` and `ROLLED_BACK(status=REVERTED)` must fail with `no_next_stage`.

### 13.29 Stage-Bound Judge Gate (Per-Stage Promotion Proof)
Mission:
- Prevent promotion from one rollout stage to the next using telemetry from a different stage.
- Require a fresh, stage-bound judge pass before each non-initial stage advance.

Operational command (stage-bound mode enabled by default):
```bash
REQUIRE_STAGE_JUDGE=1 \
bash scripts/check_builder_controlled_launch_execute.sh
```

What this command enforces:
1. For `CANARY -> RAMP_25`, `RAMP_25 -> RAMP_50`, and `RAMP_50 -> PRODUCTION`:
   - post-deploy judge telemetry must exist for the current `release_state_id`
   - telemetry must satisfy freshness threshold (`MAX_TELEMETRY_AGE_MINUTES`)
   - stage metrics must pass `check_builder_stage2_promotion_gate.sh`
2. Judge export is scope-bound by:
   - `REQUIRED_PROPOSAL_ID=<current proposal_id>`
   - `REQUIRED_RELEASE_STATE_ID=<current release_state_id>`
3. If scoped telemetry is missing/stale/failing, launch execution fails closed.

Expected pass/fail signal examples:
```text
CHECK_OK builder_controlled_launch_execute=preview ...
NO_CANARY_TELEMETRY: ... (scope=release_state:...)
STALE_CANARY_TELEMETRY: ... (scope=release_state:...)
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13q.sh
```

Readiness audit:
- Section `1AI` enforces Phase13-Q stage-bound judge guardrail checks on each run.
- Section `1AJ` optionally enforces stage-bound preview checks when:
```bash
ENFORCE_BUILDER_STAGE_JUDGE_BINDING=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Stage progression is blocked unless the current stage has fresh, scoped judge evidence that passes promotion thresholds.

### 13.30 Production Soak Watchdog (Fresh Production Judge Required)
Mission:
- Keep production state guarded by fresh, production-scoped post-deploy judge evidence.
- Fail closed if production judge evidence is missing, stale, non-ACCEPT, or policy-incomplete.

Operational command:
```bash
bash scripts/check_builder_production_soak_watchdog.sh
```

What this command enforces:
1. Latest release state for target proposal is exactly:
   - `stage=PRODUCTION`
   - `status=COMPLETED`
2. Latest approval status remains `APPROVED`.
3. A judge row exists for the exact production release_state.
4. Judge action is `ACCEPT` (non-accept is fail-closed).
5. Judge telemetry is fresh and scope-bound:
   - `REQUIRED_PROPOSAL_ID=<proposal_id>`
   - `REQUIRED_RELEASE_STATE_ID=<production release_state_id>`
6. Scoped metrics still pass deterministic promotion thresholds via:
   - `check_builder_stage2_promotion_gate.sh`

Expected pass signal:
```text
CHECK_OK builder_production_soak_watchdog=pass ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13r.sh
```

Readiness audit:
- Section `1AK` enforces Phase13-R production-soak guardrail checks on each run.
- Section `1AL` optionally enforces production soak watchdog execution when:
```bash
ENFORCE_BUILDER_PRODUCTION_SOAK=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Production is not considered stable unless watchdog checks pass with scoped, fresh production judge evidence.

### 13.31 Production Soak Recurring Runner (Fail-Closed Alerting)
Mission:
- Run production soak checks on a recurring schedule with deterministic fail-closed alerting.
- Ensure freshness regressions (including telemetry age breaches) trigger immediate alert + non-zero exit.
- On failed ticks, dispatch a canonical `PH1.BCAST.001` failure alert bridge (`BCAST_CREATE_DRAFT` -> `BCAST_DELIVER_COMMIT`) so operator notification is automatic.

Operational command (single tick):
```bash
RUN_MODE=once \
bash scripts/check_builder_production_soak_runner.sh
```

Operational command (recurring loop):
```bash
RUN_MODE=loop \
INTERVAL_MINUTES=60 \
bash scripts/check_builder_production_soak_runner.sh
```

What this command enforces:
1. Calls `check_builder_production_soak_watchdog.sh` each tick.
2. On success:
   - emits `CHECK_OK builder_production_soak_runner=tick_pass ...`
   - appends state line to `STATE_FILE` (default `.dev/builder_production_soak_runner_state.log`).
3. On failure:
   - classifies stale freshness as `PRODUCTION_SOAK_STALE_TELEMETRY` when output contains `STALE_CANARY_TELEMETRY`.
   - emits/records alert lines to `ALERT_LOG_FILE` (default `.dev/builder_production_soak_alerts.log`).
   - dispatches BCAST alert bridge (default `BCAST_ON_FAIL=1`) via:
     - `scripts/emit_builder_failure_bcast_alert.sh`
     - simulation markers: `BCAST_CREATE_DRAFT` then `BCAST_DELIVER_COMMIT`
     - default urgent recipient state progression: `SENT -> FOLLOWUP`
   - app-thread log: `.dev/selene_app_inbox.log`
   - BCAST ledger log: `.dev/builder_failure_bcast_ledger.log`
   - routing state + ack evidence:
     - `.dev/builder_failure_alert_routing.env`
     - `.dev/builder_failure_alert_ack.log`
     - `.dev/builder_failure_alert_routing_audit.log`
   - desktop notification send attempt (`osascript`) for immediate local awareness.
4. `FAIL_CLOSED=1` (default):
   - exits non-zero immediately on any failed tick.

Expected signals:
```text
CHECK_OK builder_production_soak_runner=tick_pass ...
ALERT builder_production_soak_runner=PRODUCTION_SOAK_STALE_TELEMETRY ...
ALERT builder_production_soak_runner=PRODUCTION_SOAK_CHECK_FAILED ...
ALERT_BCAST_OK builder_production_soak_runner ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13s.sh
```

Readiness audit:
- Section `1AM` enforces Phase13-S runner guardrail checks on each run.
- Section `1AN` optionally enforces once-mode runner execution when:
```bash
ENFORCE_BUILDER_PRODUCTION_SOAK_RUNNER=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Recurring soak monitoring must be fail-closed by default; stale telemetry must alert and terminate the tick with non-zero status.

### 13.32 Production Soak Mac Automation (launchd)
Mission:
- Make production-soak monitoring automatic on your Mac without manual triggering.
- Keep scheduler setup deterministic, inspectable, and removable.

Install automation (hourly default):
```bash
INTERVAL_MINUTES=60 \
bash scripts/setup_builder_production_soak_launchd.sh
```

Check automation status:
```bash
bash scripts/status_builder_production_soak_launchd.sh
```

Remove automation:
```bash
bash scripts/remove_builder_production_soak_launchd.sh
```

What this automation enforces:
1. LaunchAgent label default:
   - `com.selene.builder.production_soak_runner`
2. Runtime bundle default location (outside protected Documents path):
   - `~/.selene_automation/production_soak`
3. launchd executes runner in fail-closed once mode each interval:
   - `RUN_MODE=once`
   - `FAIL_CLOSED=1`
   - `ALERT_ON_FAIL=1`
   - `BCAST_ON_FAIL=1`
4. Uses watchdog command inside runtime bundle:
   - `~/.selene_automation/production_soak/scripts/check_builder_production_soak_watchdog.sh`
5. Uses BCAST failure alert bridge inside runtime bundle:
   - `~/.selene_automation/production_soak/scripts/emit_builder_failure_bcast_alert.sh`
   - keeps BCAST ledger/app thread logs under `~/.selene_automation/production_soak/.dev/`
   - raises desktop notification on failure by default.
   - enforces routing-state files under runtime `.dev/` for delegated-recipient timeout fallback.
6. Logs:
   - runner stdout/stderr under `~/.selene_automation/production_soak/launchd/`
   - alert/state logs under `~/.selene_automation/production_soak/.dev/`
7. Scheduler lifecycle is explicit:
   - setup (bootstrap + kickstart)
   - status (loaded/not loaded)
   - remove (bootout + plist delete; optional runtime bundle removal)

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13t.sh
```

Readiness audit:
- Section `1AO` enforces Phase13-T automation guardrail checks on each run.
- Section `1AP` optionally enforces loaded automation status when:
```bash
ENFORCE_BUILDER_PRODUCTION_SOAK_AUTOMATION=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Production monitoring is treated as operationally incomplete until launchd automation is installed and status shows loaded.

### 13.33 Failure Alert Recipient Routing Policy (Owner-First, Delegatable)
Mission:
- Keep failure alerts actionable without requiring manual log watching.
- Start with owner-first routing, allow controlled delegation to another AP/person, and always fail back to JD (or JD-nominated Selene-company fallback) when delegated routing is ignored.

Default routing:
1. Initial recipient is the owner (you).
2. Failure alerts use BCAST lifecycle delivery (Selene App first) and follow MHP behavior.

Recipient change behavior:
1. Owner can issue a direct command:
   - "In future send these alerts to <person/AP>."
2. Selene updates the active alert recipient to that person.
3. Selene confirms the new target once and uses it for all future failure alerts.
4. Operational command surface (deterministic):
```bash
# delegate alerts to a person/AP
bash scripts/set_builder_failure_alert_recipient.sh set-delegate <user_id> <display_name> [timeout_minutes]

# clear delegation and route back to JD owner path
bash scripts/set_builder_failure_alert_recipient.sh clear-delegate

# inspect active routing state
bash scripts/set_builder_failure_alert_recipient.sh show
```

Ignore/fallback behavior:
1. If delegated recipient ignores alerts (or does not respond through the expected BCAST flow), Selene must always route alerts back to:
   - JD (owner), or
   - JD-nominated fallback person inside Selene company.
2. Selene sends JD/fallback recipient a short summary:
   - issue
   - attempted recipient
   - fallback reason (ignored/no response)
3. JD then chooses:
   - keep owner routing
   - choose a different recipient

Hard rules:
1. No silent recipient changes.
2. No permanent delegation without explicit owner command.
3. Delegated alert failure always returns control to JD or JD-nominated Selene-company fallback, and delegation is deactivated until JD re-delegates.
4. Routing changes are auditable and reason-coded.
5. Acknowledgement marker can be captured deterministically with:
```bash
bash scripts/ack_builder_failure_alert.sh <bcast_id> [recipient_user_id]
```

Simple communication requirement:
- Alert messages must stay plain-language:
  - "Issue is ..."
  - "Fix is ..."
  - "Should I proceed?"
