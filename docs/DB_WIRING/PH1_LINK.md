# PH1.LINK DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.LINK`
- `purpose`: Persist deterministic onboarding/referral link lifecycle (`generate`, `send`, `open/activate`, `revoke`, recovery/forward-block/failure handling) with idempotent delivery proof history and strict token->draft mapping.
- `version`: `v1`
- `status`: `PASS`

## 2) Data Owned (authoritative)

### `onboarding_drafts`
- `truth_type`: `CURRENT`
- `primary key`: `draft_id`
- invariants:
  - one authoritative draft per invite flow
  - `invitee_type` is bounded (`COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE`)
  - `schema_version_id` is required for `invitee_type in (EMPLOYEE, COMPANY)` and optional for personal/customer invitee types
  - `draft_payload_json` may include `prefilled_profile_fields` (bounded map)
  - `missing_required_fields` is computed deterministically from invitee-type schema definitions in `docs/DB_WIRING/PH1_ONB.md` (Invitee Type Schemas section)
  - deterministic idempotency keys for retriable writes

### `onboarding_link_tokens`
- `truth_type`: `CURRENT`
- `primary key`: `token_id`
- invariants:
  - authoritative mapping `token_id -> draft_id`
  - lifecycle state is bounded (`DRAFT_CREATED | SENT | OPENED | ACTIVATED | CONSUMED | REVOKED | EXPIRED | BLOCKED`)
  - device-binding and forwarded-link blocking are explicit and deterministic

### `onboarding_draft_write_dedupe`
- `truth_type`: `LEDGER`
- `primary key`: `dedupe_id`
- invariants:
  - one dedupe record per `(scope_type, scope_id, idempotency_key)`
  - retries resolve deterministically as no-op/reused result

### PH1.F in-memory lifecycle slice (runtime lock for this row)
- `links` (current-state map) + `link_delivery_proofs` (append-only proof ledger)
- invariants:
  - link delivery proofs are append-only
  - resend/failure handling is idempotent
  - activated links enforce deterministic device binding; mismatch becomes `BLOCKED`

## 3) Reads (dependencies)

### Identity/session prerequisites
- reads: `identities`, `devices`
- keys/joins used: inviter identity existence + deterministic inviter tenant scope validation
- required indices:
  - `identities(user_id)` (PK)
  - `devices(device_id)` (PK)
- scope rules:
  - inviter user must exist
  - when tenant scope is provided, inviter `user_id` must match that tenant scope
- why this read is required: fail closed before draft/token creation

### Draft/token lookup
- reads: `onboarding_drafts`, `onboarding_link_tokens`
- keys/joins used: `token_id -> draft_id`
- required indices:
  - `onboarding_drafts(tenant_id, draft_id)`
  - `onboarding_link_tokens(token_id, tenant_id)`
  - `onboarding_link_tokens(draft_id, status)`
- scope rules:
  - no silent draft forking
  - open/activate/revoke always target one existing token
- why this read is required: deterministic lifecycle transitions and replay

## 4) Writes (outputs)

### `LINK_INVITE_GENERATE_DRAFT`
- writes: `onboarding_drafts`, `onboarding_link_tokens` (or equivalent PH1.F current records in MVP runtime lock)
- required fields:
  - inviter identity, invitee type, delivery method, payload hash, expiry, tenant scope (when present), optional prefilled profile fields
- idempotency key rule:
  - deterministic payload hash + inviter scope dedupe

### `LINK_INVITE_SEND_COMMIT`
- writes: token status transition (`DRAFT_CREATED -> SENT`) + delivery proof append
- required fields:
  - `delivery_method`, recipient hash, `delivery_proof_ref`, idempotency key
- idempotency key rule:
  - dedupe on `(link_id, delivery_method, recipient_contact_hash, idempotency_key)`
- legacy scope note:
  - `LINK_INVITE_SEND_COMMIT` is legacy; link delivery is performed by `PH1.BCAST` + `PH1.DELIVERY` via `LINK_DELIVER_INVITE`.

### `LINK_INVITE_OPEN_ACTIVATE_COMMIT`
- writes: token status transition to `ACTIVATED`, bound device fingerprint hash
- required fields:
  - `token_id`, presented device fingerprint hash, deterministic block path
- idempotency key rule:
  - deterministic no-op for repeated open on same bound device

### `LINK_INVITE_REVOKE_REVOKE`
- writes: token status transition to `REVOKED` with reason

### additional locked simulations (v1 slice)
- `LINK_INVITE_EXPIRED_RECOVERY_COMMIT`
- `LINK_INVITE_FORWARD_BLOCK_COMMIT`
- `LINK_DELIVERY_FAILURE_HANDLING_COMMIT`

## 5) Relations & Keys

FKs used by this slice:
- `onboarding_drafts.creator_user_id -> identities.user_id`
- `onboarding_link_tokens.draft_id -> onboarding_drafts.draft_id`

Unique / dedupe constraints used by this slice:
- `ux_onboarding_drafts_tenant_draft`
- `ux_onboarding_drafts_idempotency` (partial)
- `ux_onboarding_link_tokens_token_tenant`
- `ux_onboarding_draft_write_dedupe_scope_key`

State/boundary constraints:
- PH1.LINK is simulation-gated side-effect boundary.
- PH1.LINK must not use PH1.E for delivery.
- PH1.LINK must not bind voice identity or grant permissions.

## 6) Audit/Proof Emissions

PH1.LINK emits deterministic proof artifacts and reason-coded outcomes:
- link draft generation outcome (`payload_hash`, `expires_at`, `status`)
- delivery proof records (`delivery_proof_ref`, status, idempotency key)
- activation/block/revoke outcomes (bounded status transitions)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-LINK-DB-01` tenant isolation enforced
  - `at_link_db_01_tenant_isolation_enforced`
- `AT-LINK-DB-02` append-only enforcement for delivery proof ledger
  - `at_link_db_02_append_only_enforced`
- `AT-LINK-DB-03` idempotency dedupe works
  - `at_link_db_03_idempotency_dedupe_works`
- `AT-LINK-DB-04` current-state consistency with lifecycle + proof history
  - `at_link_db_04_current_table_consistency_with_lifecycle_and_proofs`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`
- tests: `crates/selene_storage/tests/ph1_link/db_wiring.rs`
