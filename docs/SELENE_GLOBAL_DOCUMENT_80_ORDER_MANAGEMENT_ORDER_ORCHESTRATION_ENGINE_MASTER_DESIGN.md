# Global Document 80 — Selene Order Management + Order Orchestration Engine

```text id="doc80_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
80

ENGINE:
PH1.ORDER / PH1.ORDER_MANAGEMENT / PH1.ORDER_ORCHESTRATION

FULL NAME:
Selene Customer Order Group, Seller Order, Order Line, Source Resolution, Store Preference, Substitution, Payment Requirement, Inventory Reservation, Fulfillment Routing, Delivery Promise, Cancellation, Change, Return, Warranty, Service Milestone, Recurring Order, Exception Recovery, Reconciliation, and Order Truth Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Order Management + Order Orchestration Engine owns the **truth and lifecycle of customer-facing orders**.

It does **not** help the customer shop.
It does **not** create purchase orders.
It does **not** pick products.
It does **not** own payment, inventory, pricing, dispatch, returns, B2B commission, or accounting.

It controls what becomes a real order after the customer decides to buy.

Simple split:

```text id="doc80_simple_split"
E-Commerce / POS = customer wants to buy.
Order Management = what exact orders now exist and what must happen next.
```

If the customer says:

```text id="doc80_customer_request"
“Selene, buy milk, bread, shoes, a handbag, and those nice biscuits.”
```

Selene may tell the customer:

```text id="doc80_customer_summary"
“Done — I ordered the milk, bread, shoes, handbag, and biscuits.”
```

But behind the scenes Document 80 creates and controls:

```text id="doc80_backend_split"
Customer Order Group:
    ├── Supermarket Seller Order
    │   ├── milk
    │   └── biscuits
    ├── Bakery Seller Order
    │   └── bread
    ├── Shoe Store / B2B Provider Seller Order
    │   └── shoes
    └── Fashion Store Seller Order
        └── handbag
```

The customer sees one clean Selene experience.

Each seller/provider receives its own correct order, payment requirement, fulfillment rule, delivery promise, return rule, warranty rule, and audit trail.

That is the point: one human request, many real backend orders, no circus fire.

---

## 2. Core Order Law

```text id="core_order_law"
Order Management must convert customer intent into correct, separated, traceable, seller-specific, line-level order truth.

Order Management must not randomly choose sellers, prices, stock, payment rules, delivery rules, B2B rules, return rules, warranty rules, or accounting rules.

Every order line must have:
- resolved source / store / provider
- product or service truth reference
- price reference
- availability / reservation state
- payment requirement
- fulfillment route
- delivery or service promise
- cancellation / return / warranty rule
- audit trail
```

Order Management must keep the customer experience simple while preserving backend truth.

```text id="doc80_law_simple"
Customer gets simplicity.
Backend gets precision.
```

No order should sit forever in “processing,” the swamp where customer trust goes to drown.

---

## 3. Order vs Purchase Order

Document 80 is **not** Purchase Orders.

```text id="order_vs_po"
Customer buys from company = Customer Order / Seller Order / Sales Order.
Company buys from supplier = Purchase Order / Procurement.
```

Purchase Orders belong to:

```text id="po_owners"
Document 70 — Procurement + Purchase Order
Document 71 — Receiving Proof
Document 72 — Daily Receiving Control
Document 73 — AP / Creditors
Document 74 — Supplier Payment
Document 75 — Supplier Statement Reconciliation
```

Document 80 handles:

```text id="doc80_handles"
E-Commerce orders
POS orders
personal Selene orders
customer shopping groups
seller orders
service bookings
restaurant table orders
B2B customer-facing orders
recurring customer orders
buying list conversions
```

Tiny terminology swamp, yes. Business named everything “order” because apparently clarity was too expensive.

---

## 4. Engine Ownership Boundary

### 4.1 Order Management owns

```text id="doc80_owns"
customer order group creation
seller order creation
order line creation
source / store / provider resolution enforcement
order line lifecycle
order group lifecycle
seller order lifecycle
draft vs confirmed vs binding order state
order promise / ETA tracking
inventory reservation request and reservation expiry tracking
payment requirement handoff
payment status recording
fulfillment routing
dispatch requirement handoff
service / appointment / milestone order routing
recipient / gift / delivery context
order cancellation routing
order change routing
return eligibility handoff
warranty context handoff
B2B order context handoff
split delivery / delivery grouping logic
substitution decision tracking
duplicate order prevention
fraud / risk / high-value review routing
customer communication timeline
exception recovery routing
recurring / scheduled order control
order closure rules
order reconciliation
order audit and versioning
```

### 4.2 Order Management references but does not own

```text id="doc80_not_own"
customer shopping UX
product master truth
product media
inventory stock truth
pricing / discount truth
payment authorization / capture / refund execution
settlement release
B2B provider payout
B2B channel commission
B2B reserves / deposits
dispatch picking / packing / courier booking
return logistics
warranty claim execution
tax treatment
ledger posting
customer memory master truth
human task infrastructure
scheduler / roster infrastructure
Broadcast / Delivery infrastructure
Reminder infrastructure
```

### 4.3 Correct owner split

```text id="doc80_owner_split"
PH1.ECOMMERCE = customer-facing online/app/personal shopping.
PH1.POS = physical/in-store commerce surface.
PH1.ORDER = real order records, order groups, seller orders, order lines, order lifecycle.
PH1.PRODUCT = product/service truth.
PH1.INVENTORY = stock truth, reservations, availability.
PH1.PRICING = price, discounts, margin, offers.
PH1.PAYMENT / SETTLEMENT = payment authorization, capture, refund, settlement.
PH1.B2B_PLATFORM = Original Provider, Channel Store, commission, provider payout, reserves.
PH1.DISPATCH = picking, packing, courier handoff.
PH1.RETURNS = return logistics and refund workflow.
PH1.WARRANTY = warranty claims and orphan-provider workflows.
PH1.CUSTOMER = customer identity, preferences, standing instructions.
PH1.MP = habits, demand, recurring patterns.
PH1.ACCOUNTING = ledger posting.
PH1.TAX = tax treatment.
PH1.AUDIT = proof.
```

Order coordinates. It does not become the king of everything. That way lies ERP soup.

---

## 5. Relationship to Documents 77, 78, and 79

### 5.1 From Document 77 — E-Commerce

E-Commerce sends order intent:

```text id="doc80_from_77"
customer wants these items
customer selected these products/services
source/store/provider preference may be resolved
recipient/delivery context
customer confirmation
payment permission request
substitution permissions
standing instructions
```

### 5.2 From Document 78 — B2B

B2B provides:

```text id="doc80_from_78"
Original Provider of Record
Channel Store attribution
B2B settlement required
provider responsibility
customer-visible return/warranty summaries
provider support route
B2B restrictions
```

Order records B2B context and sends lifecycle events back to B2B.

Order does **not** calculate commission or provider payout.

### 5.3 From Document 79 — POS

POS sends:

```text id="doc80_from_79"
cashier checkout
customer-carried cart
scan-as-you-shop cart
restaurant bill
bill split
physical return intake
POS payment status
```

Order converts POS activity into clean seller orders and order lines.

---

## 6. Customer Order Group vs Seller Orders

Order Management must distinguish:

```text id="order_group_terms"
Customer Order Group = what the customer experiences as one request.
Seller Order = the real order belonging to one seller/provider/store.
Order Line = each product/service inside a seller order.
```

Example:

```text id="order_group_example"
Customer Order Group:
“Saturday party shopping”

Seller Orders:
- Cake Shop Order
- Ice Cream Shop Order
- Supermarket Order
```

Each seller order has its own:

```text id="seller_order_properties"
seller
provider
order number
payment requirement
delivery / pickup / service method
invoice / receipt / tax treatment
return rules
warranty/guarantee rules
status
audit trail
```

The customer can see one simple Selene summary.

Each business gets its own correct order.

This is the difference between elegance and a shopping cart with a nervous breakdown.

---

## 7. Source / Store / Provider Resolution Handoff

Order Management must not randomly choose where to buy from.

Before a seller order is finalized, every order line must have a resolved source.

Source may be:

```text id="source_types"
specific company store
preferred store
usual provider
Original Provider
B2B provider
supermarket
bakery
restaurant
salon
professional service provider
fallback provider
customer-approved substitute provider
```

Source resolution comes from:

```text id="source_resolution_inputs"
E-Commerce
POS
Customer Memory
Customer Standing Instructions
B2B
Product
Inventory
Pricing
Search
Memory Pattern Engine
```

Order Management enforces:

```text id="source_resolution_rule"
No unresolved source = no confirmed seller order.
```

If unresolved:

```text id="source_unresolved"
Order line remains draft.
Customer or authorized user must decide.
```

Example:

```text id="source_example"
bread must come from Sam Bakery
shoes must come from Alice Shoes, not Raymond Shoes
milk may come from cheapest preferred supermarket
```

Selene does not buy bread from a random bakery because the word “bread” floated by. We are not building a vending machine with amnesia.

---

## 8. Item-Level Store Preference

Customer preferences must support item-level store choice.

Examples:

```text id="item_preferences"
bread → Sam Bakery
cake → Lin Cake Shop
milk → Harry’s Supermarket
shoes → Alice Shoes
handbag → Bella Fashion
toilet paper → cheapest approved supermarket
```

Preferences may include:

```text id="preference_types"
always use this store
prefer this store unless unavailable
never use this store
local only
cheapest acceptable
premium only
fastest delivery
ask every time
auto-substitute under limit
```

Order Management receives and stores the resolved source for each order line.

Customer Memory owns durable preference truth.

Order records the source used for this order.

---

## 9. Substitution Rules

Each item may have substitution rules.

Substitution policies:

```text id="substitution_policies"
no substitute
same store only
same brand only
same category allowed
cheaper substitute allowed
premium substitute allowed
fastest delivery allowed
ask every time
auto-substitute under price limit
auto-substitute under quality threshold
```

Examples:

```text id="substitution_examples"
Bread:
ask before substituting

Milk:
same brand only

Toilet paper:
cheapest approved brand

Shoes:
no substitute without customer approval

Groceries:
auto-substitute if price difference is under 10%
```

Order Management must track:

```text id="substitution_tracking"
original requested item
substitute offered
substitute accepted/rejected
who approved
price difference
seller/source difference
audit evidence
```

No “I ordered bread and received a suspicious muffin.” Selene has standards. Low bar, still cleared.

---

## 10. Out-of-Stock Alternatives

If preferred source is out of stock, Order Management must route recovery.

Example:

```text id="out_stock_example"
Sam Bakery is out of the customer’s usual bread.
```

Selene can offer:

```text id="out_stock_options"
same store, different item
same item, different store
different brand
cheaper option
premium option
wait for restock
cancel line
ask customer
use standing substitute rule
```

Customer-facing example:

```text id="out_stock_phrase"
“Sam Bakery is out of your usual bread. I found two options: similar bread from Sam Bakery, or the same style from Lin Bakery.”
```

If customer standing rules allow auto-substitution, Order may proceed and record the rule used.

If not, Order line pauses for decision.

---

## 11. Customer Approval Thresholds

Order Management must know when Selene can act and when the customer must confirm.

Approval thresholds may be based on:

```text id="approval_threshold_basis"
price difference
seller change
brand change
delivery delay
substitution category
high-value item
new recipient
new address
B2B provider
professional service
payment method
```

Examples:

```text id="approval_threshold_examples"
auto-approve grocery substitutions under $5
ask before changing bread bakery
ask before changing shoe store
ask before spending over $100
ask before using credit instead of points
```

Order Management must record:

```text id="approval_threshold_record"
threshold rule
decision applied
customer confirmation if required
audit reference
```

---

## 12. Delivery Cost / Timing Comparison

Before finalizing multi-source orders, Selene should compare delivery cost and timing.

Checks:

```text id="delivery_cost_checks"
separate delivery fees
combined delivery possibility
pickup option
same-day vs later delivery
different sellers
different provider fulfillment
delivery to recipient
delivery windows
customer urgency
```

Customer-facing example:

```text id="delivery_cost_phrase"
“Using separate stores adds $8 delivery. I can combine milk and biscuits from one supermarket, but bread still comes from Sam Bakery.”
```

Order Management records selected delivery plan.

Dispatch/Delivery executes.

Order does not book the courier; Order creates the need.

---

## 13. Order-Line Lifecycle

Every order line has its own lifecycle.

Example:

```text id="line_lifecycle_example"
milk = delivered
bread = waiting for bakery confirmation
shoes = dispatched
handbag = out of stock
biscuits = cancelled
```

Order-line statuses:

```text id="line_statuses"
Draft
SourceUnresolved
SubstitutionRequired
PendingCustomerDecision
PendingPayment
PaymentAuthorized
Confirmed
SellerAccepted
InventoryReservationRequested
Reserved
ReservationExpired
AwaitingFulfillment
AwaitingDispatch
Dispatched
Delivered
ServiceScheduled
ServiceInProgress
ServiceAccepted
Cancelled
ReturnRequested
Returned
RefundPending
Refunded
WarrantyOpen
Closed
```

Order groups and seller orders must summarize line status without hiding important exceptions.

No one-word “processing.” That word has done enough damage.

---

## 14. Draft vs Confirmed vs Binding Order

Order Management must distinguish order commitment state.

States:

```text id="commitment_states"
Draft = customer/user is still deciding
Confirmed = customer approved the order
PaymentAuthorized = payment authorization exists
SellerAccepted = seller/provider accepted responsibility
Binding = change requires cancellation/return rules
Closed = complete and reconciled
```

This prevents confusion between:

```text id="draft_vs_order"
Selene drafted it
Selene confirmed it
seller accepted it
payment went through
goods are actually coming
```

Customer must not believe a draft is a paid order.

Seller must not receive a draft as a binding order.

Simple, but apparently software needs adult supervision.

---

## 15. Inventory Reservation + Expiry

Order Management requests reservations from Inventory.

Inventory owns stock truth.

Reservation must define:

```text id="reservation_fields"
product
variant
quantity
location
seller
reservation expiry time
payment dependency
customer decision dependency
fallback action
audit reference
```

Reservation outcomes:

```text id="reservation_outcomes"
reserved
partially reserved
not available
reservation expired
reservation released
reservation failed
```

If customer delays payment or decision, reservation may expire.

Order must handle:

```text id="reservation_recovery"
ask customer to reconfirm
request new reservation
offer substitute
cancel line
backorder
```

No more “it was in stock when I clicked” mythology without reservation truth.

---

## 16. Allocation Priority

When stock is limited, Order must follow allocation policy.

Possible priority rules:

```text id="allocation_priority"
first paid
first confirmed
VIP customer
store account customer
reserved stock
business-critical order
perishable expiry priority
manual override
customer promise priority
```

Order does not invent priority.

Inventory / Seller / Policy owns priority rules.

Order records applied priority and audit.

---

## 17. Payment Timing Per Seller / Order Line

Order Management must support payment timing differences.

Payment timing may be:

```text id="payment_timing"
pay now
authorize now, capture later
pay after seller confirmation
pay after dispatch
pay after service acceptance
store account terms
customer credit
installment plan
B2B settlement hold
restaurant split bill
partial payment
deposit + balance
milestone payment
```

Order sends Payment/Settlement:

```text id="payment_handoff"
customer order group
seller order
order lines
amounts
tax references
discount references
payment method
payment timing
settlement context
B2B context
audit ref
```

Payment executes.

Order records payment status.

Order does not touch the money with its tiny order fingers.

---

## 18. Settlement Context

Order must flag settlement context.

Possible settlement contexts:

```text id="settlement_contexts"
normal seller settlement
Selene settlement trust
B2B settlement
provider payout hold
channel commission context
store account
customer credit
wallet / reward credit
restaurant split payment
service deposit / milestone
```

Order sends lifecycle events to Payment, B2B, Accounting, and Tax.

Order does not release settlement.

Money movement belongs to Payment/Settlement and B2B.

---

## 19. Split Delivery + Delivery Grouping

Order Management must support:

```text id="delivery_grouping"
one delivery
split delivery
seller-specific delivery
provider fulfillment
pickup
recipient delivery
gift delivery
service appointment
digital delivery
restaurant table service
warehouse pickup
```

Order should group when useful:

```text id="grouping_goals"
reduce delivery cost
meet customer deadline
avoid delays
separate perishable goods
separate fragile goods
respect seller/provider rules
respect recipient privacy
```

Customer-facing summary:

```text id="delivery_summary"
“Groceries arrive today. Bread arrives tomorrow morning. Shoes arrive Friday.”
```

Order tracks the promises.

Dispatch/Delivery executes the routes.

---

## 20. Order Promise / ETA Management

Order Management must track what Selene promised the customer.

Promises may include:

```text id="promise_fields"
delivery date
pickup time
dispatch window
restaurant booking time
service appointment time
provider SLA
gift delivery date
recipient notification timing
```

If a promise breaks:

```text id="promise_break_actions"
warn customer
offer substitute
offer cancellation
offer delayed delivery
offer partial fulfillment
route exception
update seller/provider score where relevant
```

Customer should not need to ask, “Where is it?” like their order vanished into a 2012 tracking portal.

---

## 21. Fulfillment Source Switching

If the selected source cannot fulfill after confirmation, Order must handle source switching.

Triggers:

```text id="source_switch_triggers"
stock disappeared
seller rejected order
provider cannot fulfill
delivery no longer possible
store closed
quality issue
recall
customer changed delivery requirement
```

Recovery:

```text id="source_switch_recovery"
use approved fallback
ask customer
cancel line
backorder
alternate store
alternate provider
split delivery
```

Order must record:

```text id="source_switch_record"
original source
new source
reason
customer approval if required
price/delivery difference
B2B/settlement changes
audit ref
```

---

## 22. Order Dependency Rules

Some order lines depend on other lines.

Examples:

```text id="dependency_examples"
cake + candles + flowers
shoes + matching bag
restaurant booking + pre-ordered dishes
machine + installation service
gift + recipient address
event ticket + transport
```

Dependency rules:

```text id="dependency_rules"
all-or-nothing
continue without dependent item
ask customer
replace failed dependency
hold entire group
cancel related lines
```

Example:

```text id="dependency_phrase"
“The cake is unavailable. Should I still order the candles and flowers?”
```

Without dependency logic, Selene may send birthday candles with no cake, which is just emotional vandalism.

---

## 23. Service and Appointment Order Handling

Order must support non-product orders.

Service order types:

```text id="service_order_types"
restaurant booking
restaurant pre-order
salon appointment
professional service
installation
repair
maintenance
event ticket
training session
consulting milestone
digital service
```

Service order fields:

```text id="service_order_fields"
service provider
appointment time
location
customer / recipient
deposit
balance
cancellation rule
service acceptance rule
milestone status
professional compliance reference if applicable
```

Services do not dispatch like boxes.

Order tracks the service lifecycle and hands execution to the correct service engine.

---

## 24. Milestone / Partial Service Orders

Some services require milestones.

Milestone states:

```text id="milestone_states"
DepositRequired
DepositPaid
Scheduled
Started
MilestonePending
MilestoneAccepted
MilestoneRejected
FinalAcceptancePending
Completed
BalancePayable
Closed
```

Applies to:

```text id="milestone_applies"
installations
repairs
professional services
events
projects
maintenance
custom work
```

Payment may be tied to milestones.

Order tracks.

Payment executes.

Service engine proves completion.

---

## 25. Recurring / Scheduled Orders

Order must support recurring orders.

Examples:

```text id="recurring_examples"
weekly groceries
monthly office supplies
subscription boxes
routine business supplies
restaurant standing orders
recurring service appointments
```

Recurring order controls:

```text id="recurring_controls"
auto-place
confirm before placing
pause
skip
change quantity
change date
change source
change payment method
cancel recurrence
```

Recurring order should use:

```text id="recurring_inputs"
customer standing instructions
Memory Pattern Engine
Inventory signals
schedule
payment rules
source preferences
substitution policies
```

No accidental double weekly groceries because the app refreshed. Selene is not a bread duplicator.

---

## 26. Customer Standing Instructions

Order Management must apply standing instructions from Customer Memory.

Examples:

```text id="standing_instruction_examples"
always use Sam Bakery for bread
never substitute shoes without asking
use cheapest acceptable milk
deliver groceries to office on Fridays
ask before spending over $100
use points before card
do not tell mum the gift price
send work supplies to warehouse
```

Order records which standing instruction was applied.

Customer Memory owns durable rule truth.

Order applies rule to this order.

---

## 27. Duplicate Order Prevention

Order Management must prevent accidental duplicates.

Triggers:

```text id="duplicate_triggers"
same customer
same items
same seller
same timeframe
same payment retry
same voice/chat repeated
network retry
POS resend
recurring order overlap
```

Order must support:

```text id="duplicate_controls"
idempotency key
duplicate warning
repeat intent confirmation
safe retry
customer confirmation
auto-block duplicate where policy allows
```

Example:

```text id="duplicate_phrase"
“You already ordered your weekly list this morning. Do you want to order it again?”
```

Nobody needs 14 loaves of bread because Wi-Fi sneezed.

---

## 28. Order Change Rules

Customer may request changes:

```text id="order_change_types"
quantity
size
color
variant
seller/source
delivery address
recipient
delivery date
pickup/delivery option
payment method
substitution rule
gift privacy
```

Order checks:

```text id="change_checks"
order state
payment state
seller acceptance
inventory reservation
dispatch status
B2B settlement impact
return/cancel requirement
authority/verification requirement
```

If change is allowed, Order versions the record.

If change is not allowed, Order routes to cancel/return workflow.

---

## 29. Cancellation Routing

Customer may cancel:

```text id="cancel_examples"
“Cancel the shoes.”
“Cancel the bread.”
“Cancel the whole order.”
```

Order identifies:

```text id="cancel_identify"
order group
seller order
order line
payment state
dispatch state
seller/provider
B2B context
return/cancellation rule
```

Cancellation outcomes:

```text id="cancel_outcomes"
cancel draft
cancel before payment
void authorization
refund captured payment
stop dispatch
request seller/provider cancellation
courier intercept
convert to return
deny cancellation under policy
```

Order routes action to:

```text id="cancel_routing"
Payment
Dispatch
B2B
Seller/Provider
Returns
Customer
Audit
```

Order does not refund money itself.

---

## 30. Return Eligibility Handoff

If customer wants to return, Order sends Returns:

```text id="return_handoff"
customer
order group
seller order
order line
item
seller/provider
delivery date
payment state
return window
recipient/payer relationship
B2B context
condition evidence if available
audit ref
```

Returns owns:

```text id="returns_owns"
return logistics
inspection
courier pickup
refund execution handoff
restock / damage / quarantine
```

Order updates lifecycle based on Returns events.

---

## 31. Warranty Handoff

If customer raises warranty issue, Order identifies:

```text id="warranty_context"
order line
seller
Original Provider
purchase date
warranty summary
receipt/proof
delivery proof
customer/recipient
B2B context
```

Order sends to Warranty Engine.

Warranty owns claim workflow.

Order records warranty status.

---

## 32. Human / External Action Orchestration

Order must follow the Selene Human / External Action Orchestration Law.

Any human/external action must define:

```text id="action_orchestration_fields"
action type
owner
recipient
backup owner where needed
authority requirement
schedule / due time
delivery method
required confirmation
required evidence
reminder rule
escalation path
closure condition
audit reference
```

Examples:

```text id="order_action_examples"
customer must confirm substitution
provider must confirm availability
seller must accept order
manager must approve high-value order
customer must provide missing address
dispatch must confirm urgent fulfillment
```

Order creates the action requirement.

Task / Scheduler / Broadcast / Delivery / Reminder / Access / Audit manage execution.

No “notify someone.” Selene assigns, sends, chases, confirms, escalates, and audits. Like a project manager, but less likely to say “circle back.”

---

## 33. Fraud / Risk / High-Value Review

Some orders require risk review.

Risk triggers:

```text id="risk_triggers"
high order value
new delivery address
new recipient
unusual quantity
unusual source
many failed payments
many returns
suspicious payment method
B2B high-risk item
professional service risk
customer behavior anomaly
manual override
```

Risk outcomes:

```text id="risk_outcomes"
allow
step-up verification
manual review
hold order
split order
deny line
request more information
route to fraud/compliance
```

Order does not own fraud intelligence fully.

Order detects and routes.

---

## 34. Exception Recovery Playbooks

Order must route exceptions to recovery actions.

Examples:

```text id="exception_playbooks"
payment failed → ask for another payment method
stock failed → substitute / backorder / cancel line
source failed → alternate source / customer decision
dispatch failed → reroute / reschedule / refund option
delivery failed → courier chase / replacement / refund
provider failed → alternate provider / cancel / escalate
address missing → request address
customer unavailable → reschedule
seller rejected → alternate seller / cancel
```

Every exception needs a next action.

“Problem occurred” is not an action. It is software shrugging in public.

---

## 35. Customer Communication Timeline

Order Management must manage customer-visible updates.

Events may include:

```text id="communication_events"
order drafted
order confirmed
payment authorized
seller accepted
stock reserved
substitution needed
item unavailable
order split
dispatch started
delivery delayed
delivered
return window active
return started
refund issued
warranty opened
order closed
```

Communication must respect:

```text id="communication_rules"
privacy
gift mode
recipient permissions
channel preference
quiet hours
human/external action orchestration where action is required
```

Customer should not need to ask what is happening. Selene should already be mildly ahead of them, as usual.

---

## 36. Privacy and Seller Data Walls

Order must enforce data boundaries.

Customer can see:

```text id="customer_can_see"
full customer order group
all seller orders
all delivery statuses
all payment summaries
all return/warranty options
```

Seller can see:

```text id="seller_can_see"
only its own seller order
its own order lines
fulfillment data needed
recipient/delivery data needed
customer support data allowed
```

Seller cannot see:

```text id="seller_cannot_see"
other seller orders
customer full personal Selene profile
private shopping history
other provider data
unrelated payment methods
unrelated family relationships
```

Example:

```text id="privacy_example"
Bakery sees bread order.
Supermarket sees groceries.
Shoe Store sees shoes.
Customer sees all.
```

Everyone gets their truth, not the whole village diary.

---

## 37. Audit and Order Versioning

Every meaningful order change must be versioned.

Track:

```text id="versioning_fields"
who changed it
what changed
when changed
why changed
old value
new value
approval
customer confirmation
engine decision
audit reference
```

Versioned changes include:

```text id="versioned_changes"
source change
substitution
quantity change
price change
delivery address change
recipient change
payment method change
cancellation
return
refund
warranty opening
seller/provider change
manual override
```

No mystery “system updated order.” The system has a reason and a receipt.

---

## 38. Order Closure Rules

Order closes only when all required lifecycle obligations are resolved.

Closure checks:

```text id="closure_checks"
seller orders complete or cancelled
order lines delivered / accepted / completed / cancelled
payment settled or refunded
B2B lifecycle events sent
dispatch closed
returns closed
refunds closed
warranty proof stored if needed
accounting/tax handoff complete
customer communication complete
audit complete
```

Order must detect zombie states:

```text id="zombie_states"
paid but not dispatched
dispatched but not delivered
delivered but not settled
returned but not refunded
seller order missing from customer group
B2B event missing
```

No haunted “processing” orders left in the basement.

---

## 39. Order Reconciliation

Order Management must reconcile:

```text id="order_reconciliation"
customer order group
seller orders
order lines
payment records
settlement records
inventory reservations
dispatch records
delivery proof
return/refund records
B2B events
warranty events
accounting/tax handoff
audit trail
```

If mismatch:

```text id="reconciliation_actions"
open exception
route to owner
create action requirement
remind/escalate
block closure until resolved
```

Order reconciliation is the final sanity check before the system pretends everything is fine.

---

## 40. Order Intelligence Signals

Order sends signals to other engines.

Signals include:

```text id="order_signals"
usual order group
frequent substitution
preferred delivery day
seller reliability
provider delays
cancel habits
return habits
stockout pattern
delivery promise failures
customer reorder rhythm
recurring order suitability
```

Recipients:

```text id="signal_recipients"
Customer Memory
Memory Pattern Engine
Inventory
B2B
Seller/Provider scoring
Marketing
Pricing
Returns
Warranty
Audit
```

Order sends signals.

Memory engines own durable learning.

Order is not a diary. It is the order truth machine.

---

## 41. State Machines

### 41.1 Customer Order Group State

```text id="state_order_group"
Draft
SourceResolutionPending
CustomerDecisionPending
Confirmed
PartiallyConfirmed
PaymentPending
PartiallyPaid
SellerOrdersCreated
PartiallyFulfilled
Fulfilled
PartiallyCancelled
Cancelled
ReturnOpen
WarrantyOpen
ReconciliationPending
Closed
```

### 41.2 Seller Order State

```text id="state_seller_order"
Draft
PendingSellerAcceptance
Accepted
Rejected
PaymentPending
PaymentAuthorized
AwaitingStock
ReadyForFulfillment
PartiallyFulfilled
Fulfilled
PartiallyDelivered
Delivered
Cancelled
ReturnOpen
RefundPending
Closed
```

### 41.3 Order Line State

```text id="state_order_line"
Draft
SourceUnresolved
SubstitutionRequired
PendingCustomerDecision
PendingPayment
PaymentAuthorized
Confirmed
SellerAccepted
ReservationRequested
Reserved
ReservationExpired
AwaitingFulfillment
AwaitingDispatch
Dispatched
Delivered
ServiceScheduled
ServiceInProgress
ServiceAccepted
Cancelled
ReturnRequested
Returned
RefundPending
Refunded
WarrantyOpen
Closed
```

### 41.4 Source Resolution State

```text id="state_source_resolution"
NotStarted
PreferenceFound
SourceSuggested
SourceConfirmed
SourceUnavailable
AlternativeSuggested
CustomerDecisionRequired
FallbackApplied
Failed
Closed
```

### 41.5 Substitution State

```text id="state_substitution"
NotRequired
Required
AutoAllowed
CustomerDecisionRequired
Offered
Accepted
Rejected
Unavailable
Closed
```

### 41.6 Payment Requirement State

```text id="state_payment_requirement"
NotRequired
Required
AuthorizationPending
Authorized
CapturePending
Captured
Failed
AlternativeRequired
RefundRequired
Refunded
Closed
```

### 41.7 Fulfillment State

```text id="state_fulfillment"
NotReady
ReadyForFulfillment
InventoryReserved
ProviderPending
DispatchRequested
Dispatched
Delivered
ServiceScheduled
ServiceCompleted
Failed
ExceptionOpen
Closed
```

### 41.8 Exception State

```text id="state_exception"
None
Detected
OwnerAssigned
CustomerActionRequired
SellerActionRequired
ProviderActionRequired
PaymentActionRequired
DispatchActionRequired
RecoveryInProgress
Escalated
Resolved
Closed
```

### 41.9 Reconciliation State

```text id="state_reconciliation"
NotReady
Pending
MismatchDetected
OwnerAssigned
CorrectionPending
Resolved
Closed
```

---

## 42. Reason Codes

```text id="reason_codes"
ORDER_INTENT_RECEIVED
CUSTOMER_ORDER_GROUP_CREATED
SELLER_ORDER_CREATED
ORDER_LINE_CREATED
SOURCE_RESOLUTION_REQUIRED
SOURCE_PREFERENCE_FOUND
SOURCE_CONFIRMED
SOURCE_UNAVAILABLE
ALTERNATIVE_SOURCE_SUGGESTED
CUSTOMER_DECISION_REQUIRED
SUBSTITUTION_REQUIRED
SUBSTITUTION_AUTO_APPLIED
SUBSTITUTION_ACCEPTED
SUBSTITUTION_REJECTED
DELIVERY_COST_COMPARISON_REQUIRED
DELIVERY_GROUPING_SELECTED
INVENTORY_RESERVATION_REQUESTED
INVENTORY_RESERVATION_CONFIRMED
INVENTORY_RESERVATION_EXPIRED
PAYMENT_REQUIREMENT_CREATED
PAYMENT_AUTHORIZED
PAYMENT_FAILED
B2B_CONTEXT_ATTACHED
B2B_SETTLEMENT_HANDOFF_REQUIRED
DISPATCH_HANDOFF_CREATED
SERVICE_ORDER_HANDOFF_CREATED
RECIPIENT_DELIVERY_ATTACHED
GIFT_PRIVACY_ATTACHED
ORDER_PROMISE_CREATED
ORDER_PROMISE_AT_RISK
ORDER_PROMISE_BROKEN
SELLER_ACCEPTED_ORDER
SELLER_REJECTED_ORDER
FULFILLMENT_SOURCE_SWITCH_REQUIRED
ORDER_DEPENDENCY_FAILED
ORDER_CHANGE_REQUESTED
ORDER_CHANGE_ALLOWED
ORDER_CHANGE_DENIED
DUPLICATE_ORDER_DETECTED
RISK_REVIEW_REQUIRED
HIGH_VALUE_REVIEW_REQUIRED
CANCELLATION_REQUESTED
CANCELLATION_ROUTED
RETURN_HANDOFF_CREATED
WARRANTY_HANDOFF_CREATED
RECURRING_ORDER_CREATED
RECURRING_ORDER_SKIPPED
RECURRING_ORDER_PAUSED
CUSTOMER_COMMUNICATION_SENT
HUMAN_ACTION_ORCHESTRATION_REQUIRED
ORDER_EXCEPTION_OPENED
ORDER_EXCEPTION_RESOLVED
ORDER_VERSION_CREATED
ORDER_RECONCILIATION_MISMATCH
ORDER_RECONCILIATION_RESOLVED
ORDER_CLOSED
```

---

## 43. Required Simulations

```text id="required_simulations"
customer asks Selene to buy bread, milk, shoes, handbag, and biscuits
customer order group created
bread source resolved to Sam Bakery
shoes source resolved to Alice Shoes
customer rejects Raymond Shoes as source
Sam Bakery out of favorite bread
Selene offers same bakery alternative and different bakery alternative
customer approves alternative bread
customer standing instruction prevents shoe substitution
seller orders created for bakery, supermarket, shoe store, fashion store
order lines created with separate lifecycle states
inventory reservation expires before payment
payment timing differs by seller order
B2B item creates B2B handoff
channel/original provider context attached but not calculated by Order
delivery cost comparison shows separate delivery cost
customer chooses combined delivery for groceries
cake unavailable and dependent candles decision required
duplicate weekly order detected
customer confirms duplicate is not intended
high-value handbag triggers risk review
new delivery address triggers step-up verification
restaurant table order becomes service/seller order
salon appointment becomes service order
installation service creates milestone order
customer changes delivery address before dispatch
customer cancellation routes to payment/dispatch/B2B
order line return creates Returns handoff
warranty claim creates Warranty handoff
seller rejects order and source switching begins
source switching requires customer approval
recurring grocery order is skipped for one week
paid but not dispatched mismatch detected in reconciliation
returned but not refunded mismatch detected
seller can only see its own seller order
customer sees full order group
order closes after all seller orders and handoffs complete
```

---

## 44. Integration Map

```text id="integration_map"
PH1.ORDER / ORDER_MANAGEMENT
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_MEMORY / STANDING_INSTRUCTIONS
↔ PH1.MP / MEMORY_PATTERN
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.PRICING / MARGIN / DISCOUNT
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.SETTLEMENT_TRUST / CUSTOMER_PROTECTION
↔ PH1.B2B_PLATFORM
↔ PH1.ORIGINAL_PROVIDER / PROVIDER_OF_RECORD
↔ PH1.DISPATCH / PACKING / COURIER_HANDOFF
↔ PH1.LOGISTICS / DELIVERY
↔ PH1.RETURNS / REFUNDS / REVERSE_LOGISTICS
↔ PH1.WARRANTY / AFTER_SALES
↔ PH1.RESTAURANT / MENU / BOOKING
↔ PH1.EVENTS / INVITATIONS / RSVP
↔ PH1.PROFESSIONAL_SERVICES_COMPLIANCE
↔ PH1.CUSTOMER_CREDIT / WALLET / VIRTUAL_SETTLEMENT
↔ PH1.REWARDS / LOYALTY
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.CASHFLOW
↔ PH1.ACCESS / AUTHORITY
↔ PH1.TASK / HUMAN_WORKLOAD
↔ PH1.SCHEDULER / ROSTERS
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
```

---

## 45. Required Logical Packets

```text id="logical_packets"
OrderIntentPacket
CustomerOrderGroupPacket
SellerOrderPacket
OrderLinePacket
SourceResolutionPacket
ItemStorePreferencePacket
SubstitutionPolicyPacket
OutOfStockAlternativePacket
CustomerApprovalThresholdPacket
DeliveryCostTimingComparisonPacket
InventoryReservationRequestPacket
InventoryReservationStatusPacket
PaymentRequirementPacket
SettlementContextPacket
B2BOrderContextPacket
FulfillmentRoutingPacket
DispatchRequirementPacket
ServiceOrderHandoffPacket
MilestoneOrderPacket
RecipientOrderContextPacket
GiftPrivacyOrderPacket
OrderPromisePacket
FulfillmentSourceSwitchPacket
OrderDependencyPacket
OrderChangeRequestPacket
CancellationRoutingPacket
ReturnEligibilityHandoffPacket
WarrantyContextHandoffPacket
DuplicateOrderDetectionPacket
RiskReviewPacket
RecurringOrderPacket
CustomerStandingInstructionPacket
CustomerCommunicationTimelinePacket
OrderExceptionPacket
OrderVersionPacket
OrderReconciliationPacket
OrderAuditEvidencePacket
```

Logical only.

No runtime packet structs. The schema goblin can wait outside and stop licking the delivery labels.

---

## 46. What Codex Must Not Do

```text id="codex_must_not"
Do not make Order Management own E-Commerce shopping UX.
Do not make Order Management own Purchase Orders.
Do not make Order Management own product truth.
Do not make Order Management own inventory stock truth.
Do not make Order Management own pricing truth.
Do not make Order Management own payment execution.
Do not make Order Management own settlement release.
Do not make Order Management own B2B commission, provider payout, or reserves.
Do not make Order Management own dispatch operations.
Do not make Order Management own return logistics.
Do not make Order Management own warranty execution.
Do not make Order Management own tax treatment.
Do not make Order Management own ledger posting.
Do not create seller orders with unresolved source/provider.
Do not silently substitute without customer rule or approval.
Do not merge all sellers into one fake seller order.
Do not expose one seller’s order data to another seller.
Do not leave order lines stuck in vague “processing.”
Do not ignore reservation expiry.
Do not ignore duplicate order detection.
Do not bypass step-up verification for protected changes.
Do not use vague notification/escalation without Human / External Action Orchestration.
Do not let GPT-5.5 invent stock, price, delivery, payment, seller acceptance, warranty, or refund facts.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 47. Final Architecture Sentence

Selene Order Management + Order Orchestration Engine is the order truth and lifecycle control layer that converts customer intent from E-Commerce, POS, scan-as-you-shop, buying lists, services, restaurant/table orders, and personal Selene search into clean customer order groups, seller orders, and line-level order records; enforces source/store/provider resolution, item-level preferences, substitutions, availability, reservations, payment requirements, delivery promises, seller acceptance, B2B context, service milestones, recurring order rules, cancellation, change, return, warranty, privacy, audit, reconciliation, and closure; while leaving product truth, stock truth, pricing, payment, B2B settlement, dispatch, returns, warranty, tax, accounting, and human-action delivery to their proper owner engines.

Simple version:

```text id="simple_version"
E-Commerce/POS helps the customer buy.
Order Management creates the real orders.
Customer may see one clean shopping group.
Each seller gets its own real order.
Each item gets its own order line and lifecycle.
Source/store/provider must be resolved before confirmation.
Substitutions require rules or approval.
Payment is handed to Payment.
Stock is handed to Inventory.
B2B context is handed to B2B.
Dispatch is handed to Dispatch.
Returns are handed to Returns.
Warranty is handed to Warranty.
Accounting and Tax get structured events.
Order tracks everything until truly closed.
Everything important is audited.
```

That is Global Document 80 — Selene Order Management + Order Orchestration Engine. It is not “order placed.” It is “order controlled until reality stops misbehaving.”

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
