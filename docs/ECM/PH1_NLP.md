# PH1.NLP ECM Spec

## Engine Header
- `engine_id`: `PH1.NLP`
- `purpose`: Persist deterministic NLP decision outputs (`intent_draft`, `clarify`, `chat`) as bounded audit rows.
- `data_owned`: `audit_events` writes in PH1.NLP scope
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1NLP_INTENT_DRAFT_COMMIT_ROW`
- `name`: Commit NLP intent draft decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, intent_type, extracted_fields, required_fields, ambiguity_flags, overall_confidence, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_CLARIFY_COMMIT_ROW`
- `name`: Commit NLP clarify decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, what_is_missing, clarification_unit_id, accepted_answer_formats, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_CHAT_COMMIT_ROW`
- `name`: Commit NLP chat/default decision
- `input_schema`: `(now, tenant_id, correlation_id, turn_id, session_id?, user_id, device_id, reason_code, idempotency_key)`
- `output_schema`: `Result<AuditEventId, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1NLP_READ_AUDIT_ROWS`
- `name`: Read PH1.NLP audit rows for one correlation thread
- `input_schema`: `correlation_id`
- `output_schema`: `AuditEvent[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- intent/clarify/chat outputs must always carry PH1.NLP deterministic reason codes, including:
  - `NLP_INTENT_OK`
  - `NLP_INTENT_UNKNOWN`
  - `NLP_MULTI_INTENT`
  - `NLP_CLARIFY_MISSING_FIELD`
  - `NLP_CLARIFY_AMBIGUOUS_REFERENCE`
  - `NLP_UNCERTAIN_SPAN`
  - `NLP_CHAT_DEFAULT`
- scope/contract failures are fail-closed with deterministic PH1.NLP reason coding.

## Audit Emission Requirements Per Capability
- write capabilities emit PH1.J audit rows with bounded keys only:
  - `decision`
  - `intent_type`
  - `required_fields`
  - `ambiguity_flags`
  - `overall_confidence`
  - `what_is_missing`
  - `clarification_unit_id`
- read capability emits audit only in explicit replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1NlpRepo`)
- `docs/DB_WIRING/PH1_NLP.md`

## Related Engine Boundary (`PH1.PRUNE`)
- PH1.NLP may provide `required_fields_missing` to Selene OS for optional PH1.PRUNE candidate narrowing.
- PH1.NLP does not depend on PH1.PRUNE for core intent/clarify decisions; PH1.PRUNE is a turn-optional assist path only.

## Related Engine Boundary (salience ranking)
- PH1.NLP may consume one Selene OS-curated `selected_focus_span` and ordered salience hints from deterministic upstream context handling.
- PH1.NLP must not treat salience metadata as intent authority; PH1.NLP remains the deterministic intent/clarify owner.

## Related Engine Boundary (`PH1.SRL`)
- PH1.NLP consumes Selene OS-curated SRL repaired transcript/frame output as deterministic upstream normalization input.
- PH1.NLP remains final owner of intent/clarify/chat decision mode and must not treat SRL as execution authority.

## Related Engine Boundary (tangled utterance parsing)
- PH1.NLP performs tangled-utterance unraveling internally and may consume only bounded upstream ambiguity metadata.
- PH1.NLP remains authoritative for final decision mode and field completeness.
- When ambiguity remains unresolved, PH1.NLP must keep clarify-first behavior and must not guess missing fields.

## Related Engine Boundary (`PH1.KNOW`)
- PH1.NLP may consume Selene OS-curated PH1.KNOW dictionary hints as advisory metadata only.
- PH1.NLP remains deterministic owner of final intent/clarify/chat outputs and must not treat PH1.KNOW hints as transcript evidence replacement.
- PH1.KNOW-derived hints must remain tenant-scoped and authorized-only in PH1.NLP capability execution.

## FDX Design Lock (Section 5F)
- PH1.NLP owns incremental intent hypothesis generation over partial transcript streams.
- PH1.NLP must emit bounded `IntentHypothesis` candidates (`intent`, `score`, `required_slots`, `ambiguity_flags`) as advisory output only.
- PH1.NLP must not execute actions or finalize interruption branches.
- When intent confidence is below threshold, PH1.NLP must preserve clarify-first posture and explicit missing-field outputs.
- PH1.NLP must keep no-guess behavior during duplex partials and finalization handoff.
