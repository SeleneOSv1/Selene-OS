# Selene Industry Detection + Industry Starter Packs Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / INDUSTRY DETECTION + INDUSTRY STARTER PACKS

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define how Selene detects company industry and applies industry starter packs as overlays that shape onboarding questions, module activation proposals, go-live checklists, and cross-engine handoffs without merging source owners or claiming runtime implementation.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future Industry Detection and Starter Pack onboarding behavior only. It does not implement PH1.ONB.INDUSTRY, industry classifiers, modules, packets, schemas, APIs, clients, adapters, migrations, tests, or activation code.

Current repo truth does not prove complete runtime Industry Detection or industry starter-pack activation. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

## 1. Purpose

This document defines how Selene detects industry and applies industry starter packs.

Industry packs are overlays, not isolated templates.

A company may have multiple overlays.

Example:

```text
Bakery = Restaurant/Food + Retail + E-Commerce + possibly B2B Catering
Manufacturer with showroom = Manufacturing + Retail + B2B
Salon selling products = Salon/Beauty + Retail
```

This design preserves starter packs for:

```text
retail
restaurant / cafe / food
manufacturing
salon / beauty
supplier / vendor
professional / freelancer services
```

## 2. Executive Target

Selene must understand what kind of business this is and ask industry-relevant questions without dragging the human through irrelevant setup.

A restaurant should not be asked about machine work-in-progress unless it manufactures food at production scale.

A manufacturer should not be asked about table bookings unless it also runs a hospitality channel.

## 3. Master Law

```text
Industry detection shapes questions.
Industry overlays activate likely engines.
Industry does not replace company size or business model.
Multiple industry overlays may apply.
Industry packs are overlays, not isolated templates.
Canonical engines own their own truth.
```

## 4. Owner Split

| Area | Owner |
| --- | --- |
| Industry detection | PH1.ONB.INDUSTRY / Company Onboarding |
| Product/service setup | PH1.PRODUCT |
| Stock/materials | PH1.INVENTORY |
| POS / E-Commerce / B2B | Distinct commerce channel owners or sub-engines |
| Customer relationship | PH1.CUSTOMER |
| Supplier identity | PH1.SUPPLIER |
| Procurement | PH1.PROCUREMENT |
| Receiving / inspection | PH1.PROC.RECEIVE |
| Logistics / delivery | PH1.LOGISTICS |
| Staffing | HR / Payroll / Roster |
| Finance/accounting | Future Finance/Accounting owners |
| AP / AR | Future AP / AR owners |
| Tax | Tax / Compliance owner |
| Banking | Banking / Payment owner |
| Reporting | Future Reporting owner |
| Compliance/restricted fields | Compliance / Tax / Legal owners |
| Audit | Audit |

## 5. Commerce Naming Reconciliation

Commerce channel naming is pending Grand Architecture Reconciliation. The current design recognizes POS, E-Commerce, and B2B as distinct commerce channel owners or sub-engines. Exact canonical repo names must later be reconciled with PH1.POS.COMMERCE and any future PH1.ECOMMERCE / PH1.B2B engine names. Codex must not create duplicate commerce brains from this document alone.

Do not silently create three conflicting commerce engines.

Do not merge all commerce into POS.

Do not let POS own Product or Inventory truth.

## 6. Industry Pack A — Retail

### What Selene Asks

```text
What products do you sell?
Do customers buy in-store, online, or both?
Do products have sizes, colours, or variants?
Do you use barcodes?
Do you offer returns or exchanges?
Do businesses buy from you in bulk?
```

### Likely Modules

```text
PH1.PRODUCT
PH1.INVENTORY
PH1.POS.COMMERCE
E-Commerce channel
B2B channel if trade/bulk exists
PH1.CUSTOMER
Returns/Refunds
Supplier
AP/AR
Tax
Accounting
Banking/Payments
Reporting
```

### Product/Service Setup

Product identity, variants, photos, pricing, descriptions, labels, channel visibility.

### POS/E-Commerce/B2B Relevance

Very high.

### Inventory Relevance

High if physical stock exists.

### Staffing Relevance

Cashiers, stock staff, managers.

### Finance/Accounting Relevance

Sales, tax, COGS, returns, supplier bills, cash/card reconciliation.

### Compliance/Restricted Fields

Age-restricted products, regulated goods, warranty, consumer law returns.

### Go-Live Checklist

```text
first product added
payment tested
POS path tested if in-store
online store ready or opted out
tax/receipt rules set
returns policy set
inventory minimal active
```

## 7. Industry Pack B — Restaurant / Food / Cafe

### What Selene Asks

```text
What food or drinks do you sell?
Do customers dine in, take away, order online, or book?
Do you have a menu?
Do you track ingredients?
Do items expire?
Do you need staff rosters?
Do you deliver?
```

### Likely Modules

```text
Menu/Product Engine
POS
E-Commerce/Menu Ordering
Inventory
Raw Materials
Roster
Payroll
Customer
Delivery/Logistics
Supplier
Tax
Accounting
Banking
Reporting
```

### Product/Service Setup

Menu categories, dishes, prices, photos, modifiers, allergens, service times.

### Inventory Relevance

High for ingredients, perishables, recipe deduction, FEFO, and JIT prep.

### Staffing Relevance

Kitchen, front-of-house, shifts, rosters.

### Finance/Accounting Relevance

Sales, food cost, waste, payroll, supplier AP, tax.

### Compliance/Restricted Fields

Food safety, allergens, alcohol, licences, expiry.

### Go-Live Checklist

```text
menu captured
payment tested
ordering/POS path active
service hours set
basic tax active
delivery/bookings configured if used
ingredient tracking deferred or minimal active
```

## 8. Industry Pack C — Manufacturing

### What Selene Asks

```text
What do you manufacture?
Do you make to stock or make to order?
Do you use raw materials?
Do you track WIP?
Do business customers order from you?
Do you need production stages?
Do you need quality checks?
```

### Likely Modules

```text
PH1.PRODUCT
PH1.INVENTORY
Supplier
Procurement
Receiving
Production
Quality
B2B
Customer Credit
AP/AR
COGS handoff
Logistics
Accounting
Banking
Tax
Reporting
```

### Product/Service Setup

Finished goods, specs, variants, BOM, lead times, trade catalog.

### Inventory Relevance

Very high: raw materials, WIP, finished goods.

### Staffing Relevance

Operators, supervisors, production managers.

### Finance/Accounting Relevance

COGS, inventory valuation, AP, AR, cashflow, production costs.

### Compliance/Restricted Fields

QA, batch/lot, certifications, safety, export/import.

### Go-Live Checklist

```text
product line captured
trade/order path active
supplier basics active
inventory structure minimal active
production flow minimal active
accounting/costing handoff identified
```

## 9. Industry Pack D — Salon / Beauty

### What Selene Asks

```text
What services do you offer?
How long does each service take?
Who performs each service?
Do clients book online?
Do you sell products?
Do you take deposits?
Do staff earn commission?
```

### Likely Modules

```text
Service/Product Engine
Booking/E-Commerce
POS
Payment
Customer CRM
Roster
Payroll
Inventory if products sold
Loyalty
Tax
Accounting
Banking
Reporting
```

### Product/Service Setup

Services, durations, prices, staff eligibility, packages, memberships, product upsells.

### Inventory Relevance

Medium to high if products are sold or supplies tracked.

### Staffing Relevance

High for service providers and appointments.

### Finance/Accounting Relevance

Service revenue, product sales, deposits, commissions, payroll.

### Compliance/Restricted Fields

Licences, consent forms, safety, client health notes where relevant.

### Go-Live Checklist

```text
services added
booking calendar active
payment/deposit path tested
staff schedule minimal active
client reminders active
```

## 10. Industry Pack E — Supplier / Vendor

### What Selene Asks

```text
What do you supply?
Do you sell in bulk?
What are your payment terms?
Where do you deliver?
Do you allow returns?
Do you provide quality certificates?
```

### Likely Modules

```text
Supplier Master
Product
B2B
Inventory/Availability
Delivery/Logistics
AP/AR
Payment Terms
Quality/Disputes
Supplier Scorecard
Tax
Accounting
Banking
Reporting
```

### Product/Service Setup

Supplier SKUs, pack sizes, MOQ, trade pricing, delivery zones, certificates.

### POS/E-Commerce/B2B Relevance

B2B very high. E-commerce trade catalog default.

### Inventory Relevance

Depends whether supplier stocks goods, manufactures goods, or drop-ships.

### Staffing Relevance

Sales/admin/logistics contacts.

### Finance/Accounting Relevance

Payment terms, receivables/payables, credit notes, refunds.

### Compliance/Restricted Fields

Certificates, insurance, product safety, trade regulations.

### Go-Live Checklist

```text
supplier identity confirmed
products/services listed
payment terms confirmed
delivery terms confirmed
B2B catalog ready
dispute/return path set or deferred
```

## 11. Industry Pack F — Professional / Freelancer Services

### What Selene Asks

```text
What services do you offer?
Do clients book time or request quotes?
Do you charge hourly, fixed fee, retainer, or package?
Do you invoice later?
Do clients upload documents?
Do you work with individuals, businesses, or both?
```

### Likely Modules

```text
Service Catalog
Booking/Quote Portal
Payment Links
Customer/Client Engine
AR/Invoicing
Tax
Reporting
Document Vault
B2B if business clients
Accounting
Banking
```

### Product/Service Setup

Services, fees, durations, deliverables, retainers, quote templates.

### POS/E-Commerce/B2B Relevance

E-commerce as service/booking/quote gateway. B2B if business clients.

### Inventory Relevance

Usually low unless physical deliverables exist.

### Staffing Relevance

Depends on team size.

### Finance/Accounting Relevance

Invoicing, deposits, retainers, tax, basic reporting.

### Compliance/Restricted Fields

Professional licences, client confidentiality, regulated advice boundaries.

### Go-Live Checklist

```text
service catalog ready
booking/quote path active
payment link tested
client contact path active
invoice/receipt setup active
```

## 12. Multi-Industry Overlay Logic

Selene must allow multiple industry overlays to coexist.

Examples:

```text
Restaurant + Retail = menu + packaged products + POS + inventory
Manufacturing + B2B + Retail = BOM/WIP + trade orders + showroom checkout
Salon + Retail = booking/services + product sales + staff commissions
Professional services + B2B = quotes, retainers, AR, business clients
```

If overlays conflict, Selene must ask the source owner or the human to confirm the intended operating model before activation.

## 13. Required Logical Packets

Future mapping targets:

```text
IndustryDetectionPacket
IndustryConfidencePacket
IndustryOverlayActivationPacket
IndustryQuestionSetPacket
IndustryComplianceFlagPacket
IndustryGoLiveChecklistPacket
MultiIndustryOverlayPacket
IndustryOwnerHandoffPacket
```

These are logical design packets only. They are not claimed runtime packets.

## 14. What Must Not Happen

```text
no industry packs treated as standalone onboarding scripts
no irrelevant industry questions forced on users
no Product and Inventory merge
no POS owning product or stock truth
no Accounting owning physical stock truth
no Banking owning AP/AR/Payroll truth
no mixed-industry companies ignored
no compliance rules activated without source/owner truth
no e-commerce/B2B/POS naming conflict silently introduced
no implementation claims
```

## 15. Future Simulation Targets

```text
SIM_COMPANY_INDUSTRY_001_detect_retail
SIM_COMPANY_INDUSTRY_002_detect_restaurant_food
SIM_COMPANY_INDUSTRY_003_detect_manufacturing
SIM_COMPANY_INDUSTRY_004_detect_salon_beauty
SIM_COMPANY_INDUSTRY_005_detect_supplier_vendor
SIM_COMPANY_INDUSTRY_006_detect_professional_service
SIM_COMPANY_INDUSTRY_007_apply_multiple_overlays
SIM_COMPANY_INDUSTRY_008_restaurant_activates_perishable_jit_prep
SIM_COMPANY_INDUSTRY_009_manufacturing_activates_procurement_receiving_cogs
SIM_COMPANY_INDUSTRY_010_retail_activates_pos_product_inventory_ecommerce
```

## 16. Final Architecture Sentence

Selene Industry Detection + Industry Starter Packs uses industry overlays to ask relevant human questions, activate the right starter modules, and avoid forcing every business through the same setup path, while preserving owner boundaries and allowing multiple industry identities to coexist.
