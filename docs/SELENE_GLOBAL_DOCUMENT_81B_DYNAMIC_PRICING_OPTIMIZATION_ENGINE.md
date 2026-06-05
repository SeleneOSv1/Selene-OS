# Global Document 81B — Selene Dynamic Pricing Optimization Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE SUB-DOCUMENT / PRICING PACK ADDENDUM

PARENT DOCUMENT:
Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

SUB-DOCUMENT:
81B

ENGINE:
PH1.PRICING.81B / PH1.DYNAMIC_PRICING / PH1.REVENUE_OPTIMIZATION

FULL NAME:
Selene Dynamic Pricing Optimization, Trigger Matrix, Price Movement, Demand Response, Inventory Lifecycle, Holiday/Event Pricing, Occupancy/Yield, Fleet Utilization, Labour Cost, Raw Material Shock, Finance Carrying Cost, Clearance, Simulation, Monitoring, Rollback, and Revenue Optimization Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_PRICING_PACK_SUB_DOCUMENT
```

---

## 1. Purpose

Document 81B is Selene’s **Dynamic Pricing Optimization Engine**.

It answers:

```text
Should the price, offer, discount, package, or markdown change now?
```

It does **not** own the final price.

```text
81B recommends movement.
81 Core approves, governs, explains, publishes, and audits the final pricing decision.
```

81B watches changing conditions:

```text
demand
supply
stock
expiry
seasonality
events
public holidays
hotel occupancy
fleet utilization
restaurant labour cost
supplier cost changes
raw material shocks
market price movement
B2B economics
cashflow pressure
overdraft costs
floorplan finance
construction finance
holding costs
promotion performance
returns and warranty claims
```

Simple version:

```text
81B tells Selene when yesterday’s price has become today’s problem.
```

Or, less poetic:

```text
81B detects pricing change triggers, runs the correct pricing playbook, simulates the impact, recommends action, monitors results, and triggers rollback if reality misbehaves.
```

Dynamic pricing without guardrails is financial wildlife. Selene is not wildlife. Selene is supervised capitalism with receipts.

---

## 2. Core Dynamic Pricing Law

```text
Selene must not change prices merely because a signal changed.

Every dynamic price recommendation must define:
- trigger source
- detection method
- scan cadence or event source
- affected product/service/customer/channel
- required data
- decision playbook
- permitted action
- margin impact
- brand impact
- customer trust impact
- approval requirement
- simulation result where required
- publication path
- monitoring rule
- rollback rule
- closure condition
- audit reference
```

No design may say:

```text
If stock is expiring, discount it.
```

without defining:

```text
How Selene knows it is expiring.
How often Selene checks.
Which products qualify.
Which markdown rule applies.
Who approves.
Where the discount publishes.
How sell-through is monitored.
What happens if it does not work.
What happens when the product expires.
```

Otherwise it is architecture theatre wearing a pricing hat.

---

## 3. Relationship to Document 81 Core

Document 81 Core owns:

```text
final price
final offer
margin guardrail
discount governance
price lock enforcement
business explanation
customer explanation
approval routing
pricing audit
```

Document 81B owns:

```text
dynamic trigger detection
dynamic pricing playbooks
pricing movement recommendation
pricing scan cadence
occupancy/utilization pricing signals
holiday/event pricing signals
carrying cost pricing signals
expiry/clearance pricing signals
simulation requirement
rollback recommendation
post-change monitoring
dynamic pricing learning loop
```

Simple split:

```text
81B = should anything change?
81 = may we publish it?
```

Do not let 81B become a rogue price robot. It will put surge pricing on bread and call it strategy. We are better than that, barely.

---

## 4. Engine Ownership Boundary

### 4.1 81B owns

```text
dynamic trigger matrix
pricing scan cadence matrix
event-driven pricing signal intake
scheduled pricing scan intake
manual pricing review request intake
dynamic eligibility check
price movement recommendation
markdown recommendation
clearance recommendation
occupancy/yield pricing signal
fleet utilization pricing signal
duration pricing signal
holiday/public holiday/special event signal
labour multiplier pricing signal
raw material shock pricing signal
overdraft / finance pressure signal
inventory carrying cost signal
price lock review recommendation
promotion performance reaction
dynamic offer switching
pricing simulation request
post-publication monitoring
rollback / emergency stop recommendation
dynamic pricing audit evidence
```

### 4.2 81B references but does not own

```text
final pricing approval
final price publication
product master truth
inventory stock truth
hotel room inventory truth
vehicle fleet inventory truth
staff payroll truth
supplier cost truth
market intelligence truth
B2B commission calculation
customer segmentation truth
brand policy truth
cashflow truth
debt/treasury truth
legal/compliance truth
payment execution
accounting ledger posting
```

### 4.3 Correct owner split

```text
81A = market movement / competitor signal
81B = dynamic change recommendation
81C = customer price sensitivity signal
81D = brand/luxury guardrail
81E = B2B pricing stack
81F = promotion experiment results
81G = explainability, fairness, audit
81H = company capability / cost-to-serve
81I = geography / local event / delivery zone cost
81J = presentation / perceived value
81 Core = final governed pricing decision
```

---

## 5. Dynamic Pricing Participation Profile

Companies must choose how much Selene may participate in pricing.

Participation levels:

```text
Manual pricing only
Selene recommendations only
Selene drafts price changes for approval
Selene auto-adjusts within approved rules
Selene full dynamic pricing autopilot
Dynamic pricing only for selected categories
Clearance-only automation
B2B pricing optimization only
Customer-offer optimization only
Event/holiday pricing only
Occupancy/utilization pricing only
```

Selene may recommend a participation mode.

Example:

```text
“Your hotel business is highly suited to occupancy-based dynamic pricing. Do you want to test automated pricing for 30 days with manager approval before publication?”
```

Company controls:

```text
allowed categories
maximum price movement
minimum margin
brand restrictions
approval thresholds
customer communication rules
rollback thresholds
testing period
```

No algorithmic raccoon pricing unless the company invited the raccoon and gave it a cage.

---

## 6. Dynamic Trigger Matrix

81B must classify how Selene knows something changed.

Trigger types:

```text
event-driven
scheduled scan
manual request
external market signal
threshold-based
time-based
calendar-based
location/event-based
inventory-based
finance-based
demand-based
capacity-based
```

Examples:

```text
Product expiring soon = scheduled inventory expiry scan
Supplier cost changed = Procurement/Supplier event
Competitor price changed = 81A market signal
Hotel occupancy changed = booking/availability event
Car fleet shortage = fleet availability event
Public holiday tomorrow = calendar event
Grand Prix in town April 14–25 = special event calendar signal
Raw material cost shock = supplier/market/currency event
Overdraft cost rising = Cashflow / Treasury signal
Construction interest accumulating = Real Estate / Debt / Treasury signal
```

Every trigger must have:

```text
source engine
scan frequency or event source
confidence score
affected pricing scope
required playbook
```

---

## 7. Monitoring Cadence Matrix

81B must define how often different pricing checks run.

Example cadences:

```text
high-velocity retail = hourly / daily
hotel occupancy = real-time / hourly
car rental fleet availability = real-time / hourly
perishable expiry = daily / intraday near expiry
fashion stock aging = daily / weekly
real estate holding cost = daily / weekly / monthly
supplier cost changes = event-driven
competitor market scan = scheduled + event-triggered
public holiday/event pricing = calendar-driven
promotion performance = daily / campaign-defined
account price lock review = scheduled before expiry
```

Cadence may depend on:

```text
industry
product category
sales velocity
risk level
stock perishability
event proximity
pricing participation profile
data confidence
```

A cake expiring tomorrow and an apartment unsold for 142 days do not need the same scan rhythm. One goes stale. The other charges interest while looking expensive.

---

## 8. Dynamic Eligibility Check

Before recommending a price move, 81B must check whether dynamic action is allowed.

Dynamic pricing may be blocked or restricted by:

```text
account price lock
contract price
quote validity period
government/regulated price
brand no-discount rule
luxury guardrail
customer-specific agreed price
B2B fixed retail price
promotion already committed
legal/compliance restriction
manual approval requirement
channel agreement
reseller/MAP/RRP policy where applicable
```

Example:

```text
Supplier cost increased, but ABC customer has a 6-month locked price.
81B may recommend a future review.
81B must not override the locked price.
```

Dynamic goblins do not get to break contracts. Even glamorous ones.

---

## 9. Price Movement Types

81B should support more than “raise price” or “lower price.”

Allowed movement recommendations:

```text
hold price
increase price
reduce price
controlled markdown
clearance markdown
private sale
VIP-only offer
bundle offer
free delivery offer
loyalty credit
customer benefit
cashback
price lock review
price lock renewal
pause promotion
change promotion
rollback price
emergency stop
limited experiment
```

Sometimes the correct move is:

```text
change offer, not price
```

Example:

```text
Premium product demand softens.
Do not publicly discount.
Recommend private VIP packaging or loyalty benefit.
```

---

## 10. Demand and Sales Velocity Logic

81B must monitor demand.

Signals:

```text
units sold per hour/day/week/month
conversion rate
cart abandonment
POS sales velocity
E-Commerce views vs purchases
repeat purchase rate
customer segment response
time since last sale
sell-through rate
booking pace
reservation pace
fleet utilization
table occupancy
```

Possible outputs:

```text
demand rising → hold/increase price if allowed
demand falling → offer/markdown review
views high, sales low → price/value issue
sales high, stock low → protect margin
sales low, stock high → markdown/bundle/clearance review
```

---

## 11. Inventory Aging and Lifecycle Logic

81B must react to inventory lifecycle.

Stages:

```text
launch
growth
mature
seasonal
slow-moving
aging
overstock
clearance
obsolete
expiry-risk
scarce
backorder
limited edition
```

Dynamic actions:

```text
launch premium
hold price
raise within guardrail
offer bundle
controlled discount
stage markdown
private clearance
outlet / B2B clearance
urgent expiry markdown
```

Example:

```text
Fashion item is last season and sell-through is slow.
81B recommends staged markdown.
```

Fashion stock ages like milk with better lighting.

---

## 12. Inventory Carrying Cost, Capital Cost + Finance Pressure

This is mandatory.

Stock may accumulate cost while unsold.

81B must include:

```text
overdraft interest
bank facility cost
working capital cost
stock finance cost
floorplan finance
vehicle finance interest
construction loan interest
bank fees
drawdown fees
line fees
holding charges
storage cost
insurance cost
security cost
maintenance cost
cold storage cost
capital tied up
discharge / release fees
sales/agent costs
aging stock cash drag
```

Examples:

```text
Car dealer:
vehicle on floorplan finance for 90 days → finance cost increasing.

Real estate:
unsold apartment has construction interest, holding costs, bank fees, discharge/release costs.

Fashion:
old season stock ties up cash and storage.

Frozen goods:
cold storage cost and expiry risk increase over time.
```

Dynamic pricing may recommend:

```text
hold price if margin still strong
review price after finance cost threshold
controlled markdown
targeted campaign
bundle
clearance
finance-cost recovery adjustment
```

Stock is not free while it waits to be sold. It eats money quietly, like an accountant’s pet.

---

## 13. Overdraft + Working Capital Pricing Logic

If company uses overdraft or working capital finance, pricing must account for cash pressure.

Signals:

```text
overdraft balance
overdraft interest rate
facility limit
near-limit warning
cashflow pressure
upcoming payroll/tax/supplier payments
slow stock
high inventory capital lockup
```

Possible pricing actions:

```text
promote high-margin items
clear stock to release cash
avoid low-margin discounts
use prepayment offers
offer bundles with strong cash margin
recommend account price review
avoid promotions that delay cash
```

Example:

```text
Company is using overdraft and stock is slow-moving.
81B recommends controlled clearance of selected stock to reduce capital pressure.
```

This links to:

```text
Cashflow
Debt / Treasury
Inventory
Accounting
81 Core
```

---

## 14. Expiry / Perishable Markdown Logic

For perishables, Selene must act before waste happens.

Inputs:

```text
expiry date
remaining shelf life
cold-chain status
daily sell-through
current stock
waste risk
discount elasticity
brand/quality risk
customer safety
```

Playbook:

```text
no action
bundle
small markdown
urgent markdown
same-day sale
private offer
staff/internal sale
donation path
waste path
block sale if expired
```

Example:

```text
Fresh cakes expire tomorrow.
Recommend 20% afternoon markdown or bundle with coffee.
If still unsold by closing, route donation/waste workflow.
```

Do not wait until the cake becomes archaeology.

---

## 15. Clearance and Seasonal Markdown Logic

81B must manage staged markdowns.

Clearance triggers:

```text
season ending
last-season stock
limited sizes left
style aging
competitor clearance wave
warehouse/storage pressure
cashflow need
brand-safe clearance route
```

Markdown options:

```text
public markdown
private VIP sale
member-only sale
outlet sale
B2B clearance
bundle clearance
staged markdown
last-chance sale
```

Luxury/premium guardrail:

```text
avoid public discount if brand damage risk is high
use private sale / outlet / VIP offer
```

Example:

```text
Winter jackets are aging after season.
Stage 1: private customer offer.
Stage 2: public markdown if brand allows.
Stage 3: outlet/B2B clearance.
```

Clearance is not panic. It is controlled stock exit.

---

## 16. Holiday, Public Holiday + Special Event Pricing

81B must handle calendar-driven pricing.

Event types:

```text
public holidays
school holidays
religious holidays
national holidays
tourism seasons
city-wide events
sports events
Grand Prix
trade fairs
concerts
conventions
festivals
weather events
local one-off events
```

Example:

```text
Grand Prix is in town from April 14 to April 25.
Hotels, car rentals, flights, restaurants, taxis, and event-adjacent services may all require pricing adjustment.
```

Event pricing must consider:

```text
event start/end date
booking lead time
expected demand
local occupancy/utilization
distance from event
customer segment
minimum stay / rental duration
cancellation rules
competitor market movement
brand/service level
legal restrictions
```

Dynamic actions:

```text
increase rates before peak
minimum stay requirement
package offer
event bundle
higher deposit
limited cancellation
premium delivery/service surcharge
fleet availability surcharge
public holiday labour surcharge
post-event clearance / lower rate
```

Selene must not simply say:

```text
special event = raise price
```

It must calculate:

```text
how much demand exists
how much capacity remains
how close the event is
what competitors are doing
what brand and customer trust allow
```

Otherwise pricing becomes “Grand Prix in town, everybody panic.” Very human. Not Selene.

---

## 17. Hotel Occupancy / Yield Management Logic

Hotel rooms are perishable inventory.

If tonight passes, unsold room revenue disappears.

Inputs:

```text
occupancy percentage
rooms remaining
days until stay date
booking pace
cancellations
room type availability
competitor hotel rates
local events
seasonality
public holidays
minimum stay strategy
package inclusions
brand tier
```

Dynamic logic:

```text
low occupancy far from date → lower rate / package
low occupancy near date → discount / bundle / channel push
medium occupancy → normal rate
high occupancy → raise rate
near full → premium rate
event period → higher starting rate
post-event drop → normalize or clearance-like room fill
```

Example:

```text
Hotel occupancy is 32% for next Friday.
81B recommends lower rate or package.

Occupancy reaches 82%.
81B recommends raising rate and removing discount.

Grand Prix week forecast is high demand.
81B recommends event pricing calendar and minimum-stay rules.
```

Hotel pricing is not selling rooms. It is selling time-sensitive capacity before midnight eats it.

---

## 18. Car Rental Fleet Utilization, Duration + Return-Flow Pricing

Car rental pricing must consider fleet availability and rental duration.

Inputs:

```text
cars available
cars booked
vehicle class
location
weekday/weekend
airport/city demand
return congestion
maintenance schedule
one-way rental
rental duration
insurance/excess package
fuel/charging cost
season/event demand
fleet balancing
```

Duration pricing:

```text
1-day rental = high daily rate
3-day rental = medium daily rate
weekly rental = lower daily rate
monthly rental = low daily rate
```

Return-flow pricing:

```text
most cars return Monday morning
location may become clogged
offer cheap extra day to smooth returns
discount return to alternate location
incentivize longer rental
premium charge for scarce weekend cars
```

Package options:

```text
insurance excess included
premium excess reduction
mileage included
fuel/charging package
child seat / GPS / extras
airport pickup
long-term rental package
weekend bundle
event-week package
```

Example:

```text
Weekend demand high, fleet availability low.
81B recommends higher weekend rate.

Monday morning return congestion predicted.
81B recommends offering customers an extra day at low cost to spread returns.
```

This is pricing as operational control. Fancy little lever. Very useful.

---

## 19. Restaurant Public Holiday / Labour Multiplier Pricing

Restaurants and service businesses may face labour multiplier costs.

Triggers:

```text
public holiday
weekend penalty rates
staff overtime
special event staffing
minimum shift cost
ingredient cost surge
delivery surcharge
table demand
booking demand
```

Inputs:

```text
labour multiplier
menu margin
table occupancy
booking demand
staff roster
holiday calendar
event calendar
ingredient cost
customer segment
brand tolerance
legal/service charge rules
```

Dynamic actions:

```text
public holiday surcharge
special menu
higher booking deposit
reduced menu
premium package
service charge where allowed
limited seating
prepaid booking
normal price next day
```

Example:

```text
Public holiday staff cost is 2.5x.
81B recommends holiday surcharge or fixed holiday menu.
Next day labour cost returns to normal and pricing normalizes.
```

No “holiday vibes surcharge.” It must tie to real costs, policy, and disclosure.

---

## 20. Raw Material Shock + External Crisis Repricing

81B must handle sudden cost shocks.

Triggers:

```text
war
geopolitical disruption
fuel price shock
currency collapse
raw material shortage
shipping disruption
supplier unavailable
import duty change
weather disaster
factory closure
commodity price spike
```

Example:

```text
Manufacturer buys raw material at X.
Tomorrow raw material increases 50%.
```

Possible actions:

```text
hold existing confirmed orders
reprice new quotes
shorten quote validity
apply temporary surcharge
pause quoting
recommend supplier alternatives
review account price locks
notify sales team
route management approval
change product mix
```

Important rule:

```text
Confirmed orders may be protected by existing price.
New quotes may require updated pricing.
Account contracts may require notice/review.
```

Selene must know which business commitments are already locked and which can move.

---

## 21. Market Movement Response

81B reacts to 81A signals.

Market triggers:

```text
competitor drops price
competitor raises price
market band shifts
competitor stockout
new competitor enters
market clearance wave
market shortage
market demand spike
trusted competitor changes package
```

Response options:

```text
ignore if non-comparable
hold price if service/brand justifies
match if commodity and margin safe
use offer instead of price cut
raise price if shortage and brand allows
clearance if market clearing
manual review if data confidence low
```

No chasing a fake competitor price into the gutter. 81A filters the garbage. 81B decides whether movement is needed.

---

## 22. Elasticity Logic

81B estimates how price changes affect demand.

Inputs:

```text
historical price response
promotion test results
customer segment
market type
brand tier
product type
competitor density
substitute pressure
seasonality
```

Outputs:

```text
price-sensitive market
brand/service-sensitive market
premium segment tolerant
discount unlikely to help
small increase likely safe
large decrease may damage margin
```

Example:

```text
10 RMB baijiu competes on price.
5,000 RMB baijiu competes on status, rarity, and gifting.
Different elasticity universes.
```

Selene should know which universe she is standing in before touching price.

---

## 23. B2B Dynamic Pricing Logic

For B2B offers, 81B must preserve B2B economics.

Signals:

```text
provider net changed
channel commission changed
Selene fee changed
reserve/deposit changed
delivery cost changed
return risk increased
provider performance changed
B2B adoption weak
market price shifted
```

Dynamic actions:

```text
raise retail price
recommend provider price review
adjust customer benefit pool
adjust commission if allowed
pause B2B listing
restrict auto-add
clearance through B2B
route approval
```

81E owns B2B pricing stack.

81B detects changed conditions and recommends action.

---

## 24. Customer Segment Dynamic Response

81B may vary offer strategy by customer segment.

Examples:

```text
price-sensitive customer → loyalty credit / bundle / lower-cost option
status customer → premium service / packaging / early access
service-first customer → faster delivery / priority support
warranty-focused customer → warranty extension
business account → price lock / contract review
```

Guardrail:

```text
Prefer dynamic offers and value changes over hidden dynamic base pricing.
```

Selene should make customers feel valued, not hunted. Subtle difference. Very important.

---

## 25. Brand-Safe Dynamic Pricing

81B must respect brand strategy.

For luxury/premium:

```text
public markdown may be blocked
discount may require authority
private sale may be allowed
value-add offer may be preferred
outlet/B2B clearance may be safer
```

For commodity:

```text
price match
bulk discount
clearance markdown
bundle savings
```

Dynamic pricing must match brand identity.

No “Versace flash sale because the stock looks lonely” unless the brand policy explicitly permits it. Even then, maybe whisper.

---

## 26. Channel Conflict Dynamic Check

81B must detect channel conflicts.

Examples:

```text
online price undercuts POS store
B2B price undercuts reseller
marketplace price undercuts company store
account customer sees public price lower than locked price
restaurant app price conflicts with dine-in menu
hotel direct booking conflicts with channel partner rules
car rental app price conflicts with corporate account rate
```

Actions:

```text
block change
route approval
create channel-specific offer
adjust all channels
explain conflict
```

---

## 27. Account Price Lock Dynamic Review

81B must monitor price locks and review cycles.

Checks:

```text
lock expiry date
review date
supplier cost movement
margin erosion
customer volume
contract terms
notice period
renewal opportunity
```

Example:

```text
Account price locked for 6 months.
Supplier cost increased.
81B schedules review 30 days before expiry.
```

Dynamic pricing recommends review. 81 Core enforces lock.

---

## 28. Promotion Performance Reaction

81B reacts to 81F results.

If promotion works:

```text
extend
scale
target similar segment
protect margin
```

If promotion fails:

```text
pause
modify
rollback
change offer type
route review
```

If promotion sells more but lowers profit:

```text
stop or redesign
```

Selling more while earning less is not success. It is a bigger receipt for a smaller brain.

---

## 29. Return / Warranty / Damage Reaction

If after-sale costs increase, 81B must respond.

Signals:

```text
higher return rate
higher warranty claims
higher damage during delivery
higher customer complaints
higher refund cost
fraud risk
```

Actions:

```text
reduce discounting
increase risk cost signal
change offer from discount to warranty/service
pause promotion
recommend product review
recommend provider review
```

Do not promote items that come back like boomerangs with invoices attached.

---

## 30. Supply Constraint Logic

When supply is limited:

```text
protect margin
remove unnecessary discount
allocate to priority customers
offer waitlist
limit quantity
raise price within guardrails
hold for account customers
```

When supply is excessive:

```text
bundle
discount
clearance
B2B clearance
subscription offer
targeted campaign
```

Supply should steer price, but not without margin, brand, and fairness controls.

---

## 31. Dynamic Bundle / Offer Switching

81B may recommend switching offer type.

Examples:

```text
discount not working → try bundle
bundle not working → try free delivery
free delivery too expensive → try loyalty credit
public markdown unsafe → try private VIP offer
low stock → remove discount
event demand high → package instead of discount
hotel low occupancy → room + breakfast package
car rental return congestion → cheap extra day offer
```

Dynamic pricing is not only price movement. It is value movement.

---

## 32. Data Confidence and Safe Defaults

81B must respect data confidence.

Low confidence sources:

```text
stale market data
unknown delivery cost
unknown stock accuracy
unclear customer segment
missing return history
unverified competitor price
uncertain event demand
unreliable occupancy forecast
```

If confidence is low:

```text
hold price
run limited test
route manual review
request market research task
use safe margin
avoid aggressive movement
```

Selene should not guess with spiritual confidence. Humans already do that enough.

---

## 33. Simulation Before Publication

Before major dynamic changes, 81B should simulate.

Simulation fields:

```text
expected unit sales
expected revenue
expected gross margin
expected net margin
stock movement
cashflow impact
return impact
warranty impact
brand risk
channel conflict risk
B2B commission impact
customer reaction risk
event demand scenario
occupancy/utilization scenario
```

Outputs:

```text
recommend publish
recommend limited test
requires approval
not recommended
requires rollback plan
```

---

## 34. Price Movement Guardrails

81B must enforce movement limits.

Examples:

```text
max increase per day/week
max decrease per day/week
max discount depth
minimum margin floor
brand-approved markdown depth
customer notice period
contract price protection
account price lock
event pricing cap where policy applies
requires approval above threshold
```

This prevents Selene from pricing like a panicked stock trader in a supermarket aisle.

---

## 35. Rollback and Emergency Stop

81B must monitor after publishing and trigger rollback if needed.

Rollback triggers:

```text
sales collapse
margin collapse
wrong price
wrong discount
offer stacks incorrectly
brand violation
channel conflict
customer complaints spike
B2B sale unprofitable
tax/duty display problem
holiday/event price failed
occupancy forecast wrong
fleet utilization forecast wrong
```

Actions:

```text
pause price rule
rollback to prior price
block checkout if severe
notify authority
create pricing incident
send audit evidence
```

---

## 36. Human / External Action Orchestration

If 81B needs human review, it must follow the Human / External Action Orchestration Law.

Actions may include:

```text
manager approval for price increase
brand owner approval for markdown
finance approval for below-margin clearance
B2B provider approval for commission change
account manager approval for price lock review
restaurant manager approval for holiday surcharge
hotel revenue manager approval for event pricing
fleet manager approval for rental pricing change
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

No “notify pricing manager.” That phrase has been banned and sent to a farm.

---

## 37. Monitoring After Dynamic Change

After publishing, 81B must monitor:

```text
sales velocity
conversion rate
gross margin
net margin
return rate
customer complaints
stock movement
competitor response
channel conflict
cashflow effect
B2B profitability
occupancy/utilization
event demand actuals
rollback thresholds
```

It must compare actual outcome to simulation.

If actuals differ:

```text
adjust
rollback
run experiment
route review
update learning model
```

---

## 38. Learning Loop

81B must learn from every dynamic pricing action.

Record:

```text
what changed
why changed
trigger source
playbook used
expected outcome
actual outcome
customer segment response
market response
brand impact
profit impact
cashflow impact
rollback needed
future recommendation
audit reference
```

Feeds:

```text
81 Core
81A market trend
81C customer segmentation
81F promotion experimentation
Marketing
Inventory
Cashflow
B2B
Order/POS/E-Commerce
```

Selene should not make the same pricing mistake twice. Humans will handle that department.

---

## 39. Dynamic Pricing Playbooks

81B must maintain playbooks.

Required playbooks:

```text
Expiry Markdown Playbook
Seasonal Clearance Playbook
Slow Stock Playbook
Overstock Playbook
Scarcity / Low Stock Playbook
Hotel Occupancy / Yield Playbook
Car Rental Fleet Utilization Playbook
Restaurant Public Holiday Labour Playbook
Holiday / Special Event Pricing Playbook
Raw Material Shock Playbook
Overdraft / Cash Pressure Playbook
Floorplan Finance / Stock Finance Playbook
Real Estate Holding Cost Playbook
B2B Margin Squeeze Playbook
Promotion Failure Playbook
Return Rate Spike Playbook
Warranty Cost Spike Playbook
Market Price Change Playbook
Account Price Review Playbook
```

Each playbook must define:

```text
trigger
data required
scan cadence
decision rules
allowed actions
approval requirement
publication path
monitoring rule
rollback rule
closure condition
audit reference
```

---

## 40. Outputs from 81B

81B outputs:

```text
dynamic price recommendation
hold price recommendation
markdown recommendation
clearance recommendation
event price recommendation
occupancy/yield recommendation
fleet utilization recommendation
duration pricing recommendation
public holiday/labour surcharge recommendation
raw material shock recommendation
finance pressure recommendation
offer switch recommendation
account price review recommendation
B2B price adjustment recommendation
simulation result
risk flags
approval requirement
rollback condition
monitoring plan
data confidence
business explanation
audit evidence
```

---

## 41. State Machines

### Dynamic Pricing State

```text
Idle
TriggerDetected
EligibilityChecking
DataCollecting
PlaybookSelected
SimulationRequired
SimulationRunning
RecommendationCreated
ApprovalRequired
Approved
Rejected
Published
Monitoring
RollbackRequired
RolledBack
Closed
```

### Event Pricing State

```text
NoEvent
EventDetected
EventWindowCreated
DemandForecasting
EventPricingRecommended
EventPricingApproved
EventPricingActive
EventEnding
PostEventNormalization
Closed
```

### Occupancy / Utilization State

```text
CapacityAvailable
LowUtilization
NormalUtilization
HighUtilization
NearFull
OverDemand
PriceIncreaseRecommended
PackageRecommended
MarkdownRecommended
Closed
```

### Carrying Cost State

```text
NoPressure
CostAccumulating
ThresholdApproaching
ThresholdExceeded
MarkdownRecommended
FinanceCostRecoveryRecommended
ClearanceRecommended
Closed
```

### Rollback State

```text
NotRequired
TriggerDetected
RollbackRecommended
AuthorityRequired
Approved
RolledBack
IncidentLogged
Closed
```

---

## 42. Reason Codes

```text
DYNAMIC_TRIGGER_DETECTED
SUPPLIER_COST_CHANGE_DETECTED
RAW_MATERIAL_SHOCK_DETECTED
MARKET_PRICE_CHANGE_DETECTED
STOCK_LOW_TRIGGER
STOCK_HIGH_TRIGGER
EXPIRY_RISK_TRIGGER
SEASONAL_CLEARANCE_TRIGGER
EVENT_PRICING_TRIGGER
PUBLIC_HOLIDAY_TRIGGER
GRAND_PRIX_EVENT_WINDOW_TRIGGER
HOTEL_OCCUPANCY_LOW
HOTEL_OCCUPANCY_HIGH
CAR_RENTAL_FLEET_LOW_AVAILABILITY
CAR_RENTAL_RETURN_CONGESTION_RISK
RESTAURANT_LABOUR_MULTIPLIER_TRIGGER
OVERDRAFT_COST_PRESSURE_TRIGGER
FLOORPLAN_FINANCE_COST_TRIGGER
REAL_ESTATE_HOLDING_COST_TRIGGER
ACCOUNT_PRICE_LOCK_BLOCKED_DYNAMIC_CHANGE
PRICE_LOCK_REVIEW_SCHEDULED
B2B_MARGIN_SQUEEZE_DETECTED
PROMOTION_UNPROFITABLE
RETURN_RATE_SPIKE
WARRANTY_COST_SPIKE
CHANNEL_CONFLICT_DETECTED
BRAND_GUARDRAIL_BLOCKED_PUBLIC_DISCOUNT
DYNAMIC_SIMULATION_REQUIRED
DYNAMIC_SIMULATION_COMPLETED
PRICE_MOVEMENT_RECOMMENDED
OFFER_SWITCH_RECOMMENDED
ROLLBACK_REQUIRED
ROLLBACK_COMPLETED
DYNAMIC_PRICING_AUDIT_CAPTURED
```

---

## 43. Required Simulations

```text
supplier cost increases and 81B recommends review
account price lock blocks dynamic price increase
product expiring tomorrow triggers markdown playbook
fashion last-season stock triggers staged clearance
luxury product blocks public markdown and recommends private VIP sale
hotel occupancy low triggers package/lower rate
hotel occupancy high triggers rate increase
Grand Prix event window triggers hotel/car rental/event pricing
car rental fleet low availability triggers higher weekend price
car rental Monday return congestion triggers cheap extra-day incentive
restaurant public holiday labour cost triggers surcharge/special menu recommendation
raw material cost increases 50% after crisis and new quotes require repricing
overdraft pressure triggers cashflow-aware clearance recommendation
vehicle floorplan finance cost triggers markdown review
unsold apartment holding cost triggers price/campaign review
B2B commission stack makes dynamic discount unprofitable
promotion sells more but lowers profit and is paused
return rate spike stops aggressive discounting
market data confidence low blocks aggressive price movement
dynamic price published and monitored
wrong discount triggers emergency rollback
```

---

## 44. Integration Map

```text
PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING / DOCUMENT_81_CORE
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.PROCUREMENT
↔ PH1.SUPPLIER
↔ PH1.CASHFLOW
↔ PH1.DEBT_TREASURY
↔ PH1.ACCOUNTING
↔ PH1.B2B_PLATFORM
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.RETURNS
↔ PH1.WARRANTY
↔ PH1.MARKETING
↔ PH1.EVENTS / CALENDAR
↔ PH1.HOTEL / HOSPITALITY_OPERATIONS
↔ PH1.FLEET / CAR_RENTAL
↔ PH1.RESTAURANT / ROSTER / PAYROLL
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

## 45. Required Logical Packets

```text
DynamicTriggerPacket
DynamicPricingEligibilityPacket
DynamicPricingSignalPacket
DynamicPriceRecommendationPacket
PriceMovementGuardrailPacket
DemandVelocityPacket
InventoryLifecyclePricingPacket
CarryingCostPricingPacket
OverdraftPressurePacket
ExpiryMarkdownPacket
ClearanceTriggerPacket
EventPricingPacket
HolidayPricingPacket
OccupancyYieldPacket
FleetUtilizationPricingPacket
RentalDurationPricingPacket
RestaurantLabourMultiplierPacket
RawMaterialShockPacket
B2BDynamicPricingPacket
CustomerSegmentDynamicOfferPacket
BrandSafeDynamicPricingPacket
ChannelConflictPacket
PriceLockReviewPacket
PromotionPerformanceReactionPacket
ReturnWarrantyRiskReactionPacket
SupplyConstraintPricingPacket
DynamicOfferSwitchPacket
DynamicSimulationPacket
DynamicRollbackPacket
DynamicMonitoringPacket
DynamicLearningSignalPacket
DynamicPricingAuditEvidencePacket
```

Logical only.

No runtime packet structs. The pricing goblin may not touch production.

---

## 46. What Codex Must Not Do

```text
Do not make 81B own final price publication.
Do not make 81B override Document 81 Core.
Do not allow dynamic pricing to override account price locks or contract prices without authority.
Do not allow dynamic price changes without trigger source and cadence.
Do not create “if X then maybe Y” without a playbook, action, monitoring, and closure rule.
Do not ignore event calendars, holidays, occupancy, utilization, labour multipliers, or finance carrying costs.
Do not let dynamic pricing damage premium/luxury brand rules.
Do not allow B2B dynamic changes without commission/reserve/fee checks.
Do not publish major price changes without simulation where required.
Do not ignore channel conflict.
Do not ignore rollback/emergency stop rules.
Do not use vague approval/notification without Human / External Action Orchestration.
Do not let GPT-5.5 invent costs, occupancy, utilization, supplier shocks, market events, approvals, or pricing results.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 47. Final Architecture Sentence

Selene Dynamic Pricing Optimization Engine is the pricing pack sub-engine that continuously detects event-driven, scheduled, threshold, market, inventory, demand, capacity, holiday, public-event, labour-cost, raw-material, finance-cost, cashflow, B2B, promotion, return, warranty, and channel signals; selects the correct dynamic pricing playbook; checks eligibility, locks, contracts, brand, margin, fairness, confidence, and approval rules; simulates the recommended price, offer, markdown, package, surcharge, hold, or rollback action; sends the recommendation to Document 81 Core for governed publication; monitors the actual outcome; learns from the result; and prevents Selene from leaving stale prices in a world where demand, costs, stock, events, finance, and humans keep changing everything like inconsiderate little economic gremlins.

Simple version:

```text
81B watches change.
It knows when prices may need to move.
It handles holidays, events, occupancy, fleet availability, labour costs, raw material shocks, finance costs, expiry, clearance, market shifts, and demand changes.
It recommends action.
81 Core approves and publishes.
81B monitors results.
If the price goes wrong, 81B triggers rollback.
Everything is audited.
```

That is 81B: not “dynamic pricing” as a shiny buzzword, but a controlled pricing nervous system for a business that refuses to price hotel rooms, rental cars, cakes, fashion stock, apartments, and toilet paper with the same tragic little markup formula.

---

## 48. 81E Dynamic B2B Recalculation Handoff

81B must request 81E recalculation whenever B2B provider cost, provider net, Channel Store commission, Selene B2B fee, delivery cost, return courier/reverse logistics cost, reserve level, provider risk score, market condition, performance result, refund/return rate, chargeback risk, brand approval outcome, bottom-line profit target, or customer benefit policy changes.

Dynamic B2B pricing cannot publish until 81E confirms pricing-stack viability or routes the proposed change through the required approval path.

---

## 49. 81F-81J Dynamic Pricing Handoff

81B should use 81F promotion results before scaling, repeating, ending, or modifying dynamic promotions, and every dynamic pricing change must be explainable, fair, versioned, and auditable through 81G.

81B capacity, event, service, operational, location, geography, and reverse-logistics triggers must coordinate with 81H and 81I before publication. Dynamic launch, event, promotion, channel, or customer-facing display changes must check 81J presentation readiness where display, claim, perceived value, or product-page framing changes.

Rollback, emergency stop, failed experiment, or dynamic price incident events must create 81G rollback and incident audit evidence.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
