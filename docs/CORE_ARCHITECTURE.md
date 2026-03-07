# Selene Client Runtime Architecture - Universal Device Session Model

## 0) Status Legend
- `CURRENT`: Implemented and expected to be true in active runtime behavior.
- `TARGET`: Approved architectural direction; not fully implemented yet.
- `GAP`: Known missing or partial area that must be completed.

## 1) Core Architecture Laws
- `[CAR-001] [CURRENT]` Selene operates through client applications on supported device classes: iPhone, Android phone, Desktop.
- `[CAR-002] [TARGET]` Tablet is a formal platform class in the parent architecture and must be added to contracts and runtime policy enforcement.
- `[CAR-003] [CURRENT]` The client is never the authority layer; cloud is always authoritative for identity, access, session truth, execution, and audit.
- `[CAR-004] [CURRENT]` The client is the user terminal for speaking, hearing, viewing, and interacting with Selene.
- `[CAR-005] [CURRENT]` Universal behavior across clients is mandatory: capture input, open or resume session, submit turns, render response, play audio, and synchronize safely.

## 2) Universal Client Principle
- `[CAR-010] [CURRENT]` iPhone, Android, Desktop clients follow one shared model and must not fork core runtime semantics.
- `[CAR-011] [TARGET]` Tablet follows the same shared model once platform contract support is added.
- `[CAR-012] [CURRENT]` Device-local state is assist-only and may improve speed/continuity but may not overwrite cloud truth.

## 3) Trigger Policy by Platform
- `[TRG-001] [CURRENT]` iPhone is explicit-only: side-button or approved explicit action opens/resumes session; wake word is blocked.
- `[TRG-002] [CURRENT]` Android supports wake word and explicit entry; both are session-entry methods.
- `[TRG-003] [CURRENT]` Desktop supports wake word and explicit entry; both are session-entry methods.
- `[TRG-004] [TARGET]` Tablet supports wake word and explicit entry, aligned to Android-style policy unless constrained by OS policy.
- `[TRG-005] [CURRENT]` Trigger differences affect entry method only; they must never fork session lifecycle semantics.

## 4) Session-First System Law
- `[SES-001] [CURRENT]` All meaningful Selene work must execute from an open/resumed cloud session context.
- `[SES-002] [CURRENT]` Session context is required for simulation processing, web operations, link workflows, onboarding actions, and future tools.
- `[SES-003] [CURRENT]` Devices may open/resume/reflect sessions and cache hints; devices may not invent session truth.
- `[SES-004] [TARGET]` Session inactivity policy standard: default soft-close transition at 30 seconds of inactivity, configurable by governed policy.
- `[SES-005] [TARGET]` Before final close, system should confirm if user is finished when interaction policy requires close-check behavior.
- `[SES-006] [CURRENT]` Current reopen behavior is effectively scoped by actor plus device.
- `[SES-007] [GAP]` True cross-device shared session continuity is not yet fully implemented as a public contract.

## 5) Canonical Runtime Flow
- `[CAR-020] [CURRENT]` Canonical flow is: trigger -> capture input -> submit turn -> ingress validation -> resolve/open session -> apply identity/onboarding gates -> memory/access/understanding/execution -> return result -> client render/playback -> sync outcome state.
- `[CAR-021] [CURRENT]` This canonical flow is the parent runtime model for all client classes.

## 6) Universal Session Payload Contract
- `[SES-020] [TARGET]` Canonical client-visible session payload must include `session_id`, `turn_id`, and `session_state` with stable semantics across platforms.
- `[SES-021] [CURRENT]` Clients must not infer or fabricate identifiers; server values are authoritative.
- `[SES-022] [GAP]` Some current API responses still do not expose full session identifiers in client-facing payloads.

## 7) Client Ingress Contract (Cross-Platform)
- `[NET-001] [CURRENT]` Canonical ingress route family:
  - `/v1/invite/click`
  - `/v1/onboarding/continue`
  - `/v1/voice/turn`
- `[NET-002] [CURRENT]` Clients must follow server-defined security semantics and fail closed on invalid security fields.
- `[NET-003] [CURRENT]` Clients must never bypass these flows locally.

### 7.1 Protocol Contract Matrix

| Requirement ID | Endpoint | Required Headers | Idempotency | Replay Guard | Request Core Fields | Response Core Fields | Status |
|---|---|---|---|---|---|---|---|
| `NET-010` | `/v1/invite/click` | `Authorization`, `X-Request-Id`, `X-Nonce`, `X-Timestamp-Ms`, `X-Idempotency-Key` | Required | Nonce + timestamp window | invite token/signature, app platform, app instance/device identity context | onboarding session start + next step | CURRENT |
| `NET-011` | `/v1/onboarding/continue` | `Authorization`, `X-Request-Id`, `X-Nonce`, `X-Timestamp-Ms`, `X-Idempotency-Key` | Required | Nonce + timestamp window | onboarding session id, action, platform receipts and action payload | next step, blockers, status deltas | CURRENT |
| `NET-012` | `/v1/voice/turn` | `Authorization`, `X-Request-Id`, `X-Nonce`, `X-Timestamp-Ms`, `X-Idempotency-Key` | Required | Nonce + timestamp window | trigger, actor, device, capture references, user/voice payload | turn outcome and response payload | CURRENT |
| `NET-013` | `/v1/voice/turn` session identifiers | Same as above | Same as above | Same as above | Same as above | explicit `session_id`, `turn_id`, `session_state` in response contract | TARGET |

## 8) Shared Client Responsibilities
- `[CAR-030] [CURRENT]` All clients must support audio capture, preprocessing, voice entry handling, session entry/resume request, response rendering, and playback.
- `[CAR-031] [CURRENT]` All clients must implement local outbox and deterministic retry behavior.
- `[CAR-032] [CURRENT]` All clients must maintain lightweight assist cache only (never authoritative).
- `[CAR-033] [CURRENT]` All clients must submit required platform receipts and device state proofs to cloud onboarding/session paths.
- `[CAR-034] [TARGET]` All clients should expose functionally equivalent rich interaction capabilities (file upload, image capture, structured result rendering, print workflows) with platform-native UX.

## 9) Shared Cloud Responsibilities
- `[CAR-040] [CURRENT]` Cloud authoritative domains: identity, access, NLP/LLM reasoning, simulation selection/execution, session truth, onboarding truth, learning governance, artifact authority, and audit logging.
- `[CAR-041] [CURRENT]` Builder/repair workflows belong to cloud authority layer and must remain session-first.
- `[CAR-042] [CURRENT]` No device override of cloud truth is permitted.

## 10) Onboarding + Voice Enrollment Authority
- `[ONB-001] [CURRENT]` Voice enrollment is mandatory before onboarding completion.
- `[ONB-002] [CURRENT]` Voice enrollment must reach locked state and produce synchronized artifact receipt before complete.
- `[ONB-003] [CURRENT]` Voice profile identity scope is cloud-authoritative; clients act as capture terminals.
- `[ONB-004] [CURRENT]` Canonical onboarding order includes: invite/open -> onboarding start -> required receipts -> missing fields/terms/primary device/sender verification (if required) -> voice enrollment -> wake enrollment where required -> personality lock -> access step -> complete -> ready.

### 10.1 Platform Divergence Matrix
- `[ONB-010] [CURRENT]` iPhone: wake enrollment disabled; `ios_side_button_configured` receipt required.
- `[ONB-011] [CURRENT]` Android: wake enrollment required.
- `[ONB-012] [TARGET]` Tablet: wake enrollment required unless a later platform contract defines a constrained policy.
- `[ONB-013] [CURRENT]` Desktop: wake enrollment required.
- `[ONB-014] [CURRENT]` Despite platform divergence, onboarding authority and progression remain cloud-governed.

## 11) Personality Lock Semantics
- `[EMO-001] [CURRENT]` Personality classification categories: `Passive`, `Domineering`, `Undetermined`.
- `[EMO-002] [CURRENT]` Personality lock is authoritative in onboarding record state.
- `[EMO-003] [CURRENT]` Personality affects tone/style only in current implementation.
- `[EMO-004] [CURRENT]` Personality must never influence access control, simulation execution authorization, security checks, or system authority.
- `[EMO-005] [GAP]` Strong permanent opposite-response behavior is not currently implemented and requires separate design/build.

## 12) Session Lifecycle Contract
- `[SES-030] [CURRENT]` Canonical session states: `Closed`, `Open`, `Active`, `SoftClosed`, `Suspended`.
- `[SES-031] [CURRENT]` Session state transitions are server-controlled.
- `[SES-032] [TARGET]` Policy standard for inactivity: open session remains active until user stops speaking for configured inactivity period; default target is 30s for inactivity transition logic.
- `[SES-033] [TARGET]` Session close confirmation behavior should be policy-driven and applied before full close where required.
- `[SES-034] [CURRENT]` Clients reflect server lifecycle state and never synthesize state transitions.
- `[SES-035] [GAP]` Cross-device shared session continuity remains a future alignment target.

## 13) Memory Authority and Session Memory Handling
- `[MEM-001] [CURRENT]` Memory is cloud-authoritative and identity-scoped.
- `[MEM-002] [CURRENT]` Memory storage is keyed by user identity; sessions reference memory but do not own memory records.
- `[MEM-003] [CURRENT]` Authoritative memory read/write requires confirmed voice identity.
- `[MEM-004] [CURRENT]` Sensitive memory requires permission checks before use.

### 13.1 Retention Classes (Normative)
- `[MEM-010] [TARGET]` Hot memory retention minimum: 72 hours.
- `[MEM-011] [TARGET]` Medium memory retention minimum: 30 days.
- `[MEM-012] [TARGET]` Cold memory retention: indefinite until policy/delete command requires removal.

### 13.2 How Memory Must Behave Across Sessions
- `[MEM-020] [TARGET]` On session open/resume, memory load must run through identity scope checks and policy filters before candidate hydration.
- `[MEM-021] [TARGET]` During active session, memory read path must enforce sensitivity/use-policy/confidence gates for every turn.
- `[MEM-022] [TARGET]` Memory writes generated during session must append to ledger first, then materialize current state deterministically.
- `[MEM-023] [TARGET]` On session close, session lifecycle may end while cloud memory persists by retention class policy.
- `[MEM-024] [CURRENT]` Device memory/cache remains assist-only; cloud memory remains authoritative on divergence.

## 14) Offline, Sync, Outbox, and Deduplication
- `[SYNC-001] [CURRENT]` Every client must implement durable outbox, operation journal, retry state, idempotent resend, and cloud-acknowledged completion.
- `[SYNC-002] [CURRENT]` If local and cloud diverge, cloud state wins.
- `[SYNC-003] [CURRENT]` Reconnect sequence must include auth refresh, session truth refresh, pending flush, upload queue drain, approved update pull/apply, and UI refresh.
- `[SYNC-004] [CURRENT]` Duplicate execution must be prevented via stable idempotency identities.

## 15) Learning and Update Loop
- `[LUP-001] [CURRENT]` Clients may upload approved learning artifacts/telemetry; cloud decides acceptance/promotion.
- `[LUP-002] [CURRENT]` Clients may receive approved profile/config updates and must apply with safe sequence: download -> verify -> stage -> apply -> confirm -> rollback when required.
- `[LUP-003] [CURRENT]` Promotion/governance authority remains cloud-side.

## 16) Link Generation and Delivery Model
- `[LNK-001] [CURRENT]` Link generation and delivery are cloud-owned operations executed under session context.
- `[LNK-002] [CURRENT]` Clients render outcomes but must not generate authoritative links locally.
- `[LNK-003] [CURRENT]` Public client-facing invite generation API is not defined in this parent architecture.

## 17) Device vs Cloud Responsibility Boundary
- `[CAR-050] [CURRENT]` Device responsibilities: capture, preprocessing, rendering/playback, assist caching, sync/outbox behavior, platform hardware integration.
- `[CAR-051] [CURRENT]` Cloud responsibilities: identity/access/session/onboarding authority, simulation execution, memory authority, learning governance, artifact authority, audit/compliance.

## 18) Cross-Platform Capability Parity
- `[CAR-060] [CURRENT]` iPhone, Android, Desktop must preserve one functional shape: cloud authority, session-first runtime, ingress family, onboarding authority, voice-enrollment authority, memory authority, sync/retry model.
- `[CAR-061] [TARGET]` Tablet must be fully integrated into the same parity contract.
- `[CAR-062] [CURRENT]` Allowed platform differences are limited to trigger mechanics, hardware handling, and OS constraints.

## 19) Retention, Purge, and Delete Lifecycle
- `[RET-001] [GAP]` Retention/purge/delete execution must be explicit for wake artifacts, memory records, device assist cache, operation journals/outbox state, and session-bound compliance records.
- `[RET-002] [CURRENT]` Unimplemented lifecycle areas must remain explicitly marked as gaps.

## 20) Known Architectural Gaps
- `[GAP-001] [GAP]` Explicit session identifiers are not yet exposed in all client-facing response contracts.
- `[GAP-002] [GAP]` True cross-device shared session continuity is not fully implemented.
- `[GAP-003] [GAP]` Android wake runtime parity remains incomplete.
- `[GAP-004] [GAP]` Tablet platform modeling remains incomplete in runtime contracts.
- `[GAP-005] [GAP]` Personality behavior remains tone-scoped only.
- `[GAP-006] [GAP]` Retention/purge/delete lifecycle policy implementation remains incomplete.
- `[GAP-007] [CURRENT]` Native app implementation spans separate repositories and must align to this parent architecture.

## 21) Acceptance Criteria and Ownership Matrix

| Area | Requirement IDs | Acceptance Criteria (Proof of Completion) | Owner |
|---|---|---|---|
| Core universal model | `CAR-001..005`, `CAR-020..021` | Client and server docs agree on one runtime flow; no contradictory platform forks in parent docs | PH1.OS |
| Trigger policy | `TRG-001..005` | Central platform-aware trigger validation exists; iPhone wake blocked; non-iPhone wake/explicit policy enforced | PH1.OS, PH1.W |
| Session lifecycle | `SES-001..007`, `SES-030..035` | Session state transitions and reopen rules pass deterministic tests; cross-device continuity gaps explicitly tracked | PH1.L, PH1.F |
| Session payload | `SES-020..022`, `NET-013` | Public contract exposes canonical ids (`session_id`,`turn_id`,`session_state`) with compatibility plan | PH1.L, adapter ingress |
| Ingress protocol | `NET-001..012` | Endpoint matrix enforced with bearer binding, nonce/timestamp replay checks, idempotency, and deterministic failure mapping | adapter ingress, PH1.QUOTA |
| Onboarding + voice/wake | `ONB-001..014` | Onboarding progression enforces platform matrix, voice lock receipt, wake rules by platform, and completion gates | PH1.ONB, PH1.VOICE.ID, PH1.W |
| Personality semantics | `EMO-001..005` | Personality lock remains tone-only; no bypass of access/simulation/security paths; stronger behavior marked as future scope | PH1.EMO, PH1.PERSONA |
| Memory handling | `MEM-001..024` | Identity-gated recall/write, policy filters, session-aware load behavior, retention classes implemented and test-covered | PH1.M, PH1.F |
| Sync and dedup | `SYNC-001..004` | Durable outbox/journal and idempotent retry paths proven in integration tests | client runtime, PH1.F |
| Learning/update loop | `LUP-001..003` | Bidirectional update loop enforced with safe apply/rollback and cloud promotion governance | PH1.LEARN, artifact sync |
| Retention lifecycle | `RET-001..002` | Purge/delete workers and verification receipts exist for defined data classes | PH1.F, PH1.J |

## 22) Governing Rule
- `[GOV-001] [CURRENT]` This document is the parent architecture for all Selene client platforms.
- `[GOV-002] [CURRENT]` Platform-specific documents may extend this model but must not contradict it.
- `[GOV-003] [CURRENT]` Future build runs must align to this session-first law before adding new feature surface.
