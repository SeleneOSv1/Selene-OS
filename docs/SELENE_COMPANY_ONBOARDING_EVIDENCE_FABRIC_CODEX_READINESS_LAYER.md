# Selene Company Onboarding Evidence Fabric + Codex Readiness Layer

```text
DOCUMENT TYPE:
MASTER DESIGN / COMPANY ONBOARDING / EVIDENCE FABRIC + CODEX READINESS LAYER

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION
PENDING_CODEX_REPO_MAPPING

PURPOSE:
Define the company onboarding readiness bridge: owner maps, evidence packets, state machines, module activation statuses, missing information tracking, deferred setup, go-live readiness, audit model, simulation targets, acceptance tests, preservation ledger, cross-engine relationship matrices, and future Codex mapping requirements.
```

## 0. Authority And Scope

AGENTS.md controls execution.

This is a docs-only master design readiness layer.

No runtime code was changed.

This document does not authorize implementation.

This document is part of the Company Onboarding Master Design Set. It defines future evidence, readiness, state, simulation, acceptance-test, and Codex mapping targets only. It does not implement packets, schemas, migrations, APIs, clients, adapters, tests, simulations, activation packs, onboarding runtime, or engine activation.

Current repo truth does not prove complete runtime company onboarding evidence fabric, logical packets, state machines, module activation router, or readiness layer. This document is future architecture pending Grand Architecture Reconciliation and Codex repo mapping.

## 1. Purpose

This document is the readiness bridge.

It tells Codex how to convert the onboarding design into evidence, packets, simulations, acceptance tests, and repo mapping later.

It must not implement anything.

It must not claim runtime exists.

This document defines:

```text
owner map
onboarding evidence packets
company profile packet
classification packet
size pack packet
industry overlay packet
business model packet
governance source packet
module activation packet
deferred setup packet
go-live readiness packet
missing information packet
audit model
state machines
simulation targets
acceptance tests
preservation ledger
cross-engine relationship matrix
Codex batch recommendation
```

## 2. Executive Target

Codex should be able to take these designs and later answer:

```text
Which engine owns this?
What evidence proves it?
What packet carries it?
What status is it in?
What simulation proves it?
What is missing?
What must not be implemented yet?
```

The purpose is to prevent future implementation from confusing design intent, runtime truth, owner boundaries, and repo reality.

## 3. Master Law

```text
No implementation from this design alone.
No module activation without owner truth.
No protected action without Access/Authority/Simulation/Audit.
No repo claims without repo mapping.
No legal/finance/payment/tax/governance truth without source owner.
Selene does the work; humans confirm important decisions.
PH1.D/GPT-5.5 proposes and explains only.
PH1.N extracts meaning.
PH1.X validates route and risk.
PH1.WRITE owns final human wording.
Access/Governance owns permissions and authority.
Canonical engines own their truth.
Audit records proof.
```

## 4. Canonical Owner Map

| Responsibility | Canonical Owner |
| --- | --- |
| Onboarding session | PH1.ONB / Company Onboarding |
| Company profile | Company Identity owner |
| Meaning extraction | PH1.N |
| Route/risk | PH1.X |
| Proposal/drafting | PH1.D |
| Human wording | PH1.WRITE |
| Memory/checkpoint | PH1.M / PH1.ONB |
| Invite links | PH1.LINK |
| Access/authority | Access/Governance |
| Audit | Audit |
| Company size | PH1.ONB.SIZE |
| Industry detection | PH1.ONB.INDUSTRY |
| Business model | PH1.ONB.MODEL |
| Governance intake | PH1.ONB.GOVSOURCE |
| Product identity | PH1.PRODUCT |
| Stock truth | PH1.INVENTORY |
| Supplier identity/performance | PH1.SUPPLIER |
| Purchase orders | PH1.PROCUREMENT |
| Goods receipt/inspection | PH1.PROC.RECEIVE |
| Commerce checkout | PH1.POS.COMMERCE |
| Online commerce | E-Commerce future channel owner |
| B2B | B2B future channel owner |
| Customers | PH1.CUSTOMER |
| Trade credit | PH1.CREDIT |
| Collections | PH1.AR.COLLECT |
| Logistics | PH1.LOGISTICS |
| Assets | PH1.ASSET |
| Fleet | PH1.FLEET |
| Insurance | PH1.INSURANCE |
| Board | PH1.BOARD |
| Shareholders | PH1.SHAREHOLDER |
| Finance | Finance/Budget future owner |
| Accounting | Accounting future owner |
| Banking | Banking/Payment future owner |
| Tax | Tax / Compliance future owner |
| Payroll | Payroll owner |
| HR | HR owner |
| Reporting | Reporting / Analytics future owner |

The finance extraction remains a guardrail: current repo truth does not prove complete Finance, Accounting, AP, AR, Banking, Payment, Ledger, Budget, Profitability, or Tax runtime engines. Future design must keep owners separate and must not mistake Access AP for Accounts Payable.

## 5. Commerce Naming Reconciliation

Commerce channel naming is pending Grand Architecture Reconciliation. The current design recognizes POS, E-Commerce, and B2B as distinct commerce channel owners or sub-engines. Exact canonical repo names must later be reconciled with PH1.POS.COMMERCE and any future PH1.ECOMMERCE / PH1.B2B engine names. Codex must not create duplicate commerce brains from this document alone.

Do not silently create three conflicting commerce engines.

Do not merge all commerce into POS.

Do not let POS own Product or Inventory truth.

## 6. Required Logical Packets

Future mapping targets:

```text
CompanyOnboardingSessionPacket
CompanyProfilePacket
CompanyIdentityEvidencePacket
LegalNameConfirmationPacket
JurisdictionPacket
ClassificationPacket
SizePackPacket
IndustryOverlayPacket
BusinessModelPacket
GovernanceSourcePacket
ModuleActivationPacket
DeferredSetupPacket
GoLiveReadinessPacket
MissingInformationPacket
TechnicalConnectionPacket
HumanConfirmationPacket
AccessPreparationPacket
TeamInvitePacket
CommerceReadinessPacket
FinanceReadinessPacket
AuditEvidencePacket
PreservationLedgerPacket
```

These are logical packets.

They are not runtime claims.

Codex must later map them to repo truth or create an approved activation pack.

## 7. Company Profile Packet

Future logical packet:

```text
CompanyProfilePacket:
  company_profile_id
  tenant_id
  company_id
  legal_entity_id
  legal_name
  trading_name
  country
  region
  jurisdiction
  registration_number_ref
  tax_registration_ref
  legal_entity_type
  business_address_ref
  operating_address_refs
  contact_channel_refs
  currency
  language
  time_zone
  fiscal_year
  accounting_period
  size_classification_ref
  industry_overlay_refs
  business_model_refs
  governance_complexity_ref
  module_activation_refs
  deferred_setup_refs
  go_live_readiness_ref
  audit_ref
```

## 8. Classification Packet

Future logical packet:

```text
ClassificationPacket:
  classification_id
  company_id
  size_classification_ref
  industry_detection_refs
  business_model_refs
  confidence_score
  source_evidence_refs
  human_confirmation_ref
  recheck_schedule_ref
  audit_ref
```

## 9. Size Pack Packet

Future logical packet:

```text
SizePackPacket:
  size_pack_id
  company_id
  provisional_size
  setup_depth
  governance_depth
  access_complexity
  finance_depth
  reporting_depth
  team_onboarding_depth
  go_live_requirements
  reclassification_triggers
  audit_ref
```

## 10. Industry Overlay Packet

Future logical packet:

```text
IndustryOverlayPacket:
  industry_overlay_id
  company_id
  industry_type
  overlay_confidence
  question_set_ref
  likely_module_refs
  compliance_flag_refs
  go_live_checklist_ref
  source_evidence_refs
  audit_ref
```

## 11. Business Model Packet

Future logical packet:

```text
BusinessModelPacket:
  business_model_id
  company_id
  business_model_tags
  customer_payment_timing
  commerce_channel_refs
  delivery_model_ref
  revenue_model_ref
  module_activation_refs
  blocked_reason_refs
  deferred_reason_refs
  audit_ref
```

## 12. Governance Source Packet

Future logical packet:

```text
GovernanceSourcePacket:
  governance_source_id
  company_id
  legal_entity_id
  source_document_refs
  extracted_rule_refs
  conflict_refs
  review_status
  authorized_reviewer_refs
  activation_status
  audit_ref
```

## 13. Module Activation Packet

Future logical packet:

```text
ModuleActivationPacket:
  module_activation_id
  company_id
  engine_owner
  activation_reason
  activation_status
  required_data_refs
  missing_information_refs
  authority_requirement_refs
  deferred_until
  source_evidence_refs
  audit_ref
```

## 14. Deferred Setup Packet

Future logical packet:

```text
DeferredSetupPacket:
  deferred_setup_id
  company_id
  engine_owner
  deferred_reason
  risk_if_deferred
  next_review_at
  reminder_ref
  user_visible_status
  audit_ref
```

## 15. Go-Live Readiness Packet

Future logical packet:

```text
GoLiveReadinessPacket:
  go_live_readiness_id
  company_id
  readiness_status
  identity_ready
  access_ready
  jurisdiction_ready
  commerce_ready
  payment_ready
  finance_ready
  tax_ready
  audit_ready
  blocker_refs
  deferred_setup_refs
  human_confirmation_refs
  audit_ref
```

## 16. Missing Information Packet

Future logical packet:

```text
MissingInformationPacket:
  missing_information_id
  company_id
  engine_owner
  missing_field
  why_needed
  blocker_level
  asked_at
  answered_at
  source_evidence_ref
  audit_ref
```

## 17. CompanyOnboardingStatus State Machine

```text
Draft
EvidenceCollecting
Classifying
AwaitingConfirmation
MinimalSetupReady
GovernanceReviewRequired
FinanceSetupPending
CommerceSetupPending
TeamInvitePending
GoLiveReady
Live
DeferredSetupActive
Blocked
Cancelled
Archived
```

## 18. ModuleActivationStatus State Machine

```text
NotNeeded
Available
MinimalActive
FullyConfigured
Deferred
BlockedPendingData
BlockedPendingAuthority
```

## 19. Audit Model

Every onboarding action records:

```text
audit_event_id
actor
session
timestamp
source
input method
document/image/integration reference
extracted field
confidence score
human confirmation
module affected
owner engine
status before
status after
reason code
audit hash/reference
company_id
legal_entity_id
country
```

No hidden setup mutation.

No unaudited owner handoff.

No protected action without Access/Authority/Simulation/Audit.

## 20. Preservation Ledger

The preservation ledger records:

```text
old source material preserved
which wave supplied which concept
which document absorbed it
which modules were deferred
what was intentionally not implemented
what requires repo mapping
what requires Grand Architecture Reconciliation
what requires future activation pack
what requires acceptance simulation
```

This prevents source concepts from being lost during future reconciliation and implementation planning.

## 21. Acceptance Tests

Required future acceptance tests:

```text
small business gets full Selene capability but simplified setup
large enterprise triggers governance pack
retail activates product/POS/inventory/e-commerce options
manufacturing activates inventory/procurement/receiving/COGS
restaurant activates menu/raw material/perishable/JIT prep
B2B activates customer credit/AR/B2B ordering
no module is activated without owner truth
Selene resumes after interruption
Selene asks only missing fields
company name spelling confirmation
governance document upload creates review, not automatic legal truth
board approval rules feed Access/Authority
bank/payment approvals follow governance pack
PH1.D proposals do not mutate setup
PH1.WRITE owns human explanation
deferred modules remain tracked
Codex must not implement from docs alone
```

These are future acceptance tests. This task does not create tests.

## 22. Future Simulation Targets

```text
SIM_COMPANY_ONB_EVIDENCE_001_company_onboarding_start
SIM_COMPANY_ONB_EVIDENCE_002_business_licence_ocr_extraction
SIM_COMPANY_ONB_EVIDENCE_003_company_name_confirmation
SIM_COMPANY_ONB_EVIDENCE_004_size_classification
SIM_COMPANY_ONB_EVIDENCE_005_industry_detection
SIM_COMPANY_ONB_EVIDENCE_006_business_model_detection
SIM_COMPANY_ONB_EVIDENCE_007_governance_source_upload
SIM_COMPANY_ONB_EVIDENCE_008_governance_rule_review
SIM_COMPANY_ONB_EVIDENCE_009_module_activation_proposal
SIM_COMPANY_ONB_EVIDENCE_010_deferred_setup_tracking
SIM_COMPANY_ONB_EVIDENCE_011_technical_payment_setup
SIM_COMPANY_ONB_EVIDENCE_012_bank_connection_setup_blocked_pending_authority
SIM_COMPANY_ONB_EVIDENCE_013_team_invite_flow
SIM_COMPANY_ONB_EVIDENCE_014_retail_setup
SIM_COMPANY_ONB_EVIDENCE_015_restaurant_setup
SIM_COMPANY_ONB_EVIDENCE_016_manufacturing_setup
SIM_COMPANY_ONB_EVIDENCE_017_b2b_setup
SIM_COMPANY_ONB_EVIDENCE_018_resume_after_interruption
SIM_COMPANY_ONB_EVIDENCE_019_go_live_readiness
SIM_COMPANY_ONB_EVIDENCE_020_blocked_go_live
SIM_COMPANY_ONB_EVIDENCE_021_live_with_deferred_setup
```

## 23. Global Must-Not-Happen List

The design set must preserve these prohibitions:

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

Additional readiness prohibitions:

```text
Do not implement from this document alone.
Do not claim database tables exist.
Do not claim packet structs exist.
Do not claim APIs exist.
Do not activate finance/payment without owner truth.
Do not confuse Access AP with Accounts Payable.
Do not let Onboarding grant authority.
Do not let GPT-5.5 mutate setup.
Do not merge Product and Inventory.
Do not let POS own product or stock truth.
Do not let Accounting own physical stock truth.
Do not let Banking own AP/AR/payroll truth.
```

The finance repo-truth extraction warns that Finance/Accounting/AP/AR/Banking/Payment/Budget/Tax runtime owners are not proven and future slices must be derived only after reconciliation and activation packs.

## 24. Cross-Engine Relationship Matrix

| Engine | Onboarding role |
| --- | --- |
| PH1.ONB | Owns onboarding session |
| PH1.D | Proposes, drafts, explains |
| PH1.N | Extracts human meaning |
| PH1.X | Classifies route/risk |
| PH1.WRITE | Owns human wording |
| PH1.M | Memory/checkpoint continuity |
| PH1.REM | Follow-up reminders |
| PH1.BCAST / DELIVERY | Sends messages |
| PH1.LINK | Invite links |
| Access/Governance | Grants permissions/authority |
| Audit | Records proof |
| PH1.PRODUCT | Product/service identity |
| PH1.INVENTORY | Stock truth |
| PH1.SUPPLIER | Supplier identity/performance |
| PH1.PROCUREMENT | Purchase orders |
| PH1.PROC.RECEIVE | Goods receipt/inspection |
| PH1.POS.COMMERCE | In-person checkout / commerce channel under reconciliation |
| E-Commerce future channel owner | Online commercial front door |
| B2B future channel owner | Trade/business ordering |
| PH1.CUSTOMER | Customer relationship |
| PH1.CREDIT | Trade terms/credit |
| PH1.AR.COLLECT | Collections |
| PH1.LOGISTICS | Delivery |
| PH1.ASSET | Asset lifecycle |
| PH1.FLEET | Vehicles |
| PH1.INSURANCE | Policies/claims |
| HR/Payroll/Position | Employee setup |
| PH1.BOARD | Board approvals |
| PH1.SHAREHOLDER | Shareholder rights/votes |
| Finance / Budget | Finance setup, budget setup, reserve/profit policy |
| Accounting | Accounting setup, chart/journal/reporting handoff |
| Banking / Payment | Bank/payment setup and proof |
| Tax / Compliance | Jurisdiction and tax setup |
| Reporting | Report/board-pack readiness |

## 25. Company Onboarding To Finance/Accounting Wiring Map

| Onboarding discovery | Finance/accounting wiring |
| --- | --- |
| Country/jurisdiction | Tax, currency, fiscal year |
| Bank/payment setup | Banking/Payment readiness; no execution without authority |
| Customers pay later | AR, Credit, Collections |
| Suppliers invoice company | AP, Procurement, Receiving |
| Company keeps stock | Inventory, COGS handoff |
| Employees exist | Payroll/HR handoff |
| Assets exist | Asset/Depreciation handoff |
| Budgets exist | Finance/Budget authority |
| Board approval exists | Board -> Access/Authority |
| Multiple entities | Multi-entity accounting, consolidation future target |

Finance/accounting runtime must not be assumed. The repo-truth extraction says those owners are future gaps and must be activated carefully.

## 26. Company Onboarding To Commerce/Product/Inventory Wiring Map

| Human answer | Internal Selene wiring |
| --- | --- |
| "We sell products" | Product + Inventory + E-Commerce/POS |
| "We sell services" | Service/Product + Booking/Quote/Payment |
| "Customers order online" | E-Commerce |
| "Customers buy in-store" | POS |
| "Businesses buy from us" | B2B + Credit + AR |
| "We keep stock" | Inventory |
| "We manufacture" | Product + Inventory + Supplier + Procurement + Receiving + Production |
| "We deliver" | Logistics |
| "We sell perishable items" | Inventory shelf-life/JIT/FEFO |
| "We use suppliers" | Supplier + Procurement + Receiving + AP |

Product and Inventory must remain separate.

POS must not own Product or Inventory truth.

Accounting must not own physical stock truth.

## 27. Company Onboarding To Governance/Access Wiring Map

| Governance input | Routing |
| --- | --- |
| Owner/admin | Access baseline |
| Employee roles | Position + HR + Access |
| Approval limits | Access/Governance |
| Payment rules | Banking/Payment + Access |
| Purchase approval rules | Procurement + Access |
| Board rules | PH1.BOARD + Access |
| Shareholder rights | PH1.SHAREHOLDER |
| Bank signing rules | Banking + Access + Audit |
| Delegation authority | Access templates/overrides |
| Constitution/articles | Governance source review |
| Shareholder agreement | Governance source review |
| Conflicts | Legal/human review before activation |

Access/Governance owns permissions and authority.

PH1.D/GPT-5.5 is proposal-only.

PH1.WRITE owns final user-facing wording.

## 28. Codex Batch Recommendation

### Batch 1 — Docs-Only Master Design Creation

Create six docs:

```text
SELENE_COMPANY_ONBOARDING_UNIVERSAL_SPINE_MASTER_DESIGN.md
SELENE_COMPANY_ONBOARDING_SIZE_CLASSIFICATION_SIZE_PACKS_MASTER_DESIGN.md
SELENE_COMPANY_ONBOARDING_INDUSTRY_DETECTION_STARTER_PACKS_MASTER_DESIGN.md
SELENE_COMPANY_ONBOARDING_BUSINESS_MODEL_ACTIVATION_ROUTER_MASTER_DESIGN.md
SELENE_COMPANY_ONBOARDING_GOVERNANCE_SOURCE_PACK_MASTER_DESIGN.md
SELENE_COMPANY_ONBOARDING_EVIDENCE_FABRIC_CODEX_READINESS_LAYER.md
```

### Batch 2 — Index Only

Update:

```text
SELENE_MASTER_ARCHITECTURE_BUILD_SET.md
```

Do not rewrite final build plan yet.

### Batch 3 — No Implementation

Codex must report:

```text
docs-only
no runtime code
no schema
no packet implementation
no API implementation
no activation
no migrations
no tests claiming runtime
future logical packets only
pending Grand Architecture Reconciliation
pending Codex repo mapping
```

### Batch 4 — Later Activation Pack

After review, Codex may later create an approved activation pack if JD explicitly requests it:

```text
SELENE_COMPANY_ONBOARDING_REPO_TRUTH_ACTIVATION_PACK.md
```

That future pack would map these designs to real repo owners, symbols, storage, routes, simulations, tests, gaps, and old-path risks.

This task does not create that activation pack.

## 29. Final Architecture Sentence

Selene Company Onboarding Evidence Fabric + Codex Readiness Layer converts the six onboarding master designs into future logical packets, owner maps, status machines, evidence trails, acceptance tests, simulations, and preservation records so Codex can later map the work to repo truth without inventing runtime implementation or breaking Selene's owner boundaries.
