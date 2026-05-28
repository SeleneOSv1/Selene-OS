# Selene Company Size Classification + Size Packs Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / SIZE CLASSIFICATION + SIZE PACKS

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define how Selene classifies company size and applies size-based onboarding depth across solo/freelancer, small business, mid-size company, large enterprise, and multi-entity group while preserving full Selene capability for every company.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future size classification and size-pack onboarding behavior only. It does not implement PH1.ONB.SIZE, runtime classifiers, packets, schemas, APIs, modules, tests, migrations, activation logic, or any runtime company setup state.

Current repo truth does not prove complete runtime company size classification or size-pack activation. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

## 1. Purpose

This document defines how Selene classifies company size and adjusts onboarding depth.

Core law:

```text
Size changes onboarding depth, not Selene capability.
```

Small companies do not get toy Selene.

They get simpler onboarding.

Large companies get deeper governance, finance, access, reporting, and audit setup.

This design defines size packs for:

```text
solo/freelancer
small business
mid-size company
large enterprise
multi-entity group
```

It also requires reclassification after live operating evidence appears.

## 2. Executive Target

Selene must avoid two failures:

```text
forcing enterprise setup on a solo user
giving weak setup to a small but complex business
```

A solo user may need B2B and e-commerce.

A small company may have serious stock and supplier complexity.

A mid-size company may have deep governance risk.

A large enterprise may appear simple in the first conversation while still requiring formal access, finance, approval, and audit depth.

## 3. Master Law

```text
Company size is provisional.
Selene classifies during onboarding.
Selene rechecks after real operations.
Selene adapts setup depth as evidence appears.
Small companies get simpler setup, not weaker Selene.
```

Review points:

```text
30 days
90 days
180 days
annual review
major growth event
new branch/entity
new payroll complexity
new B2B/trade activity
new board/shareholder activity
new inventory/procurement complexity
new access/governance complexity
```

## 4. Owner Split

| Area | Owner |
| --- | --- |
| Size classification | PH1.ONB.SIZE / Company Onboarding |
| Size reclassification | PH1.ONB + Analytics/Evidence |
| Module activation proposal | PH1.ONB with source-engine boundaries |
| Access depth | Access/Governance |
| Finance depth | Finance/Accounting future owners |
| Commerce depth | POS / E-Commerce / B2B commerce channel owners |
| Industry overlay | PH1.ONB.INDUSTRY |
| Business model routing | PH1.ONB.MODEL |
| Audit | Audit |

## 5. Classification Inputs

Selene should classify size using more than employee count.

Inputs:

```text
employee count
contractor count
revenue range
transaction volume
customer count
supplier count
number of locations
legal entity count
countries/regions
inventory complexity
product/service complexity
commerce channels
B2B/trade activity
payroll complexity
governance complexity
board/shareholder presence
approval matrix complexity
bank accounts
tax jurisdictions
reporting needs
```

Selene may collect these from:

```text
human answers
uploaded company documents
accounting exports
POS exports
product lists
supplier lists
customer lists
websites
public profiles where permitted
integrations where authorized
```

## 6. Size Pack A — Solo / Freelancer

### Setup Depth

Fast, human, and low friction.

### Likely Modules

```text
Company/Profile
Banking/payment
Simple invoicing
Payment links
Tax setup
Small product/service catalog
Basic customer records
Basic reporting
Booking/quote page if service-based
E-commerce/service profile
Accounting minimal active
```

### Simplified Questions

```text
What do you offer?
Do clients book you or request a quote?
How do you get paid?
Do you need invoices?
Do you sell to businesses?
```

### Deferred Modules

```text
Payroll
Staff access
Advanced inventory
Board/shareholders
Complex AP/AR
Fleet
Insurance unless detected
```

### Governance Depth

Owner/admin only unless incorporated, multiple owners, directors, or shareholder rules exist.

### Finance/Accounting Depth

Minimal:

```text
income
expenses
tax category
invoice/payment link
basic cashflow
```

### Access Complexity

Simple owner/admin access.

### Team Onboarding

Optional contractor/client links later.

### Reporting Needs

```text
income
expenses
tax summary
client invoices
cash position
```

### Go-Live Requirements

```text
identity confirmed
service/product listed
payment path active or warning
basic invoice/receipt path
audit enabled
```

## 7. Size Pack B — Small Business

### Setup Depth

Simple but serious.

Small companies receive full Selene capability with a lighter setup path.

### Likely Modules

```text
POS/e-commerce
basic inventory
AP/AR
payroll if employees
supplier setup
customer setup
basic budgets
bank reconciliation
tax
cashflow
product/service catalog
Access baseline
```

### Simplified Questions

```text
Do you have staff?
Do you sell products, services, or both?
Do customers buy in-store, online, or on account?
Do you buy from suppliers?
Do you keep stock?
```

### Deferred Modules

```text
advanced governance
board/shareholder engine unless detected
advanced asset/fleet unless detected
deep analytics
multi-entity
```

### Governance Depth

Owner/admin plus simple approval limits.

### Finance/Accounting Depth

Minimal-to-structured:

```text
bank connection
basic AP/AR
tax setup
basic budgets
basic cashflow
```

### Access Complexity

Owner, staff, cashier, manager if needed.

### Team Onboarding

Selene invites team members by secure links.

### Reporting Needs

```text
daily sales
cashflow
stock movement
supplier bills
customer invoices
payroll summary
tax summary
```

### Go-Live Requirements

```text
identity
payment
commerce/POS path
basic tax
first product/service
admin access
audit
```

## 8. Size Pack C — Mid-Size Company

### Setup Depth

Structured, progressive, and department-aware.

### Likely Modules

```text
departments
managers
payroll
rosters
AP approvals
AR collections
inventory
procurement
customer credit
reporting
budgets
access templates
supplier management
cashflow
commerce gateway
B2B if relevant
```

### Simplified Questions

```text
Do you have departments?
Who approves purchases?
Do customers pay later?
Do you manage stock across locations?
Do suppliers deliver goods to you?
Do managers need different permissions?
```

### Deferred Modules

```text
board/shareholder if not present
enterprise consolidation
advanced tax packs
multi-country unless detected
```

### Governance Depth

Departmental authority, approval routing, fallback approvers.

### Finance/Accounting Depth

Structured:

```text
AP/AR
cashflow
budgets
banking status
tax
supplier/customer payment terms
```

### Access Complexity

Role templates, manager authority, user invite flows.

### Team Onboarding

Employee onboarding links, role-bound access setup, department assignment.

### Reporting Needs

```text
department budgets
aged debtors
aged creditors
inventory health
cash forecast
payroll reports
sales reports
```

### Go-Live Requirements

```text
identity
access hierarchy
payment path
commerce path
finance minimum
product/service setup
supplier/customer minimum
approval routing
audit
```

## 9. Size Pack D — Large Enterprise

### Setup Depth

Formal, multi-session, and governance-first.

### Likely Modules

```text
board
shareholders
multi-country
multi-currency
consolidated reporting
detailed access
approval matrices
tax packs
audit
multiple bank accounts
intercompany
legal entities
payroll regions
finance controls
commerce/customer portals
```

### Simplified Questions

```text
Do you have more than one legal entity?
Do you have board or shareholder approval rules?
Do different regions approve spending differently?
Do you operate in multiple countries?
Do you need consolidated reporting?
```

### Deferred Modules

Only non-critical optional capabilities.

### Governance Depth

Required governance source pack.

### Finance/Accounting Depth

Structured and governed:

```text
entity setup
banking
tax jurisdictions
budget controls
AP/AR authority
reporting packs
cashflow
audit
```

### Access Complexity

Role templates, per-user access, board/shareholder roles, region/entity scope.

### Team Onboarding

Mass invite links, SSO/identity provider if applicable, staged department rollout.

### Reporting Needs

```text
board packs
financial reporting
region/entity reporting
audit logs
budgets
cashflow
risk
compliance
```

### Go-Live Requirements

```text
governance pack
authority matrix
finance minimum
audit active
security/identity proof
critical approvers
payment controls
simulation pass
```

## 10. Size Pack E — Multi-Entity Group

### Setup Depth

Entity-by-entity and group-aware.

### Likely Modules

```text
legal entity structure
intercompany
consolidation
multi-currency
multi-tax
entity-specific banking
entity-specific access
group reporting
board/shareholder routing
shared services
```

### Simplified Questions

```text
How many legal entities are there?
Do entities share customers or suppliers?
Do they trade with each other?
Do they share employees or bank accounts?
Do you need group reports?
```

### Governance Depth

Group authority, entity authority, board/shareholder rules, intercompany approvals, and reporting scope.

### Finance/Accounting Depth

Entity-led setup with future multi-entity/consolidation handoff.

### Access Complexity

User access must be scoped by tenant, company, legal entity, country, function, and authority role.

### Go-Live Requirements

```text
at least one operating entity ready
group structure captured
entity scope enforced
intercompany deferred or configured
finance/governance authority mapped
audit
```

## 11. Size Reclassification

Company size is provisional until real operating evidence confirms or changes it.

Selene should reclassify when evidence shows:

```text
new employees or contractors
new locations
new legal entities
higher transaction volume
more complex supplier/customer terms
B2B/trade credit activation
inventory/procurement complexity
board/shareholder governance
multi-country or multi-currency operations
larger reporting and approval needs
```

Selene says:

```text
Your setup is becoming more complex because you now have trade customers, inventory, and department approvals. I recommend moving from Small Business setup depth to Mid-Size Company setup depth.
```

## 12. Required Logical Packets

Future mapping targets:

```text
CompanySizeClassificationPacket
CompanySizeConfidencePacket
SizePackActivationPacket
SizeReclassificationPacket
SizeReviewSchedulePacket
SizePackDeferredSetupPacket
SizePackGoLiveRequirementPacket
SizeComplexitySignalPacket
```

These are logical design packets only. They are not claimed runtime packets.

## 13. What Must Not Happen

```text
no classification only by employee count
no weak Selene for small companies
no enterprise setup forced on solo users
no complexity signals ignored
no forgotten reclassification
no modules activated without owner truth
no Access shortcuts from size pack alone
no governance assumptions from company size alone
no reporting depth reduced below actual company risk
no implementation claims
```

## 14. Future Simulation Targets

```text
SIM_COMPANY_SIZE_001_classify_solo_user
SIM_COMPANY_SIZE_002_classify_small_business
SIM_COMPANY_SIZE_003_classify_mid_size_company
SIM_COMPANY_SIZE_004_classify_enterprise
SIM_COMPANY_SIZE_005_classify_multi_entity_group
SIM_COMPANY_SIZE_006_small_company_later_becomes_mid_size
SIM_COMPANY_SIZE_007_solo_company_activates_b2b
SIM_COMPANY_SIZE_008_large_enterprise_requires_governance_pack
SIM_COMPANY_SIZE_009_size_reclassification_after_90_days
```

## 15. Final Architecture Sentence

Selene Company Size Classification + Size Packs adjusts onboarding depth, tone, and sequencing while preserving full Selene capability for every company, then reclassifies after live operations reveal the real complexity of the business.
