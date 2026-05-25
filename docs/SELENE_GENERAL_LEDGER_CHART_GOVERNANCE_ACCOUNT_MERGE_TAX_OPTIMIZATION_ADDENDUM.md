# Selene General Ledger Addendum — Chart Governance, Account Merge, Tax Optimization + Error-Prevention Law

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / GENERAL LEDGER CHART GOVERNANCE + ACCOUNT MERGE + TAX OPTIMIZATION + ERROR PREVENTION

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define addendum law for chart governance, adding/retiring/renaming/merging account and cost categories, spare-parts classification, lawful tax-optimization study, shareholder-benefit analysis boundary, and prevention-first accounting correction behavior.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document extends the Selene General Ledger + Chart of Accounts + Journals + Period Close Master Design. It must be read with that document and with the Finance + Accounting Autonomous Umbrella.

Current repo truth does not prove these runtime capabilities. These are future master-design rules pending Grand Architecture Reconciliation.

## A. Chart Of Accounts Change Governance

Authorized management, Finance, or Accounting users may ask Selene to add, retire, rename, merge, or reorganize:

```text
account codes
subaccounts
cost categories
reporting categories
budget categories
inventory categories
tax mappings
```

Core accounting types such as Asset, Liability, Equity, Revenue, COGS, and Expense should not be casually changed because they support statutory reporting and double-entry rules.

Selene must distinguish:

```text
core account type
account code
subaccount
cost type
cost center
inventory category
reporting group
tax mapping
budget category
```

Example:

User says:

```text
Selene, add a new account type for spare parts.
```

Selene should not blindly create a new core account type.

Selene should ask:

```text
Should spare parts be treated as inventory, repair expense, maintenance stock, or part of a fixed asset?
```

Possible outcomes:

```text
Spare Parts Inventory under Assets
Spare Parts Expense under Repairs and Maintenance
Spare Parts COGS category
Spare Parts Asset Component where capitalization rules apply
```

Selene must preview reporting, tax, budget, and posting impact before activation.

Change preview should include:

```text
what will be created or changed
which owner controls the change
affected account codes
affected reports
affected tax mappings
affected budgets
effective date
approval requirement
audit refs
rollback/correction path if supported
```

Chart change authority must come from Access/Governance. PH1.D may help explain the request. PH1.WRITE owns final user-facing wording. Accounting owns chart truth.

## B. Account / Cost Type Merge Law

Selene must support controlled merging of account codes, cost types, reporting categories, and budget categories.

Example:

```text
Selene, merge these two cost types into one called XYZ.
```

Selene must:

1. identify both source categories,
2. check authority,
3. check whether they are ledger accounts, cost centers, reporting categories, or budget categories,
4. preview affected transactions,
5. preview affected reports,
6. preview tax mapping impact,
7. preview budget impact,
8. ask effective date,
9. ask historical treatment,
10. require approval if needed,
11. retain full history,
12. audit the change.

Historical treatment options:

```text
future-only merge
reporting-only merge
approved reclassification journal
archive old categories but keep history
```

Hard rule:

```text
No historical transaction, ledger posting, invoice, bill, payroll record, tax record, or audit record may be erased merely because categories are merged.
```

Merge output should identify:

```text
merge_request_id
source_category_refs
target_category_ref
category_type
effective_date
historical_treatment
affected_transaction_count
affected_report_refs
affected_tax_mapping_refs
affected_budget_refs
approval_refs
audit_ref
```

If historical treatment requires reclassification, Accounting must use approved reclassification journals. Reporting-only merges must not rewrite ledger truth.

## C. Lawful Tax Optimization + Shareholder Benefit Study

Selene should help management lawfully reduce tax liabilities and maximize shareholder benefit.

Selene may perform:

```text
tax study
accounting treatment study
depreciation method comparison
capital vs expense analysis
claimable deduction review
GST/VAT claimability review
asset purchase timing analysis
dividend timing analysis
retained earnings analysis
shareholder class distribution analysis
entity/country comparison
cashflow impact analysis
board summary preparation
```

Selene must use:

```text
source-backed country/region rules
approved tax/accounting providers where available
accountant-approved company policy
effective-date rule packs
audit evidence
```

Selene must not:

```text
invent tax law
recommend illegal tax evasion
execute tax strategy without approval
hide liabilities
misclassify expenses to reduce tax
change accounting treatment without valid rule and authority
```

PH1.D / GPT-5.5 may help explain and summarize.

Tax/Compliance and Accounting owners validate.

Finance/Board approve where required.

Tax optimization output should be framed as a governed study or recommendation packet, not as execution:

```text
TaxOptimizationStudyPacket
  study_id
  requested_by
  legal_entity_id
  country
  region
  topic
  source_rule_refs
  accounting_treatment_candidates
  cashflow_impact
  tax_impact
  shareholder_benefit_notes
  required_approvals
  accountant_review_status
  board_review_status
  audit_ref
```

This document does not authorize a Tax runtime engine or a dividend/distribution engine.

## D. Error Prevention First, Correction Second

Selene's duty is to prevent wrong entries before posting.

Before posting, Selene must validate:

```text
source evidence
account code
tax code
cost center
legal entity
currency
accounting period
approval status
double-entry balance
restricted account rules
period lock status
owner handoff
```

Correction and reversal flows exist only for:

```text
discovered evidence errors
supplier/customer correction
approved reclassification
tax treatment correction
period-end adjustment
duplicate bank/payment issue
approved accounting correction
audit-required reversal
```

Selene must not knowingly post wrong entries.

Selene must not silently edit posted ledger entries.

If an error is detected after posting, Selene must use approved correction, reversal, or reclassification flow with audit.

Prevention-first validation should produce:

```text
PostingPreventionPacket
  source_event_id
  proposed_journal_ref
  prevention_reason_code
  blocked_field
  required_owner
  suggested_resolution
  audit_ref
```

Post-posting correction should produce:

```text
AccountingCorrectionPacket
  original_journal_id
  correction_type
  correction_reason
  evidence_refs
  period_status
  authority_result
  approval_refs
  resulting_journal_refs
  audit_ref
```

## E. What Must Not Happen Additions

```text
no casual creation of new core accounting types
no account merge that erases history
no reporting category merge that changes tax truth silently
no cost-type merge without effective date and audit
no tax optimization without source-backed rules
no illegal tax avoidance/evasion recommendation
no shareholder-benefit strategy without Finance/Board/Tax approval where required
no knowingly wrong journal posting
no silent correction of posted entries
no historical ledger rewrite disguised as cleanup
no PH1.D/GPT-5.5 authority to change the chart
no client or adapter authority to merge accounting categories
no implementation from this document alone
```

## F. Final Addendum Sentence

Selene General Ledger Addendum — Chart Governance, Account Merge, Tax Optimization + Error-Prevention Law ensures future Accounting does not casually mutate the financial language of the company: chart changes, category merges, tax studies, shareholder-benefit studies, and posting corrections must preserve history, preview reporting/tax/budget impact, use source-backed rules, require authority and approval where needed, prevent wrong postings first, and correct already-posted truth only through governed reversal, reclassification, or correction with audit.
