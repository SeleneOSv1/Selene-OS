PHASE C4 — STORAGE + PROOF + LAW INTEGRATION BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `566cccdf480db102265c66b31d730f58333a5a53`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
- `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md`
- `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C2_WAKE_ARTIFACT_LIFECYCLE_WORKERS_BUILD_PLAN.md`
- `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C3_MEMORY_RETENTION_PURGE_DELETE_ENFORCEMENT_BUILD_PLAN.md`
- `docs/WAKE_BUILD_PLAN.md`
- `docs/12_MEMORY_ARCHITECTURE.md`
- `docs/CORE_ARCHITECTURE.md`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_03.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_05.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
- `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
- `docs/DB_WIRING/ARTIFACTS_LEDGER_TABLES.md`
- `docs/DB_WIRING/PH1_J.md`
- `docs/DB_WIRING/PH1_GOV.md`
- `docs/DB_WIRING/PH1_LAW.md`
- `docs/DB_WIRING/PH1_M.md`
- `docs/DB_WIRING/PH1_OS.md`
- `crates/selene_kernel_contracts/src/runtime_execution.rs`
- `crates/selene_kernel_contracts/src/runtime_governance.rs`
- `crates/selene_kernel_contracts/src/runtime_law.rs`
- `crates/selene_kernel_contracts/src/ph1art.rs`
- `crates/selene_kernel_contracts/src/ph1m.rs`
- `crates/selene_storage/src/ph1j.rs`
- `crates/selene_storage/src/ph1f.rs`
- `crates/selene_storage/src/repo.rs`
- `crates/selene_os/src/device_artifact_sync.rs`

B) PURPOSE
- C4 defines the integration build plan that joins already-frozen C2 and C3 lifecycle commits to PH1.J proof visibility, PH1.GOV governed visibility, and PH1.LAW protected-completion posture.
- C4 does not redesign lifecycle ownership. C4 consumes C1 authority law, C2 wake worker boundaries, and C3 memory worker boundaries, then freezes:
- which authoritative commits must land before any downstream visibility is emitted
- which lifecycle actions require PH1.J proof capture or audit visibility
- which lifecycle actions require PH1.GOV or PH1.LAW visibility
- how protected completion is gated
- how replay, reconciliation, and re-entry behave when storage is committed but downstream visibility is missing
- how fail-closed escalation works without inventing a second authority path
- C4 does not author code, tests, closure evidence, or new proof/governance/law architectures.

C) DEPENDENCY RULE
- C4 depends on the frozen outputs of:
- C1 for lifecycle plane ownership, receipt-vs-authority law, reverse-authority refusal, proof/governance/law visibility categories, and purge/restore terminality
- C2 for wake artifact authoritative writes, rollout projection boundaries, target deployment boundaries, queue control boundaries, and wake replay/re-entry law
- C3 for PH1.M authoritative ownership, derived-view refusal, hold/archive/tombstone/purge/restore law, and memory replay/re-entry law
- Build Section 04 for verification-before-authority
- Build Section 05 for persistence, replay, dedupe, reconciliation, and quarantine law
- Build Section 09 for governance visibility and proof-capture enforcement
- Build Section 11 for proof-critical protected completion, law escalation, quarantine, and safe-mode posture
- If repo truth is weaker than the frozen lifecycle law, C4 must name the gap explicitly and fail closed rather than infer a missing proof, decision, or storage surface.

D) ARCHITECTURAL POSITION
- Selene remains cloud-authoritative.
- C4 sits after authoritative storage ownership is already frozen in C2 and C3, but before lifecycle work can be considered protected-complete when proof/governance/law visibility is required.
- C4 therefore governs the seam between:
- plane-owned authoritative commits
- PH1.J evidence / proof visibility
- PH1.GOV governed decision visibility
- PH1.LAW final protected-action judgment
- PH1.OS orchestration posture
- C4 is not a new storage owner and is not a new lifecycle writer. It is the integration law that prevents:
- proof rows from becoming alternate authority
- governance or law visibility from mutating lifecycle truth directly
- authoritative commits from appearing protected-complete when required downstream visibility is missing
- replay from inventing a second authoritative mutation

E) C1 / C2 / C3 ASSUMPTIONS CONSUMED
- C4 consumes frozen C1 storage and evidence law:
- `os_process.artifacts_ledger` owns artifact identity truth
- `wake_artifact_apply_*` owns target deployment truth
- PH1.M-owned ledger/config surfaces own memory truth
- `audit.audit_events`, `proof_entry_ref`, and `proof_record_ref` are evidence/proof surfaces only
- `gov_decision_bundle`, `RuntimeExecutionEnvelope.governance_state`, `RuntimeExecutionEnvelope.law_state`, and `os_decision_bundle` are decision/visibility surfaces only
- C4 consumes frozen C2 wake boundaries:
- promotion projection is bridge scope only
- runtime-use evidence is never deployment or artifact identity authority
- queue ACK / retry / dead-letter is control truth only
- authoritative wake re-entry always starts from plane-owned storage rows
- C4 consumes frozen C3 memory boundaries:
- ledger-first PH1.M truth wins over current/index/graph/archive/bundle surfaces
- purge is terminal
- hold blocks terminal delete/purge
- restore must not reopen a purged same-subject record
- current repo lacks standalone PH1.M hold, purge, and restore markers, so those remain explicit PH1.M-owned additive gaps rather than implicit inference from current views or audit rows
- C4 consumes Build Section 05 law:
- same `idempotency_key` must produce the same authoritative result
- replay may not bypass canonical authority decisions
- stale or quarantined persistence posture is a governed input, not a local-only detail
- C4 consumes Build Section 11 law:
- proof-required actions are not complete without PH1.J success
- proof failure is a runtime-law failure, not optional logging loss
- replay inconsistency and persistence quarantine can escalate into `BLOCK`, `QUARANTINE`, or `SAFE_MODE`

F) CURRENT AUTHORITATIVE SURFACES IN SCOPE
- Current repo truth already splits storage ownership across wake and memory domains. C4 must preserve that split while standardizing downstream visibility expectations.
- Hard law:
- authoritative plane-owned commit first
- proof / governance / law visibility second
- operational receipts, current projections, and device acknowledgements last
- No derived or evidence surface may reverse-authorize a lifecycle action.

Current Authoritative Surface → C4 Integration Scope Mapping
| authoritative surface | lifecycle plane or domain | current role | proof visibility required | governance visibility required | law visibility required | notes / constraints |
| --- | --- | --- | --- | --- | --- | --- |
| `os_process.artifacts_ledger` | `ARTIFACT_IDENTITY_LIFECYCLE` | append-only artifact-global identity truth | YES when lifecycle-significant or protected | YES when governed activation / rollback / deprecation path applies | YES when protected runtime completion depends on the transition | append-only row is authoritative first; no projection, audit, or device receipt may pre-authorize it |
| `wake_promotion_ledger` | rollout bridge between artifact identity and target deployment | authoritative rollout history only | YES when rollout change drives protected activation / rollback | YES when enterprise governance is enabled | SOMETIMES; only when rollout state changes protected execution posture | rollout history is authoritative only for bridge scope, never for artifact-global identity |
| `wake_promotion_current` | rollout bridge between artifact identity and target deployment | authoritative rollout current pointer only | YES when lagging projection would hide a protected transition | YES when governed rollout posture is required | SOMETIMES | current pointer can trigger downstream visibility repair but cannot replace `artifacts_ledger` |
| `wake_artifact_apply_ledger` | `TARGET_DEPLOYMENT_LIFECYCLE` | append-only per-device deployment truth | YES when apply / rollback is lifecycle-significant | SOMETIMES; only when governed rollout path reaches protected execution | SOMETIMES; protected activation/rollback paths only | authoritative for deployment history, not artifact-global truth |
| `wake_artifact_apply_current` | `TARGET_DEPLOYMENT_LIFECYCLE` | per-device active/staged/last-known-good truth | YES when protected activation or rollback requires visibility | SOMETIMES | SOMETIMES | current row is deployment authority only; `last_known_good_artifact_version` is the rollback anchor |
| `wake_runtime_events` | `RUNTIME_USE_LIFECYCLE` | authoritative runtime-use evidence only | SOMETIMES; only when runtime-use event is itself lifecycle-significant evidence | NO direct governance ownership in C4 | SOMETIMES when runtime-use evidence changes protected law posture | evidence only; must never become artifact or deployment authority |
| `memory.memory_atoms_ledger` | `MEMORY_LIFECYCLE` | append-only atom lifecycle truth | YES for tombstone/delete, purge, restore, and policy-expiry transitions | SOMETIMES when governed retention/hold policy exists | SOMETIMES when memory lifecycle changes live protected execution admissibility | current repo has `Stored | Updated | Forgotten`; explicit purge/restore markers remain PH1.M-owned C3 completion work |
| `memory.memory_atoms_current` | `MEMORY_LIFECYCLE` | derived current retrieval view | NO by itself | NO by itself | NO by itself | included because C4 must forbid treating current view repair as authoritative completion |
| `memory.memory_threads_ledger` | `MEMORY_LIFECYCLE` | append-only thread lifecycle truth | YES for thread forget/archive/purge/restore visibility | SOMETIMES | SOMETIMES | authoritative thread lifecycle truth; current view follows later |
| `memory.memory_threads_current` | `MEMORY_LIFECYCLE` | derived thread continuity projection | NO by itself | NO by itself | NO by itself | stale current rows do not block parent authoritative truth, but protected completion may remain pending if required visibility is missing |
| `memory.emotional_threads_ledger` | `MEMORY_LIFECYCLE` | append-only emotional continuity lifecycle truth | SOMETIMES when delete/purge is lifecycle-significant | SOMETIMES | SOMETIMES | emotional continuity remains PH1.M-owned; proof/law only when the path is already governed |
| `memory.emotional_threads_current` | `MEMORY_LIFECYCLE` | derived emotional continuity projection | NO by itself | NO by itself | NO by itself | projection only |
| `memory.memory_archive_index` | `MEMORY_LIFECYCLE` | pointer/index metadata for archive page-in | NO by itself; follows parent authoritative archive/restore truth | NO by itself | NO by itself | pointer existence never proves archive success or lawful restore |
| `memory.memory_graph_nodes` / `memory.memory_graph_edges` | `MEMORY_LIFECYCLE` derived index domain | graph/index continuity surface | NO by itself | NO by itself | NO by itself | derived index only; graph cleanup lag must not be mistaken for unresolved parent truth |
| `memory.memory_thread_refs` | `MEMORY_LIFECYCLE` derived pointer domain | bounded evidence pointer set | NO by itself | NO by itself | NO by itself | pointer set only; cannot re-establish a forgotten or purged thread |

G) CURRENT PROOF / GOVERNANCE / LAW SURFACES IN SCOPE
- Repo truth already exposes PH1.J, PH1.GOV, PH1.LAW, and PH1.OS surfaces, but they are not lifecycle writers.
- C4 must keep these hard boundaries:
- PH1.J is canonical evidence / proof only
- PH1.GOV is deterministic governance decision only
- PH1.LAW is final protected-completion judgment only
- PH1.OS is orchestration-only normalization and dispatch legality
- Explicit repo-truth ambiguity:
- PH1.J proof persistence is clearly present through `Ph1jRuntime::emit_proof`, `append_proof_record`, replay/verify APIs, `proof_entry_ref`, `proof_record_ref`, and `RuntimeExecutionEnvelope.proof_state`.
- The current DB-wiring slice names `audit.audit_events` more explicitly than any dedicated proof-table name.
- Safest C4 planning assumption: use exact PH1.J proof API/ref surfaces and do not invent a table name for canonical proof records.

Proof / Governance / Law Surface Mapping Matrix
| surface | exact current repo truth name | authoritative / evidence / decision role | producer | consumers | must never be treated as | C4 integration note |
| --- | --- | --- | --- | --- | --- | --- |
| PH1.J audit surface | `audit.audit_events` | evidence-only for lifecycle semantics; authoritative for audit history only | PH1.J append writer via `append_audit_event` / `emit_audit` | operators, replay tooling, PH1.GOV, PH1.LAW, C2/C3 re-entry readers | lifecycle storage truth | every protected or lifecycle-significant transition must bind reason-coded audit visibility here when required |
| PH1.J proof record surface | canonical proof record store exposed by `Ph1jRuntime::emit_proof`, `append_proof_record`, `proof_records_by_request_id_bounded`, `proof_records_by_session_turn_bounded`, `verify_proof_records_by_request_id_bounded` | canonical proof-only | PH1.J | PH1.GOV, PH1.LAW, verification tooling, protected-path replay | lifecycle storage truth or governance decision | C4 must consume the exact proof refs and replay APIs, not invent an alternate proof path or proof table |
| proof linkage refs | per-artifact `proof_entry_ref`; turn-level `proof_record_ref` | canonical proof linkage only | PH1.J / A4 seam | PH1.GOV, PH1.LAW, C4 completion gating | interchangeable substitutes or state commits | `proof_entry_ref` and `proof_record_ref` are not interchangeable; C4 must preserve that distinction |
| runtime proof state | `RuntimeExecutionEnvelope.proof_state` | proof execution visibility / verification posture | upstream runtime path after PH1.J interaction | PH1.LAW, protected completion gate, replay logic | proof record store or lifecycle authority | C4 completion uses `proof_write_outcome`, `proof_failure_class`, and `proof_chain_status` as visibility posture, not as lifecycle truth |
| governance decision bundle | `gov_decision_bundle` | decision-only | PH1.GOV | C2/C4 wake commit gates, governed memory policy flows, PH1.LAW | committed lifecycle row | governs allow/block posture before protected commit or protected completion |
| governance execution state | `RuntimeExecutionEnvelope.governance_state` | decision / drift / quarantine visibility | PH1.GOV and governance path | PH1.LAW, protected completion gate, operators | lifecycle authority | carries policy version, cluster consistency, quarantine posture, and canonical artifact-trust linkage |
| governance decision log ref | `GovernanceExecutionState.decision_log_ref` and `GovernanceDecisionLogEntry` | governance trace only | PH1.GOV | replay, incident review, PH1.LAW | lifecycle authority or proof record | used to verify governed visibility was produced, not to author state |
| runtime law surface | `RuntimeExecutionEnvelope.law_state` | final protected-completion judgment | PH1.LAW | protected-path callers, operators, replay tooling | lifecycle storage authority or governance decision | protected completion is not lawful without final `law_state` where the path requires it |
| runtime law decision log | `RuntimeLawExecutionState.decision_log_ref` and `RuntimeLawDecisionLogEntry` | law trace only | PH1.LAW | replay, review, C5 verification later | lifecycle storage truth | C4 uses this for completion traceability and failure escalation only |
| PH1.OS orchestration surface | `os_decision_bundle` | orchestration legality / normalization only | PH1.OS | dispatch/orchestration readers | proof truth, governance truth, law truth, or lifecycle truth | C4 may consume it as correlation/legality context only; it cannot complete a lifecycle action |

H) STORAGE-COMMIT BEFORE VISIBILITY LAW
- Hard C4 law:
- authoritative storage commit must land before PH1.J evidence, PH1.GOV governed visibility, PH1.LAW protected-completion judgment, queue ACK, or client acknowledgement can represent the transition as complete
- if authoritative storage commit is missing, no downstream proof, audit, governance, law, or receipt surface may stand in as a substitute
- if authoritative storage commit succeeds but downstream visibility fails, the authoritative commit remains canonical while completion stays unsatisfied for any path that requires that visibility
- C4 storage-commit ordering by domain:
- wake artifact identity:
  - `artifacts_ledger` append first
  - PH1.J audit/proof second
  - PH1.GOV / PH1.LAW visibility third if required
  - queue/receipt/client acknowledgement last
- wake rollout projection:
  - `wake_promotion_ledger` / `wake_promotion_current` first
  - PH1.J audit second
  - PH1.GOV / PH1.LAW visibility third only when the rollout change participates in protected completion
- wake target deployment:
  - `wake_artifact_apply_ledger` / `wake_artifact_apply_current` first
  - PH1.J audit second
  - PH1.GOV / PH1.LAW visibility third only when activation/rollback is protected
  - sync ACK last
- memory lifecycle:
  - PH1.M authoritative ledger/config commit first
  - derived/index cleanup second
  - PH1.J audit/proof visibility third
  - PH1.GOV / PH1.LAW visibility fourth only for already-governed paths
  - device mirror/ACK last
- Verification-before-completion rule:
- Build Section 04 still gates authoritative mutation before commit.
- C4 adds the post-commit rule that protected completion remains incomplete until required proof/governance/law visibility is present and non-blocking.

I) LIFECYCLE TRANSITION → STORAGE / PROOF / LAW INTEGRATION MODEL
- C4 integration is transition-scoped, not subsystem-generic.
- Each in-scope transition must explicitly answer:
- what authoritative row commits first
- whether PH1.J audit is required
- whether canonical proof linkage is required
- whether PH1.GOV visibility is required
- whether PH1.LAW final posture is required
- what counts as complete
- what replay may safely retry

Lifecycle Transition Integration Matrix
| transition | authoritative storage commit | proof event or proof visibility requirement | governance visibility requirement | law visibility requirement | completion condition | replay / retry note | fail-closed or bounded-retry note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wake artifact rollout `Canary -> Active` plus artifact-global `ACTIVE` / prior `REPLACED` | `wake_promotion_ledger` / `wake_promotion_current` first, then append-only `os_process.artifacts_ledger` | PH1.J `STATE_TRANSITION` audit required; canonical proof linkage required when protected artifact authority path applies | `gov_decision_bundle` and `governance_state` required when governance is enabled | `RuntimeExecutionEnvelope.law_state` required when activation is protected | complete only when projection commit, artifact-global append, and all required proof/governance/law outputs exist and are non-blocking | replay re-reads projection and artifact rows first; if rows exist, only missing visibility may be retried | protected activation fails closed if proof or required decision visibility is missing |
| wake artifact rollout `Active -> RolledBack` plus artifact-global rollback history | `wake_promotion_*` first, then `artifacts_ledger` rollback append | PH1.J audit required; proof linkage required if rollback is protected or safety-significant | governance visibility required when rollback is governed | law visibility required when rollback changes protected execution posture | complete only when authoritative rollback rows exist and required downstream visibility is recorded | replay may backfill PH1.J / GOV / LAW visibility but may not append a second rollback row | fail closed on governed/law-required rollback if required visibility cannot be produced |
| wake target deployment `Staged -> Active` | `wake_artifact_apply_ledger` append plus `wake_artifact_apply_current` update | PH1.J audit required when lifecycle-significant; canonical proof required only when protected activation path requires it | conditional on governed protected path | conditional on protected activation class | complete when deployment commit exists and any required PH1.J / GOV / LAW outputs exist; local ACK is never the completion gate | replay re-reads deployment rows and reuses the same activate idempotency family; only missing visibility/ACK can be retried | bounded retry for missing receipt/audit; fail closed if protected activation lacks required proof/law posture |
| wake target deployment rollback | `wake_artifact_apply_ledger` rollback append plus `wake_artifact_apply_current` restore to `last_known_good_artifact_version` | PH1.J audit required when lifecycle-significant | conditional | conditional | complete when rollback commit exists and any required downstream visibility exists | replay returns the existing rollback result and only repairs missing downstream visibility | fail closed if rollback path is governed/protected and required visibility is absent |
| wake runtime-use accepted/rejected event | append `wake_runtime_events` only; no C4 lifecycle write | PH1.J visibility only if the runtime-use event itself is lifecycle-significant evidence | no direct governance write in C4 | conditional if PH1.LAW already treats the runtime-use outcome as a protected input | complete when runtime event exists and any required evidence capture exists | replay may not convert runtime evidence into deployment or artifact identity truth | fail closed on any attempt to reverse-authorize higher-plane truth from runtime evidence |
| memory tombstone / delete accepted | PH1.M authoritative ledger/config commit first (`Forgotten` / `THREAD_FORGOTTEN` current rows or additive PH1.M-owned marker for explicit delete-request acceptance where needed) | PH1.J audit required; canonical proof linkage required only for protected or governed memory paths | conditional when governed retention/hold policy applies | conditional when live protected execution admissibility changes | complete when PH1.M authoritative commit exists and any required visibility is recorded; derived cleanup may lag without reopening the decision | replay re-reads the PH1.M authoritative subject and only repairs derived, audit, or receipt lag | fail closed on missing hold/policy context; no current/index/evidence surface may substitute |
| memory archive / policy-expiry transition | PH1.M authoritative archive/expiry commit first; `memory_archive_index` only after parent commit | PH1.J audit required when lifecycle-significant | conditional | conditional | complete when authoritative archive/expiry posture exists and required visibility exists; pointer lag does not reopen authority | replay repairs pointer/index lag from the parent authoritative state | fail closed if archive/expiry would require missing governed decision input |
| memory purge | required PH1.M-owned terminal purge marker/commit first; until the marker exists, the action is not lawful to complete | PH1.J audit required; proof linkage required when policy class makes purge protected | conditional when governed deletion class exists | conditional when purge changes protected execution posture | complete only when PH1.M terminal purge commit exists and required downstream visibility succeeds; derived cleanup is follow-on only | replay must look for the terminal purge marker first and refuse duplicate mutation | until explicit PH1.M purge marker exists, purge must fail closed rather than infer terminality from current/index cleanup |
| memory restore | required PH1.M-owned restore commit first; `memory_archive_index` or current views cannot authorize it | PH1.J audit required; proof linkage conditional | conditional | conditional | complete only when lawful restore commit exists and required downstream visibility exists | replay re-reads authoritative pre-restore and restore rows; only missing derived/visibility updates may be retried | fail closed if parent subject was purged, if hold blocks restore, or if current repo lacks the needed authoritative restore marker |
| PH1.J-driven lifecycle evidence backfill for already-committed authoritative row | no new storage commit; authoritative row already exists | emit missing audit/proof only | no new governance write unless separately missing and required | no new law write unless separately missing and required | complete when missing downstream evidence is backfilled without touching the authoritative row | replay must prove the authoritative row already exists, then emit only the missing downstream record | bounded retry only; never replay the authoritative mutation |

J) COMPLETION GATING AND VERIFICATION-BEFORE-COMPLETION
- C4 completion rule has two layers:
- authority layer:
  - Build Section 04 verification-before-authority must already have passed before any authoritative lifecycle mutation lands
- protected completion layer:
  - after authoritative commit, any path marked proof-required, governance-required, or law-required is still incomplete until those required visibility surfaces exist and are non-blocking
- C4 must therefore distinguish:
- authoritative commit exists
- downstream visibility exists
- protected completion is satisfied
- A stored authoritative row may be canonical and still not be protected-complete.
- This is not a new lifecycle state family. It is a completion-gating rule over the already-frozen lifecycle states.
- If required downstream visibility is missing:
- do not write a second authoritative row
- do not treat audit-only or projection-only surfaces as a substitute
- retry/backfill the missing downstream visibility if lawful
- if the action class is proof-critical or law-critical and retries fail, escalate into PH1.LAW governed refusal/quarantine/safe mode as appropriate

Completion-Gating Matrix
| lifecycle action class | authoritative commit required | proof success required | governance visibility required | law posture required | allowed response if proof/write/decision visibility fails | final completion rule |
| --- | --- | --- | --- | --- | --- | --- |
| non-protected wake rollout projection change | YES | audit required when lifecycle-significant; canonical proof not always required | NO unless governed rollout path applies | NO unless protected path is implicated | bounded retry/backfill of missing PH1.J visibility; no second projection mutation | complete when projection commit exists and all required visibility for that exact path exists |
| protected wake artifact identity activation / replacement / rollback | YES | YES when protected artifact authority class applies | YES when governance is enabled for the path | YES when PH1.LAW classifies the path as protected | authoritative row stands; action is not complete; retry/backfill downstream visibility only; if required visibility cannot be produced, final posture is governed block/quarantine/safe mode rather than silent success | protected-complete only when authoritative row exists, proof is canonical, required governance visibility exists, and final `law_state` is non-blocking |
| non-protected wake target staging | YES | NO canonical proof by default; audit only if lifecycle-significant | NO | NO | bounded retry of missing audit/receipt only | complete once deployment staging commit exists and any required audit exists |
| protected wake target activation / rollback | YES | YES when protected path requires it | conditional | conditional to YES | authoritative deployment row stands; retry missing downstream visibility only; refuse duplicate activation/rollback mutation | protected-complete only when deployment commit exists and all required downstream visibility is present |
| memory tombstone / delete under ordinary policy | YES | YES for lifecycle-significant delete/tombstone visibility | conditional | conditional | authoritative PH1.M decision stands; retry derived cleanup and required visibility; do not reopen the subject from stale views | complete when authoritative PH1.M delete/tombstone commit exists and required visibility exists |
| memory purge | YES, and the authoritative purge marker must be explicit | YES when purge is lifecycle-significant or governed | conditional | conditional to YES when purge affects protected execution | if purge marker or required visibility is missing, refuse final completion; stale derived cleanup alone does not count | complete only when explicit PH1.M purge commit exists and all required downstream visibility succeeds |
| memory restore | YES | YES when restore is lifecycle-significant or governed | conditional | conditional | if restore visibility fails, authoritative restore commit stands but protected completion is unsatisfied; if explicit restore marker is absent, refuse restore entirely | complete only when lawful restore commit exists and all required downstream visibility succeeds |
| derived cleanup only (`memory_graph`, `memory_thread_refs`, `memory_archive_index`, current views, queue ACK`) | NO new authoritative commit | NO | NO | NO | bounded retry or rebuild | complete when the derived repair matches the authoritative parent; it never changes protected completion by itself |

K) REPLAY, RECONCILIATION, RE-ENTRY, AND DEDUPE LAW
- C4 replay law is:
- authoritative row first
- replay reads the authoritative row first
- replay reuses the original idempotency family
- replay repairs only what is missing downstream
- replay never invents a second authoritative mutation because evidence or visibility is missing
- C4 consumes existing idempotency law from C2, C3, PH1.J, and Section 05. It does not invent a new dedupe registry.
- `RuntimeExecutionEnvelope.persistence_state` is a real current surface and must be consumed during C4 replay:
- `reconciliation_decision`
- `acknowledgement_state`
- `conflict_severity`
- `cross_node_dedupe_applied`
- When replay sees authoritative commit present but downstream visibility absent:
- reuse prior authoritative outcome
- emit only the missing PH1.J / PH1.GOV / PH1.LAW / queue-ACK side effect if the path still requires it
- if persistence state is `QuarantinedLocalState` or replay inconsistency is present, do not continue as a silent local retry; elevate into governed/law posture as required

Replay / Re-entry / Dedupe Matrix
| action or worker path | idempotency basis | authoritative source of truth during replay | proof / decision replay expectation | stale or duplicate handling | re-entry rule | bounded ambiguity note if any |
| --- | --- | --- | --- | --- | --- | --- |
| wake artifact identity integration | `(scope_type, scope_id, artifact_type, artifact_version, idempotency_key)` | `os_process.artifacts_ledger` | PH1.J, PH1.GOV, and PH1.LAW visibility may be backfilled only after the append is confirmed | duplicate append returns prior authoritative outcome; conflicting scope/version without matching idempotency is refused | start from `artifacts_ledger`; if row exists, emit only missing visibility | no ambiguity about authority; only downstream visibility may be incomplete |
| wake rollout projection integration | `(artifact_version, to_state, idempotency_key)` | `wake_promotion_ledger` / `wake_promotion_current` | replay may backfill missing PH1.J or governed/law visibility only | invalid `from -> to` stays refused; blocked version requires explicit revalidation | start from `wake_promotion_current`, then repair only missing downstream outputs | rollout projection does not settle artifact-global authority by itself |
| wake target deployment activation / rollback integration | `(device_id, artifact_version, state, idempotency_key)` using the exact C2 per-state family | `wake_artifact_apply_ledger` / `wake_artifact_apply_current` | replay may backfill PH1.J and decision visibility; ACK replay is queue-only | duplicate activate/rollback returns existing deployment truth; stale staged pointer is refused | start from apply ledger/current; if deployment row exists, do not mutate again | runtime-use evidence remains read-only during replay |
| sync receipt delivery after prior authoritative commit | `(sync_job_id, receipt_ref, envelope.idempotency_key)` | owning lifecycle row plus queue row | proof / decision replay is not re-derived from queue success; only missing send/ACK side effects are retried | ACK after foreign lease or after dead-letter is refused | start from queue row and owning lifecycle row together | queue state never upgrades completion by itself |
| PH1.M atom/thread/emotional lifecycle integration | current repo truth: `(user_id, idempotency_key)` or `(user_id, thread_key, idempotency_key)` for the owned PH1.M row family | PH1.M ledger/config surfaces | required PH1.J visibility may be backfilled after authoritative PH1.M row is confirmed | stale restore/delete/purge attempts lose to the latest lawful PH1.M state | start from the latest PH1.M authoritative subject, then rebuild derived surfaces and missing visibility only | explicit hold/purge/restore markers remain an additive gap and must fail closed until present |
| PH1.M graph/archive/current rebuild after authoritative change | parent authoritative subject plus current derived-surface idempotency keys (for example `(user_id, idempotency_key)` for graph updates) | parent PH1.M authoritative subject | no proof or decision replay unless the parent lifecycle action required it | stale graph/archive/current rows are overwritten or removed deterministically | start from parent authoritative row; derived repair only | no ambiguity if parent authority is present |
| PH1.J audit append | scoped dedupe `(tenant_id, work_order_id, idempotency_key)` when scope exists, otherwise fallback `(correlation_id, idempotency_key)` | plane-owned authoritative lifecycle row plus PH1.J append result | duplicate append returns prior audit row; proof store remains separate | stale audit row can never force lifecycle mutation | if authoritative row exists and audit is missing, append audit only | audit can lag; it cannot authorize |
| PH1.J proof replay / verification | request-id or session/turn bounded replay over canonical proof store | canonical proof record store plus `proof_entry_ref` / `proof_record_ref` | replay must preserve canonical proof linkage and ordering | duplicate proof write must resolve to deterministic prior proof result | if authoritative protected action already committed, retry proof write only when the same protected action correlation still applies | current repo documents proof APIs and refs more explicitly than a proof-table name; C4 must not invent one |
| protected completion replay with persistence anomaly | original C2/C3 idempotency family plus `RuntimeExecutionEnvelope.persistence_state` | authoritative lifecycle row remains primary; `persistence_state` decides whether replay may continue or must quarantine | missing downstream visibility may be retried only if `reconciliation_decision` permits lawful reuse of prior authoritative outcome | `QuarantinedLocalState` or replay inconsistency blocks silent continuation | start from authoritative row and envelope persistence posture together | persistence anomaly is a law input, not a local-only retry hint |

L) FAILURE CLASSIFICATION, ESCALATION, AND FAIL-CLOSED BEHAVIOR
- C4 failure handling must classify every post-commit integration failure into one of four buckets:
- bounded local retry
- replay/backfill only
- governed/law escalation
- safe refusal before completion
- C4 fail-closed law:
- no protected lifecycle action may appear complete if required proof/governance/law visibility is missing
- no proof, audit, governance, or law surface may back-write lifecycle authority
- no replay may create a second authoritative mutation when the first one already succeeded
- proof success without authoritative storage commit is evidence of a broken path, not a substitute for the missing commit
- If broader normalization is still missing after C4 work, stop at a bounded refusal/quarantine boundary and leave verification/closure proof to C5.

Failure / Escalation Matrix
| case | what remains authoritative | what is incomplete | immediate response | final runtime posture | compensation / retry / quarantine note | defer-to-C5 note if only verification remains |
| --- | --- | --- | --- | --- | --- | --- |
| authoritative storage write succeeds, proof write fails | committed lifecycle row in `artifacts_ledger`, `wake_promotion_*`, `wake_artifact_apply_*`, or PH1.M surface | proof visibility and any protected completion gate | keep authoritative row; retry proof with the same correlation/idempotency family; do not replay storage write | non-protected path: bounded retry; protected path: not complete, escalate to PH1.LAW `BLOCK` / `QUARANTINE` / `SAFE_MODE` as required | proof backfill only; if proof remains impossible, quarantine the protected path rather than mutating storage again | YES if the only remaining work is verification/acceptance evidence after the required downstream refs already exist |
| authoritative storage write succeeds, governance visibility write fails | committed lifecycle row | governed visibility / decision trace | keep authoritative row; retry governed visibility only if the path still requires it | governed protected path remains incomplete; non-governed path may remain complete if governance was not required | no second lifecycle mutation; if governance drift persists, escalate to governance quarantine/safe mode as appropriate | YES only when storage and required runtime posture already exist and the remaining work is verification packaging |
| authoritative storage write succeeds, law visibility update fails | committed lifecycle row | final protected completion judgment | keep authoritative row; retry law-state population only if the same protected action correlation is intact | protected path remains incomplete and may be blocked/quarantined/safe-mode gated | never treat missing `law_state` as optional for protected actions | YES only if C5 is merely verifying already-present law refs, not creating them |
| proof write succeeds, authoritative commit fails | no lifecycle mutation; proof row and/or audit row exist as evidence of a failed path only | authoritative storage mutation | stop; mark the proof as evidence of failure, not success; refuse downstream completion | fail closed; authoritative state remains prior lawful row | no storage retry unless the original action is re-admitted lawfully from the start | NO; this is a real integration failure, not a verification-only gap |
| stale replay sees completed authoritative commit but missing downstream visibility | original authoritative row | PH1.J / GOV / LAW / ACK visibility | reuse prior authoritative outcome; emit only the missing downstream side effect if lawful | bounded retry unless persistence/governance/law posture requires quarantine | no second authoritative write | YES if only acceptance-pack verification remains after backfill succeeds |
| duplicate retry after partial success | first authoritative outcome | maybe missing downstream visibility only | return prior authoritative result and repair only the missing downstream surface | deterministic no-op replay or bounded backfill | duplicate authoritative mutation is forbidden | NO if downstream refs are still materially missing |
| bounded local retry exhausted | original authoritative row or original refusal result | unresolved downstream visibility or worker-local delivery | stop local retries; escalate according to whether the path is protected and whether cross-subsystem interpretation is required | queue dead-letter for local control failures; governed or law quarantine/safe mode for protected-path failures | do not widen into ad hoc redesign; escalate only through existing PH1.GOV / PH1.LAW posture | YES only when runtime correctness is already settled and C5 is simply proving it |
| conflicting lifecycle signal across surfaces | highest-authority plane-owned row | lower-authority surface is conflicting or stale | preserve highest-authority row; refuse reverse-authority mutation; backfill or quarantine the lower-authority surface | bounded refusal for local evidence conflicts; governed/law escalation when the conflict affects protected execution | queue/Audit/current/graph/runtime evidence may be dead-lettered or rebuilt, never promoted upward | NO; this is an integration correctness issue until resolved |

M) WORKER / ENGINE VISIBILITY BOUNDARIES
- C4 does not create new lifecycle writers.
- C4 preserves these ownership lines:
- C2 worker families own wake authoritative commits
  - `WakeArtifactIdentityTransitionWorker` -> `os_process.artifacts_ledger`
  - `WakePromotionProjectionWorker` -> `wake_promotion_*`
  - `WakeTargetDeploymentWorker` -> `wake_artifact_apply_*`
  - `WakeSyncReceiptDeliveryWorker` -> queue-control rows only
- C3 worker families own PH1.M lifecycle commits and derived cleanup
- PH1.J owns evidence/proof append and replay/verify surfaces only
- PH1.GOV owns governed decision posture only
- PH1.LAW owns final protected-completion judgment only
- PH1.OS owns orchestration legality only
- `RuntimeExecutionEnvelope` is the canonical transport for:
- `persistence_state`
- `governance_state`
- `proof_state`
- `artifact_trust_state`
- `law_state`
- Engine visibility law:
- C2/C3 writers may consume `gov_decision_bundle`, `governance_state`, `proof_state`, and `law_state` as gating inputs
- PH1.J / PH1.GOV / PH1.LAW / PH1.OS may observe lifecycle rows and envelope state
- PH1.J / PH1.GOV / PH1.LAW / PH1.OS may not directly mutate lifecycle storage rows at C4 scope
- completion-gating judgment is read from the plane-owned storage row plus required downstream visibility surfaces together; it is not owned by a new writer

N) EXPLICIT NON-GOALS / DEFERRED TO C5
- C4 does not redesign:
- C2 wake worker ownership
- C3 memory worker ownership
- PH1.J proof architecture
- PH1.GOV governance architecture
- PH1.LAW runtime-law architecture
- PH1.OS orchestration architecture
- C4 does not create new lifecycle states, new queues, or new alternate authority rows.
- C4 does not author closure evidence, final acceptance packs, or freeze verification bundles.
- Those belong to C5.

C4 → C5 Boundary Matrix
| concern | handled in C4 | deferred to C5 | rationale |
| --- | --- | --- | --- |
| authoritative-commit before downstream-visibility law for C2 and C3 lifecycle work | YES | NO | this is the core purpose of C4 |
| proof / governance / law visibility requirements and failure escalation rules | YES | NO | C4 defines runtime integration law, not closure evidence |
| replay / re-entry / dedupe rules when authoritative commit exists but downstream visibility lags | YES | NO | C4 must freeze these integration rules before implementation |
| final verification pack proving every required C2/C3 path satisfies C4 integration law | NO | YES | C5 is the closure and verification phase |
| acceptance test inventory, evidence manifest, audit pack, freeze packet | NO | YES | C5 owns verification closure artifacts |
| post-implementation confirmation that every protected lifecycle action reaches lawful completion under fault injection | NO | YES | C4 defines the rule; C5 proves it was met |

O) COMPLETION CRITERIA
- C4 is complete when the build plan makes the following non-inferential:
- which C2 and C3 storage rows are authoritative
- which PH1.J / PH1.GOV / PH1.LAW / PH1.OS surfaces are evidence-only or decision-only
- which lifecycle transitions require proof visibility, governance visibility, and law visibility
- which lifecycle transitions are not complete until those required downstream surfaces succeed
- how replay decides between backfilling downstream visibility and refusing duplicate authoritative mutation
- how persistence anomaly, proof failure, governance drift, and law failure escalate
- how C4 stops cleanly before C5
- Approval-grade C4 means:
- no proof surface is treated as alternate authority
- no governance/law surface is treated as a lifecycle writer
- no protected lifecycle action can silently look complete when required proof/law posture is missing
- no replay path can invent a second authoritative mutation
- no boundary between C4 and C5 is left inferential
