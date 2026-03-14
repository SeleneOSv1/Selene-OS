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
| request routing layer | `IN_SCOPE_SLICE_1` | Required only as the runtime boundary routing shell; it must not embed the Section 03 canonical turn pipeline |
| environment configuration / config governance / secrets | `IN_SCOPE_SLICE_1` | Required to start deterministically and fail closed on missing config |
| runtime clock service | `IN_SCOPE_SLICE_1` | Required for canonical timestamps, ordering, and later deterministic replay alignment |
| global error model | `IN_SCOPE_SLICE_1` | Required so runtime refusals, retries, and fatal conditions stay classified consistently from the start |
| configuration governance and secure secrets provider | `IN_SCOPE_SLICE_1` | Required so configuration access, secret injection, and secret redaction stay governed rather than ad hoc |
| dependency graph validation | `IN_SCOPE_SLICE_1` | Required to reject invalid runtime dependency wiring before the runtime becomes ready |
| observability / structured logging / metrics / tracing hooks | `IN_SCOPE_SLICE_1` | Required to make runtime and session behavior auditable from the start |
| health / readiness / startup checks | `IN_SCOPE_SLICE_1` | Required to prove runtime lifecycle posture before later layers attach |
| startup self-check / preflight | `IN_SCOPE_SLICE_1` | Required to block startup when required runtime foundations are missing or inconsistent |
| runtime state machine | `IN_SCOPE_SLICE_1` | Required to control `STARTING` through shutdown posture |
| execution envelope foundation | `IN_SCOPE_SLICE_1` | Required as the canonical carrier for later protected execution |
| request security middleware foundation | `IN_SCOPE_SLICE_1` | Required as the fail-closed admission seam before Section 04 exists |
| execution-budget propagation | `IN_SCOPE_SLICE_1` | Required so runtime resource budgets stay explicit and consistent across startup, routing, admission, and later runtime stages |
| timeout, execution budget, and backpressure foundations | `IN_SCOPE_SLICE_1` | Required to keep Slice 1 runtime behavior bounded under load and before deeper execution exists |
| redaction framework | `IN_SCOPE_SLICE_1` | Required to prevent raw sensitive runtime values from leaking through logs, traces, and diagnostics |
| diagnostic mode | `IN_SCOPE_SLICE_1` | Required as a governed deep-debug posture that stays inside runtime law and redaction boundaries |
| runtime identity, build metadata, and capability manifest | `IN_SCOPE_SLICE_1` | Required so every runtime instance identifies itself and its bounded capabilities consistently |
| admission control | `IN_SCOPE_SLICE_1` | Required to refuse work safely before deeper execution begins under unsafe runtime conditions |
| invariant checker | `IN_SCOPE_SLICE_1` | Required to block invalid startup or runtime combinations before they drift into execution |
| feature flags | `IN_SCOPE_SLICE_1` | Required only as deterministic runtime enablement controls |
| request classification and rate limiting foundations | `IN_SCOPE_SLICE_1` | Required to preserve real-time runtime posture and basic abuse-defense before later slices expand |
| standardized metrics schema | `IN_SCOPE_SLICE_1` | Required so runtime metrics, labels, and units stay canonical and comparable across kernel surfaces |
| circuit breaker, dependency health graph, and resource guardrails | `IN_SCOPE_SLICE_1` | Required to keep the runtime stable under dependency or capacity failure |
| internal runtime event bus | `IN_SCOPE_SLICE_1` | Required to coordinate lifecycle and health events without introducing hidden coupling |
| replay foundation | `IN_SCOPE_SLICE_1` | Required to support deterministic debugging and envelope-based replay later without retrofitting the kernel |
| service-level objectives / latency governance | `IN_SCOPE_SLICE_1` | Required so runtime performance posture is defined and observable before later slices attach heavier execution |
| cold start, safe runtime upgrade, and multi-region failover foundations | `IN_SCOPE_SLICE_1` | Required to keep the runtime kernel operable under restart, rollout, and failover conditions |
| data residency / retention governance | `IN_SCOPE_SLICE_1` | Required so runtime outputs, diagnostics, and bounded recovery artifacts stay governed from Slice 1 onward |
| runtime sandbox mode | `IN_SCOPE_SLICE_1` | Required only as a controlled non-authoritative isolation surface inside the runtime kernel |
| cryptographic execution proof foundation | `IN_SCOPE_SLICE_1` | Required as the runtime primitive layer for later PH1.J and runtime governance integration, not as Section 04 enforcement |
| simulation registry hardening, dependency trust grading, and gold-path certification foundations | `IN_SCOPE_SLICE_1` | Required as kernel-level hardening primitives that later sections will rely on without redefining them |
| graceful shutdown / panic isolation | `IN_SCOPE_SLICE_1` | Required to keep runtime exit controlled and auditable |
| session state model | `IN_SCOPE_SLICE_1` | Required canonical session-law implementation |
| session identifiers | `IN_SCOPE_SLICE_1` | Required for canonical session and turn scoping |
| session transitions | `IN_SCOPE_SLICE_1` | Required for deterministic lifecycle control |
| session exposure to runtime response layer | `IN_SCOPE_SLICE_1` | Required so `session_id`, `turn_id`, and `session_state` stay explicit to downstream runtime surfaces |
| attach / resume / recover / detach | `IN_SCOPE_SLICE_1` | Required as session container foundations |
| single-writer rule | `IN_SCOPE_SLICE_1` | Required to prevent concurrent session mutation drift |
| device timeline tracking | `IN_SCOPE_SLICE_1` | Required for deterministic per-device sequencing foundations |
| session partitioning and cluster coordination foundations | `IN_SCOPE_SLICE_1` | Required to keep session mutation ownership explicit in distributed runtime posture |
| session ownership / lease / failover foundations | `IN_SCOPE_SLICE_1` | Required as foundations only, not full later-layer distributed recovery behavior |
| ownership transfer | `IN_SCOPE_SLICE_1` | Required so session mutation ownership can move safely without split-brain behavior |
| coordination-state exposure | `IN_SCOPE_SLICE_1` | Required so ownership certainty and recovery posture stay visible rather than implicit |
| consistency levels | `IN_SCOPE_SLICE_1` | Required so downstream systems can distinguish normal versus degraded session safety posture |
| access classes | `IN_SCOPE_SLICE_1` | Required so active, view-only, limited, and recovery attach roles remain explicit and enforceable |
| conflict resolution | `IN_SCOPE_SLICE_1` | Required to handle concurrent attach, retry, resume, and stale-turn contention deterministically |
| integrity checks | `IN_SCOPE_SLICE_1` | Required to detect impossible state, owner loss, corrupted snapshot posture, and invalid device timelines fail closed |
| session certification targets | `IN_SCOPE_SLICE_1` | Required to make session correctness measurable rather than implied |
| cluster coordination certification targets | `IN_SCOPE_SLICE_1` | Required to make distributed ownership safety measurable rather than implied |
| session backpressure | `IN_SCOPE_SLICE_1` | Required to prevent an overloaded session from destabilizing the runtime kernel |
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
| request routing layer | `IN_SCOPE_SLICE_1` | Accept requests into the runtime boundary and forward only into Slice 1 admission surfaces | request routing exists without embedding the Section 03 canonical turn pipeline |
| environment configuration and secrets handling | `IN_SCOPE_SLICE_1` | Validate required config at startup and keep secret handling inside governed runtime boundaries | startup refuses missing or malformed required config |
| runtime clock service | `IN_SCOPE_SLICE_1` | Provide canonical time, monotonic ordering support, and clock access to runtime services | runtime timestamps and ordering signals remain consistent across startup, lifecycle, and replay surfaces |
| global error model | `IN_SCOPE_SLICE_1` | Keep runtime-wide refusal and retry posture deterministic before deeper execution exists | errors classify consistently and do not devolve into ad hoc runtime-specific shapes |
| configuration governance and secure secrets provider | `IN_SCOPE_SLICE_1` | Govern config schema, secret injection, redaction, and environment separation | config and secret access remain governed and fail closed |
| dependency graph validation | `IN_SCOPE_SLICE_1` | Reject invalid dependency wiring, cycles, and startup ordering faults before readiness | invalid runtime dependency graphs block startup cleanly |
| structured logging, metrics, and tracing hooks | `IN_SCOPE_SLICE_1` | Emit canonical runtime lifecycle, admission, and session foundation telemetry | runtime and session events are observable without app-side reconstruction |
| health, readiness, and startup checks | `IN_SCOPE_SLICE_1` | Expose liveness, readiness, and startup posture for runtime lifecycle control | probes reflect `STARTING`, `READY`, `DEGRADED`, `DRAINING`, and shutdown truth |
| startup self-check / preflight | `IN_SCOPE_SLICE_1` | Verify configuration, dependency, observability, and critical service posture before readiness | the runtime refuses to claim readiness when foundational checks fail |
| runtime lifecycle state machine | `IN_SCOPE_SLICE_1` | Implement the canonical runtime state progression and degrade posture | state transitions are deterministic, ordered, and auditable |
| execution envelope foundation | `IN_SCOPE_SLICE_1` | Define the canonical runtime carrier for request identity, session scoping, and later protected execution context | envelope fields are required, validated, and consistently propagated |
| request security middleware foundation | `IN_SCOPE_SLICE_1` | Enforce baseline admission and envelope validation before later authority logic exists | invalid or incomplete requests fail closed before work begins |
| execution-budget propagation | `IN_SCOPE_SLICE_1` | Carry execution budgets across runtime stages so resource posture remains explicit from admission onward | execution budgets propagate deterministically and remain visible to downstream kernel services |
| timeout, execution budget, request classification, and backpressure foundations | `IN_SCOPE_SLICE_1` | Bound request behavior before deep execution exists and prevent runtime overload drift | the runtime keeps admission, timeout, and overload posture explicit and deterministic |
| redaction framework | `IN_SCOPE_SLICE_1` | Sanitize sensitive values across logs, traces, diagnostics, and errors | sensitive runtime values never appear raw in observable outputs |
| diagnostic mode | `IN_SCOPE_SLICE_1` | Provide a governed deep-debug posture without bypassing admission, redaction, or lifecycle controls | diagnostic mode can be enabled safely without creating hidden execution paths or raw-data leakage |
| runtime identity, build metadata, and capability manifest | `IN_SCOPE_SLICE_1` | Expose node identity, build posture, and active kernel capabilities deterministically | operators can identify the runtime instance and its bounded capability set without guesswork |
| admission control and invariant checker | `IN_SCOPE_SLICE_1` | Refuse unsafe work before deep execution and reject invalid foundational runtime combinations | invalid runtime posture blocks work before deeper sections attach |
| feature flag surface | `IN_SCOPE_SLICE_1` | Provide deterministic enablement controls for Slice 1 runtime capabilities only | flags do not create alternate authority paths |
| standardized metrics schema | `IN_SCOPE_SLICE_1` | Keep runtime metrics names, labels, units, and cardinality posture canonical from the kernel outward | emitted runtime metrics conform to one standardized schema rather than local conventions |
| event bus and replay foundation | `IN_SCOPE_SLICE_1` | Coordinate lifecycle events and preserve deterministic replay hooks without retrofitting the kernel later | lifecycle events are publishable and replay hooks exist without creating alternate execution paths |
| circuit breaker, dependency health, and resource guardrails | `IN_SCOPE_SLICE_1` | Preserve runtime stability under dependency failure, saturation, and degraded capacity | the runtime degrades or refuses safely instead of drifting into uncontrolled failure |
| service-level objectives / latency governance | `IN_SCOPE_SLICE_1` | Define runtime service objectives and latency posture before later slices attach deeper execution | runtime latency governance is observable, bounded, and not left to app-side interpretation |
| cold start, safe runtime upgrade, runtime sandbox, and failover foundations | `IN_SCOPE_SLICE_1` | Keep restart, rollout, isolated non-authoritative execution, and failover posture inside the kernel | these operational modes exist without creating alternate authority paths |
| data residency / retention governance | `IN_SCOPE_SLICE_1` | Define governance for runtime diagnostics, replay artifacts, and bounded Slice 1 data handling | runtime data location and retention posture remain governed and auditable from the kernel onward |
| cryptographic proof and certification foundations | `IN_SCOPE_SLICE_1` | Provide runtime primitives for later proof, dependency trust, simulation registry hardening, and gold-path certification work | later Sections 04, 09, and 11 can attach without inventing parallel kernel primitives |
| graceful shutdown and panic isolation | `IN_SCOPE_SLICE_1` | Drain safely, preserve audit posture, and prevent uncontrolled crash semantics | shutdown is orderly and panic isolation does not bypass lifecycle state handling |

G) session-engine capability matrix

| Section 02 capability | classification | Slice 1 build intent | completion proof |
|---|---|---|---|
| canonical session state model | `IN_SCOPE_SLICE_1` | Implement `Closed`, `Open`, `Active`, `SoftClosed`, and `Suspended` without local reinterpretation | state transitions are deterministic and validated |
| session identifiers | `IN_SCOPE_SLICE_1` | Establish canonical `session_id` and `turn_id` usage for runtime scoping | identifiers are mandatory where session law requires them |
| session transition rules | `IN_SCOPE_SLICE_1` | Encode lawful transitions and refusal conditions | invalid transitions are rejected with fail-closed posture |
| session exposure to runtime response layer | `IN_SCOPE_SLICE_1` | Keep `session_id`, `turn_id`, and `session_state` explicit to downstream runtime and client-facing response surfaces | session-bound outputs expose the canonical synchronization anchors |
| attach / resume / recover / detach foundations | `IN_SCOPE_SLICE_1` | Support canonical session attachment and bounded recovery foundations | attach and recovery outcomes are explicit, ordered, and auditable |
| single-writer rule | `IN_SCOPE_SLICE_1` | Enforce one session mutation writer at a time | concurrent mutation attempts are refused or serialized deterministically |
| device timeline tracking | `IN_SCOPE_SLICE_1` | Establish per-device sequencing foundations and monotonic timeline posture | device sequencing is monotonic and conflict-visible |
| session partitioning and cluster coordination foundations | `IN_SCOPE_SLICE_1` | Provide the distributed ownership substrate required for deterministic session locality and recovery posture | session ownership posture remains explicit across runtime nodes |
| session ownership / lease / failover foundations | `IN_SCOPE_SLICE_1` | Provide bounded ownership and lease primitives required for later distributed safety | lease and ownership posture exist without inventing later-layer takeover behavior |
| ownership transfer | `IN_SCOPE_SLICE_1` | Support deterministic handoff of session ownership between runtime nodes without split-brain mutation | ownership transfer is acknowledged, drained, and verified explicitly |
| coordination-state exposure | `IN_SCOPE_SLICE_1` | Expose ownership certainty and failover posture to downstream runtime systems explicitly | coordination posture is visible and can drive safe refusal behavior |
| consistency levels | `IN_SCOPE_SLICE_1` | Make strict, leased, and degraded recovery posture explicit rather than implicit | downstream systems can distinguish normal versus degraded session safety |
| access classes | `IN_SCOPE_SLICE_1` | Distinguish active, limited, recovery, and observer attach roles deterministically | device participation posture is explicit and enforceable |
| conflict resolution | `IN_SCOPE_SLICE_1` | Resolve simultaneous attach, retry, resume, and stale-turn contention deterministically | conflicting session interactions do not create ambiguous outcomes |
| integrity checks | `IN_SCOPE_SLICE_1` | Detect invalid lifecycle transitions, owner loss, corrupted snapshot posture, and impossible device timelines | compromised session integrity fails safe immediately |
| cluster coordination certification targets | `IN_SCOPE_SLICE_1` | Make distributed ownership and failover correctness measurable | coordination safety can be certified rather than assumed |
| session certification targets | `IN_SCOPE_SLICE_1` | Make session-first, single-writer, lease, timeline, and recovery correctness measurable | session correctness can be certified rather than assumed |
| session backpressure | `IN_SCOPE_SLICE_1` | Detect overloaded sessions and apply deterministic throttle, defer, or refusal posture | a single overloaded session cannot destabilize the runtime |
| snapshot / recovery foundations | `IN_SCOPE_SLICE_1` | Persist and reload bounded session container state needed for lawful recovery | snapshot load and recovery behavior preserve canonical state semantics |
| session event stream and observability | `IN_SCOPE_SLICE_1` | Emit auditable state-change, recovery, and fail-closed events | session history is observable without client-side guesswork |
| cross-device client rendering behavior | `EXPLICIT_NON_GOAL` | Deferred to client phases and later runtime consumers | no client workarounds introduced in Slice 1 |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 1 role |
|---|---|---|
| runtime contract surfaces | `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_kernel_contracts/src/ph1health.rs`, `crates/selene_kernel_contracts/src/lib.rs` | Contract carriers for runtime lifecycle, request routing boundaries, health posture, execution envelope, and startup readiness foundations |
| session contract surfaces | `crates/selene_kernel_contracts/src/common.rs`, `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_kernel_contracts/src/ph1lease.rs` | Canonical session states, identifiers, snapshots, lease, and transition contracts |
| runtime bootstrap and orchestration | `crates/selene_os/src/lib.rs`, `crates/selene_os/src/ph1os.rs`, `crates/selene_os/src/ph1health.rs` | Process startup, request routing shell, runtime clock, health/readiness, admission control, invariant checker, execution-budget propagation, and lifecycle hosting |
| runtime observability and replay foundations | `crates/selene_os/src/lib.rs`, `crates/selene_os/src/ph1health.rs`, `crates/selene_kernel_contracts/src/runtime_execution.rs` | Standardized metrics schema, diagnostic mode, event bus, replay foundation, runtime identity, capability manifest, and latency-observability posture |
| runtime governance and retention foundations | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs` | Service-level objectives / latency governance plus data residency / retention governance foundations that Slice 1 must define without pulling Section 05 forward |
| session runtime implementation | `crates/selene_os/src/ph1l.rs`, `crates/selene_os/src/ph1lease.rs` | Session container, transition handling, response exposure, ownership transfer, coordination-state, conflict resolution, integrity checks, backpressure, and recovery foundations |
| runtime guardrail seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs` | Downstream seams that Slice 1 must preserve without pulling full Section 09 or 11 logic forward |
| session storage seam | `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/lib.rs` | Bounded session snapshot and recovery foundations only |
| deferred downstream seam | `crates/selene_os/src/app_ingress.rs` | Not a Slice 1 implementation target; preserved as the later Section 03 integration point |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 1 must be:

1. establish runtime bootstrap, service framework, dependency injection, engine registry, and the request routing layer as a runtime shell only
2. establish configuration governance, secure secrets provider posture, runtime clock service, global error model, and runtime identity / build metadata foundations
3. establish structured observability, the standardized metrics schema, health and readiness, startup self-check / preflight, dependency graph validation, and the invariant checker
4. establish admission control, request classification, execution-budget propagation, timeout posture, backpressure foundations, circuit breaker posture, dependency health visibility, resource guardrails, and service-level objectives / latency governance
5. establish the execution envelope foundation, request security middleware foundation, redaction framework, diagnostic mode, data residency / retention governance, event bus, replay foundation, runtime capability manifest, and graceful shutdown / panic isolation
6. establish cold start, safe runtime upgrade, runtime sandbox, failover, proof foundation, and certification hardening primitives without pulling Section 04 or Section 09 logic forward
7. establish canonical session contracts, session state transitions, session identifiers, and session exposure to the runtime response layer
8. establish attach / resume / recover / detach behavior, the single-writer rule, device timeline tracking, access classes, and conflict resolution
9. establish session partitioning, ownership / lease / failover posture, ownership transfer, coordination-state exposure, and consistency levels
10. establish snapshot / recovery foundations, integrity checks, session backpressure, certification targets, and session event / observability closure
11. establish final Slice 1 verification closure against every mandatory Section 01 and Section 02 foundation captured in this document

Slice 1 must not reverse this order. Later stateful session behavior must not be built before the runtime lifecycle and envelope foundations exist.

J) verification and acceptance matrix

| proof area | required verification | Slice 1 acceptance condition |
|---|---|---|
| runtime startup proof | clean startup, deterministic configuration validation, and orderly transition into `READY` | runtime can start from clean state without ad hoc manual repair |
| request routing proof | request routing accepts requests only into the Slice 1 runtime shell and never embeds Section 03 execution logic | request routing exists without Section 03 bleed |
| runtime clock proof | canonical runtime timestamps and ordering signals remain stable across lifecycle and replay surfaces | runtime clock posture is deterministic enough for later replay and proof consumers |
| global error model proof | runtime refusals, retryable failures, and fatal startup conditions classify consistently | runtime failure posture does not drift into ad hoc behavior |
| configuration governance and dependency graph validation proof | required config, secret posture, and dependency graph validity are checked before readiness | invalid configuration or dependency posture blocks startup |
| health and readiness proof | liveness, startup, and readiness surfaces reflect actual runtime posture | runtime never advertises readiness before Slice 1 services are available |
| startup self-check / preflight proof | startup self-check / preflight blocks invalid runtime startup before traffic is accepted | required runtime foundations are verified before readiness |
| runtime lifecycle proof | transitions through startup, ready, degraded, draining, and shutdown states are auditable | runtime state machine is ordered, deterministic, and observable |
| execution envelope proof | required envelope fields are validated and propagated consistently | no Slice 1 work begins without a valid envelope |
| request security proof | invalid or incomplete admission posture is rejected fail closed | baseline middleware foundation blocks unsafe entry rather than guessing |
| admission control and invariant checker proof | unsafe runtime posture, invalid combinations, and overload conditions refuse before deep execution begins | the runtime blocks unsafe work before later sections attach |
| execution-budget propagation proof | execution budgets remain explicit and correctly propagated across runtime stages, routing shells, and bounded kernel services | resource posture stays deterministic across the Slice 1 runtime path |
| diagnostic mode proof | diagnostic mode can be enabled in a governed way without bypassing admission, redaction, or lifecycle controls | deep debugging exists without creating hidden execution or data-leak paths |
| standardized metrics schema proof | runtime metrics emit with canonical names, labels, units, and bounded cardinality | runtime telemetry is comparable and machine-consumable without local reinterpretation |
| event bus and replay foundation proof | lifecycle events are publishable and replay artifacts can be captured without creating alternate execution paths | runtime coordination and deterministic replay hooks exist from the kernel onward |
| service-level objectives / latency governance proof | runtime objectives and latency posture are defined, observable, and enforced at the kernel boundary | latency governance exists before later slices add heavier execution |
| data residency / retention governance proof | runtime diagnostics, replay artifacts, and bounded Slice 1 data handling follow explicit residency and retention posture | runtime data governance is auditable and not left to local convention |
| runtime control-surface proof | feature flags, request classification, redaction, runtime identity, capability manifest, guardrails, and graceful shutdown all behave deterministically | operators can observe and govern the runtime without local workaround logic |
| session-state proof | lawful session transitions succeed and invalid transitions refuse deterministically | canonical session state semantics are preserved |
| identifier proof | `session_id`, `turn_id`, and device-sequencing foundations remain consistent and monotonic | downstream sections can rely on canonical identifiers without redefinition |
| session exposure proof | `session_id`, `turn_id`, and `session_state` remain explicit to downstream runtime response surfaces | clients and later layers receive canonical session anchors rather than inferred state |
| attach / resume / recover / detach proof | lawful recovery paths preserve state semantics and refusal reasons | recovery behavior is explicit and auditable |
| single-writer proof | concurrent mutation attempts cannot create ambiguous session ownership | only one writer can mutate session state at a time |
| ownership transfer and coordination-state proof | session ownership handoff, coordination-state exposure, and failover posture remain explicit and safe | distributed session control does not drift into split-brain ambiguity |
| consistency levels, access classes, and conflict resolution proof | degraded safety posture, attach roles, and concurrent interaction conflicts stay explicit and deterministic | session safety posture remains machine-visible and enforceable |
| integrity checks and session backpressure proof | impossible session states, corrupted recovery posture, and overloaded sessions fail safe or throttle deterministically | session integrity and overload posture do not destabilize the runtime |
| snapshot / recovery proof | bounded session snapshots can be stored and reloaded without semantic drift | session recovery foundations are stable enough for later persistence work |
| certification-target proof | session certification targets and cluster coordination certification targets are defined and observable | Slice 1 correctness is measurable rather than implied |
| fail-closed proof | missing required config, invalid state transitions, incomplete envelopes, and unsafe session mutations refuse cleanly | Slice 1 fails closed where it owns the boundary |
| acceptance closeout | tests, readiness evidence, startup evidence, request routing evidence, runtime control-surface evidence, session-state evidence, distributed coordination evidence, and fail-closed evidence all pass on a clean tree | Slice 1 can be declared complete only when every mandatory Section 01 and Section 02 foundation captured here is proven without Section 03+ implementation |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| request routing shell versus Section 03 ingress | `IN_SCOPE_SLICE_1` | request routing may exist only as a runtime shell and must not become a hidden Section 03 canonical turn path |
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
| no client-owned coordination or ownership transfer | `IN_SCOPE_SLICE_1` | ownership transfer, coordination-state, conflict resolution, and session backpressure remain cloud session-engine responsibilities only |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_1` | all Slice 1 work must enter through the canonical runtime envelope foundation |
| no incomplete completion claim | `IN_SCOPE_SLICE_1` | Slice 1 must not be called complete while any mandatory Section 01 or Section 02 foundation remains uncaptured or unverified in this document |

L) SLICE 1 COMPLETION STANDARD

Slice 1 is complete only when the runtime kernel and canonical session container are both present, bounded to Sections 01 and 02, and proven by acceptance and verification evidence. Completion requires:

- deterministic runtime startup and shutdown posture
- request routing, runtime clock, global error model, configuration governance, dependency graph validation, startup self-check / preflight, admission control, invariant checker, execution-budget propagation, and diagnostic mode evidence
- execution envelope, request security, redaction, standardized metrics schema, event bus, replay foundation, runtime identity, capability manifest, runtime control-surface evidence, and service-level objectives / latency governance evidence
- data residency / retention governance evidence for bounded Slice 1 runtime diagnostics, replay artifacts, and governed data-handling posture
- canonical session state, identifiers, session exposure, attach / resume / recover / detach, single-writer, device timeline, ownership transfer, coordination-state, consistency levels, access classes, conflict resolution, integrity checks, session backpressure, and recovery evidence
- certification-target evidence for both session correctness and distributed coordination posture
- fail-closed behavior where Slice 1 owns the boundary
- proof that Section 03 and later remain deferred
- proof that Apple implementation is still deferred
- proof that no mandatory Section 01 or Section 02 foundation is missing from the scope matrix, runtime/session matrices, internal implementation order, verification matrix, or completion standard
- proof that no mandatory Section 01 or Section 02 foundation remains uncaptured, unordered, or unverified when Slice 1 is declared complete
- proof that no mandatory Section 01 or Section 02 foundation remains uncaptured or unverified in this plan

M) PHASE BOUNDARY

This plan governs Slice 1 only. Section 03 and later remain downstream work. Apple implementation is not part of Slice 1 and may begin only after the Slice 1 runtime/session spine is stable enough to support thin lawful clients without local workaround architecture.
