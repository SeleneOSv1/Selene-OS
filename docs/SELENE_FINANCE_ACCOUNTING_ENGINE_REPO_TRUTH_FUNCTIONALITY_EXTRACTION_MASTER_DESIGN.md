# Selene Finance + Accounting Engine — Repo-Truth Functionality Extraction Master Design

```text
DOCUMENT TYPE:
REPO_TRUTH_EXTRACTION / FINANCE + ACCOUNTING FUNCTIONALITY RECOVERY

STATUS:
REPO_TRUTH_EXTRACTION
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only repo-truth extraction. No runtime code was changed. This document does not authorize implementation.

This document reconstructs current Finance / Accounting / Ledger / AP / AR / Banking / Payments / Budget / Profitability / Tax design and functionality from repo evidence. It also marks missing, partial, design-only, wrong-owner, and ambiguous surfaces so future Finance/Accounting work does not accidentally wire a money system from nearby words like "budget", "ledger", "payment", or "AP".

Future implementation, refactor, retirement, or activation requires explicit build instruction, approved file scope, tests, backend evidence, JD approval, and where visible JD live proof.

## 1. Executive Summary

Repo truth does not show a complete standalone Finance runtime engine or complete standalone Accounting runtime engine.

Current repo truth shows these finance-adjacent surfaces:

- `crates/selene_os/src/web_search_plan/realtime/adapters/finance.rs` implements a `RealtimeFinance` public realtime information adapter. It is not a business accounting engine.
- `PH1.COMP` implements deterministic quantitative computation, including a budget/quota posture calculation. It is not a business Finance/Budget engine, does not own budgets, and writes no database rows directly.
- Access/Governance has approval-policy and board-policy infrastructure. Its `AP` naming means Access Approval Policy, not Accounts Payable.
- Adapter protected-intent routing blocks official company accounting execution requests such as "Post this invoice" and "Run the official company P&L from accounting records" with `NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION`.
- Payroll/HR future design documents name Finance/Accounting/AP/Banking as future owners for payment, accounting, contractor, tax/reporting, and ledger handoffs, but they do not prove those owners exist as runtime engines today.
- Storage has several append-only "ledger" tables for memory, artifacts, delivery, access, and evidence. These are not General Ledger / accounting ledger tables.

Current repo truth does not prove:

- a General Ledger,
- Chart of Accounts,
- journal entry engine,
- Accounts Payable engine,
- Accounts Receivable engine,
- invoice/bill/receipt engine,
- payment execution or banking rails,
- bank reconciliation,
- budget/profitability/cost-center engine,
- GST/VAT/sales tax owner,
- payroll accounting handoff,
- contractor AP payment runtime,
- customer/supplier payment runtime,
- finance-specific client surfaces.

The biggest risks are wrong-owner wiring:

- treating `PH1.COMP` budget/quota computation as Finance/Budget truth,
- treating `RealtimeFinance` public data lookup as internal finance truth,
- treating Access `AP` as Accounts Payable,
- treating storage ledgers as accounting General Ledger,
- letting Adapter/Desktop/iPhone protected prompts become payment or accounting authority,
- letting Payroll/HR future handoff docs be mistaken for implemented Finance/Accounting runtime.

## 2. Finance vs Accounting Separation Decision

| Question | Repo Evidence | Answer |
|---|---|---|
| Are Finance and Accounting separate current engines? | No `finance.rs`, `accounting.rs`, `ledger.rs`, `journal.rs`, `ap.rs`, `ar.rs`, `invoice.rs`, `payment.rs`, `banking.rs`, `budget.rs`, `tax.rs`, or equivalent runtime engine files were found under `crates/selene_kernel_contracts/src`, `crates/selene_engines/src`, or business runtime paths. `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` has no Finance/Accounting engine row. | NO / NOT_FOUND |
| Should Finance and Accounting be separate future owners? | Payroll/HR automation and Evidence Fabric docs repeatedly split employee pay calculation, accounting/payment handoff, AP, Finance/Budget, and bank provider execution. Master Access says Access answers "is this person allowed", while Finance/Budget answers "is the money/budget available". | YES, but NEEDS_RECONCILIATION before implementation |
| Is AP separate from Payroll? | Current Accounts Payable runtime is NOT_FOUND. Future Payroll/HR and Leave/Benefits/Contractor/AP docs state contractor/vendor payment should route through AP/Accounting unless explicitly governed. | Current: NOT_FOUND. Future: YES / DESIGN_ONLY |
| Is AR separate from Invoicing? | No AR or invoice engine found. Adapter only blocks protected official invoice posting and can provide public advisory P&L templates. | UNKNOWN / NOT_FOUND |
| Is Banking/Payment execution separate from Accounting? | No payment execution engine found. Future Evidence Fabric says Payroll creates employee pay instruction, AP creates contractor/vendor pay instruction, Accounting/Finance/Bank provider executes payment rail through authority, simulation, audit, and provider proof. | Current: NOT_FOUND. Future: YES / DESIGN_ONLY |
| Is Budget/Profitability separate from Ledger? | No business budget/profitability/ledger engine found. `PH1.COMP` computes budget/quota posture for `PH1.COST`, `PH1.QUOTA`, and `PH1.LAW`; it does not own business budgets. | Current: NOT_FOUND with PH1.COMP PARTIAL computation only. Future: YES / NEEDS_RECONCILIATION |
| Is Tax/GST/VAT separate from Invoice/Payroll? | No tax/GST/VAT engine or rule pack found. Payroll/HR docs require source-backed or owner-approved tax/contribution rules. | Current: NOT_FOUND. Future: YES / DESIGN_ONLY |

Default future owner law unless repo truth proves otherwise:

- Accounting owns books, ledger, journals, chart of accounts, reconciliation, and financial reports.
- Finance/Budget owns budgets, approvals, cashflow, forecasting, profitability, spend authority, and financial governance.
- Accounts Payable owns supplier/contractor/vendor bills, invoices received, and payment approval workflow.
- Accounts Receivable owns customer invoices, receipts, collections, and credit notes.
- Banking/Payment Provider owns payment rails and bank confirmations.
- Payroll owns employee pay calculation and employee pay instruction.
- Contractor/vendor payments should usually route through AP, not Payroll, unless explicitly governed.
- Tax/Compliance owns source-backed tax/GST/VAT rules and reporting obligations.
- Access/Governance owns who may view/change/approve financial records and payment execution.
- PH1.D/GPT-5.5 may explain, classify, and draft, but must not execute payments or invent accounting/tax truth.
- PH1.WRITE owns final user-facing explanation.

## 3. Repo Evidence Inventory

| Evidence Area | File / Path | Symbols / Routes / Tables | Status | Notes |
|---|---|---|---|---|
| Required governance | `AGENTS.md` | docs-only, no Python, no runtime mutation law | ACTIVE | Controls this extraction. |
| Core client/runtime boundary | `docs/CORE_ARCHITECTURE.md` | runtime owns authority/state; clients do not decide | ACTIVE | Future finance/payment execution must stay cloud/runtime governed. |
| Build ordering | `docs/SELENE_BUILD_EXECUTION_ORDER.md` | authority before business features | ACTIVE | Finance/payment must not be built before authority and simulation gates. |
| Engine inventory | `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md` | no Finance/Accounting/AP/AR/Payment engine row found | NOT_FOUND | Absence is material. |
| Missing engine files | `crates/selene_kernel_contracts/src`, `crates/selene_engines/src`, `crates/selene_os/src` | no `finance.rs`, `accounting.rs`, `ledger.rs`, `journal.rs`, `ap.rs`, `ar.rs`, `invoice.rs`, `payment.rs`, `banking.rs`, `budget.rs`, `tax.rs` business engines | NOT_FOUND | Required search target files are absent. |
| Public realtime finance adapter | `crates/selene_os/src/web_search_plan/realtime/adapters/finance.rs` | `ADAPTER_ID: "RealtimeFinance"`, `execute`, `Finance API`, `SELENE_REALTIME_FINANCE_API_KEY` | PARTIAL / NOT_ACCOUNTING | Public finance lookup adapter, not company books/payment owner. |
| Realtime config | `crates/selene_os/src/web_search_plan/realtime/mod.rs` | `finance_endpoint`, `finance_api_key_override`, `finance_vault_secret_id_override` | PARTIAL / PUBLIC_TOOL | Provider config only. |
| Realtime tests | `crates/selene_os/src/web_search_plan/realtime/realtime_tests.rs` | `test_t7_missing_retrieved_at_fails_without_inference` uses `RealtimeFinance` for timestamp validation | PARTIAL | Proves realtime evidence freshness validation, not accounting. |
| PH1.COMP DB wiring | `docs/DB_WIRING/PH1_COMP.md` | "ranking, consensus, normalization, and budget math"; "no authoritative PH1.COMP-specific database tables" | ACTIVE / NOT_FINANCE_ENGINE | Deterministic math helper only. |
| PH1.COMP ECM | `docs/ECM/PH1_COMP.md` | `COMP_COMPUTE_BUDGET_POSTURE`, side effects `NONE` | ACTIVE / NOT_BUDGET_ENGINE | Budget/quota posture only. |
| PH1.COMP runtime | `crates/selene_os/src/ph1comp.rs` | `FORMULA_BUDGET_V1`, `BudgetQuotaComputationRequest`, `compute_budget_quota_packet` | ACTIVE / PARTIAL | Computes budget/quota ratios and threshold reason codes, not budgets. |
| PH1.COMP engine math | `crates/selene_engines/src/ph1comp.rs` | `BudgetPosture`, `budget_posture` | ACTIVE / PARTIAL | Deterministic utility math. |
| PH1.COMP tests | `crates/selene_os/src/ph1comp.rs` | `at_comp_06_budget_quota_calculation_is_deterministic`, `at_comp_07_failure_classes_surface_correctly` | ACTIVE / PARTIAL | Proves deterministic computation, not Finance/Accounting lifecycle. |
| Access approval policy | `crates/selene_kernel_contracts/src/ph1access.rs` | `ACCESS_AP_SCHEMA_*`, `AccessApprovalPrimitive`, `BoardQuorumPercent`, `require_cfo_approval` | ACTIVE / AP_AMBIGUITY | AP = Access Approval Policy, not Accounts Payable. |
| Access AP storage | `crates/selene_storage/migrations/0016_access_ap_authoring_review_tables.sql` | `access_ap_authoring_review_ledger`, `access_ap_rule_review_actions_ledger` | ACTIVE / AP_AMBIGUITY | Approval-policy authoring storage. |
| Access board policy | `docs/DB_WIRING/PH1_ACCESS_001_PH2_ACCESS_002.md` | `access_board_policy_ledger`, `access_board_votes_ledger` | ACTIVE / NOT_FINANCE | Can support approvals, not accounting truth. |
| Simulation catalog | `docs/08_SIMULATION_CATALOG.md` | Access AP schema simulations, no Finance/Accounting payment simulations found | PARTIAL / TEST_GAP | Simulation support exists for Access, not Finance/Accounting. |
| Adapter protected boundary | `crates/selene_adapter/src/lib.rs` | `h406_official_company_execution_request`, `h406_protected_business_execution_stays_fail_closed` | PARTIAL | Blocks official P&L/accounting execution and invoice posting without protected simulation/authority. |
| Adapter public advisory P&L | `crates/selene_adapter/src/lib.rs` | `h406_public_advisory_fallback_answer` returns "Simple Profit and Loss Template" and "not official company accounting execution" | PARTIAL / ADVISORY_ONLY | Public template, not company accounting runtime. |
| Desktop/iPhone client search | `apple/mac_desktop`, `apple/iphone` | no dedicated Finance/Accounting UI found in targeted search | NOT_FOUND | Existing ledger mentions are conversation/memory UI only. |
| Storage repository | `crates/selene_storage/src/repo.rs`, `crates/selene_storage/src/ph1f.rs` | artifact/memory/access/work-order ledgers; no finance/accounting tables found | NOT_FOUND / LEDGER_NAME_RISK | "Ledger" names are not General Ledger. |
| Payroll/HR extraction | `docs/SELENE_PAYROLL_HR_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md` | Finance/Budget marked future owner; payroll runtime incomplete | DESIGN_ONLY | Important dependency but not Finance runtime proof. |
| Payroll/HR Evidence Fabric | `docs/SELENE_PAYROLL_HR_EVIDENCE_FABRIC_CODEX_READINESS_CONVERSION_LAYER.md` | `AccountingHandoffPacket`, `PaymentInstructionPacket`, `PaymentConfirmationPacket` future logical packets | DESIGN_ONLY | Future bridge only. |
| Leave/Benefits/Contractor/AP design | `docs/SELENE_LEAVE_BENEFITS_FINAL_PAY_CONTRACTOR_AP_PAYMENT_AUTOMATION_MASTER_DESIGN.md` | contractor/AP and payment automation future design | DESIGN_ONLY | Not runtime proof. |
| Master Access Governance | `docs/SELENE_MASTER_ACCESS_GOVERNANCE_PER_USER_ACCESS_JOURNEY_MASTER_DESIGN.md` | spend authority, finance/private data, supplier bank details, budget approval | DESIGN_ONLY / PARTIAL_ACCESS | Access future model, not Finance owner. |

## 4. Current Owner Map

| Responsibility | Current Owner / File | Correct Future Owner Hypothesis | Status | Notes |
|---|---|---|---|---|
| chart of accounts | NOT_FOUND | Accounting | ACCOUNTING_GAP | No CoA contract/table/runtime found. |
| general ledger | NOT_FOUND | Accounting | ACCOUNTING_GAP | Storage ledgers are not GL. |
| journal entries | NOT_FOUND | Accounting | ACCOUNTING_GAP | No journal packet or posting runtime found. |
| accounting period | NOT_FOUND | Accounting | ACCOUNTING_GAP | No fiscal period/close owner found. |
| fiscal year | NOT_FOUND | Accounting / Finance | ACCOUNTING_GAP | No fiscal year storage found. |
| bank accounts | Payroll onboarding has future bank setup; no Finance bank records | Banking / Accounting / Payroll by context | BANKING_GAP | Employee bank details are Payroll; company/supplier bank details missing. |
| bank feeds | NOT_FOUND | Banking Provider / Accounting | BANKING_GAP | No feed import/reconciliation. |
| bank reconciliation | NOT_FOUND | Accounting | ACCOUNTING_GAP | No reconciliation engine. |
| payment instruction | Future logical packets in Payroll/HR Evidence Fabric | Payroll/AP creates, Accounting validates | DESIGN_ONLY | No runtime contract. |
| payment execution | NOT_FOUND | Bank/Payment Provider | PAYMENT_GAP | No bank transfer rail. |
| payment confirmation | Future logical packet only | Bank/Payment Provider / Accounting | DESIGN_ONLY | No provider proof runtime. |
| payment failure | Adapter/protected fail-closed exists for protected business requests | Bank/Payment Provider / Accounting | PARTIAL / DESIGN_GAP | No payment failure lifecycle. |
| AP supplier bill | NOT_FOUND | Accounts Payable | AP_GAP | Access AP is not Accounts Payable. |
| AP contractor invoice | Future design only | Accounts Payable | DESIGN_ONLY | Contractor docs say AP should own payment. |
| AP approval | Access approval-policy machinery exists; no AP bill approval runtime | AP + Access/Governance | PARTIAL | Approval primitive can be reused later, but no bill flow. |
| AP payment | NOT_FOUND | AP + Accounting + Banking | AP_GAP | No supplier payment runtime. |
| AR customer invoice | NOT_FOUND | Accounts Receivable / Invoice owner | AR_GAP | Adapter blocks "post this invoice"; no invoice engine. |
| AR receipt | NOT_FOUND | Accounts Receivable | AR_GAP | No receipt/allocation runtime. |
| AR payment allocation | NOT_FOUND | Accounts Receivable / Accounting | AR_GAP | Missing. |
| refunds | Adapter protects "send the customer refund" | AR / Finance / Banking | PARTIAL / PAYMENT_GAP | Protected phrase detection only. |
| credit notes | NOT_FOUND | Accounts Receivable | AR_GAP | Missing. |
| debit notes | NOT_FOUND | Accounts Receivable / AP | AR_GAP | Missing. |
| purchase orders | NOT_FOUND | Procurement / AP / Finance | FINANCE_GAP | Missing. |
| reimbursements | Access future docs mention reimbursement approvals | Finance/AP/Payroll by policy | DESIGN_ONLY | No runtime. |
| expenses | NOT_FOUND | Finance/AP | FINANCE_GAP | Missing. |
| budgets | `PH1.COMP` budget/quota math; Access future docs say Finance/Budget owns actual budgets | Finance/Budget | PARTIAL / FINANCE_GAP | Computation exists, business budget owner missing. |
| budget limits | Access future spend authority and PH1.COMP threshold math | Finance/Budget + Access | PARTIAL | No budget records. |
| cost centers | Payroll/HR future docs mention cost center; no runtime found | Finance/Budget / Accounting | NOT_FOUND | Missing. |
| department budgets | Access future design examples | Finance/Budget | DESIGN_ONLY | Missing runtime. |
| profitability | Adapter public P&L template only | Finance/Budget / Accounting reports | DESIGN_ONLY / NOT_FOUND | No company records. |
| margin | Access docs mention gross margin private read; no runtime | Finance/Budget | DESIGN_ONLY | Missing. |
| forecasts | NOT_FOUND | Finance/Budget | FINANCE_GAP | Missing. |
| payroll journal | Future Payroll/HR docs mention payroll journal | Accounting consumes Payroll handoff | DESIGN_ONLY | No journal runtime. |
| payroll liabilities | Future Payroll/HR docs mention tax/super/pension remittance | Accounting / Payroll / Tax | DESIGN_ONLY | Missing runtime. |
| tax/GST/VAT | Future docs require source-backed tax rules | Tax/Compliance | TAX_GAP | No tax engine. |
| tax reporting | Future docs only | Tax/Compliance / Accounting | TAX_GAP | Missing runtime. |
| financial reports | Adapter advisory template; official P&L blocked | Accounting / Finance | PARTIAL / NOT_FOUND | Public template only. |
| approval matrix | Access has approval primitives, board policies/votes | Access/Governance | PARTIAL | Not finance-specific execution yet. |
| spend authority | Master Access future design | Access + Finance/Budget | DESIGN_ONLY | No Finance runtime. |
| board/CFO/CEO/chairman approvals | Access approval primitive supports board quorum and CFO field; CEO/chairman only design text | Access/Governance | PARTIAL | Approval substrate, not spend execution. |
| Desktop rendering | No finance UI found | Desktop render-only | NOT_FOUND | No finance authority. |
| iPhone rendering | No finance UI found | iPhone render-only | NOT_FOUND | No finance authority. |
| Adapter transport | Protected detection/advisory fallback only | Adapter transport-only | PARTIAL | Must not execute. |
| audit/provenance | Generic audit/proof ledgers; no finance-specific audit | Audit + Finance/Accounting owners | PARTIAL / AUDIT_GAP | Missing journal/payment audit. |
| storage/migrations | No finance/accounting migration found | Finance/Accounting storage | NOT_FOUND | `0016_access_ap...` is Access AP. |
| old compatibility paths | Adapter protected phrases, public P&L template, Access AP, artifact ledgers | Correct future owners | RISK | Needs retirement/active-caller checks later. |

## 5. Current Accounting Lifecycle

Repo truth does not prove a current Accounting lifecycle.

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Change | Audit Evidence | Status |
|---|---|---|---|---|---|---|---|
| chart of accounts setup | NOT_FOUND | no CoA files/tables | UNKNOWN | UNKNOWN | none found | none found | ACCOUNTING_GAP |
| accounting period setup | NOT_FOUND | no accounting period files/tables | UNKNOWN | UNKNOWN | none found | none found | ACCOUNTING_GAP |
| journal entry draft | NOT_FOUND | no journal engine | UNKNOWN | UNKNOWN | none found | none found | ACCOUNTING_GAP |
| journal entry approval | Access may later approve protected actions | `ph1access.rs`, Access docs | future approval request | approval decision | Access AP/board policy only | Access audit possible | PARTIAL / NOT_ACCOUNTING |
| journal entry posting | NOT_FOUND | no GL/journal posting runtime | UNKNOWN | UNKNOWN | none found | none found | ACCOUNTING_GAP |
| ledger update | NOT_FOUND | storage ledgers are not GL | UNKNOWN | UNKNOWN | none found | none found | LEDGER_NAME_RISK |
| reconciliation | NOT_FOUND | no reconcile engine | bank/payment data | reconciliation result | none found | none found | ACCOUNTING_GAP |
| period close | NOT_FOUND | no period lock/close | accounting period | closed period | none found | none found | ACCOUNTING_GAP |
| financial report generation | Adapter advisory P&L only | `h406_public_advisory_fallback_answer` | public/advisory prompt | template text | no company state | no accounting audit | PARTIAL / ADVISORY_ONLY |
| correction/reversal | NOT_FOUND | no accounting reversal state | UNKNOWN | UNKNOWN | none found | none found | ACCOUNTING_GAP |
| audit/proof | generic proof ledgers | storage PH1.F/audit surfaces | non-accounting events | generic evidence | non-accounting ledgers | generic only | AUDIT_GAP |

Accounting lifecycle conclusion:

Current repo has protected refusal and public advisory template behavior for accounting-like prompts, but it does not have books, journals, posting, period close, reconciliation, or financial report truth. Any future accounting lifecycle must be built as a new governed owner or explicitly reconciled with a future Accounting engine.

## 6. Current Finance / Budget / Profitability Lifecycle

Repo truth shows deterministic budget/quota computation, not business budget lifecycle.

| Stage | Owner | Symbols / Files | Inputs | Outputs | State Change | Audit Evidence | Status |
|---|---|---|---|---|---|---|---|
| budget creation | NOT_FOUND | no budget table/engine | budget proposal | budget record | none found | none found | FINANCE_GAP |
| budget line setup | NOT_FOUND | no budget line model | line data | line record | none found | none found | FINANCE_GAP |
| budget approval | Access future design only | `AccessApprovalPrimitive`, board policy docs | approval case | approval result | access policy records only | Access audit | PARTIAL / DESIGN_GAP |
| spend limit setup | Access/Governance future | Master Access doc | role/amount/category policy | access/spend authority policy | access policy only | Access AP/board ledgers | PARTIAL |
| department/cost-center budget | NOT_FOUND | no cost center model | UNKNOWN | UNKNOWN | none found | none found | FINANCE_GAP |
| forecast | NOT_FOUND | no forecast engine | UNKNOWN | UNKNOWN | none found | none found | FINANCE_GAP |
| profitability check | Adapter advisory only | `h406_public_advisory_fallback_answer` | user-provided/advisory prompt | template | none | none | DESIGN_ONLY |
| margin buffer | NOT_FOUND | Access mentions gross margin as private read example | private financial data | answer | none found | none found | FINANCE_GAP |
| cashflow forecast | NOT_FOUND | no cashflow engine | UNKNOWN | UNKNOWN | none found | none found | FINANCE_GAP |
| over-budget detection | PH1.COMP budget threshold math | `compute_budget_quota_packet`, `budget_threshold_crossed` | caller-supplied numeric limits/usage | `ComputationPacket` | none | computation state only | PARTIAL / NOT_BUDGET_TRUTH |
| approval escalation | Access future/partial | `access_board_policy_ledger`, `access_board_votes_ledger` | approval policy | approval records | Access ledgers | Access audit | PARTIAL |
| board/CFO/CEO/chairman review | Access has board quorum and CFO field; CEO/chairman only design text | `AccessApprovalPrimitive::BoardQuorumPercent`, `require_cfo_approval` | access approval policy | approval policy result | Access only | Access audit | PARTIAL |

Finance lifecycle conclusion:

`PH1.COMP` can compute deterministic budget/quota posture from supplied numbers. It does not create budgets, own financial records, enforce spend limits by itself, or approve spending. Access has approval machinery and examples for spend authority, but Finance/Budget truth is a future gap.

## 7. Current Accounts Payable Lifecycle

Repo truth does not prove an Accounts Payable runtime.

| Stage | Owner | Symbols / Files | Inputs | Outputs | Status | Notes |
|---|---|---|---|---|---|---|
| supplier/vendor setup | NOT_FOUND | no supplier/vendor model found | supplier data | supplier record | AP_GAP | Missing. |
| supplier invoice received | NOT_FOUND | no supplier bill/invoice intake | invoice data | bill draft | AP_GAP | Missing. |
| bill created | NOT_FOUND | no bill engine | bill fields | bill record | AP_GAP | Missing. |
| invoice/bill validation | NOT_FOUND | no validation model | invoice/bill fields | validation status | AP_GAP | Missing. |
| purchase order matching | NOT_FOUND | no PO model | PO + bill | match result | AP_GAP | Missing. |
| goods/services receipt matching | NOT_FOUND | no receipt/matching owner | receipt evidence | match result | AP_GAP | Missing. |
| contractor hours/milestone evidence matching | DESIGN_ONLY | Payroll/HR and HWM future docs | contractor evidence | AP evidence bundle | DESIGN_ONLY | No runtime. |
| approval routing | PARTIAL | Access approval policy/board votes | approval case | approval decision | PARTIAL | Generic Access only, not AP-specific. |
| payment instruction | DESIGN_ONLY | Payroll/HR Evidence Fabric future `PaymentInstructionPacket` | approved bill | payment instruction | DESIGN_ONLY | No runtime. |
| payment execution handoff | NOT_FOUND | no bank/payment provider | instruction | transfer | PAYMENT_GAP | Missing. |
| payment confirmation | NOT_FOUND | no confirmation lifecycle | provider response | confirmation | PAYMENT_GAP | Missing. |
| reconciliation | NOT_FOUND | no bank reconciliation | payment/bank data | reconciliation | ACCOUNTING_GAP | Missing. |
| audit/proof | PARTIAL | generic audit/evidence only | event refs | generic proof | AUDIT_GAP | No AP audit model. |

Contractor payment belongs to AP/Accounting in future design unless a future policy explicitly routes a specific contractor class through Payroll. Current repo truth does not prove either runtime path.

## 8. Current Accounts Receivable Lifecycle

Repo truth does not prove an Accounts Receivable runtime.

| Stage | Owner | Symbols / Files | Inputs | Outputs | Status | Notes |
|---|---|---|---|---|---|---|
| customer setup | NOT_FOUND | no customer/account model found | customer data | customer record | AR_GAP | Missing. |
| customer invoice draft | NOT_FOUND | adapter only detects "Post this invoice" as protected | invoice fields | draft invoice | AR_GAP | No invoice runtime. |
| invoice approval/sending | NOT_FOUND | no invoice send simulation or BCAST route found | invoice | sent invoice | AR_GAP | Missing. |
| invoice delivery | NOT_FOUND | BCAST/DELIVERY generic only | delivery request | receipt | DESIGN_GAP | Future invoice sending must use BCAST/DELIVERY. |
| receipt/payment received | NOT_FOUND | no receipt engine | payment info | receipt | AR_GAP | Missing. |
| payment allocation | NOT_FOUND | no AR allocation logic | receipt + invoices | allocation | AR_GAP | Missing. |
| overdue invoice | NOT_FOUND | no overdue model | invoice status | overdue case | AR_GAP | Missing. |
| reminder/collection | DESIGN_ONLY | PH1.REM/BCAST generic capability | due date/reminder | message | DESIGN_GAP | No AR owner. |
| credit note/refund | PARTIAL phrase detection | adapter detects refund protected intent | refund request | blocked protected response | PARTIAL | No refund runtime. |
| reconciliation | NOT_FOUND | no bank reconciliation | receipt/bank feed | match | ACCOUNTING_GAP | Missing. |
| audit/proof | PARTIAL generic only | PH1.F/audit surfaces | event refs | generic proof | AUDIT_GAP | No AR audit. |

BCAST/DELIVERY is not currently wired to invoices as a source owner. Future AR/invoice sending must produce a delivery request, not send directly.

## 9. Current Banking / Payment Provider Lifecycle

Current repo truth does not show business banking or payment execution.

| Area | Evidence | Status | Notes |
|---|---|---|---|
| bank account | Payroll onboarding/future docs mention employee bank details; no company/supplier bank account model found | DESIGN_ONLY / BANKING_GAP | Employee pay account is Payroll-sensitive, not Accounting bank rail. |
| payment instruction | Future logical packets in Payroll/HR Evidence Fabric | DESIGN_ONLY | Not implemented. |
| bank/payment provider | No bank provider or payment rail module found | PAYMENT_GAP | `RealtimeFinance` is public information lookup, not payment provider. |
| payment approval | Access approval machinery exists | PARTIAL | Generic authority only. |
| payment send | NOT_FOUND | PAYMENT_GAP | No transfer execution. |
| payment confirmation | Future logical `PaymentConfirmationPacket` only | DESIGN_ONLY | No provider callback. |
| payment failure | NOT_FOUND | PAYMENT_GAP | Missing failure states. |
| partial failure | NOT_FOUND | PAYMENT_GAP | Missing. |
| payment reconciliation | NOT_FOUND | ACCOUNTING_GAP | Missing. |
| provider proof | public realtime evidence proof exists for tools; no bank provider proof | PARTIAL / PAYMENT_GAP | Need future fake/real bank provider proof. |
| bank account validation | onboarding simulation/design only; no live bank validation provider | DESIGN_ONLY | Do not claim live validation. |
| payment file export | NOT_FOUND | PAYMENT_GAP | Missing. |
| direct bank transfer | NOT_FOUND | PAYMENT_GAP | Missing. |

Correct future rule:

Payment execution is high-risk and must require Access/Authority/Simulation/Audit plus provider proof. Accounting/Finance/Bank provider executes the payment rail. Payroll creates employee pay instruction. AP creates contractor/vendor pay instruction. No current repo code proves this execution path.

## 10. Tax / GST / VAT / Compliance

Repo truth does not show a tax/GST/VAT runtime owner.

| Area | Evidence | Status | Notes |
|---|---|---|---|
| tax codes | NOT_FOUND | TAX_GAP | No tax code model. |
| tax rates | NOT_FOUND | TAX_GAP | No tax rate model. |
| GST/VAT/sales tax | NOT_FOUND | TAX_GAP | No country tax engine. |
| invoice tax | NOT_FOUND | TAX_GAP | No invoice engine. |
| payroll tax/remittance | Future Payroll/HR docs mention tax/super/pension/CPF | DESIGN_ONLY | No runtime. |
| contractor tax | Future contractor/AP docs mention reporting handoff | DESIGN_ONLY | No runtime. |
| withholding | NOT_FOUND | TAX_GAP | Missing. |
| tax authority reporting | Future docs only | DESIGN_ONLY | Missing runtime. |
| country/region rules | Payroll/Compensation docs require source-backed rules | DESIGN_ONLY | No active rule pack. |
| effective dates | NOT_FOUND | TAX_GAP | Missing. |
| rule versioning | NOT_FOUND | TAX_GAP | Missing. |
| manual override | Access can govern approval; no tax override model | DESIGN_GAP | Missing. |
| audit | generic only | AUDIT_GAP | No tax audit. |

Correct future rule:

GPT-5.5 may explain tax concepts but must not invent tax truth. Tax rules must be source-backed or owner-approved, versioned, jurisdiction-scoped, and audited.

## 11. Payroll / HR / Compensation Interaction

Current repo truth shows Payroll/HR runtime is incomplete and Finance/Accounting runtime is missing. Interactions are therefore design-only.

| Interaction | Evidence | Current Status | Correct Boundary |
|---|---|---|---|
| payroll journal | Payroll/HR automation docs mention payroll journal under Finance/Accounting | DESIGN_ONLY | Payroll calculates employee pay; Accounting posts books. |
| payroll liabilities | Future docs mention tax/super/pension/CPF remittance | DESIGN_ONLY | Payroll/Tax define liability evidence; Accounting records liability; Banking remits. |
| employee pay payment instruction | Evidence Fabric future `PaymentInstructionPacket` | DESIGN_ONLY | Payroll creates employee pay instruction; bank provider executes. |
| salary advance recovery | Payroll/HR future docs mention recovery and deductions | DESIGN_ONLY | Payroll computes; Accounting records; Finance controls money impact. |
| super/pension/CPF remittance | Future docs only | DESIGN_ONLY | Tax/Compliance + Accounting + payment provider. |
| tax remittance | Future docs only | DESIGN_ONLY | Tax/Compliance + Accounting + payment provider. |
| final pay accounting | Leave/Final Pay design says Finance/Accounting processes payment | DESIGN_ONLY | Payroll calculates final pay; Accounting records and Finance/Banking pays. |
| bonus/commission accounting | Compensation future docs; Sales owns sales truth | DESIGN_ONLY | Compensation formula, Payroll pay, Accounting entries, Finance exposure. |
| reimbursement | Access docs mention reimbursement authority; no runtime | DESIGN_ONLY | Finance/AP or Payroll depending policy, needs reconciliation. |
| payroll budget impact | Position/Access docs mention Finance/Budget impact | DESIGN_ONLY | Finance/Budget validates impact, not Payroll. |
| payroll approval threshold | Access approval primitives can model approval | PARTIAL / DESIGN_GAP | Access checks authority; Payroll/Finance provide source truth. |

Critical rule:

Payroll calculates employee pay. Accounting posts books and payment/ledger entries. Finance/Budget validates money impact and approvals. Banking/Payment provider executes transfer. Current repo does not implement this full chain.

## 12. Contractor / Supplier / AP Interaction

Current repo truth does not prove a contractor AP runtime.

| Interaction | Evidence | Current Status | Correct Boundary |
|---|---|---|---|
| contractor onboarding/payment refs | Position and Payroll/HR future docs | DESIGN_ONLY | Contractor identity/work setup separate from AP payment. |
| contractor hourly/lump-sum/milestone payment | Leave/Contractor/AP future design | DESIGN_ONLY | Contractor evidence from HWM/Task; AP owns bill/payment flow. |
| contractor invoice | NOT_FOUND in runtime | AP_GAP | AP should own supplier/contractor invoice. |
| supplier invoice | NOT_FOUND | AP_GAP | Missing. |
| AP approval | Access approval substrate partial | PARTIAL | Access approves authority; AP owns bill state. |
| contractor overrun approval | HWM future docs mention overrun signal; no runtime | DESIGN_ONLY | HWM detects; Finance/AP approves exposure. |
| contractor access expiry/payment hold | Access/HWM future docs | DESIGN_ONLY | Access expires permissions; AP handles payment hold only by lawful policy. |
| AP payment instruction | Future `PaymentInstructionPacket` | DESIGN_ONLY | AP creates vendor payment instruction; Accounting/Banking execute. |

Critical rule:

Contractor/vendor payment should not default to employee Payroll unless repo truth proves that path. Repo truth does not prove that path.

## 13. Customer / Sales / AR Interaction

Current repo truth does not show a Sales, AR, customer invoice, or receipt engine.

| Interaction | Evidence | Current Status | Notes |
|---|---|---|---|
| customer invoice | Adapter protects "Post this invoice"; no engine | NOT_FOUND | AR_GAP |
| customer payment | NOT_FOUND | AR_GAP | Missing. |
| sales order | NOT_FOUND | DESIGN_GAP | Missing. |
| sales commission trigger | Compensation docs say Sales should own sales truth | DESIGN_ONLY | Sales engine absent. |
| refund | Adapter protected phrase detects customer refund | PARTIAL | No refund runtime. |
| credit note | NOT_FOUND | AR_GAP | Missing. |
| AR receipt | NOT_FOUND | AR_GAP | Missing. |
| overdue reminders | PH1.REM/BCAST generic only | DESIGN_GAP | Missing AR owner. |
| customer statement | NOT_FOUND | AR_GAP | Missing. |
| sales tax/GST/VAT | NOT_FOUND | TAX_GAP | Missing. |

If Sales is later built, Sales should own sales event truth; AR should own invoice/receipt/collections; Accounting should own books; Finance/Budget should own exposure and profitability.

## 14. Interaction With Master Access / Authority

Repo truth shows Access/Governance has the closest existing authority substrate for future finance actions, but it is not Finance/Accounting.

Must-answer summary:

| Question | Repo Truth Answer |
|---|---|
| Who can view finance reports? | Current runtime-specific rule NOT_FOUND. Master Access future design requires field/private data permissions for finance records. |
| Who can view bank accounts? | Current runtime-specific rule NOT_FOUND. Master Access examples mention supplier bank details and private payroll/bank fields as field-level access concerns. |
| Who can approve supplier payments? | NOT_FOUND. Access approval primitives can represent approval policy but no AP payment flow exists. |
| Who can approve contractor invoices? | NOT_FOUND. Future contractor/AP docs require approval. |
| Who can approve refunds? | NOT_FOUND. Adapter marks refund protected; no refund approval engine. |
| Who can approve budget changes? | NOT_FOUND. Access future design includes budget/spend authority; Finance/Budget truth missing. |
| Who can approve over-budget spend? | NOT_FOUND. Access future docs mention CEO/chairman/board escalation; no Finance runtime. |
| Who can approve payroll payment instructions? | NOT_FOUND. Payroll/Finance chain design-only. |
| Who can export financial data? | NOT_FOUND. PH1.EXPORT exists for audit/work/conversation exports, not finance reports. |
| Does repo gate financial reads/writes? | PARTIAL. Adapter fail-closes protected official company accounting execution and private/protected payroll lanes. No field-level finance data runtime. |
| Does protected execution require authority/simulation? | YES as global law and adapter protected refusal behavior; no Finance simulation exists. |
| What is missing? | Finance-specific access matrix, payment authority, field-level financial reads, export rules, dual approval, provider proof, audit. |

Critical rule:

Finance/accounting/payment data is private/high-risk. Access/Governance controls field-level read/write/approve. Authority + Simulation controls protected finance execution. Access does not own money truth.

## 15. Interaction With PH1.TASK / HWM / Roster / Payroll Evidence

Current repo truth has future design references, not implemented money handoffs.

| Interaction | Evidence | Status | Notes |
|---|---|---|---|
| staffing cost forecast | Roster/Workload docs mention Finance/Budget boundary | DESIGN_ONLY | No finance runtime. |
| labour cost evidence | Payroll/HR and Roster docs mention payroll handoff | DESIGN_ONLY | No accounting handoff. |
| task/project cost evidence | HWM future docs mention contractor/project evidence | DESIGN_ONLY | No cost engine. |
| contractor work evidence | HWM and Leave/Contractor/AP future docs | DESIGN_ONLY | Should feed AP later. |
| reimbursement tasks | Access docs mention reimbursement approvals | DESIGN_ONLY | Missing owner. |
| approval tasks | Access approval substrate | PARTIAL | Not finance-specific. |
| payment reminders | PH1.REM generic reminders only | PARTIAL / DESIGN_GAP | No payment owner. |
| budget overrun alerts | PH1.COMP threshold math and Access future docs | PARTIAL / DESIGN_GAP | No budget records. |
| resource forecast | HWM future docs | DESIGN_ONLY | Missing finance consumption. |
| payroll evidence bundle accounting handoff | Evidence Fabric future packet | DESIGN_ONLY | No runtime. |

These must remain handoffs. Scheduler/Roster/Attendance/Task must not calculate or post money truth.

## 16. Interaction With PH1.REM / PH1.BCAST / PH1.DELIVERY

Current repo truth shows reminder/delivery engines exist, but finance/accounting-specific delivery flows are missing.

| Delivery Need | Current Evidence | Status | Correct Boundary |
|---|---|---|---|
| invoice sending | NOT_FOUND | AR_GAP | AR/Invoice owner creates send request; BCAST/DELIVERY sends. |
| invoice reminders | Generic PH1.REM/BCAST capability | DESIGN_GAP | REM owns timing; AR owns truth. |
| overdue reminders | NOT_FOUND finance-specific | AR_GAP | AR + REM + BCAST/DELIVERY. |
| payment approval reminders | Access approval + PH1.REM generic | DESIGN_GAP | Access/Finance/AP source; REM timing. |
| supplier payment notifications | NOT_FOUND | AP_GAP | AP/Accounting source; BCAST/DELIVERY transport. |
| contractor payment notifications | Future contractor/AP docs | DESIGN_ONLY | AP source; BCAST/DELIVERY transport. |
| payroll payment confirmation | Payroll/HR future docs | DESIGN_ONLY | Payroll/Accounting source; BCAST/DELIVERY optional. |
| customer receipt notifications | NOT_FOUND | AR_GAP | AR source; BCAST/DELIVERY transport. |
| budget approval reminders | Access future docs | DESIGN_ONLY | Access/Finance source; REM timing. |
| finance report delivery | NOT_FOUND | FINANCE_GAP | Finance/Accounting source; BCAST/DELIVERY transport. |

Critical rule:

PH1.REM owns timing. PH1.BCAST/DELIVERY owns notification delivery. Finance/Accounting/AP/AR owns source truth.

## 17. PH1.D / GPT-5.5 / PH1.N / PH1.X Interaction

Current repo truth shows protected finance/accounting-like prompts are handled as protected risk, and public advisory templates are allowed only as non-official guidance.

OpenAI/GPT-5.5 may help in future:

- understand messy finance/accounting requests,
- explain financial reports,
- summarize invoices/bills,
- draft payment approval explanations,
- classify expense categories as proposals,
- identify missing invoice fields,
- draft customer/supplier messages,
- explain budget variance,
- assist with reconciliation explanations.

OpenAI/GPT-5.5 must not:

- post ledger entries,
- approve payments,
- execute bank transfers,
- invent tax law,
- approve refunds,
- approve budget changes,
- expose private financial data,
- bypass Access/Authority/Simulation.

Repo-truth evidence:

- Adapter `h406_official_company_execution_request` detects official company P&L/accounting-record/report requests and invoice posting as protected execution.
- Adapter tests prove "Run the official company P&L from accounting records" and "Post this invoice" stay fail-closed with `NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION`.
- Public "Simple Profit and Loss Template" wording is explicitly advisory and not official company accounting execution.

## 18. PH1.WRITE Interaction

Current finance/accounting wording is not governed by a dedicated Finance/Accounting owner. Existing repo truth shows a hardcoded public advisory P&L template and protected failure text from Adapter/protected path tests.

| Wording Area | Evidence | Status | Risk |
|---|---|---|---|
| payment approval explanations | NOT_FOUND | FINANCE_WRITING_OWNER_RISK | PH1.WRITE future boundary needed. |
| invoice summaries | NOT_FOUND | ACCOUNTING_WRITING_OWNER_RISK | No invoice owner. |
| finance denials | Adapter protected failure phrase | PARTIAL | Needs PH1.WRITE finance wording. |
| budget variance explanations | NOT_FOUND | FINANCE_WRITING_OWNER_RISK | Missing. |
| supplier/customer notices | NOT_FOUND | ACCOUNTING_WRITING_OWNER_RISK | Missing. |
| payroll/payment confirmations | Payroll/HR future docs only | DESIGN_ONLY | Missing Finance/Accounting runtime. |
| tax/reporting explanations | NOT_FOUND | TAX_GAP | Missing. |
| sensitive financial field explanations | Access future docs only | DESIGN_ONLY | Missing field owner. |

Correct future rule:

PH1.WRITE owns final wording for payment approval explanations, invoice summaries, finance denials, budget variance explanations, supplier/customer notices, payroll/payment confirmations, tax/reporting explanations, and sensitive financial field explanations. Adapter-side hardcoded finance text should be retired or bounded once PH1.WRITE finance surfaces exist.

Risks:

- `FINANCE_WRITING_OWNER_RISK`
- `ACCOUNTING_WRITING_OWNER_RISK`
- `HARDCODED_FINANCE_TEXT_RISK`
- `CLIENT_FINANCE_TEXT_RISK`
- `ADAPTER_FINANCE_TEXT_RISK`

## 19. Desktop / iPhone / Adapter Boundaries

| Surface | Current Behavior | Runtime Behavior | Risk |
|---|---|---|---|
| Desktop finance/accounting UI | No dedicated finance UI found in targeted search | None found | DESKTOP_FINANCE_AUTHORITY_RISK if future UI mutates directly |
| iPhone finance/accounting UI | No dedicated finance UI found in targeted search | None found | IPHONE_FINANCE_AUTHORITY_RISK if future UI mutates directly |
| Adapter finance/accounting route | Detects official company P&L/accounting/report/invoice posting as protected and fail-closes; can return public advisory P&L template | No official accounting execution | ADAPTER_FINANCE_AUTHORITY_RISK if hardcoded path becomes execution |
| invoice routes | Adapter protected phrase only | No invoice runtime | ADAPTER_ACCOUNTING_AUTHORITY_RISK |
| payment routes | No bank/payment provider route found | No payment runtime | ADAPTER_PAYMENT_AUTHORITY_RISK |
| bank routes | No business bank route found | No bank runtime | BANKING_GAP |
| budget routes | PH1.COMP computation only; no client route | No budget runtime | FINANCE_GAP |
| report routes | Public advisory P&L template; official P&L blocked | No company-report runtime | ACCOUNTING_GAP |
| private finance display | No finance data display found | None | SECURITY_GAP |

State clearly:

- Client currently does not appear to decide finance/accounting truth.
- Runtime currently has no Finance/Accounting business owner.
- Adapter currently protects some official accounting/payment-like prompts and provides public advisory templates only.
- Adapter must remain transport/routing/presentation boundary, not finance truth.

## 20. Security / Privacy / Compliance Model

| Sensitive Area | Repo Evidence | Status |
|---|---|---|
| bank account privacy | Payroll/HR future docs; no Finance bank model | BANKING_GAP |
| supplier bank details | Master Access examples mention supplier bank access | DESIGN_ONLY |
| customer financial records | Access examples mention customer invoices | DESIGN_ONLY |
| invoices | protected phrase detection only | AR_GAP |
| tax identifiers | Payroll/HR future docs only | TAX_GAP |
| payroll liabilities | future docs only | DESIGN_ONLY |
| financial reports | official P&L protected in Adapter | PARTIAL |
| budget privacy | Access future spend authority | DESIGN_ONLY |
| payment approvals | Access approval substrate | PARTIAL / PAYMENT_GAP |
| field-level access | Master Access future design | DESIGN_ONLY |
| tenant/workspace/company scope | global Access/tenant patterns exist | PARTIAL |
| audit | generic audit/proof; no finance-specific audit | AUDIT_GAP |
| idempotency | Access/storage patterns exist | PARTIAL |
| tax compliance | NOT_FOUND | TAX_GAP |
| accounting period locks | NOT_FOUND | ACCOUNTING_GAP |
| payment export/download | NOT_FOUND | PAYMENT_GAP |
| bank provider credentials | Realtime public finance API key config exists, not bank credentials | BANKING_GAP |
| financial data retention | NOT_FOUND | COMPLIANCE_GAP |
| role-based visibility | Access general/future | PARTIAL |
| dual approval | Access approval primitive supports `NOfM` and board quorum | PARTIAL |

Missing security/compliance items must be treated as blockers for any future implementation: `SECURITY_GAP`, `COMPLIANCE_GAP`, `AUDIT_GAP`, `BANKING_GAP`, `PAYMENT_GAP`, `TAX_GAP`.

## 21. State Machines

No Finance/Accounting runtime state machines were found. The following are RECONSTRUCTED_FROM_REPO_EVIDENCE as future target shapes only. They must not be read as implemented.

### Invoice State Machine

```text
RECONSTRUCTED_FROM_REPO_EVIDENCE
Draft
PendingApproval
Approved
Sent
PartiallyPaid
Paid
Overdue
Cancelled
Credited
Archived
```

Status: NOT_IMPLEMENTED / AR_GAP.

### Supplier Bill / AP State Machine

```text
RECONSTRUCTED_FROM_REPO_EVIDENCE
Received
Draft
Matched
PendingApproval
Approved
PaymentInstructionCreated
Paid
Disputed
Cancelled
Archived
```

Status: NOT_IMPLEMENTED / AP_GAP.

### Payment State Machine

```text
RECONSTRUCTED_FROM_REPO_EVIDENCE
Draft
PendingApproval
Approved
SentToBank
Confirmed
PartiallyFailed
Failed
Reconciled
Reversed
Cancelled
```

Status: NOT_IMPLEMENTED / PAYMENT_GAP.

### Journal State Machine

```text
RECONSTRUCTED_FROM_REPO_EVIDENCE
Draft
PendingApproval
Posted
Reversed
Locked
Archived
```

Status: NOT_IMPLEMENTED / ACCOUNTING_GAP.

### Budget State Machine

```text
RECONSTRUCTED_FROM_REPO_EVIDENCE
Draft
PendingApproval
Active
OverBudget
Revised
Closed
```

Status: NOT_IMPLEMENTED. PH1.COMP can compute threshold posture from supplied values only.

## 22. Error Handling And Reason Codes

Current evidence:

- `RealtimeFinance` can emit `provider_unconfigured` and `policy_violation` through realtime adapter error kinds.
- `PH1.COMP` budget/quota computation can emit `invalid_budget_quota_limits`, `budget_projection_overflow`, `budget_threshold_crossed`, `budget_threshold_clear`, and `invalid_budget_packet`.
- Access AP schema paths can emit `ACCESS_AP_REQUIRED`, `ACCESS_AP_SCHEMA_INVALID`, `ACCESS_AP_SCOPE_VIOLATION`, `ACCESS_AP_ACTIVATION_CONFLICT`, and `ACCESS_AP_PROJECTION_CONFLICT`.
- Adapter protected accounting/payment-like execution returns fail-closed wording containing `NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION`.

Missing Finance/Accounting reason-code gaps:

| Missing Reason | Status |
|---|---|
| account not found | ACCOUNTING_GAP |
| ledger missing | ACCOUNTING_GAP |
| chart of accounts missing | ACCOUNTING_GAP |
| accounting period closed | ACCOUNTING_GAP |
| budget missing | FINANCE_GAP |
| budget exceeded | PARTIAL: PH1.COMP threshold only; no budget owner |
| approval required | PARTIAL: Access generic |
| authority missing | PARTIAL: global/Access |
| supplier not found | AP_GAP |
| customer not found | AR_GAP |
| invoice missing | AR_GAP |
| bill missing | AP_GAP |
| payment instruction missing | PAYMENT_GAP |
| bank account missing | BANKING_GAP |
| bank validation failed | BANKING_GAP |
| payment provider failed | PAYMENT_GAP |
| partial payment failed | PAYMENT_GAP |
| reconciliation mismatch | ACCOUNTING_GAP |
| tax code missing | TAX_GAP |
| GST/VAT missing | TAX_GAP |
| duplicate invoice | AR_GAP |
| duplicate payment | PAYMENT_GAP |
| refund requires approval | DESIGN_GAP |
| contractor invoice mismatch | AP_GAP |
| payroll journal blocked | ACCOUNTING_GAP |
| client route mismatch | ADAPTER_FINANCE_AUTHORITY_RISK |

## 23. Audit / Provenance / Evidence

| Audit Question | Repo Truth Answer | Status |
|---|---|---|
| Is journal creation audited? | No journal engine found. | AUDIT_GAP |
| Is ledger posting audited? | No General Ledger found. | AUDIT_GAP |
| Is invoice creation/sending audited? | No invoice engine found. | AUDIT_GAP |
| Is bill approval audited? | No AP bill engine found. Access approvals can be audited generically. | PARTIAL / AUDIT_GAP |
| Is payment approval audited? | Access approval substrate exists; no payment approval runtime. | PARTIAL / PAYMENT_GAP |
| Is bank transfer audited? | No bank transfer runtime. | AUDIT_GAP |
| Is payment confirmation audited? | No payment confirmation runtime. | AUDIT_GAP |
| Is reconciliation audited? | No reconciliation engine. | AUDIT_GAP |
| Is budget change audited? | Access policy changes audited; no budget owner. | PARTIAL / FINANCE_GAP |
| Is tax/GST/VAT calculation audited? | No tax engine. | TAX_GAP |
| Is payroll accounting handoff audited? | Future design only. | DESIGN_ONLY / AUDIT_GAP |
| Is contractor AP payment audited? | Future design only. | DESIGN_ONLY / AUDIT_GAP |
| Are supplier/customer bank detail views audited? | Access future design only. | SECURITY_GAP |
| Are access checks audited? | Access storage/docs show AP/board ledgers and audit patterns. | PARTIAL |
| Are client/adapter finance events audited? | Adapter has protected fail-closed tests and conversation/proof ledgers generally. No finance-specific event model. | PARTIAL / AUDIT_GAP |

Generic evidence ledgers are useful but not sufficient. Accounting and payment actions need finance-specific audit fields, old/new refs, authority result, simulation id, payment provider refs, reconciliation refs, and retention policy refs.

## 24. Current Tests / Smokes / Acceptance

| Test / Smoke | Path | What It Proves | What It Does Not Prove | Status |
|---|---|---|---|---|
| `at_comp_06_budget_quota_calculation_is_deterministic` | `crates/selene_os/src/ph1comp.rs` | Identical budget/quota inputs produce deterministic computation packets. | Does not prove budget records, approvals, finance owner, accounting, payment, or persistence. | PARTIAL |
| `at_comp_core_01_budget_posture_is_deterministic` | `crates/selene_engines/src/ph1comp.rs` | Deterministic math for budget posture. | Does not prove business Finance/Budget engine. | PARTIAL |
| PH1.COMP DB wiring tests | `docs/DB_WIRING/PH1_COMP.md` | Required proof list for computation packets and budget/quota math. | Not Finance/Accounting lifecycle tests. | PARTIAL |
| `test_t7_missing_retrieved_at_fails_without_inference` | `crates/selene_os/src/web_search_plan/realtime/realtime_tests.rs` | Realtime evidence packet must include retrieved timestamp and fail without inference. | Not finance accounting; uses `RealtimeFinance` only as realtime adapter label. | PARTIAL |
| `h406_protected_business_execution_stays_fail_closed` | `crates/selene_adapter/src/lib.rs` | Prompts such as "Post this invoice" and "Run the official company P&L from accounting records" fail closed without protected simulation/authority. | Does not implement invoice, accounting report, payment, or approval execution. | PARTIAL |
| `h407_voice_like_public_committed_turns_preserve_public_lane_and_h406_presentation` | `crates/selene_adapter/src/lib.rs` | Public "profit and loss report template" can be answered as advisory, not official company accounting. | No official accounting data/reporting. | PARTIAL |
| Access AP/board tests | `crates/selene_kernel_contracts/src/ph1access.rs`, `crates/selene_storage/src/ph1f.rs` | Approval-policy schema and board policy/vote substrate. | AP here is not Accounts Payable; no finance payment approval lifecycle. | PARTIAL |

No tests were found proving Finance/Accounting/AP/AR/Banking/Payment/Budget/Tax as complete product runtime owners. Mark `TEST_GAP`.

## 25. Old Paths / Compatibility / Wrong-Owner Risks

| Path / Symbol | Current Status | Correct Canonical Owner | Retirement Condition | Active-Caller Check Needed |
|---|---|---|---|---|
| `PH1.COMP` budget/quota math | Active computation helper | Finance/Budget owns business budgets; PH1.COMP computes only | Keep as math helper; never call it budget truth | Yes: caller must supply authoritative budget inputs |
| `RealtimeFinance` adapter | Active public realtime tool adapter | Search/Realtime public information lane | Keep public lookup; never internal company finance | Yes: prevent private/company accounting data use |
| Access `AP` schema tables | Active Access Approval Policy | Access/Governance | Keep naming documented; do not confuse with Accounts Payable | Yes: Accounts Payable future files should avoid ambiguous `AP` without context |
| `artifacts_ledger`, memory/conversation ledgers | Active evidence/storage ledgers | PH1.F / owning engine | Never treat as General Ledger | Yes: GL search must distinguish accounting ledger |
| Adapter protected phrase detection | Active safety guard | PH1.X/Access/future Finance owner | Retire hardcoded phrases only after PH1.X + Finance route proof | Yes: protected official company accounting remains fail-closed |
| Adapter public P&L template | Active advisory fallback | PH1.WRITE + public advisory lane | Retire into PH1.WRITE once finance wording exists | Yes: must keep "not official company accounting execution" boundary |
| Payroll/HR future payment handoffs | Design-only | Payroll + Accounting + Banking + AP | Activate only after Finance/Accounting repo-truth build | Yes: no payment action from docs alone |
| Contractor payment in Payroll docs | Design-only risk | AP/Accounting by default | Reconcile contractor/AP owner before implementation | Yes: contractor must not default to employee payroll |
| Desktop/iPhone future finance UI | Not found | Render-only clients | Any future client must be render/submit only | Yes: no client-side money authority |
| Adapter payment shortcuts | Not found now | Banking/Payment Provider + Finance/Accounting | Block future shortcuts | Yes: no provider call without simulation/proof |
| Invoice delivery bypassing BCAST/DELIVERY | Not found now | AR source + BCAST/DELIVERY transport | Future invoice design must route through delivery owners | Yes |
| Duplicate finance/accounting/payment engines | Not found now | Future reconciled owners | Activation pack must prevent duplicates | Yes |
| Stale docs or design-only owner claims | Present in future docs by design | Grand Architecture Reconciliation | Map before implementation | Yes |

## 26. Full Functionality Master List

| Functionality | Description | Evidence | Owner | Status | Future Action |
|---|---|---|---|---|---|
| create chart of accounts | Define account codes/classes | No files/tables found | Accounting | NOT_FOUND | Build Accounting owner slice. |
| create journal entry | Draft journal | No journal engine found | Accounting | NOT_FOUND | Build journal contract. |
| post journal | Post to ledger | No GL found | Accounting | NOT_FOUND | Build posting + lock model. |
| reverse journal | Reverse posted entry | No runtime found | Accounting | NOT_FOUND | Build reversal states. |
| accounting period close | Close fiscal period | No period model found | Accounting | NOT_FOUND | Build fiscal period/lock. |
| create invoice | Customer invoice draft | Adapter protects "Post this invoice"; no engine | AR / Invoice | NOT_FOUND | Build AR/invoice model. |
| send invoice | Deliver invoice | No invoice delivery | AR + BCAST/DELIVERY | NOT_FOUND | Build delivery handoff. |
| record customer payment | Capture receipt | No receipt model | AR | NOT_FOUND | Build receipt/allocation. |
| allocate receipt | Match payment to invoice | No AR model | AR | NOT_FOUND | Build allocation flow. |
| manage overdue invoice | Track overdue and reminders | No AR runtime | AR + REM/BCAST | NOT_FOUND | Build overdue/reminder. |
| create credit note | Credit customer | No credit note model | AR | NOT_FOUND | Build credit/refund model. |
| create supplier bill | AP bill record | No AP engine | AP | NOT_FOUND | Build AP bill model. |
| approve supplier bill | Approval workflow | Access generic approval only | AP + Access | PARTIAL | Build AP-specific approval. |
| process contractor invoice | Contractor AP payment evidence | Future docs only | AP | DESIGN_ONLY | Build contractor/AP boundary. |
| create payment instruction | Prepare approved payment | Future logical packet only | Payroll/AP + Accounting | DESIGN_ONLY | Build payment instruction contract. |
| send bank transfer | Execute payment rail | No bank provider | Banking Provider | NOT_FOUND | Build protected provider integration. |
| record payment confirmation | Store provider confirmation | No runtime | Banking/Accounting | NOT_FOUND | Build confirmation/failure lifecycle. |
| reconcile bank payment | Match bank feed/payment | No reconcile engine | Accounting | NOT_FOUND | Build reconciliation. |
| create budget | Budget owner setup | No business budget model | Finance/Budget | NOT_FOUND | Build budget model. |
| check spend limit | Verify authority and money availability | Access docs + PH1.COMP math partial | Access + Finance/Budget | PARTIAL | Build Finance/Budget truth plus Access gate. |
| approve over-budget spend | Escalate over budget | Access future design | Finance/Budget + Access | DESIGN_ONLY | Build multi-approver flow. |
| calculate GST/VAT | Tax calculation | No tax engine | Tax/Compliance | NOT_FOUND | Build source-backed rule pack. |
| tax reporting | Prepare/submission handoff | Future docs only | Tax/Compliance + Accounting | DESIGN_ONLY | Build reporting owner. |
| create payroll journal | Post payroll to books | Future docs only | Accounting | DESIGN_ONLY | Build Payroll/Accounting handoff. |
| payroll payment instruction | Employee pay instruction | Future docs only | Payroll + Accounting | DESIGN_ONLY | Build protected payment handoff. |
| process contractor AP payment | Vendor/contractor payment | Future docs only | AP + Accounting + Banking | DESIGN_ONLY | Build AP payment flow. |
| create reimbursement | Employee expense/reimbursement | Access examples only | Finance/AP/Payroll by policy | DESIGN_GAP | Reconcile owner. |
| supplier bank detail view | View sensitive supplier bank data | Access examples only | Access + AP/Finance | DESIGN_ONLY | Build field-level access. |
| official P&L report | Run company report from records | Adapter blocks as protected | Accounting / Finance | PARTIAL / NOT_IMPLEMENTED | Build report owner after GL. |
| advisory P&L template | Public template from user request | Adapter fallback | Public answer / PH1.WRITE future | ACTIVE / ADVISORY_ONLY | Move to PH1.WRITE later. |

## 27. Comparison To Master Architecture

Payroll/HR extraction and automation docs:

- Current Payroll/HR repo truth says payroll is not complete and Finance/Budget is future money owner.
- Payroll/HR automation docs assume Finance/Accounting/AP/Banking future owners for payment, ledger, tax/reporting, contractor payment, and accounting handoff.
- This extraction confirms those are future architecture claims, not current Finance/Accounting runtime proof.

Payroll/HR Evidence Fabric:

- Future `AccountingHandoffPacket`, `PaymentInstructionPacket`, and `PaymentConfirmationPacket` are readiness concepts.
- No runtime packet implementation was found for Finance/Accounting.

PH1.POSITION Position Journey:

- Position may hand off compensation/cost-to-company/budget impact.
- Position must not own finance/pay truth.

HWM / Task / Roster / Attendance / Staffing docs:

- These docs may produce labour/work/contractor evidence.
- They must not calculate money truth, post journals, or execute payments.

Master Access Governance + Per-User Access Journey:

- Access has the strongest current substrate for spend/private-field authority.
- Access owns "can this actor do it", not "is the budget true" or "post the books".

PH1.REM:

- Reminder timing exists generically; invoice/payment/budget reminders need a future Finance/AP/AR source.

PH1.BCAST / PH1.DELIVERY:

- Delivery ownership exists. Invoice/payment notices must route through it later.

PH1.D Proposal Gateway:

- May draft/explain/propose. Must not post, approve, execute, or invent finance/tax truth.

PH1.N Meaning Unravelling:

- May extract finance request candidates later. It does not own financial data.

PH1.X Request Decision Lattice:

- Already protects official company accounting execution prompts at Adapter/protected path level. Future Finance routes must consume PH1.X risk validation.

PH1.WRITE Human Presentation:

- Future finance wording should move from hardcoded Adapter fallback into PH1.WRITE-owned, truth-bound text.

Identity + Access + Authority Spine:

- Required for financial private reads, approvals, payment instructions, provider send, exports, and bank data changes.

Tenant / Workspace Governance:

- Finance data must be tenant/company scoped; current finance-specific scope model is missing.

Desktop/iPhone render-only boundary:

- No finance UI found. Future UI must render/submit bounded inputs only.

Adapter transport-only boundary:

- Adapter currently detects and blocks protected accounting-like requests. It must not become Finance/Accounting runtime.

Old Compatibility Path Retirement:

- Retire or bound hardcoded protected phrase detection and advisory accounting templates only after PH1.X/PH1.WRITE/Finance owners exist.

## 28. Gaps / Missing Pieces

| Gap | Evidence | Risk | Recommended Future Action | Priority |
|---|---|---|---|---|
| missing standalone Finance engine | no engine files/inventory row | FINANCE_GAP | Finance/Accounting Repo-Truth Activation Pack | P0 |
| missing standalone Accounting engine | no accounting/ledger/journal files | ACCOUNTING_GAP | Accounting owner split design | P0 |
| unclear Finance vs Accounting split | future docs split but no runtime | OWNER_GAP | Grand Architecture Reconciliation | P0 |
| missing AP engine | no Accounts Payable files | AP_GAP | AP supplier/contractor model | P0 |
| missing AR engine | no AR/customer invoice files | AR_GAP | AR invoice/receipt model | P0 |
| missing GL / ledger | storage ledgers are not GL | ACCOUNTING_GAP | General Ledger model | P0 |
| missing chart of accounts | no CoA model | ACCOUNTING_GAP | CoA setup slice | P0 |
| missing journal entries | no journal runtime | ACCOUNTING_GAP | Journal draft/post/reverse | P0 |
| missing invoice engine | only protected phrase detection | AR_GAP | Invoice lifecycle slice | P0 |
| missing bill engine | no bill model | AP_GAP | AP bill lifecycle slice | P0 |
| missing payment provider boundary | no bank/payment provider | PAYMENT_GAP | Provider mock + protected rail | P0 |
| missing bank reconciliation | no reconcile engine | ACCOUNTING_GAP | Reconciliation model | P1 |
| missing budget/profitability engine | PH1.COMP math only | FINANCE_GAP | Budget/cost center/profitability owner | P0 |
| missing cost center | no model | FINANCE_GAP | Cost center model | P1 |
| missing GST/VAT/tax owner | no tax engine | TAX_GAP | Source-backed tax rule owner | P0 |
| missing payroll accounting handoff | future docs only | ACCOUNTING_GAP | Payroll journal handoff | P0 |
| missing contractor AP boundary | future docs only | AP_GAP | Contractor/AP packet flow | P0 |
| missing customer/supplier payment flow | no AP/AR/banking | AP_GAP / AR_GAP | Supplier/customer payment lifecycle | P0 |
| missing refund flow | Adapter phrase only | PAYMENT_GAP | Refund authority + AR flow | P1 |
| missing reimbursement flow | Access examples only | FINANCE_GAP | Reimbursement owner reconciliation | P1 |
| missing spend approval matrix | Access partial/future | OWNER_GAP | Spend authority matrix | P0 |
| missing payment authority gate | global protected only | PAYMENT_GAP | Payment protected execution gate | P0 |
| missing field-level financial access | Access future only | SECURITY_GAP | Finance field classification | P0 |
| missing PH1.WRITE finance/accounting wording | hardcoded Adapter fallback | FINANCE_WRITING_OWNER_RISK | PH1.WRITE finance boundary | P1 |
| missing PH1.D/PH1.N finance/accounting proposal path | no finance-specific route | DESIGN_GAP | Proposal/extraction shell | P1 |
| missing audit | no finance-specific audit | AUDIT_GAP | Finance/Accounting audit evidence pack | P0 |
| missing SQL persistence | no finance migrations | ACCOUNTING_GAP | Storage schema design | P0 |
| missing Desktop/iPhone render-only proof | no finance UI | CLIENT_GAP | Render-only acceptance proof later | P2 |
| missing Adapter transport-only proof | Adapter blocks some prompts | ADAPTER_FINANCE_AUTHORITY_RISK | Harden/retire old path | P1 |
| missing JD live acceptance | no live finance proof | TEST_GAP | JD live Finance/Accounting acceptance pack | P2 |

## 29. Recommended Future Build Slices

Based on repo truth, future slices should be derived only after Grand Architecture Reconciliation:

1. Finance/Accounting Repo-Truth Activation Pack
2. Finance vs Accounting Owner Split Reconciliation
3. AP vs Access Approval Policy Naming Reconciliation
4. Chart of Accounts / Ledger Model
5. Journal Entry Draft / Approval / Posting
6. Accounting Period / Fiscal Year / Period Lock
7. AP Supplier Bill / Contractor Invoice Model
8. AP Approval / Payment Instruction Flow
9. AR Customer Invoice / Receipt Model
10. Invoice Sending / Reminder Delivery Boundary
11. Payment Provider / Bank Handoff Boundary
12. Payment Confirmation / Failure / Reconciliation
13. Budget / Cost Center / Spend Limit Model
14. Profitability / Forecast / Cashflow Boundary
15. Tax / GST / VAT Rule Boundary
16. Payroll Accounting Handoff
17. Contractor/AP Boundary
18. Customer/Supplier Financial Access Model
19. Payment Protected Execution Gate
20. PH1.D + PH1.N Finance/Accounting Proposal Shell
21. PH1.X Finance/Accounting Route/Risk Validation
22. PH1.WRITE Finance/Accounting Explanation Boundary
23. Finance/Accounting Audit Evidence Pack
24. Desktop/iPhone Render-Only Finance Proof
25. Adapter Transport-Only Finance Proof
26. JD Live Finance/Accounting Acceptance Pack

## 30. What Codex Must Not Do

- do not invent Finance/Accounting behavior
- do not create duplicate Finance/Accounting engines
- do not merge Finance and Accounting if repo/future architecture requires split
- do not let Payroll own accounting truth
- do not let Finance own payroll calculation
- do not let AP ambiguity confuse Approval Policy with Accounts Payable
- do not let Scheduler/Roster/Attendance calculate money truth
- do not let GPT-5.5/OpenAI post journals, approve payments, or invent tax truth
- do not let Desktop/iPhone decide finance/accounting/payment truth
- do not let Adapter decide finance/accounting/payment truth
- do not call bank/payment providers without protected simulation and provider proof
- do not claim tax/accounting compliance without source/owner proof
- do not treat contractor payment as employee payroll without proof
- do not treat `RealtimeFinance` as internal accounting truth
- do not treat `PH1.COMP` budget/quota posture as business budget truth
- do not treat Access `AP` as Accounts Payable
- do not treat artifact/memory/conversation ledgers as General Ledger
- do not delete old paths before proof
- do not implement from this extraction document alone

## 31. Final Extracted Architecture Sentence

Selene Finance + Accounting is the governed money, books, budget, payment, invoice, AP/AR, bank, reconciliation, and tax-reporting boundary where repo truth supports it: current repo truth does not prove complete Finance, Accounting, AP, AR, Banking, Payment, Ledger, Budget, Profitability, or Tax runtime engines; Accounting may own ledger, journals, chart of accounts, reconciliation, and financial reports in future; Finance/Budget may own budgets, cashflow, profitability, spend authority, and financial governance; AP/AR may own supplier/contractor bills and customer invoices/receipts; Banking/Payment providers may execute payment rails only through protected proof; Payroll, HR, Compensation, Scheduler/Roster/Attendance, Position, Onboarding, Access, Reminder, Broadcast/Delivery, PH1.D, PH1.N, PH1.X, and PH1.WRITE must remain separate canonical owners unless repo truth proves otherwise.
