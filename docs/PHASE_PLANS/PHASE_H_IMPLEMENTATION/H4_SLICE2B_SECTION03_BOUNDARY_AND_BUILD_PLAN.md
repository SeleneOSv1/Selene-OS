PHASE H4 — SLICE 2B SECTION 03 BOUNDARY AND BUILD PLAN

A) PURPOSE

This document resolves the Section 03 ambiguity that remained after accepted Slice 2A and freezes exactly one bounded next Section 03 slice.

H4 exists because the accepted read-only scope gate concluded that Slice 2B was not explicit enough to implement safely from H3 alone. H4 therefore does one thing only:

- identify the remaining lawful Section 03 surface after accepted Slice 2A
- compare the candidate next-slice options against current repo truth
- select exactly one bounded Slice 2B
- defer every non-selected candidate explicitly
- freeze the implementation order, verification law, and phase boundary for that chosen Slice 2B

H4 is planning law only. It does not authorize runtime code changes outside the future bounded Slice 2B implementation run, it does not reopen frozen H1/H2/H3 law, and it does not authorize Section 04, Section 05, Apple, or app/client implementation.

B) FROZEN LAW INPUTS

Slice 2B is derived from the following binding inputs:

- frozen A1-A6, B1-B5, C1-C5, D1-D5, E1-E5, F1-F5, and G1-G2 as upstream design-law constraints
- [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) as the sequencing-law anchor
- [H2](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H2_SLICE1_RUNTIME_AND_SESSION_BUILD_PLAN.md) as the accepted Slice 1 implementation-law anchor
- [H3](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H3_SLICE2A_CANONICAL_INGRESS_AND_TURN_BUILD_PLAN.md) as the accepted Slice 2A implementation-law anchor
- [Build Section 01](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md), [Build Section 02](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md), and [Build Section 03](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md) as the main governing implementation sections
- [Build Section 04](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md), [Build Section 05](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md), [Build Section 09](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md), and [Build Section 11](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md) as downstream dependency law that Slice 2B must preserve without pulling forward
- [CORE_ARCHITECTURE](docs/CORE_ARCHITECTURE.md), [SELENE_BUILD_EXECUTION_ORDER](docs/SELENE_BUILD_EXECUTION_ORDER.md), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY](docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md), [COVERAGE_MATRIX](docs/COVERAGE_MATRIX.md), [11_DESIGN_LOCK_SEQUENCE](docs/11_DESIGN_LOCK_SEQUENCE.md), and [02_BUILD_PLAN](docs/02_BUILD_PLAN.md) as architecture, sequencing, and readiness truth
- [PHASE_F_FREEZE_SUMMARY](docs/PHASE_PLANS/PHASE_F_IPHONE/PHASE_F_FREEZE_SUMMARY.md) and [PHASE_G_APPLE_FREEZE_SUMMARY](docs/PHASE_PLANS/PHASE_G_APPLE/PHASE_G_APPLE_FREEZE_SUMMARY.md) as frozen client/app/Apple boundary law that Slice 2B must not reopen

Accepted implementation baselines consumed by this plan:

- Slice 1A accepted at `f9769797c28bc991df3720d085639a7117b3d7c8`
- Slice 1B accepted at `eb4be3fdfc100fa22684293cacee471faf7d7847`
- Slice 1C accepted at `b1eba355e716887f0fe399cc6930988e0423e7db`
- Slice 1D accepted at `46b021dece36b9c1d8589362cf0ada0187603a83`
- Slice 2A accepted at `743ea0fe3e2ef884efb1c28bec706fe4efab91c9`

These accepted slices establish non-negotiable repo truth for H4:

- the runtime bootstrap, routing, request-security, admission, observability, session, ownership, transfer, access, and backpressure substrate already exists and must be consumed rather than reimplemented
- accepted Slice 2A already opened the canonical Section 03 family in the router, but only `/v1/voice/turn` is executable
- accepted Slice 2A already established one canonical turn-start carrier, one canonical `RuntimeExecutionEnvelope` binding path, and one bounded pre-authority stop line
- accepted Slice 2A already keeps `/v1/invite/click` and `/v1/onboarding/continue` registered as canonical family members while rejecting them as compatibility-only routes
- [app_ingress.rs](crates/selene_os/src/app_ingress.rs) already contains app-specific invite-open and onboarding-start execution wiring; that is explicit repo truth, but it is not the lawful Slice 2B implementation surface because H4 must not reopen app/client or Apple work

Sections 04 and 05 remain downstream dependencies only. They define the protected execution and persistence boundaries that Slice 2B must preserve, not execution work that Slice 2B may implement.

C) CURRENT / TARGET / GAP

CURRENT

- Accepted Slice 2A delivered the first lawful Section 03 execution-entry layer: `/v1/voice/turn` is live, executable modalities converge into one canonical turn-start carrier, the canonical `RuntimeExecutionEnvelope` path exists, the pre-authority stage-order scaffold is machine-visible, and success stops before Section 04 and Section 05 behavior begins.
- Accepted Slice 2A preserved the broader canonical ingress family by registering `/v1/invite/click` and `/v1/onboarding/continue` as canonical family members without executing them.
- H3 explicitly deferred three later Section 03 surfaces only:
  - invite click execution behavior
  - onboarding continue execution behavior
  - deep modality-specific payload behavior beyond canonical normalization
- Repo truth outside Section 03 already shows a separate app-specific invite-open and onboarding-start path in [app_ingress.rs](crates/selene_os/src/app_ingress.rs) using `LINK_INVITE_OPEN_ACTIVATE_COMMIT` and `ONB_SESSION_START_DRAFT`. That proves the downstream business execution semantics exist elsewhere in repo truth, and it also proves Slice 2B must not widen into app/client or alternative authority paths.

TARGET

- Freeze exactly one bounded next Section 03 slice after accepted Slice 2A.
- Resolve the H3 ambiguity so the next implementation run can proceed without guessing.
- Preserve the accepted Slice 2A invariants:
  - one canonical turn-start carrier for `/v1/voice/turn`
  - one canonical `RuntimeExecutionEnvelope` path
  - one bounded pre-authority stop line
  - no Section 04 or Section 05 execution
  - no Apple/client widening
  - no alternate authority path

GAP

- H3 correctly deferred the remaining Section 03 work, but it did not partition that deferred bucket into one uniquely bounded next slice.
- Without H4, an implementer would have to guess between invite-click execution, onboarding-continue execution, a combined compatibility-route slice, or deeper modality semantics.
- The missing artifact is therefore not new architecture. The missing artifact is a binding next-slice decision with explicit scope, explicit deferrals, and explicit proof law.

D) SLICE 2B SELECTION DECISION

Slice 2B is:

- the canonical `/v1/invite/click` compatibility execution foundation

This is the selected next bounded Section 03 slice.

Why this is the next lawful slice:

- It is one of the two explicit `DEFER_LATER_SECTION03` compatibility-route behaviors left by H3.
- It is upstream of onboarding continuation in current repo truth. The existing app-specific path in [app_ingress.rs](crates/selene_os/src/app_ingress.rs) performs `LINK_INVITE_OPEN_ACTIVATE_COMMIT` before `ONB_SESSION_START_DRAFT`, which shows that invite-open activation logically precedes onboarding session-start and therefore precedes onboarding continuation.
- It is smaller and more disciplined than opening both deferred compatibility routes together.
- It resolves the remaining route-family ambiguity without reopening deeper modality semantics, which belong to the already-live `/v1/voice/turn` path rather than the next canonical route-family expansion.
- It can still terminate at a bounded pre-authority handoff and therefore can remain wholly inside Section 03.

What Slice 2B means precisely:

- `/v1/invite/click` becomes the next executable canonical Section 03 compatibility route.
- The route must normalize into one bounded invite-click compatibility carrier.
- That carrier must reuse the accepted Slice 2A request-envelope discipline, session discipline, `RuntimeExecutionEnvelope` discipline, and pre-authority stop line.
- The slice must not execute `PH1.LINK`, `PH1.ONB`, Section 04 authority, or Section 05 persistence behavior.
- `/v1/onboarding/continue` remains deferred.

Why the non-selected candidates are deferred:

- onboarding-continue execution remains deferred because it depends on invite-activation and onboarding-session context that repo truth already shows as downstream of invite-open activation
- the combined compatibility-route slice remains deferred because it is broader than necessary and mixes two different downstream business seams (`PH1.LINK` and `PH1.ONB`)
- deep modality-specific payload behavior remains deferred because it does not resolve the route-family ambiguity that blocked Slice 2B, and it risks pulling engine-specific media behavior into the Section 03 boundary before the compatibility route surface is frozen
- app-open / deep-link client production remains out of scope because H3 never deferred it as a Section 03 runtime slice, and frozen Phase F / Phase G already reserve that territory as client/app/Apple law

E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/invite/click` compatibility execution foundation | `IN_SCOPE_SLICE_2B` | H3 explicitly deferred invite click execution; Section 03 keeps `/v1/invite/click` in the canonical ingress family; repo truth shows invite-open activation is upstream of onboarding session start | selected | smallest next route-family expansion that can stay pre-authority and avoid Apple/client widening |
| canonical `/v1/onboarding/continue` compatibility execution foundation | `DEFER_LATER_SECTION03` | H3 explicitly deferred onboarding-continue execution; repo truth shows onboarding session-start is downstream of invite-open activation and depends on activation context | deferred | not the minimal next slice because invite-open compatibility must be stabilized first |
| combined `/v1/invite/click` + `/v1/onboarding/continue` compatibility execution slice | `DEFER_LATER_SECTION03` | both routes are canonical family members, but current repo truth binds them to different downstream seams (`PH1.LINK` activation first, then `PH1.ONB`) | deferred | too broad for the next bounded slice and reintroduces the ambiguity H4 is required to remove |
| deep modality-specific payload behavior beyond Slice 2A normalization | `DEFER_LATER_SECTION03` | H3 explicitly deferred it; the accepted `/v1/voice/turn` path is already live and bounded; deeper modality work does not decide the remaining compatibility-route boundary | deferred | not the next route-family step and more likely to pull perception/media semantics forward |
| app-open / deep-link / client-produced compatibility execution slice | `OUT_OF_SCOPE_THIS_PHASE` | H3 did not defer app-open as a later Section 03 slice; frozen Phase F / Phase G and [app_ingress.rs](crates/selene_os/src/app_ingress.rs) place app-open production in client/app law | rejected as a Slice 2B candidate | reopening app/client work here would violate frozen Apple and app boundaries |

F) selected slice scope and dependency matrix

| item | classification | Slice 2B position | dependency / guardrail |
|---|---|---|---|
| canonical `/v1/invite/click` executable compatibility route | `IN_SCOPE_SLICE_2B` | next executable canonical Section 03 route after accepted Slice 2A | extend the accepted Slice 1B/2A route shell; no new route family |
| invite-click compatibility request-shape foundation using the frozen `InviteOpenActivateCommitRequest` field truth (`token_id`, `token_signature`, `device_fingerprint`, `app_platform`, `app_instance_id`, `deep_link_nonce`, `link_opened_at`, `idempotency_key`) plus canonical request-envelope fields | `IN_SCOPE_SLICE_2B` | define the bounded Section 03 request shape for `/v1/invite/click` | consume existing `PH1.LINK` contract truth only; do not execute `PH1.LINK` |
| canonical invite-click compatibility carrier | `IN_SCOPE_SLICE_2B` | normalize `/v1/invite/click` into one deterministic ingress-owned compatibility carrier | additive only; must not replace or fork the accepted `/v1/voice/turn` turn-start carrier |
| accepted `/v1/voice/turn` turn-start carrier and execution path | `IN_SCOPE_SLICE_2B` | preserved baseline, not redefined | Slice 2B must not change the accepted Slice 2A turn path |
| accepted Slice 1B request-envelope validation, request security, admission, and invariant foundations | `IN_SCOPE_SLICE_2B` | consumed exactly as built | no duplicate admission stack and no route-shell replacement |
| trigger validation for invite-click compatibility requests | `IN_SCOPE_SLICE_2B` | validate the bounded trigger posture at ingress before any deeper stage begins | must consume accepted `PlatformRuntimeContext` truth without inventing app/client trigger rules |
| bounded session resolve-or-open or lawful session anchoring for compatibility execution | `IN_SCOPE_SLICE_2B` | keep execution inside the accepted session model and envelope discipline | consume accepted Slice 1C/1D session truth only; no local session law |
| canonical `RuntimeExecutionEnvelope` reuse for compatibility requests | `IN_SCOPE_SLICE_2B` | the invite-click route must enter the same canonical envelope path used by Section 03 | no raw compatibility request survives past the envelope boundary |
| compatibility-route classification and pre-authority handoff foundation | `IN_SCOPE_SLICE_2B` | classify the invite-click route at the bounded Section 03 handoff without authorizing protected execution | must stop before real `PH1.LINK`, `PH1.ONB`, Section 04, or Section 05 behavior |
| canonical response/failure carrier for invite-click pre-authority outcomes | `IN_SCOPE_SLICE_2B` | return deterministic pre-authority acceptance or refusal anchors | no authoritative activation, onboarding creation, or sync acknowledgement claims |
| `/v1/onboarding/continue` execution behavior | `DEFER_LATER_SECTION03` | later compatibility-route slice only | keep family membership preserved but non-executable in Slice 2B |
| deep modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | later refinement of the already-live `/v1/voice/turn` path only | do not mix route-family expansion with deep media semantics |
| actual `PH1.LINK` invite-open activation execution | `DEFER_SECTION04` | downstream protected execution only | no simulation dispatch, no link-state mutation, no authoritative activation result in Slice 2B |
| actual `PH1.ONB` session-start or onboarding continuation execution | `DEFER_SECTION04` | downstream protected execution only | no onboarding state mutation, no onboarding progression execution in Slice 2B |
| durable replay, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, and authoritative ack behavior for compatibility routes | `DEFER_SECTION05` | downstream persistence and sync only | no local or in-memory pseudo-persistence substitute |
| PH1.J / GOV / LAW execution beyond preserved envelope hooks | `DEFER_LATER_RUNTIME` | preserved seam only | no proof, governance, or runtime-law execution path in Slice 2B |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | downstream only | later layers may consume Slice 2B outputs but may not be implemented here |
| Apple/client app-open production, deep-link production, and any app-specific compatibility workaround path | `OUT_OF_SCOPE_THIS_PHASE` | explicit non-target | no Apple work, no app work, no client-owned execution truth |

G) route-family / request-shape matrix

| request family or shape | canonical route or boundary | Slice 2B posture | normalized outcome | mandatory boundary |
|---|---|---|---|---|
| accepted canonical turn family | `/v1/voice/turn` | `IN_SCOPE_SLICE_2B` as protected baseline | continue using the accepted canonical turn-start carrier and accepted envelope path unchanged | Slice 2B must not alter the accepted Slice 2A turn path |
| selected invite-click compatibility family | `/v1/invite/click` | `IN_SCOPE_SLICE_2B` | normalize into one bounded invite-click compatibility carrier using the frozen `InviteOpenActivateCommitRequest` field truth and then reuse the canonical `RuntimeExecutionEnvelope` path | no direct `PH1.LINK` execution and no hidden alternate authority path |
| onboarding-continue compatibility family | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain registered as a canonical family member without executable Slice 2B behavior | must not become a hidden continuation or onboarding authority path in Slice 2B |
| deep modality-specific turn semantics | `/v1/voice/turn` internal modality handling only | `DEFER_LATER_SECTION03` | keep the accepted Slice 2A normalization outcome unchanged for now | no voice/file/image/camera-specific business logic in Slice 2B |
| app-open / deep-link client-produced invite-open shape | client/app surface only, not a canonical Section 03 route family member | `OUT_OF_SCOPE_THIS_PHASE` | no Section 03 runtime normalization target in Slice 2B | Phase F / Phase G and `app_ingress.rs` retain this boundary; Slice 2B must not reopen it |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2B role | mandatory posture |
|---|---|---|---|
| route-family activation and router integration | `crates/selene_os/src/runtime_request_foundation.rs` | extend the accepted Slice 1B/2A route shell so `/v1/invite/click` becomes the one new executable Section 03 route while `/v1/onboarding/continue` remains compatibility-only | reuse the accepted router and middleware stack; do not replace it |
| Section 03 compatibility execution module | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` or a new additive Section 03 compatibility module under `crates/selene_os/src/`, plus `crates/selene_os/src/lib.rs` only if strictly required | host the invite-click normalization, envelope reuse, stage-order reuse, and pre-authority response/failure foundation | additive only; no rewrite of the accepted Slice 2A turn path |
| compatibility request-shape and envelope-adjacent contract support | `crates/selene_kernel_contracts/src/ph1link.rs`, `crates/selene_kernel_contracts/src/runtime_execution.rs` | consume existing contract truth and add only strictly required additive support for the bounded compatibility carrier if current carriers are insufficient | preserve existing contract meaning and ordering; additive only |
| session integration seam | `crates/selene_os/src/runtime_session_foundation.rs`, `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_kernel_contracts/src/common.rs` | consume accepted session ids, turn ids, attach outcomes, state, ownership, and conflict/backpressure truth | no Section 02 reimplementation or semantic drift |
| downstream link and onboarding business seams | `crates/selene_os/src/app_ingress.rs`, `crates/selene_os/src/ph1link.rs`, `crates/selene_os/src/ph1onb.rs`, `crates/selene_storage/src/ph1f.rs` | explicit non-targets preserved as downstream execution consumers only | not a Slice 2B implementation target |
| downstream authority, persistence, governance, and law seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs` | preserved downstream envelope and stage seams only | not a Slice 2B implementation target |
| Apple/client and app-specific ingress surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2B | no Apple work, no app work, no client workaround architecture |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2B must be:

1. extend the accepted Slice 1B/2A route shell so `/v1/invite/click` becomes the only new executable canonical Section 03 route while `/v1/onboarding/continue` remains registered but non-executable
2. define the bounded invite-click request shape using the already-frozen `PH1.LINK` open-activate field truth and accepted request-envelope fields, without inventing app/client production behavior
3. normalize `/v1/invite/click` into one bounded invite-click compatibility carrier without changing the accepted `/v1/voice/turn` turn-start carrier
4. reuse accepted Slice 1B request security, replay-hook posture, idempotency propagation, and trigger validation foundations so invite-click entry remains fail closed
5. bind the invite-click compatibility path into the accepted Slice 1C/1D session and envelope discipline only to the bounded extent required to keep the execution path session-bound and canonical
6. reuse the canonical `RuntimeExecutionEnvelope` creation path and preserve every downstream Section 04/05/09/11 field as unset
7. reuse the accepted pre-authority stage-order scaffold by adding only the bounded compatibility-route classification needed for `/v1/invite/click`
8. implement deterministic pre-authority response/failure carriers, events, metrics, and trace propagation for the selected compatibility route
9. stop the success path at the bounded pre-authority handoff; do not execute `PH1.LINK`, `PH1.ONB`, Section 04, or Section 05 behavior
10. close the slice only when route-scope proof, request-shape proof, envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no client/app bleed proof, and accepted Slice 1A-1D plus Slice 2A regression proof all pass on a clean tree

Slice 2B must not reverse this order. Route activation may not outrun bounded request-shape definition. Compatibility normalization may not outrun accepted request security and trigger validation. Envelope creation may not outrun lawful session discipline. No protected execution may be entered before the slice is complete and a later Section 04 slice is explicitly opened.

J) verification and acceptance matrix

| proof area | required verification | Slice 2B acceptance condition |
|---|---|---|
| route-scope proof | prove that `/v1/invite/click` is the only newly executable route in Slice 2B, that `/v1/voice/turn` remains the accepted canonical turn path, and that `/v1/onboarding/continue` remains non-executable | Slice 2B opens exactly one new canonical compatibility route and nothing wider |
| invite-click request-shape proof | prove bounded invite-click requests require the frozen `InviteOpenActivateCommitRequest` field truth plus canonical request-envelope fields and reject malformed or incomplete inputs fail closed | compatibility requests enter the runtime with one deterministic shape only |
| compatibility normalization proof | prove `/v1/invite/click` requests normalize into one deterministic invite-click compatibility carrier and do not alter the accepted turn-start carrier | the selected route gains one bounded normalized path and nothing parallel |
| request-envelope and trigger proof | prove accepted Slice 1B request-envelope rules and accepted trigger law remain enforced for invite-click execution | no route-specific security or trigger shortcut exists |
| session discipline proof | prove Slice 2B consumes accepted Slice 1C/1D session truth to keep the compatibility route lawful and session-bound without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| execution-envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and that raw compatibility requests do not survive past the envelope boundary | there is still one canonical runtime envelope path |
| stage-order proof | prove the invite-click compatibility path records deterministic stage order and stops at the pre-authority boundary | the selected route remains inside Section 03 and cannot skip or reorder gates |
| fail-closed proof | prove invalid request shape, invalid trigger posture, invalid session posture, invalid stage progression, and invalid envelope posture all reject immediately with deterministic failure class | no failed compatibility request can drift into later stages |
| pre-authority stop proof | prove the successful `/v1/invite/click` path stops at a bounded pre-authority handoff and does not execute `PH1.LINK`, `PH1.ONB`, Section 04, or Section 05 behavior | Slice 2B stays upstream of protected execution and persistence |
| observability proof | prove the selected route emits deterministic events, counters, metrics, and trace propagation for every Slice 2B-owned stage | compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no identity verification, onboarding authority decision, access authorization, simulation execution, or protected business execution in Slice 2B | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable replay protection, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2B | Section 05 remains downstream and intact |
| no PH1.J / GOV / LAW bleed proof | prove Slice 2B preserves hook surfaces only and does not execute proof, governance, or runtime-law behavior | later runtime law surfaces remain downstream |
| no client/app bleed proof | prove there is no Apple/client behavior, no app-open production behavior, and no app-specific workaround architecture introduced by Slice 2B | clients remain downstream thin terminals |
| accepted Slice 1A-1D and Slice 2A regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2B code state | Slice 2B must not break the accepted runtime, session, or Slice 2A foundation |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2B is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/invite/click` compatibility execution foundation | `IN_SCOPE_SLICE_2B` | this is the only new executable Section 03 route in Slice 2B |
| preserve accepted `/v1/voice/turn` route, accepted turn-start carrier, and accepted pre-authority handoff behavior | `IN_SCOPE_SLICE_2B` | Slice 2B may extend Section 03, but it may not reinterpret or replace accepted Slice 2A truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2B` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2B` | Slice 2B must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2B` | Slice 2B must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| `/v1/onboarding/continue` execution behavior | `DEFER_LATER_SECTION03` | do not pull onboarding-continue execution into Slice 2B |
| combined invite and onboarding compatibility-route execution | `DEFER_LATER_SECTION03` | do not widen Slice 2B into a two-route compatibility bucket |
| deep modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | do not embed voice/file/image/camera-specific business logic into Slice 2B |
| actual `PH1.LINK` invite-open activation execution | `DEFER_SECTION04` | no simulation dispatch, no authoritative link-state mutation, and no activated result claims in Slice 2B |
| actual `PH1.ONB` session-start or onboarding-continue execution | `DEFER_SECTION04` | no onboarding-state mutation or onboarding progression execution in Slice 2B |
| durable replay protection, duplicate outcome reuse, outbox, journal, reconcile, dedupe, and authoritative sync acknowledgement | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2B |
| PH1.J / proof execution, governance execution, and runtime-law execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve envelope seams only; do not build a hidden PH1.J / GOV / LAW path |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | these later layers may consume Slice 2B outputs but may not be implemented here |
| Apple/client behavior, app-open/deep-link production, app-specific ingress behavior, and any client-local authority or retry workaround path | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no hidden Section 04 path | `IN_SCOPE_SLICE_2B` | Section 03 must not become a parallel authority layer |
| no hidden Section 05 path | `IN_SCOPE_SLICE_2B` | Section 03 must not become a parallel persistence layer |
| no incomplete completion claim | `IN_SCOPE_SLICE_2B` | Slice 2B may not be called complete while any in-scope item or mandatory proof remains uncaptured |

L) SLICE 2B COMPLETION STANDARD

Slice 2B is complete only when the next lawful compatibility-route expansion of Section 03 exists and is proven without bleeding into later sections. Completion requires:

- `/v1/invite/click` is live as the only new executable compatibility route in the canonical Section 03 family
- `/v1/voice/turn` remains the accepted canonical turn route unchanged
- `/v1/onboarding/continue` remains non-executable and explicitly deferred
- invite-click requests validate against one bounded deterministic request shape anchored to the frozen `PH1.LINK` open-activate contract fields plus canonical request-envelope truth
- invite-click requests normalize into one bounded compatibility carrier without altering the accepted turn-start carrier
- accepted Slice 1B request security, accepted Slice 1C/1D session truth, accepted Slice 2A envelope path, and accepted Slice 2A pre-authority scaffold are all reused rather than replaced
- the selected route stops at a deterministic pre-authority handoff and returns deterministic success/refusal carriers
- no `PH1.LINK`, `PH1.ONB`, Section 04, or Section 05 behavior exists in the slice
- no PH1.J / GOV / LAW execution behavior exists in the slice beyond preserved hook surfaces
- no client/app/Apple behavior exists in the slice
- accepted Slice 1A-1D and Slice 2A regressions all remain green
- the slice closes on a clean tree with no uncaptured in-scope item, no uncaptured deferred boundary, and no unproven completion claim

Slice 2B is not complete if `/v1/invite/click` becomes executable through app-specific wiring, if the accepted turn-start carrier or accepted envelope path is duplicated or bypassed, if onboarding-continue or deep modality work is pulled forward, if `PH1.LINK` or `PH1.ONB` execution is embedded directly into the slice, or if any mandatory proof in `J)` is missing.

M) PHASE BOUNDARY

H4 governs Slice 2B only.

Slice 2B ends at the second lawful Section 03 pre-authority boundary:

- `/v1/invite/click` is live as the only new executable compatibility route
- the bounded invite-click request shape is deterministic
- invite-click requests normalize into one bounded compatibility carrier
- the canonical `RuntimeExecutionEnvelope` path is reused and stage-boundary validated
- deterministic pre-authority compatibility response/failure carriers exist
- no `PH1.LINK` execution, no `PH1.ONB` execution, no Section 04 behavior, and no Section 05 behavior exist in the slice

The next lawful step after this plan is the bounded Slice 2B implementation run only.

This plan does not open:

- `/v1/onboarding/continue` execution
- deep modality-specific payload behavior beyond accepted Slice 2A normalization
- Section 04 authority execution
- Section 05 persistence or sync execution
- PH1.J / GOV / LAW execution beyond preserved hook surfaces
- Section 06 memory behavior, personality/emotional work, or PH1.COMP
- Apple/client or app-specific ingress behavior

The next lawful step after H4 is a bounded Slice 2B implementation instruction against this plan. It is not a Slice 2C start, not a Section 04 start, not a Section 05 start, and not Apple work.
