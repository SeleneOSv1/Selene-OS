# Global Document 79 — Selene POS + Commerce Execution Engine

```text id="doc79_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
79

ENGINE:
PH1.POS / PH1.POS.COMMERCE / PH1.PHYSICAL_COMMERCE_EXECUTION

FULL NAME:
Selene Point of Sale, Customer-Carried POS, Scan-As-You-Shop, Private Bill, Tender Optimization, Store Checkout, Restaurant Split Bill, Barcode/QR/RFID/NFC, Scale/Label Interface, B2B-Aware Sale, Returns Intake, Payment Recommendation, Staff Authority, Offline Sync, and Physical Commerce Execution Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene POS is the **physical commerce execution surface** of Selene.

It is not just a cash register.

It is not just:

```text id="old_pos"
scan item
show total
take payment
print receipt
open drawer
goodbye, tiny consumer
```

That is old POS. A calculator with a drawer and a superiority complex.

Selene POS is the real-time commerce intelligence layer for physical stores, restaurants, salons, supermarkets, warehouses, events, service counters, mobile sellers, self-checkout, customer phones, and staff devices.

Selene POS must support:

```text id="pos_supports"
customer-carried POS
scan-as-you-shop
live cart
private customer bill
best payment recommendation
points / credits / rewards
store account / credit terms
interest-free payment options
split payments
cashier checkout
self-checkout
staff mobile checkout
restaurant table bills
bill splitting
barcode / QR / RFID / NFC scanning
weighing and label interfaces
B2B-sourced item recognition
original provider routing
channel store handoff
returns intake
refund initiation
receipt / warranty proof
inventory movement signals
JIT reorder signals
discount and campaign execution
fraud / shrinkage signals
staff authority and overrides
offline / degraded mode
audit evidence
```

Selene POS should make the customer feel in control before reaching the cashier.

Customer can scan products as they shop, see the live bill, receive offer prompts, choose payment privately, and pay on their device. Store POS receives confirmation.

The customer can become the POS as they shop.

The store POS becomes confirmation, support, audit, exception handling, and official commerce execution.

That is the future. The cash drawer may need a moment.

---

## 2. Core POS Law

```text id="core_pos_law"
POS is the physical commerce execution layer.
POS must know store context.
POS must identify customer context where permitted.
POS must show the customer the best available payment and benefit options privately where possible.
POS must execute pricing, discount, loyalty, B2B, inventory, payment, tax, return, and audit handoffs without owning those engines.
POS must support customer-carried checkout and store-assisted checkout.
POS must preserve privacy at the counter.
POS must never invent product, price, stock, discount, tax, payment, reward, or settlement truth.
Protected actions require authority or verification.
Everything important is audited.
```

Selene POS must not be a dumb terminal.

It must be:

```text id="pos_identity"
Point of Sale
Point of Service
Point of Settlement Intent
Point of Stock Movement
Point of Customer Trust
Point of Tender Optimization
Point of Physical Commerce Truth
```

Yes, still POS. We just gave it a brain. Retail was not ready, but retail rarely is.

---

## 3. Engine Ownership Boundary

### 3.1 POS owns

```text id="pos_owns"
physical sale execution surface
cashier checkout session
customer-carried scan-as-you-shop session
self-checkout session
mobile staff checkout
restaurant table bill session
customer phone bill presentation
bill verification between customer cart and store cart
payment option presentation
tender recommendation display
barcode / QR / RFID / NFC scan event intake
scale / label event intake
store context capture
customer identity capture at POS
receipt / tax invoice presentation
cash drawer / terminal / printer interface request
staff authority prompt
manager override request
returns intake at physical store
exchange intake at physical store
B2B-sourced item recognition request
order handoff to Order Management
payment handoff to Payment / Settlement
discount validation request to Pricing
promotion/cross-sell request to Marketing
loyalty/points request to Rewards
stock movement request to Inventory
audit events
```

### 3.2 POS references but does not own

```text id="pos_not_own"
product master truth
inventory stock truth
pricing/discount truth
campaign truth
points/reward balances
customer credit underwriting
payment authorization/capture
settlement release
B2B original provider responsibility
B2B channel commission
B2B provider payout
B2B reserves/deposits
bulk breakdown/yield/waste/repack truth
legal-for-trade scale compliance
warehouse operations
dispatch operations
return logistics
warranty execution
tax treatment
ledger posting
customer memory master truth
```

### 3.3 Correct owner split

```text id="pos_owner_split"
PH1.POS = physical commerce execution and customer/store checkout surface.
PH1.ECOMMERCE = online/app/personal commerce experience.
PH1.B2B_PLATFORM = Original Provider, Channel Store, commission, provider payout, reserves, B2B settlement rules.
PH1.PRODUCT = product/service truth and media.
PH1.INVENTORY = stock truth, stock movement, JIT.
PH1.PRICING = price, discount, margin, offer validity.
PH1.MARKETING = campaigns, cross-sell strategy, customer segmentation.
PH1.REWARDS = points, loyalty, customer benefit pool redemption.
PH1.CUSTOMER_CREDIT / WALLET = credit, wallet, installment, store account balances.
PH1.PAYMENT / SETTLEMENT = payment authorization, capture, refund, settlement.
PH1.WEIGHING_SCALE_LABEL = legal-for-trade weight capture and label printing.
PH1.BULK_BREAKDOWN = parent-lot to child-pack transformation, yield, waste, repack.
PH1.ORDER = order orchestration.
PH1.DISPATCH = picking, packing, courier handoff.
PH1.RETURNS = return logistics and refund execution.
PH1.ACCOUNTING = ledger posting.
PH1.TAX = tax treatment.
PH1.AUDIT = proof.
```

This split matters. Otherwise POS starts thinking it owns bananas, credit cards, wine commissions, beef trim, and tax law. Bold. Horrible.

---

## 4. Relationship to Documents 77 and 78

Document 79 sits after:

```text id="doc_sequence"
77 — E-Commerce / Personal Commerce / Company Store Commerce
78 — B2B Platform + Trade Ecosystem
```

POS must obey both.

### From Document 77, POS inherits

```text id="pos_inherits_77"
customer identity model
company store context
personal Selene context
customer privacy rules
typed / voice / device interaction rules
customer-facing payment permission
recipient and delivery context
unified buying list interaction
conversation-to-action guardrail
```

### From Document 78, POS inherits

```text id="pos_inherits_78"
B2B-sourced item recognition
Original Provider of Record
Channel Store attribution
provider responsibility
B2B settlement overlay
provider payout hold
channel commission handoff
provider reserves/deposits
professional-service compliance status
```

POS must not duplicate B2B mechanics.

If an item is B2B-sourced, POS requests B2B context and sends the sale handoff to Document 78.

POS is the counter.

B2B is the marketplace machinery.

Different beasts. Same circus.

---

## 5. Store Context Detection

Selene POS must know which store, branch, table, counter, cart, or checkout context the customer is in.

Store context can be determined by multiple signals.

### 5.1 Highest-confidence context

```text id="highest_confidence_context"
POS sends bill/session to customer
cashier scans customer Selene QR
customer scans checkout QR
customer scans table QR
customer scans cart QR
customer taps NFC counter/table/cart tag
customer opens bill from merchant link
```

### 5.2 Medium-confidence context

```text id="medium_confidence_context"
Bluetooth beacon / iBeacon
geofence
appointment context
order context
store Wi-Fi hint
recent store visit
calendar/booking context
```

### 5.3 Context confidence ladder

```text id="store_context_ladder"
Level 5 — Explicit POS bill/session link
Level 4 — Explicit QR/NFC store, table, counter, cart, or checkout confirmation
Level 3 — Strong indoor beacon + matching store
Level 2 — Broad geofence/location/store proximity
Level 1 — Weak context from Wi-Fi/recent habit/calendar
Level 0 — Unknown
```

Rule:

```text id="store_context_rule"
Protected actions and payments require Level 4 or Level 5 context.
Browsing and offer suggestions may use lower confidence with customer confirmation.
```

Examples:

```text id="store_context_examples"
Hair salon:
staff sends bill to customer phone or customer taps counter NFC.

Restaurant:
customer scans table QR or taps table NFC.

Supermarket:
customer scans store/cart QR at entrance.

Pop-up stall:
customer scans seller QR.
```

Selene should guess where the customer is, then confirm before money moves.

Magical enough to impress. Deterministic enough to avoid paying Frank’s Restaurant while standing inside Harry’s Supermarket.

---

## 6. Customer-Carried POS + Scan-As-You-Shop

Selene POS must allow the customer’s phone to become a private live POS companion.

Customer can:

```text id="scan_as_you_shop_customer_can"
scan products while shopping
see running total
see discounts as they qualify
see promotions at product point
see points/credits available
see payment options privately
check cashier/store total against their own live cart
pay on their device
receive digital receipt
trigger paid confirmation to store POS
```

Example:

```text id="scan_as_you_shop_example"
Customer shops with kids.
Customer tells kids to scan each product before placing it in the cart.
Selene builds live cart.
Selene shows total, offers, missed discounts, payment options, and checkout readiness.
```

This creates:

```text id="scan_as_you_shop_benefits"
budget control
bill visibility before cashier
private payment decision
promotion awareness at shelf
checkout speed
overcharge detection
better customer trust
```

Customer-carried POS is optional.

```text id="optional_scan"
Customer may use normal cashier checkout.
Customer may use self-checkout.
Customer may use scan-as-you-shop.
Customer may use assisted checkout.
```

Do not force innovation onto someone buying bananas. They came for potassium, not onboarding trauma.

---

## 7. Live Cart, Running Total + Bill Verification

The customer’s live cart must show:

```text id="live_cart_shows"
items scanned
quantity
weight where applicable
unit price
discounts
bundle progress
tax estimate where applicable
running total
points/rewards available
best payment options
B2B/provider disclosure if required
return/warranty flags
```

At checkout, Selene compares:

```text id="bill_verification_compare"
customer live cart
cashier/store POS cart
self-checkout scan
scale/cart weight verification where applicable
```

If mismatch:

```text id="bill_mismatch"
Selene shows difference.
Selene routes correction.
Selene prevents payment until resolved if material.
```

Examples:

```text id="bill_mismatch_examples"
“Your app cart shows $86.40. Store scan shows $91.20. Difference: toothpaste bundle not applied.”

“Your cart has 3 bananas. Store scale label has 2.1kg bananas. Confirm?”
```

This protects the customer from overcharging and helps the store avoid disputes.

Old checkout makes the customer stare suspiciously at a display. Selene lets them know before the register ambush.

---

## 8. Product-Point Offer Awareness

Selene POS must show offers while the customer can still act on them.

Examples:

```text id="product_point_examples"
Customer scans shampoo:
“Add two toothpaste tubes to qualify for the bundle.”

Customer scans toothpaste:
“Buy 4 and get 1 free.”

Customer scans pasta:
“This sauce has a bundle discount.”

Customer scans shoes:
“Add the shoe-care kit and save $6.”
```

Customer can:

```text id="offer_customer_actions"
add offer
ignore
save for later
ask why
compare alternative
```

Ownership split:

```text id="offer_owner_split"
Marketing = campaign and cross-sell strategy.
Pricing = discount validity, margin guardrail, offer rules.
Inventory = availability.
Rewards = customer-specific benefits.
POS = live product-point presentation and execution.
```

This is one of Selene POS’s biggest advantages: it helps the customer before they leave the shelf.

Normal POS tells you at the end that you missed the deal. Selene tells you while the toothpaste is still within arm’s reach. Civilization advances, slightly.

---

## 9. Tender Optimization + Best Payment Recommendation

Selene POS must always evaluate the best available payment methods for the customer, privately where possible.

Payment options may include:

```text id="tender_options"
points
reward credits
store credit
gift cards
customer benefit pool
cashback balance
expiring credits
Selene wallet
customer credit
installment plan
interest-free period
credit card
debit card
bank payment
cash
store account
trade account
split tender
```

Selene may suggest:

```text id="tender_suggestions"
“You can cover this with points.”
“You have $18 credit expiring this week.”
“You can split this between points and card.”
“You can pay interest-free over 4 weeks.”
“You’ll save more if you use your store credit first.”
```

Privacy rule:

```text id="tender_privacy"
Customer-private payment recommendations should appear on customer phone, customer display, or secure payment terminal where possible.
Staff should see only what the customer permits or what store operations require.
```

Cashier should not loudly learn that the customer is using points, credit, or an installment plan.

Humans are weird about money. Selene should be civilized about it.

---

## 10. Payment, Wallet, Credit + Settlement Handoff

POS presents and captures payment intent.

Payment/Settlement executes.

POS may support:

```text id="pos_payment_methods"
cash
card terminal
wallet
QR payment
Tap to Pay / SoftPOS where available
Selene wallet later
points
gift card
store credit
reward credit
customer account terms
interest-free installment
company account
split tender
```

Payment card handling must respect payment security requirements. PCI DSS defines security requirements for environments where payment account data is stored, processed, or transmitted, so POS must use appropriate tokenized/provider-led flows rather than casual raw-card handling. ([PCI Security Standards Council][1])

SoftPOS/mobile checkout should be supported where available. Apple describes Tap to Pay on iPhone as letting payment apps accept contactless payments on iPhone without extra terminals or hardware. ([Apple Developer][2])

POS sends:

```text id="payment_handoff"
payment_intent
customer_id
store_id
cart_id
amount
tender_options_selected
verification_ref
B2B_context_if_any
audit_ref
```

Payment returns:

```text id="payment_return"
authorized
captured
failed
requires_step_up
requires_alternative_tender
refund_available
audit_ref
```

POS does not become a bank.

We already have enough little monsters.

---

## 11. Barcode / QR / 2D / RFID / NFC Scan Layer

POS must support modern item, customer, receipt, return, and store identification.

Supported scan/input technologies:

```text id="scan_technologies"
1D barcode
2D barcode
QR code
GS1 Digital Link
DataMatrix
RFID
NFC
customer QR
receipt QR
return QR
warranty QR
loyalty QR
table QR
cart QR
checkout QR
product passport QR
```

Retail POS should be ready for 2D barcode standards. GS1 states that all retail POS systems should be capable of reading and processing defined 2D barcodes with GS1 standards in addition to existing linear barcodes, and GS1’s retail guidance covers 2D barcode criteria for items scanned at POS. ([GS1][3])

2D/QR-based product scanning may unlock:

```text id="2d_unlocks"
product detail
batch
expiry
recall status
allergens
origin
warranty
instructions
care guide
videos
returns
product passport
cross-sell
```

Old barcode says “item 123.”

Selene scan says “item 123, batch 9, expires Friday, has a bundle offer, and your kid just scanned it twice.”

---

## 12. Equipment + Device Adapter Layer

POS must normalize events from physical equipment.

Equipment includes:

```text id="pos_equipment"
barcode scanner
2D scanner
RFID reader
NFC reader
payment terminal
SoftPOS device
cash drawer
receipt printer
label printer
customer display
kitchen display
scale
self-checkout kiosk
staff tablet
mobile POS phone
warehouse scanner
```

POS should receive normalized events:

```text id="device_events"
ProductScanned
CustomerIdentified
StoreContextConfirmed
WeightCaptured
LabelScanned
CartQRCodeScanned
PaymentAuthorized
PaymentFailed
CashDrawerOpened
ReceiptPrinted
ManagerOverrideRequested
ReturnQRCodeScanned
```

POS should not care what brand of scanner or scale is attached.

POS cares about the trusted event.

The hardware can have its little personality crisis elsewhere.

---

## 13. Scale, Weight + Label Interface

POS must integrate with weighing/label systems but must not own trade-measurement compliance.

Future owner:

```text id="scale_engine"
Selene Weighing, Scale, Label Printing + Trade Measurement Compliance Engine
```

POS receives:

```text id="scale_pos_receives"
product_id
weight
price_per_unit
calculated price
tare
batch
expiry
label_id
barcode/QR
scale_id
operator_id
audit_ref
```

Use cases:

```text id="scale_use_cases"
bananas
beef
ham
cheese
fish
bulk nuts
loose vegetables
deli products
prepacked meat
```

Scale modes:

```text id="scale_modes"
customer self-weigh
staff weigh and label
checkout scale verification
prepack label scan
```

Scales used in trade must meet applicable measurement requirements. NIST Handbook 44 covers specifications, tolerances, and technical requirements for weighing and measuring devices. ([NIST][4])

POS scans the label or receives weight event.

Scale/Label engine owns compliance.

This avoids POS becoming a legal-for-trade scale philosopher. Nobody wants that at checkout.

---

## 14. Bulk Breakdown / Repack Handoff

POS must support products created from bulk breakdown, but does not own transformation lineage.

Future owner:

```text id="bulk_engine"
Selene Bulk Breakdown, Yield, Waste, Repack + Packaged Goods Transformation Engine
```

Example:

```text id="bulk_example"
Store receives 100kg beef.
Only 50kg is cut today.
Remaining 50kg stays in parent lot.
Later, more cuts happen from same parent lot.
Each retail pack must trace back to parent lot and cutting session.
```

Bulk Breakdown owns:

```text id="bulk_owns"
parent lot
original received weight
remaining balance
cutting sessions
operator
date/time
child packs
pack weights
waste
trim
yield
spoilage
cost allocation
markup
batch
expiry
label
COGS handoff
```

POS only sees final sellable pack:

```text id="pos_sees_pack"
pack_id
barcode/QR
weight
price
batch
expiry
parent_lot_ref
audit_ref
```

When POS scans the pack, Selene can trace:

```text id="trace_example"
This pack came from Beef Lot BEEF-0001,
cut in Session CUT-0003,
with this cost allocation, expiry, and batch.
```

POS sells the steak.

Bulk Breakdown remembers the cow’s accounting journey.

Healthy boundary. Slightly disturbing sentence.

---

## 15. Inventory, JIT + Stock Movement Signal Layer

POS must send stock movement signals to Inventory.

Inventory owns stock truth.

POS sends:

```text id="inventory_signals"
item sold
quantity sold
weight sold
returned item
damaged return
exchange
void
waste signal if recorded at POS
store transfer scan if applicable
stockout at checkout
substitution accepted
```

Inventory may return:

```text id="inventory_returns"
available stock
reserved stock
low stock
JIT reorder trigger
stockout warning
expiry warning
recall warning
substitution suggestion
```

JIT belongs to Inventory/Replenishment.

POS is the real-time signal source.

The cashier scans. Inventory learns. The shelf stops lying. In theory.

---

## 16. Pricing, Discount + Margin Guardrail Handoff

Discounts are owned by:

```text id="discount_owner"
Document 81 — Pricing, Margin, Discount + Offer Governance Engine
```

POS must request and apply valid pricing.

POS asks Pricing:

```text id="pricing_request"
base price
store price
customer price
promotion eligibility
bundle eligibility
points/coupon compatibility
margin guardrail
manager approval requirement
stacking rules
expiry
channel/store/POS eligibility
```

Pricing returns:

```text id="pricing_response"
approved price
discounts applied
discounts available
bundle progress
margin warning
approval required
reason codes
audit_ref
```

POS may show:

```text id="discount_phrases"
“$8 discount applied.”
“Buy one more to unlock the bundle.”
“This discount requires manager approval.”
“This offer is not valid at this store.”
```

POS executes discounts.

Pricing owns discounts.

Marketing may create the campaign.

Rewards may provide points.

POS is not a discount casino.

---

## 17. Marketing Cross-Sell / Bundle Execution Layer

Marketing owns strategy.

POS executes live offer presentation.

Cross-sell examples:

```text id="cross_sell_examples"
cake → candles / cards / flowers
shoes → socks / shoe-care kit
pasta → sauce / wine
printer → paper / toner
shampoo → conditioner / toothpaste bundle
```

POS must check:

```text id="cross_sell_checks"
customer eligibility
inventory availability
pricing validity
margin rules
store policy
B2B eligibility
customer preference
privacy settings
```

POS should not push useless offers.

Selene is not a desperate kiosk employee with a headset.

---

## 18. B2B-Sourced Item Recognition

POS must identify when an item is B2B-sourced.

If item is B2B-sourced, POS requests B2B context.

B2B returns:

```text id="b2b_context"
Original Provider of Record
Channel Store if applicable
B2B settlement required
provider support route
customer-visible return/warranty summary
provider disclosure if required
commission/settlement flags for backend
```

POS must not show backend profit share unless required.

Example:

```text id="b2b_pos_example"
Hair Salon sells Wine Store wine at POS.
POS scans wine.
B2B identifies Wine Store as Original Provider.
Hair Salon remains Channel Store.
Selene creates B2B settlement context.
Customer pays.
Provider fulfillment/support responsibility stays with Wine Store.
```

POS executes the physical sale.

B2B owns the commercial machinery.

Retail survives another day.

---

## 19. Original Provider / Channel Store Routing Handoff

POS must preserve the right routing.

If customer later asks:

```text id="provider_later_questions"
“What other wine is like this?”
“What warranty applies?”
“This product is faulty.”
“Can I return this?”
```

Selene routes:

```text id="routing_handoff"
product/service question → Original Provider
delivery status → Order / Dispatch / Delivery
return request → Returns
warranty claim → Warranty / Original Provider
commission/settlement → B2B
```

POS receipt must include enough digital evidence to route later.

Customer does not need to know the plumbing.

Selene knows.

Selene always knows. Mildly spooky, commercially useful.

---

## 20. Staff POS + Assisted Checkout

Staff POS supports:

```text id="staff_pos_supports"
scan item
lookup customer
create sale
apply approved discounts
request manager override
accept payment
charge customer account
issue receipt
start return
process exchange
open cash drawer
void item
hold sale
resume sale
send bill to customer phone
send payment link
```

Staff must have roles.

Roles may include:

```text id="staff_roles"
cashier
supervisor
manager
owner
service staff
restaurant waiter
warehouse counter staff
mobile seller
event staff
support agent
```

Protected staff actions:

```text id="protected_staff_actions"
manual price override
manager discount
refund without receipt
cash drawer open
void after payment
gift card activation
store credit issue
account charge approval
return outside policy
B2B settlement exception
```

Protected actions require authority and audit.

Humans near cash drawers become philosophers of temptation. Selene should be unimpressed.

---

## 21. Self-Checkout, Scan-and-Go + Assisted Checkout

POS modes:

```text id="checkout_modes"
cashier checkout
self-checkout
scan-as-you-shop
scan-and-go
mobile staff checkout
table-side checkout
salon chair-side checkout
event checkout
warehouse pickup checkout
drive-through checkout
```

Self-checkout must support:

```text id="self_checkout_support"
product scan
cart verification
weight verification
restricted item approval
payment
receipt
exit proof
exception handling
staff assist
fraud/shrinkage signals
```

Scan-and-go must support:

```text id="scan_go_support"
customer identity
store context
live cart
payment on device
exit verification
random audit if required
receipt QR
```

Selene should let the store choose operating mode.

One POS brain. Many checkout skins.

A supermarket, salon, restaurant, hardware store, and pop-up stall should not need five unrelated little cash-register cults.

---

## 22. Restaurant Table Bill + Split Payment

POS must support restaurant/table payment modes.

Customer can:

```text id="restaurant_customer_can"
scan table QR
view live table bill
order from phone
split bill by person
split bill by item
split equally
pay own share
pay full bill
include family/friends
apply points/credits privately
tip where applicable
request digital receipt
```

Restaurant staff see:

```text id="restaurant_staff_see"
table paid
partial paid
items unpaid
split status
tips
kitchen/order status
exceptions
```

Examples:

```text id="restaurant_examples"
Dad pays kids’ meals.
Friend pays own drinks.
Wife pays her items.
Selene tracks who paid what.
```

This solves the ancient restaurant tragedy:

```text id="restaurant_tragedy"
“Who ordered the extra entrée?”
```

Selene knows.

Selene remembers.

Selene judges quietly.

---

## 23. Returns, Exchange + Refund Intake

POS must allow physical return intake.

Customer says:

```text id="return_intake_examples"
“I want to return this.”
“I bought this online.”
“This item is faulty.”
“I want to exchange size.”
```

POS identifies:

```text id="return_identifies"
receipt / QR / customer identity
original order
store-owned vs B2B item
return window
condition
inspection requirement
refund method
provider responsibility
stock disposition
commission reversal if B2B
```

POS does not own return logistics.

POS sends handoff to Returns.

If item is B2B-sourced, B2B manages provider/commission settlement consequences.

If item is faulty/warranty-related, Warranty/Original Provider owns responsibility.

POS is the counter where the conversation starts.

Not the entire legal universe of the shoe.

---

## 24. Digital Receipt, Tax Invoice + Living Proof

POS receipt should be a living digital record.

Receipt includes:

```text id="receipt_includes"
receipt
tax invoice where required
payment method summary
points earned/used
discounts applied
warranty proof
return button
product care info
delivery tracking
B2B provider support route if applicable
digital manuals
recall alerts
reorder shortcut
audit reference
```

Receipt QR can support:

```text id="receipt_qr_support"
return
warranty
reorder
support
tax invoice download
business expense claim
proof of payment
product instructions
```

Paper receipts are tiny ghosts from accounting.

Selene receipts should be useful after the sale, which is apparently a radical idea.

---

## 25. SoftPOS / Mobile POS / Tap-to-Pay Layer

POS must support hardware-light selling where possible.

Use cases:

```text id="mobile_pos_use_cases"
market stall
mobile staff
restaurant table-side
salon chair-side
delivery driver
event ticket sales
warehouse pickup
small business counter
pop-up store
```

Mobile POS may use:

```text id="mobile_pos_methods"
phone/tablet app
Tap to Pay / SoftPOS where payment provider supports it
Bluetooth receipt printer
digital receipt only
QR payment
customer device payment
portable scanner
camera scanning
```

POS must still enforce:

```text id="mobile_pos_controls"
staff authority
payment security
receipt/audit
store context
tax
inventory update
offline/degraded limits
```

Hardware-light does not mean control-light.

A phone can be a POS. It should not be a tiny fraud trampoline.

---

## 26. Offline / Degraded Mode + Sync Reconciliation

POS must work when connectivity is weak, within limits.

Modes:

```text id="offline_modes"
online normal
degraded sync delay
offline limited sale
offline cash only
offline account sale with limit
offline payment terminal fallback
sync-and-reconcile later
```

Offline limitations:

```text id="offline_limits"
no high-risk credit offline
transaction value limits
staff role limits
customer account limits
no unknown B2B settlement exceptions
no high-value refunds offline
no professional service compliance bypass
```

On reconnection, POS must reconcile:

```text id="offline_reconcile"
sales
payments
cash
inventory
discounts
tax
returns
receipts
audit
conflicts
```

Because the internet will fail exactly when someone is buying 23kg of meat and a birthday cake. It has a sense of humor.

---

## 27. Loss Prevention, Shrinkage + Fraud Signals

POS must help detect fraud/shrinkage.

Signals:

```text id="fraud_signals"
unscanned item
weight mismatch
barcode mismatch
price override anomaly
void pattern
cash drawer anomaly
refund abuse
return without receipt
gift card abuse
staff override frequency
self-checkout item mismatch
RFID exit mismatch
basket/cart mismatch
customer scan vs cashier scan mismatch
B2B item refund abuse
```

Actions:

```text id="fraud_actions"
staff assist
manager approval
random cart audit
hold refund
route fraud review
preserve evidence
notify audit
update risk score
```

Computer vision/RFID/weight signals may be future integrations.

POS should provide the hooks.

Retail shrinkage is not a mystery. It is usually a barcode, a bag, and optimism.

---

## 28. Tax, Accounting + Cashflow Handoff

POS sends structured sale events.

Handoff includes:

```text id="accounting_handoff"
store_id
branch_id
terminal_id
staff_id
customer_id if known
items
quantities
weights
prices
discounts
tax
payment methods
cash/card/wallet split
points/credits used
B2B context
returns/exchanges
refunds
rounding
cash drawer events
receipt_ref
audit_ref
```

Accounting owns ledger.

Tax owns tax treatment.

BankRec proves cash/card settlement.

Cashflow handles liquidity impact.

POS provides sale truth.

POS does not post ledger because apparently we are not letting the till become an accountant. Growth.

---

## 29. Conversation-to-Action Guardrail

POS conversations are not fixed phrase matching.

Customer/staff may say:

```text id="pos_conversation_examples"
“Add these to the bill.”
“Use her store credit.”
“Put it on account.”
“Refund that order.”
“Split the bill.”
“Send the receipt to my phone.”
“Apply the birthday voucher.”
“Charge my company account.”
```

Correct flow:

```text id="pos_conversation_flow"
GPT-5.5 interprets natural language.
PH1.X resolves live context.
PH1.M / relevant customer/store/order memory resolves durable context.
POS and owner engines verify facts before action.
Protected actions require authority or customer verification.
Everything important is audited.
```

GPT-5.5 may:

```text id="gpt_may"
explain bill
summarize discounts
compare payment choices
explain return policy
help staff find product
draft customer-facing receipt/support language
```

GPT-5.5 must not:

```text id="gpt_must_not"
approve refund
override price
change payment method
invent discount
invent stock
invent weight
invent tax
open cash drawer
release payment
bypass manager approval
```

GPT speaks.

POS verifies.

Payment pays.

Accounting posts.

The checkout toaster remains safely unarmed.

---

## 30. Human-Like POS Interactions

### Customer scan-as-you-shop

```text id="scan_phrase"
“You qualify for the bundle if you add one more.”
```

### Private payment suggestion

```text id="payment_suggestion_phrase"
“You can use $18.40 in points. Want to apply it?”
```

### Bill verification

```text id="bill_verify_phrase"
“Your phone cart and store scan match.”
```

### Restaurant split

```text id="split_phrase"
“I split the bill by person. You can pay your share now.”
```

### Staff override

```text id="override_phrase"
“This refund is outside policy and needs manager approval.”
```

### Weighed product

```text id="weighed_phrase"
“This label has weight, price, batch, and expiry. Ready to scan.”
```

### B2B item

```text id="b2b_phrase"
“This item is fulfilled by an approved Selene provider. I’ll route support through the provider if needed.”
```

Selene should sound like a competent assistant, not a compliance spreadsheet wearing a name tag.

---

## 31. Automation and Exception-Only Review

Selene auto-handles:

```text id="auto_handles"
store context detection
customer cart creation
scan-as-you-shop cart updates
running total calculation
valid offer display
payment option ranking
discount validation request
points/reward option display
B2B context request
receipt generation
routine payment confirmation
routine return intake
digital bill send
restaurant split calculation
inventory sale signal
tax/accounting handoff
audit event capture
```

Selene escalates:

```text id="escalates"
high-value refund
refund outside policy
cash drawer anomaly
manager override
restricted item sale
age/compliance item where applicable
weight mismatch
self-checkout fraud signal
B2B settlement exception
professional service compliance issue
offline high-risk transaction
payment failure
customer privacy conflict
```

Rule:

```text id="automation_rule"
Routine sale = Selene handles.
Protected money/action = verification.
High-risk / legal / fraud / policy issue = route.
Everything important = audit.
```

Checkout should be fast until it should not be fast.

That is the whole trick.

---

## 32. POS State Machines

### 32.1 POS Session State

```text id="state_pos_session"
Idle
StoreContextDetected
CustomerIdentified
CartStarted
Scanning
PricingCalculated
PaymentOptionsPresented
VerificationRequired
PaymentPending
PaymentComplete
ReceiptIssued
Closed
Abandoned
```

### 32.2 Customer-Carried Cart State

```text id="state_customer_carried_cart"
NotStarted
StoreContextConfirmed
ItemScanned
OfferPresented
OfferAccepted
OfferIgnored
RunningTotalUpdated
ReadyForCheckout
SubmittedToStorePOS
PaymentComplete
ExitProofReady
Closed
```

### 32.3 Bill Verification State

```text id="state_bill_verification"
NoComparison
CustomerCartReady
StoreCartReady
Comparing
Matched
MismatchDetected
MismatchResolved
OverrideRequired
Closed
```

### 32.4 Tender Optimization State

```text id="state_tender"
NoCustomer
BenefitsChecking
BestTenderCalculated
CustomerPrivatePromptShown
TenderSelected
SplitTenderSelected
VerificationRequired
PaymentSubmitted
PaymentComplete
Closed
```

### 32.5 Store Context State

```text id="state_store_context"
Unknown
LowConfidenceDetected
StoreSuggested
QRConfirmed
NFCConfirmed
POSSessionConfirmed
TableConfirmed
CartConfirmed
PaymentContextConfirmed
Closed
```

### 32.6 Staff Authority State

```text id="state_staff_authority"
Allowed
RoleCheckRequired
ManagerApprovalRequired
CustomerVerificationRequired
Denied
Approved
AuditCaptured
Closed
```

### 32.7 Scale / Label Interface State

```text id="state_scale_label"
NotRequired
ScaleRequested
WeightCaptured
LabelGenerated
LabelScanned
WeightMismatch
TradeMeasurementException
Closed
```

### 32.8 B2B POS Context State

```text id="state_b2b_pos"
NotB2B
B2BItemDetected
B2BContextRequested
OriginalProviderIdentified
ChannelStoreConfirmed
SettlementHandoffRequired
ProviderSupportRouteStored
Closed
```

### 32.9 Return Intake State

```text id="state_return_intake"
Requested
ReceiptMatched
OrderMatched
EligibilityChecking
InspectionRequired
ProviderRouteRequired
ReturnsEngineSubmitted
RefundPending
Closed
```

### 32.10 Offline Mode State

```text id="state_offline"
Online
Degraded
OfflineLimited
OfflineCashOnly
OfflineAccountLimited
SyncPending
SyncConflict
Reconciled
Closed
```

---

## 33. Reason Codes

```text id="reason_codes"
POS_SESSION_STARTED
STORE_CONTEXT_DETECTED
STORE_CONTEXT_CONFIRMED_BY_QR
STORE_CONTEXT_CONFIRMED_BY_NFC
STORE_CONTEXT_CONFIRMED_BY_POS_SESSION
CUSTOMER_IDENTIFIED_AT_POS
CUSTOMER_PRIVATE_BILL_SENT
CUSTOMER_CARRIED_CART_STARTED
SCAN_AS_YOU_SHOP_ITEM_ADDED
LIVE_CART_UPDATED
PRODUCT_POINT_OFFER_PRESENTED
PRODUCT_POINT_OFFER_ACCEPTED
PRODUCT_POINT_OFFER_IGNORED
BILL_VERIFICATION_MATCHED
BILL_VERIFICATION_MISMATCH
BEST_TENDER_CALCULATED
POINTS_AVAILABLE_FOR_PAYMENT
STORE_CREDIT_AVAILABLE_FOR_PAYMENT
INTEREST_FREE_OPTION_AVAILABLE
SPLIT_TENDER_SELECTED
PAYMENT_SENT_TO_CUSTOMER_DEVICE
PAYMENT_COMPLETED_ON_CUSTOMER_DEVICE
POS_PAYMENT_CONFIRMED
BARCODE_SCANNED
QR_SCANNED
RFID_READ
NFC_TAPPED
WEIGHT_CAPTURED
LABEL_SCANNED
BULK_REPACK_ITEM_SCANNED
PRICE_DISCOUNT_VALIDATION_REQUESTED
MANAGER_OVERRIDE_REQUIRED
STAFF_AUTHORITY_DENIED
B2B_ITEM_DETECTED_AT_POS
ORIGINAL_PROVIDER_ROUTE_STORED
B2B_SETTLEMENT_HANDOFF_REQUIRED
RETURN_INTAKE_STARTED
RETURN_SUBMITTED_TO_RETURNS_ENGINE
RESTAURANT_TABLE_BILL_OPENED
RESTAURANT_BILL_SPLIT_CREATED
OFFLINE_MODE_ENTERED
OFFLINE_SYNC_RECONCILED
POS_AUDIT_EVENT_CAPTURED
```

---

## 34. Required Simulations

```text id="required_simulations"
customer scans store QR and starts scan-as-you-shop
customer scans product and live cart updates
customer receives toothpaste bundle offer at shelf
customer accepts bundle offer
customer ignores bundle offer
customer sees running total before cashier
customer cart matches cashier cart
customer cart mismatches cashier cart due to missing discount
customer pays on phone and cashier receives paid confirmation
customer privately uses points to pay
customer privately uses store credit and card split tender
customer selects interest-free payment option
customer scans item with 2D barcode / GS1 Digital Link
customer scans receipt QR for return
cashier scans customer Selene QR
restaurant customer scans table QR
restaurant bill split by person
restaurant bill split by item
wife pays her items and customer pays kids’ meals
staff sends salon bill to customer phone
customer pays salon bill privately
POS detects B2B item
POS requests Original Provider route
POS creates B2B settlement handoff
POS scans weighed banana label
POS detects scale weight mismatch
POS scans beef pack from bulk parent lot
POS sends inventory sale signal
POS requests discount validation from Pricing
POS presents cross-sell from Marketing
POS triggers low-stock/JIT signal to Inventory
POS starts return intake
POS routes B2B faulty item to Returns/B2B/Warranty
POS operates in offline limited mode
POS syncs offline sale
staff requests refund outside policy
manager approval required
cash drawer opened and audit captured
self-checkout unscanned item signal
```

---

## 35. Integration Map

```text id="integration_map"
PH1.POS / POS.COMMERCE
↔ PH1.ECOMMERCE
↔ PH1.B2B_PLATFORM
↔ PH1.ORIGINAL_PROVIDER / PROVIDER_OF_RECORD
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.PRICING / MARGIN / DISCOUNT
↔ PH1.MARKETING
↔ PH1.REWARDS / LOYALTY
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_CREDIT / WALLET / VIRTUAL_SETTLEMENT
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.SETTLEMENT_TRUST / CUSTOMER_PROTECTION
↔ PH1.ORDER / ORCHESTRATION
↔ PH1.DISPATCH / PACKING / COURIER_HANDOFF
↔ PH1.RETURNS / REFUNDS / REVERSE_LOGISTICS
↔ PH1.WARRANTY / AFTER_SALES
↔ PH1.WEIGHING_SCALE_LABEL
↔ PH1.BULK_BREAKDOWN / YIELD / REPACK
↔ PH1.WAREHOUSE
↔ PH1.MANUFACTURING / BOM / WIP
↔ PH1.RECIPE / FOOD_PREP
↔ PH1.RESTAURANT / MENU / BOOKING
↔ PH1.EVENTS / INVITATIONS / RSVP
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.CASHFLOW
↔ PH1.BANKREC
↔ PH1.SAAS_TENANCY / DEVICE_ACCESS
↔ PH1.PREDICTIVE_INPUT
↔ PH1.MP / MEMORY_PATTERN
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
```

---

## 36. Required Logical Packets

```text id="logical_packets"
POSSessionPacket
StoreContextPacket
CustomerPOSIdentityPacket
CustomerCarriedCartPacket
ScanAsYouShopItemPacket
LiveCartPacket
BillVerificationPacket
ProductPointOfferPacket
TenderOptimizationPacket
PrivatePaymentSuggestionPacket
POSTenderSelectionPacket
POSPaymentIntentPacket
POSPaymentConfirmationPacket
POSBarcodeScanPacket
POSQRCodeScanPacket
POSRFIDReadPacket
POSNFCTapPacket
POSScaleWeightEventPacket
POSLabelScanPacket
POSBulkRepackItemPacket
POSPricingValidationPacket
POSMarketingOfferExecutionPacket
POSRewardBalancePacket
POSB2BItemContextPacket
POSOriginalProviderRoutePacket
POSB2BSettlementHandoffPacket
POSInventoryMovementPacket
POSReturnIntakePacket
POSRestaurantTableBillPacket
POSBillSplitPacket
POSDigitalReceiptPacket
POSStaffAuthorityPacket
POSManagerOverridePacket
POSOfflineTransactionPacket
POSSyncReconciliationPacket
POSFraudSignalPacket
POSAuditEvidencePacket
```

Logical only.

No runtime packet structs.

Codex can leave the schema goblin outside with the receipt printer.

---

## 37. What Codex Must Not Do

```text id="codex_must_not"
Do not make POS own E-Commerce.
Do not make POS own B2B settlement.
Do not make POS own Product-to-B2B readiness.
Do not make POS own Original Provider responsibility.
Do not make POS own Channel Store commission.
Do not make POS own provider payout.
Do not make POS own product master truth.
Do not make POS own inventory stock truth.
Do not make POS own discount/pricing truth.
Do not make POS own marketing strategy.
Do not make POS own points/reward balances.
Do not make POS own customer credit underwriting.
Do not make POS own payment authorization/capture.
Do not make POS own accounting ledger posting.
Do not make POS own tax treatment.
Do not make POS own legal-for-trade scale compliance.
Do not make POS own bulk breakdown/yield/repack truth.
Do not make POS own warehouse operations.
Do not make POS own return logistics.
Do not bypass staff authority or manager approval.
Do not expose private customer payment options to staff unless permitted.
Do not use fixed phrase matching.
Do not let GPT-5.5 invent price, stock, discount, points, tax, weight, payment, refund, or warranty facts.
Do not store raw payment-card data casually.
Do not store raw biometric data.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 38. Final Architecture Sentence

Selene POS + Commerce Execution Engine is the physical commerce intelligence surface that lets customers, staff, restaurants, salons, supermarkets, events, warehouses, and stores complete in-person commerce through cashier checkout, customer-carried scan-as-you-shop, self-checkout, mobile staff checkout, restaurant table bills, private phone payment, best-tender recommendations, barcode/QR/RFID/NFC scanning, weighing/label interfaces, B2B-aware item recognition, returns intake, digital receipts, staff authority, offline sync, fraud/shrinkage detection, and audit; while Product, Inventory, Pricing, Marketing, Rewards, Payment, B2B, Order, Dispatch, Returns, Warranty, Scale/Label, Bulk Breakdown, Accounting, Tax, and Audit engines preserve their own truth and execution ownership.

Simple version:

```text id="simple_version"
POS is Selene’s physical commerce surface.
Customer can scan as they shop.
Customer can see the bill before cashier.
Customer can get product-point offers.
Customer can pay privately on their own phone.
Customer can split restaurant bills.
Store POS receives payment confirmation.
POS scans barcode, QR, RFID, NFC, labels, and weighed goods.
POS recognizes B2B items.
POS routes B2B responsibility to Original Provider through B2B.
POS shows best payment options but Payment executes.
POS shows discounts but Pricing owns discount truth.
POS shows rewards but Rewards owns balances.
POS updates stock signals but Inventory owns stock truth.
POS takes returns but Returns owns return execution.
POS sells weighed/repacked goods but Scale/Label and Bulk Breakdown own their truth.
GPT-5.5 makes it human.
Deterministic engines verify before action.
Everything important is audited.
```

That is Global Document 79 — Selene POS + Commerce Execution Engine. It is not a cash register. It is the in-store commerce brain where the customer can become the POS, the phone becomes the private bill, the store becomes smarter, the cashier becomes a confirmation point, the scale stops lying, the barcode gets upgraded, the payment method is optimized, and the receipt finally does something useful besides fading in a glovebox like a tiny financial ghost.

[1]: https://www.pcisecuritystandards.org/standards/?utm_source=chatgpt.com "Payment Card Data Security Standards (PCI DSS)"
[2]: https://developer.apple.com/tap-to-pay/?utm_source=chatgpt.com "Tap to Pay on iPhone"
[3]: https://www.gs1.org/industries/retail/solution-provider-2d-readiness?utm_source=chatgpt.com "Compliance for 2D barcodes at Point of Sale (POS)"
[4]: https://www.nist.gov/pml/owm/nist-handbook-44-current-edition?utm_source=chatgpt.com "NIST Handbook 44 - Current Edition"

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
