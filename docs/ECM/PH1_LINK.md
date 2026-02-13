# PH1.LINK ECM Spec

## Engine Header
- `engine_id`: `PH1.LINK`
- `purpose`: Persist deterministic onboarding/referral link lifecycle with token->draft mapping, delivery proofs, activation binding, revoke/recovery, and forward-block guards.
- `data_owned`: `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe`, PH1.F link runtime maps/proofs
- `version`: `v1`
- `status`: `ACTIVE`

## Capability List

### `PH1LINK_INVITE_GENERATE_DRAFT_ROW`
- `name`: Generate link draft + token mapping
- `input_schema`: `(now, inviter_user_id, invitee_type, recipient_contact, delivery_method, tenant_id?, schema_version_id?, prefilled_profile_fields?, expiration_policy_id?)`
- `output_schema`: `Result<(draft_id, token_id, link_url, missing_required_fields[]), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_DRAFT_UPDATE_COMMIT_ROW`
- `name`: Commit creator draft updates and recompute missing required fields
- `input_schema`: `(now, draft_id, creator_update_fields, idempotency_key)`
- `output_schema`: `Result<(LinkRecord, missing_required_fields[]), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_GET_LINK_ROW`
- `name`: Read link lifecycle row
- `input_schema`: `token_id`
- `output_schema`: `Option<LinkRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1LINK_DELIVERY_PROOFS_FOR_LINK_ROW`
- `name`: Read delivery proof history for one link
- `input_schema`: `token_id`
- `output_schema`: `LinkDeliveryProofRecord[]`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1LINK_INVITE_SEND_COMMIT_ROW`
- `name`: Commit link send + append delivery proof
- `input_schema`: `(now, token_id, delivery_method, recipient_contact, idempotency_key)`
- `output_schema`: `Result<LinkDeliveryProofRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `legacy_note`: `PH1LINK_INVITE_SEND_COMMIT_ROW` is legacy and is not referenced by `LINK_INVITE`; `token_id` is canonical (`link_id` deprecated alias). Link delivery is handled by `PH1.BCAST` + `PH1.DELIVERY` via `LINK_DELIVER_INVITE`.

### `PH1LINK_INVITE_OPEN_ACTIVATE_COMMIT_ROW`
- `name`: Commit link open/activate with device binding
- `input_schema`: `(now, token_id, device_fingerprint)`
- `output_schema`: `Result<(token_id, draft_id, activation_status, missing_required_fields[], bound_device_fingerprint_hash, conflict_reason?, prefilled_context_ref?), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_REVOKE_REVOKE_ROW`
- `name`: Revoke link
- `input_schema`: `(token_id, reason)`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_EXPIRED_RECOVERY_COMMIT_ROW`
- `name`: Recover expired link with deterministic replacement flow
- `input_schema`: `(now, expired_token_id, delivery_method?, recipient_contact?, idempotency_key)`
- `output_schema`: `Result<LinkRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_FORWARD_BLOCK_COMMIT_ROW`
- `name`: Block forwarded-link open attempt
- `input_schema`: `(token_id, presented_device_fingerprint)`
- `output_schema`: `Result<(LinkStatus, reason?), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_DELIVERY_FAILURE_HANDLING_COMMIT_ROW`
- `name`: Commit delivery failure handling/retry proof row
- `input_schema`: `(now, token_id, attempt, idempotency_key)`
- `output_schema`: `Result<LinkDeliveryProofRecord, StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_APPEND_ONLY_GUARD`
- `name`: Guard against overwrite of link delivery proof ledger rows
- `input_schema`: `delivery_proof_ref`
- `output_schema`: `Result<(), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

## Failure Modes + Reason Codes
- PH1.LINK uses deterministic reason-coded outcomes from simulation and lifecycle guards:
  - tenant scope mismatch
  - invalid token state transition
  - forwarded-device block
  - delivery idempotency replay
- all failure paths are fail-closed and auditable.

## Audit Emission Requirements Per Capability
- every write capability must emit PH1.J with deterministic reason code and bounded payload.
- simulation-bound write paths must preserve `simulation_id`, `correlation_id`, `turn_id`, `idempotency_key` linkage.
- guard/read-only capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1LinkRepo`)
- `docs/DB_WIRING/PH1_LINK.md`
