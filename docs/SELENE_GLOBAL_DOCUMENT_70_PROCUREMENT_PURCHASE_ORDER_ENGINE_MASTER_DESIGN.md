# Global Document 70 — Selene Procurement + Purchase Order Engine v2

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
70

ENGINE:
PH1.PROCUREMENT / PH1.PURCHASE_ORDER / PH1.BUYING_INTELLIGENCE

FULL NAME:
Selene Procurement, Conversational Purchase Order Creation, Usual Supplier Memory, Quantity Optimization, Cashflow Protection, Approval Routing, Broadcast/Delivery Approval, Reorder Reminders, Central/Local Purchasing, Receiving Readiness, PO Lifecycle, and Supplier Buying Intelligence Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Procurement + Purchase Order Engine owns the company-side process of buying goods and services from suppliers.

This is different from customer orders.

```text
Customer buys from company = Sales Order / Customer Order / Order Management.
Company buys from supplier = Purchase Order / Procurement.
```

This engine must make company buying simple, safe, cashflow-aware, approval-aware, and receiving-ready.

A user should be able to say:

```text
“Selene, order the usual toilet paper.”
“Who do we normally buy this from?”
“Order more packaging.”
“Buy enough cleaning supplies for the next month.”
“Find the best supplier for printer paper.”
“Create the PO and send it for approval.”
```

Selene must then do the hard work:

```text
understand what is being requested
find usual supplier
find usual quantity
check current stock
check expected usage
recommend quantity
check agreed supplier pricing
check budget
check cashflow
check authority
route approval quietly if needed
draft PO
issue PO after approval
notify Receiving in advance
track supplier confirmation
track delivery
escalate delays
handoff to Receiving and AP
```

Procurement must not be “buy shit.”

It must be:

```text
buy the right amount
from the right supplier
at the right price
with the right approval
without damaging cashflow
with Receiving ready
and with AP able to match it later
```

That is Selene-level procurement. The toilet paper deserves governance. Apparently.

---

## 2. Core Procurement Law

```text
Selene must not create or issue a PO just because a user asked.

Selene must check:
- whether the purchase is needed
- whether the quantity is sensible
- whether the company can afford it
- whether the supplier is correct
- whether pricing is correct
- whether approvals are required
- whether access/authority must be routed
- whether Receiving can accept the goods
- whether storage/cold-chain/shelf space exists
- whether AP can later match the invoice
```

Procurement must protect the company from:

```text
over-ordering
cashflow damage
wrong supplier
wrong price
unapproved spending
duplicate orders
ordering stock already in storage
supplier delays
goods arriving with no receiver ready
goods arriving with no freezer/shelf/storage space
POs becoming payable without receiving proof
```

Tiny business wisdom: the cheapest disaster is the one Selene prevents before the truck arrives.

---

## 3. Engine Ownership Boundary

### 3.1 Procurement owns

```text
purchase request interpretation
supplier buying recommendation
usual supplier memory usage
usual order memory usage
quantity recommendation
PO drafting
PO approval need detection
PO authority routing request
supplier quote comparison
price movement warning
cashflow/budget procurement gate
central vs local purchasing classification
group ordering structure
PO issuing after authority
PO lifecycle tracking
supplier confirmation tracking
PO delay escalation
PO amendment / cancellation / reorder logic
Receiving handoff
AP matching handoff
procurement audit evidence
```

### 3.2 Procurement references but does not own

```text
inventory stock truth
warehouse storage truth
cashflow forecast truth
budget ledger truth
supplier master truth
supplier bank trust truth
payment execution
goods receiving execution
invoice/AP payable creation
supplier payment
tax treatment
final accounting posting
user access master rules
Broadcast / Delivery infrastructure
Reminder infrastructure
```

### 3.3 Correct owner split

```text
PH1.PROCUREMENT = purchase order drafting, approval routing need, supplier/order decision.
PH1.INVENTORY = stock truth, reorder points, usage, JIT signals.
PH1.CASHFLOW = current/future cashflow effect.
PH1.BUDGET = budget/cost centre/project availability.
PH1.SUPPLIER = supplier identity, supplier score, supplier terms.
PH1.SUPPLIER_BANK_TRUST = supplier bank safety.
PH1.ACCESS / AUTHORITY = who may approve / issue / request access.
PH1.BCAST / DELIVERY = sends approval/access/PO/receiving notifications.
PH1.REM = reminders and escalation timing.
PH1.RECEIVING = physical receipt, inspection, storage confirmation.
PH1.AP = invoice/PO/receiving matching.
PH1.ACCOUNTING = ledger.
PH1.AUDIT = proof.
```

Procurement is the buying brain.

It is not the warehouse, not the bank, not the approver, and not the accountant. We’re keeping the goblins in their boxes.

### 3.4 Receiver assignment ownership clarification

Procurement does not own final receiver assignment.

Procurement creates the receiving requirement and handoff.

Receiving Engine owns receiver assignment and receipt execution.

Task / Human Workload / Rosters check availability.

Access / Authority checks receiver permission.

Broadcast / Delivery notifies.

Reminder Engine reminds.

Audit records proof.

### 3.5 Supplier qualification ownership clarification

Procurement does not own supplier discovery, qualification, or supplier rating.

Supplier Intelligence Engine owns supplier finding, qualification, scoring, and rating.

Supplier Bank Trust Engine owns supplier bank safety.

Procurement consumes supplier recommendations and uses them for PO drafting.

---

## 4. Conversation-to-Action Guardrail

Procurement must follow the global Conversation-to-Action Guardrail.

Selene must not use fixed phrase matching.

Correct flow:

```text
1. GPT-5.5 understands the request.
2. PH1.X resolves live context.
3. PH1.M / procurement memory resolves usual supplier, usual item, usual quantity, prior instruction.
4. Procurement verifies supplier, stock, quantity, price, budget, cashflow, authority, receiving readiness.
5. Protected actions require authority.
6. Broadcast / Delivery quietly routes approval/access requests where needed.
7. PO is issued only after rules are satisfied.
8. Everything important is audited.
```

GPT-5.5 may:

```text
understand messy buying requests
summarize supplier comparison
explain price changes
draft PO summaries
explain approval reasons
suggest negotiation wording
```

GPT-5.5 must not:

```text
issue PO without authority
invent supplier terms
invent stock levels
invent cashflow status
invent approval
invent receiving readiness
bypass access rules
override budget/cashflow/authority gates
```

Selene can sound human.

The PO still needs proof.

---

## 5. Conversational PO Creation

User can say:

```text
“Order the usual toilet paper.”
“Order more screws.”
“Buy the usual packaging.”
“Who do we normally buy this from?”
“Place the normal order.”
“Get enough stock for next month.”
```

Selene should respond with useful buying intelligence, not a blank form.

Example:

```text
“You usually buy toilet paper from Supplier A.
Usual quantity is 60 cases.
Last order was 5 weeks ago.
Current stock appears low.
Supplier delivers in 3 days.
I recommend 30 cases because cashflow is tight and delivery is reliable.
I can draft the PO now.”
```

If user confirms, Selene drafts the PO and continues through authority, approval, issuing, receiving, and tracking flows.

---

## 6. Usual Supplier Memory

Selene should know:

```text
usual supplier
usual price
usual quantity
usual delivery time
last order date
last invoice price
supplier reliability
supplier issues
supplier delivery performance
supplier return/credit history
agreed supplier pricing
contract pricing
group pricing
branch pricing
```

Example:

```text
“You usually buy this from Supplier A.
Last order was 60 cases at $18.40 each.
Current agreed price is $18.90.”
```

If there is no usual supplier:

```text
Selene compares available suppliers and recommends one.
```

---

## 7. Usual Order Memory

Selene should remember recurring and routine orders:

```text
toilet paper monthly
packaging every Friday
cleaning supplies first Monday
printer paper every 6 weeks
screws every 2 weeks
raw ingredients by production cycle
```

Selene should suggest when routine order timing appears due.

Example:

```text
“Toilet paper is usually reordered around now.
Current stock suggests 12 days remaining.
Should I prepare the usual order?”
```

---

## 8. Reorder Reminders

Procurement must generate reorder reminders when history and stock suggest it is time to buy.

Signals:

```text
order history
usage rate
current stock
minimum stock
safety stock
supplier lead time
seasonal demand
sales activity
production schedule
receiver feedback
supplier delivery reliability
```

Engines involved:

```text
Inventory confirms stock.
Memory Pattern detects buying habit.
Procurement prepares reorder recommendation.
Reminder Engine schedules reminder.
Broadcast / Delivery sends it to the responsible person.
```

Example reminder:

```text
“Toilet paper usually gets reordered every 4 weeks.
You are approaching the usual reorder window.
Current stock looks low.
Should I prepare the usual PO?”
```

Procurement does not just wait for someone to remember. Humans forget. That is apparently their hobby.

---

## 9. Quantity Recommendation + Optimization

Selene must not blindly copy the last order.

Selene should recommend quantity using:

```text
current stock
usage history
time between orders
supplier lead time
delivery reliability
minimum stock
safety stock
storage capacity
cashflow
budget
bulk discount
price break
expiry/spoilage risk
cash tied up in inventory
sales forecast
production forecast
seasonality
```

Bad:

```text
“You ordered 400 cases last time. Order 400 again?”
```

Selene-level:

```text
“You ordered 400 cases last time, but it took 18 months to use them.
Supplier delivers weekly.
Cashflow is tighter this month.
I recommend 30 cases.”
```

Selene should balance:

```text
unit price
delivery frequency
stockout risk
storage limits
cash tied up
bulk savings
waste/spoilage risk
```

Cheap bulk buying is not cheap if it traps cash for 18 months and blocks the storeroom like a toilet paper monument.

---

## 10. Cashflow Protection Gate

A user’s spend limit is not enough.

A person may be authorized, but the company may still be cashflow-tight.

Selene must check:

```text
current cash
forecast cash
upcoming payroll
tax obligations
loan repayments
upcoming supplier payments
expected customer receipts
sales slowdown
seasonality
budget remaining
inventory cash tied up
critical spending commitments
```

Example:

```text
“This is within your purchase limit, but cashflow is tight for the next 21 days.
I recommend ordering 20 cases instead of 60.”
```

Procurement must prevent the bleed before it starts.

If the company is tight on cash, Selene should recommend:

```text
reduced quantity
split delivery
defer optional items
use supplier credit terms
choose smaller order
choose cheaper supplier
wait for expected receipts
escalate for management review
```

---

## 11. Essential vs Deferrable Classification

Selene must classify purchase need:

```text
Essential
Important
Routine
Optional
Deferrable
Risky
Cashflow-sensitive
```

Examples:

```text
toilet paper = essential
raw materials for confirmed customer order = important / essential
luxury office chairs = deferrable
extra promotional stock = cashflow-sensitive
equipment upgrade = may need management review
```

If cashflow is tight, Selene should reduce essential order quantity rather than block it.

Example:

```text
“Cashflow is tight, but toilet paper is essential.
I recommend ordering 20 cases to cover four weeks instead of the usual 60.”
```

Some spending is not optional. Apparently business civilisation still requires bathroom supplies.

---

## 12. Suspicious Reorder Detection

Selene must challenge orders that do not make sense.

Example:

```text
Someone tries to order toilet paper again.
Selene expects 6,000 boxes still in storage.
```

Selene should say internally or to the responsible user as appropriate:

```text
“This looks unusual. Based on the last order and expected usage, you should still have enough stock.
Please check storage before I proceed.”
```

Possible explanations:

```text
stock stolen
stock damaged
wrong location count
inventory not updated
unexpected usage increase
previous receipt was wrong
duplicate ordering
ordering person missed stored inventory
```

Selene should route to:

```text
Inventory check
Warehouse check
Receiver check
Audit check
Shrinkage/loss review if needed
```

This is not just procurement. It is responsible buying. Imagine that, a company not purchasing while blindfolded.

---

## 13. Budget Check

Selene must check budget before PO issue.

Checks:

```text
cost centre budget
department budget
project budget
branch budget
monthly cap
category cap
remaining allocation
committed spend
pending POs
future obligations
```

Example:

```text
“This fits the office supplies budget.”
```

Or:

```text
“This exceeds the monthly budget by $420.”
```

Budget issues should route for approval or recommendation, not silently proceed.

---

## 14. Person Purchase Limits

Each user should have purchasing limits:

```text
single purchase limit
monthly limit
category limit
supplier limit
department limit
project limit
branch/location limit
emergency limit
```

Example:

```text
“Tom can approve office supplies up to $1,000.”
```

Limit checks must use Master Access and Per-User Access rules.

Procurement does not own identity authority.

It asks Access / Authority.

---

## 15. Access Request Auto-Routing

If a user lacks authority, Selene should not hard-deny immediately.

Follow Master Access / Per-User Access rules.

Correct rule:

```text
If user lacks authority, Selene quietly routes the access/approval request to the correct authority where policy allows.
If authority approves, Selene proceeds.
If authority rejects, Selene informs the requester politely.
```

Selene does not need to say upfront:

```text
“You do not currently have authority.”
```

unless policy requires disclosure or action is blocked without routing.

Quiet flow:

```text
User asks to issue PO.
Selene detects authority missing.
Selene routes request to correct approver/authority.
If approved, Selene continues.
If rejected, Selene informs user.
```

Authority can decide:

```text
approve one-time action
grant permanent access
grant temporary access
grant amount-limited access
grant category-limited access
reject request
request more information
```

Procurement must follow Master Access / Per-User Access rules.

No such thing as “you can’t” until the right authority has actually rejected it. Then yes, Selene can gently tell the idiot no. Professionally. Tragically.

---

## 16. Clerk / Assistant Drafting

A clerk or assistant may draft a PO but not issue it.

Example:

```text
Assistant drafts order.
Selene checks quantity, supplier, price, cashflow, budget, and receiving readiness.
Selene routes to manager for authority.
Manager approves.
Selene issues PO.
```

This matches the real world:

```text
Assistant prepares.
Boss approves.
Selene handles the boring middle.
```

Finally, management gets to approve things before they are broken.

---

## 17. Multi-Approver Routing

Some purchases require more than one approval.

Selene must support:

```text
single approver
dual approval
sequential approval
parallel approval
manager + finance approval
department head + owner approval
board approval
substitute approver
delegated approver
conflict-aware approver exclusion
```

Example:

```text
“This PO needs Sarah and Michael approval.”
```

If one approver is unavailable:

```text
Selene routes to delegated substitute if allowed.
```

If there is a conflict of interest:

```text
Selene excludes conflicted approver and routes to alternate authority.
```

---

## 18. Approval Routing Through Broadcast + Delivery

Approval routing must use Broadcast + Delivery.

Procurement owns approval need.

Authority owns approval rule.

Broadcast / Delivery sends the request.

Reminder handles follow-up.

Audit proves it happened.

Selene must:

```text
send approval request
show PO summary
allow approve/reject/comment
remind if no response
escalate if overdue
route to substitute approver if needed
record proof
```

Approval request includes:

```text
PO summary
supplier
items
quantity
price
cashflow effect
budget effect
why approval is required
risk flags
receiving readiness
recommended action
approve/reject/comment options
```

Approval is not a status hidden in a screen. It is a message to an actual human, which means Broadcast and Delivery need to do their jobs instead of lurking like decorative modules.

---

## 19. Approval Reminders and Escalation

Selene should remind approvers.

Example:

```text
“Sarah, this PO is still waiting for approval.
Delivery is needed by Friday.”
```

If no action:

```text
remind again
escalate to next authority
route to backup approver
warn requester
pause draft
expire draft
cancel if stale
```

Escalation uses:

```text
Reminder Engine
Broadcast / Delivery
Authority Engine
Audit
```

---

## 20. Conflict-of-Interest Check

Selene should flag:

```text
employee buying from related supplier
approver owns supplier
requester and approver are same person where not allowed
supplier bank recently changed
new supplier added
urgent purchase with weak explanation
price unusually high
same supplier repeatedly selected despite better alternatives
```

Conflict routes to authority, audit, or compliance.

---

## 21. Supplier Comparison

Selene should compare suppliers by:

```text
price
agreed price
delivery speed
delivery reliability
quality
supplier score
return rate
credit terms
discounts
free delivery threshold
past issues
availability
location
cashflow impact
```

Example:

```text
“Supplier B is cheaper, but Supplier A delivers faster and has fewer receiving issues.”
```

Selene should recommend, not blindly choose.

---

## 22. Agreed Supplier Pricing

If a supplier has agreed pricing, Selene must enforce or alert.

Checks:

```text
contract price
group price
branch price
volume price
promo price
last invoice price
quote price
supplier increase
free delivery threshold
bulk discount
```

If quote differs from agreed price:

```text
“This quote is above the agreed price.
I recommend querying the supplier before issuing the PO.”
```

---

## 23. Price Movement Detection

Selene should warn when price changes.

Checks:

```text
last price
current price
agreed price
percentage increase
supplier explanation
alternate suppliers
bulk discount impact
```

Example:

```text
“Price increased 14% since last order.
I recommend asking Supplier A to confirm or match Supplier B.”
```

---

## 24. Supplier Negotiation Recommendation

Selene should suggest negotiation when useful:

```text
price increased
large order
supplier late recently
competitor cheaper
cashflow supports early payment discount
bulk discount available
free delivery threshold nearby
```

Example:

```text
“Add 3 more boxes to qualify for free delivery, but only if storage and cashflow allow it.”
```

Selene may draft negotiation message but must not send without authority.

---

## 25. Centralized / Group Ordering

Selene must support central purchasing.

Example:

```text
Head Office orders toilet paper for 12 branches.
Supplier ships to multiple locations.
Each branch receives its own allocation.
```

Central order supports:

```text
one central PO
multiple delivery locations
multiple receiving parties
branch allocation
branch-level expected quantities
branch-level receiver assignment
central approval
local receiving proof
group pricing
shared supplier terms
split delivery dates
```

---

## 26. Local Purchasing Authority

Some branches/locations may be authorized to buy locally.

Example:

```text
Branch A can buy cleaning supplies locally up to $500.
Head Office handles bulk stock above $500.
```

Selene must know:

```text
central purchasing rules
local purchasing rules
branch authority
local supplier list
group supplier pricing
emergency purchase rules
approval difference between local and central
```

Selene should decide whether the request belongs to:

```text
local procurement
central procurement
emergency local procurement
group purchasing
```

---

## 27. Delivery Address Intelligence

PO must know where goods go.

Possible delivery locations:

```text
warehouse
store
branch
project site
office
restaurant
salon
customer site
temporary site
central depot
multiple locations
```

Selene should ask only if unclear.

If group order:

```text
Selene splits expected receiving by location.
```

---

## 28. Storage Capacity Check Before Ordering

Before issuing PO, Selene should check whether the receiving/storage location can handle the order.

Checks:

```text
freezer capacity
cold room space
shelf space
warehouse capacity
hazardous goods storage
secure storage
expiry constraints
dock access
forklift/staff availability
```

If not enough space:

```text
“Storage capacity is insufficient for this quantity.
I recommend reducing quantity or scheduling split delivery.”
```

This prevents ordering frozen goods when no freezer space exists. A lesson apparently learned the expensive way.

---

## 29. Split Delivery / Staged Delivery

For big orders, Selene should suggest split delivery.

Example:

```text
Instead of 400 cases at once:
100 cases monthly
or 50 cases every two weeks
```

Helps with:

```text
cashflow
storage
spoilage
warehouse space
supplier delivery schedule
```

---

## 30. Receiving Handoff

Once PO is approved/issued, Receiving must know what is coming.

Procurement sends Receiving:

```text
PO number
supplier
expected items
quantity
expected date/time
delivery location
inspection requirements
batch/expiry requirements
serial number requirements
photos required
cold-chain/freezer requirements
shelf/storage requirements
receiver assigned
who to notify
```

Procurement creates buying commitment.

Receiving owns physical acceptance.

AP later matches invoice against PO + receiving proof.

---

## 31. Receiver Assignment

Selene should assign the right receiver based on:

```text
delivery location
warehouse/store
product category
risk level
inspection skill
roster/availability
authority
cold-chain skill
high-value handling
```

Examples:

```text
Frozen goods → cold-chain trained receiver
Meat → food receiver
Electronics → serial-number receiver
Office supplies → general warehouse receiver
High-value asset → manager + photo proof
```

---

## 32. Receiver Advance Warning

Receiver must know early.

Selene notifies:

```text
when PO is issued
when supplier confirms
one day before delivery
same day reminder
delay warning
arrival warning if courier/tracking available
```

Receiver should prepare:

```text
freezer space
shelf space
dock access
forklift/staff
inspection tools
storage bins
cold-chain readiness
security for high-value items
```

Example:

```text
“Frozen goods arriving Friday 9am.
Required freezer space: 2 pallets.
Receiver: Tom.
Please confirm freezer capacity.”
```

Surprise is for birthdays, not frozen deliveries.

---

## 33. PO-to-Receiving Tracking

Procurement must track whether issued POs are received.

If not received by expected date:

```text
warn receiver
warn requester
check supplier confirmation
check courier/tracking
escalate if critical
recommend cancel/reorder
```

Selene must not let missing goods quietly sit in “expected delivery” status forever like a ghost shipment.

---

## 34. Supplier Confirmation and Delay Tracking

After PO is issued, Selene tracks:

```text
supplier acceptance
expected delivery date
delayed shipment
lost in transit
damaged in transit
partial shipment
supplier cancellation
backorder
```

If delay occurs:

```text
notify requester
notify receiver
notify management if critical
suggest alternate supplier
cancel/reorder if needed
```

Example:

```text
“Toilet paper delivery is delayed.
Current stock covers 3 days.
I recommend urgent reorder from backup supplier.”
```

Because civilisation has minimum bathroom requirements.

---

## 35. Cancel and Reorder if Goods Not Received

If goods do not arrive:

```text
check supplier status
check courier status
warn responsible person
contact supplier
cancel PO if necessary
create replacement PO
route urgent approval if needed
notify Receiving
notify AP if invoice exists
```

---

## 36. PO Amendment Rules

If something changes:

```text
price
quantity
supplier
delivery date
item
substitution
delivery location
```

Selene must decide:

```text
minor amendment allowed
new approval required
new PO version required
cancel and recreate PO
notify receiver
notify AP
```

---

## 37. PO Cancellation Rules

Selene must support:

```text
cancel draft
cancel before approval
cancel after approval
cancel after issue before supplier accepts
cancel after supplier accepts
partial cancellation
cancel and reorder
supplier cancellation
```

---

## 38. PO Lifecycle

PO remains a PO until goods/services are received and invoice matching begins.

States:

```text
Draft
NeedsInfo
NeedsApproval
Approved
Issued
SupplierConfirmed
PartiallyReceived
FullyReceived
Closed
Cancelled
Disputed
Expired
```

PO does not become payable just because someone created it.

Flow:

```text
PO issued
goods/services received
supplier invoice arrives
AP matches PO + receiving + invoice
approved payable created
supplier payment later
```

Do not pay for ghosts. Simple. Apparently revolutionary.

---

## 39. Supplier Bank Safety Handoff

Before issuing or paying high-risk purchases, Selene may check supplier trust.

Risk triggers:

```text
new supplier
supplier bank changed
supplier details changed
large value
urgent request
unusual supplier
manual override
```

Procurement hands off to:

```text
Supplier Intelligence
Supplier Bank Trust
AP
Supplier Payment
```

Procurement does not pay.

Procurement ensures risk is visible early.

---

## 40. Smart PO Draft

Selene should draft PO with:

```text
supplier
items
quantity
unit price
tax
currency
delivery location
expected delivery date
budget/cost centre/project
payment terms
supplier terms
agreed pricing reference
approval route
receiver assignment
inspection rules
cashflow note
quantity recommendation reason
audit evidence
```

---

## 41. Executive PO Summary

Every PO should have a plain-English summary.

Example:

```text
“This PO buys 30 cases of toilet paper from Supplier A for $567.
You usually order 60 cases, but cashflow is tight and supplier delivers weekly.
This covers 4 weeks.
It is within your limit.
No second approval required.
Receiving has been notified for Friday delivery.”
```

Approval summary:

```text
“This exceeds Tom’s limit and requires Sarah + Michael approval.
Cashflow is green.
Storage capacity confirmed.
Supplier price matches agreed pricing.”
```

---

## 42. Approval Reason Explanation

Approvers should know why they are approving.

Approval summary includes:

```text
why purchase is needed
why quantity is recommended
budget impact
cashflow impact
supplier history
price comparison
risk flags
delivery urgency
storage readiness
receiver readiness
```

No mystery approvals. Mystery approvals are how companies buy 400 chairs no one wanted.

---

## 43. Procurement Conversation Memory

Selene should remember procurement instructions:

```text
avoid Supplier B unless Supplier A is out
always route IT equipment to Michael
order local if urgent
never bulk order perishables beyond 2 weeks
prefer Supplier C for screws unless price rises over 10%
```

Memory helps.

Deterministic checks still verify.

Selene remembers preferences, not excuses.

---

## 44. State Machines

### 44.1 Purchase Request State

```text
Requested
IntentResolved
NeedsInfo
SupplierSuggested
QuantitySuggested
CashflowChecking
BudgetChecking
AuthorityChecking
DraftReady
Closed
```

### 44.2 PO State

```text
Draft
NeedsInfo
NeedsApproval
ApprovalRouting
Approved
Rejected
Issued
SupplierConfirmed
PartiallyReceived
FullyReceived
Closed
Cancelled
Disputed
Expired
```

### 44.3 Approval Route State

```text
NotRequired
Required
QuietAccessRequestRouted
OneTimeApprovalRequested
PermanentAccessRequested
SequentialApprovalPending
ParallelApprovalPending
Approved
Rejected
Escalated
Expired
Closed
```

### 44.4 Reorder Reminder State

```text
NotDue
DueSoon
ReminderQueued
ReminderSent
UserConfirmed
DraftCreated
Dismissed
Snoozed
Escalated
Closed
```

### 44.5 Quantity Optimization State

```text
NotRequired
UsageAnalyzing
StockChecking
LeadTimeChecking
CashflowChecking
StorageChecking
Recommended
UserOverrode
EscalationRequired
Closed
```

### 44.6 Receiving Handoff State

```text
NotReady
ReceiverAssignmentPending
ReceiverAssigned
StorageCheckPending
StorageConfirmed
ReceiverNotified
ReminderScheduled
DeliveryExpected
Received
IssueRaised
Closed
```

### 44.7 Supplier Delay State

```text
NoDelay
ConfirmationPending
SupplierConfirmed
DelayDetected
ReceiverWarned
RequesterWarned
ManagementWarned
AlternativeSuggested
CancelReorderRecommended
Resolved
Closed
```

---

## 45. Reason Codes

```text
PROCUREMENT_REQUEST_RECEIVED
PROCUREMENT_INTENT_RESOLVED
USUAL_SUPPLIER_FOUND
USUAL_ORDER_FOUND
REORDER_REMINDER_TRIGGERED
QUANTITY_RECOMMENDED
QUANTITY_REDUCED_FOR_CASHFLOW
QUANTITY_REDUCED_FOR_STORAGE
QUANTITY_REDUCED_FOR_USAGE_HISTORY
BULK_ORDER_NOT_RECOMMENDED
SUSPICIOUS_REORDER_DETECTED
STOCK_SHOULD_EXIST_CHECK_REQUIRED
CASHFLOW_GATE_GREEN
CASHFLOW_GATE_WARNING
CASHFLOW_GATE_BLOCKED
BUDGET_GATE_GREEN
BUDGET_GATE_WARNING
USER_LIMIT_CHECKED
AUTHORITY_ROUTE_REQUIRED
QUIET_ACCESS_REQUEST_ROUTED
ONE_TIME_APPROVAL_REQUESTED
PERMANENT_ACCESS_REQUESTED
MULTI_APPROVER_ROUTE_REQUIRED
BROADCAST_APPROVAL_SENT
DELIVERY_APPROVAL_SENT
APPROVAL_REMINDER_SENT
APPROVAL_ESCALATED
CONFLICT_OF_INTEREST_FLAGGED
SUPPLIER_PRICE_MATCHED_AGREED_PRICE
SUPPLIER_PRICE_ABOVE_AGREED_PRICE
SUPPLIER_COMPARISON_COMPLETED
NEGOTIATION_RECOMMENDED
CENTRAL_ORDER_REQUIRED
LOCAL_ORDER_ALLOWED
GROUP_ORDER_CREATED
RECEIVER_ASSIGNED
RECEIVER_ADVANCE_NOTICE_SENT
STORAGE_CAPACITY_WARNING
FREEZER_SPACE_REQUIRED
SPLIT_DELIVERY_RECOMMENDED
PO_DRAFT_CREATED
PO_APPROVED
PO_REJECTED
PO_ISSUED
SUPPLIER_CONFIRMED
SUPPLIER_DELAY_DETECTED
GOODS_NOT_RECEIVED_BY_EXPECTED_DATE
CANCEL_REORDER_RECOMMENDED
PO_AMENDMENT_REQUIRED
PO_CANCELLED
RECEIVING_HANDOFF_CREATED
AP_MATCHING_HANDOFF_READY
PROCUREMENT_AUDIT_CAPTURED
```

---

## 46. Required Simulations

```text
user asks Selene to order usual toilet paper
Selene finds usual supplier and usual quantity
Selene recommends lower quantity due to cashflow
Selene recommends lower quantity due to storage limit
Selene warns last 400 cases took 18 months to use
Selene detects suspicious reorder because stock should exist
Selene routes inventory/warehouse check before PO
assistant drafts PO but cannot issue
Selene quietly routes authority request
authority approves one-time access
authority grants permanent category-limited access
authority rejects access and Selene informs requester
PO requires Sarah and Michael approval
approval sent through Broadcast/Delivery
approver does not respond and reminder sends
approval escalates to substitute approver
supplier price exceeds agreed price
Selene recommends negotiation
supplier B cheaper but delivery slower
cashflow tight but item essential
Selene recommends reduced essential order
central group order for 12 branches
local branch purchase allowed under limit
PO creates receiving handoff
receiver is assigned
freezer space warning blocks quantity
split delivery recommended
receiver receives advance notice
supplier delays delivery
Selene warns receiver/requester/management
goods not received by expected date
Selene recommends cancel and reorder
PO partially received
PO fully received
PO cancelled before supplier accepts
PO amended after price change
AP receives PO/receiving match handoff
```

---

## 47. Integration Map

```text
PH1.PROCUREMENT / PURCHASE_ORDER
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER_BANK_TRUST
↔ PH1.INVENTORY
↔ PH1.WAREHOUSE
↔ PH1.CASHFLOW
↔ PH1.BUDGET / COST_CENTER
↔ PH1.ACCESS / AUTHORITY
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.RECEIVING
↔ PH1.AP / CREDITORS
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.AUDIT
↔ PH1.MP / MEMORY_PATTERN
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
```

---

## 48. Required Logical Packets

```text
ProcurementRequestPacket
ProcurementConversationIntentPacket
UsualSupplierMemoryPacket
UsualOrderMemoryPacket
ReorderReminderPacket
QuantityOptimizationPacket
CashflowProcurementGatePacket
EssentialityClassificationPacket
SuspiciousReorderPacket
SmartPODraftPacket
PersonPurchaseLimitProfilePacket
ApprovalPolicyMatrixPacket
QuietAccessRoutingPacket
MultiApproverRoutePacket
ApprovalBroadcastPacket
ApprovalReminderPacket
ApprovalEscalationPacket
SupplierComparisonPacket
SupplierPriceMovementPacket
SupplierNegotiationRecommendationPacket
CentralGroupOrderPacket
LocalPurchaseAuthorityPacket
StorageCapacityCheckPacket
ReceiverAssignmentPacket
ReceivingAdvanceNoticePacket
SupplierDelayTrackingPacket
CancelReorderRecommendationPacket
POAmendmentPacket
POCancellationPacket
POReceivingHandoffPacket
POAPMatchingHandoffPacket
POExecutiveSummaryPacket
ProcurementAuditEvidencePacket
```

Logical only.

No runtime packet structs.

---

## 49. What Codex Must Not Do

```text
Do not make Procurement own inventory stock truth.
Do not make Procurement own cashflow truth.
Do not make Procurement own budget ledger truth.
Do not make Procurement own supplier bank trust.
Do not make Procurement own receiving execution.
Do not make Procurement own AP payable creation.
Do not make Procurement own supplier payment.
Do not hard-deny access before routing through authority where rules allow quiet routing.
Do not bypass Master Access / Per-User Access rules.
Do not issue PO without required approval.
Do not skip Broadcast / Delivery for approval routing.
Do not skip receiver advance notice.
Do not ignore cashflow warnings.
Do not blindly copy last order quantity.
Do not ignore suspicious reorder signals.
Do not allow PO to become payable without receiving/AP match.
Do not use fixed phrase matching.
Do not let GPT-5.5 invent supplier, price, cashflow, stock, approval, or receiving readiness.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 50. Final Architecture Sentence

Selene Procurement + Purchase Order Engine is the company buying brain that turns human purchasing intent into safe, cashflow-aware, quantity-optimized, supplier-aware, approval-routed, receiving-ready purchase orders; remembers usual suppliers and routine orders; reminds responsible users when reorders are due; prevents unnecessary or suspicious reorders; quietly routes access and approval requests through Master Access / Per-User Access, Broadcast, Delivery, and Reminder engines; checks budgets, cashflow, agreed pricing, supplier risk, storage capacity, and receiver readiness; supports central and local purchasing; tracks supplier confirmation, delays, cancellation, reorder, amendment, receiving handoff, and AP matching readiness; and uses GPT-5.5 for natural procurement conversation while deterministic Selene engines verify stock, supplier, cashflow, authority, receiving, accounting, and audit truth.

Simple version:

```text
User asks Selene to buy something.
Selene checks if it is needed.
Selene checks who we usually buy from.
Selene recommends how much to buy.
Selene checks cashflow.
Selene checks budget.
Selene checks authority.
Selene quietly routes approval/access if needed.
Selene drafts the PO.
Selene issues it after approval.
Selene tells Receiving early.
Selene tracks whether goods arrive.
Selene cancels/reorders if they do not.
Selene hands off to AP only after receiving proof.
Everything important is audited.
```

That is the upgraded Document 70 direction. Procurement is no longer “place order.” Procurement is “make sure buying this thing will not quietly punch the company in the cashflow, storage, receiving, or approval system.” A small but meaningful improvement over buying 400 cases of toilet paper because someone forgot how cupboards work.
