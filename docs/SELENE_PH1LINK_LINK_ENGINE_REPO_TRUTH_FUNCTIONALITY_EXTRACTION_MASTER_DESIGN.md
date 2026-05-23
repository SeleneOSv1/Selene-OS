# Selene PH1.LINK Link Engine — Repo-Truth Functionality Extraction Master Design

DOCUMENT STATUS:
REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution for this extraction.

This is docs-only repo-truth extraction. No runtime code was changed. This document does not authorize implementation. It reconstructs current PH1.LINK design and functionality from repo evidence.

Future implementation, refactor, retirement, or acceptance requires explicit build instruction, approved file scope, tests, backend evidence, and JD approval.

## 1. Executive Summary

PH1.LINK appears in current repo truth as Selene's governed invite/onboarding link lifecycle boundary.

The current repo supports PH1.LINK as the authoritative owner for generating deterministic invite/onboarding draft links, storing token-to-draft mappings, computing missing onboarding fields from schema/prefill context, updating draft selector/prefill hints, opening and activating invite links from app-open/deep-link context, binding first activation to a device fingerprint and app-open envelope, blocking forwarded-link activation on device mismatch, revoking eligible link tokens, recovering expired links through replacement token generation, proposing role drafts, escalating dual-role conflicts, and handing activated link context into PH1.ONB onboarding start.

PH1.LINK is not the onboarding engine. Repo truth shows PH1.ONB consumes an already activated token context and starts invited onboarding. PH1.ONB does not own token signature validation, expiry validation, revocation validation, or device binding.

PH1.LINK is not the delivery engine. Repo truth repeatedly states that invite delivery belongs to LINK_DELIVER_INVITE through PH1.BCAST and PH1.DELIVERY. Legacy send/resend/failure simulations are explicitly marked LEGACY_DO_NOT_WIRE and are absent from the current PH1.LINK kernel contract.

PH1.LINK is not an access or authority engine. It validates token lifecycle and app-open context, but it must not grant authority, bind voice identity, or bypass onboarding/access governance. Link generation is expected to be access/simulation gated. Raw link text alone does not grant protected execution authority.

PH1.LINK connects to tenant/access context through invite generation preconditions, tenant scope validation, tenant hints in app ingress, and onboarding handoff. Workspace-specific link binding was not found as a concrete current PH1.LINK feature and is marked NOT_FOUND in this extraction.

Desktop/iPhone/Adapter surfaces are boundary surfaces. The Adapter exposes `/v1/invite/click` and maps HTTP/adapter request payloads into OS app ingress. The iPhone source parses explicit-entry URLs and renders read-only route context. Native Desktop invite/deep-link handling was not found in the Mac Desktop app source, though Desktop voice E2E support code seeds and opens invite links as part of runtime proof setup.

Active current repo truth:

- PH1.LINK kernel contracts exist.
- PH1.LINK OS runtime exists.
- PH1.F-backed storage implementation exists.
- SQL migration contract exists for onboarding draft/link token tables.
- ONB activation handoff exists.
- Adapter invite-click route exists.
- Tests exist for storage, runtime, app ingress, adapter, and ONB handoff.

Partial or unclear current repo truth:

- SQL migrations exist, but current MVP wiring is documented as in-memory through PH1.F.
- Audit exists through PH1.J runtime transitions, while full SQL audit persistence is PARTIAL from current extraction.
- iPhone route parsing is render/read-only shell behavior, not activation execution.
- Desktop native invite/deep-link handling is NOT_FOUND.
- Workspace-specific link binding is NOT_FOUND.

## 2. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
| --- | --- | --- | --- | --- |
| Execution control | `AGENTS.md` | Docs-only task control, no Python, no runtime edits | FOUND | Controls this extraction task. |
| Core architecture | `docs/CORE_ARCHITECTURE.md` | `Link generation and delivery model` | FOUND | Establishes cloud-owned link generation/delivery and client non-authority. |
| Engine inventory | `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` | `PH1.LINK` row | FOUND | Defines PH1.LINK as authoritative invite link lifecycle and selector-hint capture owner. |
| Master index | `docs/SELENE_MASTER_ARCHITECTURE_BUILD_SET.md` | Architecture document list | FOUND | Current index reached document 20 before this extraction. |
| DB wiring doc | `docs/DB_WIRING/PH1_LINK.md` | `LINK_INVITE_GENERATE_DRAFT`, `LINK_INVITE_OPEN_ACTIVATE_COMMIT`, link tables | FOUND | Most complete repo-truth design doc for current PH1.LINK lifecycle. |
| ECM doc | `docs/ECM/PH1_LINK.md` | PH1.LINK capabilities, failure modes, audit requirements | FOUND | Confirms ownership boundaries and legacy do-not-wire simulations. |
| Invite blueprint | `docs/BLUEPRINTS/LINK_INVITE.md` | `LINK_INVITE`, generate/draft/update flow | FOUND | Describes invite draft generation and send-later handoff. |
| App-open activation blueprint | `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md` | `LINK_OPEN_ACTIVATE`, app-open required fields | FOUND | Describes activation, device binding, and ONB handoff. |
| Delivery blueprint | `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md` | `LINK_DELIVER_INVITE`, PH1.BCAST/PH1.DELIVERY | FOUND | Confirms PH1.LINK does not send invites. |
| Invited onboarding blueprint | `docs/BLUEPRINTS/ONB_INVITED.md` | `ONB_INVITED`, activated token context | FOUND | PH1.ONB starts after link activation. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | PH1.LINK simulations and legacy rows | FOUND | Lists active and legacy do-not-wire simulation IDs. |
| Kernel contract | `crates/selene_kernel_contracts/src/ph1link.rs` | `Ph1LinkRequest`, `LinkRecord`, `LinkStatus`, `LinkActivationResult` | FOUND | Canonical current type and validation surface. |
| Engine crate file | `crates/selene_engines/src/ph1link.rs` | N/A | NOT_FOUND | PH1.LINK implementation is not located in `selene_engines`. |
| OS runtime | `crates/selene_os/src/ph1link.rs` | `Ph1LinkRuntime`, result reason codes, audit transitions | FOUND | Runtime dispatch and PH1.J audit evidence. |
| ONB runtime | `crates/selene_os/src/ph1onb.rs` | `start_session_from_link_activation` | FOUND | Consumes activated link context and starts onboarding. |
| App ingress | `crates/selene_os/src/app_ingress.rs` | `AppInviteLinkOpenRequest`, `run_invite_link_open_and_start_onboarding` | FOUND | Combines PH1.LINK activation with PH1.ONB session start through simulations. |
| Runtime request foundation | `crates/selene_os/src/runtime_request_foundation.rs` | `/v1/invite/click`, `InviteClickCompatibility` | FOUND | Compatibility route foundation. |
| Runtime ingress turn foundation | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | `CanonicalTurnPayloadCarrier::InviteClick` | FOUND | Normalizes invite-click into bounded canonical carrier. |
| Storage trait | `crates/selene_storage/src/repo.rs` | `Ph1LinkRepo` trait methods | FOUND | Defines storage capability surface. |
| PH1.F storage | `crates/selene_storage/src/ph1f.rs` | `ph1link_*` methods, in-memory `links` | FOUND | Current MVP link lifecycle implementation. |
| SQL migration | `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` | `onboarding_drafts`, `onboarding_link_tokens`, `onboarding_draft_write_dedupe` | FOUND | SQL contract exists; current MVP runtime remains PH1.F-backed. |
| Storage tests | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | `at_link_db_*`, bad signature test | FOUND | Storage and lifecycle proof coverage. |
| Adapter library | `crates/selene_adapter/src/lib.rs` | `InviteLinkOpenAdapterRequest`, adapter runtime invite open | FOUND | Maps adapter request into OS app ingress. |
| HTTP adapter | `crates/selene_adapter/src/bin/http_adapter.rs` | `/v1/invite/click`, `run_invite_click` | FOUND | HTTP route with security envelope handling. |
| iPhone shell | `apple/iphone/SeleneIPhone/SessionShellView.swift` | `ExplicitEntryRouter`, `inviteLike`, `openLike`, `appOpenLike` | FOUND | Parses and renders explicit-entry context; does not activate locally. |
| Mac Desktop app | `apple/mac_desktop/SeleneMacDesktop` | Native invite/deep-link handler | NOT_FOUND | No current native Desktop invite/deep-link production route located. |
| Desktop voice E2E helper | `crates/selene_adapter/src/bin/desktop_voice_e2e.rs` | PH1.LINK seed/open calls | FOUND | Test/support path seeds and activates link context for E2E proof. |
| Delivery boundary scripts | `scripts/check_delivery_ownership_boundaries.sh` | Legacy LINK_* absence checks | FOUND | Enforces delivery ownership boundaries. |

## 3. Current Link Engine Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
| --- | --- | --- | --- | --- |
| Link creation | `Ph1LinkRuntime` + PH1.F storage | PH1.LINK | FOUND | `LINK_INVITE_GENERATE_DRAFT` creates draft/token/link URL. |
| Link validation | Kernel contract + PH1.F storage | PH1.LINK | FOUND | Token signature, state, expiry, idempotency, and device binding are validated. |
| Link activation | `Ph1LinkRuntime`, `app_ingress`, PH1.F storage | PH1.LINK for activation; PH1.ONB for onboarding start | FOUND | Activation and ONB start are separate simulations. |
| Link expiry | PH1.F storage | PH1.LINK | FOUND | Expired status on activation if `now > expires_at`; recovery creates replacement. |
| Link revocation | PH1.F storage + OS runtime | PH1.LINK with AP override policy integration | PARTIAL | Opened/activated revoke without AP override refuses; override support itself is not wired in current request shape. |
| Invite link | PH1.LINK docs/contracts/runtime | PH1.LINK | FOUND | Core current product function. |
| Onboarding link | PH1.LINK + PH1.ONB | PH1.LINK activation, PH1.ONB invited onboarding | FOUND | ONB consumes activated token context. |
| Enrollment link | PH1.LINK / ONB evidence | PH1.LINK plus onboarding/enrollment owners | PARTIAL | Voice enrollment link was not found as a concrete PH1.LINK-specific type. |
| Access/role binding | `LINK_ROLE_PROPOSE_DRAFT`, access docs/tests | Access/Governance + PH1.LINK proposal surfaces | PARTIAL | Role proposal exists; PH1.LINK does not grant access. |
| Tenant binding | PH1.F storage + app ingress | PH1.LINK plus Access/Governance | FOUND | Tenant scope validated on generation and app ingress. |
| Workspace binding | N/A | Access/Governance / workspace owner | NOT_FOUND | No direct workspace invite binding found in PH1.LINK current code. |
| App-open/deep-link routing | `app_ingress`, runtime foundation, Adapter, iPhone parser | PH1.LINK/PH1.L/Adapter/client boundary by role | FOUND/PARTIAL | Runtime route exists; iPhone parses only; Desktop native route not found. |
| Desktop rendering | Mac Desktop source | Desktop render/play/capture only | NOT_FOUND | No native link rendering path found. |
| iPhone rendering | `SessionShellView.swift` | iPhone render-only explicit-entry preview | FOUND | Current shell explicitly avoids activation/authority. |
| Adapter transport | `selene_adapter` | Adapter transport and security-envelope boundary only | FOUND | Adapter maps request to OS; must not own activation semantics. |
| Audit/provenance | `Ph1LinkRuntime::audit_transition`, docs | PH1.J + PH1.LINK proof | PARTIAL | Runtime audit exists; full SQL audit persistence not clearly found. |
| Storage/migrations | PH1.F + SQL migration | PH1.LINK storage owner via canonical storage | FOUND/PARTIAL | SQL migration exists; current MVP documented as in-memory. |
| Error handling | Contract validation, OS runtime reason codes, storage errors | PH1.LINK + PH1.WRITE user-safe surfacing | FOUND/PARTIAL | Machine errors exist; final user-safe wording is outside PH1.LINK. |
| Protected/private boundary | Core docs, blueprints, access gates, simulation executor | Access/Authority/Simulation + PH1.LINK lifecycle | FOUND | Link open does not equal authority. |

## 4. Current Link Lifecycle

### Stage 1 — Create Link

Owner: PH1.LINK.

Evidence:

- `docs/DB_WIRING/PH1_LINK.md`
- `docs/BLUEPRINTS/LINK_INVITE.md`
- `crates/selene_kernel_contracts/src/ph1link.rs`
- `crates/selene_os/src/ph1link.rs`
- `crates/selene_storage/src/ph1f.rs`

Inputs:

- inviter user id,
- invitee type,
- optional tenant id,
- schema version id,
- prefilled context,
- expiration policy id,
- current runtime time through request envelope.

Outputs:

- `draft_id`,
- `token_id`,
- `link_url`,
- `payload_hash`,
- `expires_at`,
- `missing_required_fields`,
- `status`,
- optional `prefilled_context_ref`.

State changes:

- PH1.F creates a `LinkRecord` in the in-memory `links` map.
- SQL contract defines `onboarding_drafts` and `onboarding_link_tokens`.
- Initial status is `DRAFT_CREATED`.

Audit evidence:

- OS runtime emits PH1.J state transition with `link_create:<token_id>`.

Gaps:

- SQL persistence is contract-defined, but current MVP wiring is documented as PH1.F-backed.

### Stage 2 — Store Link

Owner: PH1.LINK storage.

Evidence:

- `crates/selene_storage/src/repo.rs`
- `crates/selene_storage/src/ph1f.rs`
- `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`

Inputs:

- canonical link record fields,
- token signature,
- deterministic payload hash,
- expires_at,
- tenant/scope/prefill hints.

Outputs:

- retrievable `LinkRecord` by `token_id`.

State changes:

- PH1.F stores `LinkRecord`.
- Migration defines `onboarding_drafts`, `onboarding_link_tokens`, and `onboarding_draft_write_dedupe`.

Audit evidence:

- Runtime PH1.J transition exists for create/update/open/revoke/recovery/block flows.

Gaps:

- Direct SQL write/read runtime wiring is PARTIAL in this extraction because docs identify current MVP runtime as in-memory.

### Stage 3 — Deliver / Share Link

Owner: Not PH1.LINK for delivery. Delivery belongs to LINK_DELIVER_INVITE through PH1.BCAST and PH1.DELIVERY.

Evidence:

- `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md`
- `docs/DB_WIRING/PH1_LINK.md`
- `docs/ECM/PH1_LINK.md`
- `docs/08_SIMULATION_CATALOG.md`

Inputs:

- link URL generated by PH1.LINK,
- recipient contact,
- delivery method,
- delivery classification,
- requester and tenant context,
- idempotency key.

Outputs:

- delivery result from broadcast/delivery stack, not PH1.LINK.

State changes:

- PH1.LINK may support `SENT` projection through `ph1link_mark_sent_commit`, but delivery sending is not PH1.LINK-owned.

Audit evidence:

- Delivery proof belongs to PH1.BCAST / PH1.DELIVERY.

Gaps:

- Legacy `LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, and `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` are present in docs as LEGACY_DO_NOT_WIRE, not current kernel-contract capabilities.

### Stage 4 — Open Link From Client

Owner: Client captures/forwards context; Adapter transports; OS app ingress coordinates; PH1.LINK activates.

Evidence:

- `crates/selene_adapter/src/bin/http_adapter.rs`
- `crates/selene_adapter/src/lib.rs`
- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_os/src/runtime_request_foundation.rs`
- `crates/selene_os/src/runtime_ingress_turn_foundation.rs`
- `apple/iphone/SeleneIPhone/SessionShellView.swift`

Inputs:

- token id,
- token signature,
- tenant hint,
- app platform,
- device fingerprint,
- app instance id,
- deep link nonce,
- link opened timestamp,
- idempotency key.

Outputs:

- normalized invite-click carrier,
- app ingress request,
- eventual onboarding session outcome if activation and ONB start pass.

State changes:

- Client source inspected does not mutate authority.
- Adapter maps transport payload to OS request.
- PH1.LINK activation mutates token status.

Audit evidence:

- OS PH1.LINK audit transition on activation.

Gaps:

- iPhone native code currently parses and renders context; full native activation producer remains PARTIAL from current source.
- Native Desktop invite/deep-link handling is NOT_FOUND.

### Stage 5 — Validate Link

Owner: PH1.LINK.

Evidence:

- `crates/selene_kernel_contracts/src/ph1link.rs`
- `crates/selene_storage/src/ph1f.rs`

Inputs:

- token id,
- token signature,
- stored token signature,
- status,
- expiry time,
- idempotency key,
- device fingerprint,
- app-open fields.

Outputs:

- valid activation result,
- terminal status result,
- conflict reason,
- refusal/error.

State changes:

- Depending on current status, validation may transition to `OPENED`, `ACTIVATED`, `EXPIRED`, or `BLOCKED`.

Audit evidence:

- Runtime transition audit records state from/to.

Gaps:

- User-facing error language is outside PH1.LINK and should remain PH1.WRITE-owned in future reconciliation.

### Stage 6 — Activate Link

Owner: PH1.LINK.

Evidence:

- `LINK_INVITE_OPEN_ACTIVATE_COMMIT`
- `InviteOpenActivateCommitRequest`
- `LinkActivationResult`
- `Ph1LinkRuntime::invite_open_activate_commit`
- PH1.F `ph1link_invite_open_activate_commit_with_idempotency`

Inputs:

- `token_id`,
- `token_signature`,
- `device_fingerprint`,
- `app_platform`,
- `app_instance_id`,
- `deep_link_nonce`,
- `link_opened_at`,
- `idempotency_key`.

Outputs:

- `activation_status`,
- `draft_id`,
- `missing_required_fields`,
- optional `conflict_reason`,
- bound device hash,
- app-open context fields,
- optional `prefilled_context_ref`.

State changes:

- `DRAFT_CREATED` or `SENT` may move through `OPENED` to `ACTIVATED`.
- First activation binds the device fingerprint hash.
- Device mismatch blocks the token through forward-block path.

Audit evidence:

- `LINK_OK_OPEN_ACTIVATE` and state transition payload.

Gaps:

- No evidence that raw link activation grants role/authority; this is correct and should be preserved.

### Stage 7 — Bind To User / Session / Tenant / Workspace

Owner:

- PH1.LINK validates token/draft/app context.
- App ingress derives tenant context and checks simulation chain.
- PH1.ONB starts onboarding session from activated link.
- Access/Governance owns future access scope.

Evidence:

- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_os/src/ph1onb.rs`
- `docs/DB_WIRING/PH1_ONB.md`

Inputs:

- activated link result,
- tenant context,
- device/app context,
- simulation chain state,
- ONB request.

Outputs:

- onboarding session id,
- next step,
- required fields,
- required verification gates.

State changes:

- PH1.ONB creates an onboarding session draft.

Audit evidence:

- PH1.LINK audits activation.
- PH1.ONB audit/proof is separate from this extraction and not owned by PH1.LINK.

Gaps:

- Workspace binding is NOT_FOUND.
- Authority grant from link activation is NOT_FOUND and must remain disallowed unless future access/onboarding policy explicitly proves it.

### Stage 8 — Route To Onboarding / Enrollment / Access Flow

Owner: PH1.ONB / Access/Governance after PH1.LINK activation.

Evidence:

- `ONB_INVITED`
- `PH1_ONB`
- `start_session_from_link_activation`

Inputs:

- activated token context,
- draft id,
- prefilled context ref,
- tenant id,
- device fingerprint,
- app-open context.

Outputs:

- invited onboarding session.

State changes:

- PH1.ONB session state begins.

Audit evidence:

- ONB side evidence is separate.

Gaps:

- Voice enrollment link as a separate concrete PH1.LINK type is NOT_FOUND.

### Stage 9 — Expire Link

Owner: PH1.LINK.

Evidence:

- PH1.F activation logic,
- `LINK_INVITE_EXPIRED_RECOVERY_COMMIT`,
- runtime recovery implementation.

Inputs:

- current time,
- stored `expires_at`.

Outputs:

- terminal `EXPIRED` activation result,
- optional replacement token via recovery flow.

State changes:

- token status becomes `EXPIRED` when opened after expiry.
- recovery creates replacement `DRAFT_CREATED` token with extended TTL.

Audit evidence:

- OS runtime audits expired recovery.

Gaps:

- Scheduled expiry job was not found. Expiry appears enforced on activation/recovery path.

### Stage 10 — Revoke Link

Owner: PH1.LINK.

Evidence:

- `LINK_INVITE_REVOKE_REVOKE`,
- PH1.F revoke logic,
- runtime refuse behavior.

Inputs:

- token id,
- reason.

Outputs:

- `LinkRevokeResult` or refuse/error.

State changes:

- eligible token moves to `REVOKED`.
- already revoked is idempotent.

Audit evidence:

- OS runtime audits revoke success.

Gaps:

- AP override reference required for opened/activated revoke is mentioned as a contract violation field but is not present as a first-class request parameter in current kernel request.

### Stage 11 — Audit Link Use

Owner: PH1.LINK runtime + PH1.J.

Evidence:

- `Ph1LinkRuntime::audit_transition`.

Inputs:

- transition states,
- session/correlation/turn refs,
- user id or idempotency key where available.

Outputs:

- PH1.J audit event with `AuditEngine::Other("ph1_link")` and type `StateTransition`.

State changes:

- Audit state is external to link record.

Gaps:

- Full audit table linkage for failed activation and old-route compatibility is DESIGN_GAP from current evidence.

### Stage 12 — Fail / Deny Invalid Link

Owner: PH1.LINK / app ingress / adapter security depending failure point.

Evidence:

- storage validation,
- app ingress tenant/simulation chain checks,
- HTTP adapter security envelope tests,
- bad signature tests.

Inputs:

- malformed token/signature,
- expired/revoked/consumed/blocked status,
- wrong tenant,
- missing app-open context,
- device mismatch,
- simulation inactive,
- security envelope failure.

Outputs:

- refusal/error,
- blocked terminal result,
- HTTP 401/400 at adapter boundary where applicable.

State changes:

- Device mismatch can set `BLOCKED`.
- Expired can set `EXPIRED`.
- Security envelope rejection at adapter does not mutate link state.

Audit evidence:

- Runtime audits successful deterministic state transitions; full failed-at-boundary audit is PARTIAL.

## 5. Data Model / Contracts / Packets

### Request Structs

| Struct / Request | File | Status | Notes |
| --- | --- | --- | --- |
| `Ph1LinkRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Envelope with schema version, correlation id, turn id, now, simulation id/type, and request enum. |
| `LinkRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Variants for generate, update, open activate, revoke, recovery, forward block, role propose, dual-role escalation. |
| `InviteGenerateDraftRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Inviter, invitee type, tenant, schema version, prefilled context, expiration policy. |
| `InviteDraftUpdateCommitRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Draft update fields plus idempotency. |
| `InviteOpenActivateCommitRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Token/signature/device/app-open activation request. |
| `InviteRevokeRevokeRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Token and reason. |
| `InviteExpiredRecoveryCommitRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Token and optional idempotency key. |
| `InviteForwardBlockCommitRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Token and presented device fingerprint. |
| `RoleProposeDraftRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Role proposal draft. |
| `DualRoleConflictEscalateDraftRequest` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Dual-role escalation draft. |
| `AppInviteLinkOpenRequest` | `crates/selene_os/src/app_ingress.rs` | FOUND | App-open request passed to OS app ingress. |
| `InviteLinkOpenAdapterRequest` | `crates/selene_adapter/src/lib.rs` | FOUND | Adapter-facing request mapped to app ingress. |

### Response Structs

| Struct / Response | File | Status | Notes |
| --- | --- | --- | --- |
| `Ph1LinkResponse` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | `Ok` or `Refuse`. |
| `Ph1LinkOk` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Exactly one result branch is expected. |
| `Ph1LinkRefuse` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Refusal with reason and optional field. |
| `LinkGenerateResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Draft/token/link URL/missing fields/expires/status output. |
| `LinkDraftUpdateResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Draft status and missing fields after update. |
| `LinkActivationResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Activation status, conflict reason, bound device, app context. |
| `LinkRevokeResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Revoked status and reason. |
| `LinkExpiredRecoveryResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Replacement token/link URL. |
| `RoleProposalResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Pending AP approval role proposal. |
| `DualRoleConflictEscalationResult` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Escalation result. |
| `AppInviteLinkOpenOutcome` | `crates/selene_os/src/app_ingress.rs` | FOUND | Onboarding session outcome after invite click. |

### Records

| Record | File | Status | Notes |
| --- | --- | --- | --- |
| `LinkRecord` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Canonical token/draft/status/app context record. |
| `PrefilledContext` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Policy-safe selector/prefill context. |
| PH1.F `links` map | `crates/selene_storage/src/ph1f.rs` | FOUND | Current in-memory storage surface. |
| `onboarding_drafts` | `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` | FOUND | SQL contract table for drafts. |
| `onboarding_link_tokens` | `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` | FOUND | SQL contract table for tokens. |
| `onboarding_draft_write_dedupe` | `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql` | FOUND | SQL contract table for idempotency. |

### Enums and Status States

| Enum / State | File | Status | Values / Notes |
| --- | --- | --- | --- |
| `SimulationType` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | `Draft`, `Commit`, `Revoke`. |
| `InviteeType` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Company, Customer, Employee, FamilyMember, Friend, Associate. |
| `AppPlatform` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Ios, Android, Tablet, Desktop. Docs sometimes list IOS/ANDROID for app-open minimum. |
| `LinkStatus` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | DraftCreated, Sent, Opened, Activated, Consumed, Expired, Revoked, Blocked. |
| `DraftStatus` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | DraftCreated, DraftReady. |
| `RoleProposalStatus` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | PendingApApproval. |
| `EscalationStatus` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Escalated. |

### IDs, Tokens, Hashes, and Signatures

| Contract | File | Status | Notes |
| --- | --- | --- | --- |
| `TokenId` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Non-empty, bounded ASCII. |
| `DraftId` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Non-empty, bounded ASCII. |
| `PrefilledContextRef` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Non-empty, bounded ASCII. |
| token signature format | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Must match `v1.<key_id>.<digest>` shape. |
| `deterministic_payload_hash_hex` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Deterministic payload hash helper. |
| `deterministic_contact_hash_hex` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Deterministic contact hash helper. |
| `deterministic_device_fingerprint_hash_hex` | `crates/selene_kernel_contracts/src/ph1link.rs` | FOUND | Deterministic device hash helper. |

### Error Types and Reason Codes

| Reason / Error | File | Status | Notes |
| --- | --- | --- | --- |
| `LINK_OK_GENERATE_DRAFT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_DRAFT_UPDATE_COMMIT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_OPEN_ACTIVATE` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_REVOKE` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_EXPIRED_RECOVERY_COMMIT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_FORWARD_BLOCK_COMMIT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_ROLE_PROPOSE_DRAFT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_OK_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime success reason. |
| `LINK_REFUSE_INVALID` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime refusal reason. |
| `LINK_REFUSE_NOT_FOUND` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime refusal reason. |
| `LINK_REFUSE_NOT_IMPLEMENTED` | `crates/selene_os/src/ph1link.rs` | FOUND | Runtime refusal reason. |
| `FORWARDED_LINK_DEVICE_MISMATCH` | `crates/selene_storage/src/ph1f.rs` | FOUND | Conflict reason for forwarded link block. |
| `TOKEN_EXPIRED`, `TOKEN_REVOKED`, `TOKEN_CONSUMED`, `TOKEN_BLOCKED` | `crates/selene_storage/src/ph1f.rs` | FOUND | Terminal activation conflict reasons. |

### Packet Mapping Status

Repo truth has concrete structs rather than separately named packets for link lifecycle. The equivalents are:

| Logical Packet Need | Current Equivalent | Status |
| --- | --- | --- |
| Link create request packet | `InviteGenerateDraftRequest` inside `Ph1LinkRequest` | EQUIVALENT_FOUND |
| Link activation request packet | `InviteOpenActivateCommitRequest`, `AppInviteLinkOpenRequest`, `InviteLinkOpenAdapterRequest` | EQUIVALENT_FOUND |
| Link activation result packet | `LinkActivationResult` | EQUIVALENT_FOUND |
| Link record packet | `LinkRecord` | EQUIVALENT_FOUND |
| Link audit packet | PH1.J `AuditEntry` emitted by `audit_transition` | PARTIAL |
| Link compatibility route packet | `CanonicalTurnPayloadCarrier::InviteClick` | EQUIVALENT_FOUND |

## 6. Link Types and Product Functions

| Link Type / Function | Evidence Found | Current Behavior | Owner | Security Implications | Missing Design |
| --- | --- | --- | --- | --- | --- |
| Invite link | Strong | Generates draft/token/link URL; can be opened and activated. | PH1.LINK | Must be simulation/access gated for generation; activation does not grant authority. | Final reconciliation with access templates. |
| Onboarding link | Strong | Activated link starts PH1.ONB invited onboarding. | PH1.LINK + PH1.ONB | App-open/device binding required before ONB session start. | Full SQL-backed proof if runtime migrates from PH1.F. |
| Enrollment link | Partial | Onboarding/enrollment relation exists, but separate enrollment link type not found. | PH1.LINK / Onboarding future owner | Must not bind identity or voice profile from raw link. | DESIGN_GAP. |
| Device link | Partial | Device fingerprint binding exists on activation. | PH1.LINK | First device binds; mismatch blocks. | No standalone device link product type found. |
| App-open/deep-link | Strong | `/v1/invite/click`, app-open fields, iPhone parser. | PH1.LINK/OS app ingress/Adapter/client by boundary | Client context is evidence; runtime decides. | Desktop native path NOT_FOUND. |
| Workspace join link | Not found | No concrete workspace link type located. | Future Access/Workspace owner | Workspace authority must not be inferred from invite link. | NOT_FOUND. |
| Tenant link | Partial/strong | Tenant id/prefill context checked; tenant mismatch fails. | PH1.LINK + Access/Governance | Tenant scope is validated; user authority still separate. | Workspace/role template integration needs reconciliation. |
| Access grant link | Partial | Role proposal draft exists; access grant execution not PH1.LINK-owned. | Access/Governance/Authority | Link must not grant authority. | DESIGN_GAP for final role/access flow. |
| Voice enrollment link | Not found | No concrete PH1.LINK-specific voice enrollment link type located. | Future Voice ID/Onboarding owner | Voice identity must remain evidence-only. | NOT_FOUND. |
| Document/artifact link | Not found in PH1.LINK | Core architecture mentions generated links as artifacts generally. | Artifact owner / delivery owner | Must be cloud-owned and access scoped. | NOT_FOUND in PH1.LINK. |
| Support/recovery link | Partial | Expired invite recovery exists. | PH1.LINK | Recovery cannot revive revoked tokens. | No broader account recovery link found. |

## 7. Access / Identity / Tenant / Workspace Interaction

PH1.LINK interacts with the Identity + Access + Authority Spine, but it does not replace it.

Repo-supported interactions:

- Link generation accepts an inviter identity and optional tenant.
- PH1.F validates inviter/tenant scope using deterministic checks.
- Prefilled context may carry tenant/company/position/location/start-date selector hints.
- App ingress derives tenant context from request and/or stored link context.
- Tenant mismatch fails closed.
- App ingress requires active simulations for `LINK_INVITE_OPEN_ACTIVATE_COMMIT` and `ONB_SESSION_START_DRAFT`.
- PH1.ONB receives activated token context and starts onboarding.
- Docs state PH1.LINK must not bind voice identity or grant permissions.

Repo-supported access limits:

- Raw link text alone is insufficient for protected execution.
- Token signature must validate.
- Token state must be valid.
- App-open context must be present for activation handoff.
- Device fingerprint hash is bound on first activation.
- Forwarded device mismatch blocks.
- Onboarding must proceed through PH1.ONB and its required fields/gates.

Current evidence on authority:

- PH1.LINK can create and activate link lifecycle records.
- PH1.LINK cannot approve payroll, grant role authority, bind a user to a protected role, or mutate protected business state from raw link text.
- `LINK_ROLE_PROPOSE_DRAFT` exists as proposal/draft behavior, not direct authority.

Workspace interaction:

- Direct workspace invite/join binding was NOT_FOUND in current PH1.LINK repo truth.

Critical future rule:

Opening a link must not automatically grant authority unless deterministic access/onboarding policy says so and the correct canonical owner executes that grant. Current repo evidence aligns with this rule for protected authority, but final access-template integration remains pending Grand Architecture Reconciliation.

## 8. Desktop / iPhone / Adapter Boundaries

### Desktop

Current behavior found:

- No native Mac Desktop invite/deep-link handling path was found in `apple/mac_desktop/SeleneMacDesktop`.
- `crates/selene_adapter/src/bin/desktop_voice_e2e.rs` seeds and activates a link as support setup for Desktop voice E2E proof.

Runtime/client split:

- Desktop native production link route is NOT_FOUND.
- Desktop E2E support code is not evidence of Desktop semantic ownership.

Risk marker:

- DESKTOP_LINK_AUTHORITY_RISK: NOT_FOUND for current production route, but future implementation must keep Desktop render/open/capture-only.

### iPhone

Current behavior found:

- `SessionShellView.swift` parses explicit-entry URLs.
- It recognizes invite/open/app-open style contexts using `inviteLike`, `openLike`, and `appOpenLike` route logic.
- It extracts token hints, tenant hints, nonce, app instance, device fingerprint, route kind, onboarding session preview fields, blocking fields, and receipt refs.
- The UI states that the shell parsed explicit-entry context only and does not activate invites, complete onboarding, bind tokens, or alter cloud authority.

Runtime/client split:

- iPhone currently provides route parsing/rendering evidence.
- Link activation remains runtime-owned.

Risk marker:

- IPHONE_LINK_AUTHORITY_RISK: LOW in current source because shell is explicit about read-only behavior; future native activation producer must stay transport/evidence-only.

### Adapter

Current behavior found:

- HTTP Adapter exposes `/v1/invite/click`.
- Adapter validates request security envelope at HTTP boundary.
- Adapter maps request payload into OS `AppInviteLinkOpenRequest`.
- Adapter calls OS app ingress, which executes PH1.LINK activation and PH1.ONB start through simulation executor.

Runtime/client split:

- Adapter currently transports and maps request shape.
- Adapter does not own token lifecycle, onboarding state, or authority.

Risk marker:

- ADAPTER_LINK_AUTHORITY_RISK: PARTIAL boundary risk because Adapter invokes the OS app-ingress method and must not accumulate semantic/authority logic over time.

### Compatibility Paths

Current compatibility surfaces:

- `/v1/invite/click`
- `InviteClickCompatibility`
- `CanonicalTurnPayloadCarrier::InviteClick`
- iPhone `inviteLike`, `openLike`, `appOpenLike`

These should remain compatibility/transport surfaces until proof-based migration or retirement.

## 9. Security Model

### Token Handling

Repo truth:

- `TokenId` is bounded ASCII.
- Token signature is required and must match `v1.<key_id>.<digest>` shape.
- Storage generates deterministic token signatures.
- Activation validates presented signature against stored token signature.
- Bad signature tests exist.

Status: FOUND.

### Expiry

Repo truth:

- Default runtime TTL is 7 days.
- `expires_at` is present in `LinkRecord`.
- Activation after expiry returns/sets `EXPIRED`.
- Expired recovery can create a replacement token.

Status: FOUND.

Gap:

- No scheduled expiry sweep was found. Expiry appears activation/recovery-path enforced.

### Revocation

Repo truth:

- `REVOKED` status exists.
- Revoke requires reason.
- Opened/activated revoke without AP override refuses.
- Consumed token cannot be revoked.

Status: FOUND/PARTIAL.

Gap:

- AP override reference is not found as a complete current request field.

### One-Time Use / Replay Protection

Repo truth:

- Activation idempotency is keyed by `(token_id, idempotency_key)`.
- Draft updates are idempotent by `(draft_id, idempotency_key)`.
- Expired recovery is idempotent.
- Forward-block repeated mismatch is deterministic/idempotent.
- Token status includes `CONSUMED`, and docs state successful ONB completion consumes token.

Status: FOUND/PARTIAL.

Gap:

- Consumption path is documented; direct PH1.LINK consume implementation was not fully extracted as a separate explicit capability in this task.

### Tenant / Workspace Scope

Repo truth:

- Tenant validation exists for inviter/user prefix and prefilled context.
- App ingress requires tenant scope from request or link and rejects mismatch.

Status: FOUND for tenant.

Workspace status: NOT_FOUND.

### Role / Access Scope

Repo truth:

- Role proposal draft exists.
- Link docs say PH1.LINK must not grant permissions.
- Access gate appears in blueprints.

Status: PARTIAL.

Gap:

- Full Access Template / role permission integration remains pending reconciliation.

### Audit

Repo truth:

- Runtime PH1.J state transition audit exists for PH1.LINK runtime writes.
- DB/ECM docs require bounded audit/proof.

Status: PARTIAL.

Gap:

- Failed activation audit and old-route compatibility audit are not proven as complete across every boundary.

### Invalid / Expired / Revoked Link Handling

Repo truth:

- Invalid/malformed/signature mismatch fails.
- Expired/revoked/consumed/blocked terminal statuses return bounded results.
- Device mismatch blocks.
- Adapter security failure returns unauthorized before runtime mutation.

Status: FOUND.

### No Authority From Raw Link Text Alone

Repo truth:

- Core architecture and PH1.LINK docs support this.
- Runtime flow requires token signature, state validation, app-open context, simulation chain, and ONB handoff.

Status: FOUND.

## 10. Link Engine State Machine

State machine status: RECONSTRUCTED_FROM_REPO_EVIDENCE.

Actual states found in `LinkStatus`:

- `DRAFT_CREATED`
- `SENT`
- `OPENED`
- `ACTIVATED`
- `CONSUMED`
- `EXPIRED`
- `REVOKED`
- `BLOCKED`

Draft states found in `DraftStatus`:

- `DRAFT_CREATED`
- `DRAFT_READY`

Reconstructed token lifecycle:

1. `DRAFT_CREATED`
   - Created by `LINK_INVITE_GENERATE_DRAFT`.
   - Can be marked `SENT`.
   - Can be updated through draft update.
   - Can be opened/activated.
   - Can be revoked.
   - Can expire.

2. `SENT`
   - Projection indicating delivery handoff occurred elsewhere.
   - Can be opened/activated.
   - Can be revoked.
   - Can expire.

3. `OPENED`
   - Activation path may pass through opened.
   - First device binding occurs on open/activation.
   - Opened revoke requires AP override.

4. `ACTIVATED`
   - Link is valid and app-open context was accepted.
   - PH1.ONB may start invited onboarding from this result.
   - Activated revoke requires AP override.
   - Activated does not equal protected authority.

5. `CONSUMED`
   - Docs state successful ONB completion consumes token.
   - Consumed token cannot be revoked.
   - Activation returns terminal consumed conflict.

6. `EXPIRED`
   - Activation after expiry returns expired state.
   - Expired recovery can create replacement token.

7. `REVOKED`
   - Terminal revoked state.
   - Recovery cannot recover revoked token.

8. `BLOCKED`
   - Forwarded-link device mismatch or block path.
   - Terminal blocked state for activation.

Additional runtime process states inferred but not explicit `LinkStatus` values:

- `VALIDATING`: runtime step, not an explicit stored state.
- `DENIED`: refusal/error path, not an explicit stored `LinkStatus`.
- `FAILED`: error path, not an explicit stored `LinkStatus`.

These inferred states must not be claimed as implemented stored states without future repo proof.

## 11. Error Handling and Reason Codes

| Scenario | Current Evidence | Current Handling | Status |
| --- | --- | --- | --- |
| Invalid link | Contract/storage validation | Runtime/storage error or refusal | FOUND |
| Expired link | Expiry check against `expires_at` | Returns `EXPIRED` / conflict `TOKEN_EXPIRED` | FOUND |
| Revoked link | Terminal status | Returns `REVOKED` / conflict `TOKEN_REVOKED` | FOUND |
| Wrong tenant/workspace | App ingress tenant mismatch | Fails closed for tenant mismatch | FOUND for tenant, NOT_FOUND for workspace |
| Unauthorized activation | Simulation chain inactive / access gate docs | App ingress fails when simulations inactive | PARTIAL |
| Already used | `CONSUMED` status | Returns consumed terminal conflict | FOUND |
| Malformed token/signature | Contract/signature validation | Bad signature fails closed | FOUND |
| Onboarding incomplete | ONB required fields/gates | ONB outcome includes required fields/gates | PARTIAL, PH1.ONB-owned |
| Access denied | Blueprints/access docs | Access gate before generation/delivery | PARTIAL |
| Session mismatch | Runtime app ingress/session context | Bounded request context; no full mismatch matrix extracted | PARTIAL |
| Client route mismatch | iPhone route parser + adapter route validation | Render-only parsing / HTTP route validation | PARTIAL |
| Missing app-open context | DB wiring / contract validation | Activation requires app context for activated handoff | FOUND |
| Device mismatch | PH1.F forward-block | Sets/returns `BLOCKED` with conflict | FOUND |
| Opened/activated revoke without AP override | PH1.F + OS runtime | Refuses with invalid reason/field | FOUND/PARTIAL |

## 12. Audit / Provenance / Evidence

### Link Creation Audit

Status: FOUND/PARTIAL.

Evidence:

- `Ph1LinkRuntime::audit_transition` emits PH1.J state transition for newly created links.
- Audit reason includes `link_create:<token_id>`.

Gap:

- SQL audit persistence was not identified as a link-specific table in current extraction.

### Link Activation Audit

Status: FOUND.

Evidence:

- OS runtime audits transition around open activation.
- Payload contains `state_from` and `state_to`.
- Runtime includes correlation, turn, user/idempotency references where available.

### Failed Activation Audit

Status: PARTIAL / DESIGN_GAP.

Evidence:

- Device mismatch creates deterministic block transition, which can be audited.
- Adapter security envelope rejection is tested but not proven as PH1.LINK audit.

Gap:

- Not every failed-at-boundary case has link-specific PH1.J proof in extracted evidence.

### Client / Device / Session Refs

Status: FOUND.

Evidence:

- Activation records bound device fingerprint hash, app platform, app instance id, deep link nonce, and link opened timestamp.
- Runtime/app ingress carries correlation and idempotency.

### Tenant / Workspace / User Refs

Status: FOUND for tenant/user; NOT_FOUND for workspace.

Evidence:

- `inviter_user_id`, tenant in prefilled context, tenant in app ingress.

### Old Route Compatibility Events

Status: PARTIAL / DESIGN_GAP.

Evidence:

- Runtime ingress foundation normalizes invite-click compatibility into canonical carrier.

Gap:

- Dedicated audit for old compatibility route usage was not found.

## 13. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
| --- | --- | --- | --- | --- |
| `at_link_db_01_tenant_isolation_enforced` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Tenant isolation at storage layer | Full Access Template integration | FOUND |
| `at_link_db_02_append_only_enforced` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Lifecycle consistency constraint | SQL runtime production wiring | FOUND |
| `at_link_db_03_idempotency_dedupe_works` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Idempotency dedupe | Cross-device app flow | FOUND |
| `at_link_db_04_current_table_consistency_with_lifecycle_and_proofs` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Current lifecycle/proof consistency | Full UI proof | FOUND |
| `at_link_db_05_draft_update_success_and_idempotent_replay` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Draft update and replay | PH1.WRITE response wording | FOUND |
| `at_link_db_06_draft_update_refused_for_invalid_state` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Invalid state refusal | AP override flow | FOUND |
| `at_link_db_07_revoke_refused_for_activated_without_ap_override` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Activated revoke refusal | Override execution path | FOUND |
| `at_link_db_08_revoke_allows_non_activated_state` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Eligible revoke | UI proof | FOUND |
| `at_link_db_09_open_activate_idempotency_replay_behavior` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Activation replay behavior | Multi-client live proof | FOUND |
| `at_link_db_10_forward_block_deterministic_single_path` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Device mismatch block path | Full security audit matrix | FOUND |
| `at_link_db_11_missing_required_fields_recompute_is_schema_driven` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Schema-driven missing fields | Future schema owner reconciliation | FOUND |
| `at_link_db_12_draft_update_row_method_is_idempotent` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Row-method idempotency | SQL production wiring | FOUND |
| `at_link_db_13_open_activate_row_with_idempotency_replays_by_key` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Row-method activation replay | Native client proof | FOUND |
| `run1_link_open_activate_fails_closed_for_bad_token_signature` | `crates/selene_storage/tests/ph1_link/db_wiring.rs` | Bad signature fail-closed | All malformed URL cases | FOUND |
| `at_link_01_generate_draft_is_idempotent_and_hash_deterministic` | `crates/selene_os/src/ph1link.rs` | Runtime deterministic create | SQL persistence | FOUND |
| `at_link_03_open_binds_device_and_blocks_mismatch` | `crates/selene_os/src/ph1link.rs` | Device binding/blocking | iPhone native execution | FOUND |
| `at_link_04_draft_update_commit_runtime_is_idempotent_and_advances_status` | `crates/selene_os/src/ph1link.rs` | Runtime draft update | Full schema owner activation | FOUND |
| `at_link_05_revoke_returns_refuse_for_activated_without_ap_override` | `crates/selene_os/src/ph1link.rs` | Runtime revoke refusal | Override success path | FOUND |
| `at_link_06_expired_recovery_creates_replacement_and_is_idempotent` | `crates/selene_os/src/ph1link.rs` | Expired recovery | Scheduled expiration | FOUND |
| `at_link_07_forward_block_commit_records_block_attempt` | `crates/selene_os/src/ph1link.rs` | Forward block commit | Full audit matrix | FOUND |
| `at_link_08_role_propose_is_idempotent` | `crates/selene_os/src/ph1link.rs` | Role proposal idempotency | Access grant | FOUND |
| `at_link_09_dual_role_conflict_escalate_is_idempotent` | `crates/selene_os/src/ph1link.rs` | Dual role escalation idempotency | Resolution workflow | FOUND |
| `onb_link_activation_handoff_uses_link_app_context` | `crates/selene_os/src/ph1onb.rs` | Activated link app context reaches ONB | Native client UI | FOUND |
| `onb_link_activation_handoff_refuses_non_activated_status` | `crates/selene_os/src/ph1onb.rs` | ONB refuses non-activated link | Full user-facing denial wording | FOUND |
| `run1_invite_link_click_starts_onboarding_with_active_simulations` | `crates/selene_os/src/app_ingress.rs` | Invite click starts ONB with active sims | End-user live proof | FOUND |
| `run1_invite_link_click_fails_closed_when_onboarding_simulation_not_active` | `crates/selene_os/src/app_ingress.rs` | Simulation-chain fail-closed | All access-denied variants | FOUND |
| `run1_invite_click_adapter_starts_onboarding_without_turn_or_client_time_inputs` | `crates/selene_adapter/src/lib.rs` | Adapter maps invite click to OS safely | Browser/iPhone live proof | FOUND |
| `run1_invite_click_adapter_fails_closed_for_bad_signature` | `crates/selene_adapter/src/lib.rs` | Adapter path preserves bad signature fail-closed | All adapter security cases | FOUND |
| `ingress_invite_click_without_bearer_returns_401` | `crates/selene_adapter/src/bin/http_adapter.rs` | HTTP security envelope rejection | PH1.LINK audit for rejected HTTP request | FOUND |
| `ingress_iphone_invite_onboarding_and_explicit_voice_turn_e2e` | `crates/selene_adapter/src/bin/http_adapter.rs` | iPhone-style invite/onboarding/voice route integration | Native iPhone app proof | FOUND |

Current test gap:

- TEST_GAP: No JD live invite/open acceptance proof was extracted.
- TEST_GAP: No native Mac Desktop invite/deep-link proof was found.
- TEST_GAP: Full SQL persistence runtime proof remains partial because current MVP notes PH1.F/in-memory wiring.

## 14. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
| --- | --- | --- | --- | --- |
| `LINK_INVITE_SEND_COMMIT` | LEGACY_DO_NOT_WIRE | PH1.BCAST / PH1.DELIVERY via LINK_DELIVER_INVITE | Delivery path proven through BCAST/DELIVERY and old callers absent | Yes |
| `LINK_INVITE_RESEND_COMMIT` | LEGACY_DO_NOT_WIRE | PH1.BCAST / PH1.DELIVERY | Resend modeled through delivery owner | Yes |
| `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` | LEGACY_DO_NOT_WIRE | PH1.BCAST / PH1.DELIVERY | Failure handling owned by delivery stack | Yes |
| iPhone `inviteLike` | Compatibility/render parsing | iPhone render/transport only; PH1.LINK runtime activates | Native activation producer exists and remains evidence-only | Yes |
| iPhone `openLike` | Compatibility/render parsing | iPhone render/transport only; PH1.L/session/runtime route owner | Same as above | Yes |
| iPhone `appOpenLike` | Compatibility/render parsing | iPhone render/transport only | Same as above | Yes |
| `/v1/invite/click` | Active compatibility route | Adapter transports; OS app ingress coordinates; PH1.LINK activates | Future canonical route replaces or proves continued bounded use | Yes |
| `CanonicalTurnPayloadCarrier::InviteClick` | Active bounded compatibility carrier | Runtime ingress foundation | Future canonical link ingress packet established | Yes |
| Desktop E2E link seeding | Test/support path | Runtime/test harness only | Native Desktop production path clarified | Yes |
| PH1.F in-memory link map | Current MVP storage | Canonical storage owner | SQL-backed runtime proof if/when adopted | Yes |

Wrong-owner risks:

- DESKTOP_LINK_AUTHORITY_RISK: Native Desktop production invite route not found; future route must not decide access.
- IPHONE_LINK_AUTHORITY_RISK: Current source is read-only, but future native activation producer must not perform activation decisions.
- ADAPTER_LINK_AUTHORITY_RISK: Adapter must remain request/security transport and must not become link lifecycle owner.
- DELIVERY_OWNER_RISK: Legacy send/resend/failure link simulations must stay retired from PH1.LINK.
- ACCESS_SHORTCUT_RISK: Role/access outcomes must not be granted from raw link alone.

## 15. Link Engine Functionalities Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
| --- | --- | --- | --- | --- | --- |
| Create invite link | Generate draft id, token id, signature, link URL, missing fields. | Kernel/OS/storage/docs/tests | PH1.LINK | FOUND | Preserve as canonical link creation owner. |
| Validate link token | Verify token id and presented signature against stored record. | Contract/storage/tests | PH1.LINK | FOUND | Keep fail-closed; expand malformed URL matrix if needed. |
| Activate onboarding link | Accept app-open/deep-link context and move token to activated state. | Kernel/OS/storage/app ingress/tests | PH1.LINK | FOUND | Preserve app-open context requirement. |
| Bind tenant context | Validate tenant/prefill context and reject mismatch. | PH1.F/app ingress | PH1.LINK + Access/Governance | FOUND | Reconcile tenant binding with Access templates. |
| Bind workspace context | Workspace-level link binding. | Search result | Workspace/Access future | NOT_FOUND | Do not invent before repo truth. |
| Bind first device | Store bound device fingerprint hash on first activation. | Storage/tests | PH1.LINK | FOUND | Keep raw fingerprint out of durable record where possible. |
| Block forwarded link | Device mismatch blocks token deterministically. | Storage/runtime/tests | PH1.LINK | FOUND | Add full audit/fail wording later. |
| Update draft metadata | Apply allowed creator update fields and recompute missing fields. | Kernel/storage/tests | PH1.LINK | FOUND | Preserve protected-slot boundary. |
| Reject expired link | Activation after expiry returns expired status. | Storage/runtime/tests | PH1.LINK | FOUND | Add scheduled expiry only if future owner approves. |
| Recover expired link | Replacement token/link URL for expired token. | Kernel/storage/runtime/tests | PH1.LINK | FOUND | Keep revoked token unrecoverable. |
| Revoke link | Revoke eligible token with reason. | Kernel/storage/runtime/tests | PH1.LINK | FOUND/PARTIAL | Complete AP override path if needed. |
| Render invite in client | iPhone parses and displays explicit-entry context. | Swift source | iPhone render-only | FOUND/PARTIAL | Keep client non-authoritative. |
| Route app-open link | Adapter/OS route `/v1/invite/click`. | Adapter/app ingress/runtime tests | Adapter + OS + PH1.LINK | FOUND | Preserve transport/runtime split. |
| Start onboarding from link | ONB session starts from activated result. | PH1.ONB/app ingress/tests | PH1.ONB after PH1.LINK | FOUND | Keep owner split explicit. |
| Carry invite metadata | Prefilled context and missing fields. | Kernel/storage/docs | PH1.LINK selector hints only | FOUND | Avoid schema ownership drift. |
| Carry invite delivery | Sending/resending/failure handling. | Delivery blueprint/docs | PH1.BCAST/DELIVERY | LEGACY_DO_NOT_WIRE in PH1.LINK | Keep out of PH1.LINK. |
| Role proposal | Draft pending AP approval role proposal. | Kernel/runtime/tests | PH1.LINK + Access future | PARTIAL | Reconcile with Access/Governance. |
| Audit link lifecycle | Emit PH1.J state transitions. | OS runtime | PH1.LINK + PH1.J | PARTIAL | Expand failure audit proof. |
| SQL storage | Draft/token/dedupe migration tables. | Migration | Storage | PARTIAL | Prove active runtime SQL path before claiming complete. |

## 16. Comparison To Master Architecture

### Global Request Decision Lattice

Current PH1.LINK routes sit behind request classification and simulation IDs. Blueprints show PH1.X confirmation/access gating for invite generation/delivery. App-open activation is deterministic/policy/simulation gated. PH1.LINK should feed or receive Request Decision Lattice decisions, but must not become the global request router.

### Identity + Access + Authority Spine

Current repo truth aligns with access/authority separation:

- inviter identity and tenant scope matter for generation,
- raw link open does not grant protected authority,
- role proposal is draft/proposal status,
- onboarding/access must complete lawful scope binding elsewhere.

Future reconciliation must connect PH1.LINK to Access Templates, role permissions, tenant/workspace scope, and Authority without giving PH1.LINK direct authority power.

### Onboarding / Invite / Link / Enrollment Stack

Current repo truth strongly supports this stack:

- PH1.LINK creates and activates invite/onboarding links,
- PH1.ONB starts invited onboarding from activated context,
- delivery is separate through LINK_DELIVER_INVITE.

Enrollment-specific link flows remain PARTIAL/NOT_FOUND.

### Master Access Template / Role / Permission Stack

Current repo truth includes `LINK_ROLE_PROPOSE_DRAFT` and dual-role conflict escalation. It does not prove direct role grant from link activation. This is correct. Future access-template integration must route through Access/Governance.

### Voice Identity / Onboarding

Docs say PH1.LINK must not bind voice identity. Voice enrollment link evidence was NOT_FOUND. Future Voice Identity onboarding must keep Voice ID evidence-only and route identity/profile binding through onboarding/access governance.

### Desktop / iPhone Render-Only Boundary

iPhone currently parses and renders explicit-entry context without activation authority. Desktop native production invite handling is NOT_FOUND. Future Desktop/iPhone link behavior must be capture/open/render-only, with runtime PH1.LINK/PH1.ONB making decisions.

### Adapter Transport-Only Boundary

Adapter currently maps HTTP/adapter payloads into OS app ingress. It validates transport security but must not own link lifecycle or authority. This aligns with the master Adapter boundary if preserved.

### Old Compatibility Path Retirement

Current repo truth contains compatibility routes and old do-not-wire simulation names. Future retirement must use active-caller checks and proof, not deletion by assumption.

## 17. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
| --- | --- | --- | --- | --- |
| SQL runtime persistence not fully proven | Migration exists; DB wiring says current MVP uses PH1.F | Drift between SQL contract and runtime | PH1.LINK Repo-Truth Activation Pack should map active storage path precisely | High |
| Workspace scope not found | Searches found tenant, not workspace binding | Workspace invites may be underdefined | Add workspace-scope design only after owner discovery | Medium |
| Native Desktop link handling not found | Mac Desktop source search | Desktop invite UX unclear | Add Desktop render/open proof slice if needed | Medium |
| Voice enrollment link not found | No PH1.LINK-specific voice enrollment link located | Future voice onboarding could invent duplicate path | Map through onboarding/access before build | Medium |
| AP override revoke path partial | Revoke refusal mentions AP override field but request lacks full override | Activated revoke policy cannot complete cleanly | Reconcile Access/Policy override packet before implementation | High |
| Failed activation audit partial | Runtime transition audit exists; failed boundary audit incomplete | Failures may lack proof | Add link audit evidence pack | High |
| Old compatibility audit partial | Compatibility carrier exists | Old routes may hide behavior | Add compatibility route proof/retirement ledger | Medium |
| Delivery legacy names still in docs as do-not-wire | Simulation catalog/docs | Wrong owner regression | Keep script checks and add active-caller proof | High |
| Token consumption path partially extracted | Docs mention consume on ONB completion | Reuse/replay semantics may be incomplete | Map ONB completion consume owner in activation pack | High |
| User-facing wording out of scope | PH1.WRITE not extracted here | Raw technical errors may leak if not shaped | Future PH1.WRITE integration | Medium |
| JD live acceptance not found | Tests only | Product proof incomplete | JD live invite/open acceptance pack | Medium |
| Security envelope audit partial | HTTP 401 test exists | Rejected attempts may lack audit trail | Add adapter/security audit proof if policy requires | Medium |

## 18. Recommended Future Build Slices

No implementation is authorized by this extraction. These are future recommended slices based on repo truth only.

1. PH1.LINK Repo-Truth Activation Pack
   - Map active owners, callers, simulations, storage path, app ingress, adapter routes, iPhone route parsing, and old paths.

2. Link Contract / State Machine Normalization
   - Confirm `LinkStatus`, `DraftStatus`, terminal states, app-open fields, idempotency keys, and reason codes as canonical.

3. Invite Link Validation Proof
   - Prove token signature, expiry, revoked/consumed/blocked terminal states, malformed tokens, and bad signature handling.

4. Onboarding Link Activation Proof
   - Prove `LINK_INVITE_OPEN_ACTIVATE_COMMIT` plus `ONB_SESSION_START_DRAFT` chain from app ingress.

5. Tenant / Workspace Scope Integration
   - Keep tenant proof and define workspace behavior only after repo-truth owner discovery.

6. Access / Role Proposal Reconciliation
   - Connect `LINK_ROLE_PROPOSE_DRAFT` and dual-role escalation to Access/Governance without direct link authority.

7. Desktop / iPhone Render-Only Link Proof
   - Prove iPhone render-only route parsing and add Desktop path only if required by product build.

8. Adapter Transport-Only Link Proof
   - Prove Adapter maps and secures requests without owning lifecycle, onboarding, or authority.

9. Link Audit Evidence Pack
   - Expand PH1.J proof for creation, activation, block, revoke, recovery, failure, and compatibility route usage.

10. Old Link Compatibility Retirement Ledger
   - Track `/v1/invite/click`, compatibility carriers, iPhone parser names, and legacy do-not-wire simulations with active-caller checks.

11. SQL Storage Activation Pack
   - Decide whether PH1.F remains current runtime truth or SQL tables become active; prove before migration.

12. JD Live Invite/Open Acceptance Pack
   - Real app proof for generated invite, open link, activation, onboarding start, invalid link denial, and backend evidence agreement.

## 19. What Codex Must Not Do

Codex must not invent link behavior.

Codex must not create a duplicate link engine.

Codex must not grant access from raw link text alone.

Codex must not let Desktop or iPhone decide access.

Codex must not let Adapter decide access.

Codex must not bypass onboarding/access governance.

Codex must not let PH1.LINK bind voice identity.

Codex must not let PH1.LINK perform invite delivery.

Codex must not revive legacy send/resend/failure simulations inside PH1.LINK.

Codex must not assume workspace or voice-enrollment link behavior where repo truth is NOT_FOUND.

Codex must not delete old paths before proof.

Codex must not implement from this extraction document alone.

## 20. Final Extracted Architecture Sentence

PH1.LINK is Selene's governed link and activation boundary: it may carry onboarding, invite, app-open, device-binding, tenant-context, selector-hint, and access-proposal evidence, but activation, identity, tenant/workspace scope, access, authority, onboarding completion, delivery, and audit must remain owned by their canonical Selene engines.
