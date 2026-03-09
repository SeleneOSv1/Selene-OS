# PH1.D DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.D`
- `purpose`: Persist deterministic PH1.D LLM-router outcomes (`chat`, `intent`, `clarify`, `analysis`, `fail_closed`) as bounded audit events without introducing PH1.D-owned tables.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.audit_events`
- `truth_type`: `LEDGER`
- `primary key`: `event_id`
- invariants:
  - PH1.D outcomes are recorded with `engine=PH1.D`
  - `event_type=Other` is used with explicit bounded payload keys (contract is carried by payload + `reason_code`)
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
  - one tenant binding per `device_id` for PH1.D rows
- why this read is required: fail closed before PH1.D audit writes

### Replay reads
- reads: `audit_events` by `correlation_id` and `tenant_id`
- keys/joins used: deterministic correlation chain reads
- required indices:
  - `audit_events(correlation_id, turn_id)`
  - `audit_events(tenant_id, created_at)` (or equivalent tenant filter path)
- scope rules: no cross-tenant writes; tenant attribution required
- why this read is required: deterministic replay and dedupe verification

## 4) Writes (outputs)

### Commit `chat`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload:
    - decision payload: `decision=CHAT`, `output_mode=chat`
    - request envelope: `request_id`, `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash`
    - model assignment: `model_id`, `model_route_class`, `temperature_bp`, `max_tokens`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_TIMEOUT`

### Commit `intent`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload:
    - decision payload: `decision=INTENT`, `refined_intent_type`, `output_mode=intent`
    - request envelope: `request_id`, `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash`
    - model assignment: `model_id`, `model_route_class`, `temperature_bp`, `max_tokens`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_BUDGET_EXCEEDED`

### Commit `clarify`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload:
    - decision payload: `decision=CLARIFY`, `what_is_missing`, `output_mode=clarify`
    - request envelope: `request_id`, `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash`
    - model assignment: `model_id`, `model_route_class`, `temperature_bp`, `max_tokens`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`

### Commit `analysis`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload:
    - decision payload: `decision=ANALYSIS`, `analysis_kind`, `output_mode=analysis`
    - request envelope: `request_id`, `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash`
    - model assignment: `model_id`, `model_route_class`, `temperature_bp`, `max_tokens`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`

### Commit `fail_closed`
- writes: `audit_events`
- required fields:
  - `tenant_id`, `engine=PH1.D`, `event_type=Other`, `reason_code`, `correlation_id`, `turn_id`
  - payload:
    - decision payload: `decision=FAIL_CLOSED`, `fail_code`, `output_mode=fail`
    - request envelope: `request_id`, `prompt_template_version`, `output_schema_hash`, `tool_catalog_hash`, `policy_context_hash`, `transcript_hash`
    - model assignment snapshot (if assigned before fail): `model_id?`, `model_route_class?`, `temperature_bp?`, `max_tokens?`
- idempotency_key rule (exact formula):
  - dedupe key = `(correlation_id, idempotency_key)`
- failure reason codes (minimum examples):
  - `D_FAIL_INVALID_SCHEMA`
  - `D_FAIL_FORBIDDEN_OUTPUT`
  - `D_FAIL_TIMEOUT`
  - `D_FAIL_BUDGET_EXCEEDED`

## 5) Relations & Keys

FKs used by this slice:
- `audit_events.user_id -> identities.user_id` (nullable)
- `audit_events.device_id -> devices.device_id` (nullable)
- `audit_events.session_id -> sessions.session_id` (nullable)

Unique / dedupe constraints used by this slice:
- `audit_idempotency_index_legacy(correlation_id, idempotency_key)` in storage wiring

State/boundary constraints:
- No PH1.D-owned current table in row 15 scope.
- No PH1.D migration is required for this slice.
- PH1.D remains non-authoritative; storage scope is audit-only.

## 6) Audit Emissions (PH1.J)

PH1.D writes emit PH1.J audit events with:
- `event_type`:
  - `Other` (payload-bounded PH1.D decision contract)
- `reason_code(s)`:
  - deterministic PH1.D reason codes from the PH1.D contract output/failure path
- `payload_min` keys (bounded):
  - `decision`
  - `output_mode`
  - `refined_intent_type`
  - `what_is_missing`
  - `analysis_kind`
  - `fail_code`
  - `request_id`
  - `prompt_template_version`
  - `output_schema_hash`
  - `tool_catalog_hash`
  - `policy_context_hash`
  - `transcript_hash`
  - `model_id`
  - `model_route_class`
  - `temperature_bp`
  - `max_tokens`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-PH1-D-DB-01` tenant isolation enforced
  - `at_d_db_01_tenant_isolation_enforced`
- `AT-PH1-D-DB-02` append-only enforcement for PH1.D ledger writes
  - `at_d_db_02_append_only_enforced`
- `AT-PH1-D-DB-03` idempotency dedupe works
  - `at_d_db_03_idempotency_dedupe_works`
- `AT-PH1-D-DB-04` no PH1.D current-table rebuild is required
  - `at_d_db_04_no_current_table_rebuild_required`

## 8) Provider Boundary Notes (OCR + LLM Routing)
- `PH1.D` is the only provider/model boundary for:
  - LLM interpretation assistance.
  - OCR extraction assistance for visual/document inputs.
- Upstream engines (`PH1.VISION`, `PH1.DOC`, `PH1.NLP`) must not persist direct provider-call rows.
- Provider output remains non-authoritative until:
  - PH1.D schema/policy validation succeeds, and
  - Selene OS accepts output under normal gate order.
- Optional provider snapshot payload keys (when enabled) must stay bounded:
  - `provider_id`
  - `provider_task` (`LLM_INTERPRET | OCR_TEXT_EXTRACT`)
  - `provider_confidence_bp`
  - `provider_latency_ms`
  - `provider_cost_microunits`

## 9) PH1.D Provider Adapter Contract Fields (Step-1 Lock)
Purpose:
- Define one exact runtime request/response shape for provider calls before code wiring.
- Keep all provider I/O deterministic, auditable, and fail-closed.

### 9.1 Provider Adapter Request (exact fields)
- `schema_version` (must match PH1.D provider-adapter contract version)
- `correlation_id`
- `turn_id`
- `tenant_id`
- `request_id` (derived from stable envelope hash)
- `idempotency_key` (dedupe key under correlation thread)
- `provider_task` (`LLM_INTERPRET | OCR_TEXT_EXTRACT`)
- `provider_route_class` (`PRIMARY | SECONDARY | TERTIARY`)
- `provider_id` (resolved provider token, e.g. `openai`)
- `model_id` (resolved model token)
- `timeout_ms` (bounded)
- `retry_budget` (bounded integer)
- `temperature_bp` (for LLM tasks only)
- `max_tokens` (for LLM tasks only)
- `prompt_template_version`
- `output_schema_hash`
- `tool_catalog_hash`
- `policy_context_hash`
- `transcript_hash` (required for `LLM_INTERPRET`, omitted for pure OCR tasks)
- `input_payload_ref` (opaque bounded ref to transcript/document/image payload)
- `input_payload_kind` (`TRANSCRIPT | DOCUMENT | IMAGE`)
- `input_payload_hash` (stable content hash)
- `input_payload_inline` (bounded inline payload text when provider call requires direct content input)
- `input_mime_type` (for document/image payloads)
- `safety_tier`
- `privacy_mode`
- `do_not_disturb`

### 9.2 Provider Adapter Response (exact fields)
- `schema_version`
- `correlation_id`
- `turn_id`
- `request_id`
- `idempotency_key`
- `provider_call_id` (provider-side request reference or bounded local fallback id)
- `provider_id`
- `provider_task`
- `provider_model`
- `provider_status` (`OK | TIMEOUT | BUDGET_EXCEEDED | SAFETY_BLOCK | RATE_LIMIT | PROVIDER_ERROR`)
- `provider_latency_ms`
- `provider_cost_microunits`
- `provider_confidence_bp` (optional)
- `raw_output_hash` (hash of provider raw output payload)
- `normalized_output_json` (bounded, schema-targeted output candidate)
- `validation_status` (`SCHEMA_OK | SCHEMA_FAIL | POLICY_FAIL`)
- `reason_code`

### 9.3 Hard Validation Rules
- `provider_status=OK` is necessary but not sufficient; `validation_status` must be `SCHEMA_OK`.
- Any `SCHEMA_FAIL | POLICY_FAIL` must produce PH1.D fail-closed output.
- Provider output cannot introduce disallowed keys beyond PH1.D mode contract.
- Provider output cannot bypass clarify-first, access, simulation, or audit requirements.
- Missing `provider_call_id` is allowed only when `provider_status != OK` and reason-coded.

## 10) PH1.D Provider Adapter Runtime Checklist (Strict)
Execution order:
1. Add PH1.D provider-adapter kernel contract types (request/response enums + validators).
2. Add PH1.D provider runtime trait in engines crate:
   - `trait Ph1dProviderAdapter { fn execute(req) -> ProviderAdapterResponse; }`
3. Add OpenAI adapter implementation behind environment-configured provider selection.
4. Add deterministic route policy in PH1.D:
   - task routing (`LLM_INTERPRET` vs `OCR_TEXT_EXTRACT`)
   - provider/model selection
   - timeout/retry budget enforcement.
5. Normalize provider output into PH1.D mode contract (`chat|intent|clarify|analysis|fail_closed`).
6. Enforce strict output validation and forbidden-key checks before PH1.D returns.
7. Persist provider snapshot metadata to `audit_events` bounded payload keys.
8. Add OS wiring guardrails:
   - disabled provider path -> deterministic fail-closed/not-invoked behavior.
   - invalid provider envelope -> refusal.
9. Add tests:
   - contract validation tests (request/response field discipline)
   - adapter mapping tests (`OpenAI -> ProviderAdapterResponse`)
   - fail-closed tests (timeout, schema drift, safety block, rate limit)
   - idempotency/dedupe tests
   - OCR task and LLM task path tests.
10. Add build proof entry in `docs/03_BUILD_LEDGER.md` only after tests are green.

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: none required for row 15 (`PH1.D` uses existing `audit_events`)
- tests: `crates/selene_storage/tests/ph1_d/db_wiring.rs`
