# PH1.ONB DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.ONB`
- `purpose`: Persist deterministic invitee onboarding execution state transitions (`session_start`, `terms`, `schema-required evidence/approval gates`, `primary device proof`, `access instance create`, `complete`) under simulation-gated idempotent writes. PH1.ONB executes pinned requirements schemas and does not own schema definitions.
- `version`: `v1`
- `status`: `PASS`

Canonical naming note:
- In this repo, `PH1.ONB` is the single wired onboarding engine surface.
- Labels such as `PH1.ONB.CORE.001`, `PH1.ONB.ORCH`, `PH1.ONB.ORCH.001`, and `PH1.ONB.BIZ.001` are legacy/spec slices and are tracked as merged aliases, not separate runtime modules.

## 2) Data Owned (authoritative)

### `onboarding_sessions` (PH1.F current-state runtime slice for row 21)
- `truth_type`: `CURRENT`
- `primary key`: `onboarding_session_id`
- design-level required/optional fields:
  - `token_id` (required)
  - `device_fingerprint_hash` (required)
  - `prefilled_context_ref` (optional)
- invariants:
  - one deterministic session per activated link token (`token_id -> onboarding_session_id`)
  - session transitions are bounded by deterministic status machine (`DRAFT_CREATED` -> ... -> `COMPLETE`)
  - any schema-required verification gate remains blocked until required confirmation passes
  - session scope is tenant-safe when tenant scope is provided
  - resume session integrity requires `device_fingerprint_hash` match; mismatch fails closed

### PH1.ONB idempotency indexes (runtime dedupe scope for row 21)
- `onb_terms_idempotency_index`
- `onb_photo_idempotency_index`
- `onb_sender_verify_idempotency_index`
- `onb_primary_device_idempotency_index`
- `onb_access_instance_idempotency_index`
- `onb_complete_idempotency_index`
- invariants:
  - repeated retriable writes with same idempotency key return deterministic prior result
  - no duplicate side effects for access-instance creation / completion

### `tenant_companies` (PH1.ONB.BIZ dependency)
- `truth_type`: `CURRENT`
- `primary key`: `(tenant_id, company_id)`
- invariants:
  - employee onboarding access creation requires ACTIVE company prereq when employee prefilled context points to company/position scope

## 2A) Schema Ownership Boundary (Locked)

PH1.ONB is execution-only for requirements evaluation.

Deterministic boundary rules:
- PH1.ONB does not define, mutate, or activate requirements schemas.
- Requirements schema ownership is outside PH1.ONB:
  - position-linked onboarding (`invitee_type=EMPLOYEE`) reads active requirements from PH1.POSITION for the pinned `position_id`.
  - non-position onboarding types read active requirements from the schema registry selected by onboarding type.
- On session start, PH1.ONB pins the effective schema context for replay stability:
  - `pinned_schema_id`
  - `pinned_schema_version`
  - `pinned_overlay_set_id`
  - `pinned_selector_snapshot`
- PH1.ONB also computes and persists `required_verification_gates[]` from pinned schema rules.
- `missing_required_fields` is computed from pinned schema context plus current payload/resolved answers.
- ONB requirement prompts are schema-driven from pinned field specs only; no hardcoded ONB-only requirement branch is allowed.
- never ask twice: if a value exists in pinned payload context or resolved fields, PH1.ONB must not ask that field again.

## 3) Reads (dependencies)

### Link lifecycle prerequisites (from row 20 lock)
- reads: PH1.LINK current link record (`links`) + prefilled context refs
- Link validation + device binding are owned by PH1.LINK / LINK_OPEN_ACTIVATE; PH1.ONB consumes activated `token_id` context only.
- Onboarding starts only after `LINK_OPEN_ACTIVATE` succeeds; PH1.ONB does not validate link signatures/expiry/revocation/device binding.
- required conditions:
  - `token_id` exists
  - `token_id` is in `ACTIVATED` state
  - if request tenant scope + prefilled tenant scope are both present, they must match

### Position/company prereq checks (employee path)
- reads: `tenant_companies`, `positions`, position requirements schema view, link prefilled context
- required conditions:
  - company exists and is `ACTIVE`
  - position exists, belongs to same company, and is `ACTIVE`
  - compensation tier ref (when provided) matches position band ref
  - active position requirements schema version exists and can be pinned for the session

### Schema selection and pinning reads
- reads:
  - onboarding type schema registry view (for non-position types)
  - position requirements schema view (for position-linked onboarding)
  - selector hints from link payload
- required conditions:
  - exactly one deterministic active schema candidate resolves for the session
  - effective overlay merge order is deterministic and bounded

### Identity/device prerequisites
- reads: `identities`, `devices`
- required conditions:
  - primary-device proof uses a valid device reference
  - access-instance creation resolves user-scoped identity/access safely

### Access/approval gate prerequisites (governed ONB commits)
- reads: Access gate decision output (`ALLOW | DENY | ESCALATE`) via Selene OS orchestration for governed ONB commit paths
- required conditions:
  - governed ONB commits execute only on `ALLOW`
  - `DENY` and `ESCALATE` are fail-closed (no commit write or side effect until approval/override path resolves)
  - policy-routed approval flows remain OS-orchestrated (`PH1.ACCESS` + CAPREQ/AP policy path where required)

### Pinned context load rule (clarify loop source of truth)
- on session start, PH1.ONB resolves activated link context by `token_id` and pins schema context (`pinned_schema_id`, `pinned_schema_version`, `pinned_overlay_set_id`, `pinned_selector_snapshot`).
- this deterministic pinned context drives one-question-at-a-time clarify and prevents repeats.

## 4) Writes (outputs)

### `ONB_SESSION_START_DRAFT`
- writes: `onboarding_sessions` (create or deterministic reuse by activated token)
- required fields:
  - `token_id`, `device_fingerprint`, optional `prefilled_context_ref`, optional `tenant_id`
  - pinned schema context: `pinned_schema_id`, `pinned_schema_version`, `pinned_overlay_set_id`, `pinned_selector_snapshot`
  - `required_verification_gates[]` derived from pinned schema
- idempotency rule:
  - deterministic reuse by `onboarding_session_by_link` (one session per activated token)

### `ONB_TERMS_ACCEPT_COMMIT`
- writes: `onboarding_sessions` (`terms_version_id`, `terms_status`, state transition)
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`
- writes: schema-required evidence capture refs in `onboarding_sessions` (legacy capability id retained)
- gate rule: execute only when `required_verification_gates[]` includes photo evidence gate
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT`
- writes: schema-required sender confirmation decision (`CONFIRMED` / `REJECTED`) in `onboarding_sessions` (legacy capability id retained)
- gate rule: execute only when `required_verification_gates[]` includes sender confirmation gate
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
- writes: primary device proof fields in `onboarding_sessions`
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_ACCESS_INSTANCE_CREATE_COMMIT`
- writes:
  - per-user access instance through PH2 storage wiring
  - `onboarding_sessions.access_engine_instance_id` + status
- idempotency rule:
  - dedupe by `(user_id, role_id, idempotency_key)`

### `ONB_COMPLETE_COMMIT`
- writes: final onboarding status in `onboarding_sessions`
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_REQUIREMENT_BACKFILL_START_DRAFT`
- writes: backfill campaign draft state + deterministic target set snapshot
- process guard:
  - this backfill process is for `rollout_scope=CurrentAndNew` only
  - `NewHiresOnly` must not enter `ONB_REQUIREMENT_BACKFILL`
- idempotency rule:
  - dedupe by `(tenant_id, position_id, schema_version, idempotency_key)`

### `ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`
- writes: campaign/target progress state after per-recipient BCAST/REM handoff execution
- ownership rule:
  - PH1.ONB does not deliver and does not schedule reminders directly
  - PH1.BCAST owns delivery lifecycle; PH1.REM owns reminder timing mechanics
- idempotency rule:
  - dedupe by `(backfill_campaign_id, recipient_user_id, idempotency_key)`

### `ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT`
- writes: campaign terminal status and unresolved exception summary
- idempotency rule:
  - dedupe by `(backfill_campaign_id, idempotency_key)`

Backfill orchestration order (deterministic):
1. `ONB_REQUIREMENT_BACKFILL_START_DRAFT`
2. per-recipient loop:
   - BCAST draft/deliver
   - REM schedule (if policy requires follow-up timing)
   - `ONB_REQUIREMENT_BACKFILL_NOTIFY_COMMIT`
3. `ONB_REQUIREMENT_BACKFILL_COMPLETE_COMMIT`

## 5) Relations & Keys

- `onboarding_sessions.token_id` references `onboarding_link_tokens.token_id`.
- employee prereq validation joins:
  - `prefilled_context.tenant_id/company_id/position_id` -> `tenant_companies`, `positions`.
- access creation path uses PH2 access-instance scope keyed by `(tenant_id, user_id)`.

## 6) Audit/Proof Emissions

Row 21 lock relies on deterministic state transitions and idempotency proofs in PH1.F runtime state.
Append-only audit envelope discipline remains enforced through PH1.J (`audit_events` append-only invariant).

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-ONB-DB-01` tenant isolation enforced
  - `at_onb_db_01_tenant_isolation_enforced`
- `AT-ONB-DB-02` append-only enforced
  - `at_onb_db_02_append_only_enforced`
- `AT-ONB-DB-03` idempotency dedupe works
  - `at_onb_db_03_idempotency_dedupe_works`
- `AT-ONB-DB-04` current table consistency (no ONB-owned ledger rebuild in this row)
  - `at_onb_db_04_current_table_no_ledger_rebuild_required`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- tests: `crates/selene_storage/tests/ph1_onb/db_wiring.rs`
