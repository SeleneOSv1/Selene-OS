# Global Document 81F — Selene Promotion Experimentation + A/B Pricing Governance Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE SUB-DOCUMENT / PRICING PACK ADDENDUM

PARENT DOCUMENT:
Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

SUB-DOCUMENT:
81F

ENGINE:
PH1.PRICING.81F / PH1.PROMOTION_EXPERIMENTATION / PH1.AB_OFFER_GOVERNANCE

FULL NAME:
Selene Promotion Experimentation, A/B Pricing Governance, Offer Testing, Incremental Lift, Pull-Forward Detection, Fast Failure Capture, Promotion ROI, Customer Acquisition, Loyalty, Cashflow, Turnover, Margin, Cannibalization, Brand, B2B, Refund Reversal, and Promotion Learning Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_PRICING_PACK_SUB_DOCUMENT
```

---

## 1. Purpose

Document 81F is Selene’s **promotion testing and offer-governance engine**.

It answers:

```text
Which promotion, discount, bundle, benefit, package, or offer actually works?
```

But “works” must be defined properly.

A promotion may be designed for:

```text
maximum profit
maximum turnover
cashflow injection
stock clearance
customer recruitment
customer loyalty
customer reactivation
brand awareness
B2B adoption
repeat purchase
event/holiday demand
market share
```

So 81F must never ask only:

```text
Did sales go up?
```

It must ask:

```text
Did the promotion achieve its intended objective, create true incremental value, protect margin and brand, avoid pulling future sales forward, recruit the right customers, avoid operational damage, and produce repeatable learning?
```

Because “big sale, big smile” is caveman marketing with a banner.

---

## 2. Core Promotion Experimentation Law

```text
No promotion may be judged successful unless Selene knows the promotion objective, primary metric, guardrail metrics, true incremental lift, full cost, margin impact, return/refund impact, customer quality, brand impact, operational impact, and post-promotion effect.
```

Promotion testing must prevent:

```text
celebrating turnover while losing profit
pulling future sales into one month and calling it growth
recruiting bargain-only customers
damaging brand with public discounts
overloading operations
funding unfunded B2B benefits
leaking coupons
stacking offers incorrectly
paying commissions on refunded orders
running failed tests too long
```

Simple rule:

```text
81F proves the promotion worked.
It does not assume the promotion worked because the graph looked happy.
```

Graphs are liars with colors.

---

## 3. Relationship to Document 81 Core

Document 81 Core owns:

```text
final price
final discount
final offer approval
margin guardrail
customer explanation
business explanation
pricing decision packet
pricing audit
```

Document 81F owns:

```text
promotion objective definition
experiment design
A/B test governance
control/holdout group logic
promotion eligibility testing
promotion performance measurement
incremental lift calculation
promotion pull-forward detection
promotion fatigue detection
promotion ROI
fast failure detection
stop-loss recommendation
promotion learning loop
promotion result classification
```

Simple split:

```text
81F tests and measures.
81 Core approves and publishes.
```

---

## 4. Engine Ownership Boundary

### 4.1 81F owns

```text
promotion test design
promotion objective declaration
primary metric selection
guardrail metric selection
offer variant design
A/B / A/B/C / multivariate testing logic
control group / holdout group governance
test exposure limits
promotion budget burn tracking
promotion stop-loss rules
statistical confidence / sample size signal
incremental lift measurement
pull-forward / time-compression analysis
post-promotion sales impact
customer acquisition cost analysis
customer quality analysis
promotion fatigue detection
coupon leakage / abuse detection
promotion result classification
promotion learning signals
```

### 4.2 81F references but does not own

```text
final price approval
brand rules
B2B pricing stack
customer segmentation truth
inventory truth
cashflow truth
marketing creative
payment execution
refund execution
accounting posting
tax treatment
delivery execution
customer reward balances
```

### 4.3 Correct owner split

```text
81 Core = final promotion approval.
81A = market context.
81B = dynamic price/offer movement.
81C = customer value and offer preference.
81D = brand/luxury guardrail.
81E = B2B profitability and customer benefit funding.
81F = promotion testing and learning.
81G = explainability, fairness, audit.
Marketing = campaign creative and audience strategy.
E-Commerce / POS = promotion display and execution surface.
Order = price/offer lock into order.
Payment = payment/refund execution.
Accounting/Tax = financial treatment.
Audit = proof.
```

---

## 5. Promotion Objective Declaration

Every promotion must declare its objective before launch.

Possible objectives:

```text
maximum profit
maximum turnover
cashflow injection
stock clearance
customer acquisition
customer loyalty
customer reactivation
brand awareness
B2B adoption
repeat purchase growth
market share growth
event / holiday sales
new product launch support
slow-stock reduction
expiry reduction
```

Rule:

```text
A promotion cannot be judged unless Selene knows what it was meant to achieve.
```

A cashflow promotion is not judged like a loyalty promotion.

A clearance promotion is not judged like a luxury launch.

Context exists. Humans hate this, but spreadsheets need it.

---

## 6. Primary Metric + Guardrail Metrics

Each promotion must define:

```text
primary metric
secondary metrics
guardrail metrics
stop-loss metrics
review cadence
decision deadline
```

Examples:

```text
Primary goal: acquire new customers
Guardrails:
- margin cannot fall below X
- return rate cannot exceed Y
- acquisition cost cannot exceed Z
```

```text
Primary goal: clear aging stock
Guardrails:
- do not damage premium brand
- do not cannibalize new-season stock
- do not sell below approved floor
```

No “campaign performed well” unless Selene knows what “well” means.

---

## 7. Profit vs Turnover vs Cashflow Mode

81F must distinguish promotion modes.

Modes:

```text
profit mode
turnover mode
cashflow mode
loyalty mode
recruitment mode
reactivation mode
clearance mode
brand-awareness mode
B2B-adoption mode
```

Example:

```text
Cashflow is tight:
Selene may prioritize fast cash turnover.

Profit is strong:
Selene may protect margin and avoid discounting.

Customer recruitment is weak:
Selene may test acquisition benefits.
```

Revenue is loud. Profit is the one paying rent.

---

## 8. Promotion Types

81F must support testing:

```text
percentage discount
fixed discount
bundle offer
buy X get Y
free delivery
loyalty credit
cashback
gift-with-purchase
rare gift
VIP packaging
early access
priority delivery
warranty extension
payment terms
free setup
private sale
clearance offer
B2B customer benefit
restaurant package
hotel package
car rental package
airline fare/add-on package
event package
subscription offer
```

The right promotion may not be a discount.

Sometimes it is access, service, packaging, flexibility, or not being treated like cattle in checkout.

---

## 9. Experiment Structure

81F must support:

```text
A/B tests
A/B/C tests
multivariate tests
control groups
holdout groups
limited test markets
customer-segment tests
channel tests
geo/location tests
time-window tests
event-period tests
category tests
B2B channel tests
```

Each experiment must define:

```text
test hypothesis
audience
variant set
control group
duration
minimum sample
budget limit
success metric
stop rule
rollback rule
audit reference
```

---

## 10. Statistical Confidence + Sample Size Rules

81F must avoid declaring winners too early.

Every test should track:

```text
sample size
conversion count
minimum data threshold
confidence level
noise level
segment size
seasonality distortion
event distortion
small-sample warning
inconclusive result
```

If data is too weak:

```text
do not scale
do not declare winner
extend test
reduce scope
rerun test
route review
```

Tiny sample sizes create big opinions. This is how marketing meetings become superstition with charts.

---

## 11. Test Eligibility Rules

Before testing, Selene must check:

```text
product eligible?
customer segment eligible?
channel eligible?
brand allows it?
margin allows it?
B2B stack allows it?
price lock conflict?
account customer conflict?
legal/compliance issue?
stock available?
offer funded?
operations can handle it?
```

If not eligible:

```text
block
route approval
change test design
use safer offer
```

---

## 12. Brand Guardrail Integration

81F must ask 81D before testing:

```text
luxury discounts
new-release offers
premium brand clearance
public sale
private VIP sale
B2B brand offer
outlet strategy
brand-sensitive bundles
```

If discount harms brand:

```text
test VIP gift
test premium packaging
test early access
test private appointment
test rare gift
test service upgrade
```

No “20% off prestige” unless the brand policy has chosen violence.

---

## 13. Customer Segment Integration

81F must ask 81C which customer value group applies.

Segments may include:

```text
price-sensitive
status-driven
service-first
warranty-focused
loyalty-driven
business/account
gift buyer
travel customer
VIP
reactivation target
```

Examples:

```text
Bargain hunter → discount / bundle test
VIP → rare gift / early access test
Business traveler → flexible package test
Convenience customer → free delivery / priority service test
```

Different customers respond to different carrots. Some want carrots. Some want a velvet box.

---

## 14. B2B Integration

81F must ask 81E before testing any B2B promotion.

Check:

```text
provider net
channel commission
Selene fee
customer benefit pool
reserve
delivery cost
return courier cost
refund/clawback logic
bottom-line profit target
```

81F must not run:

```text
unfunded customer benefit
unprofitable B2B discount
commission-breaking offer
reward that cannot be reversed
```

B2B promotion without 81E is just “everyone gets paid except the business.” Charming. Fatal.

---

## 15. Dynamic Pricing Integration

81F feeds 81B.

If test works:

```text
scale offer
extend promotion
trigger dynamic offer rule
update playbook
```

If test fails:

```text
pause
rollback
try different offer
route review
```

81B may request 81F when dynamic pricing needs evidence before scaling.

---

## 16. Success Metrics

81F must measure:

```text
conversion rate
gross revenue
gross margin
net margin
contribution margin
profit after returns
average order value
repeat purchase
customer lifetime value
return rate
refund rate
delivery cost
support cost
warranty cost
marketing spend
cashflow impact
stock movement
brand impact
B2B profitability
```

Main rule:

```text
Profit beats vanity sales.
```

The only thing worse than no sales is glorious unprofitable sales.

---

## 17. Incremental Lift

81F must measure true incremental change.

Not enough:

```text
Sales increased.
```

Better:

```text
Sales increased because of the offer, compared with control group or baseline.
```

81F must use:

```text
control group
holdout group
historical baseline
forecast baseline
similar store/region comparison
customer cohort comparison
```

This avoids giving the promotion credit for Christmas, Grand Prix, payday, rain, sun, or people being hungry. Humans misattribute everything. It’s their art form.

---

## 18. Pull-Forward / Time-Compression Detection

81F must detect when a promotion only pulls future sales into the promotion period.

Example:

```text
Normal 3-month sales = 300 units.

Promotion month sells 250 units.
Next two months sell 50 units.
Total = 300 units.

Result:
No true incremental demand.
Margin sacrificed.
Marketing spend wasted.
```

81F must detect:

```text
sales pulled forward
post-promotion sales drop
promotion hangover
no true incremental demand
marketing spend wasted
margin sacrificed for sales that would have happened anyway
```

This is financial time travel, but dumb.

---

## 19. Longer Measurement Horizon

81F must measure beyond the campaign window.

Suggested horizons:

```text
during promotion
immediately after promotion
30-day follow-up
60-day follow-up
90-day follow-up
category-specific cycle
season-specific cycle
subscription renewal cycle
```

This shows whether promotion:

```text
created new demand
pulled future demand forward
recruited loyal customers
recruited one-time bargain hunters
damaged full-price demand
```

---

## 20. Market Capacity / Demand Saturation

Some markets have natural demand limits.

Examples:

```text
a suburb buys only so many pizzas
a local salon area has limited premium product buyers
a car rental branch has fixed fleet capacity
a hotel has fixed rooms
a niche product has limited purchase frequency
```

81F should estimate:

```text
local demand ceiling
customer population
purchase frequency
category repeat cycle
replacement cycle
event-driven demand
capacity constraints
```

If demand is capped, Selene should not celebrate time-shifted sales as growth.

---

## 21. Marketing Spend + Acquisition Cost

Promotion result must deduct full cost.

Costs include:

```text
advertising spend
creative cost
discount cost
free delivery cost
gift cost
B2B commission
customer benefit cost
return/refund cost
support cost
fulfillment stress cost
payment cost
promotion admin cost
```

81F calculates:

```text
net profit
cost per acquired customer
payback period
repeat purchase value
campaign ROI
cashflow impact
```

A campaign that works before marketing cost may be a campaign that politely burns money.

---

## 22. New vs Existing Customer Split

81F must separate:

```text
new customers
existing customers
reactivated customers
VIP customers
bargain-only customers
account customers
B2B channel customers
```

Important flag:

```text
Promotion mostly redeemed by customers who would have bought anyway.
```

Example:

```text
80% of redemptions came from regular customers who normally buy full price.
```

That is not customer acquisition. That is paying your existing customers to do what they were already doing. Generous. Sad.

---

## 23. Customer Quality After Promotion

Not all acquired customers are valuable.

81F should track:

```text
do new customers buy again?
do they only buy discounts?
do they return more?
do they complain more?
do they convert into loyalty?
do they refer others?
do they buy full price later?
```

This prevents recruiting bargain goblins who emerge only when margin is bleeding.

---

## 24. Promotion Fatigue / Trained-to-Wait Behavior

81F must detect promotion dependence.

Signals:

```text
full-price sales drop
customers delay purchases
discount response weakens
sale periods become expected
brand perception drops
repeat discount dependence
```

If detected:

```text
reduce public discounts
switch to private targeted benefits
use value-add offers
protect brand
```

Discount addiction starts as a campaign and ends as a business model nobody admits to.

---

## 25. Coupon Leakage / Abuse

81F must detect promotion abuse.

Examples:

```text
coupon shared publicly
staff discount abuse
same customer using multiple accounts
referral abuse
fake new customer abuse
stacking abuse
refund-after-benefit abuse
coupon resale
bot redemption
B2B benefit abuse
```

Actions:

```text
pause code
limit eligibility
route fraud review
claw back benefit
update rules
audit
```

Promotions need locks. Cute coupon codes are not security.

---

## 26. Offer Stacking Rules

81F must test stacking carefully.

Stack combinations:

```text
discount + points
discount + free delivery
bundle + coupon
VIP benefit + account price
B2B benefit + channel commission
clearance + loyalty credit
cashback + customer benefit pool
```

81 Core decides final allow/deny.

81F measures effects.

Stacking is where innocent offers become little margin crimes.

---

## 27. Stock and Inventory Effects

Promotions must consider inventory.

Signals:

```text
stock available
stock aging
expiry risk
clearance need
scarcity
replacement cost
reorder lead time
capital holding cost
floorplan finance
construction/holding cost
warehouse capacity
```

Promotion may be useful when:

```text
stock is aging
cash is tied up
expiry risk exists
storage pressure exists
```

Promotion may be dangerous when:

```text
stock is scarce
replenishment cost increased
supply chain is delayed
brand scarcity supports premium price
```

---

## 28. Operational Stress Impact

Promotions can break operations.

81F must check:

```text
can dispatch handle volume?
can warehouse fulfill?
can kitchen handle orders?
can hotel staff handle occupancy?
can car rental fleet handle pickup/return?
can customer service handle support?
will delivery cost spike?
will return volume spike?
will supplier/provider fail?
```

A promotion that sells more than the company can serve creates refunds, complaints, and reviews with teeth.

---

## 29. Event and Seasonal Testing

81F should test around:

```text
public holidays
Grand Prix
concerts
fairs
school holidays
tourism seasons
restaurant holidays
hotel event periods
car rental weekends
fashion seasons
product launches
weather events
local festivals
```

Event tests must distinguish:

```text
promotion effect
event demand effect
seasonality effect
capacity effect
```

Otherwise the campaign gets credit for the Grand Prix existing. Marketing would love that. Selene should not.

---

## 30. Test Duration and Cadence

Every test needs:

```text
start date
end date
minimum data requirement
review cadence
early review point
stop rule
rollback rule
decision deadline
post-promotion review date
```

No eternal experiments. That is indecision with charts.

---

## 31. Test Exposure Limits

Selene must limit risk.

Limits may include:

```text
customer count
order volume
discount budget
margin loss
channel exposure
brand exposure
geographic exposure
time window
B2B commission exposure
stock allocation
cashflow exposure
```

Test small. Scale winners. Stop losers.

Look at that: science, but with invoices.

---

## 32. Promotion Budget Burn + Stop-Loss Caps

81F must track budget consumption.

Budget limits:

```text
total campaign budget
discount cost budget
free delivery budget
gift budget
B2B customer benefit budget
cashback budget
media/ad spend budget
margin loss allowance
return/refund allowance
```

Stop-loss triggers:

```text
spend exceeds limit
margin loss exceeds limit
cost per acquisition exceeds limit
return rate exceeds limit
conversion below threshold
customer complaints exceed threshold
brand-risk threshold exceeded
```

If stop-loss triggers:

```text
pause promotion
route review
rollback
modify test
notify owner
audit
```

Fast failure is not pessimism. It is survival.

---

## 33. Fast Failure Detection

81F must capture failure early.

Every test needs:

```text
early review points
minimum expected response
loss limit
margin loss limit
return-rate warning
customer complaint warning
stock/fulfillment stress warning
brand-risk warning
automatic stop rule
```

Example:

```text
If after 24 hours the promotion has low conversion and high margin loss, pause and review.
```

Selene should not let a bad promotion run for 30 days because someone scheduled it and then went emotionally offline.

---

## 34. Test / Learn / Adjust / Retest Loop

81F must operate as a learning loop:

```text
test small
measure quickly
stop losers
scale winners
modify uncertain tests
retest improved version
record learning
```

Not:

```text
Run 10% off.
Hope.
Make a PowerPoint.
```

That is not experimentation. That is retail gambling with a font.

---

## 35. Test Contamination / Overlap Control

81F must prevent overlapping tests from corrupting results.

Problems:

```text
same customer in multiple tests
same product in conflicting promotions
POS and E-Commerce showing different offers unintentionally
B2B benefit overlapping with public discount
VIP offer mixed with clearance offer
customer sees control and test offer
```

Controls:

```text
mutual exclusion groups
test priority rules
customer assignment lock
channel lock
variant isolation
overlap detection
contamination flag
invalid result classification
```

If contaminated:

```text
mark test invalid or low-confidence
rerun clean test
do not scale result
```

A dirty experiment is just confusion with math symbols.

---

## 36. Attribution Window / Delayed Impact Tracking

81F must define attribution windows.

Examples:

```text
same-session purchase
24-hour attribution
7-day attribution
30-day repeat purchase
60-day retention
90-day loyalty impact
subscription renewal window
travel booking window
B2B repeat-purchase window
```

This helps determine whether promotion drove:

```text
immediate purchase
delayed purchase
repeat purchase
customer reactivation
loyalty behavior
pull-forward behavior
```

No claiming victory forever because someone clicked once in March. Calm down, analytics.

---

## 37. Approval Rules

Some tests need approval.

Approval-required examples:

```text
below-margin offer
luxury/premium offer
B2B commission-changing offer
public clearance
high-value product
account customer offer
regulated/professional service offer
large campaign budget
promotion with high operational risk
promotion with high brand risk
```

Use Human / External Action Orchestration:

```text
owner
approver
deadline
confirmation
reminder
escalation
audit
```

No “notify marketing manager.” That phrase belongs in a bin with broken workflows.

---

## 38. Compliance / Fairness Guardrail

81F must avoid:

```text
fake discounts
false urgency
misleading “was” prices
hidden fees
unfair personalized pricing
discriminatory targeting
unsupported “best price” claims
dark patterns
unclear offer terms
```

Promotions must have clear:

```text
eligibility
expiry
terms
redemption limits
refund effect
stacking rules
customer-facing explanation
```

The customer should not need a lawyer to understand “20% off.”

---

## 39. Customer Communication

81F should define how offers are shown.

Channels:

```text
public offer
private offer
VIP-only offer
account customer offer
B2B channel offer
E-Commerce display
POS private prompt
email / Selene message
push notification
in-app banner
restaurant/table prompt
hotel/travel package prompt
```

Must avoid:

```text
conflicting prices
publicly exposing private offers
showing unfunded benefits
showing expired offers
showing brand-damaging discounts
```

---

## 40. Cross-Channel Consistency

81F must check promotion consistency across:

```text
E-Commerce
POS
B2B
marketplace
private/VIP channel
account customer pricing
outlet
mobile POS
restaurant/table
hotel/travel channel
```

81F must detect:

```text
same offer displaying incorrectly
different discount unintentionally applied
account customer harmed by public offer
B2B channel undercutting main channel
outlet/private sale leaking publicly
```

---

## 41. Refund and Reversal Impact

If promotion item is refunded, Selene must reverse or adjust:

```text
discount
loyalty credit
cashback
gift benefit
free delivery benefit
B2B customer benefit
channel commission
provider payout
reward points
tax/accounting effect
```

Connects to:

```text
81E B2B pricing
Document 83 Returns
Payment
Rewards
Accounting
Tax
```

No keeping benefits after refund unless policy explicitly says so. Free money has fans. Finance is not one of them.

---

## 42. Promotion Liability / Accounting Treatment

Some promotions create future obligations.

Examples:

```text
store credit
gift card
cashback
loyalty points
future discount voucher
customer benefit pool credit
warranty extension
free future service
```

81F must flag accounting/tax treatment needs.

Questions:

```text
is this an immediate discount?
future liability?
deferred revenue?
marketing expense?
customer benefit obligation?
breakage expected?
expiry date?
refund reversal needed?
```

81F does not post accounting.

It sends required structure to Accounting / Tax.

---

## 43. Experiment Result Classification

Every test result must conclude clearly.

Classifications:

```text
winner
loser
inconclusive
profitable but brand-risky
high sales but low profit
good for acquisition
good for retention
good for clearance
good for cashflow only
not truly incremental
pulled sales forward
loss-making
bad due to returns
operationally damaging
requires more data
rerun with changes
```

No vague “campaign performed well.” Compared to what, Todd? Your hopes?

---

## 44. Automation Triggers

81F should run when:

```text
new promotion proposed
81B recommends offer test
81C finds unclear customer segment response
81D blocks discount and suggests value-add test
81E needs B2B customer benefit test
inventory aging requires clearance test
campaign underperforms
return rate spikes after promotion
holiday/event window approaches
cashflow mode changes
brand-risk signal appears
customer acquisition weakens
customer loyalty weakens
```

---

## 45. Human / External Action Orchestration

81F must follow the Human / External Action Orchestration Law when humans or external parties must act.

Possible actions:

```text
approve test
approve budget
approve brand-sensitive promotion
approve B2B promotion
approve below-margin test
review failed promotion
confirm campaign stop
approve rollback
review fraud/leakage
```

Each action must define:

```text
owner
recipient
deadline
confirmation
evidence
reminder
escalation
audit
```

No “someone should review the campaign.” Someone is a myth. Assign a human.

---

## 46. Learning Loop

81F sends learning to:

```text
81 Core
81B Dynamic Pricing
81C Customer Value
81D Brand Guardrail
81E B2B Pricing
81G Explainability/Audit
Marketing
Customer Engine
Inventory
Cashflow
B2B
E-Commerce
POS
Order
```

Selene learns:

```text
which offers work
for whom
in which channel
for which products
at what margin
with what return risk
with what loyalty effect
with what brand impact
```

---

## 47. Outputs from 81F

81F outputs:

```text
PromotionObjectivePacket
PromotionTestDesignPacket
PromotionEligibilityPacket
OfferVariantPacket
ControlHoldoutPacket
PromotionBudgetPacket
PromotionPerformancePacket
IncrementalLiftPacket
PullForwardDetectionPacket
MarginImpactPacket
CustomerAcquisitionPacket
CustomerQualityPacket
PromotionFatiguePacket
CannibalizationPacket
CouponAbusePacket
OperationalStressPacket
PromotionResultClassificationPacket
PromotionStopRecommendationPacket
PromotionRollbackPacket
PromotionLearningSignalPacket
PromotionAuditEvidencePacket
```

---

## 48. State Machines

### Promotion Test State

```text
Draft
EligibilityChecking
Approved
Active
EarlyReview
Monitoring
Paused
Stopped
Completed
RolledBack
Closed
```

### Experiment Result State

```text
CollectingData
InsufficientData
Inconclusive
Winner
Loser
Contaminated
PullForwardDetected
Profitable
Unprofitable
BrandRisk
OperationalRisk
Closed
```

### Promotion Budget State

```text
NotStarted
BudgetAssigned
BurnTracking
WarningThreshold
StopLossTriggered
Paused
Closed
```

### Promotion Learning State

```text
NoLearning
SignalDetected
Validated
AppliedToSegment
AppliedToProduct
AppliedToChannel
SentToPricingPack
Closed
```

---

## 49. Reason Codes

```text
PROMOTION_OBJECTIVE_DECLARED
PRIMARY_METRIC_SELECTED
GUARDRAIL_METRIC_SELECTED
PROMOTION_TEST_CREATED
CONTROL_GROUP_CREATED
HOLDOUT_GROUP_CREATED
TEST_ELIGIBILITY_PASSED
TEST_ELIGIBILITY_FAILED
BRAND_GUARDRAIL_REQUIRED
B2B_VIABILITY_REQUIRED
CUSTOMER_SEGMENT_TEST_SELECTED
PROMOTION_BUDGET_ASSIGNED
PROMOTION_STOP_LOSS_TRIGGERED
FAST_FAILURE_DETECTED
INCREMENTAL_LIFT_CALCULATED
PULL_FORWARD_DETECTED
MARKET_CAPACITY_LIMIT_DETECTED
MARKETING_COST_APPLIED
NEW_CUSTOMER_SPLIT_CALCULATED
EXISTING_CUSTOMER_DISCOUNT_LEAK_DETECTED
CUSTOMER_QUALITY_LOW
PROMOTION_FATIGUE_DETECTED
COUPON_ABUSE_DETECTED
OFFER_STACKING_BLOCKED
OPERATIONAL_STRESS_DETECTED
TEST_CONTAMINATION_DETECTED
STATISTICAL_CONFIDENCE_LOW
PROMOTION_WINNER_IDENTIFIED
PROMOTION_LOSER_IDENTIFIED
PROMOTION_INCONCLUSIVE
PROMOTION_ROLLBACK_RECOMMENDED
PROMOTION_LEARNING_SIGNAL_CREATED
PROMOTION_AUDIT_CAPTURED
```

---

## 50. Required Simulations

```text
profit-mode promotion chooses margin over turnover
cashflow-mode promotion prioritizes fast cash with guardrails
customer acquisition promotion calculates cost per acquired customer
promotion increases sales but loses profit and is rejected
promotion pulls three months of demand into one month and is flagged
promotion shows sales increase but no true incremental lift against control group
public discount blocked by 81D and rare gift test proposed
B2B customer benefit test blocked by 81E due to unfunded benefit pool
promotion stopped early due to fast failure rule
coupon code leaked and abuse detected
promotion causes operational overload and is paused
free delivery beats 10% discount after net margin comparison
discount boosts sales but increases returns and is rejected
VIP gift works better than discount for premium segment
event-period test separates event demand from promotion effect
promotion result marked inconclusive due to low sample size
promotion contaminated by overlapping offers and rerun required
customer acquisition promotion recruits bargain-only customers and is rejected
clearance promotion succeeds without damaging new-season sales
refund reverses loyalty credit and B2B commission
future credit promotion creates accounting liability signal
```

---

## 51. Integration Map

```text
PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING / DOCUMENT_81_CORE
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.MARKETING
↔ PH1.CUSTOMER
↔ PH1.REWARDS / LOYALTY
↔ PH1.B2B_PLATFORM
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.INVENTORY
↔ PH1.DISPATCH / DELIVERY
↔ PH1.RETURNS
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.CASHFLOW
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

## 52. Required Logical Packets

```text
PromotionObjectivePacket
PromotionMetricPacket
PromotionTestDesignPacket
PromotionEligibilityPacket
OfferVariantPacket
ControlGroupPacket
HoldoutGroupPacket
PromotionBudgetPacket
PromotionExposureLimitPacket
PromotionPerformancePacket
IncrementalLiftPacket
PullForwardDetectionPacket
MarketCapacityPacket
MarketingSpendPacket
CustomerAcquisitionCostPacket
CustomerQualityPacket
PromotionFatiguePacket
CouponAbusePacket
OfferStackingPacket
OperationalStressPacket
TestContaminationPacket
StatisticalConfidencePacket
AttributionWindowPacket
PromotionLiabilityPacket
PromotionRefundReversalPacket
PromotionResultClassificationPacket
PromotionStopRecommendationPacket
PromotionRollbackPacket
PromotionLearningSignalPacket
PromotionAuditEvidencePacket
```

Logical only.

No runtime packet structs. The coupon goblin can wait outside with its fake promo codes.

---

## 53. What Codex Must Not Do

```text
Do not make 81F decide final price.
Do not make 81F override Document 81 Core.
Do not judge promotions by revenue alone.
Do not ignore profit, margin, returns, delivery cost, support cost, or marketing spend.
Do not treat pulled-forward demand as true incremental growth.
Do not ignore post-promotion sales drop.
Do not scale tests without sufficient data confidence.
Do not ignore brand guardrails.
Do not run B2B benefits without 81E funding validation.
Do not ignore coupon abuse or stacking abuse.
Do not ignore operational stress.
Do not leave failed tests running without stop-loss logic.
Do not create customer-facing “best price” claims without evidence.
Do not create vague approval/notification without Human / External Action Orchestration.
Do not let GPT-5.5 invent test results, sample size, ROI, lift, customer quality, or promotion success.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 54. Final Architecture Sentence

Selene Promotion Experimentation + A/B Pricing Governance Engine is the pricing pack sub-engine that designs, limits, tests, measures, stops, learns from, and audits promotions across discounts, bundles, benefits, free delivery, VIP gifts, B2B customer benefits, clearance, event offers, loyalty, recruitment, reactivation, cashflow, and turnover campaigns; while proving true incremental lift, detecting pull-forward demand, accounting for full marketing and operating cost, protecting margin and brand, catching failure early, preventing promotion fatigue and abuse, measuring customer quality, reversing benefits on refunds, and sending repeatable learning back into Document 81 Core, Dynamic Pricing, Customer Value, Brand Guardrail, B2B Pricing, Marketing, Inventory, Cashflow, E-Commerce, POS, Order, and Audit.

Simple version:

```text
81F proves whether promotions actually work.

Not:
“Sales went up.”

But:
“Profit went up.”
“Cashflow improved.”
“Good customers were recruited.”
“Stock cleared safely.”
“Brand was protected.”
“Future sales were not just pulled forward.”
“Returns did not destroy the result.”
“The test had enough data.”
“The loser was stopped quickly.”
“The winner can be scaled.”
```

That is 81F: the engine that stops businesses from throwing discounts into the market like confetti and calling the mess “strategy.”

---

## 55. Commerce Stack 82-84 Promotion Reversal Handoff

Promotions must account for refund rates, return reasons, exchange/replacement patterns, benefit reversals, post-promotion return abuse, delayed return effects, and customer/seller enforcement signals from Document 83.

Promotion liability, cashback, future credit, customer benefit reversal, refund reversal, offer-stack reversal, and B2B promotion clawback logic must link to Document 83 and 81G before promotion results are classified as successful.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
