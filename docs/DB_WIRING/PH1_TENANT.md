# PH1_TENANT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.TENANT
- layer: Enterprise Support
- authority: Authoritative (tenant-context decision only)
- role: Deterministic tenant/org context resolver for `tenant_id` + policy snapshot pointers (`TENANT_POLICY_EVALUATE -> TENANT_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal prerequisite path)

## B) Ownership
- Tables owned: NONE in current runtime slice (decision-only runtime)
- Reads:
  - bounded identity/session context (`identity_context`, optional `device_id`, optional `session_id`, `now`)
  - candidate tenant bindings (`tenant_id`, `policy_context_ref`, locale, disabled/policy-block flags)
  - optional explicit tenant selection token/selection
- Writes:
  - no direct table writes in this runtime slice
  - emits deterministic tenant resolution output only (`OK | NEEDS_CLARIFY | REFUSED | FAIL`)

## C) Hard Boundaries
- Must never decide permissions (Access/Authority remains external).
- Must never guess tenant scope when identity is unknown and no signed-in user context exists.
- Must never read/write across tenant boundaries.
- Must never execute workflows, tools, or simulations.
- Must remain deterministic and reason-coded.

## D) Wiring
- Invoked_by: Selene OS before enterprise lane execution and before downstream `PH1.QUOTA`/`PH1.GOV` checks.
- Inputs_from:
  - identity/session context (voice assertion or signed-in UI identity)
  - optional explicit tenant selection token
  - bounded candidate tenant bindings
- Outputs_to:
  - `tenant_policy_bundle` (identity-known signal, candidate count, deterministic selected tenant when legal)
  - `tenant_decision_bundle` (`status`, optional `tenant_id`, optional `policy_context_ref`, optional `default_locale`, `missing_fields`, `reason_code`)
- Invocation_condition: ENTERPRISE_SUPPORT (tenant resolution enabled)
- Deterministic sequence:
  - `TENANT_POLICY_EVALUATE`:
    - validates identity/session inputs and candidate bounds.
    - computes deterministic selection posture (explicit selection, single-candidate, or clarify-needed).
  - `TENANT_DECISION_COMPUTE`:
    - emits one status (`OK | NEEDS_CLARIFY | REFUSED | FAIL`).
    - enforces no-guess rule for unknown identity and no-cross-tenant invariants.
    - emits one clarify field (`tenant_choice`) when multiple tenants require user choice.
- Not allowed:
  - engine-to-engine direct calls
  - tenant inference without deterministic binding evidence
  - bypass of downstream gate ordering

## E) Related Engine Boundaries
- `PH1.VOICE.ID`: upstream identity context source for voice mode; PH1.TENANT consumes identity outcome but never performs identity binding.
- `PH1.QUOTA`: downstream quota decisions consume resolved tenant scope and policy references from PH1.TENANT.
- `PH1.GOV`: downstream governance decisions must use tenant scope resolved by PH1.TENANT.
- `PH1.X`: clarify prompts for `tenant_choice` are asked by PH1.X; PH1.TENANT never asks users directly.

## F) Acceptance Tests
- AT-TENANT-01: deterministic tenant mapping.
- AT-TENANT-02: multi-tenant requires clarify (no guessing).
- AT-TENANT-03: disabled tenant fails closed.
- AT-TENANT-04: no cross-tenant reads/writes.
