PHASE E5 — TESTS / DOCS / VERIFICATION BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- HEAD commit at start: `039fd6f498f2c708a84334318445116c8d366348`
- target file creation proof: this file is created in this run as the only planned file change
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E3_TONE_VS_LONG_TERM_BEHAVIOR_SEPARATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E4_SAFETY_LAW_MEMORY_CONTROLS_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_CLOSURE_EVIDENCE_MANIFEST.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_CLOSURE_EVIDENCE_PACK.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_TRACEABILITY_MATRIX.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_RESIDUAL_RISK_REGISTER.md`
  - `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B4_PARITY_TESTS_FAILURE_HANDLING_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B5_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
  - `docs/DB_WIRING/PH1_PERSONA.md`
  - `docs/DB_WIRING/PH1_EMO.md`
  - `docs/DB_WIRING/PH1_EMO_CORE.md`
  - `docs/DB_WIRING/PH1_EMO_GUIDE.md`
  - `docs/DB_WIRING/PH1_CONTEXT.md`
  - `docs/DB_WIRING/PH1_M.md`
  - `docs/DB_WIRING/PH1_LEARN.md`
  - `docs/DB_WIRING/PH1_POLICY.md`
  - `docs/DB_WIRING/PH1_GOV.md`
  - `docs/DB_WIRING/PH1_LAW.md`
  - `docs/DB_WIRING/PH1_COMP.md`
  - `docs/DB_WIRING/PH1_FEEDBACK.md`
  - `crates/selene_kernel_contracts/src/ph1persona.rs`
  - `crates/selene_kernel_contracts/src/ph1e.rs`
  - `crates/selene_kernel_contracts/src/ph1emocore.rs`
  - `crates/selene_kernel_contracts/src/ph1emoguide.rs`
  - `crates/selene_kernel_contracts/src/ph1context.rs`
  - `crates/selene_kernel_contracts/src/ph1m.rs`
  - `crates/selene_kernel_contracts/src/ph1learn.rs`
  - `crates/selene_kernel_contracts/src/ph1policy.rs`
  - `crates/selene_kernel_contracts/src/ph1comp.rs`
  - `crates/selene_kernel_contracts/src/ph1feedback.rs`
  - `crates/selene_kernel_contracts/src/runtime_governance.rs`
  - `crates/selene_kernel_contracts/src/runtime_law.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/lib.rs`
  - `crates/selene_storage/Cargo.toml`
  - `.github/workflows/ph1-readiness-guardrails.yml`
  - `scripts/selene_design_readiness_audit.sh`
  - `scripts/check_ph1_readiness_strict.sh`

B) PURPOSE
- Freeze the verification and closure plan for the already frozen E1-E4 Phase E personality work.
- Define exactly what must be proven, what failure classes must be injected, what artifacts must be produced, and what gates must pass before Phase E can be declared complete.
- Reuse existing Selene closure patterns rather than inventing a new verification architecture.

C) DEPENDENCY RULE
- E5 consumes the frozen outputs of E1, E2, E3, and E4 as non-negotiable design baselines.
- E5 may verify and document those baselines, but may not reinterpret their authority model, layer boundaries, mutability law, control classes, reset horizons, or persistence law.
- If Phase E verification exposes a mismatch, E5 records it as evidence and residual risk; it does not silently redesign upstream phases.

D) ARCHITECTURAL POSITION
- E5 is the closure phase for Phase E only.
- It sits above:
  - `Section 05` replay, dedupe, and recovery law for verification of reset/carry-over/re-entry behavior
  - `Section 09` governance enforcement for personality/adaptation control verification
  - `Section 10` deterministic computation for threshold and bounded-control verification
  - `Section 11` runtime-law posture for block, degrade, quarantine, and safe-fail verification
- E5 does not introduce new runtime behavior, storage semantics, or law/governance paths.

E) E1 / E2 / E3 / E4 ASSUMPTIONS CONSUMED
- E1 froze:
  - persona vs tone vs emotion vs memory-fed personalization vs learning-fed adaptation vs constraint layers
  - authoritative vs derived personality surfaces
  - mutability and long-term stability law
- E2 froze:
  - lawful adaptive inputs
  - allowed adaptive outputs
  - disallowed adaptation paths
  - deterministic scoring / thresholds / decay
  - runtime-only vs candidate-for-persisted adaptation boundary
- E3 froze:
  - runtime tone vs long-term behavior split
  - carry-over, reset, and persistence rules
  - disallowed cross-layer mutation
  - no-authority-bleed boundaries
- E4 froze:
  - safety / law / memory control classes
  - memory suppression / retention / sensitivity controls
  - policy / governance / runtime-law control posture
  - fail-closed, degrade, block, quarantine, and safe-fail rules

F) CURRENT TEST, DOC, CI, AND EVIDENCE SURFACES IN SCOPE
Current Repo Test / Evidence Surface Mapping

| repo surface or harness | current role | phase E relevance | evidence type | E5 use | notes / constraints |
| --- | --- | --- | --- | --- | --- |
| `crates/selene_storage/tests/db_wiring_ph1persona_tables.rs` | verifies PH1.PERSONA db wiring | E1 authority / persisted profile grounding | schema / wiring proof | baseline structural verification | covers storage surface existence, not runtime behavior semantics |
| `crates/selene_storage/tests/db_wiring_ph1m_tables.rs` | verifies PH1.M db wiring | E1/E3/E4 memory and suppression grounding | schema / wiring proof | memory constraint verification baseline | does not by itself prove suppression enforcement paths |
| `crates/selene_storage/tests/db_wiring_ph1j.rs` | verifies PH1.J proof/audit wiring | E4 evidence and proof-visibility expectations | schema / audit proof | proof/evidence baseline | evidence-only, never authority |
| `crates/selene_storage/tests/db_wiring_os_core_tables.rs` | verifies OS core tables | cross-phase runtime storage baseline | schema proof | indirect closure support | no direct personality semantics |
| `cargo test -p selene_kernel_contracts --lib` | contract and runtime-state tests | E1-E4 contract / control / posture verification | unit / contract evidence | contract-stratum verification | package-wide harness, exact target tests evolve |
| `cargo test -p selene_storage --test db_wiring_ph1persona_tables` | PH1.PERSONA storage verification | E1/E3 persistence boundary support | integration evidence | storage-stratum verification | wiring-only unless expanded later |
| `cargo test -p selene_storage --test db_wiring_ph1m_tables` | PH1.M storage verification | E3/E4 memory controls support | integration evidence | suppression / retention baseline | storage-level only |
| `cargo test -p selene_storage --test db_wiring_ph1j` | PH1.J verification | E4 proof/evidence closure | integration evidence | proof visibility baseline | no new proof architecture implied |
| `.github/workflows/ph1-readiness-guardrails.yml` | repo readiness guardrail workflow | Phase E closure hygiene | CI evidence | closure-gate support | currently generic; may not mention Phase E directly |
| `scripts/selene_design_readiness_audit.sh` | architecture/doc readiness audit | Phase E document alignment proof | audit evidence | docs/closure verification | design readiness only |
| `scripts/check_ph1_readiness_strict.sh` | strict readiness aggregate | Phase E closure-pack command reference | command evidence | final command manifest source | combines repo-wide checks |
| `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_*` closure docs | established closure artifact pattern | Phase E closure artifact template | docs/evidence pattern | traceability / manifest / risk register template | pattern reuse only |
| `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_*` closure plan | reusable closure structure | Phase E verification planning model | planning evidence | closure-plan pattern | reuse, not redesign |
| `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D5_*` closure plan | session closure pattern | Phase E closure structure parity | planning evidence | closure-gate parity | reuse, not redesign |

G) PHASE E ACCEPTANCE MODEL
- Phase E is complete only when all frozen E1-E4 obligations are verified through explicit evidence, not assumed correctness.
- Phase E acceptance requires:
  - layer-boundary verification
  - adaptive-input/output and hard-limit verification
  - reset/carry-over/persistence verification
  - safety/law/memory control verification
  - docs/traceability/evidence/risk completion
- A clean closure state requires both:
  - execution evidence
  - documentation evidence
- Missing either side keeps Phase E open.

H) TEST STRATA AND COVERAGE MODEL
Test Strata Matrix

| stratum | purpose | representative targets | failure classes covered | reset / carry-over / persistence covered | governance / law / memory / policy checks | expected evidence |
| --- | --- | --- | --- | --- | --- | --- |
| contract / unit | verify frozen E1-E4 contracts and enums | `selene_kernel_contracts` lib tests for persona/emotion/context/policy/law/governance packets and state structs | invalid control class, missing bounds, illegal field combinations | reset/persist semantics at contract level | yes | contract test logs, assertions, snapshots if used |
| repository / storage integration | verify authoritative and evidence surfaces | `db_wiring_ph1persona_tables`, `db_wiring_ph1m_tables`, `db_wiring_ph1j` | missing storage surfaces, wrong authority mapping, missing evidence rows | persisted-candidate boundary support | indirect | test output, schema checks |
| runtime / behavior integration | verify runtime personality/adaptation behavior stays within frozen law | contract/runtime execution paths around persona, emotion, context, policy, law | illegal runtime adaptation, illegal tone carry, bounded-output breach | yes | yes | runtime test transcripts, state assertions |
| reset / carry-over / persistence verification | prove E3 reset horizons and E2 persistence boundaries | targeted runtime + storage scenarios | illegal carry-over, illegal persistence, stale long-term promotion | primary stratum | yes where gated | scenario evidence, before/after state capture |
| fail-closed / degradation / escalation | prove E4 control posture | policy/gov/law control scenarios | fail-closed, degrade, block, quarantine, safe-fail | where control depends on carry-over or persistence | primary stratum | outcome logs, posture assertions |
| docs / evidence / closure verification | prove closure packet completeness | readiness scripts, traceability matrix, manifest, risk register | missing evidence, missing docs, stale traceability | closure confirmation only | yes through document checks | command record, manifest, evidence pack |

I) E1 PERSONALITY ARCHITECTURE VERIFICATION SCOPE
- Verify E1 frozen obligations:
  - layer separation is preserved
  - authoritative vs derived surfaces stay explicit
  - mutability law is preserved
  - governance/law/memory/identity/policy constraints do not become alternate persona authority
- Minimum evidence:
  - contract-level proof that persona core, tone, emotion, memory-fed personalization, learning-fed adaptation, and constraint-layer outputs are not collapsed into one mutable authority
  - storage proof that `Ph1PersonaRepo` and PH1.M surfaces stay authoritative only in their frozen scopes
  - traceability links from E1 sections to verification artifacts

J) E2 BOUNDED ADAPTIVE BEHAVIOR VERIFICATION SCOPE
- Verify E2 frozen obligations:
  - lawful vs forbidden adaptive inputs
  - allowed adaptive outputs and hard limits
  - deterministic scoring / threshold / decay model
  - runtime-only vs candidate-for-persisted adaptation boundary
- Minimum evidence:
  - proof that allowed outputs stay within bounded ranges
  - proof that forbidden inputs and forbidden output classes are rejected
  - proof that deterministic thresholds, minimum confidence, and decay/reset rules produce the same outcome for the same inputs

K) E3 TONE VS LONG-TERM BEHAVIOR SEPARATION VERIFICATION SCOPE
- Verify E3 frozen obligations:
  - runtime tone vs long-term behavior split
  - turn/session/device reset horizons
  - carry-over rules
  - persistence boundaries
  - no cross-layer mutation
- Minimum evidence:
  - turn-boundary reset proof
  - session-boundary reset proof
  - cross-device identity-scoped carry-over proof
  - proof that runtime-only tone signals never persist as stable behavior
  - proof that candidate long-term behavior is bounded and does not rewrite persona core

L) E4 SAFETY / LAW / MEMORY CONTROL VERIFICATION SCOPE
- Verify E4 frozen obligations:
  - action control classes
  - memory suppression / retention / sensitivity controls
  - policy/governance/runtime-law gates
  - fail-closed / degrade / block / quarantine / safe-fail behavior
  - no-authority-bleed and no-memory-truth-rewrite law
- Minimum evidence:
  - proof that unsafe influence attempts are blocked or degraded correctly
  - proof that memory suppression prevents influence where required
  - proof that governance/law posture is visible where E4 made it mandatory
  - proof that no control participant becomes persona or memory truth writer

M) FAILURE INJECTION, RESET, CARRY-OVER, PERSISTENCE, AND RECOVERY VERIFICATION
Failure Injection / Partial-Success Matrix

| case | source phase | test stratum | expected authoritative outcome | expected governance / law / memory outcome | expected fail-closed / retry / degrade / block behavior | evidence required |
| --- | --- | --- | --- | --- | --- | --- |
| runtime tone tries to persist illegally | E3 | reset / carry-over / persistence | no persisted stable-behavior mutation | no governance/law override required unless protected path is affected | fail closed or refuse persistence | before/after storage proof and refusal output |
| stable behavior candidate lacks lawful promotion path | E2 / E3 | runtime / behavior integration | candidate remains non-authoritative | governance/law may remain visibility-only | bounded refusal; no silent promotion | runtime outcome plus storage non-mutation proof |
| memory suppression blocks influence | E4 | fail-closed / degradation / escalation | memory truth remains unchanged; influence omitted | memory suppression visibility must align | block suppressed influence; no best-effort substitute | suppression evidence and output comparison |
| identity mismatch blocks personalization | E4 | fail-closed / degradation / escalation | no identity-scoped personalization applied | identity/law posture visible where required | block or degrade by frozen rule | identity gate evidence |
| cross-user memory influence blocked | E4 | fail-closed / degradation / escalation | no foreign memory influence applied | governance/law visibility when protected | fail closed | memory-scope audit evidence |
| feedback nomination exceeds bounded path | E2 / E4 | runtime / behavior integration | no direct persona-core rewrite | policy/gov/law constraints remain intact | bounded refusal or degrade | nomination trace plus non-mutation proof |
| learning nomination exceeds bounded path | E2 / E4 | runtime / behavior integration | no direct long-term behavior rewrite | visibility if protected | bounded refusal or quarantine if risk posture requires | learning trace plus non-mutation proof |
| policy block fires | E4 | fail-closed / degradation / escalation | authoritative profile/memory truth preserved | policy/governance visible; law may enforce block | block or fail closed | policy decision evidence |
| governance visibility missing where required | E4 | fail-closed / degradation / escalation | authoritative truth preserved | missing governance visibility remains explicit | protected completion withheld or degraded | completion-gate evidence |
| runtime-law posture missing where required | E4 | fail-closed / degradation / escalation | authoritative truth preserved | law posture absent remains explicit | fail closed, degrade, or block per frozen law | runtime-law evidence |
| bounded retry exhausted and degradation or block begins | E4 | fail-closed / degradation / escalation | no unlawful mutation | governance/law escalation visible | degrade or block, never silent continue | retry history and final posture proof |

Reset / Carry-Over / Persistence / Dedupe Matrix

| path | source phase | authoritative source of truth | reset / carry-over / persistence rule to verify | duplicate / stale / re-entry behavior if relevant | evidence required | closure impact |
| --- | --- | --- | --- | --- | --- | --- |
| runtime tone phrasing reset at new turn | E3 | runtime-only tone outputs plus E3 reset law | must reset at turn boundary | stale re-entry may not resurrect prior transient tone | per-turn output comparison | required |
| session carry-over of bounded behavior candidate | E3 | bounded candidate source plus PH1.M / persona references | only allowed candidate behavior may carry across session | duplicate nomination may reuse prior bounded result, not amplify it | session-to-session trace | required |
| cross-device carry-over under same identity scope | E3 / D1 | identity-scoped authoritative truth | only lawful carry-over may propagate across devices | stale or foreign-device replay may not widen behavior | cross-device scenario proof | required |
| suppressed memory remains non-influential after replay | E4 / C4 | PH1.M suppression truth | suppression survives re-entry and replay | duplicate replay may not reapply suppressed influence | suppression/replay evidence | required |
| persisted promotion candidate remains candidate until lawful path completes | E2 / E3 / E4 | candidate storage or proposal truth only | no silent promotion to authority | duplicate promotion request may not double-apply | persistence audit evidence | required |

N) DOCS, TRACEABILITY, EVIDENCE PACK, AND RESIDUAL-RISK DELIVERABLES
Docs / Evidence Deliverables Matrix

| deliverable | source phase(s) | purpose | required contents | closure gate supported | owner or producing step |
| --- | --- | --- | --- | --- | --- |
| Phase E docs update set | E1-E4 | align canonical docs with verified implementation state | final references to frozen E1-E4 plus any implementation-phase doc deltas | documentation gate | closure-doc step |
| Phase E traceability matrix | E1-E4 | map frozen obligations to verification evidence | every major requirement, source lines/sections, verification artifact, result | traceability gate | closure-doc step |
| Phase E closure evidence manifest | E1-E4 | enumerate all proof artifacts | commands, outputs, test identifiers, screenshots/logs if needed, timestamps | evidence gate | closure-pack step |
| Phase E closure evidence pack | E1-E4 | human-reviewable proof packet | command record, selected outputs, summary, pass/fail table | evidence gate | closure-pack step |
| Phase E residual risk register | E1-E4 | record remaining bounded risks | unresolved gaps, why bounded, mitigation owner, freeze decision note | residual-risk gate | closure-doc step |
| verification command record | E1-E4 | exact reproducibility log | exact commands, order, env assumptions, pass/fail result | command-record gate | verification run step |
| approval checklist | E1-E4 | explicit freeze decision checklist | per-gate checkbox with evidence refs and decision note | final freeze gate | closure-doc step |

O) VERIFICATION COMMANDS, MANIFESTS, AND CLOSURE PACK
- The command record must include, at minimum:
  - all contract / unit test commands used for E1-E4 verification
  - all storage integration commands used for PH1.PERSONA / PH1.M / PH1.J validation
  - any runtime behavior scenario command or scripted verification path
  - readiness/doc audit commands
- Minimum command families to reference in the closure pack:
  - `cargo test -p selene_kernel_contracts --lib`
  - `cargo test -p selene_storage --test db_wiring_ph1persona_tables`
  - `cargo test -p selene_storage --test db_wiring_ph1m_tables`
  - `cargo test -p selene_storage --test db_wiring_ph1j`
  - `bash scripts/selene_design_readiness_audit.sh`
  - `bash scripts/check_ph1_readiness_strict.sh`
- The manifest must distinguish:
  - existing harnesses
  - scenario evidence produced during Phase E verification
  - closure documents authored during the closeout

P) EXPLICIT NON-GOALS / DEFERRED ITEMS
- E5 does not redesign E1, E2, E3, or E4.
- E5 does not implement runtime behavior.
- E5 does not create a new governance, law, policy, or proof subsystem.
- E5 does not expand Phase E scope into new personality architecture or new adaptive behavior law.
- E5 does not rewrite D or C phase semantics.
- Any uncovered runtime gap discovered during verification is recorded as evidence or residual risk first; it is not silently redesigned inside E5.

Q) COMPLETION CRITERIA
Phase E Requirement → Verification Coverage Matrix

| requirement or frozen plan obligation | source phase | verification stratum | failure injection needed | governance / law / memory / policy check needed | evidence artifact produced | closure gate impact |
| --- | --- | --- | --- | --- | --- | --- |
| canonical personality layer split holds | E1 | contract / unit | no | yes | traceability row, contract evidence | required |
| bounded adaptive inputs/outputs obey frozen law | E2 | runtime / behavior integration | yes | yes | runtime evidence, command record | required |
| tone vs long-term behavior separation holds | E3 | reset / carry-over / persistence verification | yes | yes | carry-over/reset evidence | required |
| safety / law / memory controls hold | E4 | fail-closed / degradation / escalation | yes | primary | control evidence, posture logs | required |
| no-authority-bleed / no-memory-truth-rewrite holds | E3 / E4 | fail-closed / degradation / escalation | yes | yes | refusal evidence, non-mutation proof | required |
| closure docs / manifest / evidence pack / risk register complete | E1-E4 | docs / evidence / closure verification | no | indirect | closure documents | required |

E5 Closure Gate Matrix

| closure gate | required test/doc/evidence input | source phase(s) | pass condition | fail condition | freeze impact |
| --- | --- | --- | --- | --- | --- |
| architecture verification gate | E1 layer/authority/mutability evidence | E1 | all frozen E1 obligations proven with explicit evidence | any layer or authority ambiguity remains unproven | Phase E cannot freeze |
| bounded adaptation gate | E2 input/output/hard-limit/scoring evidence | E2 | lawful inputs, bounded outputs, and deterministic thresholds all proven | any adaptive ambiguity or uncontrolled output remains | Phase E cannot freeze |
| separation gate | E3 reset/carry-over/persistence/no-drift evidence | E3 | runtime-only vs candidate persistence law proven | any carry-over or persistence ambiguity remains | Phase E cannot freeze |
| control gate | E4 control-class, suppression, policy/gov/law evidence | E4 | allowed/bounded/blocked/degraded/fail-closed posture proven | any control-class ambiguity remains | Phase E cannot freeze |
| failure-injection gate | injected failure scenarios and outcomes | E2-E4 | all required injected cases show expected refusal/degrade/block behavior | any injected case contradicts frozen law or lacks evidence | Phase E cannot freeze |
| docs/evidence gate | traceability matrix, manifest, evidence pack, risk register, checklist | E1-E4 | all artifacts exist, cross-reference correctly, and are complete | any artifact missing or untraceable | Phase E cannot freeze |
| command-record gate | exact reproducible command record | E1-E4 | command record is complete and reproducible | missing or partial command record | Phase E cannot freeze |
