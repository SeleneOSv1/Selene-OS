# ONB_INVITED Blueprint Record

## 1) Blueprint Header
- `process_id`: `ONB_INVITED`
- `intent_type`: `ONB_INVITED`
- `version`: `v1`
- `status`: `ACTIVE`

## 1A) Contract Boundary
- This blueprint defines orchestration flow only.
- Engine behavior/schema/capability contracts are canonical in `docs/DB_WIRING/*.md` and `docs/ECM/*.md`.
- ONB prompt/gate behavior is schema-driven from pinned field specs and required gates; no hardcoded ONB-only requirement branch is allowed.

## 1B) Entry Preconditions (Link is out-of-scope)
- Link validation/expiry/revocation/device binding are handled by the Link process (`LINK_OPEN_ACTIVATE`).
- Selene App installation is automatic; invitee only approves OS permission prompts if required by platform.
- Onboarding begins only after link activation succeeded and `token_id` is available in Selene App context.
- ONB must not re-check link signature/expiry/revocation/device binding.

## 2) Required Inputs
- `token_id`
- `device_fingerprint`
- `invitee_user_id` (or pending identity context)
- `tenant_id` (required for employee path)
- `idempotency_key` (required for commit simulations; session-start is token-idempotent)

## 2A) Preconditions / Handoff Contract
- ONB starts only after `LINK_OPEN_ACTIVATE` succeeds and `token_id` is present.
- ONB must not re-check link signature/expiry/revocation and must not perform device binding.
- ONB resume enforces onboarding session device integrity: if current `device_fingerprint` != `onboarding_session.device_fingerprint_hash`, fail closed.
- ONB loads canonical onboarding context from the activated link (`token_id`) and pinned schema context.
- ONB asks only missing fields and never re-asks fields already present.

## 3) Success Output Schema
```text
onboarding_session_id: string
status: enum (COMPLETE | BLOCKED | DEFERRED | FAILED)
pinned_schema_id: string
pinned_schema_version: string
pinned_overlay_set_id: string
pinned_selector_snapshot: object (bounded)
required_verification_gates: string[]
required_gates_remaining: string[]
access_engine_instance_id: string (optional)
```

## 4) Ordered Engine Steps

| step_id | engine_name | capability_id | required_fields | produced_fields | side_effects | timeout_ms | max_retries | retry_backoff_ms | retryable_reason_codes |
|---|---|---|---|---|---|---:|---:|---:|---|
| ONB_INVITED_S01 | PH1.ONB | PH1ONB_SESSION_START_DRAFT_ROW | token_id, device_fingerprint | onboarding_session_id, onboarding_state, pinned_schema_context, required_verification_gates | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_START_RETRYABLE] |
| ONB_INVITED_S02 | PH1.C | PH1C_TRANSCRIPT_OK_COMMIT_ROW | correlation_id, turn_id, transcript_hash | transcript_ok evidence row | DB_WRITE | 1200 | 1 | 150 | [STT_FAIL_PROVIDER_TIMEOUT, STT_FAIL_NETWORK_UNAVAILABLE] |
| ONB_INVITED_S03 | PH1.NLP | PH1NLP_INTENT_DRAFT_COMMIT_ROW | transcript_ok, onboarding_context | intent_draft / missing field signal | DB_WRITE | 200 | 1 | 100 | [NLP_INPUT_MISSING] |
| ONB_INVITED_S04 | PH1.X | PH1X_CLARIFY_COMMIT_ROW | missing field signal | one-question clarify state | DB_WRITE | 300 | 1 | 100 | [OS_CLARIFY_TIMEOUT] |
| ONB_INVITED_S05 | PH1.ONB | PH1ONB_TERMS_ACCEPT_COMMIT_ROW | onboarding_session_id, terms_version_id, accepted, idempotency_key | terms_state | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_TERMS_RETRYABLE] |
| ONB_INVITED_S06 | PH1.ONB | PH1ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT_ROW | onboarding_session_id, photo_blob_ref, sender_user_id, idempotency_key | sender_verification_pending | DB_WRITE (simulation-gated, conditional: run only when `required_verification_gates` includes photo evidence gate; legacy capability id retained) | 1000 | 2 | 350 | [ONB_PHOTO_SEND_RETRYABLE] |
| ONB_INVITED_S07 | PH1.ONB | PH1ONB_EMPLOYEE_SENDER_VERIFY_COMMIT_ROW | onboarding_session_id, sender_user_id, decision, idempotency_key | sender_verification_state | DB_WRITE (simulation-gated, conditional: run only when `required_verification_gates` includes sender confirmation gate; legacy capability id retained) | 800 | 2 | 300 | [ONB_SENDER_VERIFY_RETRYABLE] |
| ONB_INVITED_S08 | PH1.ONB | PH1ONB_PRIMARY_DEVICE_CONFIRM_COMMIT_ROW | onboarding_session_id, device_id, proof_type, proof_ok, idempotency_key | primary_device_confirmed | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_PRIMARY_DEVICE_RETRYABLE] |
| ONB_INVITED_S09 | PH1.VOICE.ID | VID_ENROLL_START_DRAFT_ROW | onboarding_session_id, device_id, consent_asserted | voice_enrollment_session_id | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [VID_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S10 | PH1.VOICE.ID | VID_ENROLL_SAMPLE_COMMIT_ROW | voice_enrollment_session_id, sample payload, idempotency_key | enrollment_progress | DB_WRITE (simulation-gated) | 900 | 3 | 250 | [VID_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S11 | PH1.VOICE.ID | VID_ENROLL_COMPLETE_COMMIT_ROW | voice_enrollment_session_id, idempotency_key | voice_profile_id, enrollment_complete | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [VID_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S12 | PH1.W | PH1W_ENROLL_START_DRAFT_ROW | onboarding_session_id, user_id, device_id, idempotency_key | wake_enrollment_session_id | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [W_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S13 | PH1.W | PH1W_ENROLL_SAMPLE_COMMIT_ROW | wake_enrollment_session_id, sample payload, idempotency_key | wake_enrollment_progress | DB_WRITE (simulation-gated) | 900 | 3 | 250 | [W_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S14 | PH1.W | PH1W_ENROLL_COMPLETE_COMMIT_ROW | wake_enrollment_session_id, wake_profile_id, idempotency_key | wake_enrollment_complete | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [W_IDEMPOTENCY_REPLAY] |
| ONB_INVITED_S15 | PH1.ONB | PH1ONB_ACCESS_INSTANCE_CREATE_COMMIT_ROW | onboarding_session_id, user_id, tenant_id, role_id, idempotency_key | access_engine_instance_id | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_ACCESS_CREATE_RETRYABLE] |
| ONB_INVITED_S16 | PH1.ONB | PH1ONB_COMPLETE_COMMIT_ROW | onboarding_session_id, idempotency_key | onboarding_complete | DB_WRITE (simulation-gated) | 700 | 2 | 250 | [ONB_COMPLETE_RETRYABLE] |

Deferral/reminder rule (S01/S04 consent/clarify flow):
- If invitee says `NOT NOW`: schedule reminders via PH1.BCAST procedures (Selene App thread first).
- If invitee says `NEVER`: stop onboarding and notify sender (JD) for human-to-human resolution.

## 5) Confirmation Points
- Terms acceptance before `ONB_INVITED_S05`.
- Sender verification gate before completion only when pinned schema requires sender confirmation (`ONB_INVITED_S07`).
- Completion confirmation before `ONB_INVITED_S16` when policy requires explicit final consent.

## 6) Simulation Requirements
- `ONB_SESSION_START_DRAFT`
- `ONB_DRAFT_UPDATE_COMMIT`
- `ONB_TERMS_ACCEPT_COMMIT`
- `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`
- `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT`
- `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
- `ONB_ACCESS_INSTANCE_CREATE_COMMIT`
- `ONB_COMPLETE_COMMIT`
- `VOICE_ID_ENROLL_START_DRAFT`
- `VOICE_ID_ENROLL_SAMPLE_COMMIT`
- `VOICE_ID_ENROLL_COMPLETE_COMMIT`
- `VOICE_ID_ENROLL_DEFER_REMINDER_COMMIT`
- `WAKE_ENROLL_START_DRAFT`
- `WAKE_ENROLL_SAMPLE_COMMIT`
- `WAKE_ENROLL_COMPLETE_COMMIT`
- `WAKE_ENROLL_DEFER_REMINDER_COMMIT`

Conditional execution note:
- `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` and `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT` execute only when pinned schema-required verification gates include them.

## 7) Refusal Conditions
- Terms declined -> `ONB_TERMS_DECLINED`
- Sender rejects schema-required verification -> `ONB_SENDER_REJECTED`
- Device proof failed -> `ONB_PRIMARY_DEVICE_PROOF_FAILED`
- Required gate timeout/defer exhausted -> `ONB_REQUIRED_GATE_NOT_MET`

## 8) Acceptance Tests
- `AT-PBS-ONB-01`: No duplicate asks when pinned onboarding context already contains field values.
- `AT-PBS-ONB-02`: Verification gates run only when pinned schema requires them; no hardcoded employee-only gate.
- `AT-PBS-ONB-03`: Voice and wake enrollment are simulation-gated.
- `AT-PBS-ONB-04`: Completion only after all required gates pass.
- `AT-PBS-ONB-06`: ONB does not perform link validation; it requires prior successful link activation + `token_id` context.
- `AT-PBS-ONB-07`: ONB evaluates missing required fields from pinned schema context and asks only those fields (never ask twice).
- `AT-PBS-ONB-08`: Resume after interruption continues from persisted onboarding session state with no re-asks.
- `AT-PBS-ONB-09`: Device mismatch on resume fails closed.
- `AT-PBS-ONB-10`: Schema-required verification gates are evaluated from pinned schema context (no hardcoded field alias fallback).
