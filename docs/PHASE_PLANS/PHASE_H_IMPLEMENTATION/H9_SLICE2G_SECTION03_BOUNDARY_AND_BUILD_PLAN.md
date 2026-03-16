PHASE H9 — SLICE 2G SECTION 03 BOUNDARY AND BUILD PLAN

A) PURPOSE

H9 freezes exactly one next bounded Section 03 slice after accepted Slice 2F.

The required decision is no longer the broad post-`TermsAccept` branch split that H8 resolved.
The remaining next-slice ambiguity is narrower and is now between:

- `EmployeePhotoCaptureSend`
- `VoiceEnrollLock`

H9 must remove that ambiguity without widening into:

- the whole sender-verification family
- `EmployeeSenderVerifyCommit`
- wake-enroll execution
- deeper modality-specific `/v1/voice/turn` work
- Section 04 or Section 05 behavior
- Apple/client/app behavior

H9 therefore freezes one exact Slice 2G winner, keeps the loser deferred, and preserves one
canonical carrier, one accepted session seam, one canonical `RuntimeExecutionEnvelope` path,
and one deterministic pre-authority stop line.

B) FROZEN LAW INPUTS

H9 is bound by the same upstream law chain already frozen and accepted:

- H1 build-order law
- H2 accepted Slice 1 implementation law
- H3 frozen Slice 2A law
- H4 frozen Slice 2B law
- H5 frozen Slice 2C law
- H6 frozen Slice 2D law
- H7 frozen Slice 2E law
- H8 frozen Slice 2F law
- accepted Slice 1A / 1B / 1C / 1D runtime law
- accepted Slice 2A / 2B / 2C / 2D / 2E / 2F runtime law

Additional binding repo-truth anchors for this decision are:

- `AppOnboardingContinueAction` in `app_ingress.rs`
- post-`TermsAccept` next-step routing in `app_ingress.rs`
- post-`PrimaryDeviceConfirm` next-step routing in `app_ingress.rs`
- real sender-verification business seams in `app_ingress.rs`
- later wake-enroll seams in `app_ingress.rs`
- current accepted Section 03 executable set in `runtime_ingress_turn_foundation.rs`

The following laws remain unchanged and are binding here:

- one canonical normalized turn-start carrier
- one accepted session resolve-or-open seam
- one canonical `RuntimeExecutionEnvelope` path
- one deterministic pre-authority stop line
- no Section 04/05 widening
- no Apple/client widening
- no real `PH1.ONB` execution in Section 03

C) CURRENT / TARGET / GAP

CURRENT:

- accepted Slice 2F leaves `/v1/onboarding/continue` executable only for:
  - `AskMissingSubmit`
  - `PlatformSetupReceipt`
  - `TermsAccept`
  - `PrimaryDeviceConfirm`
- current repo truth still exposes the unresolved next-step competition after accepted Slice 2F:
  - `SenderVerification` can still be the next app-level branch after `PrimaryDeviceConfirm`
    when sender verification remains pending
  - `VoiceEnroll` can be the next app-level branch after `PrimaryDeviceConfirm` when sender
    verification is not pending
- inside the still-deferred `SenderVerification` umbrella, repo truth exposes two exact actions:
  - `EmployeePhotoCaptureSend`
  - `EmployeeSenderVerifyCommit`
- repo truth shows `EmployeePhotoCaptureSend` before `EmployeeSenderVerifyCommit`, while
  `VoiceEnrollLock` remains a separate later exact action on the alternative voice path

TARGET:

- freeze one exact next Section 03 slice:
  `EmployeePhotoCaptureSend` compatibility execution

GAP:

- H8 lawfully selected `PrimaryDeviceConfirm`, but it explicitly deferred the unresolved
  sender-verification branch and `VoiceEnrollLock`
- current repo truth shows `EmployeePhotoCaptureSend` and `VoiceEnrollLock` are the two real
  competing exact candidates after accepted Slice 2F
- without H9, Slice 2G implementation would still have to guess whether the next lawful bounded
  Section 03 slice is:
  - the first sender-verification micro-slice
  - the voice-path entry slice
  - or something wider than either exact action

D) SLICE 2G SELECTION DECISION

Slice 2G is:

- the canonical `/v1/onboarding/continue` `EmployeePhotoCaptureSend` compatibility execution
  foundation

This is the selected next bounded Section 03 slice.

Why this is the selected winner:

- `EmployeePhotoCaptureSend` is one exact `AppOnboardingContinueAction` with one exact bounded
  repo-truth request shape: `photo_blob_ref`
- it is the first exact action inside the still-deferred `SenderVerification` branch rather than
  an umbrella family label
- current repo truth shows `EmployeePhotoCaptureSend` occurs before
  `EmployeeSenderVerifyCommit`, so it is the earliest lawful sender-verification micro-slice
- selecting `EmployeePhotoCaptureSend` resolves the remaining branch ambiguity more directly than
  `VoiceEnrollLock`, because `VoiceEnrollLock` is only lawful when sender verification is not
  pending, while the pending sender-verification branch would otherwise remain unfrozen
- it can still be frozen as compatibility-only because Section 03 can admit, normalize,
  session-bind, envelope-bind, classify, observe, and stop before real
  `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` or broader `PH1.ONB` execution

Why `VoiceEnrollLock` does not win:

- `VoiceEnrollLock` is one exact bounded action, but repo truth places it only on the branch
  where sender verification is not pending after `PrimaryDeviceConfirm`
- choosing `VoiceEnrollLock` next would leave the still-open sender-verification branch unresolved
- H9 is required to resolve the remaining branch split, and `EmployeePhotoCaptureSend` is the
  first exact action on the still-unresolved competing branch

Why `EmployeeSenderVerifyCommit` does not win:

- repo truth exposes `EmployeeSenderVerifyCommit` as a later exact sender-verification action
- it follows `EmployeePhotoCaptureSend` rather than preceding it
- selecting it now would skip the earlier micro-slice and would not match actual app ordering

What Slice 2G means precisely:

- `/v1/onboarding/continue` remains executable for accepted:
  - `AskMissingSubmit`
  - `PlatformSetupReceipt`
  - `TermsAccept`
  - `PrimaryDeviceConfirm`
- `/v1/onboarding/continue` becomes additionally executable only for
  `EmployeePhotoCaptureSend`
- the selected action must use the bounded `AppOnboardingContinueRequest` field truth:
  - `correlation_id`
  - `onboarding_session_id`
  - `idempotency_key`
  - optional `tenant_id`
  - `action = EmployeePhotoCaptureSend { photo_blob_ref }`
- the selected action must reuse the accepted canonical carrier, accepted session seam,
  canonical `RuntimeExecutionEnvelope`, and accepted pre-authority stop line
- the selected action must remain compatibility-only and must not execute real
  `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`, real `PH1.ONB`, Section 04 authority, or Section 05
  persistence/sync behavior

E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/onboarding/continue` `EmployeePhotoCaptureSend` compatibility execution foundation | `IN_SCOPE_SLICE_2G` | one exact action with one exact request shape; first exact action inside the remaining `SenderVerification` branch; occurs before `EmployeeSenderVerifyCommit` | selected | strongest exact next slice that resolves the remaining branch split while staying compatibility-only and pre-authority |
| canonical `/v1/onboarding/continue` `VoiceEnrollLock` compatibility execution foundation | `DEFER_LATER_SECTION03` | one exact action, but only lawful on the no-sender-verification-pending branch after accepted `PrimaryDeviceConfirm` | deferred | loses because it does not resolve the still-open competing sender-verification branch |
| canonical `/v1/onboarding/continue` `EmployeeSenderVerifyCommit` compatibility execution foundation | `DEFER_LATER_SECTION03` | exact sender-verification action, but later than `EmployeePhotoCaptureSend` in current repo truth | deferred | not the earliest lawful micro-slice on the unresolved branch |
| combined remaining `/v1/onboarding/continue` execution bucket | `DEFER_LATER_SECTION03` | would mix sender verification, voice entry, wake-enroll, and later completion actions into one bucket | deferred | too broad and recreates ambiguity rather than removing it |
| later onboarding actions such as `WakeEnrollStartDraft`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, and `WakeEnrollDeferCommit` | `DEFER_LATER_SECTION03` | repo truth places them after `VoiceEnrollLock` on later wake-enroll seams | deferred | clearly later than the next slice |
| deeper modality-specific `/v1/voice/turn` work | `DEFER_LATER_SECTION03` | separate later Section 03 track that does not resolve the remaining onboarding branch split | deferred | not the missing next-slice decision |

F) selected slice scope and dependency matrix

| item | classification | Slice 2G position | dependency / guardrail |
|---|---|---|---|
| canonical `/v1/onboarding/continue` `EmployeePhotoCaptureSend` compatibility execution foundation | `IN_SCOPE_SLICE_2G` | the only newly executable onboarding action in Slice 2G | add one bounded Section 03 action only |
| admitted request shape `AppOnboardingContinueRequest` plus `EmployeePhotoCaptureSend { photo_blob_ref }` | `IN_SCOPE_SLICE_2G` | exact bounded request basis for the selected slice | consume repo truth only; do not invent extra auth, actor, or user-id requirements |
| request frame fields `correlation_id`, `onboarding_session_id`, `idempotency_key`, and `tenant_id` | `IN_SCOPE_SLICE_2G` | preserve the accepted onboarding continuation frame | reuse accepted request-envelope discipline exactly |
| canonical normalized Section 03 carrier reuse | `IN_SCOPE_SLICE_2G` | keep one carrier path for all accepted onboarding compatibility actions | no second onboarding carrier and no route-family fork |
| accepted session resolve-or-open seam reuse | `IN_SCOPE_SLICE_2G` | keep the selected action bound to accepted Slice 1C/1D session truth | do not reinterpret session ids, ordering, ownership, or conflict law |
| canonical `RuntimeExecutionEnvelope` reuse | `IN_SCOPE_SLICE_2G` | keep one runtime execution path after normalization | raw compatibility requests must not survive past the envelope boundary |
| deterministic pre-authority stage order / stop line reuse | `IN_SCOPE_SLICE_2G` | keep the selected action inside the accepted Section 03 stage model | success must still terminate at the same pre-authority handoff |
| real `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` execution | `DEFER_SECTION04` | downstream protected sender-verification execution seam | no real `PH1.ONB` mutation in Slice 2G |
| real `PH1.ONB` execution beyond preserved seams | `DEFER_SECTION04` | downstream protected onboarding engine | no protected onboarding execution in Slice 2G |
| persistence correctness, sync, reconcile, dedupe, and authoritative acknowledgement | `DEFER_SECTION05` | later distributed correctness layer | do not populate persistence outcomes on success |
| `VoiceEnrollLock` | `DEFER_LATER_SECTION03` | non-selected competing branch | must remain non-executable in Slice 2G |
| `EmployeeSenderVerifyCommit` | `DEFER_LATER_SECTION03` | later sender-verification micro-slice | must remain non-executable in Slice 2G |
| wake-enroll later actions | `DEFER_LATER_SECTION03` | later onboarding chain | must remain non-executable in Slice 2G |
| PH1.J / GOV / LAW execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | later runtime layers | preserve seams only; do not execute them here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | not a Section 03 runtime target | no Apple work, no app work, no client-owned execution truth |

G) request-shape / continuation-action matrix

| action or frame item | bounded repo-truth shape | classification | Slice 2G posture |
|---|---|---|---|
| `AppOnboardingContinueRequest.correlation_id` | required request frame field | `IN_SCOPE_SLICE_2G` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.onboarding_session_id` | required request frame field | `IN_SCOPE_SLICE_2G` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.idempotency_key` | required request frame field | `IN_SCOPE_SLICE_2G` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.tenant_id` | required optional request frame field | `IN_SCOPE_SLICE_2G` | preserve accepted continuation frame |
| `AskMissingSubmit` | already accepted executable action | `IN_SCOPE_SLICE_2G` | must remain executable |
| `PlatformSetupReceipt` | already accepted executable action | `IN_SCOPE_SLICE_2G` | must remain executable |
| `TermsAccept` | already accepted executable action | `IN_SCOPE_SLICE_2G` | must remain executable |
| `PrimaryDeviceConfirm` | already accepted executable action | `IN_SCOPE_SLICE_2G` | must remain executable |
| `EmployeePhotoCaptureSend` | `photo_blob_ref` | `IN_SCOPE_SLICE_2G` | newly executable selected winner |
| `SenderVerification` | umbrella next-step label, not one exact action | `OUT_OF_SCOPE_THIS_PHASE` | must not be implemented as a bucket |
| `EmployeeSenderVerifyCommit` | `decision` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `VoiceEnrollLock` | `device_id`, `sample_seed` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `WakeEnrollSampleCommit` | `device_id`, `sample_pass` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `WakeEnrollCompleteCommit` | `device_id` | `DEFER_LATER_SECTION03` | must remain non-executable |

H) repository workstream / file-impact matrix

| repository seam or file | role in Slice 2G | classification | implementation expectation |
|---|---|---|---|
| `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | primary Section 03 ingress foundation for the selected compatibility action | `IN_SCOPE_SLICE_2G` | expected primary implementation file |
| `crates/selene_os/src/app_ingress.rs` | repo-truth source for request shape, branch order, and downstream sender-verification / voice seams | `OUT_OF_SCOPE_THIS_PHASE` | read as truth anchor only; do not modify for Slice 2G |
| `crates/selene_kernel_contracts/src/ph1onb.rs` | downstream protected sender-verification contract truth | `DEFER_SECTION04` | contract seam remains downstream and untouched |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` | canonical `RuntimeExecutionEnvelope` truth | `IN_SCOPE_SLICE_2G` as protected baseline | reuse only; do not fork envelope behavior |
| accepted Slice 1A-1D and Slice 2A-2F runtime foundation files | protected upstream runtime law | `OUT_OF_SCOPE_THIS_PHASE` | preserve behavior and prove regressions stay green |

I) INTERNAL IMPLEMENTATION ORDER

Slice 2G implementation, when later opened, must follow this order:

1. preserve the accepted executable onboarding set exactly as-is:
   `AskMissingSubmit`, `PlatformSetupReceipt`, `TermsAccept`, and `PrimaryDeviceConfirm`
2. extend `/v1/onboarding/continue` admission by exactly one newly executable action:
   `EmployeePhotoCaptureSend`
3. admit only the bounded repo-truth shape:
   `AppOnboardingContinueRequest` plus
   `AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref }`
4. normalize the selected action into the existing canonical Section 03 onboarding carrier
   instead of creating a second sender-verification carrier
5. bind the selected action to the accepted Slice 1C/1D session resolve-or-open seam
6. reuse the canonical `RuntimeExecutionEnvelope` path and preserve all downstream Section 04,
   Section 05, Section 09, and Section 11 fields as unset
7. classify the selected action inside the existing deterministic pre-authority stage order and
   preserve the same stop line already accepted upstream
8. fail closed for malformed `EmployeePhotoCaptureSend` input and fail closed if any later
   runtime state appears populated on a nominal Section 03 success path
9. emit bounded Section 03 observability for the selected action
10. keep `EmployeeSenderVerifyCommit`, `VoiceEnrollLock`, wake-enroll actions, and all later
    runtime/client work non-executable
11. close the slice only when request-shape proof, carrier/envelope proof, stage-order proof,
    fail-closed proof, no Section 04/05 bleed proof, no voice-path bleed proof, and regression
    proof for accepted Slice 1A-1D and Slice 2A-2F all pass on a clean tree

Slice 2G must not reverse this order. Action admission may not outrun normalization.
Normalization may not outrun session binding. Envelope creation may not outrun lawful
session/turn posture. No protected execution may be entered before the slice is complete and a
later Section 04 slice is explicitly opened.

J) verification and acceptance matrix

| proof area | required verification | Slice 2G acceptance condition |
|---|---|---|
| selected winner proof | prove `EmployeePhotoCaptureSend` is the one newly executable onboarding action | Slice 2G opens exactly one new canonical onboarding compatibility action |
| accepted executable action regression proof | prove `AskMissingSubmit`, `PlatformSetupReceipt`, `TermsAccept`, and `PrimaryDeviceConfirm` remain executable | previously accepted Slice 2C / 2D / 2E / 2F truth is preserved |
| deferred action non-execution proof | prove `EmployeeSenderVerifyCommit`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` remain non-executable | the non-selected candidate and later actions stay deferred |
| request-shape proof | prove `EmployeePhotoCaptureSend` accepts only `photo_blob_ref` under the accepted continuation request frame and rejects malformed or incomplete input fail closed | the selected action has one deterministic compatibility shape only |
| carrier reuse proof | prove the selected action reuses the accepted canonical Section 03 carrier rather than introducing a second carrier path | there is still one canonical normalized Section 03 path |
| session seam proof | prove Slice 2G consumes the accepted Slice 1C/1D session resolve/open truth without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and raw compatibility requests do not survive past the envelope boundary | there is still one canonical runtime execution path |
| stage-order proof | prove `EmployeePhotoCaptureSend` records deterministic pre-authority stage order and stops at the same pre-authority boundary already accepted upstream | the selected action remains wholly inside Section 03 |
| fail-closed proof | prove malformed selected input, invalid stage posture, invalid envelope posture, and invalid later-runtime population all reject immediately with deterministic failure class | no failed or over-advanced request can drift into later stages |
| observability proof | prove the selected action emits bounded Section 03 events, counters, metrics, and trace propagation | compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no real `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`, no real `PH1.ONB`, and no protected authority execution in Slice 2G | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2G | Section 05 remains downstream and intact |
| no later-runtime bleed proof | prove there is no PH1.J / GOV / LAW execution beyond hook surfaces and no memory / personality / PH1.COMP behavior | later runtime remains downstream and intact |
| accepted regression proof | rerun accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2G code state | Slice 2G must not break accepted Slice 1A-1D or Slice 2A-2F foundations |
| clean-tree acceptance closeout | prove formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2G is not complete until proof and cleanliness both pass |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/onboarding/continue` `EmployeePhotoCaptureSend` compatibility execution foundation | `IN_SCOPE_SLICE_2G` | this is the only newly executable onboarding action in Slice 2G |
| preserve accepted `AskMissingSubmit`, `PlatformSetupReceipt`, `TermsAccept`, and `PrimaryDeviceConfirm` behavior | `IN_SCOPE_SLICE_2G` | Slice 2G may extend Section 03, but it may not reinterpret or replace accepted Slice 2C / 2D / 2E / 2F truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2G` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2G` | Slice 2G must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2G` | Slice 2G must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| `VoiceEnrollLock` | `DEFER_LATER_SECTION03` | do not pull the competing voice-path candidate into Slice 2G |
| `EmployeeSenderVerifyCommit` | `DEFER_LATER_SECTION03` | do not widen Slice 2G into the later sender-verification micro-slice |
| `SenderVerification` family as a bucket | `DEFER_LATER_SECTION03` | do not implement the umbrella branch as a multi-action executable family |
| `WakeEnrollStartDraft`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, and `WakeEnrollDeferCommit` | `DEFER_LATER_SECTION03` | later onboarding actions remain out of scope |
| combined remaining onboarding execution | `DEFER_LATER_SECTION03` | do not widen Slice 2G into a remaining-bucket implementation |
| deeper modality-specific `/v1/voice/turn` work | `DEFER_LATER_SECTION03` | do not reopen the separate later Section 03 voice track here |
| real `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` and real `PH1.ONB` execution | `DEFER_SECTION04` | no protected onboarding execution or state mutation in Slice 2G |
| Section 04 authority decisions | `DEFER_SECTION04` | no hidden authority path may appear inside Section 03 |
| Section 05 persistence / sync execution | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2G |
| PH1.J / GOV / LAW execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve downstream seams only; do not build a hidden governance or law path |
| memory, personality/emotional behavior, and `PH1.COMP` | `DEFER_LATER_RUNTIME` | later runtime layers may consume Section 03 outputs but may not be implemented here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no alternate authority path | `IN_SCOPE_SLICE_2G` | Section 03 must not become a parallel authority layer |

L) SLICE 2G COMPLETION STANDARD

Slice 2G is complete only when all of the following are true:

- `EmployeePhotoCaptureSend` is the one newly executable onboarding sub-action on
  `/v1/onboarding/continue`
- `AskMissingSubmit`, `PlatformSetupReceipt`, `TermsAccept`, and `PrimaryDeviceConfirm` remain
  executable exactly as already accepted
- `EmployeeSenderVerifyCommit`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and
  `WakeEnrollCompleteCommit` remain non-executable
- malformed `EmployeePhotoCaptureSend` input fails closed
- the selected action reuses the existing canonical normalized Section 03 carrier
- the selected action reuses the accepted session resolve-or-open seam from Slice 1C/1D
- the selected action reuses the canonical `RuntimeExecutionEnvelope` path
- the deterministic pre-authority stage order and stop line remain the same as accepted
  Slice 2A-2F truth
- bounded Section 03 observability exists for the selected action
- no real `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT` execution appears
- no real `PH1.ONB` execution appears
- no Section 04 or Section 05 execution appears
- no PH1.J / GOV / LAW execution beyond hook surfaces appears
- no memory, personality, emotional-runtime, `PH1.COMP`, Apple/client, or app-layer workaround
  work appears
- the implementation footprint remains bounded and no widening beyond the proven file scope is
  assumed without proof
- all required validation, regression, and cleanliness gates pass on the exact closeout commit

Slice 2G is not complete if the implementation widens into `VoiceEnrollLock`,
`EmployeeSenderVerifyCommit`, wake-enroll work, app/client behavior, or any protected execution
path.

M) PHASE BOUNDARY

H9 governs Slice 2G only.

H9 freezes exactly one next bounded Section 03 slice:

- canonical `/v1/onboarding/continue` `EmployeePhotoCaptureSend` compatibility execution
  foundation

H9 does not authorize:

- `VoiceEnrollLock`
- `EmployeeSenderVerifyCommit`
- `SenderVerification` as a bucket
- `WakeEnrollStartDraft`
- `WakeEnrollSampleCommit`
- `WakeEnrollCompleteCommit`
- `WakeEnrollDeferCommit`
- deeper modality-specific `/v1/voice/turn` work
- real `PH1.ONB`
- real `ONB_EMPLOYEE_PHOTO_CAPTURE_SEND_COMMIT`
- Section 04 authority execution
- Section 05 persistence / sync execution
- Apple/client/app work
- H10 or any later planning document

PHASE BOUNDARY:

- one canonical carrier
- one accepted session seam
- one canonical `RuntimeExecutionEnvelope` path
- one pre-authority stop line
- one newly executable bounded action only

The next lawful step after H9 is:

- the bounded Slice 2G implementation only
