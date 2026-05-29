# Finance / Accounting Document 15 — Selene Tax Compliance + Filing Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
15

ENGINE:
PH1.TAX / PH1.TAX_COMPLIANCE

FULL NAME:
Selene Tax Compliance, Indirect Tax, Direct Tax Evidence, Payroll Tax Handoff, Filing Calendar, Tax Payment, and Regulatory Pack Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition. No runtime code, schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.TAX / PH1.TAX_COMPLIANCE. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Tax Compliance + Filing Engine owns the company's tax rule coordination, tax evidence, tax return preparation, filing readiness, tax payment obligations, and tax compliance calendar.

It answers:

```text
Which tax rules apply to this company?
Which jurisdictions does the company operate in?
Which tax registrations are active?
What tax codes apply to sales, purchases, payroll, assets, imports, exports, and adjustments?
What tax has been collected?
What tax is claimable?
What tax is payable?
What tax evidence is missing?
Which filings are due?
Which tax payments are due?
Which tax items require review?
Can Selene prepare the return pack?
Can Selene submit or route filing for approval?
```

This engine must be source-backed, jurisdiction-aware, versioned, auditable, and authority-controlled.

Tax is not a place for probabilistic invention. Selene may explain tax work humanly, but tax truth must come from approved jurisdiction rule packs, source evidence, deterministic checks, and authorized review.

## 2. Core Selene Law

```text
Selene must never invent tax truth.

Tax rules must be jurisdiction-scoped, source-backed, effective-dated, versioned, and auditable.

Routine tax classification and return preparation should be automated where rules are clear.

Unclear, high-risk, unusual, or material tax matters require review by the correct tax authority, accountant, adviser, or company approver.
```

Selene must reduce human work by:

```text
classifying routine transactions
detecting missing tax evidence
preparing filing packs
tracking deadlines
calculating draft tax positions
reconciling tax accounts
warning before tax cash outflows
preparing payment instructions
explaining tax issues clearly
```

Core compliance rules:

```text
no tax rule without jurisdiction scope
no final tax treatment without approved rule evidence
no filing treated as filed without filing proof
no tax payment treated as paid without BankRec proof
no tax rate invented by PH1.D or GPT-5.5
no tax exception hidden
no draft return treated as submitted
no compliance filing used as tax optimization without Document 16 handoff
```

## 3. Engine Boundary

### 3.1 PH1.TAX owns

```text
tax profile setup
tax registration tracking
jurisdiction rule packs
effective-dated tax rules
tax code registry
indirect tax calculation rules
tax evidence requirements
tax treatment proposals
tax return preparation
tax filing calendar
tax payment obligation tracking
tax account reconciliation support
bad debt tax relief tracking
import/export tax evidence
withholding tax evidence
payroll tax handoff review
asset tax evidence
tax adjustment workflow
tax audit pack
tax exception worklist
tax filing readiness
tax filing approval workflow
```

### 3.2 PH1.TAX does not own

```text
sales invoice creation
supplier invoice validation
payroll calculation
bank payment execution
ledger posting
product stock truth
asset physical lifecycle
legal advice final authority
tax adviser sign-off
board approval
cashflow ownership
legal tax optimization strategy
```

### 3.3 Correct owner split

```text
Sales/POS/E-Commerce/B2B = sales transaction source.
AR = customer invoice/receipt truth.
AP = supplier invoice/payable truth.
Payroll = payroll calculation and employee tax data.
Asset = asset lifecycle and asset evidence.
Accounting = ledger posting and tax control accounts.
Tax Engine = tax rules, tax evidence, filing readiness, tax calculations, tax packs.
Tax Optimization = lawful tax-saving opportunity research and strategy review.
Cashflow = tax payment impact and liquidity planning.
BankRec = proof of tax payment.
Access/Authority = who may approve, file, submit, or pay.
Audit = proof trail.
```

Tax does not create source transactions. Tax interprets tax treatment from source transactions.

## 4. Tax Types Selene Must Support

Selene must be able to support multiple tax categories depending on jurisdiction and company activity.

```text
GST / VAT / sales tax / indirect tax
use tax where applicable
withholding tax
payroll tax
employment taxes
super / pension / CPF or retirement contribution handoffs
income tax provision evidence
corporate tax evidence
fringe benefit / employee benefit tax evidence
capital gains tax evidence
asset tax depreciation evidence
customs and import duties
excise duties
property / land tax evidence
insurance premium taxes where applicable
digital services tax where applicable
environmental / ESG-linked taxes or levies where applicable
bad debt tax relief
tax credits / rebates / incentives handoff to Document 16 where optimization is involved
```

Selene detects relevant tax modules from:

```text
jurisdiction
legal entity type
sales channels
products/services sold
employees
payroll
asset ownership
imports/exports
customer locations
supplier locations
B2B/e-commerce activity
insurance/fleet/property activity
industry compliance
```

Selene starts simple and deepens only when source-backed requirements justify deeper tax handling.

## 5. Tax Profile Setup

Each legal entity has a Tax Profile.

```text
tax_profile_id
legal_entity_id
jurisdiction
tax_registration_number
tax_registration_status
tax_types_active
filing_frequency
filing_basis
tax_periods
tax_authority_references
tax_representative / accountant
tax_payment_account
tax_filing_authority_owner
tax_review_owner
currency
effective_date
status
audit_ref
```

Tax Profile statuses:

```text
Draft
PendingInformation
PendingVerification
Active
Suspended
Deregistered
Archived
```

Selene can capture tax details from:

```text
tax registration certificate
business licence
government letter
invoice
supplier/customer tax document
payroll setup file
accountant upload
company onboarding document
```

Example:

```text
Upload your tax registration document and I will extract the registration number, jurisdiction, and filing frequency for confirmation.
```

Extraction is a proposal until confirmed by an authorized owner or approved source rule.

## 6. Jurisdiction Rule Packs

Tax rules must be stored as jurisdiction rule packs.

Each rule pack includes:

```text
jurisdiction
tax_type
rule_version
effective_date
expiry_date
source_authority_reference
tax_rates
tax_categories
exemptions
zero_rated_rules
reverse_charge_rules
place_of_supply_rules where applicable
claimability_rules
document_evidence_requirements
filing_boxes / return_fields
rounding_rules
thresholds
review_requirements
audit_ref
```

Rule pack statuses:

```text
Draft
PendingReview
Approved
Active
Superseded
Retired
```

Rule pack law:

```text
No source-backed or owner-approved rule = no final tax truth.
```

If a rule is missing, Selene says:

```text
I do not have an approved tax rule for this treatment. I can flag it for tax review.
```

## 7. Tax Code Registry

Selene needs a tax code registry.

Tax code record:

```text
tax_code_id
jurisdiction
tax_type
code
description
rate
claimable_percentage
output/input_classification
effective_date
reporting_box
accounting_mapping
default_transaction_types
review_required
status
audit_ref
```

Examples of tax code types:

```text
standard-rated sale
zero-rated sale
exempt sale
out-of-scope sale
standard input tax
non-claimable input tax
partially claimable input tax
reverse charge
import tax
withholding tax
tax-free payroll item
taxable employee benefit
bad debt relief adjustment
capital asset tax adjustment
```

Selene should not force humans to choose cryptic codes when a guided question is safer.

Example:

```text
Was this purchase used for business, private, mixed, or exempt activity?
```

Selene proposes the tax code. Humans confirm only where policy requires.

## 8. Tax Classification Pipeline

Selene classifies tax from source transactions.

Pipeline:

```text
source transaction created
-> source owner sends tax classification request
-> Tax Engine identifies jurisdiction and tax type
-> Tax Engine applies effective-dated rule pack
-> Tax Engine proposes tax code/treatment
-> source engine records proposed treatment
-> Accounting receives posting mapping
-> exceptions route to tax review
-> lawful optimization candidates route to Document 16
-> Audit records evidence
```

Source transactions include:

```text
POS sale
e-commerce sale
B2B invoice
customer credit note
supplier invoice
supplier credit note
payroll run
employee benefit
asset purchase
asset sale
vehicle/fleet expense
insurance premium
import/export transaction
intercompany transaction
bank fee/interest
bad debt write-off/recovery
```

Selene should auto-classify routine transactions where the rule pack is clear.

Selene should route unusual, material, mixed-use, cross-border, or unsupported items to tax review.

Example:

```text
This supplier invoice looks like a vehicle repair. It is business-related and tax appears claimable under the current profile, but the vehicle has mixed-use status, so I am sending it for review.
```

## 9. Sales Tax / GST / VAT / Indirect Tax

Selene must manage indirect tax across sales and purchases.

### 9.1 Output tax

Sources:

```text
POS sales
e-commerce sales
B2B invoices
service invoices
subscription invoices
asset sales
deposits where taxable
credit note reversals
```

Selene calculates:

```text
net_amount
tax_rate
gross_amount
tax_code
jurisdiction
reporting_box
transaction_date / tax_point
rule_pack_version
```

### 9.2 Input tax

Sources:

```text
supplier invoices
expense claims
asset purchases
imports
subscriptions
repairs/maintenance
insurance where applicable
```

Selene checks:

```text
valid tax invoice
supplier tax ID if required
business purpose
claimability
private/mixed use
exempt activity restrictions
document evidence
period eligibility
rule_pack_version
```

### 9.3 Credit notes and adjustments

Tax must adjust for:

```text
supplier credit note
customer credit note
refund
return
discount after sale
damaged goods adjustment
invoice cancellation
bad debt write-off/recovery
```

Selene must link each tax adjustment to the original transaction.

## 10. Payroll Tax And Employment Tax Handoff

Payroll calculates payroll.

Tax Engine validates tax compliance evidence.

Payroll sends:

```text
gross_wages
tax_withheld
employer_tax_or_contribution_obligations
employee_tax_categories
benefits
leave/final_pay
pension/super/CPF contributions
jurisdiction
period
```

Tax Engine checks:

```text
tax rule pack exists
filing/payment period
liability accounts
submission due date
payment due date
missing employee tax data
benefit tax review
payroll tax handoff completeness
```

Boundary:

```text
Payroll owns employee pay.
Tax owns tax rule compliance.
Accounting posts liabilities.
BankRec proves payment.
```

Tax Engine does not calculate payroll from scratch unless Payroll owner delegates a tax calculation component through approved source-backed rules.

## 11. Withholding Tax

Selene supports withholding tax where applicable.

Trigger sources:

```text
supplier payment
contractor payment
royalty
interest
dividend
cross-border service
rent
professional fee
```

Checks:

```text
supplier jurisdiction
tax treaty/evidence if applicable
withholding rate
gross vs net payment
withholding certificate
filing due date
payment due date
supplier remittance statement
Document 16 optimization handoff if reduction/treaty strategy is possible
```

Example:

```text
This cross-border supplier payment may require withholding tax. I have no approved rule for this supplier category, so I am routing it for tax review before payment.
```

## 12. Import, Export, Customs, And Duties

If the company imports or exports, Selene tracks:

```text
customs declaration
import VAT/GST
duties
tariff/classification evidence
shipping documents
landed cost
supplier country
destination country
customs broker
duty payment
claimability
```

Selene connects with:

```text
Procurement
Supplier
Inventory
Logistics
Accounting
Tax
```

Tax Engine handles tax/duty evidence and filing treatment. Inventory handles stock truth. Accounting handles landed cost posting. Logistics handles shipment truth.

## 13. Asset Tax, Depreciation, And Capital Gains Evidence

Asset Engine owns asset lifecycle.

Accounting owns financial depreciation.

Tax Engine tracks tax evidence:

```text
asset_purchase_date
cost_base
tax_depreciation_category
business/private_use
capital_improvements
disposal_date
sale_proceeds
tax_depreciation_claimed
balancing_adjustment
capital_gain_or_loss_evidence
claimable_vs_non_claimable_portions
```

Selene may say:

```text
This vehicle has mixed business/private use. I can prepare tax evidence, but the claim percentage needs approved policy or adviser review.
```

Tax Engine prepares evidence and routes review. Document 16 owns optimization analysis for claim strategy, accelerated allowances, treaty, jurisdiction, or broader legal tax reduction.

## 14. Bad Debt Tax Relief

If a debtor becomes bad debt, Tax Engine checks possible relief.

Inputs:

```text
customer_invoice
tax_previously_reported
write_off_approval
collection_attempts
insolvency_evidence
payment_later_recovered
jurisdiction_rule_pack
```

Outputs:

```text
bad_debt_relief_candidate
tax_adjustment_period
required_evidence
recovery_adjustment_if_later_paid
review_status
Document 16 optimization handoff if a lawful tax-saving opportunity requires strategy review
```

Example:

```text
This written-off customer invoice may qualify for tax relief if the jurisdiction rule is satisfied. I will prepare the evidence pack for review.
```

## 15. Tax Filing Calendar

Selene maintains tax filing and payment deadlines.

Calendar includes:

```text
tax_type
jurisdiction
filing_period
filing_due_date
payment_due_date
reminder_schedule
responsible_owner
filing_status
payment_status
evidence_status
```

Statuses:

```text
Upcoming
DataCollecting
DraftPrepared
ReviewRequired
ReadyForApproval
Approved
Filed
PaymentScheduled
Paid
Overdue
Archived
```

Example:

```text
The Q2 GST return is due in 10 days. Bank reconciliation and AP are ready. AR has two unmatched receipts that may affect the return.
```

## 16. Tax Return Preparation

Tax return lifecycle:

```text
PeriodOpen
DataCollecting
DraftCalculating
DraftPrepared
ExceptionReview
ReadyForReview
ReadyForApproval
Approved
Submitted
Accepted
PaymentScheduled
Paid
Closed
Archived
```

Return pack includes:

```text
period
jurisdiction
tax_type
sales/output_tax
purchases/input_tax
adjustments
credit_notes
bad_debt_relief
imports/exports
payroll_tax if applicable
withholding_tax if applicable
balance_payable_or_refundable
source_transaction_list
exceptions
review_notes
approval_proof
filing_proof
payment_proof
audit_ref
```

Selene should prepare draft returns automatically once source data is ready.

Humans review exceptions and approve final filing.

## 17. Tax Exception Worklist

Selene routes only unresolved tax issues.

Exception types:

```text
missing tax invoice
missing supplier tax ID
unknown tax code
mixed-use purchase
private-use component
cross-border service
possible withholding
reverse charge review
large tax adjustment
bad debt relief review
asset disposal review
tax rate mismatch
unusual refund
filing box mismatch
missing rule pack
```

Exception fields:

```text
exception_id
source_transaction
tax_type
jurisdiction
amount
reason
risk_level
owner
recommended_action
status
audit_ref
```

Example:

```text
There are four tax exceptions. Three are missing supplier tax invoices. One cross-border service payment needs withholding review.
```

## 18. Tax Payments

Tax Engine identifies tax payable.

Payment execution belongs to Banking/Payment.

Flow:

```text
Tax Engine calculates payable/refundable position
-> Finance/Cashflow reserves cash
-> Access/Authority approves filing/payment if required
-> Payment Engine schedules tax payment
-> BankRec confirms payment cleared
-> Accounting records payment
-> Tax Engine stores filing/payment proof
```

Tax Engine does not move money. It prepares tax obligation and payment schedule.

Example:

```text
The tax return shows 18,200 payable. I have reserved it in the cash forecast and prepared the payment instruction for approval.
```

## 19. Tax Audit Pack

Selene should maintain regulator-ready evidence.

Audit pack includes:

```text
return copy
source transaction list
tax codes used
rule pack version
sales reports
purchase reports
credit notes
bad debt evidence
import/export documents
payroll tax evidence
withholding evidence
asset tax evidence
bank payment proof
filing receipt
review approvals
exception resolutions
audit_hash/reference
```

Selene should be able to answer:

```text
Show the evidence behind the Q3 VAT return.
```

And produce the regulator-ready pack from source evidence.

## 20. Automation And Exception-Only Review

Selene auto-handles:

```text
routine sales tax classification
routine supplier input tax classification
standard credit note adjustments
routine tax calendar reminders
draft return preparation
tax account reconciliation support
evidence pack assembly
payment due reminders
```

Selene escalates:

```text
new jurisdiction
new tax registration
missing rule pack
unusual transaction
large adjustment
mixed-use items
withholding tax
cross-border supply
asset disposal
bad debt relief
tax refund anomaly
late filing risk
manual override
possible lawful tax-saving strategy requiring Document 16 review
```

Routine tax should not become human busywork. High-risk tax should not become automated guesswork.

## 21. PH1.D / GPT-5.5 Role

GPT-5.5 should be used for explanation, classification assistance, and drafting.

### 21.1 GPT-5.5 may help

```text
explain tax exceptions in plain English
summarize filing pack
draft accountant review notes
draft supplier request for valid tax invoice
draft customer tax adjustment explanation
summarize tax payment timeline
translate tax authority messages
classify messy invoice descriptions as proposals
prepare management tax summary
```

### 21.2 GPT-5.5 must not

```text
invent tax rules
set tax rates without source-backed rule pack
approve tax filing
submit tax return
execute tax payment
override tax exceptions
decide final legal tax treatment
hide tax risk
post ledger
apply Document 16 optimization strategy without source-backed review and authority
```

GPT-5.5 can make tax understandable. It cannot make tax true.

## 22. Human-Like Selene Interaction

### Tax profile setup

```text
Upload your tax registration certificate. I will read the registration number and filing frequency, then ask you to confirm.
```

### Missing evidence

```text
This supplier invoice may not be claimable yet because the tax invoice number is missing. I will request a corrected invoice.
```

### Return preparation

```text
The Q2 GST return is drafted. Two purchase invoices need review before the return can be approved.
```

### Tax payment

```text
The return shows 18,200 payable. I have added it to cashflow and prepared the payment for approval.
```

### Bad debt relief

```text
This written-off customer invoice may qualify for tax relief, but the rule requires evidence of recovery attempts. I will prepare the evidence pack.
```

### Optimization handoff

```text
The draft return shows a larger tax payable than usual. I found possible lawful tax-saving review candidates and routed them to Tax Optimization for source-backed research.
```

## 23. State Machines

### Tax Profile State

```text
Draft
PendingInformation
PendingVerification
Active
Suspended
Deregistered
Archived
```

### Tax Rule Pack State

```text
Draft
PendingReview
Approved
Active
Superseded
Retired
```

### Tax Return State

```text
PeriodOpen
DataCollecting
DraftCalculating
DraftPrepared
ExceptionReview
ReadyForReview
ReadyForApproval
Approved
Submitted
Accepted
PaymentScheduled
Paid
Closed
Archived
```

### Tax Exception State

```text
Open
EvidenceRequested
UnderReview
Resolved
Escalated
WaivedUnderPolicy
Closed
```

### Tax Payment State

```text
NotDue
DueSoon
ReadyForPayment
Scheduled
Paid
Failed
Overdue
Closed
```

### Tax Compliance To Optimization Handoff State

```text
NotApplicable
CandidateDetected
OpportunityPacketPrepared
RoutedToTaxOptimization
OptimizationReviewInProgress
ReturnedToCompliance
Closed
```

## 24. Reason Codes

```text
TAX_PROFILE_CREATED
TAX_REGISTRATION_MISSING
TAX_RULE_PACK_MISSING
TAX_RULE_PACK_ACTIVE
TAX_CODE_APPLIED
TAX_CODE_REVIEW_REQUIRED
TAX_INVOICE_MISSING
SUPPLIER_TAX_ID_MISSING
OUTPUT_TAX_CALCULATED
INPUT_TAX_CLAIMABLE
INPUT_TAX_NOT_CLAIMABLE
PARTIAL_CLAIM_REVIEW_REQUIRED
WITHHOLDING_REVIEW_REQUIRED
REVERSE_CHARGE_REVIEW_REQUIRED
IMPORT_TAX_DETECTED
BAD_DEBT_RELIEF_CANDIDATE
ASSET_TAX_REVIEW_REQUIRED
TAX_RETURN_DRAFTED
TAX_RETURN_EXCEPTION_OPEN
TAX_RETURN_READY_FOR_APPROVAL
TAX_RETURN_SUBMITTED
TAX_PAYMENT_SCHEDULED
TAX_PAYMENT_CONFIRMED
TAX_FILING_OVERDUE
TAX_OPTIMIZATION_OPPORTUNITY_ROUTED
TAX_COMPLIANCE_TO_OPTIMIZATION_HANDOFF_CREATED
```

## 25. Required Simulations

```text
create tax profile from registration document
apply standard output tax to POS sale
apply tax to e-commerce sale
apply tax to B2B invoice
classify supplier input tax
detect missing tax invoice
detect missing supplier tax ID
process customer credit note tax adjustment
process supplier credit note tax adjustment
prepare GST/VAT/sales tax return
detect tax return exception
bad debt relief candidate
withholding tax review
asset purchase tax treatment review
asset sale tax evidence
import tax evidence
tax payment scheduling
tax payment confirmed by BankRec
tax audit pack generation
tax rule pack missing
tax filing overdue alert
tax optimization opportunity routed to Document 16
```

## 26. Integration Map

```text
PH1.TAX / PH1.TAX_COMPLIANCE
↔ PH1.TAX.OPTIMIZE / PH1.TAX.INTELLIGENCE
↔ PH1.ACCOUNTING / GL
↔ PH1.AR / DEBTORS
↔ PH1.CREDITORS / AP
↔ PH1.PAYROLL
↔ PH1.BANKREC / TREASURY
↔ PH1.CASHFLOW
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.POS
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.ASSET
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.LOGISTICS
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 27. Required Logical Packets

```text
TaxProfilePacket
TaxRegistrationEvidencePacket
TaxRulePackPacket
TaxCodePacket
TaxClassificationRequestPacket
TaxClassificationResultPacket
OutputTaxPacket
InputTaxPacket
TaxCreditNoteAdjustmentPacket
BadDebtTaxReliefPacket
WithholdingTaxReviewPacket
ImportTaxPacket
AssetTaxEvidencePacket
PayrollTaxHandoffPacket
TaxReturnPacket
TaxReturnExceptionPacket
TaxFilingCalendarPacket
TaxPaymentObligationPacket
TaxAuditPackPacket
TaxAuthoritySubmissionPacket
TaxOptimizationOpportunityPacket
TaxComplianceToOptimizationHandoffPacket
AuditEvidencePacket
```

Logical only. Codex maps later. This document does not create packet structs.

## 28. Addendum A — Boundary Between Tax Compliance And Legal Tax Optimization

```text
DOCUMENT TYPE:
DOCUMENT 15 ADDENDUM

ADDENDUM:
A — Boundary Between Tax Compliance and Legal Tax Optimization

STATUS:
INCLUDED_IN_FINANCE_ACCOUNTING_DOCUMENT_15
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
```

### 28.1 Purpose

Document 15 owns tax compliance, filing, calendars, evidence, tax return packs, tax payment obligations, and tax audit packs.

Document 16 owns legal tax optimization strategy.

This boundary prevents the compliance engine from becoming the owner of treaty strategy, jurisdiction structuring, transfer pricing strategy, low-tax regime review, or broader tax planning.

### 28.2 Boundary Addition

Document 15 / PH1.TAX owns:

```text
tax profiles
tax registrations
tax rule packs
tax codes
routine tax classification
indirect tax
payroll tax handoff
tax return preparation
tax calendars
tax payment obligations
filing packs
tax audit packs
tax exception worklists
```

Document 16 / PH1.TAX.OPTIMIZE owns:

```text
legal tax minimization
tax deferral
deduction optimization
claimable expense optimization
capital allowance / depreciation optimization
bad debt relief optimization
treaty relief
withholding reduction
foreign tax credits
jurisdiction intelligence
zero/low-tax jurisdiction review
free zone / territorial regime review
China payroll/social contribution optimization readiness
permanent establishment risk
transfer pricing review prompts
substance checks
CFC / anti-abuse checks
Pillar Two risk prompts
incentive discovery
tax adviser review packets
```

### 28.3 Core Law Addition

Tax Compliance must not be passive.

When tax filings or tax calculations reveal a possible lawful tax-saving opportunity, Document 15 must generate a TaxOptimizationOpportunityPacket and route it to Document 16 / PH1.TAX.OPTIMIZE.

```text
Document 15 prepares and files correctly.
Document 16 investigates how to legally reduce, defer, credit, exempt, or optimize the tax burden.
```

Example:

```text
Document 15 detects large tax payable.
Document 15 calculates the draft return.
Document 16 searches for credits, deductions, bad debt relief, foreign tax credit, treaty relief, or deferral opportunities.
```

### 28.4 Integration Map Addition

```text
PH1.TAX / TAX_COMPLIANCE
↔ PH1.TAX.OPTIMIZE / TAX.INTELLIGENCE
```

### 28.5 Logical Packet Additions

```text
TaxOptimizationOpportunityPacket
TaxComplianceToOptimizationHandoffPacket
```

### 28.6 Addendum Rule

```text
Document 15 calculates and files.
Document 16 legally reduces and optimizes.
Document 15 finds tax pressure.
Document 16 researches lawful tax-saving strategy.
```

Document 15 remains the tax compliance and filing engine, while Document 16 becomes the legal tax optimization and jurisdiction intelligence engine. Whenever compliance work detects possible lawful tax savings, Document 15 hands the opportunity to Document 16 for source-backed research, risk scoring, adviser/human review, and approval before implementation.

## 29. What Codex Must Not Do

```text
Do not invent tax rules.
Do not hardcode tax rates without approved rule pack.
Do not let GPT-5.5 decide final tax treatment.
Do not let Tax create sales or supplier invoice truth.
Do not let Tax calculate payroll without Payroll owner.
Do not let Tax execute payments.
Do not let Tax post ledger directly.
Do not submit tax returns without authority.
Do not hide tax exceptions.
Do not treat draft returns as filed.
Do not treat Document 15 as the legal tax optimization engine.
Do not bury treaty, free-zone, transfer pricing, jurisdiction structuring, or tax planning strategy inside Document 15.
Do not allow Document 15 to apply tax-saving strategies without Document 16 review where strategy is material, cross-border, high-risk, or adviser-controlled.
Do not implement from this document alone.
```

## 30. Final Architecture Sentence

Selene Tax Compliance + Filing Engine is the jurisdiction-aware tax compliance brain that manages tax profiles, tax registrations, source-backed rule packs, effective-dated tax codes, routine tax classification, indirect tax, withholding review, payroll tax handoffs, asset tax evidence, bad debt relief, import/export tax evidence, tax calendars, tax return preparation, exception worklists, tax payment obligations, filing readiness, and regulator-ready audit packs, while using GPT-5.5 to explain and draft human-friendly tax communication but never allowing probabilistic output to invent tax law, approve filings, execute payments, post ledger, hide exceptions, or apply legal tax optimization strategies without Document 16 handoff.

## 31. Simple Version

```text
Selene knows the company's tax profile.
Selene uses approved tax rules.
Selene classifies routine tax automatically.
Selene flags risky tax issues.
Selene prepares tax returns.
Selene tracks filing deadlines.
Selene prepares tax payment obligations.
Selene stores evidence.
Selene routes possible lawful tax-saving opportunities to Document 16.
Humans approve filings and exceptions.
Everything is audited.
```
