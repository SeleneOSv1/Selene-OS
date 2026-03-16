PHASE H8 — SLICE 2F SECTION 03 BOUNDARY AND BUILD PLAN


A) PURPOSE

H8 resolves the post-Slice 2E Section 03 branch split that current repo truth leaves unfrozen.

H8 exists because accepted Slice 2E opened `TermsAccept`, but the next lawful Section 03 slice is not uniquely explicit from H7 alone. Current repo truth shows two materially different post-`TermsAccept` candidates:

- `PrimaryDeviceConfirm`
- sender-verification onboarding work (`EmployeePhotoCaptureSend` / `EmployeeSenderVerifyCommit`)

H8 therefore does one thing only:

- compare the strongest remaining post-`TermsAccept` Section 03 candidates
- select exactly one bounded Slice 2F winner
- explicitly defer the non-selected branch and all later Section 03 work
- freeze the implementation order, verification law, and phase boundary for the selected winner

H8 is planning law only. It does not authorize runtime code changes outside the future bounded Slice 2F implementation run, it does not reopen frozen H1/H2/H3/H4/H5/H6/H7 law, and it does not authorize Section 04, Section 05, PH1.ONB execution, Apple, app/client, or deeper modality implementation.


B) FROZEN LAW INPUTS

Slice 2F is derived from the following binding inputs:

- frozen A1-A6, B1-B5, C1-C5, D1-D5, E1-E5, F1-F5, and G1-G2 as upstream design-law constraints
- [H1](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H1_BUILD_METHOD_AND_IMPLEMENTATION_SEQUENCE.md) as the sequencing-law anchor
- [H2](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H2_SLICE1_RUNTIME_AND_SESSION_BUILD_PLAN.md) as the accepted Slice 1 implementation-law anchor
- [H3](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H3_SLICE2A_CANONICAL_INGRESS_AND_TURN_BUILD_PLAN.md) as the frozen Slice 2A implementation-law anchor
- [H4](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H4_SLICE2B_SECTION03_BOUNDARY_AND_BUILD_PLAN.md) as the frozen Slice 2B implementation-law anchor
- [H5](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H5_SLICE2C_SECTION03_BOUNDARY_AND_BUILD_PLAN.md) as the frozen Slice 2C implementation-law anchor
- [H6](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H6_SLICE2D_SECTION03_BOUNDARY_AND_BUILD_PLAN.md) as the frozen Slice 2D implementation-law anchor
- [H7](docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H7_SLICE2E_SECTION03_BOUNDARY_AND_BUILD_PLAN.md) as the frozen Slice 2E implementation-law anchor
- [Build Section 01](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md), [Build Section 02](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md), and [Build Section 03](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md) as the main governing implementation sections
- [Build Section 04](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md), [Build Section 05](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md), [Build Section 09](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md), and [Build Section 11](docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md) as downstream dependency law that Slice 2F must preserve without pulling forward
- [CORE_ARCHITECTURE](docs/CORE_ARCHITECTURE.md), [SELENE_BUILD_EXECUTION_ORDER](docs/SELENE_BUILD_EXECUTION_ORDER.md), [SELENE_AUTHORITATIVE_ENGINE_INVENTORY](docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md), [COVERAGE_MATRIX](docs/COVERAGE_MATRIX.md), [11_DESIGN_LOCK_SEQUENCE](docs/11_DESIGN_LOCK_SEQUENCE.md), and [02_BUILD_PLAN](docs/02_BUILD_PLAN.md) as architecture, sequencing, and readiness truth
- [PHASE_F_FREEZE_SUMMARY](docs/PHASE_PLANS/PHASE_F_IPHONE/PHASE_F_FREEZE_SUMMARY.md) and [PHASE_G_APPLE_FREEZE_SUMMARY](docs/PHASE_PLANS/PHASE_G_APPLE/PHASE_G_APPLE_FREEZE_SUMMARY.md) as frozen Apple / app / client boundary law that Slice 2F must not reopen

Accepted implementation baselines consumed by this plan:

- Slice 1A accepted at `f9769797c28bc991df3720d085639a7117b3d7c8`
- Slice 1B accepted at `eb4be3fdfc100fa22684293cacee471faf7d7847`
- Slice 1C accepted at `b1eba355e716887f0fe399cc6930988e0423e7db`
- Slice 1D accepted at `46b021dece36b9c1d8589362cf0ada0187603a83`
- Slice 2A accepted at `743ea0fe3e2ef884efb1c28bec706fe4efab91c9`
- Slice 2B accepted at `cd340ea14cfb2c8b7c15fc0cb578daf1e8e168fe`
- Slice 2C accepted at `2bf95ea098082747f53c1df575fe350385df9fc1`
- Slice 2D accepted at `1ba64a5ebbfa71a8123e33c5881fb70042aabd85`
- Slice 2E accepted at `172c10b86a0f116e4dbaa4fa4c08b1998e09bafd`

These accepted slices establish non-negotiable repo truth for H8:

- the runtime bootstrap, routing, request-security, admission, observability, session, ownership, transfer, access, and backpressure substrate already exists and must be consumed rather than reimplemented
- accepted Slice 2A already established one canonical Section 03 carrier family, one canonical `RuntimeExecutionEnvelope` path, and one bounded pre-authority stop line
- accepted Slice 2B, Slice 2C, Slice 2D, and Slice 2E already opened the lawful Section 03 compatibility path for `/v1/invite/click`, `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept`
- accepted Slice 2E leaves `/v1/onboarding/continue` executable only for `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept`
- [app_ingress.rs](crates/selene_os/src/app_ingress.rs) already proves that real post-`TermsAccept` behavior fans out toward `SenderVerification` or `PrimaryDeviceConfirm`, and that both branches terminate in real `PH1.ONB` commit seams

Sections 04 and 05 remain downstream dependencies only. They define the protected execution and persistence boundaries that Slice 2F must preserve, not execution work that Slice 2F may implement.


C) CURRENT / TARGET / GAP

CURRENT

- Accepted Slice 2E leaves `/v1/onboarding/continue` executable for `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept` only.
- Current repo truth still exposes later onboarding actions under the same route family, including `PrimaryDeviceConfirm`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, `WakeEnrollCompleteCommit`, `EmployeePhotoCaptureSend`, and `EmployeeSenderVerifyCommit`.
- Current repo truth also shows a real post-`TermsAccept` branch split:
  - `TermsAccept` can return `SenderVerification`
  - `TermsAccept` can return `PrimaryDeviceConfirm`
- The sender-verification branch is not one exact action in repo truth. It is at least two distinct `AppOnboardingContinueAction` variants with different request shapes and different downstream `PH1.ONB` commit seams.

TARGET

- Freeze exactly one bounded next Section 03 slice after accepted Slice 2E.
- Make Slice 2F explicit enough that later implementation can proceed without guessing.
- Preserve one canonical carrier path, one accepted session seam, one canonical `RuntimeExecutionEnvelope` path, and one deterministic pre-authority stop line.

GAP

- H7 correctly deferred the remaining onboarding actions, but it did not freeze how to resolve the post-`TermsAccept` branch split.
- Without H8, an implementer would have to guess between `PrimaryDeviceConfirm`, a sender-verification family slice, a narrower sender-verification micro-slice, or a wider combined remaining-onboarding bucket.
- The missing artifact is therefore not new architecture. The missing artifact is a binding next-slice decision with explicit scope, explicit deferrals, and explicit proof law.


D) SLICE 2F SELECTION DECISION

Slice 2F is:

- the canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation

This is the selected next bounded Section 03 slice.

Why this is the selected winner:

- `PrimaryDeviceConfirm` is one exact `AppOnboardingContinueAction` with one bounded request shape in repo truth: `device_id` plus `proof_ok`.
- It is the strongest remaining post-`TermsAccept` candidate that can still be frozen as one bounded compatibility-only Section 03 slice.
- It aligns with actual app-layer branch truth:
  - `TermsAccept` can flow directly to `PrimaryDeviceConfirm`
  - `EmployeeSenderVerifyCommit` can also flow forward to `PrimaryDeviceConfirm`
- That means `PrimaryDeviceConfirm` is not merely parallel to sender verification; it is also the reconvergence point after sender verification confirms successfully.
- The real app-layer implementation for `PrimaryDeviceConfirm` is materially downstream because it already invokes proof-governance posture and `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`. That makes it appropriate for a bounded pre-authority compatibility freeze: Section 03 can admit and normalize the request without executing the protected path.

Why sender verification is not the selected winner:

- Repo truth does not expose sender verification as one exact action. It exposes at least two distinct actions:
  - `EmployeePhotoCaptureSend`
  - `EmployeeSenderVerifyCommit`
- Those two actions have different request shapes and different downstream `PH1.ONB` simulation ids.
- `TermsAccept` returns the umbrella next step `SenderVerification`, not one exact next action. Freezing the whole sender-verification branch would therefore widen Slice 2F beyond one bounded action. Freezing one narrower sender-verification action would require a second branch-selection decision that repo truth does not uniquely resolve here.

Why the later candidates remain deferred:

- `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` are downstream of the unresolved post-`TermsAccept` work and are therefore clearly later than the next slice.
- Combined remaining onboarding execution is broader than necessary and would collapse multiple different downstream `PH1.ONB` seams into one ambiguous bucket.
- Deeper modality-specific `/v1/voice/turn` work does not resolve the branch split that currently blocks Section 03 sequencing.

What Slice 2F means precisely:

- `/v1/onboarding/continue` gains one additional newly executable compatibility action: `PrimaryDeviceConfirm`.
- The route must admit exactly the bounded request-shape truth from repo code: `AppOnboardingContinueRequest` plus `AppOnboardingContinueAction::PrimaryDeviceConfirm { device_id, proof_ok }`.
- The slice must reuse the accepted canonical Section 03 carrier, the accepted session resolve-or-open seam, the canonical `RuntimeExecutionEnvelope` path, and the same deterministic pre-authority stop line already accepted upstream.
- The slice must remain compatibility-only and must not execute `govern_protected_action_proof`, `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`, real `PH1.ONB`, Section 04 authority, or Section 05 persistence/sync behavior.


E) candidate-scope comparison matrix

| candidate next slice | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `IN_SCOPE_SLICE_2F` | one exact action with one exact request shape; direct post-`TermsAccept` branch when sender verification is not pending; reconvergence target after successful sender verification | selected | strongest single bounded post-`TermsAccept` action that can stay compatibility-only and pre-authority |
| sender-verification onboarding sub-slice (`EmployeePhotoCaptureSend` plus `EmployeeSenderVerifyCommit`) | `DEFER_LATER_SECTION03` | real post-`TermsAccept` branch exists, but it is a family of at least two actions with different request shapes and distinct `PH1.ONB` seams | deferred | too wide to freeze as one next slice |
| narrower sender-verification micro-slice (`EmployeePhotoCaptureSend` only or `EmployeeSenderVerifyCommit` only) | `DEFER_LATER_SECTION03` | repo truth exposes the family, but H8 does not have a uniquely frozen repo-truth basis for which exact sender-verification action should land first | deferred | would require a second split decision that is not yet frozen |
| combined remaining `/v1/onboarding/continue` execution bucket | `DEFER_LATER_SECTION03` | would mix `PrimaryDeviceConfirm`, sender verification, and later wake/voice-enroll behavior into one bucket | deferred | broader than necessary and not lawfully bounded |
| `VoiceEnrollLock` compatibility execution | `DEFER_LATER_SECTION03` | downstream of `PrimaryDeviceConfirm` and/or sender verification in current repo truth | deferred | clearly later than the next slice |
| `WakeEnrollSampleCommit` compatibility execution | `DEFER_LATER_SECTION03` | later wake-enroll action with different downstream seams and later onboarding dependency posture | deferred | clearly later than the next slice |
| `WakeEnrollCompleteCommit` compatibility execution | `DEFER_LATER_SECTION03` | later wake-enroll completion action with later dependency posture | deferred | clearly later than the next slice |
| deeper modality-specific `/v1/voice/turn` behavior | `DEFER_LATER_SECTION03` | does not resolve the post-`TermsAccept` branch split and belongs to a separate later Section 03 track | deferred | not the missing next-slice decision |


F) selected slice scope and dependency matrix

| item | classification | Slice 2F position | dependency / guardrail |
|---|---|---|---|
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `IN_SCOPE_SLICE_2F` | the only newly executable onboarding action in Slice 2F | add one bounded Section 03 action only; do not widen beyond the selected winner |
| admitted request shape `AppOnboardingContinueRequest` plus `PrimaryDeviceConfirm { device_id, proof_ok }` | `IN_SCOPE_SLICE_2F` | exact bounded action basis for the selected slice | consume repo truth only; do not invent extra auth, actor, or user-id requirements |
| request frame fields `correlation_id`, `onboarding_session_id`, `idempotency_key`, and `tenant_id` | `IN_SCOPE_SLICE_2F` | preserve the accepted onboarding continuation frame | reuse the already accepted Section 03 request envelope discipline |
| canonical normalized Section 03 carrier reuse | `IN_SCOPE_SLICE_2F` | keep one carrier path for all accepted onboarding compatibility actions | do not create a second carrier or a parallel route family |
| accepted session resolve-or-open seam reuse | `IN_SCOPE_SLICE_2F` | keep the selected action bound to the accepted Slice 1C/1D session substrate | do not reinterpret session ids, turn ids, attach outcomes, or ordering law |
| canonical `RuntimeExecutionEnvelope` reuse | `IN_SCOPE_SLICE_2F` | keep one runtime execution path after normalization | raw requests must not survive past the envelope boundary |
| deterministic pre-authority stage order / stop line reuse | `IN_SCOPE_SLICE_2F` | keep the selected action inside the accepted Section 03 stage model | success must still terminate at the same pre-authority handoff |
| real proof-governance execution for `PrimaryDeviceConfirm` | `DEFER_LATER_RUNTIME` | downstream protected proof posture exists in repo truth | no `govern_protected_action_proof` execution in Slice 2F |
| real `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT` execution | `DEFER_SECTION04` | downstream protected onboarding execution seam | no `PH1.ONB` state mutation in Slice 2F |
| sender-verification onboarding branch (`EmployeePhotoCaptureSend` / `EmployeeSenderVerifyCommit`) | `DEFER_LATER_SECTION03` | non-selected competing branch | remain non-executable in Slice 2F |
| `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` | `DEFER_LATER_SECTION03` | later onboarding actions | remain non-executable in Slice 2F |
| persistence correctness, sync, reconcile, dedupe, and authoritative acknowledgement | `DEFER_SECTION05` | later distributed correctness layer | do not populate persistence outcomes or sync state on success |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | not a Section 03 runtime implementation target | no app-specific workaround architecture and no Apple widening |


G) request-shape / continuation-action matrix

| action or frame item | bounded repo-truth shape | classification | Slice 2F posture |
|---|---|---|---|
| `AppOnboardingContinueRequest.correlation_id` | required request frame field | `IN_SCOPE_SLICE_2F` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.onboarding_session_id` | required request frame field | `IN_SCOPE_SLICE_2F` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.idempotency_key` | required request frame field | `IN_SCOPE_SLICE_2F` | preserve accepted continuation frame |
| `AppOnboardingContinueRequest.tenant_id` | required request frame field | `IN_SCOPE_SLICE_2F` | preserve accepted continuation frame |
| `AskMissingSubmit` | already accepted executable action | `IN_SCOPE_SLICE_2F` | must remain executable |
| `PlatformSetupReceipt` | already accepted executable action | `IN_SCOPE_SLICE_2F` | must remain executable |
| `TermsAccept` | already accepted executable action | `IN_SCOPE_SLICE_2F` | must remain executable |
| `PrimaryDeviceConfirm` | `device_id`, `proof_ok` | `IN_SCOPE_SLICE_2F` | newly executable selected winner |
| `EmployeePhotoCaptureSend` | `photo_blob_ref` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `EmployeeSenderVerifyCommit` | `decision` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `VoiceEnrollLock` | `device_id`, `sample_seed` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `WakeEnrollSampleCommit` | `device_id`, `sample_pass` | `DEFER_LATER_SECTION03` | must remain non-executable |
| `WakeEnrollCompleteCommit` | `device_id` | `DEFER_LATER_SECTION03` | must remain non-executable |


H) repository workstream / file-impact matrix

| workstream | likely repository paths | Slice 2F role | file-impact law |
|---|---|---|---|
| primary Section 03 implementation seam | `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | primary target for admitting `PrimaryDeviceConfirm`, normalizing it into the accepted carrier, reusing session/envelope/stage law, and stopping pre-authority | this is the expected first and primary file; widening beyond it is not assumed or allowed without proof |
| accepted Section 01/02 baselines | `crates/selene_os/src/runtime_bootstrap.rs`, `crates/selene_os/src/runtime_request_foundation.rs`, `crates/selene_os/src/runtime_session_foundation.rs` | consumed baseline only | not expected Slice 2F edit targets unless strict proof shows a bounded integration requirement |
| contract and repo-truth anchors | `crates/selene_kernel_contracts/src/runtime_execution.rs`, `crates/selene_kernel_contracts/src/ph1onb.rs`, `crates/selene_kernel_contracts/src/ph1link.rs`, `crates/selene_os/src/app_ingress.rs` | anchor the selected request shape, downstream seams, and envelope law | repo-truth references only; not planned implementation targets |
| downstream governance, law, and persistence seams | `crates/selene_os/src/runtime_governance.rs`, `crates/selene_os/src/runtime_law.rs`, `crates/selene_storage/src/ph1f.rs` | preserved downstream seams only | not a Slice 2F implementation target |
| Apple / app / client surfaces | `crates/selene_os/src/app_ingress.rs` and all Apple/client implementation paths | explicit non-target for Slice 2F | no app work, no Apple work, no client workaround architecture |


I) INTERNAL IMPLEMENTATION ORDER

Implementation order inside Slice 2F must be:

1. admit only the bounded `PrimaryDeviceConfirm` request shape on `/v1/onboarding/continue` while preserving `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept` exactly as accepted
2. normalize `PrimaryDeviceConfirm` into the existing canonical Section 03 carrier rather than creating any new route-local or action-local execution carrier
3. reuse the accepted Slice 1B request-security, request-envelope, idempotency propagation, and fail-closed admission foundations
4. reuse the accepted Slice 1C/1D session resolve-or-open seam and lawful turn/session binding without reinterpreting session law
5. reuse the canonical `RuntimeExecutionEnvelope` creation path and preserve all downstream Section 04, Section 05, Section 09, and Section 11 fields as unset
6. classify the selected action inside the existing deterministic pre-authority stage order and preserve the same stop line already accepted upstream
7. fail closed for malformed `PrimaryDeviceConfirm` input and fail closed if any later-runtime state appears populated on a nominal Section 03 success path
8. emit bounded Section 03 observability for the selected action
9. keep sender-verification actions and later onboarding actions non-executable
10. close the slice only when request-shape proof, carrier/envelope proof, stage-order proof, fail-closed proof, no Section 04/05 bleed proof, no sender-verification bleed proof, and regression proof for accepted Slice 1A-1D and Slice 2A-2E all pass on a clean tree

Slice 2F must not reverse this order. Action admission may not outrun normalization. Normalization may not outrun session binding. Envelope creation may not outrun lawful session/turn posture. No protected execution may be entered before the slice is complete and a later Section 04 slice is explicitly opened.


J) verification and acceptance matrix

| proof area | required verification | Slice 2F acceptance condition |
|---|---|---|
| selected winner proof | prove `PrimaryDeviceConfirm` is the one newly executable onboarding action | Slice 2F opens exactly one new canonical onboarding compatibility action |
| accepted executable action regression proof | prove `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept` remain executable | previously accepted Slice 2C / 2D / 2E truth is preserved |
| deferred action non-execution proof | prove `EmployeePhotoCaptureSend`, `EmployeeSenderVerifyCommit`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` remain non-executable | the competing branch and later actions stay deferred |
| request-shape proof | prove `PrimaryDeviceConfirm` accepts only `device_id` plus `proof_ok` under the accepted continuation request frame and rejects malformed or incomplete inputs fail closed | the selected action has one deterministic compatibility shape only |
| carrier reuse proof | prove the selected action reuses the accepted canonical Section 03 carrier rather than introducing a second carrier path | there is still one canonical normalized Section 03 path |
| session seam proof | prove Slice 2F consumes the accepted Slice 1C/1D session resolve/open truth without reinterpreting session law | Section 03 remains downstream of the accepted session engine |
| envelope reuse proof | prove the canonical `RuntimeExecutionEnvelope` path is reused and raw compatibility requests do not survive past the envelope boundary | there is still one canonical runtime execution path |
| stage-order proof | prove `PrimaryDeviceConfirm` records deterministic pre-authority stage order and stops at the same pre-authority boundary already accepted upstream | the selected action remains wholly inside Section 03 |
| fail-closed proof | prove malformed `PrimaryDeviceConfirm` input, invalid stage posture, invalid envelope posture, and invalid later-runtime population all reject immediately with deterministic failure class | no failed or over-advanced request can drift into later stages |
| observability proof | prove the selected action emits bounded Section 03 events, counters, metrics, and trace propagation | compatibility ingress behavior is auditable without app-side reconstruction |
| no Section 04 bleed proof | prove there is no proof-governance execution, no real `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT`, no real `PH1.ONB`, and no protected authority execution in Slice 2F | Section 04 remains downstream and intact |
| no Section 05 bleed proof | prove there is no durable outcome reuse, outbox, journal, reconcile, dedupe, or authoritative sync acknowledgement in Slice 2F | Section 05 remains downstream and intact |
| no later-runtime bleed proof | prove there is no PH1.J / GOV / LAW execution beyond hook surfaces and no memory / personality / PH1.COMP behavior | later runtime remains downstream and intact |
| accepted regression proof | rerun the accepted runtime bootstrap, request foundation, session foundation, ingress-turn foundation, health, PH1.OS, and PH1.L regressions against the Slice 2F code state | Slice 2F must not break accepted Slice 1A-1D or Slice 2A-2E foundations |
| clean-tree acceptance closeout | prove tests, formatting, lint posture for touched files, design-readiness evidence, and clean tree state on the exact closeout commit | Slice 2F is not complete until proof and cleanliness both pass |


K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation | `IN_SCOPE_SLICE_2F` | this is the only newly executable onboarding action in Slice 2F |
| preserve accepted `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept` behavior | `IN_SCOPE_SLICE_2F` | Slice 2F may extend Section 03, but it may not reinterpret or replace accepted Slice 2C / 2D / 2E truth |
| no runtime-envelope bypass | `IN_SCOPE_SLICE_2F` | all post-normalization execution must flow through the canonical `RuntimeExecutionEnvelope` path |
| no session-law drift | `IN_SCOPE_SLICE_2F` | Slice 2F must consume accepted Slice 1C/1D session truth and must not reinterpret ids, ordering, ownership, access, or conflict law |
| no duplicate admission stack | `IN_SCOPE_SLICE_2F` | Slice 2F must consume accepted Slice 1B routing, security, and admission foundations rather than replacing them |
| sender-verification onboarding branch (`EmployeePhotoCaptureSend` / `EmployeeSenderVerifyCommit`) | `DEFER_LATER_SECTION03` | do not widen Slice 2F into the competing branch family |
| narrower sender-verification micro-slice not explicitly frozen here | `DEFER_LATER_SECTION03` | do not guess between `EmployeePhotoCaptureSend` and `EmployeeSenderVerifyCommit` in Slice 2F |
| `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` | `DEFER_LATER_SECTION03` | later onboarding actions remain out of scope |
| combined remaining onboarding execution | `DEFER_LATER_SECTION03` | do not widen Slice 2F into a remaining-bucket implementation |
| deeper modality-specific `/v1/voice/turn` work | `DEFER_LATER_SECTION03` | do not reopen a separate later Section 03 track here |
| real `ONB_PRIMARY_DEVICE_CONFIRM_COMMIT` and real `PH1.ONB` execution | `DEFER_SECTION04` | no protected onboarding execution or state mutation in Slice 2F |
| Section 04 authority decisions | `DEFER_SECTION04` | no hidden authority path may appear inside Section 03 |
| Section 05 persistence / sync execution | `DEFER_SECTION05` | do not implement persistence correctness inside Slice 2F |
| PH1.J / GOV / LAW execution beyond hook surfaces | `DEFER_LATER_RUNTIME` | preserve downstream seams only; do not build a hidden governance or law path |
| `LINK_INVITE_DRAFT_UPDATE_COMMIT` and previously accepted downstream link seams | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen prior link-slice downstream seams while building Slice 2F |
| memory, personality/emotional behavior, and `PH1.COMP` | `DEFER_LATER_RUNTIME` | later runtime layers may consume Section 03 outputs but may not be implemented here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | no Apple work, no app work, no client-owned execution truth |
| no alternate authority path | `IN_SCOPE_SLICE_2F` | Section 03 must not become a parallel authority layer |


L) SLICE 2F COMPLETION STANDARD

Slice 2F is complete only when all of the following are true:

- `PrimaryDeviceConfirm` is the one newly executable onboarding sub-action on `/v1/onboarding/continue`
- `AskMissingSubmit`, `PlatformSetupReceipt`, and `TermsAccept` remain executable exactly as already accepted
- `EmployeePhotoCaptureSend`, `EmployeeSenderVerifyCommit`, `VoiceEnrollLock`, `WakeEnrollSampleCommit`, and `WakeEnrollCompleteCommit` remain non-executable
- malformed `PrimaryDeviceConfirm` input fails closed
- the selected action reuses the existing canonical normalized Section 03 carrier
- the selected action reuses the accepted session resolve-or-open seam from Slice 1C/1D
- the selected action reuses the canonical `RuntimeExecutionEnvelope` path
- the deterministic pre-authority stage order and stop line remain the same as accepted Slice 2A-2E truth
- bounded Section 03 observability exists for the selected action
- no real proof-governance execution appears
- no real `PH1.ONB` execution appears
- no Section 04 or Section 05 execution appears
- no PH1.J / GOV / LAW execution beyond hook surfaces appears
- no memory, personality, `PH1.COMP`, Apple/client, or app-layer workaround work appears
- the implementation footprint remains bounded and no widening beyond the proven file scope is assumed without proof
- all required validation, regression, and cleanliness gates pass on the exact closeout commit

Slice 2F is not complete if the implementation widens into sender verification, later wake-enroll work, app/client behavior, or any protected execution path.


M) PHASE BOUNDARY

H8 governs Slice 2F only.

H8 freezes exactly one next bounded Section 03 slice:

- canonical `/v1/onboarding/continue` `PrimaryDeviceConfirm` compatibility execution foundation

H8 does not authorize:

- sender-verification implementation
- any narrower sender-verification micro-slice
- `VoiceEnrollLock`
- `WakeEnrollSampleCommit`
- `WakeEnrollCompleteCommit`
- deeper modality-specific `/v1/voice/turn` work
- real `PH1.ONB`
- real proof-governance execution
- Section 04 authority execution
- Section 05 persistence / sync execution
- Apple/client/app work
- H9 or any later planning document

The next lawful step after H8 is:

- the bounded Slice 2F implementation run against accepted Slice 1A-1D, accepted Slice 2A-2E, and frozen H1-H8 law only
