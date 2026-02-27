# PH1.C DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.C`
- `purpose`: Persist deterministic STT gate outcomes without inventing new PH1.C tables by wiring transcript acceptance to `conversation_ledger` and STT decisions/metadata to `audit_events`.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.conversation_ledger`
- `truth_type`: `LEDGER`
- `primary key`: `conversation_turn_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - optional FK `device_id -> devices.device_id`
  - optional FK `session_id -> sessions.session_id`
  - transcript commits use `source=VOICE_TRANSCRIPT`, `role=USER`
  - transcript status is represented deterministically:
    - accepted transcript -> `conversation_ledger` row present + `TranscriptOk` audit row
    - rejected transcript -> no `conversation_ledger` row + `TranscriptReject` audit row
  - idempotent append dedupe on `(correlation_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - STT gate outcomes are emitted as PH1.C audit rows (`TranscriptOk`, `TranscriptReject`, `SttCandidateEval`)
  - idempotent append dedupe on `(correlation_id, idempotency_key)` for this row scope
  - append-only; overwrite/delete prohibited

## 3) Reads (dependencies)

### Identity/device/session scope checks
- reads: `identities`, `devices`, `sessions`
- keys/joins used: direct FK existence lookups plus deterministic session scope check `(session.user_id, session.device_id)`
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
- scope rules:
  - device must belong to `user_id`
  - one tenant binding per `device_id` in PH1.C runtime scope
- why this read is required: fail closed before transcript/audit persistence

### Transcript/audit replay reads
- reads:
  - `conversation_ledger` by `correlation_id`
  - `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation threading
- required indices:
  - `conversation_ledger(correlation_id, turn_id)`
  - `audit_events(correlation_id, turn_id)`
- scope rules: no cross-tenant writes; tenant attribution is required on PH1.C audit rows
- why this read is required: deterministic replay and no duplicate transcript/audit rows on retries

## 4) Writes (outputs)

### Commit `transcript_ok`
- writes:
  - `conversation_ledger` (`VOICE_TRANSCRIPT` turn)
  - `audit_events` (`TranscriptOk` + `SttCandidateEval`)
- required fields:
  - transcript row: `created_at`, `correlation_id`, `turn_id`, `session_id?`, `user_id`, `device_id`, `text`, `text_hash`, `idempotency_key`
  - audit row: `tenant_id`, `engine=PH1.C`, `event_type`, `reason_code`, `correlation_id`, `turn_id`, payload, `evidence_ref?`
  - provider arbitration indicators (audit payload only): `route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`
  - PH1.K handoff indicators (audit payload only, when provided): `ph1k_handoff_interrupt_confidence_band`, `ph1k_handoff_vad_confidence_band`, `ph1k_handoff_quality_metrics_summary`, `ph1k_handoff_degradation_class_bundle`, `ph1k_selected_stt_strategy`
  - evidence span indicators (audit evidence_ref): `transcript_hash`, `critical_spans[]` with `start_byte`, `end_byte`, `field_hint?`
- idempotency_key rule (exact formula):
  - transcript dedupe key = `(correlation_id, idempotency_key)`
  - audit dedupe keys = `(correlation_id, idempotency_key + ":transcript_ok")` and `(correlation_id, idempotency_key + ":candidate_eval_ok")`
- failure reason codes (minimum examples):
  - `STT_FAIL_AUDIO_DEGRADED`
  - `STT_FAIL_LOW_CONFIDENCE`
  - `STT_FAIL_LANGUAGE_MISMATCH`
  - locale mismatch evaluation rule: use normalized locale-family matching (`en`, `en-US`, `en-GB` same family); enforce mismatch only when language family differs or when explicit script subtags conflict.
  - confidence calibration rule: confidence bucket selection must use calibrated score combining token signal + acoustic signal + context signal. Acoustic/context-missing paths must fail open to token baseline (not zeroed), while degraded acoustic signal must reduce calibrated score deterministically.

### Commit `transcript_reject`
- writes:
  - `audit_events` (`TranscriptReject` + `SttCandidateEval`)
- required fields:
  - reject row: `tenant_id`, `engine=PH1.C`, `event_type=TranscriptReject`, `reason_code`, `correlation_id`, `turn_id`, payload (`transcript_hash` optional)
  - candidate row: retry guidance metadata (`retry_advice`, `decision`) + arbitration indicators (`route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`)
  - PH1.K handoff indicators (audit payload only, when provided): `ph1k_handoff_interrupt_confidence_band`, `ph1k_handoff_vad_confidence_band`, `ph1k_handoff_quality_metrics_summary`, `ph1k_handoff_degradation_class_bundle`, `ph1k_selected_stt_strategy`
  - evidence span indicators (audit evidence_ref): `transcript_hash?`, `uncertain_spans[]` with byte offsets when available
- idempotency_key rule (exact formula):
  - audit dedupe keys = `(correlation_id, idempotency_key + ":transcript_reject")` and `(correlation_id, idempotency_key + ":candidate_eval_reject")`
- failure reason codes (minimum examples):
  - `STT_FAIL_EMPTY`
  - `STT_FAIL_LOW_COVERAGE`
  - `STT_FAIL_BUDGET_EXCEEDED`

## 5) Relations & Keys

FKs used by this slice:
- `conversation_ledger.user_id -> identities.user_id`
- `conversation_ledger.device_id -> devices.device_id` (nullable)
- `conversation_ledger.session_id -> sessions.session_id` (nullable)
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `conversation_idempotency_index(correlation_id, idempotency_key)`
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)`

State/boundary constraints:
- No PH1.C-owned current table in row 13 scope.
- No PH1.C migration is required for this slice.
- PH1.C persistence remains ledger-only over existing core tables.

## 6) Audit Emissions (PH1.J)

PH1.C writes emit PH1.J audit events with:
- `event_type`:
  - `TranscriptOk`
  - `TranscriptReject`
  - `SttCandidateEval`
- `reason_code(s)`:
  - pass/reject reason codes from PH1.C
  - deterministic candidate-eval reason codes for replay bucketing
- `payload_min` allowlisted discipline:
  - `TranscriptOk` / `TranscriptReject`: `transcript_hash` only
  - `SttCandidateEval`: bounded metadata (`decision`, `language_tag`, `confidence_bucket`, `retry_advice`, `route_class_used`, `attempt_count`, `candidate_count`, `selected_slot`, `mode_used`, `second_pass_used`)
  - `evidence_ref` bounded structure (when present): `transcript_hash`, `spans[]` (`start_byte`, `end_byte`, `field_hint?`)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-C-01` tenant isolation enforced
  - `at_c_db_01_tenant_isolation_enforced`
- `AT-C-02` append-only enforcement for transcript/audit ledgers
  - `at_c_db_02_append_only_enforced`
- `AT-C-03` idempotency dedupe works
  - `at_c_db_03_idempotency_dedupe_works`
- `AT-C-04` no PH1.C current-table rebuild is required
  - `at_c_db_04_no_current_table_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 13 (`PH1.C` uses existing `conversation_ledger` + `audit_events`)
- tests: `crates/selene_storage/tests/ph1_c/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.ENDPOINT`)

- Before PH1.C transcript finalization, Selene OS may invoke PH1.ENDPOINT and pass back one selected endpoint hint.
- PH1.C remains authoritative for transcript pass/reject and reason codes; PH1.ENDPOINT hints are advisory only.
- PH1.C audit payloads may include endpoint-hint references only as bounded metadata and must remain provider-invisible.

## 9) Related Engine Boundary (`PH1.KNOW`)

- Selene OS may provide tenant-scoped PH1.KNOW vocabulary hints to PH1.C before transcript quality gating.
- PH1.C remains transcript-gate authority and must treat PH1.KNOW hints as advisory only.
- PH1.KNOW hints must remain tenant-scoped, authorized-only, and provider-invisible in PH1.C audit payloads.
- PH1.C live STT provider calls may include normalized lexicon hint payloads (`tenant_vocabulary_pack_id`, `user_vocabulary_pack_id`, `tenant_lexicon_terms[]`, `domain_lexicon_terms[]`) sent through PH1.D boundary only.
- Lexicon boosts in PH1.C acceptance logic must be bounded and deterministic; zero hints means zero boost.

## 10) Related Engine Boundary (`PH1.QUOTA`)

- Selene OS may apply PH1.QUOTA lane decisions (`ALLOW | WAIT | REFUSE`) before PH1.C execution.
- PH1.C must fail closed when quota returns `REFUSE` and must not execute hidden fallback paths.
- `WAIT` posture is an OS orchestration pause only; PH1.C transcript authority and audit discipline remain unchanged when resumed.

## 11) Related Engine Boundary (`PH1.K`)

- Selene OS may pass optional `ph1k_handoff` into PH1.C request contract with:
  - `interrupt_confidence_band`
  - `vad_confidence_band`
  - `quality_metrics` summary
  - `degradation_class_bundle`
- PH1.C must select bounded STT strategy from this handoff only:
  - `STANDARD`
  - `NOISE_ROBUST`
  - `CLOUD_ASSIST`
  - `CLARIFY_ONLY`
- Unknown/missing handoff fields must fail closed at contract-validation level.
- Selene OS wiring strict handoff mode (`require_ph1k_handoff=true`) rejects missing `ph1k_handoff` with fail-closed PH1.C refusal before runtime execution.

## 12) FDX Wiring Lock (Section 5F)

- PH1.C wiring must support ordered partial transcript emission metadata in duplex flows (`stable`, `revision_id`, confidence band).
- Canonical contract surfaces:
  - `PartialTranscript`
  - `PartialTranscriptBatch`
- Canonical runtime canonicalization boundary:
  - `Ph1cRuntime::canonicalize_partial_transcripts(...)`
- Deterministic invariants:
  - revisions are strictly ordered and contiguous from `1`
  - duplicate revisions resolve deterministically (stable-first, then higher confidence)
  - finalized batches require `stable=true` on the last revision
- Transcript quality failures in duplex mode must remain fail-closed and reason-coded.
- PH1.C must persist enough bounded timing evidence to audit FDX latency gates (`capture -> PH1.C handoff`, partial-first-chunk latency).

## 13) STT Provider Ladder Lock (Round-2 Step 4)

- PH1.C STT execution ladder is strict and deterministic:
  - `PRIMARY (OpenAI)` -> `SECONDARY (Google)` -> `clarify/reject`
- PH1.C must not reorder providers by strategy class in round-2.
- PH1.C retries are bounded by config:
  - `max_retries_per_provider`
  - `max_attempts_per_turn`
  - `max_total_latency_budget_ms`
- PH1.C must reject medium/low confidence transcript candidates with `STT_FAIL_LOW_CONFIDENCE`; low-confidence text must not be committed as `TranscriptOk`.
- If retry or total budget is exceeded, PH1.C must fail closed with bounded reason code (`STT_FAIL_BUDGET_EXCEEDED`).
- Provider identities remain upstream-invisible:
  - PH1.C output contracts expose slot class only (`PRIMARY|SECONDARY`) and must not expose vendor/model ids.
- Coupled TTS routing lock is owned by PH1.TTS (Round-2 Step 5):
  - PH1.C must not execute or bypass any TTS provider path directly.

## 14) Cross-Engine Handoff Integrity (`PH1.C -> PH1.NLP`) (Round-2 Step 6)

- PH1.C output passed into PH1.NLP must carry a valid `TranscriptOk` contract with:
  - matching contract schema version
  - valid language tag
  - bounded uncertain-span offsets (UTF-8 safe, transcript-bounded)
- Selene OS strict handoff mode (`require_ph1c_handoff=true`) requires `transcript_ok.audit_meta` on PH1.NLP requests.
- Minimum PH1.C audit metadata required for PH1.NLP handoff acceptance:
  - `attempt_count > 0`
  - `selected_slot != NONE`
- Missing/malformed handoff metadata must fail closed before PH1.NLP runtime execution with deterministic reason code (`PH1_NLP_HANDOFF_INVALID`).

## 14A) Intent-Aware Transcript Repair Lock (5H Step 8)

- PH1.C now runs a bounded repair branch before final transcript pass/reject when transcript shape is rambling/broken/scrambled.
- Repair branch is strict and deterministic:
  - run PH1.SRL `SrlFrameBuild` then `SrlArgumentNormalize`
  - reject repair if SRL returns ambiguity (`clarify_required=true`) or validation failure
  - apply bounded disfluency collapse (filler removal + duplicate-run compression)
  - require lexical-overlap safety threshold between original and repaired transcript
  - require PH1.NLP acceptance guard (no quality regression, no intent drift)
- Repair acceptance does not bypass PH1.C gate authority:
  - repaired transcript still must pass PH1.C language, coverage, and calibrated confidence gates
  - if any repair guard fails, PH1.C falls back to original fail-closed reason path (no guess commit)
- Audit/persistence posture remains unchanged:
  - PH1.C remains transcript authority
  - provider/model internals remain upstream-invisible
  - no new PH1.C-owned tables introduced for this step

## 14B) Clarify Precision Lock (5H Step 9)

- Cross-engine clarify behavior is now hard-locked to:
  - exactly one precise clarify question for an unresolved turn, then
  - execute on valid follow-up input, or escalate if still unresolved.
- Runtime ownership:
  - PH1.NLP remains clarify-owner for missing/ambiguous fields.
  - PH1.X enforces no-loop execution policy by refusing repeated clarify chains.
- Escalation discipline:
  - second clarify attempt in the same pending clarify chain must emit escalation response, not another clarify question.
  - escalation path remains fail-closed and non-executing until fresh disambiguating input arrives.

## 14C) Gold-Loop Lock (5H Step 10)

- PH1.C miss/correction events are captured as PH1.FEEDBACK improvement-path events with deterministic gold-case ids/fingerprints.
- Capture-time rule remains strict:
  - miss/correction gold events are `gold_status=PENDING` until verification evidence exists.
- LEARN packaging rule remains strict:
  - improvement-path artifacts must be gold-verified (`gold_status=VERIFIED` + provenance method), else LEARN package-build fails closed.
- Verified miss/correction events must route deterministically through:
  - `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE`
- Replay/idempotency lock:
  - repeated execution over identical verified inputs must preserve stable self-heal ids (`failure fingerprint`, `problem_id`, `fix_id`, `decision_id`).
- Authority boundary is unchanged:
  - FEEDBACK/LEARN/PAE outputs remain advisory-only and no-execution-authority.

## 14D) In-House Shadow Route Lock (5H Step 11)

- PH1.C now exposes an explicit in-house STT shadow-compare surface:
  - `Ph1cRuntime::evaluate_inhouse_shadow_route(...)`
- Mandatory shadow inputs:
  - provider truth attempt (must pass PH1.C quality gate)
  - in-house candidate attempt
  - deterministic slice key: `locale`, `device_route`, `tenant_id`
  - governed gate proof flag (`governed_gate_passed`)
- Fail-closed guarantees:
  - invalid slice keys or locale-family mismatches fail closed
  - invalid provider-truth candidates fail closed
  - shadow mode holds by default when governed gate proof is absent
- Promotion eligibility is bounded and deterministic:
  - requires governed gate proof
  - requires high transcript overlap and bounded confidence/latency deltas vs provider truth
  - any drift keeps route in `HOLD_SHADOW` (no authority promotion)
- Authority boundary remains unchanged:
  - PH1.C shadow compare is advisory-only and does not self-promote runtime authority.

## 15) Gold-Case Capture Wiring (Round-2 Step 8)

- Selene OS now emits deterministic `GoldCaseCapture` envelopes from PH1.C outcomes through PH1.FEEDBACK wiring (`crates/selene_os/src/ph1feedback.rs`).
- PH1.C trigger set for gold-case candidate emission:
  - `TranscriptReject`
  - low-confidence or uncertain-span `TranscriptOk`
- Each capture includes:
  - pending `gold_case_id`
  - bounded `reason_code_chain`
  - deterministic clustering keys (`primary_failure_fingerprint`, `secondary_failure_fingerprint`)
  - owner marker `PH1.C`
- PH1.C sourced captures are fail-closed validated and converted into PH1.FEEDBACK improvement-path events with bounded metrics and deterministic idempotency keys.

## 16) Benchmark/Eval Harness Lock (Round-2 Step 11)

- Canonical PH1.C round-2 eval snapshot path:
  - `docs/fixtures/ph1c_round2_eval_snapshot.csv`
- Canonical harness scripts:
  - `scripts/check_ph1c_round2_eval_snapshot.sh`
  - `scripts/check_ph1c_round2_eval_gates.sh`
- Snapshot harness fail-closed checks:
  - strict CSV schema and formula consistency (`stt_accept + stt_reject == stt_turns`, fallback-resolution equality, provider-schema-valid bounds)
  - coverage completeness across locale, tenant, device route, noise class, and overlap flags
  - bounded metric ranges for latency, cost, audit, isolation, and quality-eval counts
- Category gate coverage (must emit and gate):
  - quality acceptance + provider-schema-valid rates
  - latency (`partial_first_chunk_p95_ms`, `eos_to_first_token_p95_ms`, `capture_to_ph1c_handoff_p95_ms`)
  - fallback success continuity
  - cost per turn (`stt_cost_microunits_per_turn`, `tts_cost_microunits_per_turn`)
  - audit completeness + tenant isolation
  - multilingual/code-switch quality
  - rambling-to-structured quality
  - broken-English normalization quality
  - accent robustness quality
  - scrambled-speech clarify recovery quality
- Readiness audit integration:
  - `scripts/selene_design_readiness_audit.sh` sections `1C7` and `1C8`.

## 17) Builder Remediation Wiring Lock (Round-2 Step 12)

- PH1.C recurring unresolved failure clusters now map deterministically into PH1.BUILDER intake through PH1.OS:
  - mapper: `map_recurring_failure_cluster_to_builder_offline_input(...)` in `crates/selene_os/src/ph1os.rs`
  - eligibility gate: `problem_card.recurrence_count >= 3` and `problem_card.state != RESOLVED`
- 5H linkage lock:
  - unresolved PH1.C/PH1.D shadow-route comparison regressions (Step 11) must enter the same remediation governance path; no bypass route is allowed.
- Promotion fail-closed requirement:
  - PH1.OS now enforces `check_builder_remediation_promotion_gate(...)`.
  - For promotion actions, all three proofs are mandatory:
    - `code_permission_gate_passed=true`
    - `launch_permission_gate_passed=true`
    - `release_hard_gate_passed=true`
- Deterministic builder handoff payload:
  - fixed outcome entries include `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.PAE`, `PH1.OS` for clustering and audit continuity.
  - bounded idempotency keys/hashes are generated from card-chain identifiers.
- Readiness audit integration:
  - `scripts/selene_design_readiness_audit.sh` section `1C9`
  - `scripts/check_ph1c_round2_builder_remediation_gate.sh`

## 18) Acceptance Test Expansion Lock (Round-2 Step 13)

- PH1.C Step-13 deterministic acceptance tests now lock:
  - primary provider success path (OpenAI/primary slot selected)
  - fallback path on primary failure (Google/secondary slot selected)
  - terminal fail-closed reject when both slots fail (no guessed transcript)
  - partial transcript revision canonicalization correctness
- Cross-boundary acceptance locks included in Step 13:
  - PH1.D provider schema-drift fail-closed behavior
  - PH1.FEEDBACK gold-case creation on correction + escalation mapping
- Canonical Step-13 gate script:
  - `scripts/check_ph1c_round2_acceptance_tests.sh`
- Readiness audit integration:
  - `scripts/selene_design_readiness_audit.sh` section `1C10`

## 19) Release Threshold Gate Lock (Round-2 Step 14)

- Canonical strict release gate script:
  - `scripts/check_ph1c_round2_release_gate.sh`
- Gate inputs:
  - PH1.C eval snapshot: `docs/fixtures/ph1c_round2_eval_snapshot.csv`
  - PH1.K eval snapshot (cross-engine interrupt proof): `docs/fixtures/ph1k_round2_eval_snapshot.csv`
- Enforced fail-closed thresholds:
  - STT fallback continuity success `>= 99.90%`
  - provider schema-valid response rate `>= 99.50%`
  - partial transcript first chunk latency `p95 <= 250ms`
  - end-of-speech to first response token `p95 <= 300ms`
  - false interrupt rate `<= 0.3/hour` (from PH1.K snapshot)
  - missed interrupt rate `<= 2%` (from PH1.K snapshot)
  - audit completeness `= 100%`
  - tenant isolation `= 100%`
  - multilingual transcript acceptance `>= 95%`
  - heavy-accent transcript acceptance `>= 93%`
  - broken-English normalization acceptance `>= 90%`
  - rambling/scrambled clarify-to-resolution within 2 turns `>= 90%`
- Readiness audit integration:
  - `scripts/selene_design_readiness_audit.sh` section `1C11`
