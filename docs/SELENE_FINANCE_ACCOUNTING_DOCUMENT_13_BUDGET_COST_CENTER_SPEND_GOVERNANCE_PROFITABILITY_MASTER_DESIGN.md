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
