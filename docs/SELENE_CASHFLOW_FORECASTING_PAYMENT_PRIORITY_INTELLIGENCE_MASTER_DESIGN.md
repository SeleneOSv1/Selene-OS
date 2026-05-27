# Selene Cashflow Forecasting + Payment Priority Intelligence Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / CASHFLOW FORECASTING + PAYMENT PRIORITY + CASH RISK INTELLIGENCE + AUTONOMOUS CASH ACTIONS

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's cashflow intelligence engine: real-time bank-aware cash forecasting, minimum cash reserve protection, incoming-cash acceleration, debtor collection priority, sales/revenue activation, outgoing payment prioritization, critical payment timing, payment deferral strategy, cash shortage prevention, cash risk modes, scenario planning, board/management escalation, and autonomous cashflow recovery.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 4. It defines future Cashflow Forecasting, Payment Priority Intelligence, cash risk modes, incoming cash acceleration, outgoing payment prioritization, scenario planning, reserve protection, critical payment timing, and cash recovery planning. It does not implement Cashflow, Banking, Payment Provider, Budgeting, Accounting, AP, AR, Payroll, Tax, Sales, Marketing, POS, Inventory, Debt, Insurance, Credit Cards, Dividends, Board, Shareholder, Desktop, iPhone, Adapter, packets, migrations, tests, or runtime state.

Current repo truth does not prove complete runtime Cashflow, Payment Priority, or autonomous cash recovery ownership. This document is future architecture pending Grand Architecture Reconciliation.

There is no Document 8 addendum in Finance/Accounting Design Batch 4.

## 1. Executive Target

Cashflow is not just "how much money is in the bank."

Cashflow is:

```text
what cash exists now
what cash is expected to arrive
what cash must leave
when each movement happens
which payments are critical
which receipts are late
which actions can improve cash
which obligations cannot be missed
what happens if nothing changes
```

Old cashflow process:

```text
human checks bank account
human checks unpaid invoices
human checks bills
human forgets loan payment
human delays supplier
human panics before payroll
human asks accountant for spreadsheet
everyone pretends this was unavoidable
```

Selene cashflow process:

```text
Selene reads live/stale bank balances
Selene reads AR, AP, payroll, tax, cards, loans, leases, subscriptions, budgets, POS, sales forecasts, and scheduled payments
Selene forecasts cash 7 / 14 / 30 / 60 / 90 / 180 days ahead
Selene detects cash gaps early
Selene first chases receivables
Selene second activates sales/revenue actions
Selene third manages outgoing payments by priority
Selene protects payroll, tax, rent, loans, leases, insurance, and critical suppliers
Selene routes approvals only when a real decision is needed
Selene reports cause, solution, risk, and next action
```

The target:

```text
real-time cash awareness
forecasted cash position
cash risk modes
receivables-first recovery
sales/revenue activation
payment-priority intelligence
reserve protection
critical payment timing
payment deferral and negotiation
cashflow scenario planning
bank/provider proof
management/board escalation only when needed
```

Tiny miracle: Selene does not wait until Friday to discover payroll is on Monday.

## 2. Master Law

```text
Cashflow problems must be predicted before they become emergencies.

Cashflow recovery starts by increasing incoming cash before delaying outgoing obligations.

Selene must collect money owed first, increase revenue second, and manage outgoing payments third.

Critical obligations must not be delayed casually.

Minimum cash reserve must be protected unless emergency override passes.

Net profit floor must be considered when cash actions affect margin.

Outgoing payments must be ranked by legal, operational, credit, supplier, payroll, tax, customer, and cash impact.

Selene must propose solutions before escalating approvals.

PH1.D/GPT-5.5 may help explain, summarize, draft, and propose.

PH1.D/GPT-5.5 must not decide cash truth, approve payments, execute payments, override reserve, or invent forecast truth.
```

Short law:

```text
Collect faster.
Sell smarter.
Pay carefully.
Escalate only when necessary.
Audit everything.
```

## 3. Owner Split

### Cashflow Intelligence Owns

```text
cash forecast
cash gap detection
cash risk mode
payment priority ranking
cash reserve impact
cash recovery plan
incoming-cash acceleration recommendation
outgoing payment prioritization
critical payment risk
cash scenario comparison
cashflow exception
cashflow audit evidence
```

### Banking Owns

```text
bank balances
available balance
ledger balance
bank feeds
payment confirmations
bank transaction proof
payment provider status
bank feed freshness
credit card settlement evidence
```

Cashflow consumes bank truth. Cashflow does not invent balances.

### Accounts Receivable Owns

```text
customer invoices
customer balances
receipts
debtor aging
promise-to-pay
payment plans
collections status
expected customer receipts
```

Cashflow asks AR what money can realistically come in.

### Accounts Payable Owns

```text
supplier bills
contractor invoices
installments
leases
scheduled outgoing payments
payment terms
due dates
supplier payment readiness
```

Cashflow ranks AP payments. AP owns bill truth.

### Payroll Owns

```text
payrun timing
payroll amount
employee payment obligations
payroll tax/super/pension/CPF liabilities
bonus/commission payout timing
salary advance recovery
```

Payroll is critical cashflow truth.

### Tax / Compliance Owns

```text
tax payment obligations
GST/VAT remittance
payroll tax
withholding
super/pension/CPF remittance rules
statutory due dates
```

Tax payments are critical. Cashflow must not casually delay them.

### Budget / Finance Owns

```text
budget policy
minimum cash reserve
net profit floor
spend control
budget recovery strategy
board financial policy
cash governance
```

### Sales / Marketing Owns

```text
sales campaigns
pricing actions
customer outreach for sales
renewal campaigns
discount campaigns
high-margin push campaigns
slow-stock clearance campaigns
```

Cashflow can recommend revenue actions. Sales/Marketing owns execution.

### POS / Commerce Owns

```text
sales events
payment attempts
card/cash takings
store/e-commerce receipts
merchant settlement evidence
```

### Inventory Owns

```text
stock availability
slow stock
high-margin stock
stock reorder requirements
stock liquidation candidates
COGS evidence
```

### Access / Governance Owns

```text
who can view cashflow
who can change reserve
who can approve payment priority changes
who can approve cash reserve breach
who can approve payment deferral
who can approve sales discount campaign
who can approve supplier negotiation
who can approve board escalation
```

### PH1.REM Owns

```text
cash warning reminders
payment cutoff reminders
approval deadline reminders
customer promise-to-pay reminders
supplier negotiation follow-up reminders
cashflow review reminders
```

### PH1.BCAST / PH1.DELIVERY Owns

```text
payment approval requests
customer collection messages
supplier negotiation messages
management cash alerts
board cash reports
sales campaign delivery where approved
```

### PH1.WRITE Owns

```text
cashflow explanations
management summaries
customer payment reminders
supplier payment negotiation wording
cash risk alerts
board cash pack wording
approval request wording
```

### PH1.D / GPT-5.5 May Assist

```text
explain cashflow risk
draft customer collection messages
draft supplier negotiation messages
summarize cash gap causes
suggest cash recovery options
draft board cash summary
rank narrative options
explain forecast uncertainty
```

But must not:

```text
approve payment
execute bank transfer
override reserve
invent bank balances
invent customer receipts
invent sales forecasts
change payment priority alone
commit sales campaigns
change supplier terms
```

## 4. Cashflow Is Not Budget, Banking, Or Accounting

Selene must keep these owners separate.

```text
Banking = what cash actually exists and moved.
Accounting = what has been posted to the books.
Budgeting = what the company planned and approved.
Cashflow = when cash is expected to come in and go out.
```

Example:

```text
A supplier bill is approved.
Accounting may record AP liability.
Budget may show spend is allowed.
Cashflow may still say: pay next week, not today, because payroll and tax are due first.
```

Cashflow is timing intelligence.

Not all approved payments should be paid immediately.

Not all delayed payments are wise.

This is why Selene needs a cashflow brain instead of a "Pay Everything" button.

## 5. Cashflow Forecast Horizons

Selene must forecast across multiple horizons.

```text
today
next 3 days
7 days
14 days
30 days
60 days
90 days
180 days
12 months
custom date range
```

Each horizon serves different decisions:

```text
today / 3 days = urgent payment safety
7 days = payroll, critical bills, bank cutoff
14 days = collections and AP sequencing
30 days = monthly cash control
60 days = working capital trend
90 days = tax, payroll, debt, supplier risk
180 days = strategic cash planning
12 months = budget and board cash planning
```

Forecast fields:

```text
cashflow_forecast_id
company_id
legal_entity_id
country
currency
forecast_start
forecast_end
opening_cash_balance
available_cash
expected_receipts
expected_payments
scheduled_payments
critical_obligations
minimum_cash_reserve
forecast_closing_cash
lowest_forecast_cash_point
cash_gap_amount
cash_gap_date
risk_mode
confidence_score
data_freshness_summary
audit_ref
```

## 6. Cashflow Inputs

Selene must gather cashflow inputs automatically.

### Incoming Cash Sources

```text
open customer invoices
customer payment plans
promise-to-pay dates
expected POS sales
e-commerce sales forecast
subscription renewals
contract milestone billings
rent income
interest income
tax refunds
supplier refunds
insurance claim proceeds
asset sale proceeds
shareholder contributions
loan drawdowns
intercompany receipts
```

### Outgoing Cash Sources

```text
supplier bills
contractor invoices
employee payroll
payroll tax
super/pension/CPF
GST/VAT/tax payments
loan repayments
lease payments
rent
insurance premiums
credit card payments
subscriptions
utility bills
purchase orders
scheduled installments
refunds to customers
dividend payments
intercompany transfers
asset purchases
legal obligations
```

### Operational Forecast Sources

```text
sales forecast
marketing campaigns
inventory reorder plans
production plans
roster/overtime forecast
staffing plan
fleet maintenance forecast
asset replacement plan
insurance renewal schedule
debt/lease schedules
budget commitments
```

## 7. Data Freshness And Trust

Cashflow must know how reliable each input is.

Trust statuses:

```text
confirmed
provider_confirmed
approved
scheduled
forecast
estimated
customer_promised
historical_pattern
low_confidence
stale
missing
disputed
blocked
```

Examples:

```text
Bank balance from live feed = provider_confirmed
Customer promise to pay Friday = customer_promised
Sales forecast from prior season = forecast
Supplier bill due tomorrow = confirmed
Unmatched bank receipt = missing/allocation_pending
```

Selene must explain uncertainty:

```text
This forecast is medium confidence. The biggest uncertainty is whether three customers pay by Friday as promised.
```

## 8. Cash Risk Modes

Selene must classify cashflow risk into modes.

```text
GREEN
YELLOW
ORANGE
RED
BLACK
```

### GREEN - Normal

```text
Cash is healthy.
Reserve safe.
Critical obligations covered.
Payments proceed by schedule.
```

### YELLOW - Tightening

```text
Cash is safe but tightening.
Selene watches spending, sends early debtor reminders, and avoids unnecessary early payments.
```

### ORANGE - Cash Risk

```text
Cash gap possible.
Selene activates collections, revenue acceleration, payment prioritization, and management awareness.
```

### RED - Shortage Likely

```text
Cash shortage likely.
Selene protects payroll, tax, loans, leases, rent, and critical suppliers.
Optional spend freezes or delays.
Finance leadership alerted.
```

### BLACK - Critical

```text
Critical cash shortage.
Emergency approval required for non-essential payments.
Board/executive escalation may be required.
Selene prepares survival cash plan.
```

Risk mode fields:

```text
risk_mode
trigger_reason
cash_gap_amount
cash_gap_date
reserve_breach_date
critical_payment_at_risk
forecast_confidence
recommended_actions
escalation_required
audit_ref
```

## 9. Autonomous Cashflow Protocol

Selene's cashflow response must follow a strict order.

```text
1. Predict cash shortage before it happens.
2. Chase money owed.
3. Increase incoming cash / revenue.
4. Reduce or defer non-critical outgoing cash.
5. Prioritize critical payments.
6. Seek approval only when required.
7. Escalate unresolved risk.
```

This must be enforced as product behavior.

Bad behavior:

```text
Cash tight -> delay supplier payments immediately.
```

Correct Selene behavior:

```text
Cash tight -> collect receivables, accelerate sales, prioritize payments, then defer only what is safe.
```

Poor cash practice delays bills first. Selene does not.

## 10. Step 1 - Predict Cash Shortage Before It Happens

Selene must run scheduled and event-driven cash forecasts.

Triggers:

```text
daily cashflow check
new large AP bill
payroll draft
large customer payment delayed
bank balance drop
tax payment due soon
loan/lease payment due soon
large purchase order request
budget overrun
sales forecast drop
major refund request
credit card statement due
critical supplier payment due
```

Selene calculates:

```text
current available bank balance
+ expected customer receipts
+ expected POS/e-commerce receipts
+ expected other receipts
- payroll
- tax
- supplier bills
- contractor payments
- rent
- loans
- leases
- insurance
- scheduled card payments
- approved purchases
- other obligations
= future cash position
```

If risk detected:

```text
CashflowRiskPacket created.
Risk mode assigned.
Recovery plan generated.
```

## 11. Step 2 - Chase Money Owed First

Before delaying payments, Selene must look at AR.

Selene checks:

```text
overdue invoices
due-soon invoices
high-value unpaid invoices
customer payment history
customer likely-to-pay score
promise-to-pay dates
broken promises
disputed invoices
payment plan status
Selene-connected customer status
non-Selene customer channels
payment link availability
account manager relationship
```

Selene prioritizes:

```text
largest collectible invoices
fastest likely receipts
customers with strong payment history
customers with broken promise risk
customers whose payment closes cash gap
customers whose nonpayment threatens reserve
customers with no dispute
```

Collection actions:

```text
send polite reminder
send payment link
send SMS/WhatsApp/email follow-up
request remittance
ask customer's Selene for payment status
alert account manager
offer payment plan if allowed
request partial payment
apply credit hold if policy says
escalate to collections if overdue/risky
```

Example:

```text
Cash gap in 14 days: AUD 40,000
Outstanding AR: AUD 120,000
Selene identifies 10 invoices most likely to pay within 7 days.
Selene sends payment links and account-manager follow-ups.
```

Selene says:

```text
We have a projected cash gap in 14 days. I found AUD 68,000 in invoices likely to be collected quickly. I'll start with those before touching supplier payment timing.
```

## 12. Step 3 - Increase Revenue / Cash Inflow

If collections are not enough, Selene should look for ways to increase incoming cash.

Possible revenue actions:

```text
targeted sales campaign
high-margin product push
slow-stock clearance
repeat-customer offer
abandoned quote follow-up
renewal reminder
subscription upgrade
early payment discount
limited-time bundle
prepayment offer
deposit requirement
service capacity increase
price increase where demand supports it
reduce unnecessary discounting
```

Selene must consider:

```text
time to cash
gross margin
available inventory
available staff/capacity
customer buying history
marketing cost
discount impact
profit floor impact
brand/customer impact
approval required
```

Owner split:

```text
Cashflow identifies need.
Sales/Marketing owns campaign.
Pricing owns price changes.
Inventory confirms stock.
Operations confirms capacity.
PH1.WRITE drafts messages.
BCAST/DELIVERY sends approved campaign.
Finance measures cash/profit result.
```

Example:

```text
Cash gap: AUD 30,000
Product A has available stock, high margin, strong prior conversion.
Selene recommends a targeted campaign expected to generate AUD 42,000 gross margin within 21 days.
```

Selene says:

```text
I can reduce the cash gap by pushing high-margin Product A to repeat customers. It should bring cash faster than delaying critical payments.
```

## 13. Step 4 - Manage Outgoing Payments Carefully

Only after collections and revenue actions are considered should Selene manage outgoing payments.

Outgoing payment classes:

```text
must_pay
critical
important
flexible
delayable
disputed
blocked
```

### Must Pay

```text
payroll
tax/statutory payments
super/pension/CPF
loan payments
lease payments
rent
insurance where lapse is dangerous
legal/court obligations
```

### Critical

```text
key suppliers
utilities
production blockers
customer-delivery blockers
security/safety obligations
regulatory dependencies
```

### Important

```text
normal suppliers
contractors
approved operating expenses
subscriptions required for operations
```

### Flexible

```text
optional purchases
non-critical upgrades
discretionary marketing
non-urgent capex
training/events
nice-to-have subscriptions
```

### Delayable

```text
future-dated purchases
non-essential POs
low-risk supplier payments within terms
early payments without discount
orders not yet committed
```

### Disputed / Blocked

```text
supplier invoice under dispute
goods not received
inspection failed
duplicate invoice suspected
bank details under review
```

Selene options:

```text
pay now
pay on due date
pay earlier for discount
partial payment
split payment batch
delay within terms
request supplier extension
negotiate payment plan
hold disputed invoice
cancel non-critical PO
pause purchase
route emergency override
```

## 14. Payment Priority Ranking

Selene must rank outgoing payments.

Priority factors:

```text
legal/statutory risk
payroll/employee impact
loan/lease default risk
late fee risk
cash reserve impact
supplier criticality
customer delivery impact
production impact
credit rating impact
service cutoff risk
contract breach risk
early payment discount
supplier relationship
cashflow timing
budget status
dispute status
payment terms
bank processing time
public holiday timing
approval time required
```

Payment priority packet:

```text
PaymentPriorityPacket:
  payment_id
  source_owner
  amount
  currency
  due_date
  latest_safe_start_date
  payment_class
  priority_score
  priority_reason
  cash_impact
  reserve_impact
  delay_risk
  supplier_customer_impact
  recommendation
  audit_ref
```

Selene says:

```text
Payroll, tax, and the vehicle lease must be protected this week. I recommend delaying the non-critical office furniture order and requesting a seven-day extension from Supplier B.
```

## 15. Critical Payment Timing

Critical obligations must be scheduled early enough to clear by due date.

Selene must calculate:

```text
due_date
- bank_processing_days
- receiver_clearing_days
- weekend/public_holiday_adjustment
- approval_lead_time
- provider_risk_buffer
= latest_safe_start_date
```

Critical payment examples:

```text
bank loan
car loan
equipment finance
property loan
lease
rent
tax
payroll
super/pension/CPF
insurance premium
critical supplier payment
legal obligation
```

Selene says:

```text
The loan payment is due Friday, but funds need two business days to clear and both CFO and CEO approval are required. I'll start the approval flow today.
```

This connects to Document 3 Addendum and Document 5.

## 16. Minimum Cash Reserve Protection

Selene must enforce minimum cash reserve policies.

Reserve fields:

```text
reserve_policy_id
company_id
legal_entity_id
currency
minimum_reserve_amount
effective_from
effective_to
set_by
approval_ref
emergency_override_policy_ref
audit_ref
```

Reserve check:

```text
payment_id
forecast_cash_after_payment
minimum_reserve_amount
reserve_breach
breach_amount
emergency_override_required
recommended_action
```

If payment breaches reserve:

```text
normal payment blocked
recovery options checked
collections/revenue actions activated
payment delay/partial options checked
emergency override requested only if necessary
```

Selene says:

```text
This payment would bring cash below the AUD 12 million reserve. I've checked alternatives. The safest option is to delay non-critical payments and collect two overdue invoices first. If you still want to proceed, it needs emergency approval.
```

## 17. Net Profit Floor And Cashflow

Cash can be available while profit is unhealthy.

Selene must check both.

Example:

```text
Bank balance is healthy.
But discounts and overtime push forecast net profit below 8%.
```

Cashflow must ask Budget/Finance:

```text
Does this cash action protect or damage net profit?
Does early payment discount improve profit?
Does sales discount improve or reduce margin?
Does delaying payment create penalties?
Does using cash for asset purchase affect profit/cash differently?
```

Selene says:

```text
Cash is available, but this spend would push forecast net profit below the 8% floor. I recommend a cheaper supplier or delaying the purchase until revenue improves.
```

Document 7 owns profit floor.

Cashflow considers timing and liquidity.

## 18. Cashflow Scenario Planning

Selene must compare cash scenarios.

Scenario types:

```text
base case
best case
worst case
collections improved
sales campaign succeeds
customer pays late
supplier delayed
payroll increase
tax payment moved
asset purchased cash
asset leased
loan drawdown
payment plan negotiated
budget cut
```

Scenario packet:

```text
CashflowScenarioPacket:
  scenario_id
  scenario_name
  forecast_period
  assumptions
  opening_cash
  expected_receipts
  expected_payments
  lowest_cash_point
  reserve_breach
  profit_impact
  risk_mode
  recommendation
  audit_ref
```

Selene says:

```text
If the top five customers pay on time, cash stays above reserve. If they pay 14 days late, we breach reserve on the 22nd. I recommend chasing those five today and delaying non-critical purchases.
```

## 19. Cashflow Recovery Plan

When risk appears, Selene creates a recovery plan.

Recovery plan fields:

```text
cashflow_recovery_plan_id
cash_gap_amount
cash_gap_date
risk_mode
root_causes
incoming_cash_actions
revenue_actions
outgoing_payment_actions
approval_actions
responsible_owners
expected_recovery_amount
confidence
status
audit_ref
```

Recovery actions:

```text
collect invoices
send payment links
contact customer Selene
launch campaign
raise price where approved
reduce discounting
delay non-critical payment
negotiate supplier terms
partial pay
release cancelled commitment
pause discretionary spend
request emergency override
```

Recovery statuses:

```text
Draft
AutoActionReady
InProgress
PendingOwnerAction
PendingApproval
Resolved
Escalated
Failed
Closed
```

Selene says:

```text
I've built a cash recovery plan: collect AUD 68,000 from likely payers, delay AUD 22,000 in non-critical purchases, and run a targeted campaign expected to add AUD 35,000 gross margin. This should avoid breaching reserve.
```

## 20. Supplier Payment Negotiation

If a payment cannot safely be made on time and is not critical, Selene may prepare supplier negotiation.

Options:

```text
extension request
partial payment proposal
payment plan
early next-cycle payment
temporary hold due to dispute
settlement discount
batch payment schedule
```

Owner split:

```text
AP owns supplier bill and terms.
Cashflow recommends timing.
PH1.WRITE drafts supplier message.
BCAST/DELIVERY sends through approved channel.
Access/Authority approves negotiation where required.
```

Selene says:

```text
Supplier B is non-critical and still within terms. I can ask for a seven-day extension while we protect payroll and the tax payment.
```

## 21. Customer Cash Acceleration

Selene may accelerate incoming cash.

Options:

```text
payment links
direct debit invitation
early payment discount
payment plan
partial payment
account manager call
Selene-to-Selene payment request
credit hold warning
deposit requirement
cash-before-delivery for risky customers
```

AR/Credit owns customer account truth.

Cashflow ranks priority.

Selene says:

```text
Northside usually pays after SMS reminders. I recommend sending SMS payment links today and asking for partial payment on the oldest invoice.
```

## 22. Sales And Marketing Cash Activation

Cashflow may ask Sales/Marketing to generate cash quickly.

Possible campaigns:

```text
high-margin product campaign
slow-stock clearance
renewal campaign
subscription upgrade
repeat-customer offer
abandoned quote follow-up
bundle offer
limited-time offer
deposit/preorder campaign
```

Selene must consider:

```text
margin
cash speed
inventory availability
production capacity
brand impact
discount cost
customer segment
delivery capacity
marketing spend
approval requirement
```

Selene should not simply discount everything as a panic response.

Example:

```text
Discounting Product B would bring cash quickly but hurts margin. Product A has better margin and enough stock. I recommend pushing Product A first.
```

## 23. Working Capital Intelligence

Cashflow must monitor working capital.

Metrics:

```text
DSO_days_sales_outstanding
DPO_days_payables_outstanding
DIO_days_inventory_outstanding
cash_conversion_cycle
AR_aging
AP_aging
inventory_turnover
gross_margin
cash_runway
reserve_coverage_days
```

Selene should explain:

```text
Cash is tightening because customers are paying 12 days slower while supplier terms have stayed the same. Improving collections will help more than delaying suppliers.
```

## 24. Cashflow And Purchase Orders

Purchase orders affect future cash before invoices arrive.

Cashflow must see:

```text
approved PO
expected delivery date
expected invoice date
expected payment terms
expected payment date
cash impact
budget commitment
```

If PO will create cash risk:

```text
Selene flags before commitment.
```

Selene says:

```text
This purchase fits the budget, but it creates a cash dip in three weeks when payroll and tax are also due. I recommend delaying delivery or splitting payment.
```

## 25. Cashflow And Inventory

Inventory can trap cash.

Cashflow must consider:

```text
stock purchases
slow-moving inventory
high-margin stock
stockout risk
clearance opportunity
reorder cycles
supplier payment terms
seasonal demand
```

Example:

```text
Selene sees cash tied up in slow-moving stock and recommends clearance campaign before delaying critical supplier payments.
```

Selene says:

```text
We have AUD 90,000 tied in slow-moving stock. A clearance campaign could improve cash without cutting essential spend.
```

## 26. Cashflow And Payroll

Payroll is critical.

Cashflow must know:

```text
payrun dates
gross/net pay
tax withholding
super/pension/CPF due dates
bonuses
commissions
overtime forecast
new hires
terminations/final pay
salary advances/deductions
```

Selene must warn early:

```text
Payroll is due in five days. Current expected cash remains safe only if two customer payments arrive as promised.
```

Selene should activate AR collection, not gamble with payroll.

## 27. Cashflow And Debt / Loans / Leases

Debt/Loans/Leases future document owns obligations.

Cashflow must forecast:

```text
principal payments
interest payments
lease payments
balloon payments
loan covenants
maturity dates
refinancing needs
late fee risk
security/collateral risk
```

Critical rule:

```text
Loan and lease payments must be scheduled to clear by due date.
```

If cash risk:

```text
escalate early
check refinance/payment extension options
protect covenant compliance
```

## 28. Cashflow And Insurance

Insurance future engine owns policies and premiums.

Cashflow must forecast:

```text
premium due dates
renewal premiums
monthly installments
annual premiums
claims excess
policy lapse risk
```

Insurance payments may be critical if lapse would expose company.

Selene says:

```text
The public liability policy premium is due next week. If unpaid, cover may lapse. I recommend treating it as critical.
```

## 29. Cashflow And Credit Cards

Document 6 owns card spend.

Cashflow must forecast:

```text
credit card statement payment
employee card spend trend
executive card spend
personal charge recoveries
large card settlements
merchant/card fees
```

If card statement threatens cash:

```text
The company card statement is higher than forecast because travel spend increased. I've identified AUD 3,400 in personal charges pending recovery.
```

## 30. Cashflow And Dividends / Shareholder Distributions

Future Document 14 owns dividends/equity.

Cashflow must check:

```text
available retained earnings
cash reserve
profit floor
tax obligations
loan covenants
board/shareholder approval
distribution timing
```

Selene must not pay dividends if it creates liquidity risk or violates governance.

## 31. Cashflow Reports And Presentations Boundary

Full reporting belongs to future:

```text
Document 17 - Financial Reporting + Analytics + Board Packs + Device Presentation
```

Cashflow provides report-ready data:

```text
cash balance
available cash
cash forecast
cash gap
reserve risk
critical payments
expected receipts
collections plan
sales cash plan
payment priority list
risk mode
scenario comparison
recovery plan
```

Presentation owner split:

```text
Cashflow = cash truth and forecast packet
Banking = bank proof
AR/AP/Payroll/Tax = source truth
PH1.D = explanation proposal
PH1.WRITE = final wording
Desktop/iPhone = render
Access = visibility/export/share
BCAST/DELIVERY = approved distribution
```

## 32. Autonomous Human Conversation Style

Selene should be calm, direct, and solution-first.

Bad:

```text
Cash short. Approval needed.
```

Better:

```text
Cash will be tight in 14 days. I found three fixes: collect two overdue invoices, delay one non-critical supplier bill within terms, and pause a discretionary purchase. If we do those, reserve stays safe.
```

Selene should explain:

```text
what is happening
when it happens
why it matters
what she checked
what she recommends
what needs approval
what she can do now
```

This is human-like without becoming noisy or theatrical.

## 33. Access And Authority

Protected cashflow actions:

```text
view cashflow forecast
view bank balances
set cash reserve
override cash reserve
approve critical payment priority
delay supplier payment
offer customer payment plan
approve early payment discount
approve sales discount campaign
approve supplier extension request
approve payment deferral
approve cashflow recovery plan
send board cash report
export cashflow report
```

Authority depends on:

```text
role
legal entity
country
amount
payment type
cash risk mode
supplier/customer risk
board policy
reserve impact
profit impact
time urgency
```

Step-up may be required for:

```text
cash reserve override
payment deferral above threshold
critical payment priority change
board cash report approval
large supplier negotiation
emergency cash action
```

## 34. PH1.D / GPT-5.5 Role

Allowed:

```text
summarize cashflow issues
draft customer payment reminders
draft supplier extension requests
explain cash risk modes
draft board cash summary
compare options in plain English
suggest narrative for recovery plan
translate cashflow messages
```

Forbidden:

```text
approve payment delay
approve reserve breach
execute payment
mark receipt collected
invent customer promises
invent sales forecast
change supplier terms
change customer terms
send collection messages directly
```

## 35. PH1.WRITE Wording

PH1.WRITE owns final wording.

### Cash Risk Alert

```text
Cash is forecast to fall below reserve in 18 days unless two major customer payments arrive or we delay non-critical purchases.
```

### Recovery Plan

```text
I found a recovery path that avoids reserve breach: collect AUD 68,000 from likely payers, delay AUD 22,000 in non-critical spend, and run a high-margin stock campaign.
```

### Supplier Extension

```text
Could we extend payment for invoice INV-884 by seven days? We value the relationship and expect to clear it next week.
```

### Board Summary

```text
The company remains solvent under the base case, but the downside scenario breaches reserve on 22 June if major customers pay late. Selene recommends immediate collections and delaying discretionary capex.
```

## 36. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
cashflow_forecast_id
scenario_id
risk_mode
source_evidence_refs
bank_balance_refs
AR_refs
AP_refs
payroll_refs
tax_refs
budget_refs
recovery_plan_ref
recommendation_refs
approval_refs
step_up_refs
delivery_refs
timestamp
company_id
legal_entity_id
country
currency
reason_code
```

No hidden cashflow changes.

No unlogged reserve override.

No silent payment reprioritization.

## 37. Failure Branches

### Bank Feed Stale

```text
Selene flags low confidence.
Refresh required before major payment decision.
```

### Customer Does Not Pay As Promised

```text
Forecast updates.
Risk mode may increase.
Collections escalate.
Payment priority recalculated.
```

### Sales Campaign Underperforms

```text
Revenue action confidence reduced.
Cash recovery plan revised.
```

### Supplier Refuses Extension

```text
Payment priority recalculated.
Alternative deferral/cash options checked.
```

### Approval Timeout

```text
Reminder/escalation created.
Critical payment timing rechecked.
```

### Reserve Breach Unavoidable

```text
Emergency approval required.
Board/executive escalation if policy says.
```

### Forecast Data Missing

```text
Selene identifies missing owner:
AR, AP, Payroll, Banking, Tax, Sales, or Budget.
Forecast marked low-confidence.
```

## 38. Required Logical Packets

Future logical packets:

```text
CashflowForecastPacket
CashflowInputEvidencePacket
CashRiskModePacket
CashflowRiskPacket
CashflowRecoveryPlanPacket
PaymentPriorityPacket
CriticalPaymentTimingPacket
ReserveCheckPacket
CashReserveBreachPacket
CustomerCollectionPriorityPacket
RevenueAccelerationPacket
SalesCashActionPacket
SupplierPaymentNegotiationPacket
CashflowScenarioPacket
WorkingCapitalMetricsPacket
CashflowForecastConfidencePacket
CashflowExceptionPacket
CashflowAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 39. Example - 14-Day Cash Gap

```text
Forecast:
cash gap in 14 days: AUD 40,000

Source:
payroll due
tax due
supplier bills due
customers paying slowly

Selene actions:
1. identifies AUD 120,000 open receivables
2. selects AUD 68,000 likely collectible quickly
3. sends payment links and account-manager follow-ups
4. recommends delaying AUD 22,000 non-critical spend within terms
5. protects payroll and tax
```

Selene says:

```text
We have a projected AUD 40,000 gap in 14 days. I can close it without touching payroll or tax by collecting the fastest receivables and delaying non-critical purchases within terms.
```

## 40. Example - Cash Reserve Breach

```text
Payment: AUD 450,000
Reserve: AUD 12,000,000
Forecast after payment: AUD 11,840,000
Breach: AUD 160,000
```

Selene says:

```text
This payment would breach the reserve by AUD 160,000. I found two safer options: delay a non-critical supplier payment or collect two invoices due this week. If you still want to proceed, I need emergency approval.
```

## 41. Example - Increase Sales Instead Of Delay Bills

```text
Cash risk:
AUD 30,000 gap in 21 days.

Selene finds:
slow stock with high margin
repeat customers likely to buy
campaign cost low
cash conversion fast
```

Selene says:

```text
Instead of delaying supplier payments, I recommend a targeted high-margin stock campaign. It is likely to cover the cash gap faster and protect supplier relationships.
```

## 42. Example - Critical Loan Payment

```text
Loan payment due Friday.
Funds need two business days to clear.
Approval requires CFO + CEO.
Today is Tuesday.
```

Selene says:

```text
The loan payment must start today to clear by Friday. I've prepared the approval request for CFO and CEO now.
```

If not approved:

```text
The loan payment is now at risk. Approval is still missing, and the bank cutoff is approaching. I'm escalating this as a critical payment issue.
```

## 43. Example - Bank Feed Stale

User asks:

```text
Can we pay this supplier today?
```

Bank feed:

```text
last updated 24 hours ago
large pending payments likely
```

Selene says:

```text
I need to refresh the bank feed first. The last balance is 24 hours old, and I shouldn't use stale cash data for a payment decision.
```

## 44. What Must Not Happen

```text
no cashflow decision from stale bank data without warning
no delaying payments before collections/revenue options are checked
no critical payment delayed casually
no payroll/tax/loan/lease/rent treated as flexible spend
no reserve breach without emergency approval
no cash forecast presented as actual
no customer promise treated as confirmed receipt
no sales forecast treated as guaranteed cash
no ambiguous bank receipt treated as collected cash
no supplier extension assumed without confirmation
no PH1.D invented cashflow truth
no payment priority change without audit
no board/executive escalation without solution attempt unless policy requires immediate escalation
no customer reminders sent outside AR/BCAST/WRITE protocol
no supplier negotiation sent outside AP/BCAST/WRITE protocol
no implementation from this document alone
```

## 45. Future Simulation Targets

```text
SIM_CASH_001_daily_30_60_90_day_cash_forecast
SIM_CASH_002_cash_gap_detected_14_days_out
SIM_CASH_003_collect_receivables_before_delaying_payments
SIM_CASH_004_sales_campaign_revenue_acceleration
SIM_CASH_005_payment_priority_ranking
SIM_CASH_006_cash_reserve_breach_recovery_plan
SIM_CASH_007_critical_loan_payment_latest_safe_start
SIM_CASH_008_payroll_protection_cashflow_mode
SIM_CASH_009_bank_feed_stale_blocks_payment_decision
SIM_CASH_010_customer_promise_fails_forecast_updates
SIM_CASH_011_supplier_extension_refused_reprioritize
SIM_CASH_012_red_cash_mode_spend_freeze_recommendation
SIM_CASH_013_black_cash_mode_board_escalation
SIM_CASH_014_inventory_clearance_cash_recovery
SIM_CASH_015_scenario_planning_late_customers
SIM_CASH_016_working_capital_dso_dpo_dio_analysis
```

## 46. Final Architecture Sentence

Selene Cashflow Forecasting + Payment Priority Intelligence is the real-time cash brain: it consumes bank truth, receivables, payables, payroll, tax, debt, leases, cards, inventory, sales, budgets, and commitments; forecasts cash across short and long horizons; detects risk before crisis; first collects what is owed, second accelerates revenue, third manages outgoing payments by priority; protects payroll, tax, loans, leases, rent, insurance, cash reserve, and profit floor; proposes recovery plans and scenarios; and escalates only when every safe autonomous solution has been attempted or policy requires human governance.
