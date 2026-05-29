# Finance / Accounting Batch 7 — Documents 13–14 Budget, Spend Governance, Period Close + Reporting

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

- Finance / Accounting Document 13 — Budget, Cost Center, Spend Governance + Profitability
- Finance / Accounting Document 14 — Period Close + Financial Reporting

This batch mechanically consolidates the previously accepted standalone source documents below. Architecture content is preserved verbatim between source-file markers.

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_13_BUDGET_COST_CENTER_SPEND_GOVERNANCE_PROFITABILITY_MASTER_DESIGN.md -->
# Finance / Accounting Document 13 — Selene Budget, Cost Center, Spend Governance + Profitability Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
13

ENGINE:
PH1.BUDGET / PH1.PROFITABILITY / PH1.SPEND_GOV

FULL NAME:
Selene Budget, Cost Center, Spend Governance, Forecast Variance, and Profitability Control Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition.

No runtime code is implemented by this document.

No schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.BUDGET / PH1.PROFITABILITY / PH1.SPEND_GOV. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Budget + Profitability Engine owns the company's planned money, spend control, cost center discipline, and profitability intelligence.

It answers:

```text
What is the approved budget?
Who owns the budget?
How much has been spent?
How much is committed but not yet invoiced?
How much is forecast to be spent?
Is this purchase inside budget?
Is this department overspending?
Is this customer profitable?
Is this product profitable?
Is this supplier cost rising?
Is this project losing money?
Should Selene allow, warn, block, or escalate the spend?
Should the company revise budget or cut/redirect spending?
```

Accounting records actual financial truth.

Cashflow forecasts available cash.

Budget decides whether spending is planned, controlled, justified, and affordable within company governance.

Important distinction:

```text
Cashflow asks: Can we pay?
Budget asks: Should this spend exist under the approved plan?
Accounting asks: What actually happened?
```

A company may have cash and still be over budget.

A company may be within budget and still lack cash.

Budget availability is not the same as cash availability.

Cashflow safety is not the same as budget approval.

## 2. Core Selene Law

```text
Selene must manage budgets continuously, not once a year.

Routine in-budget spend should flow automatically under policy.
Out-of-budget, high-risk, unusual, strategic, or material spend should escalate.
Selene should detect budget drift early, explain it clearly, and recommend corrective action before humans discover the damage at month-end.
```

Selene's job is not to make managers beg for every small purchase.

Selene's job is to:

```text
know the budget
know the commitments
know the actuals
know the forecast
know the authority
know the risk
act automatically where safe
escalate only where needed
```

Routine in-budget spend should not create pointless approval noise.

Committed spend must be tracked before invoices arrive.

## 3. Engine Boundary

### 3.1 PH1.BUDGET / PH1.SPEND_GOV Owns

```text
budget creation workflow
budget versions
budget lines
department budgets
cost center budgets
project budgets
branch/location budgets
campaign budgets
capex budgets
opex budgets
budget ownership
budget approval state
budget lock/freeze
budget revision workflow
committed spend tracking
budget vs actual reporting
budget vs forecast reporting
spend limit checks
spend governance rules
over-budget detection
variance analysis
profitability reporting
margin performance intelligence
```

### 3.2 PH1.BUDGET / PH1.SPEND_GOV Does Not Own

```text
bank balance proof
cash movement
supplier invoice validation
customer invoice creation
ledger posting
tax law
payment execution
product stock truth
supplier qualification
employee payroll calculation
```

### 3.3 Correct Owner Split

```text
Accounting = actual ledger truth.
Cashflow = future liquidity and cash pressure.
Budget = planned spend, commitments, limits, and governance.
Access/Authority = who may approve exceptions.
Procurement = purchase order creation.
AP = supplier invoice payable truth.
AR = customer invoice/receipt truth.
Payroll = employee pay calculation.
Product/Inventory = product and stock truth.
Profitability = business performance analysis using Accounting + Operational data.
```

Budget is the financial traffic controller.

It does not drive the trucks, post the books, or move the cash.

It tells Selene whether the road is approved.

## 4. Budget Is Not One Thing

Selene must support many budget types.

```text
annual budget
quarterly budget
monthly budget
rolling budget
department budget
branch/location budget
cost center budget
project budget
campaign budget
payroll budget
inventory purchasing budget
capex budget
opex budget
supplier budget
customer acquisition budget
R&D budget
fleet budget
maintenance budget
insurance budget
tax reserve budget
board-approved strategic fund
```

Small companies may begin with a simple budget.

Large enterprises may need deep budget structures.

Selene must start simple and deepen as needed.

## 5. Budget Object Model

### 5.1 Budget

```text
budget_id
tenant_id
legal_entity_id
budget_name
budget_type
budget_period
currency
version
status
owner
approval_policy
created_by
approved_by
locked_by
source_assumptions
linked_board_resolution
linked_governance_rule
audit_ref
```

### 5.2 Budget Line

```text
budget_line_id
budget_id
account_code
category
department
cost_center
project
location
supplier_group
customer_group
product_group
period
budget_amount
committed_amount
actual_amount
forecast_amount
available_amount
variance_amount
variance_percent
status
```

### 5.3 Cost Center

```text
cost_center_id
name
owner
department
legal_entity
location
budget_policy
approval_threshold
active_status
reporting_group
```

### 5.4 Commitment

A commitment is approved spend not yet fully invoiced or paid.

Examples:

```text
purchase order issued
contract signed
campaign approved
payroll hire approved
asset purchase approved
supplier retainer approved
lease commitment
subscription renewal
```

Commitment record:

```text
commitment_id
source_engine
source_ref
budget_line_id
amount
currency
period
status
expected_invoice_date
expected_payment_date
supplier/customer/employee_ref
audit_ref
```

Budget must track commitments so managers cannot confuse "not invoiced yet" with "not committed."

## 6. Budget Lifecycle

```text
Draft
Preparing
PendingReview
PendingApproval
Approved
Active
Locked
Frozen
RevisionRequested
Revised
Superseded
Closed
Archived
```

### 6.1 Draft

Selene prepares a draft budget from:

```text
historical actuals
known contracts
payroll plans
inventory needs
supplier price trends
customer revenue forecasts
marketing plans
asset maintenance
tax schedules
loan obligations
business growth assumptions
```

### 6.2 Review

Budget owners review only material assumptions.

Selene should say:

```text
I built the draft from last year’s actuals, current payroll, supplier contracts, and planned growth. I only need confirmation on three assumptions.
```

Not:

```text
Please manually complete 400 budget cells.
```

### 6.3 Approval

Approval may require:

```text
department manager
finance manager
CFO
CEO
board
shareholder approval for reserved matters
```

### 6.4 Active

Selene tracks actuals, commitments, and forecast continuously.

### 6.5 Revision

Budget revision occurs when:

```text
business model changes
major customer lost
supplier costs increase materially
cashflow pressure changes
new branch opens
major asset purchase approved
board approves change
economic conditions shift
```

### 6.6 Closed

Budget closes after period end and actual variance is finalized.

## 7. Budget Creation Methods

Selene should support multiple budget methods.

### 7.1 Historical Budget

Based on prior actuals.

```text
last year/month actuals
adjusted for growth/inflation/seasonality
```

### 7.2 Zero-Based Budget

Every line justified from zero.

Useful for cost control.

Selene asks:

```text
Do you still need this spend, or is it just living here because last year’s budget had it?
```

### 7.3 Driver-Based Budget

Based on operational drivers.

Examples:

```text
sales volume
headcount
store count
production units
customers served
orders shipped
machine hours
marketing leads
```

### 7.4 Rolling Budget

Continuously updated.

```text
actuals replace forecast
future months roll forward
```

### 7.5 Scenario Budget

```text
base
optimistic
pessimistic
cash rescue
growth
cost reduction
```

Selene must allow companies to use simple budgets first, then mature into better budgeting.

## 8. Budget Vs Actual

Budget vs actual compares:

```text
approved budget
actual spend/revenue
committed spend
forecast spend
variance
variance reason
```

Selene should not merely show variance.

Selene must explain why.

Variance causes:

```text
supplier price increase
higher sales volume
lower sales volume
unexpected repair
payroll increase
tax change
inventory overbuy
campaign overspend
foreign exchange
timing difference
missing invoice
misclassified transaction
```

Selene says:

```text
Marketing is 18% over budget mainly because two campaigns were approved in the same month and supplier invoice timing shifted earlier than planned.
```

A number is not enough. Selene explains the business reason.

## 9. Committed Spend Control

Selene must check spend before it becomes an invoice.

Spend requests come from:

```text
Procurement
Purchase Orders
Inventory reorder
Marketing campaign
HR hiring
Payroll increase
Asset purchase
Fleet repair
Insurance renewal
Subscription renewal
Contract approval
Travel/expense request
```

Budget check formula:

```text
Available Budget
= Approved Budget
- Actual Spend
- Committed Spend
- Known Forecast Obligations
```

If request fits:

```text
allow / auto-route under policy
```

If request exceeds:

```text
recommend reduction
recommend split
recommend delay
request budget transfer
escalate exception
block if policy says
```

Selene says:

```text
This purchase is within the warehouse supplies budget and below Tom’s authority limit. I’ll approve it under policy.
```

Or:

```text
This purchase exceeds the remaining budget by $1,200. I recommend delaying it or requesting a budget transfer.
```

This is how Selene avoids invoices becoming surprises.

## 10. Spend Governance

Spend governance controls:

```text
who can request spend
who can approve spend
what categories are allowed
what limits apply
which department/cost center pays
what requires dual approval
what requires board approval
what can be auto-approved
what must be blocked
```

### 10.1 Routine Auto-Spend

Examples:

```text
cleaning supplies under threshold
recurring subscriptions within contract
approved stock reorder
approved payroll run
approved utilities
approved rent
```

Selene should process under policy.

### 10.2 Exception Spend

Examples:

```text
over budget
wrong category
new supplier
supplier restricted
new asset
large campaign
high-risk payment
board-level threshold
```

Selene escalates.

### 10.3 No Approval Noise

Routine spend should not require endless approval.

Selene's rule:

```text
If the budget allows it,
and authority allows it,
and policy allows it,
and risk is normal,
Selene acts.
```

Humans handle judgment, not button-clicking.

## 11. Budget Transfer And Reallocation

Sometimes money must move between budget lines.

Selene supports:

```text
budget transfer
budget reallocation
budget reserve release
department-to-department transfer
project budget adjustment
capital-to-operating restriction checks
```

Selene checks:

```text
source budget available
destination budget need
authority
policy
board/shareholder reserved matters if relevant
cashflow impact
audit trail
```

Selene says:

```text
Operations has $8,000 unused budget, while Warehouse is short $3,000 due to supplier price increases. I can prepare a budget transfer request.
```

Selene may auto-transfer only if policy allows.

Otherwise she prepares the request.

## 12. Profitability Engine

Profitability is not only company-level profit.

Selene must support:

```text
company profitability
department profitability
branch/location profitability
project profitability
customer profitability
product profitability
service profitability
supplier profitability impact
channel profitability
campaign profitability
B2B profitability
POS profitability
e-commerce profitability
contract profitability
```

### 12.1 Profitability Formula

Basic model:

```text
Revenue
- direct cost
- allocated cost
- delivery cost
- payment fees
- returns/refunds
- discounts
- commissions/profitshare
= contribution / profit
```

More advanced models may include:

```text
labor cost
overhead allocation
inventory holding cost
customer service cost
marketing acquisition cost
bad debt risk
warranty cost
```

### 12.2 Product Profitability

Selene uses:

```text
sell price
cost of goods
supplier cost
landed cost
delivery
returns
payment fees
discounts
B2B profitshare
stockholding cost
expiry/waste
```

### 12.3 Customer Profitability

Selene uses:

```text
revenue
gross margin
delivery cost
payment behavior
returns/disputes
service cost
bad debt risk
discounts
support time
```

Selene says:

```text
Customer ABC buys often, but late payments, delivery disputes, and discounts reduce their true profitability.
```

### 12.4 Channel Profitability

Channels:

```text
POS
E-Commerce
B2B
marketplace
subscription
wholesale
agent/referral
```

Selene compares:

```text
gross sales
net revenue
fees
returns
delivery
commission
payment delay
margin
```

## 13. Strategic Low-Margin Products

Selene must understand product roles.

Not every low-margin product should be killed.

Some products are:

```text
traffic drivers
habit builders
basket builders
loss leaders
subscription anchors
B2B relationship products
strategic retention products
```

Selene evaluates:

```text
repeat purchase behavior
basket attachment
customer retention
cross-sell rate
lifetime value
B2B reorder dependency
brand trust
marketing pull
```

Selene says:

```text
This product has low margin, but customers who buy it usually purchase higher-margin items in the same basket. I recommend keeping it as a traffic driver but controlling stock tightly.
```

This prevents Selene from becoming a narrow margin-only system.

## 14. Forecast Vs Budget

Selene compares:

```text
budget
actuals
latest forecast
prior forecast
scenario forecast
```

Forecast updates may come from:

```text
sales trend
cashflow
customer orders
inventory demand
supplier cost changes
payroll changes
market campaigns
seasonality
new contracts
lost contracts
```

Selene says:

```text
Forecast spend is now expected to exceed budget by 9% even though actual spend is currently within budget. The risk is future supplier commitments.
```

That is early warning before actuals become the problem.

## 15. Autonomous Budget Actions

Selene can automatically:

```text
create draft budgets from actuals
suggest budget lines
detect missing budget owner
calculate budget vs actual
calculate committed spend
detect over-budget risk
explain variance
recommend budget transfer
recommend spend delay
recommend spend freeze
recommend campaign reduction
recommend inventory reorder split
recommend supplier renegotiation
update forecast
prepare management report
prepare board budget pack
alert budget owner
```

Selene can act under policy:

```text
auto-approve routine in-budget spend
auto-warn budget owner
auto-create variance explanation
auto-schedule review
auto-block clearly prohibited spend
```

Selene requires authority for:

```text
budget approval
budget lock
budget transfer above threshold
over-budget spend
budget override
board-level budget change
strategic fund release
major capex
department freeze
```

## 16. PH1.D / GPT-5.5 Role

GPT-5.5 should be used heavily for budget explanation and planning support.

### GPT-5.5 May Help

```text
summarize budget variance
draft budget commentary
explain profitability changes
draft manager-friendly budget reports
translate budget terms into plain language
draft board budget pack narrative
generate budget assumption wording
summarize spend exception reasons
suggest questions for budget owner
```

Selene can say:

```text
Operations is over budget because supplier transport charges increased and two urgent repairs were pulled forward from next quarter.
```

### GPT-5.5 Must Not

```text
approve budgets
change budget limits
override spend rules
invent actuals
invent forecasts
post ledger
grant authority
hide variance
decide official profitability truth
```

GPT-5.5 explains.

Selene deterministic engines calculate.

Access approves exceptions.

Accounting records actuals.

## 17. Human-Like Selene Interaction

### Budget Setup

```text
I built a draft budget using last year’s actuals and current supplier contracts. I only need you to confirm payroll growth, marketing plans, and major asset purchases.
```

### In-Budget Purchase

```text
This is within the approved warehouse budget and below the routine purchase limit, so I’ve approved it under policy.
```

### Over-Budget Warning

```text
This would push Marketing 12% over its quarterly budget. I recommend delaying it or requesting a budget transfer.
```

### Variance Explanation

```text
The branch is over budget mainly because sales volume increased and packaging costs rose with it. This is not necessarily bad, but margin needs review.
```

### Profitability

```text
Customer ABC has high revenue but low profitability due to late payments, frequent discounts, and expensive deliveries.
```

Human-like, but financially literate.

## 18. State Machines

### Budget State

```text
Draft
Preparing
PendingReview
PendingApproval
Approved
Active
Locked
Frozen
RevisionRequested
Revised
Superseded
Closed
Archived
```

### Budget Line State

```text
Planned
Active
NearLimit
OverBudget
Frozen
Revised
Closed
```

### Spend Check State

```text
Requested
BudgetChecking
WithinBudget
NearLimit
OverBudget
Blocked
Escalated
ApprovedUnderPolicy
ApprovedByAuthority
Rejected
Committed
```

### Variance State

```text
NoVariance
MinorVariance
MaterialVariance
Explained
Unexplained
ActionRequired
Resolved
```

### Profitability State

```text
Unknown
Positive
LowMargin
StrategicLowMargin
LossMaking
UnderReview
ImprovementPlan
DiscontinueCandidate
```

## 19. Reason Codes

```text
BUDGET_DRAFT_CREATED
BUDGET_OWNER_MISSING
BUDGET_PENDING_APPROVAL
BUDGET_APPROVED
BUDGET_LOCKED
BUDGET_REVISION_REQUESTED
SPEND_WITHIN_BUDGET
SPEND_NEAR_LIMIT
SPEND_OVER_BUDGET
SPEND_BLOCKED_BY_BUDGET
SPEND_AUTO_APPROVED_UNDER_POLICY
BUDGET_TRANSFER_RECOMMENDED
COMMITTED_SPEND_CREATED
COMMITTED_SPEND_RELEASED
VARIANCE_DETECTED
VARIANCE_EXPLAINED
FORECAST_EXCEEDS_BUDGET
PROFITABILITY_LOW_MARGIN
PROFITABILITY_STRATEGIC_LOW_MARGIN
PROFITABILITY_LOSS_MAKING
BOARD_BUDGET_APPROVAL_REQUIRED
```

## 20. Required Simulations

```text
create draft budget from prior actuals
approve annual budget
lock budget version
routine spend inside budget
purchase order creates commitment
supplier invoice converts commitment to actual
spend exceeds department budget
budget transfer requested
cashflow allows but budget blocks spend
budget allows but cashflow blocks payment
forecast exceeds budget before actuals do
variance explanation generated
product profitability analysis
customer profitability analysis
project profitability analysis
channel profitability analysis
strategic low-margin product retained
loss-making product flagged
board budget approval required
budget revision mid-year
```

## 21. Integration Map

```text
PH1.BUDGET / PH1.SPEND_GOV / PH1.PROFITABILITY
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.BANKREC / TREASURY
↔ PH1.PROCUREMENT
↔ PH1.CREDITORS / AP
↔ PH1.AR / DEBTORS
↔ PH1.PAYROLL
↔ PH1.TAX
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.CUSTOMER
↔ PH1.SUPPLIER
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.MARKETING
↔ PH1.ASSET
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.BOARD
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 22. Required Logical Packets

```text
BudgetPacket
BudgetVersionPacket
BudgetLinePacket
CostCenterPacket
ProfitCenterPacket
BudgetCommitmentPacket
SpendCheckPacket
BudgetVariancePacket
BudgetTransferPacket
BudgetForecastPacket
ProfitabilityPacket
CustomerProfitabilityPacket
ProductProfitabilityPacket
ProjectProfitabilityPacket
ChannelProfitabilityPacket
BudgetApprovalPacket
BudgetBoardPackPacket
AuditEvidencePacket
```

Logical only. Codex maps later. Do not create packet structs from this document alone.

## 23. What Codex Must Not Do

```text
Do not let Budget post accounting.
Do not let Budget move cash.
Do not let Budget validate invoices.
Do not let Budget execute payments.
Do not confuse cash available with budget available.
Do not confuse budget allowed with cashflow safe.
Do not let GPT-5.5 approve budgets.
Do not let routine in-budget spend require pointless approval.
Do not ignore committed spend.
Do not calculate profitability without source data.
Do not implement from this document alone.
```

## 24. Final Architecture Sentence

Selene Budget, Cost Center, Spend Governance + Profitability Engine is the planned-money and performance-control brain that creates and manages budgets, cost centers, commitments, budget approvals, budget locks, budget revisions, spend checks, budget vs actuals, budget vs forecast, variance explanations, and profitability across products, customers, projects, departments, branches, channels, and strategic low-margin roles, while letting Selene auto-handle routine in-budget spend under policy, escalate meaningful exceptions, use GPT-5.5 for human-like explanation, and keep Accounting, Cashflow, Procurement, AP, Access, and Audit as separate truth and execution owners.

Simple version:

```text
Selene knows the budget.
Selene knows what is already spent.
Selene knows what is committed.
Selene knows what will likely be spent.
Selene checks if new spend is allowed.
Selene approves routine safe spend under policy.
Selene escalates real exceptions.
Selene explains variances.
Selene measures profitability.
Selene protects the company from spending by vibes.
```

The company has cash awareness and spend discipline.
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_13_BUDGET_COST_CENTER_SPEND_GOVERNANCE_PROFITABILITY_MASTER_DESIGN.md -->

---

<!-- BEGIN_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_14_PERIOD_CLOSE_FINANCIAL_REPORTING_MASTER_DESIGN.md -->
# Finance / Accounting Document 14 — Selene Period Close + Financial Reporting Engine

```text
DOCUMENT TYPE:
FINANCE / ACCOUNTING MASTER DESIGN

DOCUMENT NUMBER:
14

ENGINE:
PH1.PERIOD_CLOSE / PH1.FIN_REPORTING

FULL NAME:
Selene Period Close, Financial Reporting, Management Accounts, Board Pack, Compliance Pack, and Continuous Close Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

## 0. Authority And Scope

AGENTS.md controls this document.

This is a docs-only architecture addition.

No runtime code is implemented by this document.

No schemas, migrations, APIs, packet structs, tests, or engine code are created by this document.

This document defines future canonical architecture for PH1.PERIOD_CLOSE / PH1.FIN_REPORTING. Repo-truth activation, simulation mapping, owner mapping, tests, and approved implementation slices must happen later before runtime behavior can be claimed.

## 1. Purpose

Selene Period Close + Financial Reporting Engine owns the process of turning all financial activity into clean, reviewable, auditable reports.

It answers:

```text
Is the accounting period ready to close?
Are bank reconciliations complete?
Are AP and AR reconciled?
Are payroll, tax, inventory, assets, depreciation, accruals, prepayments, and journals complete?
Are there unresolved exceptions?
Can we produce the P&L, balance sheet, cashflow report, budget variance report, and management accounts?
What still blocks close?
Who needs to fix it?
What can Selene close automatically?
What requires human review?
What reports should be sent to management, board, owners, investors, or regulators?
```

This is not a generate-report button.

This is the accounting control room.

A normal system says:

```text
Here is a P&L.
```

Selene says:

```text
The P&L is ready. Bank, AR, AP, payroll, tax, inventory, and accrual checks are complete. Two immaterial exceptions were auto-resolved under policy. One supplier credit note remains open but is disclosed in the close pack.
```

That is financial reporting with source readiness.

## 2. Core Selene Law

```text
Selene must run a continuous close, not a monthly panic ritual.

Routine checks and reconciliations should run automatically throughout the period.
Humans should only review exceptions, judgment areas, material adjustments, and final approval.
Reports must never pretend to be final if source engines are incomplete.
```

The goal:

```text
month-end should be confirmation, not archaeology.
```

Selene should already know what is ready, what is blocked, and who owns the blocker.

## 3. Engine Boundary

### 3.1 PH1.PERIOD_CLOSE / PH1.FIN_REPORTING Owns

```text
period close checklist
continuous close monitoring
close readiness scoring
financial reporting pack generation
management accounts
board reporting pack
owner/investor reporting pack
compliance reporting pack preparation
P&L reporting
balance sheet reporting
cashflow reporting
budget vs actual reporting
working capital reporting
aged debtors/creditors reporting
close exception worklist
close certification workflow
report version control
report distribution readiness
close audit evidence pack
```

### 3.2 PH1.PERIOD_CLOSE / PH1.FIN_REPORTING Does Not Own

```text
bank transaction proof
AP invoice validation
AR invoice creation
payroll calculation
inventory physical truth
asset lifecycle
depreciation policy ownership
tax law truth
payment execution
budget approval
board voting
ledger posting source truth
```

### 3.3 Correct Owner Split

```text
Accounting / GL = ledger, journals, accounting records.
BankRec = bank reconciliation proof.
AP = supplier invoice/payable truth.
AR = customer invoice/receipt truth.
Payroll = payroll calculation and payroll liabilities.
Tax = tax rules, tax returns, tax obligations.
Inventory = stock truth and inventory handoff.
Asset = asset lifecycle and depreciation evidence.
Budget = budget and variance.
Cashflow = liquidity forecasts.
Period Close = readiness, reporting, close pack, management accounts, board pack.
Access/Authority = report access and close approval.
Audit = proof trail.
PH1.WRITE/GPT-5.5 = human explanation and narrative drafting.
```

Period Close does not create truth.

It gathers truth, checks it, packages it, versions it, and tells humans whether it is safe to rely on.

## 4. Continuous Close Model

Selene should not wait for the end of the month.

She should perform continuous close checks daily or event-driven.

### 4.1 Continuous Checks

```text
bank feed synced
bank reconciliations current
supplier invoices processed
customer receipts matched
payroll runs posted/ready
tax obligations calculated or pending
inventory receipts/adjustments reviewed
asset additions/disposals reviewed
depreciation ready
accruals identified
prepayments identified
intercompany items identified if applicable
suspense accounts monitored
unmatched transactions reviewed
budget variances monitored
```

### 4.2 Close Readiness Score

Selene gives a readiness score:

```text
GREEN — ready
YELLOW — minor exceptions
ORANGE — material exceptions
RED — close blocked
BLACK — source data unreliable
```

Selene says:

```text
May close readiness is 87%. Bank and payroll are complete. AP has two supplier credit notes pending. Inventory has one stock variance above threshold.
```

## 5. Period Close Lifecycle

```text
PeriodOpen
ContinuousCloseMonitoring
PreCloseReview
CloseChecklistGenerated
SourceEngineValidation
ExceptionsDetected
AdjustmentsProposed
PendingReview
ReadyToClose
Closed
Locked
Reported
Reopened
Archived
```

### 5.1 PeriodOpen

Transactions flow normally.

### 5.2 ContinuousCloseMonitoring

Selene continuously checks readiness.

### 5.3 PreCloseReview

Selene warns before close date:

```text
The month closes in three days. I’m still waiting on two supplier statements and one bank feed sync.
```

### 5.4 CloseChecklistGenerated

Selene creates the close checklist by company type, size, industry, and active modules.

### 5.5 SourceEngineValidation

Selene asks each source engine for status.

### 5.6 ExceptionsDetected

Selene isolates material exceptions.

### 5.7 AdjustmentsProposed

Selene proposes accruals, prepaids, reclassifications, depreciation, FX, and routine close journals where source rules support it.

### 5.8 ReadyToClose

All blockers cleared or documented.

### 5.9 Closed

Accounting period is closed.

### 5.10 Locked

No changes except controlled reopening/adjustment.

## 6. Close Checklist

The close checklist depends on active modules.

Universal checklist:

```text
bank reconciliation complete
cash position verified
AP invoices processed
supplier credit notes reviewed
aged creditors reviewed
AR invoices processed
customer receipts matched
aged debtors reviewed
payroll processed
tax liabilities reviewed
suspense/unmatched items reviewed
accruals reviewed
prepayments reviewed
depreciation reviewed
management reports generated
audit pack generated
```

Conditional checklist:

```text
inventory valuation if stock exists
COGS review if products sold
WIP review if manufacturing active
recipe/food cost review if restaurant active
asset additions/disposals if assets exist
fleet costs if fleet active
insurance prepayments/claims if insurance active
intercompany if multi-entity active
foreign currency if multi-currency active
board pack if board reporting active
shareholder/investor pack if applicable
```

Selene should not ask small companies for irrelevant enterprise close tasks.

## 7. Source Engine Close Validation

Each source engine reports close status.

### 7.1 BankRec

```text
all bank accounts reconciled
unmatched transactions listed
cash position verified
payment provider settlements matched
```

### 7.2 AP / Creditors

```text
supplier invoices processed
credit notes applied/pending
supplier statements reconciled
aged creditors ready
AP holds listed
```

### 7.3 AR / Debtors

```text
invoices issued
receipts matched
credit notes/refunds processed
aged debtors ready
bad debt risk listed
```

### 7.4 Payroll

```text
payroll run completed
payroll liabilities recorded
employee payments confirmed
tax/super/pension obligations listed
```

### 7.5 Tax

```text
GST/VAT/sales tax draft position
payroll tax/withholding status
tax adjustments pending
filing due dates
```

### 7.6 Inventory

```text
stock movements processed
receipts accepted
stock variances reviewed
inventory valuation handoff ready
COGS handoff ready
damaged/expired/write-off items listed
```

### 7.7 Assets

```text
asset additions reviewed
disposals reviewed
depreciation ready
impairment flags listed
capital vs expense items reviewed
```

### 7.8 Budget

```text
budget vs actual ready
committed spend updated
material variances explained
```

## 8. Accruals And Prepayments

Selene must identify timing adjustments.

### 8.1 Accrual Candidates

```text
goods/services received but not invoiced
supplier work completed but invoice missing
utilities incurred but bill not received
payroll earned but unpaid
tax incurred
interest incurred
contract milestones completed
```

Selene says:

```text
The repair work was accepted before month-end but the supplier invoice has not arrived. I recommend an accrual.
```

### 8.2 Prepayment Candidates

```text
annual insurance paid upfront
software subscription paid yearly
rent paid in advance
maintenance contract prepaid
licences paid ahead
```

Selene says:

```text
This insurance payment covers 12 months. I recommend recognizing one month as expense and the rest as prepaid.
```

Accounting owns posting.

Selene proposes and routes.

## 9. Depreciation And Asset Close

Selene checks with Asset Engine and Accounting.

Close checks:

```text
new assets capitalized
asset receipts accepted
asset disposals recorded
depreciation run prepared
depreciation exceptions listed
impairment indicators flagged
asset under construction reviewed
vehicle/fleet assets linked
insurance/asset linkage reviewed
```

Selene says:

```text
Three assets were purchased this month. Two are ready for capitalization. One lacks receiving acceptance, so depreciation cannot start yet.
```

Depreciation should not start solely because an invoice exists.

## 10. Inventory And COGS Close

If inventory exists, close must include:

```text
stock movement completeness
receiving acceptance
inventory valuation
damaged stock
expired stock
write-offs
stock count variances
COGS calculation
WIP if manufacturing
raw materials if production/restaurant
```

Selene should flag:

```text
Inventory valuation is blocked because Warehouse B has an unresolved stock count variance.
```

Close does not pretend stock is correct while material stock variance remains unresolved.

## 11. Intercompany And Multi-Entity Close

If multi-entity exists, Selene supports:

```text
intercompany invoices
intercompany loans
intercompany expenses
management fees
transfer pricing entries
elimination candidates
multi-currency translations
entity-level close status
group close status
```

Selene says:

```text
Entity A is ready to close, but Entity B has unreconciled intercompany charges. Group reporting is not ready.
```

No group report should be finalized from half-closed entities.

## 12. Suspense And Unclassified Transactions

Selene must monitor suspense accounts and unclassified items.

Sources:

```text
unknown bank receipts
unknown payments
unclassified expenses
missing supplier/customer mapping
temporary clearing accounts
payment provider clearing
cash clearing
```

Selene action:

```text
auto-classify if high confidence and policy allows
propose classification
request evidence
escalate material unexplained items
block close if above threshold
```

Selene says:

```text
There are four suspense items totaling $180. Policy allows close with disclosure. One $8,000 unknown payment blocks close.
```

Not every small item deserves executive escalation. Material unexplained items do.

## 13. Financial Reports

Selene generates reports only when source status supports them.

Core reports:

```text
Profit and Loss
Balance Sheet
Cashflow Statement / Cash Movement Report
Trial Balance
General Ledger Summary
Budget vs Actual
Forecast vs Actual
Aged Debtors
Aged Creditors
Bank Reconciliation Summary
Inventory / COGS Summary
Payroll Summary
Tax Summary
```

Management reports:

```text
monthly management accounts
department performance
branch performance
product profitability
customer profitability
supplier cost trend
working capital report
cashflow forecast
variance explanations
KPI dashboard
```

Board reports:

```text
financial summary
cash position
budget variance
risk exceptions
major spend approvals
working capital
debt/loan status
capital expenditure
supplier/customer risk
forecast scenarios
management commentary
```

Selene must label reports:

```text
Draft
Preliminary
ClosePending
Final
BoardApproved
Filed
Superseded
```

Draft reports must be clearly labelled.

Final reports require source-engine readiness.

## 14. Report Version Control

Every report has:

```text
report_id
report_type
period
version
status
generated_at
source_data_cutoff
source_engine_status
adjustments_included
exceptions_disclosed
approved_by
distributed_to
audit_ref
```

If a report changes:

```text
new version
diff summary
reason
prior version retained
audit trail
```

Selene says:

```text
This is Version 3 of the May management accounts. It changed from Version 2 because the supplier credit note was received and AP balance reduced.
```

No mystery revisions.

## 15. Report Access And Distribution

Financial reports are sensitive.

Selene checks Access before showing or sending.

Access controls:

```text
who can view P&L
who can view balance sheet
who can view cash
who can view payroll summaries
who can view customer/supplier details
who can view board pack
who can export reports
who can distribute externally
```

Distribution channels:

```text
Selene portal
secure email
board portal
management dashboard
PDF export
Excel export
API/report pack
regulator filing pack where applicable
```

Selene should say:

```text
You do not have access to payroll detail. I can show the department-level payroll summary.
```

## 16. Management Commentary

Selene should generate human-readable commentary.

Inputs:

```text
P&L movement
budget variance
cashflow changes
working capital
supplier/customer issues
inventory changes
payroll changes
tax obligations
one-off events
```

GPT-5.5 may draft commentary, but source facts must be deterministic.

Example commentary:

```text
Revenue increased by 14% compared with last month, mainly due to B2B orders. Gross margin fell because supplier costs increased and discounts were used to clear slow-moving stock. Cash remains stable due to improved receivables collection.
```

That is board-readable.

## 17. Close Exception Worklist

Selene creates a worklist only for unresolved close blockers.

Exception fields:

```text
exception_id
source_engine
period
materiality
description
reason_code
owner
deadline
suggested action
status
audit_ref
```

Exception statuses:

```text
Open
Assigned
InProgress
Resolved
WaivedUnderPolicy
Escalated
Closed
```

Selene should not ask humans to inspect every clean section.

She should say:

```text
There are three close exceptions. Everything else is complete.
```

## 18. Close Approval

Close approval may require:

```text
accountant
finance manager
CFO
CEO
board
external accountant/adviser
```

Close approval states:

```text
NotReady
ReadyForPreparerReview
ReadyForFinanceReview
ReadyForCFOApproval
Approved
Locked
Reopened
```

Selene should support policy:

```text
small company → owner/accountant approval
mid-size → finance manager/CFO approval
enterprise → controller/CFO/board pack workflow
```

Routine close can be mostly automatic.

Final close approval remains authority-controlled.

## 19. Reopening A Closed Period

Closed periods should not be changed casually.

Reopen requires:

```text
reason
authority
affected reports
adjustment type
audit trail
version impact
distribution impact
tax/reporting impact
```

Selene says:

```text
Reopening May will supersede the management accounts already distributed. I need approval and a reason before proceeding.
```

No casual time travel in accounting.

## 20. Audit And Evidence Pack

Close pack includes:

```text
period status
bank reconciliation proof
AP reconciliation proof
AR reconciliation proof
payroll proof
tax proof
inventory valuation proof
asset/depreciation proof
journal summary
adjustments
exceptions
approvals
report versions
distribution record
audit hash/reference
```

Selene should be able to answer:

```text
Show me why May was closed.
```

And produce the evidence.

## 21. Autonomous Close Actions

Selene can automatically:

```text
monitor close readiness
create close checklist
request missing source statuses
auto-match routine close items
prepare accrual suggestions
prepare prepayment schedules
prepare depreciation run review
prepare variance explanations
generate draft reports
generate management commentary
prepare board pack
create exception worklist
send reminders to owners
close immaterial exceptions under policy
prepare final close approval request
archive close evidence
```

Selene requires authority for:

```text
period close approval
period reopen
material journal approval
material accrual/prepayment judgment
write-off
impairment
tax filing approval
board report release
external distribution
```

## 22. PH1.D / GPT-5.5 Role

GPT-5.5 should be heavily used for explanation, commentary, and report narration.

### GPT-5.5 May Help

```text
draft management commentary
summarize period results
explain variances
draft board pack narrative
summarize close exceptions
translate accounting language into plain English
draft report cover notes
prepare CFO briefing
```

### GPT-5.5 Must Not

```text
close period
approve reports
post journals
invent financial numbers
hide exceptions
override source engine status
approve tax filing
release board pack
change report version
```

GPT-5.5 writes like a human.

Selene deterministic engines decide if numbers are ready.

## 23. Human-Like Selene Interaction

### Pre-Close

```text
Month-end is in three days. Bank and payroll are ready. AP still has two supplier credit notes pending, and Inventory has one material stock variance.
```

### Close Readiness

```text
May is ready to close. All source engines are reconciled, and the remaining exceptions are immaterial and disclosed.
```

### Report Generation

```text
The management accounts are ready. Revenue increased, margin fell slightly, and cash improved due to faster collections.
```

### Close Blocked

```text
I cannot close the period yet. The operating bank account has an unmatched $8,000 outflow and AP has one unresolved supplier dispute.
```

### Reopen Warning

```text
Reopening April will supersede reports already sent to the board. Please provide a reason and approval.
```

Human-like. Calm. Direct.

## 24. State Machines

### Period Close State

```text
PeriodOpen
ContinuousCloseMonitoring
PreCloseReview
CloseChecklistGenerated
SourceEngineValidation
ExceptionsDetected
AdjustmentsProposed
PendingReview
ReadyToClose
Closed
Locked
Reported
Reopened
Archived
```

### Report State

```text
Draft
Preliminary
PendingSourceValidation
ReadyForReview
Final
Distributed
BoardApproved
Filed
Superseded
Archived
```

### Close Exception State

```text
Open
Assigned
InProgress
Resolved
WaivedUnderPolicy
Escalated
Closed
```

### Close Approval State

```text
NotReady
ReadyForPreparerReview
ReadyForFinanceReview
ReadyForCFOApproval
Approved
Rejected
Locked
Reopened
```

## 25. Reason Codes

```text
PERIOD_CLOSE_STARTED
CONTINUOUS_CLOSE_CHECK_COMPLETE
BANK_RECON_INCOMPLETE
AP_RECON_INCOMPLETE
AR_RECON_INCOMPLETE
PAYROLL_CLOSE_INCOMPLETE
TAX_CLOSE_INCOMPLETE
INVENTORY_VALUATION_INCOMPLETE
ASSET_DEPRECIATION_PENDING
SUSPENSE_ITEMS_OPEN
MATERIAL_EXCEPTION_OPEN
IMMATERIAL_EXCEPTION_WAIVED
ACCRUAL_RECOMMENDED
PREPAYMENT_RECOMMENDED
DEPRECIATION_READY
REPORT_DRAFT_CREATED
REPORT_PENDING_VALIDATION
REPORT_FINALIZED
PERIOD_READY_TO_CLOSE
PERIOD_CLOSED
PERIOD_LOCKED
PERIOD_REOPEN_REQUESTED
BOARD_PACK_READY
MANAGEMENT_ACCOUNTS_READY
```

## 26. Required Simulations

```text
continuous close readiness check
pre-close reminder
bank rec blocks close
AP credit note blocks close
AR receipt mismatch blocks close
payroll close complete
tax close pending
inventory variance blocks close
asset depreciation pending
accrual proposed
prepayment proposed
suspense item reviewed
management accounts generated
board pack generated
draft report labelled preliminary
final report generated
period close approved
period locked
period reopened with reason
report version superseded
audit close pack generated
```

## 27. Integration Map

```text
PH1.PERIOD_CLOSE / PH1.FIN_REPORTING
↔ PH1.ACCOUNTING / GL
↔ PH1.BANKREC / TREASURY
↔ PH1.CREDITORS / AP
↔ PH1.CREDITORS.RECON
↔ PH1.AR / DEBTORS
↔ PH1.AR.COLLECT
↔ PH1.PAYROLL
↔ PH1.TAX
↔ PH1.INVENTORY
↔ PH1.ASSET
↔ PH1.CASHFLOW
↔ PH1.BUDGET / PROFITABILITY
↔ PH1.BOARD
↔ PH1.SHAREHOLDER / INVESTOR ACCESS
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

## 28. Required Logical Packets

```text
PeriodPacket
PeriodCloseChecklistPacket
SourceEngineCloseStatusPacket
CloseReadinessPacket
CloseExceptionPacket
AccrualProposalPacket
PrepaymentProposalPacket
DepreciationClosePacket
InventoryValuationClosePacket
FinancialReportPacket
ManagementAccountsPacket
BoardPackPacket
ReportVersionPacket
ReportDistributionPacket
CloseApprovalPacket
PeriodLockPacket
PeriodReopenPacket
CloseAuditPackPacket
```

Logical only. Codex maps later. Do not create packet structs from this document alone.

## 29. What Codex Must Not Do

```text
Do not let Period Close create source truth.
Do not let Period Close post journals directly.
Do not let GPT-5.5 invent financial commentary unsupported by source data.
Do not generate final reports if source engines are incomplete.
Do not hide unresolved exceptions.
Do not distribute sensitive reports without Access.
Do not reopen closed periods without authority.
Do not treat preliminary reports as final.
Do not implement from this document alone.
```

## 30. Final Architecture Sentence

Selene Period Close + Financial Reporting Engine is the continuous accounting close and reporting brain that monitors source-engine readiness throughout the period, validates bank, AP, AR, payroll, tax, inventory, assets, depreciation, accruals, prepayments, suspense items, budgets, and cashflow, creates close checklists and exception worklists, generates management accounts, financial statements, board packs, report versions, distribution controls, and audit evidence, while using GPT-5.5 for human-readable commentary and keeping source truth, ledger posting, authority, tax, and final approval under their proper deterministic engines.

Simple version:

```text
Selene watches the close all month.
Selene checks every source engine.
Selene finds blockers early.
Selene proposes accruals and prepayments.
Selene prepares reports.
Selene explains results like a human.
Selene shows what is final and what is draft.
Selene closes only when truth is ready.
Humans review exceptions and approve final close.
Everything is audited.
```

Month-end becomes a continuous, evidence-backed process where humans review judgment items instead of hunting missing invoices.
<!-- END_SOURCE_FILE: docs/SELENE_FINANCE_ACCOUNTING_DOCUMENT_14_PERIOD_CLOSE_FINANCIAL_REPORTING_MASTER_DESIGN.md -->
