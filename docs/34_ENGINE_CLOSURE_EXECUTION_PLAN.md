# Engine Closure Execution Plan (Hybrid, End-to-End)

## 0) Purpose
- Define one canonical execution playbook to move Selene OS from current build state to production readiness.
- Prevent drift/loss of direction by combining:
  - global roadmap control, and
  - one-engine-at-a-time closure units.

## 1) Operating Model (Hybrid)
- Use `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/33_ENGINE_REVIEW_TRACKER.md` as fixed engine order and status truth.
- Execute one engine at a time as the unit of closure.
- Keep cross-engine relationship checks mandatory before marking any engine `DONE`.
- Do not bypass gates:
  - No Simulation -> No Execution
  - Access/Authority gate enforced by `PH1.ACCESS.001 -> PH2.ACCESS.002`
  - Engines never call engines directly; Selene OS orchestrates.

## 2) Canonical Control Files
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/33_ENGINE_REVIEW_TRACKER.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/07_ENGINE_REGISTRY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/06_ENGINE_MAP.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/08_SIMULATION_CATALOG.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/09_BLUEPRINT_REGISTRY.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/10_DB_OWNERSHIP_MATRIX.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/COVERAGE_MATRIX.md`
- `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/03_BUILD_LEDGER.md`

## 3) Per-Engine Closure Unit (Repeat Per Engine)
1. Lock target engine ID and related-engine set.
2. Extract target requirements from canonical source docs.
3. Produce explicit gap list:
   - docs gap
   - kernel contract gap
   - runtime/wiring gap
   - test gap
4. Patch docs:
   - DB_WIRING
   - ECM
   - registry/map/coverage/tracker where needed
5. Patch kernel contracts (typed request/response + validation).
6. Patch runtime wiring (engine + OS orchestration path).
7. Add/adjust tests (acceptance + fail-closed + deterministic behavior).
8. Run compile/test gates.
9. Run relationship check gate.
10. Mark tracker status and append build ledger proof entry.

## 4) Relationship Check Gate (Mandatory Per Engine)
1. Upstream/downstream contract compatibility.
2. Capability IDs resolve to ACTIVE entries.
3. Simulation IDs resolve to ACTIVE entries for side effects.
4. Access gate path cannot be bypassed.
5. No engine-to-engine direct call path.
6. DB ownership remains single-writer per table contract.
7. Idempotency/lease invariants enforced where applicable.
8. Reason code coverage and deterministic failure mapping present.
9. Audit envelope integrity present for governed actions.
10. SLO/telemetry hooks present where engine has latency contract.

## 5) Three Production Rounds

### Round 1: Closure Round (Contract + Wiring Closure)
Goal:
- Close all in-scope engines to `DONE` using the per-engine closure unit.

Scope:
- Engine-by-engine completion across tracker order, honoring EXEMPT/MERGED rules.

Required output:
- Docs + kernel contracts + runtime wiring + tests aligned per engine.
- Tracker row closed with proof.

Exit criteria:
- No open actionable engine rows in tracker (excluding EXEMPT/MERGED).
- No unresolved relationship-check items.
- Workspace compiles and targeted engine suites pass.

### Round 2: Hardening Round (Cross-Engine Reliability Closure)
Goal:
- Prove deterministic safety and orchestration behavior end-to-end.

Scope:
- Cross-engine integration, not isolated engine implementation.

Mandatory checks:
1. Gate-order enforcement suites (OS sequencing).
2. Fail-closed matrix (invalid input, missing access, missing simulation, drifted contracts).
3. Idempotency and lease replay/takeover tests.
4. Side-effect dedupe and audit chain integrity.
5. Cross-engine E2E paths for core voice/text flows.

Exit criteria:
- Full workspace test suite passes.
- Integration matrix passes with no critical open defects.
- No regression in previously closed engine acceptance tests.

### Round 3: Production Readiness Round (Operational Release Closure)
Goal:
- Validate runtime operations under production-like conditions.

Scope:
- SLO, staging, operability, and release governance.

Mandatory checks:
1. Latency/SLO measurement for contracted paths (p95/p99 tracked).
2. Staging soak run with deterministic logs/audit traceability.
3. Postgres backup/restore proof and replay integrity checks.
4. Migration traceability and rollback safety checks.
5. Release checklist sign-off (engineering + product + ops).

Exit criteria:
- SLO gates satisfied or approved with explicit bounded exceptions.
- Operational drills passed.
- Release sign-off completed and recorded in build ledger.

## 6) Wave Staging (Execution Safety)
- Wave A (core voice stack): `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.NLP`, `PH1.D`, `PH1.X`, `PH1.WRITE`, `PH1.TTS`, `PH1.L`
- Wave B (governance/execution): access/capreq/position/onboarding/link/broadcast/reminder + enterprise control engines
- Wave C (assist/learning): non-authoritative assist engines and offline optimization chain

Checkpoint cadence:
- Every 3 engines: compile + targeted regression.
- End of each wave: full workspace tests + readiness audit.

## 7) Definition of Done (Program-Level)
- Engine closure complete (Round 1).
- Hardening complete (Round 2).
- Operational readiness complete (Round 3).
- No unresolved critical blockers in tracker/build ledger.

## 8) Change Control Rule
- Any mid-cycle engine/contract/simulation/blueprint change must:
  1. update canonical files in Section 2,
  2. rerun impacted relationship checks,
  3. append a ledger proof entry before proceeding.

## 9) Cross-Engine Risk TODO Backlog (Must Close)
Purpose:
- Convert architecture findings into explicit work items so they are reviewed, decided, implemented, and verified.
- Prevent unresolved overlap/conflict/performance risk from leaking into production rounds.

### 9.1 Decision + Execution Protocol (Per Issue)
1. Review issue with owner + counterpart reviewer (you + implementation lead).
2. Choose one target model (no dual-model ambiguity).
3. Patch docs/contracts/runtime/tests as a single scoped change-set.
4. Run targeted + integration tests.
5. Mark issue status and log proof in `docs/03_BUILD_LEDGER.md`.

### 9.2 Risk Issue Tracker

| issue_id | issue summary | potential impact | decision required | remediation plan | owner | target round | status |
|---|---|---|---|---|---|---|---|
| RISK-01 | Learning ownership overlap (`PH1.LEARN_FEEDBACK_KNOW` vs `PH1.LEARN` + `PH1.FEEDBACK` + `PH1.KNOW`) | Dual-writer risk, drifted artifacts, ambiguous debugging | Choose one ownership model: aggregate storage-only wrapper or full split ownership | (1) locked split runtime ownership in map/registry/db-ownership (`PH1.FEEDBACK`, `PH1.LEARN`, `PH1.KNOW`) and demoted `PH1.LEARN_FEEDBACK_KNOW` to storage-group-only pointer, (2) updated grouped DB_WIRING/ECM boundaries to explicit single-writer artifact types, (3) added storage single-writer invariant enforcement + test (`at_learn_db_05_single_writer_artifact_types_enforced`), (4) added CI/readiness guardrail script (`scripts/check_learning_ownership_boundaries.sh`) | PH1.LEARN | Round 1 | CLOSED |
| RISK-02 | Intent-shaping overlap (`PH1.LANG`, `PH1.SRL`, `PH1.PUZZLE`, `PH1.ATTN`, `PH1.NLP`, `PH1.PRUNE`, `PH1.DIAG`, `PH1.CONTEXT`) | Conflicting outcomes, clarify loops, higher latency | Lock strict precedence chain and one clarify authority | (1) locked clarify ownership in PH1.OS contracts/runtime (`clarify_owner_engine_id` must be `PH1.NLP`), (2) added deterministic optional assist policy bounds (`PH1.PRUNE`/`PH1.DIAG`) with fail-closed top-level refusal, (3) updated map/DB_WIRING/ECM precedence docs, (4) added PH1.OS contract/runtime tests + CI guardrail script (`scripts/check_understanding_clarify_precedence.sh`) | PH1.NLP | Round 1-2 | CLOSED |
| RISK-03 | Governance gate contention (`PH1.POLICY`, `PH1.ACCESS.001/PH2.ACCESS.002`, `PH1.TENANT`, `PH1.GOV`, `PH1.QUOTA`, `PH1.OS`) | Contradictory block/allow outcomes, unsafe gate order drift | Approve one global precedence matrix and terminal-state policy | (1) adopted Section 10 matrix as canonical precedence order, (2) enforced governance decision trace + contradiction fail-closed in PH1.OS contracts/runtime, (3) added conflicting-gate PH1.OS tests, (4) blocks execution on unresolved contradiction | PH1.OS | Round 1-2 | CLOSED |
| RISK-04 | Delivery orchestration overlap (`PH1.LINK`, `PH1.BCAST`, `PH1.DELIVERY`, `PH1.REM`, `PH1.ONBOARDING_SMS`) | Duplicate sends, inconsistent reminder/sms behavior, retry storms | Confirm one lifecycle owner for outbound comms | (1) locked lifecycle boundaries: LINK token only, BCAST lifecycle owner, DELIVERY provider attempts, REM timing, ONBOARDING_SMS gate-only, (2) added idempotency + duplicate-send guard tests (`at_sim_exec_17`, `at_sim_exec_18`), (3) enforced no-cross-owner writes via runtime/CI ownership guardrail (`scripts/check_delivery_ownership_boundaries.sh`) | PH1.BCAST.001 | Round 1-2 | CLOSED |
| RISK-05 | Large TURN_OPTIONAL set can degrade p95/p99 latency | Slower responses and unstable user experience | Define turn-level optional-engine budget policy | (1) kept per-turn optional-engine budget enforcement in PH1.OS orchestration (GATE-U3), (2) added deterministic optional-engine tier classification (`STRICT | BALANCED | RICH`) and utility review scoring over outcome-utilization entries, (3) implemented GATE-U4/GATE-U5 policy actions (`KEEP | DEGRADE | DISABLE_CANDIDATE`) with sustained-fail streak handling, (4) added CI utility-gate checker script (`scripts/check_optional_engine_utility_gates.sh`) and wired readiness audit invocation | PH1.OS | Round 2 | CLOSED |
| RISK-06 | Runtime boundary leak risk for OFFLINE/control-plane engines (`PH1.PATTERN`, `PH1.RLL`, `PH1.GOV`, `PH1.EXPORT`, `PH1.KMS`) | Runtime coupling, unsafe authority drift, performance overhead | Confirm strict out-of-band execution boundary | (1) enforced PH1.OS top-level runtime-boundary fail-closed guard rejecting OFFLINE/control-plane engine ids in live turn wiring, (2) preserved OFFLINE/control-plane ownership by keeping these engines out of `ALWAYS_ON` and `TURN_OPTIONAL` runtime sequences, (3) added CI guard script `scripts/check_runtime_boundary_guards.sh` and wired readiness audit checks for docs/code drift, (4) added PH1.OS runtime tests proving boundary-violation refusal reason-codes | PH1.GOV | Round 1-2 | CLOSED |
| RISK-07 | Namespace duplication pressure over time | Reintroduced duplicate engines and review confusion | Approve duplication prevention guardrail | (1) kept family->implementation merge map in tracker, (2) added guardrail script `scripts/check_engine_tracker_duplicates.sh` and wired it into readiness audit script, (3) enforced merge-note requirement and normalized missing tracker merge-note rows | PH1.F (governance data hygiene) | Round 1 | CLOSED |
| RISK-08 | `PH1.POLICY` docs/runtime wiring drift (`ALWAYS_ON` in map/registry but missing concrete runtime module in `selene_os`) | Prompt dedupe and policy snapshot gate can be bypassed in runtime path; docs/runtime divergence risk | Approve canonical runtime placement and enforcement point for `PH1.POLICY` before `PH1.X` | (1) added concrete `PH1.POLICY` contract + engine + OS wiring modules, (2) enforced call order `Selene OS -> PH1.POLICY -> PH1.X`, (3) added fail-closed contract/wiring/integration tests for invalid/missing policy outputs, (4) enforced `PH1.OS` prompt-policy gate fields in policy evaluate path and decision compute, (5) recorded build proof in ledger | PH1.OS | Round 1 | CLOSED |
| RISK-09 | Full turn orchestrator slice is incomplete (engine-level adapters exist, but no single runtime path proving full `ALWAYS_ON + TURN_OPTIONAL` sequencing) | Fragmented orchestration behavior, inconsistent cross-engine outcomes, weaker end-to-end determinism | Approve one canonical top-level turn orchestrator boundary and sequencing contract | (1) implemented PH1.OS top-level orchestration wiring slice with path-locked ALWAYS_ON sequence checks (voice/text), (2) centralized TURN_OPTIONAL invocation ordering and bounded budget handling at one control point, (3) added voice/text + gate-failure tests proving fail-closed behavior, (4) documented canonical boundary in engine map/registry + PH1.OS DB_WIRING/ECM and recorded ledger proof | PH1.OS | Round 1-2 | CLOSED |
| RISK-10 | `PH1.OS` turn-level optional-engine budget enforcement missing from runtime contract surface | Latency inflation under heavy optional-engine use; unstable p95/p99 and degraded UX | Approve required budget fields and decision policy in `PH1.OS` contracts | (1) extended `PH1.OS` contract/wiring with explicit per-turn optional budget fields (`optional_invocations_requested/budget/skipped` + `optional_latency_budget_ms/estimated_ms`), (2) enforced deterministic skip/degrade policy in top-level orchestration with fail-closed refusal on budget-policy drift, (3) added contract/runtime/wiring tests for GATE-U3 semantics and latency budget breach, (4) preserved owner precedence and `No Simulation -> No Execution` behavior unchanged | PH1.OS | Round 2 | CLOSED |

### 9.3 Issue Closure Gate (Mandatory)
An issue can move from `OPEN` to `CLOSED` only when:
1. Decision is recorded (single chosen model).
2. Canonical docs are updated (`07/06/08/09/10` + relevant DB_WIRING/ECM).
3. Runtime/tests prove behavior and fail-closed semantics.
4. `docs/03_BUILD_LEDGER.md` has an auditable proof entry.

## 10) Strict Owner/Decision Precedence Matrix (Cluster-Level)
Purpose:
- Guarantee one decision owner per cluster.
- Keep all non-owner engines advisory only.
- Fail closed when owner decision is missing, invalid, or contradictory.

### 10.1 Cluster Matrix

| cluster | decision owner (single) | allowed advisors (non-authoritative) | hard fail-closed rule |
|---|---|---|---|
| Understanding + Clarify | PH1.NLP | PH1.LANG, PH1.SRL, PH1.CONTEXT, PH1.PRUNE, PH1.DIAG | If required fields/ambiguity remain or advisor outputs conflict, emit one-field `clarify` and block dispatch/execution |
| Evidence + Query Grounding | PH1.CONTEXT | PH1.E, PH1.SEARCH, PH1.DOC, PH1.SUMMARY, PH1.VISION, PH1.MULTI, PH1.KG | If evidence provenance/citation/validation fails, exclude that evidence and return clarify/refusal-safe response; no side effects |
| Delivery + Outbound Messaging | PH1.BCAST.001 | PH1.LINK, PH1.DELIVERY, PH1.REM.001, PH1.ONBOARDING_SMS | If lifecycle state is incomplete, access is not `ALLOW`, or simulation/idempotency checks fail, no send/resend/notify execution |
| Learning + Personalization | PH1.LEARN | PH1.FEEDBACK, PH1.PAE, PH1.CACHE, PH1.KNOW, PH1.LISTEN, PH1.PERSONA, PH1.EMO.GUIDE, PH1.EMO.CORE, PH1.MULTI, PH1.CONTEXT | If artifact governance/validation is missing, drop advisory updates; never alter authority, confirmation, or execution path |
| Governance + Runtime Control | PH1.OS | PH1.POLICY, PH1.ACCESS.001/PH2.ACCESS.002, PH1.TENANT, PH1.GOV, PH1.QUOTA, PH1.WORK, PH1.LEASE, PH1.SCHED, PH1.CAPREQ | If any mandatory gate returns deny/escalate/invalid or contradictions cannot be resolved deterministically, block commit and end in refusal/escalation only |

### 10.2 Mandatory Precedence Invariants
1. Owner output is the only terminal decision for the cluster.
2. Advisor outputs are hints/signals only; never terminal authority.
3. Conflicting advisor outputs must not produce execution.
4. Missing owner output must be treated as failure, not fallback execution.
5. All cluster owner decisions must be reason-coded and auditable.

### 10.3 Cluster Test Requirements (Add/Keep in Hardening Round)
1. Owner-over-advisor precedence tests for every cluster.
2. Contradictory advisor input tests (must fail closed).
3. Missing owner-output tests (must fail closed).
4. Idempotent replay tests for side-effect clusters.
5. Latency budget checks for hot-path clusters.

## 11) Engine Redundancy / Retirement Backlog (Decision Queue)
Purpose:
- Identify engines that can be merged/retired without weakening safety, authority boundaries, or outcomes.
- Reduce orchestration complexity and p95/p99 latency pressure by removing low-yield overlap.

### 11.1 Retirement Evaluation Gate (Must Pass Before Any Retirement)
1. Owner test: authoritative owner engines are not retirement candidates.
2. Unique-outcome test: candidate has no unique terminal decision/output that is required by blueprint/simulation gates.
3. Wiring test: docs/runtime parity is known; shadow (docs-only) engines are treated as merge/defer candidates.
4. Dependency test: removing candidate does not break canonical blueprint/simulation execution paths.
5. Delta test: A/B run shows no safety regression and acceptable outcome delta under approved thresholds.
6. Performance test: removal/merge improves or preserves p95/p99 and reduces per-turn optional budget pressure.
7. Audit test: decision traces remain reason-coded and replay-safe after merge/retirement.

### 11.2 Ranked Candidate Backlog
| rank | candidate engine/cluster | overlap signal | proposed action | decision gate to close | owner | target round | status |
|---|---|---|---|---|---|---|---|
| 1 | `PH1.LEARN_FEEDBACK_KNOW` | Overlaps split runtime roles of `PH1.LEARN` + `PH1.FEEDBACK` + `PH1.KNOW`; dual-model confusion risk | Keep as persistence/contract grouping only; remove standalone runtime-engine identity | Registry/map/coverage lock now enforces split runtime ownership + grouped storage pointer; storage single-writer artifact type tests and guardrail script are active | PH1.LEARN | Round 1 | CLOSED |
| 2 | `PH1.LEARNING_ADAPTIVE` | Functional overlap with LEARN/PAE feedback-to-adaptation loop; low distinct runtime value | Merged into `PH1.LEARN` + `PH1.PAE`; standalone runtime engine identity retired | Merge proof complete: runtime optional list + registry/map/ownership/coverage + blueprint wiring moved to LEARN/PAE with no standalone runtime references | PH1.LEARN | Round 1-2 | CLOSED |
| 3 | `PH1.POLICY` (separate engine identity) | Docs/runtime mismatch is closed; `PH1.POLICY` now exists as concrete runtime gate under `PH1.OS` orchestration | Keep separate runtime identity with explicit prompt-policy gate contract surface in `PH1.OS` | Canonical path selected as `PH1.POLICY` runtime gate before `PH1.X`, validated with fail-closed contract/runtime tests and ledger proof | PH1.OS | Round 1 | CLOSED |
| 4 | `PH1.REVIEW` | Governance assist overlaps existing approval/governance paths (`PH1.GOV` + access escalation flows) | Merged into governance/access approval routing; standalone runtime identity retired | Merge proof complete: runtime optional list + registry/map/coverage + simulation owning_domain/domain-profile moved under Governance with no standalone runtime references | PH1.GOV | Round 1-2 | CLOSED |
| 5 | `PH1.SEARCH` + `PH1.WEBINT` | Two sequential assist engines around read-only evidence pipeline can duplicate transformation/validation passes | Consolidated into one evidence-query assist pipeline under `PH1.SEARCH`; retired standalone `PH1.WEBINT` identity | Merge proof complete: runtime optional/tier wiring and LEARN target routing now use `PH1.SEARCH` only; context/coverage/registry references updated and standalone `PH1.WEBINT` active identity retired | PH1.CONTEXT | Round 2 | CLOSED |
| 6 | `PH1.PRIORITY` + `PH1.COST` | Both emit routing/pacing metadata only; overlapping turn-policy knobs | Consolidated into one turn-policy budget/priority module under `PH1.COST`; retired standalone `PH1.PRIORITY` identity | Merge proof complete: runtime optional/tier wiring now uses `PH1.COST` only; map/registry/coverage/docs synchronized to unified turn-policy surface with unchanged no-authority/no-execution semantics | PH1.OS | Round 2 | CLOSED |
| 7 | `PH1.ATTN` + `PH1.PRUNE` + `PH1.DIAG` + `PH1.PUZZLE` | Understanding-assist overlap risk (clarify loops/latency inflation) with `PH1.NLP` owner | Consolidated to minimum assist set (`PH1.PRUNE` + `PH1.DIAG`) under `PH1.NLP`; retired standalone `PH1.ATTN` + `PH1.PUZZLE` identities | Merge proof complete: runtime optional list/tier and map/registry/coverage/docs now keep only `PH1.PRUNE` + `PH1.DIAG` as understanding assists; no clarify-owner precedence changes | PH1.NLP | Round 2 | CLOSED |

### 11.3 Non-Candidate Guardrail (Do Not Retire)
1. Core authoritative path: `PH1.K`, `PH1.W`, `PH1.VOICE.ID`, `PH1.C`, `PH1.NLP`, `PH1.X`, `PH1.E`, `PH1.L`.
2. Governance/storage truth owners: `PH1.F`, `PH1.J`, `PH1.ACCESS.001/PH2.ACCESS.002`, `PH1.OS`, `PH1.WORK`, `PH1.LEASE`, `PH1.SCHED`, `PH1.QUOTA`.
3. Side-effect owners: `PH1.BCAST.001`, `PH1.DELIVERY`, `PH1.REM.001`, `PH1.ONB`, `PH1.LINK`, `PH1.POSITION`.

### 11.4 Closure Rule For This Backlog
1. No candidate can move to `CLOSED` without a recorded decision (KEEP/MERGE/RETIRE) plus rationale.
2. KEEP requires measurable unique value proof.
3. MERGE/RETIRE requires docs/contracts/runtime/tests + ledger proof updates in one scoped change-set.
4. Any safety regression automatically reopens the candidate and rolls back activation.

## 12) Outcome Utilization Execution Block (Latency + Compute ROI)
Purpose:
- Ensure every engine outcome/report is machine-consumed and results in action, learning, bounded audit, or deterministic drop.
- Eliminate "generated but unused" outcomes that add latency/compute cost without product value.

### 12.1 Baseline Snapshot (Current)
1. Engine map scope: 52 PH1 engines
2. Contract outcome surfaces: 53 `Ph1*Response` enums
3. OS wiring outcome surfaces: 50 `*WiringOutcome` types
4. Current risk signal: PH1.OS now enforces top-level orchestration, optional-budget contract posture, utility demotion scoring (`GATE-U1..U5`), runtime-boundary guards for OFFLINE/control-plane engines, delivery ownership fail-closed guardrails for LINK/BCAST/DELIVERY/REM/ONBOARDING_SMS, understanding/clarify precedence fail-closed guards (`PH1.NLP` single clarify owner + optional assist policy bounds), and learning ownership split with storage-group-only persistence pointer (`PH1.LEARN_FEEDBACK_KNOW`); no open critical risk items remain in Section 9.2.

### 12.2 Outcome Action Contract (Mandatory For Every Emitted Outcome)
Every emitted outcome/report must be classified by Selene OS as exactly one:
1. `ACT_NOW`: affects current turn decision/order/gates now
2. `QUEUE_LEARN`: queued for governed learning/adaptation pipeline
3. `AUDIT_ONLY`: persisted only for bounded audit/replay use
4. `DROP`: explicitly discarded as no-value for current policy/profile

Required metadata for each outcome event:
1. `engine_id`
2. `outcome_type`
3. `correlation_id`
4. `turn_id`
5. `action_class`
6. `consumed_by` (owner engine or `NONE`)
7. `latency_cost_ms`
8. `decision_delta` (`true|false`)
9. `reason_code`

Hard rule:
- No unclassified outcome is allowed.
- No human review dependency is allowed for routine outcome triage.

### 12.3 Machine-Only Review Loop (Selene Reviews Outcomes)
1. Per-turn: `PH1.OS` classifies each outcome via the Action Contract.
2. Post-turn: outcome-utilization rows are appended to ledger storage (append-only).
3. Batch window: Selene computes per-engine utility and waste scores.
4. Policy action: Selene auto-applies `KEEP | DEGRADE | DISABLE_CANDIDATE` for optional engines.
5. Governance-only escalations: only high-risk merges/retirements require formal sign-off.

### 12.4 Thresholds (Concrete)
Window defaults:
1. rolling_turn_window: 5000
2. review_cadence: every 24h
3. fail_streak_window_days: 7

Global thresholds:
1. outcome_classification_coverage: 100%
2. unresolved_outcome_rate: 0%
3. reason_code_coverage: 100%

TURN_OPTIONAL efficiency thresholds (per engine):
1. `decision_delta_rate >= 0.08` OR `queue_learn_conversion_rate >= 0.20`
2. `no_value_rate (DROP) <= 0.60`
3. `latency_cost_p95_ms <= 20`
4. `latency_cost_p99_ms <= 40`

Always-on safety thresholds:
1. classification coverage: 100%
2. unresolved outcomes: 0%
3. no gate-order/safety regressions: 0 tolerance

OFFLINE_ONLY thresholds:
1. online invocation count: 0 (hard fail if non-zero)
2. artifact recommendation acceptance rate: `>= 0.15` after warmup, otherwise demote frequency

### 12.5 Pass/Fail Gates
| gate_id | gate | pass condition | fail action |
|---|---|---|---|
| GATE-U1 | Classification completeness | 100% outcomes have valid `action_class` | block release; fail closed |
| GATE-U2 | Unresolved outcomes | 0 outcomes with missing `consumed_by` when `action_class` is not `DROP`/`AUDIT_ONLY` | block release; route to owner fix |
| GATE-U3 | Optional budget | per-turn optional-engine budget enforced with deterministic skip/degrade | disable optional tier on breach |
| GATE-U4 | Optional engine utility | thresholds in 12.4 satisfied for rolling window | mark engine `DISABLE_CANDIDATE` |
| GATE-U5 | Sustained low utility | GATE-U4 fail for 7 consecutive days | open MERGE/RETIRE patch item |
| GATE-U6 | Safety after disable | no increase in critical fail/safety reason-codes beyond +0.2% | rollback disable immediately |
| GATE-U7 | Latency ROI | overall turn p95 does not regress (>3%) after changes | rollback and re-tune |

### 12.6 Execution Stages
Stage A (Round 1): Instrument + Classify
1. Add Action Contract fields to OS orchestration output path.
2. Add outcome-utilization append-only ledger rows.
3. Enforce GATE-U1 and GATE-U2 in CI.

Stage B (Round 2): Budget + Auto-Demotion (Implemented)
1. Enforce per-turn optional-engine budget in `PH1.OS`.
2. Enable daily machine-only utility scoring.
3. Auto-demote low-yield optional engines via GATE-U4/GATE-U5.

Stage C (Round 2-3): Consolidation + Retirement
1. Execute ranked merge/retire backlog from Section 11.
2. Apply GATE-U6 and GATE-U7 after each merge/retire change-set.
3. Record proofs in build ledger for every decision.

### 12.7 Immediate Candidate Actions Under This Block
1. `PH1.LEARN_FEEDBACK_KNOW`: completed lock as persistence grouping only (no standalone runtime path).
2. `PH1.LEARNING_ADAPTIVE`: completed merge into `PH1.LEARN` + `PH1.PAE`; standalone runtime identity retired.
3. `PH1.SEARCH` + `PH1.WEBINT`: completed consolidation under `PH1.SEARCH`; standalone `PH1.WEBINT` retired.
4. `PH1.PRIORITY` + `PH1.COST`: completed consolidation under `PH1.COST`; standalone `PH1.PRIORITY` retired.
5. `PH1.ATTN` + `PH1.PRUNE` + `PH1.DIAG` + `PH1.PUZZLE`: completed consolidation to `PH1.PRUNE` + `PH1.DIAG`; standalone `PH1.ATTN` + `PH1.PUZZLE` retired.

## 13) Governed Self-Change Pipeline (Runtime Selene + Builder Selene)
Purpose:
- Enable Selene to improve code/config safely from factual signals.
- Keep production runtime deterministic and non-self-modifying.
- Require explicit approvals for risk-bearing changes.

### 13.1 Non-Negotiable Boundary Split
1. Runtime Selene (`PH1.OS` path):
   - may emit signals, scores, and patch requests,
   - must never write source code, migrations, or release state directly.
2. Builder Selene (offline control plane):
   - may generate patch proposals and run validation in sandbox/staging,
   - may never bypass approval and release gates.
3. Hard rule:
   - `No Approval -> No Merge`
   - `No Simulation/Access Gate -> No Runtime Execution`

### 13.2 System Components (Concrete)
1. Signal Intake:
   - input: outcome-utilization ledger + audit/reason codes + latency/SLO counters.
   - output: normalized `improvement_signal` records.
2. Proposal Engine:
   - input: clustered `improvement_signal` records.
   - output: patch proposal package (`diff`, rationale, risk tier, expected effect).
3. Sandbox Validator:
   - runs deterministic checks: compile, tests, gate scripts, migration safety, SLO guard.
4. Approval Gateway:
   - enforces risk-tier policy (auto/one-approver/two-approver).
5. Release Controller:
   - staging -> canary -> production progression with rollback hooks.
6. Post-Deploy Judge:
   - compares before/after metrics and accepts/reverts change automatically by policy.

### 13.3 Change Classes + Approval Policy
| class | examples | authority | required approval |
|---|---|---|---|
| CLASS-A (low risk) | optional-engine ranking thresholds, safe cache hints, wording/template non-authority patches | Builder Selene | Auto-apply allowed only if all validation gates pass and rollback is ready |
| CLASS-B (medium risk) | runtime wiring updates outside authority path, test additions, non-breaking contract tightening | Builder Selene | One human approval required before merge |
| CLASS-C (high risk) | access/authority/simulation gate logic, DB schema migrations, privacy/retention behavior | Builder Selene | Two human approvals required (`tech + product/security`) |

Hard rule:
- Any change touching `PH1.ACCESS.001`, `PH2.ACCESS.002`, `PH1.OS` gate order, or simulation commit paths is forced to `CLASS-C`.

### 13.4 Proposal Contract (Mandatory Fields)
Every patch proposal must include:
1. `proposal_id`
2. `created_at`
3. `source_signal_window`
4. `source_signal_hash`
5. `target_files[]`
6. `change_class`
7. `risk_score` (0.0..1.0)
8. `expected_effect` (latency, quality, safety deltas)
9. `validation_plan`
10. `rollback_plan`
11. `status` (`DRAFT | VALIDATED | APPROVED | RELEASED | REVERTED`)

Hard rule:
- Missing any mandatory field blocks approval.

### 13.5 Validation Gates (Builder Pipeline)
| gate_id | gate | pass condition | fail action |
|---|---|---|---|
| BLD-G1 | Reproducible diff | patch applies cleanly and deterministically | reject proposal |
| BLD-G2 | Compile/test | required crate/workspace tests pass | reject proposal |
| BLD-G3 | Contract guardrails | kernel contract invariants remain valid | reject proposal |
| BLD-G4 | Ownership/precedence | single-owner matrices and fail-closed checks pass | reject proposal |
| BLD-G5 | Runtime boundary | OFFLINE/control-plane boundary checks pass | reject proposal |
| BLD-G6 | Delivery/idempotency | no duplicate-send or idempotency regressions | reject proposal |
| BLD-G7 | Migration safety (if schema touched) | upgrade + rollback drill passes | reject proposal |
| BLD-G8 | Security/privacy | no policy/retention/secret leakage violations | reject proposal |
| BLD-G9 | Latency guard | projected p95/p99 regression within approved budget | reject proposal |
| BLD-G10 | Audit traceability | reason-code + correlation/audit completeness preserved | reject proposal |

### 13.6 Rollout Policy (Release Controller)
1. Stage 1: Staging (100% synthetic + replay fixtures)
2. Stage 2: Canary (1-5% controlled traffic/class)
3. Stage 3: Ramp (25% -> 50% -> 100%)

Promotion conditions at each stage:
1. no critical safety regressions,
2. p95/p99 within threshold,
3. error/fail-closed rates within baseline tolerance,
4. audit completeness remains 100%.

### 13.7 Auto-Rollback Triggers
Rollback immediately if any occurs:
1. authority/gate-order violation,
2. duplicate side-effect event class appears,
3. p95 latency regression > 3% sustained for 30 minutes,
4. p99 latency regression > 5% sustained for 15 minutes,
5. critical reason-code spike > +0.2% absolute vs baseline.

### 13.8 Implementation Plan (Concrete Build Sequence)
Phase 13-A (contracts + ledger schema) [Implemented]:
1. add proposal/run/result contract types to `selene_kernel_contracts`,
2. add append-only storage tables for proposals/runs/results in `selene_storage`,
3. add idempotency keys for proposal and validation runs.

Phase 13-B (builder orchestration module) [Implemented]:
1. implement offline builder orchestration module (not in runtime `ALWAYS_ON`/`TURN_OPTIONAL`),
2. implement deterministic signal clustering and proposal generation,
3. implement sandbox execution + gate result collector.

Phase 13-C (approval + release controls) [Implemented]:
1. add approval state machine with class-based approver requirements,
2. add release controller with staged rollout state and rollback hooks,
3. block production rollout when approval class is unresolved.

Phase 13-D (observability + close-loop) [Implemented]:
1. add before/after metric comparison and automatic accept/revert decision logic,
2. log every decision in `docs/03_BUILD_LEDGER.md`,
3. wire CI job to fail on missing proposal fields or missing gate outcomes.

### 13.9 Program-Level Success Criteria
1. Runtime Selene remains non-self-modifying.
2. Builder Selene can deliver validated patch proposals with full audit trace.
3. High-risk changes cannot ship without explicit approval.
4. Failed changes roll back automatically within policy thresholds.
5. Net outcome: lower latency waste, fewer regressions, faster safe iteration.

### 13.10 Stage-2 Canary Entry Checkpoint (2026-02-17)
Status:
- `PASS` (entry criteria met; canary execution still requires explicit release approval).

Mandatory entry gates and current proof:
1. `BLD-G1..BLD-G10` pipeline guardrails must pass on current tree.
2. Runtime boundary + ownership + clarify precedence guardrails must pass.
3. Optional utility gates (`GATE-U4/U5`) must pass with no disable candidates.
4. Isolated Selene Postgres verification must pass (`selene_os_dev` only; no foreign DB/role/active connections).
5. Builder offline/runtime release-controller tests must pass.
6. Workspace regression must pass before canary start.
7. Stage-2 promotion threshold gate must pass (`p95/p99/reason spike/audit completeness/fail-closed`).

Proof command bundle (latest passing run):
```bash
scripts/dev_postgres_verify_isolation.sh
bash scripts/check_builder_stage2_canary_replay.sh
bash scripts/check_builder_stage2_promotion_gate.sh docs/fixtures/stage2_canary_metrics_snapshot.csv
bash scripts/check_builder_pipeline_phase13a.sh
bash scripts/check_builder_pipeline_phase13b.sh
bash scripts/check_builder_pipeline_phase13c.sh
bash scripts/check_builder_pipeline_phase13d.sh
bash scripts/check_runtime_boundary_guards.sh
bash scripts/check_learning_ownership_boundaries.sh
bash scripts/check_engine_tracker_duplicates.sh
bash scripts/check_understanding_clarify_precedence.sh
bash scripts/check_optional_engine_utility_gates.sh docs/fixtures/optional_engine_utility_snapshot.csv --fail-on-u4
cargo test -p selene_os ph1builder -- --nocapture
cargo test --workspace
scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Passing this checkpoint authorizes only Stage-2 canary entry readiness, not automatic promotion to Stage-3 ramp.
- Any rollback trigger in Section `13.7` immediately invalidates this checkpoint until re-proven.

### 13.11 Stage-3 Ramp Promotion Gate (Real Telemetry Required)
Status:
- `ENFORCED_BY_GATE` (promotion is blocked unless release gate passes on exported real canary telemetry).

Promotion gate source:
1. Export latest canary telemetry from isolated Selene Postgres (`builder_post_deploy_judge_results` + gate coverage).
   - Freshness is mandatory: telemetry age must be within `MAX_TELEMETRY_AGE_MINUTES` (default `180`).
2. Compute deterministic promotion metrics:
   - `p95_delta_bp`
   - `p99_delta_bp`
   - `critical_reason_spike_bp`
   - `audit_completeness_bp`
   - `fail_closed_delta_bp`
   - authority/duplicate side-effect violation flags
3. Evaluate against Section `13.7` rollback thresholds and fail-closed rules.

Mandatory command (before any Stage-3 ramp action):
```bash
bash scripts/check_builder_stage3_release_gate.sh .dev/stage2_canary_metrics_snapshot.csv
```

Readiness-audit integration:
- Optional hard enforcement is available via:
```bash
ENFORCE_STAGE3_RELEASE_GATE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- No successful `CHECK_OK builder_stage3_release_gate=pass` means no Stage-3 ramp progression.
- No canary telemetry rows (`NO_CANARY_TELEMETRY`) is an automatic fail-closed block on Stage-3 ramp.
- Stale canary telemetry (`STALE_CANARY_TELEMETRY`) is an automatic fail-closed block on Stage-3 ramp.

### 13.12 Builder Human Permission Interrupt Loop (BCAST + REM)
Status:
- `REQUIRED_FOR_OPERATION` (policy lock).

Mission:
- Before any Builder-driven code write/generation action, Selene must interrupt via `PH1.BCAST.001` and request explicit human permission.
- Before any launch/ramp progression, Selene must issue a second explicit permission request.
- If the user is busy/unavailable, Selene must schedule reminder follow-ups via `PH1.REM.001` (timing-only) and remain blocked until approval.
- Human-facing permission messages must stay plain-language only (`issue`, `fix`, `should I proceed?`, `all tests passed, can I launch?`).
- Daily cadence is mandatory: Selene must run one daily review cycle and refresh `DAILY_REVIEW_DATE_UTC` before either permission gate can pass.

Non-negotiable sequence:
1. Compose change brief (machine-generated, plain language):
   - `Issue` ("I received these issues ...")
   - `Fix` ("This is the fix ...")
   - `Should I Proceed` ("Should I proceed?")
   - `Launch Question` ("All tests passed. Can I launch?")
2. Send permission request through `PH1.BCAST.001` lifecycle:
   - `BCAST_CREATE_DRAFT` -> `BCAST_DELIVER_COMMIT`
3. Wait for explicit decision:
   - code phase: approve/deny code change
   - launch phase: approve/deny launch progression
4. Busy path:
   - if no decision and busy mode is active, schedule `PH1.REM.001` follow-up (`reminder_type=BCAST_MHP_FOLLOWUP`) and keep state blocked
5. Only after approval:
   - code phase gate may pass
   - launch phase gate may pass

Hard rules:
- No permission means no code generation/write.
- No launch permission means no Stage-2/Stage-3 progression.
- Reminder scheduling does not grant permission; it only preserves follow-up continuity.
- No daily review refresh (`DAILY_REVIEW_OK=1` + `DAILY_REVIEW_DATE_UTC=today(UTC)`) means no code/launch permission gate pass.
- All approvals/denials/reminder handoffs must be reason-coded and auditable.

Operational gate checks:
```bash
# uses .dev/builder_change_brief.md + .dev/builder_permission.env
bash scripts/check_builder_human_permission_gate.sh code
bash scripts/check_builder_human_permission_gate.sh launch
```

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_HUMAN_PERMISSION=1 scripts/selene_design_readiness_audit.sh
```

Template:
- permission env template: `docs/fixtures/builder_permission_template.env`

### 13.13 Operator Runbook (Code Permission + Launch Permission)
Step A: Prepare brief + permission state files.
```bash
cp docs/fixtures/builder_change_brief_template.md .dev/builder_change_brief.md
cp docs/fixtures/builder_permission_template.env .dev/builder_permission.env
```
Then set daily review fields in `.dev/builder_permission.env`:
- `DAILY_REVIEW_OK=1`
- `DAILY_REVIEW_DATE_UTC=$(date -u +%Y-%m-%d)`

Step B: Fill brief and send code permission request (BCAST lifecycle).
1. Populate `.dev/builder_change_brief.md` with:
   - issue
   - fix
   - should-I-proceed question
   - launch question
2. Send BCAST request using:
   - `BCAST_CREATE_DRAFT`
   - `BCAST_DELIVER_COMMIT`
3. Apply decision into `.dev/builder_permission.env` using the decision ingest script:
```bash
BCAST_ID=<code_bcast_id> DECISION_REF=<code_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code approve
```
Alternative (decision file):
```bash
cp docs/fixtures/builder_permission_decision_template.env .dev/builder_code_decision.env
# fill: PHASE=code, DECISION=approve, BCAST_ID, DECISION_REF
DECISION_FILE=.dev/builder_code_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```
4. If busy/no decision:
   - schedule `REMINDER_SCHEDULE_COMMIT` (`reminder_type=BCAST_MHP_FOLLOWUP`)
   - apply pending state:
```bash
REMINDER_REF=<code_reminder_ref> ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code pending
```

Step C: Enforce code gate before any code generation/write.
```bash
TODAY_UTC="$(date -u +%Y-%m-%d)"
sed -i.bak "s/^DAILY_REVIEW_OK=.*/DAILY_REVIEW_OK=1/" .dev/builder_permission.env
sed -i.bak "s/^DAILY_REVIEW_DATE_UTC=.*/DAILY_REVIEW_DATE_UTC=${TODAY_UTC}/" .dev/builder_permission.env
rm -f .dev/builder_permission.env.bak

ENV_FILE=.dev/builder_permission.env BRIEF_FILE=.dev/builder_change_brief.md \
bash scripts/check_builder_human_permission_gate.sh code
```

Step D: Before launch/canary/ramp, repeat permission request for launch.
1. Send launch permission via BCAST (`BCAST_CREATE_DRAFT` -> `BCAST_DELIVER_COMMIT`).
2. Apply launch decision:
```bash
BCAST_ID=<launch_bcast_id> DECISION_REF=<launch_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh launch approve
```
Alternative (decision file):
```bash
cp docs/fixtures/builder_permission_decision_template.env .dev/builder_launch_decision.env
# fill: PHASE=launch, DECISION=approve, BCAST_ID, DECISION_REF
DECISION_FILE=.dev/builder_launch_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```
Optional one-command sync (if both decision files are populated):
```bash
ENV_FILE=.dev/builder_permission.env \
CODE_DECISION_FILE=.dev/builder_code_decision.env \
LAUNCH_DECISION_FILE=.dev/builder_launch_decision.env \
bash scripts/sync_builder_permission_from_decision_files.sh
```
3. If busy/no decision, schedule REM follow-up and set launch reminder fields.

Step E: Enforce launch gate + Stage-3 release gate.
```bash
ENV_FILE=.dev/builder_permission.env BRIEF_FILE=.dev/builder_change_brief.md \
bash scripts/check_builder_human_permission_gate.sh launch

ENFORCE_BUILDER_HUMAN_PERMISSION=1 ENFORCE_STAGE3_RELEASE_GATE=1 \
scripts/selene_design_readiness_audit.sh
```
Preferred strict release path:
```bash
bash scripts/check_builder_release_hard_gate.sh
```
If Step E is green, rollout can proceed automatically through release-controller states (`STAGING -> CANARY -> RAMP25 -> RAMP50 -> PRODUCTION`) under existing Stage-2/Stage-3 fail-closed thresholds.
If this cycle is learning-triggered, also set `ENFORCE_BUILDER_LEARNING_BRIDGE=1`.

Hard rule:
- Any non-pass output from Step C or Step E is a hard block (`No Approval -> No Code`, `No Launch Approval -> No Launch`).

### 13.14 Learning -> Builder Improvement Bridge (Evidence-Backed, Non-Guessing)
Status:
- `REQUIRED_WHEN_LEARNING_TRIGGERED`.

Mission:
- When learning engines produce actionable outcomes, Builder Selene can consume them as structured inputs for deterministic fixes.
- Learning outputs must be evidence-backed and measurable; vague narrative reports are blocked.

Applicable learning sources (minimum):
- `PH1.FEEDBACK`
- `PH1.LEARN`
- `PH1.KNOW`
- optional: `PH1.PAE`, `PH1.CACHE`, `PH1.PRUNE`, `PH1.CONTEXT`

Hard rules:
- No evidence refs means no Builder patch generation from learning signals.
- Learning reports are advisory inputs only; they do not bypass approval, release gates, or authority boundaries.
- The same human permission loop remains mandatory (`Issue/Fix/Should I proceed?` + `All tests passed. Can I launch?`).

Operational gate:
```bash
ENV_FILE=.dev/builder_learning_bridge.env \
bash scripts/check_builder_learning_bridge_gate.sh
```

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_LEARNING_BRIDGE=1 scripts/selene_design_readiness_audit.sh
```

Templates:
- env template: `docs/fixtures/builder_learning_bridge_template.env`
- report template: `docs/fixtures/builder_learning_report_template.md`

### 13.15 Learning-Triggered Runbook (When Improvement Comes From Learning Engines)
Step A: Prepare learning bridge files.
```bash
cp docs/fixtures/builder_learning_bridge_template.env .dev/builder_learning_bridge.env
```

Step B: Auto-generate learning report during Builder offline run.
1. Builder writes `.dev/builder_learning_report.md` by default (or `learning_report_output_path` override).
2. Generated report must include:
   - learning issues received
   - root-cause evidence refs
   - deterministic fix plan
   - expected improvement
   - decision prompt (`Should I proceed with this learning-driven fix?`)
3. Manual edits are allowed, but report structure must still pass the learning bridge gate.

Step C: Fill bridge env and enforce the gate.
1. Set:
   - `LEARNING_TRIGGERED=1`
   - `LEARNING_REPORT_VALIDATED=1`
   - `LEARNING_REPORT_ID=<stable id>`
   - `LEARNING_REPORT_FILE=.dev/builder_learning_report.md`
   - `LEARNING_SOURCE_ENGINES=<comma-separated ids>`
   - `LEARNING_SIGNAL_COUNT=<n>`
2. Run:
```bash
ENV_FILE=.dev/builder_learning_bridge.env \
bash scripts/check_builder_learning_bridge_gate.sh
```

Step D: Continue standard builder flow.
1. Run code permission gate.
2. Run tests/validation gates.
3. Run launch permission gate.
4. Run Stage-3 release gate before ramp.

Hard rule:
- Learning bridge gate pass enables consideration only; execution still requires the standard code+launch permission and release gates.

### 13.16 Single-Command E2E Gate Chain (Learning + Approval + Stage Gates)
Mission:
- Provide one deterministic command that enforces:
  - learning bridge gate
  - code approval gate
  - launch approval gate
  - stage gate (fixture or live telemetry mode)

Operational command:
```bash
bash scripts/check_builder_e2e_gate_flow.sh
```

Modes:
- `STAGE_GATE_MODE=fixture`: uses deterministic fixture stage gate (`check_builder_stage2_promotion_gate.sh`).
- `STAGE_GATE_MODE=live`: uses live telemetry stage gate (`check_builder_stage3_release_gate.sh`).
- `AUTO_SYNC_DECISION_FILES=1`: pre-applies `.dev/builder_code_decision.env` + `.dev/builder_launch_decision.env` into `.dev/builder_permission.env` before permission-gate checks.

Readiness/CI enforcement mode:
```bash
ENFORCE_BUILDER_LEARNING_BRIDGE=1 \
ENFORCE_BUILDER_HUMAN_PERMISSION=1 \
ENFORCE_BUILDER_E2E_GATE_FLOW=1 \
ENFORCE_STAGE3_RELEASE_GATE=1 \
ENFORCE_BUILDER_RELEASE_HARD_GATE=1 \
scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Any failure in the chain is a hard block on patch/launch progression.

### 13.17 Human Brief Auto-Generation (Simple Issue/Fix Prompts)
Mission:
- Remove manual drift in permission briefs by auto-generating a plain-language brief during Builder offline runs.
- Keep operator communication simple and deterministic:
  - issue
  - fix
  - `Should I proceed?`
  - `All tests passed. Can I launch?`

Runtime behavior:
1. `PH1.BUILDER` writes `.dev/builder_change_brief.md` by default (or `change_brief_output_path` override).
2. Generated brief remains compatible with `scripts/check_builder_human_permission_gate.sh`.
3. Brief generation is advisory communication output only; it does not auto-approve code/launch.

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13f.sh
```

Readiness audit:
- Section `1S` enforces Phase13-F brief autogen checks on each run.

Hard rule:
- If brief autogen contract/wiring/test checks fail, permission-loop readiness is blocked until fixed.

### 13.18 Permission Packet Auto-Generation (BCAST + REM Ready)
Mission:
- Ensure each Builder cycle emits a deterministic, machine-readable permission packet that maps the human-approval flow to the exact simulation sequence.
- Keep approvals manual while removing operator ambiguity in "what to send next".

Runtime behavior:
1. `PH1.BUILDER` writes `.dev/builder_permission_packet.md` by default (or `permission_packet_output_path` override).
2. Packet includes two permission phases:
   - code permission request (`Should I proceed?`)
   - launch permission request (`All tests passed. Can I launch?`)
3. Packet includes deterministic simulation mapping:
   - `BCAST_CREATE_DRAFT`
   - `BCAST_DELIVER_COMMIT`
   - busy follow-up via `REMINDER_SCHEDULE_COMMIT` (`BCAST_MHP_FOLLOWUP`)

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13g.sh
```

Readiness audit:
- Section `1T` enforces Phase13-G packet autogen checks on each run.

Hard rule:
- Permission packet generation never implies approval; code/launch still require explicit pass from the human permission gate.

### 13.19 Decision Ingest Automation (Explicit Approval Capture)
Mission:
- Reduce manual mistakes when updating `.dev/builder_permission.env` after BCAST decisions.
- Keep approval explicit: this script records decisions; it never creates approvals automatically.

Operational command:
```bash
# code phase
BCAST_ID=<code_bcast_id> DECISION_REF=<code_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code approve

# launch phase
BCAST_ID=<launch_bcast_id> DECISION_REF=<launch_decision_ref> \
ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh launch approve

# pending busy follow-up
REMINDER_REF=<reminder_ref> ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh code pending
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13h.sh
```

Readiness audit:
- Section `1U` enforces Phase13-H decision-ingest checks on each run.

Hard rule:
- Decision ingest updates fields only. Permission still must pass `scripts/check_builder_human_permission_gate.sh` before code/launch progression.

### 13.20 Decision-File Ingest (Structured BCAST Outcome Import)
Mission:
- Allow deterministic approval capture from a structured decision file so operators can import BCAST outcomes with one command.
- Maintain fail-closed behavior on malformed or ambiguous file inputs.

Template:
- `docs/fixtures/builder_permission_decision_template.env`

Operational command:
```bash
# fill PHASE/DECISION/BCAST_ID/DECISION_REF (or REMINDER_REF for pending)
DECISION_FILE=.dev/builder_code_decision.env ENV_FILE=.dev/builder_permission.env \
bash scripts/apply_builder_permission_decision.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13i.sh
```

Readiness audit:
- Section `1V` enforces Phase13-I decision-file ingest checks on each run.

Hard rule:
- Decision file import does not grant authority by itself; `check_builder_human_permission_gate.sh` remains the execution gate.

### 13.21 Auto Decision-Seed Export (Per-Run Files)
Mission:
- Reduce operator setup time by auto-exporting decision seed files on every Builder run.
- Ensure both code and launch decision files are always available and path-linked in the permission packet.

Runtime behavior:
1. Builder auto-writes (by default in `.dev/`):
   - `builder_code_decision.env`
   - `builder_launch_decision.env`
2. Files are prefilled with:
   - `PHASE`
   - `DECISION=approve`
   - `REFRESH_DAILY_REVIEW=1`
   - deterministic `PERMISSION_REF` + `PROPOSAL_ID`
3. Operator fills only event outputs (`BCAST_ID`, `DECISION_REF`) and applies with one command.

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13j.sh
```

Readiness audit:
- Section `1W` enforces Phase13-J decision-seed export checks on each run.

Hard rule:
- Seed-file export is convenience only; explicit approval capture and permission gate checks remain mandatory.

### 13.22 Decision-File Auto-Sync Pre-Gate (Optional One-Command Apply)
Mission:
- Allow one deterministic command to apply both code/launch decision files before running permission gates.
- Keep authority unchanged: this only imports explicit decisions already recorded in decision files.

Operational command:
```bash
ENV_FILE=.dev/builder_permission.env \
CODE_DECISION_FILE=.dev/builder_code_decision.env \
LAUNCH_DECISION_FILE=.dev/builder_launch_decision.env \
bash scripts/sync_builder_permission_from_decision_files.sh
```

E2E command mode:
```bash
AUTO_SYNC_DECISION_FILES=1 \
PERMISSION_ENV_FILE=.dev/builder_permission.env \
bash scripts/check_builder_e2e_gate_flow.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13k.sh
```

Readiness audit:
- Section `1X` enforces Phase13-K decision-file auto-sync checks on each run.

Hard rule:
- Auto-sync never grants approval. If decision files are missing/invalid, sync fails closed and code/launch progression remains blocked.

### 13.23 Strict Release Hard Gate (Single Entrypoint)
Mission:
- Provide one strict command for release readiness so no optional/partial gate path is used.
- Enforce: decision-file auto-sync, human permission gates, learning bridge gate, and live telemetry Stage-3 gate in one flow.

Operational command:
```bash
bash scripts/check_builder_release_hard_gate.sh
```

Required defaults in hard gate:
- `AUTO_SYNC_DECISION_FILES=1`
- `STAGE_GATE_MODE=live`

Readiness audit enforcement:
```bash
ENFORCE_BUILDER_RELEASE_HARD_GATE=1 \
scripts/selene_design_readiness_audit.sh
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13l.sh
```

Readiness audit:
- Section `1Z` enforces Phase13-L hard-gate guardrail checks on each run.

Hard rule:
- This is the canonical release-check entrypoint.
- If hard gate fails (including `NO_CANARY_TELEMETRY`), no launch/ramp progression is allowed.

### 13.24 Freeze Checkpoint (Remote Published)
Status:
- `FROZEN_REMOTE_PUBLISHED` (checkpoint is committed, tagged, and pushed to origin).

Frozen release point:
- commit: `a3a002a` (`chore(release): freeze stage3 fresh-cycle checkpoint`)
- tag: `freeze-stage3-fresh-cycle-20260217`
- ledger proof: `docs/03_BUILD_LEDGER.md` entry `FREEZE_CHECKPOINT_STAGE3_FRESH_CYCLE`
- remote publish proof:
  - `origin/main` head includes commit `65a10ed`
  - `origin` tag `freeze-stage3-fresh-cycle-20260217` exists

Local verification commands:
```bash
git show -s --oneline a3a002a
git tag --list "freeze-stage3-fresh-cycle-20260217"
bash scripts/check_builder_release_hard_gate.sh
ENFORCE_BUILDER_LEARNING_BRIDGE=1 ENFORCE_BUILDER_HUMAN_PERMISSION=1 ENFORCE_BUILDER_E2E_GATE_FLOW=1 ENFORCE_STAGE3_RELEASE_GATE=1 ENFORCE_BUILDER_RELEASE_HARD_GATE=1 scripts/selene_design_readiness_audit.sh
```

Remote verification commands:
```bash
git ls-remote --heads origin main
git ls-remote --tags origin freeze-stage3-fresh-cycle-20260217
git show -s --oneline freeze-stage3-fresh-cycle-20260217^{}
```

Hard rule:
- This published freeze tag is the canonical rollback anchor for the Stage-3 fresh-cycle release checkpoint.

### 13.25 Controlled Rollout Start Command (Single Kickoff Gate)
Mission:
- Start rollout only from a synchronized, published checkpoint with deterministic rollback anchor.
- Avoid manual operator drift across release-controller replay checks, approval gates, and live telemetry gates.

Operational command:
```bash
bash scripts/check_builder_controlled_rollout_start.sh
```

What this command enforces:
1. Local `HEAD` must match `origin/main` (no hidden unpushed rollout start).
2. Freeze tag `freeze-stage3-fresh-cycle-20260217` must exist locally and on remote with the same target commit.
3. Release-controller staged-transition replay tests must pass (`check_builder_stage2_canary_replay.sh`).
4. Canonical strict release hard gate must pass (`check_builder_release_hard_gate.sh`).
   - Live telemetry freshness is fail-closed (`MAX_TELEMETRY_AGE_MINUTES`, default `180`).

Expected pass signal:
```text
CHECK_OK builder_controlled_rollout_start=pass commit=<head> freeze_tag=freeze-stage3-fresh-cycle-20260217 freeze_target=<commit>
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13m.sh
```

Readiness audit:
- Section `1AA` enforces Phase13-M rollout-start guardrail checks on each run.
- Section `1AB` optionally enforces live rollout-start gate execution when:
```bash
ENFORCE_BUILDER_CONTROLLED_ROLLOUT_START=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If this gate fails, rollout does not start.
- No manual bypass is allowed; fix failing precondition(s) and re-run this command.

### 13.26 Controlled Rollback Drill (Dry-Run Revert Safety)
Mission:
- Prove rollback safety remains executable before/while rollout.
- Ensure regression-triggered revert path and missing-gate fail-closed path remain intact.

Operational command:
```bash
bash scripts/check_builder_rollback_drill.sh
```

What this command enforces:
1. Post-deploy judge rollback path is still executable:
   - `at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach`
2. Missing gate outcomes still fail closed:
   - `at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes`

Expected pass signal:
```text
CHECK_OK builder_rollback_drill=pass
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13n.sh
```

Readiness audit:
- Section `1AC` enforces Phase13-N rollback-drill guardrail checks on each run.
- Section `1AD` optionally enforces rollback drill execution when:
```bash
ENFORCE_BUILDER_ROLLBACK_DRILL=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If rollback drill fails, rollout progression is blocked until rollback safety is restored.

### 13.27 Pre-Launch Bundle Command (Single Final Checklist)
Mission:
- Provide one deterministic final command before launch progression.
- Ensure rollout-start gating, rollback safety, and strict release hard-gate are all green in a single operator action.

Operational command:
```bash
bash scripts/check_builder_prelaunch_bundle.sh
```

What this command enforces:
1. Controlled rollout-start gate is green:
   - `check_builder_controlled_rollout_start.sh`
2. Rollback drill safety is green:
   - `check_builder_rollback_drill.sh`
3. Strict release hard-gate is re-confirmed:
   - `check_builder_release_hard_gate.sh`

Expected pass signal:
```text
CHECK_OK builder_prelaunch_bundle=pass
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13o.sh
```

Readiness audit:
- Section `1AE` enforces Phase13-O pre-launch-bundle guardrail checks on each run.
- Section `1AF` optionally enforces pre-launch bundle execution when:
```bash
ENFORCE_BUILDER_PRELAUNCH_BUNDLE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- If pre-launch bundle fails, no launch/ramp progression is allowed.

### 13.28 Controlled Launch Executor (One-Step Stage Advance)
Mission:
- Execute one deterministic release-stage promotion at a time after all gates are green.
- Keep launch progression explicit, idempotent, and fail-closed.

Operational command (preview default):
```bash
bash scripts/check_builder_controlled_launch_execute.sh
```

Execution command (writes one release-state row):
```bash
EXECUTE=1 \
LAUNCH_EXECUTE_ACK=YES \
LAUNCH_EXECUTE_IDEMPOTENCY_KEY=<unique_key> \
bash scripts/check_builder_controlled_launch_execute.sh
```

What this command enforces:
1. Pre-launch bundle passes first (`check_builder_prelaunch_bundle.sh`), unless explicitly disabled with `PRECHECK=0`.
2. Launch permission gate passes (`check_builder_human_permission_gate.sh launch`).
3. Promotion is one-step only (`STAGING -> CANARY -> RAMP_25 -> RAMP_50 -> PRODUCTION`).
4. `PRODUCTION` promotion requires latest approval status `APPROVED`.
5. Execution mode requires explicit ack + idempotency key; duplicate key resolves deterministically as idempotent reuse.

Expected pass signals:
```text
CHECK_OK builder_controlled_launch_execute=preview ...
CHECK_OK builder_controlled_launch_execute=executed ...
CHECK_OK builder_controlled_launch_execute=idempotent_reuse ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13p.sh
```

Readiness audit:
- Section `1AG` enforces Phase13-P launch-executor guardrail checks on each run.
- Section `1AH` optionally enforces launch-executor preview checks when:
```bash
ENFORCE_BUILDER_CONTROLLED_LAUNCH_EXECUTE=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- No execution write is allowed without `EXECUTE=1`, `LAUNCH_EXECUTE_ACK=YES`, and `LAUNCH_EXECUTE_IDEMPOTENCY_KEY`.
- If execution preconditions fail, launch progression is blocked.
- Terminal stages are non-promotable: `PRODUCTION(status=COMPLETED)` and `ROLLED_BACK(status=REVERTED)` must fail with `no_next_stage`.

### 13.29 Stage-Bound Judge Gate (Per-Stage Promotion Proof)
Mission:
- Prevent promotion from one rollout stage to the next using telemetry from a different stage.
- Require a fresh, stage-bound judge pass before each non-initial stage advance.

Operational command (stage-bound mode enabled by default):
```bash
REQUIRE_STAGE_JUDGE=1 \
bash scripts/check_builder_controlled_launch_execute.sh
```

What this command enforces:
1. For `CANARY -> RAMP_25`, `RAMP_25 -> RAMP_50`, and `RAMP_50 -> PRODUCTION`:
   - post-deploy judge telemetry must exist for the current `release_state_id`
   - telemetry must satisfy freshness threshold (`MAX_TELEMETRY_AGE_MINUTES`)
   - stage metrics must pass `check_builder_stage2_promotion_gate.sh`
2. Judge export is scope-bound by:
   - `REQUIRED_PROPOSAL_ID=<current proposal_id>`
   - `REQUIRED_RELEASE_STATE_ID=<current release_state_id>`
3. If scoped telemetry is missing/stale/failing, launch execution fails closed.

Expected pass/fail signal examples:
```text
CHECK_OK builder_controlled_launch_execute=preview ...
NO_CANARY_TELEMETRY: ... (scope=release_state:...)
STALE_CANARY_TELEMETRY: ... (scope=release_state:...)
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13q.sh
```

Readiness audit:
- Section `1AI` enforces Phase13-Q stage-bound judge guardrail checks on each run.
- Section `1AJ` optionally enforces stage-bound preview checks when:
```bash
ENFORCE_BUILDER_STAGE_JUDGE_BINDING=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Stage progression is blocked unless the current stage has fresh, scoped judge evidence that passes promotion thresholds.

### 13.30 Production Soak Watchdog (Fresh Production Judge Required)
Mission:
- Keep production state guarded by fresh, production-scoped post-deploy judge evidence.
- Fail closed if production judge evidence is missing, stale, non-ACCEPT, or policy-incomplete.

Operational command:
```bash
bash scripts/check_builder_production_soak_watchdog.sh
```

What this command enforces:
1. Latest release state for target proposal is exactly:
   - `stage=PRODUCTION`
   - `status=COMPLETED`
2. Latest approval status remains `APPROVED`.
3. A judge row exists for the exact production release_state.
4. Judge action is `ACCEPT` (non-accept is fail-closed).
5. Judge telemetry is fresh and scope-bound:
   - `REQUIRED_PROPOSAL_ID=<proposal_id>`
   - `REQUIRED_RELEASE_STATE_ID=<production release_state_id>`
6. Scoped metrics still pass deterministic promotion thresholds via:
   - `check_builder_stage2_promotion_gate.sh`

Expected pass signal:
```text
CHECK_OK builder_production_soak_watchdog=pass ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13r.sh
```

Readiness audit:
- Section `1AK` enforces Phase13-R production-soak guardrail checks on each run.
- Section `1AL` optionally enforces production soak watchdog execution when:
```bash
ENFORCE_BUILDER_PRODUCTION_SOAK=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Production is not considered stable unless watchdog checks pass with scoped, fresh production judge evidence.

### 13.31 Production Soak Recurring Runner (Fail-Closed Alerting)
Mission:
- Run production soak checks on a recurring schedule with deterministic fail-closed alerting.
- Ensure freshness regressions (including telemetry age breaches) trigger immediate alert + non-zero exit.

Operational command (single tick):
```bash
RUN_MODE=once \
bash scripts/check_builder_production_soak_runner.sh
```

Operational command (recurring loop):
```bash
RUN_MODE=loop \
INTERVAL_MINUTES=60 \
bash scripts/check_builder_production_soak_runner.sh
```

What this command enforces:
1. Calls `check_builder_production_soak_watchdog.sh` each tick.
2. On success:
   - emits `CHECK_OK builder_production_soak_runner=tick_pass ...`
   - appends state line to `STATE_FILE` (default `.dev/builder_production_soak_runner_state.log`).
3. On failure:
   - classifies stale freshness as `PRODUCTION_SOAK_STALE_TELEMETRY` when output contains `STALE_CANARY_TELEMETRY`.
   - emits/records alert lines to `ALERT_LOG_FILE` (default `.dev/builder_production_soak_alerts.log`).
4. `FAIL_CLOSED=1` (default):
   - exits non-zero immediately on any failed tick.

Expected signals:
```text
CHECK_OK builder_production_soak_runner=tick_pass ...
ALERT builder_production_soak_runner=PRODUCTION_SOAK_STALE_TELEMETRY ...
ALERT builder_production_soak_runner=PRODUCTION_SOAK_CHECK_FAILED ...
```

Guardrail command:
```bash
bash scripts/check_builder_pipeline_phase13s.sh
```

Readiness audit:
- Section `1AM` enforces Phase13-S runner guardrail checks on each run.
- Section `1AN` optionally enforces once-mode runner execution when:
```bash
ENFORCE_BUILDER_PRODUCTION_SOAK_RUNNER=1 scripts/selene_design_readiness_audit.sh
```

Hard rule:
- Recurring soak monitoring must be fail-closed by default; stale telemetry must alert and terminate the tick with non-zero status.
