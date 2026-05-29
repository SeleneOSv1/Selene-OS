# Global Document 65 — Selene Inventory Intelligence Engine

```text id="doc65_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
65

ENGINE:
PH1.INVENTORY / PH1.STOCK_INTELLIGENCE / PH1.INVENTORY_OPTIMIZATION

FULL NAME:
Selene Autonomous Inventory Intelligence, Stock Optimization, Traceability, Shelf-Life, Replenishment, JIT, Stock Health, and Working Capital Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Inventory Intelligence Engine owns the truth of stock.

It answers:

```text id="inventory_questions"
How many do we have?
Where are they?
Are they available?
Are they reserved?
Are they damaged?
Are they expired?
Are they quarantined?
Are they in transit?
Are they sellable?
Are they allocated to a customer?
Are they needed for production?
Will we run out?
Are we holding too much?
Should we reorder?
Should we transfer?
Should we discount?
Should we discontinue?
Should we use JIT?
Should we hold strategic stock?
Can cashflow support buying more?
```

Product Engine tells Selene **what the thing is**.

Inventory Engine tells Selene **how many exist, where they are, and what condition they are in**.

This is not a stock table.

This is not “quantity = 12.”

That is not inventory intelligence. That is counting with a login screen.

Selene Inventory must become the company’s autonomous stock brain: forecasting demand, minimizing waste, avoiding stockouts, reducing dead stock, protecting cashflow, managing shelf life, prioritizing what moves, and ordering before humans remember to panic.

---

## 2. Product Comes First, Inventory Comes Second

Inventory depends on Product.

```text id="product_inventory_order"
Product Engine defines:
- product identity
- SKU / barcode
- unit of measure
- variants
- expiry requirement
- batch / lot requirement
- serial requirement
- storage requirement
- category
- channel readiness

Inventory Engine tracks:
- quantity
- location
- stock state
- batch / lot / serial
- expiry
- reservations
- movements
- stock health
- reorder needs
```

Example:

```text id="inventory_example"
Product:
Organic Shampoo 500ml

Inventory:
143 units on hand
82 in Store A
41 in Warehouse B
20 reserved for B2B customer order
6 expire in 40 days
3 damaged
12 should be discounted before slow-moving risk increases
```

Product is identity.

Inventory is physical and operational truth.

If these merge, the system becomes a warehouse octopus wearing a product-catalog hat. We are not doing that again, humanity has suffered enough.

---

## 3. Core Selene Law

```text id="inventory_core_law"
Selene must know what stock exists, where it is, what condition it is in, what it is likely to do next, and what action should happen before a human asks.
```

Selene must reduce human work by:

```text id="inventory_reduce_human"
forecasting demand
detecting low stock
detecting overstock
detecting expiry risk
detecting dead stock
detecting shrinkage
detecting supplier delay risk
suggesting reorder
creating draft purchase orders
auto-creating routine purchase orders under policy
suggesting stock transfers
suggesting markdowns
suggesting bundles
suggesting discontinuation
updating e-commerce availability
updating B2B availability
updating POS availability
blocking expired/unsafe stock
requesting cycle counts only where needed
```

Humans should not have to say:

> “Selene, maybe we’re running low?”

Selene should say:

> “You will run out in six days. Supplier lead time is five days. I recommend ordering today.”

That is inventory management. The other thing is a storage room with anxiety.

---

## 4. Modern Inventory Standards Selene Must Be Ready For

Selene must support modern traceability, scanning, forecasting, and network optimization.

GS1’s Global Traceability Standard focuses on traceability data across the “who, what, when, where and why” dimensions, so Selene must store inventory events with context, not just quantity changes. ([GS1][1])

FEFO means “First Expired, First Out,” where the items with the earliest expiry dates are sold or used first; it is especially relevant for perishables, pharmaceuticals, cosmetics, and other time-sensitive goods. ([ShipBob][2])

Multi-echelon inventory optimization looks across the whole supply chain network rather than optimizing one warehouse or store in isolation, and this matters when Selene must decide whether to reorder, transfer stock, or hold safety stock at the right node. ([Manhattan Associates][3])

Barcode/RFID-enabled inventory workflows and cycle counting are widely used to improve stock accuracy and reduce manual errors; Selene should treat scans, counts, and movement evidence as part of the inventory truth fabric, not as optional admin decoration. ([LaceUp Solutions][4])

Recent forecasting and inventory-optimization research also points toward evaluating forecasting models by their operational inventory cost and service-level impact, not merely by abstract forecast accuracy; Selene should judge forecasts by whether they reduce stockouts, overstock, waste, and working-capital drag. ([arXiv][5])

Translation: modern inventory is not “count boxes.” It is forecasting, traceability, safety stock, shelf life, cashflow, supplier risk, channel allocation, and commercial strategy all fighting in the same tiny warehouse. Selene’s job is to referee.

---

## 5. Engine Boundary

### 5.1 PH1.INVENTORY owns

```text id="inventory_owns"
stock quantity
stock location
stock state
stock movement
stock reservation
available-to-promise
batch tracking
lot tracking
serial tracking
expiry tracking
shelf-life management
FEFO / FIFO / rotation rules
stock receiving handoff acceptance
putaway
transfers
stock counts
cycle counts
shrinkage detection
damage tracking
quarantine
recall holds
stock health score
reorder signal
safety stock recommendation
JIT logic
overstock detection
dead stock detection
inventory working capital intelligence
stock availability by channel
inventory audit evidence
```

### 5.2 PH1.INVENTORY does not own

```text id="inventory_not_own"
product identity
product media
supplier qualification
purchase order creation
supplier invoice validation
supplier payment
customer checkout
final pricing approval
ledger posting
tax treatment
bank payment
```

### 5.3 Correct owner split

```text id="inventory_owner_split"
PH1.PRODUCT = what the item is.
PH1.INVENTORY = how many exist, where, and in what condition.
PH1.SUPPLIER = who supplies it and how reliable they are.
PH1.PROCUREMENT = whether/how Selene buys it.
PH1.PROC.RECEIVE = what arrived and was accepted.
PH1.CREDITORS / AP = invoice truth.
PH1.ACCOUNTING = financial posting and inventory valuation.
PH1.CASHFLOW = whether buying stock is affordable.
PH1.MARKETING = campaigns/discount execution.
PH1.PRICING = approved price and margin controls.
```

Inventory does not become Product. Product does not become Inventory. This is how we prevent architecture from turning into soup with a barcode.

---

## 6. Inventory Master Record

Every stock-holding unit gets inventory records.

```text id="inventory_master_record"
inventory_id
product_id
variant_id
legal_entity_id
location_id
warehouse_id
zone_id
bin_id
shelf_id
stock_state
quantity_on_hand
quantity_available
quantity_reserved
quantity_allocated
quantity_picked
quantity_packed
quantity_in_transit
quantity_quarantined
quantity_damaged
quantity_expired
quantity_on_order
quantity_backordered
batch_number
lot_number
serial_number
expiry_date
manufacture_date
received_date
supplier_id
purchase_order_id
receiving_id
unit_cost_ref
landed_cost_ref
valuation_category
storage_requirement
temperature_requirement
handling_requirement
shelf_life_days
FEFO_required
FIFO_required
stocktake_priority
last_counted_at
last_movement_at
stock_health_score
audit_ref
```

Inventory truth must be event-based, not overwritten casually by a human typing “new quantity 500” because Wednesday felt inspiring.

---

## 7. Inventory Types Selene Must Support

Selene must support:

```text id="inventory_types"
retail stock
e-commerce stock
B2B stock
raw materials
ingredients
work in progress
finished goods
spare parts
maintenance parts
consumables
packaging
perishable goods
cold-chain goods
regulated goods
serialised goods
batch/lot goods
restaurant ingredients
menu-linked stock
fashion variants
pharmaceutical/medical stock where legally applicable
construction materials
manufacturing components
dropship stock
consignment stock
supplier-held stock
customer-owned stock
stock in transit
returned stock
damaged stock
quarantined stock
recalled stock
```

Selene cannot be built only for retail shelves.

She must support factories, restaurants, salons, warehouses, trade counters, service parts, cold rooms, and whatever business humans invent next to make inventory worse.

---

## 8. Stock State Model

Every stock quantity must sit in a clear state.

```text id="stock_states"
Expected
Received
Accepted
PutawayPending
Available
Reserved
Allocated
Picked
Packed
Shipped
InTransit
ReturnedPendingInspection
Quarantined
Damaged
Expired
Recalled
Disposed
WrittenOff
Archived
```

The key truth:

```text id="stock_truth_law"
On hand is not the same as available.
Available is not the same as sellable.
Sellable is not the same as profitable to sell.
```

Example:

```text id="stock_truth_example"
100 units on hand
20 reserved
10 damaged
5 expired
15 quarantined

Available sellable stock = 50
```

A lesser system says “100 in stock.”

Selene says “50 can actually be sold.” That is why Selene gets to sit at the grown-up table.

---

## 9. Inventory Lifecycle

```text id="inventory_lifecycle"
Product template created
→ stock tracking rules set
→ purchase/replenishment need detected
→ Procurement creates PO
→ Receiving verifies what arrived
→ accepted stock handed to Inventory
→ stock put away
→ stock available/reserved/allocated
→ stock sold/used/transferred/produced
→ stock counted/verified
→ stock discounted/transferred/discontinued if unhealthy
→ stock disposed/written off if expired/damaged/recalled
```

Inventory increases only after Receiving accepts stock.

```text id="accepted_stock_rule"
Ordered quantity ≠ received quantity ≠ accepted quantity.
Inventory increases only by accepted quantity.
```

Example:

```text id="received_example"
PO ordered: 100
Supplier delivered: 100
Receiving accepted: 90
Damaged: 10

Inventory adds: 90 accepted
Supplier obligation created: 10 damaged
AP holds disputed value
```

No more paying for broken goods and then wondering why stock math looks possessed.

---

## 10. Stock Location Intelligence

Selene must know where stock is.

Location hierarchy:

```text id="location_hierarchy"
country
legal entity
region
branch
warehouse
store
vehicle
production site
cold room
zone
aisle
rack
bin
shelf
customer consignment location
supplier-held location
in-transit location
```

Location-specific rules:

```text id="location_rules"
storage capacity
temperature requirement
hazardous storage
high-value cage
fast-pick location
FEFO shelf
B2B reserve area
returns inspection area
quarantine area
```

Selene should say:

> “Put this dairy batch in Cold Room 2, Shelf B. It expires earlier than current stock, so I’ll prioritize it for FEFO picking.”

Warehouse workers should not need to remember expiry strategy in their heads like chilled-product monks.

---

## 11. Stock Reservation and Allocation

Inventory must reserve stock for demand.

Demand sources:

```text id="reservation_sources"
POS sale
e-commerce order
B2B order
customer quote
customer pickup
manufacturing work order
restaurant prep
subscription order
internal use
service repair job
promotion campaign
```

Reservation states:

```text id="reservation_states"
Requested
Reserved
Allocated
Picked
Released
Expired
Cancelled
ConvertedToSale
```

Selene must prevent overselling.

Example:

```text id="reservation_example"
Stock on hand: 100
Reserved for B2B order: 70
Reserved for e-commerce orders: 20
Damaged: 5

Available for POS: 5
```

Selene says:

> “Only 5 units are available for new sales. The rest is reserved or damaged.”

This prevents the classic “available online but not actually available in reality” customer-service bonfire.

---

## 12. Available-to-Promise and Capable-to-Promise

Selene must support ATP and CTP.

### Available-to-Promise

```text id="atp"
Available-to-Promise = what can be promised from current and incoming stock.
```

Considers:

```text id="atp_inputs"
available stock
reserved stock
incoming purchase orders
supplier lead time
receiving confidence
delivery capacity
channel priority
customer priority
credit status
```

### Capable-to-Promise

```text id="ctp"
Capable-to-Promise = what can be promised based on production capacity, raw materials, labor, and lead time.
```

Used for manufacturing, made-to-order, and custom products.

Selene says:

> “We do not have finished stock, but raw materials and production capacity allow delivery in 8 days.”

This is how manufacturers stop handling orders by inbox archaeology.

---

## 13. Batch, Lot, Serial, and Traceability

Selene must support traceable stock.

Traceability fields:

```text id="traceability_fields"
batch
lot
serial
expiry
manufacture date
supplier
origin
receiving event
inspection result
location history
customer shipment
recall status
```

Traceability must answer:

```text id="traceability_questions"
Which supplier provided this batch?
Where is the batch now?
Which customers received it?
Which stock remains unsold?
Which stock must be recalled?
Which receiving event accepted it?
Which documents prove it?
```

GS1’s traceability model emphasizes capturing and sharing traceability context across who, what, when, where, and why; Selene should mirror this through stock events and audit packets. ([GS1][1])

Selene says:

> “Batch B-442 was supplied by ABC, received on 12 May, sold to 18 customers, and 42 units remain in Warehouse A.”

That is traceability. Not “probably in the back somewhere.”

---

## 14. Shelf-Life and Expiry Management

Selene must manage shelf life aggressively.

Expiry-sensitive products include:

```text id="expiry_products"
food
beverages
medicine
cosmetics
chemicals
batteries
consumables
perishable restaurant ingredients
dated promotional products
regulated items
```

Rotation methods:

```text id="rotation_methods"
FEFO
FIFO
LIFO where allowed
batch-specific pick
expiry-blocked sale
```

FEFO prioritizes stock by earliest expiry date, which is essential where expiry matters; Selene should therefore route perishable and time-sensitive items by expiry, not merely by receipt date. ([ShipBob][2])

Selene actions:

```text id="expiry_actions"
warn before expiry
prioritize FEFO pick
recommend discount
recommend transfer
recommend bundle
block sale after expiry
trigger disposal workflow
trigger supplier claim if shelf life was too short on receipt
```

Example:

> “This yogurt batch expires in 9 days. Sales velocity is too slow. I recommend a 20% discount today or transfer to Store B, where it sells faster.”

Yogurt now has a strategy. Humanity may yet recover.

---

## 15. JIT and Lean Inventory Mode

Selene should operate as close to JIT as safely possible.

But not stupid JIT.

```text id="jit_law"
Selene should minimize stockholding cost and cash tied up in inventory while protecting service level, supplier risk, demand spikes, and business continuity.
```

Inventory modes:

```text id="inventory_modes"
JIT Mode
Lean Buffer Mode
Normal Reorder Mode
Seasonal Surge Mode
Strategic Reserve Mode
Perishable Same-Day Mode
Make-to-Order Mode
Discontinue / Exit Mode
```

Selene decides mode by:

```text id="mode_decision"
sales velocity
demand variability
supplier lead time
supplier reliability
margin
cashflow
storage cost
expiry risk
customer service level
criticality
substitute availability
```

Example:

```text id="jit_examples"
fresh cakes → Perishable Same-Day / JIT
toilet paper → Normal Reorder / Lean Buffer
critical machine part → Strategic Reserve
slow-moving low-margin item → Discontinue or clearance
```

Selene says:

> “This item sells predictably and supplier lead time is short. I recommend Lean Buffer instead of holding 60 days of stock.”

Holding too much stock is just turning cash into shelf decoration.

---

## 16. Demand Forecasting

Selene must forecast demand continuously.

Forecast inputs:

```text id="forecast_inputs"
historical sales
seasonality
weekday patterns
public holidays
school holidays
weather where relevant
local events
promotions
price changes
B2B orders
customer reorder cycles
e-commerce traffic
POS velocity
marketing campaigns
stockouts
returns
supplier availability
economic signals where available
```

Forecast outputs:

```text id="forecast_outputs"
expected demand
confidence score
stockout date
overstock risk
recommended reorder quantity
recommended reorder timing
recommended safety stock
recommended transfer
recommended markdown
cash impact
```

Selene should judge forecasting models by operational outcomes: fewer stockouts, lower overstock, lower waste, better service level, and lower working-capital cost — not just mathematical accuracy. Recent research explicitly evaluates forecasting by inventory cost and service impact, which is exactly the direction Selene should follow. ([arXiv][5])

Selene says:

> “Based on recent sales, public holiday uplift, and supplier lead time, you should reorder today. Waiting two days creates a 68% stockout risk.”

That is better than “stock below minimum.” The bar is low, but here we are.

---

## 17. Reorder Intelligence

Selene calculates reorder timing and quantity.

Basic concept:

```text id="reorder_basic"
Reorder need = demand during lead time + safety stock - available stock - incoming stock
```

Selene must go beyond the basic formula.

Reorder logic considers:

```text id="reorder_logic"
current available stock
reserved stock
incoming purchase orders
forecast demand
supplier lead time
supplier reliability
lead-time variability
demand variability
safety stock
service level target
MOQ
pack size
cashflow
budget
storage capacity
expiry
margin
customer importance
B2B commitments
production demand
```

Possible actions:

```text id="reorder_actions"
do not reorder
reorder now
reorder later
split order
transfer stock instead
use backup supplier
increase safety stock
reduce safety stock
clear stock before reorder
```

Selene says:

> “I recommend ordering 340 units, not 500. That covers demand and supplier delay risk without tying up unnecessary cash.”

This is inventory as working-capital intelligence. Not “buy more because shelf empty soon.” A bold leap forward for box civilization.

---

## 18. Restocking Automation Frequency

Restocking is not weekly by default.

Selene uses event-driven and scheduled restocking.

### 18.1 Event triggers

```text id="reorder_event_triggers"
sale occurs
B2B order arrives
supplier delay reported
stock received
stock damaged
expiry risk changes
promotion scheduled
public holiday detected
weather/event signal changes
cashflow changes
product velocity changes
```

### 18.2 Scheduled review cadence

```text id="reorder_cadence"
perishable / fresh / high velocity = continuous or hourly
restaurant prep items = daily before prep plus intraday adjustment
A-class high-value / high-impact items = daily
fast-moving retail = daily
B-class normal items = 2–3 times per week or weekly
C-class slow movers = weekly or monthly
dead stock = monthly liquidation / discontinue review
critical parts = daily risk check even if low volume
seasonal items = increased review before and during season
```

Selene should not run every product through the same cadence. That would be inventory management by blanket, and blankets are not algorithms.

---

## 19. Safety Stock and Service Level

Selene must optimize safety stock by product role and risk.

Safety stock inputs:

```text id="safety_stock_inputs"
demand variability
lead-time variability
supplier reliability
service level target
stockout cost
substitute availability
customer importance
storage cost
cashflow
expiry risk
```

Service levels differ by product.

```text id="service_level_examples"
critical spare part → high service level
fresh pastry → lower buffer due to waste risk
habit product → high availability but lean stock
slow-moving luxury item → low safety stock
B2B contract item → contract-specific service level
```

Selene says:

> “This item has low margin but drives repeat customer visits. I recommend high availability with lean replenishment, not deep stock.”

Good. Nuance. Inventory needed it badly.

---

## 20. Multi-Location and Multi-Echelon Optimization

Selene must optimize across the network.

Locations:

```text id="network_locations"
supplier
factory
warehouse
distribution center
store
vehicle
customer consignment
marketplace fulfillment center
B2B customer reserve
```

Decisions:

```text id="network_decisions"
where stock should sit
which location should fulfill
whether to transfer or reorder
where safety stock should be held
which branch should receive replenishment
whether to centralize or decentralize stock
```

Multi-echelon optimization considers inventory across the supply chain network rather than one node at a time; Selene should therefore avoid buying more when stock can be transferred from another location without damaging service levels. ([Manhattan Associates][3])

Selene says:

> “Store A is overstocked and Store C will stock out in five days. Transfer 40 units instead of buying more.”

This prevents companies from buying stock while identical stock is quietly aging in another branch like a forgotten warehouse mushroom.

---

## 21. Restaurant and Perishable Production JIT

Selene must support same-day JIT for perishables and restaurants.

Example: cakes.

Selene considers:

```text id="cake_forecast_inputs"
last Tuesday sales
last 4 Tuesdays average
public holiday uplift
weather
local events
school holidays
pre-orders
delivery app demand
walk-in trend
staff availability
oven capacity
ingredient stock
shelf life
waste tolerance
target sell-out time
profit margin
```

Example output:

```text id="cake_output"
Normal Tuesday sales: 40
Public holiday uplift: +35%
Pre-orders: 8
Recommended production: 60–65 cakes

Bake plan:
- 40 before opening
- 15 at 11 AM if sales pace confirms
- 10 at 2 PM only if demand remains high
```

Selene says:

> “Today is a public holiday. I recommend baking 40 now and holding ingredients for a second batch at 11 AM. This protects sales without creating waste.”

That is JIT with judgment. Not “bake 80 and hope people have cake feelings.”

---

## 22. Strategic Product Role Logic

Inventory must understand product roles from Product Engine.

Product roles:

```text id="product_roles"
Hero Product
Profit Driver
Traffic Driver
Habit Builder
Basket Builder
Loss Leader
B2B Anchor Product
Seasonal Product
Clearance Product
Dead Stock Candidate
Discontinue Candidate
Strategic Reserve
```

Inventory decisions must not be based only on margin.

Selene checks:

```text id="role_checks"
repeat visits
basket attachment
cross-sell rate
customer lifetime value
B2B reorder dependency
subscription retention
brand trust
customer habit creation
```

Example:

> “This product has low margin, but customers who buy it also buy higher-margin items. Keep it stocked leanly as a traffic driver.”

This is where Selene becomes better than a basic margin report. A margin report would kill the milk and then wonder why customers stopped visiting.

---

## 23. Overstock, Dead Stock, and Discontinuation

Selene must detect unhealthy stock.

Signals:

```text id="dead_stock_signals"
low sales velocity
high days of cover
weak margin
high returns
high storage cost
expiry risk
new model replacing old
supplier unreliability
low search/customer interest
cash tied up
no strategic role
```

Actions:

```text id="dead_stock_actions"
discount
bundle
transfer
B2B clearance
return to supplier if allowed
pause reorders
replace supplier
discontinue
write-off if unsellable
```

Selene says:

> “This product has 112 days of stock, low sales velocity, no strategic customer role, and weak margin. I recommend pausing reorders and preparing a clearance plan.”

No more stock becoming museum exhibits with SKU labels.

---

## 24. Dynamic Markdown and Promotion Triggers

Inventory must feed Pricing and Marketing.

Markdown triggers:

```text id="markdown_triggers"
near expiry
overstock
slow movement
season ending
replacement model launched
cashflow pressure
warehouse capacity pressure
supplier promotion
B2B clearance opportunity
```

Selene recommends:

```text id="markdown_actions"
discount
bundle
loyalty offer
B2B bulk offer
staff/customer offer
online feature
cross-sell
transfer before markdown
```

Selene must consider:

```text id="markdown_considerations"
margin floor
brand damage
customer habit value
cashflow need
expiry deadline
stock quantity
sales channel
supplier funding
```

Selene says:

> “This product is overstocked but sells well to B2B buyers. I recommend a 12-unit trade bundle instead of retail discounting.”

Good. Smarter than “everything 20% off because warehouse sad.”

---

## 25. Stock Health Score

Every product-location-batch has a stock health score.

Inputs:

```text id="stock_health_inputs"
days of cover
sales velocity
forecast confidence
stockout risk
overstock risk
expiry risk
margin
supplier reliability
return rate
damage rate
shrinkage
cash tied up
storage cost
channel demand
product role
```

Health states:

```text id="stock_health_states"
Excellent
Healthy
Watch
ActionNeeded
Urgent
Blocked
DiscontinueCandidate
```

Selene says:

> “Stock Health is Action Needed. You have 74 days of stock, weak sales, and cash tied up. I recommend discounting or pausing reorders.”

Normal system: “Quantity 842.”

Selene: “This stock is becoming a financial swamp.”

Much better. Worse for denial, better for business.

---

## 26. Stock Accuracy and Cycle Counting

Selene must maintain inventory accuracy without making humans count everything all the time.

Counting methods:

```text id="count_methods"
cycle counting
ABC counting
risk-based counting
blind count
barcode scan count
RFID-assisted count
photo-assisted shelf count
variance review
full stocktake
```

Cycle counting can catch errors earlier than annual full counts, and barcode/RFID/mobile scan workflows can reduce manual errors and help keep stock current. ([LaceUp Solutions][4])

Count priority based on:

```text id="count_priority"
high value
high movement
high shrinkage
high discrepancy history
expiry risk
stockout risk
B2B critical item
recent return
recent transfer
supplier dispute item
```

Selene says:

> “I recommend counting Product X today. It has high movement and a mismatch between POS sales and shelf scans.”

No annual warehouse panic as the only control. That is accounting cosplay with ladders.

---

## 27. Shrinkage, Loss, and Fraud Detection

Shrinkage sources:

```text id="shrinkage_sources"
theft
damage
expiry
wrong receiving
wrong picking
supplier short delivery
returns fraud
POS scanning error
warehouse misplacement
employee misuse
integration error
```

Selene detects:

```text id="shrinkage_detection"
count mismatch
unexpected stock adjustment
stock movement without sale
sale without stock movement
high return pattern
damaged goods spike
supplier short-delivery pattern
POS no-scan pattern
inventory adjustment abuse
```

Selene says:

> “This product has three unexplained stock adjustments this month. I recommend manager review before more adjustments are accepted.”

Suspicion with receipts. The best kind.

---

## 28. Returns, Quarantine, and Inspection

Returned stock does not automatically become sellable.

Selene checks:

```text id="return_checks"
condition
packaging
seal intact
expiry
batch/lot
serial
customer reason
damage photos
restock eligibility
recall status
contamination risk
```

Return outcomes:

```text id="return_outcomes"
restock
quarantine
repair
refurbish
return to supplier
discounted resale
dispose
write-off
```

Selene says:

> “This returned product cannot go back into sellable stock until inspection confirms the seal is intact.”

Customers do strange things to returns. Inventory should not trust them blindly. Sorry, humanity.

---

## 29. Recall and Safety Holds

Selene must support recall events.

Recall sources:

```text id="recall_sources"
supplier notice
regulator notice
internal quality issue
customer complaint pattern
batch defect
food safety issue
expiry/contamination issue
```

Selene action:

```text id="recall_actions"
identify affected batch/lot/serial
block sale
identify locations
identify customers shipped
notify relevant engines
trigger returns/recall workflow
prepare supplier claim
prepare compliance evidence
```

Selene says:

> “Batch B-442 is affected by supplier recall. I have blocked sale, identified remaining stock, and listed customers who received it.”

No “maybe check the back room.” Recall is not a treasure hunt.

---

## 30. Inventory Valuation Handoff

Inventory supports accounting valuation, but Accounting owns final posting.

Inventory provides:

```text id="valuation_handoff"
quantity accepted
quantity on hand
quantity damaged
quantity expired
quantity written off
unit cost reference
landed cost reference
valuation category
stock movement
COGS handoff
inventory adjustment evidence
```

Accounting owns:

```text id="accounting_owns_inventory"
inventory asset value
COGS posting
write-off posting
valuation method
period-end inventory financials
tax/reporting treatment
```

Selene must not let Inventory post ledger directly.

Inventory proves stock.

Accounting posts money truth.

A beautiful division, like civilized people pretending to like meetings.

---

## 31. Inventory and Cashflow

Inventory ties up cash.

Selene must report:

```text id="inventory_cashflow"
cash tied in stock
days inventory on hand
dead stock value
overstock value
stockout risk cost
reorder cash impact
supplier payment timing
expected sales conversion
markdown recovery value
```

Cashflow-aware reorder logic:

```text id="cashflow_reorder"
if reorder needed but cash tight:
- split order
- delay flexible portion
- transfer stock
- negotiate supplier terms
- choose faster smaller supplier
- clear slow stock
```

Selene says:

> “This reorder protects availability but would reduce cash below payroll buffer. I recommend ordering 60% now and the rest after receivables clear.”

Good. Inventory should not mug Cashflow behind the warehouse.

---

## 32. Inventory and Supplier Intelligence

Supplier Engine provides:

```text id="supplier_to_inventory"
lead time
lead-time reliability
quality score
delivery score
short-delivery history
damaged-goods history
replacement speed
supplier restriction status
```

Inventory uses supplier reliability to adjust:

```text id="supplier_adjustments"
safety stock
reorder timing
supplier selection recommendation
JIT viability
promotion risk
B2B promise risk
```

Selene says:

> “Supplier A is cheaper, but unreliable. Because this product has high stockout cost, I recommend Supplier B or increased buffer.”

Cheap supplier made expensive by consequences. A classic business folktale.

---

## 33. Inventory and Procurement

Inventory does not create final purchase orders.

Inventory sends:

```text id="inventory_to_procurement"
reorder recommendation
quantity needed
timing
stockout risk
overstock risk
cash impact
preferred supplier suggestion
transfer alternative
```

Procurement decides:

```text id="procurement_decides"
create PO
split order
route approval
choose supplier
delay purchase
transfer instead
cancel reorder
```

Selene says:

> “Inventory recommends reorder, but Procurement found sufficient stock at Branch 2. I recommend transfer instead of purchase.”

This is why engines need to talk. Otherwise the company buys more while the other branch has the goods sitting there like a smug little secret.

---

## 34. Inventory and Receiving

Receiving proves accepted stock.

Inventory accepts only:

```text id="inventory_accepts"
accepted quantity
accepted batch/lot/serial
accepted expiry
accepted condition
accepted location
```

Inventory does not accept:

```text id="inventory_rejects"
damaged quantity
short quantity
wrong item
uninspected high-risk goods
quarantined goods as sellable
```

Receiving sends:

```text id="receiving_to_inventory"
InventoryReceiptHandoffPacket
```

Inventory updates stock state.

This is one of the most important boundaries in Selene.

If Receiving is weak, Inventory becomes fiction.

If Inventory is fiction, Accounting becomes fan fiction.

---

## 35. Inventory and Commerce

Inventory feeds availability to:

```text id="commerce_channels"
POS
E-Commerce
B2B
marketplaces
quote portal
subscription engine
customer portal
```

Availability rules may differ by channel.

```text id="channel_allocation"
reserve stock for B2B
reserve stock for POS
reserve stock for online
reserve stock for critical customers
reserve production materials
allow backorder for trade customers
block backorder for retail customers
```

Selene says:

> “Only 12 units remain. I recommend reserving 8 for contract B2B customers and leaving 4 for retail.”

This is not “first come, first served” chaos. This is policy, margin, and promise management.

---

## 36. Inventory and Manufacturing

Manufacturing inventory includes:

```text id="manufacturing_inventory"
raw materials
components
subassemblies
work in progress
finished goods
scrap
rework
quality hold
batch traceability
production consumption
```

Manufacturing flow:

```text id="manufacturing_flow"
customer / B2B order
→ production demand
→ raw material check
→ WIP tracking
→ finished goods
→ delivery
→ accounting handoff
```

Selene says:

> “You can accept this order, but raw material A will run short in six days unless Supplier B confirms delivery.”

Manufacturers need inventory to understand capacity, not just shelves. Big machines are surprisingly useless when one tiny part is missing. Comedy, but expensive.

---

## 37. Inventory and Restaurant / Menu

Restaurant inventory links ingredients to menu items.

Fields:

```text id="restaurant_inventory_fields"
ingredient
recipe
portion size
waste
expiry
supplier
food cost
menu price link
prep quantity
same-day production
```

Dish sale deducts ingredients.

Example:

```text id="dish_deduction"
Chicken Salad:
- 200g chicken
- 50g lettuce
- 20ml dressing
```

Selene says:

> “Chicken cost increased 14%. This dish margin is now below target. I recommend price review or supplier switch.”

Gordon Ramsay, but calmer and with fewer shattered plates.

---

## 38. Inventory and Marketing

Inventory tells Marketing what should be promoted.

Signals:

```text id="inventory_marketing_signals"
overstock
near-expiry
slow-moving
high-margin available stock
B2B clearance opportunity
seasonal product
habit product
new product launch
dead stock recovery
```

Marketing must respect:

```text id="marketing_constraints"
margin floor
stock availability
expiry
supplier replenishment risk
channel allocation
brand positioning
cashflow need
```

Selene says:

> “Promote Product A, not Product B. Product A has excess stock and strong margin. Product B is nearly sold out.”

This stops marketing from accidentally selling what operations cannot fulfill. A bold innovation in some companies.

---

## 39. Inventory and Product Discontinuation

Inventory can recommend discontinuation, but Product/Management approves.

Discontinuation signals:

```text id="discontinue_signals"
low movement
no strategic role
weak margin
high storage cost
high expiry/waste
supplier issues
high returns
low customer interest
replacement product available
```

Selene says:

> “This product is a discontinuation candidate. It has low sales, weak margin, and no basket-building effect.”

If product is a traffic driver:

> “Do not discontinue yet. It has low margin but supports repeat visits and basket value.”

Inventory must understand commercial role, not just stock math. That’s how Selene avoids becoming a dead-eyed optimization goblin.

---

## 40. Automation and Exception-Only Review

Selene auto-handles:

```text id="inventory_auto_handles"
stock movement from POS/e-commerce/B2B
receiving accepted-stock handoff
reservation and allocation
low-stock detection
stockout forecasting
overstock detection
expiry warning
FEFO pick recommendation
routine reorder recommendation
routine draft PO
routine transfer recommendation
cycle count scheduling
stock health scoring
availability updates to commerce
dead stock warnings
```

Selene needs human/authority for:

```text id="inventory_needs_approval"
large write-off
major stock adjustment
accepting unverified stock
overriding quarantine/recall hold
changing valuation-impacting stock
supplier switch if protected
large purchase order
major markdown outside policy
discontinuing live product
manual override of stock truth
```

Selene does not ask for approval to notice stock is low.

Selene does not ask for approval to schedule a count.

Selene does ask before writing off $50,000 of inventory because someone’s cousin “counted it probably.” Good.

---

## 41. PH1.D / GPT-5.5 Role

GPT-5.5 should help with explanation, summarization, and recommendations.

### 41.1 GPT-5.5 may help

```text id="gpt_inventory_allowed"
explain stockout risk
summarize inventory health
draft supplier messages
draft transfer instructions
explain reorder reasoning
summarize overstock plan
draft markdown campaign wording
explain JIT plan in plain English
summarize stock count variance
draft management inventory report
```

### 41.2 GPT-5.5 must not

```text id="gpt_inventory_forbidden"
alter stock truth
invent counts
approve write-offs
release quarantine
remove recall holds
approve large POs
change valuation
override receiving proof
invent supplier confirmations
```

GPT-5.5 talks nicely.

Inventory truth remains deterministic.

The eloquent robot does not get to say “we probably have 200.” No. Bad robot. Count first.

---

## 42. Human-Like Selene Interaction

### Low stock

> “You are likely to run out of this product in five days. I recommend ordering 240 units today.”

### JIT

> “Demand is high today because of the public holiday. I recommend producing a first batch now and a second batch only if the sales pace confirms.”

### Overstock

> “This item has 92 days of stock and weak sales. I recommend pausing reorders and preparing a bundle offer.”

### Supplier risk

> “Supplier A is cheaper, but their last three deliveries were late. I recommend Supplier B for this reorder.”

### Expiry

> “This batch expires in 12 days. I recommend moving it to Store B or discounting it online.”

### Stock count

> “The system expects 180 units, but recent scans suggest fewer. Please run a quick count before I approve more online availability.”

Human-like means Selene explains business reasons, not just alerts. Alerts are cheap. Judgment is valuable.

---

## 43. State Machines

### Inventory Item State

```text id="inventory_item_state"
Expected
Received
Accepted
PutawayPending
Available
Reserved
Allocated
Picked
Packed
Shipped
ReturnedPendingInspection
Quarantined
Damaged
Expired
Recalled
Disposed
WrittenOff
Archived
```

### Replenishment State

```text id="replenishment_state"
Monitoring
ReorderSuggested
DraftPORequested
PendingProcurement
POCreated
SupplierConfirmed
InTransit
Received
PartiallyReceived
Closed
Cancelled
Disputed
```

### Stock Transfer State

```text id="transfer_state"
Suggested
PendingApproval
Picked
InTransit
Received
Accepted
Closed
Lost
Damaged
Cancelled
```

### Stock Count State

```text id="stock_count_state"
Scheduled
InProgress
VarianceDetected
VarianceReview
Adjusted
Escalated
Closed
```

### Stock Health State

```text id="stock_health_state"
Excellent
Healthy
Watch
ActionNeeded
Urgent
Blocked
DiscontinueCandidate
```

### Expiry Risk State

```text id="expiry_state"
NotApplicable
Healthy
Watch
Urgent
Expired
Blocked
Disposed
```

---

## 44. Reason Codes

```text id="inventory_reason_codes"
INVENTORY_TEMPLATE_CREATED
STOCK_RECEIVED_ACCEPTED
STOCK_PUTAWAY_PENDING
STOCK_AVAILABLE
STOCK_RESERVED
STOCK_ALLOCATED
STOCK_IN_TRANSIT
STOCK_QUARANTINED
STOCK_DAMAGED
STOCK_EXPIRED
STOCK_RECALLED
STOCKOUT_RISK
LOW_STOCK
OVERSTOCK_RISK
DEAD_STOCK_RISK
REORDER_RECOMMENDED
REORDER_BLOCKED_CASHFLOW
TRANSFER_RECOMMENDED
JIT_MODE_RECOMMENDED
SAFETY_STOCK_INCREASE_RECOMMENDED
SAFETY_STOCK_REDUCTION_RECOMMENDED
FEFO_PICK_REQUIRED
EXPIRY_DISCOUNT_RECOMMENDED
B2B_STOCK_RESERVED
ECOMMERCE_STOCK_RESERVED
POS_STOCK_RESERVED
STOCK_COUNT_REQUIRED
VARIANCE_DETECTED
SHRINKAGE_RISK
RECALL_HOLD_APPLIED
WRITE_OFF_REQUIRES_APPROVAL
DISCONTINUE_RECOMMENDED
```

---

## 45. Required Simulations

```text id="inventory_simulations"
create inventory template from Product
receive accepted stock from Receiving
reject damaged quantity from Receiving
putaway stock
reserve stock for e-commerce
reserve stock for B2B
POS sale stock deduction
restaurant recipe ingredient deduction
manufacturing raw material consumption
stockout forecast
JIT reorder recommendation
public holiday demand uplift
cake staged production plan
supplier lead-time adjusted reorder
cashflow-blocked reorder
stock transfer recommendation
overstock markdown recommendation
expiry FEFO pick
cycle count scheduled
stock count variance detected
shrinkage anomaly detected
returned stock quarantine
recall batch hold
dead stock discontinuation recommendation
traffic-driver low-margin product retained
inventory valuation handoff to Accounting
```

---

## 46. Integration Map

```text id="inventory_integration_map"
PH1.INVENTORY / STOCK_INTELLIGENCE
↔ PH1.PRODUCT
↔ PH1.SUPPLIER
↔ PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.PROC.RECEIVE
↔ PH1.CREDITORS / AP
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.BUDGET
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.PRICING
↔ PH1.MARKETING
↔ PH1.CUSTOMER
↔ PH1.LOGISTICS
↔ PH1.RETURNS
↔ PH1.MANUFACTURING / PRODUCTION
↔ PH1.RESTAURANT / MENU
↔ PH1.ASSET_ACCOUNTING
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 47. Required Logical Packets

```text id="inventory_packets"
InventoryTemplatePacket
InventoryStockRecordPacket
StockMovementPacket
StockStatePacket
StockReservationPacket
StockAllocationPacket
AvailableToPromisePacket
CapableToPromisePacket
BatchLotTraceabilityPacket
SerialTraceabilityPacket
ExpiryRiskPacket
FEFOPickPacket
PutawayInstructionPacket
InventoryReceiptHandoffPacket
StockTransferPacket
StockCountPacket
StockVariancePacket
ShrinkageRiskPacket
StockHealthPacket
ReorderRecommendationPacket
JITRecommendationPacket
SafetyStockPacket
OverstockActionPacket
MarkdownRecommendationPacket
DiscontinuationRecommendationPacket
InventoryCashImpactPacket
InventoryAccountingHandoffPacket
AuditEvidencePacket
```

Logical only. Codex maps later. No runtime packet goblins yet.

---

## 48. What Codex Must Not Do

```text id="codex_no_inventory"
Do not merge Inventory into Product.
Do not let Inventory own product identity.
Do not let Inventory create supplier truth.
Do not let Inventory create purchase orders directly.
Do not let Inventory accept stock without Receiving proof.
Do not let Inventory pay supplier invoices.
Do not let Inventory post ledger directly.
Do not let GPT-5.5 alter stock counts.
Do not allow expired/quarantined/recalled stock to become sellable without proper workflow.
Do not treat on-hand as available.
Do not ignore cashflow in reorder logic.
Do not remove low-margin traffic-driver products automatically.
Do not implement from this document alone.
```

---

## 49. Final Architecture Sentence

Selene Inventory Intelligence Engine is the autonomous stock brain that tracks every accepted unit, batch, lot, serial, expiry, location, reservation, allocation, movement, transfer, return, quarantine, damage, recall, and stock count; forecasts demand using sales, seasonality, holidays, supplier lead times, events, B2B commitments, and customer behavior; optimizes replenishment through JIT, safety stock, transfers, supplier reliability, and cashflow; manages shelf life through FEFO and expiry actions; detects stockouts, overstock, dead stock, shrinkage, and strategic low-margin traffic drivers; updates POS, e-commerce, B2B, manufacturing, restaurant, accounting, cashflow, procurement, and supplier workflows; and keeps humans involved only for judgment-heavy exceptions while Selene performs routine inventory work autonomously.

Simple version:

```text id="inventory_simple"
Product tells Selene what the item is.
Inventory tells Selene how many exist and where.
Selene knows what is available, reserved, damaged, expired, or quarantined.
Selene predicts when stock will run out.
Selene orders before humans panic.
Selene avoids excess stock.
Selene uses JIT where safe.
Selene protects cashflow.
Selene discounts or transfers stock before it dies.
Selene keeps strategic low-margin products when they create customer habits.
Selene sends accepted stock to Accounting.
Humans approve only real exceptions.
Everything is audited.
```

That is Global Document 65 — Inventory Intelligence Engine. Not a stock table. A living, forecasting, shelf-life-aware, supplier-aware, cash-aware, customer-habit-aware inventory brain that knows when to order, when to wait, when to transfer, when to discount, and when to tell the business, “Please stop buying this dead product, it has become furniture.”

[1]: https://www.gs1.org/standards/gs1-global-traceability-standard/current-standard?utm_source=chatgpt.com "GS1 Global Traceability Standard"
[2]: https://www.shipbob.com/blog/fefo/?utm_source=chatgpt.com "FEFO: First Expired, First Out"
[3]: https://www.manh.com/solutions/supply-chain-planning-software/demand-forecasting-software/multi-echelon-inventory-optimization?utm_source=chatgpt.com "Multi-Echelon Inventory Optimization"
[4]: https://www.laceupsolutions.com/real-time-cycle-counting-for-inventory-accuracy/?utm_source=chatgpt.com "Real-Time Cycle Counting for Inventory Accuracy"
[5]: https://arxiv.org/abs/2603.16815?utm_source=chatgpt.com "Beyond Accuracy: Evaluating Forecasting Models by Multi-Echelon Inventory Cost"
