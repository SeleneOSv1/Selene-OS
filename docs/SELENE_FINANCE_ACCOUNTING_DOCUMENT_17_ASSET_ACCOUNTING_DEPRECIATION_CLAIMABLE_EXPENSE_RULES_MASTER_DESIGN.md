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
