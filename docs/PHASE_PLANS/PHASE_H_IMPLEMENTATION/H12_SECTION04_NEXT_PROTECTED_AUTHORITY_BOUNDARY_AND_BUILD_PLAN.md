PHASE H12 — SECTION 04 NEXT PROTECTED AUTHORITY BOUNDARY AND BUILD PLAN

A) PURPOSE

H10 froze the completed Section 03 onboarding-continue chain at the explicit phase boundary after
accepted `CompleteCommit`.

H11 then froze and repo truth now accepts the first lawful Section 04 winner:

- governance-first protected execution via `RuntimeGovernanceRuntime::govern_voice_turn_execution`

H12 exists to remove the remaining ambiguity after that accepted/pushed H11 winner.

H12 must decide whether current repo truth now exposes one exact next bounded Section 04
protected/authority winner after governance-first execution, or whether Section 04 still requires
an internal sub-boundary before any further implementation can lawfully begin.

H12 must not:

- reopen the completed Section 03 boundary frozen by H10
- reinterpret generic deeper `/v1/voice/turn` work as if it were the next Section 04 winner
- widen into a generic Section 04 bucket
- jump ahead to Section 05 persistence and sync
- jump ahead to Apple/client/app work
- invent a false exact next winner where repo truth still exposes only a family
- treat older comparison FAIL rows as defects instead of candidate eliminations

B) FROZEN LAW INPUTS

The following repo-truth inputs are binding for H12:

- `AGENTS.md` remains the active repository execution law
- `docs/CORE_ARCHITECTURE.md` remains the canonical architecture boundary
- `docs/SELENE_BUILD_EXECUTION_ORDER.md` remains the build-order law
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` remains the authoritative engine inventory
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md` remains the completed pre-authority ingress law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` remains the protected/authority layer law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md` remains the downstream persistence layer law
- accepted H1 through H11 remain binding planning law inputs

The binding Section 03 to Section 04 handoff preserved by H10 and H11 is:

- Section 03 stops at `ReadyForSection04Boundary`
- Section 03 emits one canonical `RuntimeExecutionEnvelope`
- the handoff posture is `AdmissionState::ExecutionAdmitted`
- the handoff preserves one canonical request/session/envelope path
- the handoff leaves later Section 04 and Section 05 fields unset unless already lawfully populated

The binding H11 outcome for H12 is:

- the first lawful Section 04 winner is governance-first protected execution via
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`
- H11 populated `governance_state` only
- H11 explicitly deferred later proof-governance
- H11 explicitly deferred later artifact-trust governance
- H11 explicitly deferred identity / authority completion
- H11 explicitly deferred runtime law completion
- H11 explicitly preserved Section 05, Section 03 reopening, and Apple/client/app work as
  deferred

The binding Section 04 law from current repo truth is:

- Section 04 owns protected execution before Section 05 persistence and sync
- Section 04 must operate on the canonical `RuntimeExecutionEnvelope`
- protected execution remains session-bound, deterministic, auditable, and fail-closed
- no protected action may bypass the canonical authority path

C) CURRENT / TARGET / GAP

CURRENT

- H10 froze the Section 03 phase boundary after the completed onboarding-continue chain
- H11 selected and current repo truth now accepts governance-first protected execution via
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`
- current repo truth still preserves one canonical admitted handoff at
  `ReadyForSection04Boundary`
- current repo truth now allows `governance_state` to be populated while `proof_state`,
  `identity_state`, `authority_state`, `artifact_trust_state`, and `law_state` remain deferred
- current repo truth exposes multiple residual Section 04 candidates:
  `govern_protected_action_proof`, `govern_protected_action_proof_state`,
  `govern_artifact_activation_execution`, identity / authority completion family, and
  `RuntimeLawRuntime::evaluate` / `govern_completion`

TARGET

- freeze one exact next Section 04 protected/authority winner after accepted H11
- preserve the H10 Section 03 boundary and the H11 governance-first winner unchanged
- preserve one canonical admitted request/session/envelope handoff into later Section 04 work
- preserve the Section 04 / Section 05 boundary
- defer all non-selected Section 04 and later-runtime work explicitly

GAP

- after H11, Section 04 is narrower than before but still not fully linear
- current repo truth exposes more than one residual Section 04 candidate
- H12 must therefore decide whether one exact bounded next winner now clearly follows H11
- if not, H12 must freeze an internal Section 04 sub-boundary instead of inventing a false winner

D) SECTION 04 NEXT SELECTION DECISION

The Section 04 next winner selected by H12 is:

- the proof-governance protected execution foundation centered on
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state`
- with `RuntimeGovernanceRuntime::govern_protected_action_proof` preserved as the bounded helper
  seam inside the same winner

This is the selected H12 outcome.

Why this is the correct decision:

- H11 already froze governance-first execution and repo truth now accepts that `governance_state`
  is the first populated Section 04 state
- current repo truth exposes `govern_protected_action_proof_state` as an exact bounded follow-on
  governance surface that consumes canonical `proof_state` rather than inventing a new carrier or
  alternate authority path
- `govern_protected_action_proof_state` is more exact than the broader identity / authority
  completion family
- `govern_protected_action_proof_state` is earlier and less specialized than
  `govern_artifact_activation_execution`, which requires canonical `artifact_trust_state`
- `RuntimeLawRuntime::evaluate` and `govern_completion` are broader downstream law-completion
  surfaces that consume richer protected posture, including `governance_state`, `proof_state`,
  `identity_state`, `authority_state`, `artifact_trust_state`, and persistence posture
- selecting a generic Section 04 bucket would preserve ambiguity instead of removing it

What H12 means precisely:

- the next lawful Section 04 winner after H11 is proof-governance only
- the exact primary seam is `govern_protected_action_proof_state`
- the exact bounded helper seam is `govern_protected_action_proof`
- the future implementation must still consume the canonical admitted envelope/session/turn path
- the future implementation must not widen into artifact-trust governance, identity / authority
  completion, or PH1.LAW completion

E) candidate-scope comparison matrix

| candidate next scope or boundary | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| proof-governance protected execution foundation via `RuntimeGovernanceRuntime::govern_protected_action_proof_state` plus bounded helper `govern_protected_action_proof` | `IN_SCOPE_SECTION04_NEXT` | exact runtime-governance surfaces exist and operate on proof-bearing posture without requiring Section 05 persistence or full law completion | selected | first exact bounded next winner after H11 |
| artifact-trust governance via `RuntimeGovernanceRuntime::govern_artifact_activation_execution` | `DEFER_LATER_SECTION04` | exact governance surface exists, but it requires canonical `artifact_trust_state` transport and complete artifact-trust evidence | deferred | exact later Section 04 candidate, but more specialized and later than proof-governance |
| identity / authority completion family | `DEFER_LATER_SECTION04` | identity and authority are required later by repo truth, but current repo truth exposes them as a family rather than one exact next implementation seam | deferred | broader family, not one exact winner |
| runtime law completion via `RuntimeLawRuntime::evaluate` and `govern_completion` | `DEFER_LATER_SECTION04` | exact law surfaces exist, but they consume broader protected posture across governance, proof, identity, authority, artifact trust, and persistence | deferred | later Section 04 completion stage, not the next bounded winner |
| generic Section 04 bucket | `DEFER_LATER_SECTION04` | Section 04 law remains broader than one lawful implementation slice | deferred | bucket winners are not lawful |
| Section 05 persistence / sync / reconcile / dedupe work | `DEFER_SECTION05` | execution-order law places persistence after Section 04 | deferred | downstream phase only |
| reopened Section 03 work | `OUT_OF_SCOPE_THIS_PHASE` | H10 froze the Section 03 phase boundary and H11/H12 begin from that frozen handoff | not available | Section 03 reopening is not lawful here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | client work remains non-authoritative and downstream | not available | wrong phase and wrong owner |

F) selected scope and dependency matrix

| item | classification | Section 04 position | dependency / guardrail |
|---|---|---|---|
| `RuntimeGovernanceRuntime::govern_protected_action_proof_state` | `IN_SCOPE_SECTION04_NEXT` | exact primary H12 winner | must remain the primary future implementation seam |
| `RuntimeGovernanceRuntime::govern_protected_action_proof` | `IN_SCOPE_SECTION04_NEXT` | bounded helper inside the selected winner | may support the selected winner only; must not widen scope |
| accepted `ReadyForSection04Boundary` admitted handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | binding entry boundary into later Section 04 work | no pull-back into Section 03 and no bypass |
| accepted canonical `RuntimeExecutionEnvelope` path | `IN_SCOPE_SECTION04_NEXT` as protected baseline | single protected carrier for proof-governance input | no alternate envelope path |
| accepted canonical request / session / turn stack | `IN_SCOPE_SECTION04_NEXT` as protected baseline | one canonical runtime identity for the selected winner | no parallel request-family or session path |
| `governance_state` | `IN_SCOPE_SECTION04_NEXT` as protected baseline | already populated by accepted H11 | preserve and consume; do not reinterpret |
| `proof_state` | `IN_SCOPE_SECTION04_NEXT` | newly consumed protected posture for H12 winner | consume fail-closed; do not widen beyond proof-governance |
| `artifact_trust_state` | `DEFER_LATER_SECTION04` | later artifact-authority seam | do not require or populate it in the H12 winner |
| `identity_state` and `authority_state` | `DEFER_LATER_SECTION04` | later protected completion family | do not require or populate them in the H12 winner |
| `law_state` | `DEFER_LATER_SECTION04` | later runtime law-completion seam | do not require or populate it in the H12 winner |
| Section 05 persistence / sync posture | `DEFER_SECTION05` | downstream persistence layer | consume only as already-carried posture if needed by baseline law; do not implement Section 05 behavior |
| PH1.ONB business execution | `DEFER_LATER_RUNTIME` | downstream business runtime only | no business execution widening |
| PH1.COMP, memory, personality/emotion runtime | `DEFER_LATER_RUNTIME` | later runtime only | no later-runtime widening |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | non-authoritative client surface | no client-owned execution truth |

G) execution-surface / state-boundary matrix

| execution surface or state | current boundary posture | classification | H12 posture |
|---|---|---|---|
| `AdmissionState::ExecutionAdmitted` | already set by Section 03 at the canonical handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as entry requirement for all later Section 04 work |
| `ReadyForSection04Boundary` | accepted deterministic stop line from Section 03 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as the only lawful Section 03 to Section 04 boundary |
| `session_id`, `turn_id`, and `device_turn_sequence` on the admitted envelope | already available at the handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | future proof-governance must reuse them through the canonical envelope path |
| `governance_state` | now lawfully populated by accepted H11 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as prior accepted output and required predecessor posture |
| `proof_state` | still unset by the Section 03 handoff and H11 winner | `IN_SCOPE_SECTION04_NEXT` | exact new posture the H12 winner governs |
| `artifact_trust_inputs` | ingress-carried non-authoritative input only | `DEFER_LATER_SECTION04` | preserve; do not collapse into the proof-governance winner |
| `artifact_trust_state` | still unset for the H11-selected winner | `DEFER_LATER_SECTION04` | do not populate in the H12 winner |
| `identity_state` | still unset after H11 | `DEFER_LATER_SECTION04` | do not populate in the H12 winner |
| `authority_state` | still unset after H11 | `DEFER_LATER_SECTION04` | do not populate in the H12 winner |
| `law_state` | still unset after H11 | `DEFER_LATER_SECTION04` | do not populate in the H12 winner |
| `persistence_state` | may already be carried as baseline posture | `DEFER_SECTION05` as downstream baseline | do not reinterpret carried persistence posture as Section 05 implementation |
| `memory_state`, `computation_state`, and wider later-runtime state | unset for the H11 winner | `DEFER_LATER_RUNTIME` | keep unset |
| reopened `/v1/onboarding/continue` or generic deeper `/v1/voice/turn` work | completed or unfrozen Section 03 concern only | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen |

H) repository workstream / file-impact matrix

| repository seam or file | role in H12 | classification | mandatory posture |
|---|---|---|---|
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H12_SECTION04_NEXT_PROTECTED_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | the only artifact H12 authorizes directly | `IN_SCOPE_SECTION04_NEXT` | planning-only output |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H11_SECTION04_PROTECTED_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | frozen prior Section 04 winner input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reinterpret |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H10_SLICE2P_SECTION03_BOUNDARY_AND_BUILD_PLAN.md` | frozen Section 03 boundary input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reopen |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` | authoritative Section 04 law input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not widen |
| `crates/selene_os/src/runtime_governance.rs` | source of the selected next winner and competing later governance candidates | `IN_SCOPE_SECTION04_NEXT` as future implementation anchor | primary future implementation file for the selected winner |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` | canonical envelope state carrier for governance, proof, identity, authority, artifact trust, and law | `IN_SCOPE_SECTION04_NEXT` as protected baseline | consume existing contract truth only |
| `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | accepted Section 03 stop-line and admitted handoff truth | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only; no Section 03 reopening |
| `crates/selene_os/src/runtime_request_foundation.rs` | canonical request-stack boundary into runtime ingress/execution | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only; no alternate request path |
| `crates/selene_os/src/runtime_law.rs` | later law-completion surfaces | `DEFER_LATER_SECTION04` | read only; not part of the next winner |
| Section 05 persistence/sync files and workstreams | downstream persistence layer | `DEFER_SECTION05` | not part of H12 winner |
| Apple/client/app workstreams | non-target client surface | `OUT_OF_SCOPE_THIS_PHASE` | not part of H12 winner |

I) INTERNAL IMPLEMENTATION ORDER

H12 is planning only, but it freezes the future bounded implementation order for the selected
winner:

1. preserve the accepted H10 Section 03 phase boundary and H11 governance-first winner unchanged
2. preserve the accepted `ReadyForSection04Boundary` admitted handoff unchanged
3. preserve the canonical `RuntimeExecutionEnvelope` path and request/session/turn stack unchanged
4. bind the next Section 04 implementation slice only to
   `RuntimeGovernanceRuntime::govern_protected_action_proof_state`
5. keep `RuntimeGovernanceRuntime::govern_protected_action_proof` as a bounded helper inside the
   same proof-governance slice
6. consume proof-bearing posture only through canonical `proof_state` and canonical
   session/turn-bound runtime inputs
7. fail closed when proof-bearing posture is missing, incomplete, or indicates critical proof
   failure
8. keep `artifact_trust_state`, `identity_state`, `authority_state`, `law_state`,
   `memory_state`, `computation_state`, and Section 05 execution behavior downstream and unset
9. add bounded verification proving the selected proof-governance slice reuses the H10/H11
   boundary chain without widening into artifact-trust, identity / authority, PH1.LAW, or
   Section 05
10. refuse any widening into generic Section 04, Section 05, reopened Section 03, or
    Apple/client/app work

J) verification and acceptance matrix

| proof area | required verification | H12 acceptance condition |
|---|---|---|
| H10 boundary carry-forward proof | prove H10 still freezes the completed Section 03 onboarding-continue chain | H12 begins from the frozen Section 03 boundary, not from reopened Section 03 work |
| H11 winner carry-forward proof | prove H11 still freezes governance-first protected execution via `govern_voice_turn_execution` | H12 begins after the accepted first Section 04 winner, not beside it |
| admitted handoff proof | prove current repo truth still reaches `ReadyForSection04Boundary` with `AdmissionState::ExecutionAdmitted` | the selected H12 winner remains grounded in the actual boundary shape |
| proof-governance winner proof | prove `govern_protected_action_proof_state` and helper `govern_protected_action_proof` form one exact bounded proof-governance seam | H12 freezes one exact next winner rather than a family |
| artifact-trust deferral proof | prove `govern_artifact_activation_execution` requires canonical `artifact_trust_state` transport and complete artifact-trust evidence | artifact-trust governance remains explicitly later |
| identity / authority deferral proof | prove identity / authority remains a broader completion family rather than one exact next winner | identity / authority completion remains deferred |
| law-completion deferral proof | prove `RuntimeLawRuntime::evaluate` / `govern_completion` consume broader protected posture across governance, proof, identity, authority, artifact trust, and persistence | law completion remains explicitly later |
| canonical-path preservation proof | prove one canonical request stack, one canonical envelope path, and one accepted Section 03 stop line remain binding | no alternate protected-execution path is implied |
| no Section 05 bleed proof | prove persistence and sync remain downstream only | H12 does not pull Section 05 forward |
| no later-runtime bleed proof | prove PH1.ONB business execution, PH1.COMP, memory, personality/emotion runtime, and Apple/client work remain deferred | H12 stays bounded |
| planning-artifact proof | prove the H12 document states CURRENT / TARGET / GAP explicitly and records one exact next winner explicitly | H12 removes the residual ambiguity after H11 |
| cleanliness and readiness proof | prove H12 validates cleanly on a one-file doc-only tree | H12 is not complete until the planning artifact itself validates cleanly |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| proof-governance protected execution foundation via `govern_protected_action_proof_state` plus helper `govern_protected_action_proof` | `IN_SCOPE_SECTION04_NEXT` | this is the only lawful H12 winner |
| preserve accepted H10 Section 03 boundary and H11 governance-first winner | `IN_SCOPE_SECTION04_NEXT` | do not reinterpret or replace accepted prior truth |
| preserve one canonical admitted request / session / envelope path | `IN_SCOPE_SECTION04_NEXT` | no alternate request-family, session, or envelope path |
| consume proof-bearing posture only through canonical `proof_state` | `IN_SCOPE_SECTION04_NEXT` | do not invent a second proof carrier |
| later artifact-trust governance via `govern_artifact_activation_execution` | `DEFER_LATER_SECTION04` | do not pull artifact-trust work into the proof-governance slice |
| later identity / authority completion | `DEFER_LATER_SECTION04` | do not pull identity / authority family work into the proof-governance slice |
| later runtime law completion via `evaluate` / `govern_completion` | `DEFER_LATER_SECTION04` | do not pull PH1.LAW completion into the proof-governance slice |
| generic Section 04 bucket | `DEFER_LATER_SECTION04` | do not widen into a bucket implementation |
| Section 05 persistence / sync execution | `DEFER_SECTION05` | do not pull persistence correctness forward |
| PH1.ONB business execution | `DEFER_LATER_RUNTIME` | no business execution widening |
| PH1.COMP, memory, personality/emotion runtime | `DEFER_LATER_RUNTIME` | no later-runtime widening |
| any Section 03 reopening, including `/v1/onboarding/continue` and generic deeper `/v1/voice/turn` work | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen completed or unfrozen Section 03 concerns |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | no client-owned execution truth |
| no alternate authority path | `IN_SCOPE_SECTION04_NEXT` | Section 04 must remain inside the canonical authority path |

L) COMPLETION STANDARD

H12 is complete only when all of the following are true:

- H12 explicitly states that H10 froze the completed Section 03 onboarding-continue chain
- H12 explicitly states that H11 froze the first lawful Section 04 winner as governance-first
  protected execution
- H12 explicitly states CURRENT / TARGET / GAP
- H12 freezes one exact next Section 04 winner rather than a generic bucket
- the selected next winner is proof-governance via
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state`
- H12 explicitly preserves `RuntimeGovernanceRuntime::govern_protected_action_proof` as the
  bounded helper inside the same winner
- H12 explicitly preserves the canonical admitted handoff, request/session stack, and
  `RuntimeExecutionEnvelope` path
- H12 explicitly keeps artifact-trust governance, identity / authority completion, and runtime law
  completion deferred
- H12 explicitly keeps Section 05 persistence / sync deferred
- H12 explicitly keeps PH1.ONB, PH1.COMP, memory, personality/emotion runtime, Apple/client/app
  work, and all Section 03 reopening deferred
- the H12 planning artifact passes title, heading, token, and design-readiness validation on a
  clean tree

H12 is not complete if it merely repeats “Section 04 next,” if it leaves the next winner implicit,
if it silently widens into generic authority work, or if it reopens completed Section 03 behavior.

M) PHASE BOUNDARY

H12 governs the next bounded Section 04 protected/authority planning winner only.

H12 authorizes no implementation by itself, but it freezes the next lawful Section 04 target as:

- proof-governance protected execution via
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state`
- with `RuntimeGovernanceRuntime::govern_protected_action_proof` preserved as the bounded helper
  seam inside that exact winner

H12 does not authorize:

- any reopening of `/v1/onboarding/continue`
- any reopening of generic deeper `/v1/voice/turn` work
- any generic Section 04 protected/authority bucket
- any artifact-trust governance via `govern_artifact_activation_execution`
- any identity / authority completion family work
- any runtime law completion via `RuntimeLawRuntime::evaluate` or `govern_completion`
- any Section 05 persistence / sync execution
- any PH1.ONB business execution
- any PH1.COMP, memory, personality/emotion runtime execution
- any Apple/client/app work

PHASE BOUNDARY:

- H10 remains the frozen Section 03 boundary
- H11 remains the accepted governance-first protected execution winner
- H12 freezes proof-governance as the next exact Section 04 winner only
- later Section 04 artifact-trust, identity / authority, and law-completion slices remain unfrozen
- Section 05 and later runtime remain downstream
- any future implementation instruction must stay bounded to proof-governance only and must not
  reopen completed Section 03 scope or widen beyond this selected winner
