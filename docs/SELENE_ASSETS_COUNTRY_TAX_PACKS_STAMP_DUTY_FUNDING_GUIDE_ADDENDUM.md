# Selene Assets Addendum — Country Tax Packs + Stamp Duty + Asset Funding Decision Intelligence + Capability Guide Boundary

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / ASSET TAX PACKS + CLAIMABILITY RULES + STAMP DUTY + FUNDING DECISION INTELLIGENCE + CAPABILITY GUIDE BOUNDARY

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Selene Assets + Depreciation + Claimable Expense Rules Master Design

PURPOSE:
Strengthen the asset master design with country-specific tax rule packs, claimability packs, official-source update protocols, versioned depreciation/capital allowance rules, stamp duty / transfer duty checks, asset transfer government charge handling, cash-vs-loan-vs-lease decision intelligence, and the future PH1.GUIDE human capability explainer boundary.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 5. It defines future country tax packs, rule registries, claimability packs, official-source candidate update protocols, asset transfer duty checks, funding decision intelligence, and PH1.GUIDE capability explanation boundaries. It does not implement Tax, Assets, Accounting, Finance, Debt/Treasury, PH1.GUIDE, Access, packets, migrations, tests, official-source connectors, or runtime state.

Current repo truth does not prove complete runtime Tax/Compliance country packs, PH1.ASSET, Debt/Treasury, Real Estate, PH1.GUIDE, or full Finance/Accounting ownership. This addendum is future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Master Addendum Law

Selene must not guess tax, depreciation, claimability, stamp duty, transfer duty, asset deductibility, or government charges.

Selene must use governed country-specific rule packs.

```text
Every asset tax treatment must be country/region-specific.

Every depreciation or capital allowance rule must come from an approved rule pack.

Every claimability decision must be source-backed, versioned, effective-date controlled, and auditable.

Every asset transfer must check stamp duty, transfer duty, registration fees, capital gains, GST/VAT, land/property charges, and other jurisdiction-specific obligations where relevant.

Every cash-vs-loan-vs-lease decision must compare tax, cashflow, accounting, debt, insurance, asset lifecycle, collateral, and approval impact.

Selene may search official sources and propose updates.

Selene must not activate new tax rules without validation and approval.

PH1.D / GPT-5.5 may help read, summarize, explain, and compare.

Deterministic Tax, Accounting, Finance, Asset, Debt, Legal, and Access owners decide what becomes active truth.
```

Selene may do the homework. She does not become the tax authority.

## 2. Country Tax Pack Framework

Selene needs a country-specific tax pack framework.

Do not build every country in one shot.

Build the framework first, then add country packs as needed.

```text
1. Build TaxRulePackRegistry.
2. Add first live country packs based on real company jurisdictions.
3. Support country/region/state/province overlays.
4. Version every rule.
5. Track effective dates.
6. Track official source references.
7. Require Tax/Compliance approval before activation.
8. Retire old rule versions only by effective-date closure, not deletion.
```

Example country packs:

```text
Australia Tax Pack
USA Tax Pack
UK Tax Pack
Singapore Tax Pack
EU VAT Pack
Country + State/Province overlays where needed
```

A country pack may contain:

```text
income tax rules
GST / VAT / sales tax rules
payroll tax rules
super / pension / CPF / retirement contribution rules
depreciation / capital allowance rules
claimability rules
stamp duty / transfer duty rules
land/property tax rules
withholding rules
vehicle rules
fringe benefit / benefit-in-kind rules
salary/minimum wage references
employment contribution references
reporting due dates
registration requirements
effective dates
official source references
approval status
version
audit refs
```

Hard rule:

```text
No country pack becomes active merely because Selene found a webpage.

Discovery creates a candidate.
Validation creates an approved rule.
Activation creates live tax behavior.
```

## 3. Tax Rule Pack Registry

Future logical structure:

```text
TaxRulePackRegistry:
  registry_id
  country
  region
  jurisdiction_level
  rule_pack_type
  version
  effective_from
  effective_to
  status
  source_refs
  approved_by
  reviewed_at
  next_review_due
  audit_ref
```

Rule pack types:

```text
asset_depreciation
capital_allowance
claimability
GST_VAT_sales_tax
stamp_transfer_duty
payroll_tax
super_pension_CPF
withholding
vehicle_tax
property_tax
fringe_benefit
dividend_tax
contractor_tax
```

Statuses:

```text
Draft
SourceDiscovered
CandidateExtracted
PendingValidation
Validated
Approved
Active
Superseded
Rejected
Expired
Archived
```

## 4. Official-Source Search And Update Protocol

Selene should do the research work.

But she must treat research as candidate evidence until validated.

Flow:

```text
Selene detects country/region/company rule need
→ Selene searches approved official sources or approved provider feeds
→ Selene extracts candidate rule
→ Selene records source URL/ref, publication date, effective date, jurisdiction, and affected rule type
→ Selene compares candidate against current active rule pack
→ Selene creates TaxRuleUpdateCandidate
→ Selene prepares human-readable impact summary
→ Tax/Compliance owner validates
→ approved candidate becomes new active rule version from effective date
→ old rule remains preserved as historical version
```

Selene should prefer:

```text
government tax authority
government revenue office
official legislation source
official regulator
approved licensed tax provider
approved accounting standards provider
company-approved tax adviser source
```

Selene should avoid relying on:

```text
random blogs
unverified forum posts
old PDFs without effective date
scraped summaries with no authority
provider output without source references
LLM answer alone
```

Selene says:

```text
I found an official-source update that may affect asset depreciation from the next tax year. I’ve prepared the impact summary, but I need Tax approval before it becomes the active rule.
```

## 5. Tax Rule Update Candidate

Future logical packet:

```text
TaxRuleUpdateCandidatePacket:
  candidate_id
  country
  region
  rule_pack_type
  source_ref
  source_title
  source_published_at
  extracted_effective_from
  extracted_effective_to
  current_rule_version_ref
  proposed_rule_version
  affected_asset_classes
  affected_expense_categories
  summary_of_change
  confidence
  validation_required
  approved_by
  activation_status
  audit_ref
```

Candidate outcomes:

```text
approve_and_activate
approve_future_effective_date
reject
needs_tax_adviser_review
needs_more_sources
manual_override
defer
```

Hard rule:

```text
A TaxRuleUpdateCandidate is not an active rule.

It is a proposed rule change with evidence.
```

## 6. Claimability Rule Pack

Selene needs a claimability pack.

The claimability pack tells Selene whether an asset or expense is:

```text
fully claimable
partially claimable
not claimable
capitalized and claimed over time
deductible immediately where allowed
subject to private/business split
subject to documentation threshold
subject to tax-owner review
subject to country/region restriction
```

Future logical structure:

```text
ClaimabilityRulePack:
  rule_pack_id
  country
  region
  asset_or_expense_type
  business_use_required
  private_use_treatment
  claimable_percentage_rule
  GST_VAT_claimability
  capitalized_or_expensed
  depreciation_or_capital_allowance_treatment
  stamp_duty_treatment
  documentation_required
  receipt_or_tax_invoice_required
  effective_from
  effective_to
  official_source_refs
  approved_by
  status
  audit_ref
```

Selene uses claimability packs for:

```text
vehicles
fuel
repairs
insurance
registration
property costs
software
tools
home office
travel
meals
training
equipment
building improvements
lease payments
loan interest
mixed-use assets
employee reimbursements
credit card spend
```

Selene says:

```text
This expense may be only partly claimable because it has mixed business and private use. I’ll apply the approved claimability rule pack and route anything uncertain to Tax review.
```

## 7. Versioning And Effective Dates

Tax, depreciation, claimability, and duty rules change over time.

Selene must never overwrite old rules as if history did not exist.

Each rule needs:

```text
rule_id
country
region
rule_type
version
effective_from
effective_to
source_ref
approved_by
approval_date
superseded_by
status
audit_ref
```

When a rule changes:

```text
new rule version created
old rule version receives effective_to date
transactions before change keep old rule
transactions after effective date use new rule
adjustment required only if rule says retrospective treatment applies and Tax approves
```

Hard rule:

```text
No historical asset depreciation, tax claim, or duty calculation may be silently recalculated because a rule changed later.
```

## 8. Stamp Duty / Transfer Duty / Government Charge Checks

Asset transfers can trigger government charges.

This must be added clearly.

Applies to:

```text
real estate
land
buildings
vehicles
business assets
shares or interests in land-rich entities where relevant
leases where jurisdiction applies
high-value asset transfers
intercompany transfers
related-party transfers
asset sale/disposal
asset restructuring
```

Potential charges to check:

```text
stamp duty
transfer duty
registration fee
land transfer fee
vehicle transfer fee
title registration fee
capital gains tax
GST / VAT / sales tax
withholding tax
land tax
property tax
foreign purchaser surcharge where relevant
intercompany transfer tax
customs/import duty where cross-border
```

Selene must ask:

```text
What asset is being transferred?
Who owns it now?
Who will own it after transfer?
Are both parties in the same group?
What country/region/state applies?
What is the market value?
Is there debt or security attached?
Is this sale, gift, restructuring, contribution, or correction?
Is there board/shareholder approval requirement?
```

Hard law:

```text
No asset transfer may proceed without checking jurisdiction-specific transfer taxes, stamp duties, registration charges, debt/security restrictions, and approval requirements.
```

## 9. Intercompany Asset Transfer Logic

Asset transfers inside a group can still trigger taxes or duties.

Selene must not assume "same group" means "free transfer."

Flow:

```text
transfer requested
→ legal entities identified
→ ownership group relationship checked
→ jurisdiction checked
→ asset type identified
→ market value / carrying value checked
→ stamp/transfer duty candidate calculated
→ tax/capital gain candidate checked
→ debt/collateral check
→ board/shareholder approval route checked
→ legal/tax review if required
→ transfer proceeds only if approved
```

Selene says:

```text
This property transfer is within the group, but it may still trigger transfer duty and tax consequences. I recommend tax/legal review before proceeding.
```

Very important:

```text
There may be no business purpose in transferring an asset inside a group if duty/tax cost outweighs the benefit.

Selene must compare benefit against transfer cost before recommending transfer.
```

## 10. Asset Transfer Decision Packet

Future logical packet:

```text
AssetTransferDecisionPacket:
  transfer_id
  asset_id
  asset_type
  current_owner_entity_id
  proposed_owner_entity_id
  relationship_between_entities
  country
  region
  market_value
  book_value
  tax_value
  debt_or_collateral_ref
  transfer_reason
  stamp_duty_candidate
  transfer_duty_candidate
  registration_fee_candidate
  capital_gains_candidate
  GST_VAT_candidate
  legal_review_required
  tax_review_required
  board_approval_required
  recommendation
  audit_ref
```

Recommendations:

```text
proceed
do_not_transfer
needs_tax_review
needs_legal_review
needs_board_approval
alternative_structure_recommended
transfer_after_debt_release
transfer_after_valuation
```

## 11. Cash Vs Loan Vs Lease Decision Intelligence

Selene must decide asset funding using both probabilistic and deterministic layers.

### 11.1 Probabilistic Layer

PH1.D / GPT-5.5 may help:

```text
understand the user request
read messy quotes or finance offers
summarize lease/loan/cash options
draft plain-English comparison
explain trade-offs
identify missing assumptions
prepare board/management wording
```

Example PH1.D proposal:

```text
intent: asset_funding_comparison
asset: delivery van
options: cash purchase, loan, lease
risk_hints:
  - reserve impact
  - tax/depreciation treatment
  - interest cost
  - insurance
  - utilization
  - debt/covenant effect
missing_fields:
  - purchase price
  - loan rate
  - lease term
  - residual value
  - expected usage
```

But:

```text
PH1.D may propose comparison structure.

PH1.D must not decide final recommendation or approval.
```

### 11.2 Deterministic Layer

Final decision uses canonical owners:

```text
Finance/Budget = budget, ROI, capex approval
Cashflow = reserve and timing
Tax/Compliance = depreciation, GST/VAT, claimability, capital allowance
Accounting = book treatment
Debt/Treasury = loan/lease liability, interest, covenants
PH1.ASSET = asset lifecycle/value
PH1.FLEET = vehicle utilization if vehicle
PH1.INSURANCE = premium/coverage impact
Access/Governance = approval authority
PH1.BOARD = board approval if threshold requires
```

Selene's recommendation must be based on deterministic evidence.

## 12. Asset Funding Scenario Comparison

Selene must compare:

```text
cash purchase
bank loan
vehicle/equipment finance
lease
hire purchase
short-term rental
repair existing asset
delay purchase
buy used
buy new
shared asset
outsourced service
```

Comparison factors:

```text
upfront cash impact
minimum cash reserve impact
monthly payment
total cost over term
interest cost
fees
GST/VAT treatment
depreciation / capital allowance
tax deductibility
lease accounting / right-of-use asset impact
loan liability
balance sheet impact
debt covenants
collateral/security
insurance cost
maintenance cost
expected resale value
asset utilization
risk of obsolescence
operational reliability
downtime risk
cashflow volatility
profit floor impact
budget impact
approval threshold
board approval requirement
```

Output structure:

```text
Option A — Cash Purchase
  pros
  cons
  cash impact
  accounting/tax impact
  risk

Option B — Loan
  pros
  cons
  cash impact
  debt impact
  accounting/tax impact
  risk

Option C — Lease
  pros
  cons
  cash impact
  lease liability impact
  accounting/tax impact
  risk

Recommended option
Reason
Approval required
```

## 13. Funding Decision Example — Delivery Van

```text
Asset:
Delivery van

Cash purchase:
- lowest total cost
- no interest
- reduces cash reserve for 6 weeks
- ownership and resale value retained

Loan:
- preserves some cash
- creates debt liability
- interest cost applies
- asset may be collateral

Lease:
- protects cash reserve
- predictable monthly payments
- may cost more over 36 months
- lease accounting may create right-of-use asset and liability
```

Selene says:

```text
Buying the van is cheapest overall, but it breaches the cash reserve. Leasing costs more but protects cash and keeps payments predictable. I recommend leasing unless the board approves a temporary reserve breach for the cash purchase.
```

## 14. Funding Decision Packet

Future logical packet:

```text
AssetFundingDecisionPacket:
  decision_id
  asset_candidate_id
  asset_class
  cash_purchase_option
  loan_option
  lease_option
  rental_option
  repair_existing_option
  buy_used_option
  cashflow_impact_ref
  tax_impact_ref
  accounting_impact_ref
  debt_impact_ref
  insurance_impact_ref
  asset_lifecycle_ref
  ROI_summary
  risk_summary
  recommended_option
  approval_required
  approver_refs
  audit_ref
```

## 15. Country Tax Pack Interaction With Asset Funding

Country tax packs may affect funding choice.

Examples:

```text
cash purchase depreciation
lease payment deductibility
GST/VAT timing
loan interest deductibility
capital allowance timing
low-value asset threshold
vehicle private-use treatment
stamp duty or transfer duty
property tax
registration charges
```

Selene must not decide funding based on cashflow alone.

Example:

```text
Leasing protects cash, but the tax treatment differs from buying. I’ll compare both under the active country tax pack before recommending the final option.
```

## 16. Asset Transfer And Stamp Duty Example — Property

```text
User:
Transfer the warehouse property from Company A to Company B.

Selene checks:
- property jurisdiction
- current legal owner
- proposed legal owner
- group relationship
- market value
- book value
- tax value
- title
- mortgage/security
- stamp/transfer duty candidate
- GST/VAT candidate
- capital gain candidate
- board/shareholder approval
```

Selene says:

```text
This transfer may trigger stamp duty and tax costs even though both entities are in the same group. I do not recommend proceeding until Tax, Legal, and Board approval confirm the benefit outweighs the transfer cost.
```

## 17. PH1.GUIDE — Capability Guide + Human Action Explainer Boundary

Add future engine:

```text
PH1.GUIDE
Selene Capability Guide + Human Action Explainer Engine
```

Purpose:

```text
Help humans understand what Selene can do, how to ask for it, what Selene needs, what Selene will do next, and which safe process applies — without requiring humans to know engine names.
```

Humans will not know:

```text
that asset accounting is separate from PH1.ASSET
that vehicle operations belong to PH1.FLEET
that tax packs drive depreciation
that bank changes require step-up
that supplier disputes use Receiving/AP
that budgets check before PO
that cashflow collects before delaying payments
```

Selene must guide them.

Example:

```text
User:
How do I add a new delivery van?
```

Selene:

```text
I can help. I’ll check whether this should be treated as an asset, compare cash, loan, and lease options if needed, check budget and cashflow, prepare approval, then create the asset, fleet, insurance, and depreciation setup once approved.
```

PH1.GUIDE owns:

```text
capability discovery
user education
human action explanations
what-to-do-next guidance
engine capability map
permission-aware help
process explanations
example prompts
guided "how do I" flows
```

PH1.GUIDE must not:

```text
execute business actions
grant authority
change data
bypass PH1.X
bypass Access
replace PH1.WRITE
invent capabilities
show restricted functions to unauthorized users
```

Owner split:

```text
PH1.GUIDE = explains capabilities and next steps
PH1.D = drafts helpful explanations
PH1.WRITE = final wording
Access = filters what user may see/use
PH1.M = remembers user learning preferences if allowed
Canonical engines = provide capability metadata
```

## 18. Capability Metadata

Every major engine should eventually publish capability metadata for PH1.GUIDE.

Future logical structure:

```text
SeleneCapabilityMetadataPacket:
  capability_id
  engine_owner
  user_friendly_name
  what_it_does
  example_user_phrases
  required_permissions
  required_inputs
  protected_actions
  related_engines
  common_failures
  help_text_ref
  last_updated
  audit_ref
```

Example asset capability metadata:

```text
capability_id: add_company_asset
engine_owner: PH1.ASSET / Accounting / Tax / Finance
user_friendly_name: Add a company asset
example_user_phrases:
  - Add a new delivery van
  - Record this machine as an asset
  - Should I lease or buy this vehicle?
required_permissions:
  - asset_create_candidate
  - capex_request
protected_actions:
  - approve purchase
  - approve classification
  - activate depreciation
```

## 19. Human Explanation Standard

Selene should always explain complex asset/tax/funding processes in human language.

Bad:

```text
AssetFundingDecisionPacket requires deterministic owner validation.
```

Good:

```text
I need to compare the cash impact, tax treatment, insurance, depreciation, and approval rules before recommending whether to buy, lease, or finance this asset.
```

Selene should explain:

```text
what she is checking
why it matters
what humans need to provide
what Selene can do automatically
what needs approval
what happens next
what risks exist
```

## 20. Access And Authority

Protected actions added by this addendum:

```text
activate new country tax pack
approve tax rule update
override claimability rule
approve stamp duty / transfer duty treatment
approve asset transfer
approve intercompany asset transfer
approve funding option
approve cash purchase that breaches reserve
approve loan/lease commitment
approve tax treatment for asset transfer
approve PH1.GUIDE capability visibility for restricted functions
```

Authority may depend on:

```text
country
region
legal entity
asset value
asset type
tax risk
duty risk
cash reserve impact
profit floor impact
debt/collateral impact
board threshold
shareholder/related-party status
regulated advice risk
```

Step-up may be required for:

```text
tax rule activation
asset transfer approval
intercompany transfer
high-value asset funding approval
reserve-breach asset purchase
restricted capability export
```

## 21. PH1.D / GPT-5.5 Role

Allowed:

```text
search planning for official sources
source reading assistance
draft tax rule summary
draft claimability explanation
draft lease/buy comparison narrative
summarize stamp duty risk
explain capability to user
draft PH1.GUIDE help text
identify missing assumptions
```

Forbidden:

```text
activate tax rule
invent tax law
approve claimability
approve depreciation
approve stamp duty treatment
approve asset transfer
approve funding decision
grant authority
execute payment
post accounting
show restricted capability without Access
```

## 22. PH1.WRITE Wording

PH1.WRITE owns final user-facing wording.

Examples:

### Tax Rule Update

```text
I found a possible official-source update affecting this asset class. I’ve prepared the impact summary, but Tax must approve it before Selene uses it.
```

### Stamp Duty

```text
This transfer may trigger stamp duty or transfer tax. I’ll check the applicable country and region pack before recommending whether the transfer makes financial sense.
```

### Lease Vs Buy

```text
Buying is cheaper overall, but leasing protects cash. I’ll compare tax, depreciation, interest, insurance, and cash reserve impact before recommending an option.
```

### Capability Guide

```text
You can ask me to add a company asset, compare lease versus buy, check depreciation, or prepare an asset disposal review. I’ll guide you through the safe steps.
```

## 23. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
country
region
rule_pack_id
rule_version
source_refs
candidate_rule_ref
approval_refs
asset_id
transfer_id
funding_decision_id
claimability_ref
stamp_duty_ref
tax_review_ref
PH1GUIDE_capability_ref
old_value_ref
new_value_ref
timestamp
company_id
legal_entity_id
reason_code
```

No unlogged tax rule changes.

No silent claimability updates.

No asset transfer without duty/tax audit.

## 24. Failure Branches

### Official Source Not Found

```text
Rule remains unverified.
Prior approved rule remains active.
Tax review required.
```

### Conflicting Sources

```text
Create TaxRuleConflictPacket.
Tax/Compliance owner reviews.
No activation.
```

### Effective Date Unclear

```text
Do not activate.
Request review.
Use current approved rule until clarified.
```

### Stamp Duty Unknown

```text
Asset transfer blocked or routed to legal/tax review.
```

### Lease/Buy Missing Data

```text
Selene asks for missing purchase price, rate, lease term, residual value, or usage assumptions.
```

### User Asks PH1.GUIDE About Restricted Capability

```text
Access filters response.
Selene may say the function requires permission.
```

## 25. Required Logical Packets

Future logical packets:

```text
TaxRulePackRegistryPacket
CountryTaxPackPacket
TaxRuleUpdateCandidatePacket
OfficialSourceEvidencePacket
ClaimabilityRulePackPacket
RuleVersioningPacket
StampTransferDutyAssessmentPacket
GovernmentChargeAssessmentPacket
AssetTransferDecisionPacket
IntercompanyAssetTransferPacket
AssetFundingDecisionPacket
LeaseBuyScenarioPacket
TaxFundingImpactPacket
CapabilityGuideRequestPacket
SeleneCapabilityMetadataPacket
CapabilityVisibilityPacket
CapabilityExplanationPacket
TaxRuleConflictPacket
AssetTaxAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 26. Example — Updating Depreciation Rule Pack

```text
Selene detects:
Company operates in Australia.
Asset class: delivery vehicle.
Depreciation/effective-life rule may have been updated.

Selene:
- searches approved official source list
- finds candidate update
- extracts effective date and affected asset class
- compares to active Australian Tax Pack
- prepares impact summary
- routes to Tax owner
```

Selene says:

```text
I found a possible official update affecting delivery vehicle depreciation. I’ve prepared the comparison against the current rule pack, but I need Tax approval before using it.
```

## 27. Example — Asset Transfer With Duty Risk

```text
User:
Transfer the property from Company A to Company B.

Selene:
- identifies asset as real estate
- checks jurisdiction
- checks same-group relationship
- checks market value
- checks loan/security
- checks stamp/transfer duty candidate
- checks CGT/GST/VAT candidate
- checks board/shareholder approval
```

Selene says:

```text
This transfer may trigger duty and tax costs. There may be no benefit if the duty cost outweighs the reason for transfer. I recommend tax and legal review before proceeding.
```

## 28. Example — User Does Not Know Asset Engine Exists

```text
User:
I bought a new machine. What do I do?

Selene:
- recognizes asset candidate
- explains asset workflow
- checks receipt/invoice
- asks whether machine is installed or in service
- routes asset setup
```

Selene says:

```text
I can help. This may need to be added as a company asset rather than treated as a normal expense. I’ll check the invoice, whether it is in service, tax treatment, depreciation, insurance, and approval requirements.
```

## 29. Additions To Asset Must-Not-Happen Law

```text
no global one-size-fits-all tax treatment
no country tax pack activated without source-backed validation
no GPT-5.5 invented depreciation or claimability rule
no official-source search result treated as active law automatically
no historical tax/depreciation recalculation without effective-date and Tax approval
no asset transfer without stamp/transfer duty and government charge check
no same-group transfer assumed tax-free
no asset funding recommendation based only on purchase price
no cash-vs-loan-vs-lease recommendation without cashflow, tax, accounting, debt, asset, insurance, and approval evidence
no PH1.GUIDE execution of business actions
no PH1.GUIDE showing restricted capabilities to unauthorized users
no user forced to know internal engine names to use Selene
no implementation from this addendum alone
```

## 30. Future Simulation Targets

```text
SIM_ASSET_TAX_001_country_tax_pack_selected_by_jurisdiction
SIM_ASSET_TAX_002_official_source_update_creates_candidate_not_active_rule
SIM_ASSET_TAX_003_tax_rule_update_requires_tax_owner_approval
SIM_ASSET_TAX_004_claimability_pack_applies_business_private_split
SIM_ASSET_TAX_005_historical_depreciation_preserved_after_new_rule
SIM_ASSET_DUTY_001_property_transfer_stamp_duty_check
SIM_ASSET_DUTY_002_same_group_transfer_not_assumed_free
SIM_ASSET_FUND_001_cash_vs_loan_vs_lease_vehicle_comparison
SIM_ASSET_FUND_002_lease_recommended_due_to_cash_reserve
SIM_ASSET_FUND_003_cash_purchase_requires_board_reserve_override
SIM_GUIDE_001_user_asks_how_to_add_asset
SIM_GUIDE_002_restricted_capability_hidden_without_access
SIM_GUIDE_003_capability_explanation_does_not_execute_action
```

## 31. Final Addendum Architecture Sentence

Selene Assets + Depreciation must become country-aware, duty-aware, claimability-aware, and funding-intelligent: each asset decision must use approved country tax packs, official-source update candidates, versioned effective-date rules, claimability packs, stamp/transfer duty checks, government charge assessments, and deterministic lease-vs-buy-vs-loan comparisons across cashflow, tax, accounting, debt, insurance, asset lifecycle, and governance — while PH1.GUIDE teaches humans how to use Selene’s asset capabilities without granting power, bypassing owners, or requiring users to know the machinery behind the curtain.
