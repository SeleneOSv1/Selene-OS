# Global Document 71 — Selene Goods Receiving + Inspection + Supplier Credit Automation Engine v2

## Receiving Proof, Inspection, Acceptance, Inventory Handoff + AP Hold Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
71

ENGINE:
PH1.RECEIVING / PH1.GOODS_RECEIVING / PH1.RECEIPT_PROOF

FULL NAME:
Selene Goods Receiving, Inspection, Receipt Proof, Quantity Verification, Damage Evidence, Traceability Capture, Quarantine, Accepted Stock Handoff, Supplier Obligation Trigger, AP Hold, and Receiving Audit Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Document 71 owns the **proof of what actually arrived**.

Procurement says what was ordered.
Receiving proves what arrived.
Inventory only accepts proven accepted stock.
AP only pays what can be matched to PO + receiving proof + invoice.

Receiving answers:

```text
What was expected?
Did it arrive?
Who received it?
How much arrived?
Was it damaged?
Was it short?
Was it over-delivered?
Was it wrong?
Was it expired?
Was it cold-chain valid?
Was it accepted, rejected, quarantined, or held?
What evidence proves it?
What can Inventory accept?
What should AP pay or hold?
What does the supplier owe?
```

Simple rule:

```text
No accepted receiving proof = no clean inventory increase.
No accepted receiving proof = no clean supplier invoice payment.
```

No paying for ghost toilet paper. No stocking imaginary cartons. Apparently this needs saying.

---

## 2. Core Receiving Law

```text
Only goods/services proven as received and accepted may become clean inventory, usable assets, service completion proof, or payable supplier obligation.

Short, damaged, wrong, expired, rejected, quarantined, missing, or unaccepted goods/services must create supplier obligations, AP holds, Procurement updates, and audit evidence.

Receiving proof must be source-backed, receiver-linked, location-linked, PO-linked, supplier-linked, Inventory-linked, AP-linked, and audit-linked.
```

Receiving must protect against:

```text
paying for goods not received
adding damaged goods to usable stock
accepting wrong items silently
accepting over-delivery without authority
accepting goods without PO proof
losing traceability
missing batch/serial/expiry evidence
ignoring cold-chain breaches
allowing AP to pay incorrect invoices
letting supplier shortages become company losses
```

Receiving is not a polite warehouse nod. It is proof.

---

## 3. Engine Ownership Boundary

### 3.1 Receiving owns

```text
expected delivery intake from Procurement / Document 70
delivery arrival confirmation
delivery note / ASN / despatch advice reference
SSCC / logistics unit capture where available
PO matching at delivery
receiver identity reference
quantity check
condition check
damage capture
photo/video evidence
batch / lot / serial capture
expiry capture
manufacture date capture where required
temperature / cold-chain capture
quality inspection workflow
accepted quantity
rejected quantity
short quantity
over-delivered quantity
damaged quantity
quarantine
wrong item record
supplier obligation trigger
supplier credit / replacement / refund trigger
AP hold instruction
Inventory accepted-stock handoff
asset/service receiving proof
receiving audit evidence
```

### 3.2 Receiving references but does not own

```text
PO creation
supplier selection
supplier qualification / rating
supplier bank trust
stock forecasting
quantity recommendation
cashflow approval
budget approval
final receiver scheduling
human workload allocation
Broadcast / Delivery infrastructure
Reminder infrastructure
supplier payment
invoice payable creation
final accounting posting
tax treatment
```

### 3.3 Correct owner split

```text
Procurement = what was ordered, why it was ordered, and what Receiving should expect.
Receiving = what actually arrived and what was accepted.
Document 72 = daily manifest, assigned work, reminders, supplier chasing, correction control.
Inventory = accepted stock truth.
AP = PO + receiving + invoice matching.
Supplier Intelligence = supplier performance and rating.
Supplier Payment = supplier payment execution.
Accounting = ledger posting.
Audit = proof.
```

This boundary matters. Receiving should not become Procurement, AP, Inventory, and a warehouse babysitter all at once. We have Document 72 for the babysitting.

---

## 4. Relationship to Document 70 and Document 72

### Document 70 — Procurement

Procurement sends Receiving:

```text
PO number
supplier
expected items
expected quantities
expected delivery date/window
delivery location
special handling
inspection requirements
storage requirements
batch/serial/expiry requirements
receiver requirement
AP matching requirement
audit reference
```

### Document 71 — Receiving Proof

Receiving proves:

```text
what arrived
what was counted
what was inspected
what was accepted
what was rejected
what was damaged
what was short
what was over-delivered
what must be held
what Inventory may accept
what AP must hold
```

### Document 72 — Daily Receiving Control

Document 72 manages:

```text
daily receiving manifest
receiver assignment workflow
scheduler / roster / task allocation
receiver readiness confirmation
shelf/freezer/dock readiness
reminders
escalations
supplier/courier chasing
supplier correction workflows
corrected invoice / credit note / replacement confirmations
```

Simple version:

```text
71 proves reality.
72 chases reality until it behaves.
```

---

## 5. Receiving Master Record

Every receiving event must create or update a Receiving Master Record.

Required fields include:

```text
receiving_id
legal_entity_id
branch_id
warehouse_id
location_id
po_id
purchase_order_line_id
supplier_id
supplier_delivery_note_ref
ASN_ref
SSCC_ref
carrier_ref
tracking_ref
delivery_date_time
received_by_user_id
backup_receiver_id where applicable
inspector_user_id where applicable
product_id
variant_id
supplier_sku
ordered_quantity
expected_quantity
delivered_quantity
counted_quantity
accepted_quantity
rejected_quantity
short_quantity
over_delivered_quantity
damaged_quantity
quarantined_quantity
batch_id
lot_id
serial_numbers
expiry_date
manufacture_date
temperature_reading
condition_status
inspection_status
photo_evidence_refs
video_evidence_refs
delivery_note_image_ref
supplier_document_refs
supplier_obligation_refs
AP_hold_ref
inventory_handoff_ref
audit_ref
```

Receiving records must be:

```text
time-stamped
receiver-linked
location-linked
PO-linked
supplier-linked
evidence-backed
Inventory-linked
AP-linked
audit-linked
```

No receiving séance. Proof or it did not happen.

---

## 6. Expected Delivery Intake

Receiving consumes expected delivery data from Procurement.

Expected Delivery Packet includes:

```text
PO number
supplier
items
quantities
expected delivery date/time
delivery location
receiver requirement
inspection requirement
batch/serial/expiry requirement
cold-chain requirement
storage requirement
special handling
AP match requirement
audit reference
```

Receiving uses this packet to prepare:

```text
delivery checklist
inspection checklist
traceability checklist
Inventory handoff readiness
AP proof readiness
exception handling
```

Document 72 uses the same packet for daily manifest, receiver assignment, reminders, and supplier chasing.

---

## 7. Delivery Arrival Confirmation

When delivery arrives, Receiving must match it to an expected delivery.

Supported methods:

```text
scan PO
scan supplier delivery note
scan barcode / QR
scan SSCC / logistics label
upload delivery note
photo delivery note
carrier tracking event
supplier Selene shipment message
manual receiving search
voice-guided receiving
```

If no matching PO or expected delivery exists:

```text
hold goods
block clean Inventory acceptance
block AP normal payment
route Procurement review
create supplier warning
require authority before acceptance
```

Goods without buying proof do not stroll into inventory like they own the warehouse.

---

## 8. Proof Capture Levels

Receiving must use risk-based proof.

```text
Level 1 — simple confirmation
Level 2 — delivery note photo
Level 3 — goods photo
Level 4 — barcode / QR / SSCC / batch / lot / serial / expiry scan
Level 5 — high-risk proof: photos, count, condition, batch/lot/serial, expiry, temperature, receiver identity, location/time, inspection result
```

Rules:

```text
Low-risk delivery = light proof.
High-risk delivery = strong proof.
Short/damaged/wrong delivery = evidence required.
Missing evidence = Selene asks for evidence, not random approval.
```

Do not photograph every pencil. Do not accept machinery with “looks fine.” Balance, the thing humans keep forgetting.

---

## 9. Quantity Verification

Receiving compares:

```text
quantity ordered
quantity expected
quantity delivered
quantity counted
quantity accepted
quantity rejected
quantity short
quantity over-delivered
```

Outcomes:

```text
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

```text
PO ordered 100.
Receiver counted 90.
Selene records 90 received, 10 short.
Accepted stock = 90.
Supplier obligation = 10 short.
AP hold = 10 not payable unless corrected.
```

No one pays for the missing 10. The supplier can send a corrected invoice, credit note, replacement, or their best apology in PDF form.

---

## 10. Over-Delivery Handling

If supplier sends more than ordered, Receiving must not auto-accept the extra.

Selene must:

```text
record expected quantity
record extra quantity
hold extra quantity
check tolerance policy
check Procurement authority
check Inventory need
check cashflow/budget impact if extra spend is possible
route Procurement decision
accept extra only if authorized
reject/return extra if not authorized
```

Suppliers do not get to ship extra and quietly invoice the company like a magician with cartons.

---

## 11. Damage, Condition + Wrong Item Handling

Receiving must inspect and classify goods.

Damage/condition outcomes:

```text
accepted clean
accepted with note
damaged
wrong item
substituted item
expired
near expiry
temperature breach
missing certificate
missing parts
packaging damage
quarantine required
rejected
```

For damaged goods, Selene must:

```text
capture photo/video proof
record damaged quantity
mark damaged stock not accepted unless policy allows quarantine
create supplier obligation
request credit/replacement/refund through Document 72
send AP hold
update Procurement
send supplier performance signal
```

---

## 12. Batch, Lot, Serial, Expiry + Cold-Chain Capture

Receiving must capture traceability when required.

Required for:

```text
food
medicine / health-sensitive products
perishables
cold-chain goods
regulated goods
warranty goods
serialized goods
high-value assets
electronics
vehicles
machinery
batch/lot controlled goods
```

Capture fields:

```text
batch
lot
serial
VIN where applicable
expiry
manufacture date
temperature
cold-chain evidence
certificate
condition photo
```

If required traceability is missing:

```text
hold / quarantine
do not accept clean stock
notify Procurement / Supplier
create AP hold
route exception to Document 72
```

Traceability is not decorative. It is what saves the business when a recall arrives wearing boots.

---

## 13. Quarantine

Quarantine applies when goods physically exist but cannot be used/sold yet.

Triggers:

```text
inspection pending
damage suspected
wrong item
temperature breach
missing certificate
recall notice
contamination risk
regulatory hold
supplier dispute
unidentified product
```

Quarantined goods may be handed to Inventory only as non-sellable / non-usable stock.

Quarantine status must include:

```text
reason
location
quantity
evidence
owner
review deadline
release/reject decision
audit reference
```

---

## 14. Service Acceptance

Receiving also handles service completion proof.

Examples:

```text
repair work
maintenance
contractor work
consulting milestone
installation
professional service milestone
cleaning service
software/digital deliverable
```

Service acceptance must capture:

```text
service order / PO reference
supplier
work completed
completion evidence
accepting person
defects/issues
accepted / rejected / partially accepted
AP hold if not accepted
```

Supplier invoice does not prove service completion. It proves someone wants money, which is a weaker standard.

---

## 15. Asset Receiving

Receiving must support high-value assets.

Asset receiving captures:

```text
asset description
serial number
VIN / registration where applicable
condition photos
manuals
warranty documents
location
custodian
insurance requirement
asset accounting flag
```

Asset receiving handoff goes to:

```text
Asset Engine
Insurance Engine
Accounting
Inventory where applicable
AP
Audit
```

---

## 16. Supplier Obligation Trigger

Receiving exceptions create supplier obligations.

Triggers:

```text
short delivery
damaged goods
wrong item
expired goods
near-expiry violation
temperature breach
missing certificate
unaccepted service
missing serial/batch evidence
rejected goods
```

Supplier obligations may require:

```text
credit note
corrected invoice
replacement shipment
refund
re-delivery
discount
formal dispute
```

Document 71 creates the obligation evidence.

Document 72 manages supplier chasing and closure.

---

## 17. AP Hold Instruction

Receiving exceptions must automatically create AP hold instructions.

AP hold reasons:

```text
short delivered
damaged
wrong item
rejected
quarantined
not received
service not accepted
missing evidence
supplier correction pending
```

AP should receive:

```text
PO reference
supplier
invoice risk
accepted quantity/value
held quantity/value
reason
evidence refs
supplier obligation ref
audit ref
```

Rule:

```text
AP must not pay the disputed portion until receiving exception is resolved.
```

Invoices are requests. Receiving proof is truth. Annoying for suppliers, excellent for the company.

---

## 18. Inventory Accepted-Stock Handoff

Only accepted goods become clean inventory.

Receiving sends Inventory:

```text
product_id
variant_id
accepted_quantity
location
batch/lot/serial
expiry
condition
quarantine flag if applicable
cost reference
PO reference
supplier
audit ref
```

Rejected, damaged, short, missing, or quarantined goods must not become clean sellable stock.

---

## 19. Human / External Action Orchestration Handoff

Document 71 must follow the Selene Human / External Action Orchestration Law.

Receiving may detect that an action is needed, but does not own all action execution.

Any action involving a person, supplier, AP user, manager, receiver, courier, or external party must define:

```text
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

```text
short delivery detected → supplier correction workflow in Document 72
AP hold needed → AP action through Broadcast/Delivery with acknowledgement if required
damaged goods → supplier correction + AP hold + Inventory quarantine
missing evidence → receiver task correction request
```

Receiving creates proof and action requirements.

Document 72, Task/Scheduler, Broadcast/Delivery, Reminder, Access/Authority, and Audit manage the action lifecycle.

No “notify AP” nonsense. It becomes a routed, tracked, confirmed action.

---

## 20. State Machines

### Receiving Event State

```text
Expected
Arrived
MatchedToPO
Counting
InspectionPending
InspectionComplete
Accepted
PartiallyAccepted
Rejected
Quarantined
Closed
```

### Quantity State

```text
NotCounted
Matched
Short
OverDelivered
Partial
WrongUOM
Disputed
Closed
```

### Condition State

```text
Unchecked
Clean
Damaged
Expired
NearExpiry
WrongItem
TemperatureBreach
MissingCertificate
QuarantineRequired
Rejected
Closed
```

### Supplier Obligation State

```text
None
Created
EvidenceAttached
SentToDocument72
AwaitingSupplierCorrection
Resolved
Disputed
Closed
```

### AP Hold State

```text
NotRequired
HoldRequired
HoldSent
AcknowledgementRequired
Acknowledged
Released
Closed
```

### Inventory Handoff State

```text
NotReady
AcceptedQuantityReady
QuarantineHandoffReady
SentToInventory
InventoryAccepted
InventoryRejected
Closed
```

---

## 21. Reason Codes

```text
RECEIVING_EXPECTED_DELIVERY_LOADED
RECEIVING_DELIVERY_ARRIVED
RECEIVING_PO_MATCHED
RECEIVING_NO_PO_FOUND
RECEIVING_DELIVERY_NOTE_CAPTURED
RECEIVING_PHOTO_EVIDENCE_CAPTURED
RECEIVING_QUANTITY_MATCHED
RECEIVING_SHORT_DELIVERY
RECEIVING_OVER_DELIVERY
RECEIVING_PARTIAL_DELIVERY
RECEIVING_WRONG_ITEM
RECEIVING_DAMAGED_GOODS
RECEIVING_EXPIRED_GOODS
RECEIVING_NEAR_EXPIRY_WARNING
RECEIVING_TEMPERATURE_BREACH
RECEIVING_BATCH_CAPTURED
RECEIVING_LOT_CAPTURED
RECEIVING_SERIAL_CAPTURED
RECEIVING_QUARANTINE_REQUIRED
RECEIVING_ACCEPTED_QUANTITY_SET
RECEIVING_REJECTED_QUANTITY_SET
SUPPLIER_OBLIGATION_CREATED
SUPPLIER_CORRECTION_HANDOFF_TO_DOC72
AP_HOLD_FROM_RECEIVING_CREATED
INVENTORY_ACCEPTED_STOCK_HANDOFF_CREATED
SERVICE_ACCEPTANCE_CONFIRMED
SERVICE_ACCEPTANCE_REJECTED
ASSET_RECEIVING_CAPTURED
RECEIVING_AUDIT_CAPTURED
```

---

## 22. Required Simulations

```text
expected delivery arrives and matches PO
delivery arrives with no PO
delivery note captured by photo
100 ordered and 100 accepted
100 ordered and 90 accepted with 10 short
supplier over-delivers 120 against 100 ordered
wrong item delivered
damaged goods delivered with photos
expired goods delivered
cold-chain temperature breach
batch/lot/serial captured before acceptance
goods quarantined pending inspection
partial delivery received
supplier obligation created for short goods
AP hold created for damaged goods
accepted stock handed to Inventory
service completion accepted
service completion rejected
asset received with serial/photo/warranty
high-value equipment requires stronger proof
Receiving sends supplier correction requirement to Document 72
Receiving sends AP hold requirement as action orchestration record
```

---

## 23. Integration Map

```text
PH1.RECEIVING
↔ PH1.PROCUREMENT / PURCHASE_ORDER
↔ PH1.RECEIVING_DAILY_MANIFEST / DOCUMENT_72
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER_BANK_TRUST
↔ PH1.INVENTORY
↔ PH1.WAREHOUSE
↔ PH1.AP / CREDITORS
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.TAX
↔ PH1.LOGISTICS / DELIVERY
↔ PH1.RETURNS
↔ PH1.ASSET
↔ PH1.INSURANCE
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
```

---

## 24. Required Logical Packets

```text
ExpectedDeliveryPacket
ReceivingEventPacket
DeliveryArrivalPacket
DeliveryNoteCapturePacket
QuantityCheckPacket
InspectionPacket
DamageEvidencePacket
TraceabilityCapturePacket
ColdChainCapturePacket
QuarantinePacket
AcceptedStockHandoffPacket
SupplierObligationPacket
APHoldFromReceivingPacket
InventoryReceiptHandoffPacket
ServiceAcceptancePacket
AssetReceivingPacket
ReceivingActionRequirementPacket
ReceivingAuditEvidencePacket
```

Logical only.

No runtime packet structs. The schema goblin can wait behind the loading dock.

---

## 25. What Codex Must Not Do

```text
Do not merge Receiving into Inventory.
Do not merge Receiving into AP.
Do not merge Receiving into Procurement.
Do not let supplier invoice create receiving truth.
Do not add stock before accepted quantity is proven.
Do not pay for damaged, missing, rejected, quarantined, or unaccepted goods.
Do not close supplier obligations without proof.
Do not auto-accept over-delivery.
Do not treat no-PO deliveries as clean receiving.
Do not make Receiving own final receiver scheduling/reminders; Document 72 and Task/Scheduler/Reminder own action control.
Do not use vague “notify” without action orchestration.
Do not let GPT-5.5 invent delivery proof, photos, count, condition, batch, serial, expiry, or temperature.
Do not let GPT-5.5 confirm goods arrived.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 26. Final Architecture Sentence

Selene Goods Receiving + Inspection Engine is the receiving proof layer that verifies what actually arrived against Procurement expectations, captures quantity, condition, traceability, damage, expiry, cold-chain, service, and asset evidence, determines accepted/rejected/short/over-delivered/quarantined quantities, triggers supplier obligations, blocks AP from paying incorrect invoices, hands only accepted stock to Inventory, and creates audit-backed receiving truth while Document 72 manages the daily manifest, human task orchestration, receiver readiness, supplier chasing, reminders, escalations, and correction closure.

Simple version:

```text
Procurement says what should arrive.
Receiving proves what actually arrived.
Inventory only accepts proven accepted stock.
AP only pays what matches PO + receiving + invoice.
Document 72 chases the humans and suppliers until exceptions are fixed.
Everything important is audited.
```

---

## 27. Commerce Stack 82 Warehouse Location Handoff

Receiving must ensure accepted stock is placed into a known warehouse, store, branch, bin, shelf, pallet, cold-room, secure-cage, or other governed stock location before it can be made dispatch-ready.

Inventory/Warehouse owns stock location truth. Document 82 consumes location truth for picking and dispatch. If location, batch, serial, expiry, cold-chain, or secure-storage proof is missing, dispatch must block and create a stock-location investigation task. Receiving and warehouse location changes must be auditable.
