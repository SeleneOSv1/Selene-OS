# Selene Universal Company Onboarding Spine Master Design

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / UNIVERSAL COMPANY SETUP SPINE

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define Selene's universal company onboarding journey: company identity, legal/trading name confirmation, jurisdiction setup, evidence intake, size classification, industry detection, business-model routing, module proposal, human confirmation, progress checkpointing, deferred setup, technical connection help, go-live readiness, and cross-engine setup initiation.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future onboarding architecture only. It does not implement PH1.ONB, company setup runtime, Access, Finance, Accounting, Banking, Tax, Commerce, Product, Inventory, Customer, Supplier, Payroll, HR, Board, Shareholder, Reporting, packets, schemas, APIs, clients, adapters, migrations, tests, or activation code.

Current repo truth does not prove complete runtime Company Onboarding, company setup routing, onboarding module activation, or cross-engine setup ownership. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

Future implementation requires explicit build instruction, approved file scope, repo-truth activation, tests, backend evidence, Access/Governance proof, PH1.WRITE proof, audit proof, and JD live acceptance where visible.

## 1. Purpose

The Selene Universal Company Onboarding Spine is the main company setup journey.

It is not a static form.

It is not a generic setup wizard.

It is Selene learning the business, reducing human work, identifying which engines are needed, configuring safe first operations, and guiding the company toward go-live.

Selene should behave like:

```text
technical concierge
business setup expert
operations partner
finance-aware assistant
governance-aware assistant
human-friendly guide
```

Selene should ask normal human questions like "What do you sell?" and internally map the answers to engines such as Accounting, POS, B2B, Cashflow, Product, Inventory, and Governance.

## 2. Executive Target

The target is to let any company begin using Selene without knowing what Selene modules are called.

The user should not need to know:

```text
AP
AR
GL
COGS
POS
B2B
board quorum
access templates
cashflow engine
inventory COGS handoff
```

Selene asks:

```text
What is your business called?
What country is it registered in?
What do you sell?
Do customers pay before or after delivery?
Do you keep stock?
Do you have employees?
Do you want customers to buy, book, order, or pay online?
Do you have board or shareholder approval rules?
```

Then Selene maps answers internally.

Tiny miracle: the human speaks human; Selene handles the machinery.

## 3. Master Law

```text
Selene does the work.
Humans confirm important decisions.
Canonical engines own truth.
Access grants authority.
Audit records proof.
PH1.D/GPT-5.5 proposes and explains.
PH1.N extracts meaning.
PH1.X validates route and risk.
PH1.WRITE owns final human wording.
```

Selene may infer and propose from documents, photos, websites, integrations, and files, but humans must confirm legal, financial, authority, payment, tax, and governance decisions.

The onboarding spine preserves:

```text
probabilistic communication with deterministic execution
checkpointing
safe resumption
minimum human input
full capability for every company
provisional size classification
commerce gateway defaults
Product and Inventory separation
distinct POS / E-Commerce / B2B commerce channel ownership
governance routing
central Access control
audit proof
deferred setup tracking
go-live readiness evidence
```

## 4. Owner Split

| Area | Owner |
| --- | --- |
| Company onboarding session | PH1.ONB / Company Onboarding |
| Human wording | PH1.WRITE |
| Messy language extraction | PH1.N |
| Risk/route classification | PH1.X |
| GPT-5.5 proposal/drafting | PH1.D |
| Memory/continuity | PH1.M |
| Invite links | PH1.LINK |
| Roles/permissions/authority | Access/Governance |
| Proof trail | Audit |
| Company legal facts | Company Identity / Legal Entity owner |
| Finance setup | Finance / Accounting future owners |
| Payment authority | Banking/Payment + Access/Governance |
| Products | PH1.PRODUCT |
| Stock truth | PH1.INVENTORY |
| Employees | HR / Payroll / Position / Access |
| Customers | PH1.CUSTOMER / PH1.CREDIT / AR |
| Suppliers | PH1.SUPPLIER / Procurement / Receiving / AP |
| Board/shareholders | PH1.BOARD / PH1.SHAREHOLDER |
| Reporting | Future Reporting / Analytics owner |
| Operational engines | Canonical operational owners |

Onboarding prepares.

Access grants.

Source engines own truth.

Onboarding must not become the owner of every downstream business domain.

## 5. Universal Company Profile

Selene creates a universal company profile with:

```text
company legal name
trading name
country / jurisdiction
registration number
tax registration
legal entity type
business address
operating address
contact channels
currency
language
time zone
fiscal year
accounting period
business size estimate
industry estimate
business model tags
branches / locations
owners / directors / admins
employees / contractors
customers
suppliers
products / services
commerce channels
payment channels
banking setup status
governance complexity
module activation map
deferred setup map
go-live readiness status
```

## 6. Legal Identity Capture

Selene should reduce typing by reading:

```text
business registration document
licence
tax registration
address document
utility bill
lease
company website
public business profile where permitted
uploaded certificate
photo of business licence
```

Selene says:

```text
Show me your business registration or licence. I'll read the details and ask you to confirm.
```

Selene extracts:

```text
legal name
trading name
registration number
tax ID
registered address
jurisdiction
licence expiry
owner/director names where visible
```

Then:

```text
I found the legal name as GreenLeaf Trading Pty Ltd. Please confirm the spelling before I save it.
```

Spelling confirmation is required because one wrong letter in a legal name can create preventable administrative and legal problems.

## 7. Company Onboarding Lifecycle

```text
User wants to onboard company
-> Selene asks simple human questions
-> Selene reads uploaded/public/provided evidence where permitted
-> Selene confirms legal name and spelling
-> Selene classifies company size
-> Selene detects industry
-> Selene detects business model
-> Selene checks governance complexity
-> Selene proposes required modules
-> Selene asks for confirmations
-> Selene sets up minimum viable operations
-> Selene defers non-blocking modules
-> Selene creates go-live checklist
-> Selene invites required users
-> Selene connects finance/banking/tax/accounting where needed
-> Selene activates customer/supplier/product/commerce paths where needed
```

## 8. Module Activation Map

Each engine may be:

```text
NotNeeded
Available
MinimalActive
FullyConfigured
Deferred
BlockedPendingData
BlockedPendingAuthority
PendingGovernance
```

Example: a small bakery:

```text
PH1.PRODUCT: MinimalActive
PH1.INVENTORY: MinimalActive
PH1.POS.COMMERCE: MinimalActive
E-Commerce channel: MinimalActive
B2B channel: Available
Payroll: MinimalActive if staff exist
Governance: MinimalActive
Board: NotNeeded unless directors/board exist
Shareholder: NotNeeded unless shareholders exist
```

Example: a manufacturing company:

```text
PH1.PRODUCT: FullyConfigured target
PH1.INVENTORY: FullyConfigured target
PH1.SUPPLIER: MinimalActive -> FullyConfigured
PH1.PROCUREMENT: MinimalActive
PH1.PROC.RECEIVE: MinimalActive
Production/COGS: Required future activation
B2B: MinimalActive / FullyConfigured
Accounting: Required future activation
```

## 9. Commerce Naming Reconciliation

Commerce channel naming is pending Grand Architecture Reconciliation. The current design recognizes POS, E-Commerce, and B2B as distinct commerce channel owners or sub-engines. Exact canonical repo names must later be reconciled with PH1.POS.COMMERCE and any future PH1.ECOMMERCE / PH1.B2B engine names. Codex must not create duplicate commerce brains from this document alone.

Do not silently create three conflicting commerce engines.

Do not merge all commerce into POS.

Do not let POS own Product or Inventory truth.

## 10. Go-Live Readiness

Selene classifies readiness:

```text
NotReady
MinimalSetupReady
GoLiveReady
Live
LiveWithDeferredSetup
Blocked
```

Go-live cannot happen without:

```text
confirmed company identity
confirmed admin/owner identity
jurisdiction/country
minimum commerce path or explicit opt-out
minimum payment path or explicit deferred warning
minimum tax/accounting setup status
access/authority baseline
audit enabled
module activation map
deferred setup list
```

If missing:

```text
You can continue later, but I cannot mark the company ready yet because payments are not tested and tax jurisdiction is not confirmed.
```

## 11. Deferred Setup

Selene should not force all setup at once.

She separates:

```text
Required now
Useful soon
Ask later when needed
```

Example:

A small retailer can go live with:

```text
legal identity
admin
first products
payment test
online store/POS path
basic tax
```

Later Selene collects:

```text
full supplier list
advanced inventory thresholds
staff rosters
insurance
assets
trade customers
B2B partner settings
```

Selene says:

```text
We do not need to finish every detail today. I'll get the business live safely first, then help complete suppliers, products, staff, and assets as they become relevant.
```

## 12. Progress Checkpointing And Resume

Every confirmed field is saved.

Selene must never ask again for a valid completed field.

Checkpoint stores:

```text
current step
completed steps
confirmed fields
unconfirmed extracted fields
uploaded files
integration status
deferred modules
missing information
go-live blockers
last safe resume point
```

If internet drops:

```text
We were connecting your payment provider. I saved everything up to the verification step. Shall we continue from there?
```

## 13. Technical Concierge

Selene should handle setup guidance for:

```text
bank feed
payment gateway
POS device
online store
domain/storefront
shipping provider
email/SMS notifications
tax integration
accounting import
product spreadsheet import
staff invite links
supplier/customer import
```

Human experience:

```text
I'll open the secure bank connection screen. I will not see your password. After you approve it, I'll test whether the connection works.
```

The human authorizes.

Selene configures, tests, and reports.

## 14. Status Assistant

Selene should always be able to tell the user:

```text
what is done
what is missing
what is blocked
what is optional
what can wait
what needs human confirmation
what will happen next
```

Example:

```text
Company identity is confirmed, product setup is started, and payments are still pending bank approval. You can keep building the product list while the bank connection is waiting.
```

## 15. Troubleshooting Assistant

Selene helps recover onboarding problems without making the user understand the underlying stack.

Troubleshooting examples:

```text
bank connection failed
document upload unreadable
tax registration missing
company name mismatch
duplicate company candidate
commerce channel not ready
payment test failed
team invite expired
governance document conflict
user lacks setup authority
```

Selene says:

```text
The payment connection did not finish. I saved your setup and can retry the secure connection, or we can continue with products and staff first.
```

## 16. Cross-Engine Setup Links

Company onboarding links to, but does not own, these future or existing owner domains:

```text
finance
accounting
banking
tax
commerce
product
inventory
payroll
HR
access
governance
board
shareholder
customer
supplier
reporting
operations
procurement
receiving
logistics
assets
fleet
insurance
employee wellbeing
```

Onboarding activates or prepares modules but does not own their runtime truth.

## 17. Example Flow

User:

```text
I want to set up my business in Selene.
```

Selene:

```text
Of course. I'll help you set it up step by step. First, what's the legal name of the business, and what country is it registered in?
```

User:

```text
Green Leaf.
```

Selene:

```text
Is the full legal name Green Leaf Pty Ltd, or is Green Leaf only the trading name?
```

User uploads registration.

Selene:

```text
I found the legal name as Green Leaf Trading Pty Ltd and the trading name as Green Leaf. Is that correct?
```

Then Selene continues:

```text
What do you sell, and do customers buy from you in-store, online, or on account?
```

## 18. Required Logical Packets

Future mapping targets:

```text
CompanyProfilePacket
CompanyIdentityEvidencePacket
LegalNameConfirmationPacket
JurisdictionPacket
OwnerAdminSetupPacket
BusinessClassificationPacket
ModuleActivationProposalPacket
DeferredSetupPacket
GoLiveReadinessPacket
OnboardingCheckpointPacket
TechnicalConnectionPacket
HumanConfirmationPacket
AuditEvidencePacket
```

These are logical design packets, not claimed runtime packets. Codex must map later.

## 19. What Must Not Happen

```text
no one-size-fits-all onboarding
no small companies getting weak Selene
no module names dumped on users
no onboarding asking 100 questions up front
no Product and Inventory merge
no POS owning Product or Inventory truth
no Accounting owning physical stock truth
no Banking owning AP/AR/Payroll truth
no Onboarding granting access directly
no GPT-5.5 granting authority
no governance documents becoming legal truth without confirmation
no board/shareholder rules invented
no supplier/customer bank setup without protected validation
no financial modules activated without jurisdiction/company setup
no hidden tax assumptions
no missing audit
no missing progress checkpoint
no missing resume memory
no robotic explanation
no implementation claims
```

## 20. Future Simulation Targets

```text
SIM_COMPANY_ONB_001_start_company_onboarding
SIM_COMPANY_ONB_002_capture_company_identity_from_document
SIM_COMPANY_ONB_003_confirm_legal_name_spelling
SIM_COMPANY_ONB_004_resume_after_interruption
SIM_COMPANY_ONB_005_classify_company_size
SIM_COMPANY_ONB_006_detect_industry
SIM_COMPANY_ONB_007_detect_business_model
SIM_COMPANY_ONB_008_propose_modules
SIM_COMPANY_ONB_009_defer_non_blocking_setup
SIM_COMPANY_ONB_010_technical_payment_connection_success
SIM_COMPANY_ONB_011_technical_payment_connection_failure
SIM_COMPANY_ONB_012_go_live_readiness_pass
SIM_COMPANY_ONB_013_go_live_readiness_blocked
SIM_COMPANY_ONB_014_invite_team_members
```

## 21. Final Architecture Sentence

Selene Universal Company Onboarding Spine is the autonomous front door to Selene's operating system, where Selene reads the business, asks only simple human questions, infers the required setup, confirms important facts, activates the correct engines, defers non-blocking work, checkpoints progress, resumes safely, and guides the company to go-live without forcing humans to understand the machinery underneath.
