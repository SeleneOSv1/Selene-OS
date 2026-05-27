# Selene Budgeting + Spend Control + Board Approval Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / BUDGETING + SPEND CONTROL + BOARD APPROVAL + FINANCIAL GOVERNANCE

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's budgeting and spend governance system: annual budgets, department budgets, cost-center budgets, project budgets, capex budgets, payroll budgets, credit-card budgets, minimum cash reserve policy, net profit floor governance, spend limits, budget variance, budget increase approvals, board/CEO/CFO/manager approvals, multi-signature authority, and autonomous budget monitoring.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 4. It defines future Budgeting, Spend Control, Board Approval, budget checks, cash reserve governance, net profit floor governance, budget-aware procurement/AP/cards/payroll/hiring, and board-ready budget evidence. It does not implement Budgeting, Cashflow, Accounting, AP, AR, Payroll, Banking, Credit Cards, Assets, Procurement, Board, Shareholder, Desktop, iPhone, Adapter, packets, migrations, tests, or runtime state.

Current repo truth does not prove complete runtime Budgeting, Cashflow, Board Approval, Payment Priority, or autonomous cash recovery ownership. This document is future architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, Access/Governance proof, PH1.WRITE proof, PH1.D provider-off/fake-provider proof where relevant, audit proof, simulation proof, and JD live acceptance where visible.

## 1. Executive Target

Selene must not only record money after it happens.

Selene must help the company plan, control, protect, and govern money before it moves.

Old budget process:

```text
people export last year's numbers
someone builds spreadsheet
managers guess
board approves a PDF
nobody watches it properly
spend exceeds budget
everyone acts surprised
```

Selene budget process:

```text
Selene reads historical actuals
Selene forecasts income and costs
Selene prepares annual and department budgets
Selene checks every spend request against budget
Selene watches cash reserve and profit targets
Selene escalates over-budget spend before money moves
Selene prepares board-ready explanations
Selene monitors budget vs actual continuously
Selene recommends corrective action early
```

Target:

```text
real-time budget governance
annual budget preparation
board-approved financial plans
department and cost-center spend control
project and capex budgets
cash reserve protection
net profit floor monitoring
automatic spend checks
budget increase requests
multi-authority approvals
clear board summaries
audit-backed financial decisions
```

Tiny miracle: no one gets to say, "I didn't know we were over budget," after the company has already committed to unnecessary spend.

## 2. Master Law

```text
Every controlled spend must check budget before payment or commitment.

Every approved budget must have owner, period, scope, amount, currency, approval, and audit.

Every over-budget action must be blocked, warned, or escalated according to policy.

Every budget increase must have reason, impact, authority, and approval.

Every minimum cash reserve rule must be respected unless emergency override passes.

Every net profit floor must be monitored against actual and forecast performance.

Board-approved budgets must become active governance rules inside Selene.

PH1.D/GPT-5.5 may help explain, forecast narratives, and draft board summaries.

PH1.D/GPT-5.5 must not approve budgets, override spend controls, approve reserve breaches, or invent financial truth.
```

## 3. Owner Split

### Finance / Budget Owns

```text
annual budget
department budget
cost-center budget
project budget
capex budget
payroll budget
inventory budget
marketing budget
credit-card budget
spend limits
budget variance
budget increase request
cash reserve policy
net profit floor policy
financial governance policy
board approval workflow
budget audit evidence
```

### Accounting Owns

```text
actual ledger values
posted expenses
posted revenue
journal evidence
trial balance
financial statement base
budget-vs-actual source values
```

Finance/Budget uses Accounting actuals. It does not invent them.

### AP Owns

```text
supplier bill amount
contractor invoice amount
installment payment
lease payment
scheduled payment request
AP approval readiness
```

AP requests budget checks. Budget does not own supplier bills.

### AR Owns

```text
customer receivables
expected receipts
debtor aging
collection forecast evidence
```

AR feeds income/cash assumptions.

### Banking / Payment Owns

```text
bank balances
payment confirmations
cash movement proof
bank feeds
payment rail proof
```

Banking proves money moved. Budget decides whether spending is allowed.

### Payroll Owns

```text
payroll forecast
payroll actuals
wage cost
salary changes
overtime impact
bonus/commission cost
employment-cost evidence
```

### Credit Cards / Employee Spend Owns

```text
card transaction evidence
employee card budgets
executive card spend
spend exceptions
personal charge recovery handoff
```

### Assets / Procurement Owns

```text
purchase request
purchase order
capex request
asset acquisition proposal
goods receipt / acceptance evidence
```

### Cashflow Owns

```text
cash position
cash forecast
payment priority
reserve breach warning
shortfall response
```

Document 8 expands this.

### Access / Governance Owns

```text
who can create budget
who can approve budget
who can change budget
who can approve over-budget spend
who can approve reserve breach
who can approve net profit target changes
who can approve board packs
who can export budget reports
```

### PH1.WRITE Owns

```text
budget explanations
approval summaries
board-ready wording
over-budget warnings
cash reserve warnings
profit floor warnings
budget increase request wording
```

### PH1.D / GPT-5.5 May Assist

```text
summarize budget variance
draft board explanations
explain budget impact
suggest budget categories
draft spend approval summaries
prepare management-friendly narratives
help compare options
```

But must not:

```text
approve budget
change budget
override budget
approve spend
approve cash reserve breach
approve net profit floor breach
invent actuals
invent forecast truth
```

## 4. Budget Scope

Selene must support many budget types.

```text
annual company budget
monthly budget
quarterly budget
department budget
cost-center budget
project budget
customer budget
product budget
location budget
country/legal entity budget
capex budget
opex budget
payroll budget
contractor budget
marketing budget
inventory purchase budget
fleet budget
insurance budget
IT/software budget
travel budget
credit-card budget
tax budget
loan/lease payment budget
cash reserve policy
profit target policy
```

Budget must support:

```text
legal_entity_id
company_id
country
region
currency
fiscal_year
period
department
cost_center
project
product
customer
supplier
employee
asset_class
spend_category
approval_owner
```

## 5. Budget Data Model

### Budget Header Fields

```text
budget_id
tenant_id
company_id
legal_entity_id
budget_name
budget_type
country
currency
fiscal_year
period_start
period_end
status
created_by
approved_by
board_approval_ref
effective_from
effective_to
audit_ref
```

### Budget Line Fields

```text
budget_line_id
budget_id
account_code_ref
category
department_id
cost_center_id
project_id
location_id
asset_class_ref
employee_group_ref
supplier_group_ref
amount_budgeted
amount_committed
amount_actual
amount_remaining
variance_amount
variance_percent
warning_threshold_percent
hard_limit_percent
approval_required_above_threshold
audit_ref
```

### Budget Statuses

```text
Draft
UnderReview
PendingManagementApproval
PendingBoardApproval
Approved
Active
RevisionRequested
Revised
Suspended
Closed
Archived
```

## 6. Annual Budget Creation

Selene should help prepare annual budgets.

Inputs:

```text
prior year actuals
current year actuals
sales forecast
AR collection forecast
AP schedule
payroll forecast
overtime forecast
contractor forecast
inventory forecast
asset purchase plan
loan/lease schedules
tax schedules
insurance renewals
marketing plan
project plans
department requests
cash reserve policy
net profit target
growth assumptions
country/entity rules
```

Flow:

```text
Selene gathers historical actuals
-> Selene requests department inputs
-> Selene generates baseline budget
-> Selene applies growth and inflation assumptions
-> Selene applies payroll/roster/contractor forecasts
-> Selene applies known scheduled payments
-> Selene checks cash reserve and profit floor
-> Selene prepares draft budget
-> management reviews
-> board reviews
-> approved budget becomes active governance rule
```

Selene says:

```text
I've prepared the draft annual budget using last year's actuals, current payroll plans, scheduled lease payments, supplier forecasts, and the 8% minimum profit target. Three areas need review: overtime, vehicle maintenance, and marketing spend.
```

## 7. Department And Cost-Center Budgets

Each department may have its own budget.

Examples:

```text
Warehouse
Sales
Marketing
Finance
HR
IT
Fleet
Operations
Customer Support
Research and Development
```

Budget controls:

```text
monthly limit
quarterly limit
annual limit
category limit
manager approval threshold
finance approval threshold
board threshold
warning threshold
hard stop threshold
```

Example:

```text
Warehouse maintenance budget: AUD 50,000 per quarter.
Current actual + committed: AUD 47,500.
New supplier bill: AUD 8,200.
Result: budget would be exceeded.
Selene routes approval or recommends budget increase.
```

Selene says:

```text
This repair would take Warehouse Maintenance AUD 5,700 over the quarterly budget. I can request Finance approval or prepare a budget increase summary.
```

## 8. Project Budgets

Projects need budget control.

Project budget fields:

```text
project_id
project_manager_id
approved_budget
committed_spend
actual_spend
forecast_spend
remaining_budget
customer_revenue_ref
profit_margin_target
budget_status
audit_ref
```

Selene checks:

```text
purchase order
contractor invoice
employee time cost
material cost
travel cost
customer billing
margin impact
cashflow impact
```

If project goes over budget:

```text
Selene prepares:
- cause
- current variance
- forecast final cost
- margin impact
- recommended actions
- approval route
```

## 9. Capex Budget

Capex budgets cover major asset purchases.

Examples:

```text
vehicles
buildings
machinery
computers
warehouse equipment
software implementation
fit-outs
plant
land
```

Capex request flow:

```text
purchase request
-> budget check
-> asset classification candidate
-> cashflow check
-> depreciation impact
-> tax/claimability check
-> approval threshold
-> board approval if required
-> PO / AP / asset workflow
```

Selene says:

```text
The delivery van fits the approved vehicle capex budget, but the payment would reduce cash close to the reserve threshold. I recommend CFO review before the PO is finalized.
```

## 10. Payroll And Labour Budget

Payroll is often the largest cost.

Selene must budget:

```text
base wages
salaries
overtime
commissions
bonuses
contractors
employer contributions
super/pension/CPF
payroll tax
benefits
leave payout
final pay risk
new hires
probation completions
seasonal staffing
```

Connections:

```text
Payroll -> actual labour cost
Roster/Attendance -> worked hours/overtime forecast
Position -> planned hiring cost
HR -> employment changes
HWM/Workload -> staffing pressure
Finance/Budget -> budget control
```

Example:

```text
Warehouse overtime is forecast to exceed budget by AUD 22,000 this month.
Selene compares:
- overtime cost
- hiring casual staff
- using contractor
- shifting roster
```

Selene says:

```text
Warehouse overtime is forecast to exceed budget by AUD 22,000. Hiring one casual worker for the next four weeks may be cheaper than continuing overtime at the current rate.
```

## 11. Credit Card And Employee Spend Budgets

Document 6 owns card spend workflow.

Document 7 owns budget policy.

Budget may be set by:

```text
employee
position
department
card type
merchant category
project
month
trip
event
```

Examples:

```text
Sales Manager monthly travel budget: AUD 5,000
Fleet fuel card monthly limit: AUD 2,000
Executive entertainment limit: AUD 10,000
Project card limit: AUD 25,000 for project duration
```

If exceeded:

```text
Selene blocks, warns, or routes approval depending policy.
```

## 12. Minimum Cash Reserve Policy

Authorized executives or board may set a minimum cash reserve.

Example:

```text
Selene, our reserve cash should now be AUD 12 million. Do not allow normal payments below that.
```

Selene must:

```text
check authority
check whether approval is required
ask effective date / period
define legal entity/company scope
store reserve policy
apply to payment scheduling
notify Finance/Cashflow
audit
```

Fields:

```text
reserve_policy_id
company_id
legal_entity_id
amount
currency
effective_from
effective_to
minimum_cash_type
approved_by
override_policy_ref
audit_ref
```

Reserve breach:

```text
normal payment blocked
emergency override available only through required authority
reason required
approval required
audit required
```

Selene says:

```text
This payment would take cash below the AUD 12 million reserve. I can request emergency approval, delay the payment if terms allow, or propose a partial payment.
```

Document 8 expands cashflow execution.

## 13. Net Profit Floor / Profitability Governance

Authorized executives or board may set a minimum net profit target.

Example:

```text
Selene, we need to maintain at least 8% net profit.
```

Selene must:

```text
check authority
define period
define entity/company scope
define target percentage
store profit floor
monitor actual and forecast profit
warn when forecast drops below target
suggest corrective actions
escalate major spend if target threatened
audit
```

Fields:

```text
profit_floor_policy_id
company_id
legal_entity_id
target_percent_min
target_percent_max_optional
period
effective_from
effective_to
approved_by
audit_ref
```

Selene monitors:

```text
revenue
COGS
gross margin
payroll
contractor spend
overheads
discounting
freight
inventory waste
credit-card spend
supplier price increases
tax impact
forecast month-end profit
forecast year-end profit
```

Selene says:

```text
Current forecast net profit is 7.4%, below the 8% floor. The main pressure is overtime, freight, and discounting. I recommend freezing non-essential spend and reviewing warehouse overtime.
```

Profit floor does not mean Selene hides costs or manipulates accounting.

It means Selene alerts, recommends, and governs future spend.

## 14. Spend Request Budget Check

Every controlled spend request must check budget.

Sources:

```text
purchase order
supplier bill
contractor invoice
card transaction
employee reimbursement
asset purchase
marketing campaign
software subscription
travel request
project expense
payroll/overtime decision
fleet repair
insurance renewal
```

Budget check packet:

```text
BudgetCheckPacket:
  budget_check_id
  source_owner
  source_document_ref
  company_id
  legal_entity_id
  amount
  currency
  budget_line_ref
  cost_center_ref
  department_ref
  project_ref
  available_budget
  committed_budget
  actual_spend
  forecast_spend
  variance_after_request
  warning_threshold_breached
  hard_limit_breached
  recommendation
  audit_ref
```

Recommendations:

```text
allow
warn
allow_with_manager_approval
requires_finance_approval
requires_budget_increase
requires_board_approval
block
defer
split_payment
seek alternative
```

## 15. Budget Commitment Vs Actual

Selene must distinguish committed spend from actual spend.

```text
Committed spend = approved PO, signed contract, scheduled payment, approved purchase, approved payroll increase.
Actual spend = posted accounting actuals / paid transaction / recognized expense.
```

Example:

```text
Budget: AUD 100,000
Actual spent: AUD 60,000
Committed POs: AUD 30,000
Remaining true capacity: AUD 10,000
```

Selene must not say "we have 40,000 left" if 30,000 is already committed.

## 16. Budget Variance

Variance means actual or forecast differs from budget.

Variance fields:

```text
budget_line_id
period
budget_amount
actual_amount
committed_amount
forecast_amount
variance_amount
variance_percent
variance_direction
reason_candidate
owner_id
action_required
audit_ref
```

Variance categories:

```text
favorable
unfavorable
timing_difference
volume_difference
price_difference
efficiency_difference
one-off
recurring
policy_change
error_candidate
```

Selene may use PH1.D to draft explanation, but Finance/Accounting evidence decides truth.

Selene says:

```text
Marketing is AUD 18,000 over budget this month. Most of the variance comes from the trade-show campaign approved last week. This is a timing issue, not recurring overspend.
```

## 17. Budget Increase Request

If spending needs to exceed budget, Selene prepares a budget increase request.

Required fields:

```text
budget_increase_request_id
budget_id
budget_line_id
requested_amount
currency
reason
urgency
impact_if_denied
cashflow_impact_ref
profit_floor_impact_ref
funding_source_candidate
requested_by
required_approvers
approval_status
audit_ref
```

Flow:

```text
budget breach detected
-> Selene explains variance
-> Selene proposes options
-> requester asks for increase
-> Finance reviews
-> approvers/board decide
-> budget updated if approved
-> audit
```

Selene says:

```text
This request exceeds the approved fleet maintenance budget by AUD 9,400. I can prepare a budget increase request showing why it happened, cashflow impact, and options for approval.
```

## 18. Board Approval

Some budgets and financial decisions require board approval.

Board approval may apply to:

```text
annual budget
major capex
large budget increase
cash reserve policy
net profit floor
dividend policy
large loans/debt
property purchase
large supplier contract
major headcount plan
over-budget spend above threshold
related-party transaction
```

Board approval packet:

```text
BoardApprovalRequestPacket:
  request_id
  request_type
  company_id
  legal_entity_id
  amount
  currency
  summary_ref
  supporting_reports_refs
  risk_summary_ref
  options
  recommendation
  required_board_members
  approval_rule
  status
  audit_ref
```

Approval rules may include:

```text
chair approval
CEO approval
CFO approval
simple majority
unanimous
2_of_3
board quorum
chairman casting vote
class shareholder approval where relevant
```

This connects to future Document 14 - Equity + Dividends, and Document 17 - Reporting + Board Packs.

## 19. Multi-Authority Approval

Budget/spend approval may require multiple people.

Examples:

```text
Manager + Finance
CFO + CEO
2 of 3 Directors
Board quorum
Chairman + CFO
Department Head + Project Sponsor
```

Approval requirements:

```text
approval_request_id
action_type
amount
currency
required_roles
required_number_of_approvers
specific_required_users
step_up_required
expires_at
decision_status
audit_ref
```

Step-up may include:

```text
Face ID
fingerprint
secure passcode
approved device confirmation
```

Selene says:

```text
This budget increase needs approval from both the CFO and CEO. I've prepared the summary and will route it to both for review.
```

## 20. Budget Controls: Hard Stop Vs Soft Warning

Budget rules may be hard or soft.

```text
soft_warning
manager_approval
finance_approval
executive_approval
board_approval
hard_stop
```

### Soft Warning

```text
Warn but allow if within policy.
```

### Hard Stop

```text
Block unless authorized override passes.
```

Example:

```text
Budget exceeded by 3% -> manager approval.
Budget exceeded by 10% -> Finance approval.
Budget exceeded by 25% -> board approval.
Budget exceeded by 50% -> hard stop unless emergency.
```

## 21. Emergency Spend Override

Sometimes a payment must proceed even if over budget or below reserve.

Examples:

```text
critical supplier to keep production running
tax payment due
loan payment due
insurance policy renewal
urgent safety repair
legal obligation
```

Emergency override requires:

```text
reason
evidence
risk if unpaid
cashflow impact
budget impact
required approvers
step-up
audit
post-event review
```

Selene says:

```text
This emergency repair exceeds budget, but delaying it may stop production. I can request emergency approval from Finance and Operations.
```

## 22. Spend Freeze / Cost Control Mode

If profit floor, cash reserve, or budget risk is serious, Selene may recommend spend freeze.

Spend freeze levels:

```text
advisory_watch
discretionary_spend_pause
non-essential_purchase_hold
capex_freeze
hiring_freeze_candidate
travel_freeze_candidate
executive_approval_only
board_control_mode
```

Spend freeze requires authority.

Selene may recommend:

```text
pause non-essential purchases
require Finance approval for all new POs
hold discretionary card categories
delay low-priority supplier payments if terms allow
freeze new subscriptions
review overtime
review contractor spend
```

Selene must not freeze payroll/tax/legal obligations casually.

## 23. Spend Forecasting

Selene should forecast budget impact before actual spend occurs.

Inputs:

```text
open POs
scheduled AP payments
payroll forecast
contractor forecast
card spending trends
subscriptions
loan/lease payments
tax schedules
planned campaigns
inventory reorder cycles
asset replacement plans
```

Forecast outputs:

```text
forecast_spend_by_period
forecast_budget_remaining
forecast_overrun_date
risk_level
recommended_action
```

Example:

```text
At current spend rate, Sales travel will exceed monthly budget on the 22nd.
```

## 24. Budget-Aware Procurement

Procurement must check budget before PO approval.

Flow:

```text
purchase request
-> budget check
-> cashflow check if material
-> approval route
-> PO creation
-> budget commitment recorded
-> goods receipt
-> AP bill
-> actual spend
```

Budget should reserve committed amount when PO approved.

If PO cancelled, commitment releases.

## 25. Budget-Aware AP

AP must check budget before payment or bill approval where policy requires.

Flow:

```text
supplier bill received
-> AP validation
-> budget check
-> if within budget, continue approval
-> if over budget, route budget exception
-> if approved, payment may proceed
```

Bill can be valid but budget-blocked.

Selene says:

```text
The invoice is valid, but the department budget is already exhausted. Finance approval is required before payment can be scheduled.
```

## 26. Budget-Aware Credit Cards

Credit card spend must respect budgets.

Flow:

```text
card transaction received
-> card spend budget checked
-> category budget checked
-> employee/department/project budget checked
-> if over limit, exception created
-> personal/recovery or approval path if needed
```

Document 6 owns card transaction workflow.

Document 7 owns budget policy.

## 27. Budget-Aware Payroll And Hiring

Hiring and pay changes affect budget.

Flow:

```text
new position / salary change / overtime / bonus request
-> Payroll/HR/Position evidence
-> budget check
-> profit floor check
-> cashflow check
-> approval route
```

Selene says:

```text
Adding this role would increase annual payroll cost by AUD 92,000 and reduce forecast net profit from 8.6% to 7.9%. This needs Finance review because it breaches the profit floor.
```

## 28. Budget Reporting And Presentations Boundary

Budgeting needs reports, but full reporting belongs to future:

```text
Document 17 - Financial Reporting + Analytics + Board Packs + Device Presentation Master Design
```

Document 7 must provide report-ready data:

```text
budget vs actual
budget vs committed
budget vs forecast
variance
approval history
cash reserve status
profit floor status
budget increase requests
department budget status
board approval status
```

Presentation owner split:

```text
Finance/Budget = report truth
Accounting = actuals
PH1.D = explanation draft
PH1.WRITE = final wording
Desktop/iPhone = render
BCAST/DELIVERY = approved distribution
Access = who can view/export/share
```

Selene must produce board-ready summaries, but device presentation is handled later.

## 29. User Natural Language Examples

User:

```text
Selene, can we afford to buy another van?
```

Selene checks:

```text
capex budget
cash reserve
cashflow forecast
profit floor
loan/lease options
depreciation/tax impact
approval thresholds
```

Selene says:

```text
The vehicle capex budget has room for one more van, but buying it outright would bring cash close to the reserve floor. Leasing keeps cash safer but adds monthly obligations. I recommend CFO review before approval.
```

User:

```text
Selene, increase the marketing budget by 50,000.
```

Selene checks:

```text
authority
current budget
reason
cashflow
profit floor
approval threshold
board requirement
```

Selene says:

```text
I can prepare that request. It would increase annual marketing spend by AUD 50,000 and reduce forecast net profit from 8.4% to 8.1%. It needs CFO approval before it becomes active.
```

## 30. PH1.D / GPT-5.5 Role

Allowed:

```text
summarize budget variance
draft board explanation
explain cash reserve warning
suggest possible corrective actions
compare options in plain English
draft budget increase justification
draft department manager summary
explain profit floor breach
translate budget summaries
```

Forbidden:

```text
approve budget
approve spend
change budget
override reserve
override profit floor
invent actuals
invent forecast truth
hide variance
move spend categories to avoid approval
```

## 31. PH1.WRITE Wording

PH1.WRITE owns final user-facing wording.

### Budget Warning

```text
This purchase would take the Warehouse Repairs budget AUD 5,700 over the approved quarterly limit. Finance approval is required before it can continue.
```

### Board Summary

```text
The proposed annual budget keeps forecast net profit above the 8% floor, preserves the AUD 12 million cash reserve, and funds the planned warehouse expansion. The main risk is overtime growth in Q3.
```

### Reserve Warning

```text
This payment would reduce cash below the approved reserve. I can request emergency approval or propose a safer payment schedule.
```

## 32. Access And Authority

Protected actions:

```text
create annual budget
approve annual budget
activate budget
change active budget
increase budget
approve over-budget spend
set cash reserve
override cash reserve
set profit floor
change profit floor
approve emergency spend
approve board budget pack
export budget report
view restricted budget
```

Authority depends on:

```text
role
legal entity
country
amount
budget type
department
cost center
project
risk level
cash reserve impact
profit floor impact
board policy
```

Step-up may be required for:

```text
board approval
cash reserve override
large budget increase
profit floor change
capex approval
emergency spend override
budget activation
```

## 33. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
budget_id
budget_line_id
old_value_ref
new_value_ref
amount
currency
source_evidence_refs
approval_refs
step_up_refs
cashflow_check_ref
profit_floor_check_ref
board_approval_ref
reason_code
timestamp
company_id
legal_entity_id
country
period
```

No silent budget changes.

No hidden reserve override.

No unlogged board approval.

## 34. Failure Branches

### Budget Missing

```text
No active budget found.
Selene blocks controlled spend or routes Finance review depending policy.
```

### Budget Exceeded

```text
Spend request exceeds budget.
Approval or increase required.
```

### Cash Reserve Breach

```text
Payment/spend would breach reserve.
Emergency override required.
```

### Profit Floor Breach

```text
Spend or budget change would lower forecast profit below floor.
Executive/board review required.
```

### Approver Unavailable

```text
Approval pending.
Reminder scheduled.
Escalation based on policy.
No execution until approval.
```

### Forecast Uncertainty

```text
Forecast confidence low.
Selene explains uncertainty and requests missing data.
```

### Board Approval Missing

```text
Board-required action remains blocked.
```

## 35. Required Logical Packets

Future logical packets:

```text
BudgetPacket
BudgetLinePacket
BudgetPolicyPacket
BudgetCheckPacket
BudgetCommitmentPacket
BudgetVariancePacket
BudgetIncreaseRequestPacket
BudgetApprovalRequestPacket
BoardApprovalRequestPacket
MultiAuthorityApprovalPacket
CashReservePolicyPacket
CashReserveBreachPacket
ProfitFloorPolicyPacket
ProfitFloorBreachPacket
SpendFreezeRecommendationPacket
SpendForecastPacket
BudgetReportDataPacket
BudgetAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 36. Example - Annual Budget Approved

```text
Selene prepares annual budget.
Management reviews.
CFO adjusts payroll forecast.
Board approves.
Budget becomes Active.
Selene begins checking spend against active budget.
```

Selene says:

```text
The annual budget is now active. I'll use it to check purchase orders, payroll changes, card spend, contractor invoices, and capex requests from today.
```

## 37. Example - Budget Overrun Request

```text
Warehouse repairs budget: AUD 50,000
Actual + committed: AUD 48,000
New repair bill: AUD 9,400
Overrun: AUD 7,400
```

Selene says:

```text
This repair would exceed the Warehouse Repairs budget by AUD 7,400. I can request a budget increase or hold the invoice until Finance approves the overrun.
```

## 38. Example - Profit Floor Breach

```text
Company profit floor: 8%
Forecast after new campaign: 7.6%
```

Selene says:

```text
This campaign would push forecast net profit below the approved 8% floor. I can prepare an approval request with the expected revenue upside, but it needs executive review before proceeding.
```

## 39. Example - Reserve Override

```text
Cash reserve: AUD 12,000,000
Payment: AUD 450,000
Forecast cash after payment: AUD 11,840,000
Payment type: critical supplier
```

Selene says:

```text
This payment would breach the reserve by AUD 160,000. Because it is a critical supplier, I can request emergency override approval from the CFO and CEO.
```

## 40. What Must Not Happen

```text
no active budget changed without authority
no budget increase without reason and audit
no board-required budget approved by lower role
no cash reserve set by unauthorized user
no reserve breach without emergency override
no profit floor ignored
no over-budget spend silently allowed
no committed spend ignored in remaining budget
no forecast presented as actual
no GPT-5.5 budget approval
no PH1.D invented actuals or forecast truth
no client/adapter budget authority
no hidden spend freeze
no payroll/tax/legal obligations frozen casually
no budget reports shared without Access
no implementation from this document alone
```

## 41. Future Simulation Targets

```text
SIM_BUDGET_001_annual_budget_creation_and_board_approval
SIM_BUDGET_002_department_budget_overrun_finance_approval
SIM_BUDGET_003_budget_increase_request
SIM_BUDGET_004_cash_reserve_policy_set_by_authorized_exec
SIM_BUDGET_005_cash_reserve_breach_emergency_override
SIM_BUDGET_006_net_profit_floor_warning
SIM_BUDGET_007_profit_floor_blocks_discretionary_spend
SIM_BUDGET_008_capex_budget_check_for_vehicle_purchase
SIM_BUDGET_009_card_spend_budget_over_limit
SIM_BUDGET_010_payroll_hiring_request_profit_floor_check
SIM_BUDGET_011_project_budget_forecast_overrun
SIM_BUDGET_012_board_approval_2_of_3_directors
SIM_BUDGET_013_budget_commitment_released_after_cancelled_PO
SIM_BUDGET_014_spend_freeze_recommendation
```

## Related Addendum

Solution-first budget resolution, purchase commitment control, reallocation before budget increase, revenue and cost offset search, lease-vs-buy intelligence, over-budget root-cause/prevention, approval-only-when-necessary law, and profit protection intelligence are defined in SELENE_BUDGETING_BUDGET_RESOLUTION_PURCHASE_COMMITMENT_PROFIT_PROTECTION_ADDENDUM.md and must be read with this document.

## 42. Final Architecture Sentence

Selene Budgeting + Spend Control + Board Approval is the governed financial planning and spend-control engine: it creates and activates annual, department, project, payroll, card, capex, and cost-center budgets; monitors actual, committed, and forecast spend; protects minimum cash reserves and net profit floors; checks every controlled spend before commitment or payment; routes over-budget, emergency, reserve-breach, and board-level decisions through the correct authority gates; prepares human and board-ready explanations through PH1.WRITE; and ensures Selene runs the company against approved financial strategy rather than reacting after the money has already escaped.
