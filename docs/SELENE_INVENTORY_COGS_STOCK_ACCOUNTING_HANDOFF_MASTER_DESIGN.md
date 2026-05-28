# Selene Inventory + COGS + Stock Accounting Handoff Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / INVENTORY + STOCK MOVEMENT + REORDER INTELLIGENCE + COGS + STOCK ACCOUNTING HANDOFF

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

PURPOSE:
Define Selene's inventory architecture: physical stock truth, stock movements, warehouses, batches, expiry, reservations, reorder intelligence, JIT/lean inventory, stock transfers, stocktaking, damages, write-offs, product-role intelligence, supplier/procurement/receiving boundaries, COGS handoff, stock valuation evidence, inventory-to-accounting handoff, and cashflow-aware replenishment.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 5. It defines future Inventory, COGS, stock accounting handoff, accepted-goods admission, reorder intelligence, JIT/lean/buffer/perishable modes, product role logic, valuation evidence, stocktake, shrinkage, expiry, returns, manufacturing/WIP, restaurant/recipe stock, and cashflow-aware replenishment. It does not implement Inventory, Product, Supplier, Procurement, Receiving, AP, Accounting, Cashflow, Tax, POS, Logistics, packets, migrations, tests, or runtime state.

Current repo truth does not prove complete runtime PH1.INVENTORY, PH1.PRODUCT, PH1.SUPPLIER, PH1.PROCUREMENT, PH1.PROC.RECEIVE, Tax/Compliance, or full Finance/Accounting ownership. This document is future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Executive Target

Selene Inventory must not merely count stock.

Old inventory system:

```text
stock goes down
human notices late
human orders too much or too little
supplier delays
cash is trapped in stock
dead stock sits in warehouse
popular items run out
accounting calculates COGS later
everyone blames "demand"
```

Selene Inventory:

```text
watches sales and stock in real time
forecasts demand
understands supplier lead times
knows what is reserved, available, damaged, expired, and incoming
recommends reorder quantity and timing
transfers stock before buying more
protects cashflow
uses JIT where safe and buffers where needed
knows which low-margin products drive customer habit
feeds COGS and stock value to Accounting
warns before stockouts, waste, overstock, and dead stock
```

Target:

```text
real-time stock truth
autonomous reorder intelligence
cashflow-aware replenishment
JIT / lean inventory where safe
strategic buffer where required
stock movement audit
batch / expiry / serial tracking
stocktake and variance control
accepted-goods-only stock admission
inventory valuation evidence
COGS handoff to Accounting
product role intelligence
supplier/procurement/receiving separation
```

Tiny human translation: Selene should not say "you have 12 left." Selene should say, "you have 12 left, you sell 8 per day, supplier takes 4 days, tomorrow is a public holiday, order today or customers will be disappointed."

## 2. Master Law

```text
Inventory owns physical stock truth.

Product owns item identity, SKU, description, pricing, variants, bundles, and commerce listing.

Accounting owns stock value, COGS, journals, and financial reporting.

Procurement owns purchase requests and purchase orders.

Receiving owns physical receipt, quantity check, inspection, accepted quantity, rejected quantity, and supplier resolution.

AP owns supplier invoice and payment readiness.

Cashflow owns whether the company can afford reorder timing.

Tax owns GST/VAT/claimability and tax treatment.

Selene must not merge Product, Inventory, Procurement, Receiving, AP, Accounting, or Cashflow into one blob.

No stock may be added as available inventory without receiving/acceptance evidence where policy requires it.

No COGS may be posted without source sale/stock movement evidence.

No stock write-off may occur without reason, authority, and audit.

No GPT-5.5 may invent stock truth, valuation, or COGS.
```

## 3. Owner Split

### PH1.INVENTORY Owns

```text
stock quantity
stock location
stock availability
batch / lot
serial tracking where required
expiry dates
stock reservation
stock transfer
stock adjustment
stocktake
stock health
stock damage
stock shrinkage
stock write-off request
stock reorder recommendation
stock movement history
inventory evidence handoff
```

### PH1.PRODUCT Owns

```text
product identity
SKU
barcode
description
attributes
variants
bundles
media
pricing presentation
channel listing
product status
sales catalog
product role candidate
```

Product says what the item is.

Inventory says where it physically is and how much exists.

### PH1.SUPPLIER Owns Future Supplier Truth

```text
supplier identity
supplier terms
supplier score
lead time
reliability
quality history
delivery history
price history
alternative supplier candidates
supplier risk
```

### PH1.PROCUREMENT Owns

```text
purchase request
reorder execution
supplier selection
purchase order
budget check before commitment
approval workflow
purchase commitment
```

### PH1.PROC.RECEIVE Owns

```text
goods receipt
delivery note
quantity received
quantity accepted
quantity rejected
damage check
quality check
missing items
wrong items
supplier dispute
photos/evidence
4-way match input
```

### AP Owns

```text
supplier bill
supplier invoice
payment terms
invoice matching
supplier credit note
refund/replacement claim
payment readiness
```

### Accounting Owns

```text
inventory asset value
COGS posting
stock write-off journal
inventory adjustment journal
valuation method
financial statement impact
inventory reserve/impairment accounting
```

### Tax / Compliance Owns

```text
GST/VAT on inventory purchases/sales
tax claimability
import/export tax
country rules
write-off tax treatment
inventory valuation tax rules
```

### Finance / Cashflow Owns

```text
cashflow-aware reorder timing
budget availability
stockholding cash impact
payment timing
inventory cash trap warnings
working capital impact
```

### POS / Commerce Owns

```text
sale event
refund event
return trigger
sales channel transaction
checkout evidence
```

### Logistics Owns

```text
dispatch
delivery
shipment
proof of delivery
return pickup
customer delivery issue
```

### PH1.D / GPT-5.5 May Assist

```text
forecast explanation
supplier issue summary
stock health summary
product role narrative
reorder recommendation explanation
receipt/document reading assistance
human-friendly stock warning wording
```

PH1.D / GPT-5.5 must not:

```text
create stock truth
approve reorder
approve write-off
decide final COGS
invent supplier lead time
invent stock quantity
execute purchase order
post accounting journal
```

## 4. Inventory Scope

Selene Inventory must support:

```text
raw materials
finished goods
work in progress
retail stock
restaurant ingredients
perishable goods
spare parts
maintenance stock
consumables
packaging materials
serialized items
batch/lot-controlled items
expiry-controlled items
dangerous/restricted goods where lawful
customer-reserved stock
B2B reserved stock
consignment stock
returned stock
damaged stock
quarantine stock
dead stock
stock in transit
stock held at supplier
stock held at warehouse
stock held at branch/store
stock held by technician/vehicle
stock held by third-party logistics provider
```

Not all inventory is equal. A cake, a forklift spare part, a handbag, and a steel coil need different rules.

## 5. Product Vs Inventory Separation

This must be explicit.

```text
PH1.PRODUCT = what the item is.

PH1.INVENTORY = how much physically exists, where it is, and what condition it is in.

Accounting = what it is worth.

POS/E-Commerce/B2B = how it is sold.

Procurement = how it is ordered.

Receiving = what actually arrived.

AP = what supplier billed.

Logistics = what was shipped/delivered.
```

Example:

```text
Product:
5L Motor Oil
SKU: MO-5L
Sell price: AUD 42
Tax category: standard
Listing: retail + B2B

Inventory:
North Warehouse has 42 units
10 reserved for B2B orders
5 damaged
reorder point 20
supplier lead time 4 days

Accounting:
stock value and COGS handoff

POS/E-Commerce:
sale event reduces available stock
```

What must not happen:

```text
no POS owning product truth
no Accounting owning physical stock truth
no Product owning warehouse quantity
no Inventory owning final selling price policy
no AP invoice creating accepted stock without Receiving proof
```

## 6. Inventory Data Model

### Inventory Item Record

```text
inventory_item_id
product_id
company_id
legal_entity_id
location_id
warehouse_id
stock_status
quantity_on_hand
quantity_available
quantity_reserved
quantity_allocated
quantity_in_transit
quantity_damaged
quantity_quarantined
quantity_expired
quantity_on_order
reorder_point
reorder_quantity
safety_stock
inventory_mode
valuation_method_ref
supplier_refs
last_stocktake_at
audit_ref
```

### Stock Location Fields

```text
location_id
location_type
warehouse_id
store_id
branch_id
vehicle_id
technician_id
third_party_location_ref
country
region
storage_capacity
temperature_controlled
hazard_flag
active_status
audit_ref
```

### Batch / Lot Fields

```text
batch_id
product_id
supplier_id
received_date
manufacture_date
expiry_date
lot_number
quantity_received
quantity_remaining
quality_status
recall_status
audit_ref
```

### Serial Item Fields

```text
serial_id
product_id
serial_number
asset_candidate_flag
location_id
status
warranty_ref
customer_order_ref
audit_ref
```

## 7. Inventory Statuses

```text
Available
Reserved
Allocated
InTransit
Incoming
InspectionHold
Quarantine
Damaged
Expired
ReturnedPendingInspection
SupplierDispute
CustomerReturnHold
DeadStock
DiscontinueCandidate
WriteOffPending
WrittenOff
Archived
```

Inventory status must be distinct from Product status.

Product can be active while one batch is expired.

Product can be discontinued while stock still exists.

## 8. Stock Movement Types

Every inventory movement must be recorded.

```text
purchase_receipt
receipt_adjustment
sales_issue
B2B_order_allocation
ecommerce_order_allocation
POS_sale
customer_return
supplier_return
warehouse_transfer
branch_transfer
vehicle_stock_transfer
production_issue
production_receipt
assembly_build
disassembly
stocktake_adjustment
damage_adjustment
expiry_writeoff
shrinkage
theft_loss
sample/giveaway
internal_use
repair_parts_use
consignment_movement
recall_hold
quarantine_release
```

Stock movement fields:

```text
stock_movement_id
movement_type
product_id
inventory_item_id
location_from
location_to
quantity
unit_of_measure
batch_id
serial_id
source_owner
source_document_ref
reason_code
cost_value_ref
created_by
approved_by
audit_ref
created_at
```

Hard law:

```text
No stock quantity changes without a stock movement event.
```

## 9. Goods Receiving And Accepted Stock

Inventory may only increase accepted available stock from accepted receiving evidence.

Flow:

```text
purchase order approved
→ goods arrive
→ Receiving records quantity delivered
→ Inspection records accepted / damaged / rejected / missing
→ Inventory increases accepted stock only
→ damaged/rejected stock goes to hold/quarantine/dispute
→ AP uses accepted quantity for invoice/payment release
→ Accounting posts inventory/AP based on accepted goods
```

Accepted goods rule:

```text
Only accepted goods become available inventory.

Damaged, wrong, missing, rejected, or uninspected goods must not become available stock.
```

Example:

```text
PO ordered 100.
Delivered 95.
Accepted 90.
Damaged 5.
Missing 5.

Inventory available increases by 90.
Damaged 5 enters hold.
Missing 5 enters supplier resolution.
AP may pay only accepted quantity unless authorized override passes.
```

## 10. Inventory Reservations And Allocation

Inventory must support reservation before physical dispatch.

Reservation sources:

```text
customer order
B2B order
e-commerce order
POS layby/hold
production order
service job
project requirement
internal transfer
contract customer allocation
promotion campaign
```

Reservation fields:

```text
reservation_id
product_id
quantity_reserved
location_id
customer_id optional
order_id optional
priority
expiry_at
reservation_status
release_rule
audit_ref
```

Statuses:

```text
Reserved
Allocated
Released
Expired
ConvertedToDispatch
Cancelled
Backordered
```

If reservation expires, stock returns to available.

Selene should not oversell unless business rules explicitly allow backorders.

## 11. Backorders And Stockouts

Backorder support depends on company policy.

Backorder fields:

```text
backorder_id
product_id
customer_id
order_ref
quantity
expected_availability_date
supplier_or_production_ref
customer_promised_date
risk_status
audit_ref
```

Stockout prevention:

```text
forecast demand
compare available + incoming - reserved
check supplier lead time
check cashflow
trigger reorder or production
warn sales if stockout likely
stop accepting unrealistic promises
```

Selene says:

```text
At the current sales pace, this product will stock out in three days. Supplier lead time is five days, so I recommend ordering today or limiting new B2B commitments.
```

## 12. Inventory Modes

Selene should classify each item into an inventory mode.

```text
JIT_Mode
Lean_Buffer_Mode
Normal_Reorder_Mode
Seasonal_Surge_Mode
Strategic_Reserve_Mode
Perishable_Same_Day_Mode
Make_To_Order_Mode
Discontinue_Exit_Mode
```

### JIT Mode

```text
Use where supplier reliability is high, demand is predictable, storage cost matters, and stockout risk is manageable.
```

### Lean Buffer Mode

```text
Keep small buffer because demand or supplier timing has moderate uncertainty.
```

### Strategic Reserve Mode

```text
Keep higher stock for critical parts, essential materials, safety items, or business-continuity goods.
```

### Perishable Same-Day Mode

```text
Use for fresh food, cakes, prep items, flowers, perishables, and items with short shelf life.
```

### Make-To-Order Mode

```text
Use when inventory is created after order or customer specification.
```

### Discontinue / Exit Mode

```text
Use for dead stock, low-value slow movers, obsolete products, or strategic exit candidates.
```

## 13. Reorder Intelligence

Selene's reorder logic must consider more than "below minimum."

Inputs:

```text
forecast demand
sales velocity
supplier lead time
lead time variability
supplier reliability
demand variability
service level target
seasonality
public holidays
local events
weather where relevant
current stock
reserved stock
incoming stock
expiry/shelf life
cashflow
storage capacity
supplier score
product margin
product role
customer dependency
stockout penalty
minimum order quantity
bulk discount
budget availability
```

Basic logic:

```text
recommended_reorder =
forecast demand during lead time
+ safety stock
- available stock
- incoming stock
+ reserved/allocated demand adjustment
```

Advanced logic:

```text
recommended quantity and timing must balance:
availability
cashflow
storage cost
expiry risk
supplier risk
margin
customer impact
working capital
```

Selene says:

```text
I recommend ordering 340 units, not 500. That covers expected demand and supplier delay risk without tying up unnecessary cash.
```

## 14. Cashflow-Optimized Inventory

Inventory must not trap cash unnecessarily.

Before reorder, Selene must check:

```text
cash available
upcoming payroll
tax payments
supplier payment terms
expected customer receipts
storage cost
stockholding cost
expiry risk
gross margin
sales forecast confidence
budget
cash reserve
profit floor
```

If cash is tight, Selene may recommend:

```text
smaller order
split order
delayed second order
supplier with faster delivery
internal stock transfer
B2B clearance of slow stock
promotion to convert old stock into cash
delay non-critical stock
reorder only critical SKUs
```

Example:

```text
Ordering 1,000 units protects availability but reduces cash below payroll buffer.
Selene recommends 600 units now and 400 after receivables clear.
```

Selene says:

```text
The full reorder is safe for stock but tight for cash. I recommend 600 units now and a second order after expected customer receipts clear.
```

## 15. Product Role Intelligence

Inventory should not judge products only by margin.

Selene must classify products by business role.

```text
Hero_Product
Profit_Driver
Traffic_Driver
Habit_Builder
Bundle_Support_Product
Seasonal_Product
Strategic_Product
Slow_Mover
Dead_Stock
Discontinue_Candidate
```

### Hero Product

```text
High customer demand, brand importance, or core sales role.
```

### Profit Driver

```text
Strong margin and meaningful sales volume.
```

### Traffic Driver

```text
May have low margin but brings customers in.
```

### Habit Builder

```text
Creates repeat customer behavior.
```

### Bundle Support Product

```text
Often bought with higher-margin products.
```

### Slow Mover

```text
Low velocity but not necessarily dead.
```

### Dead Stock

```text
Not moving, cash trapped, space wasted, likely discount/write-off/discontinue candidate.
```

Product role fields:

```text
product_id
product_role
margin
velocity
basket_attachment_rate
repeat_purchase_rate
customer_visit_impact
B2B_dependency
seasonality
stockholding_cost
expiry_risk
recommendation
audit_ref
```

## 16. Low-Margin Customer Habit Products

Some low-margin products are still valuable.

Examples:

```text
milk
bread
coffee
popular cakes
basic salon product
printer toner
common spare part
low-margin grocery item
frequently ordered B2B consumable
```

Selene must measure:

```text
repeat customer visits
basket attachment
cross-sell rate
customer lifetime value
average order value after purchase
frequency lift
customer retention effect
B2B reorder dependency
brand trust effect
```

Selene says:

```text
This product has low margin, but customers who buy it often add higher-margin items. I recommend keeping it as a traffic driver while controlling waste tightly.
```

Hard rule:

```text
Do not discontinue a low-margin product without checking customer habit, basket attachment, B2B dependency, and strategic role.
```

## 17. Restocking Automation Frequency

Restocking must be event-driven and scheduled.

### Event Triggers

```text
sale occurs
B2B order arrives
e-commerce order arrives
supplier delay reported
stock received
stock damaged
stock expires soon
promotion scheduled
public holiday detected
weather/event signal changes
large customer quote appears
cashflow changes
product velocity changes
inventory transfer occurs
stocktake variance detected
```

### Review Cadence

```text
perishable / fresh / high velocity: continuous or hourly
restaurant prep items: daily before prep + intraday adjustment
A-class high-value / high-impact items: daily
fast-moving retail: daily
B-class normal items: 2–3 times per week or weekly
C-class slow movers: weekly or monthly
dead stock: monthly liquidation/discontinue review
critical parts: daily risk check even if low volume
seasonal items: increased review before and during season
```

Simple law:

```text
Fast, risky, perishable, expensive, or critical stock gets frequent review.

Slow, cheap, predictable stock gets less frequent review.
```

## 18. JIT Cakes / Perishable Production Example

Selene should know what to prepare before humans ask.

Inputs:

```text
last Tuesday sales
last 4 Tuesdays average
same public holiday history
weather
local events
school holidays
pre-orders
walk-in trends
delivery app traffic
staff availability
oven capacity
ingredient availability
shelf life
waste tolerance
target sell-out time
profit margin
stockout penalty
```

Example plan:

```text
Normal Tuesday cake sales: 40
Public holiday uplift: +35%
Rainy weather uplift: +10%
Pre-orders: 8
Waste tolerance: low

Recommended production: 65 cakes

Bake plan:
- 40 before opening
- 15 at 11 AM if sales pace confirms
- 10 at 2 PM only if demand remains high
```

Selene says:

```text
Today is a public holiday and cake demand usually rises. I recommend baking 40 now and holding ingredients for a second batch at 11 AM. This reduces waste if demand is weaker than forecast.
```

## 19. Staged Replenishment And Production

Selene should use staged action where demand is uncertain.

For perishables:

```text
produce base quantity
watch live sales
trigger second batch if sales pace supports it
discount late-day leftovers
stop production before waste risk
```

For retail:

```text
hold lean stock
trigger reorder before lead-time risk
use supplier score
split order if cashflow tight
transfer stock before buying more
```

For manufacturing:

```text
order raw materials based on confirmed and forecasted orders
reserve critical materials
avoid overproduction
sequence production by due date and margin
```

For B2B:

```text
reserve stock for account customers
accept backorders only if supplier/production can meet promise
prioritize high-value or contract customers where policy allows
```

## 20. Stock Transfers Before Buying

Selene should check whether existing stock can be moved before new purchases.

Transfer logic:

```text
location A low stock
→ check nearby branches/warehouses
→ check reserved status
→ check transfer cost
→ check delivery time
→ check demand risk at source location
→ compare to supplier reorder
→ recommend transfer or purchase
```

Selene says:

```text
North Warehouse is low, but South Warehouse has surplus. Transferring 40 units is cheaper and faster than buying more.
```

## 21. Supplier Lead Time And Reliability

Inventory must understand supplier behavior.

Supplier inputs:

```text
average lead time
lead time variability
on-time delivery rate
short delivery rate
damage rate
wrong item rate
quality score
price stability
minimum order quantity
bulk discount
payment terms
credit note/refund behavior
```

Supplier reliability affects:

```text
safety stock
reorder timing
preferred supplier
backup supplier
cashflow timing
purchase recommendation
supplier risk warning
```

Selene says:

```text
Supplier A is cheaper but has frequent delays. Supplier B costs 4% more but delivers reliably. For this critical item, I recommend Supplier B.
```

## 22. Inventory And Procurement Boundary

Inventory recommends reorder.

Procurement creates and approves purchase orders.

Flow:

```text
Inventory detects reorder need
→ Inventory creates ReorderRecommendationPacket
→ Cashflow/Budget check affordability and policy
→ Procurement prepares PO
→ Access/Authority validates purchase
→ Receiving accepts goods
→ Inventory updates accepted stock
```

Hard law:

```text
Inventory must not directly create committed purchase orders unless Procurement policy explicitly permits automated reorder within pre-approved limits.
```

Automated reorder may be allowed only if:

```text
item is approved for auto-reorder
supplier is approved
quantity within policy
budget available
cashflow safe
no price anomaly
no supplier risk flag
audit enabled
```

## 23. Inventory And Receiving Boundary

Receiving proves what arrived.

Inventory updates only from accepted evidence.

Receiving evidence includes:

```text
purchase_order_ref
delivery_note_ref
received_quantity
accepted_quantity
damaged_quantity
rejected_quantity
wrong_items
missing_items
photos/evidence
inspector_id
inspection_status
```

If goods are damaged:

```text
do not mark available
status = Damaged / SupplierDispute / Quarantine
supplier resolution opened
AP payment adjusted or held
Accounting waits for final treatment
```

## 24. Inventory And AP Boundary

AP matches supplier invoice to accepted goods.

Inventory provides accepted quantity/value evidence.

AP owns invoice/payment.

```text
Supplier invoice arrives
→ AP validates bill
→ AP requests accepted quantity from Receiving/Inventory
→ AP matches PO + receipt + inspection + invoice
→ AP pays accepted quantity only unless override
```

Inventory does not approve payment.

## 25. Inventory And Accounting Boundary

Inventory provides movement and quantity evidence.

Accounting posts financial effect.

Inventory accounting handoff types:

```text
inventory_purchase_receipt
sales_COGS
stock_adjustment
stock_writeoff
damage_writeoff
expiry_writeoff
stock_transfer
inventory_revaluation
manufacturing_consumption
finished_goods_receipt
customer_return_restock
supplier_return
```

### Purchase Receipt Journal

```text
Debit: Inventory
Debit: GST/VAT Receivable if claimable
Credit: Accounts Payable
```

### Sale / COGS Journal

```text
Debit: Cost of Goods Sold
Credit: Inventory
```

### Write-Off Journal

```text
Debit: Inventory Shrinkage / Waste / Write-Off Expense
Credit: Inventory
```

### Return To Stock

```text
Debit: Inventory
Credit: COGS / Returns Adjustment depending policy
```

Accounting owns posting.

Inventory owns physical evidence.

## 26. COGS Handoff

COGS must be triggered by sale or consumption evidence.

COGS sources:

```text
POS sale
e-commerce sale
B2B shipment
customer delivery
manufacturing consumption
internal use
stock write-off
service job material use
```

COGS handoff packet:

```text
COGSHandoffPacket:
  cogs_handoff_id
  source_owner
  source_event_ref
  product_id
  inventory_item_id
  quantity
  valuation_method_ref
  unit_cost
  total_cost
  location_id
  batch_id
  accounting_period_id
  audit_ref
```

Hard law:

```text
No COGS posting without sale/issue/consumption evidence and valuation method.
```

## 27. Inventory Valuation Methods

Accounting owns valuation method.

Inventory must provide required quantity/movement evidence.

Possible valuation methods:

```text
FIFO
weighted_average
specific_identification
standard_cost
actual_cost
batch_cost
serial_specific_cost
manufacturing_cost_rollup
```

Selene must not switch valuation method casually.

Changing valuation method may require:

```text
accounting approval
tax review
board/finance policy
effective date
audit
comparative impact
```

Inventory must preserve movement history so valuation can be calculated.

## 28. Stocktaking

Selene must support stocktakes.

Stocktake types:

```text
full_stocktake
cycle_count
spot_check
high_value_count
expiry_count
perishable_close_count
warehouse_transfer_count
store_close_count
serial_number_audit
```

Stocktake fields:

```text
stocktake_id
location_id
product_ids
counted_by
counted_at
system_quantity
counted_quantity
variance
variance_reason_candidate
approval_required
adjustment_status
audit_ref
```

Variance statuses:

```text
NoVariance
MinorVariance
RequiresReview
ApprovedAdjustment
RejectedAdjustment
ShrinkageInvestigation
TheftSuspected
WriteOffPending
```

Selene says:

```text
The count is short by 8 units. I’ll open a variance review before adjusting inventory.
```

## 29. Shrinkage, Damage, Expiry, Waste

Inventory must handle losses.

Reasons:

```text
damage
expiry
spoilage
theft
loss
breakage
quality failure
supplier defect
customer return damage
internal use
production waste
count variance
```

Loss flow:

```text
loss detected
→ evidence captured
→ reason assigned
→ authority check
→ write-off request
→ Accounting/Tax treatment
→ inventory quantity adjusted
→ audit
```

Perishable example:

```text
10 cakes expired unsold.
Selene records waste.
Accounting posts write-off.
Forecast adjusts next production recommendation.
```

Selene says:

```text
These items are expired and cannot be sold. I’ll record the write-off and adjust the next reorder forecast so we do not repeat the waste.
```

## 30. Expiry And Batch Control

Expiry-sensitive goods need special control.

Applies to:

```text
food
medicine
cosmetics
chemicals
batteries where relevant
safety equipment
regulated goods
perishable raw materials
```

Selene must track:

```text
expiry_date
batch_id
received_date
supplier_id
quantity_remaining
sell_by_date
use_by_date
recall_status
quarantine_status
```

Selene must warn:

```text
expiry approaching
discount before expiry
transfer to faster-moving location
use in production first
stop sale after expiry
write-off
supplier claim if short-dated
```

## 31. Returns And Restocking

Customer returns must be inspected before restocking.

Flow:

```text
customer return initiated
→ Logistics/Customer/POS records return reason
→ Receiving/Inventory inspects condition
→ accepted for resale / repair / quarantine / write-off
→ AR/refund/credit note proceeds if approved
→ Accounting adjusts stock/COGS/refund
```

Inventory must not put returned goods back into available stock without inspection.

## 32. Manufacturing / WIP Inventory

Manufacturing inventory requires stages.

Inventory types:

```text
raw_material
work_in_progress
finished_goods
scrap
rework
byproduct
```

Manufacturing flow:

```text
production order created
→ raw materials reserved
→ raw materials issued to production
→ WIP updated
→ finished goods received
→ scrap/waste recorded
→ COGS/cost rollup prepared
```

Manufacturing cost inputs:

```text
raw material cost
labor cost
overhead allocation
machine time
scrap
rework
subcontractor cost
```

Manufacturing engine may be standalone later, but Inventory must support WIP/finished-goods handoff.

## 33. Restaurant / Recipe / Raw Material Inventory

Restaurant inventory needs recipe logic.

Inputs:

```text
menu item
recipe
ingredient quantities
prep batches
expiry
waste
sales forecast
staff prep capacity
supplier delivery
kitchen stock
```

Flow:

```text
menu item sold
→ ingredients consumed
→ raw material inventory reduced
→ COGS handoff prepared
→ reorder/prep forecast updated
```

Example:

```text
Cake recipe uses:
flour
sugar
eggs
cream

Cake sale reduces ingredient inventory based on recipe quantities.
```

## 34. Inventory And Product Lifecycle

Product status affects inventory decisions.

Product statuses may include:

```text
Draft
Active
Seasonal
Discontinued
ExitMode
Blocked
Recall
Archived
```

Inventory response:

```text
active product → normal stock rules
seasonal product → seasonal reorder rules
discontinued product → no reorder, clearance plan
recall product → quarantine/stop sale
exit mode → markdown/transfer/B2B clearance
```

Product owns product status.

Inventory acts on it.

## 35. Dead Stock And Clearance

Selene must detect dead stock.

Dead stock signals:

```text
no sales over period
low velocity
high holding cost
high storage cost
expiry risk
obsolete product
low margin
low basket attachment
repeated returns
supplier discontinuation
```

Actions:

```text
discount
bundle
transfer
B2B clearance
return to supplier
donate if allowed
write off
discontinue
replace with better product
```

Selene says:

```text
This product has not sold in 90 days and takes up storage. I recommend clearing it through B2B discount before writing it off.
```

## 36. Inventory Health Metrics

Selene must track inventory health.

Metrics:

```text
stockout_risk
overstock_risk
days_of_cover
sales_velocity
inventory_turnover
gross_margin_return_on_inventory
dead_stock_value
expiry_risk_value
slow_mover_value
reserved_stock_ratio
supplier_delay_risk
stock_accuracy
shrinkage_rate
cash_tied_in_stock
```

Selene says:

```text
Inventory health is mostly good, but cash tied in slow-moving stock is rising. I recommend clearing three product lines before ordering more in that category.
```

## 37. Inventory And Cashflow

Inventory must report working-capital impact.

Cashflow needs:

```text
planned reorder cash impact
stockholding cost
slow stock cash trapped
dead stock liquidation opportunity
reorder timing
supplier payment terms
cash reserve risk
inventory purchase commitments
```

Selene should avoid:

```text
buying stock that protects availability but harms payroll, tax, reserve, or critical payments.
```

Selene says:

```text
This reorder improves availability but ties up AUD 40,000. Cashflow is tight, so I recommend split ordering and clearing slow stock first.
```

## 38. Inventory And Budget

Budgeting owns spend control.

Inventory must ask Budget before reorders where policy requires.

Flow:

```text
Inventory recommends reorder
→ Budget checks category/department/project budget
→ Cashflow checks affordability
→ Procurement prepares PO
```

If budget pressure exists:

```text
Selene searches:
- transfer stock
- reduce order quantity
- delay second order
- switch supplier
- use substitute product
- clear dead stock
```

## 39. Inventory And Reporting

Full reporting belongs to future Reporting design.

Inventory provides report-ready data:

```text
stock on hand
stock value
COGS evidence
stockout risk
overstock risk
dead stock
expiry risk
reorder forecast
supplier performance
stock movement history
inventory turnover
cash tied in stock
gross margin by product
product role classification
```

Presentation belongs to Reporting/Device layers.

Inventory provides truth.

## 40. Access And Authority

Protected inventory actions:

```text
approve stock write-off
approve stock adjustment
override reorder recommendation
approve auto-reorder rule
approve supplier switch
approve stock transfer above threshold
approve release from quarantine
approve damaged stock sale
approve inventory valuation method change
approve stocktake adjustment
approve discontinuation
approve major markdown
approve stock reserve release for another customer
export stock reports
```

Authority depends on:

```text
role
warehouse/location
product category
stock value
risk level
regulated goods
customer impact
supplier impact
cashflow impact
accounting impact
tax impact
```

Step-up may be required for:

```text
high-value write-off
large stock adjustment
regulated goods release
valuation method change
major markdown
inventory export
```

## 41. PH1.D / GPT-5.5 Role

Allowed:

```text
explain reorder recommendation
summarize stock health
draft supplier issue summary
explain dead stock recommendation
draft human-friendly stock warning
summarize demand drivers
draft clearance campaign wording through proper owner
suggest likely product role candidates
```

Forbidden:

```text
create final stock quantity
approve reorder
approve write-off
approve stock adjustment
determine final COGS
invent supplier lead time
invent sales forecast truth
post journal
execute purchase order
```

## 42. PH1.WRITE Wording

PH1.WRITE owns final user-facing wording.

Examples:

### Reorder

```text
I recommend ordering 340 units today. That covers expected demand, supplier delay risk, and the public holiday uplift without tying up extra cash.
```

### Stockout

```text
This item may stock out in three days. Supplier lead time is five days, so we need to order today or limit new commitments.
```

### Dead Stock

```text
This product has not moved in 90 days and is tying up cash. I recommend a clearance offer before writing it off.
```

### Damaged Goods

```text
These items were received damaged. I’ve kept them out of available stock and opened a supplier resolution case.
```

## 43. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
product_id
inventory_item_id
location_id
quantity_old
quantity_new
movement_type
source_owner
source_document_ref
receiving_ref
inspection_ref
stocktake_ref
writeoff_ref
COGS_handoff_ref
valuation_ref
approval_refs
step_up_refs
timestamp
company_id
legal_entity_id
country
currency
reason_code
```

No silent stock changes.

No unaudited write-offs.

No hidden valuation changes.

## 44. Failure Branches

### Stock Quantity Unknown

```text
Selene marks stock confidence low.
Stocktake or verification required.
```

### Receiving Missing

```text
Incoming goods cannot become available.
AP payment may be held.
```

### Inspection Failed

```text
Stock goes to hold/quarantine/dispute.
No available stock increase.
```

### Reorder Cashflow Unsafe

```text
Selene proposes split order, transfer, delay, or clearance before PO.
```

### Supplier Delay

```text
Forecast updates.
Safety stock/reorder timing recalculated.
Customer/order promises checked.
```

### Expired Stock

```text
Stop sale.
Write-off or disposal flow.
Forecast adjusts.
```

### COGS Valuation Missing

```text
Accounting handoff blocked until valuation method/evidence exists.
```

### Ambiguous Product Mapping

```text
Inventory cannot update product stock until Product owner resolves identity.
```

## 45. Required Logical Packets

Future logical packets:

```text
InventoryItemPacket
StockLocationPacket
StockBatchPacket
SerialStockPacket
StockMovementPacket
InventoryStatusPacket
GoodsAcceptedInventoryAdmissionPacket
StockReservationPacket
BackorderPacket
InventoryModePacket
ReorderRecommendationPacket
CashflowAwareReorderPacket
ProductRoleIntelligencePacket
TrafficDriverProductPacket
RestockingCadencePacket
PerishableProductionPlanPacket
StagedReplenishmentPacket
StockTransferRecommendationPacket
SupplierLeadTimeRiskPacket
ProcurementReorderHandoffPacket
ReceivingInventoryHandoffPacket
APInventoryMatchHandoffPacket
InventoryAccountingHandoffPacket
COGSHandoffPacket
InventoryValuationEvidencePacket
StocktakePacket
StockVariancePacket
InventoryLossPacket
ExpiryBatchControlPacket
CustomerReturnInspectionPacket
ManufacturingInventoryHandoffPacket
RecipeIngredientConsumptionPacket
DeadStockRecommendationPacket
InventoryHealthPacket
InventoryCashflowImpactPacket
InventoryBudgetCheckPacket
InventoryAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 46. Example — Retail Reorder

```text
Product:
5L Motor Oil

Current available stock:
12 units

Sales velocity:
8 per day

Supplier lead time:
4 days

Public holiday tomorrow:
expected demand +35%

Cashflow:
safe for 60 units, not safe for 100

Selene recommends:
order 60 units today
```

Selene says:

```text
You have 12 units left and demand will rise tomorrow. Supplier lead time is four days. I recommend ordering 60 units today, not 100, because that protects availability without tying up unnecessary cash.
```

## 47. Example — Transfer Before Purchase

```text
North Warehouse:
low stock

South Warehouse:
surplus stock

Supplier:
5-day lead time

Transfer:
1-day internal transfer
```

Selene says:

```text
I recommend transferring stock from South Warehouse instead of buying more. It is faster, cheaper, and avoids tying up cash in extra inventory.
```

## 48. Example — Low-Margin Habit Product

```text
Product:
Milk

Margin:
low

Customer behavior:
weekly repeat visits
high basket attachment
customers often buy higher-margin items with it
```

Selene says:

```text
Milk has low margin, but it brings customers back and drives higher-margin basket purchases. I recommend keeping it as a traffic driver while controlling waste tightly.
```

## 49. Example — Damaged Goods

```text
PO:
100 units

Delivered:
100 units

Inspection:
90 accepted
10 damaged

Inventory:
90 available
10 damaged hold

AP:
pay only accepted quantity unless override

Supplier:
resolution case opened
```

Selene says:

```text
Ten units arrived damaged, so I’ve kept them out of available stock and opened a supplier resolution case. AP can release payment only for the accepted quantity unless an authorized override is approved.
```

## 50. Example — Perishable Cake Production

```text
Normal Tuesday sales:
40 cakes

Public holiday uplift:
35%

Rain:
10% uplift

Pre-orders:
8

Recommendation:
65 cakes in staged batches
```

Selene says:

```text
I recommend baking 40 cakes now, then another 15 at 11 AM if sales pace confirms, and 10 more at 2 PM only if demand stays strong. This protects sales while reducing waste.
```

## 51. What Must Not Happen

```text
no Product and Inventory merge
no POS owning Product or Inventory truth
no Accounting owning physical stock truth
no AP invoice creating available stock without Receiving acceptance
no available stock increase without accepted receiving evidence where required
no COGS posting without sale/issue/consumption evidence
no stock adjustment without movement event
no stock write-off without reason, authority, and audit
no low-margin product discontinued without checking habit/basket/customer role
no reorder that ignores cashflow
no JIT mode applied blindly to unreliable suppliers or critical goods
no expired/damaged/rejected goods sold as available stock
no returned goods restocked without inspection
no supplier lead time invented by GPT-5.5
no PH1.D final stock quantity or COGS truth
no valuation method change without Accounting/Tax approval
no hidden stocktake variance
no implementation from this document alone
```

## 52. Future Simulation Targets

```text
SIM_INV_001_product_inventory_separation
SIM_INV_002_goods_receipt_accepted_stock_only
SIM_INV_003_damaged_goods_supplier_resolution
SIM_INV_004_reorder_recommendation_with_lead_time
SIM_INV_005_cashflow_unsafe_reorder_split_order
SIM_INV_006_transfer_before_purchase
SIM_INV_007_low_margin_habit_product_kept
SIM_INV_008_dead_stock_clearance_recommendation
SIM_INV_009_perishable_cake_staged_production
SIM_INV_010_stocktake_variance_review
SIM_INV_011_expired_batch_stop_sale_writeoff
SIM_INV_012_POS_sale_triggers_COGS_handoff
SIM_INV_013_customer_return_requires_inspection
SIM_INV_014_manufacturing_raw_material_to_finished_goods
SIM_INV_015_recipe_sale_consumes_ingredients
SIM_INV_016_auto_reorder_blocked_by_budget
SIM_INV_017_supplier_delay_recalculates_safety_stock
SIM_INV_018_B2B_reserved_stock_prevents_oversell
SIM_INV_019_stock_writeoff_requires_authority
SIM_INV_020_inventory_valuation_missing_blocks_COGS
```

## 53. Related Addendum

Autonomous JIT/reorder intelligence, product role logic, low-margin habit products, restocking cadence, staged replenishment, and inventory operating autonomy modes are further strengthened in SELENE_INVENTORY_AUTONOMOUS_JIT_REORDER_PRODUCT_ROLE_AUTONOMY_ADDENDUM.md and must be read with this document.

## 54. Final Architecture Sentence

Selene Inventory + COGS + Stock Accounting Handoff is the governed stock-truth and inventory-intelligence layer: it keeps physical stock accurate by product, location, batch, status, and movement; admits only accepted goods into available inventory; forecasts demand and reorder needs using lead time, supplier reliability, cashflow, seasonality, product role, expiry, and customer behavior; uses JIT where safe and buffers where needed; manages transfers, stocktakes, damage, expiry, returns, manufacturing, recipes, and dead stock; hands COGS and inventory valuation evidence to Accounting; and preserves clean boundaries between Product, Supplier, Procurement, Receiving, AP, Cashflow, POS, Logistics, Tax, PH1.D, PH1.WRITE, Access, and Audit.
