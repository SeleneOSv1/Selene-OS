PHASE H2 — SLICE 1 RUNTIME AND SESSION BUILD PLAN

A) PURPOSE

This document converts Slice 1 from [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) into a build-grade implementation plan. Slice 1 covers only the runtime kernel and the canonical session container. It is planning law only. It does not authorize coding outside Sections 01 and 02, and it does not authorize Apple implementation.

B) FROZEN LAW INPUTS

Slice 1 is derived from the following governing inputs:

- frozen A1-A6, B1-B5, C1-C5, D1-D5, E1-E5, F1-F5, and G1-G2 as upstream design-law constraints
- [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) as the sequencing-law anchor
- [Build Section 01](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md) and [Build Section 02](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md) as the main governing implementation sections
- [SELENE_BUILD_EXECUTION_ORDER](docs/SELENE_BUILD_EXECUTION_ORDER.md), the design lock sequence, and the authoritative coverage table as gating truth

Sections 03 and later are downstream dependencies only. They may define seams that Slice 1 must preserve, but they are not implementation scope in this document.

C) CURRENT / TARGET / GAP

CURRENT
- Repo truth already authorizes a `HYBRID_BUILD_AND_DESIGN` method and places Slice 1 first in dependency order.
- Design lock has advanced far enough to begin broad runtime wiring.
- The runtime and session contracts are already explicit enough to support guarded implementation.

TARGET
- Deliver a precise plan for building the Section 01 core runtime skeleton and the Section 02 session engine foundation.
- Define exact Slice 1 scope, internal order, likely workstreams, verification, acceptance, and guardrails.

GAP
- The remaining gap is not missing architecture for Slice 1. The gap is disciplined execution: strict section boundaries, fail-closed behavior, stable runtime/session semantics, and proof that later layers are not pulled forward.

D) SLICE 1 DELIVERY POSITION

Slice 1 is the opening implementation slice for Selene. It establishes the minimal runtime substrate that every later section depends on. It must:

- build the runtime process shell before any feature-first work
- establish the canonical session container before any turn-pipeline, authority, persistence, or client implementation work
- leave Apple, memory, personality, and numeric computation as downstream work

There is no repo-truth basis to treat Session as a blocker for Slice 1. There is also no repo-truth basis to treat personality or emotional behavior as part of this slice.

E) slice-1 scope and dependency matrix

| item | classification | Slice 1 position |
|---|---|---|
| runtime process startup | `IN_SCOPE_SLICE_1` | Foundational runtime bootstrap and admission posture |
| service framework / DI container | `IN_SCOPE_SLICE_1` | Required to host runtime services and engine surfaces |
| engine registry / engine discovery | `IN_SCOPE_SLICE_1` | Required to register bounded runtime and session capabilities |
| environment configuration / config governance / secrets | `IN_SCOPE_SLICE_1` | Required to start deterministically and fail closed on missing config |
| observability / structured logging / metrics / tracing hooks | `IN_SCOPE_SLICE_1` | Required to make runtime and session behavior auditable from the start |
| health / readiness / startup checks | `IN_SCOPE_SLICE_1` | Required to prove runtime lifecycle posture before later layers attach |
| runtime state machine | `IN_SCOPE_SLICE_1` | Required to control `STARTING` through shutdown posture |
| execution envelope foundation | `IN_SCOPE_SLICE_1` | Required as the canonical carrier for later protected execution |
| request security middleware foundation | `IN_SCOPE_SLICE_1` | Required as the fail-closed admission seam before Section 04 exists |
| feature flags | `IN_SCOPE_SLICE_1` | Required only as deterministic runtime enablement controls |
| graceful shutdown / panic isolation | `IN_SCOPE_SLICE_1` | Required to keep runtime exit controlled and auditable |
| session state model | `IN_SCOPE_SLICE_1` | Required canonical session-law implementation |
| session identifiers | `IN_SCOPE_SLICE_1` | Required for canonical session and turn scoping |
| session transitions | `IN_SCOPE_SLICE_1` | Required for deterministic lifecycle control |
| attach / resume / recover / detach | `IN_SCOPE_SLICE_1` | Required as session container foundations |
| single-writer rule | `IN_SCOPE_SLICE_1` | Required to prevent concurrent session mutation drift |
| device timeline tracking | `IN_SCOPE_SLICE_1` | Required for deterministic per-device sequencing foundations |
| session ownership / lease / failover foundations | `IN_SCOPE_SLICE_1` | Required as foundations only, not full later-layer distributed recovery behavior |
| snapshot / recovery foundations | `IN_SCOPE_SLICE_1` | Required as bounded session recovery substrate |
| session event stream / observability | `IN_SCOPE_SLICE_1` | Required to audit state transitions and recovery posture |
| Section 03 ingress + turn pipeline implementation | `DOWNSTREAM_DEPENDENCY` | Must attach after Slice 1 stabilizes |
| Section 04 authority implementation | `DOWNSTREAM_DEPENDENCY` | Must attach to the Slice 1 envelope and session substrate |
| Section 05 persistence + sync implementation | `DOWNSTREAM_DEPENDENCY` | Must consume Slice 1 runtime and session boundaries, not redefine them |
| Apple app implementation | `EXPLICIT_NON_GOAL` | Not part of Slice 1 |
| memory implementation beyond Section 02-consumed interfaces | `EXPLICIT_NON_GOAL` | Not part of Slice 1 |
| personality / emotional implementation | `EXPLICIT_NON_GOAL` | Not part of Slice 1 |
| PH1.COMP broad implementation | `EXPLICIT_NON_GOAL` | Not part of Slice 1 |

F) core-runtime component matrix

| Section 01 component | classification | Slice 1 build intent | completion proof |
|---|---|---|---|
| runtime bootstrap shell | `IN_SCOPE_SLICE_1` | Start the runtime process deterministically and establish admission posture | clean startup proof and controlled transition into `READY` |
| service framework and dependency container | `IN_SCOPE_SLICE_1` | Provide stable service registration and internal wiring boundaries | runtime boots with required services only and fails closed on missing required services |
| engine registry and engine discovery | `IN_SCOPE_SLICE_1` | Register only Slice 1 runtime/session surfaces and preserve later attachment seams | registry exposes bounded services without Section 03+ execution bleed |
| environment configuration and secrets handling | `IN_SCOPE_SLICE_1` | Validate required config at startup and keep secret handling inside governed runtime boundaries | startup refuses missing or malformed required config |
| structured logging, metrics, and tracing hooks | `IN_SCOPE_SLICE_1` | Emit canonical runtime lifecycle, admission, and session foundation telemetry | runtime and session events are observable without app-side reconstruction |
| health, readiness, and startup checks | `IN_SCOPE_SLICE_1` | Expose liveness, readiness, and startup posture for runtime lifecycle control | probes reflect `STARTING`, `READY`, `DEGRADED`, `DRAINING`, and shutdown truth |
| runtime lifecycle state machine | `IN_SCOPE_SLICE_1` | Implement the canonical runtime state progression and degrade posture | state transitions are deterministic, ordered, and auditable |
| execution envelope foundation | `IN_SCOPE_SLICE_1` | Define the canonical runtime carrier for request identity, session scoping, and later protected execution context | envelope fields are required, validated, and consistently propagated |
| request security middleware foundation | `IN_SCOPE_SLICE_1` | Enforce baseline admission and envelope validation before later authority logic exists | invalid or incomplete requests fail closed before work begins |
| feature flag surface | `IN_SCOPE_SLICE_1` | Provide deterministic enablement controls for Slice 1 runtime capabilities only | flags do not create alternate authority paths |
| graceful shutdown and panic isolation | `IN_SCOPE_SLICE_1` | Drain safely, preserve audit posture, and prevent uncontrolled crash semantics | shutdown is orderly and panic isolation does not bypass lifecycle state handling |

G) session-engine capability matrix

| Section 02 capability | classification | Slice 1 build intent | completion proof |
|---|---|---|---|
| canonical session state model | `IN_SCOPE_SLICE_1` | Implement `Closed`, `Open`, `Active`, `SoftClosed`, and `Suspended` without local reinterpretation | state transitions are deterministic and validated |
| session identifiers | `IN_SCOPE_SLICE_1` | Establish canonical `session_id` and `turn_id` usage for runtime scoping | identifiers are mandatory where session law requires them |
| session transition rules | `IN_SCOPE_SLICE_1` | Encode lawful transitions and refusal conditions | invalid transitions are rejected with fail-closed posture |
| attach / resume / recover / detach foundations | `IN_SCOPE_SLICE_1` | Support canonical session attachment and bounded recovery foundations | attach and recovery outcomes are explicit, ordered, and auditable |
| single-writer rule | `IN_SCOPE_SLICE_1` | Enforce one session mutation writer at a time | concurrent mutation attempts are refused or serialized deterministically |
| device timeline tracking | `IN_SCOPE_SLICE_1` | Establish per-device sequencing foundations and monotonic timeline posture | device sequencing is monotonic and conflict-visible |
| session ownership / lease / failover foundations | `IN_SCOPE_SLICE_1` | Provide bounded ownership and lease primitives required for later distributed safety | lease and ownership posture exist without inventing later-layer takeover behavior |
| snapshot / recovery foundations | `IN_SCOPE_SLICE_1` | Persist and reload bounded session container state needed for lawful recovery | snapshot load and recovery behavior preserve canonical state semantics |
| session event stream and observability | `IN_SCOPE_SLICE_1` | Emit auditable state-change, recovery, and fail-closed events | session history is observable without client-side guesswork |
| cross-device client rendering behavior | `EXPLICIT_NON_GOAL` | Deferred to client phases and later runtime consumers | no client workarounds introduced in Slice 1 |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 1 role |
|---|---|---|
| runtime contract surfaces | `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_kernel_contracts/src/ph1health.rs`, `crates/selene_kernel_contracts/src/lib.rs` | Contract carriers for runtime lifecycle, health posture, and execution envelope foundations |
| session contract surfaces | `crates/selene_kernel_contracts/src/common.rs`, `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_kernel_contracts/src/ph1lease.rs` | Canonical session states, identifiers, snapshots, lease, and transition contracts |
| runtime bootstrap and orchestration | `crates/selene_os/src/lib.rs`, `crates/selene_os/src/ph1os.rs`, `crates/selene_os/src/ph1health.rs` | Process startup, runtime lifecycle hosting, and health/readiness surfaces |
| session runtime implementation | `crates/selene_os/src/ph1l.rs`, `crates/selene_os/src/ph1lease.rs` | Session container, transition handling, ownership, and recovery foundations |
| runtime guardrail seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs` | Downstream seams that Slice 1 must preserve without pulling full Section 09 or 11 logic forward |
| session storage seam | `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/lib.rs` | Bounded session snapshot and recovery foundations only |
| deferred downstream seam | `crates/selene_os/src/app_ingress.rs` | Not a Slice 1 implementation target; preserved as the later Section 03 integration point |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 1 must be:

1. establish runtime bootstrap, runtime lifecycle states, required configuration validation, and bounded service registration
2. establish structured observability, startup checks, readiness checks, and graceful shutdown posture
3. establish the execution envelope foundation and baseline request security middleware foundation
4. establish canonical session contracts and runtime-backed session state transitions
5. establish session identifiers, attach / resume / recover / detach behavior, and the single-writer rule
6. establish device timeline tracking, ownership and lease foundations, and bounded snapshot / recovery support
7. establish session event emission and final Slice 1 verification closure

Slice 1 must not reverse this order. Later stateful session behavior must not be built before the runtime lifecycle and envelope foundations exist.

J) verification and acceptance matrix

| proof area | required verification | Slice 1 acceptance condition |
|---|---|---|
| runtime startup proof | clean startup, deterministic configuration validation, and orderly transition into `READY` | runtime can start from clean state without ad hoc manual repair |
| health and readiness proof | liveness, startup, and readiness surfaces reflect actual runtime posture | runtime never advertises readiness before Slice 1 services are available |
| runtime lifecycle proof | transitions through startup, ready, degraded, draining, and shutdown states are auditable | runtime state machine is ordered, deterministic, and observable |
| execution envelope proof | required envelope fields are validated and propagated consistently | no Slice 1 work begins without a valid envelope |
| request security proof | invalid or incomplete admission posture is rejected fail closed | baseline middleware foundation blocks unsafe entry rather than guessing |
| session-state proof | lawful session transitions succeed and invalid transitions refuse deterministically | canonical session state semantics are preserved |
| identifier proof | `session_id`, `turn_id`, and device-sequencing foundations remain consistent and monotonic | downstream sections can rely on canonical identifiers without redefinition |
| attach / resume / recover / detach proof | lawful recovery paths preserve state semantics and refusal reasons | recovery behavior is explicit and auditable |
| single-writer proof | concurrent mutation attempts cannot create ambiguous session ownership | only one writer can mutate session state at a time |
| snapshot / recovery proof | bounded session snapshots can be stored and reloaded without semantic drift | session recovery foundations are stable enough for later persistence work |
| fail-closed proof | missing required config, invalid state transitions, incomplete envelopes, and unsafe session mutations refuse cleanly | Slice 1 fails closed where it owns the boundary |
| acceptance closeout | tests, readiness evidence, startup evidence, session-state evidence, and fail-closed evidence all pass on a clean tree | Slice 1 can be declared complete without Section 03+ implementation |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| Section 03 ingress + turn pipeline implementation | `EXPLICIT_NON_GOAL` | do not build canonical turn execution inside Slice 1 |
| Section 04 authority / simulation / proof enforcement | `EXPLICIT_NON_GOAL` | do not pull protected execution logic forward beyond baseline middleware seams |
| Section 05 persistence + sync implementation | `EXPLICIT_NON_GOAL` | do not pull outbox, replay, reconcile, or full sync machinery into Slice 1 |
| Section 06 memory implementation beyond session-consumed interfaces | `EXPLICIT_NON_GOAL` | do not pull memory retrieval or write logic into the session foundation |
| Apple implementation | `EXPLICIT_NON_GOAL` | do not start iPhone or Mac client work from incomplete Slice 1 runtime semantics |
| personality / emotional runtime work | `EXPLICIT_NON_GOAL` | do not introduce tone or emotional logic into runtime/session foundations |
| PH1.COMP broad implementation | `EXPLICIT_NON_GOAL` | do not introduce late-layer quantitative paths into Slice 1 |
| no client authority | `IN_SCOPE_SLICE_1` | no device or client surface may authoritatively own runtime or session truth |
| no Section 03+ implementation bleed | `IN_SCOPE_SLICE_1` | later sections may define seams only; they may not become hidden Slice 1 work |
| no app-layer workarounds | `IN_SCOPE_SLICE_1` | clients must not compensate for missing runtime/session law |
| no session-law drift | `IN_SCOPE_SLICE_1` | do not reinterpret canonical states, identifiers, transition order, or writer rules |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_1` | all Slice 1 work must enter through the canonical runtime envelope foundation |

L) SLICE 1 COMPLETION STANDARD

Slice 1 is complete only when the runtime kernel and canonical session container are both present, bounded to Sections 01 and 02, and proven by acceptance and verification evidence. Completion requires:

- deterministic runtime startup and shutdown posture
- health, readiness, and lifecycle evidence
- canonical session state and transition evidence
- single-writer, identifier, and recovery evidence
- fail-closed behavior where Slice 1 owns the boundary
- proof that Apple implementation is still deferred

M) PHASE BOUNDARY

This plan governs Slice 1 only. Section 03 and later remain downstream work. Apple implementation is not part of Slice 1 and may begin only after the Slice 1 runtime/session spine is stable enough to support thin lawful clients without local workaround architecture.
