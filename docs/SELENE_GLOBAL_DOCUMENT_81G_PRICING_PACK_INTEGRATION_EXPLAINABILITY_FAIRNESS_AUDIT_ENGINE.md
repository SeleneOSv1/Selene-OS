# Global Document 81G — Selene Pricing Pack Integration, Explainability, Fairness + Audit Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE SUB-DOCUMENT / PRICING PACK ADDENDUM

PARENT DOCUMENT:
Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

SUB-DOCUMENT:
81G

ENGINE:
PH1.PRICING.81G / PH1.PRICING_EXPLAINABILITY / PH1.PRICING_AUDIT / PH1.PRICING_FAIRNESS

FULL NAME:
Selene Pricing Pack Integration, Pricing Decision Trace, Explainability, Fairness, Compliance Handoff, Audit Evidence, Versioning, Approval Trace, Pricing Incident, Rollback Audit, Customer/Business Explanation, Decision Replay, Pricing Governance Change Control, and Evidence Report Pack Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_PRICING_PACK_SUB_DOCUMENT
```

---

## 1. Purpose

Document 81G is Selene’s **pricing proof, explanation, fairness, integration, and audit engine**.

It answers:

```text
Why is this the price?
What data was used?
Which pricing engines contributed?
Was it fair?
Was it allowed?
Was it approved?
What did the customer see?
What version of the rule applied?
Can Selene prove it later?
```

81G does **not** set the final price.

```text
81G = explains, validates, audits, and proves.
81 Core = final pricing decision.
```

Simple version:

```text
81G stops pricing from becoming:
“because the algorithm said so.”
```

That phrase is just corporate astrology with a subscription fee.

---

## 2. Core 81G Law

```text
Every Selene pricing decision must be explainable, versioned, confidence-scored, fairness-checked, audit-backed, and replayable.

If Selene cannot explain, prove, or replay the decision, the decision is not clean.
```

81G must ensure every pricing decision has:

```text
pricing decision packet
source data references
engine contribution trace
rule/version references
approval trace if required
customer-facing explanation where needed
business-facing explanation where needed
audit evidence
fairness/compliance flags
confidence score
time validity
rollback path if applicable
```

No mystery pricing.

No secret spreadsheet goblins.

No “the system did it” nonsense. The system has a name, a reason, a timestamp, and hopefully a little shame.

---

## 3. Relationship to Document 81 Core

Document 81 Core owns:

```text
final pricing decision
final discount decision
final offer decision
margin guardrail
approval routing
price publication
pricing decision packet creation
```

Document 81G owns:

```text
pricing decision completeness validation
pricing pack integration trace
customer explanation
business explanation
audit explanation
compliance/fairness review signal
approval trace
versioning
data lineage
confidence scoring
pricing evidence locker
pricing incident audit
rollback audit
decision replay
pricing governance dashboard
audit/evidence report packs
```

Simple split:

```text
81 decides.
81G proves and explains.
```

---

## 4. Pricing Pack Integration Role

81G connects and validates the full Pricing Pack.

It must receive or reference signals from:

```text
81A — Market Pricing Intelligence + Competitive Research
81B — Dynamic Pricing Optimization
81C — Customer Value Segmentation + Price Sensitivity
81D — Brand Positioning + Premium / Luxury Pricing Guardrail
81E — B2B Profit Share + Commission Pricing Model
81F — Promotion Experimentation + A/B Pricing Governance
81H — Company Capability, Service-Level, Packaging + Cost-to-Serve Pricing
81I — Geography, Delivery Zone, Local Market + Cost-to-Serve Pricing
81J — Product Presentation, Merchandising, Perceived Value + Offer Packaging
```

81G must identify:

```text
which engines contributed
which engines were not required
which engines were missing
which engine signals conflicted
which signals were low confidence
which final rule won
```

81G does not become the pricing brain.

It becomes the pricing court reporter, referee, historian, compliance goblin, and receipt folder. Horrible job. Necessary.

---

## 5. Engine Ownership Boundary

### 5.1 81G owns

```text
pricing decision completeness check
pricing decision packet validation
pricing explanation generation
customer/business/audit explanation separation
pricing fairness check
sensitive attribute exclusion proof
personalization governance trace
pricing conflict identification
pricing priority hierarchy trace
approval trace
pricing versioning
time validity validation
data lineage
data confidence aggregation
evidence locker
customer pricing dispute support
business pricing dispute support
rollback audit
pricing incident management
cross-channel audit
customer-facing price claim evidence
promotion audit integration
B2B audit integration
brand audit integration
dynamic pricing audit integration
market audit integration
customer value audit integration
compliance handoff
privacy/data minimization review
explainable AI guardrail
audit immutability requirement
retention/archive flagging
decision replay
missing-data behavior
fairness monitoring over time
pricing governance dashboard
pricing governance change control
audit export / evidence report pack
```

### 5.2 81G references but does not own

```text
final price decision
final discount approval
final offer publication
product master truth
inventory stock truth
market data collection
customer memory master truth
brand policy ownership
B2B settlement execution
payment execution
refund execution
tax law
accounting ledger posting
legal/compliance final opinion
```

### 5.3 Correct owner split

```text
81 Core = final pricing decision.
81G = proof, explanation, fairness, audit, decision replay.
Audit Engine = global audit infrastructure.
Legal/Compliance = legal interpretation and compliance approval where required.
Accounting/Tax = financial and tax posting/treatment.
GPT-5.5 = human-readable explanation only from verified facts.
```

---

## 6. Pricing Decision Completeness Check

81G must verify that required pricing inputs exist before a pricing decision is considered clean.

Required inputs may include:

```text
product/service identity
source cost
landed cost
true cost-to-serve
capital/carrying/finance cost where applicable
market range
customer value signal
brand guardrail signal
B2B stack if applicable
company capability signal
geography/delivery-zone signal
presentation/perceived-value signal
return/warranty risk
tax/duty basis
discount rules
price lock
contract/account rule
promotion result if applicable
approval rule
audit evidence
```

If required data is missing, 81G may signal:

```text
block pricing decision
use safe default
route review
mark low confidence
prevent customer-facing claim
prevent dynamic publication
```

No “final price because vibes.” That belongs in horoscopes and budget forecasts.

---

## 7. Pricing Decision Packet Validation

81G validates the Pricing Decision Packet from Document 81 Core.

It checks:

```text
pricing decision ID
who requested price
what product/service
which customer/customer segment
which channel
which seller/provider
which order/cart/session
which engines contributed
which inputs were used
which rules applied
which rules blocked action
which approval happened
which version applies
customer explanation generated
business explanation generated
audit reference created
```

If packet is incomplete:

```text
return to 81 Core
route missing-data action
mark low confidence
block publication if risk is material
```

The packet is the pricing black box recorder. Preferably before the crash, not after.

---

## 8. Explanation Types

81G must generate different explanations for different audiences.

### 8.1 Customer explanation

Customer-safe explanation may include:

```text
final price
discount applied
offer applied
delivery fee
tax/duty estimate
points/credits used
offer expiry
account price if applicable
refund/return price basis where relevant
```

Customer should not see:

```text
business margin
provider net
commission waterfall
internal risk score
private customer segmentation logic
sensitive inference
```

### 8.2 Business explanation

Business explanation may include:

```text
source cost
landed cost
margin
market range
brand rule
B2B stack
customer value logic
promotion result
risk cost
capital/carrying cost
discount impact
approval reason
data confidence
```

### 8.3 Internal / audit explanation

Internal audit explanation includes:

```text
full decision trace
inputs
engine signals
rules
owners
approvals
timestamps
version IDs
confidence scores
exceptions
evidence references
```

### 8.4 Legal / compliance explanation

Compliance explanation includes:

```text
fairness flags
personalization logic
sensitive-signal exclusion
pricing claim evidence
disclosure logic
customer-facing representation
approval trail
regulated-pricing review signals
```

---

## 9. Explainability Depth Control

Not every party may see the same explanation.

Access depends on role:

```text
customer sees customer-safe explanation
seller sees seller-relevant explanation
provider sees provider-relevant explanation
channel store sees channel-relevant economics only if allowed
business owner sees margin and pricing logic
auditor sees full trace
compliance sees fairness/legal flags
tax/accounting sees financial/tax basis
support sees customer-safe dispute explanation
```

Rule:

```text
Explain enough to be useful.
Do not expose more than the role is allowed to see.
```

The customer does not need the B2B profit-share waterfall. They asked for shoes, not a financial autopsy.

---

## 10. Fairness Guardrail

81G must check whether a pricing decision is fair and defensible.

Checks:

```text
no unfair discrimination
no protected attribute use
no hidden exploitative pricing
no fake urgency
no fake discount
no misleading “was” price
no hidden mandatory fees
no unsupported “best price” claim
no unfair personalized base price
no contract/customer price lock violation
```

If fairness risk exists:

```text
flag compliance review
block or hold publication where required
route approval
record evidence
```

Profit without trust becomes churn with receipts.

---

## 11. Sensitive Attribute Exclusion

81G must ensure pricing does not improperly use protected or sensitive traits.

Do not use improper signals such as:

```text
race
religion
health status
disability
gender
protected age category
sensitive personal data
protected vulnerability
```

Permitted commercial signals may include:

```text
declared preferences
purchase behavior
loyalty tier
account terms
contract price
product category behavior
service preference
delivery preference
payment preference
business relationship
```

If sensitive-signal risk appears:

```text
exclude signal
record exclusion
route compliance review where required
audit the decision
```

---

## 12. Personalization Governance

81G must separate acceptable personalization from high-risk personalization.

Preferred personalization:

```text
personalized offer
personalized reward
personalized service
personalized package
personalized payment option
personalized delivery option
```

High-risk personalization:

```text
hidden personalized base price
charging more because customer appears wealthy
charging more based on protected/sensitive traits
opaque price manipulation
exploiting urgency/vulnerability
```

Preferred Selene model:

```text
same transparent base price where appropriate
personalized value and benefit around it
```

Risky model:

```text
secretly different base price per person
```

That second path is where trust goes to die wearing a recommendation badge.

---

## 13. Conflict Resolution Between Pricing Engines

81G must detect and report conflicts between pricing engines.

Examples:

```text
81A says market price is lower
81B says raise price due to demand
81C says customer wants discount
81D says brand blocks discount
81E says B2B stack not viable
81F says promotion works but hurts brand
81H says service level supports premium
81I says delivery zone cost is high
81J says presentation does not support premium price
```

81G must not decide final price by itself.

It sends conflict to Document 81 Core with:

```text
conflict summary
affected engines
risk level
recommended resolution path
approval requirement
missing evidence
audit reference
```

---

## 14. Pricing Priority Hierarchy

81G must help enforce decision priority.

Example hierarchy:

```text
law / compliance
contract / account price lock
brand / official-channel rules
customer fairness / sensitive data guardrail
margin floor / price floor
B2B viability
tax / duty / all-in price disclosure
customer value strategy
market competitiveness
promotion optimization
```

Example:

```text
Customer wants discount.
Brand blocks public discount.
Margin floor blocks price cut.
Selene recommends loyalty benefit or VIP service instead.
```

Priority hierarchy prevents the loudest pricing signal from winning. This is useful because discounts are loud little gremlins.

---

## 15. Approval Trace

81G must prove approvals.

Approval trace includes:

```text
who requested
who approved
approval time
approval reason
approval scope
one-time or permanent
expiry
conditions
evidence
audit reference
```

Approval-required examples:

```text
below-margin price
brand-sensitive discount
B2B commission exception
manual override
customer price lock exception
dynamic pricing change above threshold
fairness/compliance risk
pricing rollback
```

If approval is required, it must follow Human / External Action Orchestration.

No “manager approved” with no manager. That is folklore with an invoice.

---

## 16. Human / External Action Orchestration

81G must apply the Selene Human / External Action Orchestration Law.

Actions may include:

```text
pricing review required
brand approval required
B2B provider approval required
compliance review required
manager override required
customer clarification required
audit evidence request
rollback approval
pricing incident review
```

Every action must define:

```text
owner
recipient
deadline
delivery method
required confirmation
required evidence
reminder rule
escalation path
closure condition
audit reference
```

No “notify finance” swamp phrases. Assign it. Send it. Chase it. Prove it.

---

## 17. Versioning

81G must version pricing rules and decisions.

Versioned items:

```text
price version
discount version
offer version
promotion version
brand policy version
B2B pricing stack version
customer segment version
market data version
tax/duty version
approval version
pricing rule version
fairness rule version
explanation template version
```

Selene must be able to answer:

```text
What rule was active when this customer bought?
```

That question will arrive. It always does. Usually from someone angry.

---

## 18. Time Validity

81G must validate time-based rules.

Checks:

```text
price still valid
quote still valid
cart price not expired
promotion not expired
account price lock active
B2B price still active
brand policy still current
market data not stale
approval not expired
dynamic pricing window still active
event/holiday price window still active
```

If stale:

```text
refresh
block
route review
show updated price
honor lock if applicable
```

---

## 19. Data Lineage

81G tracks where every important input came from.

Sources may include:

```text
Product
Inventory
Supplier
Procurement
Market engine
Customer engine
B2B engine
Brand policy
Promotion test
Cashflow
Debt / Treasury
Tax
Accounting
Manual approval
External source
```

Lineage must include:

```text
source engine
source record
timestamp
version
confidence
evidence reference
```

No mystery data. Mystery data grows into lawsuits and dashboards.

---

## 20. Data Confidence Scoring

81G aggregates confidence from all pricing sources.

Confidence factors:

```text
market data freshness
customer segment confidence
cost certainty
delivery cost certainty
capital cost certainty
return risk confidence
B2B stack completeness
brand policy confidence
tax/duty confidence
promotion result confidence
manual input reliability
```

Outputs:

```text
high confidence
medium confidence
low confidence
insufficient data
manual review required
```

Low confidence may trigger:

```text
block
safe default
limited test
manual review
customer-safe disclosure
```

Selene should know when she is guessing. That alone puts her ahead of several boardrooms.

---

## 21. Evidence Locker

81G must maintain pricing evidence.

Evidence may include:

```text
pricing decision packet
market screenshots/references
supplier cost records
PO cost reference
inventory cost reference
capital cost reference
B2B pricing waterfall
approval logs
customer-facing price display
terms shown
promotion rules
refund/reversal logic
price lock agreement
brand policy reference
tax/duty basis
customer explanation shown
business explanation generated
```

Evidence must be:

```text
timestamped
versioned
actor-linked
engine-linked
decision-linked
order/cart/session-linked where applicable
tamper-evident where required
```

---

## 22. Customer Pricing Dispute Support

If customer disputes price, 81G helps explain:

```text
price at time of order
offer applied
offer expired
account price applied
tax/delivery included
points/credits used
why current price differs
what changed since purchase
refund/return price basis
```

Customer explanation must be clear and safe.

Do not show the customer the margin goblin.

---

## 23. Business Pricing Dispute Support

If business asks why price changed, 81G explains:

```text
supplier cost changed
capital cost increased
market shifted
brand rule blocked discount
promotion failed
B2B commission stack failed
customer price lock applied
delivery zone cost changed
return rate increased
manual override applied
```

This helps management distinguish “Selene made a mistake” from “the business rule did exactly what you asked, genius.”

---

## 24. Price Rollback Audit

If pricing is rolled back, 81G records:

```text
why rollback happened
who approved rollback
affected products/services
affected customers/orders/carts
old price
incorrect price
correct price
customer communication needed
refund/credit needed
affected channels
audit incident
```

Rollback must connect to:

```text
81B Dynamic Pricing
81 Core
Payment
Order
E-Commerce
POS
Accounting
Audit
```

---

## 25. Pricing Incident Management

81G should classify pricing incidents.

Incident types:

```text
wrong price
wrong discount
expired offer shown
B2B stack wrong
brand violation
tax/duty display wrong
customer charged incorrectly
coupon abuse
channel mismatch
personalization fairness issue
price lock violation
rollback failure
approval missing
```

Incident actions:

```text
block
rollback
customer remediation
refund/credit
approval review
compliance review
audit report
root-cause review
rule change request
```

---

## 26. Cross-Channel Audit

81G validates price consistency across channels.

Channels:

```text
E-Commerce
POS
personal Selene commerce
B2B
marketplace
private/VIP channel
account pricing
outlet
mobile POS
restaurant/table
hotel/travel channel
subscription channel
```

Cross-channel audit detects:

```text
wrong channel price
public/private price leakage
account customer undercut
B2B undercutting main store
outlet price leaking
POS override conflict
marketplace mismatch
expired campaign still visible
```

---

## 27. Customer-Facing Price Claim Control

81G must approve or validate claims like:

```text
best price
lowest price
limited time
exclusive
VIP only
free delivery
save X
was/now price
market-leading
members only
last chance
only today
```

Each claim needs:

```text
supporting evidence
validity window
terms
source
confidence
audit reference
```

No “best price” because marketing felt poetic. Evidence or hush.

---

## 28. Promotion Audit Integration — 81F

81G receives from 81F:

```text
promotion objective
test design
control group
holdout group
incremental lift
margin result
pull-forward result
brand impact
return impact
customer acquisition cost
customer quality
learning outcome
```

81G stores proof and links it to future pricing decisions.

This prevents the company from repeating a promotion that looked good only because everyone ignored next month’s sales crater.

---

## 29. B2B Audit Integration — 81E

81G receives from 81E:

```text
provider net
channel commission
Selene fee
customer benefit pool
reserve
return courier cost
clawback rules
bottom-line contribution
brand approval status
price floor
price ceiling
viability score
profit-share waterfall
```

This allows B2B pricing to be explained later.

Because someone will ask why the hair salon got commission for wine. 81G keeps the receipts.

---

## 30. Brand Audit Integration — 81D

81G receives from 81D:

```text
brand tier
sub-brand tier
prestige floor
official-channel rule
discount permission
launch/no-discount rule
brand approval
presentation compliance
channel suitability
brand dilution risk
```

This supports brand-sensitive pricing decisions.

---

## 31. Dynamic Pricing Audit — 81B

81G receives from 81B:

```text
trigger
scan cadence
playbook
simulation
approval
published change
monitoring result
rollback condition
dynamic confidence
```

This answers:

```text
Why did the price change today?
```

Without this, dynamic pricing becomes “because the machine twitched.” Delightful. Unacceptable.

---

## 32. Market Audit Integration — 81A

81G receives from 81A:

```text
competitor set
market range
normalization
quantity adjustment
quality adjustment
package adjustment
trust adjustment
data confidence
outliers removed
market trend
market alert
```

This supports market-based pricing explanations.

---

## 33. Customer Value Audit Integration — 81C

81G receives from 81C:

```text
customer value profile
category context
declared preference
confidence score
offer preference
fairness flags
sensitive-signal exclusions
```

This supports value-based offers without creepy pricing behavior.

---

## 34. Company Capability / Geography / Presentation Audit Integration — 81H / 81I / 81J

When 81H–81J exist, 81G must capture:

```text
company capability signal
service-level value
packaging/service cost-to-serve
geography/delivery zone cost
venue/location premium
local market adjustment
product presentation score
merchandising/perceived value signal
presentation improvement requirement
```

If 81H–81J are pending, 81G must record these as pending future pricing pack evidence integrations.

Do not invent missing signals.

The pricing pack is powerful. It is not clairvoyant. Sadly.

---

## 35. Compliance Handoff

81G must route legal/compliance review when pricing risk appears.

Triggers:

```text
regulated product/service
personalized pricing risk
sensitive data risk
deceptive discount risk
fake urgency risk
cross-border tax/duty uncertainty
professional service pricing issue
financial/credit-related pricing issue
best-price claim risk
consumer law risk
anti-discrimination risk
```

81G does not give final legal opinion.

It creates the compliance handoff and records the outcome.

---

## 36. Privacy and Data Minimization

81G must ensure explanations reveal only what is appropriate.

Rules:

```text
seller does not see full customer profile
provider does not see unrelated customer history
customer does not see business margin
staff does not see private wealth/payment assumptions
B2B channel does not see provider economics unless allowed
support sees only customer-safe reasoning
```

Pricing explainability must not become data leakage with nicer sentences.

---

## 37. Explainable AI Guardrail

GPT-5.5 may write human-readable explanations only from verified facts.

GPT-5.5 may explain:

```text
why offer was selected
why discount was not allowed
why price changed
why account price applied
why B2B benefit was funded
```

GPT-5.5 must not invent:

```text
cost
margin
approval
market data
customer segment
tax
brand rule
B2B commission
price lock
fairness result
```

Deterministic engines provide facts.

GPT-5.5 provides language.

The talking parrot does not get to invent the invoice.

---

## 38. Audit Immutability

81G records must be:

```text
timestamped
versioned
tamper-evident where required
actor-linked
engine-linked
decision-linked
order/cart/session-linked where applicable
approval-linked
evidence-linked
```

Audit data should not be quietly editable by some midnight spreadsheet goblin.

---

## 39. Retention and Archive Rules

Pricing evidence may need retention based on:

```text
company policy
tax policy
legal policy
audit policy
customer dispute window
contract term
warranty period
B2B settlement period
promotion period
regulatory requirement
```

81G must flag retention requirements and archive readiness.

---

## 40. Decision Replay

81G must support replaying historical pricing decisions.

Replay should show:

```text
old product cost
old market data
old customer profile confidence
old brand rule
old B2B stack
old promotion result
old price lock
old approval
old channel
old final price
old customer-facing display
```

This answers:

```text
Why did Selene charge this price six months ago?
```

The answer should not be “we have no idea, but the vibes were strong.”

---

## 41. Missing-Data Behavior

If required evidence is missing, 81G may require:

```text
block price
hold publication
use safe default
route review
mark low confidence
prevent customer-facing claim
prevent dynamic publishing
prevent discount approval
```

Missing data must be visible.

Hidden missing data is how clean dashboards lie.

---

## 42. Fairness Monitoring Over Time

81G should monitor fairness patterns.

Signals:

```text
certain groups consistently receive worse offers
certain customers unfairly excluded
pricing personalization causing complaints
offer targeting appears biased
manual overrides favoring some groups
account/customer price treatment inconsistent
discount access unfairly distributed
```

If detected:

```text
route compliance review
pause rule
audit decision
adjust rules/model
create fairness incident
```

Fairness is not one check at launch. It is ongoing, because humans and algorithms both find new ways to be weird.

---

## 43. Pricing Governance Dashboard

81G should maintain a pricing governance control board.

Dashboard items:

```text
active pricing decisions
low-confidence prices
manual overrides
high-risk discounts
brand exceptions
B2B unviable offers
fairness flags
rollback incidents
expired price locks
pending approvals
audit gaps
pricing incidents
unresolved compliance handoffs
cross-channel mismatches
pending evidence packs
```

This gives management a live view of pricing risk.

Not “pricing is fine because nobody screamed yet.” A bold strategy, but not Selene.

---

## 44. Pricing Governance Change Control

81G must control changes to pricing governance rules.

Change-controlled items:

```text
pricing priority hierarchy
fairness rules
audit requirements
explanation templates
approval thresholds
price claim rules
data confidence thresholds
rollback rules
retention rules
personalization rules
brand-sensitive evidence requirements
B2B audit requirements
```

Each change must record:

```text
requester
reason
risk level
approver
effective date
version
rollback path
affected documents/engines
audit reference
```

Governance rule changes must not happen casually.

Changing how Selene explains or audits prices is like changing the brakes while the car is moving. Thrilling. Bad.

---

## 45. Audit Export / Evidence Report Pack

81G must generate evidence packs for review.

Report pack types:

```text
management pricing report
customer price dispute pack
business pricing dispute pack
B2B provider dispute pack
B2B channel commission dispute pack
brand approval dispute pack
promotion performance pack
dynamic pricing decision pack
rollback incident pack
tax/accounting support pack
compliance/legal review pack
audit committee pack
```

Each pack may include:

```text
pricing decision packet
engine contribution trace
source data
version history
approval trace
customer-facing display
business explanation
fairness/compliance flags
audit log
supporting evidence
open issues
resolution status
```

This is Selene saying:

```text
Here is the evidence.
Please stop shouting into the finance channel.
```

A beautiful moment.

---

## 46. Outputs from 81G

81G should output:

```text
PricingExplanationPacket
CustomerPriceExplanationPacket
BusinessPriceExplanationPacket
AuditPriceExplanationPacket
PricingAuditPacket
FairnessReviewPacket
ComplianceReviewPacket
PricingDecisionTracePacket
PricingVersionPacket
DataLineagePacket
DataConfidencePacket
ApprovalTracePacket
PricingIncidentPacket
RollbackAuditPacket
PriceClaimEvidencePacket
DecisionReplayPacket
PricingGovernanceChangePacket
AuditEvidenceReportPack
```

---

## 47. State Machines

### Pricing Explanation State

```text
NotRequired
Required
FactsCollected
CustomerExplanationCreated
BusinessExplanationCreated
AuditExplanationCreated
ComplianceExplanationCreated
Published
Archived
Closed
```

### Pricing Audit State

```text
NotStarted
DecisionCaptured
EvidenceLinked
VersionLinked
ApprovalLinked
FairnessChecked
ConfidenceScored
AuditReady
AuditGapDetected
Closed
```

### Fairness Review State

```text
NotRequired
SignalDetected
SensitiveSignalExcluded
ReviewRequired
ComplianceRouted
Approved
Rejected
RulePaused
Closed
```

### Pricing Incident State

```text
NoIncident
IncidentDetected
Classified
OwnerAssigned
RollbackRequired
CustomerRemediationRequired
ComplianceReviewRequired
Resolved
Archived
Closed
```

### Decision Replay State

```text
Requested
HistoricalInputsLoaded
VersionRulesLoaded
DecisionRebuilt
ExplanationGenerated
EvidencePackCreated
Closed
```

### Governance Change State

```text
ChangeRequested
RiskChecking
ApprovalRequired
Approved
Rejected
Scheduled
Active
RolledBack
Archived
Closed
```

---

## 48. Reason Codes

```text
PRICING_DECISION_COMPLETENESS_CHECKED
PRICING_DECISION_PACKET_VALIDATED
PRICING_EXPLANATION_CREATED
CUSTOMER_PRICE_EXPLANATION_CREATED
BUSINESS_PRICE_EXPLANATION_CREATED
AUDIT_PRICE_EXPLANATION_CREATED
COMPLIANCE_PRICE_EXPLANATION_CREATED
FAIRNESS_GUARDRAIL_APPLIED
SENSITIVE_ATTRIBUTE_EXCLUDED
PERSONALIZATION_RISK_DETECTED
PRICING_ENGINE_CONFLICT_DETECTED
PRICING_PRIORITY_HIERARCHY_APPLIED
APPROVAL_TRACE_CAPTURED
PRICE_VERSION_CAPTURED
TIME_VALIDITY_CHECKED
DATA_LINEAGE_CAPTURED
DATA_CONFIDENCE_SCORED
EVIDENCE_LOCKER_UPDATED
CUSTOMER_PRICE_DISPUTE_SUPPORTED
BUSINESS_PRICE_DISPUTE_SUPPORTED
PRICE_ROLLBACK_AUDITED
PRICING_INCIDENT_CREATED
CROSS_CHANNEL_AUDIT_COMPLETED
CUSTOMER_CLAIM_EVIDENCE_REQUIRED
PROMOTION_AUDIT_LINKED
B2B_AUDIT_LINKED
BRAND_AUDIT_LINKED
DYNAMIC_PRICING_AUDIT_LINKED
MARKET_AUDIT_LINKED
CUSTOMER_VALUE_AUDIT_LINKED
COMPLIANCE_HANDOFF_CREATED
PRIVACY_MINIMIZATION_APPLIED
GPT_EXPLANATION_FACT_CHECK_REQUIRED
AUDIT_IMMUTABILITY_REQUIRED
RETENTION_RULE_APPLIED
DECISION_REPLAY_COMPLETED
MISSING_DATA_BLOCKED_PUBLICATION
FAIRNESS_PATTERN_MONITORING_TRIGGERED
PRICING_GOVERNANCE_CHANGE_REQUESTED
AUDIT_EVIDENCE_REPORT_PACK_CREATED
```

---

## 49. Required Simulations

```text
customer asks why price changed and 81G explains old/new pricing basis
business asks why discount was blocked and 81G shows brand/margin rule
B2B provider disputes commission and 81G produces waterfall evidence
customer claims best-price promise and 81G checks claim evidence
dynamic pricing raises price and 81G replays trigger/playbook/approval
promotion increased sales but lost profit and 81G links 81F evidence
price rollback occurs and 81G records affected carts/orders/customers
account price lock overrides dynamic pricing and 81G explains lock version
sensitive attribute signal detected and excluded from pricing
low-confidence market data blocks aggressive pricing claim
brand policy version changed and 81G records governance change
B2B customer benefit shown only after 81E funding proof
customer-facing explanation hides business margin
auditor requests pricing evidence report pack
missing tax/duty basis blocks all-in price publication
cross-channel price mismatch creates pricing incident
decision replay reconstructs price from six months ago
GPT-5.5 explanation blocked because deterministic facts missing
```

---

## 50. Integration Map

```text
PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING / DOCUMENT_81_CORE
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.SUPPLIER
↔ PH1.PROCUREMENT
↔ PH1.CASHFLOW
↔ PH1.DEBT_TREASURY
↔ PH1.B2B_PLATFORM
↔ PH1.CUSTOMER
↔ PH1.REWARDS / LOYALTY
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.RETURNS
↔ PH1.ACCOUNTING
↔ PH1.TAX
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
```

---

## 51. Required Logical Packets

```text
PricingExplanationPacket
CustomerPriceExplanationPacket
BusinessPriceExplanationPacket
AuditPriceExplanationPacket
PricingDecisionTracePacket
PricingDecisionCompletenessPacket
PricingVersionPacket
PricingRuleVersionPacket
DataLineagePacket
DataConfidencePacket
FairnessReviewPacket
SensitiveSignalExclusionPacket
PersonalizationGovernancePacket
ApprovalTracePacket
TimeValidityPacket
EvidenceLockerPacket
CustomerDisputePricingPacket
BusinessDisputePricingPacket
RollbackAuditPacket
PricingIncidentPacket
CrossChannelAuditPacket
PriceClaimEvidencePacket
ComplianceHandoffPacket
PrivacyMinimizationPacket
DecisionReplayPacket
RetentionArchivePacket
PricingGovernanceDashboardPacket
PricingGovernanceChangePacket
AuditEvidenceReportPack
PricingAuditEvidencePacket
```

Logical only.

No runtime packet structs. The audit goblin can file paperwork in theory, not production.

---

## 52. What Codex Must Not Do

```text
Do not make 81G decide final price.
Do not make 81G override Document 81 Core.
Do not allow unexplained pricing decisions.
Do not allow pricing decisions without version trace.
Do not allow customer-facing claims without evidence.
Do not expose business margins to customers.
Do not expose full customer profile to sellers/providers.
Do not use protected/sensitive attributes for unfair pricing.
Do not allow GPT-5.5 to invent pricing facts.
Do not treat low-confidence data as high-confidence evidence.
Do not ignore missing data.
Do not ignore pricing incidents.
Do not allow rollback without audit trace.
Do not create vague approval/notification without Human / External Action Orchestration.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 53. Final Architecture Sentence

Selene Pricing Pack Integration, Explainability, Fairness + Audit Engine is the pricing pack sub-engine that validates pricing decision completeness, connects all pricing sub-engine signals, records data lineage, confidence, versions, approvals, fairness checks, compliance handoffs, explanations, customer-visible claims, B2B waterfalls, brand rules, promotion evidence, dynamic pricing triggers, rollback incidents, cross-channel audits, governance changes, and evidence report packs; so every Selene price can be explained to the customer, justified to the business, replayed for audit, reviewed for fairness, exported for compliance, and proven later without relying on “the algorithm said so,” which is not an explanation, it is a confession.

Simple version:

```text
81G proves the price.

It answers:
what price
why
based on what
which engines contributed
was it fair
was it legal/compliance-aware
who approved it
what did the customer see
can we replay it later
can we export the evidence

81G does not set price.
81G makes pricing explainable, auditable, fair, versioned, and defensible.
```

That is 81G: the pricing pack’s receipt, witness, referee, librarian, and annoyed compliance goblin.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
