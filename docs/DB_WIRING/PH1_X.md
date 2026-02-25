# PH1.X DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.X`
- `purpose`: Persist deterministic PH1.X conversational directives (`confirm`, `clarify`, `respond`, `dispatch`, `wait`) as bounded audit events without introducing PH1.X-owned tables, including interruption continuity branch metadata.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.X outcomes are recorded with `engine=PH1.X`
  - `confirm` uses `event_type=XConfirm`
  - `dispatch` uses `event_type=XDispatch`
  - `clarify/respond/wait` use `event_type=Other` with bounded directive payload keys
  - payload values are bounded and reason-coded
  - idempotent append dedupe on `(correlation_id, idempotency_key)`
  - append-only; overwrite/delete prohibited

## 3) Reads (dependencies)

### Identity/device/session scope checks
- reads: `identities`, `devices`, `sessions`
- keys/joins used: direct FK existence + deterministic scope check `(session.user_id, session.device_id)`
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
  - `sessions(session_id)` (PK)
- scope rules:
  - device must belong to `user_id`
  - one tenant binding per `device_id` for PH1.X rows
- why this read is required: fail closed before PH1.X audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

### WorkOrder + lease scope checks (dispatch gating references)
- reads:
  - `work_orders_current` by `(tenant_id, work_order_id)` for current status
  - `work_order_leases` by `(tenant_id, work_order_id)` for active lease ownership
- keys/joins used:
  - deterministic key lookup on `work_orders_current(tenant_id, work_order_id)`
  - active lease filter `lease_state='ACTIVE'` with latest `lease_expires_at`
- required indices:
  - `ux_work_orders_current_tenant_work_order`
  - `ux_work_order_leases_tenant_work_order_idempotency` (tenant/work-order key path)
- scope rules:
  - PH1.X reads only in-tenant work-order rows bound to the current `correlation_id`
  - PH1.X does not mutate work-order/lease tables; writes remain owned by `SELENE_OS_CORE_TABLES`
- why this read is required:
  - deterministic clarify/confirm/dispatch gating against current WorkOrder status
  - deterministic no-dispatch rule when lease is missing/expired

## 4) Writes (outputs)

### Commit `confirm`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=XConfirm`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=confirm`, `confirm_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `clarify`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=clarify`, `what_is_missing`, `clarification_unit_id`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `respond`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=respond`, `response_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `dispatch`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=XDispatch`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=dispatch`, `dispatch_target`, `work_order_id`, `work_order_status_snapshot`, `pending_state`, `lease_token_hash?`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

### Commit `wait`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.X`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload: `directive=wait`, `wait_kind`, `work_order_id`, `work_order_status_snapshot`, `pending_state`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.X-owned current table in row 16 scope.
- No PH1.X migration is required for this slice.
- PH1.X remains non-authoritative; storage scope is audit-only.
- PH1.X gating states are explicit and reason-coded:
  - `CLARIFY` (missing/ambiguous fields)
  - `CONFIRM` (impactful intent awaiting yes/no)
  - `DISPATCH` (candidate handoff only)
  - `RESPOND` / `WAIT` (non-executing output/control)
- PH1.X continuity gates are explicit and fail closed:
  - `subject_ref` is required on every PH1.X request and is stamped into returned thread state.
  - `active_speaker_user_id` is required on every PH1.X request and is stamped into returned thread state.
  - continuity thread-state fields are deterministic:
    - `active_subject_ref`
    - `interrupted_subject_ref`
    - `resume_buffer`
    - `return_check_pending`
    - `return_check_expires_at`
  - if `thread_state.active_speaker_user_id != request.active_speaker_user_id`: PH1.X emits a single clarify (`X_CONTINUITY_SPEAKER_MISMATCH`) and does not dispatch.
  - if `thread_state.active_subject_ref != request.subject_ref` while a pending state exists: PH1.X emits a single clarify (`X_CONTINUITY_SUBJECT_MISMATCH`) and does not dispatch.
- WorkOrder state transitions (`DRAFT -> CLARIFY -> CONFIRM -> EXECUTING`) are persisted by `SELENE_OS_CORE_TABLES`; PH1.X emits directive rows that deterministically wire back into those transitions.
- Interruption continuity branch contract (Step-1 lock):
  - PH1.K handoff projection contract lock (Step-10):
    - PH1.K handoff payload is `Ph1kToPh1xInterruptHandoff` with bounded fields only:
      - `candidate_confidence_band`
      - `gate_confidences`
      - `degradation_context`
      - `risk_context_class`
    - PH1.K handoff is advisory input only; PH1.X remains authoritative for interruption action and branch outcome.
  - request fields:
    - when `interruption` is present, payload must include:
      - `trigger_phrase_id` (must match `phrase_id`)
      - `trigger_locale`
      - `candidate_confidence_band` in `HIGH | MEDIUM | LOW`
      - `risk_context_class` in `LOW | GUARDED | HIGH`
      - `degradation_context` (`capture_degraded`, `aec_unstable`, `device_changed`, `stream_gap_detected`)
      - `timing_markers` (`window_start`, `window_end`) with `window_end == interruption.t_event`
      - `speech_window_metrics` (`voiced_window_ms` bounded)
      - `subject_relation_confidence_bundle` (`lexical`, `vad`, `speech_likeness`, `echo_safe`, optional `nearfield`, `combined`) with deterministic parity against gate confidences
    - when interruption continuity is active (`interruption` present or `thread_state.resume_buffer` present), `interrupt_subject_relation` in `SAME | SWITCH | UNCERTAIN` is required.
    - when interruption continuity is active, `interrupt_subject_relation_confidence` in `[0.0, 1.0]` is required and paired with relation field.
  - response field:
    - `interrupt_continuity_outcome` in `SAME_SUBJECT_APPEND | SWITCH_TOPIC_THEN_RETURN_CHECK` (optional)
    - `interrupt_resume_policy` in `RESUME_NOW | RESUME_LATER | DISCARD` (optional)
  - fail-closed rule:
    - when relation is missing/uncertain or confidence is insufficient, PH1.X must emit one `clarify` and must not dispatch.
    - when `tts_resume_snapshot` is present:
      - `interruption.t_event` must be `<= now`
      - snapshot age relative to interruption must be bounded
      - `spoken_cursor_byte` must leave non-empty unsaid remainder
      - timing window mismatches fail closed
  - SAME-branch merge rule:
    - when relation is `SAME` with sufficient confidence and resume buffer is active, PH1.X emits one `respond` output that merges `resume_buffer.unsaid_remainder` with the new chat response, sets `interrupt_continuity_outcome=SAME_SUBJECT_APPEND`, and stamps `interrupt_resume_policy=RESUME_NOW`.
  - SWITCH-branch return-check rule:
    - when relation is `SWITCH` with sufficient confidence and resume buffer is active, PH1.X emits one `respond` output that answers the new topic and appends one return-check question ("Do you still want to continue the previous topic?"), keeps `resume_buffer` intact, sets `interrupt_continuity_outcome=SWITCH_TOPIC_THEN_RETURN_CHECK`, and stamps `interrupt_resume_policy=RESUME_LATER`.
  - UNCERTAIN-branch clarify rule:
    - when interruption continuity is active and relation is `UNCERTAIN` (or confidence is below branch threshold), PH1.X emits exactly one bounded `clarify` question, blocks dispatch/action, and preserves `resume_buffer` until clarified.
    - clarify contract is deterministic: question is single-line and `<= 240` chars; `accepted_answer_formats` is bounded to `2..=3` entries.
  - return-check decision rule:
    - if `return_check_pending=true` and user answers `confirm_answer=Yes`, PH1.X resumes the interrupted remainder and stamps `interrupt_resume_policy=RESUME_NOW`.
    - if `return_check_pending=true` and user answers `confirm_answer=No`, PH1.X clears `resume_buffer` explicitly and stamps `interrupt_resume_policy=DISCARD`.
    - silent discard is forbidden.
  - expiry/replay determinism:
    - if `resume_buffer.expires_at <= now`, PH1.X clears `resume_buffer`, `interrupted_subject_ref`, `return_check_pending`, and `return_check_expires_at`.
    - if `return_check_expires_at <= now`, PH1.X clears `return_check_pending` and `return_check_expires_at`.

## 6) Audit Emissions (PH1.J)

PH1.X writes emit PH1.J audit events with:
- `event_type`:
  - `XConfirm`
  - `XDispatch`
  - `Other` (for `clarify`, `respond`, `wait`)
- `reason_code(s)`:
  - deterministic PH1.X reason codes from the PH1.X contract output path
  - interruption branch reason codes are persisted directly in `audit_events.reason_code` (no remap).
- `payload_min` keys (bounded):
  - `directive`
  - `confirm_kind`
  - `what_is_missing`
  - `clarification_unit_id`
  - `response_kind`
  - `dispatch_target`
  - `wait_kind`
  - `work_order_id`
  - `work_order_status_snapshot`
  - `pending_state`
  - `lease_token_hash`
  - `interrupt_subject_relation`
  - `interrupt_subject_relation_confidence`
  - `interrupt_continuity_outcome`
  - `interrupt_resume_policy`
  - `interrupted_subject_ref`
  - `return_check_pending`
  - `return_check_expires_at`

Reporting lock:
- PH1.HEALTH display/report paths must consume PH1.X interruption outcomes using:
  - `owner_engine_id=PH1.X` (engine visibility),
  - `latest_reason_code` (branch outcome reason code),
  - `issue_fingerprint` as topic marker (topic visibility).

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-X-01` tenant isolation enforced
  - `at_x_db_01_tenant_isolation_enforced`
- `AT-X-02` append-only enforcement for PH1.X ledger writes
  - `at_x_db_02_append_only_enforced`
- `AT-X-03` idempotency dedupe works
  - `at_x_db_03_idempotency_dedupe_works`
- `AT-X-04` no PH1.X current-table rebuild is required
  - `at_x_db_04_no_current_table_rebuild_required`
- `AT-X-13` continuity speaker mismatch is fail-closed and clarify-only
  - `at_x_continuity_speaker_mismatch_fails_closed_into_one_clarify`
- `AT-X-14` continuity subject mismatch with pending state is fail-closed and clarify-only
  - `at_x_continuity_subject_mismatch_with_pending_fails_closed_into_one_clarify`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 16 (`PH1.X` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_x/db_wiring.rs`

## 8) Related Engine Boundary (`PH1.PRUNE`)

- Before PH1.X `clarify` commit rows are emitted, Selene OS may invoke PH1.PRUNE when multiple required fields exist.
- If PH1.PRUNE is used, `what_is_missing` should map to PH1.PRUNE `selected_missing_field`; if PH1.PRUNE validation fails, Selene OS must fail closed and skip PRUNE-derived handoff.
- PH1.X remains authoritative for one conversational move per turn; PH1.PRUNE output is advisory input only.

## 9) Related Engine Boundary (`PH1.EXPLAIN`)

- PH1.X may request PH1.EXPLAIN packets only for explicit explain triggers (`why/how/what happened`) or accountability prompts.
- PH1.X decides whether to surface PH1.EXPLAIN output; PH1.EXPLAIN remains advisory and non-executing.
- PH1.X must never treat PH1.EXPLAIN text as authority, confirmation, or execution approval.

## 10) Related Engine Boundary (`PH1.EMO.GUIDE`)

- Before PH1.X final `respond` directive shaping, Selene OS may invoke PH1.EMO.GUIDE to compute a bounded style profile hint (`DOMINANT | GENTLE` + ordered modifiers).
- If PH1.EMO.GUIDE validation fails (`validation_status != OK`), Selene OS must fail closed on EMO.GUIDE handoff and continue without EMO-guided tone hints.
- PH1.X may consume PH1.EMO.GUIDE hints for tone/pacing only; PH1.X must not allow EMO.GUIDE to alter facts, missing-field logic, confirmation semantics, or dispatch gating.

## 11) Related Engine Boundary (`PH1.PERSONA`)

- Before PH1.X final `respond` directive shaping, Selene OS may invoke PH1.PERSONA and forward persona hints only when `PERSONA_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X may consume `style_profile_ref` + `delivery_policy_ref` as phrasing/tone posture hints only.
- PH1.X must not allow PH1.PERSONA output to alter truth, intent selection, missing-field logic, confirmation semantics, access outcomes, or dispatch gating.
- If PH1.PERSONA validation fails, Selene OS must fail closed on persona handoff and continue with deterministic default PH1.X behavior.

## 12) FDX Wiring Lock (Section 5F)

- PH1.X wiring is authoritative for duplex interruption branch outcomes and final turn commit decisions.
- PH1.X rows must preserve branch/result proof fields:
  - `interrupt_subject_relation`
  - `interrupt_subject_relation_confidence`
  - `interrupt_continuity_outcome`
  - `interrupt_resume_policy`
- Speculative planning outputs must remain non-executing and auditable until PH1.X emits final commit posture.
