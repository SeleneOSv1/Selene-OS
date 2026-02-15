# PH1.ACCESS.001 + PH2.ACCESS.002 DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.ACCESS.001 + PH2.ACCESS.002`
- `purpose`: Persist master-access schema truth (global/tenant AP versions, overlays, board policy state) plus per-user access truth/overrides while keeping PH1.ACCESS gate evaluation deterministic and fail-closed.
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
  - compiled lineage refs are present when schema-chain compile is enabled:
    - `compiled_global_profile_id`, `compiled_global_profile_version`
    - `compiled_tenant_profile_id` (nullable), `compiled_tenant_profile_version` (nullable)
    - `compiled_overlay_set_ref` (nullable)
    - `compiled_position_id` (nullable)
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

### `os_core.access_ap_schemas_ledger`
- `truth_type`: `LEDGER` (append-only AP lifecycle/version rows)
- `primary key`: `event_id`
- invariants:
  - `ap_scope` in `GLOBAL | TENANT`
  - `lifecycle_state` in `DRAFT | ACTIVE | RETIRED`
  - `event_action` in `CREATE_DRAFT | UPDATE_DRAFT | ACTIVATE | RETIRE`
  - tenant rows require non-null `tenant_id`
  - global rows require null `tenant_id`
  - idempotent append dedupe on `(coalesce(tenant_id,'GLOBAL'), access_profile_id, schema_version_id, event_action, idempotency_key)`
  - overwrite/delete prohibited

### `os_core.access_ap_schemas_current`
- `truth_type`: `CURRENT` (projection from AP schema ledger)
- `primary key`: `(scope_key, access_profile_id)`
- invariants:
  - exactly one ACTIVE version per `(scope_key, access_profile_id)`
  - projection source must reference one ledger `event_id`
  - scope_key uses `GLOBAL` or tenant id

### `os_core.access_ap_overlay_ledger`
- `truth_type`: `LEDGER` (append-only overlay lifecycle rows)
- `primary key`: `event_id`
- invariants:
  - tenant-scoped only (`tenant_id` required)
  - `event_action` in `CREATE_DRAFT | UPDATE_DRAFT | ACTIVATE | RETIRE`
  - `overlay_ops_json` contains bounded operations only:
    - `ADD_PERMISSION`
    - `REMOVE_PERMISSION`
    - `TIGHTEN_CONSTRAINT`
    - `SET_ESCALATION_POLICY`
  - idempotent append dedupe on `(tenant_id, overlay_id, overlay_version_id, event_action, idempotency_key)`
  - overwrite/delete prohibited

### `os_core.access_ap_overlay_current`
- `truth_type`: `CURRENT` (projection from overlay ledger)
- `primary key`: `(tenant_id, overlay_id)`
- invariants:
  - exactly one ACTIVE overlay version per `(tenant_id, overlay_id)`
  - projection source must reference one overlay-ledger `event_id`

### `os_core.access_board_policy_ledger`
- `truth_type`: `LEDGER` (append-only board/approval policy lifecycle rows)
- `primary key`: `event_id`
- invariants:
  - tenant-scoped only (`tenant_id` required)
  - `policy_primitive` in `SINGLE_APPROVER | N_OF_M | BOARD_QUORUM_PERCENT | UNANIMOUS_BOARD | MIXED`
  - `required_approvals`/`approver_pool_size` satisfy `1 <= n <= m` when primitive is `N_OF_M`
  - `board_quorum_percent` in `1..100` when primitive is `BOARD_QUORUM_PERCENT`
  - idempotent append dedupe on `(tenant_id, board_policy_id, policy_version_id, event_action, idempotency_key)`

### `os_core.access_board_policy_current`
- `truth_type`: `CURRENT` (projection from board policy ledger)
- `primary key`: `(tenant_id, board_policy_id)`
- invariants:
  - exactly one ACTIVE version per `(tenant_id, board_policy_id)`
  - projection source must reference one board-policy-ledger `event_id`

### `os_core.access_board_votes_ledger`
- `truth_type`: `LEDGER` (append-only vote rows per escalation case)
- `primary key`: `vote_row_id`
- invariants:
  - tenant-scoped only (`tenant_id` required)
  - dedupe on `(tenant_id, escalation_case_id, voter_user_id, idempotency_key)`
  - vote rows immutable after append

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

### AP schema chain reads
- reads:
  - `access_ap_schemas_current` (global + tenant profile versions)
  - `access_ap_overlay_current` (tenant overlays)
- keys/joins used:
  - `(scope_key, access_profile_id)` for AP versions
  - `(tenant_id, overlay_id)` for overlays
- required indices:
  - `ux_access_ap_schemas_current_scope_profile`
  - `ux_access_ap_overlay_current_tenant_overlay`
- scope rules: tenant chain may read:
  - one global AP version
  - tenant AP version for same logical profile (if present)
  - tenant overlays only for the same tenant
- why this read is required: deterministic effective permission compile before gate decision

### Board policy + vote reads
- reads:
  - `access_board_policy_current`
  - `access_board_votes_ledger` (for active escalation cases)
- keys/joins used:
  - `(tenant_id, board_policy_id)` and `(tenant_id, escalation_case_id)`
- required indices:
  - `ux_access_board_policy_current_tenant_policy`
  - `ix_access_board_votes_tenant_case`
- scope rules: tenant-scoped only; cross-tenant board reads forbidden
- why this read is required: deterministic `N_OF_M` and board quorum threshold evaluation

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

### Append AP schema lifecycle row (draft/update/activate/retire)
- writes: `access_ap_schemas_ledger`
- required fields:
  - `event_id`, `tenant_id` (nullable for global), `access_profile_id`, `schema_version_id`,
  - `event_action`, `lifecycle_state`, `profile_payload_json`, `reason_code`,
  - `created_by_user_id`, `created_at`, `idempotency_key`
- ledger event_type (if ledger): `ACCESS_AP_SCHEMA_*`
- idempotency_key rule (exact formula):
  - dedupe key = `(coalesce(tenant_id,'GLOBAL'), access_profile_id, schema_version_id, event_action, idempotency_key)`
- failure reason codes:
  - `ACCESS_AP_SCHEMA_INVALID`
  - `ACCESS_AP_SCOPE_VIOLATION`
  - `ACCESS_AP_ACTIVATION_CONFLICT`

### Upsert AP current projection row (commit)
- writes: `access_ap_schemas_current`
- required fields:
  - `scope_key`, `access_profile_id`, `active_schema_version_id`, `active_event_id`, `updated_at`
- ledger event_type (if ledger): n/a (`CURRENT` row)
- idempotency_key rule (exact formula):
  - dedupe key = `(scope_key, access_profile_id, active_schema_version_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_AP_PROJECTION_CONFLICT`

### Append overlay lifecycle row (draft/update/activate/retire)
- writes: `access_ap_overlay_ledger`
- required fields:
  - `event_id`, `tenant_id`, `overlay_id`, `overlay_version_id`, `event_action`,
  - `overlay_ops_json`, `reason_code`, `created_by_user_id`, `created_at`, `idempotency_key`
- ledger event_type (if ledger): `ACCESS_AP_OVERLAY_*`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, overlay_id, overlay_version_id, event_action, idempotency_key)`
- failure reason codes:
  - `ACCESS_OVERLAY_OP_INVALID`
  - `ACCESS_OVERLAY_SCOPE_VIOLATION`

### Upsert overlay current projection row (commit)
- writes: `access_ap_overlay_current`
- required fields:
  - `tenant_id`, `overlay_id`, `active_overlay_version_id`, `active_event_id`, `updated_at`
- ledger event_type (if ledger): n/a (`CURRENT` row)
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, overlay_id, active_overlay_version_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_OVERLAY_PROJECTION_CONFLICT`

### Append board policy lifecycle row (draft/update/activate/retire)
- writes: `access_board_policy_ledger`
- required fields:
  - `event_id`, `tenant_id`, `board_policy_id`, `policy_version_id`, `event_action`,
  - `policy_payload_json`, `reason_code`, `created_by_user_id`, `created_at`, `idempotency_key`
- ledger event_type (if ledger): `ACCESS_BOARD_POLICY_*`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, board_policy_id, policy_version_id, event_action, idempotency_key)`
- failure reason codes:
  - `ACCESS_BOARD_POLICY_INVALID`
  - `ACCESS_BOARD_POLICY_SCOPE_VIOLATION`

### Upsert board policy current projection row (commit)
- writes: `access_board_policy_current`
- required fields:
  - `tenant_id`, `board_policy_id`, `active_policy_version_id`, `active_event_id`, `updated_at`
- ledger event_type (if ledger): n/a (`CURRENT` row)
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, board_policy_id, active_policy_version_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_BOARD_POLICY_PROJECTION_CONFLICT`

### Append board vote row (commit)
- writes: `access_board_votes_ledger`
- required fields:
  - `vote_row_id`, `tenant_id`, `escalation_case_id`, `board_policy_id`, `voter_user_id`,
  - `vote_value`, `reason_code`, `created_at`, `idempotency_key`
- ledger event_type (if ledger): `ACCESS_BOARD_VOTE_COMMIT`
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, escalation_case_id, voter_user_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_BOARD_VOTE_DUPLICATE`
  - `ACCESS_BOARD_MEMBER_REQUIRED`

### Access instance compile lineage upsert (commit)
- writes: `access_instances`
- required fields:
  - all fields required by `ACCESS_UPSERT_INSTANCE_COMMIT_ROW`
  - plus compile lineage refs:
    - `compiled_global_profile_id`, `compiled_global_profile_version`
    - `compiled_tenant_profile_id` (nullable), `compiled_tenant_profile_version` (nullable)
    - `compiled_overlay_set_ref` (nullable), `compiled_position_id` (nullable)
- ledger event_type (if ledger): n/a (`CURRENT` row)
- idempotency_key rule (exact formula):
  - dedupe key = `(tenant_id, user_id, idempotency_key)`
- failure reason codes:
  - `ACCESS_SCHEMA_REF_MISSING`
  - `ACCESS_PROFILE_NOT_ACTIVE`
  - `ACCESS_OVERLAY_REF_INVALID`

### Read-only gate decision output contract (universal)
- output fields (deterministic):
  - `access_decision` in `ALLOW | DENY | ESCALATE`
  - `escalation_trigger` (optional; includes `AP_APPROVAL_REQUIRED` and `SMS_APP_SETUP_REQUIRED`)
  - `required_approver_selector` (optional; deterministic selector payload)
  - `requested_scope` (optional; bounded)
  - `requested_duration` (optional; bounded)
- decision rule:
  - resolve effective permission chain in deterministic order:
    - global AP version
    - tenant AP version (if present)
    - tenant overlays
    - position-local bounded rules
    - active per-user overrides
  - if any required schema reference cannot be resolved, fail closed with `DENY` (`ACCESS_SCHEMA_REF_MISSING`)
  - if resolved AP/overlay/policy version is not `ACTIVE` for decision time, fail closed with `DENY` (`ACCESS_PROFILE_NOT_ACTIVE`)
  - if requester is an in-tenant employee and action is approvable by AP policy, return `ESCALATE` (never silent `DENY`)
  - if `requested_action` requires SMS delivery and `sms_app_setup_complete=false`, return `ESCALATE` with `SMS_APP_SETUP_REQUIRED`
  - if approvable action requires board or `N_OF_M` threshold and threshold is not yet met, return `ESCALATE`
  - return `DENY` only when policy forbids an approval path
- consumer enforcement rule:
  - any governed commit path in other engines must execute only when `access_decision=ALLOW`
  - `DENY` or `ESCALATE` must fail closed (no governed write/side effect until approval/override path resolves)

### LINK invite requested_action mapping
- `invitee_type=FAMILY_MEMBER | FRIEND | ASSOCIATE` -> `requested_action=INVITE_PERSONAL`
- `invitee_type=CUSTOMER` -> `requested_action=INVITE_CUSTOMER`
- `invitee_type=EMPLOYEE` -> `requested_action=INVITE_EMPLOYEE`
- `invitee_type=COMPANY` -> `requested_action=INVITE_COMPANY`

Dual AP trigger note:
- dual AP applies only to `INVITE_EMPLOYEE` when tenant policy requires it.

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
- `access_ap_schemas_current.active_event_id -> access_ap_schemas_ledger.event_id`
- `access_ap_overlay_current.active_event_id -> access_ap_overlay_ledger.event_id`
- `access_board_policy_current.active_event_id -> access_board_policy_ledger.event_id`
- `access_board_votes_ledger.board_policy_id -> access_board_policy_current.board_policy_id`

Unique constraints:
- `access_instances(access_instance_id)` (PK)
- `ux_access_instances_tenant_user`
- `ux_access_instances_tenant_instance`
- `ux_access_overrides_tenant_override`
- `ux_access_overrides_tenant_instance_idempotency`
- `ux_access_overrides_active_scope`
- `ux_access_ap_schemas_ledger_scope_profile_version_action_idem`
- `ux_access_ap_schemas_current_scope_profile`
- `ux_access_ap_overlay_ledger_tenant_overlay_version_action_idem`
- `ux_access_ap_overlay_current_tenant_overlay`
- `ux_access_board_policy_ledger_tenant_policy_version_action_idem`
- `ux_access_board_policy_current_tenant_policy`
- `ix_access_board_votes_tenant_case`

State/boundary constraints:
- PH1.ACCESS gate path is read-only in this slice
- PH2.ACCESS owns writes to both tables
- `access_overrides` is append-only (no overwrite path)
- AP/overlay/board ledger tables are append-only (no overwrite path)
- AP/overlay/board current tables are projection-only (no direct business mutation bypassing ledger events)

## 6) Audit Emissions (PH1.J)

PH1.ACCESS/PH2.ACCESS writes and gate outcomes must emit PH1.J audit events with:
- `event_type`:
  - `ACCESS_DECISION`
  - `ACCESS_INSTANCE_UPSERT_COMMIT`
  - `ACCESS_OVERRIDE_TEMP_GRANT_COMMIT`
  - `ACCESS_OVERRIDE_PERM_GRANT_COMMIT`
  - `ACCESS_OVERRIDE_REVOKE_COMMIT`
  - `ACCESS_AP_SCHEMA_CREATE_DRAFT`
  - `ACCESS_AP_SCHEMA_UPDATE_COMMIT`
  - `ACCESS_AP_SCHEMA_ACTIVATE_COMMIT`
  - `ACCESS_AP_SCHEMA_RETIRE_COMMIT`
  - `ACCESS_AP_OVERLAY_UPDATE_COMMIT`
  - `ACCESS_BOARD_POLICY_UPDATE_COMMIT`
  - `ACCESS_BOARD_VOTE_COMMIT`
  - `ACCESS_INSTANCE_COMPILE_COMMIT`
- `reason_code(s)`:
  - `ACCESS_ALLOWED`
  - `ACCESS_DENIED`
  - `ACCESS_ESCALATE_REQUIRED`
  - `ACCESS_INSTANCE_MISSING`
  - `ACCESS_SCOPE_MISMATCH`
  - `ACCESS_AP_REQUIRED`
  - `ACCESS_SENSITIVE_DENY`
  - `ACCESS_DEVICE_UNTRUSTED`
  - `ACCESS_SCHEMA_REF_MISSING`
  - `ACCESS_PROFILE_NOT_ACTIVE`
  - `ACCESS_OVERLAY_REF_INVALID`
  - `ACCESS_BOARD_POLICY_INVALID`
  - `ACCESS_BOARD_MEMBER_REQUIRED`
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
- `AT-ACCESS-09` AP schema chain missing refs fail closed with deterministic deny
  - `at_access_09_schema_ref_missing_fail_closed`
- `AT-ACCESS-10` AP lifecycle activation uniqueness enforced (single ACTIVE version per scope/profile)
  - `at_access_10_single_active_ap_version_per_scope_profile`
- `AT-ACCESS-11` overlay merge order is deterministic (global -> tenant -> overlay -> position -> override)
  - `at_access_11_overlay_merge_order_deterministic`
- `AT-ACCESS-12` board threshold evaluation deterministic for `N_OF_M` and quorum policies
  - `at_access_12_board_threshold_deterministic`
- `AT-ACCESS-13` compiled lineage refs are persisted on access instance compile
  - `at_access_13_access_instance_compile_lineage_persisted`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0009_access_instance_tables.sql`
- tests: `crates/selene_storage/tests/ph1_access_ph2_access/db_wiring.rs`
