# Selene Payroll + HR Engine — Repo-Truth Functionality Extraction Master Design

REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only repo-truth extraction.

No runtime code was changed.

This document does not authorize implementation.

The document reconstructs current Payroll / HR / Employment / Compensation / Salary / Benefits / Leave / Final Pay / HR lifecycle design and functionality from repo evidence.

Future implementation/refactor/retirement requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

## 1. Executive Summary

Current repo truth does not show a complete standalone Payroll runtime engine. Payroll appears as a protected business domain in the Simulation Catalog with DRAFT-status payroll simulations, in adapter classification/fail-closed logic, and in architecture documents. No `ph1payroll.rs`, `payroll.rs`, payrun runtime module, payslip module, payroll storage table, payroll migration, or payroll engine capability map was found in the active runtime crates.

Current repo truth does not show a complete standalone HR runtime engine. HR appears as future-owner language in architecture docs and as authorized org/vocabulary context in some docs. Employee setup exists mainly through PH1.ONB and PH1.POSITION: onboarding can require employee position/company prerequisites and prefilled setup fields, but that is not an HR employee lifecycle engine.

Payroll and HR are therefore not proven as separate current engines. They are also not proven as one combined current engine. The repo currently has partial Payroll placeholders and governance intent, while HR is mostly absent as an active runtime owner.

Compensation is also not proven as a payroll-adjacent business runtime engine. `PH1.COMP` exists, but repo evidence defines it as deterministic computation for ranking, consensus, normalization, and budget math. It is not a Compensation Engine for salary packages, allowances, bonuses, commissions, tax, payslips, or payroll execution. Position records contain `compensation_band_ref`, and onboarding may carry `compensation_tier_ref`, but those are references and prereq checks, not pay truth.

Active or partial product functions based on repo truth:

| Product Function | Current Status | Repo-Truth Reading |
|---|---:|---|
| Payroll protected intent classification | PARTIAL | Adapter detects public payroll knowledge, private payroll reads, and governed/protected payroll actions. |
| Payroll draft/commit simulations | PARTIAL | `PAYROLL_PREPARE_DRAFT` and `PAYROLL_COMMIT_RUN` exist in `docs/08_SIMULATION_CATALOG.md` with DRAFT status. |
| Payroll runtime execution | NOT_FOUND | No runtime payroll engine/module/table/migration found. |
| HR employee lifecycle | NOT_FOUND | No HR runtime engine/module/table/migration found. |
| Employee onboarding prerequisites | PARTIAL | PH1.ONB validates active company/position and optional compensation tier reference. |
| Salary/pay-rate storage | NOT_FOUND | Position stores `compensation_band_ref` only; no raw salary/pay-rate owner found. |
| Payroll groups | NOT_FOUND | No payroll group table or runtime contract found. |
| Payrun | PARTIAL / DESIGN_ONLY | Simulation catalog names payroll commit run, but no runtime implementation found. |
| Payslips | NOT_FOUND | No payslip contract, engine, table, route, or client surface found. |
| Deductions/taxes/benefits | NOT_FOUND | Simulation catalog payroll draft output mentions deductions, but no calculation engine or tax/benefit rules found. |
| Leave balances | NOT_FOUND | No leave-balance runtime owner found. |
| Termination/resignation/retirement/final pay | NOT_FOUND | Access and HWM docs discuss future boundaries; runtime owner not found. |
| Contractor payment/AP | NOT_FOUND | Contractor payment belongs to future Finance/AP or Supplier/AP unless future repo truth proves otherwise. |
| Payroll approval / salary override | PARTIAL / DESIGN_ONLY | Payroll commit simulation requires access/confirmation; no approval workflow runtime found. |
| Attendance/timesheet handoff | NOT_FOUND | Scheduler/Roster extraction found no workforce attendance/timesheet runtime. |

Biggest risks/gaps:

- PAYROLL_GAP: payroll simulations exist as catalog placeholders, but no payroll engine, storage, payrun, payslip, tax, deduction, benefit, or approval implementation was found.
- HR_GAP: no HR engine owns employment status, employment lifecycle, HR records, probation, disciplinary notices, resignation, termination, retirement, or final pay.
- OWNER_GAP: Compensation, Payroll, HR, Finance/Budget, Attendance, Position, Onboarding, and Access boundaries are described in architecture docs but not implemented as complete owner handoffs.
- SECURITY_GAP: salary, bank, tax, health, benefits, and HR records need field-level access proof; current evidence only shows general protected/private-intent gating.
- AUDIT_GAP: payroll protected fail-closed evidence can be stored, but payroll setup/change/approval/payrun/payslip audit trails are not implemented.

## 2. Payroll vs HR Separation Decision

| Question | Repo Evidence | Answer |
|---|---|---|
| Are Payroll and HR separate current engines? | No `ph1payroll.rs`, `payroll.rs`, `ph1hr.rs`, `hr.rs`, payroll/HR migration, payroll/HR DB ownership row, or payroll/HR runtime module found. Payroll has DRAFT simulations in `docs/08_SIMULATION_CATALOG.md`; HR does not have equivalent runtime evidence. | NO as current active engines; Payroll is PARTIAL as simulation-catalog intent; HR is NOT_FOUND. |
| Should Payroll and HR be separate future owners? | Position Journey, HWM, Access Governance, and Request Lattice docs consistently split employment lifecycle from pay execution. Repo truth lacks a combined engine that would justify merging them. | YES, pending Grand Architecture Reconciliation. |
| Is Compensation separate from Payroll? | `PH1.COMP` exists but is computation, not compensation. Position has `compensation_band_ref`; onboarding has `compensation_tier_ref`; no compensation package engine found. | Current runtime: NO / PARTIAL only by references. Future: YES likely, but NEEDS_RECONCILIATION. |
| Is Leave owned by HR, Payroll, Scheduler/Roster, or missing? | No leave engine/table found. HWM docs split leave classification/evidence/pay calculation across HR, Attendance/Roster, and Payroll. | Current runtime: NOT_FOUND. Future: HR/Leave classification + Attendance/Roster evidence + Payroll calculation. |
| Is Contractor payment owned by Payroll, Finance/AP, Supplier/AP, or missing? | No contractor payment runtime found. Position Journey and HWM docs route contractor payment away from employee payroll unless owner proof exists. | Current runtime: NOT_FOUND. Future: likely Finance/AP or Supplier/AP, not Payroll by default. |

Default future owner law unless repo truth proves otherwise:

- HR owns employment/personnel lifecycle.
- Payroll owns pay calculation and pay execution.
- Compensation owns salary package, allowances, bonus, commission, overtime recommendation/logic where present.
- Finance/Budget owns money truth, budget, profitability, cost-to-company, spend limits.
- Scheduler/Roster/Attendance owns worked-time evidence.
- Access/Governance owns who can view/change/approve payroll/HR fields.
- PH1.ONB collects setup fields but does not own HR/Payroll truth.
- PH1.POSITION owns job/position truth but not pay truth.

## 3. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---:|---:|---|
| Execution law | `AGENTS.md` | docs-only, shell-only, no runtime edits | FOUND | Controls this extraction task. |
| Core runtime boundary | `docs/CORE_ARCHITECTURE.md` | client is never authority; cloud runtime owns protected authority | FOUND | Payroll/HR private/protected actions must not be client-owned. |
| Engine inventory | `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` | `PH1.COMP` row; no Payroll/HR row found | PARTIAL | Inventory proves computation owner, not payroll/HR business owner. |
| Simulation catalog summary | `docs/08_SIMULATION_CATALOG.md` | `PAYROLL_PREPARE_DRAFT`, `PAYROLL_COMMIT_RUN` | PARTIAL | Payroll simulations exist but are `DRAFT`. |
| Payroll draft simulation | `docs/08_SIMULATION_CATALOG.md` | input `employee_id`, `pay_period`; output `payroll_draft_id`, `gross_pay`, `deductions`, `net_pay` | PARTIAL | Declared simulation contract only; runtime implementation not found. |
| Payroll commit simulation | `docs/08_SIMULATION_CATALOG.md` | input `payroll_draft_id`, `confirmation_token`; output `payroll_run_id`, `status: COMMITTED` | PARTIAL | Commit simulation is cataloged, requires access/confirmation, but is not active implementation proof. |
| Payroll DB binding | `docs/08_SIMULATION_CATALOG.md` | Payroll reads `identities`, `tenant_companies`, `positions`; writes `artifacts_ledger` | PARTIAL | Domain binding profile, not a payroll schema. |
| DB ownership matrix | `docs/10_DB_OWNERSHIP_MATRIX.md` | no Payroll/HR owner row found | NOT_FOUND | Payroll/HR persistence owner missing. |
| PH1.COMP contract | `crates/selene_kernel_contracts/src/ph1comp.rs` | `ComputationPacket`, `AggregateMethod`, `ConsensusMethod`, `ComputationExecutionState` | FOUND | Deterministic computation only. |
| PH1.COMP runtime | `crates/selene_os/src/ph1comp.rs` | computation packet builders, ranking, consensus, budget posture | FOUND | Not payroll/compensation business logic. |
| PH1.COMP engine helpers | `crates/selene_engines/src/ph1comp.rs` | `budget_posture`, `utilization_bp`, scoring helpers | FOUND | Utility math; no salary/payroll truth. |
| PH1.COMP DB wiring | `docs/DB_WIRING/PH1_COMP.md` | no standalone DB tables; no direct writes | FOUND | Explicitly prevents direct state mutation. |
| PH1.COMP ECM | `docs/ECM/PH1_COMP.md` | `COMP_RANK_CANDIDATES`, `COMP_EVALUATE_CONSENSUS`, `COMP_COMPUTE_BUDGET_POSTURE` | FOUND | Side effects `NONE`; not compensation owner. |
| Position compensation ref | `crates/selene_kernel_contracts/src/ph1position.rs` | `PositionRecord.compensation_band_ref`, `PositionBandPolicyCheckRequest` | FOUND | Position stores a band ref, not salary truth. |
| Position storage | `crates/selene_storage/src/repo.rs`; `crates/selene_storage/src/ph1f.rs` | position record and band policy check storage paths | FOUND | Supports position compensation-band policy path only. |
| ONB compensation tier prereq | `crates/selene_storage/src/ph1f.rs` | `compensation_tier_ref` must match position `compensation_band_ref` | FOUND | Onboarding prereq, not payroll setup. |
| ONB DB wiring | `docs/DB_WIRING/PH1_ONB.md` | compensation tier ref must match position band ref | FOUND | Confirms ONB consumes setup refs. |
| Adapter payroll lane | `crates/selene_adapter/src/lib.rs` | `payroll_public_business_knowledge_intent`, `payroll_governed_business_intent`, `payroll_private_business_read_intent` | PARTIAL | Adapter classifies payroll requests but does not own payroll truth. |
| Adapter protected detection | `crates/selene_adapter/src/lib.rs` | `h380_detects_protected_intent`, `h411_looks_like_protected_execution` | PARTIAL | Payroll/salary/leave approval prompts fail closed. |
| Adapter public payroll wording | `crates/selene_adapter/src/lib.rs` | hardcoded public payroll advisory response | OWNER_GAP | PH1.WRITE should own future user wording. |
| Adapter payroll tests | `crates/selene_adapter/src/lib.rs` | `h412_payroll_lane_detection_separates_public_read_and_protected_execution` | FOUND | Tests classification, not payroll execution. |
| Protected fail-closed evidence | `crates/selene_storage/src/ph1f.rs` | `protected_fail_closed:payroll:authority_missing`, `no_execution_proof:payroll_tim` | FOUND | Append-only evidence for blocked protected request. |
| PH1.N examples | `crates/selene_engines/src/ph1n.rs` | `payroll.approve`, `Monthly payroll processing` | PARTIAL | Capability/request examples only. |
| PH1.X examples | `crates/selene_engines/src/ph1x.rs`; `crates/selene_kernel_contracts/src/ph1x.rs` | payroll approval examples; salary-change protected-risk fixture | PARTIAL | Risk classification examples, not payroll runtime. |
| Storage migrations | `crates/selene_storage/migrations` | no payroll/payrun/payslip/HR/leave/timesheet tables found | NOT_FOUND | Only employee invitee-type onboarding migration matched broad search. |
| Desktop client | `apple/mac_desktop` | employee onboarding/photo sender verification surfaces; no payroll UI found | PARTIAL | Client is render/input only for ONB surfaces; payroll UI missing. |
| iPhone client | `apple/iphone` | no payroll/HR UI found | NOT_FOUND | No mobile payroll/HR surface located. |
| HTTP adapter | `crates/selene_adapter/src/bin/http_adapter.rs` | onboarding seed paths with `compensation_tier_ref`; no payroll route found | PARTIAL | Adapter payroll runtime route missing. |
| HWM docs | `docs/SELENE_PH1ROSTER_ATTENDANCE_STAFFING_WORKTIME_OPERATIONS_MASTER_DESIGN.md`; `docs/SELENE_PH1WORKLOAD_AWARENESS_NEGOTIATION_PERFORMANCE_MASTER_DESIGN.md` | payroll evidence and reward handoffs | DESIGN_ONLY | Future owner split only. |

## 4. Current Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---:|---|
| employee profile | PH1.ONB/Position refs only | HR | NOT_FOUND | Onboarding session is not employee profile truth. |
| employment status | Not found | HR | HR_GAP | Position lifecycle is job lifecycle, not employment lifecycle. |
| employment type | Position `schedule_type` partial | HR + Position context | PARTIAL | No employment contract/status truth. |
| salary/pay rate | Not found | Payroll / Compensation | PAYROLL_GAP | No raw salary/pay-rate storage found. |
| compensation package | `compensation_band_ref` reference only; `PH1.COMP` is compute | Compensation | PARTIAL | Band/tier refs are not compensation package truth. |
| allowances | Not found | Compensation / Payroll | NOT_FOUND | No allowance model found. |
| commission | Not found | Compensation / Payroll | NOT_FOUND | No commission model found. |
| bonuses | Performance future docs only | Compensation / Payroll | DESIGN_GAP | Reward evidence docs are design-only. |
| overtime | HWM future docs only | Attendance / Compensation / Payroll | NOT_FOUND | No attendance/timesheet runtime. |
| deductions | Payroll simulation output field only | Payroll | PARTIAL | No deduction rules engine. |
| tax/contributions | Not found | Payroll / Compliance | NOT_FOUND | No country tax/contribution engine. |
| benefits | Not found | HR / Payroll / Benefits | NOT_FOUND | No benefits owner. |
| bank/pay details | Not found | Payroll with Access/privacy | NOT_FOUND | No bank detail storage proof. |
| payroll group | Not found | Payroll | NOT_FOUND | No payroll group table or contract. |
| pay frequency | Not found | Payroll | NOT_FOUND | No pay-period calendar or pay frequency model. |
| payroll run/payrun | Simulation catalog only | Payroll | PARTIAL | `PAYROLL_COMMIT_RUN` is DRAFT catalog record. |
| payslip | Not found | Payroll + BCAST/Delivery handoff | NOT_FOUND | No payslip generation/delivery proof. |
| payroll approval | Simulation catalog gate + adapter fail-closed | Payroll + Access/Governance | PARTIAL | No approval workflow runtime. |
| salary override | Protected-risk examples only | Compensation + Payroll + Finance + Access | DESIGN_GAP | No override flow implementation. |
| pay change | Protected-risk examples only | HR / Payroll / Compensation | DESIGN_GAP | No pay-change state machine. |
| final pay | Not found | Payroll + HR | NOT_FOUND | No final-pay owner. |
| termination | Access docs/future HWM only | HR + Access handoff | DESIGN_GAP | No HR termination runtime. |
| resignation | Access docs/future HWM only | HR + Access handoff | DESIGN_GAP | No resignation runtime. |
| retirement | Access docs/future HWM only | HR + Payroll final-pay handoff | DESIGN_GAP | No retirement runtime. |
| probation | Future docs only | HR | NOT_FOUND | No probation owner. |
| leave balances | Not found | HR/Leave + Payroll | NOT_FOUND | No leave balance model. |
| sick/annual/unpaid leave | Adapter protected detection and future HWM only | HR/Leave + Attendance + Payroll | DESIGN_GAP | No leave runtime. |
| HR documents | Not found | HR + Document/Artifact owners | NOT_FOUND | No HR document lifecycle. |
| disciplinary/warning notices | Future HWM performance docs only | HR + PH1.WRITE + Access | DESIGN_GAP | No formal notice runtime. |
| performance-to-reward handoff | Future HWM docs only | HWM Performance -> Compensation/Payroll | DESIGN_GAP | No compensation execution. |
| contractor payment | Future position/HWM docs only | Finance/AP or Supplier/AP | NOT_FOUND | Must not be treated as employee payroll without proof. |
| invoice/AP boundary | Not found for Accounts Payable | Finance/AP | NOT_FOUND | `AP` in Access docs means approval policy in several contexts, not Accounts Payable proof. |
| attendance/timesheet handoff | Scheduler/Roster extraction found missing runtime | Attendance -> Payroll | NOT_FOUND | No worked-time evidence owner. |
| roster/schedule handoff | Position `schedule_type` and future HWM only | Roster/Attendance -> Payroll | PARTIAL / DESIGN_GAP | No payroll handoff implementation. |
| position handoff | `compensation_band_ref`, active-position prereq | Position -> Payroll/HR/Compensation refs | PARTIAL | Position does not create payroll/HR records. |
| onboarding handoff | `compensation_tier_ref`, `start_date`, `working_hours` setup refs | ONB -> Payroll/HR | PARTIAL | ONB collects refs but does not store payroll truth. |
| access/authority gate | Simulation catalog + adapter classification + Access docs | Access/Governance | PARTIAL | Field-level salary access missing. |
| Desktop rendering | Employee ONB surfaces only | Desktop render-only | NOT_FOUND for payroll | No payroll/HR UI found. |
| iPhone rendering | Not found | iPhone render-only | NOT_FOUND | No payroll/HR UI found. |
| Adapter transport | Adapter payroll heuristics | Adapter transport only | OWNER_GAP | Adapter must not own payroll decisions. |
| audit/provenance | Protected fail-closed evidence; sim catalog audit requirements | Payroll/HR audit owner missing | PARTIAL | No payroll event audit table/proof. |
| storage/migrations | No payroll/HR migration found | PH1.F + domain schemas | NOT_FOUND | Payroll/HR persistence gap. |
| old compatibility paths | Adapter protected/public/private payroll heuristics | Runtime owner after activation | PARTIAL | Must be checked before retirement. |

## 5. Current Payroll Lifecycle

Repo truth does not implement a full payroll lifecycle. The only payroll lifecycle evidence is a pair of DRAFT Simulation Catalog records and fail-closed adapter/storage evidence.

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit Evidence | Status / Gaps |
|---|---|---|---|---|---|---|---|
| employee/payee setup | Unknown | ONB employee prereq in `crates/selene_storage/src/ph1f.rs` | company, position, optional compensation tier | onboarding prereq pass/fail | ONB session state only | ONB audit only where implemented | HR_GAP: no payroll profile. |
| payroll field collection | PH1.ONB partial | `compensation_tier_ref`, `start_date`, `working_hours` | sender-prefilled context | onboarding fields | ONB session/draft | ONB evidence | PARTIAL: no bank/tax/payroll fields. |
| salary/pay-rate setup | Not found | none found | unknown | unknown | none | none | PAYROLL_GAP. |
| payroll group assignment | Not found | none found | unknown | unknown | none | none | PAYROLL_GAP. |
| timesheet/attendance input | Not found | Scheduler/Roster extraction found no attendance/timesheet runtime | unknown | unknown | none | none | PAYROLL_GAP / ATTENDANCE_GAP. |
| leave input | Not found | no leave engine/table found | unknown | unknown | none | none | HR_GAP. |
| earnings calculation | Not found | Payroll simulation output names `gross_pay` only | employee/pay period in catalog | gross pay in catalog output | draft record if implemented | catalog requires audit events | DESIGN_GAP: no runtime calculation. |
| deduction/tax/contribution calculation | Not found | Payroll simulation output names `deductions` only | employee/pay period in catalog | deductions in catalog output | draft record if implemented | catalog requires audit events | COMPLIANCE_GAP. |
| payroll draft | Payroll simulation catalog | `PAYROLL_PREPARE_DRAFT` | `employee_id`, `pay_period` | `payroll_draft_id`, `gross_pay`, `deductions`, `net_pay` | draft write declared | `SIMULATION_STARTED`, `SIMULATION_FINISHED`, `SIMULATION_REASON_CODED` declared | PARTIAL; no active runtime found. |
| payroll review | Not found | no review workflow found | payroll draft | unknown | none | none | PAYROLL_GAP. |
| payroll approval | Access + Simulation placeholder | `PAYROLL_COMMIT_RUN`; adapter protected intent | draft id, confirmation token | run id/status in catalog | commit declared | catalog requires audit | PARTIAL; no approval workflow implementation. |
| payrun commit | Payroll simulation catalog | `PAYROLL_COMMIT_RUN` | `payroll_draft_id`, `confirmation_token` | `payroll_run_id`, `status: COMMITTED` | irreversible commit declared | catalog requires audit | DRAFT catalog only. |
| payslip generation | Not found | no `payslip` contract/module/table found | payroll run | payslip | none | none | PAYROLL_GAP. |
| payment/disbursement handoff | Not found | no disbursement module found | approved payrun | payment instruction | none | none | FINANCE/PAYROLL_GAP. |
| payroll correction | Not found | none found | payroll run | correction/reversal | none | none | PAYROLL_GAP. |
| payroll reversal | Not found | catalog notes irreversible without compensating simulation, but no compensating sim found | payroll run | reversal | none | none | PAYROLL_GAP. |
| audit/proof | Partial | sim catalog audit events; storage fail-closed evidence | protected fail-closed refs | internal-history evidence | append-only evidence row | `audit_event:protected_block_1` in test | AUDIT_GAP for successful payroll actions. |

## 6. Current HR / Employment Lifecycle

Repo truth does not implement a full HR/employment lifecycle. The nearest active surfaces are PH1.ONB employee onboarding, PH1.POSITION position lifecycle, and Access future/offboarding design docs. Those are not HR employment truth.

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Changes | Audit Evidence | Status / Gaps |
|---|---|---|---|---|---|---|---|
| employee profile creation | PH1.ONB partial | employee invitee/onboarding session | invitee type, company/position refs | onboarding session | ONB draft/session | ONB ledger where implemented | HR_GAP: no employee profile record. |
| offer/contract setup | Not found | none found | offer terms | contract | none | none | HR_GAP. |
| onboarding handoff | PH1.ONB | PH1.ONB extraction docs and `ph1f.rs` prereq checks | company, position, start date, optional tier | readiness / missing fields | ONB states | ONB audit | PARTIAL; not employment start. |
| employment start | Not found | none found | start date | active employment | none | none | HR_GAP. |
| probation | Not found | future docs only | probation terms | probation status | none | none | HR_GAP. |
| promotion | Access/Position docs mention future risk | none as HR runtime | position/pay/access changes | unknown | none | none | DESIGN_GAP. |
| demotion | Access/Position docs mention future risk | none as HR runtime | position/pay/access changes | unknown | none | none | DESIGN_GAP. |
| role/position change | PH1.POSITION can change position lifecycle/schema; no employee role assignment owner | position refs | position state | position records | PH1.POSITION audit where implemented | PARTIAL outside HR. |
| leave/sick/unavailable | Not found | adapter protected detection and HWM design only | leave request | leave status | none | none | HR_GAP. |
| performance review | HWM design only | HWM Performance doc | performance evidence | manager review | none | none | DESIGN_GAP. |
| warning/disciplinary process | HWM design only | HWM Performance doc | evidence | warning notice | none | none | DESIGN_GAP. |
| resignation | Access docs future only | no HR runtime | resignation notice | employment transition | none | none | HR_GAP. |
| termination | Access docs future only | no HR runtime | termination request | employment transition/access handoff | none | none | HR_GAP / SECURITY_GAP. |
| retirement | Access docs future only | no HR runtime | retirement record | employment transition/final pay | none | none | HR_GAP. |
| offboarding | Access future docs; ONB does not own offboarding | no HR runtime | employee exit | access/payroll/task handoffs | none | none | HR_GAP. |
| rehire/reactivation | Not found | none found | former employee | rehire record | none | none | HR_GAP. |
| audit/proof | Partial generic protected fail-closed | `protected_fail_closed:payroll:authority_missing` | blocked protected prompt | evidence refs | internal history append | test proof | No HR lifecycle audit. |

## 7. Compensation / Earnings Lifecycle

Current repo truth has no Compensation Engine for salary packages or earnings. It has:

- `PH1.COMP`, a deterministic computation engine.
- PH1.POSITION `compensation_band_ref`.
- PH1.ONB `compensation_tier_ref` prereq validation against the position band.
- Future master-design docs for compensation handoff.

| Compensation / Earnings Area | Evidence | Runtime-Backed? | Feeds Payroll? | Finance/Budget Required? | Missing |
|---|---|---:|---:|---:|---|
| base salary | no salary storage found | NO | NO | YES future | Salary owner and access policy. |
| allowances | none found | NO | NO | YES future | Allowance model/rules. |
| commissions | none found | NO | NO | YES future | Commission model/rules. |
| bonuses | HWM reward evidence future docs only | NO | NO | YES future | Bonus approval and payroll handoff. |
| overtime | HWM attendance/staffing future docs only | NO | NO | YES future | Attendance evidence + overtime approval. |
| salary benchmark | Position Journey future doc only | NO | NO | YES future | Source-backed benchmark provider. |
| cost-to-company | Position Journey future doc; `PH1.COMP` budget math can compute bounded numbers if called | NO as business truth | NO | YES | Finance/Budget owner. |
| profitability check | `PH1.COMP` can compute budget posture, not business approval | PARTIAL compute only | NO | YES | Finance/Budget workflow. |
| compensation override | future docs only | NO | NO | YES | Approval escalation. |
| reward eligibility handoff | HWM future docs only | NO | NO | YES | Compensation execution owner. |
| performance-to-bonus handoff | HWM future docs only | NO | NO | YES | Protected reward execution. |

Answer summary:

- Is this separate from Payroll? Current runtime cannot prove it. Future architecture should split Compensation from Payroll.
- Is it only in docs? Compensation business logic is currently docs/reference-only plus position/onboarding refs.
- Is it runtime-backed? No salary/earnings engine is runtime-backed. `PH1.COMP` is runtime-backed computation only.
- Does it feed Payroll? Not in current repo truth.
- Does it require Finance/Budget? Future salary benchmark, CTC, bonus, and override flows require Finance/Budget owner proof.
- What is missing? Compensation package schema, salary/pay-rate storage, access gates, benchmark sources, approval workflows, payroll handoff, audit.

## 8. Leave / Absence / Final Pay

Current runtime support for leave, absence, and final pay is not found.

| Area | Current Evidence | Current Status | Correct Future Owner |
|---|---|---:|---|
| annual leave | Future HWM docs only | NOT_FOUND | HR/Leave classification; Payroll pay treatment; Attendance evidence. |
| sick leave | Future HWM docs and adapter protected detection only | DESIGN_GAP | HR/Leave + Attendance/Roster + Payroll. |
| unpaid leave | Future docs only | NOT_FOUND | HR/Leave + Payroll. |
| public holiday | Future HWM docs only | NOT_FOUND | Scheduler/Compliance evidence + Payroll treatment. |
| leave balance | none found | NOT_FOUND | HR/Leave + Payroll. |
| leave payout | none found | NOT_FOUND | Payroll + HR final-pay owner. |
| absence classification | Future HWM docs only | DESIGN_GAP | HR/Leave, with Attendance evidence. |
| final pay | none found | NOT_FOUND | Payroll + HR. |
| severance | none found | NOT_FOUND | HR + Payroll + Legal/Compliance. |
| termination payout | none found | NOT_FOUND | Payroll + HR. |
| unused leave calculation | none found | NOT_FOUND | Payroll + HR/Leave. |
| notice period | none found | NOT_FOUND | HR. |
| exit package | none found | NOT_FOUND | HR + Payroll + Finance. |

Future rule:

- HR may own leave classification and employment state.
- Scheduler/Roster/Attendance may own actual absence/worked-time evidence.
- Payroll owns pay calculation.
- Finance/Budget owns money truth.
- Access owns who may view/change/approve.

## 9. Contractor Payment / AP Boundary

Contractor payment is not implemented in current repo truth.

| Question | Answer |
|---|---|
| Are contractors modeled? | PARTIAL in future Position/HWM design docs; no active contractor payment runtime found. |
| Are hourly/lump-sum contractor modes present? | DESIGN_ONLY in Position Journey and HWM docs. |
| Are contractor invoices present? | NOT_FOUND. |
| Are contractor hours tracked? | NOT_FOUND; HWM future docs define contractor hours as future Attendance/Task evidence. |
| Are contractor overrun alerts present? | DESIGN_ONLY in HWM docs. |
| Does contractor payment feed payroll or AP? | Current repo truth does not decide. Future owner should likely be Finance/AP or Supplier/AP unless Payroll ownership is explicitly implemented later. |
| Does contractor onboarding differ from employee onboarding? | Future Position Journey says yes; current ONB has invitee types but no contractor payment setup runtime. |
| What is missing? | Contractor profile, contract terms, hourly/lump-sum billing, invoice/AP owner, payment approval, access gates, audit. |

Correct future rule:

Contractor role/setup may connect to PH1.POSITION and PH1.ONB. Contractor work/time evidence may connect to Task/Scheduler/Attendance. Contractor payment should likely connect to Finance/AP or Supplier/AP unless repo truth proves Payroll owns it. Access controls who may view/approve.

Note: existing Access documentation uses `AP` for approval-policy concepts in places. That is not proof of Accounts Payable ownership.

## 10. Interaction With PH1.POSITION

PH1.POSITION currently stores position/job truth and payroll-adjacent references:

- `PositionRecord.compensation_band_ref`.
- `PositionCreateDraftRequest.compensation_band_ref`.
- `PositionBandPolicyCheckRequest`.
- `PositionRecord.schedule_type`.
- `PositionRecord.permission_profile_ref`.

Must-answer results:

| Question | Current Answer |
|---|---|
| Does Position store `compensation_band_ref`? | YES. |
| Does Position store salary/pay truth? | NO evidence. It stores a reference only. |
| Does Position store schedule type only? | It stores `PositionScheduleType`; no actual roster/attendance/payroll schedule truth found. |
| Does Position feed payroll requirements? | PARTIAL by reference only. No Payroll owner consumes it in runtime. |
| Does Position feed compensation preview? | NOT_FOUND in runtime. Future design only. |
| Does Position activate payroll/HR records? | NO evidence. |
| What is current vs missing? | Current: position lifecycle/schema and refs. Missing: payroll/HR record creation, salary setup, pay band owner, payroll group handoff, employee lifecycle handoff. |

Critical rule:

PH1.POSITION defines the job. Payroll/HR/Compensation own pay/employment truth.

## 11. Interaction With PH1.ONB

PH1.ONB currently supports employee setup prerequisites and required-field collection, not Payroll/HR ownership.

Current evidence:

- `ph1onb_employee_position_prereq` validates active company.
- It validates active position.
- It validates that position company matches prefilled company.
- If `compensation_tier_ref` exists, it must match `position.compensation_band_ref`.
- Sender update fields include `start_date`, `working_hours`, `compensation_tier_ref`, and jurisdiction tags.

Must-answer results:

| Question | Current Answer |
|---|---|
| Does onboarding collect payroll fields? | PARTIAL for setup refs such as `compensation_tier_ref`; NOT_FOUND for bank, tax, benefits, payroll group. |
| Does onboarding collect bank details/tax/super/health numbers? | NOT_FOUND in current evidence. |
| Does onboarding create HR employee profile? | NOT_FOUND. |
| Does onboarding create payroll setup? | NOT_FOUND. |
| Does onboarding hand off salary/start date/position/department? | PARTIAL: start date, working hours, company/position refs, and compensation tier refs can be captured; no Payroll/HR owner consumes them. |
| Does onboarding create access before payroll/HR? | ONB can trigger access instance creation after prerequisites; Payroll/HR integration is not proven. |
| What is active vs partial vs missing? | Active: ONB field/prereq mechanics. Partial: payroll-adjacent refs. Missing: HR/payroll profile storage and activation. |

Critical rule:

PH1.ONB collects/coordinated setup fields. Payroll/HR owners store and govern payroll/employment truth.

## 12. Interaction With Scheduler/Roster/Attendance/Timesheet

The Scheduler/Roster repo-truth extraction found no complete workforce Scheduler, Roster, Attendance, Timesheet, Leave, or Workload runtime engine. Therefore current attendance-to-payroll, timesheet-to-payroll, roster-to-payroll, overtime-to-payroll, leave-to-payroll, and contractor-hours-to-AP handoffs are not implemented.

| Handoff | Current Support | Notes |
|---|---:|---|
| attendance to payroll | NOT_FOUND | No attendance runtime/table found. |
| timesheet to payroll | NOT_FOUND | No timesheet runtime/table found. |
| roster to payroll | NOT_FOUND | Position has schedule type; no roster/payroll evidence handoff. |
| overtime to payroll | NOT_FOUND | Future HWM docs only. |
| leave to payroll | NOT_FOUND | Future HWM docs only. |
| contractor hours to AP/payroll | NOT_FOUND | Future contractor handoff design only. |
| public holiday pay | NOT_FOUND | No compliance/calendar owner proof. |
| schedule group/payroll group relation | NOT_FOUND | No payroll group or schedule group runtime. |

Critical rule:

Scheduler/Roster/Attendance may produce approved worked-time evidence. Payroll calculates and pays. Scheduler/Roster must not calculate payroll.

## 13. Interaction With Master Access / Authority

Payroll/HR access and authority are partially represented by protected-action classification and Simulation Catalog requirements, but no complete payroll/HR field-level governance implementation was found.

Must-answer results:

| Question | Current Answer |
|---|---|
| Who can view salary? | NOT_FOUND. Access docs say field-level future owner; no payroll salary field gate exists. |
| Who can edit salary? | NOT_FOUND. Protected salary/change intent exists as risk class only. |
| Who can approve payroll? | PARTIAL: payroll simulations require `POLICY_ROLE_BOUND` resolved by PH1.ACCESS/PH2.ACCESS; no runtime approval path found. |
| Who can approve salary override? | NOT_FOUND. |
| Who can view HR records? | NOT_FOUND. |
| Who can view bank/tax/health/super fields? | NOT_FOUND. |
| Does current repo gate payroll/private HR reads? | PARTIAL: adapter classifies private payroll reads; no payroll data read engine found. |
| Does protected execution require authority/simulation? | PARTIAL: catalog and adapter enforce the law conceptually; no Payroll runtime execution found. |
| What is missing? | Field-level access, authority matrix, approval workflow, private-view audit, payroll/HR records. |

Critical rule:

Payroll/HR data is private/high-risk. Access/Governance controls field-level read/write/approve. Authority + Simulation controls protected payroll execution.

## 14. Interaction With PH1.TASK / HWM / Performance

Current runtime does not implement payroll/HR interaction with PH1.TASK, HWM, or Performance. The HWM master design set defines future evidence flows only.

| Interaction | Current Status | Future Owner Split |
|---|---:|---|
| task performance to bonus eligibility | DESIGN_ONLY | HWM Performance produces evidence; Compensation/Payroll execute approved reward. |
| performance scorecard to reward | DESIGN_ONLY | HWM Performance -> Compensation/Payroll. |
| poor performance to HR review | DESIGN_ONLY | HWM Performance prepares evidence; HR owns formal process. |
| handover tasks during exit | NOT_FOUND | HR/offboarding + PH1.TASK. |
| open-task reassignment during resignation/termination | NOT_FOUND | HR + Access + PH1.TASK/HWM. |
| manager review reminders | DESIGN_ONLY | PH1.REM timing + BCAST delivery. |
| probation performance check | NOT_FOUND | HR + HWM Performance + REM. |

## 15. Interaction With PH1.REM / PH1.BCAST / PH1.DELIVERY

Current repo truth does not show payroll/HR source-truth integration with reminders or delivery.

| Flow | Current Evidence | Status |
|---|---|---:|
| payroll approval reminders | Future docs only | DESIGN_GAP |
| probation review reminders | Future docs only | DESIGN_GAP |
| contract expiry reminders | Future docs only | DESIGN_GAP |
| final pay reminders | Not found | NOT_FOUND |
| payroll run reminders | Not found | NOT_FOUND |
| leave approval notifications | Future docs only | DESIGN_GAP |
| payslip delivery | Not found | NOT_FOUND |
| salary-change notifications | Not found | NOT_FOUND |
| termination/offboarding notifications | Future Access/HWM docs only | DESIGN_GAP |

Critical rule:

PH1.REM owns timing. PH1.BCAST/DELIVERY owns notification delivery. Payroll/HR owns source truth.

## 16. PH1.D / GPT-5.5 / PH1.N / PH1.X Interaction

Current repo truth contains payroll/HR-related protected-routing examples, but no Payroll/HR probabilistic execution path.

Evidence:

- Adapter classifies payroll prompts into public knowledge, private read, or protected execution.
- PH1.N/PH1.X examples include `payroll.approve` capability-style strings and protected payroll handling.
- PH1.X contract fixtures include a salary-change request classified as protected risk.
- PH1.D provider law in architecture docs says provider proposals cannot execute or authorize protected business actions.

Correct future rule:

OpenAI/GPT-5.5 may help:

- understand messy payroll/HR requests
- extract fields
- explain salary breakdowns
- draft HR letters
- summarize payroll changes
- prepare approval explanations
- write employee-friendly explanations
- translate HR/payroll guidance

OpenAI/GPT-5.5 must not:

- calculate final payroll truth
- approve pay changes
- invent tax/labor law
- approve termination
- execute payroll
- expose private fields
- bypass Access/Authority/Simulation

Must-answer results:

| Question | Current Answer |
|---|---|
| Does current repo use GPT/OpenAI for payroll/HR? | NOT_FOUND as Payroll/HR runtime. Provider proposal architecture exists generally. |
| Is PH1.N extraction present? | PARTIAL via generic examples and capability request strings. |
| Is PH1.X route/risk validation present? | PARTIAL via protected payroll/salary examples and adapter fail-closed tests. |
| What future proposal path is needed? | PH1.D proposal -> PH1.N field extraction -> PH1.X payroll/HR route/risk validation -> Access/Authority/Simulation -> Payroll/HR deterministic owner -> PH1.WRITE wording. |

## 17. PH1.WRITE Interaction

Payroll/HR wording is currently not proven as PH1.WRITE-owned.

Evidence:

- Adapter contains hardcoded public payroll advisory wording.
- Adapter contains hardcoded protected-action/fail-closed responses in tests.
- PH1.WRITE master design states final user-facing writing must be generated from validated truth and must not invent business state.
- No payroll/HR-specific PH1.WRITE template owner was found.

Risk labels:

- PAYROLL_WRITING_OWNER_RISK: payroll explanations currently appear in adapter hardcoded pathways.
- HR_WRITING_OWNER_RISK: HR notice/termination/leave wording owner not implemented.
- HARDCODED_PAYROLL_TEXT_RISK: adapter public payroll advisory text is hardcoded.
- CLIENT_PAYROLL_TEXT_RISK: no direct payroll client UI found, but any future client payroll wording must remain render-only.
- ADAPTER_PAYROLL_TEXT_RISK: adapter must not become canonical payroll/HR presentation owner.

Correct future rule:

PH1.WRITE owns final wording for salary explanations, payroll denial, approval requests, payslip explanation, termination/resignation summaries, leave responses, HR notices, sensitive field explanation, and employee-facing messages.

## 18. Desktop / iPhone / Adapter Boundaries

| Surface | Current Behavior | Runtime Behavior | Risk |
|---|---|---|---|
| Desktop payroll UI | No payroll/payrun/payslip/salary UI found. Desktop employee onboarding surfaces exist. | No payroll runtime found. | DESKTOP_PAYROLL_AUTHORITY_RISK if future UI decides payroll locally. |
| Desktop HR UI | No employee lifecycle/termination/HR-record UI found. | No HR runtime found. | DESKTOP_HR_AUTHORITY_RISK if future UI owns HR truth. |
| iPhone payroll UI | Not found. | No payroll runtime found. | IPHONE_PAYROLL_AUTHORITY_RISK if future UI decides payroll locally. |
| iPhone HR UI | Not found. | No HR runtime found. | IPHONE_HR_AUTHORITY_RISK if future UI owns HR truth. |
| Adapter payroll routes | No bounded payroll route found; payroll semantic classification exists in `crates/selene_adapter/src/lib.rs`. | Adapter fails closed or routes public/private/protected intent. | ADAPTER_PAYROLL_AUTHORITY_RISK if heuristics become payroll execution. |
| Adapter HR routes | No HR record/termination/resignation route found. | No HR runtime found. | ADAPTER_HR_AUTHORITY_RISK. |
| Employee profile routes | ONB routes exist; not HR employee profile routes. | ONB handles onboarding session surfaces. | ONB must not become HR truth owner. |
| Private data display | No salary/bank/tax display found. | No payroll data read engine found. | SECURITY_GAP for future private-field display. |

Current conclusion:

Clients are not currently making payroll/HR decisions because no payroll/HR client surface was located. Adapter contains payroll classification and wording, which is useful compatibility evidence but must remain transport/routing only.

## 19. Security / Privacy / Compliance Model

Repo evidence supports a protected-action philosophy but not a complete payroll/HR privacy model.

| Area | Current Evidence | Status |
|---|---|---:|
| salary privacy | Access future docs; no salary fields found | SECURITY_GAP |
| bank details privacy | no bank field storage found | SECURITY_GAP |
| tax identifiers | no tax field storage found | COMPLIANCE_GAP |
| health identifiers | no health field storage found | COMPLIANCE_GAP |
| superannuation/member numbers | not found | COMPLIANCE_GAP |
| HR notes | not found | HR_GAP |
| disciplinary records | future HWM docs only | HR_GAP |
| payroll approvals | simulation catalog concept | PARTIAL |
| field-level access | Access future docs only for payroll/HR | SECURITY_GAP |
| tenant/workspace/company scope | position/onboarding tenant/company scope exists | PARTIAL |
| audit | fail-closed protected evidence; sim catalog audit requirements | AUDIT_GAP |
| idempotency | simulation catalog requires idempotency key for retriable writes | PARTIAL |
| legal/tax compliance | no rule owner found | COMPLIANCE_GAP |
| employment law boundaries | future docs only | COMPLIANCE_GAP |
| public holiday/labor law handling | future HWM docs only | COMPLIANCE_GAP |
| data retention | general architecture docs; no payroll retention rules | SECURITY_GAP |
| employee right to access own data | not found | SECURITY_GAP |
| contractor privacy | not found | SECURITY_GAP |
| payroll export/download | not found | PAYROLL_GAP |

## 20. State Machines

No complete Payroll/HR/Leave/Compensation state machines are implemented in active runtime code. The following are reconstructed only from repo evidence and future design needs.

### Payroll States

RECONSTRUCTED_FROM_REPO_EVIDENCE

Evidence base: `PAYROLL_PREPARE_DRAFT` and `PAYROLL_COMMIT_RUN` catalog entries, adapter protected-gate behavior, and no runtime payroll engine.

| State | Runtime Implemented? | Evidence |
|---|---:|---|
| SetupDraft | NO | Needed before payroll profile exists; no runtime. |
| ReadyForPayroll | NO | No payroll profile owner. |
| PayrollDraft | PARTIAL / CATALOG_ONLY | `PAYROLL_PREPARE_DRAFT` declares draft output. |
| PendingReview | NO | No review workflow found. |
| PendingApproval | PARTIAL / CATALOG_ONLY | `PAYROLL_COMMIT_RUN` requires access/confirmation. |
| Approved | NO | No approval record found. |
| Paid | NO | No payment/disbursement runtime found. |
| Failed | PARTIAL | Protected fail-closed evidence can be stored. |
| Corrected | NO | No correction simulation/runtime found. |
| Reversed | NO | Catalog mentions compensating simulation concept but none found. |
| Archived | NO | No payroll archive owner found. |

### HR / Employment States

RECONSTRUCTED_FROM_REPO_EVIDENCE

Evidence base: ONB employee setup, Position lifecycle, Access future docs. These are not HR state proof.

| State | Runtime Implemented? | Evidence |
|---|---:|---|
| Candidate | NO | No HR candidate engine. |
| Offered | NO | No offer/contract owner. |
| Accepted | NO | No employment acceptance owner. |
| Onboarding | PARTIAL outside HR | PH1.ONB sessions. |
| Active | NO as HR | Position can be Active; employment cannot. |
| Probation | NO | No probation model. |
| OnLeave | NO | No leave model. |
| Suspended | NO as HR | Position can be Suspended; employment cannot. |
| Resigned | NO | No resignation model. |
| Terminated | NO | No termination model. |
| Retired | NO as HR | Position can be Retired; employment cannot. |
| Offboarded | NO | No offboarding owner. |
| Rehired | NO | No rehire owner. |

### Leave States

RECONSTRUCTED_FROM_REPO_EVIDENCE

No leave runtime found.

| State | Runtime Implemented? |
|---|---:|
| Requested | NO |
| Approved | NO |
| Denied | NO |
| Active | NO |
| Completed | NO |
| PaidOut | NO |
| Cancelled | NO |

### Compensation States

RECONSTRUCTED_FROM_REPO_EVIDENCE

No compensation-business runtime found.

| State | Runtime Implemented? | Evidence |
|---|---:|---|
| Proposed | NO | Future docs only. |
| Benchmarked | NO | Future docs only. |
| AboveBenchmark | NO | Future docs only. |
| PendingApproval | NO | Future docs only. |
| Approved | NO | Future docs only. |
| Rejected | NO | Future docs only. |
| Applied | NO | No payroll handoff. |

## 21. Error Handling And Reason Codes

No Payroll/HR runtime error enum was found. Existing reason-code evidence is indirect:

- Simulation catalog preconditions require input schema, tenant boundary, blueprint reference, access, confirmation, idempotency, and audit readiness.
- Adapter protected classification fails closed for payroll and salary-changing prompts.
- Storage internal history can record protected fail-closed evidence such as `protected_fail_closed:payroll:authority_missing`.

Required Payroll/HR reason codes are currently missing or only conceptual:

| Reason / Error | Current Status | Notes |
|---|---:|---|
| employee not found | NOT_FOUND | HR/Payroll employee resolver missing. |
| payroll profile missing | NOT_FOUND | Payroll profile missing. |
| salary missing | NOT_FOUND | No salary owner. |
| pay rate missing | NOT_FOUND | No pay-rate owner. |
| payroll group missing | NOT_FOUND | No payroll group owner. |
| bank details missing | NOT_FOUND | No bank field owner. |
| tax details missing | NOT_FOUND | No tax details owner. |
| access denied | PARTIAL | Access/protected fail-closed evidence. |
| authority missing | PARTIAL | `protected_fail_closed:payroll:authority_missing`. |
| approval required | PARTIAL | Simulation catalog required roles/confirmation. |
| payroll already approved | NOT_FOUND | No payrun state. |
| payroll already paid | NOT_FOUND | No paid state. |
| payroll correction required | NOT_FOUND | No correction flow. |
| payslip missing | NOT_FOUND | No payslip owner. |
| leave balance insufficient | NOT_FOUND | No leave balance owner. |
| final pay blocked | NOT_FOUND | No final pay owner. |
| termination requires approval | DESIGN_GAP | Future HR protected execution. |
| salary override requires approval | DESIGN_GAP | Future Compensation/Finance/Access workflow. |
| tax rule missing | COMPLIANCE_GAP | No tax rule owner. |
| country rule missing | COMPLIANCE_GAP | No country payroll compliance owner. |
| compliance owner missing | COMPLIANCE_GAP | No payroll compliance engine. |
| timesheet missing | NOT_FOUND | No timesheet owner. |
| attendance missing | NOT_FOUND | No attendance owner. |
| contractor invoice missing | NOT_FOUND | No AP/invoice owner. |
| client route mismatch | PARTIAL | Adapter/client routes do not expose payroll; future risk. |

## 22. Audit / Provenance / Evidence

| Audit Question | Current Answer | Status |
|---|---|---:|
| Is payroll setup audited? | No payroll setup runtime found. | AUDIT_GAP |
| Is salary change audited? | No salary change runtime found. | AUDIT_GAP |
| Is salary override audited? | No override runtime found. | AUDIT_GAP |
| Is payroll approval audited? | Simulation catalog requires audit events, but runtime proof missing. | PARTIAL |
| Is payrun audited? | `PAYROLL_COMMIT_RUN` declares audit events, but runtime proof missing. | PARTIAL |
| Is payslip generation audited? | No payslip runtime found. | AUDIT_GAP |
| Are tax/deduction calculations audited? | No tax/deduction calculation runtime found. | AUDIT_GAP |
| Is leave approval audited? | No leave runtime found. | AUDIT_GAP |
| Is final pay audited? | No final pay runtime found. | AUDIT_GAP |
| Is termination/resignation/retirement audited? | No HR lifecycle runtime found. | AUDIT_GAP |
| Are bank/tax/private field views audited? | No private payroll/HR field storage/read path found. | AUDIT_GAP |
| Are access checks audited? | Access architecture supports audit generally; payroll-specific runtime missing. | PARTIAL |
| Are client/adapter payroll events audited? | Protected fail-closed evidence can be appended in storage tests; no successful payroll event audit. | PARTIAL |

Provenance currently supports blocked protected payroll evidence better than successful payroll lifecycle evidence.

## 23. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| Payroll lane detection separates public/private/protected | `crates/selene_adapter/src/lib.rs::h412_payroll_lane_detection_separates_public_read_and_protected_execution` | Adapter classifies payroll prompts into public knowledge, private read, and governed/protected intent. | Payroll execution, payroll data storage, access field gates, payrun. | FOUND |
| Protected continuation stays fail-closed | `crates/selene_adapter/src/lib.rs::stage8_5_protected_do_it_continuation_stays_fail_closed` | Protected continuation does not bypass gates. | Payroll runtime behavior. | FOUND |
| Mixed protected prompt fails closed | `crates/selene_adapter/src/lib.rs::stage9_search_certification_adapter_protected_mixed_prompt_is_fail_closed` | Protected payroll approval in a mixed prompt remains blocked. | Payroll approval implementation. | FOUND |
| Protected fail-closed evidence is append-only | `crates/selene_storage/src/ph1f.rs::at_f_stage7_04_protected_fail_closed_is_append_only_evidence_not_action_success` | Storage can record blocked protected payroll request evidence without action success. | Payroll action audit for successful setup/approval/run. | FOUND |
| PH1.COMP computation tests | `crates/selene_os/src/ph1comp.rs` | Computation/ranking/consensus/budget math behavior. | Compensation, salary, payroll, benefits, or payslips. | FOUND |
| Position compensation band contract/storage tests | `crates/selene_storage/tests/ph1_position/db_wiring.rs`; `crates/selene_kernel_contracts/src/ph1position.rs` | Position can store and validate `compensation_band_ref`. | Salary/pay-rate truth or payroll handoff. | FOUND |
| ONB employee position prereq tests | `crates/selene_storage/tests/ph1_onb/db_wiring.rs`; `crates/selene_storage/src/ph1f.rs` | Onboarding employee prereq can validate active company/position and compensation tier against position band. | HR employee profile or payroll setup. | FOUND |
| Payroll engine tests | no payroll runtime test file found | Nothing. | Full payroll lifecycle. | TEST_GAP |
| HR engine tests | no HR runtime test file found | Nothing. | Employment lifecycle. | TEST_GAP |

## 24. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---:|---|---|---|
| `PH1.COMP` confused with Compensation Engine | Active compute engine | PH1.COMP computes only; Compensation future owner handles salary packages | Compensation engine exists and names are reconciled | Check callers for salary/payroll use of PH1.COMP. |
| Position owning pay truth through `compensation_band_ref` | Partial reference | Position owns job ref only; Compensation/Payroll own pay truth | Compensation/Payroll refs activated | Check no raw salary enters Position. |
| ONB owning payroll truth through `compensation_tier_ref` | Partial reference | ONB collects fields only | Payroll/HR setup owner exists | Check no payroll profile write in ONB. |
| Scheduler/Roster calculating payroll | Future design risk | Attendance/Roster produce evidence; Payroll calculates | Attendance/Payroll handoff activated | Check no payroll math in roster code. |
| Attendance calculating payroll | Future design risk | Attendance owns worked-time facts only | Payroll calculation owner activated | Check no gross/net pay in attendance. |
| Access exposing salary by role string | Risk in future Access templates | Access field-level policy | Payroll/HR field policy implemented | Check role-string shortcuts. |
| Adapter payroll shortcuts | Existing heuristics | Adapter transport/classification only | Runtime payroll owner exists; routes bounded | Check active callers before removal. |
| Desktop/iPhone payroll authority | No current surface but future risk | Clients render/submit bounded inputs only | Payroll UI proof created | Check no local payroll decisions. |
| hardcoded salary/payroll explanations | Adapter public payroll wording exists | PH1.WRITE | PH1.WRITE payroll templates activated | Check adapter text fallback. |
| compensation old docs using old product names | Archive docs contain legacy source text | Modern docs use Selene only | Archive retirement/reconciliation | Do not copy legacy names. |
| payroll inside Finance only | Not implemented | Payroll owns pay execution; Finance owns money/budget truth | Finance/Payroll split reconciled | Check no Finance engine swallows payroll. |
| contractor payment treated as employee payroll without proof | Design risk | Finance/AP or Supplier/AP likely | Contractor/AP owner activated | Check contractor pay flows. |
| HR termination triggering access without correct owner | Future risk | HR owns employment state; Access owns permission changes | Offboarding handoff proof | Check HR/Access event boundaries. |
| duplicate Payroll/HR/Compensation engines | Not found now | Reconciled owner map | Grand Architecture Reconciliation | Check all files before activation. |
| stale docs | Archive and legacy master docs include payroll examples | Current architecture set | Reconciliation pack | Check docs before implementation. |

## 25. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---:|---|
| create employee payroll profile | Store employee/payee payroll setup | Not found | Payroll | NOT_FOUND | Build Payroll Profile model. |
| create HR employee profile | Store employee lifecycle profile | Not found | HR | NOT_FOUND | Build HR Employee Profile model. |
| set salary | Store approved salary | Not found | Payroll / Compensation | NOT_FOUND | Define salary field access and approval. |
| set pay rate | Store hourly/day rate | Not found | Payroll / Compensation | NOT_FOUND | Define pay-rate model. |
| assign payroll group | Assign pay frequency/group | Not found | Payroll | NOT_FOUND | Build payroll group. |
| calculate gross pay | Compute gross earnings | Simulation catalog output only | Payroll / Compensation | PARTIAL | Implement deterministic payroll draft. |
| calculate net pay | Compute net after deductions | Simulation catalog output only | Payroll | PARTIAL | Implement tax/deduction owner. |
| calculate deductions | Tax/deduction calculation | Simulation catalog output only | Payroll / Compliance | PARTIAL | Build country-rule backed deductions. |
| calculate employer contributions | Employer contribution logic | Not found | Payroll / Compliance | NOT_FOUND | Build contribution model. |
| approve payroll | Review/approve payrun | Simulation catalog + protected gate | Payroll + Access | PARTIAL | Build approval workflow. |
| generate payslip | Generate employee payslip | Not found | Payroll + PH1.WRITE/BCAST | NOT_FOUND | Build payslip artifact/delivery boundary. |
| process payrun | Commit payroll run | `PAYROLL_COMMIT_RUN` catalog | Payroll | PARTIAL | Implement only after simulation activation. |
| correct payroll | Correction after error | Not found | Payroll | NOT_FOUND | Build correction/reversal simulations. |
| process final pay | Pay termination/resignation/retirement final amounts | Not found | Payroll + HR | NOT_FOUND | Build final pay workflow. |
| record resignation | Store resignation lifecycle event | Future Access/HWM docs only | HR | DESIGN_GAP | Build HR lifecycle state. |
| record termination | Store termination lifecycle event | Future Access/HWM docs only | HR | DESIGN_GAP | Build protected HR termination flow. |
| record retirement | Store retirement lifecycle event | Future Access docs only | HR | DESIGN_GAP | Build retirement/offboarding flow. |
| calculate leave payout | Pay unused leave | Not found | Payroll + HR/Leave | NOT_FOUND | Build leave balance + payout. |
| collect payroll setup fields in onboarding | Collect setup refs/fields | `compensation_tier_ref`, start/working fields | ONB -> Payroll/HR | PARTIAL | Expand with owner-approved sensitive fields. |
| validate position compensation ref | Ensure ONB tier matches Position band | `ph1onb_employee_position_prereq` | PH1.ONB + PH1.POSITION | FOUND | Keep as ref check, not salary truth. |
| gate protected payroll requests | Fail closed without authority/simulation | Adapter tests + storage protected evidence | Access/PH1.X/Adapter route | PARTIAL | Move to canonical runtime execution gate. |
| public payroll explanation | Answer general payroll question | Adapter hardcoded response | PH1.WRITE future | PARTIAL / OWNER_GAP | Move wording to PH1.WRITE. |
| contractor AP payment | Pay contractor invoice/hours | Not found | Finance/AP or Supplier/AP | NOT_FOUND | Build separate contractor payment boundary. |
| performance-to-bonus handoff | Convert performance evidence to reward approval request | HWM future docs | HWM -> Compensation/Payroll | DESIGN_GAP | Build evidence handoff only. |
| payroll reminders | Remind approval/payrun/review | Future docs only | PH1.REM | DESIGN_GAP | Build source-truth linked reminders. |
| payslip delivery | Deliver payslip notification/document | Not found | PH1.BCAST/DELIVERY + Payroll | NOT_FOUND | Build secure delivery boundary. |
| salary private-field access | Restrict salary read/edit | Access future docs | Access/Governance | DESIGN_GAP | Build field-level policy. |
| payroll audit | Record setup/change/payrun/proof | Sim catalog audit declaration only | Payroll + Audit | PARTIAL | Build audit evidence pack. |

## 26. Comparison To Master Architecture

PH1.POSITION Position Journey:

Current repo truth aligns with Position owning job truth and references such as `compensation_band_ref`. Payroll/HR implementation is missing. Future Position must hand off pay/employment setup to Payroll/HR/Compensation, not own it.

PH1.ONB Onboarding Journey:

Current ONB can collect and validate employee setup prerequisites, including compensation tier reference against position band. It does not create payroll profiles, HR employee records, or salary truth.

Master Access Governance + Per-User Access Journey:

Access architecture is the correct future owner for payroll/HR field read/write/approve permissions. Current payroll access is partial through protected-intent gates and DRAFT simulation required-role declarations.

HWM / Task / Roster / Attendance / Staffing docs:

HWM documents define future worked-time evidence, reward evidence, performance review, leave/absence, and payroll handoff boundaries. Current runtime does not yet provide attendance/timesheet/leave evidence.

PH1.REM Reminder Journey:

Future Payroll/HR should use PH1.REM for payroll approval, payrun, probation, contract expiry, leave, final-pay, and offboarding reminders. No current payroll/HR reminder integration was found.

PH1.BCAST / PH1.DELIVERY:

Future Payroll/HR should use BCAST/DELIVERY for payroll/HR notifications and secure payslip/notice delivery. Current BCAST does not own payroll/HR truth.

Finance/Budget future owner:

Finance/Budget must own money truth, budget, cost-to-company, spend/margin impact, and financial approval evidence. Current repo has no payroll/finance integration.

Compensation future owner:

Compensation should own salary package, allowances, commissions, bonuses, benchmarks, and override recommendation logic. Current `PH1.COMP` must not be confused with this owner.

PH1.D Proposal Gateway:

PH1.D may propose payroll/HR explanations, field candidates, summaries, and drafts. It must not approve or execute payroll/HR actions.

PH1.N Meaning Unravelling:

PH1.N can extract payroll/HR intent fields in the future. Current payroll evidence is only generic capability/protected examples.

PH1.X Request Decision Lattice:

PH1.X is central for private read vs public knowledge vs protected payroll/HR execution. Current adapter/lattice evidence supports the boundary but not full runtime execution.

PH1.WRITE Human Presentation:

PH1.WRITE should own sensitive payroll/HR wording. Current adapter hardcoded payroll wording is a wrong-owner risk.

Identity + Access + Authority Spine:

Payroll/HR actions are high-risk protected actions and require identity, access, authority, simulation, confirmation where needed, and audit.

Tenant / Workspace Governance:

Position/ONB prove tenant/company-bound refs. Payroll/HR needs its own tenant/workspace/company scope model.

Desktop/iPhone render-only boundary:

No payroll/HR UI found. Future clients must render and submit bounded inputs only.

Adapter transport-only boundary:

Adapter classification exists; adapter must not become payroll/HR owner.

Old Compatibility Path Retirement:

Adapter heuristics and legacy docs should be retained until active callers and replacement runtime routes are proven.

## 27. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---:|
| missing standalone Payroll engine | no payroll runtime file/table/migration found | No payrun can execute safely | Payroll/HR Repo-Truth Activation Pack | P0 |
| missing standalone HR engine | no HR runtime file/table/migration found | No employment lifecycle owner | Payroll vs HR Owner Split Reconciliation | P0 |
| unclear Payroll vs HR split | partial docs only | Wrong owner can own pay/employment truth | Reconcile owner map before implementation | P0 |
| missing Compensation owner | `PH1.COMP` is compute only | Salary package may be wired into compute engine incorrectly | Create Compensation boundary doc/activation | P0 |
| missing employee profile lifecycle | ONB session only | No employee truth | Build HR Employee Profile / Employment Lifecycle | P0 |
| missing salary/pay-rate storage | no salary/pay-rate fields found | Cannot prepare payroll truth | Build Payroll Profile model | P0 |
| missing payroll group | no payroll group found | Pay frequency/run grouping impossible | Build payroll group/pay calendar | P1 |
| missing payrun | DRAFT catalog only | No committed payroll execution | Implement Payrun Draft/Review/Commit after gates | P0 |
| missing payslip | no payslip module found | No employee pay artifact | Build payslip generation/delivery | P1 |
| missing deduction/tax/contribution engine | no country rule engine | Legal/payroll compliance risk | Source-backed Payroll Compliance boundary | P0 |
| missing benefits | no benefits owner | HR/payroll incompleteness | Benefits model | P1 |
| missing leave balance | no leave model | Final pay and payroll errors | Leave balance + payout flow | P0 |
| missing final pay | no final pay flow | Exit/pay compliance risk | Final Pay / Termination workflow | P0 |
| missing termination/resignation/retirement flow | future docs only | HR/access/payroll offboarding risk | HR lifecycle + Access handoff | P0 |
| missing contractor AP boundary | no AP/invoice owner | Contractor payment misclassified as payroll | Contractor Payment / AP boundary | P1 |
| missing attendance/timesheet handoff | scheduler extraction found missing | Payroll lacks worked-time evidence | Attendance/Timesheet to Payroll handoff | P0 |
| missing payroll approval gate | catalog only | Protected execution cannot activate | Payroll protected execution gate | P0 |
| missing salary override workflow | future docs only | Unauthorized pay changes | Compensation override + Finance approval | P0 |
| missing field-level access | no salary/private field policy | Privacy breach risk | Payroll/HR field-level Access model | P0 |
| missing PH1.WRITE payroll/HR wording | adapter hardcoded text | Wrong-owner sensitive wording | PH1.WRITE payroll/HR templates | P1 |
| missing PH1.D/PH1.N payroll/HR proposal path | generic examples only | Provider may be overused/undergoverned | Proposal shell + validation | P1 |
| missing audit | fail-closed evidence only | No proof of payroll/HR changes | Payroll/HR Audit Evidence Pack | P0 |
| missing SQL persistence | no migrations | No durable truth | Payroll/HR schema migrations after design | P0 |
| missing Desktop/iPhone render-only proof | no payroll UI | Future UI authority drift | Client proof pack | P2 |
| missing Adapter transport-only proof | heuristics exist | Adapter may become owner | Adapter route retirement/transport proof | P1 |
| missing JD live acceptance | no live proof | Product flow unvalidated | JD Live Payroll/HR Acceptance Pack | P0 |

## 28. Recommended Future Build Slices

Based on repo truth, recommended future build slices only:

1. Payroll/HR Repo-Truth Activation Pack
2. Payroll vs HR Owner Split Reconciliation
3. Employee Profile / Employment Lifecycle
4. Payroll Profile / Payroll Group Model
5. Salary / Pay Rate / Compensation Package Boundary
6. Compensation Benchmark / Override Handoff
7. Deductions / Tax / Contribution Boundary
8. Benefits Model
9. Leave Balance / Leave Payout Flow
10. Attendance / Timesheet To Payroll Handoff
11. Payrun Draft / Review / Approval / Commit
12. Payslip Generation / Delivery Boundary
13. Final Pay / Termination / Resignation / Retirement Flow
14. Contractor Payment / AP Boundary
15. Payroll/HR Field-Level Access Model
16. Payroll/HR Protected Execution Gate
17. PH1.D + PH1.N Payroll/HR Proposal Shell
18. PH1.X Payroll/HR Route/Risk Validation
19. PH1.WRITE Payroll/HR Explanation Boundary
20. Payroll/HR Audit Evidence Pack
21. Desktop/iPhone Render-Only Payroll/HR Proof
22. Adapter Transport-Only Payroll/HR Proof
23. JD Live Payroll/HR Acceptance Pack

## 29. What Codex Must Not Do

- Do not invent Payroll/HR behavior.
- Do not create duplicate Payroll/HR engines.
- Do not merge Payroll and HR if repo/future architecture requires split.
- Do not let Position own pay truth.
- Do not let ONB own payroll truth.
- Do not let Scheduler/Roster/Attendance calculate payroll.
- Do not let Access role string expose salary/payroll fields.
- Do not let GPT-5.5/OpenAI calculate final payroll or legal tax truth.
- Do not let Desktop/iPhone decide payroll/HR truth.
- Do not let Adapter decide payroll/HR truth.
- Do not claim tax/labor/payroll compliance without source/owner proof.
- Do not treat contractor payment as employee payroll without proof.
- Do not delete old paths before proof.
- Do not implement from this extraction document alone.

## 30. Final Extracted Architecture Sentence

Selene Payroll + HR is the governed employment and pay-truth boundary where repo truth supports it: HR may own employment lifecycle, employee records, leave classification, probation, resignation, termination, retirement, and HR notices; Payroll may own salary/pay-rate setup, payruns, payslips, deductions, contributions, final pay, and payment handoff; Compensation, Finance/Budget, Scheduler/Roster/Attendance, Position, Onboarding, Access, Reminder, Broadcast/Delivery, PH1.D, PH1.N, PH1.X, and PH1.WRITE must remain separate canonical owners unless repo truth proves otherwise.
