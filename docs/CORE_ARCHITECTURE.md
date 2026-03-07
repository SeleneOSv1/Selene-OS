# Selene Client Runtime Architecture - Universal Device Session Model

## 0) Status Legend
- `CURRENT`: Implemented and expected to be true in active runtime behavior.
- `TARGET`: Approved architectural direction; not fully implemented yet.
- `GAP`: Known missing or partial area that must be completed.

## 0.1) v3 Constitutional Integration (Authoritative Overlay)
- This document is the full system-law specification for Selene runtime architecture.
- All platform implementations, engines, simulations, and subsystems must comply with this document.
- v3 constitutional wording is integrated here as an overlay without replacing requirement IDs, status truth labels, or acceptance matrices.
- If v3 wording conflicts with implementation reality, the governing status labels (`CURRENT`/`TARGET`/`GAP`) remain authoritative until code closes the gap.

### 0.1.1) v3 Clause Crosswalk (Preserves Existing Truth States)

| v3 Clause | Canonical Requirement IDs / Sections | Effective Status |
|---|---|---|
| 1 Core Architecture Law | `CAR-001..005`, `CAR-040..042`, Section 17 | CURRENT |
| 2 Universal Client Principle | `CAR-010..012`, `CAR-060..062` | CURRENT/TARGET |
| 3 Session-First System Law | `SES-001..003`, `CAR-020..021` | CURRENT |
| 4 Trigger Policy by Platform | `TRG-001..005` | CURRENT/TARGET |
| 5-7 Client responsibilities, capabilities, assist scope | `CAR-030..034`, `CAR-050`, Section 8 | CURRENT/TARGET |
| 8 Cross-platform capability rule | `CAR-060..062` | CURRENT/TARGET |
| 9 Canonical runtime pipeline | `CAR-020..021` | CURRENT |
| 10-11 Multi-platform consequence + architecture alignment phase | `CAR-010`, `GOV-003`, Section 21 matrix | CURRENT |
| 12 Ingress contract family | `NET-001..012`, Section 7.1 matrix | CURRENT |
| 13 Canonical session payload exposure | `SES-020..022`, `NET-013` | TARGET/GAP |
| 14-17 Trigger/session separation + sync + learning/apply loop | `TRG-005`, `SYNC-001..004`, `LUP-001..003` | CURRENT |
| 18-21 Cloud authority, platform inventory, scope boundary, correction-first rule | `CAR-040..042`, `CAR-002`, `CAR-061`, `GOV-001..003` | CURRENT/TARGET |
| 22-24 Voice enrollment, divergence matrix, personality semantics | `ONB-001..014`, `EMO-001..005` | CURRENT/TARGET/GAP |
| 25 Session lifecycle law | `SES-030..035`, `SES-004..005` | CURRENT/TARGET/GAP |
| 26 Canonical identifier exposure | `SES-020..022`, `NET-013` | TARGET/GAP |
| 27-29 Memory authority, persistence, local cache boundary | `MEM-001..024` | CURRENT/TARGET |
| 30-31 Learning artifact + offline/reconnect model | `LUP-001..003`, `SYNC-001..004` | CURRENT |
| 32 Link generation model | `LNK-001..003` | CURRENT |
| 33-34 Device/cloud boundary + capability parity | `CAR-050..051`, `CAR-060..062` | CURRENT/TARGET |
| 35 Retention/lifecycle governance | `RET-001..002` | GAP/CURRENT |
| 36 Known architectural gaps | `GAP-001..007` | GAP |
| 37 Governing architecture rule | `GOV-001..003` | CURRENT |
| 38 Engine ownership of gaps | Section 21 Acceptance/Ownership Matrix | CURRENT |
| 39 Build-phase transition rule | `GOV-003`, Section 21 ownership acceptance sequencing | CURRENT |

### 0.1.2) v3 Part Crosswalk (Logical Grouping)
- `PART I — FOUNDATIONAL SYSTEM LAWS`: Sections 1-5, 8-11, 17-18.
- `PART II — SESSION & EXECUTION MODEL`: Sections 6-7, 12, 14-16.
- `PART III — IDENTITY, MEMORY, AND ARTIFACT GOVERNANCE`: Sections 9-13, 19-20.
- `PART IV — ADVANCED SYSTEM GOVERNANCE`: Sections 21-22.
- Section numbering and requirement IDs remain unchanged to preserve traceability to code/tests/ledger entries.

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

## 23) Appendix A - Selene Architecture v3 Verbatim Body (Reference Copy)
- This appendix preserves the full v3 constitutional prose body as requested.
- Normative implementation truth remains governed by Sections 0-22 status-tagged requirements and matrices.

### Selene Client Runtime Architecture — Universal Device Session Model (v3)

#### 0. Status Legend
CURRENT — Implemented and expected to be true in runtime behavior.
TARGET — Approved architectural direction not yet fully implemented.
GAP — Known missing or partial area that must be completed.

This document is the full system law specification for the Selene runtime architecture.
All platform implementations, engines, simulations, and subsystems must comply with the rules defined here.

### PART I — FOUNDATIONAL SYSTEM LAWS

#### 1. Core Architecture Law
Selene is a cloud-authoritative distributed intelligence system.
Client devices serve as interaction terminals, while the Selene cloud runtime is the authoritative execution environment.

Authoritative domains owned exclusively by the cloud runtime include:
- identity verification
- access authorization
- session lifecycle control
- simulation discovery and execution
- memory governance
- artifact creation and activation
- learning evaluation and promotion
- audit and proof capture

Client devices must never finalize system truth.
All authoritative state transitions occur only inside the cloud runtime.

#### 2. Universal Client Principle
All Selene clients must implement a single universal runtime interaction model regardless of device type.

Supported platforms:
- iPhone
- Android
- Desktop
- Tablet (TARGET platform class)

Every client must support the following capabilities:
- session entry and resume
- turn submission
- response rendering
- deterministic synchronization

Hardware interaction may differ, but system logic must remain identical.

#### 3. Session-First System Law
All Selene execution originates from a cloud-controlled session.

Sessions coordinate:
- identity scope
- access policy
- memory eligibility
- simulation execution

Devices may open or resume sessions but never own session state.
Sessions are therefore the primary execution container for all Selene system behavior.

#### 4. Trigger Policy by Platform
Trigger policy determines how a session begins.

Platform trigger rules:
- iPhone — explicit trigger only
- Android — wake word or explicit
- Desktop — wake word or explicit
- Tablet — TARGET behavior mirrors Android

Trigger differences affect entry only.
Once a session begins, execution behavior is identical across all platforms.

#### 5. Client Responsibilities
Clients are responsible for interaction and reliability.

Responsibilities include:
- capturing voice, text, files, and images
- submitting turns to the cloud
- rendering responses
- maintaining retry outbox
- synchronizing with cloud runtime

Clients must never execute simulations or mutate system state.

#### 6. Client Interaction Capabilities
Clients must support rich interaction capabilities including:
- voice interaction
- text interaction
- file uploads
- image uploads
- camera capture
- structured visualization
- PDF generation
- printer workflows

All interaction must pass through the session runtime pipeline.

#### 7. Local Assist Scope
Clients may store assist caches for speed.

Allowed caches include:
- conversation display cache
- retry outbox
- voice assist embeddings

Local data must always be replaceable by cloud truth.

#### 8. Cross-Platform Capability Rule
All platforms must expose equivalent Selene capability.
Device switching must preserve:
- session state
- turn order
- memory context

#### 9. Canonical Runtime Pipeline
Every interaction follows the deterministic execution pipeline:

ingress validation
platform policy validation
session resolution
identity verification
onboarding eligibility
memory eligibility
access authorization
simulation eligibility
execution
audit capture
response synchronization

No feature may bypass this pipeline.

### PART II — SESSION & EXECUTION MODEL

#### 10. Multi-Platform Design Consequence
Selene operates as a single distributed system accessed through multiple device terminals.
Platforms must not introduce divergent runtime behavior.

#### 11. Architecture Alignment Phase
Before feature expansion, runtime implementation must align with architecture.

Alignment tasks include:
- canonical session contract exposure
- PH1.OS trigger enforcement
- capture attestation
- artifact signature verification

#### 12. Client Ingress Contract
Canonical ingress endpoints:
- /v1/invite/click
- /v1/onboarding/continue
- /v1/voice/turn

Requests must include:
- auth token
- request id
- nonce
- timestamp
- idempotency key

#### 13. Universal Session Payload
Responses must include:
- session_id
- turn_id
- session_state

These identifiers drive synchronization and audit.

#### 14. Trigger vs Session Model
Triggers vary by device.
Session execution must remain identical across platforms.

#### 15. Capability Parity
All platforms must support:
- voice interaction
- text
- file upload
- structured outputs
- cross-device continuation

#### 16. Synchronization Model
Clients must implement deterministic distributed synchronization.

Required components:
- durable outbox
- operation journal
- retry logic

Cloud acknowledgement finalizes execution.

#### 17. Learning & Update Loop
Learning is cloud-governed.
Clients submit signals.
Cloud evaluates and promotes learning artifacts.

Clients apply updates via:
- download
- verify
- stage
- apply
- confirm
- rollback

### PART III — IDENTITY, MEMORY, AND ARTIFACT GOVERNANCE

#### 18. Cloud Authority Boundary
Cloud runtime is the single authority for system state.
Clients are treated as untrusted environments.

#### 19. Platform Inventory
Supported platforms:
- iPhone
- Android
- Desktop
- Tablet (TARGET)

All must follow the same runtime architecture.

#### 20. Architecture Scope Boundary
This document defines architectural laws.
Implementation details belong in subsystem documents.

#### 21. Implementation Alignment
Architecture correction must precede feature expansion.

#### 22. Voice Enrollment Authority
Voice enrollment establishes identity scope.
Clients capture samples.
Cloud validates and locks voice identity.

#### 23. Platform Divergence Matrix
Platform differences affect hardware interaction only.
Execution pipeline remains identical.

#### 24. Personality Engine
Personality categories:
- Passive
- Domineering
- Undetermined

Current behavior influences tone only.

#### 25. Session Lifecycle
Session states:
- Closed
- Open
- Active
- SoftClosed
- Suspended

Default inactivity threshold: 30 seconds.

#### 26. Canonical Identifier Exposure
session_id and turn_id must appear in all responses.

#### 27. Memory Authority
Memory is identity-scoped and cloud authoritative.

#### 28. Memory Persistence
Ledger-first model with materialized memory view.

Retention classes:
- Hot (72h)
- Medium (30d)
- Cold (indefinite)

#### 29. Local Assist Cache
Client caches accelerate interaction but never override cloud truth.

#### 30. Learning Artifact Pipeline
Learning signals → evaluation → artifact creation → distribution.

#### 31. Offline / Reconnect Model
Clients reconcile state through idempotent operations.

#### 32. Link Generation Model
Links generated only through cloud simulations.
Delivery path uses BCAST engine.

#### 33. Device vs Cloud Boundary
Devices capture and render.
Cloud decides and executes.

#### 34. Cross-Platform Capability Parity
System capability must remain consistent across devices.

#### 35. Retention & Lifecycle Governance
All system artifacts follow governed retention policies.

#### 36. Known Architectural Gaps
Current gaps include:
- session identifier exposure
- Android wake parity
- cross-device session continuity
- tablet runtime support

### PART IV — ADVANCED SYSTEM GOVERNANCE

#### 37. Governing Architecture Rule
This document is the constitutional law of Selene runtime architecture.
All implementations must comply.

#### 38. Engine Ownership of Gaps
Each architectural gap must have a responsible engine owner.

Examples:
- PH1.W — wake
- PH1.K — audio capture
- PH1.L — session lifecycle
- PH1.M — memory
- PH1.EMO — personality
- PH1.OS — platform policy

#### 39. Build-Phase Transition Rule
Implementation phases:
- Phase 0 — architecture correction
- Phase 1 — runtime parity
- Phase 2 — platform expansion

Feature expansion must not occur before architecture alignment.

## 24) Appendix B - Engine Ownership of Gaps (Explicit Narrative Block)
The following engine-level narratives define current ownership focus for unresolved architecture gaps.

### PH1.W — Wake Engine
- Android wake runtime parity remains incomplete.
- Wake artifact retention, purge, and delete lifecycle is not fully implemented.
- Reject-reason coverage and wake runtime parity across platforms require further completion.

### PH1.K — Voice Runtime I/O
- Android microphone runtime parity with the desktop path is still incomplete.
- Capture-bundle trust and attestation boundaries require additional hardening.

### PH1.L — Session Lifecycle Engine
- Session lifecycle logic is implemented, but current reopen scope is effectively actor plus device.
- True cross-device shared session continuity remains a future architectural target.
- Public session identifier exposure is still incomplete in some API responses.
- Session close confirmation behavior and inter-device continuity require additional design and implementation work.

### PH1.VOICE.ID — Voice Identity Engine
- Voice enrollment and identity assertion are implemented in onboarding.
- Native device-side capture contract and enrollment UX still require full implementation in client applications.
- Device-side enrollment flow must stay aligned to cloud enrollment authority.

### PH1.EMO / PH1.EMO.CORE — Personality Engine
- Personality classification and persona lock exist.
- Current implementation affects tone and response style only.
- The stronger permanent opposite-response model is not currently implemented.

### PH1.M — Memory Engine
- Core memory authority and identity-scoped storage are implemented.
- Full retention, purge, and delete lifecycle policies are not yet complete.
- Hot, medium, and cold memory behavior must be finalized and enforced consistently.

### PH1.OS — Platform Orchestration Layer
- Platform-aware trigger policy needs stronger centralized enforcement.
- Tablet platform modeling is still emerging and must be fully integrated.
- Cross-device switching and universal session-first enforcement must remain consistent at orchestration level.

### PH1.F — Persistence Foundation
- Persistence is largely complete but carries unfinished lifecycle responsibilities for session artifacts, memory retention, wake artifacts, and compliance records.
- Additional lifecycle enforcement and purge workers are expected.

### PH1.J — Audit and Proof Layer
- Black-box compliance and proof capture must remain explicitly bound into authoritative runtime flow.
- Session-bound audit, proof, and compliance capture require stronger explicit treatment.

### Simulation and Link Execution Path
- Link generation and delivery are implemented through simulation dispatch and broadcast execution.
- Public client-facing generate/send APIs are product-layer decisions outside this parent law unless explicitly added.

## 25) Appendix C - Phase 0 / 1 / 2 Section Text (Explicit)
Architecture-correction sequencing is mandatory and must be preserved in planning and execution.

### Phase 0 — Architecture Correction
- Correct any runtime wiring that contradicts the parent architecture.
- Close constitutional mismatches before any new feature expansion.

### Phase 1 — Runtime Parity
- Achieve deterministic parity across supported runtime paths and platform policy enforcement.
- Ensure canonical contract behavior, auditability, and reliability standards are met.

### Phase 2 — Platform Expansion
- Expand platform implementations only after architecture correction and runtime parity are complete.
- Preserve session-first and cloud-authoritative laws while extending platform surface.

Feature expansion must not occur before architecture alignment.

## 26) Appendix D - Selene Architecture v3 Completion Note (Paragraph)
This specification defines Selene as a distributed AI operating system architecture built on deterministic execution pipeline, simulation-governed execution, cloud authority, identity-scoped memory, artifact governance, and distributed synchronization. This document serves as the complete architecture blueprint for Selene runtime development.
