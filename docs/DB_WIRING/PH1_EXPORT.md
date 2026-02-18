# PH1_EXPORT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.EXPORT
- layer: Enterprise Support
- authority: Authoritative (compliance export proof output)
- role: Deterministic compliance export builder for audit + work ledger + conversation turns (`EXPORT_ACCESS_EVALUATE -> EXPORT_ARTIFACT_BUILD`)
- placement: ENTERPRISE_SUPPORT (OS-internal, simulation/policy-gated)

## B) Ownership
- Tables owned: NONE in current runtime slice (export persistence backend is abstracted behind `export_payload_ref`)
- Reads:
  - Bounded export request context (`tenant_id`, `export_scope`, `include`, `redaction_policy_ref`).
  - OS-supplied requester identity and correlation envelope.
- Writes:
  - No direct table writes in this runtime slice.
  - Emits deterministic export artifact references + tamper-evident hash output only.

## C) Hard Boundaries
- Must never export raw audio by default.
- Must never run without deterministic redaction rules when redaction is required.
- Must never bypass authorization checks for requester identity.
- Must never emit success output without `audit_event_emitted=true`.
- Must never perform side effects or engine-to-engine direct calls.

## D) Wiring
- Invoked_by: Selene OS enterprise support path.
- Inputs_from:
  - Tenant/request context from Selene OS.
  - Export scope (`work_order_id` or `time_range`).
  - Include list (`audit_events | work_order_ledger | conversation_turns`).
  - Requester identity (`requester_user_id`) + redaction policy reference.
- Outputs_to:
  - `export_access_bundle` (`export_scope_ref`, redaction-required flags).
  - `export_artifact_bundle` (`export_artifact_id`, `export_hash`, `export_payload_ref`, status/reason).
- Invocation_condition: ENTERPRISE_SUPPORT (feature/policy enabled)
- Deterministic sequence:
  - `EXPORT_ACCESS_EVALUATE`:
    - validates requester authorization and bounded scope/range.
    - validates include list + redaction requirements.
    - emits deterministic `export_scope_ref` and explicit guard flags.
  - `EXPORT_ARTIFACT_BUILD`:
    - builds deterministic tamper-evident hash + payload reference.
    - enforces raw-audio exclusion + redaction discipline.
    - emits audited success output only when status is `OK`.
- Not allowed:
  - Engine-to-engine direct calls.
  - Raw audio export by default.
  - Silent omission of required redactions.

## E) Related Engine Boundaries
- PH1.J: every successful export must emit audit proof; export output remains bounded and reason-coded.
- PH1.WORK: export scope can include work-order ledger references; canonical work-order truth remains append-only and external to PH1.EXPORT runtime.
- PH1.KMS: export payload must never include raw secret material; only opaque refs are permitted.

## F) Acceptance Tests
- AT-EXPORT-01: Export is tamper-evident (hash is stable for identical inputs).
- AT-EXPORT-02: Redaction is applied deterministically.
- AT-EXPORT-03: Export operation is audited (success output requires audit emission flag).
