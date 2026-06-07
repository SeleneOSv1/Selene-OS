# Global Document 83 — Selene Returns, Refunds + Reverse Logistics Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
83

ENGINE:
PH1.RETURNS / PH1.REFUNDS / PH1.REVERSE_LOGISTICS / PH1.DISPUTE_RESOLUTION

FULL NAME:
Selene Returns, Refunds, Reverse Logistics, Cancellation, Exchange, Replacement, Repair, Warranty Claim, Product-Level Terms, Seller/Supplier Terms, Exception Authorization, Dispute Fact Pack, Orphan Warranty, Abuse Enforcement, Provider Responsibility, Settlement Hold, Commission Clawback, Customer Benefit Reversal, Return Courier Cost, Inspection, Inventory Disposition, and Returns Audit Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Document 83 is Selene’s **returns, refunds, reverse logistics, cancellation, exchange, replacement, repair, warranty, dispute, and after-sale resolution engine**.

It controls what happens when the customer is not keeping the original order exactly as delivered.

It answers:

```text
Can this customer cancel, return, exchange, replace, repair, refund, dispute, or claim warranty?

If yes:
- under what terms?
- who pays?
- who approves?
- does the item come back?
- where does it go?
- when does money move?
- what reverses?
- who is responsible?
- what proof is required?
- what happens if either side abuses the system?
```

Simple version:

```text
82 sends goods out.
83 deals with goods coming back, money going back, goods being replaced, goods being repaired, claims being disputed, and people arguing about policy.
```

That is commerce’s glamorous afterlife. Receipts, broken boxes, disappointed humans, and Dave again somehow involved.

---

## 2. Core Terms-First Return + Refund Law

```text
Selene must not process any cancellation, return, refund, exchange, replacement, repair, warranty claim, or dispute until it has checked:
- the product-level seller/supplier after-sale terms
- the terms version shown to the customer at purchase
- the delivery/receipt date
- the request reason
- the product/service category
- any mandatory legal/compliance override
- any approved exception path
- any B2B/provider/channel responsibility
- any payment/settlement hold state
- any abuse/fraud risk
- required proof and deadlines
```

Selene must always begin with:

```text
What did the seller/supplier say before purchase?
What was the customer shown?
What does law/compliance require?
What actually happened?
What evidence exists?
Who is responsible?
What resolution is allowed?
```

No “customer feels sad, refund everything” engine.

No “seller says no, case closed” engine.

Selene needs governed resolution, not emotional ping-pong with screenshots.

---

## 3. What Document 83 Owns

Document 83 owns:

```text
return request intake
refund request intake
cancellation request intake
exchange request intake
replacement request intake
repair request intake
warranty claim intake
product-level terms resolver
seller/supplier terms lookup
terms display evidence check
mandatory compliance/legal handoff trigger
return/refund eligibility
exception authorization workflow
return authorization / RMA
customer evidence collection
seller/provider evidence collection
return courier / reverse logistics booking
return courier cost allocation
return label / QR return label
pickup/drop-off/store/locker return flow
inspection workflow
condition grading
refund calculation
partial refund calculation
replacement decision
exchange decision
repair decision
warranty route decision
provider responsibility determination
customer responsibility determination
courier responsibility determination
B2B commission/customer benefit reversal signal
settlement hold/release/reversal signal
reserve/deposit usage signal
inventory disposition signal
dispute fact pack
appeal/dispute workflow
customer/supplier abuse signal
bad actor enforcement handoff
return/refund audit evidence
```

---

## 4. What Document 83 Does Not Own

Document 83 does **not** own:

```text
original order creation
original outbound dispatch execution
product master truth
inventory master truth
payment execution
bank settlement execution
accounting ledger posting
tax law
legal final interpretation
B2B marketplace attribution
pricing final decision
warranty policy ownership
courier company infrastructure
```

Correct owner split:

```text
Document 80 Order = order and order-line lifecycle truth.
Document 82 Dispatch = outbound shipping, delivery proof, courier handoff.
Document 83 Returns = after-sale return/refund/replacement/repair/warranty/dispute workflow.
Product = product/service truth and product-level terms metadata.
Payment/Settlement = money execution.
Accounting/Tax = financial/tax treatment.
Document 78 B2B = B2B attribution/provider responsibility framework.
Document 81E = B2B commission/customer benefit/provider payout reversal economics.
Document 81G = pricing/dispute evidence, audit, fairness, decision replay.
Inventory = stock truth and stock disposition.
Legal/Compliance = final legal/compliance interpretation.
```

83 coordinates the mess. It does not become the entire swamp. We are trying to keep the swamp in labeled containers.

---

## 5. Relationship to Document 80 Order Management

Document 80 provides:

```text
customer order group
seller order
order line
product/service
quantity
price paid
discounts/offers used
payment status
delivery promise
recipient
delivery status
B2B context
account/customer context
cancellation state
```

Document 83 updates Order with:

```text
return requested
refund requested
refund approved
refund rejected
exchange requested
replacement created
repair started
warranty claim opened
line returned
line refunded
line partially refunded
line exchanged
line replaced
line repaired
line disputed
line closed
```

Order remains the customer-facing order truth.

83 owns the after-sale resolution lifecycle.

---

## 6. Relationship to Document 82 Dispatch

Document 82 provides:

```text
dispatch proof
pick proof
pack proof
photo proof
package identity
tracking
delivery proof
courier scan
signature / OTP / ID proof
delivery exception
failed delivery
return-to-sender event
in-transit intercept result
courier damage/loss evidence
```

83 uses dispatch evidence to decide:

```text
did customer receive goods?
was item delivered late?
was wrong item dispatched?
was package damaged in transit?
was delivery refused?
did courier fail?
can package be intercepted?
does return/refund begin?
```

If outbound dispatch caused the problem, 83 does not let everyone blame the customer like a tiny corporate reflex.

---

## 7. Relationship to Product-Level Terms

Each product/service must have a product-level after-sale terms profile.

Terms may include:

```text
cancellation terms
return terms
refund terms
exchange terms
replacement terms
repair terms
warranty terms
opened-item rules
custom-made rules
perishable rules
hygiene rules
digital/service consumption rules
final-sale rules
return courier cost rule
restocking fee rule
inspection-before-refund rule
provider approval rule
return destination
refund timing
dispute process
```

Product owns the terms metadata.

83 resolves and applies those terms to each request.

---

## 8. Product-Level After-Sale Terms Profile

83 must load the applicable terms before processing any request.

Terms profile fields:

```text
product_id / service_id
seller_id
provider_id
brand_id where applicable
country/region scope
channel scope
return window
refund window
exchange window
cancellation window
warranty period
opened item rule
used item rule
custom-made rule
perishable rule
hygiene rule
service cancellation rule
digital access rule
return courier payer
inspection required
refund-before-return allowed
provider approval required
restocking fee rule
non-returnable conditions
exception approver
legal/compliance flag
version
effective date
expiry/review date
```

Example:

```text
This item cannot be returned after opening unless faulty.
Warranty is handled by supplier.
Change-of-mind return courier is paid by customer.
Refund is issued after inspection.
```

This is the spine of 83.

Without it, returns become customer feelings versus seller memory. A majestic disaster.

---

## 9. Terms Display Evidence Record

Selene must know what terms the customer was shown before purchase.

Evidence may include:

```text
product page terms version
checkout terms version
POS receipt terms
B2B listing terms
order confirmation terms
customer-facing summary
timestamp
channel
language/locale
customer acknowledgement where required
screenshotted/rendered display evidence where applicable
```

83 must be able to answer:

```text
What terms applied when the customer bought this?
Were they shown clearly?
Were they changed later?
Was the product marked final sale?
Was return courier responsibility disclosed?
```

No “policy says…” unless Selene can show which policy, when, and where. Policies without evidence are just office folklore.

---

## 10. Customer-Safe Terms Summary

Selene should present terms in plain language before purchase and during return request.

Examples:

```text
This item cannot be returned after opening unless faulty.
You may return this within 14 days if unused.
Change-of-mind return courier is paid by you.
Warranty claims are handled by the supplier.
Refund is issued after the returned item is inspected.
Custom-made items cannot be cancelled after production starts unless faulty.
```

Customer-safe terms should appear in:

```text
E-Commerce
B2B
POS
personal Selene shopping
storefront
order confirmation
receipt
customer account
return request flow
```

83 must prefer clear summary plus full terms link/reference.

Tiny grey policy text buried like a legal fossil is how disputes are born.

---

## 11. Seller/Supplier Terms vs Mandatory Compliance Handoff

Seller/supplier terms may not override mandatory law or compliance rules.

If conflict appears, 83 must route to Legal/Compliance.

Examples:

```text
seller says no refunds ever
mandatory law may require remedy for faulty goods
professional service has regulated complaint process
food safety issue requires different handling
cross-border customer protection applies
financial/credit-related product has special rules
```

83 does not decide final law.

83 triggers compliance handoff and records the outcome.

---

## 12. Terms Resolver Before Processing

Before any return/refund/cancellation/warranty workflow, 83 must resolve:

```text
product/service terms
seller/provider terms
brand/manufacturer terms
channel terms
customer/account terms
B2B terms
promotion terms
warranty terms
delivery terms
jurisdiction/compliance override
terms version shown at purchase
```

Output:

```text
normal eligible
not eligible
eligible only if unopened
eligible after inspection
eligible for warranty only
eligible for exchange only
eligible for replacement only
eligible for store credit only
exception approval required
compliance review required
manual review required
```

---

## 13. Return / Refund Request Intake

Customer or authorized party may request:

```text
refund
return
exchange
replacement
repair
warranty claim
missing item claim
damaged item claim
wrong item claim
late delivery claim
not as described claim
quality complaint
cancellation after dispatch
service cancellation
professional service dispute
```

Channels:

```text
Selene app
E-Commerce
POS
voice
chat
email
B2B portal
customer support
provider/store interface
agent/staff assisted return
```

83 must capture:

```text
order line
request reason
customer statement
requested outcome
evidence
preferred return method
contact details
deadline urgency
```

---

## 14. Return Reason Classification

Reasons include:

```text
change of mind
wrong size
wrong color
wrong item
missing item
damaged on arrival
faulty item
not as described
late delivery
expired goods
spoiled goods
poor quality
counterfeit / authenticity dispute
warranty issue
service not provided
professional service dispute
delivery failed
customer refused delivery
duplicate order
fraud / suspicious claim
custom-built mismatch
public safety issue
```

Reason affects:

```text
eligibility
proof required
who pays return courier
who approves
refund timing
provider responsibility
B2B clawback
inventory disposition
abuse score
```

---

## 15. Cancellation vs Return vs Refund vs Warranty

83 must distinguish request types.

```text
Cancellation = customer wants to stop order before completion.
Return = customer sends/returns item back.
Refund = money/credit is given back.
Exchange = customer receives different product/variant.
Replacement = same/similar product is sent due issue.
Repair = product/service is fixed.
Warranty = provider/manufacturer obligation after purchase.
```

A cancellation after dispatch may become:

```text
intercept
return-to-sender
delivery then return
refund after inspection
non-cancellable after point of no return
```

Words matter. Refund goblins love ambiguity.

---

## 16. Return Eligibility Check

83 checks:

```text
return window
refund window
warranty period
product type
condition
custom-made status
perishable status
hygiene rule
digital/service rule
proof of purchase
delivery confirmation
customer account rule
B2B provider rule
brand/manufacturer rule
promotion/final-sale rule
jurisdiction/compliance rule
abuse/fraud flag
```

Output:

```text
eligible
not eligible
manual review
warranty route
replacement only
exchange only
store credit only
provider decision required
compliance review required
exception authorization required
```

---

## 17. Return Window Logic

Return windows may be:

```text
7-day satisfaction period
14-day change-of-mind
30-day return
warranty period
food/perishable same-day rule
custom product non-returnable rule
service cancellation period
subscription billing period
account customer contract rule
B2B provider-specific rule
final-sale clearance rule
```

Window calculation uses:

```text
order date
dispatch date
delivery date
customer received date
proof of delivery
customer acknowledgement
warranty activation date
service completion date
policy start
policy expiry
exception rules
```

---

## 18. 7-Day Satisfaction Hold

From Selene’s commerce model, payment may be held until:

```text
customer received goods
customer had goods for 7 days
customer did not cancel/return/dispute during hold period
```

83 tells Payment/Settlement:

```text
release payment
continue hold
block release
partial release
refund pending
return pending
dispute pending
warranty claim pending
```

This affects:

```text
provider payout
Channel Store commission
Selene fee treatment
customer benefit release
reserve release
```

---

## 19. Refund Eligibility vs Return Eligibility

Return eligibility and refund eligibility are not always the same.

Examples:

```text
return allowed but refund waits for inspection
refund allowed without return
replacement allowed instead of refund
repair required before refund
store credit only
provider must receive goods first
customer gets refund before provider receives goods
return allowed but customer pays courier
warranty claim allowed but change-of-mind refund denied
```

83 must model:

```text
return decision
refund decision
replacement decision
exchange decision
repair decision
warranty decision
```

---

## 20. Return Authorization / RMA

83 creates return authorization where needed.

RMA fields:

```text
return authorization number
order line
return reason
return deadline
return address
return method
return label / QR
condition requirements
photo/evidence requirements
refund expectation
return courier payer
inspection requirement
customer instructions
provider instructions
audit reference
```

No customer sending products into the void and asking where the money went. The void has poor refund discipline.

---

## 21. Customer Evidence Collection

Selene may request:

```text
photos
video
description
serial number
batch/lot
expiry date
package photo
delivery photo
damage photo
wrong item proof
missing item proof
service complaint detail
receipt/order proof
temperature proof
condition proof
```

High-risk claims may require stronger evidence.

Low-risk claims may allow returnless refund or instant replacement if policy allows.

---

## 22. Seller / Provider Evidence Collection

Seller/provider may need to provide:

```text
dispatch proof
packing proof
serial/batch proof
product condition proof
delivery proof
warranty terms
service completion proof
inspection result
provider response
repair report
manufacturer response
```

If provider fails to provide evidence by deadline, 83 applies SLA/no-response rules.

---

## 23. Outside-Policy Exception Authorization Workflow

Sometimes the request is outside normal terms.

Examples:

```text
outside return window
opened item
customer hardship
VIP/goodwill exception
seller wants to help customer
supplier agrees to refund despite policy
brand-sensitive exception
provider fault unclear
```

83 must route:

```text
seller/supplier exception task
authorized approver
reason
requested remedy
refund amount
return required yes/no
deadline
approval scope
customer communication
audit
```

No silent special refunds unless authority exists.

Finance hates surprise generosity. Frankly, understandable.

---

## 24. Seller / Supplier Exception Task via Broadcast/Scheduler/Reminder

Exception flow:

```text
Selene receives outside-policy request.
Selene checks product terms.
Selene creates task for correct seller/supplier authority.
Broadcast/Delivery sends request.
Scheduler sets deadline.
Reminder chases response.
Escalation triggers if overdue.
Decision recorded.
Customer updated.
Audit stored.
```

Every outside-policy action needs:

```text
owner
approver
deadline
evidence
decision
reason
audit
```

No vague “ask supplier.” Assign the human and chase their sorry little inbox.

---

## 25. Dispute Fact Pack

When customer and seller/provider disagree, 83 creates a dispute fact pack.

Fact pack includes:

```text
order details
product/service terms
terms version shown at purchase
customer request
customer evidence
seller/provider evidence
dispatch proof
delivery proof
package proof
return evidence
inspection result
warranty terms
timeline
communications
response deadlines
missed deadlines
payment/settlement state
B2B attribution
commission/customer benefit status
policy/compliance flags
```

83 may classify:

```text
customer likely correct
seller/supplier likely correct
courier likely at fault
provider likely at fault
customer abuse suspected
seller/provider abuse suspected
unclear / human review required
compliance/legal issue
```

---

## 26. Selene Services Dispute Review

If dispute cannot be resolved automatically, route to Selene Services review.

Workflow:

```text
dispute opened
fact pack generated
review owner assigned
customer/seller/provider notified
deadline set
evidence requested if missing
decision recorded
resolution executed
audit archived
```

Possible outcomes:

```text
refund
partial refund
replacement
repair
return required
return not required
provider payout hold
commission clawback
customer claim denied
seller/provider warning
abuse enforcement signal
compliance/legal escalation
```

Selene’s role is to resolve based on facts, terms, proof, and policy, not vibes. Vibes have caused enough invoices.

---

## 27. Customer / Seller / Supplier Response SLA

Every unresolved case must have deadlines.

Deadlines may include:

```text
customer evidence deadline
customer ship-by deadline
provider response deadline
seller exception approval deadline
inspection deadline
repair estimate deadline
warranty response deadline
refund decision deadline
Selene review deadline
appeal deadline
```

If deadline missed:

```text
reminder
escalation
auto-decision if policy allows
payout hold
claim closure
customer notified
provider risk score affected
supplier/seller enforcement signal
```

No infinite refund purgatory. That’s just customer support with better lighting.

---

## 28. No-Response Outcome Rules

83 must define what happens when one side does not respond.

Customer no-response may lead to:

```text
case paused
case closed
return authorization expired
refund denied
evidence request expired
pickup cancelled
```

Seller/provider no-response may lead to:

```text
auto-approval where policy allows
payout hold
reserve usage
provider risk score increase
escalation
Selene Services review
supplier suspension signal
```

No-response rules must be shown clearly in the case timeline.

---

## 29. Return Courier / Reverse Logistics

83 handles physical return movement.

Return methods:

```text
return label
QR return label
courier pickup
customer drop-off
store return
locker return
post office return
provider-direct return
B2B provider return
freight return
international return
high-value secure return
cold-chain return
repair-center return
quarantine return
```

Return logistics must respect:

```text
product type
value
fragility
temperature
destination
provider terms
customer convenience
cost
risk
compliance
```

---

## 30. Return Courier Cost Allocation

Return courier cost is mandatory.

Who pays?

```text
customer
provider
seller
Channel Store
Selene
reserve
courier/insurance claim
shared allocation
```

Depends on:

```text
provider fault
seller fault
customer change-of-mind
wrong item
damaged goods
warranty claim
delivery failure
B2B policy
brand promise
legal/compliance rule
goodwill exception
```

83 must send return courier cost to:

```text
81E B2B pricing
Payment/Settlement
Accounting
Customer explanation
Provider responsibility record
```

Return trucks do not run on customer regret alone.

---

## 31. Return Method Selection

Return options:

```text
print label
QR return label
courier pickup
drop-off point
store return
pickup locker
provider arranged pickup
customer ships back
freight return
container/freight return
repair center drop-off
```

Selection criteria:

```text
product type
location
value
fragility
cold-chain
customer convenience
cost
policy
provider route
inspection destination
security requirement
```

---

## 32. Reverse Logistics Geography

Return geography matters.

Signals:

```text
customer location
provider location
seller location
inspection center
repair center
restocking warehouse
quarantine location
international return
remote pickup
failed pickup
return duty/tax issues
local courier availability
```

Feeds:

```text
81I geography
81E B2B pricing
82 dispatch/courier handoff
Payment/Accounting
```

---

## 33. In-Transit Return / Intercept

If customer cancels while package is not delivered:

```text
intercept
reroute
hold at depot
return to sender
too late — deliver then return
```

82 owns in-transit logistics.

83 owns refund/cancellation/return consequence.

83 must check:

```text
courier intercept result
return cost
refund timing
seller/provider responsibility
customer communication
```

---

## 34. Return-to-Sender Flow

If delivery fails and package returns:

```text
customer unavailable
wrong address
recipient refused
courier failed
customs failed
safe-drop blocked
ID/signature failed
```

83 decides:

```text
refund
redelivery
charge redelivery fee
charge return courier
hold item
cancel order
ask customer
provider review
```

---

## 35. Inspection Workflow

Returned item may need inspection.

Inspection checks:

```text
received item matches RMA
correct product
correct variant
correct serial
correct batch/lot
condition
damage
used/worn
missing parts
packaging included
warranty seal
tampering
expiry/spoilage
counterfeit swap
contamination
service evidence
```

Inspection output drives refund, replacement, repair, restock, quarantine, or denial.

---

## 36. Inspection Task Assignment

Inspection is a human/system task.

Task includes:

```text
assigned inspector
location
deadline
condition checklist
photo/video proof
serial/batch scan
evidence requirements
approval route
reminder
escalation
audit
```

Uses:

```text
Task
Scheduler
Broadcast/Delivery
Reminder
Access/Authority
Audit
```

“Someone inspect it” is warehouse folklore. We retired folklore. Mostly.

---

## 37. Condition Grading

Returned goods may be graded:

```text
unopened / new
opened but unused
used acceptable
used damaged
faulty
missing parts
tampered
unsellable
expired
contaminated
fraud suspected
counterfeit suspected
quarantine required
```

Condition determines:

```text
refund amount
restocking fee
replacement eligibility
provider responsibility
inventory disposition
fraud/abuse signal
```

---

## 38. Refund Calculation

Refund may include or adjust:

```text
product price
delivery fee
tax
discount reversal
points reversal
store credit reversal
coupon effect
B2B customer benefit reversal
Channel Store commission clawback
provider payout reversal
return courier deduction
restocking fee where allowed
partial damage deduction where allowed
goodwill credit
gift-with-purchase effect
```

Refund calculation must be line-level where possible.

No whole-order blunt instruments unless the whole order truly reversed.

---

## 39. Full Refund

Full refund may apply when:

```text
provider fault
seller fault
wrong item
damaged on arrival
not as described
customer cancellation before dispatch
legal/customer right applies
service not provided
cold-chain failure
product unsafe
```

Full refund may still require:

```text
return
no return
proof
inspection
provider approval
compliance review
```

---

## 40. Partial Refund

Partial refund may apply for:

```text
partial order return
minor defect
missing accessory
customer keeps item with discount
service partially delivered
damage deduction where allowed
late delivery goodwill
price adjustment
negotiated resolution
```

Partial refund must show:

```text
amount
reason
who approved
whether return required
whether tax/commission/rewards adjust
audit reference
```

---

## 41. Refund Timing

Refund may happen:

```text
immediately
after evidence accepted
after return label scan
after courier pickup
after item received
after inspection
after provider approval
after dispute review
after warranty confirmation
after chargeback resolution
```

Customer must be told the expected timing clearly.

No “refund processing” as a black hole with a polite label.

---

## 42. Refund Destination

Refund may go to:

```text
original payment method
store credit
wallet
points
gift card
account credit
B2B customer benefit pool
manual payment route
bank transfer
```

Payment owns execution.

83 owns refund decision and instruction.

Refund destination may depend on:

```text
payment method
gift order
account customer terms
promotion terms
legal/compliance
customer choice where allowed
policy
```

---

## 43. Replacement Flow

Replacement may apply when:

```text
damaged item
wrong item
faulty item
lost delivery
warranty replacement
customer wants same item
provider offers replacement first
```

83 must create:

```text
replacement decision
replacement order request
replacement dispatch requirement
return requirement if needed
settlement hold
customer notification
provider responsibility
audit
```

Replacement dispatch goes to Document 82.

---

## 44. Exchange Flow

Exchange may apply for:

```text
wrong size
wrong color
different variant
different product
upgrade/downgrade
gift exchange
customer preference
```

83 calculates:

```text
price difference
shipping cost
return cost
tax adjustment
discount/promotion effect
inventory availability
customer payment/refund delta
```

Exchange must respect product terms and stock availability.

---

## 45. Repair Flow

Repair may apply for:

```text
warranty repair
service repair
manufacturer repair
provider repair
third-party repair
high-value repair
technical repair
```

83 tracks:

```text
repair authorization
repair destination
repair estimate
repair time
customer approval
loan/replacement item if applicable
repair completion proof
return dispatch
warranty reserve usage
```

---

## 46. Warranty Claim Flow

Warranty flow includes:

```text
warranty eligibility
warranty period
proof of purchase
serial/batch
fault evidence
provider/manufacturer responsibility
repair/replace/refund decision
warranty reserve usage
manufacturer response
brand warranty route
```

Warranty may be owned by provider/manufacturer.

83 coordinates the customer-facing workflow and settlement/reserve consequences.

---

## 47. Orphan Warranty / Provider Failure Protection

If seller/supplier/provider disappears, goes bankrupt, is suspended, refuses obligations, or cannot honor warranty, 83 must check:

```text
manufacturer warranty
brand warranty
supplier reserve
provider deposit
warranty reserve
performance reserve
insurance
Selene protection policy
B2B reserve
platform guarantee if applicable
```

Possible outcomes:

```text
Selene honors warranty from reserve
Selene refunds customer from reserve/protection fund
manufacturer handles claim
brand owner handles claim
provider payout remains held
supplier enforcement triggered
customer routed to legal/compliance path
claim denied if no coverage and law/policy allows
```

83 should support Selene honoring/refunding **where Selene policy, reserve, insurance, or guarantee allows**.

Not unlimited fairy-money refunds. Even Selene has bills. Tragic.

---

## 48. Provider Responsibility

For B2B and supplier/provider items, Original Provider may be responsible for:

```text
faulty goods
damaged goods where provider caused issue
wrong item
warranty
technical questions
replacement
return acceptance
inspection
refund approval where policy requires
safety/authenticity issue
```

Channel Store remains introducer/display channel unless contracted otherwise.

The hair salon may earn wine commission. It does not become the wine warranty tribunal.

---

## 49. Channel Store Commission Reversal

If B2B order is refunded, 83 must trigger 81E logic:

```text
reverse commission
partial commission reversal
commission clawback
hold future commission
dispute commission
do not release commission
```

Commission reversal depends on:

```text
refund amount
refund reason
fault allocation
return status
provider responsibility
customer benefit reversal
policy
```

---

## 50. Customer Benefit Reversal

Customer benefits may include:

```text
points
cashback
credit
free delivery
gift
customer benefit pool
loyalty tier benefit
coupon
future voucher
```

83 must decide:

```text
reverse
partially reverse
allow customer to keep
charge back
expire
convert to goodwill
```

If benefit was funded by B2B economics, 81E must be notified.

---

## 51. Selene Fee Treatment

Selene platform/service fee may be:

```text
kept
reversed
partially reversed
charged to provider
charged to channel
charged to reserve
waived as goodwill
```

Depends on:

```text
policy
fault
customer promise
provider responsibility
B2B economics
compliance
manual approval
```

---

## 52. Provider Payout Hold / Reversal

83 tells Payment/Settlement:

```text
hold provider payout
release provider payout
partial release
reverse provider payout
recover from reserve
recover from future payouts
block payout pending dispute
```

Payment executes.

83 decides return/refund consequence and sends signal.

---

## 53. Reserve / Deposit Usage

83 may apply:

```text
warranty reserve
performance reserve
food/safety reserve
authenticity reserve
provider deposit
high-value reserve
B2B reserve
orphan warranty reserve
```

Used for:

```text
refund
replacement
return courier
customer compensation
warranty claim
provider failure
safety/authenticity issue
chargeback protection
```

---

## 54. Courier / Insurance Claim

If courier caused issue:

```text
open claim
attach dispatch proof
attach delivery proof
attach package proof
attach damage photos
attach invoice/value
attach tracking history
track claim status
recover amount
```

83 coordinates claim consequence with:

```text
Document 82 Dispatch
Courier/Freight
Payment
Accounting
Customer Support
Order
```

---

## 55. Inventory Disposition After Return

Returned item may go to:

```text
restock as new
restock as open-box
refurbish
repair
quarantine
dispose
donate
return to provider
return to supplier
insurance claim hold
fraud evidence hold
recall hold
rework
quality review
```

Inventory owns stock truth.

83 provides disposition decision/result.

---

## 56. Quarantine Flow

Use quarantine for:

```text
food safety
contamination
suspected counterfeit
damaged goods
warranty inspection
recall
regulated goods
fraud investigation
authenticity dispute
unknown condition
```

Quarantine requires:

```text
location
owner
inspection task
release rule
disposal rule
audit
```

---

## 57. Restocking Rules

Restock only if:

```text
item matches return authorization
condition acceptable
not expired
not contaminated
not tampered
serial matches
batch/lot acceptable
packaging acceptable
legal/policy allows resale
brand allows resale
```

Restocking result must feed Inventory.

---

## 58. Restocking Fee

Where allowed, restocking fee may apply for:

```text
change of mind
opened item
custom order cancellation
bulky goods
special handling
freight return
high-value inspection
```

Restocking fee must be:

```text
disclosed before purchase/return
allowed by policy/compliance
calculated clearly
shown to customer
audited
```

No surprise restocking goblins.

---

## 59. Non-Returnable or Restricted Goods

Some goods/services may be non-returnable or restricted:

```text
perishable food
custom-made goods
hygiene-sensitive goods
digital goods
opened regulated goods
final sale clearance
personalized items
services already performed
time-sensitive tickets/events
```

Still may require remedy if:

```text
faulty
unsafe
not as described
wrong item
mandatory law/compliance applies
```

Compliance/legal decides final jurisdiction treatment where needed.

---

## 60. Food / Perishable Returns

Food/perishable issues may include:

```text
food safety issue
spoiled goods
temperature breach
wrong item
authenticity concern
allergen issue
expiry issue
poor quality
```

Often requires:

```text
photo evidence
temperature evidence
batch/expiry evidence
refund without physical return
replacement
disposal instruction
provider safety review
compliance review
quarantine if returned
```

Food may not physically return. Selene still needs proof and resolution.

---

## 61. Cold-Chain Failure

If cold-chain fails:

```text
temperature proof
courier proof
delivery timing
recipient availability
packaging proof
refund/replacement
provider/courier fault allocation
food safety quarantine/disposal
```

83 must link to:

```text
82 Dispatch proof
81I weather/geography
81H packaging/capability
Provider responsibility
Compliance if safety issue
```

---

## 62. High-Value Return

High-value returns may require:

```text
insured return courier
signature
ID check
OTP
face scan where lawful/consented/necessary
serial verification
photo/video proof
tamper-proof packaging
chain-of-custody
inspection recording
special approval
```

Use for:

```text
luxury goods
jewelry
high-value electronics
vehicles
B2B high-risk goods
official-channel goods
regulated goods
```

Biometrics require legal basis, consent where required, security, and alternatives where required.

The customer returning a watch should not feel like they are entering airport border control unless there is a reason. But sometimes there is.

---

## 63. Custom / Build-to-Order Returns

Custom goods may include:

```text
custom car options
bespoke fashion
custom jewelry
custom furniture
premium real estate finishes
engraved products
made-to-order goods
special order
```

Return/refund depends on:

```text
customer-approved configuration
production stage
fault
misbuild
customer change
cancellation point
provider terms
legal/compliance rules
resale ability
```

Custom item terms must be shown clearly before purchase.

---

## 64. Vehicle / Boat / Asset Return

For vehicles/boats/high-value assets:

```text
handover condition report
mileage/hours
damage inspection
registration status
insurance
transport/return carrier
dock/dealer return
deposit/refund rules
finance/settlement effects
warranty route
service history
```

83 may coordinate:

```text
inspection
return carrier
dealer/provider review
refund/finance adjustment
legal/compliance handoff
```

---

## 65. Real Estate Return Equivalent

A house/apartment is not returned like shoes.

Possible flows:

```text
cooling-off period
settlement cancellation
deposit forfeiture/refund
contract rescission
possession issue
completion defect process
defect claim
warranty/defects liability
settlement dispute
```

Legal/compliance owns jurisdiction.

83 coordinates workflow where commerce system needs dispute/refund/defect tracking.

No one is putting the apartment back in the box. Finally, a simple truth.

---

## 66. Service Refund / Cancellation

For services:

```text
appointment cancellation
no-show
late cancellation
service not performed
partial service performed
customer dissatisfaction
professional dispute
milestone dispute
deposit refund
reschedule
service credit
```

Refund depends on:

```text
service terms
work completed
provider fault
customer fault
appointment timing
legal/professional rules
evidence
```

---

## 67. Professional Service Dispute

For regulated/professional services:

```text
lawyers
accountants
tax agents
financial advisers
engineers
architects
migration agents
health professionals
regulated consultants
```

83 must consider:

```text
scope of work
engagement terms
professional indemnity
licence/jurisdiction
fee dispute
milestone completion
client approval
complaint process
professional body requirements
```

Route compliance/professional review where needed.

---

## 68. Subscription / Recurring Refunds

Subscription refund logic may include:

```text
proration
unused period
billing cycle
cancellation date
minimum term
trial period
recurring commission reversal
future benefit reversal
access revocation
renewal timing
```

83 must coordinate with:

```text
Payment
Order
81E recurring B2B pricing
Rewards
Accounting
```

---

## 69. Digital Goods / Non-Physical Delivery

Digital goods/services may require:

```text
access granted?
downloaded?
licence activated?
service consumed?
streamed/viewed?
fraud risk?
refund window?
access revoked?
licence revoked?
```

No reverse courier, obviously, unless someone mails the internet back.

---

## 70. Fraud / Abuse Detection

83 must detect possible abuse.

Customer-side signals:

```text
serial mismatch
empty box return
wrong item returned
counterfeit swap
used/worn abuse
wardrobing
bracketing abuse
repeat high returns
return after benefit abuse
coupon/refund abuse
chargeback abuse
fake damage claim
multi-account abuse
threatening/abusive conduct
```

Provider-side signals:

```text
repeated bad products
unsafe food
counterfeit goods
fake dispatch proof
refusing valid claims
slow response
excessive warranty failure
bad packaging causing damage
expired/unsafe goods
```

---

## 71. Customer Return Risk Score

Customer risk signals:

```text
return frequency
refund disputes
high-value return pattern
serial mismatch history
abuse indicators
chargeback history
legitimate return history
customer lifetime value
account status
evidence quality
```

Use carefully and fairly.

If sensitive/fairness risk appears, route 81G fairness review.

Risk score may affect:

```text
instant refund eligibility
proof requirement
manual review
returnless refund eligibility
temporary restriction
```

---

## 72. Provider Return Risk Score

Provider risk signals:

```text
high fault rate
late refund response
rejects valid returns
poor warranty handling
high damage rate
slow inspection
missing return address
poor cooperation
unsafe goods
counterfeit risk
expired products
frequent disputes
```

Effects:

```text
higher reserve
longer settlement hold
manual review
B2B ranking reduction
temporary suspension
product suspension
provider enforcement
```

---

## 73. Customer Abuse Enforcement Signal

83 must support graduated action for customer abuse.

Possible actions:

```text
warning
require stronger proof
disable instant refunds
manual review only
temporary return restriction
temporary Selene restriction
longer suspension
permanent ban
```

Severity levels:

```text
minor
mild
serious
severe
critical
```

Critical examples:

```text
fraud
counterfeit swaps
repeated empty returns
chargeback abuse
threatening conduct
multi-account exploitation
```

83 creates abuse signal.

A broader Participant Trust / Abuse / Enforcement engine should manage global disqualification when built.

---

## 74. Seller / Supplier Abuse Enforcement Signal

83 must support seller/provider enforcement signals.

Seller/supplier abuse may include:

```text
unsafe food
rubbish/defective products
counterfeit products
fake goods
refusing valid refunds
missing warranty obligations
slow response
false dispatch proof
wrong product shipped repeatedly
bad packaging causing damage
expired/unsafe products
abusive conduct
fraud
```

Possible actions:

```text
warning
higher reserve
longer payout hold
manual review
temporary product suspension
temporary supplier suspension
remove B2B eligibility
remove provider from recommendations
longer suspension
permanent disqualification
compliance/legal escalation
```

Severity levels:

```text
minor
mild
serious
severe
critical
```

Critical example:

```text
A supplier selling unsafe food or counterfeit goods should be removed urgently.
```

Some suppliers do not need coaching. They need a locked door.

---

## 75. Bad Actor Suspension / Disqualification Handoff

83 must send enforcement signals to future trust/enforcement systems.

Handoff includes:

```text
actor type: customer / seller / supplier / provider / channel
offence type
severity
evidence
frequency
financial impact
customer impact
safety impact
recommended action
deadline
review path
appeal path
audit
```

83 must not silently ban where governance requires review.

It must route authority through Access/Authority and Human Orchestration when required.

---

## 76. Return Communication to Customer

Customer must receive clear updates:

```text
return accepted/rejected
why
return label/QR
pickup/drop-off instructions
deadline
who pays return courier
refund timing
inspection requirement
replacement/exchange status
repair status
warranty route
appeal path
support contact
```

No “refund denied” with no reason. That’s how customers become volcanic.

---

## 77. Provider / Seller Communication

Provider/seller may receive:

```text
return request
reason
evidence
approval task
exception authorization request
inspection task
refund decision
replacement task
repair/warranty task
return shipment tracking
deadline
escalation
risk/enforcement warning
```

Communication must use:

```text
Broadcast/Delivery
Task
Scheduler
Reminder
Audit
```

---

## 78. Return Status Tracking

Return statuses include:

```text
requested
eligibility checking
terms checking
evidence requested
approved
rejected
exception approval pending
awaiting customer shipment
pickup booked
in transit to return center
received
inspection pending
inspected
refund approved
refund processing
refund sent
replacement requested
replacement dispatched
exchange completed
repair in progress
warranty review
closed
disputed
appealed
```

---

## 79. Customer Tracking for Return

Customer can see:

```text
request received
terms checked
evidence needed
return approved
label ready
pickup booked
item received
inspection started
refund approved
refund processing
replacement shipped
repair in progress
case closed
case disputed
```

Clarity reduces support tickets. Support tickets are weeds.

---

## 80. Return Deadline Management

83 tracks:

```text
label expiry
pickup deadline
customer ship-by date
evidence deadline
provider response deadline
seller exception approval deadline
inspection deadline
refund deadline
replacement deadline
repair deadline
warranty response deadline
appeal deadline
```

Uses reminders and escalation.

Deadlines apply to both sides. Customers and suppliers both enjoy vanishing when responsibility arrives.

---

## 81. Task Orchestration

Human/external tasks include:

```text
customer uploads photo
provider reviews claim
seller approves exception
warehouse inspects item
courier picks return
repair center assesses item
manager approves refund
compliance reviews regulated return
Selene Services reviews dispute
```

Each task must define:

```text
owner
recipient
deadline
confirmation
evidence
reminder
escalation
closure condition
audit
```

No “someone should review this.” Someone is a myth with a calendar invite.

---

## 82. Return Pickup Scheduling

Pickup requires:

```text
pickup address
package size
item type
pickup window
customer availability
courier selection
label/QR
proof of pickup
failed pickup handling
return destination
special handling
```

Failed pickup may lead to:

```text
reschedule
customer reminder
customer fee where policy allows
case hold
case closure after repeated failure
```

---

## 83. Return Packaging Instructions

Customer may need instructions:

```text
use original packaging
include accessories
include manuals/cards
remove personal data
pack safely
attach label
do not ship hazardous item
temperature-sensitive instructions
take photo before pickup
seal package
include RMA reference
```

Wrong return packing may affect refund or courier claim.

---

## 84. Data / Privacy Before Return

For electronics/devices, 83 must remind customer to:

```text
remove personal data
factory reset
remove SIM/card
logout accounts
disable tracking/activation lock
backup data
remove accessories not being returned
```

If customer does not do this, 83 must show risk/warning.

No one wants to return a tablet with their life still logged into it. Well, some do. Selene should stop them.

---

## 85. Return Inspection Proof

Inspection proof includes:

```text
photos
video
serial scan
batch scan
condition checklist
damage notes
missing parts
tamper evidence
timestamp
inspector identity
location
inspection result
```

This feeds:

```text
81G Audit
Inventory disposition
Payment/refund
Provider dispute
Customer dispute
Fraud/abuse
```

---

## 86. Refund Approval Authority

Some refunds require approval.

Approval-required cases:

```text
high-value refund
manual override
outside return window
no proof
fraud flag
B2B dispute
brand-sensitive item
professional service dispute
large customer goodwill credit
refund without return
refund before inspection
seller/supplier exception
```

Uses:

```text
Access/Authority
Human / External Action Orchestration
Audit
```

---

## 87. Refund Refusal

If refund is denied, Selene must explain:

```text
reason
terms/policy
evidence
condition result
appeal path
alternative resolution
warranty path if available
provider/seller decision where applicable
```

No “refund denied” as a brick wall. Customers will climb it with rage.

---

## 88. Appeal / Dispute Process

Customer, seller, provider, or Channel Store may dispute:

```text
eligibility
condition grade
refund amount
return courier charge
warranty rejection
commission clawback
provider fault
courier fault
customer abuse flag
provider abuse flag
```

83 must support:

```text
appeal intake
fact pack
new evidence
review owner
deadline
decision
final outcome
audit
```

---

## 89. Chargeback Handoff

If chargeback occurs, 83 supplies evidence to Payment.

Evidence may include:

```text
delivery proof
return policy
terms shown at purchase
customer communications
refund status
return status
proof of service
inspection result
provider response
tracking
customer evidence
```

Payment owns chargeback execution.

83 supplies return/refund facts.

---

## 90. Accounting and Tax Handoff

83 sends financial signals:

```text
refund amount
tax reversal
credit note
restocking fee
return shipping fee
provider payout reversal
commission clawback
reserve usage
inventory disposition
write-off
goodwill credit
voucher liability
store credit
```

Accounting/Tax own posting/treatment.

83 must not invent ledger entries, despite probably being tempted by all the drama.

---

## 91. AP / Supplier Credit Note Handoff

If provider/supplier owes credit:

```text
credit note requested
credit note received
statement adjusted
provider payout held
supplier payable reduced
dispute tracked
reserve applied
clawback created
```

Connects to Documents 73–75.

Supplier credit notes are where promises go to become paperwork.

---

## 92. Settlement Hold and Release

83 tells settlement:

```text
hold payment
release payment
partial release
reverse payout
clawback commission
use reserve
keep hold pending inspection
keep hold pending provider response
```

Settlement changes must include:

```text
reason
amount
party
deadline
approval
audit
```

---

## 93. Gift Return Logic

Gift returns may require special privacy and refund routing.

Options:

```text
refund to buyer
exchange for recipient
store credit to recipient
gift credit
hide price
hide buyer payment details
notify buyer or not depending policy
gift message handling
```

Gift returns can get socially weird. Selene should be less weird than the humans involved.

---

## 94. Multi-Address / Multi-Recipient Returns

One buyer may have many recipients.

Example:

```text
100 gifts sent to 100 addresses.
10 recipients request returns.
Each return has its own address, proof, reason, and resolution.
```

83 must handle:

```text
per-recipient return line
per-recipient courier cost
per-recipient refund/exchange/replacement
buyer vs recipient communication
gift privacy
batch analytics
```

---

## 95. Bundle Return Logic

Bundle return may be:

```text
full bundle return
partial bundle return
gift item kept
missing bundle component
bundle savings reversal
discount recalculation
free item return requirement
```

83 must recalculate refund fairly.

If customer keeps the “free” item, the math goblin wakes up.

---

## 96. Promotion Refund Logic

If order had promotion:

```text
discount reversal
coupon reuse decision
free delivery reversal
loyalty credit reversal
cashback reversal
gift-with-purchase return requirement
promotion eligibility recalculation
customer benefit reversal
promotion liability adjustment
```

Connects to 81F and 81G.

---

## 97. Price Adjustment / Goodwill Refund

Sometimes customer keeps item but receives:

```text
price adjustment
goodwill credit
partial refund
delivery fee refund
voucher
service recovery credit
account credit
```

Requires:

```text
reason
authority
amount
customer communication
accounting/tax signal
audit
```

---

## 98. Product Recall Flow

If recall happens:

```text
identify affected batch/serial
notify customers
stop dispatch
create return flow
refund/replacement/repair
regulatory reporting
quarantine returned stock
provider responsibility
audit
```

Recall may involve:

```text
Product
Inventory
Dispatch
Returns
Customer
Compliance
Supplier/Provider
B2B
```

---

## 99. Safety / Authenticity Claims

For food/luxury/regulated goods, 83 must handle:

```text
authenticity dispute
food safety issue
allergen issue
contamination
counterfeit claim
certificate issue
brand verification
provider investigation
```

Possible actions:

```text
quarantine
refund/replacement
brand verification
compliance escalation
supplier suspension signal
customer notification
recall investigation
```

---

## 100. Return Cost Actual vs Estimated

83 tracks cost variance.

Costs:

```text
estimated return courier
actual return courier
inspection cost
restocking cost
repair cost
replacement cost
write-off
goodwill cost
admin/support cost
B2B clawback cost
provider credit
```

Feeds:

```text
81E
81H
81I
Pricing
Accounting
Provider risk
Product improvement
```

---

## 101. Return Profitability Impact

83 reports:

```text
product margin lost
delivery cost lost
return courier cost
commission clawback
provider credit
reserve usage
write-off
restock value
customer value impact
provider risk impact
brand impact
```

This matters because returns are not just “customer got money back.” They are margin crime scenes with labels.

---

## 102. Return Reason Analytics

83 learns why returns happen.

Reasons may point to:

```text
bad sizing
bad photos
bad description
poor packaging
courier damage
wrong pick
wrong pack
provider quality
late delivery
customer misunderstanding
product defect
service failure
misleading claim
brand mismatch
```

Feeds:

```text
Product
81J Presentation
82 Dispatch
81H Capability
81F Promotions
81E B2B
Inventory
Provider score
Customer support
```

---

## 103. Return Abuse Prevention

Actions for abuse risk:

```text
require stronger proof
restrict instant refund
require inspection first
block returnless refund
manual review
limit abuse-prone offers
adjust customer risk score
temporary restriction
case escalation
```

Must be fair, auditable, and not based on protected/sensitive traits.

---

## 104. Returnless Refund

Sometimes it is cheaper or safer to refund without return.

Allowed examples:

```text
low-value item
food/perishable
damaged item
return shipping exceeds value
customer goodwill
provider fault
photo evidence sufficient
public safety issue
```

Returnless refund still requires:

```text
reason
evidence
approval if needed
inventory/accounting signal
provider responsibility
abuse check
audit
```

---

## 105. Replacement Without Return

Allowed when:

```text
food spoiled
low-value item
courier damaged item
photo evidence enough
provider authorizes
brand policy allows
return cost exceeds value
safety issue
```

Must still handle:

```text
replacement dispatch
provider responsibility
settlement hold
abuse/fraud signal
audit
```

---

## 106. Return Label Fraud Control

83 must prevent:

```text
label reuse
wrong item sent
empty parcel
return to wrong address
QR abuse
expired label use
return label resale
multiple returns against one label
```

Controls:

```text
single-use label
expiry
package weight check
tracking match
RMA match
item scan on receipt
proof required
fraud flag
```

---

## 107. Return Destination Rules

Return destination may be:

```text
warehouse
provider
supplier
repair center
inspection center
store
brand-authorized center
freight depot
quarantine center
recycling/disposal center
manufacturer
```

Selene selects based on:

```text
product
condition
reason
provider terms
brand rules
warranty route
geography
cost
risk
compliance
```

---

## 108. Customer Self-Serve Return

Self-serve return may be allowed if:

```text
eligible
low-risk
within window
standard reason
standard product
no high-value flag
no fraud flag
no compliance issue
terms clear
evidence not required or simple
```

Otherwise:

```text
manual review
provider approval
Selene Services review
compliance review
```

Self-serve is lovely until the returned box contains a brick. Guardrails, darling.

---

## 109. Provider Approval SLA

Provider must respond within defined timeframe.

SLA source:

```text
product terms
B2B terms
warranty terms
provider contract
legal/compliance requirement
Selene policy
```

If provider misses SLA:

```text
auto-approve where policy allows
escalate
hold payout
use reserve
notify management
increase provider risk score
trigger enforcement signal
```

---

## 110. Return Dashboard

83 dashboard should show:

```text
new requests
terms checking
awaiting evidence
awaiting customer
awaiting provider
exception approval pending
pickup scheduled
in transit returns
awaiting inspection
refund pending
replacement pending
repair pending
warranty pending
high-value review
fraud flags
overdue SLAs
provider no-response
appeals
disputes
```

---

## 111. Monitoring

83 monitors:

```text
return rate
refund rate
reason mix
provider response time
inspection time
refund time
return courier cost
return abuse
provider abuse
customer satisfaction
product defect trend
presentation-caused returns
dispatch-caused returns
promotion-caused returns
warranty claim rate
```

---

## 112. Learning Loop

83 learns:

```text
which products return too often
which providers cause issues
which couriers damage goods
which descriptions mislead
which packaging fails
which customers abuse returns
which policies are too generous
which policies are too strict
which terms cause disputes
which suppliers need higher reserves
which items need better presentation
which dispatch locations mispack
```

Feeds:

```text
Product
81J Presentation
82 Dispatch
81H Capability
81I Geography
81E B2B Pricing
81F Promotion Testing
81G Audit
Inventory
Provider Score
Customer Trust/Abuse
Accounting/Cashflow
```

---

## 113. Outputs from Document 83

83 outputs:

```text
ReturnRequest
RefundRequest
TermsResolverResult
TermsDisplayEvidence
ReturnEligibilityResult
RefundEligibilityResult
ExceptionAuthorizationTask
DisputeFactPack
ReturnAuthorization
ReturnLabel
ReturnCourierBooking
ReturnPickupStatus
InspectionTask
InspectionResult
ConditionGrade
RefundDecision
RefundInstruction
PartialRefundInstruction
ReplacementOrderRequest
ExchangeOrderRequest
RepairRequest
WarrantyClaim
ProviderResponsibilitySignal
CustomerResponsibilitySignal
CourierClaimSignal
CommissionClawbackSignal
CustomerBenefitReversal
ProviderPayoutHold
ReserveUsageSignal
InventoryDisposition
QuarantineSignal
AbuseRiskSignal
EnforcementHandoff
ReturnAuditEvidence
```

---

## 114. State Machines

### Return Request State

```text
NotStarted
Requested
TermsChecking
EvidenceRequired
EligibilityChecking
Approved
Rejected
ExceptionApprovalRequired
Disputed
Closed
```

### Return Logistics State

```text
NotRequired
AuthorizationCreated
LabelCreated
PickupScheduled
AwaitingCustomerShipment
InTransitToReturnLocation
Received
PickupFailed
LostInReturnTransit
Closed
```

### Inspection State

```text
NotRequired
Pending
Assigned
Inspecting
EvidenceCaptured
ConditionGraded
ApprovedForRefund
Rejected
QuarantineRequired
FraudSuspected
Closed
```

### Refund State

```text
NotRequested
EligibilityChecking
PendingReturn
PendingInspection
PendingProviderApproval
Approved
PartiallyApproved
Rejected
InstructionSentToPayment
Processing
Completed
Failed
Closed
```

### Replacement / Exchange State

```text
NotRequested
Requested
EligibilityChecking
StockChecking
PriceDeltaChecking
Approved
ReplacementOrderCreated
ExchangeOrderCreated
Dispatched
Completed
Rejected
Closed
```

### Warranty State

```text
NotRequired
ClaimOpened
EligibilityChecking
ProviderReview
ManufacturerReview
RepairApproved
ReplacementApproved
RefundApproved
Rejected
Closed
```

### Dispute State

```text
NoDispute
DisputeOpened
FactPackCreated
EvidenceRequested
SeleneReviewAssigned
DecisionPending
Resolved
Appealed
EscalatedToCompliance
Closed
```

### Enforcement State

```text
NoSignal
SignalDetected
SeverityClassified
WarningRecommended
RestrictionRecommended
SuspensionRecommended
DisqualificationRecommended
ReviewRequired
Actioned
Closed
```

---

## 115. Reason Codes

```text
RETURN_REQUEST_RECEIVED
REFUND_REQUEST_RECEIVED
CANCELLATION_REQUEST_RECEIVED
PRODUCT_TERMS_LOADED
TERMS_DISPLAY_EVIDENCE_FOUND
TERMS_DISPLAY_EVIDENCE_MISSING
MANDATORY_COMPLIANCE_REVIEW_REQUIRED
RETURN_ELIGIBLE
RETURN_NOT_ELIGIBLE
REFUND_ELIGIBLE
REFUND_NOT_ELIGIBLE
EXCEPTION_AUTHORIZATION_REQUIRED
SELLER_SUPPLIER_EXCEPTION_TASK_CREATED
CUSTOMER_EVIDENCE_REQUIRED
PROVIDER_EVIDENCE_REQUIRED
RMA_CREATED
RETURN_LABEL_CREATED
RETURN_COURIER_COST_ALLOCATED_TO_CUSTOMER
RETURN_COURIER_COST_ALLOCATED_TO_PROVIDER
RETURN_COURIER_COST_ALLOCATED_TO_RESERVE
RETURN_PICKUP_BOOKED
RETURN_RECEIVED
INSPECTION_REQUIRED
INSPECTION_COMPLETED
CONDITION_GRADED_NEW
CONDITION_GRADED_DAMAGED
CONDITION_GRADED_TAMPERED
FRAUD_SUSPECTED
FULL_REFUND_APPROVED
PARTIAL_REFUND_APPROVED
REFUND_REJECTED
REPLACEMENT_APPROVED
EXCHANGE_APPROVED
REPAIR_APPROVED
WARRANTY_CLAIM_OPENED
ORPHAN_WARRANTY_PROTECTION_REQUIRED
PROVIDER_RESPONSIBLE
CUSTOMER_RESPONSIBLE
COURIER_RESPONSIBLE
COMMISSION_CLAWBACK_REQUIRED
CUSTOMER_BENEFIT_REVERSAL_REQUIRED
PROVIDER_PAYOUT_HOLD_REQUIRED
RESERVE_USAGE_REQUIRED
INVENTORY_RESTOCK_AS_NEW
INVENTORY_RESTOCK_OPEN_BOX
INVENTORY_QUARANTINE_REQUIRED
RETURNLESS_REFUND_APPROVED
REPLACEMENT_WITHOUT_RETURN_APPROVED
DISPUTE_FACT_PACK_CREATED
SELENE_SERVICES_REVIEW_REQUIRED
CUSTOMER_ABUSE_SIGNAL_CREATED
PROVIDER_ABUSE_SIGNAL_CREATED
ENFORCEMENT_HANDOFF_CREATED
RETURN_AUDIT_CAPTURED
```

---

## 116. Required Simulations

```text
customer requests return and product terms allow 14-day unused return
customer requests opened-item refund but product terms say no refund after opening unless faulty
terms display evidence proves no-return condition was shown at purchase
seller/supplier exception approval task created for outside-policy refund
supplier fails to respond by SLA and payout hold is applied
customer uploads photo evidence of damaged goods
provider uploads dispatch/packing evidence
dispute fact pack shows supplier was at fault
Selene Services review resolves customer/supplier dispute
7-day satisfaction hold blocks provider payout pending return request
return courier cost allocated to customer for change-of-mind
return courier cost allocated to provider for wrong item
food safety claim triggers refund without physical return and provider review
cold-chain failure uses dispatch temperature proof
high-value return requires serial, photo, ID/OTP proof
custom-built item cancellation after production requires provider approval
vehicle return uses handover condition report and mileage
real estate defect claim routes legal/compliance workflow
digital good refund checks access/download/licence state
B2B refund reverses Channel Store commission and customer benefit
customer benefit kept as goodwill after approval
provider payout reversed from reserve after valid warranty claim
return inspection grades item as open-box and Inventory restock signal sent
counterfeit swap suspected and fraud review opens
customer repeat abuse triggers manual-review restriction
supplier unsafe food issue triggers severe enforcement handoff
product recall creates affected batch return workflow
promotion refund reverses cashback and coupon eligibility
gift recipient receives exchange credit without buyer price exposure
100 gift recipients create 10 separate return lines
return label fraud detected from duplicate label use
returnless refund approved because return shipping exceeds product value
```

---

## 117. Integration Map

```text
PH1.RETURNS / DOCUMENT_83
↔ PH1.ORDER / DOCUMENT_80
↔ PH1.DISPATCH / DOCUMENT_82
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.B2B_PLATFORM / DOCUMENT_78
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.ECOMMERCE / DOCUMENT_77
↔ PH1.POS / DOCUMENT_79
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.AP_CREDITORS / DOCUMENT_73
↔ PH1.SUPPLIER_PAYMENT_HANDOFF / DOCUMENT_74
↔ PH1.SUPPLIER_STATEMENT_RECONCILIATION / DOCUMENT_75
↔ PH1.COURIER / DELIVERY_PROVIDER
↔ PH1.WARRANTY
↔ PH1.CUSTOMER
↔ PH1.SUPPLIER / PROVIDER
↔ PH1.BRAND_OWNER
↔ PH1.LEGAL
↔ PH1.COMPLIANCE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.TASK / HUMAN_WORKLOAD
↔ PH1.SCHEDULER / ROSTERS
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
↔ FUTURE PH1.PARTICIPANT_TRUST_ABUSE_ENFORCEMENT
```

---

## 118. Required Logical Packets

```text
ReturnRequestPacket
RefundRequestPacket
CancellationRequestPacket
ProductAfterSaleTermsPacket
TermsDisplayEvidencePacket
TermsResolverPacket
CustomerSafeTermsSummaryPacket
ReturnEligibilityPacket
RefundEligibilityPacket
ExceptionAuthorizationPacket
DisputeFactPackPacket
ReturnAuthorizationPacket
ReturnLabelPacket
ReturnCourierBookingPacket
ReturnCourierCostAllocationPacket
CustomerEvidencePacket
ProviderEvidencePacket
InspectionTaskPacket
InspectionResultPacket
ConditionGradePacket
RefundCalculationPacket
RefundInstructionPacket
PartialRefundPacket
ReplacementRequestPacket
ExchangeRequestPacket
RepairRequestPacket
WarrantyClaimPacket
OrphanWarrantyProtectionPacket
ProviderResponsibilityPacket
CustomerResponsibilityPacket
CourierResponsibilityPacket
CommissionClawbackPacket
CustomerBenefitReversalPacket
ProviderPayoutHoldPacket
ReserveUsagePacket
CourierClaimPacket
InventoryDispositionPacket
QuarantinePacket
RestockingPacket
ReturnlessRefundPacket
ReturnFraudRiskPacket
CustomerAbuseSignalPacket
ProviderAbuseSignalPacket
EnforcementHandoffPacket
ChargebackEvidencePacket
AccountingTaxReturnSignalPacket
APCreditNoteSignalPacket
SettlementHoldReleasePacket
ReturnStatusPacket
ReturnSLADeadlinePacket
ReturnAuditEvidencePacket
```

Logical only.

No runtime packet structs. The refund goblin may sit quietly in policy review.

---

## 119. What Codex Must Not Do

```text
Do not make Document 83 own original Order.
Do not make Document 83 own outbound Dispatch.
Do not make Document 83 own Product master truth.
Do not make Document 83 own Inventory stock truth.
Do not make Document 83 own Payment execution.
Do not make Document 83 own Accounting ledger posting.
Do not make Document 83 own Tax law.
Do not make Document 83 own Legal/Compliance final interpretation.
Do not process returns/refunds without checking product-level seller/supplier terms.
Do not ignore the terms shown to customer at purchase.
Do not ignore mandatory compliance/legal override.
Do not allow outside-policy refunds without seller/supplier authority where required.
Do not allow unresolved customer/seller disputes without fact pack and escalation route.
Do not ignore return courier cost.
Do not release B2B provider payout or channel commission when return/refund/dispute hold applies.
Do not ignore commission/customer benefit/provider payout reversal.
Do not ignore orphan warranty/provider failure protection where policy/reserve applies.
Do not ignore customer or provider abuse signals.
Do not suspend or disqualify actors without governed evidence and authority.
Do not use vague human/provider tasks without Human / External Action Orchestration.
Do not let GPT-5.5 invent terms, legal rights, refund approvals, warranty coverage, evidence, courier status, provider responsibility, or abuse conclusions.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 120. Final Architecture Sentence

Selene Returns, Refunds + Reverse Logistics Engine is the after-sale resolution layer that receives cancellation, return, refund, exchange, replacement, repair, warranty, and dispute requests; checks product-level seller/supplier terms, terms shown at purchase, mandatory compliance/legal overrides, delivery proof, product evidence, provider evidence, B2B responsibility, return courier cost, settlement holds, provider reserves, and abuse signals; then controls return authorization, reverse logistics, inspection, condition grading, refund calculation, replacement/exchange/repair flow, warranty routing, commission/customer benefit/payout reversals, dispute fact packs, exception approvals, customer/provider deadlines, enforcement handoffs, inventory disposition, accounting/tax signals, and audit evidence until the case is resolved, rejected, escalated, refunded, replaced, repaired, closed, or enforced.

Simple version:

```text
83 decides what happens when the customer does not keep the original order exactly as delivered.

It checks:
what terms applied
what customer was shown
what law/compliance may override
what evidence exists
who is responsible
who pays return courier
whether refund/return/replacement/repair/warranty is allowed
what money/commission/benefits reverse
whether abuse exists
who must decide
what deadline applies
what proof is stored

83 does not just “process returns.”
It governs after-sale resolution.
```

That is Document 83: the engine that stops returns from becoming “customer says bad, supplier says no, courier says maybe, finance says who paid whom, and everyone points at Dave again.”

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
