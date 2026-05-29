# Global Document 67 — Product + Inventory Codex Readiness Overview

```text id="doc67_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE OVERVIEW / CODEX READINESS CONTROL DOCUMENT

GLOBAL DOCUMENT NUMBER:
67

PACKAGE:
PRODUCT + INVENTORY FOUNDATION MINI-BATCH

FULL NAME:
Selene Product + Inventory Codex Readiness, Ownership, Handoff, Simulation, Indexing, and Repo-Truth Preparation Overview

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_CODEX_INSERTION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_BATCH_CONTROL_DOCUMENT
```

---

## 1. Purpose

This document controls the first **post-Finance global architecture mini-batch**:

```text
Global Document 64 — Selene Product Intelligence Engine
Global Document 65 — Selene Inventory Intelligence Engine
Global Document 66 — Product ↔ Inventory Boundary + Handoff Contract
Global Document 67 — Product + Inventory Codex Readiness Overview
```

This document tells Codex how to add the Product + Inventory foundation cleanly, without creating:

```text
duplicate source-of-truth owners
runtime code
packet structs
database schemas
API routes
migrations
fake implementation
architecture soup
```

The Product + Inventory batch exists because Selene cannot properly manage suppliers, procurement, receiving, AP, commerce, B2B, POS, or accounting until she first knows:

```text
what the product is
how the product is sold
what stock exists
where stock is
what condition stock is in
what is available
what is reserved
what is expiring
what should be reordered
```

Product is identity.

Inventory is stock truth.

Boundary Contract prevents them from stealing each other’s chairs like badly raised ERP modules.

---

## 2. Batch Scope

This batch contains exactly four global master documents.

| Global # | Document                                            | Role                                                                                                                                       |
| -------: | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
|       64 | **Selene Product Intelligence Engine**              | Product identity, presentation, media, attributes, channel readiness, compliance fields, supplier product data, product passport readiness |
|       65 | **Selene Inventory Intelligence Engine**            | Stock quantity, location, movement, reservation, expiry, batch/serial, JIT, reorder, stock health, working capital intelligence            |
|       66 | **Product ↔ Inventory Boundary + Handoff Contract** | Source-of-truth split, handoff packets, impact review rules, conflict prevention                                                           |
|       67 | **Product + Inventory Codex Readiness Overview**    | Codex batch control, verification, indexing, simulation map, acceptance tests                                                              |

No supplier/procurement/AP documents belong in this batch.

Those come after this batch is committed, pushed, and verified.

Codex gets a neat lunchbox, not the entire buffet thrown at its little repo face.

---

## 3. Expected Global Numbering After Codex Inserts This Batch

Current known state before this batch:

```text
Highest global master architecture document number: 63
Next expected global document number: 64
Total indexed linked files: 76
```

If Codex adds these four documents as four linked files:

```text
64 = Product Intelligence Engine
65 = Inventory Intelligence Engine
66 = Product ↔ Inventory Boundary + Handoff Contract
67 = Product + Inventory Codex Readiness Overview
```

Then expected state after Codex:

```text
Highest global master architecture document number: 67
Next expected global document number: 68
Total global master items: 67
Total indexed linked files: 80
```

If Codex creates extra addendum files, the linked-file count changes.

Codex should **not** create extra addendum files for this batch unless explicitly instructed.

Important:

```text
Inventory Addendum A is merged into Document 65.
It should not be created as a separate global document or linked file.
```

Tiny counting goblin contained. For now.

---

## 4. Core Batch Law

```text
Product owns product truth.
Inventory owns stock truth.
Boundary owns the handoff rules.
Codex must not merge these owners.
```

Simple:

```text
Product = what the item is and how it is presented/sold.
Inventory = how many exist, where they are, and what state they are in.
Boundary = how Product and Inventory safely talk.
```

Codex must preserve this law across all index entries, future packets, simulations, and activation packs.

If Codex lets Product own stock counts or Inventory own product descriptions, we put the repo in a corner and make it think about what it did.

---

## 5. Product Engine Summary

Document 64 defines Selene Product Intelligence Engine.

Product owns:

```text
product_id
product name
brand
category
subcategory
attributes
variants
bundles
descriptions
images
videos
labels
SKU / barcode / identifier readiness
supplier product metadata
compliance requirements
product passport fields
POS listing readiness
e-commerce listing readiness
B2B listing readiness
marketplace readiness
product content quality
product media quality
product market positioning
product lifecycle state
```

Product does **not** own:

```text
stock quantity
warehouse location
stock movement
reservations
stock counts
supplier approval
purchase orders
receiving proof
supplier invoice payment
ledger posting
```

Product’s job:

```text
turn a product into a rich, accurate, channel-ready, compliant commercial object
```

Product is where Selene makes the shampoo beautiful, searchable, explainable, and sellable.

Product does not decide how many bottles are in Warehouse B.

---

## 6. Inventory Engine Summary

Document 65 defines Selene Inventory Intelligence Engine.

Inventory owns:

```text
quantity on hand
quantity available
quantity reserved
quantity allocated
quantity in transit
quantity damaged
quantity quarantined
quantity expired
stock location
warehouse / bin / shelf
batch / lot / serial values
expiry dates
stock movement
stock count
stock transfer
stock health
stockout risk
overstock risk
JIT logic
reorder recommendation
safety stock
FEFO / FIFO picking
availability by channel
inventory audit evidence
```

Inventory does **not** own:

```text
product name
product description
product media
product category
supplier qualification
purchase order creation
supplier payment
final pricing authority
ledger posting
tax treatment
```

Inventory’s job:

```text
know the physical and operational stock truth before humans ask
```

Inventory is where Selene says:

> “You will run out in six days, Supplier B takes five days, and cashflow supports ordering 240 units today.”

Not:

> “Quantity: 12.”

That is not intelligence. That is arithmetic wearing a badge.

---

## 7. Boundary Contract Summary

Document 66 defines the boundary.

Product → Inventory handoff:

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

Inventory → Product handoff:

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

Shared governance packets:

```text
ProductInventoryImpactReviewPacket
ProductInventoryMigrationPacket
ProductInventoryConflictPacket
ProductInventoryAuditPacket
```

Important rule:

```text
Handoff packet names are logical future architecture only.
Codex must not create runtime packet structs unless explicitly instructed in a later repo-truth activation phase.
```

No packet goblinry. Not yet.

---

## 8. Exact Source-of-Truth Rules Codex Must Preserve

| Area                                 | Owner                           |
| ------------------------------------ | ------------------------------- |
| Product identity                     | Product                         |
| Product name / description           | Product                         |
| Product images / videos              | Product                         |
| Product variants                     | Product                         |
| Product bundle definition            | Product                         |
| Product category                     | Product                         |
| SKU / barcode / identifier readiness | Product                         |
| Expiry required flag                 | Product                         |
| Batch/serial required flag           | Product                         |
| Actual batch/serial values           | Inventory                       |
| Actual expiry dates                  | Inventory                       |
| Stock quantity                       | Inventory                       |
| Stock location                       | Inventory                       |
| Stock movement                       | Inventory                       |
| Stock reservation                    | Inventory                       |
| Available-to-promise                 | Inventory                       |
| Channel eligibility                  | Product                         |
| Channel availability                 | Inventory                       |
| Product compliance requirement       | Product                         |
| Stock compliance hold                | Inventory enforcement           |
| Product recall trigger               | Product / Compliance / Supplier |
| Recall stock execution               | Inventory                       |
| Product discontinuation decision     | Product / Authority             |
| Remaining stock plan                 | Inventory                       |
| Pricing proposal/presentation        | Product                         |
| Approved price/margin                | Pricing Engine                  |
| Inventory valuation evidence         | Inventory + Accounting          |
| Ledger posting                       | Accounting                      |

Codex must not “simplify” this table.

This table is why the future system does not grow tentacles.

---

## 9. Required File Names

Preferred file names:

```text
docs/SELENE_GLOBAL_DOCUMENT_64_PRODUCT_INTELLIGENCE_ENGINE_MASTER_DESIGN.md

docs/SELENE_GLOBAL_DOCUMENT_65_INVENTORY_INTELLIGENCE_ENGINE_MASTER_DESIGN.md

docs/SELENE_GLOBAL_DOCUMENT_66_PRODUCT_INVENTORY_BOUNDARY_HANDOFF_CONTRACT_MASTER_DESIGN.md

docs/SELENE_GLOBAL_DOCUMENT_67_PRODUCT_INVENTORY_CODEX_READINESS_OVERVIEW.md
```

If repo naming convention differs, Codex should follow existing master-index naming convention while preserving:

```text
global document number
title
engine identity
batch role
```

No creative file naming. We are not naming indie bands.

---

## 10. Master Index Registration Rules

Codex must update:

```text
docs/SELENE_MASTER_ARCHITECTURE_BUILD_SET.md
```

Add these entries after Global Document 63:

```text
Global Document 64 — Selene Product Intelligence Engine

Global Document 65 — Selene Inventory Intelligence Engine

Global Document 66 — Product ↔ Inventory Boundary + Handoff Contract

Global Document 67 — Product + Inventory Codex Readiness Overview
```

Each entry must link to its file.

Each entry must identify status:

```text
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
```

Document 67 must state that this batch is the foundation for the later Supplier / Procurement / Receiving / AP mini-batch.

Codex must not alter existing Finance / Accounting batch mappings.

Codex must not make PH1.X Document 64.

PH1.X remains legacy/review candidate until explicitly handled later.

---

## 11. Expected Count After Index Update

After adding the four files:

```text
Highest global master architecture document number: 67
Next expected global document number: 68
```

Linked files:

```text
Previous linked files: 76
New linked files added: 4
Expected linked files: 80
```

Codex must verify this in the final report.

If the count is not 80, Codex must explain exactly why.

Acceptable reasons:

```text
repo index counts non-canonical review candidates separately
file naming convention changed
additional addendum file was created by explicit instruction
```

Unacceptable reason:

```text
Codex shrugged and math became vibes.
```

---

## 12. Dependencies

This batch depends on:

```text
existing master architecture index
PH1.D / GPT-5.5
PH1.WRITE
PH1.ACCESS / AUTHORITY
PH1.AUDIT
existing Finance / Accounting batch mappings
```

This batch prepares future dependencies for:

```text
Supplier Intelligence
Procurement + Purchase Order
Goods Receiving + Inspection
AP / Creditors
Supplier Payment
Supplier Statement Reconciliation
E-Commerce
B2B
POS
Order Management
Pricing
Marketing
Customer
Logistics
Returns
Accounting
Tax
```

Product + Inventory are foundational.

Do not build supplier/procurement/AP on sand. Sand has enough problems.

---

## 13. Simulation Catalog For This Batch

Document 64 simulations:

```text
add product by voice
add product from photo
add product from supplier invoice
add product from spreadsheet
detect duplicate product
classify product category
create internal SKU
prepare barcode label
prepare GS1 Digital Link candidate
prepare POS listing
prepare e-commerce listing
prepare B2B listing
enhance product photo
create product video draft
suggest market positioning
detect missing compliance data
detect product passport data gap
create inventory template
handoff to Inventory
custom attribute captured
custom attribute approved into schema
supplier-risk hold
compliance hold
publish product after confirmation
```

Document 65 simulations:

```text
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

Document 66 simulations:

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

Document 67 simulations:

```text
Codex inserts Product + Inventory batch
master index updates to 64–67
linked file count increases from 76 to 80
Product/Inventory boundary preserved
no runtime code generated
no packet structs generated
no Finance mappings damaged
no PH1.X global number changed
```

---

## 14. Acceptance Tests

Codex must later prove, at architecture level:

```text
Product does not own quantity.
Inventory does not own product description.
Product creates inventory template.
Inventory accepts only Receiving-accepted stock.
Inventory availability feeds Product/Commerce.
Product compliance hold blocks Inventory sellability.
UOM changes with active stock require impact review.
Duplicate product merge with stock requires reconciliation.
Discontinuation with remaining stock creates stock runout plan.
Stockout/expiry/dead-stock signals feed Product health.
Finance / Accounting mappings remain untouched.
Master index count updates correctly.
```

This is not runtime testing yet.

This is architectural acceptance.

No runtime tests claiming implementation exists. We have not built the engine. We are drawing the machine without pretending it already walks.

---

## 15. Codex Insert Rules

When Codex receives this batch, it must:

```text
read AGENTS.md
confirm repo clean
create four document files
update master index
do docs-only changes
stage intended files only
commit
push
verify clean tree
verify HEAD equals upstream
report counts and links
```

Commit message:

```text
Add Selene product and inventory foundation documents 64 to 67
```

Codex must report:

```text
Document 64 created
Document 65 created
Document 66 created
Document 67 created
master index updated
highest global document now 67
next expected global document 68
linked file count now 80, or explain difference
final git status clean
HEAD equals upstream
```

If Codex cannot verify counts, it must stop and report.

No “probably.” Probably is where bad repos hatch.

---

## 16. What Codex Must Not Do

```text
Do not create runtime code.
Do not create migrations.
Do not create APIs.
Do not create packet structs.
Do not create tests claiming implementation.
Do not edit Product/Inventory content into existing unrelated docs.
Do not alter Finance / Accounting batches.
Do not alter PH1.X legacy/review candidate.
Do not make PH1.X Document 64.
Do not add Supplier/Procurement/AP docs in this batch.
Do not split Inventory Addendum A into a separate file.
Do not merge Product and Inventory.
Do not change global numbering except adding 64–67.
Do not leave files uncommitted.
Do not skip push if AGENTS permits push.
```

If Codex violates any of these, it gets backend seats behind a pillar and no snacks.

---

## 17. Future Package After This Batch

After this batch is inserted, committed, pushed, and reviewed, the next package begins at:

```text
Global Document 68
```

Expected next package:

```text
68 — Selene Supplier Intelligence Engine
69 — Supplier Bank Change + Selene-to-Selene Counterparty Addendum
70 — Selene Procurement + Purchase Order Engine
71 — Selene Goods Receiving + Inspection + Supplier Credit Automation Engine
72 — Receiving Daily Manifest + Credit Note Automation Addendum
73 — Selene AP / Creditors Engine
74 — Selene Supplier Payment + Banking Execution Handoff Engine
75 — Selene Supplier Statement Reconciliation + Creditor Reporting Engine
76 — Supplier / Procurement / Receiving / AP Mini-Batch Overview
```

That supplier chain must not be sent before Product + Inventory batch is safely in place.

Product and Inventory are the floor.

Supplier chain is the plumbing.

Do not install plumbing in the sky. We have standards now. Allegedly.

---

## 18. Human-Like Selene Philosophy For This Batch

Selene must feel like a smart operator, not software.

Product:

> “Show me the product. I’ll prepare the listing.”

Inventory:

> “You will run out in six days. I recommend ordering today.”

Boundary:

> “Changing pack size affects active stock and open POs. I’ll prepare an impact review before applying it.”

The human should feel:

```text
Selene understands the business.
Selene reduces manual setup.
Selene finds missing data.
Selene acts automatically where safe.
Selene asks only meaningful questions.
Selene explains clearly.
Selene does not lose the plot.
```

A standard software system asks humans to manage the system.

Selene manages the business and asks humans for judgment.

That is the difference.

---

## 19. Final Architecture Sentence

Selene Product + Inventory Codex Readiness Overview is the batch-control document that packages Global Documents 64–67 into a clean, docs-only, Codex-ready foundation, ensuring Product owns product identity and commercial presentation, Inventory owns stock truth and operational availability, the Product ↔ Inventory Boundary governs all handoffs and impact reviews, the master index updates from global 63 to 67 without disturbing Finance / Accounting or PH1.X, and Codex inserts, commits, pushes, and verifies the batch without creating runtime implementation or corrupting source-of-truth ownership.

Simple version:

```text
Add Product.
Add Inventory.
Add the boundary.
Add this overview.
Update the master index.
Do not touch runtime.
Do not touch Finance.
Do not touch PH1.X.
Do not merge Product and Inventory.
Commit and push.
Next document becomes 68.
```

That is Global Document 67. It is the batch seatbelt. Because apparently if we don’t buckle Codex in, it may try to drive Product, Inventory, Supplier, AP, and a forklift all at once, which is how architecture ends up in a ditch wearing a barcode scanner.
