# Selene Inventory Addendum A — Autonomous JIT + Reorder Intelligence + Product Role Logic + Inventory Operating Autonomy Mode

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / INVENTORY AUTONOMY + JIT + REORDER INTELLIGENCE + PRODUCT ROLE LOGIC

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Selene Inventory + COGS + Stock Accounting Handoff Master Design

PURPOSE:
Strengthen the inventory master design with autonomous JIT/reorder intelligence, product role logic, low-margin customer habit products, restocking cadence, staged replenishment, supplier/procurement/receiving boundaries, and company-selectable Inventory Operating Autonomy Modes.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

This document is Finance/Accounting Design Batch 5. It defines future inventory operating autonomy, supplier-aware and cashflow-aware reorder intelligence, JIT/lean/perishable/make-to-order replenishment, product role logic, low-margin habit-product protection, staged production/replenishment, and Procurement/Receiving/AP/Accounting handoffs. It does not implement Inventory, Product, Supplier, Procurement, Receiving, AP, Accounting, Cashflow, Budget, Tax, POS, B2B, E-Commerce, packets, migrations, tests, or runtime state.

Current repo truth does not prove complete runtime PH1.INVENTORY, PH1.PRODUCT, PH1.SUPPLIER, PH1.PROCUREMENT, PH1.PROC.RECEIVE, inventory autonomy, or reorder execution ownership. This addendum is future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Master Addendum Law

Selene Inventory must become an autonomous, cashflow-aware, supplier-aware, demand-forecasting stock intelligence system.

Selene must answer:

```text
What will sell?
When will it sell?
How much should we hold?
When should we reorder?
Which supplier can deliver in time?
Can cashflow handle the reorder?
Should we transfer stock instead of buying?
Should we discount it?
Should we discontinue it?
Should we keep it even if margin is low because it brings customers back?
```

JIT is the target where safe, but not blind JIT.

Selene must not force one fixed behavior on every company, product, supplier, location, or stock category.

## 2. Owner Boundaries Preserved

Future standalone owner boundaries remain separate:

```text
PH1.PRODUCT
Product Catalogue + Pricing + SKU + Variants + Offers + Commerce Listing Engine

PH1.INVENTORY
Inventory + Stock Movement + Warehouse + Stocktake + Reorder + COGS Handoff Engine

PH1.SUPPLIER
Supplier Identity + Terms + Score + Risk + Performance Engine

PH1.PROCUREMENT
Purchase Request + Purchase Order + Supplier Selection + Reorder Execution Engine

PH1.PROC.RECEIVE
Goods Receiving + Inspection + Supplier Resolution Engine
```

Product Engine owns:

```text
product identity
description
media
attributes
SKU
variants
bundles
pricing presentation
channel readiness
commerce listing readiness
product lifecycle status
```

Inventory Engine owns:

```text
stock truth
quantity
location
batch
lot
serial
expiry
reservation
allocation
movement
stock health
stockout risk
overstock risk
stocktake evidence
COGS handoff evidence
```

Supplier Engine owns:

```text
supplier identity
supplier score
delivery performance
quality history
credit/refund obligations
supplier risk
supplier terms
alternative supplier search
lead time reliability
price stability
```

Procurement Engine owns:

```text
purchase request
purchase order
approval
budget check
supplier selection
order generation
reorder automation
purchase commitment
```

Receiving Engine owns:

```text
goods receipt
quantity check
damage check
inspection
accepted quantity
short delivery
wrong item
supplier dispute
4-way match input
receiving evidence
```

Clean flow:

```text
Product says what item is.
Inventory says stock is low.
Supplier says who can supply it.
Procurement creates/approves PO.
Receiving verifies what arrived.
Inventory adds accepted stock.
AP matches invoice.
Accounting posts.
```

## 3. Inventory Modes

Inventory modes must include:

```text
JIT Mode
Lean Buffer Mode
Normal Reorder Mode
Seasonal Surge Mode
Strategic Reserve Mode
Perishable Same-Day Mode
Make-to-Order Mode
Discontinue / Exit Mode
```

### JIT Mode

Use only when:

```text
demand is predictable
supplier reliability is high
lead time is stable
stockout penalty is acceptable
cashflow benefits from low stockholding
product is not critical beyond policy tolerance
```

### Lean Buffer Mode

Use when demand or supplier timing is moderately uncertain and the company should hold a small buffer.

### Normal Reorder Mode

Use standard reorder points, reorder quantities, budget checks, and supplier timing.

### Seasonal Surge Mode

Use for seasonal items, holiday demand, local events, school periods, weather-driven demand, and campaign-driven uplift.

### Strategic Reserve Mode

Use for critical parts, production-stopping inputs, safety stock, business-continuity materials, restricted supply chains, and supplier-risk periods.

### Perishable Same-Day Mode

Use for fresh food, cakes, prep items, flowers, perishables, and items where overproduction becomes waste quickly.

### Make-To-Order Mode

Use when inventory is produced or assembled after order confirmation or customer specification.

### Discontinue / Exit Mode

Use for dead stock, obsolete products, product retirement, low-role slow movers, or planned clearance.

## 4. Reorder Intelligence

Reorder logic must consider:

```text
demand
forecast confidence
sales velocity
lead time
lead time variability
supplier reliability
safety stock
seasonality
holidays/events
current stock
reserved stock
incoming stock
allocated stock
expiry
shelf life
storage capacity
storage cost
supplier score
margin
product role
customer dependency
B2B dependency
minimum order quantity
bulk discount
cashflow
budget
minimum cash reserve
profit floor
restricted goods rules
tax/compliance blocks
```

Selene should recommend quantity and timing, not simply declare a reorder point was crossed.

Basic logic:

```text
recommended_reorder =
forecast demand during lead time
+ safety stock
- available stock
- incoming stock
+ reserved/allocated demand adjustment
```

Advanced logic must balance:

```text
availability
cashflow
storage
expiry/waste
supplier risk
margin
customer impact
working capital
budget pressure
strategic product role
```

## 5. Cashflow-Optimized Inventory

Cashflow-optimized inventory must be preserved.

Before reorder, Selene checks:

```text
cash available
minimum cash reserve
upcoming payroll
tax payments
critical AP payments
supplier payment terms
expected customer receipts
budget
profit floor
stockholding cost
expiry risk
gross margin
sales forecast confidence
```

If cashflow is tight, Selene may recommend:

```text
smaller order
split order
delayed second order
supplier with faster delivery
internal stock transfer
B2B clearance of slow stock
promotion to turn old stock into cash
delay non-critical stock
reorder only critical SKUs
renegotiate supplier terms
```

Selene says:

```text
The full reorder protects availability but strains cash. I recommend a smaller order now and a second order after expected receipts clear.
```

## 6. Product Movement And Discontinuation Logic

Product movement and discontinuation logic must not be based on margin alone.

Selene evaluates:

```text
sales velocity
margin
basket attachment
repeat purchase
frequency lift
customer retention
B2B dependency
seasonality
stockholding cost
expiry risk
supplier stability
replacement availability
brand trust effect
promotion response
return rate
dead-stock value
```

Possible recommendations:

```text
keep
increase stock
reduce stock
change supplier
bundle
discount
transfer
B2B clearance
seasonal hold
exit after sell-through
write-off review
discontinue
replace with better product
```

Hard rule:

```text
Do not discontinue a low-margin product without checking customer habit, basket attachment, B2B dependency, and strategic role.
```

## 7. Product Role Categories

Product role categories must include:

```text
Hero Product
Profit Driver
Traffic Driver
Habit Builder
Bundle Support Product
Seasonal Product
Slow Mover
Dead Stock
Discontinue Candidate
Strategic Product
```

### Hero Product

High customer demand, brand importance, or core sales role.

### Profit Driver

Strong margin and meaningful volume.

### Traffic Driver

Low or moderate margin product that brings customers in.

### Habit Builder

Product that creates recurring behavior or routine visits.

### Bundle Support Product

Product that increases basket size or attaches to higher-margin products.

### Seasonal Product

Product with time-bound demand cycles.

### Slow Mover

Low movement but not automatically dead.

### Dead Stock

No meaningful movement, high storage/cash cost, likely clearance or write-off candidate.

### Discontinue Candidate

Product Selene should recommend exiting after role checks.

### Strategic Product

Product that matters for contracts, trust, market positioning, production continuity, or B2B dependency even if short-term margin is weak.

## 8. Low-Margin Customer Habit Products

Low-margin customer habit products must be preserved.

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
repeat visits
basket attachment
cross-sell
CLV
AOV after purchase
frequency lift
retention
B2B dependency
brand trust effect
```

Selene says:

```text
This product has low margin, but customers who buy it often add higher-margin items. I recommend keeping it as a traffic driver while controlling waste tightly.
```

## 9. Restocking Cadence

Restocking must use event-driven and scheduled cadence.

Event-driven triggers:

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
supplier risk changes
budget availability changes
```

Scheduled cadence:

```text
perishable / fresh / high velocity: continuous or hourly
restaurant prep items: daily before prep + intraday adjustment
A-class high-value / high-impact items: daily
fast-moving retail: daily
B-class normal items: 2-3 times per week or weekly
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

## 10. Perishable / JIT Cake Example

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

## 11. Advanced JIT Logic By Business Type

### Perishables

```text
produce base quantity
watch live sales
trigger second batch if sales pace supports it
discount late-day leftovers
stop production before waste risk
write off only with evidence and authority
update next forecast from waste and sellout evidence
```

### Retail

```text
hold lean stock
trigger reorder before lead-time risk
use supplier score
split order if cashflow tight
transfer stock before buying more
protect traffic-driver products from careless discontinuation
clear slow stock before increasing category buy
```

### Manufacturing

```text
order raw materials based on confirmed and forecasted orders
reserve critical materials
avoid overproduction
sequence production by due date and margin
protect strategic reserve for production-stopping inputs
use WIP and finished-goods handoff evidence
```

### B2B

```text
reserve stock for account customers
accept backorders only if supplier/production can meet promise
prioritize high-value or contract customers where policy allows
protect customer-credit and AR boundaries
avoid overselling reserved stock
```

## 12. Stock Transfers Before Buying

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

## 13. Supplier-Aware Replenishment

Supplier behavior changes reorder recommendations.

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
supplier risk
alternative supplier availability
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
critical-stock escalation
```

Selene says:

```text
Supplier A is cheaper but has frequent delays. Supplier B costs 4% more but delivers reliably. For this critical item, I recommend Supplier B.
```

## 14. Inventory Operating Autonomy Mode

Each company must be able to choose how much authority Selene has over inventory ordering.

Selene must support:

```text
Autonomous Inventory Operator Mode
Human Approval / Prepared Order Mode
Hybrid Mode
```

The mode may be set at:

```text
company level
legal entity level
location level
product category level
supplier level
SKU/product level
spend-threshold level
inventory mode level
```

Selene must not use one fixed behavior for every company or every product.

## 15. Autonomous Inventory Operator Mode

In Autonomous Inventory Operator Mode, Selene manages inventory as the operating brain within approved policy.

Selene may automatically:

```text
forecast demand
calculate reorder point
check current stock
check reserved stock
check incoming stock
check supplier lead time
check supplier reliability
check expiry/waste risk
check budget
check cashflow
check minimum cash reserve
check product role
check storage capacity
check tax/compliance blocks
check restricted goods posture
select approved supplier
prepare purchase order
place order if within policy
schedule receiving expectation
track supplier delivery
update inventory after accepted receiving proof
```

Autonomous ordering is allowed only when:

```text
item approved for auto-order
supplier approved
price within expected range
quantity within approved limits
budget available
cashflow safe
minimum reserve protected
no unusual demand spike risk
no supplier risk flag
no tax/compliance block
no restricted goods issue
audit enabled
Procurement policy permits automated reorder
Receiving remains required before available stock increases
AP remains responsible for invoice/payment readiness
Accounting remains responsible for posting
```

Example:

```text
Consumable reorder rule:
approved supplier
monthly usage predictable
low risk
budget available
cashflow safe
quantity within approved range

Selene orders automatically.
Human is notified only if something is unusual.
```

Selene says:

```text
Stock is projected to fall below safe level next week. I’ve ordered 40 cartons from the approved supplier within the monthly budget.
```

## 16. Human Approval / Prepared Order Mode

In Human Approval / Prepared Order Mode, Selene does the work but does not commit the order.

Selene prepares:

```text
what to order
how much to order
why it is needed
supplier recommendation
price comparison
cashflow impact
budget impact
stockout risk
expiry/waste risk
alternative options
recommended decision
purchase order draft
```

Then Selene asks the authorized human to approve.

Example:

```text
Selene prepares PO for 600 units.
Selene explains:
- why 600, not 1,000
- cashflow impact
- stockout risk
- supplier lead time
- alternatives
```

Selene says:

```text
I’ve prepared the reorder. I recommend 600 units now and 400 later after receivables clear. This avoids stockout without putting pressure on payroll cash. Do you want to approve the order?
```

This mode means Selene calculates the decision and the human grants authority.

## 17. Hybrid Mode Should Be The Default Recommendation

Most companies should use Hybrid Mode.

```text
Low-risk predictable items
→ Selene auto-orders within guardrails.

High-value / risky / unusual items
→ Selene prepares recommendation and asks for approval.

Critical stock
→ Selene escalates early before stockout.

Cashflow-sensitive orders
→ Selene proposes split order, transfer, or delay.

Budget-breaking orders
→ Selene searches alternatives before approval request.
```

Example:

```text
Milk for café:
Selene auto-reorders daily/near-daily within limits.

New commercial oven:
Selene prepares options and asks for approval.

Critical machine part:
Selene may auto-order if stockout would stop production and policy allows emergency replenishment.

Slow-moving luxury product:
Selene asks before reordering.
```

## 18. InventoryAutonomyPolicy Fields

```text
InventoryAutonomyPolicy:
  policy_id
  company_id
  legal_entity_id
  location_id
  product_id optional
  product_category optional
  supplier_id optional
  inventory_mode
  autonomy_mode:
    - autonomous_operator
    - prepared_order_human_approval
    - hybrid
  max_auto_order_amount
  max_auto_order_quantity
  max_auto_order_frequency
  approved_suppliers
  budget_required
  cashflow_required
  reserve_check_required
  approval_required_above_threshold
  exception_rules
  audit_ref
```

Additional policy dimensions may include:

```text
restricted_goods_allowed
tax_compliance_required
supplier_risk_block_threshold
price_variance_threshold
demand_spike_review_threshold
critical_stock_emergency_policy
allowed_order_days
receiving_expectation_window
notification_recipients
step_up_required_above_threshold
policy_effective_from
policy_effective_to
approved_by
```

## 19. Autonomy Access And Authority

Protected autonomy actions:

```text
enable autonomous ordering
change autonomy mode
approve auto-order supplier
approve auto-order product/SKU
change spend threshold
override cashflow block
override budget block
approve restricted goods order
approve emergency replenishment
disable autonomy
export inventory autonomy policy
```

Authority depends on:

```text
company
legal entity
location
role
product category
supplier
stock value
cashflow impact
budget impact
restricted goods risk
tax/compliance risk
critical-stock status
board policy
```

## 20. PH1.D / GPT-5.5 Role

Allowed:

```text
explain reorder recommendation
draft supplier comparison summary
summarize stockout risk
summarize cashflow impact
explain product role reasoning
draft human approval request
explain why autonomy did or did not trigger
identify missing assumptions
```

Forbidden:

```text
create final stock truth
approve reorder
place order
override policy
invent supplier lead time
invent supplier reliability
invent sales forecast truth
approve write-off
approve valuation
post journal
grant authority
```

## 21. PH1.WRITE Wording

PH1.WRITE owns final human-facing explanations.

### Autonomous Order

```text
Stock is projected to fall below safe level next week, and this item is approved for automatic reorder. I placed the order within budget and cashflow limits.
```

### Prepared Order

```text
I’ve prepared the reorder recommendation and PO draft. The safer choice is 600 units now and 400 later after expected customer receipts clear.
```

### Hybrid Exception

```text
This item normally auto-orders, but supplier reliability has dropped and the order is above threshold. I’m asking for approval before committing it.
```

### Cashflow Block

```text
The reorder would protect stock but strain cash. I recommend transferring stock first and placing a smaller order.
```

## 22. Audit Requirements

Audit must record:

```text
audit_event_id
actor_id
action
company_id
legal_entity_id
location_id
product_id
supplier_id
policy_id
autonomy_mode
inventory_mode
stock_level_ref
forecast_ref
supplier_score_ref
cashflow_ref
budget_ref
reserve_check_ref
tax_compliance_ref
reorder_recommendation_ref
purchase_order_ref
receiving_expectation_ref
approval_refs
exception_reason
timestamp
reason_code
```

No silent autonomous order.

No unlogged policy change.

No available inventory increase without accepted receiving proof.

## 23. Failure Branches

### Policy Missing

```text
Autonomous ordering blocked.
Selene prepares recommendation for human approval.
```

### Supplier Not Approved

```text
Auto-order blocked.
Selene recommends approved supplier or routes supplier approval.
```

### Price Abnormal

```text
Auto-order blocked.
Selene prepares price variance explanation and alternatives.
```

### Cashflow Unsafe

```text
Auto-order blocked.
Selene proposes split order, transfer, delay, or clearance.
```

### Budget Blocked

```text
Auto-order blocked.
Selene searches alternatives before approval request.
```

### Supplier Reliability Poor

```text
Auto-order may be blocked if item is critical.
Selene recommends backup supplier, strategic reserve, or escalation.
```

### Unusual Demand Spike

```text
Auto-order blocked or reduced.
Selene asks whether the spike is real, temporary, promotion-driven, or an anomaly.
```

### Receiving Evidence Missing

```text
Stock remains incoming or inspection hold.
Inventory does not add available stock.
AP payment may remain blocked or limited.
```

## 24. Required Logical Packets

Future logical packets:

```text
InventoryAutonomyPolicyPacket
InventoryAutonomyDecisionPacket
AutonomousReorderExecutionPacket
PreparedOrderRecommendationPacket
HybridInventoryModePacket
InventoryModeDecisionPacket
JITReorderIntelligencePacket
CashflowOptimizedInventoryPacket
ProductRoleIntelligencePacket
LowMarginHabitProductPacket
RestockingCadencePacket
PerishableProductionPlanPacket
StagedReplenishmentPacket
StockTransferBeforeBuyPacket
SupplierReliabilityReorderPacket
ProcurementAutomationHandoffPacket
ReceivingExpectationPacket
InventoryAutonomyAuditEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 25. What Must Not Happen

```text
no autonomous ordering without company policy
no auto-order from unapproved supplier
no auto-order if cashflow is unsafe
no auto-order if budget is blocked
no auto-order if price is abnormal
no auto-order if supplier reliability is poor and item is critical
no auto-order if unusual demand spike risk is unresolved
no auto-order if tax/compliance block exists
no auto-order if restricted goods issue exists
no human approval mode that leaves humans to calculate everything manually
no prepared order without Selene recommendation
no inventory autonomy bypassing Procurement, Budget, Cashflow, Receiving, AP, or Audit
no Inventory bypassing Accounting for valuation or COGS
no Product and Inventory merge
no Supplier, Procurement, and Receiving merge
no GPT-5.5 final stock truth or authority
no implementation from this addendum alone
```

## 26. Future Simulation Targets

```text
SIM_INV_AUTO_001_autonomous_order_low_risk_item_within_policy
SIM_INV_AUTO_002_autonomous_order_blocked_no_policy
SIM_INV_AUTO_003_auto_order_blocked_unapproved_supplier
SIM_INV_AUTO_004_auto_order_blocked_cashflow_unsafe
SIM_INV_AUTO_005_auto_order_blocked_budget_unavailable
SIM_INV_AUTO_006_auto_order_blocked_price_abnormal
SIM_INV_AUTO_007_hybrid_mode_prepared_order_for_high_value_item
SIM_INV_AUTO_008_hybrid_mode_auto_orders_predictable_low_risk_stock
SIM_INV_AUTO_009_prepared_order_includes_cashflow_budget_stockout_and_alternatives
SIM_INV_AUTO_010_critical_stock_escalates_early
SIM_INV_AUTO_011_transfer_before_buy_recommended
SIM_INV_AUTO_012_low_margin_habit_product_kept
SIM_INV_AUTO_013_perishable_staged_production_reduces_waste
SIM_INV_AUTO_014_receiving_proof_required_before_stock_available
SIM_INV_AUTO_015_inventory_autonomy_does_not_bypass_procurement_ap_accounting_audit
```

## 27. Final Rule

```text
Selene supports inventory autopilot and inventory co-pilot.

Autopilot:
Selene orders within safe rules.

Co-pilot:
Selene prepares everything and asks the human to approve.

Hybrid:
Selene auto-orders the boring safe stock and asks humans only for meaningful decisions.
```

## 28. Final Addendum Architecture Sentence

Selene Inventory Addendum A makes Inventory autonomous where safe and deeply assistive where authority is required: Selene forecasts demand, understands supplier reliability, protects cashflow and budget, uses JIT where reliable, keeps buffers where critical, values low-margin habit products correctly, stages production for perishables and uncertainty, checks transfers before buying, and supports company-selectable Autonomous Inventory Operator, Human Approval / Prepared Order, and Hybrid modes without merging Product, Inventory, Supplier, Procurement, Receiving, AP, Accounting, Cashflow, Access, PH1.D, PH1.WRITE, or Audit.
