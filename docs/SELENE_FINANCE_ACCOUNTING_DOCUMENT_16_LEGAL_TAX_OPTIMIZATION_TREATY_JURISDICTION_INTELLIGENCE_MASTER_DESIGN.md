# Finance / Accounting Document 16 — Selene Legal Tax Optimization + Treaty + Jurisdiction Intelligence Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
16

ENGINE:
PH1.TAX.OPTIMIZE / PH1.TAX.INTELLIGENCE

FULL NAME:
Selene Legal Tax Optimization, Treaty Relief, Jurisdiction Intelligence, Incentive Discovery, Deferral, Credit, Structure, and Tax Burden Reduction Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
MUST_BE_SOURCE_BACKED
MUST_BE_REVIEWABLE_BY_TAX_AUTHORITY / ADVISER / APPROVED HUMAN OWNER
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition. No runtime code, schemas, migrations, APIs, packet structs, tests, live tax rule packs, tax rates, jurisdiction rules, or engine code are created by this document.

This document defines future canonical architecture for PH1.TAX.OPTIMIZE / PH1.TAX.INTELLIGENCE. Repo-truth activation, simulation mapping, owner mapping, tests, source pack governance, and approved implementation slices must happen later before runtime behavior can be claimed.

This document is not legal or tax advice. It defines how Selene must structure future source-backed, reviewable legal tax optimization workflows.

## 1. Purpose

Selene Legal Tax Optimization + Treaty + Jurisdiction Intelligence Engine owns the company's legal tax-reduction brain.

Document 15 makes sure the company files, reports, pays, evidences, and calendars tax correctly.

Document 16 asks:

```text
Are we legally paying more tax than we need to?
```

Selene must continuously search for, detect, evaluate, and propose lawful ways to:

```text
reduce tax
defer tax
claim deductions
claim credits
claim incentives
use treaty relief
avoid double taxation
structure cross-border transactions correctly
use tax depreciation / capital allowances efficiently
claim bad debt relief
optimize payroll/social contribution handling
avoid unnecessary withholding
avoid duplicate taxation
evaluate zero/low-tax jurisdiction opportunities
manage foreign tax credit
manage transfer pricing support
avoid anti-abuse and substance failures
```

This engine is a legal optimization engine:

```text
deep research
source-backed rules
evidence
simulation
risk scoring
adviser review
authority approval
audit trail
```

Selene's job is to legally reduce tax burden while making unlawful, unsupported, artificial, or high-risk strategies visible and blocked.

## 2. Core Selene Law

```text
Selene must legally minimize tax, not merely calculate tax.

Every tax reduction strategy must be:
- lawful
- source-backed
- jurisdiction-scoped
- effective-dated
- evidence-supported
- anti-abuse checked
- transfer-pricing checked where relevant
- substance checked where relevant
- adviser/human-reviewable
- audit-recorded
```

Selene must never:

```text
invent tax law
hide income
fake deductions
create artificial substance
abuse treaties
ignore transfer pricing
ignore permanent establishment risk
ignore CFC/anti-avoidance rules
ignore Pillar Two where applicable
```

Legal tax planning is allowed only when source evidence, rule packs, risk checks, authority, and audit support it.

## 3. Engine Boundary

### 3.1 PH1.TAX.OPTIMIZE owns

```text
legal tax opportunity discovery
tax reduction strategy registry
tax deferral strategy registry
deduction optimization
claimable expense optimization
capital allowance / depreciation optimization
bad debt relief optimization
foreign tax credit opportunity detection
double tax treaty relief analysis
withholding tax reduction analysis
cross-border tax treatment proposals
entity/jurisdiction planning intelligence
zero/low-tax jurisdiction review
territorial tax regime review
free zone / special regime review
R&D / export / industry incentive detection
substance and economic presence checks
permanent establishment risk analysis
transfer pricing risk prompts
Pillar Two / global minimum tax risk prompts
China payroll/social contribution readiness prompts
tax adviser review packets
tax strategy risk scoring
tax optimization audit trail
```

### 3.2 PH1.TAX.OPTIMIZE does not own

```text
tax return filing
final tax payment
bank payment execution
ledger posting
payroll calculation
legal opinion final authority
transfer pricing documentation final sign-off
tax adviser sign-off
entity incorporation
director/shareholder governance decisions
```

### 3.3 Correct owner split

```text
Document 15 / PH1.TAX = compliance, filings, tax profiles, tax returns, tax calendars, tax payment obligations.
Document 16 / PH1.TAX.OPTIMIZE = legal reduction, treaty relief, credits, deferrals, incentives, structure intelligence, opportunity monitoring.
Accounting = financial posting.
Payroll = payroll calculation.
Legal = entity, contracts, governance, legal review.
Board/Shareholder = strategic structuring approvals if required.
Access/Authority = who may approve tax strategies.
Audit = proof.
PH1.D/GPT-5.5 = research assistance, drafting, and explanation, not final tax truth.
```

Simple split:

```text
Document 15 files the tax.
Document 16 fights the tax legally.
```

## 4. Compliance Vs Optimization Split

Compliance asks:

```text
What do we owe, file, evidence, and pay under approved rules?
```

Optimization asks:

```text
Is there a lawful, source-backed way to reduce, defer, credit, exempt, or restructure the tax burden?
```

Document 15 may detect an optimization candidate, but Document 16 owns the research, risk scoring, review packet, and approval path.

Document 16 may recommend a strategy, but Document 15 owns filing and compliance execution after the strategy is approved and mapped to source-backed treatment.

## 5. Legal Optimization Vs Illegal Evasion

Selene must classify tax strategies into risk categories.

```text
Green — standard legal claim / routine deduction / routine credit
Yellow — legal but documentation-sensitive
Orange — legal but requires adviser review
Red — high-risk / anti-abuse / transfer pricing / substance risk
Black — prohibited / evasion / false evidence / do not proceed
```

### 5.1 Green examples

```text
claim valid business expenses
claim available input tax credit with valid invoice
claim standard depreciation/capital allowance
claim bad debt relief with evidence
use available foreign tax credit
```

### 5.2 Yellow examples

```text
mixed-use asset allocation
employee benefit classification
partial input tax claim
home-office / vehicle allocation
cross-border service withholding review
```

### 5.3 Orange examples

```text
treaty relief
entity restructuring
free zone qualification
transfer pricing
intercompany charges
IP royalty planning
permanent establishment review
```

### 5.4 Red examples

```text
treaty shopping risk
no-substance offshore structure
aggressive profit shifting
unclear beneficial ownership
large tax avoidance motive
```

### 5.5 Black examples

```text
fake invoices
fake expenses
hidden revenue
false residency
false substance
fabricated tax certificates
backdated documents
```

Example:

```text
This appears to reduce tax, but it requires adviser review because treaty relief may be denied under anti-abuse rules.
```

## 6. Tax Opportunity Registry

Every legal tax-saving opportunity gets a structured record.

```text
tax_opportunity_id
legal_entity_id
jurisdiction
tax_type
opportunity_type
description
estimated_tax_saving
estimated_cashflow_impact
deferral_or_permanent_saving
source_rule_reference
effective_date
expiry_date
evidence_required
documentation_status
risk_level
anti_abuse_review_status
transfer_pricing_review_status
substance_review_status
adviser_review_status
approval_status
implementation_status
audit_ref
```

Opportunity statuses:

```text
Detected
EvidenceNeeded
Researching
NeedsReview
ApprovedForAction
Rejected
Implemented
Expired
Superseded
Archived
```

Tax-saving ideas must be recorded, evidenced, risk-scored, routed, and audited.

## 7. Legal Tax Research Engine

Selene must support deep tax research with source ranking.

### 7.1 Source hierarchy

```text
Tier 1 — tax authority / legislation / official treaty text / official guidance
Tier 2 — OECD / supranational guidance / regulator guidance
Tier 3 — reputable accounting/legal/tax professional summaries
Tier 4 — news / commentary / blogs
Tier 5 — unverified sources, not usable for final tax rule
```

### 7.2 Research workflow

```text
tax opportunity detected
-> jurisdiction identified
-> tax type identified
-> official sources searched
-> treaty/legislation/guidance retrieved
-> professional commentary compared
-> rule extracted
-> effective date checked
-> anti-abuse checked
-> evidence checklist created
-> risk scored
-> human/adviser review packet prepared
```

### 7.3 Research output

```text
TaxResearchPacket:
- question
- jurisdiction
- tax_type
- sources
- rule_summary
- effective_date
- eligibility_criteria
- exclusions
- anti_abuse_concerns
- evidence_checklist
- estimated_saving
- risk_rating
- recommended_next_action
```

Selene may use GPT-5.5 to summarize and compare sources, but final tax truth requires approved source-backed rule packs and authorized review.

## 8. Treaty Relief And Double Taxation Engine

Selene must detect when income may be taxed in two countries and whether treaty relief, exemption, reduced withholding, or foreign tax credit may apply.

Treaty relief is never automatic.

### 8.1 Treaty relief inputs

```text
payer_country
recipient_country
recipient_tax_residence
income_type
domestic_withholding_rate
treaty_withholding_rate
beneficial_owner
permanent_establishment
certificate_of_residence
contract
invoice
service_location
payment_date
entity_relationship
anti_abuse_status
```

### 8.2 Income types

```text
dividends
interest
royalties
service fees
management fees
capital gains
employment income
business profits
shipping/air transport where applicable
rent
director fees
pensions
```

### 8.3 Treaty workflow

```text
cross-border payment/income detected
-> domestic withholding checked
-> treaty existence checked
-> treaty article selected
-> treaty rate compared
-> relief requirements checked
-> certificate of residence requested
-> beneficial ownership checked
-> PE risk checked
-> anti-abuse checked
-> relief packet prepared
-> adviser/human approval
```

### 8.4 Treaty rule

Selene must require:

```text
residence evidence
beneficial ownership evidence
permanent establishment review
anti-abuse review
source rule reference
adviser/human approval where required
```

## 9. Foreign Tax Credit Engine

Selene must identify where foreign tax has already been paid and whether credit, exemption, pooling, or relief may apply.

### 9.1 Inputs

```text
foreign_income_amount
foreign_tax_paid
source_country
residence_country
income_type
tax_treaty_status
tax_receipt/certificate
remittance_status
local_tax_rule
credit_limitation
tax_period
```

### 9.2 Outputs

```text
foreign_tax_credit_candidate
foreign_tax_credit_amount_estimate
credit_limitation_warning
evidence_checklist
review_required
filing_period
```

Example:

```text
This foreign income may qualify for tax credit. I have captured the foreign tax paid and prepared the evidence checklist.
```

## 10. Withholding Tax Reduction Engine

Selene must review cross-border supplier/customer payments for withholding tax.

### 10.1 Triggers

```text
cross-border supplier payment
royalty payment
interest payment
dividend payment
management fee
service fee
director fee
contractor payment
software/licence fee
```

### 10.2 Checks

```text
domestic_withholding_rate
treaty_rate
exemption_possibility
recipient_residency
beneficial_ownership
certificate_of_residence
payment_category
gross_up_clause
filing_deadline
payment_deadline
documentation
```

### 10.3 Possible outcomes

```text
withhold domestic rate
apply treaty rate
apply exemption
request documents
gross-up warning
route adviser review
block payment until tax treatment confirmed
```

Example:

```text
This royalty payment may qualify for reduced withholding under treaty, but the certificate of residence is missing. I will request it before payment.
```

## 11. Permanent Establishment Risk Engine

Selene must detect whether business activity in another country risks creating taxable presence.

Signals:

```text
employees working abroad
salespeople closing contracts abroad
warehouse abroad
dependent agent
office/fixed place abroad
project site abroad
construction duration
local inventory
service delivery in-country
management/control activity
```

Outputs:

```text
PE risk low/medium/high
country involved
activity causing risk
tax registration review
adviser review
entity setup recommendation
```

Example:

```text
Your sales team is regularly closing contracts in Country X. This may create permanent establishment risk. I recommend tax review before expanding further.
```

## 12. Transfer Pricing And Arm's-Length Engine

Selene must monitor related-party and intercompany transactions.

### 12.1 Related-party transaction types

```text
intercompany sales
management fees
royalties
IP licensing
loans
interest
cost sharing
central services
distribution margins
manufacturing charges
shared employees
asset transfers
```

### 12.2 Selene checks

```text
related party identified
transaction type
amount
jurisdictions
pricing method
benchmark needed
local documentation requirement
master file/local file/CbCR trigger where applicable
economic substance
benefit test
contract support
invoice support
```

### 12.3 Outputs

```text
transfer pricing review required
arm's-length support needed
benchmark required
documentation packet
high-risk transaction alert
```

Selene must not shift profit without arm's-length support, substance, contracts, source evidence, and review.

Example:

```text
This intercompany royalty reduces tax in the high-tax entity, but needs arm's-length support, substance, contract evidence, and local documentation review.
```

## 13. Entity And Jurisdiction Planning Intelligence

Selene must research legal entity structures and jurisdiction possibilities.

### 13.1 Review factors

```text
tax rate
territorial vs worldwide tax
foreign tax credits
treaty network
withholding tax
substance requirements
CFC rules
management/control location
permanent establishment risk
Pillar Two exposure
transfer pricing
commercial purpose
banking/payment feasibility
regulatory requirements
employment/payroll obligations
VAT/GST/sales tax
local accounting/filing cost
reputation/risk
```

### 13.2 Possible outputs

```text
do nothing
register for tax in new country
use existing entity
create local subsidiary
use branch
use distributor/reseller model
use free zone if substance and qualifying income exist
route to adviser/board
reject as high-risk/artificial
```

Selene should never recommend jurisdiction planning based only on headline tax rate.

## 14. Zero / Low-Tax And Territorial Regime Intelligence

Selene must maintain an opportunity database for low-tax, territorial, free-zone, and incentive regimes, with risk and substance checks.

### 14.1 Regime categories

```text
zero or nominal corporate tax jurisdictions
territorial tax systems
foreign-sourced income exemptions
free zones
special economic zones
IP boxes
R&D incentive regimes
export incentives
holding company regimes
shipping/aviation regimes
finance/treasury regimes
startup incentives
```

### 14.2 Required warnings

```text
zero/low tax is not automatically safe
substance requirements must be checked
CFC rules must be checked
transfer pricing must be checked
anti-abuse rules must be checked
Pillar Two must be checked for large groups
reputation/commercial purpose must be checked
```

Example:

```text
This jurisdiction appears low-tax, but we need substance, control, transfer pricing, anti-abuse, and commercial-purpose checks before any strategy is viable.
```

## 15. Pillar Two / Global Minimum Tax Risk

For large multinational enterprise groups, Selene must check whether global minimum tax rules may apply.

Selene must track:

```text
group_revenue_threshold
jurisdictions
effective_tax_rate_by_jurisdiction
covered_taxes
GloBE_income
qualified_domestic_minimum_top_up_tax
IIR / UTPR exposure
safe_harbours
data_availability
board/tax_adviser_review
```

Example:

```text
This low-tax jurisdiction may not reduce group tax if Pillar Two applies. I recommend a global minimum tax exposure review before restructuring.
```

No Pillar Two rule, threshold, or rate may be hardcoded without an approved source-backed rule pack.

## 16. China Tax And Payroll Readiness Layer

Selene must be China-ready.

China-facing payroll and tax readiness must support:

```text
individual income tax withholding
annual individual income tax reconciliation
employee social insurance contributions
employer social insurance contributions
housing fund contributions
city-specific contribution bases and caps
foreign employee rules
social security treaty/exemption checks
payroll filings
tax payment deadlines
```

### 16.1 Employer and employee contribution model

Selene must store:

```text
employee_gross_salary
employee_IIT_withholding
employee_social_insurance_contribution
employer_social_insurance_contribution
housing_fund_employee_portion
housing_fund_employer_portion
city_contribution_base
monthly_cap/floor
foreign_employee_status
treaty/social_security_agreement_status
annual_reconciliation_status
```

Employees generally contribute employee portions; employers contribute employer portions. Selene must handle both through source-backed, city-specific, effective-dated rule packs.

### 16.2 China payroll optimization opportunities

Selene should detect lawful possibilities such as:

```text
correct use of statutory deductions
proper employee benefit classification
social security base accuracy
city-specific contribution compliance
foreign employee exemption/treaty possibilities where applicable
annual reconciliation planning
housing fund contribution optimization within legal limits
tax-efficient but compliant compensation structuring
```

Selene must not reduce China payroll tax by underreporting salary or contributions.

### 16.3 China readiness rule

```text
Do not hardcode rates.
Do not assume one China rate for every city.
Do not apply final payroll tax treatment without source-backed, city-specific, effective-dated rule packs.
```

Example:

```text
This employee is in Shanghai. I need city-specific social insurance and housing fund bases, IIT withholding rules, and residency status before final payroll tax treatment.
```

## 17. Deduction And Claimable Expense Optimizer

Selene must scan transactions for missed deductions and claimable expenses.

Sources:

```text
AP invoices
expense claims
bank transactions
credit card feeds
supplier statements
asset purchases
fleet expenses
insurance premiums
marketing spend
employee reimbursements
travel expenses
training costs
R&D spend
professional fees
bad debts
home office / mixed-use expenses where legally allowed
```

Selene checks:

```text
business purpose
evidence
tax invoice
claimability
private-use portion
capital vs expense
deduction timing
jurisdiction rule
threshold
review requirement
```

Example:

```text
This expense may be deductible, but the business purpose is missing. I will ask the user for a simple confirmation and attach it to the tax evidence.
```

## 18. Capital Allowance / Depreciation Optimizer

Selene must compare accounting depreciation and tax depreciation/capital allowance opportunities.

Inputs:

```text
asset_type
purchase_date
cost
business_use
jurisdiction
tax_depreciation_class
accelerated_allowance_availability
instant_write_off / expensing_rules where applicable
capital_improvement_vs_repair
disposal_plan
financing
```

Outputs:

```text
tax_deduction_timing
accelerated_deduction_opportunity
capital_vs_expense_review
repair_vs_improvement_review
cash_tax_impact
deferred_tax_impact if applicable
```

Example:

```text
This equipment purchase may qualify for accelerated tax deduction under current rules, but I need asset class confirmation before applying.
```

## 19. Bad Debt Relief Optimizer

Selene should automatically look for possible tax relief on bad debts.

Inputs:

```text
customer_invoice
tax_previously_remitted
collection_attempts
write_off_approval
insolvency_notice
time_elapsed
jurisdiction_rule
later_recovery_risk
```

Outputs:

```text
bad_debt_relief_candidate
evidence_checklist
period_to_claim
tax_adjustment_amount
recovery_reversal_reminder
```

Example:

```text
This customer debt was written off and tax was previously paid. I found a possible bad debt relief opportunity. I will prepare the evidence pack.
```

## 20. Incentive Discovery Engine

Selene must search for incentives.

Categories:

```text
R&D tax incentive
innovation grants
export incentives
training credits
investment allowances
green/energy credits
manufacturing incentives
free zone benefits
industry subsidies
hiring incentives
regional development incentives
digital transformation incentives
```

Workflow:

```text
detect qualifying activity
search approved sources
match eligibility
estimate benefit
list evidence required
risk score
prepare application/review pack
track deadline
```

Example:

```text
Your software development payroll and contractor costs may relate to R&D. I can prepare a review pack for incentive eligibility.
```

Selene must not claim incentives without source-backed eligibility and evidence.

## 21. Tax Deferral Engine

Selene should identify legal deferral opportunities.

Examples:

```text
timing of income recognition
timing of deductions
capital allowance timing
prepayment treatment
inventory valuation where permitted
installment/payment plan
tax payment scheduling
loss carryforward usage
deferred revenue treatment
bad debt timing
```

Selene must distinguish:

```text
permanent tax saving
temporary deferral
cashflow timing benefit
accounting timing difference
```

Example:

```text
This strategy does not permanently reduce tax, but it may defer cash tax for one period and improve liquidity.
```

## 22. Losses, Credits, And Carryforwards

Selene must track:

```text
tax losses
capital losses
foreign tax credits
R&D credits
investment credits
withholding credits
bad debt relief carried forward
unused deductions
expiry dates
usage restrictions
ownership-change restrictions
jurisdiction limits
```

Example:

```text
This entity has unused losses that may offset future taxable income, but ownership-change restrictions should be checked before relying on them.
```

## 23. Tax Strategy Review And Approval

Every optimization strategy has an approval flow.

```text
Detected
Researching
EvidenceNeeded
TaxReviewRequired
AdviserReviewRequired
ManagementApprovalRequired
BoardApprovalRequired
Approved
Rejected
Implemented
Monitored
Archived
```

Approval depends on:

```text
estimated tax saving
risk level
jurisdiction
materiality
anti-abuse risk
entity restructuring
board/shareholder reserved matters
cashflow impact
reputation risk
```

Selene may auto-implement only routine low-risk claims under approved policy.

Major strategies require human, adviser, management, board, or shareholder approval as policy requires.

Examples:

```text
Claiming this supplier input tax is routine and supported by a valid tax invoice. I will include it automatically.
```

```text
Moving IP income to a low-tax jurisdiction requires transfer pricing, substance, legal, and board review.
```

## 24. Continuous Tax Optimization Monitoring

Selene should monitor continuously.

Triggers:

```text
new country activity
new supplier/customer jurisdiction
large new asset purchase
employee hired in new country
cross-border payment
profit increasing
loss entity exists
new tax law update
new incentive announced
bad debt write-off
supplier invoice missing tax claim
large tax payable forecast
new free zone opportunity
foreign income received
```

Selene should say:

```text
You are now selling into Country X and holding inventory there. This may create tax registration or permanent establishment risk. I recommend review.
```

Or:

```text
The company is expected to pay high tax this quarter. I found possible claimable expenses and bad debt relief candidates worth review.
```

## 25. PH1.D / GPT-5.5 Role

GPT-5.5 should be used for research assistance, explanation, drafting, and pattern spotting.

### 25.1 GPT-5.5 may help

```text
search and summarize official tax guidance
compare jurisdiction summaries
draft tax adviser questions
draft tax opportunity explanations
summarize treaty articles in plain English
prepare tax strategy memos
explain risk factors
draft evidence checklists
translate tax authority guidance
prepare management/board tax summaries
```

### 25.2 GPT-5.5 must not

```text
invent tax law
approve tax strategy
submit tax filing
execute tax payment
create legal opinion
claim treaty relief without evidence
ignore anti-abuse rules
fabricate substance
decide transfer pricing
change accounting postings
```

GPT-5.5 is the research and explanation assistant. Deterministic Selene engines and human/adviser review own tax decisions.

## 26. Human-Like Selene Interaction

### Missed deduction

```text
I found three expenses that may be deductible but are missing business-purpose notes. I will ask the relevant users for simple confirmations.
```

### Treaty relief

```text
This payment may qualify for lower withholding under a treaty, but we need a certificate of residence and beneficial ownership evidence before applying it.
```

### China payroll

```text
This employee is in China. I need the city contribution base and housing fund rate before finalizing payroll tax treatment.
```

### Low-tax jurisdiction

```text
A free zone structure may reduce tax, but only if qualifying income, substance, and anti-abuse conditions are met. I will prepare the review pack.
```

### Transfer pricing

```text
This intercompany charge reduces tax in one entity and increases income in another. I recommend arm's-length support before posting.
```

### Tax deferral

```text
This does not permanently reduce tax, but it may defer payment and protect cash this quarter. I will prepare the timing analysis for review.
```

### Bad debt relief

```text
This written-off customer invoice may create a bad debt relief opportunity. I will gather collection evidence and route it for tax review.
```

### Incentive discovery

```text
Your export sales and product development work may qualify for incentive review. I will prepare the eligibility checklist and source references.
```

## 27. State Machines

### Tax Opportunity State

```text
Detected
Researching
EvidenceNeeded
NeedsReview
ApprovedForAction
Rejected
Implemented
Monitored
Expired
Archived
```

### Treaty Relief State

```text
Potential
ResidenceEvidenceNeeded
BeneficialOwnershipReview
PERiskReview
AntiAbuseReview
ReadyForAdviserReview
Approved
Rejected
Applied
Archived
```

### Jurisdiction Strategy State

```text
Candidate
Researching
SubstanceCheck
TransferPricingCheck
CFCCheck
PillarTwoCheck
LegalReview
BoardReview
Approved
Rejected
Implemented
Monitored
Archived
```

### China Payroll Tax Readiness State

```text
NotApplicable
ChinaEntityDetected
CityRulesNeeded
EmployeeDataNeeded
ContributionBaseSet
PayrollTaxReady
ExceptionReview
Archived
```

### Incentive State

```text
Potential
EligibilityResearch
EvidenceNeeded
ApplicationReady
Submitted
Approved
Rejected
Claimed
Expired
Archived
```

### Tax Deferral State

```text
Candidate
TimingAnalysis
EvidenceNeeded
ReviewRequired
Approved
Rejected
Applied
Reversed
Archived
```

## 28. Reason Codes

```text
TAX_OPTIMIZATION_OPPORTUNITY_DETECTED
TAX_DEDUCTION_CANDIDATE
CLAIMABLE_EXPENSE_EVIDENCE_MISSING
CAPITAL_ALLOWANCE_OPPORTUNITY
BAD_DEBT_RELIEF_OPPORTUNITY
FOREIGN_TAX_CREDIT_CANDIDATE
TREATY_RELIEF_CANDIDATE
CERTIFICATE_OF_RESIDENCE_REQUIRED
BENEFICIAL_OWNERSHIP_REVIEW_REQUIRED
WITHHOLDING_TAX_REDUCTION_CANDIDATE
PERMANENT_ESTABLISHMENT_RISK
TRANSFER_PRICING_REVIEW_REQUIRED
SUBSTANCE_REVIEW_REQUIRED
CFC_REVIEW_REQUIRED
PILLAR_TWO_REVIEW_REQUIRED
ZERO_TAX_JURISDICTION_REVIEW_REQUIRED
FREE_ZONE_QUALIFICATION_REVIEW
TERRITORIAL_TAX_REVIEW
CHINA_PAYROLL_TAX_RULE_REQUIRED
CHINA_SOCIAL_CONTRIBUTION_REVIEW
R_AND_D_INCENTIVE_CANDIDATE
EXPORT_INCENTIVE_CANDIDATE
TAX_DEFERRAL_CANDIDATE
ANTI_ABUSE_REVIEW_REQUIRED
TAX_ADVISER_REVIEW_REQUIRED
BOARD_TAX_STRATEGY_APPROVAL_REQUIRED
```

## 29. Required Simulations

```text
detect missed deductible expense
detect missing tax evidence
detect capital allowance opportunity
detect bad debt relief opportunity
detect foreign tax credit
detect treaty relief candidate
withholding tax reduction review
certificate of residence request
permanent establishment risk from overseas employee
transfer pricing review for intercompany service fee
zero/low-tax jurisdiction review
Hong Kong territorial source review
UAE free zone qualification review
Pillar Two exposure warning
China employee payroll tax readiness
China social insurance contribution setup
R&D incentive candidate
export incentive candidate
tax deferral scenario
tax adviser review pack
board approval for tax structure
reject high-risk treaty shopping strategy
```

## 30. Integration Map

```text
PH1.TAX.OPTIMIZE / PH1.TAX.INTELLIGENCE
↔ PH1.TAX / TAX_COMPLIANCE
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.BUDGET / PROFITABILITY
↔ PH1.PAYROLL
↔ PH1.AR / DEBTORS
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.BANKREC / TREASURY
↔ PH1.ASSET
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.LOGISTICS
↔ PH1.LEGAL / CONTRACTS
↔ PH1.BOARD
↔ PH1.SHAREHOLDER
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 31. Required Logical Packets

```text
TaxOptimizationOpportunityPacket
TaxResearchPacket
TaxDeductionCandidatePacket
ClaimableExpenseReviewPacket
CapitalAllowanceOpportunityPacket
BadDebtReliefOpportunityPacket
ForeignTaxCreditPacket
TreatyReliefPacket
WithholdingTaxReductionPacket
PermanentEstablishmentRiskPacket
TransferPricingReviewPacket
JurisdictionStrategyPacket
SubstanceReviewPacket
CFCReviewPacket
PillarTwoReviewPacket
FreeZoneReviewPacket
TerritorialTaxReviewPacket
ChinaPayrollTaxReadinessPacket
IncentiveDiscoveryPacket
TaxDeferralPacket
TaxAdviserReviewPacket
BoardTaxStrategyApprovalPacket
AuditEvidencePacket
```

Logical only. Codex maps later. This document does not create packet structs.

## 32. What Codex Must Not Do

```text
Do not merge Document 16 into Document 15.
Do not treat compliance filing as tax optimization.
Do not invent tax laws.
Do not hardcode jurisdiction rules without source-backed rule packs.
Do not treat zero-tax jurisdictions as automatically safe.
Do not ignore substance requirements.
Do not ignore treaty anti-abuse rules.
Do not ignore transfer pricing.
Do not ignore Pillar Two for large groups.
Do not apply China payroll/social contribution rules without city-specific rule pack.
Do not let GPT-5.5 approve tax strategy.
Do not let Selene implement tax structure without adviser/human/authority review.
Do not implement from this document alone.
```

## 33. Final Architecture Sentence

Selene Legal Tax Optimization + Treaty + Jurisdiction Intelligence Engine is the legal tax-reduction brain that continuously searches source-backed tax rules, treaties, credits, deductions, incentives, deferrals, capital allowances, bad debt relief, withholding relief, foreign tax credits, entity/jurisdiction strategies, China payroll/social contribution requirements, zero/low-tax and territorial regimes, substance obligations, transfer pricing, permanent establishment, CFC, anti-abuse, and Pillar Two risks to identify lawful tax-saving opportunities, prepare evidence and adviser review packs, reduce tax burden where legally supportable, and prevent Selene from becoming either a passive tax filing assistant or an overconfident tax strategy executor.

## 34. Simple Version

```text
Selene files tax correctly in Document 15.
Selene reduces tax legally in Document 16.
Selene searches deeply.
Selene finds deductions, credits, treaties, incentives, and deferrals.
Selene checks China, cross-border, zero/low-tax, free-zone, and territorial rules.
Selene checks substance, transfer pricing, anti-abuse, and Pillar Two.
Selene prepares evidence.
Humans/advisers approve material strategies.
Everything is audited.
```
