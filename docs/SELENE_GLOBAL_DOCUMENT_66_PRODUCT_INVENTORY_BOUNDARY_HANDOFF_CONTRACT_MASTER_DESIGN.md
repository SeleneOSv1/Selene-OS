# Global Document 66 — Product ↔ Inventory Boundary + Handoff Contract

```text id="doc66_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
66

ENGINE:
PH1.PRODUCT_INVENTORY_BOUNDARY / PH1.PRODUCT_INVENTORY_HANDOFF

FULL NAME:
Selene Product ↔ Inventory Boundary, Source-of-Truth, Handoff, Stock Template, Availability, Traceability, and Change-Control Contract

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

This document defines the exact boundary between:

```text
Global Document 64 — Selene Product Intelligence Engine
Global Document 65 — Selene Inventory Intelligence Engine
```

It answers:

```text
What does Product own?
What does Inventory own?
What data is shared?
What happens when Product changes?
What happens when stock exists?
What happens when products are duplicated, merged, discontinued, recalled, or made B2B-ready?
How does Product create inventory templates?
How does Inventory update stock availability back to Product and commerce channels?
What must never be mixed?
```

This is the contract that prevents Selene from becoming a tragic ERP swamp where:

```text
Product owns quantity.
Inventory owns descriptions.
Supplier owns barcodes.
Accounting owns stock counts.
Everyone cries into spreadsheets.
```

Absolutely not. We are building Selene, not a haunted stockroom with tabs.

---

## 2. Core Boundary Law

```text
Product owns product truth.
Inventory owns stock truth.
Neither engine may overwrite the other’s truth.
All shared changes must pass through defined handoff packets, versioned rules, audit, and authority where required.
```

Simple:

```text
Product = what the thing is.
Inventory = how many exist, where they are, and what state they are in.
```

Example:

```text
Product:
Organic Shampoo 500ml, category, barcode, description, images, expiry required, B2B eligible.

Inventory:
143 units, 82 in Store A, 41 in Warehouse B, 20 reserved, 3 damaged, batch B-12 expires in 40 days.
```

If anyone tries to merge those jobs, please spray the architecture with cold water.

---

## 3. Source-of-Truth Matrix

| Data / Decision                     | Product Owns |                     Inventory Owns | Notes                                                     |
| ----------------------------------- | -----------: | ---------------------------------: | --------------------------------------------------------- |
| Product name                        |            ✅ |                                  ❌ | Inventory displays but does not edit                      |
| Product description                 |            ✅ |                                  ❌ | Used by commerce channels                                 |
| Product photos/videos               |            ✅ |                                  ❌ | Inventory may capture damage/stock photos separately      |
| SKU / product identity              |            ✅ |                                  ❌ | Inventory references SKU/product_id                       |
| Barcode / GTIN / QR / GS1 readiness |            ✅ |                                  ❌ | Inventory scans but does not create identity codes        |
| Product category                    |            ✅ |                                  ❌ | Inventory may use category for count priority             |
| Product attributes                  |            ✅ |                                  ❌ | Size, colour, material, ingredients, etc.                 |
| Unit of measure definition          |            ✅ |                             Shared | Product defines; Inventory stores stock in canonical unit |
| Pack/case/carton definitions        |            ✅ |                             Shared | Inventory uses for receiving/reorder/count                |
| Batch required flag                 |            ✅ |                                  ❌ | Inventory captures actual batch values                    |
| Serial required flag                |            ✅ |                                  ❌ | Inventory captures actual serial values                   |
| Expiry required flag                |            ✅ |                                  ❌ | Inventory captures actual expiry dates                    |
| Storage requirement                 |            ✅ |                             Shared | Inventory enforces location/handling                      |
| Temperature requirement             |            ✅ |                             Shared | Inventory enforces cold-chain/hold logic                  |
| Product compliance requirements     |            ✅ |                             Shared | Inventory may block sale/recall stock                     |
| Product passport fields             |            ✅ |                                  ❌ | Inventory may provide traceability events                 |
| Channel readiness                   |            ✅ |                             Shared | Inventory provides stock availability                     |
| Stock quantity                      |            ❌ |                                  ✅ | Product may display but cannot change                     |
| Stock location                      |            ❌ |                                  ✅ | Product may show summary only                             |
| Stock reservation                   |            ❌ |                                  ✅ | Product does not reserve stock                            |
| Available-to-promise                |            ❌ |                                  ✅ | Product displays availability through Inventory           |
| Stock movement                      |            ❌ |                                  ✅ | Inventory event truth                                     |
| Stock damage                        |            ❌ |                                  ✅ | Product may use for health score                          |
| Stock expiry risk                   |            ❌ |                                  ✅ | Product may show warning                                  |
| Reorder signal                      |            ❌ |                                  ✅ | Procurement acts on it                                    |
| Supplier product link               |            ✅ |                             Shared | Supplier performance from Supplier Engine                 |
| Product discontinuation             |            ✅ |                             Shared | Inventory manages remaining stock plan                    |
| Product merge                       |            ✅ |                             Shared | Inventory must reconcile active stock                     |
| Inventory valuation                 |            ❌ | Accounting with Inventory evidence | Product provides category hints only                      |

---

## 4. Engine Ownership

## 4.1 PH1.PRODUCT owns

```text
product_id
product identity
product title
product category
product attributes
product variants
product bundles
product descriptions
product media
product barcodes / identifiers
supplier product data link
compliance requirements
product passport requirements
channel presentation
POS listing data
e-commerce listing data
B2B listing data
marketplace feed data
inventory template requirements
product lifecycle state
```

## 4.2 PH1.INVENTORY owns

```text
quantity on hand
quantity available
quantity reserved
quantity allocated
quantity in transit
quantity damaged
quantity quarantined
quantity expired
location
warehouse/bin/shelf
batch/lot/serial values
expiry dates
stock movement
stock count
stock transfer
stock health
stockout risk
overstock risk
reorder recommendation
JIT recommendation
FEFO/FIFO picking
inventory audit evidence
```

## 4.3 Shared but controlled areas

```text
unit of measure
pack/case/carton conversion
expiry/batch/serial requirements
storage requirements
channel availability
product discontinuation
product recall
product merge
inventory template
commerce availability
traceability
```

Shared does **not** mean “both can edit randomly like toddlers with markers.”

Shared means:

```text
one engine owns the definition
the other engine consumes/enforces it
changes are versioned
changes are audited
dangerous changes require migration or approval
```

---

## 5. Product-to-Inventory Handoff

When Product creates or updates a product, it sends Inventory an inventory template.

```text
Product → Inventory:
InventoryTemplatePacket
```

## 5.1 Inventory template fields

```text
product_id
variant_id
track_stock yes/no
canonical_unit_of_measure
sell_unit
purchase_unit
pack_size
case_quantity
carton_quantity
batch_required
lot_required
serial_required
expiry_required
shelf_life_required
temperature_required
storage_requirement
hazardous_flag
regulated_flag
default_stock_locations
default_reorder_category
default_stocktake_priority
default_valuation_category_hint
default_FEFO_required
default_FIFO_required
audit_ref
```

## 5.2 Product creates template

Example:

```text
Product:
Organic Milk 1L
expiry_required = true
temperature_required = refrigerated
FEFO_required = true
unit = bottle
case_quantity = 12

Inventory:
creates tracking template
requires expiry on receiving
restricts putaway to refrigerated locations
uses FEFO for picking
```

## 5.3 Product update triggers Inventory review

Product changes may trigger Inventory impact.

```text
unit of measure changed
pack size changed
expiry requirement changed
batch/serial requirement changed
storage requirement changed
regulated flag changed
product discontinued
product recalled
product merged
variant changed
channel availability changed
```

Selene must not silently apply dangerous changes when live stock exists.

Example:

> “Changing unit of measure from bottle to carton affects 1,240 units currently in stock. I’ll prepare an inventory migration review.”

Good. No casually turning bottles into cartons because someone edited a field and felt powerful.

---

## 6. Inventory-to-Product Handoff

Inventory sends Product stock and health signals.

```text
Inventory → Product:
ProductStockStatusPacket
ProductAvailabilityPacket
ProductStockHealthPacket
ProductExpiryRiskPacket
ProductDiscontinuationSignalPacket
```

## 6.1 Inventory sends

```text
product_id
variant_id
quantity_available
quantity_reserved
stockout_risk
overstock_risk
expiry_risk
damaged_stock_count
quarantine_count
recall_hold_status
location_summary
channel_allocation_summary
inventory_health_score
recommended_action
audit_ref
```

## 6.2 Product uses this to show

```text
online availability
B2B availability
POS availability
product health warnings
supplier/product risk warnings
promotion readiness
discontinue candidate warnings
```

Product may display:

> “Low stock.”

But Product may not change stock.

Inventory may say:

> “Only 12 units available.”

Product may use that for commerce readiness.

Product cannot magically “set available to 50” because marketing got excited. We’ve met marketing. Lovely people. Dangerous with stock.

---

## 7. Shared Identity and Units

## 7.1 Product defines canonical units

Product owns unit definitions:

```text
each
bottle
carton
case
kg
g
litre
ml
metre
hour
service unit
subscription period
```

Inventory stores stock using canonical unit plus pack conversions.

Example:

```text
Product unit:
bottle

Pack:
1 case = 12 bottles

Inventory:
120 bottles available
or 10 cases equivalent
```

## 7.2 Unit changes require control

If no stock exists:

```text
Product can update unit under policy.
Inventory template updates.
```

If stock exists:

```text
Product change triggers InventoryMigrationRequired.
Inventory must convert quantities safely.
Audit required.
Authority may be required.
```

Selene says:

> “This product has active stock. Changing unit from each to carton requires inventory conversion. I’ll prepare a migration plan.”

This is how we avoid turning 100 bottles into 100 cartons and briefly becoming billionaires in a very false way.

---

## 8. Variants and Inventory

Product owns variants.

Inventory tracks stock by variant where relevant.

Example:

```text
Product:
T-shirt

Variants:
Small / Black
Medium / Black
Large / Black
Small / White
```

Inventory:

```text
Small Black: 12
Medium Black: 18
Large Black: 7
Small White: 0
```

## 8.1 Variant creation

Product creates variant.

Inventory creates stock template for variant.

## 8.2 Variant deletion

If no stock and no transaction history:

```text
variant can be archived
```

If stock/history exists:

```text
variant cannot be deleted casually
must be discontinued or merged
Inventory impact review required
```

Selene says:

> “This variant has stock and sales history. I cannot delete it. I can mark it discontinued or merge it after review.”

No deleting reality because the dropdown is untidy.

---

## 9. Product Merge and Duplicate Handling

Product detects duplicates.

Inventory must protect stock truth during merge.

## 9.1 Duplicate scenarios

```text
same product entered twice
same barcode
same supplier SKU
same product with different spelling
same product imported from old store and supplier invoice
```

## 9.2 Merge rules

If no stock/history:

```text
Product may merge under policy.
```

If stock exists:

```text
Inventory merge review required.
Stock balances must be reconciled.
Transactions must be preserved.
Audit required.
```

Handoff:

```text
ProductDuplicateMergeRequestPacket
InventoryMergeImpactPacket
ProductMergeApprovalPacket
```

Selene says:

> “These look like the same product. One record has 20 units in stock and the other has 14. I’ll prepare a merge plan instead of merging automatically.”

Good. Product deduplication is where stock truth goes to die if nobody is careful.

---

## 10. Product Discontinuation and Inventory Runout

Product owns product lifecycle.

Inventory owns remaining stock plan.

## 10.1 Product discontinuation states

```text
Active
DiscontinuePlanned
DiscontinuedForPurchase
DiscontinuedForSale
Archived
```

## 10.2 Inventory actions

When product is marked DiscontinuePlanned:

```text
stop reorder recommendation
identify stock on hand
identify reserved stock
identify incoming POs
recommend clearance
recommend transfer
recommend supplier return if allowed
recommend bundle/discount
block future purchasing if policy says
```

Selene says:

> “This product is marked for discontinuation. There are 240 units remaining and 80 incoming from Supplier ABC. I recommend cancelling the incoming PO if possible.”

Product decides lifecycle.

Inventory cleans up the stock mess.

---

## 11. Compliance Holds and Inventory Holds

Product owns compliance requirements.

Inventory enforces stock availability holds.

## 11.1 Product compliance hold

Product sets:

```text
ComplianceHold
ComplianceDataMissing
RestrictedSaleFlag
RecallRequired
```

Inventory responds:

```text
block sale
block channel availability
quarantine affected stock
trigger recall workflow if needed
notify commerce channels
```

Example:

> “This product is missing a safety certificate. I will block e-commerce and B2B sale until compliance is approved.”

Product knows the rule.

Inventory blocks the stock.

Commerce stops selling.

Everyone lives slightly longer.

---

## 12. Recall Boundary

Recall may originate from Product, Supplier, Compliance, or Inventory.

## 12.1 Product recall trigger

Product sends:

```text
ProductRecallTriggerPacket
```

Fields:

```text
product_id
variant_id
batch/lot/serial criteria
recall reason
effective date
sale block required
customer identification required
supplier claim required
compliance evidence required
```

## 12.2 Inventory action

Inventory:

```text
identifies affected stock
blocks affected stock
identifies locations
identifies shipments/customers where possible
updates stock state to Recalled
triggers Returns/Logistics/Customer notification workflows
```

Selene says:

> “Product recall applies to Batch B-442. I blocked remaining stock and identified 18 customers who received it.”

Product says what is recalled.

Inventory finds the physical reality.

---

## 13. Expiry / Batch / Serial Requirements

Product defines whether tracking is required.

Inventory captures actual values.

## 13.1 Product flags

```text
expiry_required
batch_required
lot_required
serial_required
temperature_required
```

## 13.2 Inventory enforces

```text
Receiving cannot accept without required batch/expiry/serial.
Stock cannot become available until required data exists.
FEFO/FIFO rules apply.
Channel availability respects expiry and recall.
```

Example:

> “This product requires expiry tracking. Receiving cannot mark it accepted until expiry date is captured.”

This avoids the classic “we have stock but no idea whether it expires tomorrow” game. A game with no winners and many refunds.

---

## 14. Channel Availability Boundary

Product controls channel eligibility.

Inventory controls channel availability.

## 14.1 Product says

```text
can sell on POS
can sell on e-commerce
can sell on B2B
can sell on marketplace
restricted by compliance
requires age check
requires customer account
```

## 14.2 Inventory says

```text
quantity available for each channel
quantity reserved
stockout risk
location availability
fulfillment possibility
backorder possibility
```

Example:

```text
Product:
B2B eligible = yes

Inventory:
Available for B2B = 0 because all stock reserved for retail and existing customer orders.
```

Selene says:

> “This product is B2B-ready but not currently available for B2B because available stock is fully reserved.”

Eligibility is not availability. A tiny concept with huge consequences.

---

## 15. Product Passport and Traceability Boundary

Product owns passport fields.

Inventory owns traceability events.

## Product owns

```text
materials
composition
manufacturer
supplier certificates
repair instructions
recycling instructions
warranty
compliance documents
product passport schema
```

## Inventory owns

```text
batch movements
location history
receiving events
customer shipment events
stock state events
recall stock events
```

Product passport may need Inventory data.

Handoff:

```text
InventoryTraceabilityEventPacket
ProductPassportTraceabilityPacket
```

Selene says:

> “The product passport is missing traceability events for Batch B-442. Inventory will supply receiving and movement history.”

Product explains the product.

Inventory proves what happened to stock.

---

## 16. Supplier Product Data and Inventory Reorder

Product links supplier product information.

Inventory uses supplier-related operational signals, but Supplier Engine owns supplier performance.

## Product owns supplier product metadata

```text
supplier_sku
supplier_product_name
pack_size
case_quantity
MOQ
supplier catalog description
supplier images
supplier certificates
country_of_origin
```

## Inventory uses

```text
lead time
pack size
MOQ
case quantity
supplier availability signal
```

## Supplier Engine owns

```text
supplier quality
supplier delivery reliability
supplier restrictions
supplier obligations
supplier bank/payment risk
```

Example:

> “Supplier SKU changed. Product updated supplier product link. Inventory reorder rules need review because pack size changed.”

Selene must coordinate, not let one supplier file quietly ruin reorder math.

---

## 17. Pricing, Margin, and Inventory

Product may show price presentation.

Inventory may recommend markdown.

Pricing Engine owns final pricing.

Inventory can say:

```text
overstock markdown recommended
expiry discount recommended
dead stock clearance recommended
B2B bundle recommended
```

Product can say:

```text
product role
market positioning
content/offer presentation
```

Pricing decides:

```text
approved price
discount limit
margin floor
campaign price
```

Selene says:

> “Inventory recommends markdown due to expiry risk. Pricing must confirm discount stays above margin floor.”

No discounting stock into a loss hole unless someone with authority enjoys pain.

---

## 18. Product Changes That Require Inventory Impact Review

Selene must require impact review when Product changes:

```text
canonical unit of measure
pack/case/carton conversion
variant structure
batch/serial/expiry requirement
storage requirement
regulated/compliance status
product merge
product split
product discontinuation
SKU/barcode identity
tax/accounting classification hint
channel eligibility
```

Impact review checks:

```text
active stock
reserved stock
open orders
open POs
receiving in progress
stock counts
valuation implications
commerce availability
traceability obligations
```

Selene says:

> “This product change affects active inventory and open orders. I’ll prepare an impact review before applying it.”

That is change control. Not red tape. Difference matters.

---

## 19. Inventory Changes That Inform Product

Inventory must notify Product when:

```text
stockout risk high
overstock risk high
dead stock risk
expiry risk
supplier reliability affects sale-readiness
recall hold
quarantine hold
high return/damage rate
stock health poor
stock unavailable for channel
```

Product may respond by:

```text
pausing promotion
changing product health score
updating product recommendation
flagging listing warning
suggesting media/positioning improvement
suggesting discontinuation review
```

Example:

> “Inventory reports high returns and damage. Product Health Score should move to Risk and supplier/product quality review should open.”

Inventory doesn’t rewrite product facts.

It informs product health.

---

## 20. Handoff Packet Library

## 20.1 Product → Inventory

```text
InventoryTemplatePacket
ProductTrackingRequirementPacket
ProductUOMChangePacket
ProductVariantChangePacket
ProductChannelEligibilityPacket
ProductComplianceHoldPacket
ProductRecallTriggerPacket
ProductDiscontinuationPacket
ProductMergeRequestPacket
ProductSupplierPackChangePacket
```

## 20.2 Inventory → Product

```text
ProductStockStatusPacket
ProductAvailabilityPacket
ProductStockHealthPacket
ProductExpiryRiskPacket
ProductRecallStockPacket
InventoryTraceabilityEventPacket
InventoryProductDiscontinuationSignalPacket
InventoryMarkdownSignalPacket
InventoryChannelAvailabilityPacket
```

## 20.3 Shared governance packets

```text
ProductInventoryImpactReviewPacket
ProductInventoryMigrationPacket
ProductInventoryConflictPacket
ProductInventoryAuditPacket
```

Logical only. Codex maps later. No packet pottery today.

---

## 21. Conflict Resolution Rules

## 21.1 Product says sellable, Inventory says no stock

Result:

```text
Product remains channel-ready.
Inventory availability = zero.
Commerce shows unavailable/out of stock/backorder depending policy.
```

## 21.2 Product says compliance hold, Inventory says stock available

Result:

```text
Inventory blocks sellable state.
Commerce sale blocked.
```

## 21.3 Inventory says stock expired, Product says active

Result:

```text
Expired stock blocked.
Product remains active for non-expired stock if any.
```

## 21.4 Product changes UOM while stock exists

Result:

```text
Change held pending impact review.
No silent conversion.
```

## 21.5 Duplicate product merge affects stock

Result:

```text
Merge requires inventory reconciliation and audit.
```

## 21.6 Product discontinued but stock remains

Result:

```text
Inventory creates runout/clearance/return plan.
No automatic deletion.
```

---

## 22. End-to-End Examples

## 22.1 New perishable product

```text
Product:
Creates Organic Yogurt 500g.
Sets expiry_required = true.
Sets refrigerated storage.
Sets FEFO.

Inventory:
Creates inventory template.
Receiving must capture expiry.
Inventory stores refrigerated.
FEFO pick required.
```

## 22.2 B2B-ready product but no stock

```text
Product:
B2B listing ready.

Inventory:
No available B2B stock.

Output:
Product shows B2B-ready but unavailable.
Procurement may receive reorder signal.
```

## 22.3 Product recall

```text
Product/Compliance:
Recall Batch B-442.

Inventory:
Blocks stock.
Finds locations.
Finds affected customers.
Triggers returns/logistics/customer workflows.
```

## 22.4 Product merge

```text
Product:
Two shampoo records appear duplicate.

Inventory:
Record A has 20 units.
Record B has 14 units.

Selene:
Prepares merge impact review.
Does not auto-merge live stock.
```

## 22.5 Unit conversion

```text
Product:
Pack size changes from 12 to 24 per carton.

Inventory:
Open POs and stock counts may be affected.
Reorder math changes.
Impact review required.
```

---

## 23. Automation and Exception-Only Review

Selene auto-handles:

```text
inventory template creation
routine tracking requirement handoff
channel availability updates
stock health feedback
expiry warnings to Product
product readiness updates from inventory availability
low-risk product template updates with no stock
```

Selene requires review for:

```text
UOM change with active stock
pack conversion change with open POs
product merge with active stock
product deletion/discontinuation with stock
compliance hold release
recall closure
batch/serial/expiry requirement changes with existing stock
stock-impacting product identity changes
```

Routine handoff should be automatic.

Risky change should be reviewed.

No approval circus. No silent chaos. Both are bad; one just wears more forms.

---

## 24. PH1.D / GPT-5.5 Role

GPT-5.5 may help:

```text
explain boundary conflicts
summarize product/inventory impact
draft human-friendly review prompts
explain UOM conversion issue
summarize stock availability for product users
draft merge/discontinuation explanations
```

GPT-5.5 must not:

```text
change stock counts
approve product merge
approve UOM migration
release compliance hold
invent inventory availability
delete products
alter product identity
override source-of-truth ownership
```

GPT-5.5 explains the problem.

Selene deterministic engines decide what is safe.

---

## 25. State Machines

## 25.1 Product Inventory Template State

```text
NotCreated
PendingInventoryTemplate
TemplateCreated
TemplateActive
TemplateNeedsReview
TemplateMigrating
TemplateRetired
```

## 25.2 Product Inventory Impact Review State

```text
NotRequired
ImpactDetected
ReviewPending
InventoryMigrationRequired
Approved
Rejected
Applied
Archived
```

## 25.3 Product Merge Impact State

```text
Candidate
NoStockSafeMerge
StockImpactDetected
ReconciliationRequired
Approved
Merged
Rejected
Archived
```

## 25.4 Product Discontinuation Inventory State

```text
Active
RunoutPlanned
ReorderBlocked
ClearanceRecommended
TransferRecommended
RemainingStockDisposed
Closed
```

## 25.5 Compliance-to-Inventory Hold State

```text
NoHold
ComplianceHoldRequested
InventoryHoldApplied
CommerceBlocked
ReviewPending
HoldReleased
Archived
```

---

## 26. Reason Codes

```text
PRODUCT_INVENTORY_TEMPLATE_CREATED
PRODUCT_INVENTORY_TEMPLATE_NEEDS_REVIEW
PRODUCT_UOM_CHANGE_STOCK_IMPACT
PRODUCT_PACK_CHANGE_STOCK_IMPACT
PRODUCT_VARIANT_STOCK_IMPACT
PRODUCT_COMPLIANCE_HOLD_INVENTORY_BLOCK
PRODUCT_RECALL_INVENTORY_BLOCK
PRODUCT_DISCONTINUATION_STOCK_REMAINING
PRODUCT_MERGE_STOCK_RECONCILIATION_REQUIRED
INVENTORY_AVAILABILITY_UPDATED_PRODUCT
INVENTORY_STOCKOUT_PRODUCT_WARNING
INVENTORY_EXPIRY_PRODUCT_WARNING
INVENTORY_DEAD_STOCK_PRODUCT_WARNING
INVENTORY_MARKDOWN_PRODUCT_SIGNAL
PRODUCT_CHANNEL_READY_BUT_NO_STOCK
PRODUCT_B2B_READY_BUT_NO_B2B_ALLOCATION
PRODUCT_PASSPORT_TRACEABILITY_REQUIRED
```

---

## 27. Required Simulations

```text
product creates inventory template
product creates perishable inventory template
product creates serialized inventory template
product UOM change with no stock
product UOM change with active stock blocked
product pack size change affects reorder
product variant creation creates inventory template
product variant deletion blocked by stock
product merge with no stock
product merge with active stock requires reconciliation
product compliance hold blocks inventory availability
product recall identifies inventory stock
inventory stockout updates product status
inventory expiry risk updates product status
inventory dead stock suggests product discontinuation
product discontinued with remaining stock creates runout plan
B2B-ready product has no B2B stock allocation
inventory traceability feeds product passport
```

---

## 28. Integration Map

```text
PH1.PRODUCT_INVENTORY_BOUNDARY
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.SUPPLIER
↔ PH1.PROCUREMENT
↔ PH1.PROC.RECEIVE
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.PRICING
↔ PH1.MARKETING
↔ PH1.COMPLIANCE
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.AUDIT
↔ PH1.ACCESS / AUTHORITY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 29. Required Logical Packets

```text
InventoryTemplatePacket
ProductTrackingRequirementPacket
ProductUOMChangePacket
ProductVariantChangePacket
ProductChannelEligibilityPacket
ProductComplianceHoldPacket
ProductRecallTriggerPacket
ProductDiscontinuationPacket
ProductMergeRequestPacket
ProductSupplierPackChangePacket
ProductStockStatusPacket
ProductAvailabilityPacket
ProductStockHealthPacket
ProductExpiryRiskPacket
ProductRecallStockPacket
InventoryTraceabilityEventPacket
InventoryProductDiscontinuationSignalPacket
InventoryMarkdownSignalPacket
InventoryChannelAvailabilityPacket
ProductInventoryImpactReviewPacket
ProductInventoryMigrationPacket
ProductInventoryConflictPacket
ProductInventoryAuditPacket
```

---

## 30. What Codex Must Not Do

```text
Do not merge Product and Inventory.
Do not let Product own stock quantity.
Do not let Inventory own product identity.
Do not let Product change unit of measure with active stock without impact review.
Do not let Product delete variants with stock/history.
Do not let duplicate product merges alter stock without reconciliation.
Do not let Inventory mark compliance-held products as sellable.
Do not let Product publish availability without Inventory truth.
Do not let Inventory change product descriptions/media/category.
Do not create runtime packet structs from this document.
Do not implement from this document alone.
```

---

## 31. Final Architecture Sentence

Selene Product ↔ Inventory Boundary + Handoff Contract is the source-of-truth agreement that keeps Product responsible for product identity, attributes, categories, media, variants, compliance requirements, channel eligibility, and inventory template definitions, while Inventory owns quantities, locations, stock states, reservations, movements, batch/lot/serial/expiry values, stock health, reorder signals, and availability, with every shared change flowing through versioned handoff packets, impact reviews, audit, and authority where needed so Selene can automate routine product-stock coordination without corrupting product truth or stock truth.

Simple version:

```text
Product says what the item is.
Inventory says how many exist and where.
Product defines what must be tracked.
Inventory tracks the real stock.
Product says what channels are allowed.
Inventory says what stock is available.
Product can discontinue or recall.
Inventory handles remaining stock.
Product cannot change stock counts.
Inventory cannot change product identity.
Everything important is handed off, reviewed, and audited.
```

That is Global Document 66. The boundary wall is now built: Product can make the shampoo beautiful, Inventory can say there are only twelve bottles left, and neither one gets to wander into the other’s office wearing a fake moustache.
