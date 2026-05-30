# Global Document 71 — Selene Goods Receiving + Inspection + Supplier Credit Automation Engine

```text id="doc71_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
71

ENGINE:
PH1.PROC.RECEIVE / PH1.GOODS_INSPECTION / PH1.RECEIVING_PROOF

FULL NAME:
Selene Goods Receiving, Inspection, Acceptance, Variance, Quarantine, Supplier Credit, Replacement, Refund, and AP Hold Automation Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Goods Receiving + Inspection Engine owns the truth of what actually arrived from a supplier and whether it is acceptable.

It answers:

```text id="receiving_questions"
What was expected?
Did it arrive?
Who received it?
What quantity arrived?
Was it the correct product?
Was it damaged?
Was it short?
Was it over-delivered?
Was it expired or near expiry?
Was it the correct batch, lot, or serial item?
Was inspection required?
Was it accepted, rejected, quarantined, or returned?
What evidence proves the result?
What can Inventory accept?
What should AP pay?
What should AP hold?
What does the supplier owe?
```

This engine is the real-world proof layer between:

```text id="receiving_chain"
Purchase Order
→ Supplier Delivery
→ Receiving / Inspection
→ Inventory Accepted Stock
→ Supplier Obligation
→ AP Invoice Matching
→ Supplier Payment
→ Accounting
```

A PO says:

```text id="po_says"
We ordered 100.
```

Receiving says:

```text id="receiving_says"
We received 100, accepted 90, rejected 10 damaged, opened supplier credit obligation, and told AP to hold the disputed value.
```

That second sentence saves money. The first one is just optimism with a PO number.

---

## 2. Why Receiving Comes After Procurement

The chain must stay clean:

```text id="receiving_after_procurement"
Product defines the item.
Inventory detects need.
Supplier confirms supplier trust and terms.
Procurement creates the PO.
Receiving proves what actually arrived.
Inventory accepts only proven usable stock.
AP pays only matched, accepted value.
Supplier Engine tracks supplier performance and obligations.
Accounting posts final financial truth.
```

Receiving must not create the PO.

Receiving must not pay the invoice.

Receiving must not update supplier bank details.

Receiving must not post accounting.

Receiving’s job is reality:

```text id="receiving_job"
This arrived.
This did not arrive.
This is good.
This is damaged.
This is missing.
This is rejected.
This is accepted.
Here is the proof.
```

Which is more useful than the traditional warehouse method of “Dave thinks it looked fine.” Dave is trying, but Dave is not an audit trail.

---

## 3. Modern Receiving Standards Selene Must Be Ready For

Receiving must support traceability, shipping identifiers, advance shipment messages, inspection proof, and AP matching.

GS1’s Global Traceability Standard is designed to help organizations implement traceability systems using GS1 standards, meaning Selene should treat receiving as a traceability event, not merely a quantity update. ([GS1][1])

GS1 SSCCs identify logistics units such as cartons and pallets and act as a “license plate” for shipments, so Selene must be ready to capture shipment/container identifiers during receiving where available. ([GS1 US][2])

GS1 delivery management guidance describes despatch advice / advance shipment notice concepts as pre-announcements of goods being shipped, so Selene should be ready for Supplier Selene, EDI, ASN, portal, or shipment-notice data before goods arrive. ([GS1][3])

CIPS describes AP invoice verification against purchase order and goods receipt as a critical control mechanism to prevent errors and mitigate fraud risk, so Selene Receiving must generate proof that AP can trust later. ([CIPS Download][4])

Translation:

```text id="modern_receiving_translation"
Receiving is not “box arrived.”
Receiving is shipment identity, PO match, quantity proof, quality proof, traceability, supplier obligation, AP control, and audit.
```

A box is never just a box. It is a future invoice wearing cardboard.

---

## 4. Core Selene Law

```text id="receiving_core_law"
No accepted receiving proof = no clean inventory increase.
No accepted receiving proof = no clean supplier invoice payment.
Only accepted goods become usable, sellable, or payable.
Short, damaged, wrong, expired, rejected, or quarantined goods create supplier obligations and AP holds under policy.
Routine receiving variances are handled automatically by Selene.
Humans review only exceptions, high-risk cases, material value, missing evidence, supplier disputes, overrides, and protected decisions.
```

Selene must reduce human work by:

```text id="receiving_reduce_human"
telling receivers what is expected
opening camera/scan workflows
matching delivery notes to POs
reading delivery documents
guiding quantity counts
guiding inspection checks
capturing photos
capturing batch/lot/serial/expiry
detecting short deliveries
detecting damaged goods
creating supplier obligations
requesting credit notes
requesting replacements
requesting refunds
holding AP disputed amounts
updating Inventory with accepted stock only
updating Supplier score
preparing AP matching proof
```

Humans should not have to ask:

> “What was supposed to arrive today?”

Selene should say:

> “Tom, Supplier ABC is expected today with PO-771: 100 units. Please confirm count and take a photo if anything is damaged.”

That is receiving with a brain. Not a clipboard wearing sneakers.

---

## 5. Engine Boundary

### 5.1 PH1.PROC.RECEIVE owns

```text id="receiving_owns"
expected delivery intake from Procurement
delivery arrival confirmation
delivery note capture
ASN/despatch advice reference
SSCC/logistics unit capture where available
PO matching at delivery
receiver assignment reference
quantity check
condition check
damage capture
photo/video evidence
batch/lot/serial capture
expiry capture
temperature/cold-chain capture where required
quality inspection workflow
accepted quantity
rejected quantity
short quantity
over-delivered quantity
quarantined quantity
wrong item record
supplier variance record
supplier obligation trigger
supplier credit/replacement/refund trigger
AP hold instruction
Inventory accepted-stock handoff
receiving audit evidence
```

### 5.2 PH1.PROC.RECEIVE does not own

```text id="receiving_not_own"
product identity
stock forecasting
supplier qualification
purchase order creation
supplier invoice validation
supplier payment execution
ledger posting
tax treatment
supplier bank details
final supplier score ownership
```

### 5.3 Correct owner split

```text id="receiving_owner_split"
PH1.PROCUREMENT = what was ordered and expected.
PH1.PROC.RECEIVE = what arrived and what was accepted.
PH1.INVENTORY = stock after accepted handoff.
PH1.SUPPLIER = supplier performance, obligations, disputes, score.
PH1.CREDITORS / AP = invoice matching and payable amount.
PH1.ACCOUNTING = final financial posting.
PH1.AUDIT = proof.
```

Receiving proves reality.

Inventory stores usable stock.

AP pays proven value.

Accounting posts final money truth.

Beautiful. Suspiciously civilized.

---

## 6. Receiving Master Record

Every receiving event creates a record.

```text id="receiving_master_record"
receiving_id
legal_entity_id
branch_id
warehouse_id
location_id
po_id
supplier_id
supplier_status_snapshot
delivery_note_id
ASN_ref
SSCC_ref
carrier_ref
tracking_ref
delivery_date
delivery_time
received_by_user_id
backup_receiver_id
inspector_user_id
product_id
variant_id
supplier_sku
quantity_ordered
quantity_expected
quantity_delivered
quantity_counted
quantity_accepted
quantity_rejected
quantity_short
quantity_overdelivered
quantity_damaged
quantity_quarantined
batch_number
lot_number
serial_numbers
expiry_date
manufacture_date
temperature_reading
condition_status
inspection_status
photos
videos
delivery_note_image
supplier_documents
variance_reason_codes
supplier_obligation_refs
AP_hold_ref
Inventory_handoff_ref
audit_ref
```

Receiving records must be:

```text id="receiving_record_rules"
source-backed
time-stamped
receiver-linked
location-linked
PO-linked
supplier-linked
Inventory-linked
AP-linked
audit-linked
```

If it cannot be proven, Selene should not treat it as clean receiving truth. We are not running a warehouse séance.

---

## 7. Receiving Applies To More Than Physical Goods

Receiving supports:

```text id="receiving_types"
retail goods
raw materials
ingredients
finished goods
spare parts
equipment
vehicles
machinery
IT hardware
office supplies
packaging
perishable goods
cold-chain goods
regulated goods
batch/lot goods
serialized goods
high-value assets
construction materials
contractor deliverables
service completion
digital deliverables
drop-ship customer-direct deliveries
```

For services, receiving becomes **service acceptance**.

Example:

```text id="service_acceptance"
Supplier repaired air conditioner.
Manager confirms work completed.
Photo/report attached.
Service accepted.
AP may match invoice.
```

If not accepted:

```text id="service_not_accepted"
AP hold.
Supplier follow-up.
Service obligation remains open.
```

Invoices for “services rendered” should not win just because the phrase sounds fancy.

---

## 8. Expected Delivery Intake

Procurement sends Receiving an expected delivery.

```text id="expected_delivery_packet"
ExpectedDeliveryPacket:
- PO number
- supplier
- expected items
- expected quantities
- expected delivery date/window
- delivery location
- receiver
- backup receiver
- inspection requirement
- batch/serial/expiry requirements
- storage requirement
- special handling
- AP matching requirement
- audit ref
```

Receiving uses this for:

```text id="expected_delivery_uses"
daily receiving manifest
receiver notification
delivery checklists
inspection readiness
Inventory putaway planning
AP proof preparation
```

Detailed daily manifest automation is expanded in:

```text id="doc72_reference"
Global Document 72 — Receiving Daily Manifest + Credit Note Automation Addendum
```

Document 71 owns the core receiving proof model.

Document 72 expands the daily operational cadence, receiver notifications, and policy automation. Tiny boundary. Important boundary. Please don’t make me fight another document octopus.

---

## 9. Delivery Arrival Confirmation

When delivery arrives, Selene guides the receiver.

Arrival methods:

```text id="arrival_methods"
scan PO
scan barcode / QR
scan SSCC / logistics label
upload delivery note
photo delivery note
voice confirmation
supplier Selene shipment message
carrier tracking event
manual receiving search
```

Selene asks:

> “Which PO is this delivery for?”

If delivery note contains PO:

```text id="po_auto_match"
Selene reads and matches PO.
```

If no PO:

```text id="no_po_receiving"
Receiving hold.
Procurement review.
AP normal payment blocked.
Supplier record warning.
```

Selene says:

> “I cannot find a matching PO. I’ll hold this delivery for Procurement review.”

Goods arriving without buying proof should not stroll into inventory like they own the place.

---

## 10. Proof Capture Levels

Selene uses risk-based proof.

### Level 1 — Simple confirmation

For low-risk goods.

```text id="proof_level_1"
receiver confirms arrival
receiver confirms quantity
audit timestamp
```

### Level 2 — Delivery note photo

```text id="proof_level_2"
photo of delivery note
supplier name
PO number
quantity
delivery date
receiver identity
```

### Level 3 — Goods photo

```text id="proof_level_3"
pallet/carton photo
product label photo
condition photo
quantity evidence where visible
```

### Level 4 — Scan / identifier proof

```text id="proof_level_4"
barcode
QR code
SSCC
batch
lot
serial
expiry
```

### Level 5 — High-risk proof

For high-value, perishable, regulated, cold-chain, traceable, or dispute-prone goods.

```text id="proof_level_5"
delivery note photo
goods photo
quantity count
condition check
batch/lot/serial
expiry
temperature
receiver identity
location/time proof
inspection result
```

Rule:

```text id="proof_rule"
Low-risk delivery = light proof.
High-risk delivery = strong proof.
Short/damaged delivery = photo proof required.
Missing evidence = Selene asks for evidence, not random approval.
```

No photographing every pencil. No accepting high-value machinery with “looks fine.” We are aiming between silly and catastrophic.

---

## 11. Quantity Check

Selene compares:

```text id="quantity_compare"
quantity ordered
quantity expected
quantity delivered
quantity counted
quantity accepted
```

Variance types:

```text id="quantity_variance_types"
full match
short delivery
over-delivery
partial delivery
wrong pack size
wrong unit of measure
missing cartons
extra cartons
```

Example:

```text id="short_quantity_example"
PO ordered: 100
Receiver counted: 95
Short: 5
```

Selene action:

```text id="short_quantity_action"
record 95 received
mark 5 short
create supplier obligation
notify Supplier Engine
prepare AP hold for value of 5
update Procurement
```

Selene says:

> “You ordered 100 and received 95. I’ve marked 5 as short, opened a supplier obligation, and told AP to hold that value.”

No approval required if evidence and policy allow.

That is not “lack of control.” That is control doing its job without begging humans for permission to count.

---

## 12. Over-Delivery

Over-delivery occurs when supplier sends more than ordered.

```text id="over_delivery"
PO ordered: 100
Delivered: 120
Extra: 20
```

Selene must not automatically accept extra goods.

Actions:

```text id="overdelivery_actions"
record expected quantity
hold extra quantity
check policy tolerance
check demand/stock/cashflow
ask Procurement if extra spend is allowed
return/reject if not accepted
amend PO if approved
notify AP expected invoice value
```

Selene says:

> “Supplier delivered 120 against a PO for 100. I’ll hold the extra 20 until Procurement decides whether to accept or return them.”

Suppliers sometimes “accidentally” ship extra and invoice later. Very creative. Very not automatic.

---

## 13. Damage and Condition Check

Selene checks for:

```text id="damage_types"
broken item
dented packaging
water damage
leakage
contamination
temperature breach
expired goods
near-expiry goods
missing seal
faulty item
wrong specification
scratched/damaged equipment
unsafe product
```

If damage exists:

```text id="damage_actions"
capture photo/video
record damaged quantity
mark damaged stock not accepted
quarantine if needed
create supplier obligation
request credit/replacement/refund under policy
notify AP of hold
update Supplier score
```

Selene says:

> “Five units are damaged. Please take a photo. I’ll request credit or replacement under policy and keep the disputed value out of normal payment.”

This is where Selene must not ask a human to approve the existence of damage. The damage is sitting there. Looking damaged. Doing its best.

---

## 14. Wrong Item / Substitution

If supplier sends the wrong item:

```text id="wrong_item_action"
do not accept as ordered stock
capture evidence
quarantine or reject
notify Procurement
open supplier obligation
AP hold
```

If supplier sends substitute item:

```text id="substitution_action"
hold stock
Procurement review
Product review if new product/variant
Inventory template review
Pricing/compliance review if needed
AP hold until accepted
```

Selene says:

> “The supplier sent a substitute item. I will not add it as sellable stock until Procurement approves substitution and Product confirms identity.”

No mystery products entering inventory like surprise guests at a wedding.

---

## 15. Inspection Workflow

Inspection depends on product type and risk.

Inspection types:

```text id="inspection_types"
visual inspection
quantity inspection
quality inspection
technical inspection
food safety inspection
temperature inspection
serial/VIN verification
batch/lot verification
certificate verification
service completion acceptance
asset condition inspection
```

Inspection outcomes:

```text id="inspection_outcomes"
Accepted
AcceptedWithNote
PartiallyAccepted
Rejected
Quarantined
FurtherInspectionRequired
```

High-risk goods may require separate inspector.

Examples:

```text id="inspection_examples"
Routine office supplies: receiver can accept.
Medical/regulatory goods: quality/compliance inspection required.
Machine part: maintenance/engineering inspection required.
Vehicle: condition/VIN/registration inspection required.
Food: expiry/temp/allergen/condition inspection required.
```

Selene says:

> “This product requires inspection before it becomes sellable. I’ll keep it in quarantine until approved.”

No inspection? No available stock. Very annoying. Very correct.

---

## 16. Perishable, Shelf-Life, and Cold-Chain Receiving

For perishables, Selene checks:

```text id="perishable_checks"
expiry date
manufacture date
remaining shelf life
temperature at receipt
cold-chain evidence
packaging condition
batch/lot
supplier minimum shelf-life terms
```

Outcomes:

```text id="perishable_outcomes"
accept
accept with urgent-sale flag
accept with discount claim
reject
quarantine
request credit/replacement/refund
```

Selene says:

> “This dairy batch has only three days remaining shelf life. Your policy requires seven days. I recommend rejecting or requesting credit.”

This is how Selene stops businesses from buying milk that arrives already emotionally prepared to expire.

---

## 17. Batch, Lot, Serial, and Traceability Capture

If Product requires traceability, Receiving enforces capture.

Traceability fields:

```text id="trace_capture_fields"
batch number
lot number
serial number
expiry date
manufacture date
supplier certificate
country of origin
SSCC / logistics unit
receiving event
inspection result
```

If required data missing:

```text id="trace_missing"
do not mark accepted
quarantine or hold
request missing data
notify Supplier
notify Procurement
AP hold if invoice arrives
```

Selene says:

> “This product requires batch tracking. I cannot mark it accepted until the batch number is captured.”

Traceability cannot be added later by guesswork. Guesswork is not a batch number. It is a lawsuit with hobbies.

---

## 18. Quarantine

Quarantine is used when stock exists but cannot be used/sold yet.

Quarantine triggers:

```text id="quarantine_triggers"
inspection pending
damage suspected
missing certificate
temperature breach
wrong item
recall notice
contamination risk
regulatory hold
supplier dispute
unidentified product
```

Quarantine states:

```text id="quarantine_states"
QuarantinePending
Quarantined
InspectionRequired
Released
Rejected
ReturnedToSupplier
Disposed
Archived
```

Inventory receives quarantined stock only as **non-sellable** if needed.

Selene says:

> “The stock is physically here, but it is quarantined and not available for sale.”

On hand, not sellable. Once again, tiny distinction, large consequences.

---

## 19. Service Acceptance

Services need receiving proof too.

Service receiving applies to:

```text id="service_receiving_applies"
repairs
maintenance
cleaning
contractor work
consulting milestone
software setup
installation
professional services
security service
delivery service
```

Service acceptance evidence:

```text id="service_evidence"
work order
completion report
photos
service note
manager acceptance
time logs
milestone proof
asset/service link
```

If accepted:

```text id="service_accepted"
AP may match invoice.
Supplier performance updated.
```

If not accepted:

```text id="service_rejected"
AP hold.
Supplier obligation.
Procurement follow-up.
```

Selene says:

> “The service invoice cannot be approved because the work order has not been accepted.”

Consulting invoices also need proof. Yes, even the ones with confident fonts.

---

## 20. Asset Receiving

Asset receiving covers high-value items.

Examples:

```text id="asset_receiving_examples"
vehicle
forklift
machine
computer equipment
boat
aircraft
building equipment
major tools
```

Asset receiving captures:

```text id="asset_receiving_fields"
PO
invoice
delivery note
serial / VIN
photos
condition
manuals
warranty
registration
certificate
custodian
location
insurance requirement
asset accounting flag
```

Handoff:

```text id="asset_receiving_handoff"
PH1.ASSET
PH1.ASSET_ACCOUNTING
PH1.INSURANCE
PH1.FLEET if vehicle
PH1.ACCOUNTING
```

Selene says:

> “The forklift was received and accepted. I’ll create the asset handoff, notify Insurance, and route capitalization review.”

No asset should arrive and vanish into “misc equipment.” That bucket is where fixed assets go to develop identity issues.

---

## 21. Drop-Ship and Customer-Direct Receiving

If supplier ships directly to customer, Selene creates remote receiving proof.

Evidence sources:

```text id="dropship_evidence"
carrier proof of delivery
customer confirmation
customer photos
supplier dispatch proof
delivery signature
customer complaint
tracking event
Supplier Selene delivery confirmation
```

States:

```text id="dropship_states"
Shipped
DeliveredToCustomer
CustomerAccepted
CustomerRejected
DamagedAtDelivery
ShortDeliveryReported
PendingCustomerConfirmation
Closed
```

If customer reports damage:

```text id="dropship_damage_flow"
open supplier/logistics obligation
customer resolution
AP hold if unpaid
AR/customer credit if needed
logistics claim if carrier fault
```

Selene says:

> “Customer reports damaged delivery. I’ll hold supplier payable amount and open supplier/logistics resolution.”

Customer-direct delivery does not bypass receiving. It moves receiving proof outside the warehouse.

---

## 22. Selene-to-Selene Receiving Protocol

If supplier uses Selene, Supplier Selene may send:

```text id="supplier_selene_receiving_msgs"
AdvanceShipmentNoticePacket
ShipmentConfirmationPacket
DeliveryDelayNoticePacket
DeliveryDocumentPacket
BatchSerialDataPacket
CreditNotePacket
ReplacementShipmentPacket
RefundConfirmationPacket
```

Buyer Selene validates and prepares receiving.

Buyer Selene may send:

```text id="buyer_selene_receiving_msgs"
ReceivingVariancePacket
DamageEvidencePacket
ShortDeliveryPacket
WrongItemPacket
CreditNoteRequestPacket
ReplacementRequestPacket
ReceivingAcceptancePacket
```

Example:

```text id="selene_receiving_example"
Supplier Selene:
Delivery delayed until tomorrow.

Buyer Selene:
Updates expected delivery.
Updates Inventory forecast.
Checks customer/production impact.
Notifies only if impact matters.
```

Two Selenes communicating is faster than two humans forwarding PDFs titled “final delivery note revised actual final.pdf.” Not that we judge. We absolutely judge.

---

## 23. Supplier Obligation Creation

Receiving creates supplier obligations when issues exist.

Obligation triggers:

```text id="supplier_obligation_triggers"
short delivery
damaged goods
wrong goods
faulty goods
rejected goods
expired/near-expiry below policy
missing certificate
temperature breach
unaccepted service
overcharge linked to receiving
```

Obligation types:

```text id="receiving_obligation_types"
credit note required
replacement required
refund required
supplier collection required
correct invoice required
correct item required
service rework required
warranty claim required
```

Obligation handoff:

```text id="supplier_obligation_handoff"
SupplierObligationPacket
```

Receiving sends to Supplier Engine and AP.

Selene says:

> “I opened a supplier obligation for five damaged units. AP will hold the disputed value until credit or replacement is complete.”

This is the heartbeat of real supplier control.

No more “supplier owes us something, I think.” Selene knows. Selene nags. Selene wins.

---

## 24. Credit Note / Replacement / Refund Automation

Receiving must trigger supplier credit automation under policy.

Routine policy examples:

```text id="credit_policy_examples"
damaged goods under threshold → request credit note automatically
short delivery under threshold → request replacement automatically
near-expiry below policy → request credit or reject
wrong item → request replacement
supplier over-delivery → hold excess pending Procurement decision
```

Automation flow:

```text id="credit_automation_flow"
variance detected
→ evidence captured
→ disputed quantity/value calculated
→ supplier obligation created
→ credit/replacement/refund requested
→ AP hold applied
→ Supplier Engine updates score
→ Procurement notified
→ obligation tracked until closed
```

Selene says:

> “The damaged quantity is under the automatic credit policy. I’ve requested a credit note and held the AP amount.”

No manager approval for obvious routine damage. That was the whole point. We are not building an approval vending machine.

---

## 25. AP Hold Handoff

Receiving sends AP proof.

```text id="ap_hold_packet"
APHoldFromReceivingPacket:
- receiving_id
- po_id
- supplier_id
- invoice_id if known
- accepted_quantity
- disputed_quantity
- disputed_value
- reason
- evidence_refs
- credit/replacement/refund status
- AP_action
- audit_ref
```

AP action:

```text id="ap_actions_from_receiving"
pay accepted quantity only
hold disputed value
block invoice if no receiving proof
await credit note
await replacement
await refund
route exception
```

Example:

```text id="ap_example"
Invoice claims: 100
Accepted: 90
Damaged: 10

AP:
Pay/approve 90
Hold 10
Await credit note or replacement
```

AP should not pay for damaged goods just because the supplier invoice was neatly formatted. Fraud and incompetence both own PDF software.

---

## 26. Inventory Handoff

Inventory receives only accepted stock.

```text id="inventory_receipt_handoff"
InventoryReceiptHandoffPacket:
- receiving_id
- po_id
- product_id
- variant_id
- supplier_id
- accepted_quantity
- quarantine_quantity
- damaged_quantity
- rejected_quantity
- batch/lot
- serials
- expiry
- location
- putaway_instruction
- audit_ref
```

Inventory actions:

```text id="inventory_actions_from_receiving"
add accepted stock as available/putaway pending
add quarantined stock as non-sellable
do not add damaged/rejected as available
track batch/serial/expiry
update stock health
update channel availability if needed
```

Receiving does not own stock after handoff.

Inventory does.

That is the contract. No warehouse turf war. Not today.

---

## 27. Procurement Handoff

Receiving sends Procurement outcomes.

```text id="receiving_procurement_handoff"
PO fully received
PO partially received
PO not arrived
short delivery
damaged goods
wrong item
over-delivery
supplier delay
replacement pending
credit note requested
supplier substitution proposed
```

Procurement may:

```text id="procurement_response_to_receiving"
close PO
amend PO
cancel remainder
request replacement
source backup supplier
approve/reject over-delivery
open supplier dispute
```

Selene says:

> “PO remains partially open because five units are short. I requested credit/replacement and notified Procurement.”

Procurement owns the order. Receiving owns proof. AP owns payment truth. Everybody, once again, in their chair.

---

## 28. Supplier Performance Handoff

Receiving sends Supplier Engine:

```text id="supplier_performance_handoff"
delivery timeliness
quantity variance
damage variance
wrong item
quality result
inspection result
certificate issue
temperature breach
supplier response needed
evidence refs
```

Supplier Engine updates:

```text id="supplier_engine_updates"
delivery score
quality score
open obligations
dispute history
trust score
watchlist/restriction recommendation
```

Selene says:

> “Supplier quality score reduced because 10% of delivery was damaged.”

Not spite. Data.

Though a little spite is understandable.

---

## 29. Receiving and Accounting Handoff

Receiving does not post accounting.

Receiving provides evidence for:

```text id="receiving_accounting_evidence"
accepted inventory
GRNI / goods received not invoiced
accrual candidates
damaged goods
inventory write-off evidence
supplier credit/refund evidence
asset receipt evidence
service acceptance evidence
```

Accounting owns:

```text id="accounting_from_receiving"
ledger posting
inventory asset
expense
AP liability
GRNI
credit note accounting
write-off
asset capitalization
```

Selene says:

> “Goods were accepted before invoice arrived. I’ll provide GRNI evidence for period close.”

Receiving proves the operational fact.

Accounting translates it into books.

That is not optional. That is how month-end avoids interpretive dance.

---

## 30. No-PO Delivery Handling

If goods arrive without PO:

```text id="no_po_delivery_handling"
hold receiving
capture evidence
do not add to available inventory unless policy allows quarantine/hold
notify Procurement
block normal AP payment
check fraud/maverick spend risk
request authorization
```

Possible outcomes:

```text id="no_po_outcomes"
valid emergency purchase
retrospective PO exception
unauthorized delivery
supplier mistake
fraud risk
return to supplier
```

Selene says:

> “Delivery has no matching PO. I will hold it and request Procurement review before stock or AP can proceed.”

Goods without PO are not automatically gifts. Suppliers tend to send invoices after the surprise.

---

## 31. Receiving Variance Tolerance

Companies may set tolerance rules.

Tolerance types:

```text id="receiving_tolerances"
quantity tolerance
price tolerance handled by AP/Procurement
damage tolerance
over-delivery tolerance
expiry tolerance
temperature tolerance
service completion tolerance
```

Examples:

```text id="tolerance_examples"
short delivery under 2% may auto-request credit
over-delivery under 1% may auto-accept if low value and policy allows
expiry below minimum shelf life always rejects
temperature breach always quarantines for food
```

Selene must not use one universal tolerance for everything.

A 2% shortage in napkins and a 2% shortage in surgical components do not belong in the same moral universe.

---

## 32. Receiving Evidence Quality Score

Each receiving event gets an evidence score.

Inputs:

```text id="evidence_score_inputs"
PO matched
delivery note captured
goods photo captured
receiver identity captured
quantity counted
inspection completed
batch/serial/expiry captured where required
damage photo captured where needed
temperature captured where required
supplier documents attached
```

Evidence states:

```text id="evidence_states"
Strong
Sufficient
Weak
MissingCritical
Rejected
```

Selene says:

> “Receiving evidence is weak because the delivery note is missing. I’ll ask the receiver to upload it before AP can match cleanly.”

No proof, no clean payment. Simple. Beautiful. Deeply irritating to sloppy suppliers.

---

## 33. Automation and Exception-Only Review

Selene auto-handles:

```text id="receiving_auto_handles"
expected delivery matching
delivery note extraction
quantity variance detection
damage evidence request
supplier obligation creation
credit note request under policy
replacement request under policy
refund request under policy
AP hold instruction
Inventory accepted-stock handoff
Supplier performance update
Procurement update
routine receiving closure
```

Selene escalates:

```text id="receiving_escalates"
high-value damage
missing evidence
supplier disputes claim
accepting damaged goods
over-delivery acceptance increasing spend
substitute product acceptance
regulated/high-risk goods
temperature breach requiring specialist review
manual override
write-off
fraud risk
no-PO delivery
```

Rule:

```text id="receiving_exception_rule"
Routine variance = Selene handles.
Material/risky variance = Selene routes.
Protected override = authority approves.
Everything = audited.
```

No “approval required” for every dented carton.

No “auto-accept” for a freezer full of warm chicken.

Balance. Such a rare creature.

---

## 34. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_receiving_allowed"
summarize receiving variance
draft supplier credit request
draft replacement request
draft refund request
explain damaged goods issue
summarize inspection notes
translate receiving issue into human wording
draft supplier dispute message
prepare AP exception summary
prepare Procurement update
```

GPT-5.5 must not:

```text id="gpt_receiving_forbidden"
confirm goods arrived
accept goods
override inspection
add stock to inventory
release AP payment
close supplier obligation
invent proof
invent photos
override policy
approve write-off
```

GPT-5.5 can write:

> “Five units arrived damaged; attached photos show broken packaging.”

GPT-5.5 cannot decide the five units are fine because it feels optimistic. This is not a motivational warehouse.

---

## 35. Human-Like Selene Interaction

### Expected delivery

> “Supplier ABC is expected today with PO-771: 100 units. Sarah is assigned to receive it.”

### Arrival

> “Take a photo of the delivery note and I’ll match it to the PO.”

### Short delivery

> “You received 95 against an order of 100. I’ll mark 5 as short, hold that value from AP, and request credit or replacement.”

### Damage

> “Five units are damaged. Please take a photo. I’ll request a credit note under policy.”

### Inspection

> “This product requires inspection before it becomes sellable. I’ll keep it in quarantine until approved.”

### Over-delivery

> “The supplier delivered 120 but we ordered 100. I’ll hold the extra 20 until Procurement decides whether to accept or return them.”

### No PO

> “This delivery has no matching PO. I’ll hold it and request Procurement review before AP can pay.”

Human-like, direct, and allergic to supplier invoice nonsense. A healthy personality trait.

---

## 36. State Machines

### Receiving State

```text id="receiving_state"
Expected
DeliveryArrived
ProofCapturePending
POIdentified
CountingInProgress
QuantityVarianceDetected
InspectionPending
Accepted
PartiallyAccepted
Rejected
Quarantined
SupplierObligationCreated
InventoryHandoffComplete
APHandoffComplete
Closed
Archived
```

### Inspection State

```text id="inspection_state"
NotRequired
Pending
InProgress
Accepted
AcceptedWithNote
PartiallyAccepted
Rejected
Quarantined
FurtherInspectionRequired
Closed
```

### Supplier Credit / Replacement State

```text id="supplier_credit_state"
NotRequired
CreditNoteRequested
ReplacementRequested
RefundRequested
SupplierResponded
CreditNoteReceived
ReplacementReceived
RefundReceived
PartiallyResolved
Disputed
Escalated
Closed
Archived
```

### AP Hold State

```text id="receiving_ap_hold_state"
NoHold
HoldRecommended
HoldApplied
AwaitingCreditNote
AwaitingReplacement
AwaitingRefund
Released
Closed
```

### No-PO Delivery State

```text id="no_po_state"
Detected
EvidenceCaptured
ProcurementReviewPending
AuthorizedAsException
Rejected
ReturnedToSupplier
FraudReview
Closed
Archived
```

---

## 37. Reason Codes

```text id="receiving_reason_codes"
DELIVERY_EXPECTED
DELIVERY_ARRIVED
DELIVERY_NOTE_CAPTURED
ASN_MATCHED
SSCC_CAPTURED
PO_MATCHED
NO_MATCHING_PO
RECEIVER_CONFIRMED
QUANTITY_MATCH
SHORT_DELIVERY
OVER_DELIVERY
PARTIAL_DELIVERY
WRONG_ITEM_RECEIVED
SUBSTITUTE_ITEM_RECEIVED
DAMAGE_DETECTED
PHOTO_PROOF_REQUIRED
PHOTO_PROOF_CAPTURED
INSPECTION_REQUIRED
INSPECTION_ACCEPTED
INSPECTION_REJECTED
QUARANTINE_REQUIRED
EXPIRY_TOO_SHORT
TEMPERATURE_BREACH
BATCH_REQUIRED
LOT_REQUIRED
SERIAL_REQUIRED
CERTIFICATE_MISSING
ACCEPTED_QUANTITY_RECORDED
REJECTED_QUANTITY_RECORDED
SUPPLIER_OBLIGATION_CREATED
CREDIT_NOTE_REQUIRED
REPLACEMENT_REQUIRED
REFUND_REQUIRED
AP_HOLD_REQUIRED
INVENTORY_HANDOFF_READY
SUPPLIER_SCORE_UPDATE_REQUIRED
PROCUREMENT_UPDATE_REQUIRED
RECEIVING_EVIDENCE_WEAK
RECEIVING_EXCEPTION_REVIEW_REQUIRED
```

---

## 38. Required Simulations

```text id="receiving_simulations"
expected delivery received cleanly
delivery note photo matched to PO
ASN matched to PO
SSCC captured
full quantity accepted
short delivery auto-credit
damaged goods photo proof
damaged goods credit note request
wrong item received
substitute item received and held
over-delivery held
over-delivery accepted after Procurement review
perishable received with short shelf life
temperature breach quarantine
batch-tracked goods received
serialised asset received
service accepted
service rejected and AP held
drop-ship customer delivery confirmed
drop-ship customer damage reported
no-PO delivery held
supplier obligation created
AP hold created from receiving
Inventory accepted-stock handoff
Supplier score updated from receiving issue
Receiving evidence weak blocks clean AP match
```

---

## 39. Integration Map

```text id="receiving_integration_map"
PH1.PROC.RECEIVE / GOODS_INSPECTION
↔ PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.CREDITORS.RECON
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.BUDGET
↔ PH1.LOGISTICS
↔ PH1.RETURNS
↔ PH1.ASSET
↔ PH1.ASSET_ACCOUNTING
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.COMPLIANCE
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.MANUFACTURING / PRODUCTION
↔ PH1.RESTAURANT / MENU
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.REM
↔ PH1.BCAST / DELIVERY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 40. Required Logical Packets

```text id="receiving_packets"
ExpectedDeliveryPacket
ReceivingEventPacket
DeliveryArrivalPacket
DeliveryNoteCapturePacket
ASNMatchPacket
SSCCCapturePacket
QuantityCheckPacket
ConditionCheckPacket
InspectionPacket
DamageEvidencePacket
ExpiryCheckPacket
TemperatureCheckPacket
BatchLotSerialCapturePacket
QuarantinePacket
AcceptedQuantityPacket
RejectedQuantityPacket
ReceivingVariancePacket
SupplierObligationPacket
SupplierCreditRequestPacket
SupplierReplacementRequestPacket
SupplierRefundRequestPacket
APHoldFromReceivingPacket
InventoryReceiptHandoffPacket
ProcurementReceivingUpdatePacket
SupplierPerformanceEventPacket
ServiceAcceptancePacket
DropShipReceivingPacket
NoPODeliveryPacket
ReceivingAuditEvidencePacket
```

Logical only. Codex maps later. No runtime packet structs, please. The schema goblin can wait outside.

---

## 41. What Codex Must Not Do

```text id="codex_no_receiving"
Do not merge Receiving into Inventory.
Do not merge Receiving into AP.
Do not let supplier invoices create receiving truth.
Do not add stock to Inventory before accepted quantity is proven.
Do not pay for damaged/missing goods.
Do not close supplier obligations without proof.
Do not accept over-delivery automatically if it increases spend.
Do not treat no-PO deliveries as clean receiving.
Do not let GPT-5.5 confirm goods were received.
Do not let GPT-5.5 invent evidence.
Do not require human approval for routine policy-covered shortages/damage.
Do not implement from this document alone.
```

---

## 42. Final Architecture Sentence

Selene Goods Receiving + Inspection + Supplier Credit Automation Engine is the real-world proof layer that receives supplier deliveries, matches them to purchase orders and shipment notices, captures delivery notes, photos, scans, SSCCs, batch/lot/serial/expiry/temperature evidence, verifies quantity and condition, controls inspection, quarantine, rejection, and acceptance, sends only accepted stock to Inventory, creates supplier obligations for short, damaged, wrong, expired, or rejected goods, triggers credit note, replacement, or refund automation under policy, instructs AP to hold disputed amounts, updates Procurement and Supplier performance, and uses GPT-5.5 for clear human communication while deterministic Selene proof, policy, authority, and audit preserve operational and financial truth.

Simple version:

```text id="receiving_simple"
Procurement says what should arrive.
Receiving proves what actually arrived.
Inspection proves what is usable.
Good stock goes to Inventory.
Bad, missing, wrong, or rejected stock creates supplier credit/replacement/refund.
AP holds disputed value.
Supplier score updates.
Humans approve only real exceptions.
Everything is audited.
```

That is Global Document 71 — Goods Receiving + Inspection + Supplier Credit Automation Engine. The warehouse door now has a brain, a camera, a PO match, and enough backbone to tell a supplier, “Nice invoice, but ten units were broken, champ.”

[1]: https://www.gs1.org/standards/gs1-global-traceability-standard/current-standard?utm_source=chatgpt.com "GS1 Global Traceability Standard"
[2]: https://www.gs1us.org/upcs-barcodes-prefixes/serialized-shipping-container-codes?utm_source=chatgpt.com "Serialized Shipping Container Codes (SSCC) - GS1 US"
[3]: https://www.gs1.org/docs/tl/GS1Standards_DeliveryManagement.pdf?utm_source=chatgpt.com "GS1 Standards for Delivery Management"
[4]: https://cips-download.cips.org/short-reads/from-requisition-to-payment-how-the-procure-to-pay-process-streamlines-procurement?utm_source=chatgpt.com "From requisition to payment: how the procure-to-pay ..."
