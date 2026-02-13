# PH1.POLICY DB Wiring (Design vNext)

## A) Engine Header
- `engine_id`: `PH1.POLICY`
- `layer`: `Control (Global Policy)`
- `authority`: `Authoritative (policy decisions only; never execution)`
- `role`: Global Rule Base + Policy Snapshot (deterministic universal rules for prompting and response discipline).
- `placement`: `ALWAYS_ON` (consulted by Selene OS before prompting decisions).

## B) Ownership
- `tables_owned`: none (vNext design)
- `writes`: none
- `reads`: Selene OS-supplied snapshots only:
  - WorkOrder snapshot:
    - `fields_json`
    - `missing_fields_json`
    - `asked_fields_json`
    - `prompt_dedupe_keys_json`
  - user profile hint pointers:
    - preferred delivery mode (snapshot pointer)
    - language preference hint (snapshot pointer)

Scope ownership lock:
- PH1.POLICY owns no flows.
- PH1.POLICY owns no access logic.
- PH1.POLICY owns no message lifecycle logic.
- Access escalation is owned by `PH1.ACCESS.001/PH2.ACCESS.002`.
- Message interruption lifecycle is owned by `PH1.BCAST` (BCAST.MHP) + `PH1.REM` timing.

## C) Hard Boundaries
- must never execute actions
- must never call engines
- must never grant permissions
- must never modify WorkOrders or DB state
- outputs are decisions only; Selene OS enforces them

## D) Wiring
- `invoked_by`: Selene OS only
- `inputs_from`: WorkOrder snapshots and policy context snapshots
- `outputs_to`: Selene OS decision path shaping PH1.X prompt behavior only
- `invocation_condition`: ONLY when Selene OS is about to ask a question

Universal policy wiring:
- Before PH1.X emits clarify/confirm, Selene OS calls `POLICY_PROMPT_DEDUP_DECIDE`.
- Selene OS remains orchestrator; PH1.POLICY is decision-only.

## E) Acceptance Tests
- `AT-POLICY-01`: Prompt dedupe prevents re-asking a field already present in WorkOrder/authoritative snapshot.
- `AT-POLICY-02`: Prompt dedupe suppresses repeat prompts using the prompt dedupe key unless state changed.
- `AT-POLICY-03`: Ruleset snapshot is stable and deterministic (same inputs -> same outputs).
- `AT-POLICY-04`: PH1.POLICY contains no access-flow or message-lifecycle logic (pointer-only boundaries).
