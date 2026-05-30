# Global Document 70 — Selene Procurement + Purchase Order Engine

```text id="doc70_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
70

ENGINE:
PH1.PROCUREMENT / PH1.PROC.ORDER / PH1.PURCHASE_CONTROL

FULL NAME:
Selene Procurement, Purchase Request, Purchase Order, Reorder Automation, Supplier Selection, Budget Check, Cashflow Check, Authority Routing, and Buying Control Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Procurement + Purchase Order Engine owns the **decision to buy**.

It answers:

```text id="procurement_questions"
Do we need to buy this?
Why do we need it?
Who requested it?
Is it routine?
Is it urgent?
Is it inventory-driven?
Is it customer-order-driven?
Is it production-driven?
Is it asset-related?
Is it within budget?
Can cashflow support it?
Which supplier should be used?
Is the supplier approved?
Does the supplier owe credits or replacements?
Should we buy, transfer, delay, split, cancel, or source alternatives?
Should a purchase order be created?
Who, if anyone, must approve it?
```

Procurement is not merely “make a PO.”

That is old software with a stamp.

Selene Procurement is the autonomous buying brain that connects:

```text id="procurement_chain"
Inventory need
Product identity
Supplier intelligence
Budget
Cashflow
Authority
Purchase order creation
Receiving expectations
AP invoice matching
Accounting evidence
```

A weak system says:

> “Create PO.”

Selene says:

> “This item will run out in six days. Supplier B is more reliable than Supplier A, cashflow supports a split order, and the purchase is inside warehouse budget. I’ll create the PO under policy.”

That is procurement. The other thing is a printable shopping list wearing business shoes.

---

## 2. Why Procurement Comes After Supplier

The chain must remain clean:

```text id="supplier_before_procurement"
Product says what the item is.
Inventory says what is needed.
Supplier says who can supply it and whether they are trustworthy.
Procurement decides whether and how to buy.
Receiving proves what arrived.
AP validates invoice truth.
Payment pays only proven amounts.
```

Procurement must not create supplier truth.

It must ask Supplier Engine:

```text id="ask_supplier"
Is this supplier approved?
Are they reliable?
Are they restricted?
Are their documents current?
Do they owe credit notes?
Are bank details safe?
Do they have open disputes?
Are they allowed for this category?
```

Only then does Procurement create a buying event.

Buying from a bad supplier because the price is cheap is how companies discover that “cheap” can arrive late, broken, and invoiced twice.

---

## 3. Current Procurement Practice Selene Must Be Ready For

Procurement is now an end-to-end governance, risk, sustainability, automation, and supplier-performance discipline — not just a purchasing desk. ISO 20400 provides guidance for integrating sustainability within procurement across organizations of any size or sector, so Selene must be able to support procurement choices that consider supplier risk, sustainability, ethics, and responsible sourcing where the business requires it. ([ISO][1])

Procure-to-pay is also a control chain. CIPS describes invoice verification against the purchase order and goods receipt as a critical control mechanism to prevent errors and reduce fraud risk, and APQC identifies create/distribute purchase orders as a defined procurement process activity involving vendor-specific orders and terms. ([CIPS Download][2])

So Selene Procurement must support:

```text id="modern_procurement_ready"
requisition
approved suppliers
supplier risk
budget control
cashflow control
purchase order creation
PO transmission
supplier acknowledgement
delivery expectations
receiving assignment
invoice matching preparation
sustainability/compliance checks
spend governance
automation
audit
```

Procurement that starts with a supplier invoice is already late. Procurement should start when Selene detects the need, not when AP receives a PDF demanding tribute.

---

## 4. Core Selene Law

```text id="procurement_core_law"
Every material business purchase must originate from a controlled buying path:
need → supplier check → budget/cashflow check → authority check → purchase order → receiving proof → AP match → accounting evidence.
```

Selene must reduce human work by:

```text id="procurement_reduce_work"
detecting buying needs automatically
learning routine ordering cycles
creating draft purchase requests
auto-generating routine purchase orders under policy
checking approved suppliers
checking supplier score and obligations
checking budget
checking cashflow
checking authority
routing only true exceptions
sending POs to suppliers
receiving supplier acknowledgements
updating receiving manifests
preparing AP matching evidence
preventing invoice fraud
```

Humans should not manually ask:

> “Do we usually order toilet paper around now?”

Selene should say:

> “You usually reorder around this week. Stock suggests six days remaining. I can create the usual PO under policy.”

That is automation with a memory. A terrifying upgrade from humans and cupboards.

---

## 5. Engine Boundary

### 5.1 PH1.PROCUREMENT owns

```text id="procurement_owns"
purchase request
purchase reason
purchase category
routine purchase rule
reorder buying decision
supplier selection recommendation
purchase order creation
purchase order issue
purchase order amendment
purchase order cancellation
purchase authority routing
budget check request
cashflow check request
supplier risk check request
PO expected delivery data
receiver assignment
inspection requirement assignment
supplier acknowledgement tracking
purchase commitment handoff
procurement audit evidence
```

### 5.2 PH1.PROCUREMENT does not own

```text id="procurement_not_own"
product identity
stock quantity
supplier approval truth
supplier bank safety
goods receiving acceptance
supplier invoice validation
supplier payment execution
ledger posting
tax treatment
final budget ownership
final cashflow ownership
```

### 5.3 Correct owner split

```text id="procurement_owner_split"
PH1.PRODUCT = what the item is.
PH1.INVENTORY = what stock is needed.
PH1.SUPPLIER = which suppliers are approved/risky/good.
PH1.PROCUREMENT = whether/how Selene buys and creates PO.
PH1.PROC.RECEIVE = what arrived and was accepted.
PH1.CREDITORS / AP = invoice matching and payable amount.
PH1.SUPPLIER_PAYMENT = payment scheduling and banking handoff.
PH1.ACCOUNTING = final financial posting.
PH1.BUDGET = approved spend plan.
PH1.CASHFLOW = liquidity and payment timing.
PH1.ACCESS / AUTHORITY = approval rights.
PH1.AUDIT = proof.
```

Procurement creates the buying event.

It does not become Supplier, Inventory, Receiving, AP, or Accounting.

That sentence alone prevents several species of enterprise swamp creature.

---

## 6. Procurement Master Record

Every procurement action creates a controlled record.

```text id="procurement_master_record"
procurement_id
legal_entity_id
branch_id
department_id
cost_center_id
requester_id
purchase_owner_id
purchase_type
purchase_reason
source_trigger
urgency_level
product_id
service_id
asset_id
inventory_need_ref
customer_order_ref
production_order_ref
maintenance_ref
budget_ref
cashflow_ref
supplier_recommendation_ref
selected_supplier_id
supplier_score_snapshot
supplier_risk_snapshot
estimated_amount
currency
tax_category_hint
delivery_location
required_date
approval_status
PO_status
receiving_requirement
inspection_requirement
AP_matching_requirement
audit_ref
```

Procurement status:

```text id="procurement_status"
NeedDetected
RequestDrafted
SupplierEvaluating
BudgetChecking
CashflowChecking
AuthorityChecking
PendingApproval
Approved
Rejected
POCreated
POIssued
SupplierAcknowledged
InFulfillment
PartiallyReceived
FullyReceived
Closed
Cancelled
Disputed
Archived
```

---

## 7. Purchase Sources

Selene Procurement may be triggered by:

```text id="purchase_sources"
Inventory low-stock forecast
Inventory JIT replenishment need
Product launch
B2B customer order
E-commerce order
POS demand trend
Manufacturing production demand
Restaurant ingredient prep demand
Routine office supply cycle
Employee purchase request
Manager purchase request
Asset purchase request
Fleet repair / maintenance
Insurance-required repair
Contract renewal
Supplier minimum order
Emergency repair
Marketing campaign
Project spend
Tax/compliance requirement
```

Example:

```text id="source_example"
Inventory:
Product X will stock out in five days.

Supplier:
Supplier A is cheaper but unreliable.
Supplier B is reliable and approved.

Cashflow:
Cashflow supports split order.

Procurement:
Create PO for 60% now, 40% later if sales pace confirms.
```

This is buying with context.

Not “someone typed a PO.” Deeply moving, in a tragic old-software way.

---

## 8. Purchase Request

A purchase request is the pre-PO buying need.

Purchase request fields:

```text id="purchase_request_fields"
purchase_request_id
requester_id
department_id
cost_center_id
branch_id
purchase_type
item_or_service_description
product_id / service_id / asset_id
quantity_requested
unit_of_measure
estimated_cost
currency
required_date
urgency
business_reason
source_trigger
preferred_supplier_id
alternative_supplier_ids
budget_ref
cashflow_ref
approval_status
audit_ref
```

Purchase request types:

```text id="purchase_request_types"
inventory reorder
routine consumable
production material
customer-specific purchase
asset purchase
service purchase
repair/maintenance
contract renewal
emergency purchase
marketing/campaign spend
project spend
compliance-required purchase
```

Selene should auto-create routine purchase requests when evidence is clear.

Human involvement should be:

```text id="request_human_involvement"
routine = Selene creates/acts under policy
exception = Selene routes
unclear business reason = Selene asks
high-risk/high-value = authority approves
```

No one should approve a routine cleaning supply request 11 times like an office ceremony. We are not worshipping stationery.

---

## 9. Purchase Order Record

Every PO must be structured enough to support receiving, AP, and audit.

```text id="po_record"
po_id
procurement_id
supplier_id
supplier_status_snapshot
supplier_bank_safety_snapshot
supplier_terms_snapshot
legal_entity_id
branch_id
department_id
cost_center_id
requester_id
approver_id
purchase_owner_id
items
quantities
unit_of_measure
unit_price
tax_hint
total_amount
currency
delivery_address
delivery_contact
expected_delivery_date
delivery_terms
payment_terms
inspection_required
receiver_id
backup_receiver_id
quality_inspector_id
budget_ref
cashflow_ref
approval_ref
supplier_acknowledgement_ref
PO_status
audit_ref
```

PO line fields:

```text id="po_line_fields"
po_line_id
product_id
variant_id
service_id
asset_id
supplier_sku
description
quantity_ordered
unit_of_measure
unit_price
tax_hint
expected_delivery_date
inspection_required
linked_inventory_need
linked_customer_order
linked_production_order
linked_asset_request
```

A PO must answer:

```text id="po_must_answer"
what was ordered
why it was ordered
who requested it
who approved it
which supplier
which price
which quantity
where it should arrive
who receives it
what inspection is required
what AP should expect later
```

If the PO cannot answer that, it is not a PO. It is a wish with a number.

---

## 10. Procurement Lifecycle

### 10.1 Need Detection

Selene detects need from Inventory, Product, B2B, E-commerce, POS, Manufacturing, Maintenance, HR, Asset, or human request.

```text id="need_detection"
NeedDetected
```

Example:

> “Warehouse gloves are likely to run out in eight days based on usage. I recommend reordering now.”

### 10.2 Need Classification

Selene classifies:

```text id="need_classification"
routine
inventory reorder
customer-specific
production-critical
asset/capex
repair/maintenance
contract renewal
emergency
compliance-required
```

### 10.3 Supplier Evaluation

Selene asks Supplier Engine:

```text id="supplier_eval_questions"
approved?
preferred?
restricted?
blocked?
lead time?
quality score?
delivery score?
open obligations?
bank safety?
certificates current?
category approved?
```

### 10.4 Budget Check

Selene asks Budget:

```text id="budget_questions"
is budget available?
is cost center correct?
does spend exceed monthly/quarterly limit?
is this capex or opex?
is budget transfer needed?
```

### 10.5 Cashflow Check

Selene asks Cashflow:

```text id="cashflow_questions"
can cash support this?
will this affect payroll/tax/rent buffer?
should order be split?
should supplier terms be negotiated?
should purchase be delayed?
```

### 10.6 Authority Check

Selene asks Access/Authority:

```text id="authority_questions"
is auto-approval allowed?
does manager have limit?
does Finance need approval?
does board need approval?
does dual approval apply?
```

### 10.7 PO Creation

Selene creates PO or draft PO.

### 10.8 Supplier Issue

Selene sends PO to supplier.

### 10.9 Supplier Acknowledgement

Supplier accepts, rejects, changes, or backorders.

### 10.10 Receiving Preparation

Procurement sends expected delivery to Receiving.

### 10.11 Closure

PO closes after receiving/AP matching, cancellation, or dispute resolution.

---

## 11. Routine Order Automation

Selene must learn routine purchases.

Routine examples:

```text id="routine_examples"
toilet paper
cleaning supplies
printer toner
packaging tape
office supplies
restaurant ingredients
regular raw materials
spare parts
fuel cards / consumables
salon supplies
standard maintenance consumables
```

Selene tracks:

```text id="routine_tracking"
usual frequency
usual supplier
usual quantity
usual cost
usage trend
seasonality
stock level
lead time
budget
cashflow
approval policy
```

Selene says:

> “Tom usually orders packaging tape around this week. Stock suggests seven days remaining. I can prepare the usual order.”

If within policy:

```text id="routine_auto"
create PO automatically
notify responsible owner
audit
```

If abnormal:

```text id="routine_abnormal"
quantity unusually high
supplier changed
price changed
cashflow tight
budget exceeded
authority required
```

Selene escalates only the abnormal part.

No need for humans to babysit toilet paper unless the toilet paper has become suspiciously strategic.

---

## 12. Reorder Automation from Inventory

Inventory sends:

```text id="inventory_reorder_packet"
product_id
location_id
current_available_stock
reserved_stock
forecast_demand
stockout_date
recommended_quantity
recommended_timing
cash_impact
supplier_candidates
transfer_alternative
urgency
confidence_score
```

Procurement decides:

```text id="procurement_decisions"
create PO
create draft PO
split order
delay order
transfer stock instead
source alternative supplier
route approval
reject reorder
```

Selene says:

> “Inventory recommends reorder, but Branch 2 has excess stock. I recommend transfer instead of purchase.”

This is how Selene saves cash instead of buying what the company already owns somewhere else, which is apparently a revolutionary concept in multi-branch businesses.

---

## 13. Supplier Selection Logic

Procurement ranks suppliers using Supplier Engine data.

Inputs:

```text id="supplier_selection_inputs"
approved category
supplier status
supplier score
delivery reliability
quality score
lead time
MOQ
price
payment terms
open obligations
credit notes owed
bank safety
certificates/insurance
delivery zones
single-source risk
strategic relationship
cashflow impact
sustainability/compliance requirement
cyber risk if relevant
```

Selection outcomes:

```text id="supplier_selection_outcomes"
recommended supplier
backup supplier
split supplier order
supplier blocked
supplier review required
new supplier qualification required
RFQ required
```

Selene says:

> “Supplier A is 4% cheaper, but Supplier B is more reliable and can deliver before stockout. I recommend Supplier B.”

Procurement must not be price-only.

Price-only procurement is how cheap becomes expensive in three easy deliveries.

---

## 14. Budget and Spend Governance

Procurement must integrate with Budget / Spend Governance.

Budget check outputs:

```text id="budget_outputs"
within budget
near limit
over budget
budget transfer recommended
budget owner review required
capex review required
board threshold triggered
blocked by budget policy
```

Selene says:

> “This purchase is within the warehouse monthly limit and below Tom’s authority. I can approve it under policy.”

Or:

> “This exceeds the remaining marketing budget by $1,800. I recommend budget transfer or delay.”

Budget availability is not cash availability.

Cash availability is not budget approval.

Selene must check both, because businesses enjoy having two ways to be wrong.

---

## 15. Cashflow-Aware Procurement

Procurement must never ignore cashflow.

Cashflow check outputs:

```text id="cashflow_outputs"
cashflow green
cashflow watch
cashflow warning
split order recommended
delay recommended
supplier terms negotiation recommended
blocked by payroll/tax/rent buffer
urgent purchase allowed
```

Selene says:

> “This purchase is operationally needed, but paying it all now would reduce cash below payroll buffer. I recommend splitting the order.”

Procurement does not hoard stock like a dragon sitting on cash.

It buys what the business needs, when the business can support it.

---

## 16. Authority and Purchase Limits

Selene must respect role-based purchase limits.

Examples:

```text id="purchase_limits"
Warehouse Manager: up to $2,000 per month routine warehouse supplies
Restaurant Manager: up to $500 per day perishable ingredients
Branch Manager: up to $5,000 monthly consumables
Finance Manager: approve budget exceptions
CFO: approve high-value or cashflow-risk purchase
Board: approve strategic/capital spend over threshold
```

Authority inputs:

```text id="authority_inputs"
requester
role
department
cost center
monthly limit
single-purchase limit
category limit
supplier risk
budget status
cashflow status
capex/opex
emergency flag
```

Selene says:

> “This exceeds Tom’s warehouse monthly limit. I’m routing only the excess approval to Finance.”

Not the entire request if part is routine.

Selene should route the smallest necessary exception, not summon the whole approval circus.

---

## 17. Purchase Order Creation

PO creation can be:

```text id="po_creation_modes"
manual request
Selene draft
Selene auto-create under policy
recurring order
inventory-triggered
customer-order-triggered
production-triggered
emergency-triggered
contract-triggered
```

PO states:

```text id="po_states"
Draft
PendingBudgetCheck
PendingCashflowCheck
PendingAuthorityCheck
Approved
Issued
SupplierAcknowledged
SupplierChangeRequested
PartiallyFulfilled
Fulfilled
Cancelled
Closed
Disputed
Archived
```

Selene says:

> “I created the PO and sent it to Supplier B. Receiving has been assigned to Sarah, and expected delivery is Friday.”

The PO is not just buying.

It sets up Receiving and AP.

That is the whole point. The invoice later should not arrive like a mysterious stranger asking for money.

---

## 18. PO Transmission

PO can be sent through:

```text id="po_transmission"
Supplier Selene
supplier portal
email
EDI/API
B2B network
PDF
secure link
```

Selene-connected supplier flow:

```text id="selene_po_flow"
Buyer Selene sends PO.
Supplier Selene validates.
Supplier Selene accepts/rejects/proposes changes.
Buyer Selene records acknowledgement.
Procurement updates PO status.
Receiving manifest updates.
```

Non-Selene supplier flow:

```text id="non_selene_po_flow"
Selene sends PO by approved channel.
Supplier response captured.
Manual or email acknowledgement parsed.
Procurement records status.
```

Selene says:

> “Supplier acknowledged the PO but changed delivery date. I’ll check stockout impact before accepting.”

No automatic acceptance of supplier changes. Suppliers are creative. Often with your delivery date.

---

## 19. Supplier Acknowledgement and Change Handling

Supplier may respond:

```text id="supplier_ack_types"
accepted
rejected
partially accepted
backordered
price changed
quantity changed
delivery date changed
substitute proposed
MOQ changed
split delivery proposed
```

Selene checks:

```text id="change_checks"
price tolerance
quantity tolerance
delivery impact
stockout risk
customer impact
production impact
budget impact
cashflow impact
authority requirement
supplier risk
```

Possible outcomes:

```text id="change_outcomes"
accept automatically under tolerance
route approval
reject change
source backup supplier
split supplier order
cancel PO
```

Example:

> “Supplier can deliver 80 now and 20 next week. This creates stockout risk for B2B order #554. I recommend sourcing the remaining 20 from backup supplier.”

Selene does not just nod at supplier changes like a bobblehead with procurement access.

---

## 20. Receiving Preparation

Every PO must prepare Receiving.

Procurement sends:

```text id="receiving_preparation"
supplier
PO number
expected items
expected quantities
expected delivery date
delivery location
receiver
backup receiver
inspection requirement
special handling
batch/serial/expiry requirements
documents required
```

This creates:

```text id="receiving_manifest_input"
Expected Delivery
Daily Receiving Manifest
Inspection Checklist
AP Matching Expectation
```

Selene says:

> “Delivery is expected tomorrow. Sarah is assigned as receiver. This product requires expiry and batch capture.”

This is how Receiving knows what is coming, instead of being surprised by pallets like a warehouse jump scare.

---

## 21. Purchase Commitment Handoff to Budget and Accounting

A PO creates a commitment.

Budget should know:

```text id="commitment_to_budget"
amount committed
cost center
department
supplier
expected period
capex/opex
budget line
```

Accounting may later use:

```text id="commitment_to_accounting"
commitment evidence
expected invoice
GRNI/accrual candidates
asset/capex classification hints
inventory valuation hints
```

Procurement does not post ledger.

It creates commitment truth.

Accounting posts after actual evidence.

Selene says:

> “This PO commits $12,400 of the maintenance budget, but no expense is posted until invoice/receiving rules are satisfied.”

Commitment is not actual spend. Another tiny distinction with giant reporting consequences.

---

## 22. Emergency Purchase Path

Emergency purchases must exist, but not become a fraud tunnel.

Emergency reasons:

```text id="emergency_reasons"
machine breakdown
safety issue
critical stockout
customer delivery failure risk
urgent legal/compliance need
storm/damage repair
IT/security incident
```

Emergency flow:

```text id="emergency_flow"
emergency reason captured
supplier checked if possible
budget/cashflow checked if possible
temporary authority route
PO created or retrospective PO flagged
receiving required
AP matching required
post-event review required
```

Selene says:

> “This is an emergency purchase. I can route urgent approval now, and I’ll schedule post-event review.”

Emergency does not mean “skip proof.”

Emergency means “move faster, audit harder.”

Put that on the procurement office wall, if it has survived the toner shortage.

---

## 23. Retrospective Purchase Orders

Retrospective POs are dangerous.

APQC even tracks the percentage of POs created after receipt of the supplier invoice as a procurement measure, because late POs undermine control. ([APQC][3])

Selene must flag:

```text id="retro_po_triggers"
supplier invoice arrived before PO
goods received before PO
emergency purchase
manual off-system purchase
contract renewal invoice without PO
```

Retrospective PO state:

```text id="retro_po_state"
Detected
ReasonRequired
EvidenceRequired
ApprovalRequired
CreatedAsException
Rejected
Closed
```

Selene says:

> “This invoice arrived without a matching PO. I’ll not allow normal payment until Procurement confirms whether this was authorized.”

Retrospective POs should be exceptions, not lifestyle.

---

## 24. Procurement Fraud and Control

Procurement must reduce fraud.

Fraud signals:

```text id="procurement_fraud_signals"
invoice without PO
PO created after invoice
split purchases to avoid approval
new supplier for urgent purchase
supplier bank change near payment
same requester repeatedly uses same supplier
PO price above contract
quantity above usual usage
duplicate supplier
fake supplier
supplier related to employee
unusual delivery address
manual override pattern
```

Selene actions:

```text id="procurement_fraud_actions"
hold PO
route review
require dual approval
notify Audit
block supplier
request evidence
force Receiving proof
block AP payment path
```

Selene says:

> “This purchase appears split into three POs below approval threshold. I’m routing to Finance review.”

Not every pattern is fraud.

But Selene should notice patterns humans pretend not to see because meetings are easier.

---

## 25. Procurement and Sustainability / Responsible Sourcing

Where company policy requires, Procurement must consider sustainability and responsible sourcing.

Checks may include:

```text id="sustainable_procurement_checks"
supplier due diligence
environmental certificate
human rights / labor risk
modern slavery risk
local sourcing preference
recycled content
carbon footprint
ethical sourcing
conflict mineral risk
waste/packaging impact
supplier code of conduct
```

ISO 20400 is guidance, not a certifiable management system standard, but it supports integrating sustainability into procurement decisions, which means Selene must be able to store and apply sustainability procurement criteria without forcing every tiny purchase through ESG theatre. ([ISO][1])

Selene says:

> “This supplier is cheaper but lacks required sourcing documentation. I recommend Supplier B for this category under your responsible procurement policy.”

Responsible procurement should be practical. Not a 41-page form for paper towels. Unless the paper towels are suspiciously political, in which case everyone has bigger problems.

---

## 26. Procurement and Product Relationship

Product provides:

```text id="product_to_procurement"
product_id
variant_id
supplier SKU
pack size
case quantity
MOQ
product status
channel readiness
compliance requirements
storage requirements
batch/serial/expiry requirements
```

Procurement uses this to buy the right item.

If Product data is incomplete:

```text id="product_incomplete_procurement"
PO may remain draft
human confirmation required
supplier clarification requested
Product update requested
```

Selene says:

> “This product is missing supplier SKU and pack size. I can draft the PO, but I need confirmation before sending it.”

Product truth prevents procurement from ordering “the blue one.” Suppliers love ambiguity because it becomes invoiceable.

---

## 27. Procurement and Inventory Relationship

Inventory provides:

```text id="inventory_to_procurement"
reorder need
stockout risk
recommended quantity
location demand
transfer alternative
JIT mode
safety stock target
cash tied in stock
expiry/overstock constraints
```

Procurement provides back:

```text id="procurement_to_inventory"
PO created
supplier selected
quantity ordered
expected delivery
supplier acknowledgement
supplier delay
PO cancelled
```

Selene says:

> “PO created for 240 units. Expected delivery is Friday. Inventory forecast updated to avoid stockout.”

Good. Inventory should know what Procurement did. Otherwise it screams forever.

---

## 28. Procurement and Supplier Relationship

Supplier Engine provides:

```text id="supplier_to_procurement"
approval status
risk status
performance score
lead time
payment terms
MOQ
open disputes
credit notes owed
bank safety
certificate status
alternative suppliers
```

Procurement provides:

```text id="procurement_to_supplier"
PO issued
PO changes
supplier selected frequency
supplier rejected reason
supplier change response
purchase volume
```

This creates supplier performance feedback.

Selene says:

> “Supplier ABC was not selected because unresolved credit notes remain open.”

Supplier should know why business is moving away. Polite pressure. Very satisfying.

---

## 29. Procurement and Receiving Relationship

Procurement prepares receiving.

Receiving returns:

```text id="receiving_to_procurement"
fully received
partially received
not arrived
short delivery
damaged goods
wrong item
over-delivery
inspection failed
replacement pending
credit note required
```

Procurement may respond:

```text id="procurement_response_receiving"
amend PO
cancel remaining quantity
request replacement
request credit
source backup supplier
close PO
open dispute
```

Selene says:

> “Only 90 of 100 units were accepted. I’ll leave the PO partially open and request replacement or credit under policy.”

Receiving tells reality. Procurement updates the order. AP later pays only the truth.

---

## 30. Procurement and AP Relationship

AP needs procurement proof.

Procurement provides:

```text id="procurement_to_ap"
PO
approved supplier
approved quantity
approved price
payment terms
approval proof
budget code
expected invoice
receiving requirement
inspection requirement
```

AP uses it for matching:

```text id="ap_matching"
2-way match: PO + invoice
3-way match: PO + receiving + invoice
4-way match: PO + receiving + inspection + invoice
```

Three-way matching compares purchase order, goods receipt, and invoice details before payment; CIPS describes it as a critical invoice-verification control, and modern AP tools describe the same PO/GR/invoice comparison as a core control. ([CIPS Download][2])

Selene says:

> “AP received an invoice with no matching PO. I’m blocking normal payment and routing to Procurement review.”

Invoice without buying proof is not payment-ready. It is a claim wearing a PDF costume.

---

## 31. Procurement and Cashflow Relationship

Procurement must consider liquidity.

Cashflow provides:

```text id="cashflow_to_procurement"
cash status
cash buffer
payment timing risk
supplier payment capacity
payroll/tax/rent protection
early-payment discount capacity
purchase deferral recommendation
```

Procurement may choose:

```text id="procurement_cashflow_options"
split order
delay order
negotiate terms
use supplier credit
transfer stock
use backup supplier
decline optional purchase
```

Selene says:

> “Cashflow is orange. I recommend ordering only the stock needed for the next 14 days and delaying non-critical items.”

Inventory might want more stock.

Cashflow might say no.

Procurement must mediate like a tired parent in a supermarket.

---

## 32. Procurement and Accounting Relationship

Procurement sends Accounting evidence, not journals.

Accounting handoff may include:

```text id="procurement_accounting_handoff"
approved commitment
cost center
budget code
capex/opex hint
asset candidate
inventory candidate
service candidate
tax category hint
supplier terms
expected invoice
audit ref
```

Accounting owns:

```text id="accounting_procurement_owns"
final classification
journal posting
accrual
GRNI
inventory asset
expense
capex
tax posting
```

Procurement does not post ledger.

It creates structured buying proof that Accounting can trust later.

---

## 33. Procurement and Asset / Capex

Asset purchases require special handling.

Triggers:

```text id="asset_procurement_triggers"
high-value equipment
vehicle
machinery
building improvement
IT hardware
capital project
fleet asset
lease candidate
intangible/software project
```

Procurement checks:

```text id="asset_procurement_checks"
capex budget
asset approval
insurance requirement
receiving/inspection requirement
asset custodian
warranty
serial/VIN
capitalization review
board threshold
```

Selene says:

> “This looks like a capital asset. I’ll route capex approval and require serial/condition capture during receiving.”

No capital asset should arrive without a custodian and accounting handoff, unless the business enjoys expensive mysteries.

---

## 34. Procurement and Contract Renewal

Procurement tracks contract-related purchases.

Contract triggers:

```text id="contract_triggers"
subscription renewal
maintenance contract renewal
lease renewal
supplier contract renewal
software renewal
insurance renewal
professional retainer
```

Selene checks:

```text id="contract_checks"
renewal date
notice period
price increase
usage
supplier performance
budget
cashflow
cancellation option
alternative supplier
```

Selene says:

> “This software contract renews in 21 days. Usage dropped 40% and price increased 18%. I recommend review before renewal.”

No auto-renewing bad contracts because nobody noticed the email. That email was the villain.

---

## 35. Procurement Analytics

Procurement must report:

```text id="procurement_reports"
spend by supplier
spend by category
spend by department
PO cycle time
POs created after invoice
manual PO rate
supplier acceptance rate
supplier change rate
budget exceptions
cashflow-blocked purchases
emergency purchases
retrospective POs
savings achieved
supplier consolidation opportunity
single-source dependency
maverick spend
```

Selene says:

> “22% of this month’s purchases were retrospective POs. That weakens AP control. I recommend tightening purchase-request policy.”

A business can only improve what Selene is willing to point at without flinching.

---

## 36. Automation and Exception-Only Review

Selene auto-handles:

```text id="procurement_auto_handles"
routine purchase request creation
routine PO creation under policy
supplier score check
budget check request
cashflow check request
approval routing
PO transmission
supplier acknowledgement capture
receiving assignment
expected delivery creation
routine reorder cycle
routine contract reminder
procurement analytics
```

Selene escalates:

```text id="procurement_escalates"
new supplier
restricted supplier
blocked supplier
budget exceeded
cashflow warning
high value
capex
contract commitment
emergency purchase
retrospective PO
supplier change outside tolerance
supplier bank risk
fraud signal
board threshold
manual override
```

Rule:

```text id="procurement_exception_rule"
Routine = Selene handles.
Exception = Selene routes.
Protected = authority approves.
Everything = audited.
```

No approval circus. No free-for-all. Just an adult purchasing system, which is apparently rare enough to celebrate quietly.

---

## 37. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text id="gpt_procurement_allowed"
summarize purchase request
draft supplier PO notes
explain supplier comparison
draft approval request
explain cashflow/budget tradeoff
summarize procurement exception
draft supplier change response
draft emergency purchase justification
prepare procurement review report
```

GPT-5.5 must not:

```text id="gpt_procurement_forbidden"
approve purchase
create protected PO without authority
invent supplier facts
override budget
override cashflow warning
ignore supplier restriction
accept supplier change without policy
change PO terms without approval
post accounting
execute payment
```

GPT-5.5 writes beautifully.

Selene controls the buying. Because apparently eloquence should not be allowed to spend $80,000.

---

## 38. Human-Like Selene Interaction

### Routine reorder

> “You usually order packing tape around now. Stock suggests seven days remaining. I can create the normal PO under policy.”

### Supplier choice

> “Supplier A is cheaper, but Supplier B is more reliable and will arrive before stockout. I recommend Supplier B.”

### Budget issue

> “This exceeds the department’s remaining budget by $1,200. I recommend budget transfer or delay.”

### Cashflow issue

> “We need the stock, but cashflow is tight. I recommend splitting the order.”

### Supplier change

> “Supplier changed delivery date. This may affect B2B order #881. I recommend checking backup supplier.”

### Retrospective invoice

> “This invoice arrived without a matching PO. I’ve blocked normal payment and opened Procurement review.”

This is how Selene should feel: helpful, firm, commercially literate, and allergic to invoice chaos.

---

## 39. Procurement State Machines

### Purchase Request State

```text id="purchase_request_state"
NeedDetected
Drafted
PendingSupplierEvaluation
PendingBudgetCheck
PendingCashflowCheck
PendingAuthorityCheck
Approved
Rejected
ConvertedToPO
Cancelled
Archived
```

### Purchase Order State

```text id="purchase_order_state"
Draft
PendingChecks
PendingApproval
Approved
Issued
SupplierAcknowledged
SupplierChangeRequested
InFulfillment
PartiallyReceived
FullyReceived
Closed
Cancelled
Disputed
Archived
```

### Supplier Change State

```text id="supplier_change_state"
NoChange
ChangeReceived
ToleranceChecking
ImpactReview
Accepted
Rejected
BackupSupplierRecommended
POAmended
Cancelled
Archived
```

### Emergency Purchase State

```text id="emergency_purchase_state"
EmergencyDetected
ReasonCaptured
UrgentApprovalRouting
TemporaryApprovalGranted
POIssued
PostEventReviewRequired
Reviewed
Closed
Archived
```

### Retrospective PO State

```text id="retrospective_po_state"
Detected
ReasonRequired
EvidenceRequired
AuthorityReview
ApprovedAsException
Rejected
Closed
Archived
```

---

## 40. Reason Codes

```text id="procurement_reason_codes"
PURCHASE_NEED_DETECTED
ROUTINE_REORDER_DETECTED
INVENTORY_REORDER_REQUIRED
PROCUREMENT_SUPPLIER_RECOMMENDED
SUPPLIER_RESTRICTED_WARNING
SUPPLIER_BLOCKED
SUPPLIER_OPEN_OBLIGATION_WARNING
BUDGET_CHECK_REQUIRED
BUDGET_WITHIN_LIMIT
BUDGET_EXCEEDED
CASHFLOW_CHECK_REQUIRED
CASHFLOW_WARNING
SPLIT_ORDER_RECOMMENDED
TRANSFER_INSTEAD_OF_PURCHASE_RECOMMENDED
AUTHORITY_CHECK_REQUIRED
PURCHASE_AUTO_APPROVED_UNDER_POLICY
PURCHASE_APPROVAL_REQUIRED
PO_CREATED
PO_ISSUED
PO_ACKNOWLEDGED
PO_SUPPLIER_CHANGE_REQUESTED
PO_PRICE_VARIANCE
PO_QUANTITY_VARIANCE
PO_DELIVERY_DATE_VARIANCE
PO_CANCELLED
PO_PARTIALLY_RECEIVED
PO_CLOSED
RETROSPECTIVE_PO_DETECTED
EMERGENCY_PURCHASE_PATH
FRAUD_SIGNAL_SPLIT_PURCHASE
RECEIVER_ASSIGNED
AP_MATCHING_PREPARED
```

---

## 41. Required Simulations

```text id="procurement_simulations"
routine reorder creates PO
inventory-triggered purchase request
JIT reorder creates split order
supplier comparison selects better supplier
supplier restricted blocks PO
supplier open credit note warns Procurement
budget within limit auto-approval
budget exceeded routes approval
cashflow warning splits order
transfer recommended instead of purchase
purchase authority limit exceeded
PO issued to Selene-connected supplier
supplier acknowledges PO
supplier changes delivery date
supplier changes price outside tolerance
supplier partial acceptance
backup supplier recommended
receiving manifest created from PO
AP invoice arrives without PO
retrospective PO exception
emergency purchase flow
capex purchase routes asset approval
contract renewal review
split purchase fraud signal
procurement analytics report
```

---

## 42. Integration Map

```text id="procurement_integration_map"
PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.SUPPLIER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.PROC.RECEIVE
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.CREDITORS.RECON
↔ PH1.ACCOUNTING
↔ PH1.BUDGET / SPEND_GOV
↔ PH1.CASHFLOW
↔ PH1.ASSET
↔ PH1.ASSET_ACCOUNTING
↔ PH1.FLEET
↔ PH1.INSURANCE
↔ PH1.LEGAL / CONTRACTS
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

## 43. Required Logical Packets

```text id="procurement_packets"
PurchaseNeedPacket
PurchaseRequestPacket
SupplierEvaluationRequestPacket
SupplierSelectionPacket
BudgetCheckRequestPacket
BudgetCheckResultPacket
CashflowCheckRequestPacket
CashflowCheckResultPacket
AuthorityCheckPacket
PurchaseApprovalPacket
PurchaseOrderPacket
PurchaseOrderLinePacket
POTransmissionPacket
POAcknowledgementPacket
SupplierPOChangePacket
POAmendmentPacket
POCancellationPacket
ReceivingExpectationPacket
ProcurementCommitmentPacket
EmergencyPurchasePacket
RetrospectivePOPacket
ProcurementFraudSignalPacket
ProcurementAuditEvidencePacket
```

Logical only. Codex maps later. No packet structs today, little schema goblin.

---

## 44. What Codex Must Not Do

```text id="codex_no_procurement"
Do not merge Procurement into Inventory.
Do not merge Procurement into Supplier.
Do not let Procurement receive goods.
Do not let Procurement validate supplier invoices.
Do not let Procurement execute payments.
Do not let Procurement post ledger.
Do not let GPT-5.5 approve purchases.
Do not create protected POs without authority.
Do not bypass Supplier risk.
Do not bypass Budget or Cashflow checks.
Do not ignore retrospective PO risk.
Do not auto-accept supplier changes outside tolerance.
Do not implement from this document alone.
```

---

## 45. Final Architecture Sentence

Selene Procurement + Purchase Order Engine is the autonomous buying-control brain that detects purchase needs from Inventory, Product, B2B, POS, E-commerce, production, assets, maintenance, contracts, and humans; evaluates supplier reliability, obligations, risk, budget, cashflow, authority, and urgency; creates, issues, amends, cancels, and tracks purchase orders; prepares Receiving, AP, Budget, Cashflow, and Accounting handoffs; prevents invoice fraud and retrospective purchasing from becoming normal; and uses GPT-5.5 to explain and draft procurement communication while deterministic Selene policy, Access, Supplier, Budget, Cashflow, Receiving, AP, and Audit preserve truth and control.

Simple version:

```text id="procurement_simple"
Selene sees the need.
Selene checks the supplier.
Selene checks budget.
Selene checks cashflow.
Selene checks authority.
Selene creates the PO.
Selene sends it.
Selene tracks supplier response.
Selene prepares Receiving.
Selene prepares AP matching.
Selene blocks invoice chaos.
Humans approve only real exceptions.
Everything is audited.
```

That is Global Document 70 — Procurement + Purchase Order Engine. It is the buying brain between “we need this” and “supplier wants money,” which is exactly where most companies accidentally let chaos enter wearing a purchase order number.

[1]: https://www.iso.org/standard/63026.html?utm_source=chatgpt.com "ISO 20400:2017 - Sustainable procurement — Guidance"
[2]: https://cips-download.cips.org/short-reads/from-requisition-to-payment-how-the-procure-to-pay-process-streamlines-procurement?utm_source=chatgpt.com "From requisition to payment: how the procure-to-pay ..."
[3]: https://www.apqc.org/what-we-do/benchmarking/open-standards-benchmarking/measures/percentage-purchase-orders-created?utm_source=chatgpt.com "Percentage of purchase orders created after receipt ..."
