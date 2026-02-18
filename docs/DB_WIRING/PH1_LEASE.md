# PH1_LEASE DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LEASE
- layer: Enterprise Support
- authority: Authoritative (lease ownership decision only)
- role: Deterministic lease ownership gate for WorkOrder execution (`LEASE_POLICY_EVALUATE -> LEASE_DECISION_COMPUTE`)
- placement: ENTERPRISE_SUPPORT (OS-internal coordination/control path)

## B) Ownership
- Tables owned: NONE in this runtime slice (decision-only runtime).
- Canonical persistence remains in `SELENE_OS_CORE_TABLES` (`work_order_leases` current lease state).
- Reads:
  - bounded lease request context (`tenant_id`, `work_order_id`, `lease_owner_id`, `operation`, `requested_ttl_ms`, `now`)
  - active lease snapshot (`active_lease_owner_id`, `active_lease_token`, `active_lease_expires_at`)
  - optional idempotency key for deterministic replay handoff.
- Writes:
  - no direct table writes in this runtime slice.
  - emits deterministic lease decision output only (grant/deny posture + reason code).

## C) Hard Boundaries
- At most one active lease is allowed per `(tenant_id, work_order_id)`.
- Lease ownership is explicit; renew/release requires token ownership by the current lease owner.
- Lease expiry must enable deterministic takeover from persisted ledger state only (never RAM-only takeover).
- No valid lease means no step execution for lease-gated WorkOrder paths.
- PH1.LEASE must never execute side effects, grant authority, or bypass Access/Simulation ordering.

## D) Wiring
- Invoked_by: Selene OS before lease-gated WorkOrder step execution and on lease lifecycle transitions.
- Inputs_from:
  - WorkOrder scope (`tenant_id`, `work_order_id`)
  - lease command (`ACQUIRE | RENEW | RELEASE`)
  - ownership tuple (`lease_owner_id`, optional `lease_token`)
  - bounded timing (`requested_ttl_ms`, `now`)
  - active lease snapshot from persistence boundary
- Outputs_to:
  - `lease_policy_bundle` (`lease_exists`, `lease_expired`, `owner_match`, `token_match`, `ttl_in_bounds`, `grant_eligible`)
  - `lease_decision_bundle` (`LeaseGranted | LeaseDenied`, bounded holder/expires metadata, `resume_from_ledger_required`, `reason_code`)
- Invocation_condition: ENTERPRISE_SUPPORT (lease enforcement enabled)
- Deterministic sequence:
  - `LEASE_POLICY_EVALUATE`:
    - validates token/ttl bounds and active lease snapshot coherence.
    - computes deterministic grant eligibility and reason posture.
  - `LEASE_DECISION_COMPUTE`:
    - emits exactly one decision (`LeaseGranted` or `LeaseDenied`).
    - acquire/renew grant emits active lease token + lease expiry.
    - release grant emits inactive post-state (token cleared).
    - deny emits bounded holder metadata when lease is held by another owner.
- Not allowed:
  - engine-to-engine direct calls
  - hidden lease takeover paths outside persisted state snapshot
  - non-deterministic token ownership checks

## E) Related Engine Boundaries
- `SELENE_OS_CORE_TABLES`: canonical `work_order_leases` storage state remains external and authoritative.
- `PH1.WORK`: lease decisions gate which owner is allowed to advance WorkOrder step events.
- `PH1.SCHED`: scheduler retry/wait logic is separate; PH1.SCHED does not mint or validate leases.
- `PH1.EXPORT`: compliance exports can include lease decision traces; PH1.LEASE does not build export artifacts.
- `PH1.J`: every lease decision must be reason-coded for audit integrity.

## F) Acceptance Tests
- AT-LEASE-01: one executor per WorkOrder.
- AT-LEASE-02: expired lease can be taken over safely using persisted ledger state.
- AT-LEASE-03: lease token is required for renew/release.
