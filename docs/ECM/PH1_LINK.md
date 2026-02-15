# PH1.LINK ECM Spec

## Engine Header
- `engine_id`: `PH1.LINK`
- `purpose`: Persist deterministic onboarding/invite link lifecycle with token->draft mapping, draft updates, schema-driven missing-field recompute, activation binding, revoke guards, and forward-block controls.
- `data_owned`: `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe`, PH1.F link runtime state
- `version`: `v1`
- `status`: `ACTIVE`
- `handoff_boundary`: PH1.LINK captures deterministic selector/prefill hints in draft payload context; PH1.ONB consumes those hints to pin schema context at onboarding session start.
- `ownership_boundary`: PH1.LINK does not own requirements schema definitions or activation; schema ownership remains outside PH1.LINK.

## Capability List

### `PH1LINK_INVITE_GENERATE_DRAFT_ROW`
- `name`: Generate link draft + token mapping
- `input_schema`: `(inviter_user_id, invitee_type, prefilled_profile_fields?, tenant_id?, schema_version_id?)`
- `output_schema`: `Result<(draft_id, token_id, link_url, missing_required_fields[]), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_DRAFT_UPDATE_COMMIT_ROW`
- `name`: Commit creator draft updates and recompute missing required fields
- `input_schema`: `(draft_id, creator_update_fields, idempotency_key)`
- `output_schema`: `Result<(draft_id, draft_status, missing_required_fields[]), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_MARK_SENT_COMMIT_ROW`
- `name`: Project successful delivery into link token lifecycle (`DRAFT_CREATED -> SENT`)
- `input_schema`: `(token_id)`
- `output_schema`: `Result<(token_id, status=SENT), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (post-delivery projection under `LINK_DELIVER_INVITE`)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_GET_LINK_ROW`
- `name`: Read link lifecycle row
- `input_schema`: `token_id`
- `output_schema`: `Option<LinkRecord>`
- `allowed_callers`: `SELENE_OS_ONLY`
- `side_effects`: `NONE`

### `PH1LINK_INVITE_OPEN_ACTIVATE_COMMIT_ROW`
- `name`: Commit link open/activate with device binding
- `input_schema`: `(token_id, device_fingerprint, idempotency_key)`
- `output_schema`: `Result<(token_id, draft_id, activation_status, missing_required_fields[], bound_device_fingerprint_hash, conflict_reason?, prefilled_context_ref?), StorageError>` where `activation_status in {ACTIVATED, BLOCKED, EXPIRED, REVOKED, CONSUMED}` (`OPENED` is an internal transient transition state)
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`
- `notes`: device mismatch path must execute one deterministic forward-block branch; no duplicate block write path outside this simulation branch.

### `PH1LINK_INVITE_REVOKE_REVOKE_ROW`
- `name`: Revoke invite token
- `input_schema`: `(token_id, reason, ap_override_ref?)` where `ap_override_ref` is required if current link status is `ACTIVATED` (or `OPENED`)
- `output_schema`: `Result<(token_id, status=REVOKED), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_EXPIRED_RECOVERY_COMMIT_ROW`
- `name`: Create replacement token for expired invite (state-only)
- `input_schema`: `(expired_token_id, idempotency_key)`
- `output_schema`: `Result<(token_id, draft_id, status=DRAFT_CREATED), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

### `PH1LINK_INVITE_FORWARD_BLOCK_COMMIT_ROW`
- `name`: Block forwarded-link open attempt
- `input_schema`: `(token_id, presented_device_fingerprint)`
- `output_schema`: `Result<(LinkStatus, reason?), StorageError>`
- `allowed_callers`: `SELENE_OS_ONLY` (simulation-gated)
- `side_effects`: `DECLARED (DB_WRITE)`

## Failure Modes + Reason Codes
- PH1.LINK uses deterministic reason-coded outcomes from simulation and lifecycle guards:
  - tenant scope mismatch
  - invalid token state transition
  - forwarded-device block
  - activated-link revoke without AP override (fail-closed)
  - idempotency replay
- all failure paths are fail-closed and auditable.

## Audit Emission Requirements Per Capability
- every write capability must emit PH1.J with deterministic reason code and bounded payload.
- simulation-bound write paths must preserve `simulation_id`, `correlation_id`, `turn_id`, `idempotency_key` linkage.
- guard/read-only capabilities emit audit only in replay/diagnostic mode.

## Sources
- `crates/selene_storage/src/repo.rs` (`Ph1LinkRepo`)
- `docs/DB_WIRING/PH1_LINK.md`

## Legacy (Do Not Wire)
- `LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, and `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` are legacy and must not be wired to PH1.LINK. Use `LINK_DELIVER_INVITE` with `PH1.BCAST` + `PH1.DELIVERY`.
