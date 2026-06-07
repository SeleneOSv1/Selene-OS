# Global Document 84 — Selene Commerce Integration Overview + Relationship Master Map

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
84

ENGINE:
PH1.COMMERCE_INTEGRATION / PH1.COMMERCE_RELATIONSHIP_MAP / PH1.DOCUMENT_77_TO_83_ORCHESTRATION

FULL NAME:
Selene Commerce Integration Overview, Documents 77–83 Relationship Master Map, E-Commerce, B2B, POS, Order, Pricing Pack, Dispatch, Returns, Refunds, Human Orchestration, Terms, Provider Responsibility, Delivery Proof, Reversal, Audit, Status Mapping, Shared Data Spine, and Commerce Control Architecture

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Document 84 is the **grand commerce wiring diagram** for Selene Documents 77–83.

It explains how the commerce stack works end-to-end:

```text
77 — E-Commerce / Personal Commerce / Company Store
78 — B2B Platform + Trade Ecosystem
79 — POS + Commerce Execution
80 — Order Management + Order Orchestration
81 — Pricing Pack
82 — Dispatch, Packing, Courier Booking + Delivery Network Handoff
83 — Returns, Refunds + Reverse Logistics
```

Document 84 defines:

```text
what each document owns
what each document must not own
which document calls which other document
what data must pass between them
what status must be preserved
what proof must be captured
what human actions must be scheduled
what happens when something fails
what missing cross-references must be added
```

Simple version:

```text
84 is the wiring diagram.
77–83 are the machines.
```

Without 84, the commerce documents are impressive engines sitting on the floor with no belts connected. Very shiny. Utterly useless. Like a luxury car with no wheels, which is somehow still on-brand for enterprise architecture.

---

## 2. Core Commerce Integration Law

```text
Selene must not allow E-Commerce, B2B, POS, Order, Pricing, Dispatch, or Returns to operate as disconnected engines.

Every commerce action must preserve:
- customer context
- product/service truth
- provider/seller responsibility
- pricing decision
- product terms
- delivery promise
- dispatch proof
- return/refund/warranty terms
- B2B attribution where applicable
- payment/settlement effect
- human/external task ownership where required
- audit evidence
```

No commerce action may quietly drop context.

No product may be sold without clear terms.

No B2B order may lose Original Provider responsibility.

No dispatch may happen without proof path.

No refund may happen without terms and evidence.

No human task may float around as a sad little sentence.

No price/offer/claim may exist without audit.

This is Selene commerce law, because otherwise everything becomes “order confirmed” followed by seven people blaming Dave.

---

## 3. Commerce Stack Overview

### Document 77 — E-Commerce / Personal Commerce / Company Store

Document 77 owns:

```text
customer shopping experience
company store experience
personal Selene shopping
product/service display
shopping lists
cart experience
product questions
customer journey before order
customer-facing provider/store context
```

77 must call or reference:

```text
Product for product truth
81 for price/offer
81J for presentation readiness
81I for delivery/geography availability
81H for service capability
81D for brand/official-channel rules
81E for B2B customer benefits/pricing where applicable
83/Product for return/refund/warranty terms display
80 to create order
78 when product is B2B-sourced
```

77 must not:

```text
invent price
invent delivery promise
invent return terms
invent provider responsibility
display B2B products without B2B permission
display brand-sensitive products without brand/official-channel permission
create dispatch directly
process refunds directly
```

---

### Document 78 — B2B Platform + Trade Ecosystem

Document 78 owns:

```text
Original Provider
Channel Store
B2B attribution
provider responsibility
Channel Store commission rights
provider-customer attribution
brand/channel approval requests
referral-only vs direct-sale rules
B2B participation
B2B product adoption
B2B relationship governance
```

78 must call or reference:

```text
81E for B2B price stack / commission / customer benefit / reserves
81D for brand/channel/official approval
81H for service capability qualification
81I for territory/geography authorization
81J for B2B display readiness
80 for order structure
82 for provider-direct dispatch
83 for return/refund/warranty/provider responsibility
81G for audit/explainability
```

Critical law:

```text
Customer fit does not override brand authorization.
```

Example:

```text
A high-end salon may have customers who buy Armani.
But the salon cannot directly sell Armani unless official brand/channel approval allows it.
It may be referral-only.
```

The salon may have taste. That does not make it an authorized luxury distributor. Tragic for the salon. Healthy for the brand.

---

### Document 79 — POS + Commerce Execution

Document 79 owns:

```text
in-store checkout
mobile POS
scan-as-you-go
customer device bill/payment
restaurant/table ordering
split bills
payment option presentation
POS discounts/offers
barcode/QR interaction
in-store customer execution
```

79 must call or reference:

```text
81 for price/offer/payment suggestion
81C for customer payment/benefit preference
81F for POS promotion tests
81G for audit/explanation
81E for B2B item economics/refund effects
81D for brand discount override rules
82 for pickup/dispatch handoff
83 for POS returns/refunds
80 for order creation where POS creates order lines
```

79 must not:

```text
override pricing without authority
apply brand-sensitive discount without 81D/81 Core
apply B2B discount without 81E
process refunds outside 83
ignore terms display before POS sale
ignore product/service return conditions
```

POS humans are discount machines with shoes. Selene must supervise them gently, like toddlers with barcode scanners.

---

### Document 80 — Order Management + Order Orchestration

Document 80 owns:

```text
customer order group
seller orders
order lines
source/store/provider resolution
order-line lifecycle
payment timing by seller/line
split delivery groups
substitution rules
customer approval thresholds
order versioning
one customer summary / multiple seller execution
```

80 must call or reference:

```text
77 for e-commerce carts/orders
79 for POS-created orders
78 for B2B provider/channel context
81 for locked price/offer
81G for pricing decision/audit reference
82 for dispatch requirement
83 for return/refund lifecycle
Inventory for reservation
Payment for capture/authorization
Customer/Rewards for customer context
```

80 must preserve:

```text
product terms version shown at purchase
pricing decision packet reference
provider responsibility
B2B attribution
delivery promise
dispatch requirement
return/refund terms
customer approval decisions
order version
```

Order’s core role:

```text
Selene may show the customer one clean order summary,
but behind the scenes Order splits the work to each seller/provider.
```

Example:

```text
Customer asks Selene:
milk, bread, shoes, handbag, biscuits.

Customer sees one order summary.

Behind the scenes:
bakery seller order
supermarket seller order
shoe provider seller order
handbag provider seller order
```

---

### Document 81 — Pricing Pack

Document 81 owns:

```text
final price
margin
discount
offer
promotion
brand pricing
B2B profit-share pricing
customer value pricing
service/geography/presentation pricing signals
pricing explanation
pricing audit
```

Sub-documents:

```text
81A — Market Pricing Intelligence + Competitive Research
81B — Dynamic Pricing Optimization
81C — Customer Value Segmentation + Price Sensitivity
81D — Brand Positioning + Premium / Luxury Pricing Guardrail
81E — B2B Profit Share + Commission Pricing Model
81F — Promotion Experimentation + A/B Pricing Governance
81G — Pricing Pack Integration, Explainability, Fairness + Audit
81H — Company Capability, Service-Level, Packaging + Cost-to-Serve
81I — Geography, Delivery Zone, Local Market + Cost-to-Serve
81J — Product Presentation, Merchandising, Perceived Value + Offer Packaging
```

81 must provide to commerce:

```text
final price
offer
discount eligibility
customer benefit eligibility
B2B viability
brand guardrail decision
promotion eligibility
service/capability pricing
geography/delivery cost
presentation readiness
audit/explanation reference
```

Pricing law:

```text
No commerce surface should show a price, offer, benefit, delivery cost, discount, brand claim, B2B customer benefit, or promotion unless the relevant pricing owner has validated it.
```

---

### Document 82 — Dispatch, Packing, Courier Booking + Delivery Network

Document 82 owns:

```text
dispatch eligibility
fulfillment mode
warehouse/bin/source handoff
roster-aware picker/packer assignment
scan-to-pack validation
package identity
label/QR
courier/internal/local agent delivery
tracking
proof of delivery
delivery exceptions
dispatch audit
```

82 must call or reference:

```text
80 for dispatch requirement
Inventory/Warehouse for stock location
81H for capability/capacity
81I for geography/delivery zone
81D for brand packaging/official-channel dispatch
81E for B2B provider responsibility
81J for service/display promises
83 for failed delivery, return-to-sender, courier claims
```

Dispatch law:

```text
82 does not just ship boxes.
82 controls outbound reality with proof.
```

---

### Document 83 — Returns, Refunds + Reverse Logistics

Document 83 owns:

```text
terms-first return/refund decision
cancellation
return
refund
exchange
replacement
repair
warranty
reverse logistics
return courier cost
inspection
provider responsibility
B2B commission reversal
customer benefit reversal
settlement hold/release
dispute fact pack
abuse enforcement signals
return audit
```

83 must call or reference:

```text
80 for order line status
82 for delivery/dispatch proof
77/79 for customer return surfaces
78 for B2B provider responsibility
81E for commission/customer benefit/provider payout reversal
81G for audit/fairness/dispute evidence
81J for terms/presentation evidence
Product for product-level terms
Payment/Accounting/Tax for money and records
```

Returns law:

```text
Selene must check product-level seller/supplier terms and the terms shown at purchase before processing any return/refund/warranty request.
```

No refund adventures without terms. Very boring. Very necessary.

---

## 4. Ownership Boundary Table

```text
Document 77:
Owns shopping/display/cart surfaces.
Does not own final price, order lifecycle, dispatch, or refunds.

Document 78:
Owns B2B provider/channel/attribution.
Does not own final price, dispatch execution, payment execution, or returns decision.

Document 79:
Owns POS execution.
Does not own pricing override authority, refund policy, or dispatch logic.

Document 80:
Owns order group, seller orders, order lines, and order lifecycle.
Does not own physical dispatch or after-sale refund decision.

Document 81:
Owns final price/offer/pricing governance.
Does not own product truth, order truth, dispatch, payment, or returns.

Document 82:
Owns outbound pick/pack/label/courier/tracking/proof.
Does not own order creation, inventory truth, pricing, or returns after return flow begins.

Document 83:
Owns return/refund/exchange/repair/warranty/dispute workflow.
Does not own original order creation, original dispatch, payment execution, accounting posting, or tax law.

Document 84:
Owns relationship map and integration rules.
Does not own runtime execution.
```

---

## 5. End-to-End Normal Commerce Flow

```text
77 / 79
Customer shops, scans, orders, or buys

↓
81 / Pricing Pack
Price, offer, delivery/service cost, brand, promotion, B2B, presentation, and terms display validated

↓
80
Customer order group and seller order/order lines created

↓
82
Pick, pack, package ID, label, courier/local delivery/freight/handover, tracking, proof

↓
83
If issue occurs:
return, refund, exchange, replacement, repair, warranty, dispute, reversal, enforcement

↓
81G / Audit
Everything explainable, versioned, and provable
```

---

## 6. B2B Commerce Flow

```text
78
Provider product enters B2B ecosystem

↓
81D / 81H / 81I
Brand, service capability, and territory checked

↓
81E
B2B price stack calculated:
provider net + commission + Selene fee + reserve + delivery + return courier + benefit pool

↓
81J
B2B display readiness checked

↓
77 / 79
Product displayed in company store / POS / personal Selene if allowed

↓
80
Order creates provider/channel context

↓
82
Original Provider or approved channel dispatches

↓
83
If return/refund/warranty:
Original Provider responsibility + commission reversal + customer benefit reversal + payout hold

↓
81G
Evidence and audit
```

Critical B2B law:

```text
B2B commission does not transfer product responsibility to the Channel Store unless explicitly agreed.
```

The hair salon can earn wine commission. It does not become Wine Court.

---

## 7. POS Commerce Flow

```text
79
Customer scans / cashier scans / restaurant table order / customer device bill

↓
81
Price, benefit, points, discounts, POS offer, payment suggestion validated

↓
80
Order/receipt/order lines created where applicable

↓
Payment
Customer pays or account is charged

↓
82
If physical dispatch/pickup required

↓
83
If return/refund/exchange/warranty occurs

↓
81G
Audit, explanation, dispute evidence
```

POS rule:

```text
POS may execute commerce.
POS may not break pricing, brand, B2B, terms, or return rules.
```

---

## 8. Pricing-to-Commerce Relationship

Every customer-facing commerce action must ask Pricing:

```text
Can this price be shown?
Can this discount be shown?
Can this offer be shown?
Can this B2B benefit be shown?
Can this delivery/service promise be shown?
Can this return term be shown?
Is this brand display allowed?
Is this product presentation good enough?
Is this claim supported?
```

Relevant owners:

```text
81 Core = final price / offer
81D = brand-safe
81E = B2B stack viable
81F = promotion tested/eligible
81G = explain/audit
81H = service capability/cost
81I = delivery/geography cost
81J = product display supports price
```

---

## 9. Terms-Before-Sale Relationship

Before checkout or order confirmation, all commerce surfaces must show or link clear product/service terms.

Terms include:

```text
cancellation terms
return terms
refund terms
exchange terms
replacement terms
repair terms
warranty terms
return courier responsibility
restocking fee if applicable
opened-item rules
custom-made rules
perishable rules
hygiene rules
provider/manufacturer responsibility
service cancellation terms
digital goods terms
```

Documents needing this law:

```text
77 E-Commerce
78 B2B
79 POS
80 Order confirmation
81J Presentation
81G Audit
83 Returns
Product metadata
```

Core law:

```text
83 cannot enforce terms later if Selene cannot prove what terms were shown before purchase.
```

---

## 10. Terms-First Return Relationship

When 83 receives any after-sale request, it must check:

```text
product-level terms
seller/supplier terms
terms version shown at purchase
request reason
delivery proof
customer evidence
seller/provider evidence
mandatory legal/compliance overrides
B2B provider responsibility
payment/settlement hold
return courier responsibility
abuse/fraud signals
```

Only then can 83 decide:

```text
refund
return
exchange
replacement
repair
warranty
reject
exception approval
dispute review
```

---

## 11. Dispatch-Return Relationship

82 and 83 must be tightly connected.

82 gives 83:

```text
pick proof
pack proof
photo proof
package identity
courier
tracking
delivery proof
signature/OTP/ID proof
failed delivery reason
damage/lost/courier claim evidence
return-to-sender status
in-transit intercept result
```

83 uses it for:

```text
refund eligibility
courier fault
provider fault
customer fault
replacement
return courier allocation
commission reversal
provider payout hold
abuse/fraud review
```

If 82 has weak proof, 83 becomes argument theatre.

---

## 12. Product Metadata Relationship

Product must support the entire commerce chain.

Product metadata should include:

```text
product/service identity
variants
configuration options
pricing inputs
B2B eligibility
brand/sub-brand
official-channel rules
presentation/media readiness
returnable yes/no
refund terms
exchange terms
warranty terms
replacement rules
opened-item rules
custom-made rules
perishable rules
hygiene rules
return courier payer
inspection requirement
provider approval required
dispatch handling rules
packing compatibility
contamination rules
serial/batch/expiry requirements
cold-chain requirements
high-value proof requirements
regulated goods restrictions
```

Missing future update:

```text
Product document must be updated to include after-sale terms and dispatch/return handling metadata if not already present.
```

Yes, another limb. At least it has a label this time.

---

## 13. Shared Commerce Data Spine

The following objects must travel across 77–83 without losing context:

```text
CustomerOrderGroup
SellerOrder
OrderLine
ProductTermsVersion
PricingDecisionPacket
B2BAttributionPacket
OriginalProviderPacket
ChannelStorePacket
ProviderResponsibilityPacket
CustomerBenefitPacket
DispatchRequirement
DispatchPackageRecord
PackageIdentity
TrackingRecord
ProofOfDelivery
ReturnRequest
ReturnAuthorization
InspectionResult
RefundDecision
CommissionClawbackSignal
ProviderPayoutHoldSignal
AuditEvidencePacket
```

If these objects are not preserved, the chain breaks.

Enterprise systems love dropping context between modules. Selene must not join that little cult.

---

## 14. Shared ID Requirements

Documents 77–83 must preserve shared IDs.

Required IDs include:

```text
customer_order_group_id
seller_order_id
order_line_id
product_id
variant_id
service_id
seller_id
provider_id
original_provider_id
channel_store_id
pricing_decision_id
product_terms_version_id
dispatch_requirement_id
dispatch_package_id
tracking_id
proof_of_delivery_id
return_request_id
refund_decision_id
inspection_id
commission_clawback_id
audit_evidence_id
```

No handoff should rely only on names, descriptions, or “that order from yesterday.”

That’s how data becomes soup.

---

## 15. Status Mapping Across 77–83

Selene must map statuses consistently.

### Shopping / Cart Status — 77

```text
browsing
cart created
terms displayed
price validated
checkout started
checkout abandoned
order requested
```

### B2B Status — 78

```text
not B2B
B2B eligible
brand approval required
referral-only
direct sale allowed
provider-direct
blocked
```

### POS Status — 79

```text
scanned
bill created
payment pending
paid
receipt issued
POS return requested
```

### Order Status — 80

```text
created
priced
confirmed
split into seller orders
payment pending
ready for dispatch
partially dispatched
delivered
partially returned
closed
```

### Pricing Status — 81

```text
price requested
price validated
offer applied
price locked
price expired
pricing rollback required
```

### Dispatch Status — 82

```text
dispatch required
eligible
picking
packing
labelled
dispatched
in transit
delivered
failed delivery
returned to sender
```

### Return Status — 83

```text
return requested
terms checking
approved
rejected
in return transit
received
inspection pending
refund approved
refund processing
refund completed
closed
disputed
```

84 must ensure status handoffs are consistent and not contradictory.

No “delivered” while Dispatch says “still on shelf.” That kind of thing tends to annoy customers, which is allegedly bad.

---

## 16. Idempotency + Duplicate Prevention

The commerce stack must prevent duplicate execution.

Critical duplicate risks:

```text
duplicate order creation
duplicate payment capture
duplicate dispatch
duplicate label printing
duplicate courier booking
duplicate refund
duplicate replacement
duplicate commission payout
duplicate customer benefit
duplicate return authorization
duplicate provider payout reversal
```

Required controls:

```text
idempotency key per action
single source of truth per state transition
duplicate detection
safe retry
audit of retry
manual override authority
```

Example:

```text
Customer clicks checkout twice.
Selene must not create two orders and send two cakes to mum.
Mum may like cake, but finance will not.
```

---

## 17. Data Walls + Privacy Boundaries

Commerce data must be shared only with the right parties.

### Customer sees

```text
product
price
offer
terms
delivery promise
tracking
return/refund status
customer-safe explanations
```

### Seller/provider sees

```text
orders they are responsible for
items they must fulfill
return/warranty claims against them
customer delivery info needed for fulfillment
```

### Channel Store sees

```text
attribution
commission where allowed
customer relationship context where allowed
not provider-private economics unless allowed
```

### Provider does not see

```text
unrelated customer history
private customer preferences not needed
other sellers’ order lines
full business margin where not allowed
```

### Customer does not see

```text
B2B provider net
channel commission
Selene fee
internal reserves
risk score
business margin
```

Privacy rule:

```text
Share only what the party needs to perform its role.
```

Because “everyone sees everything” is not transparency. It is a data breach in a party hat.

---

## 18. Customer-Facing Truth Law

Selene must not show customers unsupported promises.

Customer-facing truth applies to:

```text
price
discount
offer
stock
delivery promise
return eligibility
warranty route
brand/official status
seller/provider identity
B2B benefit
promotion terms
service promise
delivery tracking
refund timing
```

Rule:

```text
If Selene cannot support it operationally, legally, financially, or evidentially, Selene must not show it as true.
```

No fake delivery. No fake discount. No fake warranty. No fake official badge. No fake “only 2 left.” No fake happy little UI sticker.

---

## 19. Human / External Action Relationship

Across 77–83, anytime a human or external party must act, Selene must use:

```text
Task
Scheduler / Roster
Broadcast / Delivery
Reminder
Access / Authority
Audit
```

Examples:

```text
supplier approves exception refund
picker picks order
packer uploads photo proof
brand owner approves seller
courier confirms pickup
provider responds to warranty claim
customer uploads damage photo
Selene Services reviews dispute
agent accepts local delivery
warehouse inspects return
```

Every action needs:

```text
owner
recipient
deadline
confirmation
evidence
reminder
escalation
closure
audit
```

No floating “notify supplier” nonsense.

If a human must do it, Selene must assign it, schedule it, chase it, prove it, and close it.

---

## 20. Financial/Reversal Relationship

Commerce must preserve financial consequences.

Events that may affect money:

```text
order confirmed
payment captured
seller order created
B2B commission earned
customer benefit issued
dispatch completed
delivery confirmed
return requested
refund approved
replacement shipped
provider payout released
commission clawback
customer benefit reversal
return courier charged
reserve used
credit note issued
chargeback received
```

Documents involved:

```text
80 Order
81 Pricing
81E B2B pricing/reversal
82 Dispatch cost/proof
83 Return/refund/reversal
Payment/Settlement
Accounting/Tax
AP/Creditors
81G Audit
```

Rule:

```text
No refund, payout, commission, customer benefit, or reserve release should happen without a matching order/dispatch/return/audit context.
```

Money without context is how finance develops eye twitching.

---

## 21. Audit and Evidence Flow

Every critical commerce decision must produce evidence.

Evidence includes:

```text
terms shown before purchase
pricing decision packet
B2B approval
brand approval
dispatch proof
delivery proof
return request
customer evidence
provider evidence
inspection result
refund decision
commission reversal
payout hold
customer communication
human approval
exception reason
```

81G is the audit/explanation owner.

Documents 77–83 must feed it.

Rule:

```text
If Selene cannot prove it later, Selene should not pretend it happened cleanly.
```

---

## 22. Failure and Exception Handling

Document 84 must define common failure classes.

Failures include:

```text
price invalid
terms missing
brand approval missing
B2B approval missing
payment failed
stock unavailable
address invalid
dispatch delayed
courier failed
delivery failed
return evidence missing
provider no-response
customer no-response
inspection dispute
refund failed
commission reversal failed
audit evidence missing
```

Each failure needs:

```text
owner
resolution path
customer communication
business communication
deadline
escalation
audit
```

No silent failures. Silent failures become loud customers.

---

## 23. Retry and Recovery Rules

Some actions can be retried.

Retry-safe examples:

```text
send customer notification
fetch courier tracking
reprint label after voiding old label
request provider response
retry payment refund where Payment allows
retry courier quote
retry address validation
```

Dangerous retries:

```text
payment capture
refund execution
provider payout
commission payout
customer benefit issuance
replacement dispatch
return label generation
```

Dangerous retries require idempotency keys and confirmation.

Selene must not create a second refund because an API hiccup got dramatic.

---

## 24. Cross-Document Required Updates

### Required update to Document 77

Document 77 must include:

```text
display product-level return/refund/warranty terms before purchase
show terms version / evidence link to Order/83
respect B2B brand/channel approval before display
show only funded B2B customer benefits
show delivery availability from 81I/82
show service capability from 81H
show presentation-ready products from 81J
create order through 80, not directly dispatch
route return/refund requests to 83
```

---

### Required update to Document 78

Document 78 must include:

```text
B2B seller requests brand approval before direct sale
referral-only/direct-sale/provider-direct route support
provider remains responsible for product/warranty unless contract says otherwise
B2B return/refund/warranty flows to 83
commission/customer benefit/payout reversal flows through 81E/83
service capability checks use 81H
territory checks use 81I
display readiness uses 81J
audit uses 81G
```

---

### Required update to Document 79

Document 79 must include:

```text
POS must show return/refund/warranty terms before checkout
POS must not manually override brand/B2B/pricing/refund rules without authority
POS returns must flow to 83
POS dispatch/pickup must flow to 82
POS promotions must be validated by 81F
POS customer explanations must use 81G where needed
```

---

### Required update to Document 80

Document 80 must include:

```text
order lines preserve pricing, provider, terms, delivery, B2B, dispatch, and return context
order preserves product terms version shown at purchase
order creates dispatch requirements for 82
order receives dispatch/delivery status from 82
order receives return/refund status from 83
order supports partial returns/refunds at line level
order must not confirm unsupported delivery promises
order must not confirm B2B/brand-restricted route unless approved
```

---

### Required update to Document 81

Document 81 must include:

```text
pricing receives dispatch cost actuals from 82
pricing receives return/refund/reversal cost signals from 83
81E supports return courier and commission reversal from 83
81F supports promotion refund/reversal learning from 83
81G audits price/offer/terms evidence
81H/81I feed dispatch/service cost into pricing
81J ensures product terms and claims are clearly displayed
```

---

### Required update to Document 82

Document 82 must include:

```text
preserve terms/provider/B2B context from Order
pass delivery proof and dispatch proof to 83
support in-transit intercept and return-to-sender
support local agent/employee delivery with availability/terms
support multi-address/multi-recipient dispatch
use roster-aware picking/packing tasks
enforce product compatibility/contamination rules
preserve package identity and proof
```

---

### Required update to Document 83

Document 83 must include:

```text
terms-first return/refund decision
terms shown at purchase check
seller/supplier exception approval
dispute fact pack
orphan warranty/provider failure protection
customer/supplier abuse enforcement signals
deadlines for all parties
B2B commission/customer benefit/payout reversal
return courier cost allocation
```

---

### Required update to Product

Product must include:

```text
after-sale terms
dispatch handling metadata
return handling metadata
packing compatibility
contamination rules
serial/batch/expiry requirements
brand/official-channel rules
B2B eligibility
presentation readiness
warranty/provider responsibility
```

---

## 25. Future / Missing Engines Register

Document 84 must record future engines needed or likely needed.

Future engines:

```text
Participant Trust, Abuse, Enforcement + Disqualification Engine
Courier / Delivery Provider Registry Engine
Warehouse Location / Bin / Slot Optimization Engine
Product Terms + After-Sale Policy Metadata Engine
Warranty Reserve / Protection Fund Engine
Dispute Resolution / Selene Services Review Engine
Brand Approval Workflow Engine
Local Agent / Community Delivery Network Engine
Freight / Customs / Container Logistics Engine
Courier Claim / Insurance Recovery Engine
```

Some can begin as sections inside 78/82/83.

Later they may deserve standalone engines.

Architecture warning:

```text
Do not let Returns become the abuse enforcement engine for all of Selene.
Do not let Dispatch become the courier network engine for all countries forever.
Do not let B2B become brand approval, service capability, pricing, settlement, and dispute court all at once.
```

That way lies document obesity. We are already flirting with it.

---

## 26. Master Commerce Laws

### Law 1 — No Commerce Without Terms

```text
No product/service should be sold unless its cancellation, return, refund, exchange, warranty, replacement, and return courier terms are known or explicitly marked as manual-review required.
```

### Law 2 — No Dispatch Without Proof Path

```text
No item should dispatch unless package identity, label, delivery method, and proof requirements are defined.
```

### Law 3 — No B2B Sale Without Provider Responsibility

```text
Every B2B sale must preserve Original Provider, Channel Store, commission, customer benefit, reserve, warranty, and return responsibility.
```

### Law 4 — No Human Action Without Task

```text
Any human or external action must become a scheduled, delivered, confirmed, reminded, escalated, audited task.
```

### Law 5 — No Refund Without Terms + Evidence

```text
Refund decisions must check product terms, terms shown to customer, request reason, evidence, provider responsibility, and compliance override.
```

### Law 6 — No Price Without Audit

```text
Every price, offer, benefit, promotion, dispatch cost, return cost, and refund effect must be explainable and auditable.
```

### Law 7 — No Customer Lie

```text
Selene must not show delivery, return, warranty, offer, price, brand, stock, or service promises that the system cannot support.
```

### Law 8 — No Duplicate Money Movement

```text
Payment captures, refunds, provider payouts, commission payouts, customer benefits, reserves, and reversals must be idempotent and traceable.
```

### Law 9 — No Context Loss

```text
Every handoff across 77–83 must preserve product, customer, provider, pricing, terms, B2B, delivery, and audit context.
```

### Law 10 — No Unowned Exception

```text
Every failure, dispute, overdue response, or exception must have an owner, deadline, escalation path, and closure condition.
```

Beautifully boring. Correct systems are boring until they save a fortune.

---

## 27. Grand Flow — Customer Buys B2B Product From Another Store

```text
77 displays product in Company A store
↓
78 confirms product is B2B and Company A may display/sell/referral-only
↓
81D checks brand/channel
↓
81H checks service capability
↓
81I checks territory/geography
↓
81E calculates B2B economics
↓
81J confirms presentation
↓
81G records evidence
↓
80 creates customer order + provider seller order
↓
82 dispatches via Original Provider or approved route
↓
83 handles return/warranty if issue occurs
↓
81E reverses commission/benefits if refunded
↓
Payment/Accounting/Tax receive final signals
↓
81G proves everything
```

---

## 28. Grand Flow — Customer Buys 100 Gifts to 100 Addresses

```text
77 collects gift order
↓
81 prices gift packaging/delivery/benefits
↓
83/Product terms shown before purchase
↓
80 creates one customer order group with 100 recipient delivery groups
↓
82 creates 100 package identities, labels, tracking records
↓
82 handles delivery proof per recipient
↓
83 handles any returns/refunds per recipient/order line
↓
81G audits terms, display, delivery, and refund evidence
```

---

## 29. Grand Flow — Return Dispute

```text
83 receives claim
↓
83 loads product terms + terms shown at purchase
↓
83 collects customer evidence
↓
83 pulls dispatch proof from 82
↓
83 requests seller/provider evidence
↓
83 creates dispute fact pack
↓
83 routes human review if needed
↓
83 decides refund/replacement/repair/rejection/escalation
↓
81E adjusts commission/benefits/provider payout if B2B
↓
Payment/Accounting/Tax receive instructions
↓
81G stores full audit
```

---

## 30. Grand Flow — POS Scan-As-You-Go With Dispatch

```text
79 customer scans goods / cashier scans
↓
81 validates price/offer/payment suggestion
↓
83/Product terms available before sale
↓
Payment captures payment
↓
80 creates order/receipt lines
↓
82 dispatches if goods are delivery/pickup rather than immediate carry-out
↓
83 handles return/refund through terms-first process
↓
81G audits transaction and terms
```

---

## 31. Grand Flow — In-Transit Cancellation

```text
customer requests cancellation
↓
80 checks order state
↓
82 checks dispatch state:
label only / with courier / in transit / out for delivery
↓
82 attempts cancel/intercept/reroute/return-to-sender if possible
↓
83 determines refund/return consequence and cost allocation
↓
81E reverses B2B economics if applicable
↓
Payment/Accounting process final money effects
↓
81G audits outcome
```

---

## 32. Grand Flow — Provider Failure / Orphan Warranty

```text
customer opens warranty claim
↓
83 checks warranty terms
↓
provider/supplier unavailable, suspended, insolvent, or refuses valid claim
↓
83 checks reserve/deposit/manufacturer/brand/Selene protection policy
↓
Selene resolves via refund/replacement/repair/brand route where policy allows
↓
provider enforcement signal created
↓
81E/Payment/Accounting adjust reserves and payout
↓
81G stores evidence
```

---

## 33. Cross-Engine Human Task Examples

Examples of human tasks that must be owned and scheduled:

```text
brand owner approves B2B seller
provider approves refund exception
warehouse picker picks order
packer uploads photo proof
courier confirms pickup
recipient confirms availability
seller responds to dispute
inspector grades return
Selene Services reviews dispute
compliance reviews legal override
accounting reviews credit note
```

Every task must have:

```text
owner
deadline
evidence
reminder
escalation
closure
audit
```

No “someone should.” Someone is imaginary until assigned.

---

## 34. Customer Communication Consistency

Customer should receive coherent communication across lifecycle:

```text
before purchase:
price, terms, delivery, warranty, return rules

after order:
order summary, seller/provider split if needed, delivery promise

during dispatch:
picked/packed/shipped/tracking/out-for-delivery/delivered

during issue:
return eligibility, evidence required, label, inspection, refund timing

after resolution:
refund/replacement/repair/exchange/denial explanation
```

Customer communication owners:

```text
77 / 79 = before/during purchase
80 = order status
82 = dispatch/delivery status
83 = return/refund status
81G = explanation/audit wording
Broadcast/Delivery = actual message sending
```

---

## 35. Business Communication Consistency

Businesses/providers should receive:

```text
order task
dispatch task
B2B attribution
brand approval task
return request
exception approval task
dispute fact pack
inspection result
refund decision
commission reversal
provider payout hold
risk/enforcement signal
```

Each communication must be:

```text
role-appropriate
data-minimized
actionable
deadline-based
auditable
```

---

## 36. Reconciliation Across 77–83

Selene must reconcile:

```text
orders created vs payments captured
orders confirmed vs dispatches created
dispatches created vs packages shipped
packages shipped vs packages delivered
delivered orders vs settlement releases
returns requested vs refunds issued
refunds issued vs provider payout reversals
B2B commission earned vs commission released
customer benefits issued vs benefits reversed
terms shown vs terms enforced
```

Reconciliation failures create:

```text
exception
owner
deadline
audit incident
```

This is the part where the machine checks its own pockets for missing money. Very healthy.

---

## 37. Required Simulations

```text
customer buys simple product through 77 and receives delivery through 82
customer buys through POS 79 and returns through 83
B2B product displayed in Company A store with Original Provider preserved
brand-sensitive B2B product requires approval before display
customer sees clear return terms before checkout
terms version shown at purchase is later used by 83
order splits into multiple seller orders and multiple dispatches
one customer sends 100 gifts to 100 addresses
multi-seller order has one customer summary but multiple provider dispatches
dispatch proof from 82 resolves return dispute in 83
wrong item packed creates dispatch proof mismatch and refund decision
B2B refund reverses channel commission and customer benefit
provider no-response triggers payout hold and SLA escalation
customer no-response closes return request after reminders
in-transit cancellation attempts courier intercept
return courier cost allocated to provider for wrong item
return courier cost allocated to customer for change-of-mind
orphan warranty uses provider reserve/protection fund
customer abuse creates enforcement handoff
supplier unsafe goods creates severe provider enforcement handoff
duplicate refund attempt is blocked by idempotency
private B2B economics are hidden from customer display
81G decision replay reconstructs full transaction
```

---

## 38. Integration Map

```text
PH1.COMMERCE_INTEGRATION / DOCUMENT_84
↔ PH1.ECOMMERCE / DOCUMENT_77
↔ PH1.B2B_PLATFORM / DOCUMENT_78
↔ PH1.POS / DOCUMENT_79
↔ PH1.ORDER / DOCUMENT_80
↔ PH1.PRICING / DOCUMENT_81
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.DISPATCH / DOCUMENT_82
↔ PH1.RETURNS / DOCUMENT_83
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.WAREHOUSE
↔ PH1.RECEIVING
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.AP_CREDITORS
↔ PH1.SUPPLIER_PAYMENT_HANDOFF
↔ PH1.SUPPLIER_STATEMENT_RECONCILIATION
↔ PH1.CUSTOMER
↔ PH1.REWARDS
↔ PH1.COURIER / DELIVERY_PROVIDER
↔ PH1.WARRANTY
↔ PH1.LEGAL
↔ PH1.COMPLIANCE
↔ PH1.ACCESS / AUTHORITY
↔ PH1.TASK / HUMAN_WORKLOAD
↔ PH1.SCHEDULER / ROSTERS
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
↔ FUTURE PH1.PARTICIPANT_TRUST_ABUSE_ENFORCEMENT
↔ FUTURE PH1.COURIER_PROVIDER_REGISTRY
↔ FUTURE PH1.PRODUCT_TERMS_POLICY_METADATA
↔ FUTURE PH1.SELENE_SERVICES_DISPUTE_REVIEW
```

---

## 39. Required Logical Packets

```text
CommerceIntegrationPacket
CommerceOwnershipBoundaryPacket
CommerceLifecyclePacket
CommerceStatusMappingPacket
CommerceContextPacket
CommerceIdempotencyPacket
CommerceDataWallPacket
ProductTermsPropagationPacket
TermsDisplayEvidencePacket
PricingDecisionReferencePacket
B2BAttributionPropagationPacket
ProviderResponsibilityPacket
OrderDispatchHandoffPacket
DispatchReturnEvidencePacket
ReturnRefundReversalPacket
CustomerCommunicationPacket
BusinessCommunicationPacket
HumanActionOrchestrationPacket
CommerceAuditEvidencePacket
CommerceReconciliationPacket
CommerceExceptionPacket
CommerceFutureEngineRegisterPacket
CommerceSimulationPacket
```

Logical only.

No runtime packet structs. The integration goblin may keep its diagrams on paper.

---

## 40. State Machines

### Commerce Lifecycle State

```text
Shopping
Cart
Checkout
Priced
TermsDisplayed
OrderCreated
OrderConfirmed
DispatchRequired
Dispatched
Delivered
ReturnRequested
Refunded
Replaced
Repaired
Disputed
Closed
```

### Cross-Document Handoff State

```text
NotRequired
Required
ContextPrepared
Sent
Accepted
Rejected
MissingData
RetryRequired
Completed
Audited
Closed
```

### Commerce Exception State

```text
NoException
Detected
OwnerAssigned
CustomerNotified
BusinessNotified
ActionRequired
Escalated
Resolved
Closed
```

### Commerce Reconciliation State

```text
NotStarted
Checking
Matched
MismatchDetected
InvestigationRequired
Corrected
Audited
Closed
```

### Terms Evidence State

```text
NotRequired
Required
TermsLoaded
TermsShown
TermsVersionStored
TermsEvidenceMissing
ManualReviewRequired
Closed
```

---

## 41. Reason Codes

```text
COMMERCE_INTEGRATION_CONTEXT_CREATED
DOCUMENT_77_ECOMMERCE_HANDOFF_VALIDATED
DOCUMENT_78_B2B_HANDOFF_VALIDATED
DOCUMENT_79_POS_HANDOFF_VALIDATED
DOCUMENT_80_ORDER_HANDOFF_VALIDATED
DOCUMENT_81_PRICING_HANDOFF_VALIDATED
DOCUMENT_82_DISPATCH_HANDOFF_VALIDATED
DOCUMENT_83_RETURNS_HANDOFF_VALIDATED
PRODUCT_TERMS_REQUIRED
PRODUCT_TERMS_VERSION_CAPTURED
TERMS_DISPLAY_EVIDENCE_REQUIRED
PRICING_DECISION_REFERENCE_REQUIRED
B2B_ATTRIBUTION_REQUIRED
ORIGINAL_PROVIDER_PRESERVED
CHANNEL_STORE_PRESERVED
PROVIDER_RESPONSIBILITY_PRESERVED
DISPATCH_REQUIREMENT_CREATED
DISPATCH_PROOF_LINKED_TO_RETURN
RETURN_REFUND_REVERSAL_REQUIRED
COMMISSION_CLAWBACK_CONTEXT_REQUIRED
CUSTOMER_BENEFIT_REVERSAL_CONTEXT_REQUIRED
HUMAN_ACTION_TASK_REQUIRED
DATA_WALL_APPLIED
CUSTOMER_FACING_TRUTH_CHECK_REQUIRED
IDEMPOTENCY_KEY_REQUIRED
DUPLICATE_ACTION_BLOCKED
RECONCILIATION_MISMATCH_DETECTED
AUDIT_EVIDENCE_LINKED
FUTURE_ENGINE_REQUIRED
COMMERCE_EXCEPTION_CREATED
```

---

## 42. What Codex Must Not Do

```text
Do not make Document 84 own execution of 77–83.
Do not make Document 84 own runtime code.
Do not create APIs from this document.
Do not create packet structs from this document.
Do not treat 84 as replacing 77–83.
Do not move ownership from the source documents into 84.
Do not mark missing future engines as built.
Do not ignore terms-before-sale law.
Do not ignore B2B provider responsibility.
Do not ignore dispatch proof handoff to returns.
Do not ignore refund/commission/customer benefit reversals.
Do not ignore human/external task orchestration.
Do not ignore audit/evidence requirements.
Do not let GPT-5.5 invent pricing, terms, proof, dispatch state, refund eligibility, warranty coverage, or legal conclusions.
Do not create runtime code from this document.
Do not implement from this document alone.
```

---

## 43. Final Architecture Sentence

Selene Commerce Integration Overview + Relationship Master Map is the global architecture document that connects Documents 77–83 into one governed commerce machine by defining ownership boundaries, handoff rules, shared IDs, status mapping, terms-before-sale requirements, B2B provider responsibility, pricing validation, order context preservation, dispatch proof, return/refund/reversal flow, customer/business communication, human/external task orchestration, idempotency, data walls, reconciliation, audit evidence, and future engine gaps; ensuring E-Commerce, B2B, POS, Order, Pricing, Dispatch, and Returns work together with precision instead of behaving like seven impressive but confused departments arguing over who lost the box.

Simple version:

```text
77 sells.
78 connects businesses.
79 executes POS.
80 creates and controls orders.
81 prices.
82 dispatches.
83 resolves after-sale problems.
84 wires them together.

84 makes sure terms, pricing, provider responsibility, delivery proof, return rules, human tasks, money effects, and audit evidence travel through the whole system without getting dropped in the corridor.
```

That is Document 84: the belt system that makes the commerce machine work instead of turning Selene into “order confirmed, package missing, refund disputed, supplier vanished, customer furious, and Dave says he thought someone else handled it.”

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
