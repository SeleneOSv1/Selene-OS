# PH1.C ECM Spec

## Engine Header
- `engine_id`: `PH1.C`
- `purpose`: Persist deterministic STT transcript gate outcomes (`transcript_ok` / `transcript_reject`) through conversation/audit ledgers.
- `data_owned`: `conversation_ledger` writes in PH1.C scope, `audit_events` writes in PH1.C scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1C_TRANSCRIPT_OK_COMMIT_ROW`
- `name`: Commit accepted transcript turn plus audit events
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, transcript_text, transcript_hash, language_tag, confidence_bucket, route_class_used, attempt_count, candidate_count, selected_slot, mode_used, second_pass_used, ph1k_handoff_interrupt_confidence_band?, ph1k_handoff_vad_confidence_band?, ph1k_handoff_quality_metrics_summary?, ph1k_handoff_degradation_class_bundle?, ph1k_selected_stt_strategy?, critical_spans?, idempotency_key)`
- `output_schema`: `Result<Ph1cTranscriptOkCommitResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1C_TRANSCRIPT_REJECT_COMMIT_ROW`
- `name`: Commit transcript rejection audit events
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, reject_reason_code, retry_advice, transcript_hash?, route_class_used, attempt_count, candidate_count, selected_slot, mode_used, second_pass_used, ph1k_handoff_interrupt_confidence_band?, ph1k_handoff_vad_confidence_band?, ph1k_handoff_quality_metrics_summary?, ph1k_handoff_degradation_class_bundle?, ph1k_selected_stt_strategy?, uncertain_spans?, idempotency_key)`
- `output_schema`: `Result<Ph1cTranscriptRejectCommitResult, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1C_READ_VOICE_TRANSCRIPT_ROWS`
- `name`: Read voice transcript ledger rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `ConversationTurnRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- reject/fail-closed reason codes include:
  - `STT_FAIL_EMPTY`
  - `STT_FAIL_LOW_CONFIDENCE`
  - `STT_FAIL_LOW_COVERAGE`
  - `STT_FAIL_LANGUAGE_MISMATCH`
  - `STT_FAIL_AUDIO_DEGRADED`
  - `STT_FAIL_BUDGET_EXCEEDED`
- locale mismatch gate must be family-normalized:
  - canonicalized tags (`_` -> `-`, lower-case) and language-family equivalence (`en`, `en-US`, `en-GB`) are treated as matching.
  - explicit script conflicts (for example `zh-Hans` vs `zh-Hant`) must fail closed as `STT_FAIL_LANGUAGE_MISMATCH`.
- confidence gate must be calibrated (not heuristic-only):
  - combine token signal (`avg_word_confidence`, `low_confidence_ratio`), acoustic signal (`noise_level_hint`, `vad_quality_hint`, PH1.K advanced metrics), and context signal (language-hint consistency).
  - missing acoustic/context signals must fall back to token baseline; degraded acoustic metrics must lower calibrated confidence deterministically.
- scope/contract failures are fail-closed with deterministic PH1.C reason coding.

## Audit Emission Requirements Per Capability
- `PH1C_TRANSCRIPT_OK_COMMIT_ROW` must emit PH1.J audit rows including:
  - `TranscriptOk`
  - `SttCandidateEval`
- `PH1C_TRANSCRIPT_REJECT_COMMIT_ROW` must emit PH1.J audit rows including:
  - `TranscriptReject`
  - `SttCandidateEval`
- `payload_min` remains bounded and provider-invisible while preserving arbitration indicators (`route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`).
- when present, PH1.K handoff fields must be bounded and auditable: `ph1k_handoff_interrupt_confidence_band`, `ph1k_handoff_vad_confidence_band`, `ph1k_handoff_quality_metrics_summary`, `ph1k_handoff_degradation_class_bundle`, `ph1k_selected_stt_strategy`.
- `evidence_ref` (when present) must carry bounded transcript span references only.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1cSttRepo`)
- `docs/DB_WIRING/PH1_C.md`

## Related Engine Boundary (`PH1.ENDPOINT`)
- PH1.C may consume one Selene OS-curated endpoint hint from PH1.ENDPOINT before transcript finalization.
- PH1.C must not treat PH1.ENDPOINT output as transcript authority; PH1.C remains the transcript gate owner.

## Related Engine Boundary (`PH1.KNOW`)
- PH1.C may consume one Selene OS-curated PH1.KNOW vocabulary hint bundle before transcript finalization.
- PH1.C must not treat PH1.KNOW output as transcript authority; PH1.C remains the transcript gate owner.
- PH1.KNOW-derived hints must remain tenant-scoped and authorized-only in PH1.C capability execution.
- PH1.C may forward normalized vocabulary hints to PH1.D STT provider requests (`tenant/user vocabulary pack ids`, `tenant/domain lexicon terms`) as bounded request metadata.
- PH1.C acceptance scoring may apply bounded lexicon-match confidence boost for matched domain terms; no hint input means no boost.

## Related Engine Boundary (`PH1.QUOTA`)
- PH1.C capability execution may be pre-gated by Selene OS using `PH1.QUOTA` lane decisions.
- If quota posture is `REFUSE`, PH1.C capability calls must not run.
- If quota posture is `WAIT`, Selene OS may pause before PH1.C; PH1.C output contracts remain unchanged when resumed.

## Related Engine Boundary (`PH1.K`)
- PH1.C accepts optional `ph1k_handoff` contract payload carrying confidence bands, quality-metrics summary, and degradation class bundle.
- PH1.C must map handoff to bounded strategy classes only: `STANDARD | NOISE_ROBUST | CLOUD_ASSIST | CLARIFY_ONLY`.
- Absent handoff defaults deterministically to `STANDARD`; malformed handoff fails closed at contract validation.

## FDX Design Lock (Section 5F)
- PH1.C owns partial transcript streaming quality gates in duplex sessions.
- PH1.C must emit bounded `PartialTranscript` units (`text_chunk`, `confidence`, `stable`, `revision_id`) in deterministic order.
- PH1.C contract surface for this is:
  - `PartialTranscript`
  - `PartialTranscriptBatch`
- PH1.C runtime canonicalization path is:
  - `Ph1cRuntime::canonicalize_partial_transcripts(...)`
- Finalization semantics are strict:
  - finalized stream requires last revision `stable=true`
  - out-of-order or non-contiguous revisions fail closed
- PH1.C remains transcript authority; low-confidence or malformed partial/final transcript paths must fail closed to clarify-ready posture.
- PH1.C must accept PH1.K handoff posture but must not leak provider internals or bypass transcript quality gates.
- PH1.C must preserve release targets for FDX path:
  - partial transcript first chunk p95 <= 250ms
  - capture -> PH1.C partial handoff p95 <= 120ms

## STT Provider Ladder Lock (Round-2 Step 4)
- PH1.C executes STT in one strict route order for this round:
  - `OpenAI (PRIMARY)` then `Google (SECONDARY)` then fail-closed clarify/reject.
- PH1.C strategy classification (`STANDARD|NOISE_ROBUST|CLOUD_ASSIST`) must not reorder provider ladder in round-2.
- Retry and budget control is mandatory and bounded:
  - `max_retries_per_provider`
  - `max_attempts_per_turn`
  - `max_total_latency_budget_ms`
- PH1.C must fail closed (`STT_FAIL_LOW_CONFIDENCE`) for medium/low confidence transcript candidates; low-confidence text must not pass as `TranscriptOk`.
- On budget breach PH1.C returns fail-closed rejection (`STT_FAIL_BUDGET_EXCEEDED`).
- Provider invisibility is mandatory:
  - no vendor/model identifiers are allowed in PH1.C upstream response contracts.
- Coupled TTS provider ladder is owned by PH1.TTS Round-2 Step 5; PH1.C must not issue direct TTS provider calls.

## Cross-Engine Handoff Integrity (`PH1.C -> PH1.NLP`) (Round-2 Step 6)
- PH1.C transcript output forwarded to PH1.NLP must remain schema-valid and bounded (`TranscriptOk` contract, language tag, uncertain-span offsets).
- Selene OS strict NLP handoff mode (`require_ph1c_handoff=true`) requires PH1.C audit metadata to be present on the transcript envelope.
- Required PH1.C handoff metadata minimum:
  - `attempt_count > 0`
  - `selected_slot != NONE`
- Missing/malformed PH1.C handoff metadata must fail closed before PH1.NLP runtime invocation with deterministic reason code (`PH1_NLP_HANDOFF_INVALID`).

## Intent-Aware Transcript Repair Lock (5H Step 8)
- PH1.C now includes a bounded transcript-repair branch for rambling/broken/scrambled STT candidates before final pass/reject.
- Locked sequence:
  - PH1.C calls PH1.SRL frame-build + argument-normalize on repair candidates.
  - Repair is rejected fail-closed if SRL reports ambiguity (`clarify_required`) or validation failure.
  - PH1.C applies bounded disfluency cleanup (filler suppression + duplicate-run compression).
  - PH1.C applies lexical-overlap safety check and PH1.NLP guard to block meaning drift.
- Acceptance boundary:
  - repaired output is accepted only if PH1.NLP quality is non-regressive and intent class is stable
  - accepted repaired text must still pass normal PH1.C language/coverage/confidence gates
  - any guard failure returns original PH1.C fail-closed behavior (no guessed commit)

## Clarify Precision Lock (5H Step 9)
- Clarify discipline is now strict end-to-end:
  - one precise clarify question per unresolved chain
  - next step must be either execution (if resolved) or escalation (if still unresolved)
  - repeated clarify loops are blocked
- PH1.NLP remains clarify-owner, PH1.X enforces loop-prevention runtime policy.
- On second clarify attempt in the same pending clarify chain, PH1.X emits escalation response (`X_CLARIFY_ESCALATED`) instead of another clarify directive.

## Gold-Loop Lock (5H Step 10)
- PH1.C miss/correction outputs must be represented as PH1.FEEDBACK improvement-path events with deterministic gold-case ids and clustering fingerprints.
- Gold status lifecycle is strict:
  - capture emits `gold_status=PENDING`
  - LEARN package-build for improvement artifacts requires verified gold (`gold_status=VERIFIED` + provenance), otherwise fail closed.
- Verified miss/correction events must flow deterministically through `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE`.
- Replay over identical verified inputs must keep stable deterministic self-heal ids (`failure fingerprint`, `problem_id`, `fix_id`, `decision_id`).
- Authority boundaries remain unchanged: advisory-only and no execution authority across FEEDBACK/LEARN/PAE.

## In-House Shadow Route Lock (5H Step 11)
- PH1.C now includes explicit in-house shadow comparison capability:
  - `Ph1cRuntime::evaluate_inhouse_shadow_route(...)`
- Required compare inputs:
  - provider truth attempt (must be PH1.C-valid)
  - in-house attempt
  - deterministic slice key (`locale`, `device_route`, `tenant_id`)
  - governed gate proof (`governed_gate_passed`)
- Promotion discipline:
  - default decision is `HOLD_SHADOW`
  - promotion eligibility is granted only when governed gate proof exists and parity thresholds pass (overlap/confidence/latency)
  - any mismatch or drift blocks promotion fail-closed
- PH1.C remains transcript authority; shadow comparison is advisory-only and cannot self-promote route authority.

## Gold-Case Capture Wiring (Round-2 Step 8)
- Selene OS now emits deterministic `GoldCaseCapture` envelopes for PH1.C outcomes through PH1.FEEDBACK wiring (`crates/selene_os/src/ph1feedback.rs`).
- PH1.C trigger set:
  - `TranscriptReject`
  - low-confidence or uncertain-span `TranscriptOk`
- Each PH1.C capture includes:
  - pending `gold_case_id`
  - bounded `reason_code_chain`
  - deterministic clustering keys (`primary_failure_fingerprint`, `secondary_failure_fingerprint`)
  - owner marker `PH1.C`
- PH1.C sourced captures are fail-closed validated and represented as PH1.FEEDBACK improvement-path events (no direct runtime authority).

## Benchmark/Eval Harness Lock (Round-2 Step 11)
- Canonical eval fixture:
  - `docs/fixtures/ph1c_round2_eval_snapshot.csv`
- Canonical benchmark gates:
  - `check_ph1c_round2_eval_snapshot.sh` (shape/coverage/formula fail-closed harness)
  - `check_ph1c_round2_eval_gates.sh` (category gate enforcement)
- Mandatory category outputs covered by gates:
  - transcript quality acceptance + provider-schema-valid rates
  - latency (`partial_first_chunk_p95_ms`, `eos_to_first_token_p95_ms`, `capture_to_ph1c_handoff_p95_ms`)
  - fallback continuity success
  - STT/TTS cost-per-turn
  - audit completeness + tenant isolation
  - multilingual and code-switch quality
  - rambling-to-structured quality
  - broken-English normalization quality
  - accent robustness quality
  - scrambled speech clarify-recovery quality
- Global readiness wiring:
  - `scripts/selene_design_readiness_audit.sh` sections `1C7` and `1C8` execute the PH1.C eval harness/gates on every audit run.

## Builder Remediation Wiring Lock (Round-2 Step 12)
- PH1.C recurring unresolved failure clusters are now mapped into PH1.BUILDER proposal intake by PH1.OS.
- 5H linkage:
  - unresolved PH1.C/PH1.D shadow-route regressions are required to use this same Builder governance path (no alternate auto-fix bypass).
- Entry contract lock:
  - `map_recurring_failure_cluster_to_builder_offline_input(...)` in `crates/selene_os/src/ph1os.rs`
  - recurrence threshold: `problem_card.recurrence_count >= 3`
  - resolved clusters are excluded (`problem_card.state != RESOLVED`)
- Promotion safety lock:
  - `check_builder_remediation_promotion_gate(...)` is fail-closed for promotion actions.
  - required proofs for promotion:
    - code permission gate pass
    - launch permission gate pass
    - release hard-gate pass
- Readiness wiring:
  - `scripts/check_ph1c_round2_builder_remediation_gate.sh`
  - `scripts/selene_design_readiness_audit.sh` section `1C9`

## Acceptance Test Expansion Lock (Round-2 Step 13)
- Deterministic acceptance lock now includes:
  - OpenAI(primary) success path in PH1.C
  - Google(secondary) fallback when primary fails
  - terminal fail-closed reject when both slots fail
  - partial transcript revision canonicalization correctness
  - PH1.D provider schema-drift fail-closed
  - PH1.FEEDBACK gold-case creation on correction + escalation mapping
- Canonical gate script:
  - `scripts/check_ph1c_round2_acceptance_tests.sh`
- Global readiness wiring:
  - `scripts/selene_design_readiness_audit.sh` section `1C10`

## Release Threshold Gate Lock (Round-2 Step 14)
- Strict release gate script:
  - `scripts/check_ph1c_round2_release_gate.sh`
- Required inputs:
  - `docs/fixtures/ph1c_round2_eval_snapshot.csv`
  - `docs/fixtures/ph1k_round2_eval_snapshot.csv`
- Locked thresholds enforced fail-closed:
  - STT fallback continuity `>= 99.90%`
  - provider schema-valid response rate `>= 99.50%`
  - partial first chunk p95 `<= 250ms`
  - end-of-speech to first response token p95 `<= 300ms`
  - false interrupt rate `<= 0.3/hour`
  - missed interrupt rate `<= 2%`
  - audit completeness `= 100%`
  - tenant isolation `= 100%`
  - multilingual transcript acceptance `>= 95%`
  - heavy-accent acceptance `>= 93%`
  - broken-English normalization `>= 90%`
  - rambling/scrambled clarify-to-resolution within 2 turns `>= 90%`
- Global readiness wiring:
  - `scripts/selene_design_readiness_audit.sh` section `1C11`
