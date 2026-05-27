# Selene Budgeting Addendum — Budget Resolution + Purchase Commitment Control + Profit Protection Intelligence

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / BUDGET RESOLUTION + PURCHASE COMMITMENT CONTROL + PROFIT PROTECTION INTELLIGENCE

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Document 7 — Selene Budgeting + Spend Control + Board Approval Master Design

PURPOSE:
Strengthen Document 7 so Selene does not merely report budget problems or request approvals. Selene must actively prevent budget overruns at source, manage purchase commitments before spend occurs, rebalance budgets intelligently, protect minimum net profit targets, increase revenue where possible, reduce or delay non-essential costs, compare purchase/lease/finance alternatives, and escalate to management or board only when a real human governance decision is required.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 4. It extends the Selene Budgeting + Spend Control + Board Approval Master Design. It does not implement Budgeting, Procurement, Purchase Orders, Cashflow, Board, Shareholder, Assets, Fleet, Insurance, Debt, Reporting, Sales, Marketing, AR, AP, Accounting, Access, PH1.D, PH1.WRITE, packets, migrations, tests, or runtime state.

Current repo truth does not prove runtime Budget Resolution, Purchase Commitment Control, Profit Protection, Board, Shareholder, Asset, Fleet, Insurance, Debt, or Reporting ownership. This addendum is future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Master Addendum Law

Selene must not behave like an old budget system that waits until money is gone and then reports the problem.

The upgraded budget law is:

```text
Budget control begins before commitment, not after payment.

Purchase orders, card spend, payroll changes, asset purchases, contractor invoices, supplier bills, and new commitments must be checked before the company is locked into spend.

Selene must attempt intelligent resolution before escalation.

Selene must protect approved budgets, minimum cash reserve, and net profit floor.

Selene may rebalance within approved budget scope where policy allows.

Selene may propose revenue increases, cost reductions, supplier negotiation, timing changes, substitution, lease/buy alternatives, or spend deferral before asking for budget increases.

Approval is required only when policy, authority, risk, protected execution, or unresolved trade-off demands it.

Selene must refuse or hold spend when approving it would damage the company and no safe resolution exists.
```

Short version:

```text
Selene should solve first.
Escalate second.
Block when necessary.
Audit always.
```

## 2. Solution-First Budget Intelligence

When Selene detects a budget problem, her first move must not be:

```text
You are over budget. Ask CFO?
```

Her first move should be:

```text
Why are we over budget?
Can we avoid it?
Can we reduce it?
Can we reallocate safely?
Can we increase revenue?
Can we delay or stage it?
Can we use another supplier?
Can we lease instead of buy?
Can we protect profit and cash without bothering executives?
```

Required resolution sequence:

```text
1. Confirm the spend is necessary.
2. Check whether budget exists and whether spend is already committed.
3. Check actual vs committed vs forecast budget.
4. Identify why budget pressure exists.
5. Search for offsetting underspend in same department/category/project/entity.
6. Search for supplier, quantity, timing, substitute, or scope alternatives.
7. Check whether revenue can be accelerated or increased to offset spend.
8. Check whether price increases, campaigns, collections, or sales pushes can solve the gap.
9. Check cash reserve and net profit floor.
10. Propose best resolution path.
11. Only escalate if Selene cannot solve within approved rules.
```

## 3. Purchase Commitment Control

Budget must be checked at the purchase order stage, before the company commits.

Flow:

```text
purchase request
-> business need check
-> budget check
-> committed spend check
-> cashflow check if material
-> profit floor check if material
-> supplier/price alternative check
-> reallocation/offset check
-> approval only if required
-> purchase order created
-> budget commitment recorded
```

Rule:

```text
No controlled purchase order may be committed without budget status being known.
```

Selene must check:

```text
requester authority
purchase category
supplier
item/service needed
budget line
available budget
already committed spend
forecast remaining spend
department/project/cost center scope
cashflow impact
profit impact
supplier alternatives
timing alternatives
approval threshold
```

Example:

```text
User wants to raise a PO for motor oil.

Selene checks:
- Fleet maintenance budget
- motor oil line
- other fleet lines under/over
- stock levels
- supplier price history
- consumption trend
- cashflow impact
- whether bulk buy saves money
- whether another supplier is cheaper
```

Selene says:

```text
This PO would take the motor-oil line over budget, but the overall fleet maintenance budget is still under by AUD 6,500. I can rebalance within the fleet budget and keep the total budget on target.
```

## 4. Budget Level Hierarchy

Selene must understand budget levels.

A line-item overrun may be acceptable if the higher-level budget remains healthy and policy allows it.

Budget levels:

```text
line item
category
department
cost center
project
location
legal entity
company
group
cash reserve
net profit floor
```

Example:

```text
Motor oil line is over budget by AUD 4,000.
Tyres line is under budget by AUD 3,000.
Servicing line is under budget by AUD 4,500.
Overall fleet budget is still under by AUD 3,500.
```

Selene should recommend:

```text
Rebalance within Fleet budget.
No total budget increase needed.
Audit category movement.
Monitor motor oil usage trend.
```

Selene says:

```text
Motor oil is over budget, but Fleet overall is still on target because tyres and servicing are under. I recommend rebalancing within Fleet instead of increasing total spend.
```

Hard rule:

```text
Selene may rebalance only where policy allows and must never hide line-level overruns.
```

## 5. Budget Reallocation Before Budget Increase

Before requesting more budget, Selene must try reallocation.

Reallocation sources:

```text
unused budget in same category
unused budget in same department
unused project contingency
lower-priority spend not yet committed
cancelled PO commitment
delayed purchase
supplier credit
cost recovery
insurance recovery
customer reimbursement
```

Reallocation must check:

```text
same legal entity
same department or approved cross-department rule
same funding source
tax/accounting treatment
cashflow effect
profit floor effect
approval requirement
whether original budget purpose is still needed
```

Reallocation options:

```text
future-only budget line shift
temporary internal transfer
project contingency use
department reallocation
category consolidation
spend deferral
commitment release
```

Selene must ask only when approval is required.

Example:

```text
We can cover the motor oil overrun by reducing unused cleaning supplies and delaying non-critical toolbox replacements. This keeps the Warehouse budget on target.
```

## 6. Increase Revenue Before Cutting Everything

If a budget overrun threatens profit or cashflow, Selene should look for revenue opportunities before simply blocking all spend.

Possible revenue actions:

```text
chase overdue invoices
accelerate due-soon receivables
offer early payment discount if beneficial
run targeted sales campaign
increase price where demand supports it
reduce unnecessary discounting
push high-margin products
clear slow stock
follow up abandoned quotes
renew subscriptions/contracts
upsell existing customers
activate marketing campaign
increase service capacity if profitable
```

Owner split:

```text
Budget detects profit/budget pressure.
Cashflow ranks urgency.
AR collects receivables.
Sales/Marketing owns campaigns and offers.
Pricing owner controls price changes.
PH1.WRITE drafts messages.
BCAST/DELIVERY sends approved outreach.
Accounting/Finance measure result.
```

Example:

```text
Operating costs are forecast AUD 30,000 over budget.
Selene identifies AUD 120,000 overdue receivables and a high-margin product campaign likely to generate AUD 42,000 gross margin.
Selene recommends collection + sales action before budget increase.
```

Selene says:

```text
We can offset this overrun without increasing budget if we collect the top overdue invoices and run a targeted campaign on high-margin stock. I recommend trying that first.
```

## 7. Cost Reduction Before Budget Increase

Selene should search for cost reductions.

Possible reductions:

```text
supplier renegotiation
bulk purchase discount
alternative supplier
substitute item
reduce quantity
delay non-critical purchase
cancel unused subscription
pause discretionary spend
reduce overtime through roster adjustment
use inventory on hand
repair instead of replace
lease instead of buy
buy instead of lease
share equipment across departments
reduce freight cost
combine deliveries
```

Selene should not slash important operations blindly.

Cost reduction must consider:

```text
operational impact
customer impact
safety impact
legal/compliance risk
quality risk
warranty/insurance impact
long-term cost
cashflow impact
profit impact
```

## 8. Lease Vs Buy Vs Finance Decision Intelligence

When a major asset is needed, Selene must compare options.

Options:

```text
cash purchase
loan purchase
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

Selene must compare:

```text
cash reserve impact
monthly payment
total cost of ownership
interest cost
tax treatment
depreciation
GST/VAT treatment
claimability
insurance cost
maintenance cost
expected resale value
downtime risk
asset utilization
balance sheet impact
debt covenants
collateral/security
profit floor impact
cashflow timing
board/capex approval requirement
```

Example:

```text
Delivery van needed.

Cash purchase:
- lowest total cost
- breaches cash reserve for 6 weeks

Lease:
- higher total cost
- protects cash reserve
- predictable monthly payments

Loan:
- moderate cash impact
- creates debt liability
- interest cost applies
```

Selene says:

```text
Buying the van outright is cheaper overall, but it breaches the cash reserve. Leasing costs more over 36 months but keeps cash stable. I recommend leasing unless the board approves a temporary reserve breach.
```

This connects to:

```text
Document 8 - Cashflow Forecasting + Payment Priority
Document 9 - Assets + Depreciation + Claimable Expense Rules
Document 15 - Debt + Loans + Lease Liability + Treasury Obligations
PH1.ASSET - Asset Intelligence
PH1.FLEET - Vehicle Fleet
Tax / Compliance
Accounting
Insurance
```

## 9. Net Profit Floor Protection

Selene must continuously protect the company's approved net profit floor.

Example:

```text
Board policy: net profit must not fall below 8%.
```

When spending threatens the floor, Selene must first search for solutions.

Profit protection options:

```text
reduce cost elsewhere
delay spend
increase revenue
increase pricing
reduce discounts
improve gross margin
collect overdue receivables
reduce overtime
renegotiate supplier cost
use inventory on hand
pause discretionary spend
rebalance budgets
seek higher approval
```

Profit floor breach workflow:

```text
forecast detects breach
-> identify drivers
-> split one-off vs recurring
-> propose corrective actions
-> test impact of each action
-> choose best path within policy
-> escalate only if human decision required
```

Selene says:

```text
Forecast net profit is falling below the 8% floor. The biggest causes are overtime, freight, and discounting. I recommend reducing overtime first and pausing discretionary purchases before asking for a budget increase.
```

## 10. Over-Budget Cause And Prevention Intelligence

Reports must explain why something happened and how to prevent it.

Every over-budget report should include:

```text
what went over budget
how much over budget
when it started
source transactions
main cause
one-off or recurring
responsible owner
preventive action
Selene's recommended fix
whether Selene already acted
whether approval is required
future monitoring trigger
```

Example:

```text
Warehouse freight is over budget by AUD 18,000.

Cause:
- emergency express freight increased after late supplier deliveries.

Prevention:
- order stock 7 days earlier
- add supplier delivery warning
- use consolidated weekly freight
- require approval for express freight above threshold
```

Selene says:

```text
This overrun is mostly preventable. It came from emergency freight after late ordering. I recommend earlier stock reorder triggers and approval for express freight above AUD 1,000.
```

This connects to future:

```text
Document 17 - Financial Reporting + Analytics + Board Packs + Device Presentation
```

## 11. Approval-Only-When-Necessary Law

Selene should not interrupt management for every small issue.

Approval is required when:

```text
policy requires approval
amount exceeds authority
protected action involved
cash reserve breach
net profit floor breach cannot be resolved
board threshold reached
legal/tax/compliance risk
supplier/customer/employee risk
budget reallocation outside allowed scope
human judgment needed
all safe resolution paths failed
```

Approval is not required when:

```text
policy allows auto-rebalance
spend remains within higher-level budget
cash reserve remains safe
profit floor remains safe
risk is low
supplier/price/quantity correction resolves issue
budget commitment can be released
timing shift solves problem
human approval already exists in policy
```

Selene must document the resolution even when approval is not required.

## 12. PO Refusal / Hold Authority

Selene must be allowed to hold or refuse a purchase order when it prevents harm.

Hold/refuse reasons:

```text
budget breach with no offset
cash reserve breach
net profit floor breach
supplier risk
duplicate purchase
unnecessary item
no business purpose
no receiving/inspection owner
bad ROI
better cheaper option available
wrong account/category
non-compliant purchase
high-risk timing
```

Selene says:

```text
I'm holding this purchase order. It would breach budget and reduce forecast profit below target, and I cannot find an approved offset. I can show you safer alternatives.
```

This is Selene doing the job humans hired her to do: prevent unsafe commitments before they become expensive facts.

## 13. Budget Recovery Plan

When a budget goes off track, Selene should create a recovery plan.

Recovery plan fields:

```text
recovery_plan_id
budget_id
budget_line_id
problem_summary
root_causes
current_variance
forecast_variance
cashflow_impact
profit_impact
recommended_actions
expected_recovery_amount
owner_assignments
timeline
approval_required
status
audit_ref
```

Recovery plan actions:

```text
reduce spend
pause non-essential purchases
renegotiate supplier
collect receivables
increase price
launch sales campaign
rebalance budget
delay purchase
switch supplier
lease instead of buy
buy instead of lease
recover personal charges
release cancelled commitments
```

Statuses:

```text
Draft
AutoResolvable
PendingOwnerAction
PendingApproval
InProgress
Resolved
Escalated
Failed
Closed
```

## 14. Budget Resolution Decision Packet

Future logical packet:

```text
BudgetResolutionDecisionPacket:
  decision_id
  source_request_ref
  budget_id
  budget_line_id
  detected_issue
  issue_severity
  root_cause_summary
  available_budget
  committed_spend
  actual_spend
  forecast_spend
  cash_reserve_impact
  profit_floor_impact
  resolution_options
  recommended_option
  approval_required
  authority_scope
  audit_ref
```

Resolution option fields:

```text
option_id
option_type
estimated_savings
estimated_revenue_increase
cashflow_impact
profit_impact
operational_risk
customer_impact
time_to_effect
approval_required
recommended
```

## 15. Revenue Offset Packet

Future logical packet:

```text
RevenueOffsetOpportunityPacket:
  opportunity_id
  budget_issue_ref
  source_owner: AR / Sales / Marketing / Pricing
  expected_cash_increase
  expected_margin_increase
  time_to_cash
  confidence
  required_actions
  approval_required
  audit_ref
```

Examples:

```text
collect top overdue invoices
run targeted campaign
raise price on high-demand product
reduce discounts
send renewal reminders
sell slow stock
upsell existing customer
```

## 16. Cost Offset Packet

Future logical packet:

```text
CostOffsetOpportunityPacket:
  opportunity_id
  budget_issue_ref
  source_owner
  cost_to_reduce
  estimated_savings
  affected_department
  operational_impact
  risk_level
  approval_required
  audit_ref
```

Examples:

```text
delay non-critical purchase
release unused PO commitment
switch supplier
reduce quantity
cancel unused subscription
rebalance underused category
negotiate discount
```

## 17. Commitment Release Law

Budgets must recognize commitments.

If a PO or contract is cancelled, Selene should release budget capacity.

Flow:

```text
commitment created
-> budget committed amount increases
-> purchase cancelled or reduced
-> commitment release validated
-> budget available amount restored
-> audit
```

Example:

```text
A cancelled AUD 12,000 PO releases budget capacity.
Selene can use that released capacity to solve another budget pressure without increasing total budget.
```

## 18. Autonomous Budget Concierge Behavior

Selene should behave like a capable financial operator.

Tone:

```text
clear
human
practical
solution-first
not panicked
not robotic
not approval-addicted
```

Selene should say:

```text
I found three ways to handle this without increasing the budget.
```

Not:

```text
Budget exceeded. Approval required.
```

Unless approval really is required.

Selene should explain:

```text
what happened
why it matters
what she checked
what options exist
what she recommends
what needs approval
what happens if ignored
```

## 19. Board / Management Escalation

Escalation should be structured and rare enough to matter.

Escalation levels:

```text
self-resolved within policy
manager review
Finance review
CFO review
CEO review
board review
shareholder/major governance review where required
```

Escalate to higher management when:

```text
company strategy changes
profit floor threatened
cash reserve threatened
major capex needed
budget reallocation crosses authority
overrun is recurring and material
Selene's available solutions have trade-offs requiring human judgment
```

Escalation packet:

```text
BudgetEscalationPacket:
  escalation_id
  issue_ref
  attempted_resolution_summary
  unresolved_risks
  options
  Selene_recommendation
  required_decision
  required_approvers
  deadline
  audit_ref
```

## 20. Board / Shareholder / Governance Dependencies

This addendum references future engines:

```text
PH1.BOARD - Board Governance + Voting + Resolution + Executive Oversight
PH1.SHAREHOLDER - Shareholder Registry + Voting + Class Rights + Equity Governance
Document 14 - Equity + Shareholder Distributions + Dividends
Document 17 - Financial Reporting + Analytics + Board Packs + Device Presentation
```

Document 7 may create board approval requests, but PH1.BOARD owns board process.

Document 7 may flag shareholder-level impact, but PH1.SHAREHOLDER and Equity/Dividends own shareholder governance.

## 21. Asset / Fleet / Insurance / Debt Dependencies

This addendum references future engines:

```text
PH1.ASSET - Asset Intelligence + Lifecycle + Value Optimization
PH1.FLEET - Vehicle Fleet Management
PH1.INSURANCE - Insurance Policy + Renewal + Claims + Risk Coverage
Document 15 - Debt + Loans + Lease Liability + Treasury Obligations
Document 16 - Real Estate + Property Assets + Rental Income + Collateral
```

Budget decisions involving vehicles, buildings, insurance, loans, leases, or assets must ask the correct owners for evidence.

Example:

```text
Selene cannot decide lease vs buy vehicle from budget alone.

She must ask:
- Fleet for operational need and utilization
- Asset for lifecycle/value
- Tax for treatment
- Debt/Lease for liability
- Insurance for premium/coverage
- Cashflow for reserve impact
- Accounting for book treatment
```

## 22. What Must Not Happen Additions

Add to Document 7:

```text
no budget control only after payment
no purchase order committed without budget check where policy requires it
no budget increase request before Selene checks reallocation and offsets
no approval escalation before Selene checks safe resolution options
no line-item overrun hidden inside category rebalance
no rebalance outside approved scope
no cash reserve breach hidden as budget reallocation
no net profit floor breach ignored
no revenue opportunity ignored before delaying critical spend
no supplier/payment deferral used before collection and revenue options are considered
no PO refusal without explanation and alternatives
no lease-vs-buy decision without cashflow, tax, asset, debt, and insurance evidence
no report that says "over budget" without cause and prevention
no GPT-5.5 budget approval
no PH1.D invented actuals, forecasts, or savings
no board escalation without attempted resolution summary unless policy requires immediate escalation
no implementation from this addendum alone
```

## 23. Required Logical Packets

Future logical packets:

```text
BudgetResolutionDecisionPacket
PurchaseCommitmentControlPacket
BudgetRebalanceCandidatePacket
BudgetReallocationRequestPacket
RevenueOffsetOpportunityPacket
CostOffsetOpportunityPacket
LeaseBuyScenarioPacket
ProfitProtectionPlanPacket
OverBudgetRootCausePacket
OverBudgetPreventionPlanPacket
BudgetRecoveryPlanPacket
CommitmentReleasePacket
BudgetEscalationPacket
POHoldDecisionPacket
SpendAlternativeRecommendationPacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 24. Example - Motor Oil Budget Overrun

```text
Issue:
Motor oil purchases will exceed budget by AUD 4,000.

Selene checks:
- fleet maintenance total budget
- tyres budget
- servicing budget
- oil consumption trend
- supplier price changes
- bulk discount options
- alternative supplier
- vehicle usage increase
- cashflow impact
- profit floor impact
```

Result:

```text
Tyres budget under by AUD 3,000.
Servicing budget under by AUD 4,500.
Overall fleet maintenance still under by AUD 3,500.
Selene recommends rebalancing within Fleet and monitoring oil usage.
No total budget increase required.
```

Selene says:

```text
Motor oil is over its line budget, but Fleet overall is still on target. I recommend rebalancing within Fleet instead of increasing total budget. I'll also monitor oil usage because this may become recurring.
```

## 25. Example - Purchase Order Refused

```text
Request:
New warehouse equipment, AUD 24,000.

Checks:
Budget unavailable.
No offset available.
Cash reserve safe but profit floor would fall below target.
Supplier alternative found at AUD 18,000.
Repair existing equipment costs AUD 7,500.
```

Selene says:

```text
I'm holding this PO. It would breach the approved equipment budget and reduce forecast profit below target. I found two alternatives: repair the current equipment for AUD 7,500 or use another supplier at AUD 18,000. I recommend the repair option unless Operations confirms replacement is essential.
```

## 26. Example - Revenue Offset Instead Of Budget Increase

```text
Problem:
Marketing campaign over budget by AUD 20,000.

Selene finds:
- same campaign generated high-margin sales last year
- current slow stock can be sold with targeted offer
- AR has AUD 80,000 due within 14 days
```

Selene says:

```text
The campaign is over budget, but it may still protect profit if we pair it with a high-margin stock clearance and collect the top overdue invoices. I recommend this recovery plan before asking for a budget increase.
```

## 27. Example - Lease Vs Buy Vehicle

```text
Need:
new delivery van

Option A:
cash purchase
lower total cost
breaches reserve for 6 weeks

Option B:
lease
higher total cost
protects cash reserve
predictable monthly payment

Option C:
repair old van
lowest cash impact
higher breakdown risk
```

Selene says:

```text
Buying is cheapest overall, leasing protects cash, and repairing is cheapest short-term but riskier. Based on the cash reserve and reliability risk, I recommend leasing unless the board approves a temporary reserve breach for purchase.
```

## 28. Future Simulation Targets

```text
SIM_BUDGET_015_po_budget_check_before_commitment
SIM_BUDGET_016_motor_oil_budget_rebalance
SIM_BUDGET_017_budget_increase_only_after_offset_search
SIM_BUDGET_018_overbudget_revenue_offset_strategy
SIM_BUDGET_019_po_refused_with_alternatives
SIM_BUDGET_020_lease_vs_buy_vehicle_decision
SIM_BUDGET_021_profit_floor_recovery_plan
SIM_BUDGET_022_cash_reserve_breach_solution_search
SIM_BUDGET_023_commitment_release_after_cancelled_po
SIM_BUDGET_024_board_escalation_after_solution_attempts
SIM_BUDGET_025_overbudget_report_with_prevention_plan
```

## 29. Final Addendum Architecture Sentence

Selene Budgeting must become a solution-first financial governance engine: before spend is committed, Selene checks budget, committed spend, cash reserve, and profit floor; searches for reallocation, cost reduction, supplier alternatives, timing changes, revenue acceleration, and lease/buy strategies; holds or refuses unsafe purchase orders when needed; explains root cause and prevention; and escalates to management or board only when a real authority decision remains after Selene has attempted every safe, governed resolution path.
