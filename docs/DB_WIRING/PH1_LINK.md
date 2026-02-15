# PH1.LINK DB Wiring Spec

## 1) Engine Header

- `engine_id`: `PH1.LINK`
- `purpose`: Persist deterministic onboarding/invite link lifecycle (`generate`, `draft_update`, `open/activate`, `revoke`, `forward-block`) with strict token->draft mapping, schema-driven missing-field computation, and deterministic binding guards.
- canonical identifier rule: canonical external identifier is `token_id`; internal draft identifier is `draft_id`; `link_url` is a transport artifact.
- ownership boundary: PH1.LINK captures selector/prefill hints only; PH1.LINK must not define, activate, or mutate requirements schema truth.
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
  - `status` is bounded (`DRAFT_CREATED | DRAFT_READY | COMMITTED | REVOKED | EXPIRED`) with deterministic monotonic transitions
  - `draft_payload_json` may include `prefilled_profile_fields` (bounded map)
  - `missing_required_fields_json` is computed deterministically from the active tenant schema version + selector snapshot (schema evaluator authority in `docs/DB_WIRING/PH1_ONB.md`)
  - deterministic idempotency keys for retriable writes

### `onboarding_link_tokens`
- `truth_type`: `CURRENT`
- `primary key`: `token_id`
- invariants:
  - authoritative mapping `token_id -> draft_id`
  - `token_signature` is required and deterministic for token verification
  - lifecycle state is bounded (`DRAFT_CREATED | SENT | OPENED | ACTIVATED | CONSUMED | REVOKED | EXPIRED | BLOCKED`)
  - device-binding and forwarded-link blocking are explicit and deterministic

### `onboarding_draft_write_dedupe`
- `truth_type`: `LEDGER`
- `primary key`: `dedupe_id`
- invariants:
  - one dedupe record per `(scope_type, scope_id, idempotency_key)`
  - retries resolve deterministically as no-op/reused result

### PH1.F in-memory lifecycle slice (runtime lock for this row)
- `links` (current-state map)
- invariants:
  - lifecycle transitions are deterministic and replay-safe
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

### Access/approval prerequisites (inviter-governed operations)
- reads: Access gate decision output (`ALLOW | DENY | ESCALATE`) via Selene OS for inviter-initiated governed operations (`generate draft`, `draft update`, `revoke`, delivery-governed send orchestration)
- required conditions:
  - governed inviter-side LINK commits execute only on `ALLOW`
  - `DENY` and `ESCALATE` are fail-closed (no governed LINK write/side effect until approval/override path resolves)
  - invitee token-holder open/activate path remains enforced by token lifecycle + device-binding guards and does not bypass Access policy for inviter-governed writes

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
  - inviter identity, invitee type, payload hash, expiry, tenant scope (when present), optional prefilled profile fields
- idempotency key rule:
  - deterministic payload hash + inviter scope dedupe

### `LINK_INVITE_DRAFT_UPDATE_COMMIT`
- writes: `onboarding_drafts` deterministic update + missing-required recompute
- required fields:
  - `draft_id`, `creator_update_fields`, idempotency key
- idempotency key rule:
  - dedupe on `(draft_id, idempotency_key)`
- preconditions:
  - draft exists and is not terminal (`COMMITTED | REVOKED | EXPIRED`)
  - linked token lifecycle is not terminal (`CONSUMED | REVOKED | EXPIRED`)
- postconditions:
  - same draft is updated only
  - `missing_required_fields_json` is recomputed from active tenant schema version + selector snapshot only

Legacy (Do Not Wire): `LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, and `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` are legacy and must not be used; link delivery is performed only by `LINK_DELIVER_INVITE` via `PH1.BCAST` + `PH1.DELIVERY`.
Delivery dedupe/idempotency is owned by `LINK_DELIVER_INVITE` via `PH1.BCAST` + `PH1.DELIVERY`.
`SENT` lifecycle marking is produced by the `LINK_DELIVER_INVITE` delivery path under Selene OS orchestration.
- PH1.LINK runtime projection for delivery success is deterministic: `DRAFT_CREATED -> SENT`.
- `SENT` projection is idempotent (replay-safe no-op when already `SENT`) and fail-closed for non-deliverable states.

### `LINK_INVITE_OPEN_ACTIVATE_COMMIT`
- writes: token status transition (`DRAFT_CREATED|SENT -> OPENED -> ACTIVATED`), or deterministic terminal passthrough (`BLOCKED|EXPIRED|REVOKED|CONSUMED`), plus bound device fingerprint hash
- required fields:
  - `token_id`, `device_fingerprint`, `idempotency_key`
- idempotency key rule:
  - idempotent on `(token_id, idempotency_key)` with deterministic replay result
- forwarded-link block input rule:
  - device mismatch during `LINK_INVITE_OPEN_ACTIVATE_COMMIT` executes `LINK_INVITE_FORWARD_BLOCK_COMMIT` exactly once as the mismatch branch.

### `LINK_INVITE_REVOKE_REVOKE`
- writes: token lifecycle transition to `REVOKED`
- required fields:
  - `token_id`, `reason`
  - `ap_override_ref` is required when token lifecycle is already `ACTIVATED` (or `OPENED`)
- idempotency key rule:
  - idempotent on `(token_id)`
- preconditions:
  - token exists
  - if token is `ACTIVATED` (or `OPENED`) and no approved AP override is present -> refuse (fail-closed)
- postconditions:
  - token is `REVOKED` and cannot be activated/consumed afterwards

### cross-engine completion transition (consumption)
- on successful onboarding completion (`PH1.ONB` complete commit path), token status is set to `CONSUMED` deterministically.

### additional locked simulations (v1 slice)
- `LINK_INVITE_FORWARD_BLOCK_COMMIT`

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
- activation/block/revoke outcomes (bounded status transitions)

## 7) Acceptance Tests (DB Wiring Proof)

- `AT-LINK-DB-01` tenant isolation enforced
  - `at_link_db_01_tenant_isolation_enforced`
- `AT-LINK-DB-02` append-only enforcement for draft-write dedupe ledger
  - `at_link_db_02_append_only_enforced`
- `AT-LINK-DB-03` idempotency dedupe works
  - `at_link_db_03_idempotency_dedupe_works`
- `AT-LINK-DB-04` current-state consistency with lifecycle transitions + token/draft mapping
  - `at_link_db_04_current_table_consistency_with_lifecycle_and_proofs`

Implementation references:
- storage wiring: `crates/selene_storage/src/ph1f.rs`
- typed repo: `crates/selene_storage/src/repo.rs`
- migration: `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`
- tests: `crates/selene_storage/tests/ph1_link/db_wiring.rs`
