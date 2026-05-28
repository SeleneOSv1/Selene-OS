# Selene Business Model Activation Router Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / BUSINESS MODEL ACTIVATION ROUTER

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define how Selene detects the company's business model and maps normal human answers to required, optional, deferred, or blocked Selene engines while preserving owner boundaries and avoiding runtime implementation claims.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future business-model routing only. It does not implement PH1.ONB.MODEL, engine activation, packets, schemas, APIs, clients, adapters, migrations, tests, or runtime setup logic.

Current repo truth does not prove complete runtime Business Model Activation Router ownership. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

## 1. Purpose

This document defines how Selene detects how the company makes money and activates the correct engine depth.

Industry says what kind of business it is.

Business model says how it operates.

A restaurant may be dine-in, delivery, catering, subscription meal prep, B2B office lunch provider, or all of them.

## 2. Executive Target

Selene should detect and activate business model paths for:

```text
B2C
B2B
POS
e-commerce
marketplace
subscription
appointment-based
manufacturing
trade customers
wholesale
service business
multi-branch
supplier network
delivery/logistics
rental/property income
fleet
contractor model
```

Selene asks simple human-life questions and internally maps answers to module activation.

## 3. Master Law

```text
Business model determines which engines are NotNeeded, Available, MinimalActive, FullyConfigured, Deferred, BlockedPendingData, or BlockedPendingAuthority.
Selene asks human-life questions.
Selene maps internally.
Selene explains only what the user needs to know.
Onboarding activates or prepares modules but does not own runtime truth.
```

## 4. Module Activation Status

Business model routing may set each engine to:

```text
NotNeeded
Available
MinimalActive
FullyConfigured
Deferred
BlockedPendingData
BlockedPendingAuthority
```

Definitions:

```text
NotNeeded = no current evidence that the engine is needed.
Available = likely useful later, but not required now.
MinimalActive = basic setup can proceed safely.
FullyConfigured = engine needs deeper setup before go-live or serious use.
Deferred = intentionally postponed and tracked.
BlockedPendingData = cannot proceed until required evidence is supplied.
BlockedPendingAuthority = cannot proceed until authority/approval exists.
```

## 5. Owner Split

| Business model | Primary source engines |
| --- | --- |
| B2C | Customer, POS/E-Commerce, Payment, Tax |
| B2B | Customer, Credit, AR, B2B, Logistics |
| POS | POS.COMMERCE, Product, Inventory, Payment |
| E-Commerce | E-Commerce, Product, Customer, Payment, Logistics |
| Marketplace | B2B/E-Commerce, Supplier, Settlement, Audit |
| Subscription | Billing, Customer, AR, Product/Service |
| Appointment | Booking, Roster, Customer, Payment |
| Manufacturing | Product, Inventory, Supplier, Procurement, Receiving, Production |
| Trade customers | Credit, AR, Customer, Collections |
| Wholesale | B2B, Product, Inventory, Logistics |
| Service business | Service/Product, Booking/Quote, Customer, AR/Payment |
| Multi-branch | Locations, Access, Reporting, Inventory, Payroll, Banking |
| Supplier network | Supplier, Procurement, Receiving, AP |
| Delivery/logistics | Logistics, Customer, Inventory, Fleet if vehicles exist |
| Rental/property income | Asset, Contract, AR, Accounting |
| Fleet | Fleet, Asset, Insurance, Expense |
| Contractor model | HR/Contractor, AP, Access, Payroll boundary |

## 6. Commerce Naming Reconciliation

Commerce channel naming is pending Grand Architecture Reconciliation. The current design recognizes POS, E-Commerce, and B2B as distinct commerce channel owners or sub-engines. Exact canonical repo names must later be reconciled with PH1.POS.COMMERCE and any future PH1.ECOMMERCE / PH1.B2B engine names. Codex must not create duplicate commerce brains from this document alone.

Do not silently create three conflicting commerce engines.

Do not merge all commerce into POS.

Do not let POS own Product or Inventory truth.

## 7. Human-Life Questions

Selene asks:

```text
Do customers pay you before or after delivery?
Do customers buy online, in person, or by invoice?
Do you sell to consumers, businesses, or both?
Do you offer credit terms?
Do people book appointments?
Do you manufacture or assemble products?
Do you deliver goods?
Do you own vehicles or equipment?
Do you earn rent or lease income?
Do contractors do work for you?
Do you have more than one branch?
Do you sell through marketplaces?
Do you charge subscriptions or retainers?
```

Selene does not ask users to choose engines by internal names.

## 8. Activation Example — Small Restaurant

Human answers:

```text
Customers dine in and order takeaway.
Staff work shifts.
Ingredients expire.
Payment is immediate.
Delivery optional.
```

Selene activates or prepares:

```text
POS + Menu/Product
Inventory minimal active
Roster
Tax
Banking/Payment
E-Commerce/Menu ordering optional or minimal active
Delivery optional
Accounting minimal active
```

## 9. Activation Example — B2B Supplier

Human answers:

```text
Businesses buy in bulk.
Customers pay on account.
Products ship regionally.
```

Selene activates or prepares:

```text
Product
B2B
Customer Credit
AR
AP
Inventory
Logistics
Banking
Tax
Supplier/Vendor portal
```

## 10. Activation Example — Manufacturing

Human answers:

```text
We make products using raw materials.
Customers place orders.
Suppliers deliver inputs.
```

Selene activates or prepares:

```text
Product
Inventory
Supplier
Procurement
Receiving
Production
COGS handoff
Logistics
AP/AR
Accounting
B2B trade catalog
```

## 11. Activation Example — Professional Service

Human answers:

```text
Clients request quotes and pay by invoice.
Some book calls.
```

Selene activates or prepares:

```text
Service catalog
Booking
Quote/proposal
Invoicing
Payment links
Customer
Tax
Reporting
AR if pay-later exists
```

## 12. Activation Example — Subscription

Human answers:

```text
Customers pay monthly.
Some customers are businesses.
We need recurring invoices and renewals.
```

Selene activates or prepares:

```text
Product/Service catalog
Subscription billing
Customer
AR
Payment provider
Renewal reminders
Tax
Reporting
```

## 13. Activation Example — Rental / Property Income

Human answers:

```text
We earn rent from properties or equipment.
Contracts define payment dates.
Some tenants pay late.
```

Selene activates or prepares:

```text
Asset / Property future owner
Contract evidence
AR
Cashflow
Accounting
Banking
Insurance future owner
Reporting
```

## 14. Activation Example — Fleet / Delivery

Human answers:

```text
We deliver using company vehicles.
Drivers need assignments.
Vehicle costs must be tracked.
```

Selene activates or prepares:

```text
Logistics
Fleet future owner
Asset future owner
Insurance future owner
Payroll/Roster if drivers are employees
Contractor/AP if drivers are contractors
Fuel/card spend handoff
Accounting/Tax handoff
```

## 15. Business Model Re-Evaluation

Business model detection is provisional.

Selene must re-evaluate after:

```text
new commerce channel
new customer payment terms
new marketplace channel
new subscription revenue
new appointment workflow
new trade customer
new branch
new supplier network
new delivery operation
new rental income
new fleet evidence
new contractor model evidence
```

Selene says:

```text
You now have business customers paying on account, so I recommend activating trade customer credit and AR setup.
```

## 16. Required Logical Packets

Future mapping targets:

```text
BusinessModelDetectionPacket
BusinessModelTagPacket
EngineActivationDecisionPacket
CommercialFrontDoorPacket
PaymentBehaviorPacket
CustomerPaymentTimingPacket
DeliveryModelPacket
RevenueModelPacket
ModuleActivationStatusPacket
BusinessModelRecheckPacket
```

These are logical design packets only. They are not claimed runtime packets.

## 17. What Must Not Happen

```text
no users asked to choose engines by name
no financial modules activated without jurisdiction/company setup
no payment activated without authority
no Banking owning AP/AR/Payroll truth
no E-Commerce, B2B, and POS merged into onboarding
no POS owning Product or Inventory truth
no Product and Inventory merge
no business models ignored after go-live
no implementation claims
```

## 18. Future Simulation Targets

```text
SIM_COMPANY_MODEL_001_detect_b2c
SIM_COMPANY_MODEL_002_detect_b2b
SIM_COMPANY_MODEL_003_detect_trade_customer_terms
SIM_COMPANY_MODEL_004_detect_appointment_business
SIM_COMPANY_MODEL_005_detect_manufacturing_flow
SIM_COMPANY_MODEL_006_detect_rental_property_income
SIM_COMPANY_MODEL_007_activate_b2b_from_customer_payment_terms
SIM_COMPANY_MODEL_008_activate_logistics_from_delivery_model
SIM_COMPANY_MODEL_009_activate_fleet_from_company_vehicles
SIM_COMPANY_MODEL_010_activate_ar_from_pay_later_customers
```

## 19. Final Architecture Sentence

Selene Business Model Activation Router turns normal human answers about how the business sells, gets paid, delivers, hires, owns assets, and serves customers into the correct engine activation map, while keeping engine ownership separate and explaining only what the human needs to confirm.
