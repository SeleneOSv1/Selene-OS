# Selene Assets + Depreciation + Claimable Expense Rules Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / ASSETS + DEPRECIATION + CLAIMABLE EXPENSE RULES + ASSET ACCOUNTING HANDOFF

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's asset accounting, depreciation, capitalization, claimable expense, asset purchase, asset disposal, impairment/revaluation, private/business use, tax treatment, and asset-to-accounting handoff architecture. This document covers the finance/accounting/tax side of assets while preserving PH1.ASSET as the future standalone engine for real-world asset lifecycle management.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 5. It defines future asset accounting, depreciation, claimability, capitalization, asset register, disposal, impairment, revaluation, private/business use, insurance/fleet/property/debt boundaries, and accounting/tax handoff. It does not implement Assets, Accounting, Tax, Procurement, Receiving, AP, Banking, Fleet, Insurance, Debt/Treasury, Real Estate, Access, packets, migrations, tests, or runtime state.

Current repo truth does not prove complete runtime PH1.ASSET, Tax/Compliance country packs, Fleet, Insurance, Debt/Treasury, Real Estate, Procurement, Receiving, or full Finance/Accounting ownership. This document is future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Executive Target

Selene must not treat assets like ordinary expenses wearing a nice jacket.

A company asset may be:

```text
vehicle
building
land
machinery
tools
computers
furniture
warehouse equipment
plant
software
website/platform build
intellectual property
fleet vehicle
property improvement
leasehold improvement
security/collateral for debt
income-producing property
```

Old way:

```text
someone buys equipment
invoice goes to AP
accountant later asks what it was
asset register is updated late
depreciation is guessed
insurance is forgotten
tax treatment is messy
sale/disposal creates confusion
```

Selene way:

```text
asset need detected
→ budget/cashflow checked
→ lease/buy/finance scenario compared
→ PO created if approved
→ goods received and inspected
→ supplier invoice matched
→ asset created in asset register
→ accounting capitalization decided
→ tax/claimability checked
→ depreciation schedule created
→ insurance/fleet/property/debt links checked
→ ongoing asset life tracked by PH1.ASSET
→ disposal/revaluation/impairment handled with proof
```

Target:

```text
asset accounting from evidence
automatic asset register creation
capital vs expense classification
jurisdiction-aware depreciation
claimable expense treatment
business/private use split
asset lifecycle handoff
insurance and fleet awareness
loan/lease/debt awareness
real estate/rental/collateral awareness
disposal gain/loss handling
audit-backed asset truth
```

Tiny translation: Selene should know the difference between printer paper and a building.

## 2. Master Law

```text
No asset may be capitalized without source evidence.

No asset may be expensed or depreciated without correct owner classification.

No depreciation rule may be invented by GPT-5.5.

No tax claim may be made without jurisdiction/company rule evidence.

No asset disposal may occur without book value, sale proceeds, tax/accounting treatment, and audit.

No private-use portion may be treated as fully business without proof.

No operational asset truth belongs to Accounting alone.

PH1.ASSET owns real-world asset lifecycle.

Accounting owns book value, journals, depreciation posting, impairment/revaluation accounting, and gain/loss.

Tax/Compliance owns tax depreciation, claimability, capital allowance, GST/VAT, and country rules.

Finance owns asset strategy, ROI, budget, cashflow, lease/buy decision support.

Access/Governance owns who can buy, classify, revalue, dispose, write off, or export asset records.
```

## 3. Owner Split

### PH1.ASSET Owns Future Real-World Asset Lifecycle

```text
asset identity
asset register operational profile
serial number / VIN / title references
location
custodian
condition
maintenance status
inspection
repair history
warranty
usage
asset value evidence
asset lifecycle status
asset retirement/disposal readiness
```

PH1.ASSET is future standalone and must not be silently implemented from this document.

### Accounting Owns

```text
capitalization accounting
fixed asset account mapping
accumulated depreciation
depreciation journal
impairment journal
revaluation journal where policy allows
asset disposal journal
gain/loss on disposal
book value
asset financial statement presentation
ledger audit
```

### Tax / Compliance Owns

```text
tax depreciation / capital allowance
GST/VAT claimability
deductibility
business/private use treatment
country/region claim rules
effective-date rule packs
tax reporting evidence
tax adjustment on sale/disposal
```

### Finance / Budget / Cashflow Owns

```text
asset business case
ROI
lease vs buy analysis
cashflow impact
capex budget
minimum cash reserve impact
profit floor impact
funding recommendation
board/management approval route
```

### Procurement Owns

```text
purchase request
supplier selection
purchase order
approval workflow
purchase commitment
```

### PH1.PROC.RECEIVE Owns

```text
goods receipt
inspection
accepted asset quantity
damage/wrong item/short delivery evidence
supplier resolution
```

### AP Owns

```text
supplier invoice
bill approval
payment terms
supplier credit/refund
AP payment readiness
```

### Banking Owns

```text
payment execution
payment confirmation
bank reconciliation
loan/lease payment proof where applicable
```

### PH1.FLEET Owns Vehicle Operations

```text
vehicle assignment
driver/custodian
odometer
fuel/charging
service
repairs
registration
roadworthiness
accidents
fines/tolls
```

### PH1.INSURANCE Owns Asset Cover And Claims

```text
insurance policy
renewal
coverage
claim
excess
certificate
coverage gap
```

### Debt / Treasury Owns Future Loan And Lease Obligations

```text
loan agreement
lease liability
principal/interest schedule
covenants
security/collateral
repayment schedule
```

### Real Estate Future Engine Owns Property-Specific Lifecycle

```text
property profile
tenant lease
rental income
property valuation
land/building split
collateral
property insurance
rates/taxes
property maintenance
```

### Access / Governance Owns

```text
who can approve asset purchase
who can classify asset
who can approve capitalization
who can approve disposal
who can approve impairment/write-off
who can approve revaluation
who can view asset values
who can export asset records
who can approve lease/buy strategy
```

### PH1.D / GPT-5.5 May Assist

```text
explain asset treatment in plain language
suggest capital vs expense candidates
summarize invoice/product description
draft asset business case
compare lease/buy narrative
summarize depreciation options as candidates
draft board summary
```

PH1.D / GPT-5.5 must not:

```text
decide final capitalization
invent depreciation rule
invent tax law
approve purchase
approve disposal
post depreciation
write off asset
revalue asset
execute payment
```

### PH1.WRITE Owns Wording

```text
asset explanations
purchase approval summaries
lease/buy explanations
depreciation explanations
claimability explanations
disposal summaries
board-ready asset wording
```

## 4. Asset Scope

Document 9 must support:

```text
fixed assets
low-value assets
pooled assets
leased assets
right-of-use assets
vehicles
fleet assets
plant and equipment
tools
computers and devices
furniture and fittings
buildings
land
property improvements
leasehold improvements
software and digital assets
intangible assets where policy allows
capital works
assets under construction
income-producing assets
security/collateral assets
assets held for sale
disposed assets
fully depreciated assets still in use
```

Not all of these have the same accounting treatment. Selene must classify carefully.

## 5. Asset Acquisition Flow

Standard acquisition flow:

```text
asset need detected
→ business purpose captured
→ budget/capex check
→ cashflow/reserve check
→ lease/buy/finance scenario if material
→ approval route resolved
→ purchase order created if approved
→ supplier delivers
→ receiving/inspection confirms accepted asset
→ AP receives invoice
→ tax/claimability check
→ capitalization decision
→ asset register record created
→ depreciation schedule created
→ insurance/fleet/property/debt links checked
→ accounting posts asset/AP/payment entries
→ audit
```

Selene should say:

```text
This looks like an asset purchase, not an ordinary expense. I’ll check budget, tax treatment, depreciation, insurance, and whether it should be bought, leased, or financed before it is approved.
```

## 6. Asset Candidate Detection

Selene should detect possible assets from:

```text
purchase request
supplier invoice
card transaction
bank transaction
expense claim
contract
lease agreement
loan agreement
shipping/receiving record
product description
serial number/VIN
warranty document
insurance schedule
manual user request
```

Asset candidate signals:

```text
high value
long useful life
physical item
serial number
warranty
vehicle/VIN
machinery/equipment
building/property
software build
capital project
installation cost
improvement to existing asset
lease/finance language
```

Asset candidate packet:

```text
AssetCandidatePacket:
  candidate_id
  source_owner
  source_document_ref
  supplier_id
  description
  amount
  currency
  purchase_date
  asset_class_candidate
  useful_life_candidate
  capitalization_candidate
  tax_review_required
  approval_required
  confidence
  audit_ref
```

## 7. Capitalize Vs Expense

Selene must decide whether an item is:

```text
ordinary expense
inventory
fixed asset
low-value asset
pooled asset
capital improvement
repair/maintenance expense
prepayment
right-of-use asset
intangible asset
asset under construction
```

Final classification belongs to Accounting/Tax rules, not GPT-5.5.

Decision factors:

```text
amount
useful life
business use
asset class
company capitalization threshold
country/tax rule
whether item improves existing asset
whether item only repairs existing condition
whether item is held for resale
whether item is consumed quickly
whether item is part of inventory
whether item creates future economic benefit
```

Example:

```text
Forklift repair:
May be repairs expense.

New forklift:
Likely fixed asset.

Major upgrade that extends forklift life:
Possible capital improvement.

Spare part held in warehouse:
May be inventory or maintenance stock depending policy.
```

Selene says:

```text
This may be a capital asset because it will be used for more than one accounting period. I’ll route it for asset classification before Accounting posts it.
```

## 8. Asset Register

Asset register fields:

```text
asset_id
company_id
legal_entity_id
asset_name
asset_description
asset_class
asset_category
serial_number
VIN_or_unique_identifier
supplier_id
purchase_order_ref
invoice_ref
receipt_inspection_ref
purchase_date
placed_in_service_date
cost
tax_amount
claimable_tax_amount
currency
location_id
custodian_user_id
department_id
cost_center_id
project_id
business_use_percentage
private_use_percentage
ownership_status
funding_source
loan_or_lease_ref
insurance_policy_ref
warranty_ref
depreciation_schedule_ref
current_book_value
tax_book_value
status
audit_ref
```

Asset statuses:

```text
Candidate
PendingApproval
Ordered
Received
InspectionHold
Accepted
InService
UnderMaintenance
Idle
Transferred
Impaired
Revalued
HeldForSale
Disposed
WrittenOff
Archived
```

## 9. Placed-In-Service Rule

Depreciation generally begins when the asset is ready and available for use according to company/jurisdiction policy.

Selene must capture:

```text
purchase_date
received_date
inspection_acceptance_date
installation_date
placed_in_service_date
first_use_date
depreciation_start_date
```

Example:

```text
Machine purchased in May.
Delivered in June.
Installed in July.
Ready for use in July.
Depreciation start may be July depending policy.
```

Selene must not start depreciation from invoice date by default if the asset is not ready for use.

## 10. Depreciation

Selene must support multiple depreciation methods, but must not invent which applies.

Possible methods:

```text
straight_line
declining_balance
double_declining_balance
units_of_production
immediate_write_off_where_allowed
low_value_pool
asset_pool
capital_works
software_amortization
right_of_use_asset_amortization
manual_policy_defined_method
```

Depreciation schedule fields:

```text
depreciation_schedule_id
asset_id
method
useful_life
residual_value
depreciable_amount
depreciation_start_date
depreciation_end_date
frequency
accounting_periods
book_depreciation_amount
tax_depreciation_amount
accumulated_depreciation
remaining_book_value
rule_pack_ref
approved_by
audit_ref
```

Depreciation posting:

```text
Debit: Depreciation Expense
Credit: Accumulated Depreciation
```

Selene says:

```text
I’ve prepared the depreciation schedule using the approved asset rule pack. Accounting will post depreciation each period once the asset is marked in service.
```

## 11. Book Depreciation Vs Tax Depreciation

Book and tax treatment may differ.

```text
Book depreciation = accounting financial statement treatment.
Tax depreciation / capital allowance = tax treatment.
```

Fields:

```text
book_method
book_useful_life
book_depreciation
tax_method
tax_effective_life
tax_deduction
temporary_difference
deferred_tax_ref_if_applicable
tax_rule_ref
```

Tax/Compliance owns tax treatment.

Accounting owns book depreciation.

Selene must not force them to be the same.

## 12. Business / Private Use Split

Some assets have mixed use.

Examples:

```text
vehicle
phone
laptop
home office equipment
internet/phone equipment
property
tools
travel equipment
```

Required fields:

```text
business_use_percentage
private_use_percentage
basis_of_calculation
logbook_ref
odometer_ref
usage_record_ref
approved_by
effective_from
effective_to
tax_rule_ref
audit_ref
```

Example:

```text
Vehicle business use: 80%
Private use: 20%

Selene must split claimability and tax treatment according to approved rules.
```

Selene says:

```text
This vehicle has private use, so I’ll apply the approved business-use percentage before calculating claimable expenses and depreciation.
```

## 13. Claimable Expense Rules

Selene must support country/region/company claimability rules.

Expense/asset claimability categories:

```text
fully_claimable
partially_claimable
not_claimable
capitalized_not_immediately_deductible
deductible_over_time
requires_tax_review
requires_business_use_split
requires_receipt/tax_invoice
requires_policy_approval
```

Claimability applies to:

```text
asset purchase
repairs
maintenance
fuel
insurance
registration
property rates
software
tools
training equipment
vehicle expenses
home office equipment
travel equipment
building improvements
loan interest
lease payments
```

Claimability packet:

```text
ClaimabilityAssessmentPacket:
  source_document_ref
  asset_id optional
  expense_category
  country
  region
  tax_rule_ref
  business_use_percentage
  claimable_amount
  non_claimable_amount
  requires_review
  audit_ref
```

Selene must explain:

```text
This cost appears partly claimable. I’ll apply the business-use percentage and route the tax treatment for review before it is finalized.
```

## 14. Repairs Vs Capital Improvements

Selene must distinguish repair from improvement.

Repair:

```text
restores asset to working condition
ordinary maintenance
does not materially extend useful life
usually expense, subject to rules
```

Capital improvement:

```text
extends useful life
increases capacity
improves performance
adds new capability
may be capitalized
```

Example:

```text
Replace worn tyre = repair/maintenance.
Upgrade delivery van refrigeration system = possible capital improvement.
```

Selene says:

```text
This looks more like an improvement than a repair because it increases the asset’s capacity. I’ll route it for capitalization review.
```

## 15. Asset Maintenance And Repairs Boundary

PH1.ASSET / PH1.FLEET may own maintenance events.

Document 9 owns accounting/tax treatment.

Flow:

```text
maintenance event
→ PH1.ASSET/FLEET records asset condition and service
→ AP receives supplier bill
→ Accounting/Tax classify repair vs improvement
→ expense/capitalization posted
→ asset lifecycle updated
```

Maintenance costs may be:

```text
ordinary expense
capital improvement
warranty claim
insurance claim
supplier dispute
personal/private use split
```

## 16. Asset Insurance Boundary

PH1.INSURANCE owns policy and claims.

Document 9 needs asset links.

Asset insurance fields:

```text
insurance_policy_ref
coverage_amount
insured_value
excess/deductible
coverage_start
coverage_end
claim_refs
coverage_gap_status
```

Example:

```text
New delivery van added to asset register.
Selene checks whether Fleet/Insurance has coverage before it is used.
```

Selene says:

```text
This vehicle is now an active asset, but I do not see insurance coverage linked yet. I’ll route that to Insurance before it is used.
```

## 17. Vehicle / Fleet Assets

Vehicles are assets, but operations belong to PH1.FLEET.

Vehicle-specific fields:

```text
VIN
registration
make_model
year
odometer
fuel_type
assigned_driver
fleet_id
insurance_ref
loan_or_lease_ref
business_use_percentage
service_schedule_ref
```

Document 9 handles:

```text
capitalization
depreciation
lease/right-of-use accounting
loan liability handoff
repair vs improvement classification
gain/loss on sale
tax claimability
```

PH1.FLEET handles:

```text
driver
fuel
service
odometer
accident
fines
tolls
maintenance
roadworthy
usage
```

## 18. Buildings, Land, Property, And Real Estate Boundary

Buildings/property need special treatment.

Document 9 handles general asset accounting.

Future Real Estate design handles full real estate/property lifecycle.

Important distinctions:

```text
land
building
building improvements
investment property
owner-occupied property
rental property
property held for sale
property used as collateral
```

Land may not depreciate under many accounting/tax frameworks, while buildings/improvements may have depreciation/capital works treatment depending jurisdiction and policy. Selene must not assume.

Property fields:

```text
property_id
land_component_value
building_component_value
improvements_value
purchase_date
title_ref
valuation_ref
rental_income_ref
loan_security_ref
insurance_ref
rates_tax_ref
depreciation_or_capital_works_ref
```

Selene says:

```text
This property purchase needs land/building split, insurance, title evidence, loan/security review, and tax treatment before Accounting finalizes the asset record.
```

Rental income belongs to AR / Real Estate.

Collateral/security belongs to Debt / Legal / Real Estate.

## 19. Asset Appreciation, Revaluation, And Impairment

Some assets may appreciate or lose value.

Examples:

```text
building appreciation
land appreciation
property revaluation
machine impairment
vehicle value drop
damaged equipment
obsolete technology
inventory/asset write-down
```

Selene must distinguish:

```text
market value
book value
tax value
insured value
collateral value
recoverable amount
sale value
```

Revaluation/impairment requires:

```text
accounting policy
valuation evidence
authority
audit
tax review where needed
board/management approval where material
```

Accounting entries may include:

```text
impairment loss
revaluation surplus
revaluation decrease
write-down
gain/loss on disposal
```

Selene must not automatically increase book value just because market value appears higher.

Selene says:

```text
The property valuation has increased, but Accounting needs to confirm whether the company policy allows revaluation before any book value change is posted.
```

## 20. Asset Disposal / Sale / Write-Off

Asset disposal events:

```text
sale
scrap
trade-in
write-off
theft
loss
insurance claim
donation
transfer
destruction
retirement from use
```

Disposal flow:

```text
disposal requested/detected
→ asset ownership and status checked
→ book value calculated
→ depreciation updated to disposal date
→ sale proceeds or recovery value captured
→ tax/claimability reviewed
→ approval if required
→ accounting gain/loss posted
→ asset status updated
→ insurance/legal/debt/collateral checked
→ audit
```

Disposal fields:

```text
disposal_id
asset_id
disposal_type
disposal_date
sale_proceeds
currency
book_value_at_disposal
accumulated_depreciation
gain_or_loss
buyer_or_counterparty
approval_ref
tax_treatment_ref
insurance_claim_ref
debt_release_ref
audit_ref
```

Example journal:

```text
Debit: Cash / Receivable
Debit: Accumulated Depreciation
Credit: Fixed Asset Cost
Debit/Credit: Loss/Gain on Disposal
```

## 21. Asset Transfer Between Locations / Entities

Assets may move.

Transfer types:

```text
location transfer
department transfer
custodian transfer
project transfer
legal entity transfer
intercompany transfer
country transfer
```

PH1.ASSET owns physical transfer truth.

Accounting/Tax may need:

```text
cost center change
depreciation allocation change
intercompany sale/transfer
tax review
customs/import review
currency impact
```

Intercompany/cross-border transfer tax and duty logic is expanded in the related addendum.

## 22. Leased Assets And Right-Of-Use Assets

Leases need separate treatment.

Future Debt/Loans/Lease Liability design expands debt, loans, and lease liabilities. Document 9 handles asset-side accounting handoff.

Fields:

```text
lease_id
right_of_use_asset_id
leased_asset_description
lease_start
lease_end
payment_schedule_ref
discount_rate_ref
lease_liability_ref
amortization_schedule_ref
asset_location
business_use_percentage
audit_ref
```

Possible accounting:

```text
Right-of-use asset
Lease liability
Amortization / depreciation
Interest expense
Lease payments split principal/interest
```

Selene must not treat every lease as a simple rent expense. Jurisdiction/accounting policy decides.

## 23. Assets Under Construction / Work In Progress

Some assets are built over time.

Examples:

```text
building construction
warehouse fit-out
software development
machinery installation
custom equipment build
```

Asset under construction fields:

```text
auc_id
project_id
asset_class_candidate
capitalized_costs_to_date
supplier_refs
payroll_labor_refs
contractor_refs
completion_status
placed_in_service_date
audit_ref
```

Flow:

```text
costs accumulate
→ not depreciated until ready for use
→ transferred to fixed asset when placed in service
→ depreciation begins under approved rules
```

## 24. Software And Intangible Assets

Selene must support software/intangible asset candidates.

Examples:

```text
purchased software license
custom software build
website/platform build
trademark
patent
intellectual property
implementation costs
```

Possible treatment:

```text
expense
prepaid subscription
capitalized software
intangible asset
amortization
research and development expense
```

Tax/accounting rules decide.

Selene says:

```text
This software cost may be a subscription expense or a capitalized implementation asset. I’ll route it for accounting treatment review.
```

## 25. Low-Value Assets And Tools

Small assets may be expensed, pooled, or tracked operationally.

Examples:

```text
small tools
phones
minor equipment
office devices
kitchen equipment
uniform equipment
low-cost electronics
```

Selene must support:

```text
expense immediately where allowed
low-value asset pool
track operationally without capitalization
employee/custodian assignment
write-off after loss/damage
```

Policy decides.

## 26. Asset Funding Source

Assets can be funded by:

```text
cash purchase
supplier finance
bank loan
vehicle loan
lease
hire purchase
grant
shareholder contribution
intercompany loan
insurance replacement
trade-in
```

Funding source affects:

```text
cashflow
balance sheet
liability
interest
tax
ownership
collateral
approval route
```

Selene must link asset to funding source.

## 27. Lease Vs Buy Scenario Handoff

Budgeting addendum logic introduced lease/buy intelligence. Document 9 must provide asset-side facts.

Lease/buy comparison inputs:

```text
purchase price
useful life
residual value
depreciation
maintenance
insurance
tax treatment
business use
cash purchase impact
loan/lease terms
asset utilization
resale value
operational need
```

Selene says:

```text
From the asset side, buying gives ownership and residual value, while leasing reduces upfront cash but creates ongoing liability. Finance should compare this with cashflow and tax before approval.
```

## 28. Asset Income Generation

Some assets generate revenue.

Examples:

```text
rental property
equipment rental
vehicle hire
licensing intangible assets
leased equipment to customers
billable machinery use
```

Document 9 tracks asset reference.

AR / Real Estate / Product/Service engines own income events.

Accounting posts revenue.

Selene must link:

```text
asset_id
revenue_stream_ref
customer_contract_ref
rental_income_ref
maintenance_cost_ref
profitability_ref
```

## 29. Asset As Collateral / Security

Assets may secure loans.

Examples:

```text
building securing mortgage
vehicle securing car loan
equipment securing finance
inventory securing credit line
```

Future Debt/Loans + Real Estate + Legal owners manage security.

Document 9 must store link.

Fields:

```text
asset_id
secured_debt_ref
security_document_ref
lender_ref
collateral_value_ref
release_required_before_disposal
audit_ref
```

Hard rule:

```text
No secured asset may be sold/disposed/transferred without checking debt/security release requirements.
```

## 30. Asset Count / Verification / Stocktake

PH1.ASSET owns physical verification.

Document 9 consumes evidence.

Verification types:

```text
physical sighting
photo proof
barcode/RFID scan
serial number check
custodian confirmation
location audit
condition report
```

If asset missing:

```text
missing asset exception
investigation
insurance/security review
write-off request if unresolved
accounting/tax treatment
audit
```

## 31. Access And Authority

Protected asset actions:

```text
approve asset purchase
classify asset
approve capitalization
approve depreciation override
approve asset disposal
approve write-off
approve impairment
approve revaluation
transfer asset across entities
change business/private use percentage
change useful life
change residual value
approve collateral release
export asset register
```

Authority depends on:

```text
asset value
asset class
legal entity
country
department
funding source
tax impact
collateral status
insurance status
board policy
```

Step-up may be required for:

```text
high-value asset disposal
asset write-off
revaluation approval
collateral release
large asset purchase
asset register export
```

## 32. PH1.D / GPT-5.5 Role

Allowed:

```text
summarize asset invoice
suggest asset class candidate
explain depreciation in simple terms
draft lease/buy summary
draft asset purchase business case
draft board asset summary
explain claimability uncertainty
summarize disposal options
```

Forbidden:

```text
final asset classification
final tax claimability
final depreciation rule
approval
revaluation
write-off
disposal
ledger posting
payment execution
```

## 33. PH1.WRITE Wording

PH1.WRITE owns final human explanation.

Examples:

### Asset Candidate

```text
This purchase looks like a company asset rather than an ordinary expense. I’ll route it through asset classification before Accounting posts it.
```

### Depreciation

```text
The asset is now in service, so depreciation can start from the approved start date under the selected rule pack.
```

### Claimability

```text
This cost may be only partly claimable because the asset has private use. I’ll apply the approved business-use percentage and route the tax treatment for review.
```

### Disposal

```text
Before this asset can be sold, I need to check book value, any linked loan/security, tax treatment, and approval requirements.
```

## 34. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
asset_id
source_document_ref
purchase_order_ref
invoice_ref
receipt_inspection_ref
approval_refs
tax_rule_ref
depreciation_schedule_ref
journal_ref
insurance_ref
loan_or_lease_ref
valuation_ref
old_value_ref
new_value_ref
timestamp
company_id
legal_entity_id
country
currency
reason_code
```

No silent asset changes.

No unlogged write-off.

No depreciation override without evidence.

## 35. Failure Branches

### Asset Classification Uncertain

```text
Hold posting.
Route to Accounting/Tax review.
```

### Receipt/Inspection Missing

```text
Asset cannot be finalized as accepted.
AP payment may be held under AP goods-receiving law.
```

### Tax Rule Missing

```text
Claimability/depreciation tax treatment blocked.
Route to Tax owner.
```

### Asset Not In Service

```text
Asset register may exist.
Depreciation not started until in-service evidence.
```

### Insurance Missing

```text
Warn/route to Insurance if asset requires coverage before use.
```

### Collateral Linked

```text
Disposal blocked until debt/security owner clears release.
```

### Private Use Unknown

```text
Claimability blocked or limited until business-use evidence provided.
```

### Duplicate Asset Candidate

```text
Possible duplicate asset record found.
Review before creating new asset.
```

## 36. Required Logical Packets

Future logical packets:

```text
AssetCandidatePacket
AssetClassificationPacket
AssetRegisterPacket
CapitalizationDecisionPacket
PlacedInServicePacket
DepreciationSchedulePacket
BookTaxDepreciationSplitPacket
BusinessPrivateUsePacket
ClaimabilityAssessmentPacket
RepairVsImprovementPacket
AssetMaintenanceAccountingHandoffPacket
AssetInsuranceHandoffPacket
VehicleAssetHandoffPacket
PropertyAssetHandoffPacket
AssetRevaluationPacket
AssetImpairmentPacket
AssetDisposalPacket
AssetTransferPacket
RightOfUseAssetPacket
AssetUnderConstructionPacket
IntangibleAssetPacket
LowValueAssetPacket
AssetFundingSourcePacket
LeaseBuyAssetScenarioPacket
AssetIncomeGenerationPacket
AssetCollateralPacket
AssetVerificationPacket
AssetAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 37. Example — Buying A Delivery Van

```text
Need:
New delivery van.

Selene checks:
- vehicle capex budget
- cashflow/reserve
- lease vs buy options
- tax/GST/VAT treatment
- depreciation rule
- insurance requirement
- fleet assignment
- loan/lease funding
- approval threshold

PO approved.
Vehicle delivered and inspected.
AP receives invoice.
Asset register created.
Fleet record required.
Insurance linked.
Depreciation starts when van is in service.
```

Selene says:

```text
The delivery van is accepted and ready for asset setup. I’ll create the asset record, connect it to Fleet and Insurance, and start depreciation only when it is marked in service.
```

## 38. Example — Building Purchase

```text
Company buys commercial building.

Selene checks:
- land/building split
- title/legal documents
- loan/security
- insurance
- property tax/rates
- rental income possibility
- depreciation/capital works treatment
- board approval
- cashflow/reserve
```

Selene says:

```text
This property needs land/building split, title evidence, loan/security review, insurance, and tax treatment before Accounting finalizes the asset. Rental income and collateral management will connect to the Real Estate and Debt owners.
```

## 39. Example — Repair Vs Improvement

```text
Invoice:
AUD 8,000 for warehouse door work.

Possibility A:
repair damaged door → expense

Possibility B:
install upgraded automated door → capital improvement
```

Selene says:

```text
This invoice may be a repair or an improvement. I need to know whether it restored the door or upgraded it. That determines whether it is expensed or capitalized.
```

## 40. What Must Not Happen

```text
no asset capitalized without source evidence
no asset expensed without classification where asset candidate exists
no depreciation started before in-service evidence
no GPT-5.5 invented depreciation/tax rule
no private-use asset treated as fully business without proof
no land/building/property assumptions without evidence
no asset disposal without book value and approval
no secured/collateral asset disposed without debt/security check
no asset revaluation without policy and valuation evidence
no insurance-required asset used without insurance warning
no PH1.ASSET and Accounting owner merge
no Fleet vehicle operations owned by Accounting
no Real Estate property lifecycle buried inside general fixed assets
no old asset records erased
no implementation from this document alone
```

## 41. Future Simulation Targets

```text
SIM_ASSET_001_asset_candidate_detected_from_supplier_invoice
SIM_ASSET_002_delivery_van_purchase_asset_setup
SIM_ASSET_003_asset_received_but_not_in_service_no_depreciation
SIM_ASSET_004_repair_vs_improvement_classification
SIM_ASSET_005_business_private_use_vehicle_claimability
SIM_ASSET_006_low_value_asset_policy
SIM_ASSET_007_asset_disposal_gain_loss
SIM_ASSET_008_property_purchase_land_building_split
SIM_ASSET_009_collateral_asset_disposal_blocked
SIM_ASSET_010_asset_revaluation_requires_policy
SIM_ASSET_011_asset_impairment_after_damage
SIM_ASSET_012_right_of_use_asset_from_lease
SIM_ASSET_013_asset_under_construction_to_in_service
SIM_ASSET_014_insurance_missing_warning
SIM_ASSET_015_duplicate_asset_candidate_review
```

## 42. Related Addendum

Country tax packs, claimability rule packs, official-source rule update protocols, stamp duty / transfer duty checks, asset transfer government charges, cash-vs-loan-vs-lease decision intelligence, and the PH1.GUIDE future capability explainer boundary are defined in SELENE_ASSETS_COUNTRY_TAX_PACKS_STAMP_DUTY_FUNDING_GUIDE_ADDENDUM.md and must be read with this document.

## 43. Final Architecture Sentence

Selene Assets + Depreciation + Claimable Expense Rules is the governed asset accounting and tax handoff layer: it detects asset candidates from financial and operational evidence, separates capital assets from expenses, creates asset-register accounting requirements, controls placed-in-service timing, depreciation, business/private use, tax claimability, repairs versus improvements, impairment, revaluation, disposal, funding, collateral, insurance, fleet, property, and real estate dependencies, while preserving PH1.ASSET as the future real-world asset lifecycle owner and ensuring Accounting, Tax, Finance, AP, Banking, Insurance, Fleet, Debt, Real Estate, Access, PH1.D, PH1.WRITE, and Audit each keep their proper boundaries.
