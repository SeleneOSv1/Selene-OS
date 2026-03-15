PHASE H3 — SLICE 2A CANONICAL INGRESS AND TURN BUILD PLAN

A) PURPOSE

This document converts the next lawful implementation step after accepted Slice 1A, Slice 1B, Slice 1C, and Slice 1D into a build-grade execution plan.

Slice 2A is the first bounded Section 03 slice only. It establishes the canonical ingress and turn-foundation layer that sits on top of the accepted runtime and session substrate, and it stops before real Section 04 authority execution or Section 05 persistence execution.

H3 is planning law only. It does not authorize code changes outside the future bounded Slice 2A implementation run, it does not reopen frozen design law, and it does not authorize Apple or app/client implementation.

B) FROZEN LAW INPUTS

Slice 2A is derived from the following binding inputs:

- frozen A1-A6, B1-B5, C1-C5, D1-D5, E1-E5, F1-F5, and G1-G2 as upstream design-law constraints
- [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) as the sequencing-law anchor
- [H2](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H2_SLICE1_RUNTIME_AND_SESSION_BUILD_PLAN.md) as the accepted Slice 1 implementation-law anchor
- [Build Section 01](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md), [Build Section 02](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md), and [Build Section 03](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md) as the main governing implementation sections
- [Build Section 04](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md), [Build Section 05](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md), [Build Section 09](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md), and [Build Section 11](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md) as downstream dependency law that Slice 2A must preserve without pulling forward
- [CORE_ARCHITECTURE](docs/CORE_ARCHITECTURE.md), [SELENE_BUILD_EXECUTION_ORDER](docs/SELENE_BUILD_EXECUTION_ORDER.md), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY](docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md), [COVERAGE_MATRIX](docs/COVERAGE_MATRIX.md), [11_DESIGN_LOCK_SEQUENCE](docs/11_DESIGN_LOCK_SEQUENCE.md), and [02_BUILD_PLAN](docs/02_BUILD_PLAN.md) as architecture, sequencing, and readiness truth

Accepted implementation baselines consumed by this plan:

- Slice 1A accepted at `f9769797c28bc991df3720d085639a7117b3d7c8`
- Slice 1B accepted at `eb4be3fdfc100fa22684293cacee471faf7d7847`
- Slice 1C accepted at `b1eba355e716887f0fe399cc6930988e0423e7db`
- Slice 1D accepted at `46b021dece36b9c1d8589362cf0ada0187603a83`

These accepted slices establish non-negotiable repo truth for Slice 2A:

- the runtime bootstrap, lifecycle, health, build metadata, and fail-closed startup substrate already exist
- the runtime routing, envelope, request-security, admission, feature-flag, metrics, and event-bus substrate already exist
- the session runtime, identifiers, attach/resume/recover/detach behavior, single-writer rule, device timeline, coordination posture, ownership/lease, transfer, access classes, conflict resolution, and session backpressure substrate already exist
- the canonical `RuntimeExecutionEnvelope` carrier already exists and already reserves downstream persistence, governance, proof, identity, memory, authority, artifact-trust, and law fields

Sections 04 and 05 remain downstream dependencies only. They define boundaries that Slice 2A must preserve, not execution work that Slice 2A may implement.

C) CURRENT / TARGET / GAP

CURRENT

- Repo truth already contains the Slice 1 runtime and session substrate required by [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) before Section 03 may begin.
- [runtime_request_foundation.rs](crates/selene_os/src/runtime_request_foundation.rs) already provides the accepted route shell, request-security foundation, feature-flag registry, admission controller, runtime event bus, metrics collector, and fail-closed route guardrails.
- Slice 1B intentionally blocks `/v1/` route registration so that no Section 03 ingress path can appear early or by accident.
- [runtime_session_foundation.rs](crates/selene_os/src/runtime_session_foundation.rs) already provides the accepted session and turn substrate that Section 03 must consume instead of reimplementing.
- [runtime_execution.rs](crates/selene_kernel_contracts/src/runtime_execution.rs) already provides the canonical `RuntimeExecutionEnvelope`, `AdmissionState`, `FailureClass`, `SessionAttachOutcome`, `PlatformRuntimeContext`, and downstream envelope seam fields.

TARGET

- Deliver the first bounded Section 03 implementation-law plan for canonical ingress and turn foundations.
- Authorize only the first lawful turn-ingress layer: canonical request-family foundations, canonical `/v1/voice/turn` entry, deterministic normalization, session binding, envelope creation, pre-authority stage order, fail-closed rejection posture, and bounded turn-start observability.
- Keep identity, policy, authorization, simulation execution, durable replay, persistence sync, governance, runtime law, and client/app behavior downstream.

GAP

- The repo does not yet contain a lawful Section 03 route family or canonical executable turn path.
- The repo does not yet contain a bounded normalized turn-request carrier that bridges Slice 1B routing/security/admission into the accepted `RuntimeExecutionEnvelope`.
- The repo does not yet contain the pre-authority stage-order scaffold that must exist before real Section 04 or Section 05 behavior can attach.
- The remaining need is disciplined Section 03 execution law: precise scope, stage boundaries, proof obligations, and guardrails against Section 04/05 or client bleed.

D) SLICE 2A DELIVERY POSITION

H1 defines Slice 2 as the establishment of the canonical execution path across Sections 03-05. Slice 2A is the first bounded subsection of that work. It is still Section 03 only.

Slice 2A sits:

- downstream of accepted Slice 1A runtime bootstrap and lifecycle law
- downstream of accepted Slice 1B routing, request-security, admission, and envelope-base law
- downstream of accepted Slice 1C/1D session, turn-order, coordination, ownership, access, conflict, and backpressure law
- upstream of Section 04 identity, access, authorization, simulation, and authority execution
- upstream of Section 05 durable idempotency, persistence outcome reuse, reconciliation, and cross-node deduplication

Slice 2A must therefore do four things at once:

- consume the accepted Slice 1 substrate exactly as built
- open the first lawful Section 03 route family and canonical turn path
- stop at the pre-authority boundary without inventing a hidden Section 04 path
- preserve all downstream Section 04, Section 05, Section 09, and Section 11 seams inside the envelope and stage model without implementing them

E) slice-2A scope and dependency matrix

| item | classification | Slice 2A position | dependency / guardrail |
|---|---|---|---|
| canonical ingress request-family registry for `/v1/voice/turn`, `/v1/invite/click`, and `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2A` | establish the lawful Section 03 ingress family without broadening execution scope | consume Slice 1B routing substrate; only `/v1/voice/turn` becomes executable in Slice 2A |
| canonical `/v1/voice/turn` route | `IN_SCOPE_SLICE_2A` | first executable Section 03 route | must become the single executable turn-ingress path |
| executable modality convergence (`voice`, `text`, `file`, `image`, `camera`) into one canonical turn family | `IN_SCOPE_SLICE_2A` | normalize all executable user turns into one governed turn-entry path | route name stays `/v1/voice/turn`; no modality-specific alternate routes |
| broader invite/onboarding route-family compatibility semantics | `IN_SCOPE_SLICE_2A` | preserve canonical family membership and compatibility posture | do not pull actual invite/onboarding behavior into this slice |
| invite click execution behavior | `DEFER_LATER_SECTION03` | not part of the first bounded turn slice | preserve route-family compatibility only |
| onboarding continue execution behavior | `DEFER_LATER_SECTION03` | not part of the first bounded turn slice | preserve route-family compatibility only |
| canonical normalized turn-request carrier | `IN_SCOPE_SLICE_2A` | create the deterministic request object that sits between Slice 1B routing and the runtime envelope | must remain additive and envelope-disciplined |
| ingress normalization (schema validation, content-type normalization, canonical field mapping, size limits, request-content hash) | `IN_SCOPE_SLICE_2A` | establish deterministic pre-execution input shape | fail closed before session or envelope mutation |
| deep modality-specific media semantics beyond normalization boundaries | `DEFER_LATER_SECTION03` | not part of the first ingress foundation slice | no special-case voice/file/image/camera engine logic here |
| ingress-owned request envelope validation (`Authorization`, request ids, nonce, timestamp, idempotency key, platform, device, actor, payload carrier) | `IN_SCOPE_SLICE_2A` | validate all ingress-owned fields before turn execution begins | use Slice 1B security foundation where already present; extend only what Section 03 owns |
| platform trigger validation and trigger-to-session convergence | `IN_SCOPE_SLICE_2A` | consume canonical `PlatformRuntimeContext` and `RuntimeEntryTrigger` truth | wake vs explicit must converge into one session-bound turn path |
| session resolve-or-open and turn allocation | `IN_SCOPE_SLICE_2A` | bind Section 03 to accepted Slice 1C/1D session truth | consume `create_session`, `start_new_session_turn`, `attach_session`, `resume_session`, `begin_turn`; do not reimplement session logic |
| canonical `RuntimeExecutionEnvelope` creation and session binding | `IN_SCOPE_SLICE_2A` | create the first full Section 03 execution carrier after session/turn resolution | no raw-request execution past this point |
| replay-protection hook surface (nonce/timestamp/request identity validation and deterministic pre-authority duplicate posture) | `IN_SCOPE_SLICE_2A` | provide the ingress-owned replay hook required before later persistence exists | no durable processed-request store in this slice |
| durable replay detection, duplicate-outcome lookup, and reconnect-safe reuse | `DEFER_SECTION05` | not lawful without persistence and sync machinery | no local or in-memory pseudo-persistence substitute |
| idempotency key propagation and turn-bound idempotency hook posture | `IN_SCOPE_SLICE_2A` | carry stable operation identity into the runtime envelope and turn-start path | must stop before Section 05 durable dedupe execution |
| same-key authoritative result reuse across reconnects, retries, and distributed nodes | `DEFER_SECTION05` | depends on outbox, journal, dedupe, and persistence state | no Section 05 behavior inside Slice 2A |
| admission control consumption from Slice 1B | `IN_SCOPE_SLICE_2A` | consume, not reinvent, the accepted runtime admission posture | no duplicate admission stack |
| turn classification foundation | `IN_SCOPE_SLICE_2A` | classify turn-start posture for canonical ingress processing | must remain deterministic and bounded to Section 03 ownership |
| canonical gate-order scaffold through the pre-authority boundary | `IN_SCOPE_SLICE_2A` | establish stage order and stop before real authority or persistence execution | later stages remain placeholders only |
| identity verification stage | `DEFER_SECTION04` | downstream authority boundary | Slice 2A may define the stage boundary only |
| onboarding eligibility stage | `DEFER_SECTION04` | downstream authority boundary | Slice 2A may not enforce onboarding authority itself |
| memory eligibility stage | `DEFER_LATER_RUNTIME` | depends on later memory/runtime sections even if the stage exists conceptually | Slice 2A may reserve the boundary only |
| access authorization stage | `DEFER_SECTION04` | downstream authority boundary | no authorization decisions in Slice 2A |
| simulation or tool eligibility stage | `DEFER_SECTION04` | downstream authority boundary | no simulation registry lookup or certification work in Slice 2A |
| authorized execution stage | `DEFER_SECTION04` | downstream protected execution boundary | Slice 2A must not run protected business execution |
| audit/proof hook surfaces in the execution carrier | `IN_SCOPE_SLICE_2A` | preserve downstream PH1.J / GOV / LAW / proof seams in the envelope and stage model | hook surfaces only, no real proof execution |
| real audit/proof capture execution | `DEFER_LATER_RUNTIME` | downstream of authority, proof, governance, and runtime law wiring | no PH1.J / GOV / LAW execution path in Slice 2A |
| canonical response envelope foundation for pre-authority outcomes | `IN_SCOPE_SLICE_2A` | return deterministic success/refusal carriers with session and turn anchors | must remain bounded to Section 03-owned outcome fields |
| client synchronization / reconciliation execution behavior | `DEFER_SECTION05` | depends on authoritative acknowledgement and persistence/sync law | Slice 2A returns anchors only |
| gate isolation, stage-boundary invariant validation, and deterministic fail-closed failure contract | `IN_SCOPE_SLICE_2A` | prevent illegal continuation and preserve ordered execution | every failed gate stops immediately |
| pipeline observability, turn-start events, metrics, and stage telemetry | `IN_SCOPE_SLICE_2A` | make ingress behavior auditable from the first Section 03 slice | consume Slice 1A/1B observability substrate; no analytics sprawl |

F) canonical ingress request-family matrix

| request family | canonical route | Slice 2A posture | normalized outcome | mandatory boundary |
|---|---|---|---|---|
| executable voice turn | `/v1/voice/turn` | `IN_SCOPE_SLICE_2A` | normalize into the canonical turn-request carrier and then the accepted `RuntimeExecutionEnvelope` | no alternate voice-execution path |
| executable text turn | `/v1/voice/turn` | `IN_SCOPE_SLICE_2A` | normalize into the same canonical turn-request carrier and envelope path as voice | no text-only bypass route |
| executable file turn | `/v1/voice/turn` | `IN_SCOPE_SLICE_2A` for family convergence only | normalize into the same canonical turn-request carrier; deep file semantics stay deferred | no file-specific alternate execution route |
| executable image turn | `/v1/voice/turn` | `IN_SCOPE_SLICE_2A` for family convergence only | normalize into the same canonical turn-request carrier; deep image semantics stay deferred | no image-specific alternate execution route |
| executable camera turn | `/v1/voice/turn` | `IN_SCOPE_SLICE_2A` for family convergence only | normalize into the same canonical turn-request carrier; deep camera semantics stay deferred | no camera-specific alternate execution route |
| invite click compatibility family | `/v1/invite/click` | `DEFER_LATER_SECTION03` execution, but family membership is preserved now | remain a canonical ingress-family member without executable Slice 2A behavior | must not become a hidden turn path |
| onboarding continue compatibility family | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` execution, but family membership is preserved now | remain a canonical ingress-family member without executable Slice 2A behavior | must not become a hidden turn path |
| accepted Slice 1B runtime shell routes | `/livez`, `/readyz`, `/startupz`, `/runtime/foundation/status` | baseline-only, consumed not redefined | remain runtime shell routes outside the canonical executable turn family | Section 03 must not collapse health/system routes into app ingress |

G) execution-envelope and stage-boundary matrix

| stage | Slice 2A posture | required behavior | boundary and downstream rule |
|---|---|---|---|
| Slice 1B route admission boundary | `CONSUME_ACCEPTED_BASELINE` | the request must first pass accepted Slice 1B route registration, request security, admission, and invariant validation | Slice 2A must not recreate a parallel router or admission stack |
| ingress validation and normalization | `IN_SCOPE_SLICE_2A` | validate ingress-owned fields, normalize the request into the canonical turn-request carrier, compute request-content hash, and reject malformed payloads fail closed | no session mutation or downstream stage entry before normalization succeeds |
| platform trigger validation | `IN_SCOPE_SLICE_2A` | validate `requested_trigger`, platform policy, and trigger compatibility using accepted `PlatformRuntimeContext` law | wake vs explicit may differ only at entry, never after this stage |
| session resolve or open | `IN_SCOPE_SLICE_2A` | consume accepted Slice 1C/1D session services to create, attach, resume, or open the canonical session and turn context | Slice 2A must not invent local session truth or alternate session ids |
| execution-envelope creation and binding | `IN_SCOPE_SLICE_2A` | create the canonical `RuntimeExecutionEnvelope` only after session/turn resolution so `session_id`, `turn_id`, `device_turn_sequence`, and `session_attach_outcome` are lawful | all later stages must consume the envelope rather than raw requests |
| turn classification and pre-authority admission finalization | `IN_SCOPE_SLICE_2A` | attach deterministic turn-start classification and set ingress-owned admission posture for downstream stages | this is not authorization and must not imply protected execution approval |
| stage-boundary invariant validation | `IN_SCOPE_SLICE_2A` | validate that envelope state, session posture, trigger posture, and stage order remain coherent before downstream handoff | failed invariants must reject immediately with deterministic failure class |
| response envelope assembly for pre-authority accept/reject outcomes | `IN_SCOPE_SLICE_2A` | return canonical turn-start response or refusal carriers with session/turn anchors, classification, and deterministic failure class | no persistence acknowledgement or downstream synchronization claims yet |
| identity verification | `DEFER_SECTION04` | reserved downstream stage only | Slice 2A may expose the boundary but may not execute identity decisions |
| onboarding eligibility validation | `DEFER_SECTION04` | reserved downstream stage only | Slice 2A may not enforce onboarding authority itself |
| memory eligibility evaluation | `DEFER_LATER_RUNTIME` | reserved downstream stage only | no memory-engine behavior in Slice 2A |
| access authorization | `DEFER_SECTION04` | reserved downstream stage only | no policy or authorization execution in Slice 2A |
| simulation or tool eligibility validation | `DEFER_SECTION04` | reserved downstream stage only | no simulation certification or tool authorization execution in Slice 2A |
| authorized execution | `DEFER_SECTION04` | reserved downstream stage only | Slice 2A ends before protected execution begins |
| audit and proof capture | `DEFER_LATER_RUNTIME` | reserved downstream stage only, with envelope hook surfaces preserved | no PH1.J / GOV / LAW execution in Slice 2A |
| client synchronization outcome | `DEFER_SECTION05` | reserved downstream stage only | no reconciliation, outbox, or authoritative ack behavior in Slice 2A |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2A role | mandatory posture |
|---|---|---|---|
| canonical ingress and turn contract carriers | `crates/selene_kernel_contracts/src/runtime_execution.rs` | additive-only contract support for the normalized turn carrier, stage-boundary data, response/failure carriers, or envelope-adjacent fields if strictly required | preserve existing envelope meaning and ordering; additive-only |
| Section 03 runtime implementation module | `crates/selene_os/src/` (new additive Section 03 ingress/turn module), `crates/selene_os/src/lib.rs` | host the canonical Slice 2A ingress normalization, session binding, envelope creation, stage-order scaffold, and response/failure foundation | additive-only; no rewrite of accepted Slice 1A-1D modules |
| route-family registration and runtime shell integration | `crates/selene_os/src/runtime_request_foundation.rs` | extend the accepted Slice 1B route shell so `/v1/voice/turn` becomes lawful at the correct layer and family compatibility for invite/onboarding can be preserved | reuse Slice 1B router and middleware foundations; do not replace them |
| session integration seam | `crates/selene_os/src/runtime_session_foundation.rs`, `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_kernel_contracts/src/common.rs` | consume accepted session ids, turn ids, attach outcomes, state, coordination, and conflict/backpressure behavior | no Section 02 reimplementation or semantic drift |
| runtime lifecycle and observability substrate | `crates/selene_os/src/runtime_bootstrap.rs`, `crates/selene_os/src/runtime_request_foundation.rs` | consumed baseline only for lifecycle, health, metrics, feature flags, event bus, and fail-closed admission posture | touch only if strictly necessary for bounded Section 03 integration |
| downstream authority seam | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs` | preserved downstream envelope and stage seams only | not a Slice 2A implementation target |
| downstream persistence seam | `crates/selene_storage/src/ph1f.rs` | preserved downstream session/turn/idempotency/persistence seam only | not a Slice 2A implementation target |
| app/client ingress surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2A | no app-specific ingress behavior, no Apple work, no client workaround architecture |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2A must be:

1. open the canonical Section 03 route family lawfully by extending the accepted Slice 1B route shell, while preserving the existing health/system routes and preserving invite/onboarding family compatibility without executing them
2. establish the canonical normalized turn-request carrier and ingress-owned validation rules for supported executable modalities
3. bind platform trigger validation to the accepted `PlatformRuntimeContext` and `RuntimeEntryTrigger` contract surfaces so wake and explicit entry converge before runtime execution begins
4. bind Section 03 session resolve-or-open behavior to the accepted Slice 1C/1D session substrate without reimplementing session or turn-order law
5. create the canonical `RuntimeExecutionEnvelope` only after session and turn allocation are lawful, and populate it using accepted Slice 1A/1B/1C/1D carriers
6. implement deterministic turn-start classification, ingress-owned admission progression, and stage-boundary invariant validation
7. implement the pre-authority stage-order scaffold and stop the execution path at the explicit Section 04 boundary
8. implement the canonical pre-authority response/failure envelope and the required turn-start events, metrics, and stage telemetry
9. implement only hook-level replay/idempotency integration required for ingress correctness, without introducing Section 05 durable dedupe, journal, or reconciliation behavior
10. close the slice only when route-family proof, envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no client/app bleed proof, and accepted Slice 1A-1D regression proof all pass on a clean tree

Slice 2A must not reverse this order. Route activation may not outrun normalization. Normalization may not outrun session binding. Envelope creation may not outrun lawful session and turn allocation. No protected execution may be entered before the slice is complete and Section 04 is explicitly opened later.

J) verification and acceptance matrix

| proof area | required verification | Slice 2A acceptance condition |
|---|---|---|
| canonical route and request-family proof | prove that `/v1/voice/turn` is the only executable turn route, that invite/onboarding family compatibility remains preserved, and that no hidden alternate turn route exists | Section 03 opens only one executable canonical turn path |
| modality convergence proof | prove that voice, text, file, image, and camera requests all normalize into the same canonical turn-request carrier and route family | modality differences affect input shape only, not pipeline identity |
| normalization proof | prove schema validation, content-type normalization, canonical field mapping, payload size enforcement, and request-content hashing are deterministic and fail closed | downstream stages never receive malformed or ambiguous ingress input |
| request envelope validation proof | prove ingress-owned envelope fields are required and deterministic (`Authorization`, request ids, nonce, timestamp, idempotency key, platform/device/actor/session hints, payload carrier) | malformed or incomplete ingress requests stop before execution begins |
| trigger convergence proof | prove wake and explicit triggers normalize into the same governed session-bound turn path after platform trigger validation | no platform-specific execution shortcut survives ingress |
| session binding proof | prove Section 03 consumes accepted Slice 1C/1D session and turn allocation surfaces without reinterpreting session law | `session_id`, `turn_id`, `device_turn_sequence`, and attach outcomes remain canonical |
| execution-envelope creation and binding proof | prove the first full `RuntimeExecutionEnvelope` instance is created only after lawful session/turn binding and carries the canonical ingress/session fields | no raw-request execution survives past the envelope boundary |
| stage-order proof | prove the pre-authority stage sequence is deterministic, recorded, and cannot be skipped or reordered | the Section 03 scaffold is machine-visible and ordered |
| fail-closed ingress rejection proof | prove invalid normalization, invalid trigger posture, invalid session binding, invalid invariants, and invalid ingress-owned envelope state all reject immediately with deterministic failure classes | no failed ingress request can drift into later stages |
| replay and idempotency hook proof | prove nonce/timestamp/request identity and idempotency key handling are deterministic at the ingress-owned boundary and do not invent durable persistence behavior | Slice 2A owns the hook surface only and nothing beyond it |
| response envelope proof | prove pre-authority success/refusal responses return canonical session and turn anchors, classification posture, and deterministic failure class | clients and later layers receive one canonical turn-start carrier |
| observability proof | prove turn-start events, stage telemetry, metrics, and trace propagation exist for every request family and every stage transition owned by Slice 2A | ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no identity verification, identity risk scoring, policy evaluation, authorization, simulation lookup, simulation certification, or protected execution behavior in Slice 2A | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable outbox, operation journal, dedupe store, reconciliation flow, persistence outcome reuse, or authoritative ack logic in Slice 2A | Section 05 remains downstream and intact |
| no PH1.J / GOV / LAW bleed proof | prove Slice 2A preserves hook surfaces only and does not execute proof, governance, or runtime-law behavior | later runtime law surfaces remain downstream |
| no client/app bleed proof | prove there is no Apple/client behavior, no app-specific ingress path, and no local workaround architecture introduced by Section 03 | clients remain downstream thin terminals |
| accepted Slice 1A-1D regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, health, PH1.OS, and PH1.L regressions against the Slice 2A code state | Slice 2A must not break the accepted runtime and session substrate |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2A is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/voice/turn` route and request-family foundation | `IN_SCOPE_SLICE_2A` | this is the only executable Section 03 route in Slice 2A |
| broader invite and onboarding family compatibility | `IN_SCOPE_SLICE_2A` | preserve canonical family membership without executing those routes in this slice |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2A` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` |
| no session-law drift | `IN_SCOPE_SLICE_2A` | Section 03 must consume accepted Slice 1C/1D session truth and must not reinterpret states, ids, ordering, ownership, or access law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2A` | Slice 2A must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| invite click execution behavior | `DEFER_LATER_SECTION03` | do not pull invite execution into the first turn-foundation slice |
| onboarding continue execution behavior | `DEFER_LATER_SECTION03` | do not pull onboarding execution into the first turn-foundation slice |
| deep modality-specific payload behavior beyond canonical normalization | `DEFER_LATER_SECTION03` | do not embed voice/file/image/camera-specific business logic into the first slice |
| identity verification, identity risk scoring, onboarding readiness enforcement, access authorization, and simulation eligibility | `DEFER_SECTION04` | do not build any real authority decision path inside Slice 2A |
| authorized execution | `DEFER_SECTION04` | Slice 2A must stop before protected state-mutating execution begins |
| durable replay protection, duplicate outcome reuse, outbox, journal, reconcile, dedupe, and authoritative sync acknowledgement | `DEFER_SECTION05` | do not implement persistence correctness inside Section 03 |
| PH1.J / proof execution, governance execution, and runtime-law execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve envelope seams only; do not build a hidden PH1.J / GOV / LAW path |
| memory eligibility execution, memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | these later layers may consume Section 03 outputs but may not be implemented here |
| Apple/client behavior, app-specific ingress behavior, and any client-local authority or retry workaround path | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no hidden Section 04 path | `IN_SCOPE_SLICE_2A` | Section 03 must not become a parallel authority layer |
| no hidden Section 05 path | `IN_SCOPE_SLICE_2A` | Section 03 must not become a parallel persistence layer |
| no incomplete completion claim | `IN_SCOPE_SLICE_2A` | Slice 2A may not be called complete while any in-scope Section 03 foundation or required proof remains uncaptured or unverified in this plan |

L) SLICE 2A COMPLETION STANDARD

Slice 2A is complete only when the first lawful Section 03 execution-entry layer exists and is proven without bleeding into later sections. Completion requires:

- the canonical request-family foundation exists and preserves `/v1/voice/turn`, `/v1/invite/click`, and `/v1/onboarding/continue` as the canonical ingress family
- `/v1/voice/turn` is the only executable turn route in this slice
- all executable modalities normalize into one canonical turn-request carrier and then into one canonical `RuntimeExecutionEnvelope`
- ingress-owned envelope validation, trigger validation, session binding, stage-order recording, invariant validation, and fail-closed failure classification all exist and are proven
- the pre-authority stage-order scaffold exists and ends explicitly before real Section 04 authority execution begins
- replay/idempotency hook foundations exist only to the bounded extent required by Section 03 ownership and do not implement Section 05 durability behavior
- the canonical pre-authority response/failure envelope exists and returns deterministic session and turn anchors
- turn-start observability, trace propagation, metrics, and events exist for every Slice 2A-owned stage
- no Section 04 behavior exists in the slice
- no Section 05 behavior exists in the slice
- no PH1.J / GOV / LAW execution behavior exists in the slice beyond preserved hook surfaces
- no client/app/Apple behavior exists in the slice
- accepted Slice 1A-1D regressions all remain green
- the slice closes on a clean tree with no uncaptured in-scope item, no uncaptured deferred boundary, and no unproven completion claim

Slice 2A is not complete if the route exists but normalization is partial, if the envelope exists but session binding is not canonical, if stage order is implied rather than recorded, if any later authority/persistence/client logic is pulled forward, or if any mandatory proof in `J)` is missing.

M) PHASE BOUNDARY

H3 governs Slice 2A only.

Slice 2A ends at the first lawful Section 03 pre-authority boundary:

- canonical route family is established
- `/v1/voice/turn` is live as the only executable turn route
- ingress normalization is deterministic
- platform trigger validation is deterministic
- session resolve-or-open and turn allocation consume accepted Slice 1C/1D truth
- the canonical `RuntimeExecutionEnvelope` is created and stage-boundary validated
- deterministic pre-authority response/failure carriers exist

The next lawful step after this plan is the bounded Slice 2A implementation run only.

This plan does not open:

- Section 04 authority execution
- Section 05 persistence or sync execution
- PH1.J / GOV / LAW execution beyond preserved hook surfaces
- Section 06 memory behavior, personality/emotional work, or PH1.COMP
- Section 09 governance execution
- Section 11 runtime-law execution
- Apple/client or app-specific ingress behavior

The next lawful step after H3 is a bounded Slice 2A implementation instruction against this plan. It is not a Section 04 start, not a Section 05 start, and not Apple work.
