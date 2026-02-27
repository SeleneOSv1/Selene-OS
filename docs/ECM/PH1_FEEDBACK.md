# PH1_FEEDBACK ECM (Design vNext)

## Engine Header
- engine_id: PH1.FEEDBACK
- role: Structured correction/confidence feedback capture and deterministic signal emission
- placement: TURN_OPTIONAL

## Capability List

### capability_id: FEEDBACK_EVENT_COLLECT
- input_schema:
  - bounded envelope (`correlation_id`, `turn_id`, `max_events`, `max_signals`)
  - bounded feedback events (`tenant_id`, `user_id`, `speaker_id`, `session_id`, `device_id`, `event_type`, `reason_code`, `evidence_ref`, metrics)
- output_schema:
  - one selected signal candidate id
  - deterministic ordered signal candidates (`signal_key`, target, value, sample_count, evidence_ref)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: INPUT_SCHEMA_INVALID, UPSTREAM_INPUT_MISSING, BUDGET_EXCEEDED, INTERNAL_PIPELINE_ERROR
- reason_codes:
  - PH1_FEEDBACK_OK_EVENT_COLLECT
  - PH1_FEEDBACK_INPUT_SCHEMA_INVALID
  - PH1_FEEDBACK_UPSTREAM_INPUT_MISSING
  - PH1_FEEDBACK_BUDGET_EXCEEDED
  - PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR

### capability_id: FEEDBACK_SIGNAL_EMIT
- input_schema:
  - selected candidate id + ordered candidates from `FEEDBACK_EVENT_COLLECT`
  - same envelope bounds for deterministic replay
- output_schema:
  - validation_result (`OK|FAIL`)
  - bounded diagnostics
  - emit-target flags (`emits_learn`, `emits_pae`)
  - boundary flags: `advisory_only=true`, `no_execution_authority=true`
- allowed_callers: SELENE_OS_ONLY
- side_effects: NONE
- failure_modes: VALIDATION_FAILED, INPUT_SCHEMA_INVALID, BUDGET_EXCEEDED
- reason_codes:
  - PH1_FEEDBACK_OK_SIGNAL_EMIT
  - PH1_FEEDBACK_VALIDATION_FAILED
  - PH1_FEEDBACK_INPUT_SCHEMA_INVALID
  - PH1_FEEDBACK_BUDGET_EXCEEDED

## Constraints
- Engines never call engines directly; Selene OS orchestrates all sequencing.
- Non-authoritative assist engine; outputs are advisory only.
- No execution path and no authority mutation.
- Hard rule: FEEDBACK may emit learning signals only; it must never mutate runtime state or permissions.

## Related Engine Boundaries
- PH1.LEARN and PH1.PAE consume FEEDBACK outputs only through Selene OS curated bundles.
- PH1.PAE consumption boundary is `PAE_POLICY_SCORE_BUILD` input only; FEEDBACK never influences `PAE_ADAPTATION_HINT_EMIT` directly.
- PH1.LEARN capability contracts are defined in `docs/DB_WIRING/PH1_LEARN.md` and `docs/ECM/PH1_LEARN.md`.
- FEEDBACK storage/artifact append-only contracts remain canonical in `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md`.

## FDX Design Lock (Section 5F)
- PH1.FEEDBACK must capture duplex-specific failures and corrections as bounded signals.
- Minimum FDX event coverage includes:
  - false interrupt
  - missed interrupt
  - late cancel
- low-confidence transcript fallback
- clarify-after-duplex uncertainty
- PH1.FEEDBACK outputs remain advisory and must flow only through Selene OS into PH1.LEARN/PH1.PAE.

## Round-2 Step 9 Lock (Feedback -> Learn Routing)
- Selene OS runtime now provides deterministic FEEDBACK->LEARN route mapping in `crates/selene_os/src/ph1learn.rs`:
  - `map_feedback_bundle_to_learn_turn_input(...)`
  - `route_feedback_into_learn_wiring(...)`
- Mapping preserves FEEDBACK taxonomy and gold metadata into LEARN signals:
  - `path_type -> source_path`
  - `gold_case_id/gold_status -> learn gold fields`
- Deterministic replay guarantees:
  - canonical candidate ordering
  - stable deterministic learn `signal_id` generation
  - fail-closed on correlation/turn/tenant mismatch

## Round-2 Step 10 Lock (Gold-Loop Miss/Correction Flow)
- PH1.C miss/correction gold captures are represented as PH1.FEEDBACK improvement events with deterministic ids/fingerprints.
- Event-target lock:
  - STT miss/retry classes (`SttReject`, `SttRetry`) must emit `PaeScorecard` targets for downstream PAE learning path eligibility.
- Gold verification lock:
  - PH1.FEEDBACK may carry pending gold events, but improvement-package promotion downstream requires verified gold state and provenance.
- Replay lock:
  - repeated verified FEEDBACK inputs must produce deterministic FEEDBACK bundles for LEARN/PAE.
