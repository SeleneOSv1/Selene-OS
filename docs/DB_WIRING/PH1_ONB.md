# PH1.ONB DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.ONB`
- `purpose`: Persist deterministic invitee onboarding state transitions for `PH1.ONB.CORE / PH1.ONB.ORCH / PH1.ONB.BIZ` (`session_start`, `terms`, `employee verify gates`, `primary device proof`, `access instance create`, `complete`) under simulation-gated idempotent writes.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `onboarding_sessions` (PH1.F current-state runtime slice for row 21)
- `truth_type`: `CURRENT`
- `primary key`: `onboarding_session_id`
- invariants:
  - one deterministic session per activated link (`link_id -> onboarding_session_id`)
  - session transitions are bounded by deterministic status machine (`DRAFT_CREATED` -> ... -> `COMPLETE`)
  - employee flow remains blocked until verification gates pass
  - session scope is tenant-safe when tenant scope is provided

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

## 2A) Invitee Type Schemas (Deterministic)

These schemas are used to compute `missing_required_fields` from onboarding draft payloads by `invitee_type`.

| invitee_type | required_fields | optional_fields |
|---|---|---|
| `COMPANY` | `legal_company_name`, `company_admin_name`, `company_admin_contact`, `policy_ack_required`, `schema_version_id` | `company_alias`, `billing_contact`, `notes` |
| `EMPLOYEE` | `employee_legal_name`, `employee_contact`, `company_id`, `position_id`, `schema_version_id` | `manager_contact`, `start_date`, `notes` |
| `CUSTOMER` | `customer_name`, `customer_contact` | `account_ref`, `preferred_language`, `notes` |
| `FAMILY_MEMBER` | `person_name`, `person_contact` | `relationship_label`, `preferred_language`, `notes` |
| `FRIEND` | `person_name`, `person_contact` | `relationship_label`, `preferred_language`, `notes` |
| `ASSOCIATE` | `person_name`, `person_contact` | `relationship_label`, `preferred_language`, `notes` |

Deterministic rules:
- `missing_required_fields` is always computed from the selected invitee-type schema and current draft payload.
- Never ask twice: if a value exists in the onboarding draft payload or `resolved_fields_json`, Selene OS must not ask for that field again.

## 3) Reads (dependencies)

### Link lifecycle prerequisites (from row 20 lock)
- reads: PH1.LINK current link record (`links`) + prefilled context refs
- Link validation + device binding are owned by PH1.LINK / LINK_OPEN_ACTIVATE; PH1.ONB consumes `draft_id` context only.
- required conditions:
  - `link_id` exists
  - link status is `ACTIVATED`
  - if request tenant scope + prefilled tenant scope are both present, they must match

### Position/company prereq checks (employee path)
- reads: `tenant_companies`, `positions`, link prefilled context
- required conditions:
  - company exists and is `ACTIVE`
  - position exists, belongs to same company, and is `ACTIVE`
  - compensation tier ref (when provided) matches position band ref

### Identity/device prerequisites
- reads: `identities`, `devices`
- required conditions:
  - primary-device proof uses a valid device reference
  - access-instance creation resolves user-scoped identity/access safely

## 4) Writes (outputs)

### `ONB_SESSION_START_DRAFT`
- writes: `onboarding_sessions` (create or deterministic reuse by link)
- required fields:
  - `link_id`, optional `prefilled_context_ref`, optional `tenant_id`, `device_fingerprint`
- idempotency rule:
  - deterministic reuse by `onboarding_session_by_link` (one session per link)

### `ONB_TERMS_ACCEPT_COMMIT`
- writes: `onboarding_sessions` (`terms_version_id`, `terms_status`, state transition)
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`
- writes: employee verification refs in `onboarding_sessions`
- idempotency rule:
  - dedupe by `(onboarding_session_id, idempotency_key)`

### `ONB_EMPLOYEE_SENDER_VERIFY_COMMIT`
- writes: sender decision (`CONFIRMED` / `REJECTED`) in `onboarding_sessions`
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

## 5) Relations & Keys

- `onboarding_sessions.link_id` references PH1.LINK session anchor (activated link).
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
