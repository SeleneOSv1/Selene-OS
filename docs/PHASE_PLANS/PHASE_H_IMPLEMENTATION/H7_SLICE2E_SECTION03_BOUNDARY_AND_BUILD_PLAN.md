PHASE H7 — SLICE 2E SECTION 03 BOUNDARY AND BUILD PLAN

A) PURPOSE

H7 exists because accepted Slice 2D closed only the `PlatformSetupReceipt` ambiguity for
`/v1/onboarding/continue`.

Accepted Slice 2D left a bounded later-Section03 onboarding set still deferred:

- `TermsAccept`
- `PrimaryDeviceConfirm`
- `VoiceEnrollLock`
- `WakeEnrollSampleCommit`
- `WakeEnrollCompleteCommit`
- later onboarding completion actions

H7 removes the next ambiguity in that remaining route family by freezing exactly one next bounded
Section 03 slice:

- the canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation

H7 is binding implementation law for Slice 2E only.

B) FROZEN LAW INPUTS

H7 consumes and preserves the following frozen law inputs:

- frozen Phase A through Phase G architecture and freeze law
- `docs/CORE_ARCHITECTURE.md`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
- frozen `H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md`
- frozen `H2_SLICE1_RUNTIME_AND_SESSION_BUILD_PLAN.md`
- frozen H3 plan head `ddf741bca99cdb88257dac1e1ea13e123ba8ecb7`
- frozen H4 plan head `b4af35c0ec0377905412116ffdc18547a8f0ff03`
- frozen H5 plan head `9dc0153c289a0975c176d71d12598884ad6f4097`
- frozen H6 plan head `14cba4ff81088edf3f628d7c1cbed51d57fa220f`

H7 also consumes and preserves the accepted runtime baselines:

- accepted Slice 1A baseline `f9769797c28bc991df3720d085639a7117b3d7c8`
- accepted Slice 1B baseline `eb4be3fdfc100fa22684293cacee471faf7d7847`
- accepted Slice 1C baseline `b1eba355e716887f0fe399cc6930988e0423e7db`
- accepted Slice 1D baseline `46b021dece36b9c1d8589362cf0ada0187603a83`
- accepted Slice 2A implementation head `743ea0fe3e2ef884efb1c28bec706fe4efab91c9`
- accepted Slice 2B implementation head `cd340ea14cfb2c8b7c15fc0cb578daf1e8e168fe`
- accepted Slice 2C implementation head `2bf95ea098082747f53c1df575fe350385df9fc1`
- accepted Slice 2D implementation head `1ba64a5ebbfa71a8123e33c5881fb70042aabd85`

Repo-truth anchors consumed by H7:

- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_os/src/runtime_ingress_turn_foundation.rs`
- `crates/selene_os/src/runtime_request_foundation.rs`
- `crates/selene_os/src/runtime_session_foundation.rs`
- `crates/selene_kernel_contracts/src/runtime_execution.rs`
- `crates/selene_kernel_contracts/src/ph1onb.rs`

Frozen dependency law preserved by H7:

- one canonical normalized turn-start carrier
- one accepted session resolve-or-open seam
- one canonical `RuntimeExecutionEnvelope` path
- one deterministic pre-authority stop line
- Section 04 remains downstream of Section 03
- Section 05 remains downstream of Section 04
- no Apple/client widening
- no alternate authority path

C) CURRENT / TARGET / GAP

CURRENT:

- accepted Slice 2D leaves `/v1/onboarding/continue` executable for `AskMissingSubmit` and
  `PlatformSetupReceipt` only
- repo truth still exposes later onboarding actions under the same route family:
  `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`,
  `WakeEnrollCompleteCommit`, and later completion actions
- current repo truth also shows that the real app-layer `TermsAccept` path already points toward
  `ONB_TERMS_ACCEPT_COMMIT`, which means the next slice must be frozen explicitly as
  compatibility-only before any implementation can proceed

TARGET:

- freeze one exact next Section 03 slice:
  `TermsAccept` compatibility execution

GAP:

- H6 deferred the remaining onboarding actions but did not freeze which one becomes next
- repo truth shows the remaining actions sit on materially different downstream seams
- without H7, Slice 2E implementation would have to guess whether the next bounded action is
  `TermsAccept`, `PrimaryDeviceConfirm`, `VoiceEnrollLock`, or a broader remaining-onboarding
  bucket

D) SLICE 2E SELECTION DECISION

Slice 2E is:

- the canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation

This is the selected next bounded Section 03 slice.

Why this is the next lawful slice:

- it is the immediate next onboarding continuation branch after accepted `PlatformSetupReceipt`
  in current repo truth
- it is more downstream than accepted Slice 2D but still narrower than any combined remaining
  onboarding bucket
- it can still be frozen as compatibility-only because the slice can validate, normalize,
  session-bind, envelope-bind, classify, and stop before real `ONB_TERMS_ACCEPT_COMMIT` or
  broader `PH1.ONB` execution
- it preserves the same canonical route family already accepted across Slice 2A, Slice 2B,
  Slice 2C, and Slice 2D without inventing a second onboarding carrier or a second envelope path
- later actions remain materially different: `PrimaryDeviceConfirm` adds governance proof
  posture, while `VoiceEnrollLock` and wake-enroll actions pull voice-artifact and later
  completion semantics forward
- repo truth does not provide a narrower lawful micro-slice inside `TermsAccept`; splitting by
  `accepted=true` versus `accepted=false` or by terms-version variant would invent a new
  boundary that current contracts do not freeze

What Slice 2E means precisely:

- `/v1/onboarding/continue` remains executable for the accepted `AskMissingSubmit` baseline
- `/v1/onboarding/continue` remains executable for the accepted `PlatformSetupReceipt` baseline
- `/v1/onboarding/continue` becomes additionally executable only for `TermsAccept`
- the selected action must use the bounded `AppOnboardingContinueRequest` field truth:
  - `correlation_id`
  - `onboarding_session_id`
  - `idempotency_key`
  - optional `tenant_id`
  - `action = TermsAccept { terms_version_id, accepted }`
  - canonical request-envelope fields from the accepted Section 03 ingress foundation
- the selected action must normalize into the accepted compatibility branch of the same canonical
  turn-start carrier rather than creating a second onboarding carrier
- the selected action must reuse the accepted Slice 1C/1D session discipline and the accepted
  `RuntimeExecutionEnvelope` path
- the selected action must stop at the same bounded pre-authority handoff result used by
  accepted Slice 2A, Slice 2B, Slice 2C, and Slice 2D
- the selected action remains compatibility-only and must not execute real
  `ONB_TERMS_ACCEPT_COMMIT`, real `PH1.ONB`, Section 04 authority, or Section 05 persistence

Why the non-selected candidates are deferred:

- `PrimaryDeviceConfirm` remains deferred because it is downstream of `TermsAccept`, adds proof
  posture, and already maps directly to `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`
- `VoiceEnrollLock` remains deferred because it is further downstream and pulls voice-enrollment
  semantics into the slice
- `WakeEnrollSampleCommit` and `WakeEnrollCompleteCommit` remain deferred because they are
  downstream of `VoiceEnrollLock` and are not viable next candidates ahead of the earlier
  onboarding branches
- the combined remaining onboarding-continue execution bucket remains deferred because it would
  reintroduce the same route-level ambiguity H7 is required to remove
- deeper modality-specific payload behavior remains deferred because it belongs to the already-live
  `/v1/voice/turn` path, not the next bounded onboarding compatibility slice
- any narrower micro-slice not explicitly supported by repo truth remains out of scope because it
  would invent new action boundaries rather than consuming the current contract truth

E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation | `IN_SCOPE_SLICE_2E` | `TermsAccept` is the immediate next bounded continuation branch after accepted `PlatformSetupReceipt`, while still able to reuse the accepted Section 03 carrier, session seam, and envelope path | selected | smallest lawful next onboarding sub-slice after accepted Slice 2D that can still stay pre-authority |
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth requires earlier onboarding completion and adds governance proof before `PrimaryDeviceConfirm` can proceed | deferred | downstream and attached to a stronger protected seam than the selected slice |
| canonical `/v1/onboarding/continue` `VoiceEnrollLock` compatibility execution foundation | `DEFER_LATER_SECTION03` | repo truth places it after terms and primary-device confirmation | deferred | further downstream and more semantically dense than the selected slice |
| later onboarding sub-slices such as `WakeEnrollSampleCommit` / `WakeEnrollCompleteCommit` | `DEFER_LATER_SECTION03` | repo truth places those actions after `VoiceEnrollLock` in the same later onboarding chain | deferred | not credible next slices while earlier continuation branches remain unopened |
| combined remaining `/v1/onboarding/continue` execution bucket | `DEFER_LATER_SECTION03` | the route still contains multiple materially different action families and downstream seams | deferred | too broad; it recreates the exact ambiguity H7 must remove |
| deeper modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | H3/H4/H5/H6 already defer it as a later `/v1/voice/turn` concern, not the next onboarding route-family expansion | deferred | not the next onboarding compatibility step |
| narrower micro-slice inside `TermsAccept` | `OUT_OF_SCOPE_THIS_PHASE` | repo truth defines one bounded `TermsAccept { terms_version_id, accepted }` action shape and does not freeze polarity-specific or version-specific execution slices | rejected as a Slice 2E candidate | would invent a new boundary not present in current contracts |

F) selected slice scope and dependency matrix

| item | classification | Slice 2E position | dependency / guardrail |
|---|---|---|---|
| accepted canonical `/v1/onboarding/continue` executable compatibility route for `AskMissingSubmit` only | `IN_SCOPE_SLICE_2E` as protected baseline | preserved accepted Slice 2C behavior | Slice 2E must not reinterpret or replace accepted Slice 2C truth |
| accepted canonical `/v1/onboarding/continue` executable compatibility route for accepted `AskMissingSubmit` plus accepted `PlatformSetupReceipt` only | `IN_SCOPE_SLICE_2E` as protected baseline | preserved accepted Slice 2D behavior | Slice 2E must not reinterpret or replace accepted Slice 2D truth |
| canonical `/v1/onboarding/continue` executable compatibility route for accepted `AskMissingSubmit`, accepted `PlatformSetupReceipt`, plus selected `TermsAccept` only | `IN_SCOPE_SLICE_2E` | next executable canonical Section 03 onboarding expansion after accepted Slice 2D | extend the accepted Slice 1B/2A/2B/2C/2D route shell; do not create a new route family |
| selected terms request-shape foundation using `AppOnboardingContinueRequest` and `AppOnboardingContinueAction::TermsAccept` field truth plus canonical request-envelope fields | `IN_SCOPE_SLICE_2E` | define the bounded Section 03 request shape for the selected onboarding action | consume current app/onboarding contract truth only; do not execute app-layer logic |
| canonical onboarding compatibility carrier reuse inside the accepted turn-start carrier | `IN_SCOPE_SLICE_2E` | normalize the selected action into the same canonical compatibility branch already established by accepted Slice 2A/2B/2C/2D | no second onboarding carrier and no carrier fork |
| accepted `/v1/voice/turn` route and accepted `/v1/invite/click` route | `IN_SCOPE_SLICE_2E` as protected baselines | preserved baselines, not redefined | Slice 2E must not alter accepted Slice 2A or Slice 2B behavior |
| accepted Slice 1B request-envelope validation, request security, admission, replay-hook posture, and invariant foundations | `IN_SCOPE_SLICE_2E` | consumed exactly as built | no duplicate admission stack and no route-shell replacement |
| trigger validation for selected onboarding compatibility requests | `IN_SCOPE_SLICE_2E` | validate bounded trigger posture before deeper stage entry | must consume accepted `PlatformRuntimeContext` truth only |
| bounded session resolve-or-open or lawful session anchoring for selected onboarding compatibility execution | `IN_SCOPE_SLICE_2E` | keep the selected route session-bound and lawful | consume accepted Slice 1C/1D session truth only; no onboarding-owned session law |
| canonical `RuntimeExecutionEnvelope` reuse for selected onboarding requests | `IN_SCOPE_SLICE_2E` | the selected route must enter the same canonical envelope path used by accepted Section 03 | no raw onboarding compatibility request survives past the envelope boundary |
| compatibility-route classification and pre-authority handoff foundation for the selected action | `IN_SCOPE_SLICE_2E` | classify the selected onboarding route at the bounded Section 03 handoff without authorizing protected execution | must stop before real `ONB_TERMS_ACCEPT_COMMIT`, real `PH1.ONB`, Section 04, or Section 05 behavior |
| canonical response/failure carrier for selected onboarding pre-authority outcomes | `IN_SCOPE_SLICE_2E` | return deterministic pre-authority acceptance or refusal anchors | no authoritative onboarding mutation, verification, or sync acknowledgement claims |
| `PrimaryDeviceConfirm` onboarding compatibility execution | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slice only | keep `PrimaryDeviceConfirm` deferred in Slice 2E |
| `VoiceEnrollLock`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, and later onboarding actions | `DEFER_LATER_SECTION03` | later onboarding compatibility-route slices only | do not widen Slice 2E into downstream onboarding completion work |
| actual `ONB_TERMS_ACCEPT_COMMIT` execution | `DEFER_SECTION04` | downstream protected execution only | no simulation dispatch, no authoritative terms mutation, and no committed result claims in Slice 2E |
| actual `PH1.ONB` execution beyond preserved seams | `DEFER_SECTION04` | downstream protected execution only | no onboarding state mutation or governance-protected action execution in Slice 2E |
| durable replay, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, and authoritative ack behavior for onboarding compatibility routes | `DEFER_SECTION05` | downstream persistence and sync only | no local or in-memory pseudo-persistence substitute |
| PH1.J / GOV / LAW execution beyond preserved envelope hooks | `DEFER_LATER_RUNTIME` | preserved seam only | no proof, governance, or runtime-law execution path in Slice 2E |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | downstream only | later layers may consume Slice 2E outputs but may not be implemented here |
| Apple/client app-open production, app-specific onboarding workarounds, and any client-local authority path | `OUT_OF_SCOPE_THIS_PHASE` | explicit non-target | no Apple work, no app work, no client-owned execution truth |

G) request-shape / continuation-action matrix

| request family or continuation action | canonical route or boundary | Slice 2E posture | normalized outcome | mandatory boundary |
|---|---|---|---|---|
| accepted canonical turn family | `/v1/voice/turn` | `IN_SCOPE_SLICE_2E` as protected baseline | continue using the accepted canonical turn-start carrier and accepted envelope path unchanged | Slice 2E must not alter accepted Slice 2A behavior |
| accepted invite-click compatibility family | `/v1/invite/click` | `IN_SCOPE_SLICE_2E` as protected baseline | continue using the accepted Slice 2B invite-click compatibility carrier unchanged | Slice 2E must not alter accepted Slice 2B behavior |
| accepted onboarding ask-missing action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2E` as protected baseline | continue using the accepted Slice 2C onboarding ask-missing compatibility carrier unchanged | Slice 2E must not alter accepted Slice 2C behavior |
| accepted onboarding platform-setup receipt action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2E` as protected baseline | continue using the accepted Slice 2D onboarding platform-setup compatibility carrier unchanged | Slice 2E must not alter accepted Slice 2D behavior |
| selected onboarding `TermsAccept` action | `/v1/onboarding/continue` | `IN_SCOPE_SLICE_2E` | normalize `AppOnboardingContinueRequest { onboarding_session_id, idempotency_key, tenant_id, action = TermsAccept { terms_version_id, accepted } }` plus canonical request-envelope fields into the accepted compatibility branch of the canonical carrier and then reuse the canonical `RuntimeExecutionEnvelope` path | no direct `ONB_TERMS_ACCEPT_COMMIT` execution and no hidden alternate authority path |
| `PrimaryDeviceConfirm` action | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain a canonical continuation-action shape but non-executable in Slice 2E | must not become a hidden `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT` or governance path |
| `VoiceEnrollLock`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, and later onboarding actions | `/v1/onboarding/continue` | `DEFER_LATER_SECTION03` | remain canonical continuation-action shapes but non-executable in Slice 2E | do not pull wake-enroll, sender-verification, emo-persona, access-provision, or completion forward |
| deeper modality-specific turn semantics | `/v1/voice/turn` internal modality handling only | `DEFER_LATER_SECTION03` | keep the accepted Slice 2A normalization outcome unchanged for now | no voice/file/image/camera-specific business logic in Slice 2E |
| app-open / client-produced onboarding behavior | client/app surface only | `OUT_OF_SCOPE_THIS_PHASE` | no Section 03 runtime normalization target in Slice 2E | Phase F / Phase G and `app_ingress.rs` retain this boundary |

H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2E role | mandatory posture |
|---|---|---|---|
| route-family activation and router integration | `crates/selene_os/src/runtime_request_foundation.rs` | preserve the already-open accepted Slice 1B/2A/2B/2C/2D route shell while extending `/v1/onboarding/continue` execution only for the selected terms action if strictly required | reuse the accepted router and middleware stack; do not replace it |
| Section 03 compatibility execution module | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | primary expected host for selected terms normalization, envelope reuse, stage-order reuse, and pre-authority response/failure foundation | additive only; no rewrite of accepted Slice 2A, Slice 2B, Slice 2C, or Slice 2D paths |
| onboarding compatibility request-shape and envelope-adjacent contract support | `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_kernel_contracts/src/common.rs`, `crates/selene_kernel_contracts/src/ph1onb.rs` | consume accepted contract truth for request-envelope, simulation-id seam, and ids | reference-first posture; do not widen or semantically drift shared contracts unless strictly necessary and lawfully justified |
| accepted session substrate consumption | `crates/selene_os/src/runtime_session_foundation.rs` | consume accepted session ids, attach outcomes, turn ordering, and access posture | not a Slice 2E redesign target |
| app-layer onboarding truth anchors | `crates/selene_os/src/app_ingress.rs` | repo-truth reference only for action ordering, request-shape truth, and downstream sequencing | not a Slice 2E implementation target |
| downstream onboarding, governance, and persistence seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs`, downstream PH1.ONB execution paths | preserved downstream envelope and stage seams only | not a Slice 2E implementation target |
| Apple/client and app-specific ingress surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2E | no Apple work, no app work, no client workaround architecture |

I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2E must be:

1. preserve the accepted Slice 2A `/v1/voice/turn` path, accepted Slice 2B `/v1/invite/click` path, accepted Slice 2C `AskMissingSubmit` path, and accepted Slice 2D `PlatformSetupReceipt` path while extending `/v1/onboarding/continue` execution only for the selected `TermsAccept` action
2. admit the bounded selected request shape using `AppOnboardingContinueRequest` plus `AppOnboardingContinueAction::TermsAccept { terms_version_id, accepted }` field truth and canonical request-envelope fields, without inventing app/client production behavior
3. normalize the selected onboarding request into the accepted compatibility branch of the canonical carrier without changing the accepted voice-turn carrier, accepted invite-click carrier, accepted ask-missing carrier, or accepted platform-setup carrier
4. reuse accepted Slice 1B request security, replay-hook posture, idempotency propagation, and trigger validation foundations so selected onboarding entry remains fail closed
5. reuse accepted Slice 1C/1D session resolve-or-open discipline only to the bounded extent required to keep the route session-bound and canonical
6. reuse the canonical `RuntimeExecutionEnvelope` creation path and preserve every downstream Section 04/05/09/11 field as unset
7. reuse the accepted pre-authority stage-order scaffold by adding only the bounded compatibility-route classification needed for the selected terms action
8. implement deterministic pre-authority response/failure carriers, events, metrics, and trace propagation for the selected onboarding route
9. fail closed if selected input is malformed, if trigger posture drifts, if session binding is invalid, if envelope posture drifts, or if later runtime state is populated
10. stop the success path at the bounded pre-authority handoff; do not execute `ONB_TERMS_ACCEPT_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior
11. close the slice only when route-scope proof, request-shape proof, carrier-reuse proof, envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no client/app bleed proof, and accepted Slice 1A-1D plus Slice 2A/2B/2C/2D regression proof all pass on a clean tree

Slice 2E must not reverse this order. Route activation may not outrun the bounded action shape.
Compatibility normalization may not outrun accepted request security and trigger validation.
Envelope creation may not outrun lawful session discipline. No protected execution may be
entered before the slice is complete and a later Section 04 slice is explicitly opened.

J) verification and acceptance matrix

| proof area | required verification | Slice 2E acceptance condition |
|---|---|---|
| route-scope proof | prove that `/v1/onboarding/continue` becomes executable only for the already-accepted `AskMissingSubmit` and `PlatformSetupReceipt` actions plus the selected `TermsAccept` action, that `/v1/voice/turn` and `/v1/invite/click` remain the accepted canonical baselines, and that all later onboarding actions remain non-executable | Slice 2E opens exactly one new bounded onboarding compatibility action and nothing wider |
| selected request-shape proof | prove selected onboarding requests require the bounded `AppOnboardingContinueRequest` + `TermsAccept { terms_version_id, accepted }` field truth plus canonical request-envelope fields and reject malformed or incomplete inputs fail closed | the selected onboarding action enters the runtime with one deterministic shape only |
| executable-baseline proof | prove accepted `AskMissingSubmit` remains executable and accepted `PlatformSetupReceipt` remains executable after Slice 2E is added | Slice 2E may extend Section 03, but it may not regress accepted Slice 2C or Slice 2D truth |
| deferred-action proof | prove `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` remain non-executable | no later onboarding action is pulled forward |
| canonical carrier reuse proof | prove selected onboarding requests normalize into the accepted compatibility branch of the canonical carrier and do not alter the accepted voice-turn, invite-click, ask-missing, or platform-setup carriers | the selected action gains one bounded normalized path and nothing parallel |
| request-envelope and trigger proof | prove accepted Slice 1B request-envelope rules and accepted trigger law remain enforced for selected onboarding execution | no route-specific security or trigger shortcut exists |
| session discipline proof | prove Slice 2E consumes accepted Slice 1C/1D session truth to keep the selected onboarding route lawful and session-bound without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| execution-envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and that raw onboarding compatibility requests do not survive past the envelope boundary | there is still one canonical runtime envelope path |
| stage-order proof | prove the selected onboarding compatibility path records deterministic stage order and stops at the pre-authority boundary | the selected route remains inside Section 03 and cannot skip or reorder gates |
| fail-closed proof | prove invalid selected request shape, invalid trigger posture, invalid session posture, invalid stage progression, and invalid envelope posture all reject immediately with deterministic failure class | no failed onboarding compatibility request can drift into later stages |
| pre-authority stop proof | prove the successful selected onboarding path stops at a bounded pre-authority handoff and does not execute `ONB_TERMS_ACCEPT_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior | Slice 2E stays upstream of protected execution and persistence |
| observability proof | prove the selected route emits deterministic events, counters, metrics, and trace propagation for every Slice 2E-owned stage | onboarding compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no identity verification, governance proof execution, onboarding authority decision, simulation execution, or protected business execution in Slice 2E | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable replay protection, duplicate-outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2E | Section 05 remains downstream and intact |
| no PH1.J / GOV / LAW bleed proof | prove Slice 2E preserves hook surfaces only and does not execute proof, governance, or runtime-law behavior | later runtime law surfaces remain downstream |
| no client/app bleed proof | prove there is no Apple/client behavior, no app-open or app-owned onboarding production behavior, and no app-specific workaround architecture introduced by Slice 2E | clients remain downstream thin terminals |
| accepted Slice 1A-1D and Slice 2A/2B/2C/2D regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2E code state | Slice 2E must not break the accepted runtime, session, or accepted Slice 2A/2B/2C/2D foundations |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2E is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/onboarding/continue` `TermsAccept` compatibility execution foundation | `IN_SCOPE_SLICE_2E` | this is the only new executable Section 03 onboarding action in Slice 2E |
| preserve accepted `/v1/onboarding/continue` `AskMissingSubmit`, accepted `/v1/onboarding/continue` `PlatformSetupReceipt`, accepted `/v1/voice/turn`, accepted `/v1/invite/click`, accepted canonical carrier, and accepted pre-authority handoff behavior | `IN_SCOPE_SLICE_2E` | Slice 2E may extend Section 03, but it may not reinterpret or replace accepted Slice 2A, Slice 2B, Slice 2C, or Slice 2D truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2E` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2E` | Slice 2E must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2E` | Slice 2E must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| `PrimaryDeviceConfirm` compatibility execution | `DEFER_LATER_SECTION03` | do not pull `PrimaryDeviceConfirm` into Slice 2E |
| `VoiceEnrollLock`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, and later onboarding compatibility execution | `DEFER_LATER_SECTION03` | do not widen Slice 2E into downstream onboarding completion work |
| combined remaining onboarding-continue execution bucket | `DEFER_LATER_SECTION03` | do not widen Slice 2E into a multi-action remaining-onboarding bucket |
| deeper modality-specific payload behavior beyond accepted Slice 2A normalization | `DEFER_LATER_SECTION03` | do not embed voice/file/image/camera-specific business logic into Slice 2E |
| narrower micro-slice not explicitly supported by repo truth | `OUT_OF_SCOPE_THIS_PHASE` | do not split `TermsAccept` into polarity-specific or version-specific execution slices |
| actual `ONB_TERMS_ACCEPT_COMMIT` execution | `DEFER_SECTION04` | no simulation dispatch, no authoritative terms mutation, and no committed result claims in Slice 2E |
| actual `PH1.ONB` execution beyond preserved seams | `DEFER_SECTION04` | no onboarding-state mutation, governance-protected action execution, or onboarding progression execution in Slice 2E |
| durable replay protection, duplicate outcome reuse, outbox, journal, reconcile, dedupe, and authoritative sync acknowledgement | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2E |
| PH1.J / proof execution, governance execution, and runtime-law execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve envelope seams only; do not build a hidden PH1.J / GOV / LAW path |
| memory behavior, personality/emotional behavior, PH1.COMP, and later runtime work | `DEFER_LATER_RUNTIME` | these later layers may consume Slice 2E outputs but may not be implemented here |
| Apple/client behavior, app-open behavior, app-specific onboarding behavior, and any client-local authority or retry workaround path | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no hidden Section 04 path | `IN_SCOPE_SLICE_2E` | Section 03 must not become a parallel authority layer |
| no hidden Section 05 path | `IN_SCOPE_SLICE_2E` | Section 03 must not become a parallel persistence layer |
| no incomplete completion claim | `IN_SCOPE_SLICE_2E` | Slice 2E may not be called complete while any in-scope item or mandatory proof remains uncaptured |

L) SLICE 2E COMPLETION STANDARD

Slice 2E is complete only when the next lawful onboarding-compatible expansion of Section 03
exists and is proven without bleeding into later sections. Completion requires:

- `/v1/onboarding/continue` remains live for the accepted `AskMissingSubmit` compatibility action
- `/v1/onboarding/continue` remains live for the accepted `PlatformSetupReceipt` compatibility
  action
- `/v1/onboarding/continue` becomes live for the selected `TermsAccept` compatibility action and
  nothing wider
- `/v1/voice/turn` remains the accepted canonical turn route unchanged
- `/v1/invite/click` remains the accepted canonical invite-click compatibility route unchanged
- selected onboarding terms requests validate against one bounded deterministic request shape
  anchored to `AppOnboardingContinueRequest` plus `TermsAccept { terms_version_id, accepted }`
  field truth and canonical request-envelope truth
- selected onboarding terms requests normalize into the accepted compatibility branch of the
  canonical carrier without altering the accepted voice-turn, invite-click, ask-missing, or
  platform-setup carriers
- accepted Slice 1B request security, accepted Slice 1C/1D session truth, accepted
  Slice 2A/2B/2C/2D envelope path, and accepted Slice 2A/2B/2C/2D pre-authority scaffold are
  all reused rather than replaced
- the selected route stops at a deterministic pre-authority handoff and returns deterministic
  success/refusal carriers
- `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`,
  `WakeEnrollCompleteCommit`, and every later onboarding action remain non-executable and
  explicitly deferred
- no `ONB_TERMS_ACCEPT_COMMIT`, `PH1.ONB`, Section 04, or Section 05 behavior exists in the
  slice
- no PH1.J / GOV / LAW execution behavior exists in the slice beyond preserved hook surfaces
- no memory, personality, emotional-runtime, or PH1.COMP behavior exists in the slice
- no client/app/Apple behavior exists in the slice
- accepted Slice 1A-1D and accepted Slice 2A/2B/2C/2D regressions all remain green
- the slice closes on a clean tree with no uncaptured in-scope item, no uncaptured deferred
  boundary, and no unproven completion claim

Slice 2E is not complete if raw `/v1/onboarding/continue` becomes a multi-action executable
bucket, if the accepted carrier or accepted envelope path is duplicated or bypassed, if
`PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, or
`WakeEnrollCompleteCommit` are pulled forward, if `ONB_TERMS_ACCEPT_COMMIT` or `PH1.ONB`
execution is embedded directly into the slice, or if any mandatory proof in `J)` is missing.

M) PHASE BOUNDARY

Slice 2E ends at the same Section 03 pre-authority boundary already established by accepted
Slice 2A, accepted Slice 2B, accepted Slice 2C, and accepted Slice 2D.

The selected `TermsAccept` action may:

- enter through the accepted canonical route family
- validate against bounded request-shape truth
- normalize into the accepted compatibility branch of the canonical carrier
- reuse the accepted Slice 1C/1D session discipline
- reuse the accepted `RuntimeExecutionEnvelope`
- record deterministic pre-authority classification, events, metrics, and failure posture
- stop at the bounded pre-authority handoff

The selected `TermsAccept` action may not:

- execute `ONB_TERMS_ACCEPT_COMMIT`
- execute `PH1.ONB`
- execute governance proof or protected authorization decisions
- execute Section 05 persistence or sync behavior
- create a second onboarding carrier
- create a second runtime envelope path
- create an alternate authority path
- widen into client/app/Apple behavior

After H7:

- accepted `AskMissingSubmit` remains the accepted first onboarding compatibility action
- accepted `PlatformSetupReceipt` remains the accepted second onboarding compatibility action
- selected `TermsAccept` becomes the accepted third onboarding compatibility action once its
  later implementation run is complete
- `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`,
  `WakeEnrollCompleteCommit`, and later onboarding actions remain deferred

H7 governs Slice 2E only.

The next lawful step after H7 is the bounded Slice 2E implementation run only.
