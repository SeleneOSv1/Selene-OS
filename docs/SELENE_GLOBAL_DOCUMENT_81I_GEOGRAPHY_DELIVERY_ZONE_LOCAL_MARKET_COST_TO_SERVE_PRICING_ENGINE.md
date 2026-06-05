# Global Document 81I — Selene Geography, Delivery Zone, Local Market + Cost-to-Serve Pricing Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE SUB-DOCUMENT / PRICING PACK ADDENDUM

PARENT DOCUMENT:
Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

SUB-DOCUMENT:
81I

ENGINE:
PH1.PRICING.81I / PH1.GEOGRAPHY_COST_TO_SERVE / PH1.LOCATION_PRICING

FULL NAME:
Selene Geography, Delivery Zone, Local Market, Venue Premium, Service Area, Territory, Address Validation, Event Location, Delivery Zone, Reverse Logistics, Tax/Duty Region, Currency, Local Labour, Weather, Access Difficulty, Geo-Fencing, Captive Location, Fairness, Confidence, Monitoring, and Geography Cost-to-Serve Pricing Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_PRICING_PACK_SUB_DOCUMENT
```

---

## 1. Purpose

Document 81I is Selene’s **geography, delivery zone, local market, venue, territory, and location cost-to-serve pricing engine**.

It answers:

```text
What does this place do to price, cost, risk, availability, delivery, service, tax/duty, customer expectation, local market value, and fulfillment promise?
```

81I does **not** decide final price.

```text
81I = geography / location / delivery-zone / territory / local market signal.
81 Core = final price / offer / approval / audit.
```

Simple version:

```text
Same product.
Different place.
Different cost.
Different value.
Different risk.
Different promise.
```

A cake across the street, a piano up six flights of stairs, a handbag shipped internationally, a hotel room beside Grand Prix, and a hot dog inside Disneyland are not the same pricing problem. Apparently geography has opinions.

---

## 2. Core 81I Law

```text
Selene must not price, promise, deliver, return, or service anything without considering the geography and location reality.

Every location-sensitive pricing decision must consider:
- customer location
- seller/provider location
- fulfillment location
- delivery zone
- service area
- distance and travel time
- access difficulty
- local market price
- local fees/taxes/duties
- currency and rounding
- courier/service availability
- weather and climate risk
- event/holiday impact
- local labour cost
- return/reverse logistics cost
- B2B territory rules
- official-channel / distributor territories
- geography fairness
- confidence and evidence
```

81I must prevent:

```text
offering delivery where no delivery exists
promising same-day delivery to impossible zones
ignoring tolls, permits, parking, or remote surcharges
hiding location-based fees from customer view
 zones
ignoring tolls, permits, parking, or remote surcharges
hiding location-based fees fromcharging location differences without cost/value/fairness basis
selling into unauthorized territories
ignoring customs/duties
ignoring return courier geography
mispricing captive locations
ignoring event road closures or venue restrictions
```

Geography is not decoration. It is cost, risk, value, legality, and sometimes a giant traffic jam with invoices.

---

## 3. Relationship to Document 81 Core

Document 81 Core owns:

```text
final price
final offer
discount approval
margin guardrail
pricing decision packet
pricing explanation
pricing audit
```

Document 81I owns:

```text
customer/seller/provider location signal
delivery zone signal
service area signal
distance / travel / access cost signal
local market / venue value signal
country / region / currency signal
tax / duty / local fee signal
international delivery risk signal
reverse logistics geography signal
B2B territory / authorized-region signal
event / holiday / weather geography signal
geo-fencing / eligibility signal
location margin impact
geography fairness flag
geography confidence score
geography audit evidence
```

Simple split:

```text
81I says what location does to pricing.
81 Core decides how pricing uses that location signal.
```

---

## 4. Engine Ownership Boundary

### 4.1 81I owns

```text
customer location pricing signal
recipient location pricing signal
seller/provider/branch/warehouse location signal
delivery zone mapping
service area mapping
distance/travel/time cost signal
access difficulty signal
parking/toll/permit signal
time-window geography signal
event/holiday location signal
venue/captive location premium signal
local market pricing signal
currency / FX / rounding signal
customs / duty / import cost signal
international delivery risk signal
reverse logistics geography signal
B2B geography / territory signal
official distributor territory signal
local tax/fee trigger signal
address validation status
multi-address cost signal
courier availability signal
weather/climate risk signal
local labour geography signal
regional expectation signal
location-based eligibility signal
geo-fencing signal
location-based offer signal
fulfillment location cost signal
location margin impact
captive demand guardrail
geography fairness flag
geography confidence score
geography proof/evidence
geography monitoring and learning
```

### 4.2 81I references but does not own

```text
final price decision
tax law
legal/compliance final interpretation
delivery execution
courier booking
inventory routing
order creation
B2B marketplace approval
brand-channel approval
customer memory truth
payment execution
accounting posting
weather service infrastructure
map/geocoder infrastructure
```

### 4.3 Correct owner split

```text
81 Core = final price / offer governance.
81I = geography, location, delivery-zone, territory, and local market cost/value signal.
81B = dynamic pricing timing, occupancy, event demand, fleet utilization.
81H = company capability and service availability.
81D = brand/channel/official territory guardrail.
81E = B2B pricing stack and return courier cost economics.
81A = market/competitor intelligence.
Order = order route and destination execution.
Dispatch/Delivery = actual delivery/courier execution.
Returns = return/reverse logistics workflow.
Tax/Compliance = final tax, duty, and regulatory interpretation.
```

---

## 5. Customer Location

81I must understand where the customer or recipient is.

Location types:

```text
home address
office address
recipient address
delivery site
event location
remote area
city area
rural area
apartment / building
gated community
hotel
hospital
school
warehouse
construction site
restaurant table
airport
stadium / event venue
temporary accommodation
project site
```

Location affects:

```text
delivery cost
delivery time
service availability
return pickup
courier coverage
local tax/fees
access requirements
risk/security
customer expectation
```

---

## 6. Seller / Store / Provider Location

81I must know where the seller/provider can serve from.

Relevant origins:

```text
store location
warehouse location
branch location
provider location
restaurant location
service depot
supplier dispatch point
B2B provider location
repair/service centre
return inspection centre
installation team base
```

Pricing may change depending on:

```text
nearest branch
stock location
service team location
authorized provider territory
delivery route
return destination
```

---

## 7. Delivery Zone Mapping

81I must classify delivery/service zones.

Zones may include:

```text
same-building
same-street
local zone
metro zone
regional zone
remote zone
interstate
international
same-city
same-country
cross-border
restricted area
manual quote zone
official-channel territory
blocked territory
```

Zone outputs:

```text
available
available with surcharge
available with manual quote
available provider-direct only
available referral-only
not available
restricted
```

---

## 8. Distance Pricing

81I must calculate distance-related cost.

Inputs:

```text
distance
travel time
fuel cost
courier cost
toll cost
driver time
vehicle cost
parking time
route complexity
delivery density
multi-stop efficiency
```

Distance is not just kilometres. It is kilometres plus traffic, parking, tolls, access codes, and one apartment elevator that has chosen violence.

---

## 9. Travel Time, Traffic + Route Complexity

81I must account for:

```text
traffic congestion
peak hour
school traffic
CBD delivery delays
roadworks
event road closures
public transport delays
airport access delays
bridge/tunnel delays
ferry routes
rural roads
weather-related route issues
```

Travel time may affect:

```text
same-day eligibility
delivery promise
staff/labour cost
courier cost
service appointment length
customer-facing ETA
```

---

## 10. Access Difficulty

Some locations cost more to serve.

Access factors:

```text
stairs
no elevator
loading dock required
security gate
parking restriction
building access code
concierge handoff
remote driveway
construction site induction
hospital/school delivery protocol
airport security
event venue loading rules
restricted delivery hours
```

Outputs:

```text
access surcharge signal
manual delivery review
white-glove requirement
delivery time extension
failed delivery risk
```

---

## 11. Parking, Tolls + Permits

81I must include location fees.

Examples:

```text
parking fees
tolls
loading zone permits
city access permits
airport access fees
event venue access fees
congestion charges
low-emission zone fees
security access fees
ferry charges
road user charges
```

These costs may feed:

```text
delivery price
service fee
all-in customer price
business margin
B2B cost-to-serve
```

Tiny charges. Big margin goblins.

---

## 12. Same-Day / Urgent Geography

Same-day and urgent service depends on geography.

Inputs:

```text
distance
cutoff time
driver availability
traffic
branch stock
courier coverage
customer zone
weather
access difficulty
service team capacity
```

If same-day is not realistic:

```text
do not offer same-day
offer next-day
manual quote
route to closer branch/provider
offer pickup
```

No fake same-day promises. That is just a refund seed planted in checkout.

---

## 13. Time-Window Pricing

Different time windows may carry different cost and availability.

Time windows:

```text
morning
afternoon
evening
after-hours
weekend
public holiday
event day
specific appointment window
urgent delivery
business-hours only
recipient-selected slot
```

81I provides:

```text
time-window cost signal
time-window availability signal
time-window risk signal
```

81B handles dynamic timing demand.

81H handles company capacity.

---

## 14. Public Holiday Geography

Public holidays may vary by country, state, city, religion, or region.

Effects:

```text
labour cost
delivery availability
courier surcharge
restaurant surcharge
hotel rates
car rental rates
service appointment prices
public transport/traffic changes
restricted operating hours
```

81B handles dynamic holiday trigger.

81I handles local holiday geography/cost reality.

---

## 15. Special Event Location Pricing

81I must detect event-location impact.

Event types:

```text
Grand Prix
concert
festival
trade fair
sports match
conference
tourism event
school holidays
city parade
religious event
local fair
market day
cruise ship arrival
major exhibition
```

81I must know:

```text
event location
event dates
distance from event
road/access restrictions
local demand impact
delivery disruption
venue premium
local accommodation pressure
local transport pressure
```

Example:

```text
Grand Prix is in town April 14–25.
Nearby hotels, car rentals, restaurants, delivery services, venue food, and airport transfers may require location/event pricing signals.
```

---

## 16. Venue / Captive Location Premium

Some locations create built-in pricing power.

Examples:

```text
Disneyland
airport
stadium
concert venue
hotel minibar
theme park
Grand Prix venue
luxury resort
conference centre
festival ground
cruise ship
exclusive venue dining
```

81I must identify when price reflects:

```text
venue
experience
convenience
captive location
event context
limited alternatives
brand environment
occasion value
```

Guardrail:

```text
Captive-location pricing must still be clear, fair, disclosed, and auditable.
```

A Disneyland hot dog is not just a hot dog. It is a small sausage-shaped monument to captive demand.

---

## 17. Local Market Pricing

81I should understand local price expectations.

Signals:

```text
city price level
suburb price level
tourist area pricing
premium district pricing
regional price norms
local competitor density
local customer expectations
local service standards
local wage/cost base
```

Important guardrail:

```text
Use local market context for pricing reality.
Do not use location as a creepy unfairness machine.
```

---

## 18. Country / Region Pricing

81I must support regional differences.

Examples:

```text
country-specific pricing
state/province pricing
regional pricing
urban vs rural pricing
local taxes
local fees
local regulations
currency differences
regional service coverage
region-specific product availability
```

Region affects:

```text
price display
delivery cost
tax/duty
service eligibility
warranty route
official-channel territory
B2B eligibility
```

---

## 19. Currency, FX + Local Rounding

81I provides currency and local price presentation signal.

Inputs:

```text
local currency
currency conversion
FX movement
currency fees
country rounding rules
cash rounding
minor unit rules
psychological pricing norms
tax-inclusive expectation
```

Examples:

```text
$19.99
$20.00
€24.95
¥198
round-number prestige pricing
```

81 Core handles final display.

81I provides regional expectation and FX/location signal.

---

## 20. Duties, Customs + Import Cost

For cross-border transactions, 81I must flag import/export cost.

Inputs:

```text
customs duty
import tax
GST/VAT
brokerage fee
handling fee
export restriction
country-of-origin rule
customs delay risk
import compliance requirement
restricted goods rule
```

Tax/Compliance owns final treatment.

81I signals that geography makes duty/customs relevant.

---

## 21. International Delivery Risk

International delivery may include:

```text
long transit
customs delay
lost parcel risk
damage risk
return difficulty
warranty route difficulty
language/local support issue
currency refund risk
import/export restriction
border disruption
```

81I must produce:

```text
international risk signal
cost-to-serve adjustment
return difficulty signal
customer communication requirement
```

---

## 22. Reverse Logistics Geography

Returns also have geography.

81I must include:

```text
return courier cost
pickup availability
failed pickup cost
international return cost
remote return surcharge
inspection site location
repair site location
restocking location
quarantine location
return-to-provider route
return-to-channel route
```

This feeds:

```text
81E B2B pricing
Document 83 Returns
Payment/Refund
Order
Accounting
```

Return trucks do not run on customer regret alone.

---

## 23. Service Area Coverage

81I must know where service is available.

Service area outputs:

```text
available
unavailable
premium-priced
manual quote required
partner required
provider-direct only
B2B restricted
official-channel only
local pickup only
delivery-only
```

Service area may depend on:

```text
provider
branch
brand
product category
delivery method
staff skill
territory
regulatory region
```

---

## 24. Branch Capability by Location

Some branches can do more than others.

Examples:

```text
Branch A has premium wrapping
Branch B does not
city branch has same-day delivery
regional branch does not
airport branch has premium pricing
warehouse branch has bulk dispatch
hotel location has concierge
branch lacks certified installer
```

81H owns capability.

81I owns location/geography impact.

Together they answer:

```text
Can this company serve this customer this way in this location?
```

---

## 25. B2B Geography Rules

B2B availability may depend on geography.

B2B geography checks:

```text
provider delivery area
Channel Store location
customer location
brand authorized region
official distributor territory
service coverage
franchise territory
local compliance
cross-border restrictions
regional B2B participation
```

81I provides geography/territory signal to:

```text
Document 78 B2B
81D Brand Guardrail
81E B2B Pricing
Order source resolution
E-Commerce display
```

---

## 26. Official Territory / Distributor Rights

Some products can only be sold in approved regions.

Examples:

```text
authorized reseller territory
official distributor zone
franchise area
exclusive region
brand region lock
country-specific warranty
regional product release
regional service agreement
```

81D owns brand/channel guardrail.

81I provides territory/geography signal.

If territory is blocked:

```text
do not display
do not sell direct
use provider-direct route if allowed
route approval
show unavailable
offer alternative
```

---

## 27. Local Tax and Fee Logic

81I should flag local taxes/fees.

Examples:

```text
state tax
city tax
tourism levy
resort fee
environmental fee
delivery surcharge
service fee
local duty
airport fee
venue fee
congestion fee
municipal fee
```

Tax engine owns final tax treatment.

81I says location makes the fee relevant.

All customer-facing mandatory fees must be cleanly disclosed where required. No hidden-fee goblins.

---

## 28. Address Validation

81I must support address confidence.

Address input types:

```text
typed address
voice-captured address
map pin
recipient-provided address
document/photo address capture
contact card
office/site address
hotel address
temporary address
construction/project site
```

Validation outputs:

```text
valid address
invalid address
ambiguous address
partial address
map-pin verified
recipient confirmation required
manual review required
```

This connects to:

```text
Customer Engine
E-Commerce
Order
Dispatch
Returns
Broadcast/Delivery
```

---

## 29. Multi-Address Delivery

Customers may request multiple destinations.

Examples:

```text
send one to mum
send one to office
send half to site A and half to site B
deliver gifts to multiple recipients
ship business supplies to multiple branches
```

81I must calculate:

```text
cost by address
zone by address
tax/fee by address
delivery promise by address
return pickup by address
risk by address
```

Order owns order split.

81I gives location cost/value signals.

---

## 30. Multi-Residence / Multi-Site Customer Logic

Some customers have multiple addresses:

```text
home
office
holiday home
parent’s house
warehouse
client site
project site
hotel stay
temporary accommodation
branch
worksite
```

81I must support context-specific destination pricing.

Example:

```text
Customer may receive groceries at home, business supplies at warehouse, and gifts at mum’s address.
```

Same customer. Different geography. Different cost.

---

## 31. Recipient Location Logic

Recipient address affects:

```text
delivery cost
delivery time
gift privacy
return pickup
courier coverage
notification timing
local service availability
access restrictions
language/timezone
```

Example:

```text
Buyer says “send it to mum.”
Mum’s address determines delivery zone, return pickup cost, and courier coverage.
```

---

## 32. Local Delivery Partner Availability

81I must know which delivery partners can serve which location.

Courier/service partner signals:

```text
operates in zone
cheapest available
most reliable
same-day capable
cold-chain capable
high-value capable
bulky-goods capable
fragile-goods capable
international capable
reverse-logistics capable
signature-required capable
```

Dispatch chooses and executes courier booking.

81I prices geography and availability.

---

## 33. Weather and Climate

Weather affects pricing, delivery, and service risk.

Weather factors:

```text
storms
heatwave
snow
flooding
high wind
extreme cold
fire/smoke
road closures
seasonal rain
cyclone/typhoon risk
```

Effects:

```text
delivery delay
cold-chain risk
perishable risk
outdoor service risk
event service disruption
restaurant/hotel demand
car rental demand
courier surcharge
```

Examples:

```text
heatwave affects cakes and chocolate
storm affects courier
snow affects hotel/car rental demand
flood affects regional delivery
```

---

## 34. Climate-Controlled Geography

Some products are climate-sensitive.

Examples:

```text
cold-chain goods
frozen goods
fresh food
flowers
medicine
cosmetics
wine
chocolate
specialty cheese
temperature-sensitive electronics
```

81I must account for:

```text
temperature risk
distance/time risk
packaging requirement
courier capability
failed delivery risk
return risk
customer communication
```

This connects to 81H packaging/capability and Dispatch/Returns.

---

## 35. Risk / Security by Location

Some locations require special handling.

Risk/security signals:

```text
secure delivery
insured courier
signature required
restricted delivery window
high-value security
theft risk handling
unsafe access flag
concierge-only handoff
ID verification
```

Guardrail:

```text
Use operational evidence and policy.
Do not use location as unfair discrimination.
```

Risk must be explainable, evidence-based, and auditable.

---

## 36. Local Labour Cost

Services may cost differently by location.

Examples:

```text
city labour cost
public holiday labour
regional technician travel
after-hours labour
licensed professional rates
union/award rates
local minimum wage
special event staffing
remote travel time
```

81B handles dynamic labour triggers.

81I provides local labour/cost geography signal.

---

## 37. Local Service Expectations

Customers in different markets may expect different service norms.

Examples:

```text
free delivery
fast delivery
gift wrapping
cash on delivery
scheduled appointment
premium support
included service charge
tax-inclusive price
negotiation
pickup convenience
local-language support
```

81I must provide local expectation signal to 81 Core, 81H, E-Commerce, POS, and Order.

---

## 38. Region-Specific Pricing Psychology

Price display expectations vary by region.

Examples:

```text
$19.99
$20.00
¥198
€24.95
round-number prestige pricing
tax-inclusive expectation
cash rounding
local decimal rules
```

81 Core handles final pricing display.

81I provides regional expectation signal.

---

## 39. Local Competition Density

81I should understand local competition context.

Signals:

```text
many competitors nearby
few competitors nearby
remote monopoly-like market
premium local competitor
discount-heavy local competitor
market gap
local substitute availability
```

81A owns competitor intelligence.

81I adds location context.

---

## 40. Tourism vs Local Customer Context

Pricing context may differ by location type.

Market types:

```text
tourist district
local neighborhood
airport corridor
resort area
business district
event zone
university area
industrial zone
premium suburb
remote town
```

Guardrail:

```text
Use transparent service, venue, demand, and market context.
Do not exploit location unfairly.
```

---

## 41. Hotel / Travel Geography

For hotels/travel, 81I must provide location value signals.

Inputs:

```text
distance to event
distance to airport
distance to CBD
near tourist attraction
near beach
near stadium
near convention centre
view / room location
floor level
transport access
parking availability
walkability
safety/service district
```

81B handles occupancy and yield.

81I handles location value.

A room near the Grand Prix is not the same room in pricing terms as one near absolutely nothing except sadness and a vending machine.

---

## 42. Car Rental Geography

For car rental, 81I must consider:

```text
airport pickup
city pickup
one-way rental
remote drop-off
branch fleet level
return congestion
parking/storage capacity
event-area demand
local road/toll costs
cross-border driving rules
drop-off surcharge
relocation need
```

81B handles utilization/duration.

81I handles pickup/drop-off geography and location cost.

---

## 43. Restaurant Geography

For restaurants, 81I must support:

```text
tourist area
CBD
waterfront
event district
hotel zone
venue proximity
parking difficulty
delivery radius
dine-in vs takeaway zone
outdoor dining weather
local labour/service charge
local foot traffic
```

81B handles dynamic timing/demand.

81I handles location value/cost.

---

## 44. Real Estate Geography

For real estate pricing, 81I must provide location value and cost signals.

Inputs:

```text
suburb
street
view
floor level
school zone
transport access
beach/CBD proximity
development area
local supply
local demand
rates/land tax
council charges
settlement/discharge location cost
local infrastructure
zoning
flood/fire/climate risk
```

Real Estate Accounting handles deeper project finance.

81I provides geography value/cost signal.

---

## 45. Professional Service Geography

For services, geography may define jurisdiction and cost.

Inputs:

```text
jurisdiction
licensing region
court/authority location
client travel
local professional rates
regional compliance
remote/online service option
on-site requirement
travel/disbursement
```

Compliance/Legal owns final jurisdiction/legal interpretation.

81I flags location-based service pricing relevance.

---

## 46. Local Regulatory Restrictions

81I should flag location-based restrictions.

Examples:

```text
delivery restrictions
food safety region rules
professional licensing
vehicle rental rules
event permits
cross-border restrictions
export/import restrictions
hazardous goods rules
regulated product restrictions
venue access rules
```

Compliance/Legal owns final interpretation.

81I says geography makes review required.

---

## 47. Location-Based Eligibility

Some products/services may be:

```text
available
not available
restricted
manual quote required
provider-direct only
B2B restricted
official-channel only
local pickup only
delivery-only
service-only
installation-required
```

81I must send eligibility to:

```text
E-Commerce display
POS
Order
B2B
Pricing
Dispatch
Returns
```

---

## 48. Geo-Fencing + Service Boundaries

81I should support geographic boundaries.

Boundaries:

```text
delivery radius
service radius
branch boundary
franchise territory
brand territory
authorized region
blocked region
manual review zone
tax jurisdiction
event zone
risk zone
```

Geo-fencing must be auditable and explainable.

No invisible “you can’t buy this” without reason, unless legal policy requires quiet blocking.

---

## 49. Location-Based Offer Selection

Location may drive offer choice.

Examples:

```text
free delivery inside zone
delivery surcharge outside zone
pickup discount
regional bundle
event package
hotel transfer package
remote delivery quote
local installation package
same-building delivery discount
venue package
```

81I provides location signal.

81 Core decides final offer.

---

## 50. Local Fulfillment Choice

Selene may choose fulfillment based on location.

Options:

```text
closest stock
cheapest delivery
fastest delivery
highest service reliability
brand-authorized region
customer preferred store
B2B provider territory
branch capability
courier coverage
return convenience
```

Order owns final order routing.

Inventory owns stock truth.

81I provides geography cost/value signal.

---

## 51. Delivery Promise vs Geography

81I must warn when geography makes a promise risky.

Risk factors:

```text
remote location
traffic
weather
event road closure
customs delay
limited courier coverage
access difficulty
public holiday closure
recipient unavailability zone
```

Outputs:

```text
delivery promise safe
delivery promise risky
manual quote required
promise not allowed
alternative delivery suggested
```

---

## 52. Geography-Based Stock Allocation Signal

Location affects stock allocation.

Signals:

```text
which warehouse should serve
which branch should serve
whether stock transfer is needed
whether delivery cost kills margin
whether local pickup is better
whether provider-direct is better
whether local stock should be reserved
```

Inventory/Order own stock/routing.

81I provides location cost signal.

---

## 53. Location-Specific Margin Impact

81I must calculate location margin impact.

Costs may include:

```text
extra delivery cost
extra service cost
extra tax/fee
extra return cost
extra support cost
regional labour cost
local compliance cost
fuel/toll/permit cost
weather-risk cost
access difficulty cost
```

This feeds Document 81 margin and price decision.

---

## 54. Geography Cost-to-Serve

Full geography cost-to-serve includes:

```text
distance
travel time
labour
courier
fuel
parking
tolls
permits
insurance
packing
returns
customs
local fees
security
weather risk
access difficulty
service availability
branch/service capacity
```

This may affect:

```text
price
delivery fee
service fee
offer eligibility
profit margin
B2B viability
return policy
customer communication
```

---

## 55. Location Value Uplift

Some locations increase value, not just cost.

Value uplift examples:

```text
event venue
tourist hotspot
luxury district
airport
resort
stadium
theme park
CBD convenience
premium neighborhood
waterfront
view
school zone
transport hub
exclusive site
```

81I must distinguish:

```text
cost-based price increase
value-based price increase
venue/context premium
market-based premium
```

---

## 56. Captive Demand Guardrail

Captive locations require fairness and transparency.

Captive locations:

```text
airport
theme park
stadium
event venue
hotel minibar
remote resort
cruise ship
conference venue
```

Guardrails:

```text
price difference must be explainable
fees must be disclosed
mandatory fees must not be hidden
customer-facing claim must be truthful
fairness review if risk is high
```

Yes, location can support higher price.

No, Selene should not become a fee goblin in mouse ears.

---

## 57. Price Fairness by Geography

81I must flag geography fairness risks.

Ask:

```text
is price difference cost-based?
market-based?
service-based?
venue-based?
tax-based?
territory-based?
or unfair/discriminatory?
```

If risky:

```text
route 81G fairness review
block or hold if required
create audit evidence
require compliance review
```

Geography is useful. Geography can also become creepy. Selene should choose useful.

---

## 58. Geography Confidence Score

Every geography signal needs confidence.

Confidence inputs:

```text
verified address
estimated address
confirmed courier zone
stale delivery data
manual region
low-confidence location
map-pin verified
carrier quote freshness
event calendar confidence
weather alert confidence
tax/duty source confidence
```

Outputs:

```text
high confidence
medium confidence
low confidence
manual review required
```

Low confidence may require:

```text
manual quote
address confirmation
recipient confirmation
safe default
customer warning
```

---

## 59. Address / Zone Proof

81I should keep evidence.

Proof may include:

```text
address validation
zone lookup
courier quote
tax/duty source
map distance
route estimate
delivery history
SLA history
event calendar
weather alert
territory policy
manual approval
customer/recipient confirmation
```

This feeds 81G audit.

No “delivery surcharge because map vibes.” Show the map goblin’s homework.

---

## 60. Geo-Data Privacy + Minimization

81I must avoid exposing unnecessary location detail.

Rules:

```text
seller sees only location data needed to fulfill
provider sees only allowed delivery/service context
customer support sees customer-safe location explanation
B2B channel does not see unrelated customer addresses
location-derived pricing must be explainable without exposing sensitive assumptions
```

Location data can be personal.

Pricing should not turn it into gossip.

---

## 61. Trigger Logic — How Selene Knows

81I must define when geography checks run.

Triggers:

```text
customer address added
customer address changed
recipient address added
recipient address changed
order created
delivery method selected
pickup/delivery option changed
B2B listing proposed
provider territory checked
brand territory checked
event window detected
weather warning detected
tax/duty region changes
courier quote changes
local fee changes
customer asks for delivery estimate
return request created
service appointment requested
manual quote requested
```

No magic.

Selene knows because events fire and scheduled checks run. We are not consulting a location oracle. Though honestly, maps sometimes feel like one.

---

## 62. Scan Cadence

81I checks may run:

```text
on address entry
on checkout
on order confirmation
on delivery quote
on return request
on service booking
on B2B listing/adoption
daily for event zones
event-driven for weather/road closures
event-driven for courier changes
event-driven for tax/duty changes
weekly/monthly for zone pricing review
before campaign or holiday period
before event pricing window
```

High-risk zones need more frequent checks.

Stable zones need less.

No need to re-ask the cake where it lives every three seconds.

---

## 63. Human / Task Orchestration

If location requires human action, Selene must use Human / External Action Orchestration.

Human/external actions may include:

```text
manual delivery quote
site access confirmation
recipient address confirmation
brand territory approval
customs documentation
event delivery permit
courier escalation
manual tax/duty review
service area override approval
security delivery confirmation
```

Required action fields:

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

No “someone confirm address.” Someone is fictional until Selene assigns them.

---

## 64. Monitoring After Geography-Based Pricing

After location-based pricing or service activation, 81I must monitor:

```text
delivery success
delivery cost variance
return pickup cost
customer complaints
late deliveries
remote surcharge acceptance
venue/event pricing performance
courier reliability
zone profitability
weather disruption
access failure
customs delay
service area exceptions
```

If performance fails:

```text
adjust zone pricing
mark courier unreliable
change delivery promise
route operations review
raise manual quote requirement
update geography confidence
```

---

## 65. Learning Loop

81I learns from geography outcomes.

Learning signals:

```text
which zones cost more than expected
which couriers fail in which areas
which event zones need special pricing
which regions accept premium delivery
which locations cause returns
which zones damage margin
which delivery promises fail
which addresses frequently fail validation
which access types cause delay
which territories create B2B issues
which local fees were missed
```

Feeds:

```text
81 Core
81B Dynamic Pricing
81H Company Capability
81F Promotion Testing
81G Audit
Order
Dispatch
Returns
B2B
E-Commerce
POS
Customer
Marketing
```

---

## 66. Outputs from 81I

81I outputs:

```text
geography cost-to-serve signal
delivery zone signal
distance / travel time signal
access difficulty signal
local market signal
location value uplift signal
venue premium signal
tax/duty region signal
currency/FX signal
service coverage signal
delivery promise risk
return geography cost
B2B territory eligibility
official-channel territory signal
local fee signal
weather/climate risk signal
local labour cost signal
geo-fencing signal
location-based offer signal
geography fairness flag
geography confidence score
geography audit evidence
```

---

## 67. State Machines

### Geography Pricing State

```text
NotStarted
LocationCollected
AddressValidated
ZoneMapped
CostToServeCalculated
ValueSignalCalculated
EligibilityChecked
FairnessChecked
SentTo81Core
Closed
```

### Delivery Zone State

```text
Unknown
Local
Metro
Regional
Remote
International
Restricted
ManualQuoteRequired
Unavailable
Closed
```

### Address Confidence State

```text
Unknown
Partial
Ambiguous
Validated
MapPinVerified
RecipientConfirmed
ManualReviewRequired
Rejected
Closed
```

### Event / Venue Pricing State

```text
NoEvent
EventDetected
EventWindowActive
VenuePremiumDetected
AccessRestrictionDetected
PricingSignalCreated
EventEnded
PostEventReview
Closed
```

### Territory Eligibility State

```text
NotRequired
Checking
Allowed
Restricted
OfficialChannelOnly
ProviderDirectOnly
ReferralOnly
Blocked
ApprovalRequired
Closed
```

### Reverse Logistics Geography State

```text
NotRequired
ReturnRequested
PickupZoneChecked
ReturnCostCalculated
ManualQuoteRequired
PickupUnavailable
ReturnRouteCreated
Closed
```

---

## 68. Reason Codes

```text
GEOGRAPHY_LOCATION_COLLECTED
CUSTOMER_ADDRESS_VALIDATED
RECIPIENT_ADDRESS_VALIDATED
DELIVERY_ZONE_MAPPED
DISTANCE_COST_CALCULATED
TRAVEL_TIME_COST_CALCULATED
ACCESS_DIFFICULTY_DETECTED
PARKING_TOLL_PERMIT_COST_APPLIED
SAME_DAY_GEOGRAPHY_APPROVED
SAME_DAY_GEOGRAPHY_BLOCKED
TIME_WINDOW_COST_APPLIED
PUBLIC_HOLIDAY_LOCATION_SIGNAL_CREATED
SPECIAL_EVENT_LOCATION_SIGNAL_CREATED
GRAND_PRIX_EVENT_ZONE_DETECTED
VENUE_PREMIUM_SIGNAL_CREATED
LOCAL_MARKET_SIGNAL_CREATED
COUNTRY_REGION_PRICING_SIGNAL_CREATED
CURRENCY_FX_SIGNAL_CREATED
DUTY_CUSTOMS_SIGNAL_CREATED
INTERNATIONAL_DELIVERY_RISK_CREATED
REVERSE_LOGISTICS_COST_CALCULATED
SERVICE_AREA_AVAILABLE
SERVICE_AREA_UNAVAILABLE
B2B_TERRITORY_CHECK_REQUIRED
OFFICIAL_DISTRIBUTOR_TERRITORY_APPLIED
LOCAL_TAX_FEE_SIGNAL_CREATED
MULTI_ADDRESS_COST_CALCULATED
RECIPIENT_LOCATION_SIGNAL_CREATED
COURIER_AVAILABILITY_CHECKED
WEATHER_RISK_SIGNAL_CREATED
COLD_CHAIN_GEOGRAPHY_RISK_CREATED
LOCATION_SECURITY_RISK_FLAGGED
LOCAL_LABOUR_COST_SIGNAL_CREATED
LOCAL_EXPECTATION_SIGNAL_CREATED
REGIONAL_PRICE_PSYCHOLOGY_SIGNAL_CREATED
TOURISM_ZONE_SIGNAL_CREATED
HOTEL_TRAVEL_LOCATION_VALUE_CREATED
CAR_RENTAL_LOCATION_SIGNAL_CREATED
RESTAURANT_LOCATION_SIGNAL_CREATED
REAL_ESTATE_LOCATION_VALUE_CREATED
PROFESSIONAL_SERVICE_JURISDICTION_SIGNAL_CREATED
LOCAL_REGULATORY_REVIEW_REQUIRED
GEOFENCE_APPLIED
LOCATION_BASED_OFFER_SIGNAL_CREATED
DELIVERY_PROMISE_RISK_CREATED
LOCATION_MARGIN_IMPACT_CREATED
CAPTIVE_DEMAND_GUARDRAIL_REQUIRED
GEOGRAPHY_FAIRNESS_REVIEW_REQUIRED
GEOGRAPHY_CONFIDENCE_LOW
GEOGRAPHY_AUDIT_CAPTURED
```

---

## 69. Required Simulations

```text
customer address added and delivery zone mapped
recipient address for mum creates separate delivery cost
same-day delivery blocked because customer is outside courier zone
apartment with no elevator adds access difficulty signal
CBD delivery adds parking/toll/permit cost
Grand Prix event window increases local hotel/car rental/location signals
Disneyland hot dog triggers venue/captive premium guardrail
international handbag order adds duty/customs and return difficulty
remote return pickup requires manual quote
B2B product blocked by official distributor territory
brand item provider-direct only due to regional authorization
multi-address order calculates cost for home, office, and warehouse
weather alert blocks cake delivery route
cold-chain product requires upgraded packaging and courier
hotel near stadium creates location value uplift
car rental one-way remote drop-off adds surcharge
restaurant waterfront location creates local market premium signal
real estate apartment view/floor/location affects pricing signal
professional service jurisdiction requires compliance review
location-based price difference routes fairness review
low-confidence map pin requires recipient confirmation
cross-border return cost feeds 81E B2B pricing
geography signal audited and sent to 81 Core
```

---

## 70. Integration Map

```text
PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING / DOCUMENT_81_CORE
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.CUSTOMER
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.B2B_PLATFORM
↔ PH1.PRODUCT
↔ PH1.INVENTORY
↔ PH1.DISPATCH / DELIVERY
↔ PH1.RETURNS
↔ PH1.COURIER / LOGISTICS
↔ PH1.SCHEDULER / ROSTERS
↔ PH1.TASK / HUMAN_WORKLOAD
↔ PH1.HOTEL / HOSPITALITY
↔ PH1.FLEET / CAR_RENTAL
↔ PH1.RESTAURANT
↔ PH1.REAL_ESTATE
↔ PH1.PROFESSIONAL_SERVICES
↔ PH1.TAX
↔ PH1.LEGAL
↔ PH1.COMPLIANCE
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.ACCESS / AUTHORITY
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
```

---

## 71. Required Logical Packets

```text
GeographyCostToServePacket
CustomerLocationPacket
RecipientLocationPacket
SellerProviderLocationPacket
DeliveryZonePacket
DistanceTravelCostPacket
AccessDifficultyPacket
ParkingTollPermitPacket
TimeWindowGeographyPacket
HolidayLocationPacket
SpecialEventLocationPacket
VenuePremiumPacket
LocalMarketSignalPacket
CountryRegionPricingPacket
CurrencyFXPacket
DutyCustomsPacket
InternationalDeliveryRiskPacket
ReverseLogisticsGeographyPacket
ServiceAreaCoveragePacket
BranchLocationCapabilityPacket
B2BTerritoryPacket
OfficialTerritoryPacket
LocalTaxFeePacket
AddressValidationPacket
MultiAddressDeliveryPacket
MultiResidencePacket
CourierAvailabilityPacket
WeatherClimateRiskPacket
ColdChainGeographyPacket
LocationSecurityRiskPacket
LocalLabourCostPacket
LocalExpectationPacket
RegionalPricingPsychologyPacket
GeoFencePacket
LocationEligibilityPacket
LocationBasedOfferPacket
LocalFulfillmentSignalPacket
DeliveryPromiseGeographyRiskPacket
LocationMarginImpactPacket
CaptiveDemandGuardrailPacket
GeographyFairnessPacket
GeographyConfidencePacket
GeographyAuditEvidencePacket
```

Logical only.

No runtime packet structs. The map goblin may not deploy itself.

---

## 72. What Codex Must Not Do

```text
Do not make 81I decide final price.
Do not make 81I override Document 81 Core.
Do not make 81I own tax law or compliance final interpretation.
Do not make 81I own delivery execution.
Do not make 81I own courier booking.
Do not make 81I own inventory routing.
Do not use geography for unfair or discriminatory pricing.
Do not apply location surcharge without cost/value/fee/market basis.
Do not hide mandatory location-based fees from customer-facing price where disclosure is required.
Do not offer unavailable delivery/service zones.
Do not promise same-day or time-window delivery without geography/capacity check.
Do not ignore reverse logistics geography.
Do not ignore official territory/distributor restrictions.
Do not treat low-confidence address/location data as verified.
Do not create vague human tasks without Human / External Action Orchestration.
Do not let GPT-5.5 invent address, territory, tax, duty, courier, event, weather, or delivery-zone facts.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 73. Final Architecture Sentence

Selene Geography, Delivery Zone, Local Market + Cost-to-Serve Pricing Engine is the pricing pack sub-engine that determines what customer location, recipient location, seller/provider location, delivery zone, distance, travel time, access difficulty, parking, tolls, permits, weather, climate, event windows, public holidays, venue/captive-location value, country/region, currency, duties, customs, local tax/fees, service coverage, reverse logistics, B2B territory, official distributor rights, local labour, local expectations, and geo-fencing do to price, availability, margin, customer promise, return cost, and fairness; then sends geography cost-to-serve, value uplift, risk, eligibility, confidence, and audit signals to Document 81 Core and related commerce engines.

Simple version:

```text
81I tells Selene what location does to price.

It asks:
Where is the customer?
Where is the seller?
Where is the provider?
Where is the stock?
Where is the delivery going?
Can we serve it?
What does it cost?
What value does the place add?
What local fees/taxes/duties apply?
Can it be returned?
Is it fair?
Can we prove it?

81I does not set final price.
It gives Selene the geography truth behind the price.
```

That is 81I: the engine that stops Selene from pretending a local bakery delivery, an airport food purchase, a remote courier route, a Grand Prix hotel room, an international luxury shipment, a return pickup from mum’s house, and an apartment with a sea view are all the same prici

g problem. Because they are not. Geography, annoyingly, matters.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
