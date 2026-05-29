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
