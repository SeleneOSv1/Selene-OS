# PH1.X ECM Spec

## Engine Header
- `engine_id`: `PH1.X`
- `purpose`: Persist deterministic PH1.X conversational directives (`confirm`, `clarify`, `respond`, `dispatch`, `wait`) as bounded audit rows, including interruption continuity branch metadata.
- `data_owned`: `audit_events` writes in PH1.X scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1X_CONFIRM_COMMIT_ROW`
- `name`: Commit confirm directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, confirm_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_CLARIFY_COMMIT_ROW`
- `name`: Commit clarify directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, what_is_missing, clarification_unit_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_RESPOND_COMMIT_ROW`
- `name`: Commit respond directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, response_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_TOOL_RESPONSE_RENDER`
- `name`: Render read-only PH1.E tool response into final user-facing text payload
- `input_schema`: `(tool_response, locale?, output_mode=text|tts, reason_code)`
- `output_schema`: `(response_text, source_list, citations?, retrieved_at, reason_code)`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1X_DISPATCH_COMMIT_ROW`
- `name`: Commit dispatch directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, dispatch_target, lease_token_hash?, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_WAIT_COMMIT_ROW`
- `name`: Commit wait directive
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, work_order_id, work_order_status_snapshot, pending_state, wait_kind, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1X_READ_AUDIT_ROWS`
- `name`: Read PH1.X audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.X outputs must always carry deterministic PH1.X reason_code values from the PH1.X contract path.
- storage scope/idempotency failures are fail-closed and reason-coded in PH1.X audit emissions.
- gating failures are deterministic and reason-coded (`X_FAIL_WORK_ORDER_SCOPE_INVALID`, `X_FAIL_LEASE_REQUIRED`, `X_FAIL_PENDING_STATE_INVALID`).
- continuity failures are deterministic and reason-coded:
  - `X_CONTINUITY_SPEAKER_MISMATCH`: active speaker mismatch against thread continuity; PH1.X returns one clarify and blocks dispatch.
  - `X_CONTINUITY_SUBJECT_MISMATCH`: subject drift while pending exists; PH1.X returns one clarify and blocks dispatch.
- interruption continuity branch failures (Step-1 lock):
  - insufficient subject-relation confidence => one clarify and no dispatch (`X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY` reserved for runtime mapping).
  - branch outcome reason codes reserved for runtime mapping:
    - `X_INTERRUPT_SAME_SUBJECT_APPEND`
    - `X_INTERRUPT_RETURN_CHECK_ASKED`
    - `X_INTERRUPT_RESUME_NOW`
    - `X_INTERRUPT_DISCARD`
- reason-code persistence lock:
  - branch reason codes are written verbatim into PH1.X `audit_events.reason_code` rows (no translation layer).

## Interruption Continuity Branch Contract (Step-1 Lock)

PH1.K handoff projection lock (Step-10):
- PH1.K provides bounded interruption risk context via `Ph1kToPh1xInterruptHandoff`:
  - `candidate_confidence_band`
  - `gate_confidences`
  - `degradation_context`
  - `risk_context_class`
- PH1.K handoff remains advisory substrate only; PH1.X remains authoritative for interruption branch/action outcomes.

Request contract fields:
- when `interruption` is present, payload must include:
  - `trigger_phrase_id` (must match `phrase_id`)
  - `trigger_locale`
  - `candidate_confidence_band` in `HIGH | MEDIUM | LOW`
  - `risk_context_class` in `LOW | GUARDED | HIGH`
  - `degradation_context` (`capture_degraded`, `aec_unstable`, `device_changed`, `stream_gap_detected`)
  - `timing_markers` (`window_start`, `window_end`) where `window_end == interruption.t_event`
  - `speech_window_metrics` (`voiced_window_ms` bounded)
  - `subject_relation_confidence_bundle` (`lexical`, `vad`, `speech_likeness`, `echo_safe`, optional `nearfield`, `combined`) and parity with gate confidences
- when interruption continuity is active (`interruption` present or `thread_state.resume_buffer` present), `interrupt_subject_relation` is required (`SAME | SWITCH | UNCERTAIN`)
- when interruption continuity is active, `interrupt_subject_relation_confidence` is required (float `[0.0, 1.0]`, paired with relation field)

Response contract field (forward-compatible):
- `interrupt_continuity_outcome`: `SAME_SUBJECT_APPEND | SWITCH_TOPIC_THEN_RETURN_CHECK`
- `interrupt_resume_policy`: `RESUME_NOW | RESUME_LATER | DISCARD`

Thread continuity state contract fields:
- `active_subject_ref`
- `interrupted_subject_ref`
- `resume_buffer`
- `return_check_pending`
- `return_check_expires_at`

Fail-closed rule:
- If relation is missing/uncertain or confidence is insufficient for safe routing, PH1.X must emit one clarify and block dispatch/action.
- If `tts_resume_snapshot` is present, PH1.X must fail closed when interruption timing is stale/misaligned or when `spoken_cursor_byte` leaves no unsaid remainder.
- If relation is `SAME` with sufficient confidence and resume buffer is active, PH1.X must emit one merged `respond` outcome, set `interrupt_continuity_outcome=SAME_SUBJECT_APPEND`, and stamp `interrupt_resume_policy=RESUME_NOW`.
- If relation is `SWITCH` with sufficient confidence and resume buffer is active, PH1.X must emit one `respond` outcome that includes a return-check question, keep resume buffer intact, set `interrupt_continuity_outcome=SWITCH_TOPIC_THEN_RETURN_CHECK`, and stamp `interrupt_resume_policy=RESUME_LATER`.
- If relation is `UNCERTAIN` (or branch confidence is below threshold) while interruption continuity is active, PH1.X must emit exactly one bounded clarify question and block dispatch/action until clarified.
- Clarify contract is deterministic: clarify question is single-line and `<= 240` chars; `accepted_answer_formats` is bounded to `2..=3` entries.
- If `return_check_pending=true` and user answers `confirm_answer=Yes`, PH1.X resumes interrupted remainder and stamps `interrupt_resume_policy=RESUME_NOW`.
- If `return_check_pending=true` and user answers `confirm_answer=No`, PH1.X clears `resume_buffer` explicitly and stamps `interrupt_resume_policy=DISCARD`.
- Silent discard is forbidden.
- If `resume_buffer` expires, PH1.X clears interruption continuity markers deterministically (`interrupted_subject_ref`, `return_check_pending`, `return_check_expires_at`).

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J rows using:
  - `XConfirm` for confirm
  - `XDispatch` for dispatch
  - `Other` for clarify/respond/wait (with bounded directive payload keys)
- required bounded payload keys include:
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
  - `interrupt_resume_policy`
- read capability emits audit only in explicit replay/diagnostic mode.

Reporting lock:
- PH1.HEALTH report rows must expose PH1.X interruption outcomes by:
  - `owner_engine_id=PH1.X`,
  - `latest_reason_code` for branch outcome,
  - `issue_fingerprint` as topic marker when present.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1xConversationRepo`)
- `docs/DB_WIRING/PH1_X.md`

## Related Engine Boundary (`PH1.PRUNE`)
- PH1.X clarify packet construction may consume PH1.PRUNE `selected_missing_field` when the turn has multiple missing fields.
- PH1.X must not treat PH1.PRUNE as authoritative; PH1.X remains responsible for final move selection and fail-closed behavior.

## Related Engine Boundary (`PH1.EXPLAIN`)
- PH1.X can invoke PH1.EXPLAIN only for explicit explain triggers or accountability responses.
- PH1.X must treat PH1.EXPLAIN as advisory output only; no authority or execution semantics are inferred from explanation text.

## Related Engine Boundary (`PH1.EMO.GUIDE`)
- PH1.X may consume PH1.EMO.GUIDE style-profile hints only when `EMO_GUIDE_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X must treat PH1.EMO.GUIDE as advisory tone policy only; no authority, execution, truth, or confirmation semantics can be inferred from EMO.GUIDE output.

## Related Engine Boundary (`PH1.PERSONA`)
- PH1.X may consume PH1.PERSONA style/delivery profile hints only when `PERSONA_PROFILE_VALIDATE` returns `validation_status=OK`.
- PH1.X must treat PH1.PERSONA as advisory tone/delivery policy only; no authority, execution, truth, or confirmation semantics can be inferred from PH1.PERSONA output.

## FDX Design Lock (Section 5F)
- PH1.X is the single authoritative owner for interruption branch decisions and turn commit in duplex sessions.
- PH1.X consumes:
  - PH1.K `InterruptCandidate`
  - PH1.C `PartialTranscript`
  - PH1.NLP `IntentHypothesis`
- PH1.X may build speculative drafts (`SpeculativePlan`) while user speech is in progress, but speculative output is non-executing.
- PH1.X must emit final authoritative `TurnCommitDecision` only after confidence + policy gates pass.
- PH1.X must enforce branch discipline:
  - SAME subject: merge and continue (`RESUME_NOW`)
  - SWITCH subject: answer new topic then ask return-check (`RESUME_LATER`)
  - UNCERTAIN: one clarify, no dispatch/action
- PH1.X must preserve resume policy lock (`RESUME_NOW | RESUME_LATER | DISCARD`) and reason-coded audit emission on every branch.
