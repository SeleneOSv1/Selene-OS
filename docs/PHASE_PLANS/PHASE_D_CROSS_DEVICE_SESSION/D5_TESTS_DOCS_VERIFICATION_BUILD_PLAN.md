PHASE D5 — TESTS / DOCS / VERIFICATION BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `f6d59b1e4d5465f0ef6b08346d23e861eba0934e`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D2_ATTACH_RECOVER_DETACH_CONTRACT_FIXES_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D3_RUNTIME_AND_PERSISTENCE_WIRING_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_CLOSURE_EVIDENCE_MANIFEST.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_CLOSURE_EVIDENCE_PACK.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_TRACEABILITY_MATRIX.md`
  - `docs/PHASE_PLANS/PHASE_A_ARTIFACT_TRUST/A6_PHASE_A_RESIDUAL_RISK_REGISTER.md`
  - `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B4_PARITY_TESTS_FAILURE_HANDLING_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE/B5_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_02.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
  - `docs/DB_WIRING/PH1_L.md`
  - `docs/DB_WIRING/PH1_LEASE.md`
  - `docs/DB_WIRING/PH1_MULTI.md`
  - `docs/DB_WIRING/PH1_OS.md`
  - `docs/DB_WIRING/PH1_GOV.md`
  - `docs/DB_WIRING/PH1_LAW.md`
  - `docs/DB_WIRING/PH1_J.md`
  - `crates/selene_kernel_contracts/src/runtime_execution.rs`
  - `crates/selene_kernel_contracts/src/runtime_governance.rs`
  - `crates/selene_kernel_contracts/src/runtime_law.rs`
  - `crates/selene_kernel_contracts/src/ph1l.rs`
  - `crates/selene_kernel_contracts/src/ph1lease.rs`
  - `crates/selene_kernel_contracts/src/ph1multi.rs`
  - `crates/selene_kernel_contracts/src/ph1os.rs`
  - `crates/selene_kernel_contracts/src/ph1_voice_id.rs`
  - `crates/selene_storage/src/ph1f.rs`
  - `crates/selene_storage/src/ph1j.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_os/src/device_artifact_sync.rs`
  - `crates/selene_storage/Cargo.toml`
  - `.github/workflows/ph1-readiness-guardrails.yml`
  - `scripts/selene_design_readiness_audit.sh`
  - `scripts/check_ph1_readiness_strict.sh`

B) PURPOSE
- D5 is the Phase D closure plan.
- D5 does not redesign D1, D2, D3, or D4. It proves those frozen outputs were implemented correctly.
- D5 defines the verification model, evidence model, and freeze gates that must all pass before Phase D is considered complete.
- D5 closes Phase D only when cross-device session consistency, contract normalization, runtime/persistence wiring, governance/law/proof alignment, replay/re-entry behavior, fail-closed posture, and closure documentation all satisfy one explicit evidence packet.
- If implementation diverges from any frozen D1-D4 rule, the correct D5 result is `FAIL_CLOSURE`, not reinterpretation.

C) DEPENDENCY RULE
- D5 depends on the frozen outputs of:
  - D1 canonical cross-device session consistency law
  - D2 attach / recover / detach contract law
  - D3 runtime and persistence wiring law
  - D4 governance / runtime-law / proof alignment law
- D5 also depends on:
  - Section 04 verification-before-authority law
  - Section 05 replay / dedupe / recovery law
  - Section 09 governance enforcement
  - Section 11 runtime-law posture and fail-closed rules
  - Phase C C5 as the reusable closure pattern for traceability, evidence packaging, residual risk, and freeze gating
- D5 may not:
  - create new session states
  - create new attach outcome families
  - create new governance or law response classes
  - create a new proof path
  - convert docs closure into runtime authority
- If D1-D4 implementation cannot satisfy frozen law, D5 must fail closure rather than normalizing the drift.

D) ARCHITECTURAL POSITION
- D5 is the last planning lane in Phase D.
- D5 sits after frozen cross-device semantics, contract fixes, runtime/persistence wiring, and governance/law alignment.
- D5 is verification-only, evidence-only, and freeze-gating only.
- D5 never becomes a runtime writer, persistence writer, governance writer, or law writer.
- D5 consumes authoritative truth from D3 and D4 implementation surfaces, then proves:
  - D1 authority vs visibility rules remain intact
  - D2 canonical fields and outcomes remain intact
  - D3 materialization and replay/re-entry ordering remain intact
  - D4 governance/law/proof alignment and protected completion remain intact
  - documentation and evidence artifacts tell one coherent Phase D story

E) D1 / D2 / D3 / D4 ASSUMPTIONS CONSUMED
- D1 assumptions consumed:
  - cloud-authoritative session truth stays in PH1.L/session storage
  - the canonical identity tuple remains:
    - `(session_id, turn_id, device_id, actor_identity_scope, platform_context, device_turn_sequence, owning_node_or_lease_ref)`
  - attach / resume / recover / detach remain distinct
  - stale, retry, duplicate, failover, transfer, and ownership-uncertainty rules are frozen
- D2 assumptions consumed:
  - canonical contract fields are frozen:
    - `session_id`
    - `turn_id`
    - `device_id`
    - `device_turn_sequence`
    - `actor_identity_scope`
    - `platform_context`
    - `session_attach_outcome`
    - `attach_role`
    - `coordination_state`
    - `consistency_level`
    - `owning_node_or_lease_ref`
  - canonical blocked outcome families and fail-closed cases are frozen
  - platform or identity may never become alternate session authority
- D3 assumptions consumed:
  - request / ingress to envelope to PH1.L/session storage to reconciliation order is frozen
  - authoritative versus derived / visibility surfaces are frozen
  - replay, stale, retry, duplicate, and re-entry must reread authoritative truth
  - blocked outcomes must already be materialized in runtime/persistence surfaces
- D4 assumptions consumed:
  - protected cross-device action classes are frozen
  - governance visibility, law posture, and proof/completion requirements are frozen
  - authoritative session truth survives visibility failure
  - quarantine, safe-mode, and block/degrade posture remain downstream posture, not alternate authority
- D5 inherits all of the above as frozen law and must not fork or soften them locally.

F) CURRENT TEST, DOC, CI, AND EVIDENCE SURFACES IN SCOPE
- Current repo truth already provides reusable verification lanes.
- Those lanes are not yet a Phase D closure system by themselves.
- D5 uses them as baseline harnesses and evidence sources, then defines the additional Phase D coverage and closure outputs required before freeze.
- Current CI truth is limited:
  - `.github/workflows/ph1-readiness-guardrails.yml` exists
  - shell readiness/guardrail scripts exist
  - storage db-wiring tests exist
  - contract/runtime unit and integration tests exist
  - Phase A and Phase C closure documents exist as the reusable evidence-pack pattern
- There is no dedicated Phase D closure pack yet. D5 defines it.

Current Repo Test / Evidence Surface Mapping
| repo surface or harness | current role | phase D relevance | evidence type | D5 use | notes / constraints |
| --- | --- | --- | --- | --- | --- |
| `crates/selene_storage/tests/db_wiring_ph1l_tables` | deterministic storage wiring coverage for PH1.L session tables | D1 authority, D3 PH1.L/session storage materialization | storage-schema and authority-shape evidence | baseline proof that PH1.L tables exist and stay distinct from visibility surfaces | does not prove runtime attach semantics by itself |
| `crates/selene_storage/tests/db_wiring_os_core_tables` | deterministic storage wiring coverage for `os_core` tables | D1 and D3 current-row session truth | storage wiring evidence | baseline proof for `os_core.sessions` authority lane | does not prove role/consistency/coordination semantics |
| `crates/selene_storage/tests/db_wiring_ph1j` | deterministic storage wiring coverage for PH1.J audit/proof surfaces | D4 proof visibility and replay/verify continuity | proof-surface wiring evidence | baseline proof that proof refs and audit rows exist and stay non-authoritative | does not by itself prove protected completion |
| `crates/selene_storage/tests/db_wiring_ph1w_tables` | deterministic storage wiring coverage for wake/device runtime surfaces | D3/D4 replay, queue, and runtime reconciliation precedent | storage/runtime wiring evidence | baseline support for replay/visibility checks where session flows reuse shared persistence vocabulary | D5 must keep Phase D semantics separate from wake authority |
| `crates/selene_storage/tests/db_wiring_ph1vid_tables` | deterministic storage wiring coverage for identity/device surfaces | D2 actor/device binding, D3 binding propagation, D4 protected identity checks | identity-surface wiring evidence | baseline proof for identity-linked session checks | identity surfaces remain subordinate to session truth |
| `crates/selene_storage/tests/db_wiring_ph1f` | deterministic storage wiring coverage for PH1.F current-row structures, including `SessionRecord` shape | D3 session current-row materialization and replay anchors | storage contract evidence | baseline proof for `SessionRecord`-backed authoritative shape | does not prove runtime ordering by itself |
| `cargo test -p selene_kernel_contracts --lib` | contract/unit coverage for runtime execution, governance, law, session, lease, and identity types | D1 tuple law, D2 contract field/outcome law, D4 governance/law/proof carriage law | contract invariant evidence | verifies typed fields, enums, refusal families, and response classes stay deterministic | package-wide results must be paired with focused traceability rows |
| `cargo test -p selene_os --lib` | runtime integration coverage | D3 ingress/runtime/session propagation and D4 escalation posture | runtime integration evidence | verifies runtime-session behavior and failure handling from current runtime lanes and future additive tests | package-wide result is necessary but not sufficient |
| `crates/selene_os/src/device_artifact_sync.rs` tests | concrete retry, queue, dead-letter, partial-success, and ACK-loss integration examples | D3 replay/re-entry/reconciliation strategy precedent, D5 failure-injection pattern source | worker/runtime scenario evidence | reusable failure-handling pattern for Phase D coverage | D5 must adapt the pattern to session semantics, not wake semantics |
| `crates/selene_kernel_contracts/src/runtime_execution.rs` tests | runtime envelope and persistence/proof/governance/law carriage coverage | D2 field normalization, D3 envelope materialization, D4 protected completion carriage | runtime contract evidence | baseline and future additive proof for envelope invariants | current tests do not by themselves close Phase D |
| `scripts/selene_design_readiness_audit.sh` | repo-wide design/readiness/guardrail sweep | D5 docs alignment, evidence hygiene, clean-tree proof | shell verification evidence | readiness gate input for the final closure packet | does not substitute for session-specific failure injection |
| `scripts/check_ph1_readiness_strict.sh` | strict clean-tree/readiness guardrail wrapper | D5 freeze hygiene and command-record evidence | shell verification evidence | final readiness gate input for closure pack | still not a replacement for D1-D4 behavior proof |
| `.github/workflows/ph1-readiness-guardrails.yml` | current CI guardrail lane | D5 CI visibility baseline | CI execution evidence | proves at least one enforceable verification lane exists in CI | D5 does not assume this workflow alone closes Phase D |
| Phase A closure artifacts (`A6_*`) | canonical manifest, evidence pack, traceability, and residual-risk pattern | D5 closure-document structure | closure-artifact template evidence | exact evidence-pack pattern D5 must follow for Phase D | pattern source only; D5 must produce Phase D-specific outputs later |
| Phase B docs/verification closure docs (`B4`, `B5`) | failure-matrix and verification-pack structure pattern | D5 failure-injection and docs-closure structure | plan-pattern evidence | style and matrix precedent for D5 | pattern source only; D5 remains Phase D-specific |
| Phase C closure plan (`docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`) | reusable closure architecture for lifecycle planning | D5 reusable closure model for gates and evidence | plan-pattern evidence | exact immediate structural precedent for Phase D closure planning | D5 must adapt it to session/governance/law scope |

G) PHASE D ACCEPTANCE MODEL
- Phase D closes only on proof, not on narrative confidence.
- D5 must require one stable cross-reference system spanning:
  - `phased_requirement_id`
  - `phased_test_id`
  - `phased_failure_case_id`
  - `phased_replay_case_id`
  - `phased_evidence_id`
  - `phased_doc_artifact_id`
  - `phased_closure_gate_id`
- Every critical frozen obligation from D1, D2, D3, and D4 must map to:
  - one verification stratum
  - any required failure injection
  - any required governance / law / proof check
  - one or more evidence artifacts
  - one or more closure gates
- Untraced obligations are not considered closed.

Phase D Requirement → Verification Coverage Matrix
| requirement or frozen plan obligation | source phase (D1 / D2 / D3 / D4) | verification stratum | failure injection needed | proof / governance / law check needed | evidence artifact produced | closure gate impact |
| --- | --- | --- | --- | --- | --- | --- |
| session authority stays cloud-authoritative and visibility surfaces never reverse-authorize it | D1 | contract / unit plus repository / storage integration | YES | conditional when D4 makes it protected | authority-boundary evidence set plus traceability rows | blocks all gates if violated |
| canonical identity tuple stays intact across runtime and storage | D1 | contract / unit plus runtime / session integration | YES: mismatch and stale cases | conditional under protected identity-sensitive actions | tuple verification evidence set | blocks D1/D2/D3 gates |
| attach / resume / recover / detach semantics and blocked outcome families stay canonical | D2 | contract / unit plus runtime / session integration | YES | conditional when D4 makes it protected | contract outcome evidence set | blocks D2 closure gate |
| request → envelope → PH1.L/session storage → persistence order stays intact | D3 | runtime / session integration plus repository / storage integration | YES: partial-success and visibility lag | YES where D4 requires protected completion | D3 wiring evidence set | blocks D3 closure gate |
| blocked/fail-closed outcomes materialize without alternate-authority leak | D3 | fail-closed / escalation plus runtime / session integration | YES | YES for protected classes | blocked-outcome evidence set | blocks D2/D3/D4 gates |
| replay / re-entry / dedupe reuse authoritative truth and never duplicate session mutation | D3 | replay / re-entry / dedupe | YES | conditional when protected visibility is incomplete | replay reconciliation evidence set | blocks replay gate |
| protected cross-device actions obey governance visibility, law posture, and proof/completion law | D4 | fail-closed / escalation plus runtime / session integration | YES | YES | protected-completion evidence set | blocks D4 closure gate |
| ownership uncertainty, transfer pending, and failover recovering escalate deterministically | D4 | replay / re-entry / dedupe plus fail-closed / escalation | YES | YES | escalation posture evidence set | blocks D4 closure gate |
| docs, traceability, manifest, evidence pack, residual risk, command record, and approval checklist tell one coherent Phase D story | D5 | docs / evidence / closure verification | NO | NO | closure manifest, evidence pack, risk register, command record, checklist | blocks Phase D freeze gate |

H) TEST STRATA AND COVERAGE MODEL
- D5 defines six verification strata.
- Every Phase D obligation must be mapped into at least one stratum.
- Cross-stratum proof is required where a single behavior crosses contract, storage, runtime, and governance/law layers.
- No stratum may claim authority over another stratum’s source of truth.

Test Strata Matrix
| stratum | purpose | representative targets | failure classes covered | replay / recovery covered | proof / governance / law checks | expected evidence |
| --- | --- | --- | --- | --- | --- | --- |
| contract / unit | prove frozen tuple, field, outcome, role, coordination, and law-response contracts stay deterministic | `selene_kernel_contracts` lib tests rooted in `ph1l`, `ph1lease`, `runtime_execution`, `runtime_governance`, `runtime_law`, `ph1_voice_id` | invalid field sets, invalid outcome families, invalid response classes, bad identity/platform binding | limited to contract-level duplicate/stale determinism | validates proof/gov/law field shape and required-field completeness | contract test result record plus traceability mapping |
| repository / storage integration | prove authoritative PH1.L/session-storage truth and db-wiring shape stay intact | `db_wiring_ph1l_tables`, `db_wiring_os_core_tables`, `db_wiring_ph1j`, `db_wiring_ph1f`, `db_wiring_ph1vid_tables` | wrong-row ownership, missing session fields, stale current-row assumptions, proof-row shape drift | verifies rebuild/readback source precedence | verifies PH1.J rows remain evidence-only | storage evidence packet plus wiring sweep record |
| runtime / session integration | prove D2/D3 session ingress, envelope, PH1.L, and runtime carriage behavior | `selene_os` lib tests, `runtime_execution` tests, additive Phase D runtime cases | attach/recover/detach failure, visibility lag, blocked outcomes, degraded recovery, transfer/failover conflicts | yes | yes | runtime scenario evidence set |
| replay / re-entry / dedupe | prove same-sequence retry reuse, stale rejection, fresh-state request, and partial-success repair | Section 05-aligned replay selectors, additive D3 session replay tests | duplicate retry, stale message, missing visibility backfill, reconnect after partial success, cross-device conflict | full scope | yes when protected completion is involved | replay matrix with authoritative-source snapshots |
| fail-closed / escalation | prove block/degrade/quarantine/safe-mode posture and bounded refusal behavior | runtime governance/law tests, storage + visibility failure drills, identity/platform mismatch drills | governance failure, law failure, proof visibility failure, ownership uncertainty, transfer/failover escalation | yes | yes | fail-closed evidence set with escalation outcome record |
| docs / evidence / closure verification | prove traceability, docs alignment, manifest completeness, residual-risk discipline, and freeze gating | docs sweeps, command-record checks, readiness scripts, approval checklist | stale docs, missing evidence, untraced requirements, dirty-tree evidence attempts | N/A | N/A | closure manifest, evidence pack, residual risk register, approval checklist |

I) D1 CROSS-DEVICE CONSISTENCY VERIFICATION SCOPE
- D5 must verify the following D1 invariants as implemented truth:
  - cloud-authoritative session truth remains authoritative
  - device, session, turn, actor, platform, and ownership tuple fields remain stable
  - attach / resume / recover / detach semantics remain distinct
  - stale, retry, duplicate, failover, transfer, and ownership-uncertainty law stays deterministic
  - identity, platform, memory, proof, governance, and law remain subordinate to canonical session truth
- D5 D1 verification must explicitly cover:
  - cross-device session reuse without authority drift
  - same-device retry after authoritative success
  - stale device message refusal
  - simultaneous multi-device submission
  - reconnect after partial success
  - cross-device resume conflict
  - ownership uncertainty preserving authoritative truth

J) D2 ATTACH / RECOVER / DETACH CONTRACT VERIFICATION SCOPE
- D5 must verify the following D2 invariants as implemented truth:
  - canonical field set remains intact across request, envelope, and PH1.L/session storage
  - canonical outcome families remain intact
  - blocked / fail-closed outcomes remain explicit and reason-coded
  - actor/device/platform binding rules remain fail-closed
  - `attach_role`, `coordination_state`, `consistency_level`, and `owning_node_or_lease_ref` remain cloud-authored or derived only
- D5 D2 verification must explicitly cover:
  - attach with clean identity/platform posture
  - attach blocked by identity mismatch
  - attach blocked by platform mismatch
  - recover accepted versus recovery-window-closed refusal
  - detach accepted versus `DETACH_NOT_LAWFUL_BLOCKED`
  - transfer pending blocking attach
  - failover recovering degrading or blocking attach as frozen

K) D3 RUNTIME AND PERSISTENCE WIRING VERIFICATION SCOPE
- D5 must verify the following D3 invariants as implemented truth:
  - ingress normalization feeds runtime envelope deterministically
  - runtime envelope materializes the frozen D2 field set and outcomes
  - PH1.L/session storage remains the authoritative session writer
  - persistence/reconciliation stays subordinate and replay-safe
  - blocked outcomes materialize in runtime/persistence surfaces without alternate authority leak
  - re-entry / replay reread authoritative session truth before any repair
- D5 D3 verification must explicitly cover:
  - request to envelope field propagation
  - envelope to PH1.L/session storage authoritative write order
  - attach accepted but persistence visibility lags
  - same-sequence retry reusing authoritative result
  - stale sequence blocked before mutation
  - reconnect after partial success repairing visibility only
  - ownership/lease posture materialized without inventing a second ownership writer

L) D4 LAW / GOVERNANCE / PROOF VERIFICATION SCOPE
- D5 must verify the following D4 invariants as implemented truth:
  - protected cross-device action classes match frozen D4
  - governance visibility is present where required
  - runtime-law posture is present where required
  - proof visibility is present where required
  - protected completion is withheld until required visibility succeeds
  - authoritative session truth survives delayed or failed governance/law/proof visibility
- D5 D4 verification must explicitly cover:
  - ordinary non-protected attach/resume completion
  - protected attach/recover/role-change completion gating
  - ownership uncertainty, transfer pending, and failover recovering posture
  - stale/duplicate/conflict refusal posture
  - `ALLOW`, `ALLOW_WITH_WARNING`, `DEGRADE`, `BLOCK`, `QUARANTINE`, and `SAFE_MODE` response classes
  - governance visibility missing where required
  - law posture missing where required
  - proof visibility missing where required

M) FAILURE INJECTION, REPLAY, RE-ENTRY, AND RECOVERY VERIFICATION
- Failure injection is mandatory.
- Replay verification is mandatory.
- Re-entry after partial success is mandatory.
- Closure is blocked if any Phase D area proves only happy-path behavior.

Failure Injection / Partial-Success Matrix
| case | source phase | test stratum | expected authoritative outcome | expected governance / law / proof outcome | expected fail-closed / retry / degrade / block behavior | evidence required |
| --- | --- | --- | --- | --- | --- | --- |
| attach accepted but persistence visibility lags | D3 | runtime / session integration plus replay / re-entry / dedupe | authoritative PH1.L/session decision preserved | D4 visibility may remain incomplete until repaired | bounded retry and visibility repair only; no second authoritative mutation | authoritative row snapshot, replay result, visibility repair record |
| recover allowed but lease uncertainty persists | D1/D3/D4 | fail-closed / escalation plus replay / re-entry / dedupe | authoritative session truth preserved with uncertainty posture | governance-visible; law posture must remain degraded/block/quarantine per D4 | must not clear uncertainty locally; may remain degraded or blocked | uncertainty evidence set, governance/law posture record |
| stale device message rejected | D1/D2/D3 | runtime / session integration | no authoritative mutation | governance/law only if protected | explicit stale refusal; no replay mutation | stale-case result record |
| duplicate retry reused | D1/D2/D3 | replay / re-entry / dedupe | prior authoritative outcome reused | no new governance/law/proof requirement unless prior protected visibility was incomplete | reuse only; no duplicate mutation | duplicate-reuse evidence set |
| identity mismatch on attach / recover | D2/D4 | fail-closed / escalation | prior authoritative truth preserved | law posture required; governance conditional | fail closed with blocked outcome | identity-mismatch refusal evidence |
| platform mismatch on attach / recover | D2/D4 | fail-closed / escalation | prior authoritative truth preserved | law posture required; governance conditional | fail closed or degrade/block as frozen | platform-mismatch refusal evidence |
| transfer pending blocks attach | D1/D2/D4 | fail-closed / escalation | prior authoritative truth preserved | governance and law required | explicit `TRANSFER_PENDING_BLOCKED` and preserved authority | transfer-pending evidence set |
| failover recovering degrades attach | D1/D2/D4 | runtime / session integration plus fail-closed / escalation | authoritative truth preserved | governance and law required | `DEGRADE`, `BLOCK`, or `QUARANTINE` according to D4 class | failover posture evidence set |
| governance visibility missing where required | D4 | fail-closed / escalation | authoritative truth preserved | governance missing, law may stay provisional/degraded | protected completion withheld; bounded retry or escalated posture | governance-lag evidence set |
| law posture missing where required | D4 | fail-closed / escalation | authoritative truth preserved | governance may exist; law missing | protected completion withheld; bounded retry or escalated posture | law-lag evidence set |
| proof visibility missing where required | D4 | fail-closed / escalation | authoritative truth preserved | governance/law may remain incomplete; proof missing | protected completion withheld; no false done state | proof-lag evidence set |
| bounded retry exhausted and escalation begins | D3/D4 | fail-closed / escalation | authoritative truth preserved | governance/law escalate to block/quarantine/safe-mode as frozen | explicit escalation, not silent loop | escalation evidence set |

Replay / Recovery / Dedupe Matrix
| path | source phase | authoritative source of truth | re-entry rule to verify | duplicate / stale behavior to verify | evidence required | closure impact |
| --- | --- | --- | --- | --- | --- | --- |
| same-sequence attach retry | D1/D2/D3 | PH1.L/session storage current row plus per-device sequence anchors | reuse prior authoritative outcome | duplicate attach must not mutate again | prior-row snapshot plus retry result | blocks replay gate |
| reconnect after partial success | D3 | authoritative PH1.L/session decision | reread authoritative truth before visibility repair | local retry may only repair visibility | authoritative snapshot plus repaired visibility output | blocks replay gate |
| stale device message after newer sequence | D1/D3 | `device_turn_sequences` in authoritative session truth | stale request must not reopen session mutation | explicit stale rejection | stale refusal record plus authoritative row snapshot | blocks replay gate |
| cross-device resume conflict | D1/D2/D3/D4 | authoritative session truth plus ownership/coordination posture | conflict must resolve from authoritative truth | losing side must reuse or block, not mutate | conflict evidence set plus response posture record | blocks replay and D4 gates |
| ownership-uncertainty replay | D1/D4 | authoritative uncertainty posture | replay may repair visibility only | uncertainty may not be locally cleared | uncertainty replay evidence set | blocks D4 gate |
| transfer/failover in progress replay | D1/D4 | authoritative coordination posture | replay must preserve in-progress posture until authoritative transition completes | retry must reuse degrade/block posture | transfer/failover replay evidence set | blocks D4 gate |

N) DOCS, TRACEABILITY, EVIDENCE PACK, AND RESIDUAL-RISK DELIVERABLES
- D5 must produce one coherent Phase D closure packet.
- The packet must follow the Phase A and Phase C closure pattern exactly:
  - traceability matrix
  - closure evidence manifest
  - closure evidence pack
  - residual risk register
  - verification command record
  - approval checklist
- Phase D documentation updates must also confirm that frozen D1-D4 semantics are the baseline and were not reinterpreted during implementation/verification.

Docs / Evidence Deliverables Matrix
| deliverable | source phase(s) | purpose | required contents | closure gate supported | owner or producing step |
| --- | --- | --- | --- | --- | --- |
| Phase D docs update set | D1/D2/D3/D4 | align repo docs with implemented Phase D truth | exact references to frozen D1-D4 outputs, any implementation-facing notes, no semantic drift | docs-alignment gate | D5 documentation pass |
| Phase D traceability matrix | D1/D2/D3/D4/D5 | prove every frozen obligation maps to verification and evidence | requirement IDs, test IDs, failure IDs, replay IDs, evidence IDs, closure gates | traceability gate | D5 traceability pass |
| Phase D closure evidence manifest | D5 | index all evidence artifacts and command outputs | artifact IDs, file references, command refs, timestamps, scope statement | evidence-manifest gate | D5 manifest pass |
| Phase D closure evidence pack | D1/D2/D3/D4/D5 | assemble the final approval packet | selected test outputs, failure-injection records, replay records, governance/law/proof records, docs proof | evidence-pack gate | D5 evidence-pack pass |
| Phase D residual risk register | D1/D2/D3/D4/D5 | record any remaining non-blocking risk and bounded gaps | risk ID, description, containment, why not a freeze blocker, owner | residual-risk gate | D5 risk pass |
| verification command record | D5 | provide reproducible proof of what was run | exact commands, exit status, artifact refs, environment notes | command-record gate | D5 command-record pass |
| approval checklist | D5 | final human-readable freeze checklist | all gates, signoff criteria, missing-item fail rule | final freeze gate | D5 approval pass |

O) VERIFICATION COMMANDS, MANIFESTS, AND CLOSURE PACK
- The following command families are the baseline D5 verification lanes and must be captured in the command record when Phase D implementation exists:
  - `cargo test -p selene_storage --test db_wiring_ph1l_tables`
  - `cargo test -p selene_storage --test db_wiring_os_core_tables`
  - `cargo test -p selene_storage --test db_wiring_ph1j`
  - `cargo test -p selene_storage --test db_wiring_ph1f`
  - `cargo test -p selene_storage --test db_wiring_ph1vid_tables`
  - `cargo test -p selene_kernel_contracts --lib`
  - `cargo test -p selene_os --lib`
  - `bash scripts/selene_design_readiness_audit.sh`
  - `bash scripts/check_ph1_readiness_strict.sh`
- If implementation adds focused Phase D test selectors, the command record must include both:
  - focused selectors for the changed domain
  - package-wide or harness-wide confirmation commands that prove no hidden drift
- D5 closure pack assembly order:
  1. command record
  2. traceability matrix
  3. failure/replay evidence sets
  4. governance/law/proof evidence sets
  5. docs update proof
  6. manifest
  7. evidence pack
  8. residual risk register
  9. approval checklist

P) EXPLICIT NON-GOALS / DEFERRED ITEMS
- D5 does not redesign D1, D2, D3, or D4.
- D5 does not implement runtime/session/governance/law/proof code.
- D5 does not add new CI/workflow behavior beyond planning it as a future verification lane.
- D5 does not create new proof/governance/law response classes.
- D5 does not create new session states, outcomes, or replay rules.
- D5 does not absorb post-freeze operational runbooks or production incident playbooks beyond residual-risk recording.
- Any new runtime or persistence behavior discovered during verification is an implementation drift or design gap and must fail closure rather than being normalized in D5.

Q) COMPLETION CRITERIA
- Phase D is complete only when:
  - every frozen D1 obligation is traced and proven
  - every frozen D2 contract rule is traced and proven
  - every frozen D3 wiring and replay rule is traced and proven
  - every frozen D4 governance/law/proof rule is traced and proven
  - failure injection and replay evidence exists for the required cases
  - docs, traceability, manifest, evidence pack, residual risk register, command record, and approval checklist are complete
  - no open blocker remains in any closure gate

D5 Closure Gate Matrix
| closure gate | required test/doc/evidence input | source phase(s) | pass condition | fail condition | freeze impact |
| --- | --- | --- | --- | --- | --- |
| D1 consistency gate | D1 traceability rows plus runtime/storage/replay evidence | D1 | authority, tuple, attach/resume/recover/detach, stale/retry/duplicate, ownership rules all proven | any D1 semantic drift or missing trace row | blocks Phase D freeze |
| D2 contract gate | contract/unit evidence plus blocked-outcome evidence | D2 | canonical fields and outcome families proven | any missing field/outcome proof or normalization drift | blocks Phase D freeze |
| D3 wiring gate | runtime/storage/reconciliation evidence plus write-order proof | D3 | request→envelope→PH1.L→persistence order proven and authoritative split preserved | any alternate authority leak or wiring ambiguity | blocks Phase D freeze |
| D4 protected-action gate | governance/law/proof evidence plus protected-completion cases | D4 | protected classes, response classes, completion law, escalation posture proven | any missing or contradictory governance/law/proof behavior | blocks Phase D freeze |
| replay/re-entry gate | replay matrix, partial-success evidence, stale/duplicate evidence | D1/D2/D3/D4 | authoritative truth reused and no duplicate mutation | any replay path mutates twice or clears protected posture locally | blocks Phase D freeze |
| fail-closed gate | failure-injection evidence and escalation outcomes | D2/D3/D4 | blocked/degrade/block/quarantine/safe-mode behavior proven | any silent continuation or false success | blocks Phase D freeze |
| docs/evidence gate | docs updates, traceability, manifest, evidence pack, residual risk register, command record, checklist | D5 | all closure artifacts complete and coherent | any missing artifact, stale doc, or untraced requirement | blocks Phase D freeze |
| final Phase D freeze gate | all prior gates green plus clean-tree proof | D1/D2/D3/D4/D5 | every gate passes and no blocker remains | any single gate red or tree/evidence hygiene failure | Phase D may not be frozen |
