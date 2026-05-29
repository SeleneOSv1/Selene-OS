# Finance / Accounting Batch 9 — Documents 17–18 Asset Accounting, Depreciation, Debt + Security Accounting

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN BATCH

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## Batch Contents

- Finance / Accounting Document 17 — Asset Accounting, Depreciation + Claimable Expense Rules
- Finance / Accounting Document 18 — Debt, Loans, Borrowing Costs, Covenants + Security Accounting

This batch mechanically consolidates the previously accepted standalone source documents below. Architecture content is preserved verbatim between source-file markers.

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_17_ASSET_ACCOUNTING_DEPRECIATION_CLAIMABLE_EXPENSE_RULES_MASTER_DESIGN.md -->
# Finance / Accounting Document 17 — Selene Asset Accounting, Depreciation + Claimable Expense Rules Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
17

ENGINE:
PH1.ASSET_ACCOUNTING / PH1.DEPRECIATION / PH1.EXPENSE_RULES

FULL NAME:
Selene Asset Accounting, Capitalization, Depreciation, Impairment, Disposal, Claimable Expense, Repair-vs-Capital, and Asset Evidence Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition. No runtime code, schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.ASSET_ACCOUNTING / PH1.DEPRECIATION / PH1.EXPENSE_RULES. Repo-truth activation, simulation mapping, owner mapping, tests, source pack governance, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Asset Accounting + Depreciation + Claimable Expense Rules Engine owns the accounting treatment of business assets, capital purchases, depreciation, amortization handoffs, impairment indicators, disposals, repairs, maintenance, prepayments, accrual candidates, and claimable expense classification.

It answers:

```text
Is this purchase an asset or an expense?
Should this cost be capitalized, expensed, prepaid, or accrued?
What asset class does it belong to?
When does depreciation start?
What depreciation method applies?
What useful life and residual value apply?
Is there an impairment indicator?
Is this repair maintenance or a capital improvement?
Can this expense be claimed by the business?
Is tax claimability different from accounting treatment?
Was the asset sold, scrapped, stolen, traded in, or written off?
What gain or loss on disposal should Accounting recognize?
What evidence supports the treatment?
```

The engine prevents asset, expense, repair, prepayment, accrual, tax, and disposal treatment from being guessed from invoice wording alone.

## 2. Core Selene Law

```text
Selene must classify asset and expense treatment automatically wherever policy is clear, using evidence, thresholds, accounting rules, tax rules, and company policy.

Humans review only judgment-heavy, material, unusual, tax-sensitive, impairment-sensitive, or policy-exception cases.

Routine expenses and standard depreciation schedules should not create approval noise.
```

Selene must reduce human work by:

```text
reading invoices and receipts
detecting capital assets
detecting repairs vs improvements
creating draft asset accounting records
linking assets to suppliers, receiving, insurance, fleet, maintenance, and accounting
calculating depreciation schedules
flagging tax optimization opportunities
detecting missing evidence
preparing disposal calculations
preparing journal proposals
explaining treatment in human language
```

Selene should ask simple human questions only when evidence and policy cannot decide.

Example:

```text
Did this cost improve the asset, extend its useful life, or restore it to normal working condition?
```

Selene maps the answer to accounting treatment.

## 3. Engine Boundary

### 3.1 PH1.ASSET_ACCOUNTING owns

```text
asset accounting classification
capitalization decision support
fixed asset accounting handoff
depreciation rules
depreciation schedules
accumulated depreciation tracking
right-of-use asset accounting handoff
intangible asset accounting handoff
repair vs capital improvement rules
claimable expense classification
expense evidence requirements
prepayment vs expense classification
accrual candidates for asset/expense items
impairment indicator detection
disposal accounting calculation
gain/loss on disposal calculation
insurance recovery accounting handoff
asset accounting audit evidence
```

### 3.2 PH1.ASSET_ACCOUNTING does not own

```text
physical asset custody
asset maintenance work execution
vehicle odometer/fuel tracking
insurance policy administration
supplier qualification
purchase order creation
goods receiving proof
bank payment execution
final tax law strategy
legal title transfer
board/shareholder approval
```

### 3.3 Correct owner split

```text
PH1.ASSET = real-world asset lifecycle, custodian, condition, location, maintenance, asset register truth.
PH1.ASSET_ACCOUNTING = accounting treatment, capitalization, depreciation, disposal accounting, expense rules.
PH1.PROCUREMENT = approved purchase order.
PH1.PROC.RECEIVE = proof that asset/goods/service was received and accepted.
PH1.CREDITORS/AP = supplier invoice validation.
PH1.ACCOUNTING/GL = final journal posting.
PH1.TAX = tax compliance and tax treatment.
PH1.TAX.OPTIMIZE = legal tax minimization, capital allowance, depreciation optimization, claimable expense optimization.
PH1.INSURANCE = insurance coverage and claims.
PH1.FLEET = vehicle operating lifecycle.
Access/Authority = protected approvals.
Audit = evidence.
```

Simple split:

```text
Asset Engine knows the asset exists.
Asset Accounting knows how the asset is treated in the books.
Tax decides tax compliance treatment.
Tax Optimize investigates lawful tax-saving opportunities.
Accounting posts final financial truth.
```

## 4. Asset And Expense Classification

Selene must classify every relevant transaction into one of these accounting treatment categories:

```text
operating expense
capital asset
intangible asset
right-of-use asset
inventory item
prepayment
accrual
capital work in progress
repair and maintenance
capital improvement
low-value asset expense
mixed-use asset
non-claimable / personal / blocked expense
tax-review-required item
```

### 4.1 Classification inputs

```text
supplier invoice
purchase order
receiving record
product/service description
asset type
amount
threshold
useful life
business purpose
delivery/installation status
warranty
contract
lease term
ownership/title
location
custodian
policy rules
jurisdiction accounting/tax rules
```

### 4.2 Classification examples

```text
Laptop for employee -> asset or low-value expense depending policy/threshold
Annual software subscription -> prepaid expense or operating expense
Building extension -> capital improvement
Fixing broken door back to original state -> repair expense
New delivery vehicle -> fixed asset / fleet asset
Rented office under lease -> possible right-of-use asset
Patent purchase -> intangible asset candidate
Inventory purchased for resale -> inventory, not fixed asset
Owner personal holiday -> non-claimable / blocked
```

Selene explains:

```text
This looks like a capital asset because it has future economic benefit beyond the current period and exceeds your capitalization threshold. I will prepare an asset accounting record.
```

## 5. Capitalization Rules

Selene must determine whether a cost should be capitalized.

Capitalization tests:

```text
probable future economic benefit
cost can be measured reliably
business use
expected use beyond current period
above capitalization threshold
asset controlled by company
not inventory for resale
not routine repair
not personal/non-business
```

### 5.1 Capitalization threshold

Company policy defines thresholds.

Example:

```text
Items under a company threshold may be expensed unless policy or tax rules require capitalization.
Items above a company threshold with useful life beyond the current period may be capitalized.
```

Thresholds may differ by:

```text
entity
jurisdiction
asset class
tax rule
accounting standard
materiality
board policy
```

No threshold may be hardcoded by this document.

### 5.2 Capitalized cost components

Capitalized cost may include:

```text
purchase price
import duties
non-refundable taxes
freight/delivery
installation
directly attributable professional fees
testing
site preparation
decommissioning/restoration obligations where applicable
```

Selene must exclude:

```text
training not directly attributable unless policy allows
general admin
wasted costs
abnormal losses
routine maintenance
personal use portion
non-business costs
```

## 6. Fixed Asset Accounting Record

When capitalization is supported, Selene prepares an accounting asset record.

```text
asset_accounting_id
asset_id
legal_entity_id
asset_class
asset_subclass
purchase_date
available_for_use_date
supplier_id
invoice_id
po_id
receiving_id
cost
capitalized_cost_components
residual_value
useful_life
depreciation_method
depreciation_start_date
depreciation_frequency
accumulated_depreciation
carrying_amount
tax_depreciation_profile_ref
cost_center
location
custodian
insurance_required
impairment_status
disposal_status
audit_ref
```

Selene must not start depreciation merely because an invoice was received.

Depreciation starts when the asset is available for use under policy and accounting rule.

Example:

```text
The machine was invoiced in May but installed and available for use on 12 June. I will start depreciation in June.
```

## 7. Depreciation Engine

Selene must calculate depreciation schedules.

Common methods:

```text
straight-line
diminishing value / reducing balance
units of production
component depreciation
lease/right-of-use depreciation
tax depreciation separate from book depreciation
```

Depreciation inputs:

```text
cost
residual value
useful life
depreciation method
available-for-use date
disposal date
impairment
revaluation if applicable
component breakdown
usage units if units-of-production
```

### 7.1 Book vs tax depreciation

Selene must separate:

```text
book depreciation
tax depreciation / capital allowance
management depreciation
asset lifecycle estimate
```

Document 16 / Tax Optimize owns tax depreciation optimization. This engine prepares accounting treatment and evidence.

Example:

```text
Book depreciation is straight-line over the policy useful life. Tax depreciation may allow accelerated deduction, so I will route a capital allowance opportunity to Document 16.
```

## 8. Component Accounting

Some assets have major components with different useful lives.

Examples:

```text
building structure
roof
lift/elevator
air conditioning system
aircraft engine
vehicle body vs specialized equipment
manufacturing machine components
```

Selene should detect component candidates when:

```text
asset is high value
invoice breaks out components
maintenance schedule separates components
components have materially different useful lives
industry policy requires componentization
```

Selene says:

```text
This building purchase includes roof and lift components that may have different useful lives. I recommend component accounting review.
```

Humans approve material componentization. Selene prepares the evidence and schedule.

## 9. Repairs Vs Capital Improvements

Selene must distinguish:

```text
repair / maintenance expense
capital improvement
replacement component
new asset
capital work in progress
```

### 9.1 Repair / maintenance

Usually restores asset to normal operating condition.

Examples:

```text
fix broken part
routine service
oil change
replace worn tire
patch leak
minor repainting
```

### 9.2 Capital improvement

Usually improves capacity, extends useful life, enhances performance, or changes function.

Examples:

```text
building extension
major engine upgrade
machine capacity upgrade
solar installation
vehicle conversion
new production line enhancement
structural renovation
```

Selene asks:

```text
Did this cost restore the asset to normal condition, or did it extend life, increase capacity, or improve performance?
```

If routine:

```text
expense automatically under policy
```

If material or judgmental:

```text
capitalization review required
```

## 10. Capital Work In Progress

Some projects are built over time.

Examples:

```text
building construction
factory fit-out
software development project
major machinery installation
land development
vehicle conversion
```

Selene tracks:

```text
project_id
approved budget
supplier invoices
internal costs if capitalizable
status
costs accumulated
available-for-use date
transfer to asset
depreciation start
audit_ref
```

State:

```text
Planning
Approved
InProgress
CostsAccumulating
ReadyForCapitalization
TransferredToAsset
Closed
Cancelled
Impaired
```

Selene says:

```text
This fit-out is still in progress. I will hold costs in capital work in progress until the site is available for use.
```

## 11. Leased Assets And Right-Of-Use Assets

Leases need separate handling.

Selene must detect potential leases:

```text
rental agreement
office lease
vehicle lease
equipment lease
machine hire
software/hardware bundle with embedded lease
long-term service contract with identified asset
```

Selene checks:

```text
lease term
identified asset
right to control use
payment schedule
extension options
termination options
low-value exemption
short-term exemption
discount rate
commencement date
```

Selene outputs:

```text
right-of-use asset candidate
lease liability candidate
lease exemption candidate
lease accounting review
```

Selene says:

```text
This equipment rental may contain a lease. I will prepare a right-of-use asset review.
```

## 12. Intangible Assets And Amortization

Selene must support intangible asset candidates.

Examples:

```text
software licence
purchased software
patent
trademark
customer list
development cost
website/platform development
IP rights
franchise rights
```

Selene must ask:

```text
Was this purchased or internally developed?
Is it research or development?
Is technical feasibility established?
Is commercial use/sale intended?
Can costs be measured reliably?
Is future benefit probable?
What is useful life?
Finite or indefinite life?
```

Outputs:

```text
expense
intangible asset candidate
development capitalization candidate
amortization schedule
impairment review
tax optimization opportunity
```

Selene must not capitalize research or development costs without source-backed policy and evidence.

## 13. Impairment Indicator Detection

Selene must monitor impairment indicators.

Impairment triggers:

```text
asset damaged
asset idle
asset underused
market value dropped
business unit loss-making
technology obsolete
asset no longer used
major customer lost
regulatory change
maintenance cost excessive
cash-generating unit underperforming
sale expected below carrying amount
```

Selene says:

```text
This machine is idle, repair costs are rising, and output has fallen. I recommend impairment review.
```

Selene does not calculate final impairment alone unless approved policy supports a routine review. Accounting, adviser, or authorized human approval controls material impairment.

## 14. Revaluation And Fair Value Review

If the company uses revaluation/fair value policies for certain assets, Selene must support evidence.

Examples:

```text
land
buildings
investment property
certain financial or revalued assets
```

Selene tracks:

```text
valuation date
valuer
valuation method
market evidence
carrying amount
revaluation surplus/deficit
impairment interaction
audit_ref
```

Selene must not invent fair value.

Example:

```text
The insured/replacement value differs materially from book value. I recommend valuation review.
```

## 15. Asset Disposal Accounting

Assets may be:

```text
sold
scrapped
stolen
lost
traded in
donated
written off
destroyed
insurance total loss
transferred between entities
```

Selene calculates:

```text
original cost
accumulated depreciation
carrying amount
sale proceeds
disposal costs
insurance proceeds
gain/loss on disposal
tax review needed
```

Basic disposal logic:

```text
Remove asset cost.
Remove accumulated depreciation.
Recognize proceeds.
Recognize gain or loss.
Route tax effects to Tax/Tax Optimize.
```

Selene says:

```text
The vehicle sale proceeds are above carrying amount. I have prepared a gain on disposal for review.
```

Accounting posts final journal. Tax reviews capital allowance, balancing adjustment, or capital gain effects where applicable.

## 16. Insurance Recoveries And Claims

If an asset is damaged or destroyed, Insurance Engine owns claim lifecycle.

Asset Accounting handles accounting treatment.

Events:

```text
repair claim
partial loss
total loss
replacement asset
insurance proceeds
uninsured loss
excess/deductible
```

Selene prepares:

```text
repair expense / capitalization review
insurance receivable
claim recovery
asset write-off if total loss
deductible expense
replacement asset capitalization
```

Selene says:

```text
The van was written off by the insurer. I will prepare asset disposal accounting, insurance recovery evidence, and tax review routing.
```

## 17. Claimable Expense Rules

"Claimable" must be separated into:

```text
business reimbursement claimable
accounting expense claimable
tax deductible claimable
input tax / GST / VAT claimable
non-claimable / blocked
partially claimable
needs evidence
```

### 17.1 Expense claim inputs

```text
receipt
supplier invoice
employee expense claim
credit card transaction
bank transaction
mileage log
travel booking
meal receipt
fleet/fuel record
home-office record
subscription invoice
training invoice
insurance invoice
```

### 17.2 Selene checks

```text
business purpose
date
amount
supplier
tax invoice evidence
employee/user
department/cost center
policy limit
receipt attached
private-use portion
client/project link
asset vs expense
prepaid vs current period
tax claimability
```

### 17.3 Outcomes

```text
claimable business expense
partially claimable
reimbursement allowed
tax review required
capitalization required
prepayment required
blocked / non-business
missing evidence
```

Selene asks:

```text
Was this meal for a client meeting, employee travel, or personal?
```

Humans answer the business question. Selene proposes accounting treatment and routes tax claimability to the correct owner.

## 18. Personal / Private / Mixed-Use Expenses

Selene must detect non-business or mixed-use costs.

Signals:

```text
weekend luxury purchase
personal merchant category
owner personal spending
family travel
vehicle private use
home utilities
meal without business purpose
subscription unrelated to business
missing receipt
```

Selene should clarify rather than accuse:

```text
This looks personal or mixed-use. Should any portion be treated as business-related?
```

If mixed-use:

```text
business percentage
private percentage
evidence
approval
tax review
```

Tax Engine handles tax claimability. Accounting handles expense split. Access handles approval if reimbursement is requested.

## 19. Prepayments Vs Expenses

Selene detects multi-period expenses.

Examples:

```text
annual insurance
annual software subscription
rent paid in advance
maintenance contract
licence fee
warranty/service plan
```

Selene checks:

```text
coverage period
invoice period
amount
materiality threshold
company policy
period end
```

Outcome:

```text
expense immediately
prepay and release monthly
split current/future periods
```

Selene says:

```text
This software subscription covers 12 months. I recommend treating it as a prepayment and releasing it monthly.
```

## 20. Accruals Related To Assets And Expenses

Selene detects incurred costs not invoiced.

Examples:

```text
maintenance work completed before invoice
utilities incurred
asset installation completed
consultant milestone delivered
repair completed
rent period passed
interest incurred
```

Selene proposes:

```text
accrual amount
period
supplier
evidence
reversal schedule
review owner
```

Accounting posts. Period Close uses the proposal and evidence.

## 21. Asset And Expense Evidence Fabric

Every asset/expense treatment needs evidence.

Evidence types:

```text
supplier invoice
purchase order
receiving acceptance
photo
serial/VIN
contract
warranty
delivery note
installation report
service report
business purpose note
policy reference
approval
tax invoice
insurance document
valuation
disposal document
sale agreement
bank/payment proof
```

Selene must attach evidence to:

```text
asset accounting record
depreciation record
expense claim
capitalization decision
repair-vs-capital decision
prepayment schedule
accrual proposal
disposal record
tax review packet
audit pack
```

No evidence, no confidence.

No confidence, no automatic final treatment.

## 22. Automation And Exception-Only Review

Selene auto-handles:

```text
routine expenses under policy
low-value asset expensing under threshold
standard depreciation schedules
routine prepayment schedules
routine repairs
expense receipt extraction
business-purpose prompt
asset draft creation from invoice/receiving
depreciation run proposal
evidence pack assembly
```

Selene escalates:

```text
material capitalization judgment
repair vs improvement uncertainty
asset useful life exception
impairment indicator
asset disposal
insurance total loss
intangible development capitalization
lease/right-of-use review
mixed-use/private-use item
missing evidence above threshold
tax-sensitive item
board/capex threshold
```

Rule:

```text
Routine = Selene handles.
Judgment = Selene explains and routes.
Protected = authority approves.
Everything = audited.
```

## 23. PH1.D / GPT-5.5 Role

GPT-5.5 should be used for explanation, drafting, and classification assistance.

### 23.1 GPT-5.5 may help

```text
read messy invoice descriptions
summarize asset evidence
draft business-purpose prompts
explain repair vs capital treatment
explain depreciation schedule
draft disposal explanation
draft audit notes
summarize impairment indicators
translate accounting reasoning into plain English
prepare reviewer briefing
```

### 23.2 GPT-5.5 must not

```text
approve capitalization
post journal
set official useful life without policy
decide final tax claimability
ignore missing evidence
invent asset condition
invent valuation
approve impairment
approve disposal
override Access/Authority
```

GPT-5.5 explains. Selene deterministic engines classify and route. Accounting posts.

## 24. Human-Like Selene Interaction

### Asset classification

```text
This looks like equipment with useful life beyond one year and it exceeds your capitalization threshold. I will prepare an asset record for review.
```

### Repair vs improvement

```text
This repair appears to restore the machine to normal condition, so I will treat it as maintenance expense unless you confirm it increased capacity or extended useful life.
```

### Depreciation

```text
The vehicle is available for use from 1 June. I will start depreciation from June and link it to Fleet for operating costs.
```

### Missing evidence

```text
This expense may be claimable, but the receipt is missing. I will ask the employee to upload it before reimbursement.
```

### Disposal

```text
The asset was sold for more than its carrying amount. I have prepared the gain on disposal and routed tax review.
```

## 25. State Machines

### Asset Accounting State

```text
Candidate
EvidenceCollecting
ClassificationProposed
ReviewRequired
Capitalized
Expensed
Prepaid
Accrued
Depreciating
ImpairmentReview
Disposed
Archived
```

### Depreciation State

```text
NotStarted
ReadyToStart
Active
Paused
FullyDepreciated
Disposed
Adjusted
Archived
```

### Expense Claimability State

```text
Captured
EvidenceMissing
BusinessPurposeNeeded
Claimable
PartiallyClaimable
NonClaimable
TaxReviewRequired
Approved
Rejected
Archived
```

### Repair vs Capital State

```text
Captured
RepairLikely
CapitalLikely
MixedTreatment
ReviewRequired
Resolved
Archived
```

### Disposal State

```text
Proposed
EvidenceNeeded
PendingApproval
ReadyForAccounting
GainLossCalculated
PostedByAccounting
TaxReviewRouted
Closed
Archived
```

## 26. Reason Codes

```text
ASSET_CANDIDATE_DETECTED
CAPITALIZATION_THRESHOLD_MET
LOW_VALUE_ASSET_EXPENSED
BUSINESS_PURPOSE_REQUIRED
RECEIPT_MISSING
TAX_INVOICE_MISSING
CAPITALIZATION_REVIEW_REQUIRED
REPAIR_EXPENSE_CLASSIFIED
CAPITAL_IMPROVEMENT_CLASSIFIED
PREPAYMENT_DETECTED
ACCRUAL_RECOMMENDED
DEPRECIATION_READY
DEPRECIATION_STARTED
USEFUL_LIFE_REVIEW_REQUIRED
RESIDUAL_VALUE_REVIEW_REQUIRED
COMPONENT_ACCOUNTING_REVIEW_REQUIRED
LEASE_REVIEW_REQUIRED
INTANGIBLE_ASSET_REVIEW_REQUIRED
IMPAIRMENT_INDICATOR_DETECTED
ASSET_DISPOSAL_DETECTED
GAIN_LOSS_ON_DISPOSAL_CALCULATED
INSURANCE_RECOVERY_LINKED
TAX_OPTIMIZATION_OPPORTUNITY_ROUTED
```

## 27. Required Simulations

```text
detect asset from supplier invoice
classify low-value asset as expense
capitalization threshold met
asset received and capitalized
asset invoice received but not yet available for use
depreciation schedule created
depreciation run monthly
repair classified as maintenance
capital improvement classified
mixed repair/capital review
prepayment detected
accrual detected
lease candidate detected
intangible development cost review
component accounting candidate
impairment indicator detected
asset disposal sale gain
asset disposal sale loss
insurance total loss
expense claim missing receipt
expense claim business purpose required
mixed-use vehicle expense
tax optimization opportunity routed to Document 16
```

## 28. Integration Map

```text
PH1.ASSET_ACCOUNTING / PH1.DEPRECIATION / PH1.EXPENSE_RULES
↔ PH1.ACCOUNTING / GL
↔ PH1.ASSET
↔ PH1.PROCUREMENT
↔ PH1.PROC.RECEIVE
↔ PH1.CREDITORS / AP
↔ PH1.BANKREC / TREASURY
↔ PH1.TAX / TAX_COMPLIANCE
↔ PH1.TAX.OPTIMIZE
↔ PH1.CASHFLOW
↔ PH1.BUDGET / SPEND_GOV
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.MAINTENANCE
↔ PH1.PAYROLL / HR
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 29. Required Logical Packets

```text
AssetAccountingCandidatePacket
CapitalizationDecisionPacket
FixedAssetAccountingPacket
DepreciationSchedulePacket
DepreciationRunPacket
RepairVsCapitalPacket
ClaimableExpensePacket
BusinessPurposeEvidencePacket
MixedUseExpensePacket
PrepaymentPacket
AccrualCandidatePacket
LeaseAccountingReviewPacket
IntangibleAssetReviewPacket
ComponentAccountingReviewPacket
ImpairmentIndicatorPacket
AssetDisposalAccountingPacket
GainLossOnDisposalPacket
InsuranceRecoveryAccountingPacket
TaxOptimizationOpportunityPacket
AuditEvidencePacket
```

Logical only. Codex maps later. This document does not create packet structs.

## 30. What Codex Must Not Do

```text
Do not merge Asset Accounting with real-world Asset Engine.
Do not let Asset Accounting own physical custody or maintenance.
Do not let Asset Accounting post ledger directly.
Do not let GPT-5.5 decide final capitalization.
Do not hardcode useful lives without policy/rule packs.
Do not treat tax depreciation as book depreciation.
Do not decide final tax claimability without Tax/Tax Optimize.
Do not start depreciation before available-for-use date.
Do not capitalize without evidence.
Do not expense capital improvements without review.
Do not ignore impairment indicators.
Do not implement from this document alone.
```

## 31. Final Architecture Sentence

Selene Asset Accounting, Depreciation + Claimable Expense Rules Engine is the accounting classification and evidence brain that automatically identifies asset candidates, expenses, prepayments, accruals, repairs, capital improvements, leases, intangibles, depreciation schedules, impairment indicators, disposals, insurance recoveries, and claimable expense evidence, while keeping physical asset lifecycle in PH1.ASSET, tax treatment in PH1.TAX and PH1.TAX.OPTIMIZE, final ledger posting in Accounting, and using GPT-5.5 to explain judgments humanly without allowing probabilistic output to create accounting truth.

## 32. Simple Version

```text
Selene sees the invoice or expense.
Selene decides if it looks like asset, expense, repair, prepayment, or accrual.
Selene asks simple human questions only when needed.
Selene creates asset accounting evidence.
Selene calculates depreciation schedules.
Selene flags impairment and disposals.
Selene separates accounting treatment from tax treatment.
Selene routes tax-saving opportunities to Document 16.
Humans approve only judgment-heavy exceptions.
Accounting posts final books.
Everything is audited.
```
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_17_ASSET_ACCOUNTING_DEPRECIATION_CLAIMABLE_EXPENSE_RULES_MASTER_DESIGN.md -->

---

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_18_DEBT_LOANS_BORROWING_COSTS_COVENANTS_SECURITY_ACCOUNTING_MASTER_DESIGN.md -->
# Finance / Accounting Document 18 — Selene Debt, Loans, Borrowing Costs, Covenants + Security Accounting Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
18

ENGINE:
PH1.DEBT_ACCOUNTING / PH1.LOAN / PH1.BORROWING_COSTS / PH1.COVENANTS

FULL NAME:
Selene Debt, Loan, Borrowing Cost, Interest, Covenant, Security, Refinancing, Lease Liability, and Debt Reporting Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition. No runtime code, schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.DEBT_ACCOUNTING / PH1.LOAN / PH1.BORROWING_COSTS / PH1.COVENANTS. Repo-truth activation, simulation mapping, owner mapping, tests, source pack governance, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Debt + Loan Accounting Engine owns the accounting and control treatment of company borrowings, loans, finance facilities, lease liabilities, secured debt, interest, borrowing costs, repayments, covenants, refinancing, guarantees, and debt-related reporting.

It answers:

```text
What debt does the company owe?
Who is the lender?
What are the repayment terms?
What interest rate applies?
Is the interest fixed, floating, variable, or indexed?
Is the loan secured against assets?
Which assets are pledged?
Are covenants being met?
When is the next repayment?
How much interest has accrued?
Should borrowing costs be expensed or capitalized?
Is this debt current or non-current?
Has the loan been modified, refinanced, waived, breached, or discharged?
What accounting entries are required?
What cashflow impact is coming?
```

This engine prevents debt from being treated as simple incoming cash. Debt is a future cash obligation with accounting, tax, legal, covenant, security, and reporting consequences.

## 2. Core Selene Law

```text
Selene must know every loan, repayment, interest charge, covenant, pledged asset, borrowing cost, refinancing event, and lender obligation before it becomes a cashflow or reporting surprise.

Routine repayment tracking and interest schedules should be automated.

Humans review new borrowing, modifications, refinancing, covenant breaches, secured asset releases, tax-sensitive issues, and judgment-heavy classification matters.
```

Selene must reduce human work by:

```text
reading loan agreements
extracting repayment schedules
tracking interest accruals
reminding before repayments
checking covenant risk
linking security to assets
checking insurance requirements
detecting refinancing opportunities
detecting borrowing cost capitalization candidates
preparing accounting handoffs
preparing board/lender reporting packs
updating cashflow forecasts
```

Selene should not ask humans each month whether a known loan repayment exists. Selene should know the schedule, check cashflow, verify cleared payment through BankRec, and prepare the accounting handoff.

## 3. Engine Boundary

### 3.1 PH1.DEBT_ACCOUNTING owns

```text
loan register
debt instrument register
lender profile reference
loan agreement extraction
repayment schedule
interest calculation support
effective interest / amortized cost support where applicable
current vs non-current debt classification support
borrowing cost treatment
capitalized borrowing cost candidates
debt issuance cost tracking
loan modification/refinancing accounting support
covenant tracking
security/collateral register link
guarantee tracking
debt maturity analysis
debt cashflow forecast handoff
debt reporting pack
lender reporting pack
board debt pack
debt audit evidence
```

### 3.2 PH1.DEBT_ACCOUNTING does not own

```text
bank payment execution
final ledger posting
legal contract drafting
asset physical custody
insurance policy administration
board approval
shareholder approval
tax law strategy
cashflow ownership
lender negotiation authority
```

### 3.3 Correct owner split

```text
Debt Accounting = loan/debt accounting treatment, schedules, interest, covenants, borrowing cost evidence.
Accounting / GL = final journal posting.
BankRec / Treasury = bank proof of drawdowns and repayments.
Cashflow = liquidity impact and payment planning.
Asset Engine = real-world asset lifecycle.
Asset Accounting = asset capitalization/depreciation.
Insurance = insured assets and policy compliance.
Legal = loan/security agreements and guarantees.
Board / Shareholder = major borrowings where governance requires.
Access / Authority = approvals for protected debt actions.
Tax / Tax Optimize = tax deductibility, thin capitalization, withholding, interest limitation, structuring.
Audit = proof.
```

Debt Accounting is not the bank, not the lawyer, and not the board. It is the accounting control layer for debt obligations and debt evidence.

## 4. Debt Types Selene Must Support

Selene must support many debt and financing types.

```text
term loans
bank overdrafts
revolving credit facilities
working capital loans
asset finance
equipment loans
vehicle loans
mortgages
property loans
construction loans
shareholder loans
director loans
intercompany loans
convertible debt
notes / bonds
merchant cash advances
invoice financing
factoring / receivables finance
supplier finance
lease liabilities
hire purchase
government loans
tax payment plans
credit cards
guarantees
letters of credit
standby facilities
```

Some financing arrangements are debt-like even if they do not use ordinary loan labels.

Examples:

```text
Lease may create lease liability.
Supplier financing may act like debt.
Convertible note may include debt/equity classification questions.
Factoring may be sale of receivable or secured borrowing depending terms.
```

Selene must detect and route these arrangements for classification review.

## 5. Debt Master Record

Every debt instrument needs a complete record.

```text
debt_id
legal_entity_id
lender_id
facility_name
debt_type
agreement_date
drawdown_date
maturity_date
currency
original_principal
current_principal
available_limit
drawn_amount
undrawn_amount
interest_type
interest_rate
benchmark_rate
margin
fixed_rate_period
repayment_frequency
repayment_amount
repayment_schedule
fees
debt_issue_costs
security_collateral
guarantees
covenants
insurance_requirements
tax_review_required
classification_current_non_current
accounting_measurement_basis
effective_interest_rate_if_applicable
borrowing_cost_capitalization_flag
linked_asset_id
linked_project_id
linked_bank_account
status
audit_ref
```

Debt states:

```text
Draft
PendingAgreement
Approved
Active
Drawn
PartiallyRepaid
FullyRepaid
Modified
Refinanced
Waived
CovenantBreach
DefaultRisk
Defaulted
Discharged
Archived
```

## 6. Loan Agreement Extraction

Selene should read loan agreements and extract key terms.

Sources:

```text
loan agreement PDF
facility letter
bank offer
mortgage document
asset finance contract
lease agreement
intercompany loan agreement
board resolution
lender statement
repayment schedule
bank statement
payment provider facility
```

Selene extracts:

```text
lender
borrower
principal
facility limit
drawdown date
maturity date
interest rate
benchmark rate
repayment schedule
fees
covenants
security
guarantees
events of default
insurance requirements
reporting requirements
prepayment penalties
late payment penalties
```

Selene says:

```text
I found an equipment loan with monthly repayments, a floating rate, and a covenant requiring the secured asset to remain insured. Please confirm before I activate the schedule.
```

GPT-5.5 may extract and summarize as a proposal. Deterministic owner rules and human confirmation control final schedules.

## 7. Drawdowns And Repayments

Selene tracks debt cash movement.

### 7.1 Drawdown

When loan funds arrive:

```text
BankRec detects receipt
-> Debt Engine matches facility/drawdown
-> Accounting handoff prepared
-> Cashflow updated
-> Budget/project/asset funding linked
```

Possible accounting handoff:

```text
Dr Bank
Cr Loan Liability
```

Accounting posts final entry.

### 7.2 Repayment

Repayments include:

```text
principal
interest
fees
penalties
FX difference
early repayment charge
```

Selene must split repayment correctly.

Example:

```text
Bank payment: total amount
Principal: principal portion
Interest: interest portion
Bank fee: fee portion
```

Accounting handoff:

```text
Dr Loan Liability principal
Dr Interest Expense / Borrowing Cost
Dr Bank Fees
Cr Bank
```

Selene says:

```text
The loan repayment cleared. I split it into principal, interest, and bank fee based on the lender schedule.
```

Debt Engine does not treat all repayments as expense.

## 8. Interest Calculation

Selene must calculate or validate interest schedules.

Interest types:

```text
fixed rate
floating rate
variable rate
benchmark + margin
interest-only
principal-and-interest
balloon repayment
capitalized interest
default interest
compound interest
step-up / step-down rate
```

Inputs:

```text
principal
rate
days in period
day count convention
payment date
compounding
benchmark changes
margin
fees
loan modifications
```

Selene compares:

```text
lender statement interest
calculated expected interest
bank payment split
accounting interest expense
```

If mismatch:

```text
The lender charged interest that differs from the expected amount based on the current rate and balance. I recommend review.
```

No interest rate, benchmark, or convention may be invented by GPT-5.5.

## 9. Amortized Cost And Effective Interest Method

For financial liabilities measured at amortized cost, Selene should support effective-interest-based schedules where the reporting framework requires it.

Selene tracks:

```text
gross proceeds
transaction costs
discount/premium
stated interest
effective interest rate
amortized cost
carrying amount
interest expense
cash interest paid
modification adjustments
```

Selene says:

```text
This loan includes upfront establishment fees. I will prepare an effective interest schedule instead of expensing the fee immediately, unless policy says otherwise.
```

Accounting approves final measurement and posting.

## 10. Borrowing Costs And Capitalization

Borrowing costs may be expensed or capitalized depending on whether they are directly attributable to a qualifying asset under the applicable rule pack and accounting policy.

### 10.1 Borrowing cost types

```text
interest on borrowings
finance charges
loan establishment fees if applicable
commitment fees if applicable
amortization of debt issue costs
exchange differences to extent treated as borrowing cost under rule
lease finance charges where applicable
```

### 10.2 Capitalization candidates

Selene detects:

```text
construction project
building development
major manufacturing plant
qualifying asset under construction
asset takes substantial period to get ready
specific borrowing linked to asset
general borrowing used for qualifying asset
```

### 10.3 Selene workflow

```text
borrowing cost incurred
-> linked project/asset checked
-> qualifying asset test
-> capitalization period checked
-> directly attributable amount calculated
-> capitalization proposal created
-> Accounting/authority review
```

Selene says:

```text
Interest on this construction loan may be capitalizable while the building is being prepared for use. I will prepare a borrowing cost capitalization schedule.
```

If asset is already ready for use:

```text
The asset is available for use, so further interest should be expensed unless policy says otherwise.
```

## 11. Current Vs Non-Current Debt Classification

Selene must support balance sheet classification.

Inputs:

```text
repayment due dates
maturity date
covenant breach
waiver status
refinancing agreement
right to defer settlement
reporting date
```

Outputs:

```text
current liability portion
non-current liability portion
reclassification required
covenant breach review
waiver evidence required
```

Selene says:

```text
The portion of this loan due within the next reporting cycle should be classified as current.
```

If covenant breached:

```text
This covenant breach may require reclassification unless valid waiver evidence exists by the required date. I will route review.
```

## 12. Covenants

Selene must track loan covenants.

Covenant types:

```text
debt service coverage ratio
interest cover
current ratio
quick ratio
loan-to-value ratio
minimum cash balance
maximum debt-to-EBITDA
minimum net worth
reporting deadline
asset insurance requirement
negative pledge
restricted payments/dividends
capex limits
change of control
```

### 12.1 Covenant record

```text
covenant_id
debt_id
covenant_type
test_frequency
threshold
calculation_method
source_data
next_test_date
current_result
headroom
risk_status
waiver_status
breach_status
audit_ref
```

### 12.2 Covenant risk states

```text
Healthy
Watch
AtRisk
Breached
WaiverRequested
Waived
DefaultTriggered
Closed
```

Selene says:

```text
The loan-to-value covenant is close to breach because the secured property valuation declined. I recommend lender review before quarter-end.
```

## 13. Security, Collateral, And Guarantees

Selene must link debt to security.

Security types:

```text
property mortgage
vehicle security
equipment security
inventory security
receivables security
bank deposit security
personal guarantee
corporate guarantee
share pledge
floating charge
specific asset lien
```

Security record:

```text
security_id
debt_id
asset_id
security_type
secured_amount
lender
registration_number
registration_date
release_conditions
insurance_required
valuation_required
status
audit_ref
```

Selene checks:

```text
asset exists
asset insured
asset not sold without release
asset value supports covenant
security release required before disposal
guarantee exposure disclosed
```

Selene says:

```text
This vehicle is pledged as security. It cannot be sold without lender release.
```

## 14. Loan Modifications, Waivers, And Refinancing

Selene must detect and track debt changes.

Events:

```text
interest rate change
repayment schedule change
maturity extension
payment holiday
covenant waiver
principal forgiveness
new lender refinancing
partial settlement
debt-for-equity conversion
fee change
security change
```

Selene checks:

```text
material change
accounting modification treatment
cashflow impact
covenant impact
disclosure impact
legal agreement evidence
board approval if needed
tax review if forgiveness/related party/cross-border
```

Selene says:

```text
The bank extended maturity and changed repayment terms. I will prepare a modification review and update the cashflow forecast.
```

Refinancing may affect measurement, disclosures, covenants, costs, tax, and governance approvals.

## 15. Debt Issuance Costs And Fees

Selene must classify financing costs.

Examples:

```text
loan establishment fee
legal fees
broker fees
commitment fees
underwriting fees
valuation fees
security registration fees
early repayment penalty
line fees
unused facility fees
```

Treatment depends on:

```text
directly attributable to debt
facility type
amortized cost treatment
expense vs deferred cost
effective interest method
tax treatment
```

Selene says:

```text
This loan establishment fee appears directly related to the loan. I will include it in the debt accounting review rather than expensing automatically.
```

## 16. Lease Liability Integration

Lease liabilities may be part of the debt-like obligation picture.

Selene tracks:

```text
lease_id
lessor
commencement date
lease term
payment schedule
discount rate
lease liability
right-of-use asset
interest expense
principal repayment
remeasurement triggers
extension/termination options
short-term/low-value exemption
```

Lease Engine / Asset Accounting owns right-of-use asset evidence.

Debt Accounting tracks liability schedules and repayment/interest handoff.

Selene says:

```text
This office lease creates a lease liability and right-of-use asset candidate. I will route it through lease accounting review.
```

## 17. Related-Party And Intercompany Loans

Selene must handle related-party debt carefully.

Examples:

```text
shareholder loan to company
director loan
intercompany loan
group treasury loan
parent/subsidiary loan
loan to related party
```

Checks:

```text
written agreement
interest rate
repayment terms
arm's-length review
withholding tax
transfer pricing
board/shareholder approval
classification as debt vs equity
subordination
currency
forgiveness terms
```

Selene says:

```text
This shareholder loan has no repayment terms. I recommend legal/accounting review because classification and tax treatment may be affected.
```

## 18. Convertible Debt And Hybrid Instruments

Convertible notes and hybrid instruments require classification review.

Selene tracks:

```text
principal
interest
conversion price
conversion trigger
maturity
equity conversion terms
cash settlement option
issuer/holder options
embedded derivatives
debt/equity split review
fair value review
legal agreement
```

Selene says:

```text
This convertible note may include both liability and equity features. I will route it for financial instrument classification review.
```

No automatic final classification.

## 19. Debt And Cashflow Integration

Debt Engine feeds Cashflow.

Cashflow receives:

```text
repayment schedule
interest schedule
maturity dates
balloon payments
covenant cash requirements
refinancing deadlines
available undrawn facility
debt drawdown forecast
payment holidays
rate reset dates
```

Selene warns:

```text
A balloon payment is due in the forecast horizon. Current cashflow will not cover it unless refinancing or collections improve.
```

## 20. Debt And Budget Integration

Debt affects budgets.

Budget receives:

```text
interest expense budget
principal repayment schedule
facility fees
covenant compliance costs
loan establishment costs
capital project borrowing costs
```

Selene says:

```text
Interest expense is forecast to exceed budget due to rate increases. I recommend budget revision.
```

## 21. Debt And Tax Optimization Integration

Debt-related tax issues may include:

```text
interest deductibility
thin capitalization
withholding tax on interest
foreign lender treaty relief
debt forgiveness tax
capitalized interest
related-party interest arm's-length review
lease tax treatment
```

Document 16 owns legal optimization.

Debt Engine routes opportunities/risks:

```text
TaxOptimizationOpportunityPacket
WithholdingTaxReviewPacket
TransferPricingReviewPacket
CapitalAllowanceOpportunityPacket
BorrowingCostCapitalizationPacket
```

Selene says:

```text
This cross-border interest payment may require withholding tax and may qualify for treaty reduction. I will route it to Tax Optimization.
```

## 22. Debt Reporting And Disclosures

Debt reporting includes:

```text
loan register
debt maturity report
interest expense report
covenant report
security/collateral report
loan movement report
current/non-current debt report
borrowing cost capitalization report
debt cashflow forecast
lender reporting pack
board debt pack
```

Debt reporting must preserve maturity, liquidity, credit, interest-rate, covenant, security, and related risk evidence where applicable.

Selene should produce:

```text
Debt report is ready. Total debt, current portion, maturity profile, covenant headroom, secured assets, and repayment outlook are summarized for review.
```

## 23. Automation And Exception-Only Review

Selene auto-handles:

```text
loan schedule extraction proposal
routine repayment reminders
routine interest accrual proposals
routine principal/interest split
cashflow handoff
budget handoff
covenant calculation where source data is clean
debt report generation
insurance check on secured assets
payment confirmation matching
```

Selene escalates:

```text
new borrowing
loan modification
refinancing
covenant breach
waiver request
asset security release
related-party loan
convertible debt
withholding tax on interest
borrowing cost capitalization judgment
current/non-current reclassification due to breach
debt forgiveness
payment default
```

Rule:

```text
Routine = Selene handles.
Judgment = Selene prepares and routes.
Protected = authority approves.
Everything = audited.
```

## 24. PH1.D / GPT-5.5 Role

GPT-5.5 should assist with interpretation and explanation.

### 24.1 GPT-5.5 may help

```text
summarize loan agreements
extract covenant wording as proposal
explain repayment schedule
draft lender reporting narrative
explain covenant risk
summarize refinancing options
draft board debt summary
explain borrowing cost treatment
translate financial instrument language into plain English
prepare review questions for accountant/lawyer
```

### 24.2 GPT-5.5 must not

```text
approve borrowing
classify hybrid instruments finally
post journals
execute repayments
change loan schedules without evidence
declare covenant waived
release security
invent lender terms
decide final tax deductibility
override board/authority
```

GPT-5.5 can summarize and explain. It cannot become the lender, accountant, or approving authority.

## 25. Human-Like Selene Interaction

### Loan setup

```text
I read the loan agreement and found monthly repayments, a floating rate, and two covenants. Please confirm the extracted schedule before I activate monitoring.
```

### Repayment

```text
The loan repayment cleared. I split it into principal and interest and prepared the accounting handoff.
```

### Covenant risk

```text
The current ratio covenant is getting close to breach. I recommend reviewing supplier payment timing and cash collection before month-end.
```

### Borrowing cost

```text
Interest on this construction loan may be capitalized while the building is being prepared for use. I will prepare the capitalization schedule for review.
```

### Secured asset

```text
This vehicle is security for the loan. It should not be sold until lender release is obtained.
```

## 26. State Machines

### Debt State

```text
Draft
PendingAgreement
Approved
Active
Drawn
PartiallyRepaid
FullyRepaid
Modified
Refinanced
Waived
CovenantBreach
DefaultRisk
Defaulted
Discharged
Archived
```

### Repayment State

```text
Scheduled
DueSoon
PaymentRequested
Paid
Failed
PartiallyPaid
SplitConfirmed
AccountingHandoffReady
Closed
```

### Covenant State

```text
Healthy
Watch
AtRisk
Breached
WaiverRequested
Waived
DefaultTriggered
Closed
```

### Security State

```text
Proposed
Registered
Active
InsuranceCheckRequired
ValuationRequired
ReleaseRequested
Released
Disputed
Archived
```

### Borrowing Cost State

```text
ExpenseCandidate
CapitalizationCandidate
EvidenceNeeded
ReviewRequired
Capitalized
Expensed
Closed
```

### Loan Modification State

```text
Detected
EvidenceNeeded
AccountingReview
TaxReview
AuthorityReview
Approved
Rejected
Applied
Archived
```

## 27. Reason Codes

```text
DEBT_INSTRUMENT_DETECTED
LOAN_AGREEMENT_EXTRACTED
REPAYMENT_SCHEDULE_CREATED
DRAW_DOWN_MATCHED
REPAYMENT_DUE_SOON
REPAYMENT_CLEARED
PRINCIPAL_INTEREST_SPLIT_READY
INTEREST_VARIANCE_DETECTED
EFFECTIVE_INTEREST_SCHEDULE_REQUIRED
BORROWING_COST_CAPITALIZATION_CANDIDATE
BORROWING_COST_EXPENSED
CURRENT_PORTION_RECLASSIFICATION_REQUIRED
COVENANT_HEALTHY
COVENANT_WATCH
COVENANT_AT_RISK
COVENANT_BREACHED
WAIVER_REQUIRED
SECURED_ASSET_LINKED
SECURED_ASSET_INSURANCE_REQUIRED
SECURITY_RELEASE_REQUIRED
LOAN_MODIFICATION_DETECTED
REFINANCING_REVIEW_REQUIRED
RELATED_PARTY_LOAN_REVIEW_REQUIRED
CONVERTIBLE_DEBT_CLASSIFICATION_REVIEW
WITHHOLDING_TAX_REVIEW_ON_INTEREST
PILLAR_TWO_OR_TRANSFER_PRICING_DEBT_REVIEW
```

## 28. Required Simulations

```text
loan agreement extraction
new bank loan setup
loan drawdown matched to bank receipt
monthly repayment split principal/interest
interest variance detected
floating rate update
debt issue cost treatment
effective interest schedule created
borrowing cost capitalization candidate
borrowing cost expensed after asset ready
current/non-current debt split
covenant calculation healthy
covenant watch warning
covenant breach with waiver needed
secured asset linked
secured asset insurance missing
asset sale blocked by security
loan modification detected
loan refinancing review
related-party loan review
convertible debt classification review
cross-border interest withholding review
debt cashflow forecast warning
debt board pack generated
```

## 29. Integration Map

```text
PH1.DEBT_ACCOUNTING / PH1.LOAN / PH1.BORROWING_COSTS / PH1.COVENANTS
↔ PH1.ACCOUNTING / GL
↔ PH1.BANKREC / TREASURY
↔ PH1.CASHFLOW
↔ PH1.BUDGET / SPEND_GOV
↔ PH1.ASSET
↔ PH1.ASSET_ACCOUNTING
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.TAX / TAX_COMPLIANCE
↔ PH1.TAX.OPTIMIZE
↔ PH1.LEGAL / CONTRACTS
↔ PH1.BOARD
↔ PH1.SHAREHOLDER
↔ PH1.SUPPLIER_PAYMENT / BANKING_HANDOFF
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 30. Required Logical Packets

```text
DebtInstrumentPacket
LoanAgreementExtractionPacket
DebtSchedulePacket
RepaymentSchedulePacket
InterestCalculationPacket
EffectiveInterestSchedulePacket
BorrowingCostPacket
BorrowingCostCapitalizationPacket
DebtIssueCostPacket
DebtCurrentNonCurrentPacket
CovenantPacket
CovenantTestPacket
SecurityCollateralPacket
GuaranteePacket
LoanModificationPacket
RefinancingPacket
DebtCashflowHandoffPacket
DebtBudgetHandoffPacket
DebtTaxReviewPacket
DebtBoardPackPacket
DebtAuditEvidencePacket
```

Logical only. Codex maps later. This document does not create packet structs.

## 31. What Codex Must Not Do

```text
Do not let Debt Engine execute bank payments.
Do not let Debt Engine post ledger directly.
Do not let GPT-5.5 approve debt classification.
Do not hardcode IFRS/GAAP treatment without rule packs.
Do not classify convertible debt without review.
Do not ignore covenants.
Do not ignore secured assets.
Do not allow disposal of secured assets without release check.
Do not treat all repayments as expense.
Do not capitalize all interest automatically.
Do not ignore tax/withholding on cross-border interest.
Do not implement from this document alone.
```

## 32. Final Architecture Sentence

Selene Debt, Loan, Borrowing Costs, Covenants + Security Accounting Engine is the debt-control brain that reads loan and facility agreements, extracts repayment schedules, tracks principal, interest, fees, effective interest, borrowing cost capitalization candidates, current/non-current classification, covenants, waivers, secured assets, guarantees, leases, refinancing, modifications, related-party loans, and debt disclosures, while coordinating with Accounting, BankRec, Cashflow, Budget, Asset, Insurance, Tax, Legal, Board, Access, and Audit so Selene can manage debt proactively, explain it humanly, reduce surprises, and prevent loans from becoming silent cashflow traps.

## 33. Simple Version

```text
Selene reads the loan.
Selene tracks what is owed.
Selene knows repayment dates.
Selene splits principal and interest.
Selene watches covenants.
Selene links secured assets.
Selene checks borrowing costs.
Selene warns before default.
Selene routes tax and legal issues.
Accounting posts final books.
Humans approve only judgment-heavy or protected debt actions.
Everything is audited.
```
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_18_DEBT_LOANS_BORROWING_COSTS_COVENANTS_SECURITY_ACCOUNTING_MASTER_DESIGN.md -->
