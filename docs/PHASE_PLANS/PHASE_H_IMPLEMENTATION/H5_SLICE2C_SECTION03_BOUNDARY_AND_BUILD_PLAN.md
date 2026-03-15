PHASE H5 — SLICE 2C SECTION 03 BOUNDARY AND BUILD PLAN

A) PURPOSE

H5 exists because accepted Slice 2B closed the `/v1/invite/click` ambiguity but did not make the next onboarding-compatible Section 03 slice uniquely explicit.

H5 therefore does one thing only:

- identify the remaining lawful Section 03 surfaces after accepted Slice 2B
- narrow the `/v1/onboarding/continue` umbrella into one bounded next slice
- freeze exactly one Slice 2C implementation target
- defer every non-selected onboarding branch explicitly
- preserve the accepted Slice 1A/1B/1C/1D substrate, accepted Slice 2A ingress foundation, and accepted Slice 2B compatibility foundation without reopening runtime or Apple law

H5 is planning law only. It does not implement code, it does not widen into Section 04 or Section 05, and it does not authorize any Apple or app-surface implementation.

B) FROZEN LAW INPUTS

H5 is derived from and bound by:

- frozen H1 sequencing law in `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md`
- accepted Slice 1 implementation law in `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H2_SLICE1_RUNTIME_AND_SESSION_BUILD_PLAN.md`
- frozen Slice 2A planning law in `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H3_SLICE2A_CANONICAL_INGRESS_AND_TURN_BUILD_PLAN.md`
- frozen Slice 2B planning law in `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H4_SLICE2B_SECTION03_BOUNDARY_AND_BUILD_PLAN.md`
- frozen Phase F and Phase G non-authority Apple/client law
- Build Sections `01-11`, especially Build Sections `03`, `04`, and `05`

Accepted upstream runtime baselines consumed by H5:

- Slice 1A baseline: `f9769797c28bc991df3720d085639a7117b3d7c8`
- Slice 1B baseline: `eb4be3fdfc100fa22684293cacee471faf7d7847`
- Slice 1C baseline: `b1eba355e716887f0fe399cc6930988e0423e7db`
- Slice 1D baseline: `46b021dece36b9c1d8589362cf0ada0187603a83`
- accepted Slice 2A implementation: `743ea0fe3e2ef884efb1c28bec706fe4efab91c9`
- accepted Slice 2B implementation: `cd340ea14cfb2c8b7c15fc0cb578daf1e8e168fe`

Repo-truth anchors consumed by H5:

- accepted Slice 2B runtime truth keeps `/v1/voice/turn` and `/v1/invite/click` executable while `/v1/onboarding/continue` remains registered but non-executable in `crates/selene_os/src/runtime_ingress_turn_foundation.rs`
- current onboarding continuation truth is carried by `AppOnboardingContinueRequest` and `AppOnboardingContinueAction` in `crates/selene_os/src/app_ingress.rs`
- current onboarding contract truth for downstream simulation ids and request/result carriers is in `crates/selene_kernel_contracts/src/ph1onb.rs`
- current execution-envelope contract truth remains in `crates/selene_kernel_contracts/src/runtime_execution.rs`

C) CURRENT / TARGET / GAP

CURRENT

- accepted Slice 2A established the canonical Section 03 route family, one canonical normalized turn-start carrier, one canonical `RuntimeExecutionEnvelope` path, and one pre-authority stop line
- accepted Slice 2B made `/v1/invite/click` the one new executable compatibility route while preserving `/v1/onboarding/continue` as registered but non-executable
- current repo truth shows `/v1/onboarding/continue` is not one action. `AppOnboardingContinueAction` currently spans `AskMissingSubmit`, `PlatformSetupReceipt`, `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, wake-enroll actions, sender-verification actions, and later completion actions

TARGET

- Slice 2C must open exactly one bounded executable onboarding-continue compatibility path under `/v1/onboarding/continue`
- that path must reuse the accepted Slice 2A/2B Section 03 foundations: one canonical normalized carrier, one canonical `RuntimeExecutionEnvelope` path, one accepted session-binding seam, and one pre-authority stop line
- the selected path must remain smaller than the full onboarding bucket and must stay upstream of real `PH1.ONB`, governance proof, Section 04 authority, and Section 05 persistence behavior

GAP

- H4 proved that raw `/v1/onboarding/continue` execution was too broad, but it did not freeze which onboarding continuation action becomes next
- current repo truth shows materially different downstream seams inside the same route:
  - `AskMissingSubmit` and `PlatformSetupReceipt` depend on `LINK_INVITE_DRAFT_UPDATE_COMMIT`
  - `TermsAccept` depends on `ONB_TERMS_ACCEPT_COMMIT`
  - `PrimaryDeviceConfirm` adds governance proof and `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
  - `VoiceEnrollLock` and later actions sit further downstream still
- Slice 2C therefore requires a new boundary document that freezes one exact onboarding sub-slice and defers the rest

D) SLICE 2C SELECTION DECISION

Slice 2C is:

- the canonical `/v1/onboarding/continue` `AskMissingSubmit` compatibility execution foundation

This is the selected next bounded Section 03 slice.

Why this is the next lawful slice:

- it is the smallest remaining onboarding continuation branch in current repo truth
- it is upstream of `platform_setup`, `terms_accept`, `primary_device_confirm`, and `voice_enroll`
- it depends on the same canonical route family already preserved by Slice 2A and Slice 2B without requiring a new route family or a new envelope path
- it can remain wholly inside the accepted pre-authority Section 03 stop line because the slice can validate, normalize, session-bind, envelope-bind, classify, and stop before actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` or `PH1.ONB` execution
- it resolves the next real ambiguity without reopening later onboarding branches or deep modality work

What Slice 2C means precisely:

- `/v1/onboarding/continue` becomes executable only for `AskMissingSubmit`
- the selected action must use the bounded `AppOnboardingContinueRequest` field truth:
  - `correlation_id`
  - `onboarding_session_id`
  - `idempotency_key`
  - optional `tenant_id`
  - `action = AskMissingSubmit { field_value }`
  - canonical request-envelope fields from the accepted Section 03 ingress foundation
- the selected action must normalize into the accepted compatibility branch of the same canonical turn-start carrier rather than creating a second onboarding carrier
- the selected action must reuse the accepted Slice 1C/1D session discipline and the accepted `RuntimeExecutionEnvelope` path
- the selected action must stop at the same bounded pre-authority handoff result used by accepted Slice 2A and Slice 2B

Why the non-selected candidates are deferred:

- `PlatformSetupReceipt` remains deferred because it is downstream of missing-field resolution and carries broader receipt-specific semantics than the smaller `AskMissingSubmit` slice
- `TermsAccept` remains deferred because it depends on missing-field and platform-setup completion and already maps directly to `ONB_TERMS_ACCEPT_COMMIT`
- `PrimaryDeviceConfirm` remains deferred because it adds governance proof posture and maps directly to `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
- `VoiceEnrollLock` and later onboarding actions remain deferred because they are further downstream and pull in additional voice/artifact or completion semantics
- the combined onboarding-continue execution bucket remains deferred because it would reintroduce the same route-level ambiguity H5 is required to remove
- deep modality-specific payload behavior remains deferred because it belongs to the already-live `/v1/voice/turn` path, not the next bounded onboarding compatibility slice

E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/onboarding/continue` `AskMissingSubmit` compatibility execution foundation | `IN_SCOPE_SLICE_2C` | `AppOnboardingContinueAction::AskMissingSubmit` is the first bounded continuation branch after session start and before later onboarding actions | selected | smallest lawful onboarding sub-slice that can stay pre-authority and reuse accepted Slice 2A/2B Section 03 foundations |
| canonical `/v1/onboarding/continue` `PlatformSetupReceipt` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires missing-field resolution before platform setup can be committed | deferred | not the minimal next slice and broader than the selected branch |
| canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires ask-missing and platform setup completion before `terms_accept`, and the action maps directly to `ONB_TERMS_ACCEPT_COMMIT` | deferred | too far downstream for the next bounded slice |
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires terms acceptance and adds governance proof before `primary_device_confirm` can proceed | deferred | not the next slice because it is downstream and touches a stronger authority seam |
| canonical `/v1/onboarding/continue` `VoiceEnrollLock` / `voice_enroll` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth places it after ask-missing, platform setup, terms, and primary-device confirmation | deferred | further downstream and more semantically dense than the selected slice |
| combined `/v1/onboarding/continue` execution bucket | `DEFER_LATER_SECTION03` | the route currently contains multiple materially different action families | deferred | too broad; it recreates the exact ambiguity H5 must remove |

F) selected slice scope and dependency matrix

| item | classification | Slice 2C position | dependency / guardrail |
|---|---|---|---|
| canonical `/v1/onboarding/continue` executable compatibility route for `AskMissingSubmit` only | `IN_SCOPE_SLICE_2C` | next executable canonical Section 03 compatibility route after accepted Slice 2B | extend the accepted Slice 1B/2A/2B route shell; do not create a new route family |
| selected onboarding ask-missing request-shape foundation using `AppOnboardingContinueRequest` and `AppOnboardingContinueAction::AskMissingSubmit` field truth plus canonical request-envelope fields | `IN_SCOPE_SLICE_2C` | define the bounded Section 03 request shape for the selected onboarding action | consume current app/onboarding contract truth only; do not execute app-layer logic |
| canonical onboarding compatibility carrier reuse inside the accepted turn-start carrier | `IN_SCOPE_SLICE_2C` | normalize the selected action into the same canonical compatibility branch already established by accepted Slice 2A/2B | no second onboarding carrier and no carrier fork |
| accepted `/v1/voice/turn` route and accepted `/v1/invite/click` route | `IN_SCOPE_SLICE_2C` | preserved baselines, not redefined | Slice 2C must not alter accepted Slice 2A or Slice 2B behavior |
| accepted Slice 1B request-envelope validation, request security, admission, replay-hook posture, and invariant foundations | `IN_SCOPE_SLICE_2C` | consumed exactly as built | no duplicate admission stack and no route-shell replacement |
| trigger validation for selected onboarding compatibility requests | `IN_SCOPE_SLICE_2C` | validate bounded trigger posture before deeper stage entry | must consume accepted `PlatformRuntimeContext` truth only |
| bounded session resolve-or-open or lawful session anchoring for selected onboarding compatibility execution | `IN_SCOPE_SLICE_2C` | keep the selected route session-bound and lawful | consume accepted Slice 1C/1D session truth only; no onboarding-owned session law |
| canonical `RuntimeExecutionEnvelope` reuse for selected onboarding requests | `IN_SCOPE_SLICE_2C` | the selected route must enter the same canonical envelope path used by accepted Section 03 | no raw onboarding compatibility request survives past the envelope boundary |
| compatibility-route classification and pre-authority handoff foundation for the selected action | `IN_SCOPE_SLICE_2C` | classify the selected onboarding route at the bounded Section 03 handoff without authorizing protected execution | must stop before real `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior |
| canonical response/failure carrier for selected onboarding pre-authority outcomes | `IN_SCOPE_SLICE_2C` | return deterministic pre-authority acceptance or refusal anchors | no authoritative onboarding mutation, verification, or sync acknowledgement claims |
| `PlatformSetupReceipt` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep request-shape truth visible but non-executable in Slice 2C |
| `TermsAccept` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep `terms_accept` deferred in Slice 2C |
| `PrimaryDeviceConfirm` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep `primary_device_confirm` deferred in Slice 2C |
| `VoiceEnrollLock`, wake-enroll, sender-verification, emo-persona, access-provision, and completion onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slices only | do not widen Slice 2C into downstream onboarding completion work |
| actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` execution for ask-missing mutation | `DEFER_SECTION04` | downstream protected execution only | no storage mutation, no authoritative field-update result in Slice 2C |
| actual `PH1.ONB` execution (`terms_accept`, `primary_device_confirm`, `voice_enroll`, and later onboarding mutation behavior) | `DEFER_SECTION04` | downstream protected execution only | no onboarding state mutation or governance-protected action execution in Slice 2C |
| durable replay, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, and authoritative ack behavior for onboarding compatibility routes | `DEFER_SECTION05` | downstream persistence and sync only | no local or in-memory pseudo-persistence substitute |
| PH1.J / GOV / LAW execution beyond preserved envelope hooks | `DEFER_LATER_RUNTIME` | preserved seam only | no proof, governance, or runtime-law execution path in Slice 2C |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | downstream only | later layers may consume Slice 2C outputs but may not be implemented here |
| Apple/client app-open production, app-specific onboarding workarounds, and any client-local authority path | `OUT_OF_SCOPE_THIS_PHASE` | explicit non-target | no Apple work, no app work, no client-owned execution truth |

G) request-shape / continuation-action matrix

| request family or continuation action | canonical route or boundary | Slice 2C posture | normalized outcome | mandatory boundary |
|---|---|---|---|---|
| accepted canonical turn family | `/v1/voice/turn` | `IN_SCOPE_SLICE_2C` as protected baseline | continue using the accepted canonical turn-start carrier and accepted envelope path unchanged | Slice 2C must not alter accepted Slice 2A behavior |
| accepted invite-click compatibility family | `/v1/invite/click` | `IN_SCOPE_SLICE_2C` as protected baseline | continue using the accepted Slice 2B invite-click compatibility carrier unchanged | Slice 2C must not alter accepted Slice 2B behavior |
| selected onboarding ask-missing action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2C` | normalize `AppOnboardingContinueRequest { onboarding_session_id, idempotency_key, tenant_id, action = AskMissingSubmit { field_value } }` plus canonical request-envelope fields into the accepted compatibility branch of the canonical carrier and then reuse the canonical `RuntimeExecutionEnvelope` path | no direct `LINK_INVITE_DRAFT_UPDATE_COMMIT` execution and no hidden alternate authority path |
| `platform_setup` receipt action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2C | must not become a hidden receipt-processing authority path |
| `terms_accept` action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2C | must not become a hidden `ONB_TERMS_ACCEPT_COMMIT` path |
| `primary_device_confirm` action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2C | must not become a hidden governance or protected-action path |
| `voice_enroll` and later onboarding actions | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain canonical continuation-action shapes but non-executable in Slice 2C | do not pull wake-enroll, sender-verification, completion, or later onboarding work forward |
| app-open / client-produced onboarding behavior | client/app surface only | `OUT_OF_SCOPE_THIS_PHASE` | no Section 03 runtime normalization target in Slice 2C | Phase F / Phase G and `app_ingress.rs` retain this boundary |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2C role | mandatory posture |
|---|---|---|---|
| route-family activation and router integration | `crates/selene_os/src/runtime_request_foundation.rs` | extend the accepted Slice 1B/2A/2B route shell so `/v1/onboarding/continue` becomes executable only for the selected ask-missing action while `/v1/voice/turn` and `/v1/invite/click` remain unchanged | reuse the accepted router and middleware stack; do not replace it |
| Section 03 compatibility execution module | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` or a new additive Section 03 compatibility module under `crates/selene_os/src/`, plus `crates/selene_os/src/lib.rs` only if strictly required | host the selected onboarding ask-missing normalization, envelope reuse, stage-order reuse, and pre-authority response/failure foundation | additive only; no rewrite of accepted Slice 2A or Slice 2B paths |
| onboarding compatibility request-shape and envelope-adjacent contract support | `crates/selene_kernel_contracts/src/runtime_execution.rs` and only strictly-required additive contract surfaces if current carriers are insufficient | consume current envelope truth and add only strictly required additive support for the selected compatibility shape | preserve existing contract meaning and ordering; additive only |
| onboarding and session contract truth consumption | `crates/selene_kernel_contracts/src/ph1onb.rs`, `crates/selene_kernel_contracts/src/ph1l.rs`, `crates/selene_kernel_contracts/src/common.rs`, `crates/selene_os/src/runtime_session_foundation.rs` | consume accepted onboarding ids, session ids, turn ids, attach outcomes, and ownership/conflict truth | no Section 02 or onboarding contract semantic drift |
| app-layer onboarding truth anchors | `crates/selene_os/src/app_ingress.rs` | repo-truth reference only for request/action ordering and downstream sequencing | not a Slice 2C implementation target unless a strictly additive compatibility carrier bridge is proven necessary |
| downstream onboarding, governance, and persistence seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs`, downstream PH1.ONB execution paths | preserved downstream envelope and stage seams only | not a Slice 2C implementation target |
| Apple/client and app-specific ingress surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2C | no Apple work, no app work, no client workaround architecture |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2C must be:

1. extend the accepted Slice 1B/2A/2B route shell so `/v1/onboarding/continue` becomes executable only for `AskMissingSubmit` while `/v1/voice/turn` and `/v1/invite/click` remain unchanged and all other onboarding actions remain non-executable
2. define the bounded selected request shape using `AppOnboardingContinueRequest` plus `AppOnboardingContinueAction::AskMissingSubmit` field truth and canonical request-envelope fields, without inventing app/client production behavior
3. normalize the selected onboarding request into the accepted compatibility branch of the canonical carrier without changing the accepted turn-start carrier or accepted invite-click carrier
4. reuse accepted Slice 1B request security, replay-hook posture, idempotency propagation, and trigger validation foundations so selected onboarding entry remains fail closed
5. bind the selected onboarding compatibility path into the accepted Slice 1C/1D session and envelope discipline only to the bounded extent required to keep the route session-bound and canonical
6. reuse the canonical `RuntimeExecutionEnvelope` creation path and preserve every downstream Section 04/05/09/11 field as unset
7. reuse the accepted pre-authority stage-order scaffold by adding only the bounded compatibility-route classification needed for the selected onboarding action
8. implement deterministic pre-authority response/failure carriers, events, metrics, and trace propagation for the selected onboarding route
9. stop the success path at the bounded pre-authority handoff; do not execute `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior
10. close the slice only when route-scope proof, request-shape proof, carrier-reuse proof, envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no client/app bleed proof, and accepted Slice 1A-1D plus Slice 2A/2B regression proof all pass on a clean tree

Slice 2C must not reverse this order. Route activation may not outrun the bounded action shape. Compatibility normalization may not outrun accepted request security and trigger validation. Envelope creation may not outrun lawful session discipline. No protected execution may be entered before the slice is complete and a later Section 04 slice is explicitly opened.

J) verification and acceptance matrix

| proof area | required verification | Slice 2C acceptance condition |
|---|---|---|
| route-scope proof | prove that `/v1/onboarding/continue` becomes executable only for the selected `AskMissingSubmit` action, that `/v1/voice/turn` and `/v1/invite/click` remain the accepted canonical baselines, and that all other onboarding actions remain non-executable | Slice 2C opens exactly one new bounded onboarding compatibility action and nothing wider |
| selected request-shape proof | prove selected onboarding requests require the bounded `AppOnboardingContinueRequest` + `AskMissingSubmit` field truth plus canonical request-envelope fields and reject malformed or incomplete inputs fail closed | the selected onboarding action enters the runtime with one deterministic shape only |
| canonical carrier reuse proof | prove selected onboarding requests normalize into the accepted compatibility branch of the canonical carrier and do not alter the accepted voice-turn or invite-click carriers | the selected action gains one bounded normalized path and nothing parallel |
| request-envelope and trigger proof | prove accepted Slice 1B request-envelope rules and accepted trigger law remain enforced for selected onboarding execution | no route-specific security or trigger shortcut exists |
| session discipline proof | prove Slice 2C consumes accepted Slice 1C/1D session truth to keep the selected onboarding route lawful and session-bound without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| execution-envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and that raw onboarding compatibility requests do not survive past the envelope boundary | there is still one canonical runtime envelope path |
| stage-order proof | prove the selected onboarding compatibility path records deterministic stage order and stops at the pre-authority boundary | the selected route remains inside Section 03 and cannot skip or reorder gates |
| fail-closed proof | prove invalid selected request shape, invalid trigger posture, invalid session posture, invalid stage progression, and invalid envelope posture all reject immediately with deterministic failure class | no failed onboarding compatibility request can drift into later stages |
| pre-authority stop proof | prove the successful selected onboarding path stops at a bounded pre-authority handoff and does not execute `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior | Slice 2C stays upstream of protected execution and persistence |
| observability proof | prove the selected route emits deterministic events, counters, metrics, and trace propagation for every Slice 2C-owned stage | onboarding compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no identity verification, governance proof execution, onboarding authority decision, simulation execution, or protected business execution in Slice 2C | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable replay protection, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2C | Section 05 remains downstream and intact |
| no PH1.J / GOV / LAW bleed proof | prove Slice 2C preserves hook surfaces only and does not execute proof, governance, or runtime-law behavior | later runtime law surfaces remain downstream |
| no client/app bleed proof | prove there is no Apple/client behavior, no app-open or app-owned onboarding production behavior, and no app-specific workaround architecture introduced by Slice 2C | clients remain downstream thin terminals |
| accepted Slice 1A-1D and Slice 2A/2B regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2C code state | Slice 2C must not break the accepted runtime, session, or accepted Slice 2A/2B foundations |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2C is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/onboarding/continue` `AskMissingSubmit` compatibility execution foundation | `IN_SCOPE_SLICE_2C` | this is the only new executable Section 03 onboarding action in Slice 2C |
| preserve accepted `/v1/voice/turn`, accepted `/v1/invite/click`, accepted canonical carrier, and accepted pre-authority handoff behavior | `IN_SCOPE_SLICE_2C` | Slice 2C may extend Section 03, but it may not reinterpret or replace accepted Slice 2A or Slice 2B truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2C` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2C` | Slice 2C must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2C` | Slice 2C must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| `platform_setup` receipt compatibility execution | `DEFER_LATER_SECTION03` | do not pull platform-setup execution into Slice 2C |
| `terms_accept` compatibility execution | `DEFER_LATER_SECTION03` | do not pull `terms_accept` into Slice 2C |
| `primary_device_confirm` compatibility execution | `DEFER_LATER_SECTION03` | do not pull `primary_device_confirm` into Slice 2C |
| `voice_enroll`, wake-enroll, sender-verification, emo-persona, access-provision, and completion compatibility execution | `DEFER_LATER_SECTION03` | do not widen Slice 2C into downstream onboarding completion work |
| actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` missing-field update execution | `DEFER_SECTION04` | no simulation dispatch, no authoritative field mutation, and no committed result claims in Slice 2C |
| actual `PH1.ONB` execution beyond preserved seams | `DEFER_SECTION04` | no onboarding-state mutation, governance-protected action execution, or onboarding progression execution in Slice 2C |
| durable replay protection, duplicate outcome reuse, outbox, journal, reconcile, dedupe, and authoritative sync acknowledgement | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2C |
| PH1.J / proof execution, governance execution, and runtime-law execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve envelope seams only; do not build a hidden PH1.J / GOV / LAW path |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | these later layers may consume Slice 2C outputs but may not be implemented here |
| Apple/client behavior, app-open behavior, app-specific onboarding behavior, and any client-local authority or retry workaround path | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no hidden Section 04 path | `IN_SCOPE_SLICE_2C` | Section 03 must not become a parallel authority layer |
| no hidden Section 05 path | `IN_SCOPE_SLICE_2C` | Section 03 must not become a parallel persistence layer |
| no incomplete completion claim | `IN_SCOPE_SLICE_2C` | Slice 2C may not be called complete while any in-scope item or mandatory proof remains uncaptured |

L) SLICE 2C COMPLETION STANDARD

Slice 2C is complete only when the next lawful onboarding-compatible expansion of Section 03 exists and is proven without bleeding into later sections. Completion requires:

- `/v1/onboarding/continue` is live only for the selected `AskMissingSubmit` compatibility action
- `/v1/voice/turn` remains the accepted canonical turn route unchanged
- `/v1/invite/click` remains the accepted canonical invite-click compatibility route unchanged
- selected onboarding ask-missing requests validate against one bounded deterministic request shape anchored to `AppOnboardingContinueRequest` plus `AskMissingSubmit` field truth and canonical request-envelope truth
- selected onboarding ask-missing requests normalize into the accepted compatibility branch of the canonical carrier without altering the accepted voice-turn or invite-click carriers
- accepted Slice 1B request security, accepted Slice 1C/1D session truth, accepted Slice 2A/2B envelope path, and accepted Slice 2A/2B pre-authority scaffold are all reused rather than replaced
- the selected route stops at a deterministic pre-authority handoff and returns deterministic success/refusal carriers
- `platform_setup`, `terms_accept`, `primary_device_confirm`, `voice_enroll`, and every later onboarding action remain non-executable and explicitly deferred
- no `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior exists in the slice
- no PH1.J / GOV / LAW execution behavior exists in the slice beyond preserved hook surfaces
- no client/app/Apple behavior exists in the slice
- accepted Slice 1A-1D and accepted Slice 2A/2B regressions all remain green
- the slice closes on a clean tree with no uncaptured in-scope item, no uncaptured deferred boundary, and no unproven completion claim

Slice 2C is not complete if raw `/v1/onboarding/continue` becomes a multi-action executable bucket, if the accepted carrier or accepted envelope path is duplicated or bypassed, if `platform_setup`, `terms_accept`, `primary_device_confirm`, or `voice_enroll` are pulled forward, if `LINK_INVITE_DRAFT_UPDATE_COMMIT` or `PH1.ONB` execution is embedded directly into the slice, or if any mandatory proof in `J)` is missing.

M) PHASE BOUNDARY

H5 governs Slice 2C only.

- H5 does not authorize execution beyond the selected `/v1/onboarding/continue` `AskMissingSubmit` compatibility execution foundation
- H5 does not reopen H1, H2, H3, or H4
- H5 does not authorize any Section 04 or Section 05 implementation
- H5 does not authorize Apple/client implementation or app-layer workaround architecture
- H5 does not authorize deeper modality work on `/v1/voice/turn`
- the next lawful step after H5 is a bounded Slice 2C implementation run against this exact frozen scope only
