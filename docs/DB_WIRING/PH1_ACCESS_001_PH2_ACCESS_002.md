# PH1.ACCESS.001 + PH2.ACCESS.002 DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.ACCESS.001 + PH2.ACCESS.002`
- `purpose`: Persist per-user access truth (`access_instances`) plus append-only override lifecycle (`access_overrides`) while keeping PH1.ACCESS gate evaluation read-only.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `os_core.access_instances`
- `truth_type`: `CURRENT`
- `primary key`: `access_instance_id`
- invariants:
  - FK `user_id -> identities.user_id`
  - unique `(tenant_id, user_id)` and `(tenant_id, access_instance_id)`
  - `effective_access_mode` in `R | W | A | X`
  - `verification_level` in `NONE | PASSCODE_TIME | BIOMETRIC | STEP_UP`
  - `device_trust_level` in `DTL1 | DTL2 | DTL3 | DTL4`
  - `lifecycle_state` in `RESTRICTED | ACTIVE | SUSPENDED`
  - idempotent upsert dedupe on `(tenant_id, user_id, idempotency_key)`

### `os_core.access_overrides`
- `truth_type`: `LEDGER` (append-only lifecycle rows)
- `primary key`: `override_id`
- invariants:
  - FK `access_instance_id -> access_instances.access_instance_id`
  - FK `approved_by_user_id -> identities.user_id`
  - `override_type` in `ONE_SHOT | TEMPORARY | PERMANENT | REVOKE`
  - `status` in `ACTIVE | EXPIRED | REVOKED`
  - `expires_at` is `NULL` or `> starts_at`
  - idempotent append dedupe on `(tenant_id, access_instance_id, idempotency_key)`
  - no overlapping ACTIVE override rows with identical scope for one access instance
  - overwrite/delete prohibited

## 3) Reads (dependencies)

### PH2 instance reads
- reads: `access_instances` by `access_instance_id` and `(tenant_id, user_id)`
- keys/joins used: PK + unique key lookups
- required indices:
  - `access_instances(access_instance_id)` (PK)
  - `ux_access_instances_tenant_user`
  - `ux_access_instances_tenant_instance`
- scope rules: tenant+user scoped, no cross-tenant reads
- why this read is required: PH2 decisions and PH1 gate delegation must resolve one deterministic instance

### Override reads
- reads: `access_overrides` filtered by `access_instance_id` + status/time window
- keys/joins used: instance-filtered scans with active-window checks
- required indices:
  - `ix_access_overrides_instance_status_window`
  - `ux_access_overrides_active_scope`
- scope rules: per-instance scope only
- why this read is required: deterministic override conflict checks and effective-mode derivation

### Identity FK checks
- reads: `identities.user_id`
- keys/joins used: direct FK existence checks
- required indices: `identities(user_id)` (PK)
- scope rules: request-scoped identity validation
- why this read is required: fail-closed on missing actor/approval identity

## 4) Writes (outputs)

### Upsert PH2 access instance (commit)
- writes: `access_instances`
- required fields:
  - `access_instance_id`, `tenant_id`, `user_id`, `role_template_id`, `effective_access_mode`,
  - `baseline_permissions_json`, `identity_verified`, `verification_level`,
  - `device_trust_level`, `lifecycle_state`, `policy_snapshot_ref`, `created_at`, `updated_at`
- ledger event_type (if ledger): n/a (`CURRENT` row)
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, user_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_INSTANCE_MISSING` (read/gate path)
  - `ACCESS_SCOPE_MISMATCH`
  - `ACCESS_ESCALATE_REQUIRED`

### Append override lifecycle row (commit)
- writes: `access_overrides`
- required fields:
  - `override_id`, `access_instance_id`, `tenant_id`, `override_type`, `scope_json`, `status`,
  - `approved_by_user_id`, `approved_via_simulation_id`, `reason_code`,
  - `starts_at`, `expires_at`, `created_at`, `updated_at`, `idempotency_key`
- ledger event_type (if ledger): `ACCESS_OVERRIDE_*_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, access_instance_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_AP_REQUIRED`
  - `ACCESS_DEVICE_UNTRUSTED`
  - `ACCESS_SENSITIVE_DENY`

### Read-only gate decision output contract (universal)
- output fields (deterministic):
  - `access_decision` in `ALLOW | DENY | ESCALATE`
  - `escalation_trigger` (optional; includes `AP_APPROVAL_REQUIRED` and `SMS_APP_SETUP_REQUIRED`)
  - `required_approver_selector` (optional; deterministic selector payload)
  - `requested_scope` (optional; bounded)
  - `requested_duration` (optional; bounded)
- decision rule:
  - if requester is an in-tenant employee and action is approvable by AP policy, return `ESCALATE` (never silent `DENY`)
  - if `requested_action` requires SMS delivery and `sms_app_setup_complete=false`, return `ESCALATE` with `SMS_APP_SETUP_REQUIRED`
  - return `DENY` only when policy forbids an approval path

### OS-orchestrated escalation contract (design)
- Selene OS handles escalation; PH1.ACCESS/PH2.ACCESS never sends notifications.
- flow:
  - gate returns `ESCALATE` with `AP_APPROVAL_REQUIRED`
  - Selene OS uses PH1.BCAST (simulation-gated) to contact approver
  - Selene OS waits in deterministic pending state (no repeated prompts)
  - approver response is one of: `ONE_SHOT | TEMPORARY(duration) | PERMANENT | DENY`
  - Selene OS applies approval result via existing PH2 override simulations
  - Selene OS re-runs access check before any execution path
- fail-closed:
  - approver denies -> final refusal to requester with reason code
  - no override write -> no execution
  - SMS setup unresolved -> no SMS delivery execution path

## 5) Relations & Keys

FKs:
- `access_instances.user_id -> identities.user_id`
- `access_overrides.access_instance_id -> access_instances.access_instance_id`
- `access_overrides.approved_by_user_id -> identities.user_id`

Unique constraints:
- `access_instances(access_instance_id)` (PK)
- `ux_access_instances_tenant_user`
- `ux_access_instances_tenant_instance`
- `ux_access_overrides_tenant_override`
- `ux_access_overrides_tenant_instance_idempotency`
- `ux_access_overrides_active_scope`

State/boundary constraints:
- PH1.ACCESS gate path is read-only in this slice
- PH2.ACCESS owns writes to both tables
- `access_overrides` is append-only (no overwrite path)

## 6) Audit Emissions (PH1.J)

PH1.ACCESS/PH2.ACCESS writes and gate outcomes must emit PH1.J audit events with:
- `event_type`:
  - `ACCESS_DECISION`
  - `ACCESS_INSTANCE_UPSERT_COMMIT`
  - `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT`
  - `ACCESS_OVERRIDE_PERM_GRANT_COMMIT`
  - `ACCESS_OVERRIDE_REVOKE_COMMIT`
- `reason_code(s)`:
  - `ACCESS_ALLOWED`
  - `ACCESS_DENIED`
  - `ACCESS_ESCALATE_REQUIRED`
  - `ACCESS_INSTANCE_MISSING`
  - `ACCESS_SCOPE_MISMATCH`
  - `ACCESS_AP_REQUIRED`
  - `ACCESS_SENSITIVE_DENY`
  - `ACCESS_DEVICE_UNTRUSTED`
- `payload_min` allowlisted keys:
  - `access_instance_id`
  - `tenant_id`
  - `requested_action`
  - `access_decision`
  - `effective_access_mode`
  - `override_id`
  - `override_type`
  - `override_status`

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-ACCESS-DB-01` tenant isolation enforced
  - `at_access_db_01_tenant_isolation_enforced`
- `AT-ACCESS-DB-02` append-only enforcement for override lifecycle rows
  - `at_access_db_02_append_only_enforced`
- `AT-ACCESS-DB-03` idempotency dedupe works
  - `at_access_db_03_idempotency_dedupe_works`
- `AT-ACCESS-DB-04` current-table scope consistency (no PH2 current projection rebuild table in this slice)
  - `at_access_db_04_current_table_no_ledger_rebuild_required`
- `AT-ACCESS-06` ESCALATE path triggers AP approval request via PH1.BCAST and applies override only via simulation
  - `at_access_06_escalate_via_bcast_and_simulation_only`
- `AT-ACCESS-07` AP denied returns final refusal and executes no side effects
  - `at_access_07_ap_denied_final_refusal_no_side_effects`
- `AT-ACCESS-08` SMS delivery request with incomplete setup returns `ESCALATE`/`SMS_APP_SETUP_REQUIRED`
  - `at_access_08_sms_setup_required_escalate`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0009_access_instance_tables.sql`
- tests: `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs`
