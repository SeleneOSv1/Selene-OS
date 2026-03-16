PHASE H6 — SLICE 2D SECTION 03 BOUNDARY AND BUILD PLAN

A) PURPOSE

H6 exists because accepted Slice 2C closed only the `AskMissingSubmit` ambiguity for
`/v1/onboarding/continue`, while the next bounded Section 03 onboarding slice remained
unfrozen.

H6 must therefore:

- identify the remaining lawful Section 03 onboarding surfaces after accepted Slice 2C
- compare the remaining candidate next slices against current repo truth
- select exactly one bounded Slice 2D
- defer every non-selected candidate explicitly with reason
- preserve the accepted Section 03 carrier, envelope, and pre-authority boundary law

H6 is planning law only.

H6 does not implement code.

B) FROZEN LAW INPUTS

H6 is downstream of, and bound by, all prior frozen and accepted law:

- frozen A-G architecture, sequencing, and dependency law captured by `CORE_ARCHITECTURE`,
  `SELENE_BUILD_EXECUTION_ORDER`, `SELENE_AUTHORITATIVE_ENGINE_INVENTORY`, Build
  Sections `01` through `11`, and the frozen Phase F / Phase G summaries
- accepted Slice 1A baseline `f9769797c28bc991df3720d085639a7117b3d7c8`
- accepted Slice 1B baseline `eb4be3fdfc100fa22684293cacee471faf7d7847`
- accepted Slice 1C baseline `b1eba355e716887f0fe399cc6930988e0423e7db`
- accepted Slice 1D baseline `46b021dece36b9c1d8589362cf0ada0187603a83`
- frozen H3 plan HEAD `ddf741bca99cdb88257dac1e1ea13e123ba8ecb7`
- frozen H4 plan HEAD `b4af35c0ec0377905412116ffdc18547a8f0ff03`
- frozen H5 plan HEAD `9dc0153c289a0975c176d71d12598884ad6f4097`
- accepted Slice 2A implementation HEAD `743ea0fe3e2ef884efb1c28bec706fe4efab91c9`
- accepted Slice 2B implementation HEAD `cd340ea14cfb2c8b7c15fc0cb578daf1e8e168fe`
- accepted Slice 2C implementation HEAD `2bf95ea098082747f53c1df575fe350385df9fc1`

H6 must preserve all accepted upstream runtime truth, especially:

- one canonical normalized turn-start carrier
- one canonical `RuntimeExecutionEnvelope` path
- one deterministic pre-authority stop line
- one accepted Slice 1C/1D session discipline
- no Section 04 or Section 05 widening
- no Apple/client widening
- no alternate authority path

C) CURRENT / TARGET / GAP

CURRENT

- accepted Slice 2A already established the canonical Section 03 route family, canonical
  normalized carrier, canonical session/envelope binding path, and canonical pre-authority
  stop line
- accepted Slice 2B already made `/v1/invite/click` executable as a bounded compatibility
  route without widening into client/app work or Section 04 / 05 behavior
- accepted Slice 2C already made `/v1/onboarding/continue` executable only for
  `AskMissingSubmit`
- current repo truth still leaves the same route family carrying later onboarding actions:
  `PlatformSetupReceipt`, `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`,
  `wake_enroll`, `voice_sample`, `voice_complete`, sender verification, emo-persona,
  access provision, and completion
- current repo truth also shows the remaining onboarding branches do not share one single
  downstream seam:
  - `PlatformSetupReceipt` still sits on the `LINK_INVITE_DRAFT_UPDATE_COMMIT` seam
  - `TermsAccept` already maps directly to `ONB_TERMS_ACCEPT_COMMIT`
  - `PrimaryDeviceConfirm` already maps directly to `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
  - `VoiceEnrollLock` and later actions are further downstream and semantically denser
- deeper modality-specific payload behavior remains a separate later Section 03 track on the
  already-live `/v1/voice/turn` path

TARGET

- freeze exactly one next bounded Section 03 slice after accepted Slice 2C
- keep the selected slice inside the accepted route family, accepted compatibility branch of
  the canonical carrier, accepted session discipline, accepted `RuntimeExecutionEnvelope`
  path, and accepted pre-authority stop line
- prevent any guesswork about what Slice 2D is allowed to build

GAP

- H5 correctly deferred the remaining onboarding actions, but it did not freeze which one
  becomes Slice 2D
- current repo truth proves the remaining actions are not interchangeable because they sit on
  different downstream seams and different dependency chains
- without H6, implementation would have to guess whether the next lawful slice is
  `PlatformSetupReceipt`, `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`,
  another narrower sub-slice, a combined bucket, or a deeper modality refinement

D) SLICE 2D SELECTION DECISION

Slice 2D is:

- the canonical `/v1/onboarding/continue` `PlatformSetupReceipt` compatibility execution
  foundation

This is the selected next bounded Section 03 slice.

Why this is the next lawful slice:

- it is the immediate next bounded onboarding continuation branch after accepted
  `AskMissingSubmit` in current repo truth
- it stays on the same downstream protected seam as accepted Slice 2C:
  `LINK_INVITE_DRAFT_UPDATE_COMMIT`
- it is still smaller and less authority-dense than `TermsAccept`,
  `PrimaryDeviceConfirm`, `VoiceEnrollLock`, or later onboarding completion actions
- it can remain wholly inside the accepted pre-authority Section 03 stop line because the
  slice can validate, normalize, session-bind, envelope-bind, classify, and stop before
  actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` or `PH1.ONB` execution
- it resolves the next real route-level ambiguity without reopening governance-proof,
  voice-enrollment, completion, client/app, or deep modality work
- repo truth does not provide a narrower lawful micro-slice inside `PlatformSetupReceipt`;
  receipt-kind-specific slicing would invent a new boundary that current contracts do not freeze

What Slice 2D means precisely:

- `/v1/onboarding/continue` remains executable for the accepted `AskMissingSubmit` baseline
- `/v1/onboarding/continue` becomes additionally executable only for `PlatformSetupReceipt`
- the selected action must use the bounded `AppOnboardingContinueRequest` field truth:
  - `correlation_id`
  - `onboarding_session_id`
  - `idempotency_key`
  - optional `tenant_id`
  - `action = PlatformSetupReceipt { receipt_kind, receipt_ref, signer, payload_hash }`
  - canonical request-envelope fields from the accepted Section 03 ingress foundation
- the selected action must normalize into the accepted compatibility branch of the same
  canonical turn-start carrier rather than creating a second onboarding carrier
- the selected action must reuse the accepted Slice 1C/1D session discipline and the accepted
  `RuntimeExecutionEnvelope` path
- the selected action must stop at the same bounded pre-authority handoff result used by
  accepted Slice 2A, Slice 2B, and Slice 2C

Why the non-selected candidates are deferred:

- `TermsAccept` remains deferred because it depends on ask-missing completion and platform
  setup completion and already maps directly to `ONB_TERMS_ACCEPT_COMMIT`
- `PrimaryDeviceConfirm` remains deferred because it depends on the earlier onboarding
  branches and adds governance proof posture while already mapping directly to
  `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
- `VoiceEnrollLock` and later onboarding actions remain deferred because they are further
  downstream and pull in additional voice, sender-verification, emo-persona, access, or
  completion semantics
- later sub-actions such as `voice_sample` and `voice_complete` remain deferred because they
  are downstream of `VoiceEnrollLock` and are not viable next-slice candidates ahead of the
  earlier route-family branches
- the combined remaining onboarding-continue execution bucket remains deferred because it
  would reintroduce the same route-level ambiguity H6 is required to remove
- deeper modality-specific payload behavior remains deferred because it belongs to the
  already-live `/v1/voice/turn` path, not the next bounded onboarding compatibility slice
- receipt-kind-specific micro-slicing remains out of scope because current repo truth models
  one bounded `PlatformSetupReceipt` action, not multiple frozen receipt-kind execution slices

E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/onboarding/continue` `PlatformSetupReceipt` compatibility execution foundation | `IN_SCOPE_SLICE_2D` | `PlatformSetupReceipt` is the immediate next bounded continuation branch after accepted `AskMissingSubmit`, and it remains on the `LINK_INVITE_DRAFT_UPDATE_COMMIT` seam | selected | smallest lawful next onboarding sub-slice that can stay pre-authority and reuse accepted Slice 2A/2B/2C Section 03 foundations |
| canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires ask-missing completion and platform setup completion before `TermsAccept`, and the action maps directly to `ONB_TERMS_ACCEPT_COMMIT` | deferred | too far downstream for the next bounded slice and already attached to a stronger PH1.ONB seam |
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires earlier onboarding completion and adds governance proof before `PrimaryDeviceConfirm` can proceed | deferred | not the next slice because it is downstream and touches a stronger authority seam |
| canonical `/v1/onboarding/continue` `VoiceEnrollLock` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth places it after ask-missing, platform setup, terms, and primary-device confirmation | deferred | further downstream and more semantically dense than the selected slice |
| later onboarding sub-slices such as `voice_sample` / `voice_complete` | `DEFER_LATER_SECTION03` | repo truth places those actions after `VoiceEnrollLock` in the same later onboarding chain | deferred | not credible next slices while earlier continuation branches remain unopened |
| combined remaining `/v1/onboarding/continue` execution bucket | `DEFER_LATER_SECTION03` | the route still contains multiple materially different action families and downstream seams | deferred | too broad; it recreates the exact ambiguity H6 must remove |
| deeper modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | H3/H4/H5 already defer it as a later `/v1/voice/turn` concern, not the next onboarding route-family expansion | deferred | not the next onboarding compatibility step |
| receipt-kind-specific micro-slice inside `PlatformSetupReceipt` | `OUT_OF_SCOPE_THIS_PHASE` | repo truth defines one bounded `PlatformSetupReceipt` action shape and does not freeze receipt-kind-specific execution slices | rejected as a Slice 2D candidate | would invent a new boundary not present in current contracts |

F) selected slice scope and dependency matrix

| item | classification | Slice 2D position | dependency / guardrail |
|---|---|---|---|
| accepted canonical `/v1/onboarding/continue` executable compatibility route for `AskMissingSubmit` only | `IN_SCOPE_SLICE_2D` as protected baseline | preserved accepted Slice 2C behavior | Slice 2D must not reinterpret or replace accepted Slice 2C truth |
| canonical `/v1/onboarding/continue` executable compatibility route for accepted `AskMissingSubmit` plus selected `PlatformSetupReceipt` only | `IN_SCOPE_SLICE_2D` | next executable canonical Section 03 onboarding expansion after accepted Slice 2C | extend the accepted Slice 1B/2A/2B/2C route shell; do not create a new route family |
| selected platform-setup request-shape foundation using `AppOnboardingContinueRequest` and `AppOnboardingContinueAction::PlatformSetupReceipt` field truth plus canonical request-envelope fields | `IN_SCOPE_SLICE_2D` | define the bounded Section 03 request shape for the selected onboarding action | consume current app/onboarding contract truth only; do not execute app-layer logic |
| canonical onboarding compatibility carrier reuse inside the accepted turn-start carrier | `IN_SCOPE_SLICE_2D` | normalize the selected action into the same canonical compatibility branch already established by accepted Slice 2A/2B/2C | no second onboarding carrier and no carrier fork |
| accepted `/v1/voice/turn` route and accepted `/v1/invite/click` route | `IN_SCOPE_SLICE_2D` as protected baselines | preserved baselines, not redefined | Slice 2D must not alter accepted Slice 2A or Slice 2B behavior |
| accepted Slice 1B request-envelope validation, request security, admission, replay-hook posture, and invariant foundations | `IN_SCOPE_SLICE_2D` | consumed exactly as built | no duplicate admission stack and no route-shell replacement |
| trigger validation for selected onboarding compatibility requests | `IN_SCOPE_SLICE_2D` | validate bounded trigger posture before deeper stage entry | must consume accepted `PlatformRuntimeContext` truth only |
| bounded session resolve-or-open or lawful session anchoring for selected onboarding compatibility execution | `IN_SCOPE_SLICE_2D` | keep the selected route session-bound and lawful | consume accepted Slice 1C/1D session truth only; no onboarding-owned session law |
| canonical `RuntimeExecutionEnvelope` reuse for selected onboarding requests | `IN_SCOPE_SLICE_2D` | the selected route must enter the same canonical envelope path used by accepted Section 03 | no raw onboarding compatibility request survives past the envelope boundary |
| compatibility-route classification and pre-authority handoff foundation for the selected action | `IN_SCOPE_SLICE_2D` | classify the selected onboarding route at the bounded Section 03 handoff without authorizing protected execution | must stop before real `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior |
| canonical response/failure carrier for selected onboarding pre-authority outcomes | `IN_SCOPE_SLICE_2D` | return deterministic pre-authority acceptance or refusal anchors | no authoritative onboarding mutation, verification, or sync acknowledgement claims |
| `TermsAccept` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep `TermsAccept` deferred in Slice 2D |
| `PrimaryDeviceConfirm` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep `PrimaryDeviceConfirm` deferred in Slice 2D |
| `VoiceEnrollLock`, `wake_enroll`, `voice_sample`, `voice_complete`, sender-verification, emo-persona, access-provision, and completion onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slices only | do not widen Slice 2D into downstream onboarding completion work |
| actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` execution for platform-setup receipt mutation | `DEFER_SECTION04` | downstream protected execution only | no storage mutation, no authoritative receipt-commit result in Slice 2D |
| actual `PH1.ONB` execution (`TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, and later onboarding mutation behavior) | `DEFER_SECTION04` | downstream protected execution only | no onboarding state mutation or governance-protected action execution in Slice 2D |
| durable replay, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, and authoritative ack behavior for onboarding compatibility routes | `DEFER_SECTION05` | downstream persistence and sync only | no local or in-memory pseudo-persistence substitute |
| PH1.J / GOV / LAW execution beyond preserved envelope hooks | `DEFER_LATER_RUNTIME` | preserved seam only | no proof, governance, or runtime-law execution path in Slice 2D |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | downstream only | later layers may consume Slice 2D outputs but may not be implemented here |
| Apple/client app-open production, app-specific onboarding workarounds, and any client-local authority path | `OUT_OF_SCOPE_THIS_PHASE` | explicit non-target | no Apple work, no app work, no client-owned execution truth |

G) request-shape / continuation-action matrix

| request family or continuation action | canonical route or boundary | Slice 2D posture | normalized outcome | mandatory boundary |
|---|---|---|---|---|
| accepted canonical turn family | `/v1/voice/turn` | `IN_SCOPE_SLICE_2D` as protected baseline | continue using the accepted canonical turn-start carrier and accepted envelope path unchanged | Slice 2D must not alter accepted Slice 2A behavior |
| accepted invite-click compatibility family | `/v1/invite/click` | `IN_SCOPE_SLICE_2D` as protected baseline | continue using the accepted Slice 2B invite-click compatibility carrier unchanged | Slice 2D must not alter accepted Slice 2B behavior |
| accepted onboarding ask-missing action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2D` as protected baseline | continue using the accepted Slice 2C onboarding ask-missing compatibility carrier unchanged | Slice 2D must not alter accepted Slice 2C behavior |
| selected onboarding platform-setup receipt action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2D` | normalize `AppOnboardingContinueRequest { onboarding_session_id, idempotency_key, tenant_id, action = PlatformSetupReceipt { receipt_kind, receipt_ref, signer, payload_hash } }` plus canonical request-envelope fields into the accepted compatibility branch of the canonical carrier and then reuse the canonical `RuntimeExecutionEnvelope` path | no direct `LINK_INVITE_DRAFT_UPDATE_COMMIT` execution and no hidden alternate authority path |
| `TermsAccept` action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2D | must not become a hidden `ONB_TERMS_ACCEPT_COMMIT` path |
| `PrimaryDeviceConfirm` action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2D | must not become a hidden governance or protected-action path |
| `VoiceEnrollLock`, `wake_enroll`, `voice_sample`, `voice_complete`, and later onboarding actions | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain canonical continuation-action shapes but non-executable in Slice 2D | do not pull wake-enroll, sender-verification, emo-persona, access-provision, or completion forward |
| deeper modality-specific turn semantics | `/v1/voice/turn` internal modality handling only | `DEFER_LATER_SECTION03` | keep the accepted Slice 2A normalization outcome unchanged for now | no voice/file/image/camera-specific business logic in Slice 2D |
| app-open / client-produced onboarding behavior | client/app surface only | `OUT_OF_SCOPE_THIS_PHASE` | no Section 03 runtime normalization target in Slice 2D | Phase F / Phase G and `app_ingress.rs` retain this boundary |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2D role | mandatory posture |
|---|---|---|---|
| route-family activation and router integration | `crates/selene_os/src/runtime_request_foundation.rs` | preserve the already-open accepted Slice 1B/2A/2B/2C route shell while extending `/v1/onboarding/continue` execution only for the selected platform-setup action if strictly required | reuse the accepted router and middleware stack; do not replace it |
| Section 03 compatibility execution module | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | primary expected host for selected platform-setup normalization, envelope reuse, stage-order reuse, and pre-authority response/failure foundation | additive only; no rewrite of accepted Slice 2A, Slice 2B, or Slice 2C paths |
| onboarding compatibility request-shape and envelope-adjacent contract support | `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_kernel_contracts/src/common.rs`, `crates/selene_kernel_contracts/src/ph1link.rs`, `crates/selene_kernel_contracts/src/ph1onb.rs` | consume accepted contract truth for request-envelope, simulation-id seams, and ids | reference-first posture; do not widen or semantically drift shared contracts unless strictly necessary and lawfully justified |
| accepted session substrate consumption | `crates/selene_os/src/runtime_session_foundation.rs` | consume accepted session ids, attach outcomes, turn ordering, and access posture | not a Slice 2D redesign target |
| app-layer onboarding truth anchors | `crates/selene_os/src/app_ingress.rs` | repo-truth reference only for action ordering, request-shape truth, and downstream sequencing | not a Slice 2D implementation target |
| downstream onboarding, governance, and persistence seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs`, downstream PH1.ONB execution paths | preserved downstream envelope and stage seams only | not a Slice 2D implementation target |
| Apple/client and app-specific ingress surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2D | no Apple work, no app work, no client workaround architecture |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2D must be:

1. preserve the accepted Slice 2A `/v1/voice/turn` path, accepted Slice 2B `/v1/invite/click` path, and accepted Slice 2C `AskMissingSubmit` path while extending `/v1/onboarding/continue` execution only for the selected `PlatformSetupReceipt` action
2. define the bounded selected request shape using `AppOnboardingContinueRequest` plus `AppOnboardingContinueAction::PlatformSetupReceipt` field truth and canonical request-envelope fields, without inventing app/client production behavior
3. normalize the selected onboarding request into the accepted compatibility branch of the canonical carrier without changing the accepted voice-turn carrier, accepted invite-click carrier, or accepted ask-missing carrier
4. reuse accepted Slice 1B request security, replay-hook posture, idempotency propagation, and trigger validation foundations so selected onboarding entry remains fail closed
5. bind the selected onboarding compatibility path into the accepted Slice 1C/1D session and envelope discipline only to the bounded extent required to keep the route session-bound and canonical
6. reuse the canonical `RuntimeExecutionEnvelope` creation path and preserve every downstream Section 04/05/09/11 field as unset
7. reuse the accepted pre-authority stage-order scaffold by adding only the bounded compatibility-route classification needed for the selected platform-setup action
8. implement deterministic pre-authority response/failure carriers, events, metrics, and trace propagation for the selected onboarding route
9. stop the success path at the bounded pre-authority handoff; do not execute `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior
10. close the slice only when route-scope proof, request-shape proof, carrier-reuse proof, envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no client/app bleed proof, and accepted Slice 1A-1D plus Slice 2A/2B/2C regression proof all pass on a clean tree

Slice 2D must not reverse this order. Route activation may not outrun the bounded action shape.
Compatibility normalization may not outrun accepted request security and trigger validation.
Envelope creation may not outrun lawful session discipline. No protected execution may be
entered before the slice is complete and a later Section 04 slice is explicitly opened.

J) verification and acceptance matrix

| proof area | required verification | Slice 2D acceptance condition |
|---|---|---|
| route-scope proof | prove that `/v1/onboarding/continue` becomes executable only for the already-accepted `AskMissingSubmit` action and the selected `PlatformSetupReceipt` action, that `/v1/voice/turn` and `/v1/invite/click` remain the accepted canonical baselines, and that all later onboarding actions remain non-executable | Slice 2D opens exactly one new bounded onboarding compatibility action and nothing wider |
| selected request-shape proof | prove selected onboarding requests require the bounded `AppOnboardingContinueRequest` + `PlatformSetupReceipt` field truth plus canonical request-envelope fields and reject malformed or incomplete inputs fail closed | the selected onboarding action enters the runtime with one deterministic shape only |
| canonical carrier reuse proof | prove selected onboarding requests normalize into the accepted compatibility branch of the canonical carrier and do not alter the accepted voice-turn, invite-click, or ask-missing carriers | the selected action gains one bounded normalized path and nothing parallel |
| request-envelope and trigger proof | prove accepted Slice 1B request-envelope rules and accepted trigger law remain enforced for selected onboarding execution | no route-specific security or trigger shortcut exists |
| session discipline proof | prove Slice 2D consumes accepted Slice 1C/1D session truth to keep the selected onboarding route lawful and session-bound without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| execution-envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and that raw onboarding compatibility requests do not survive past the envelope boundary | there is still one canonical runtime envelope path |
| stage-order proof | prove the selected onboarding compatibility path records deterministic stage order and stops at the pre-authority boundary | the selected route remains inside Section 03 and cannot skip or reorder gates |
| fail-closed proof | prove invalid selected request shape, invalid trigger posture, invalid session posture, invalid stage progression, and invalid envelope posture all reject immediately with deterministic failure class | no failed onboarding compatibility request can drift into later stages |
| pre-authority stop proof | prove the successful selected onboarding path stops at a bounded pre-authority handoff and does not execute `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior | Slice 2D stays upstream of protected execution and persistence |
| observability proof | prove the selected route emits deterministic events, counters, metrics, and trace propagation for every Slice 2D-owned stage | onboarding compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no identity verification, governance proof execution, onboarding authority decision, simulation execution, or protected business execution in Slice 2D | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable replay protection, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2D | Section 05 remains downstream and intact |
| no PH1.J / GOV / LAW bleed proof | prove Slice 2D preserves hook surfaces only and does not execute proof, governance, or runtime-law behavior | later runtime law surfaces remain downstream |
| no client/app bleed proof | prove there is no Apple/client behavior, no app-open or app-owned onboarding production behavior, and no app-specific workaround architecture introduced by Slice 2D | clients remain downstream thin terminals |
| accepted Slice 1A-1D and Slice 2A/2B/2C regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2D code state | Slice 2D must not break the accepted runtime, session, or accepted Slice 2A/2B/2C foundations |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2D is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/onboarding/continue` `PlatformSetupReceipt` compatibility execution foundation | `IN_SCOPE_SLICE_2D` | this is the only new executable Section 03 onboarding action in Slice 2D |
| preserve accepted `/v1/onboarding/continue` `AskMissingSubmit`, accepted `/v1/voice/turn`, accepted `/v1/invite/click`, accepted canonical carrier, and accepted pre-authority handoff behavior | `IN_SCOPE_SLICE_2D` | Slice 2D may extend Section 03, but it may not reinterpret or replace accepted Slice 2A, Slice 2B, or Slice 2C truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2D` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2D` | Slice 2D must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2D` | Slice 2D must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| `TermsAccept` compatibility execution | `DEFER_LATER_SECTION03` | do not pull `TermsAccept` into Slice 2D |
| `PrimaryDeviceConfirm` compatibility execution | `DEFER_LATER_SECTION03` | do not pull `PrimaryDeviceConfirm` into Slice 2D |
| `VoiceEnrollLock`, `wake_enroll`, `voice_sample`, `voice_complete`, sender-verification, emo-persona, access-provision, and completion compatibility execution | `DEFER_LATER_SECTION03` | do not widen Slice 2D into downstream onboarding completion work |
| combined remaining onboarding-continue execution bucket | `DEFER_LATER_SECTION03` | do not widen Slice 2D into a multi-action remaining-onboarding bucket |
| deeper modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | do not embed voice/file/image/camera-specific business logic into Slice 2D |
| actual `LINK_INVITE_DRAFT_UPDATE_COMMIT` platform-setup receipt execution | `DEFER_SECTION04` | no simulation dispatch, no authoritative receipt mutation, and no committed result claims in Slice 2D |
| actual `PH1.ONB` execution beyond preserved seams | `DEFER_SECTION04` | no onboarding-state mutation, governance-protected action execution, or onboarding progression execution in Slice 2D |
| durable replay protection, duplicate outcome reuse, outbox, journal, reconcile, dedupe, and authoritative sync acknowledgement | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2D |
| PH1.J / proof execution, governance execution, and runtime-law execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve envelope seams only; do not build a hidden PH1.J / GOV / LAW path |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | these later layers may consume Slice 2D outputs but may not be implemented here |
| Apple/client behavior, app-open behavior, app-specific onboarding behavior, and any client-local authority or retry workaround path | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no hidden Section 04 path | `IN_SCOPE_SLICE_2D` | Section 03 must not become a parallel authority layer |
| no hidden Section 05 path | `IN_SCOPE_SLICE_2D` | Section 03 must not become a parallel persistence layer |
| no incomplete completion claim | `IN_SCOPE_SLICE_2D` | Slice 2D may not be called complete while any in-scope item or mandatory proof remains uncaptured |

L) SLICE 2D COMPLETION STANDARD

Slice 2D is complete only when the next lawful onboarding-compatible expansion of Section 03
exists and is proven without bleeding into later sections. Completion requires:

- `/v1/onboarding/continue` remains live for the accepted `AskMissingSubmit` compatibility action
- `/v1/onboarding/continue` becomes live for the selected `PlatformSetupReceipt`
  compatibility action and nothing wider
- `/v1/voice/turn` remains the accepted canonical turn route unchanged
- `/v1/invite/click` remains the accepted canonical invite-click compatibility route unchanged
- selected onboarding platform-setup requests validate against one bounded deterministic
  request shape anchored to `AppOnboardingContinueRequest` plus `PlatformSetupReceipt` field
  truth and canonical request-envelope truth
- selected onboarding platform-setup requests normalize into the accepted compatibility branch
  of the canonical carrier without altering the accepted voice-turn, invite-click, or
  ask-missing carriers
- accepted Slice 1B request security, accepted Slice 1C/1D session truth, accepted
  Slice 2A/2B/2C envelope path, and accepted Slice 2A/2B/2C pre-authority scaffold are all
  reused rather than replaced
- the selected route stops at a deterministic pre-authority handoff and returns deterministic
  success/refusal carriers
- `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `wake_enroll`, `voice_sample`,
  `voice_complete`, and every later onboarding action remain non-executable and explicitly
  deferred
- no `LINK_INVITE_DRAFT_UPDATE_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior exists
  in the slice
- no PH1.J / GOV / LAW execution behavior exists in the slice beyond preserved hook surfaces
- no client/app/Apple behavior exists in the slice
- accepted Slice 1A-1D and accepted Slice 2A/2B/2C regressions all remain green
- the slice closes on a clean tree with no uncaptured in-scope item, no uncaptured deferred
  boundary, and no unproven completion claim

Slice 2D is not complete if raw `/v1/onboarding/continue` becomes a multi-action executable
bucket, if the accepted carrier or accepted envelope path is duplicated or bypassed, if
`TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `voice_sample`, or `voice_complete`
are pulled forward, if `LINK_INVITE_DRAFT_UPDATE_COMMIT` or `PH1.ONB` execution is embedded
directly into the slice, or if any mandatory proof in `J)` is missing.

M) PHASE BOUNDARY

Slice 2D ends at the same Section 03 pre-authority boundary already established by accepted
Slice 2A, accepted Slice 2B, and accepted Slice 2C.

The selected `PlatformSetupReceipt` action may:

- enter through the accepted canonical route family
- validate against bounded request-shape truth
- normalize into the accepted compatibility branch of the canonical carrier
- reuse the accepted Slice 1C/1D session discipline
- reuse the accepted `RuntimeExecutionEnvelope`
- record deterministic pre-authority classification, events, metrics, and failure posture
- stop at the bounded pre-authority handoff

The selected `PlatformSetupReceipt` action may not:

- execute `LINK_INVITE_DRAFT_UPDATE_COMMIT`
- execute `PH1.ONB`
- execute governance proof or protected authorization decisions
- execute Section 05 persistence or sync behavior
- create a second onboarding carrier
- create a second runtime envelope path
- create an alternate authority path
- widen into client/app/Apple behavior

After Slice 2D:

- accepted `AskMissingSubmit` remains the accepted first onboarding compatibility action
- selected `PlatformSetupReceipt` becomes the accepted second onboarding compatibility action
- `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `wake_enroll`, `voice_sample`,
  `voice_complete`, and later onboarding actions remain deferred
- Section 04 and Section 05 remain downstream
- the next onboarding compatibility slice is not opened by this document
