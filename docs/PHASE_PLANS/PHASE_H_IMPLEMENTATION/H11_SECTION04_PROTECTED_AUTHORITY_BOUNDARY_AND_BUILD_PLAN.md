PHASE H11 — SECTION 04 PROTECTED AUTHORITY BOUNDARY AND BUILD PLAN

A) PURPOSE

H10 froze the completed Section 03 onboarding-continue chain at the explicit phase boundary after
accepted `CompleteCommit`.

H11 exists to narrow the next lawful planning target inside Section 04 from a broad
protected/authority area into one exact bounded next winner grounded in current repo truth.

H11 must not:

- reopen the completed `/v1/onboarding/continue` chain
- reinterpret generic deeper `/v1/voice/turn` work as if it were the next winner
- jump ahead to Section 05 persistence and sync
- jump ahead to Apple/client/app work
- invent nonexistent protected-execution candidates
- treat older FAIL comparison rows as defects instead of candidate eliminations

H11 therefore decides whether current repo truth already exposes one exact first Section 04
protected/authority planning winner, or whether Section 04 itself still requires an internal
sub-boundary before any implementation planning can lawfully begin.

B) FROZEN LAW INPUTS

The following repo-truth inputs are binding for H11:

- `AGENTS.md` remains the active repository execution law
- `docs/CORE_ARCHITECTURE.md` remains the canonical architecture boundary
- `docs/SELENE_BUILD_EXECUTION_ORDER.md` remains the build-order law
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` remains the authoritative engine inventory
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md` remains the completed pre-authority ingress law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` remains the protected/authority layer law
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md` remains the downstream persistence layer law
- accepted H1 through H10 remain binding planning law inputs

The binding repo-truth handoff into Section 04 is:

- Section 03 now converges canonical ingress and compatibility handling
- Section 03 stops at `ReadyForSection04Boundary`
- Section 03 emits `RuntimeExecutionEnvelope` instances with
  `AdmissionState::ExecutionAdmitted`
- Section 03 leaves Section 04 and later protected-execution fields unset at the boundary
- the canonical runtime request stack still requires all admitted routes to flow through the
  accepted ingress and envelope path

The binding Section 04 law from current repo truth is:

- Section 04 is the first protected execution layer after Section 03
- Section 04 owns governance, policy, authorization, simulation, identity, proof, and related
  authority-facing protected execution concerns
- Section 04 must stay inside the canonical `RuntimeExecutionEnvelope` path
- Section 05 persistence, sync, reconcile, dedupe, and acknowledgement remain downstream

The binding H10 outcome for H11 is:

- the completed Section 03 onboarding-continue chain remains frozen through accepted
  `CompleteCommit`
- `WakeEnrollDeferCommit` remains deferred
- no lawful further `/v1/onboarding/continue` slice remains exposed
- no exact later Section 03 `/v1/voice/turn` slice was frozen by H10
- H10 explicitly handed the next planning decision forward into Section 04

C) CURRENT / TARGET / GAP

CURRENT

- H10 froze the Section 03 phase boundary after the completed onboarding-continue chain
- current Section 03 repo truth stops at `ReadyForSection04Boundary`
- current Section 03 repo truth produces canonical `RuntimeExecutionEnvelope` handoff state with
  `AdmissionState::ExecutionAdmitted`
- current Section 03 repo truth leaves `governance_state`, `proof_state`, `identity_state`,
  `authority_state`, `artifact_trust_state`, and `law_state` unset at the handoff
- current repo truth already exposes concrete Section 04 runtime surfaces in
  `runtime_governance.rs` and `runtime_law.rs`

TARGET

- freeze one exact next Section 04 protected/authority planning winner, or prove an internal
  Section 04 sub-boundary is required
- preserve one canonical envelope path, one accepted session/request stack, and one post-Section-03
  handoff boundary
- preserve Section 05, PH1.ONB business execution, PH1.COMP and personality/emotion execution,
  Apple/client/app work, and all Section 03 reopening as deferred unless current repo truth makes
  any of them part of the exact next Section 04 winner

GAP

- Section 04 is still broader than a lawful implementation slice
- repo truth must be narrowed from a general protected/authority layer into one exact bounded next
  planning target
- current repo truth distinguishes governance-first execution, later proof-governance work, later
  artifact-trust work, and broader law completion, but H11 must decide which one is first

D) SECTION 04 SELECTION DECISION

The Section 04 winner selected by H11 is:

- the canonical Section 04 runtime-governance protected execution foundation anchored on
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`

This is the selected H11 outcome.

Why this is the correct decision:

- current Section 03 hands off an admitted canonical `RuntimeExecutionEnvelope` at
  `ReadyForSection04Boundary`, not a broader post-authority or persistence-complete runtime state
- `RuntimeGovernanceRuntime::govern_voice_turn_execution` is the first exact Section 04 surface
  exposed by repo truth that consumes this handoff shape without requiring later proof, identity,
  artifact-trust, or law-completion state
- the selected governance-first seam records the first protected execution decision by attaching
  `governance_state` while keeping later Section 04 and Section 05 concerns downstream
- `RuntimeGovernanceRuntime::govern_protected_action_proof` and
  `RuntimeGovernanceRuntime::govern_protected_action_proof_state` are separable later Section 04
  proof-oriented candidates because they depend on proof-carrying state that the current Section 03
  handoff does not yet establish
- `RuntimeGovernanceRuntime::govern_artifact_activation_execution` is a separable later Section 04
  artifact-trust candidate because it depends on `artifact_trust_state`
- `RuntimeLawRuntime::evaluate` and `RuntimeLawRuntime::govern_completion` are broader downstream
  Section 04 law-completion surfaces that depend on richer protected-execution state than the
  current handoff exposes
- selecting a generic Section 04 bucket would preserve ambiguity instead of removing it

What H11 means precisely:

- the next lawful planning winner is Section 04 governance-first protected execution
- this winner is not a reopening of generic `/v1/voice/turn` modality work; it is the first
  protected execution seam that operates on the already-converged canonical envelope emitted by
  Section 03
- Section 04 law/proof/artifact-trust/identity/authority completion remains later inside Section 04
- Section 05 persistence and sync remain downstream
- no Section 03 reopening is authorized
- no Apple/client/app work is authorized

E) candidate-scope comparison matrix

| candidate next scope or boundary | classification | repo-truth basis | selection result | reason |
|---|---|---|---|---|
| canonical Section 04 runtime-governance protected execution foundation via `RuntimeGovernanceRuntime::govern_voice_turn_execution` | `IN_SCOPE_SECTION04_NEXT` | consumes admitted envelope and current handoff fields, writes first `governance_state`, and does not require later proof/identity/artifact-trust/law completion | selected | first exact bounded Section 04 winner exposed by repo truth |
| separable Section 04 proof-governance foundation via `RuntimeGovernanceRuntime::govern_protected_action_proof` and `govern_protected_action_proof_state` | `DEFER_LATER_SECTION04` | proof-oriented surfaces exist, but they depend on proof-bearing state that is later than the current handoff | deferred | exact later Section 04 candidate, but not first |
| separable Section 04 artifact-trust governance via `RuntimeGovernanceRuntime::govern_artifact_activation_execution` | `DEFER_LATER_SECTION04` | exact governance surface exists, but it depends on `artifact_trust_state` | deferred | exact later Section 04 candidate, but not first |
| broader Section 04 law-completion path via `RuntimeLawRuntime::evaluate` and `govern_completion` | `DEFER_LATER_SECTION04` | exact law surfaces exist, but they depend on richer protected state, including governance and other downstream posture | deferred | broader later Section 04 completion stage |
| generic Section 04 protected/authority bucket | `DEFER_LATER_SECTION04` | Section 04 law is broader than one lawful implementation slice | deferred | bucket winners are not lawful |
| Section 05 persistence / sync / reconcile / dedupe work | `DEFER_SECTION05` | execution-order law places these after Section 04 | deferred | downstream phase only |
| deeper Section 03 `/v1/voice/turn` work | `OUT_OF_SCOPE_THIS_PHASE` | H10 froze the completed onboarding-continue chain and did not expose one exact later Section 03 voice winner | not available | Section 03 reopening is not lawful here |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | client-owned work remains downstream and non-authoritative | not available | wrong phase and wrong owner |

F) selected scope and dependency matrix

| item | classification | Section 04 position | dependency / guardrail |
|---|---|---|---|
| `RuntimeGovernanceRuntime::govern_voice_turn_execution` protected execution foundation | `IN_SCOPE_SECTION04_NEXT` | the first lawful H11 winner | must consume only the accepted Section 03 handoff shape |
| accepted Section 03 `ReadyForSection04Boundary` outcome | `IN_SCOPE_SECTION04_NEXT` as protected baseline | binding entry boundary into Section 04 | no pull-back into Section 03 and no bypass |
| accepted `RuntimeExecutionEnvelope` path | `IN_SCOPE_SECTION04_NEXT` as protected baseline | single canonical protected-execution carrier | no alternate envelope path |
| accepted runtime request / session stack | `IN_SCOPE_SECTION04_NEXT` as protected baseline | one canonical routing and session truth | no parallel request-family or session path |
| `governance_state` | `IN_SCOPE_SECTION04_NEXT` | the first Section 04 state to populate | must be the only newly populated protected state for the selected winner |
| `proof_state` | `DEFER_LATER_SECTION04` | later proof-governance boundary | do not require or populate it in the first winner |
| `artifact_trust_state` | `DEFER_LATER_SECTION04` | later artifact-trust boundary | do not require or populate it in the first winner |
| `identity_state` and `authority_state` | `DEFER_LATER_SECTION04` | later protected authority completion boundary | do not require or populate them in the first winner |
| `law_state` | `DEFER_LATER_SECTION04` | later law-completion boundary | do not require or populate it in the first winner |
| Section 05 persistence / sync | `DEFER_SECTION05` | downstream persistence layer | no Section 05 pull-forward |
| PH1.ONB business execution | `DEFER_LATER_RUNTIME` | downstream business runtime only | no business execution widening |
| PH1.COMP, memory, personality/emotion runtime | `DEFER_LATER_RUNTIME` | later runtime only | no later-runtime widening |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | non-authoritative client surface | no client-owned execution truth |

G) execution-surface / state-boundary matrix

| execution surface or state | current boundary posture | classification | H11 posture |
|---|---|---|---|
| `AdmissionState::ExecutionAdmitted` | already set by Section 03 at the handoff | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as the entry requirement for the selected winner |
| `ReadyForSection04Boundary` | current deterministic stop line from Section 03 | `IN_SCOPE_SECTION04_NEXT` as protected baseline | preserve as the only lawful entry boundary |
| `session_id`, `turn_id`, and `device_turn_sequence` on the admitted envelope | already available at the handoff | `IN_SCOPE_SECTION04_NEXT` | consume through the selected governance-first seam only |
| `persistence_state` posture on the admitted envelope | already available as a carried posture input | `IN_SCOPE_SECTION04_NEXT` as protected baseline | consume without promoting Section 05 execution |
| `governance_state` | unset at the handoff | `IN_SCOPE_SECTION04_NEXT` | first bounded Section 04 state to populate |
| `proof_state` | unset at the handoff | `DEFER_LATER_SECTION04` | do not populate in the first winner |
| `artifact_trust_inputs` | present as ingress-carried non-authoritative input | `DEFER_LATER_SECTION04` as protected downstream seam | preserve; do not collapse into the first governance slice |
| `artifact_trust_state` | unset at the handoff | `DEFER_LATER_SECTION04` | do not populate in the first winner |
| `identity_state` | unset at the handoff | `DEFER_LATER_SECTION04` | do not populate in the first winner |
| `authority_state` | unset at the handoff | `DEFER_LATER_SECTION04` | do not populate in the first winner |
| `law_state` | unset at the handoff | `DEFER_LATER_SECTION04` | do not populate in the first winner |
| `memory_state` and wider later-runtime execution state | unset at the handoff | `DEFER_LATER_RUNTIME` | keep unset |
| `/v1/onboarding/continue` compatibility execution | completed Section 03 chain | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen |
| generic deeper `/v1/voice/turn` modality semantics | separate later Section 03 concern only | `OUT_OF_SCOPE_THIS_PHASE` | do not reinterpret as this Section 04 winner |

H) repository workstream / file-impact matrix

| repository seam or file | role in H11 | classification | mandatory posture |
|---|---|---|---|
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H11_SECTION04_PROTECTED_AUTHORITY_BOUNDARY_AND_BUILD_PLAN.md` | the only artifact H11 authorizes | `IN_SCOPE_SECTION04_NEXT` | planning-only output |
| `docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H10_SLICE2P_SECTION03_BOUNDARY_AND_BUILD_PLAN.md` | frozen Section 03 boundary input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not reinterpret |
| `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md` | authoritative Section 04 law input | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only; do not widen |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` | canonical envelope and protected-state surface | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only for current repo truth |
| `crates/selene_os/src/runtime_request_foundation.rs` | canonical request-stack boundary into runtime ingress/execution | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only; no alternate path |
| `crates/selene_os/src/runtime_governance.rs` | source of the selected governance-first candidate and later governance candidates | `IN_SCOPE_SECTION04_NEXT` as repo-truth anchor | read only in H11; future implementation must stay bounded to the selected winner |
| `crates/selene_os/src/runtime_law.rs` | later law-completion surfaces | `DEFER_LATER_SECTION04` | read only; not the first winner |
| `crates/selene_os/src/runtime_ingress_turn_foundation.rs` | accepted Section 03 stop-line truth | `IN_SCOPE_SECTION04_NEXT` as protected baseline | read only; no Section 03 reopening |
| Section 05 persistence/sync files and workstreams | downstream persistence layer | `DEFER_SECTION05` | not part of H11 winner |
| Apple/client/app workstreams | non-target client surface | `OUT_OF_SCOPE_THIS_PHASE` | not part of H11 winner |

I) INTERNAL IMPLEMENTATION ORDER

H11 is planning only, but it freezes the future bounded implementation order for the selected
winner:

1. preserve the accepted Section 03 handoff at `ReadyForSection04Boundary` unchanged
2. reuse the canonical `RuntimeExecutionEnvelope` path and existing request/session seam unchanged
3. bind the first Section 04 implementation slice only to the governance-first protected execution
   seam exposed by `RuntimeGovernanceRuntime::govern_voice_turn_execution`
4. consume only the already-available admitted envelope posture required by the selected seam,
   including session, turn, device-turn, and persistence posture inputs
5. populate `governance_state` only
6. keep `proof_state`, `artifact_trust_state`, `identity_state`, `authority_state`, `law_state`,
   `memory_state`, and Section 05 persistence execution behavior downstream and unset
7. add bounded verification proving that the canonical handoff reaches governance-first execution
   without opening later Section 04 or Section 05 behavior
8. refuse any widening into generic Section 04, Section 05, reopened Section 03, or client-owned
   work

J) verification and acceptance matrix

| proof area | required verification | H11 acceptance condition |
|---|---|---|
| H10 boundary carry-forward proof | prove H10 still freezes the completed Section 03 onboarding-continue chain | H11 begins from the frozen Section 03 boundary, not from a reopened Section 03 slice |
| Section 03 handoff proof | prove current repo truth reaches `ReadyForSection04Boundary` with `AdmissionState::ExecutionAdmitted` | the selected winner is grounded in the actual boundary shape |
| governance-first candidate proof | prove `RuntimeGovernanceRuntime::govern_voice_turn_execution` is the first exact Section 04 surface that consumes the current handoff without requiring later states | the selected winner is exact and bounded |
| later proof-governance deferral proof | prove proof-governance surfaces require later proof-bearing posture | later proof-governance work remains deferred |
| later artifact-trust deferral proof | prove artifact-activation governance requires `artifact_trust_state` | later artifact-trust work remains deferred |
| later law-completion deferral proof | prove runtime law evaluation and completion rely on richer downstream protected state | law completion remains deferred |
| canonical-path preservation proof | prove one canonical request stack, one canonical envelope path, and one accepted Section 03 stop line remain binding | no alternate protected-execution path is implied |
| no Section 05 bleed proof | prove persistence and sync remain downstream only | H11 does not pull Section 05 forward |
| no later-runtime bleed proof | prove PH1.ONB business execution, PH1.COMP, memory, personality/emotion runtime, and Apple/client work remain deferred | H11 stays bounded |
| planning-artifact proof | prove the H11 document states CURRENT / TARGET / GAP explicitly and records one exact winner explicitly | H11 removes Section 04 ambiguity |
| cleanliness and readiness proof | prove H11 validates cleanly on a one-file doc-only tree | H11 is not complete until the planning artifact itself validates cleanly |

K) deferred-scope / guardrail matrix

| item | classification | mandatory guardrail |
|---|---|---|
| governance-first protected execution foundation via `RuntimeGovernanceRuntime::govern_voice_turn_execution` | `IN_SCOPE_SECTION04_NEXT` | this is the only lawful H11 winner |
| preserve accepted Section 03 boundary and canonical envelope path | `IN_SCOPE_SECTION04_NEXT` | do not reinterpret or replace accepted pre-authority truth |
| preserve canonical request / session stack | `IN_SCOPE_SECTION04_NEXT` | no alternate request-family or session path |
| populate `governance_state` first and only | `IN_SCOPE_SECTION04_NEXT` | do not silently populate later protected states |
| later proof-governance candidates | `DEFER_LATER_SECTION04` | do not pull proof-dependent work into the first winner |
| later artifact-trust governance | `DEFER_LATER_SECTION04` | do not pull artifact-trust work into the first winner |
| later identity / authority completion | `DEFER_LATER_SECTION04` | do not pull final authority posture into the first winner |
| later runtime law completion | `DEFER_LATER_SECTION04` | do not pull law completion into the first winner |
| generic Section 04 bucket | `DEFER_LATER_SECTION04` | do not widen into a bucket implementation |
| Section 05 persistence / sync execution | `DEFER_SECTION05` | do not pull persistence correctness forward |
| PH1.ONB business execution | `DEFER_LATER_RUNTIME` | no business execution widening |
| PH1.COMP, memory, personality/emotion runtime | `DEFER_LATER_RUNTIME` | no later-runtime widening |
| any Section 03 reopening, including `/v1/onboarding/continue` and generic deeper `/v1/voice/turn` work | `OUT_OF_SCOPE_THIS_PHASE` | do not reopen completed or unfrozen Section 03 concerns |
| Apple/client/app work | `OUT_OF_SCOPE_THIS_PHASE` | no client-owned execution truth |
| no alternate authority path | `IN_SCOPE_SECTION04_NEXT` | Section 04 must remain inside the canonical envelope path |

L) COMPLETION STANDARD

H11 is complete only when all of the following are true:

- H11 explicitly states that H10 froze the completed Section 03 onboarding-continue chain
- H11 explicitly states CURRENT / TARGET / GAP
- H11 freezes one exact Section 04 winner rather than a generic bucket
- the selected winner is the governance-first protected execution foundation anchored on
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`
- H11 explicitly preserves the canonical Section 03 handoff, request/session stack, and
  `RuntimeExecutionEnvelope` path
- H11 explicitly limits the first Section 04 winner to populating `governance_state`
- H11 explicitly defers later proof-governance, artifact-trust, identity/authority, and law
  completion work inside Section 04
- H11 explicitly defers Section 05 persistence/sync work
- H11 explicitly defers PH1.ONB business execution, PH1.COMP, memory, personality/emotion runtime,
  Apple/client/app work, and all Section 03 reopening
- the H11 planning artifact passes title, heading, token, and design-readiness validation on a
  clean tree

H11 is not complete if it merely repeats “Section 04 planning next,” if it leaves the first
protected winner implicit, if it silently widens into generic authority work, or if it reopens
completed Section 03 behavior.

M) PHASE BOUNDARY

H11 governs the first Section 04 protected/authority planning winner only.

H11 authorizes no implementation by itself, but it freezes the next lawful planning target as:

- the canonical Section 04 runtime-governance protected execution foundation via
  `RuntimeGovernanceRuntime::govern_voice_turn_execution`

H11 does not authorize:

- any reopening of `/v1/onboarding/continue`
- any reopening of generic deeper `/v1/voice/turn` work
- any generic Section 04 protected/authority bucket
- any later Section 04 proof-governance, artifact-trust, identity/authority, or law-completion
  work
- any Section 05 persistence / sync execution
- any PH1.ONB business execution
- any PH1.COMP, memory, personality/emotion runtime execution
- any Apple/client/app work

PHASE BOUNDARY:

- the completed Section 03 chain remains frozen under H10
- the first lawful Section 04 winner is governance-first protected execution only
- later Section 04 protected-execution slices remain unfrozen beyond this selected winner
- Section 05 and later runtime remain downstream
- any future implementation instruction must stay bounded to the selected governance-first winner
  and must not reopen completed Section 03 scope
