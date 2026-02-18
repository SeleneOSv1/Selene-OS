# PH1_PRUNE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PRUNE
- layer: Conversation Assist
- authority: Non-Authoritative
- role: Missing-field pruning for one-question clarify discipline before PH1.X asks a question
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied `required_fields_missing` and ambiguity hints from PH1.NLP output.
  - Optional bounded context hints (`prefilled_fields`, `confirmed_fields`, `previous_clarify_field`) from Selene OS turn state.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard one-question rule: PH1.PRUNE output must identify one selected missing field only; PH1.X remains responsible for asking the user.
- Hard no-repeat rule: when alternatives exist, PH1.PRUNE must avoid repeating `previous_clarify_field`.

## D) Wiring
- Invoked_by: Selene OS step between PH1.NLP and PH1.X clarify path.
- Inputs_from:
  - PH1.NLP: `required_fields_missing`, `ambiguity_flags`.
  - Selene OS turn context: `uncertain_field_hints`, `prefilled_fields`, `confirmed_fields`, `previous_clarify_field`.
- Outputs_to:
  - `selected_missing_field` and `ordered_missing_fields` bundle to Selene OS.
  - Selene OS forwards selected field into PH1.X clarify packet (`what_is_missing`).
- Invocation_condition: OPTIONAL(multiple required fields missing)
- Deterministic sequence:
  - `PRUNE_MISSING_FIELDS` computes deterministic ordered candidates and one selected field.
  - `PRUNE_CLARIFY_ORDER` self-validates selected-vs-ordered consistency and no-repeat discipline.
  - If `validation_status != OK`, Selene OS fails closed and does not forward PRUNE output to PH1.X.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.NLP remains authoritative for `required_fields_missing`; PH1.PRUNE must not invent new field keys.
- PH1.X remains authoritative for conversational move choice; PH1.PRUNE only narrows clarify target selection.
- PH1.PRUNE may consume clarification-ordering artifacts derived from PH1.RLL only after governance approval.

## F) Acceptance Tests
- AT-PRUNE-01: Selene OS can invoke `PRUNE_MISSING_FIELDS` and output is schema-valid.
- AT-PRUNE-02: Ordering is deterministic for identical inputs.
- AT-PRUNE-03: Previous clarify field is not re-selected when alternatives exist.
- AT-PRUNE-04: Clarify-order validation fails closed when selected field/order drift is detected.
