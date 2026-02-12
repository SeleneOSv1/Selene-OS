# PH1_LEARNING_ADAPTIVE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LEARNING_ADAPTIVE
- layer: Learning Assist
- authority: Non-Authoritative (learning quality only)
- role: Capture user feedback and language usage signals to improve future phrasing/subject inference hints
- placement: TURN_OPTIONAL (async, post-send and post-feedback windows)

## B) Ownership
Owned tables (design-level):
- learning.adaptive_feedback_ledger (append-only)
- learning.adaptive_feedback_current (rebuildable)
- learning.adaptive_language_usage_ledger (append-only)

Required fields:
- tenant_id
- user_id
- source_engine (PH1.WRITE | PH1.NLP | PH1.LISTEN)
- feedback_type (DRAFT_CORRECTION | LANGUAGE_PREFERENCE | STYLE_ADJUSTMENT)
- feedback_payload_ref
- quality_delta_bucket
- idempotency_key
- created_at

Rules:
- All writes are non-authoritative learning signals only.
- No permission or execution state changes are allowed.
- Idempotency dedupe on (tenant_id, user_id, feedback_type, idempotency_key).

## C) Hard Boundaries
- never executes actions, never grants authority, never bypasses Access/Simulation.
- never mutates WorkOrder or access state.
- outputs are hints for PH1.WRITE/PH1.NLP ranking only.
- multilingual corpus/model adaptation is offline and artifact-driven (for example, mT5-class multilingual fine-tune pipelines), then consumed as bounded runtime hints only.

## D) Wiring
- Invoked_by: Selene OS async learning pipeline after user feedback or draft corrections.
- Inputs_from: PH1.LISTEN feedback signals, session transcript pointers, PH1.WRITE result context.
- Outputs_to: Selene OS hint snapshots consumed later by PH1.WRITE and ranking assists.
- Invocation_condition: OPTIONAL(async only).
- Not allowed:
  - in-turn mandatory execution
  - engine-to-engine direct calls
  - side effects outside learning store

## E) Acceptance Tests
- AT-LEARNING-ADAPTIVE-01: Feedback is recorded deterministically and retrievable for next-draft hinting.
- AT-LEARNING-ADAPTIVE-02: Repeated corrections from the same user produce measurable quality-hint deltas.
- AT-LEARNING-ADAPTIVE-03: Draft quality trend improves over repeated accepted corrections.
