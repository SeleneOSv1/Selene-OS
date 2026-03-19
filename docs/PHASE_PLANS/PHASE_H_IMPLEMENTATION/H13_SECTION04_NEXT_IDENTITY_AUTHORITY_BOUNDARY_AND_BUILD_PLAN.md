PHASE H13 — SECTION 04 NEXT IDENTITY / AUTHORITY BOUNDARY AND BUILD PLAN

A) PURPOSE

H10 froze the completed Section 03 onboarding-continue chain at the explicit phase boundary after
accepted `CompleteCommit`.

H11 then froze and repo truth now accepts the first lawful Section 04 winner:

- governance-first protected execution via `RuntimeGovernanceRuntime::govern_voice_turn_execution`

H12 then froze and repo truth now accepts the second lawful Section 04 winner:

- proof-governance protected execution via
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state`

Current repo truth now also accepts the third lawful Section 04 winner:

- artifact-trust governance via
  `RuntimeGovernanceRuntime::govern_artifact_activation_execution`

H13 exists to remove the remaining ambiguity after that accepted governance / proof / artifact
chain.

H13 must decide whether current repo truth now exposes one exact bounded next Section 04 winner
inside the remaining `identity` / `authority` area, or whether Section 04 still requires an
internal `identity` / `authority` sub-boundary before any further implementation can lawfully
begin.

H13 must not:

- reopen the completed Section 03 boundary frozen by H10
- reinterpret generic remaining Section 04 work as if it were one exact next winner
- widen into a generic Section 04 bucket
- jump ahead to runtime law completion via `RuntimeLawRuntime::evaluate` or `govern_completion`
- jump ahead to Section 05 persistence and sync
- jump ahead to Apple/client/app work
- invent a false exact `identity` or `authority` winner where repo truth still exposes only a
  family or a downstream consumer
- treat non-selected comparison rows as defects instead of candidate eliminations

B) FROZEN LAW INPUTS

The following repo-truth inputs are binding for H13:

- `AGENTS.md` remains the active repository execution law
- `docs/CORE_ARCHITECTURE.md` remains the canonical architecture boundary
- `docs/SELENE_BUILD_EXECUTION_ORDER.md` remains the build-order law
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` remains the authoritative engine inventory
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md` remains the completed pre-authority ingress law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` remains the protected/authority layer law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md` remains the downstream persistence layer law
- accepted H1 through H12 remain binding planning law inputs
- accepted/pushed governance, proof, and artifact-trust Section 04 implementation truth at
  `c327d3bfcc72f9bbbd221e53da1fac184be22ce6` remains binding current repo truth

The binding Section 03 to Section 04 handoff preserved by H10, H11, H12, and the accepted
artifact-trust slice is:

- Section 03 stops at `ReadyForSection04Boundary`
- Section 03 emits one canonical `RuntimeExecutionEnvelope`
- the handoff posture is `AdmissionState::ExecutionAdmitted`
- the handoff preserves one canonical request/session/envelope path
- the handoff leaves later Section 04 and Section 05 fields unset unless already lawfully
  populated by accepted downstream slices

The binding accepted Section 04 chain for H13 is:

- H11 lawfully populated `governance_state` first
- H12 lawfully populated `proof_state` next while preserving `governance_state`
- the accepted artifact-trust slice lawfully populated `artifact_trust_state` while preserving
  `governance_state` and `proof_state`
- `identity_state`, `authority_state`, and `law_state` remain deferred after the accepted
  artifact-trust slice

The binding Section 04 law from current repo truth is:

- Section 04 owns protected execution before Section 05 persistence and sync
- Section 04 must operate on the canonical `RuntimeExecutionEnvelope`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md` places `identity` gating inside Section 04 before later
  downstream phases
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` places `identity` gating and identity risk
  scoring before access authorization and authorization scope enforcement
- protected execution remains session-bound, deterministic, auditable, and fail-closed
- no protected action may bypass the canonical authority path

C) CURRENT / TARGET / GAP

CURRENT

- H10 froze the Section 03 phase boundary after the completed onboarding-continue chain
- H11 selected and current repo truth accepts governance-first protected execution via
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`
- H12 selected and current repo truth accepts proof-governance protected execution via
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state`
- current repo truth now also accepts artifact-trust governance via
  `RuntimeGovernanceRuntime::govern_artifact_activation_execution`
- current repo truth still preserves one canonical admitted handoff at
  `ReadyForSection04Boundary`
- current repo truth now allows `governance_state`, `proof_state`, and `artifact_trust_state` to
  be populated while `identity_state`, `authority_state`, and `law_state` remain deferred
- current repo truth exposes downstream `identity` and `authority` posture consumers in
  `runtime_law.rs`, but it does not expose one exact `govern_identity*` or `govern_authority*`
  implementation seam in `runtime_governance.rs`

TARGET

- freeze one exact next Section 04 `identity` / `authority` winner after the accepted
  governance / proof / artifact chain, or prove that an internal Section 04 sub-boundary is still
  required
- preserve the H10 Section 03 boundary and the accepted H11 / H12 / artifact-trust chain
  unchanged
- preserve one canonical admitted request/session/envelope handoff into later Section 04 work
- preserve the Section 04 / Section 05 boundary
- defer all non-selected Section 04 and later-runtime work explicitly

GAP

- after the accepted artifact-trust slice, the remaining Section 04 area is narrower than before
  but still not fully linear
- current repo truth exposes `identity_state` and `authority_state` as remaining deferred
  protected states
- current repo truth exposes downstream `identity` and `authority` satisfaction rules in
  `RuntimeLawRuntime::evaluate`, but not the exact upstream producers that would lawfully populate
  those states next
- H13 must therefore decide whether one exact bounded `identity` or `authority` winner now
  clearly follows the accepted artifact-trust slice, or whether H13 must freeze an internal
  `identity`-before-`authority` sub-boundary instead of inventing a false winner

D) SECTION 04 NEXT SELECTION DECISION

The Section 04 next selection frozen by H13 is:

- the internal `identity`-before-`authority` Section 04 sub-boundary
- no exact next implementation winner is yet explicit from current repo truth

This is the selected H13 outcome.

Why this is the correct decision:

- the accepted Section 04 chain now lawfully populates `governance_state`, `proof_state`, and
  `artifact_trust_state`, but still leaves `identity_state`, `authority_state`, and `law_state`
  unset
- `docs/SELENE_BUILD_EXECUTION_ORDER.md` and `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
  place `identity` gating and identity risk scoring before access authorization and authorization
  scope enforcement
- current repo truth does not expose an exact `govern_identity*`, `govern_identity_state*`,
  `govern_authority*`, or `govern_authority_state*` runtime-governance seam comparable to the
  accepted H11, H12, or artifact-trust winners
- `runtime_governance.rs` still treats pre-populated `identity_state` and `authority_state` as
  deferred later protected state rather than as already-frozen next seams
- `runtime_law.rs` exposes `identity_posture_satisfied` and `authority_posture_satisfied` only as
  later law-consumption posture, not as the exact Section 04 producers that should be implemented
  next
- runtime law completion via `RuntimeLawRuntime::evaluate` and `govern_completion` is broader and
  later because it consumes governance, proof, `identity`, `authority`, `artifact_trust`, and
  persistence posture together
- selecting a generic `identity` / `authority` family winner would preserve ambiguity instead of
  removing it

What H13 means precisely:

- the next lawful planning move remains inside the remaining `identity` / `authority` area only
- the next exact implementation winner is not yet authorized by H13
- any later exact winner must first be frozen on the `identity` side of the remaining
  `identity` / `authority` boundary
- `authority` completion remains later than the still-unfrozen exact `identity` winner
- runtime law completion via `govern_completion`, Section 05, reopened Section 03, and
  Apple/client/app work remain deferred

E) candidate-scope comparison matrix

| candidate next scope or boundary | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| internal Section 04 `identity`-before-`authority` sub-boundary | `IN_SCOPE_SECTION04_NEXT` | Build Section 04 and execution-order law place `identity` gating/risk ahead of later authorization posture; current repo truth leaves both states deferred and exposes no exact producer seams | selected | removes ambiguity without inventing a false exact implementation winner |
| one exact `identity`-governance implementation seam already exposed by repo truth | `OUT_OF_SCOPE_THIS_PHASE` | no `govern_identity*` or `govern_identity_state*` surface exists in current repo truth | not available | no exact next `identity` winner is yet exposed |
| one exact `authority`-governance implementation seam already exposed by repo truth | `OUT_OF_SCOPE_THIS_PHASE` | no `govern_authority*` or `govern_authority_state*` surface exists in current repo truth | not available | no exact next `authority` winner is yet exposed |
| broader `identity` / `authority` completion family | `DEFER_LATER_SECTION04` | remaining family is real, but current repo truth still exposes it as a family rather than one exact winner | deferred | family buckets are not lawful winners |
| runtime law completion via `RuntimeLawRuntime::evaluate` and `govern_completion` | `DEFER_LATER_SECTION04` | exact law surfaces exist, but they consume broader protected posture across governance, proof, `identity`, `authority`, `artifact_trust`, and persistence | deferred | later completion stage, not the next bounded winner |
| generic Section 04 bucket | `DEFER_LATER_SECTION04` | Section 04 law remains broader than one lawful implementation slice | deferred | bucket winners are not lawful |
| Section 05 persistence / sync / reconcile / dedupe work | `DEFER_SECTION05` | execution-order law places Section 05 after Section 04 | deferred | downstream phase only |
| reopened Section 03 work | `OUT_OF_SCOPE_THIS_PHASE` | H10 froze the Section 03 phase boundary and the accepted Section 04 chain begins from that frozen handoff | not available | Section 03 reopening is not lawful here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | client work remains non-authoritative and downstream | not available | wrong phase and wrong owner |

F) selected scope and dependency matrix

| item | classification | Section 04 position | dependency / guardrail |
|---|---|---|---|
| internal `identity`-before-`authority` Section 04 sub-boundary | `IN_SCOPE_SECTION04_NEXT` | exact H13 output | must remain the only lawful H13 selection |
| accepted `ReadyForSection04Boundary` admitted handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | binding entry boundary into later Section 04 work | no pull-back into Section 03 and no bypass |
| accepted canonical `RuntimeExecutionEnvelope` path | `IN_SCOPE_SECTION04_NEXT` as protected baseline | single protected carrier for the accepted H11 / H12 / artifact chain and all later Section 04 work | no alternate envelope path |
| accepted canonical request / session / turn stack | `IN_SCOPE_SECTION04_NEXT` as protected baseline | one canonical runtime identity for all accepted Section 04 winners | no parallel request-family or session path |
| `governance_state` | `IN_SCOPE_SECTION04_NEXT` as protected baseline | already populated by accepted H11 | preserve and consume; do not reinterpret |
| `proof_state` | `IN_SCOPE_SECTION04_NEXT` as protected baseline | already populated by accepted H12 | preserve and consume; do not reinterpret |
| `artifact_trust_state` | `IN_SCOPE_SECTION04_NEXT` as protected baseline | already populated by the accepted artifact-trust slice | preserve and consume; do not reinterpret |
| `identity_state` | `IN_SCOPE_SECTION04_NEXT` as remaining boundary focus | next still-unfrozen protected posture family | one exact producer seam must be frozen later before any implementation |
| `authority_state` | `DEFER_LATER_SECTION04` | later protected posture after the H13-selected internal boundary | may not be selected before the exact `identity` winner is frozen |
| `law_state` | `DEFER_LATER_SECTION04` | later runtime law-completion seam | do not require or populate it in the H13-selected boundary |
| Section 05 persistence posture | `DEFER_SECTION05` | downstream persistence layer | do not pull it into the identity / authority boundary |

G) execution-surface / state-boundary matrix

| surface or state | current repo-truth posture | classification | H13 boundary rule |
|---|---|---|---|
| `AdmissionState::ExecutionAdmitted` | already set by Section 03 at the canonical handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as entry requirement for all later Section 04 work |
| `ReadyForSection04Boundary` | accepted deterministic stop line from Section 03 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as the only lawful Section 03 to Section 04 boundary |
| `governance_state` | lawfully populated by accepted H11 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as prior accepted output |
| `proof_state` | lawfully populated by accepted H12 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as prior accepted output |
| `artifact_trust_state` | lawfully populated by the accepted artifact-trust slice | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as prior accepted output |
| exact `identity` producer seam | not yet explicit in current repo truth | `IN_SCOPE_SECTION04_NEXT` as sub-boundary focus | must be frozen explicitly later before implementation |
| `identity_state` | still unset after the accepted artifact-trust slice | `IN_SCOPE_SECTION04_NEXT` as remaining boundary focus | next exact winner must come from this side of the boundary |
| exact `authority` producer seam | not yet explicit in current repo truth | `DEFER_LATER_SECTION04` | may not be selected before the exact `identity` winner is frozen |
| `authority_state` | still unset after the accepted artifact-trust slice | `DEFER_LATER_SECTION04` | remains later than the H13-selected internal boundary |
| `RuntimeLawRuntime::evaluate` / `govern_completion` | later law-consumption and completion surfaces | `DEFER_LATER_SECTION04` | do not use as a substitute for missing exact `identity` / `authority` producers |
| `law_state` | still unset after the accepted artifact-trust slice | `DEFER_LATER_SECTION04` | remains later than `identity` and `authority` completion |
| Section 05 persistence state | downstream persistence posture | `DEFER_SECTION05` | remains outside H13 scope |

H) repository workstream / file-impact matrix

| repository seam or file | role in H13 | classification | mandatory posture |
|---|---|---|---|
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H13_SECTION04_NEXT_IDENTITY_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | the only artifact H13 authorizes | `IN_SCOPE_SECTION04_NEXT` | planning-only output |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H12_SECTION04_NEXT_PROTECTED_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | frozen proof-governance planning input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reinterpret |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H11_SECTION04_PROTECTED_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | frozen governance-first planning input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reinterpret |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H10_SLICE2P_SECTION03_BOUNDARY_AND_BUILD_PLAN.md` | frozen Section 03 boundary input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reinterpret |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` | authoritative Section 04 law input for `identity`, `authority`, `artifact_trust`, and verification-before-authority | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not widen |
| `crates/selene_os/src/runtime_governance.rs` | source of the accepted H11 / H12 / artifact winners and proof that later `identity_state` / `authority_state` remain deferred | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only in H13; no exact new winner is claimed without an exposed seam |
| `crates/selene_os/src/runtime_law.rs` | source of downstream `identity` and `authority` posture consumers plus later `govern_completion` law completion | `DEFER_LATER_SECTION04` as downstream anchor | read only; not the next implementation winner |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` | canonical envelope state carrier for governance, proof, `identity`, `authority`, `artifact_trust`, and law | `IN_SCOPE_SECTION04_NEXT` as protected baseline | consume existing contract truth only |
| `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | accepted Section 03 stop-line and admitted handoff truth | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only; no Section 03 reopening |
| Section 05 persistence/sync files and workstreams | downstream persistence layer | `DEFER_SECTION05` | not part of H13 |
| Apple/client/app workstreams | non-target client surface | `OUT_OF_SCOPE_THIS_PHASE` | not part of H13 |

I) INTERNAL IMPLEMENTATION ORDER

H13 is planning only, but it freezes the future bounded planning order for the remaining Section 04
area:

1. preserve the accepted H10 Section 03 phase boundary unchanged
2. preserve the accepted H11 governance-first winner unchanged
3. preserve the accepted H12 proof-governance winner unchanged
4. preserve the accepted artifact-trust winner unchanged
5. preserve the canonical admitted handoff, request/session stack, and `RuntimeExecutionEnvelope`
   path unchanged
6. treat `governance_state`, `proof_state`, and `artifact_trust_state` as already accepted
   protected baseline
7. do not select or implement an exact `authority` winner before the exact `identity` winner is
   frozen
8. do not claim an exact `identity` winner until repo truth exposes one exact bounded producer seam
   or a later planning document freezes it explicitly
9. keep runtime law completion via `RuntimeLawRuntime::evaluate` / `govern_completion` later than
   the remaining `identity` / `authority` completion area
10. refuse any widening into generic Section 04, Section 05, reopened Section 03, or
    Apple/client/app work

J) verification and acceptance matrix

| proof area | required verification | H13 acceptance condition |
|---|---|---|
| H10 boundary carry-forward proof | prove H10 still freezes the completed Section 03 onboarding-continue chain | H13 begins from the frozen Section 03 boundary, not from reopened Section 03 work |
| accepted H11 / H12 / artifact chain carry-forward proof | prove governance-first, proof-governance, and artifact-trust remain the accepted Section 04 winners already landed | H13 begins after the accepted Section 04 chain, not beside it |
| admitted handoff proof | prove current repo truth still reaches `ReadyForSection04Boundary` with `AdmissionState::ExecutionAdmitted` | the H13 boundary remains grounded in the actual handoff shape |
| no exact `identity` producer proof | prove current repo truth exposes no exact `govern_identity*` or `govern_identity_state*` seam | H13 does not invent a false exact `identity` winner |
| no exact `authority` producer proof | prove current repo truth exposes no exact `govern_authority*` or `govern_authority_state*` seam | H13 does not invent a false exact `authority` winner |
| `identity`-before-`authority` ordering proof | prove Build Section 04 and execution-order law place `identity` gating/risk before later authorization posture | H13 may lawfully freeze an internal `identity`-before-`authority` sub-boundary |
| runtime law deferral proof | prove `RuntimeLawRuntime::evaluate` / `govern_completion` consume broader protected posture across governance, proof, `identity`, `authority`, `artifact_trust`, and persistence | law completion remains explicitly later |
| no Section 05 bleed proof | prove Section 05 persistence and sync remain downstream only | H13 does not pull Section 05 forward |
| planning-artifact proof | prove the H13 document states CURRENT / TARGET / GAP explicitly and records one exact internal sub-boundary explicitly | H13 removes the residual ambiguity after the accepted artifact-trust slice |
| cleanliness and readiness proof | prove H13 validates cleanly on a one-file doc-only tree | H13 is not complete until the planning artifact itself validates cleanly |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| internal `identity`-before-`authority` Section 04 sub-boundary | `IN_SCOPE_SECTION04_NEXT` | this is the only lawful H13 output |
| preserve accepted H10 Section 03 boundary and accepted H11 / H12 / artifact-trust winners | `IN_SCOPE_SECTION04_NEXT` | do not reinterpret or replace accepted prior truth |
| preserve one canonical admitted request / session / envelope path | `IN_SCOPE_SECTION04_NEXT` | no alternate request-family, session, or envelope path |
| exact future `identity` implementation winner | `DEFER_LATER_SECTION04` | do not claim or authorize one until it is explicitly frozen later |
| later `authority` completion | `DEFER_LATER_SECTION04` | do not pull `authority` ahead of the still-unfrozen exact `identity` winner |
| later runtime law completion via `evaluate` / `govern_completion` | `DEFER_LATER_SECTION04` | do not pull PH1.LAW completion into the H13 boundary |
| generic Section 04 bucket | `DEFER_LATER_SECTION04` | do not widen into a bucket implementation |
| Section 05 persistence / sync execution | `DEFER_SECTION05` | do not pull persistence correctness forward |
| PH1.ONB business execution | `DEFER_LATER_RUNTIME` | no business execution widening |
| PH1.COMP, memory, personality/emotion runtime | `DEFER_LATER_RUNTIME` | no later-runtime widening |
| any Section 03 reopening, including `/v1/onboarding/continue` and generic deeper `/v1/voice/turn` work | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen completed or unfrozen Section 03 concerns |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | no client-owned execution truth |
| no alternate authority path | `IN_SCOPE_SECTION04_NEXT` | Section 04 must remain inside the canonical authority path |

L) COMPLETION STANDARD

H13 is complete only when all of the following are true:

- H13 explicitly states that H10 froze the completed Section 03 onboarding-continue chain
- H13 explicitly states that H11, H12, and the accepted artifact-trust slice remain the accepted
  Section 04 winner chain so far
- H13 explicitly states CURRENT / TARGET / GAP
- H13 freezes one exact internal Section 04 `identity` / `authority` sub-boundary rather than a
  generic bucket
- H13 explicitly states that the selected outcome is the internal `identity`-before-`authority`
  sub-boundary
- H13 explicitly states that no exact next implementation winner is yet exposed by current repo
  truth
- H13 explicitly preserves the canonical admitted handoff, request/session stack, and
  `RuntimeExecutionEnvelope` path
- H13 explicitly preserves `governance_state`, `proof_state`, and `artifact_trust_state` as the
  accepted protected baseline
- H13 explicitly keeps exact future `identity` and later `authority` completion unfrozen and
  deferred until a later explicit planning decision
- H13 explicitly keeps runtime law completion via `govern_completion` deferred
- H13 explicitly keeps Section 05 persistence / sync deferred
- H13 explicitly keeps PH1.ONB, PH1.COMP, memory, personality/emotion runtime, Apple/client/app
  work, and all Section 03 reopening deferred
- the H13 planning artifact passes title, heading, token, and design-readiness validation on a
  clean tree

H13 is not complete if it merely repeats “Section 04 next,” if it leaves the next boundary
implicit, if it silently promotes `authority` ahead of `identity`, if it silently invents a
`govern_identity*` or `govern_authority*` winner that repo truth does not expose, or if it jumps
to runtime law or Section 05.

M) PHASE BOUNDARY

H13 governs the next bounded Section 04 `identity` / `authority` planning boundary only.

H13 authorizes no implementation by itself, but it freezes the next lawful planning target as:

- the internal `identity`-before-`authority` Section 04 sub-boundary
- with no exact next implementation winner yet authorized from current repo truth

H13 does not authorize:

- any reopening of `/v1/onboarding/continue`
- any reopening of generic deeper `/v1/voice/turn` work
- any generic Section 04 bucket
- any exact `identity` implementation winner that is not later frozen explicitly
- any exact `authority` implementation winner that is not later frozen explicitly
- any runtime law completion via `RuntimeLawRuntime::evaluate` or `govern_completion`
- any Section 05 persistence / sync execution
- any PH1.ONB business execution
- any PH1.COMP, memory, personality/emotion runtime execution
- any Apple/client/app work

PHASE BOUNDARY:

- H10 remains the frozen Section 03 boundary
- H11 remains the accepted governance-first Section 04 winner
- H12 remains the accepted proof-governance Section 04 winner
- the accepted artifact-trust slice remains the third lawful Section 04 winner via
  `govern_artifact_activation_execution`
- H13 freezes the internal `identity`-before-`authority` sub-boundary only
- the exact next `identity` winner remains unfrozen until a later explicit planning decision
- `authority` completion remains later than the still-unfrozen exact `identity` winner
- runtime law and Section 05 remain downstream
- any future implementation instruction must stay bounded inside the still-unfrozen `identity`
  side of this boundary only and must not reopen completed Section 03 scope or widen beyond this
  selected boundary
