# PH1_KMS DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.KMS
- layer: Enterprise Support
- authority: Authoritative (secret-material lifecycle)
- role: Secret access evaluation + opaque material issuance (`GET_HANDLE|ISSUE_EPHEMERAL|ROTATE|REVOKE`)
- placement: ENTERPRISE_SUPPORT (OS-internal, simulation-gated where applicable)

## B) Ownership
- Tables owned: NONE in current runtime slice (KMS storage backend is abstracted behind opaque refs)
- Reads:
  - Structured, OS-supplied request envelope + requester identity fields.
  - Tenant-scoped secret metadata pointers only.
- Writes:
  - No direct table writes in this runtime slice.
  - Rotation/revoke decisions emit opaque outputs only; persistence/audit commit remains OS-orchestrated.

## C) Hard Boundaries
- Must never emit raw secret material in outputs, diagnostics, or audit payloads.
- Must never allow non-authorized requester engines/users to resolve handles or rotate/revoke keys.
- Must never bypass Access + Simulation ordering.
- Must never perform tool execution or engine-to-engine direct calls.
- Rotation/revoke is deterministic and fail-closed when admin/authorization constraints are missing.

## D) Wiring
- Invoked_by: Selene OS enterprise support path.
- Inputs_from:
  - Tenant/request context from Selene OS.
  - Operation request: `GET_HANDLE | ISSUE_EPHEMERAL | ROTATE | REVOKE`.
  - Requester identity: `requester_engine_id` and optional `requester_user_id`.
- Outputs_to:
  - `kms_material_bundle` returned to Selene OS with opaque `secret_handle` or `ephemeral_credential_ref`.
  - Rotation returns incremented `rotated_version` metadata only (no secret value).
- Invocation_condition: ENTERPRISE_SUPPORT (feature/policy enabled)
- Deterministic sequence:
  - `KMS_ACCESS_EVALUATE`:
    - validates requester authorization and operation preconditions.
    - validates TTL bounds for ephemeral issuance.
    - resolves opaque `secret_ref` only.
  - `KMS_MATERIAL_ISSUE`:
    - validates operation-shape integrity against evaluated context.
    - emits opaque `secret_handle` or `ephemeral_credential_ref`.
    - emits `validation_status (OK|FAIL)` + bounded diagnostics.
  - If `validation_status != OK`, Selene OS fails closed and does not forward material outputs.
- Not allowed:
  - Engine-to-engine direct calls.
  - Raw secret value exposure in any field.
  - Any authority mutation outside simulation-gated flow.

## E) Related Engine Boundaries
- PH1.J: all KMS operations must be auditable with reason codes, but audit payloads must contain opaque refs only (never secret values).
- PH1.EXPORT: export flows may include KMS audit metadata only, never secret material.
- PH1.WORK/PH1.OS orchestration (when implemented): operation retries must remain idempotent using deterministic idempotency keys and opaque refs.

## F) Acceptance Tests
- AT-KMS-01: No secret value appears in outputs/audit-facing fields.
- AT-KMS-02: Rotation produces deterministic new version metadata and is auditable.
- AT-KMS-03: Unauthorized requester fails closed with reason code.
- AT-KMS-04: Ephemeral TTL bounds are enforced deterministically.
