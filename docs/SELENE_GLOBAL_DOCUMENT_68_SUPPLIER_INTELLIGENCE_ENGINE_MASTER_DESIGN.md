# Global Document 68 — Selene Supplier Intelligence Engine

```text id="doc68_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
68

ENGINE:
PH1.SUPPLIER / PH1.SUPPLIER_INTELLIGENCE / PH1.SUPPLIER_RISK

FULL NAME:
Selene Supplier Intelligence, Qualification, Performance, Risk, Obligation, Compliance, Contract, Scorecard, and Replacement Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Supplier Intelligence Engine is the company’s supplier brain.

It owns the truth of:

```text id="1zuqv2"
who the supplier is
whether the supplier is approved
what the supplier supplies
which products/services they are linked to
how reliable they are
how often they are late
how often they send damaged goods
whether they owe credit notes, refunds, or replacements
whether their documents/certificates/insurance are current
whether they are safe to use
whether they should be preferred, watched, restricted, blocked, or replaced
```

This engine is not a vendor contact list.

A contact list says:

```text id="f4ccqr"
ABC Supplies
Mark
accounts@abc.example
```

Selene says:

```text id="hbrqte"
ABC Supplies is approved for packaging, late on 18% of deliveries, damaged two shipments this quarter, owes one credit note, has insurance expiring in 14 days, and should not be used for urgent orders until corrective action closes.
```

One of those is useful. The other is a business card that learned email.

---

## 2. Why Supplier Comes After Product + Inventory

Supplier Intelligence depends on the Product + Inventory foundation.

```text id="nmulaf"
Product says what the item is.
Inventory says what stock is needed.
Supplier says who can provide it and whether they are safe.
Procurement decides whether/how to buy.
Receiving proves what actually arrived.
AP validates invoice truth.
Payment pays only proven, approved amounts.
Reconciliation checks supplier statements.
```

Clean chain:

```text id="et32uf"
Product
→ Inventory
→ Supplier
→ Procurement / Purchase Order
→ Receiving / Inspection
→ AP / Creditors
→ Supplier Payment
→ Supplier Statement Reconciliation
→ Accounting
```

Supplier Engine must not be buried inside Product or Inventory.

Product links supplier product data.

Inventory uses supplier lead-time signals.

Supplier Engine owns supplier identity, trust, risk, performance, obligations, and approval.

Tiny difference. Huge number of invoices saved from nonsense.

---

## 3. Core Selene Law

```text id="supplier_core_law"
No supplier should be trusted blindly.

Every supplier must have identity, category, approval status, terms, risk, performance, obligations, and audit history before Selene relies on them for purchasing, receiving, AP, payment, product promotion, or B2B exposure.
```

Selene must reduce human work by:

```text id="v3sw4b"
reading supplier invoices
reading supplier registration documents
reading supplier catalogs
extracting supplier terms
detecting duplicate suppliers
requesting missing documents
monitoring certificate expiry
scoring supplier performance
tracking late deliveries
tracking damaged goods
tracking credit notes owed
tracking refunds owed
tracking replacement goods owed
warning Procurement before bad supplier use
blocking risky suppliers where policy requires
finding alternative suppliers
drafting supplier messages
creating supplier review packs
```

Humans should not have to remember:

> “Was this the supplier that sent rubbish last month?”

Selene should remember. Humans are already busy misplacing passwords and approving meetings.

---

## 4. Current Global Supplier Standards Selene Must Be Ready For

Supplier management is no longer only price and delivery. Selene must support supplier due diligence, sustainability, cyber risk, sanctions screening, and traceability.

ISO 20400 provides guidance for organizations of any size or activity on integrating sustainability into procurement, so Selene must be capable of storing sustainability, ethical sourcing, and supplier-risk evidence when a company needs it. ([ISO][1])

NIST SP 800-161 Rev. 1 provides guidance for identifying, assessing, and mitigating cybersecurity supply chain risks across organizations, so Selene should treat software/technology vendors and digital suppliers as risk-bearing suppliers, not just payable names. ([NIST Computer Security Resource Center][2])

OECD due diligence guidance helps businesses implement responsible business conduct due diligence and considers adverse impacts related to workers, human rights, environment, bribery, consumers, and corporate governance, so Selene must support supplier due diligence evidence and escalation where relevant. ([OECD][3])

Supplier screening also needs sanctions readiness. OFAC’s Sanctions List Search tool uses fuzzy logic to identify potential matches against SDN and non-SDN sanctions lists, and its Sanctions List Service provides access to sanctions list data; Selene must be able to connect to appropriate jurisdiction-specific sanctions/compliance screening providers where a company requires it. ([OFAC][4])

Translation: suppliers are not “people we buy from.” They are risk, cashflow, compliance, delivery, quality, cyber, reputation, and audit wrapped in an invoice.

Tiny vendor. Big trouble potential.

---

## 5. Engine Boundary

### 5.1 PH1.SUPPLIER owns

```text id="supplier_owns"
supplier identity
supplier qualification
supplier approval status
supplier categories
supplier terms
supplier contract references
supplier certificates
supplier insurance evidence
supplier compliance documents
supplier risk profile
supplier performance score
supplier delivery score
supplier quality score
supplier invoice accuracy score
supplier response score
supplier trust score
supplier obligations
supplier disputes
supplier corrective actions
supplier preferred/watchlist/restricted/blocked state
supplier alternative recommendations
supplier review cadence
supplier audit evidence
```

### 5.2 PH1.SUPPLIER does not own

```text id="supplier_not_own"
product identity
stock quantity
purchase order creation
goods receiving acceptance
supplier invoice validation
supplier payment execution
ledger posting
tax treatment
final contract legal approval
bank transfer execution
```

### 5.3 Correct owner split

```text id="supplier_owner_split"
PH1.PRODUCT = supplier product metadata link.
PH1.INVENTORY = supplier lead-time impact on stock/reorder.
PH1.SUPPLIER = supplier identity, approval, risk, score, obligations.
PH1.PROCUREMENT = supplier selection for PO and purchase decision.
PH1.PROC.RECEIVE = delivery proof and accepted/rejected quantities.
PH1.CREDITORS / AP = invoice matching and payable amount.
PH1.SUPPLIER_PAYMENT = payment scheduling and banking handoff.
PH1.CREDITORS.RECON = supplier statement reconciliation.
PH1.ACCOUNTING = ledger posting.
PH1.LEGAL / CONTRACTS = contract legal truth.
PH1.ACCESS / AUTHORITY = approvals.
PH1.AUDIT = proof.
```

Supplier Engine is the memory and judgment layer for who the business buys from.

It does not create the PO, receive the boxes, pay the invoice, or post the books.

It just remembers exactly why Supplier ABC should not be trusted with urgent dairy ever again.

---

## 6. Supplier Master Record

Every supplier must have a living supplier record.

```text id="supplier_master_record"
supplier_id
legal_entity_id
supplier_legal_name
supplier_trading_name
supplier_type
supplier_category
supplier_registration_number
supplier_tax_id
jurisdiction
country
registered_address
operating_address
delivery_zones
primary_contact
accounts_contact
operations_contact
support_contact
supplier_portal_status
selene_connected_status
approved_categories
products_supplied
services_supplied
supplier_product_links
payment_terms
currency
bank_detail_status
contract_refs
insurance_refs
certificate_refs
compliance_refs
quality_requirements
return_policy
credit_note_policy
refund_policy
replacement_policy
warranty_policy
lead_time_standard
lead_time_actual_average
MOQ
pack/case/carton_terms
supplier_status
risk_status
trust_score
performance_score
delivery_score
quality_score
invoice_accuracy_score
response_score
open_obligations
open_disputes
last_review_date
next_review_date
audit_ref
```

Supplier records must be:

```text id="supplier_record_rules"
versioned
source-backed
audited
permission-controlled
linked to products, procurement, receiving, AP, payment, and reconciliation
```

No “supplier terms live in Sarah’s email.” Sarah deserves a life. Also Sarah may resign.

---

## 7. Supplier Types

Selene must support many supplier types.

```text id="supplier_types"
product supplier
raw material supplier
finished goods supplier
service provider
contractor supplier
maintenance provider
fleet repairer
insurance provider / broker
logistics carrier
software vendor
cloud / technology provider
equipment supplier
professional services provider
property / landlord
manufacturer
wholesaler
distributor
drop-ship supplier
B2B trade partner
marketplace supplier
government/tax authority payee
related-party supplier
intercompany supplier
```

Different supplier types require different qualification depth.

Example:

```text id="supplier_type_examples"
Office stationery supplier = low-risk qualification.
Critical raw material supplier = quality, lead-time, contract, insurance, backup supplier review.
Software vendor = cyber/security/data/privacy risk review.
Logistics provider = insurance, delivery, damage, claims, SLA review.
Regulated product supplier = compliance certificate and traceability review.
```

Selene should not ask the same 90 questions for paper clips and aircraft parts. Humans dislike it. Aircraft regulators dislike the opposite.

---

## 8. Supplier Capture and Onboarding

Selene must create supplier records from minimal human input.

Capture sources:

```text id="supplier_capture_sources"
supplier invoice
supplier quote
supplier registration document
supplier catalog
supplier price list
supplier contract
supplier insurance certificate
supplier compliance certificate
supplier website
supplier email signature
bank details document
supplier portal registration
Selene-to-Selene supplier invitation
manual voice/text entry
```

Selene can say:

> “Upload their invoice or supplier document. I’ll extract the details and ask you to confirm.”

Selene extracts:

```text id="supplier_extracts"
legal name
trading name
tax ID
registration number
address
accounts contact
payment terms
bank detail presence
products/services supplied
supplier SKUs
prices
lead times
certificates
insurance
return/credit policies
```

Human confirms important truth.

Selene fills the forms because humans did not evolve to type supplier tax IDs accurately from PDFs at 4:58 PM.

---

## 9. Supplier Qualification

Supplier qualification decides whether Selene may rely on the supplier.

Qualification checks:

```text id="supplier_qualification_checks"
identity confirmed
tax details present
registration details present
contact details present
category assigned
payment terms known
bank details status known
insurance required/present
certifications required/present
contract required/present
sanctions/compliance screening if required
cyber/security review if required
sustainability/due diligence review if required
return/refund/credit policy known
delivery zones known
lead time known
approval owner assigned
risk score initialized
```

Qualification states:

```text id="supplier_qualification_states"
Draft
PendingInformation
PendingVerification
PendingApproval
Approved
Preferred
Watchlist
Restricted
Suspended
Blocked
Retired
```

Selene says:

> “This supplier is almost ready. I have legal name, tax details, products, and payment terms. I still need their insurance certificate before approving them for high-value orders.”

Supplier qualification should feel like a smart assistant doing the paperwork, not a portal asking humans to cosplay as procurement clerks.

---

## 10. Supplier Approval Rules

Supplier approval depends on category, risk, spend, and criticality.

### Low-risk supplier

```text id="low_risk_supplier"
low spend
routine supplies
non-critical
easy substitute
no sensitive data
no regulated products
```

May auto-approve under policy or require light admin approval.

### Medium-risk supplier

```text id="medium_risk_supplier"
recurring supplier
inventory supplier
service provider
branch-critical
moderate spend
```

Requires procurement/finance review depending policy.

### High-risk supplier

```text id="high_risk_supplier"
critical supply
large spend
regulated goods
customer-facing goods
sensitive data access
single-source dependency
bank detail risk
safety/compliance relevance
```

Requires stronger approval.

### Critical supplier

```text id="critical_supplier"
production-critical
core customer delivery dependency
only approved supplier
high-value contract
strategic supplier
significant compliance exposure
```

May require:

```text id="critical_supplier_approval"
Procurement approval
Finance approval
Legal/Compliance approval
Security review
Board/executive approval if threshold requires
```

Selene rule:

```text id="supplier_approval_law"
Routine low-risk supplier approvals can be automated under policy.
High-risk supplier approval must be routed to authority.
Supplier status must be auditable.
```

No supplier becomes “preferred” because someone liked their sales rep. Procurement by charm is how companies adopt chaos.

---

## 11. Supplier Scorecard

Selene continuously scores supplier performance.

Score categories:

```text id="supplier_score_categories"
delivery performance
quality performance
invoice accuracy
price stability
response speed
dispute resolution
credit note reliability
replacement reliability
refund reliability
documentation compliance
contract compliance
cyber/security risk where applicable
sustainability/due diligence risk where applicable
overall trust score
```

Example scorecard:

```text id="supplier_scorecard"
Delivery Score: 78/100
Quality Score: 64/100
Invoice Accuracy: 93/100
Response Score: 52/100
Credit Note Reliability: 46/100
Overall Status: Watchlist
```

Supplier ranking:

```text id="supplier_ranking"
A — Preferred
B — Approved
C — Watchlist
D — Restricted
E — Blocked
```

Selene says:

> “Supplier ABC is cheaper, but their quality score dropped below threshold after two damaged deliveries. I recommend restricting them for urgent or customer-facing orders.”

A human might forget. Selene should not.

---

## 12. Supplier Performance Inputs

Selene learns from all supplier events.

```text id="supplier_performance_inputs"
late deliveries
early deliveries
short deliveries
damaged goods
wrong goods
faulty goods
over-deliveries
missing certificates
expired certificates
duplicate invoices
invoice without PO
price variance
credit note delay
refund delay
replacement delay
supplier response time
supplier dispute behavior
supplier statement mismatch
supplier payment queries
customer impact
production impact
cashflow impact
```

Input owners:

```text id="supplier_input_owners"
Receiving sends delivery/quality events.
Procurement sends PO acceptance/change events.
AP sends invoice accuracy events.
Payment sends bank/payment-risk events.
Reconciliation sends statement mismatch events.
Inventory sends stockout/supplier lead-time impact.
Product sends supplier product data/compliance gaps.
```

Supplier Engine consolidates the story.

No more “supplier seems fine” while five engines are screaming quietly in different tabs.

---

## 13. Supplier Obligation Ledger

Every unresolved supplier obligation must stay open until resolved.

Obligation types:

```text id="supplier_obligations"
credit note owed
refund owed
replacement goods owed
missing quantity owed
damaged goods resolution
wrong goods correction
warranty claim
price correction
overcharge correction
duplicate invoice reversal
contract penalty
certificate renewal
insurance renewal
corrected invoice
```

Obligation record:

```text id="supplier_obligation_record"
obligation_id
supplier_id
source_engine
source_ref
po_id
receiving_id
invoice_id
product_id
quantity
value
currency
reason
requested_resolution
due_date
status
AP_hold_ref
evidence_refs
owner
audit_ref
```

Example:

```text id="obligation_example"
PO: 100 units
Received accepted: 95
Damaged: 5

Supplier obligation:
5 replacement units OR credit note/refund
AP hold: value of 5 units
Status: Open
Due date: 7 days
```

Selene says:

> “Supplier ABC still owes a credit note for five damaged units. I recommend holding the disputed invoice amount until resolved.”

No more supplier obligations rotting in the swamp called “someone should chase this.”

---

## 14. Supplier Dispute Lifecycle

Disputes originate from:

```text id="supplier_dispute_sources"
short delivery
damaged goods
wrong goods
faulty goods
late delivery
invoice overcharge
duplicate invoice
missing credit note
contract breach
warranty issue
service failure
supplier statement mismatch
```

Dispute states:

```text id="supplier_dispute_states"
Detected
EvidenceRequired
SupplierNotified
AwaitingSupplierResponse
ResolutionProposed
ReplacementPending
CreditNotePending
RefundPending
UnderReview
Escalated
Closed
Archived
```

Dispute evidence:

```text id="dispute_evidence"
PO
delivery note
receiving record
inspection record
photos
videos
quantity count
invoice
credit note request
supplier messages
contract terms
quality report
AP hold
payment record
```

Selene says:

> “The delivery is short by five units. I have receiving proof and will request credit or replacement under policy.”

Routine disputes should be automated.

Material, disputed, or unusual disputes escalate.

No supervisor needed to approve reality when the box is visibly broken. We have cameras. Use them.

---

## 15. Supplier Corrective Action

If failures repeat, Selene opens corrective action.

Triggers:

```text id="corrective_action_triggers"
3 late deliveries in 60 days
2 damaged deliveries in one quarter
duplicate invoice pattern
unresolved credit notes
quality failure recurrence
certificate expiry ignored
supplier misses dispute response deadline
supplier causes customer delivery failure
supplier causes production delay
```

Corrective action flow:

```text id="corrective_action_flow"
issue detected
evidence compiled
supplier notified
root cause requested
corrective action requested
deadline assigned
supplier response evaluated
performance monitored
restriction/block recommended if unresolved
```

Selene says:

> “This is the third damaged delivery from Supplier ABC this quarter. I recommend corrective action and restricting new orders until they respond.”

That is supplier management. Not “send another annoyed email and hope.” Hope has terrible procurement metrics.

---

## 16. Supplier Alternatives and Replacement Intelligence

Selene must not merely complain about bad suppliers.

She must find better options.

Alternative supplier discovery checks:

```text id="alternative_supplier_checks"
existing approved suppliers
supplier network
similar products/services
category match
price
MOQ
lead time
delivery reliability
quality score
certifications
insurance
delivery zones
payment terms
B2B network availability
geographic risk
single-source dependency
```

Possible actions:

```text id="alternative_supplier_actions"
recommend backup supplier
recommend trial order
recommend split sourcing
recommend supplier switch
recommend request for quote
recommend qualification process
```

Selene says:

> “Supplier ABC is restricted. Supplier B costs 3% more but has stronger delivery and quality scores. I recommend a trial order.”

Supplier replacement should be evidence-based, not “I found a cheaper one on the internet, let’s gamble the factory.”

---

## 17. Supplier Contract and Terms Memory

Supplier Engine stores terms.

```text id="supplier_contract_terms"
payment terms
delivery terms
return terms
credit note terms
refund terms
replacement terms
warranty terms
service-level agreement
pricing agreement
volume discount
MOQ
lead time
penalty terms
renewal date
termination notice
insurance requirement
certificate requirement
data/security requirement
confidentiality requirement
```

Selene monitors:

```text id="supplier_terms_monitoring"
contract expiry
certificate expiry
insurance expiry
price review date
volume discount threshold
SLA breach
renewal window
termination deadline
```

Selene says:

> “This supplier contract renews in 30 days. Their quality score has dropped since last renewal. I recommend review before auto-renewal.”

Contracts should not renew because humans were busy. That is not automation. That is surrender with a calendar invite.

---

## 18. Supplier Bank Detail Safety

Supplier bank details are high-risk.

This document defines the basic safety boundary.

Detailed supplier-originated bank-change protocol belongs in:

```text id="bank_addendum_reference"
Global Document 69 — Supplier Bank Change + Selene-to-Selene Counterparty Addendum
```

Supplier Engine must track bank status:

```text id="supplier_bank_status"
NotProvided
PendingVerification
Verified
ChangeRequested
ChangeUnderReview
RecentlyChanged
Approved
Rejected
Suspended
```

Basic rule:

```text id="supplier_bank_law"
No supplier payment may proceed to unverified, changed, suspended, or unknown bank details unless protected authority explicitly permits it.
```

Selene says:

> “This supplier’s bank details changed recently. I recommend payment hold until verification completes.”

No paying a new bank account because a PDF asked nicely. Fraud also has fonts.

---

## 19. Selene-Connected vs Non-Selene Suppliers

Selene must support two supplier relationship modes.

```text id="supplier_modes"
Selene-connected supplier
Non-Selene supplier
```

### 19.1 Selene-connected supplier

Supplier has their own Selene.

Can exchange authenticated messages such as:

```text id="selene_supplier_messages"
PO acknowledgement
shipment confirmation
delivery delay
credit note
replacement notice
refund confirmation
invoice submission
supplier statement
bank-change request
certificate renewal
price list update
```

Rule:

```text id="selene_supplier_rule"
Supplier Selene can request.
Buyer Selene validates.
Buyer policy decides.
Audit records.
```

### 19.2 Non-Selene supplier

Uses:

```text id="non_selene_supplier_channels"
supplier portal
verified email
signed document
callback verification
manual upload
approved contact
```

Selene treats non-Selene messages as lower trust until verified.

No shame. Just risk. Email has betrayed us all.

---

## 20. Supplier Compliance, Sanctions, and Due Diligence

Selene must support compliance checks where required.

Compliance check types:

```text id="supplier_compliance_checks"
sanctions screening
registration verification
tax ID verification
insurance verification
certificate verification
modern slavery / forced labor due diligence
human rights due diligence
environmental due diligence
anti-bribery / corruption due diligence
conflict minerals due diligence
cybersecurity / data access review
product safety certification
country/geopolitical risk
beneficial ownership check
```

Compliance status:

```text id="supplier_compliance_status"
NotRequired
Required
PendingEvidence
PendingReview
Approved
Watchlist
Restricted
Blocked
Expired
```

Selene says:

> “This supplier provides customer-facing goods from a higher-risk region. I recommend due diligence review before approval.”

Supplier compliance should be configurable by company size, industry, jurisdiction, risk, and product category.

A café buying napkins and a defense contractor buying components do not need the same supplier screening. If your system thinks they do, your system needs a nap.

---

## 21. Cyber / Software Supplier Risk

Technology suppliers require special handling.

Risk triggers:

```text id="cyber_supplier_triggers"
supplier accesses company data
supplier handles customer data
supplier connects via API
supplier provides cloud service
supplier provides software dependency
supplier handles payments
supplier provides authentication/security tooling
supplier has admin access
```

Selene tracks:

```text id="cyber_supplier_fields"
data access level
integration type
security questionnaire
SOC/ISO/security evidence where applicable
breach notification terms
data processing agreement
contractual security obligations
renewal date
risk score
```

Selene says:

> “This software vendor will access customer data through API. I recommend cybersecurity supplier review before approval.”

A supplier can deliver no boxes and still burn the company down with an API key. Modernity is adorable.

---

## 22. Supplier Review Cadence

Supplier review frequency depends on risk.

```text id="supplier_review_cadence"
critical supplier = monthly / quarterly
high-risk supplier = quarterly
medium supplier = semi-annually
low-risk supplier = annually or event-driven
watchlist supplier = active monitoring
blocked supplier = no purchasing except override
```

Event-driven reviews occur when:

```text id="supplier_review_triggers"
delivery failure
quality failure
credit note overdue
certificate expires
contract renewal
supplier bank change
large spend increase
new product/category supplied
compliance alert
cyber incident
price increase
single-source dependency detected
```

Selene says:

> “Supplier review is due because spend increased 240% this quarter.”

A supplier that was tiny last year may be business-critical this year. Growth has consequences. Mostly admin.

---

## 23. Supplier Relationship and Communication Memory

Selene stores supplier communication events.

```text id="supplier_communication_memory"
supplier emails
portal messages
Selene-to-Selene messages
dispute messages
credit note requests
replacement requests
pricing negotiations
contract renewal discussions
delivery updates
payment status replies
```

Selene summarizes:

> “Supplier ABC promised replacement by Friday. If not received, I’ll escalate under policy.”

Supplier promises should not evaporate into email fog. Fog is not an audit trail.

---

## 24. Supplier and Product Relationship

Product Engine links to Supplier Engine.

Product uses:

```text id="supplier_product_relationship"
supplier_id
supplier_sku
supplier_product_name
supplier cost
pack size
MOQ
lead time
country of origin
supplier certificates
supplier status
supplier quality risk
preferred supplier flag
backup supplier flag
```

Supplier Engine tells Product:

```text id="supplier_to_product"
supplier watchlist
supplier restricted
supplier certificate expired
supplier quality risk
supplier alternative available
supplier compliance hold
```

Product may respond:

```text id="product_response"
supplier risk hold
do not promote
B2B exposure warning
alternative supplier needed
compliance data missing
```

Selene says:

> “This product’s preferred supplier is restricted due to repeated damaged goods. I recommend pausing promotion or approving a backup supplier.”

Product doesn’t own supplier truth. It listens to it.

---

## 25. Supplier and Inventory Relationship

Inventory uses supplier intelligence.

Supplier Engine provides:

```text id="supplier_inventory_signals"
lead time
lead-time reliability
quality reliability
delivery delay risk
short delivery risk
damage risk
replacement speed
supplier restriction
single-source risk
```

Inventory adjusts:

```text id="inventory_supplier_adjustments"
safety stock
JIT viability
reorder timing
supplier recommendation
stockout risk
promotion risk
B2B promise risk
```

Selene says:

> “JIT is not safe with Supplier ABC because their lead time varies too much. I recommend increasing safety stock or switching supplier.”

JIT with an unreliable supplier is just stockout roulette in a corporate polo.

---

## 26. Supplier and Procurement Relationship

Procurement asks Supplier Engine:

```text id="procurement_supplier_queries"
is supplier approved?
is supplier preferred?
is supplier restricted?
what are payment terms?
what is lead time?
what is performance score?
are there open obligations?
is bank status safe?
are documents current?
is supplier allowed for this category?
```

Supplier Engine responds:

```text id="supplier_procurement_response"
approved / not approved
preferred / alternative
risk score
warnings
blocked reasons
watchlist notes
open obligation warning
compliance status
```

Procurement then decides PO action.

Selene says:

> “Supplier ABC is approved but has overdue credit notes. I recommend resolving obligations before issuing another PO.”

No more rewarding suppliers for failing to clean up old messes.

---

## 27. Supplier and Receiving Relationship

Receiving updates supplier performance.

Receiving sends:

```text id="receiving_supplier_events"
delivery arrived
delivery late
quantity accepted
quantity short
quantity damaged
wrong item
quality result
inspection result
certificate missing
photos/evidence
```

Supplier Engine updates:

```text id="supplier_updates_from_receiving"
delivery score
quality score
open obligations
dispute count
trust score
watchlist/restriction recommendation
```

Selene says:

> “Supplier quality score reduced because 10% of this delivery was damaged.”

Receiving proves reality.

Supplier remembers it.

A normal system forgets and orders again. Selene does not.

---

## 28. Supplier and AP / Creditors Relationship

AP asks Supplier Engine:

```text id="ap_supplier_queries"
is supplier approved?
is supplier bank verified?
is supplier on hold?
are there open disputes?
are credit notes owed?
is supplier restricted?
is invoice from recognized supplier?
```

Supplier Engine can trigger AP hold:

```text id="supplier_ap_holds"
supplier blocked
bank unverified
bank recently changed
supplier owes credit note
supplier dispute unresolved
supplier invoice accuracy risk
duplicate invoice pattern
```

Selene says:

> “This supplier still owes a credit note from the damaged delivery. AP should hold the disputed amount.”

AP validates invoice.

Supplier explains whether supplier is safe to pay.

---

## 29. Supplier and Payment Relationship

Supplier Payment asks Supplier Engine:

```text id="payment_supplier_queries"
bank details verified?
bank recently changed?
supplier restricted?
supplier blocked?
supplier payment hold?
supplier critical?
early payment discount offered?
supplier urgent payment request?
```

Supplier Engine provides supplier safety and relationship context.

Payment Engine decides timing with Cashflow and Authority.

Selene says:

> “Supplier is critical and invoice is clean, but bank details changed recently. Hold payment until verification completes.”

This is how Selene avoids being emotionally manipulated by urgent supplier payment emails. Urgency is not verification. Put that on a mug.

---

## 30. Supplier and Reconciliation Relationship

Creditor Reconciliation sends supplier statement outcomes.

Supplier Engine learns:

```text id="recon_supplier_learning"
statement mismatch frequency
unknown invoice claims
missing credit note pattern
supplier missing our payments
duplicate claim pattern
unapplied credit pattern
balance dispute behavior
```

Selene says:

> “Supplier ABC’s statements have mismatched three months in a row. I recommend accounts process review or restriction.”

Bad supplier admin is also supplier performance. It wastes humans. Selene should score that.

---

## 31. Supplier and Cashflow Relationship

Supplier terms affect cashflow.

Supplier Engine provides:

```text id="supplier_cashflow_fields"
payment terms
early payment discount terms
late fee terms
critical supplier status
negotiation history
payment flexibility
relationship risk
```

Cashflow uses this to prioritize payments.

Selene says:

> “Supplier ABC offers 2% early payment discount and cashflow is green. Payment Engine may consider early payment.”

Supplier data helps Cashflow decide, but Cashflow owns liquidity.

Everyone stays in lane. Beautiful, if a bit unnatural.

---

## 32. Supplier Risk Model

Supplier risk categories:

```text id="supplier_risk_categories"
financial risk
delivery risk
quality risk
compliance risk
cyber risk
sanctions/geopolitical risk
single-source risk
contract risk
bank/payment fraud risk
price volatility risk
sustainability/due diligence risk
reputation risk
data/privacy risk
```

Risk states:

```text id="supplier_risk_states"
Low
Medium
High
Critical
Blocked
```

Risk actions:

```text id="supplier_risk_actions"
monitor
request evidence
reduce order volume
require approval
use backup supplier
restrict category
block purchases
escalate to Legal/Compliance/Board
```

Selene says:

> “Single-source risk is high. This company depends on one supplier for 82% of Product X. I recommend approving a backup supplier.”

If one supplier can stop the business, that supplier is not a vendor. That supplier is a hostage situation with invoices.

---

## 33. Supplier Portal

Selene should support supplier self-service.

Supplier portal functions:

```text id="supplier_portal"
confirm business details
upload certificates
upload insurance
acknowledge PO
submit invoice
submit credit note
confirm shipment
respond to dispute
upload replacement ETA
update catalog/price list
submit statement
request payment status
request early payment
submit bank-change request if protocol allows
```

Supplier portal changes are not blindly trusted.

Selene validates:

```text id="supplier_portal_validation"
supplier identity
authorized supplier user
document evidence
policy rules
approval requirements
audit
```

Supplier portal is convenience.

Not truth by itself.

---

## 34. Supplier Early Payment and Discount Requests

Supplier may request:

```text id="supplier_payment_requests"
early payment
urgent payment
partial early payment
early-payment discount
payment status update
```

Supplier Engine records supplier request.

Payment/Cashflow decide.

Supplier Engine provides context:

```text id="supplier_payment_context"
supplier criticality
supplier relationship
supplier behavior history
open disputes
bank safety
supplier urgent request pattern
early payment discount history
```

Selene says:

> “Supplier requested urgent payment. Invoice is clean, but payment is not due for 18 days. Cashflow and policy should evaluate whether urgency benefits us.”

Supplier asking loudly is not the same as business urgency. Truly shocking to suppliers.

---

## 35. Automation and Exception-Only Review

Selene auto-handles:

```text id="supplier_auto_handles"
supplier detail extraction
duplicate supplier detection
supplier category suggestion
missing document request
certificate expiry reminders
routine supplier score updates
routine supplier obligation creation
routine credit note/replacement chase
supplier review scheduling
supplier status warning to Procurement/AP
supplier statement behavior scoring
supplier communication summarization
```

Selene requires review for:

```text id="supplier_review_required"
new high-risk supplier
supplier approval above threshold
supplier restriction/block
supplier bank change
critical supplier switch
contract termination
legal dispute
sanctions/compliance hit
cyber/data-risk approval
material settlement
removing supplier hold
overriding blocked supplier
```

Routine = Selene handles.

Risk = Selene routes.

Everything = audited.

No approval circus. No blind trust circus either. We are aiming between two kinds of stupid.

---

## 36. PH1.D / GPT-5.5 Role

GPT-5.5 should be used heavily for supplier intelligence explanation and drafting.

### 36.1 GPT-5.5 may help

```text id="gpt_supplier_allowed"
summarize supplier history
summarize supplier scorecard
draft supplier emails
draft credit note request
draft corrective action request
draft supplier review report
compare supplier options
explain supplier risk in plain English
summarize contract terms as proposal
translate supplier messages
prepare procurement briefing
```

### 36.2 GPT-5.5 must not

```text id="gpt_supplier_forbidden"
approve supplier
block supplier without authority
change supplier bank details
release supplier payment
invent supplier documents
invent supplier certificates
invent delivery performance
remove AP hold
close supplier obligation without proof
override compliance hit
```

GPT-5.5 writes and explains.

Selene deterministic engines decide trust, risk, policy, and proof.

The fluent assistant does not get the supplier bank details button. We like our money where it is.

---

## 37. Human-Like Selene Interaction

### New supplier

> “Upload their invoice or supplier document. I’ll extract the legal name, tax number, contact details, products, and payment terms.”

### Missing certificate

> “This supplier is almost ready, but insurance is missing. I’ll request it before approving high-value orders.”

### Supplier risk

> “Supplier ABC is cheaper, but their last three deliveries were late. I recommend using Supplier B for urgent orders.”

### Credit owed

> “Supplier ABC still owes a credit note for five damaged units. I’ll keep the disputed amount on AP hold.”

### Corrective action

> “This is the third damaged delivery this quarter. I recommend corrective action and restricting new orders until they respond.”

### Alternative supplier

> “I found two alternative approved suppliers. Supplier B costs 3% more but has better delivery reliability.”

Human-like, but not soft. Selene should be polite to suppliers, not gullible. Big difference, often measurable in money.

---

## 38. Supplier State Machines

### Supplier Approval State

```text id="supplier_approval_state"
Draft
PendingInformation
PendingVerification
PendingApproval
Approved
Preferred
Watchlist
Restricted
Suspended
Blocked
Retired
```

### Supplier Obligation State

```text id="supplier_obligation_state"
Open
SupplierAcknowledged
AwaitingReplacement
AwaitingCreditNote
AwaitingRefund
PartiallyResolved
Resolved
Overdue
Escalated
Closed
Archived
```

### Supplier Dispute State

```text id="supplier_dispute_state"
Detected
EvidenceRequired
SupplierNotified
AwaitingSupplierResponse
ResolutionProposed
ReplacementPending
CreditNotePending
RefundPending
UnderReview
Escalated
Closed
Archived
```

### Supplier Corrective Action State

```text id="supplier_corrective_action_state"
Triggered
EvidenceCompiled
SupplierNotified
AwaitingRootCause
CorrectiveActionRequested
Monitoring
ImprovementObserved
NoImprovement
RestrictionRecommended
Closed
Archived
```

### Supplier Document State

```text id="supplier_document_state"
Missing
Requested
Uploaded
UnderReview
Approved
Rejected
Expired
RenewalRequested
Archived
```

### Supplier Risk State

```text id="supplier_risk_state"
Low
Medium
High
Critical
Blocked
```

### Supplier Bank Safety State

```text id="supplier_bank_state"
NotProvided
PendingVerification
Verified
ChangeRequested
ChangeUnderReview
RecentlyChanged
Approved
Rejected
Suspended
```

---

## 39. Reason Codes

```text id="supplier_reason_codes"
SUPPLIER_CAPTURED_FROM_INVOICE
SUPPLIER_CAPTURED_FROM_DOCUMENT
SUPPLIER_DUPLICATE_POSSIBLE
SUPPLIER_PENDING_INFORMATION
SUPPLIER_PENDING_VERIFICATION
SUPPLIER_APPROVAL_REQUIRED
SUPPLIER_APPROVED
SUPPLIER_PREFERRED
SUPPLIER_WATCHLIST
SUPPLIER_RESTRICTED
SUPPLIER_BLOCKED
SUPPLIER_CERTIFICATE_MISSING
SUPPLIER_CERTIFICATE_EXPIRED
SUPPLIER_INSURANCE_MISSING
SUPPLIER_SANCTIONS_REVIEW_REQUIRED
SUPPLIER_DUE_DILIGENCE_REQUIRED
SUPPLIER_CYBER_REVIEW_REQUIRED
SUPPLIER_DELIVERY_LATE
SUPPLIER_SHORT_DELIVERY
SUPPLIER_DAMAGED_GOODS
SUPPLIER_WRONG_GOODS
SUPPLIER_FAULTY_GOODS
SUPPLIER_DUPLICATE_INVOICE_PATTERN
SUPPLIER_CREDIT_NOTE_OWED
SUPPLIER_REFUND_OWED
SUPPLIER_REPLACEMENT_OWED
SUPPLIER_CORRECTIVE_ACTION_REQUIRED
SUPPLIER_ALTERNATIVE_RECOMMENDED
SUPPLIER_SINGLE_SOURCE_RISK
SUPPLIER_BANK_DETAILS_UNVERIFIED
SUPPLIER_BANK_CHANGE_HOLD
SUPPLIER_PAYMENT_HOLD_RECOMMENDED
```

---

## 40. Required Simulations

```text id="supplier_simulations"
add supplier from invoice
add supplier from registration document
detect duplicate supplier
qualify low-risk supplier
qualify high-risk supplier
supplier missing insurance
supplier certificate expired
supplier sanctions review required
supplier cyber review required
supplier approved
supplier moved to watchlist
supplier restricted
supplier blocked
supplier delivers late
supplier delivers short
supplier delivers damaged goods
supplier owes credit note
supplier owes replacement
supplier dispute opened
supplier corrective action opened
supplier score downgraded
supplier alternative recommended
supplier selected by Procurement
supplier blocked from Procurement
supplier hold blocks AP payment
supplier payment request evaluated
supplier statement mismatch affects supplier score
supplier review report generated
```

---

## 41. Integration Map

```text id="supplier_integration_map"
PH1.SUPPLIER / SUPPLIER_INTELLIGENCE
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.PROCUREMENT / PH1.PROC.ORDER
↔ PH1.PROC.RECEIVE
↔ PH1.CREDITORS / AP
↔ PH1.SUPPLIER_PAYMENT
↔ PH1.CREDITORS.RECON
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.BUDGET
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.LEGAL / CONTRACTS
↔ PH1.COMPLIANCE
↔ PH1.LOGISTICS
↔ PH1.INSURANCE
↔ PH1.ASSET
↔ PH1.FLEET
↔ PH1.B2B
↔ PH1.ECOMMERCE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 42. Required Logical Packets

```text id="supplier_packets"
SupplierCapturePacket
SupplierIdentityPacket
SupplierQualificationPacket
SupplierApprovalPacket
SupplierCategoryPacket
SupplierTermsPacket
SupplierProductLinkPacket
SupplierDocumentPacket
SupplierCertificatePacket
SupplierInsurancePacket
SupplierCompliancePacket
SupplierDueDiligencePacket
SupplierCyberRiskPacket
SupplierScorecardPacket
SupplierPerformanceEventPacket
SupplierRiskPacket
SupplierObligationPacket
SupplierDisputePacket
SupplierCorrectiveActionPacket
SupplierAlternativeRecommendationPacket
SupplierReviewPacket
SupplierPaymentSafetyPacket
SupplierPortalMessagePacket
SupplierAuditEvidencePacket
```

Logical only. Codex maps later. No packet structs, little repo gremlin.

---

## 43. What Codex Must Not Do

```text id="codex_no_supplier"
Do not merge Supplier into Product.
Do not merge Supplier into Inventory.
Do not let Supplier create purchase orders.
Do not let Supplier receive goods.
Do not let Supplier validate invoices.
Do not let Supplier execute payments.
Do not let Supplier post ledger.
Do not let GPT-5.5 approve supplier status.
Do not let GPT-5.5 invent supplier documents.
Do not allow supplier bank changes without Document 69 protocol.
Do not close supplier obligations without proof.
Do not ignore supplier compliance/cyber/sanctions risk where required.
Do not implement from this document alone.
```

---

## 44. Final Architecture Sentence

Selene Supplier Intelligence Engine is the autonomous supplier brain that captures and qualifies suppliers from invoices, documents, catalogs, portals, and Selene-to-Selene messages; stores supplier identity, categories, terms, contracts, insurance, certificates, compliance, risk, approval status, performance, obligations, disputes, corrective actions, and alternative recommendations; continuously learns from procurement, receiving, AP, payment, inventory, product, and reconciliation events; warns or blocks supplier use where policy requires; prepares supplier review and communication through GPT-5.5; and keeps supplier trust, risk, payment safety, and audit evidence separate from product identity, stock truth, purchase orders, receiving, invoices, payments, and accounting.

Simple version:

```text id="supplier_simple"
Selene knows who the supplier is.
Selene checks if they are approved.
Selene remembers what they supply.
Selene tracks if they are late.
Selene tracks damaged and short deliveries.
Selene tracks credit notes, refunds, and replacements owed.
Selene checks documents, insurance, compliance, and cyber risk.
Selene warns before bad suppliers are used.
Selene finds better suppliers.
Selene protects AP and payments from supplier risk.
Humans approve only real exceptions.
Everything is audited.
```

That is Global Document 68 — Supplier Intelligence Engine. Supplier management is no longer a contact list and a hopeful purchase order. It is a living memory of who helps the business, who hurts it, and who keeps trying to invoice for five damaged boxes like nobody owns a camera.

[1]: https://www.iso.org/standard/63026.html?utm_source=chatgpt.com "ISO 20400:2017 - Sustainable procurement — Guidance"
[2]: https://csrc.nist.gov/pubs/sp/800/161/r1/final?utm_source=chatgpt.com "SP 800-161 Rev. 1, Cybersecurity Supply Chain Risk ..."
[3]: https://www.oecd.org/en/publications/oecd-due-diligence-guidance-for-responsible-business-conduct_15f5f4b3-en.html?utm_source=chatgpt.com "OECD Due Diligence Guidance for Responsible Business ..."
[4]: https://ofac.treasury.gov/sanctions-list-search-tool?utm_source=chatgpt.com "Sanctions List Search Tool"
