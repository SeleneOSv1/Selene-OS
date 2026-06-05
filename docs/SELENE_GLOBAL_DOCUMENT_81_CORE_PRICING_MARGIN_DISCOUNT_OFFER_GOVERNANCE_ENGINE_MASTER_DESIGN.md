# Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
81

ENGINE:
PH1.PRICING / PH1.MARGIN / PH1.DISCOUNT / PH1.OFFER_GOVERNANCE

FULL NAME:
Selene Core Pricing Intelligence, Margin Strategy, Discount Governance, Offer Governance, Dynamic Price Control, Account Price Lock, Contract Pricing, Clearance, Cost-to-Serve, Capital Carrying Cost, Brand Guardrail Handoff, B2B Pricing Handoff, Customer Value Pricing Handoff, Market Pricing Handoff, Pricing Simulation, Rollback, Explainability, Fairness, and Pricing Audit Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Document 81 is Selene’s **core pricing brain**.

It does not simply add markup.

It does not simply apply discounts.

It does not simply ask:

```text
What is the product cost?
Add 35%.
Done.
```

That is old pricing. A spreadsheet wearing shoes.

Selene Pricing must answer:

```text
What is the correct price, margin, discount, offer, incentive, customer benefit, and profit decision for this product/service, this company, this customer, this channel, this market, this location, and this moment?
```

Pricing in Selene must consider:

```text
cost
landed cost
supplier price
purchase order cost
inventory average cost
capital carrying cost
floorplan finance
construction interest
overdraft cost
holding cost
storage cost
company strategy
product type
service level
brand position
market price
customer value type
location
delivery cost
packaging
special wrapping
warranty
returns
damage
stolen goods / shrinkage
B2B commission
Selene fees
customer benefit pool
payment costs
cashflow
stock level
seasonality
clearance
price locks
contract pricing
promotion performance
fairness
compliance
audit
```

Simple version:

```text
Price is not a number.
Price is a governed business decision.
```

Document 81 is the control center that produces the final pricing decision and coordinates the deeper pricing pack.

---

## 2. Core Pricing Law

```text
Selene must not produce a selling price until it has considered the correct pricing context.

Every price must be:
- cost-aware
- landed-cost-aware
- capital-cost-aware where applicable
- margin-aware
- customer-aware
- company-strategy-aware
- market-aware
- brand-aware
- channel-aware
- B2B-aware where applicable
- service-capability-aware
- delivery/location-aware
- discount/offer-aware
- risk-aware
- explainable
- auditable
```

Selene must protect:

```text
profit
cashflow
customer trust
brand image
contract pricing
account customer agreements
legal/compliance rules
fairness
clear customer communication
```

Selene must prevent:

```text
selling below cost by mistake
selling below true cost-to-serve
ignoring capital/finance holding costs
discount stacking accidents
B2B commission losses
luxury brand damage from dumb discounts
fake clearance
hidden fee surprises
uncontrolled personalized pricing
expired price locks
channel conflict
cashflow-negative promotions
unexplained price changes
```

The goal is not “cheap.”

The goal is:

```text
right price
right value
right margin
right customer treatment
right strategy
right timing
```

A discount is not love. Sometimes it is just margin bleeding in a party hat.

---

## 3. Pricing Pack Structure

Document 81 is the **core pricing engine**.

It coordinates the full Pricing Pack:

```text
81 — Core Pricing, Margin, Discount + Offer Governance Engine

81A — Market Pricing Intelligence + Competitive Research Engine

81B — Dynamic Pricing Optimization Engine

81C — Customer Value Segmentation + Price Sensitivity Engine

81D — Brand Positioning + Premium / Luxury Pricing Guardrail Engine

81E — B2B Profit Share + Commission Pricing Model Engine

81F — Promotion Experimentation + A/B Pricing Governance Engine

81G — Pricing Pack Integration, Explainability, Fairness + Audit Engine

81H — Company Capability, Service-Level, Packaging + Cost-to-Serve Pricing Engine

81I — Geography, Delivery Zone, Local Market + Cost-to-Serve Pricing Engine

81J — Product Presentation, Merchandising, Perceived Value + Offer Packaging Engine
```

Important rule:

```text
81A–81J are sub-documents / pricing pack documents under Global Document 81.
They do not receive new global document numbers unless explicitly decided later.
```

Document 81 owns the final pricing decision.

81A–81J provide specialized intelligence.

No disconnected arms and legs. One pricing creature. Slightly terrifying, but at least it knows where its feet are.

---

## 4. Engine Ownership Boundary

### 4.1 Document 81 owns

```text
final pricing decision control
pricing decision packet
price source intake
true cost-to-serve intake
capital/carrying/finance cost intake
margin guardrails
discount governance
offer governance
price validity
price lock
account/customer contract pricing enforcement
channel pricing rules
clearance and seasonal markdown governance
price override governance
pricing approval need detection
pricing simulation requirement
pricing rollback / emergency stop
data confidence scoring
business pricing explanation
customer pricing explanation
pricing audit
handoff to Order / E-Commerce / POS / B2B / Payment / Accounting / Tax
```

### 4.2 Document 81 references but does not own

```text
product master truth
purchase order truth
supplier invoice truth
inventory stock truth
customer memory truth
B2B provider payout execution
payment capture
refund execution
ledger posting
tax law
market data crawling implementation
promotion campaign creation
customer reward balance
cashflow truth
debt/treasury truth
delivery execution
warehouse operations
```

### 4.3 Correct owner split

```text
Product = product/service identity and product entry cost.
Procurement / PO = supplier purchase price and agreed supplier buying cost.
Inventory = average cost, stock, expiry, aging, reserve status.
Supplier = supplier cost, supplier terms, agreed supplier pricing.
Debt / Treasury = overdraft cost, facility cost, finance cost, capital cost.
Real Estate Accounting / Asset Accounting = construction interest, holding charges, asset/stock carrying costs where applicable.
B2B = provider net, channel commission, Selene fee, reserves, provider settlement.
Customer = customer profile, account terms, standing instructions.
Rewards = points, credits, loyalty, customer benefit balances.
Marketing = campaign strategy and promotional creative.
Pricing = final price, margin, discount, offer, price validity, price explanation.
Order = locks price into order.
POS / E-Commerce = displays and requests price.
Payment = charges money.
Accounting = posts revenue/discount/cost events.
Tax = tax/duty treatment.
Audit = proof.
```

Pricing is the brain.

It is not the bank, warehouse, tax office, or marketing department. Tiny miracle of restraint.

---

## 5. Pricing Source Intake

Selene must know where price starts.

Input sources include:

```text
product entry cost
purchase order cost
last supplier invoice cost
supplier contract price
supplier quote
landed cost
average inventory cost
FIFO / weighted average cost where applicable
B2B provider base price
service labor cost
packaging cost
special wrapping cost
delivery cost
payment processing cost
return cost
warranty cost
storage cost
cold-chain cost
damage/shrinkage cost
customer support cost
commission cost
Selene fee
customer benefit cost
tax / duty estimate
capital carrying cost
finance cost
floorplan finance
construction loan interest
overdraft cost
bank fees
holding charges
insurance while held
security / maintenance while held
discharge / release costs where applicable
```

Core rule:

```text
Cost is the beginning of pricing, not the final price.
```

Product can tell Selene:

```text
This product costs $42 landed.
```

Pricing must decide:

```text
This should sell for $74 with free delivery, no discount, and margin preserved.
```

That is the difference between a price engine and a calculator in a suit.

---

## 6. True Cost-to-Serve Logic

Selene must calculate the real cost of serving this customer/order/channel.

Cost-to-serve includes:

```text
standard delivery
special delivery
same-day delivery
remote delivery
multi-location delivery
gift wrapping
premium packaging
white-glove service
installation
after-sales support
customer service time
returns
refunds
warranty claims
payment fees
installment costs
store account costs
B2B commission
customer benefit pool
damage risk
theft / shrinkage risk
storage cost
cold-chain cost
staff time
service time
product preparation cost
```

Example:

```text
Company A sells handbag in normal packaging.
Company B sells same handbag with premium wrapping, same-day delivery, personal styling note, and gift packaging.

Same product.
Different service value.
Different cost-to-serve.
Potentially different price.
```

Pricing without true cost-to-serve is financial cosplay. Looks professional. Quietly loses money.

---

## 7. Inventory Carrying Cost, Capital Cost + Finance Cost Logic

Some stock accumulates cost while it waits to be sold.

Document 81 must include those costs in pricing, discounting, clearance, and margin decisions.

Examples:

```text
car dealer stock financed through floorplan facility
vehicle sitting on lot with daily finance cost
real estate apartments held as stock for sale
construction loan interest accumulating on unsold apartments
bank drawdown fees / line fees / discharge fees
overdraft interest caused by capital tied in inventory
fashion stock aging across seasons
frozen stock requiring cold storage
machinery requiring storage, maintenance, and insurance
high-value goods requiring security and insurance
```

Car dealer example:

```text
Vehicle cost = purchase cost
+ floorplan finance interest
+ insurance
+ lot holding cost
+ aging stock pressure
```

Real estate example:

```text
Apartment sale price must consider:
land cost
construction cost
construction loan interest
bank fees
drawdown / line fees
discharge / release fees
council/rates/land tax where applicable
insurance
security
utilities
maintenance
sales/marketing cost
agent commission
defect/warranty reserve
holding cost per unsold apartment
target profit
```

Rule:

```text
Stock is not free because it is sitting there.
Some stock has a cost clock.
Pricing must know the clock is running.
```

This logic feeds:

```text
81B Dynamic Pricing
81H Cost-to-Serve
Cashflow
Debt / Treasury
Inventory
Asset / Real Estate Accounting
Document 81 Core
```

If finance/carrying costs erode margin, Selene may recommend:

```text
price review
controlled markdown
targeted campaign
clearance
cashflow-aware offer
finance-cost recovery adjustment
hold price if margin remains strong
```

Inventory can look peaceful while eating the bank account. Charming little parasite.

---

## 8. Company Pricing Strategy Profile

Every Selene company must have a pricing strategy profile.

Strategy types include:

```text
low-cost volume
balanced margin
premium service
luxury / prestige
discount-heavy
relationship-driven
B2B expansion focused
subscription / recurring
local community brand
fast delivery brand
professional service brand
clearance / outlet strategy
no-discount brand protection
```

Company profile includes:

```text
industry
product/service type
target customer
brand position
target margin
discount appetite
service capability
delivery capability
return/warranty risk
B2B participation
commission tolerance
customer reward strategy
country/region
competitive intensity
cashflow posture
capital/carrying cost sensitivity
```

Selene may recommend a strategy.

Company decides.

A supermarket, luxury fashion brand, cake shop, law firm, car dealer, real estate developer, and hardware store should not use the same pricing strategy unless everyone is trying to make consultants rich.

---

## 9. Product / Service Pricing Profile

Each product or service needs its own pricing profile.

Product/service types:

```text
basic commodity
known-value item
premium product
luxury/status product
perishable product
expiry-sensitive product
seasonal product
last-season product
clearance product
warranty product
high-return-risk product
B2B channel product
bundle product
subscription product
professional service
custom service
scarce product
limited-edition product
slow-moving stock
capital-intensive stock
finance-cost-sensitive stock
build-to-order product
```

Examples:

```text
Toilet paper = price-sensitive commodity.
Designer handbag = brand/status product.
Cake = freshness + occasion + presentation value.
Lawyer = expertise + risk + trust.
Bread = freshness, habit, local preference.
Car = model, trim, finance cost, demand, options.
Apartment = development cost, finance cost, location, holding time, release/discharge cost.
```

One markup rule for all products is not pricing. It is margin roulette.

---

## 10. Customer Value Profile

Selene must understand what the customer values.

Customer value profiles include:

```text
price-sensitive
quality-focused
brand/status-driven
service-first
speed/convenience-focused
warranty-focused
loyalty/reward-driven
business/account customer
premium / VIP
bargain hunter
gift buyer
repeat buyer
relationship customer
flexibility-focused
scarcity/access-focused
package-value-focused
```

Selene should use this to improve:

```text
offers
bundles
rewards
service levels
payment options
delivery options
presentation
customer benefit
```

Important guardrail:

```text
Use customer intelligence to improve value.
Be extremely careful with hidden personalized base pricing.
```

Better:

```text
This customer values premium service. Offer gift wrapping and priority delivery.
```

Risky:

```text
This customer will pay more. Secretly raise base price.
```

That second path is how customers start sharpening legal pitchforks while screenshots become evidence.

---

## 11. Market Intelligence Handoff — 81A

Document 81 asks 81A:

```text
What is the market doing?
```

Market signals include:

```text
competitor price
local market price
online market price
regional price
country price
similar product price
premium competitor range
economy competitor range
supply shortage
seasonality
demand trend
market position
```

81A returns:

```text
market range
competitor references
confidence score
premium/economy positioning
recommended market price band
market risk flags
```

Core Pricing uses this to decide whether the price is:

```text
below market
matching market
above market with reason
too cheap
too expensive
premium positioned
clearance positioned
```

---

## 12. Dynamic Pricing Handoff — 81B

Document 81 asks 81B:

```text
Should the price change now?
```

Dynamic triggers:

```text
supplier cost changed
demand increased
demand dropped
stock is low
stock is high
expiry approaching
returns increased
delivery cost changed
market price changed
cashflow pressure
seasonal demand
holiday / special event demand
hotel occupancy
car rental fleet utilization
restaurant public holiday labour cost
raw material shock
overdraft cost pressure
floorplan finance cost
construction/real estate holding cost
promotion performance
competitor changed price
B2B commission changed
```

Dynamic pricing must be governed by:

```text
minimum margin
maximum movement threshold
brand protection
customer fairness
price lock
contract pricing
approval requirement
legal/compliance disclosure
audit
```

Dynamic pricing should never become:

```text
Customer looks desperate. Charge more.
```

That is not pricing intelligence. That is algorithmic mugging with a dashboard.

---

## 13. Customer Segmentation Handoff — 81C

Document 81 asks 81C:

```text
What customer value profile applies here?
```

81C returns:

```text
customer segment
price sensitivity
brand sensitivity
service sensitivity
quality sensitivity
delivery sensitivity
reward preference
scarcity/access preference
flexibility preference
fare/package preference
confidence score
recommended offer type
personalization risk flag
```

Customer segmentation should influence:

```text
offer
bundle
reward
payment option
service level
communication
```

It should not blindly create secret individual prices without governance.

---

## 14. Brand Positioning Handoff — 81D

Document 81 asks 81D:

```text
Would this price, discount, channel, clearance, launch offer, B2B listing, or option package damage the brand?
```

81D returns:

```text
brand tier
sub-brand tier
discount permission
prestige floor
no-discount launch rule
brand-safe offer recommendation
clearance route
B2B channel qualification
official-channel requirement
service capability requirement
brand presentation compliance
venue premium signal
configurable option stack guardrail
brand dilution risk
approval requirement
```

Premium/luxury may require value instead of discount:

```text
VIP packaging
private sale
early access
exclusive event
warranty extension
premium delivery
gift service
personal consultation
rare gift
priority access
```

Discounting status products can destroy the thing people are actually buying: status.

Humans are adorable little brand mammals.

---

## 15. B2B Pricing Handoff — 81E

For B2B-sourced products/services, Document 81 asks 81E:

```text
Can this retail price support everyone who needs to be paid?
```

B2B pricing inputs:

```text
provider net
channel commission
Selene B2B fee
customer benefit pool
warranty reserve
performance reserve
delivery cost
payment cost
return risk
damage risk
retail price
```

81E returns:

```text
minimum viable retail price
recommended retail price
provider margin
channel margin
Selene fee impact
reserve impact
customer benefit funding
B2B profitability flag
```

B2B item must not look profitable while bleeding through commission, delivery, warranty, returns, and reserves.

Reality is rude. Selene should notice.

---

## 16. Promotion Experimentation Handoff — 81F

Document 81 asks 81F:

```text
Which offer actually works?
```

Promotion testing includes:

```text
discount test
bundle test
free delivery test
loyalty credit test
cashback test
VIP gift test
customer segment test
control group
return impact
repeat purchase impact
margin result
brand result
```

Measure:

```text
conversion
revenue
margin
repeat purchase
return rate
customer satisfaction
brand impact
customer lifetime value
profit after costs
```

Selling more while earning less is not success. It is failure with better foot traffic.

---

## 17. Explainability, Fairness + Audit Handoff — 81G

Document 81 asks 81G:

```text
Can this pricing decision be explained, justified, and audited?
```

81G ensures:

```text
pricing decision packet complete
business explanation available
customer explanation available
audit proof exists
fairness flags checked
compliance flags checked
override logged
price version stored
data confidence recorded
```

Magical pricing still needs receipts.

---

## 18. Company Capability Handoff — 81H

Document 81 asks 81H:

```text
What extra value or cost does this company’s capability add?
```

Company capabilities include:

```text
special wrapping
gift packaging
premium packaging
same-day delivery
white-glove delivery
installation
after-sales support
multi-location delivery
customer service level
pickup options
customization
product preparation quality
staff skill
store reputation
service capability
personal shopper
private consultation
beautiful product display
better product photos/videos
```

Same product plus better service may justify a different package and price.

A product in a plastic bag and the same product beautifully wrapped with same-day delivery are not identical offers. One is a product. The other is a little performance with a receipt.

---

## 19. Geography / Delivery Zone Handoff — 81I

Document 81 asks 81I:

```text
What does it cost and mean to serve this customer in this location?
```

Location inputs:

```text
customer location
store location
delivery zone
remote area cost
local market pricing
country/region
currency
duties/taxes
delivery time
service coverage
home vs office
multi-residence delivery
local expectations
venue / event context
```

Guardrail:

```text
Use geography for service cost, delivery cost, local market reality, availability, and tax/duty logic.
Do not use geography as a creepy unfairness machine.
```

Selene may account for the fact that remote delivery costs more.

Selene should not act like a digital snob because of where someone lives.

---

## 20. Product Presentation Handoff — 81J

Document 81 asks 81J:

```text
Does the product presentation support the price?
```

Presentation signals:

```text
photos
videos
product story
brand narrative
copywriting
packaging
display quality
product page quality
reviews
social proof
bundle presentation
in-store display
luxury perception
service framing
```

If presentation is weak, Selene may recommend:

```text
improve photos
create video
rewrite description
improve packaging
change bundle framing
avoid premium price until presentation improves
```

Same cake with a bad photo in a plastic box is not perceived like the same cake with beautiful video, premium box, gift card, and delivery story.

Pricing is perception plus math wearing a suit.

---

## 21. Channel Pricing Logic

Pricing may differ by channel, but must be governed.

Channels include:

```text
E-Commerce
POS
personal Selene commerce
B2B channel
restaurant/table order
market stall
mobile POS
subscription
wholesale
account customer
professional service
event
```

Questions:

```text
same price across all channels?
online-only offer?
POS-only offer?
B2B retail price?
account customer price?
restaurant service price?
delivery included?
```

Selene must detect channel conflicts:

```text
online price undercuts store
B2B price undercuts channel partner
POS price conflicts with account price
retail price conflicts with reseller agreement
marketplace offer harms premium brand
```

Pricing must protect channels, not start a civil war between stores.

---

## 22. Contract / Account Customer Pricing

Some customers have negotiated pricing.

Examples:

```text
account customer discount
trade price
volume price
monthly statement terms
contract price
approved special price
employee price
VIP price
business customer price
B2B partner price
```

Contract/account pricing may override normal dynamic pricing.

Rule:

```text
Contract / account price lock overrides normal dynamic pricing unless authority or contract terms allow review.
```

Selene must never casually break agreed pricing because the dynamic pricing goblin got excited.

---

## 23. Account Price Locks + Review Cycles

Pricing must support period-based price locks.

Examples:

```text
account customer price fixed for 6 months
trade customer price valid until renewal date
VIP price valid for 90 days
B2B reseller price locked until review
```

Fields:

```text
customer/account
product/service/category
locked price
start date
expiry date
review date
notice period
allowed adjustment rules
supplier cost change rules
margin floor
approval requirement
renewal terms
audit reference
```

Example:

```text
ABC Account Customer has toilet paper locked at $18.40 per case until June 30.
Supplier cost increased, but price remains locked unless contract allows review.
Selene schedules price review 30 days before expiry.
```

Dynamic pricing may recommend review, but cannot override lock without authority.

---

## 24. Price Lock and Validity Logic

All prices must have validity where relevant.

Examples:

```text
quote valid for 7 days
cart price valid for 15 minutes
promotion ends tonight
B2B price expires Friday
restaurant lunch price before 3pm
account price lock valid for 6 months
clearance markdown stage valid for 14 days
event/holiday price window
```

Order must check price validity before confirmation.

POS/E-Commerce must refresh stale prices before payment.

No “I saw that price last winter” unless the price lock says so.

---

## 25. Clearance, Seasonal Markdown + Stock Aging

Pricing must support clearance and markdown strategy.

Clearance triggers:

```text
end-of-season
last season stock
aging stock
slow-moving stock
overstock
obsolete product
fashion cycle
expiry risk
storage pressure
cashflow pressure
warehouse clearance
outlet strategy
capital carrying cost pressure
floorplan finance pressure
construction/holding cost pressure
```

Markdown types:

```text
public clearance
private VIP sale
member-only sale
outlet markdown
B2B clearance
bundle clearance
staged markdown
last-chance sale
```

Example:

```text
Fashion store has winter jackets from last season.
Selene recommends staged clearance:
20% off now
35% off in 14 days
move to outlet/B2B if unsold
```

Brand guardrail:

```text
Luxury / premium brands may require private, limited, outlet, or invite-only clearance instead of public discounting.
```

Clearance is not “slash price and cry later.” It is controlled stock exit.

---

## 26. Inventory Lifecycle Pricing

Pricing must consider product lifecycle:

```text
launch
growth
mature
slow-moving
clearance
expiry-risk
obsolete
seasonal
limited edition
scarce
backorder
overstock
capital-cost-sensitive
```

Pricing actions may include:

```text
launch premium
introductory offer
hold price
raise price within guardrails
discount
bundle
clearance
private sale
stock liquidation
finance-cost review
```

Inventory feeds lifecycle signals.

Pricing decides governed price response.

---

## 27. Return / Warranty / Damage-Adjusted Pricing

Pricing must account for risk.

Risk costs include:

```text
return rate
refund rate
damage rate
warranty claims
replacement cost
customer support cost
fraud risk
B2B reserve
category risk
delivery damage risk
```

Example:

```text
Product has high return rate.
Selene may reduce discounting, increase reserve, adjust price, or warn company.
```

Cheap price with high returns may be expensive nonsense.

---

## 28. Cashflow-Aware Pricing

Pricing should consider cashflow posture.

If cashflow is tight, Selene may recommend:

```text
promote high-margin items
clear slow stock
offer bundle with strong cash margin
avoid deep discounts that delay cash
use prepayment offers
use subscription/recurring offers
reduce stock-holding pressure
reduce overdraft/capital cost pressure
```

Cashflow does not blindly dictate price.

It informs strategy.

A business with tight cashflow should not run a promotion that wins customers and loses oxygen.

---

## 29. Tax, Duty + All-In Price Display

Pricing must support:

```text
tax-inclusive pricing
tax-exclusive pricing
GST / VAT / sales tax
import duties
delivery fees
service fees
marketplace fees where disclosed
all-in customer price
```

Customer should see clear final payable amount before payment.

No hidden fee goblins at checkout.

---

## 30. Discount Governance

Discount types include:

```text
percentage discount
fixed discount
bundle discount
quantity discount
loyalty discount
VIP discount
staff discount
damaged item discount
clearance discount
birthday offer
customer benefit pool credit
free delivery
cashback
gift-with-purchase
private sale
account customer discount
```

Pricing must check:

```text
customer eligible?
product eligible?
store/channel eligible?
time valid?
discount stackable?
margin safe?
brand safe?
authority required?
offer expired?
customer price lock conflict?
```

Discounts are not decorations. They are controlled margin decisions.

---

## 31. Offer Strategy

Discount is only one type of value.

Offer types:

```text
discount
bundle
free delivery
reward points
store credit
customer benefit
gift wrapping
VIP packaging
extended warranty
priority delivery
early access
exclusive invitation
service upgrade
personal consultation
rare gift
payment terms
```

Premium/luxury may prefer value-add offers over discounts.

Example:

```text
Status product:
Offer VIP packaging and private event access.

Commodity product:
Offer quantity discount or price match.
```

The right offer depends on product, customer, company, brand, channel, and market.

---

## 32. Promotion Experiment Input

Document 81 must support data coming from 81F.

Pricing should know:

```text
which promotion is active
which customer segment
control group
test group
expected margin
actual margin
return impact
repeat purchase impact
brand impact
```

Pricing must be able to stop or modify promotions that damage profit or brand.

---

## 33. Cross-Product Cannibalization

Pricing must detect when discounting one product harms another.

Examples:

```text
discount old handbags harms premium handbag sales
discount last-season shoes slows new-season shoes
bundle steals sales from higher-margin item
cheap substitute reduces premium product demand
```

Pricing should flag:

```text
cannibalization risk
margin loss risk
brand position risk
stock exit benefit
```

Sometimes cannibalization is acceptable for clearance.

Sometimes it is a self-inflicted wound with a discount sticker.

---

## 34. Customer Lifetime Value Guardrail

Selene may recommend long-term customer value decisions.

Examples:

```text
goodwill credit after bad delivery
VIP retention offer
account customer loyalty protection
new customer welcome benefit
service recovery credit
```

Guardrails:

```text
authority required if margin impact exceeds threshold
record reason
audit benefit
track long-term result
do not repeatedly abuse goodwill
```

Selene may invest in the relationship, but not set the margin on fire while singing about loyalty.

---

## 35. Price Override and Authority

Manual price changes require governance.

Override types:

```text
manager discount
price match
goodwill discount
damaged item discount
customer complaint discount
VIP exception
below-margin approval
staff discount
contract exception
manual B2B price adjustment
capital-cost clearance override
```

Pricing checks:

```text
requester authority
approval route
margin impact
brand impact
contract conflict
B2B settlement impact
tax/accounting impact
audit reason
```

Human / External Action Orchestration applies when approval is required.

No “cashier felt generous” pricing. That is not strategy; that is leakage with a nametag.

---

## 36. Pricing Simulation Before Publishing

Before major price/offer changes, Selene should simulate impact.

Simulation estimates:

```text
expected revenue
expected margin
expected stock movement
expected return rate
customer reaction risk
brand risk
cashflow effect
capital/carrying cost impact
B2B commission effect
channel conflict
clearance speed
```

Simulation outputs:

```text
recommended
not recommended
requires approval
requires limited test
requires rollback plan
```

Do not release pricing into the wild with “seems fine.” Pricing has teeth.

---

## 37. Price Rollback / Emergency Stop

Selene must support pricing kill switch.

Triggers:

```text
wrong discount applied
price below cost
price below true cost-to-serve
offer stacked incorrectly
luxury item publicly discounted by mistake
B2B commission makes sale unprofitable
tax/duty display wrong
contract price violated
currency error
capital-cost logic misapplied
```

Actions:

```text
pause price rule
rollback to previous price
block checkout if dangerous
notify authority
create audit incident
reprice affected carts/orders where allowed
route customer communication if needed
```

One bad pricing rule can turn a business into a charity with inventory. Selene should have brakes.

---

## 38. Data Confidence Scoring

Pricing inputs must have confidence scores.

Examples:

```text
supplier cost current = high confidence
competitor price stale = low confidence
market data uncertain = medium confidence
delivery cost unconfirmed = low confidence
customer segment unknown = low confidence
return risk known = high confidence
capital carrying cost unverified = low confidence
construction interest allocation unconfirmed = low confidence
```

If confidence is low:

```text
recommend review
use safe default
avoid aggressive dynamic pricing
require approval
mark explanation uncertainty
```

No spiritual confidence. Selene should know when she knows.

---

## 39. Currency, Rounding + Local Price Psychology

Pricing must support local market presentation.

Logic includes:

```text
currency conversion
FX fluctuation
rounding rules
tax-inclusive local pricing
psychological pricing
country price endings
cash rounding
minor unit rules
```

Examples:

```text
$19.99
$20.00
€24.95
¥198
10 RMB
5,000 RMB prestige pricing
```

Humans think one cent is magic. Selene should use this knowledge responsibly, or at least profitably.

---

## 40. Fairness, Compliance + Trust

Pricing must protect customer trust.

Guardrails:

```text
no deceptive discounts
no fake original price
no hidden mandatory fee surprise
personalized pricing disclosure where required
no unfair discrimination
respect contract prices
audit dynamic and personalized decisions
clear all-in price where required
```

Customer trust is pricing capital.

Lose it, and every future discount smells suspicious.

---

## 41. Price Communication

Selene needs business and customer explanations.

### Business explanation includes:

```text
cost
landed cost
true cost-to-serve
capital/carrying/finance cost where applicable
margin
market range
strategy
risk costs
brand impact
delivery cost
B2B economics
discount impact
approval need
data confidence
```

### Customer explanation includes:

```text
clear final price
discount applied
bundle savings
delivery cost
tax/duty estimate
points/credits used
offer expiry
account price if applicable
```

Customer does not need to see the whole margin machine.

Business does.

The customer asked for shoes, not a TED Talk from the pricing goblin.

---

## 42. Pricing Decision Packet

Every final pricing decision must produce a Pricing Decision Packet.

Fields:

```text
pricing_decision_id
product_id / service_id
company_id
customer_id / customer_segment_id where applicable
channel
store/location
source cost
landed cost
true cost-to-serve
target margin
market range
brand strategy
product/service profile
company capability value
geography / delivery zone cost
presentation / perceived value signal
B2B cost stack if applicable
delivery cost
return/warranty/damage risk
cashflow signal
capital/carrying/finance cost signal
inventory lifecycle signal
clearance signal
discounts considered
discounts applied
discounts rejected
offer selected
final price
tax/duty display basis
validity period
price lock reference
account/contract reference
approval required
data confidence score
business explanation
customer explanation
audit reference
```

This packet is the glue between 81 and 81A–81J.

Without it, pricing becomes disconnected limbs. And frankly, we’ve all had enough body horror from enterprise software.

---

## 43. State Machines

### Pricing Decision State

```text
Requested
ContextCollected
CostCalculated
MarketChecked
CustomerProfileChecked
BrandChecked
CapabilityChecked
B2BChecked
CapitalCostChecked
MarginChecked
OfferSelected
SimulationRequired
ApprovalRequired
Approved
Rejected
Published
Locked
Expired
RolledBack
Closed
```

### Discount State

```text
NotApplicable
Candidate
EligibilityChecking
StackingChecking
MarginChecking
BrandChecking
Approved
Rejected
RequiresApproval
Applied
Expired
RolledBack
Closed
```

### Price Lock State

```text
NotLocked
LockProposed
Locked
ReviewScheduled
ReviewDue
Reviewed
Renewed
Expired
OverrideRequested
OverrideApproved
Closed
```

### Clearance State

```text
NotClearance
ClearanceCandidate
StageOneMarkdown
StageTwoMarkdown
PrivateSale
OutletChannel
B2BClearance
Liquidation
Closed
```

### Dynamic Pricing State

```text
Stable
TriggerDetected
SimulationRunning
ChangeRecommended
ChangeBlockedByLock
ChangeBlockedByBrand
ChangeRequiresApproval
Published
Monitoring
RollbackRequired
Closed
```

### Override State

```text
NotRequested
Requested
AuthorityChecking
ApprovalRouted
Approved
Rejected
Applied
Audited
Closed
```

### Capital Carrying Cost State

```text
NotApplicable
Applicable
CostAccumulating
ThresholdApproaching
ThresholdExceeded
PriceReviewRequired
MarkdownRecommended
ClearanceRecommended
Closed
```

---

## 44. Reason Codes

```text
PRICING_REQUESTED
PRICING_SOURCE_COST_LOADED
LANDED_COST_CALCULATED
TRUE_COST_TO_SERVE_CALCULATED
CAPITAL_CARRYING_COST_APPLIED
FLOORPLAN_FINANCE_COST_APPLIED
CONSTRUCTION_INTEREST_APPLIED
OVERDRAFT_COST_SIGNAL_APPLIED
HOLDING_CHARGE_APPLIED
COMPANY_STRATEGY_APPLIED
PRODUCT_PRICING_PROFILE_APPLIED
CUSTOMER_VALUE_PROFILE_APPLIED
MARKET_PRICE_RANGE_FOUND
MARKET_DATA_CONFIDENCE_LOW
DYNAMIC_PRICE_TRIGGER_DETECTED
BRAND_GUARDRAIL_APPLIED
PREMIUM_DISCOUNT_BLOCKED
B2B_PRICE_STACK_CALCULATED
B2B_MARGIN_RISK_DETECTED
COMPANY_CAPABILITY_VALUE_ADDED
GEOGRAPHY_COST_TO_SERVE_APPLIED
PRESENTATION_VALUE_SIGNAL_APPLIED
ACCOUNT_PRICE_LOCK_APPLIED
PRICE_LOCK_REVIEW_SCHEDULED
CONTRACT_PRICE_OVERRIDES_DYNAMIC
CLEARANCE_CANDIDATE_DETECTED
SEASONAL_MARKDOWN_RECOMMENDED
CLEARANCE_BRAND_SAFE_ROUTE_REQUIRED
MARGIN_FLOOR_PROTECTED
DISCOUNT_STACKING_BLOCKED
DISCOUNT_APPROVED
DISCOUNT_REJECTED
OFFER_SELECTED_FREE_DELIVERY
OFFER_SELECTED_BUNDLE
OFFER_SELECTED_LOYALTY_CREDIT
PROMOTION_SIMULATION_REQUIRED
PRICING_SIMULATION_COMPLETED
PRICE_OVERRIDE_REQUIRED
PRICE_OVERRIDE_APPROVED
PRICE_OVERRIDE_REJECTED
CHANNEL_CONFLICT_DETECTED
CANNIBALIZATION_RISK_DETECTED
CUSTOMER_LIFETIME_VALUE_EXCEPTION_RECOMMENDED
PRICE_ROLLBACK_REQUIRED
PRICE_ROLLBACK_COMPLETED
CUSTOMER_PRICE_EXPLANATION_CREATED
BUSINESS_PRICE_EXPLANATION_CREATED
PRICING_DECISION_PACKET_CREATED
PRICING_AUDIT_CAPTURED
```

---

## 45. Required Simulations

```text
basic product priced from product cost
product priced from purchase order cost
supplier invoice cost changes price recommendation
landed cost changes final retail price
delivery cost makes item unprofitable
floorplan finance cost triggers vehicle price review
construction interest triggers apartment pricing review
overdraft pressure triggers clearance/cashflow offer review
stock carrying cost triggers markdown review
B2B product requires provider net, commission, Selene fee, and reserve
B2B commission makes price unprofitable
account customer has 6-month locked price
dynamic pricing blocked by account price lock
price review scheduled before lock expiry
fashion last-season stock enters staged clearance
luxury brand blocks public clearance discount
luxury product receives VIP packaging instead of discount
price-sensitive customer receives loyalty credit
status customer receives premium service offer
same product priced differently due to service capability
remote delivery cost changes offer recommendation
poor product presentation blocks premium pricing recommendation
market competitor price range changes recommended price
stock scarcity triggers price hold/increase recommendation
expiry risk triggers markdown recommendation
promotion test shows free delivery beats 10% discount
discount stacking blocked by margin floor
manual price override requires authority
wrong discount triggers rollback
low-confidence competitor data triggers review
online price conflicts with POS/channel price
B2B price conflicts with reseller/channel rule
cross-product cannibalization risk detected
customer lifetime value goodwill credit recommended
customer-facing explanation generated
business-facing explanation generated
pricing decision packet audited
```

---

## 46. Integration Map

```text
PH1.PRICING / MARGIN / DISCOUNT
↔ PH1.PRODUCT
↔ PH1.PROCUREMENT / PURCHASE_ORDER
↔ PH1.SUPPLIER
↔ PH1.INVENTORY
↔ PH1.CASHFLOW
↔ PH1.DEBT_TREASURY
↔ PH1.REAL_ESTATE_ACCOUNTING
↔ PH1.ASSET_ACCOUNTING
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_CREDIT / WALLET
↔ PH1.REWARDS / LOYALTY
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.B2B_PLATFORM
↔ PH1.MARKETING
↔ PH1.TAX
↔ PH1.ACCOUNTING
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.DISPATCH / DELIVERY
↔ PH1.RETURNS
↔ PH1.WARRANTY
↔ PH1.ACCESS / AUTHORITY
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_OPTIMIZATION
↔ PH1.PRICING.81C / CUSTOMER_VALUE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
```

---

## 47. Required Logical Packets

```text
PricingRequestPacket
PricingSourceCostPacket
TrueCostToServePacket
CapitalCarryingCostPacket
FinanceCostPricingPacket
FloorplanFinancePricingPacket
ConstructionInterestPricingPacket
OverdraftCostPricingPacket
CompanyPricingStrategyPacket
ProductPricingProfilePacket
CustomerValueProfilePacket
MarketPricingSignalPacket
DynamicPricingSignalPacket
BrandPricingGuardrailPacket
B2BPricingStackPacket
CompanyCapabilityPricingPacket
GeographyCostToServePacket
ProductPresentationValuePacket
ChannelPricingPacket
ContractAccountPricingPacket
PriceLockPacket
PriceReviewPacket
ClearanceMarkdownPacket
InventoryLifecyclePricingPacket
DiscountEligibilityPacket
DiscountStackingPacket
OfferStrategyPacket
MarginGuardrailPacket
PricingSimulationPacket
PriceRollbackPacket
DataConfidencePacket
PriceOverrideAuthorityPacket
FairnessCompliancePricingPacket
CustomerPriceExplanationPacket
BusinessPriceExplanationPacket
PricingDecisionPacket
PricingAuditEvidencePacket
```

Logical only.

No runtime packet structs. The schema goblin can remain discounted at zero.

---

## 48. What Codex Must Not Do

```text
Do not make Pricing own product master truth.
Do not make Pricing own purchase order truth.
Do not make Pricing own supplier invoice truth.
Do not make Pricing own inventory stock truth.
Do not make Pricing own debt/treasury truth.
Do not make Pricing own customer memory master truth.
Do not make Pricing own B2B provider payout.
Do not make Pricing own payment execution.
Do not make Pricing own accounting ledger posting.
Do not make Pricing own tax law.
Do not allow cost-plus markup as the only pricing logic.
Do not ignore capital carrying costs, floorplan finance, construction interest, overdraft costs, or holding charges where applicable.
Do not allow dynamic pricing to override account price locks without authority.
Do not allow discounts that violate margin guardrails without approval.
Do not allow luxury/premium brand discounting without brand guardrail.
Do not expose business margin to customers.
Do not create hidden personalized base pricing without fairness/compliance governance.
Do not allow deceptive discounts, fake original prices, or hidden fee surprises.
Do not allow B2B prices without commission/reserve/fee cost check.
Do not publish high-impact price changes without simulation where required.
Do not leave broken pricing rules without rollback/emergency stop path.
Do not use vague approval/escalation without Human / External Action Orchestration.
Do not let GPT-5.5 invent costs, market prices, margins, discounts, tax, customer eligibility, capital costs, finance costs, or approval.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 49. Final Architecture Sentence

Selene Core Pricing, Margin, Discount + Offer Governance Engine is the master pricing control layer that produces explainable, auditable, strategy-aware, customer-aware, market-aware, brand-aware, B2B-aware, capability-aware, geography-aware, presentation-aware, capital-cost-aware, margin-protected, fair, dynamic, and governed pricing decisions; using cost inputs from Product, Procurement, Supplier, Inventory, B2B, Delivery, Returns, Warranty, Rewards, Payment, Cashflow, Debt/Treasury, Real Estate Accounting, and Asset Accounting; coordinating specialized pricing intelligence from 81A through 81J; enforcing account price locks, contract pricing, clearance markdowns, capital carrying cost logic, finance-cost pricing, discount rules, channel conflict rules, margin floors, override authority, pricing simulation, rollback, data confidence, customer explanation, business explanation, and audit; while ensuring Selene prices products and services as business decisions rather than dead numbers with coupons stuck to them.

Simple version:

```text
Product tells Selene what it costs.
Finance tells Selene what holding it costs.
Market tells Selene what the world charges.
Company strategy tells Selene what the business wants to be.
Customer profile tells Selene what value matters.
Brand rules tell Selene what not to cheapen.
B2B tells Selene who must be paid.
Capability tells Selene what extra service/value exists.
Geography tells Selene what it costs to serve the customer.
Presentation tells Selene whether perceived value supports the price.
Pricing decides the final price and offer.
Order locks it.
POS/E-Commerce display it.
Payment charges it.
Accounting records it.
Audit proves it.
```

That is Global Document 81 — the pricing monster, now house-trained. It is not a markup calculator. It is Selene’s pricing mastermind: the engine that stops companies from pricing luxury like clearance socks, toilet paper like champagne, and stock financed by banks as if capital costs were imaginary little fairies.

---

## 50. 81E B2B Profit-Share Pricing Handoff

Document 81 must request 81E review before publishing, auto-adding, approving, or dynamically changing any B2B listing, channel adoption, provider net price, Channel Store commission, Selene B2B fee, customer benefit pool, reserve, return/refund policy, brand approval outcome, subscription/recurring B2B offer, service B2B offer, high-value B2B offer, regulated/professional B2B offer, or B2B offer that affects company bottom-line profit targets.

81E owns B2B pricing stack viability, provider net price, Channel Store commission, Selene B2B platform fee, customer benefit funding, warranty/performance reserves, delivery cost, return courier/reverse logistics cost, payment cost, refund/chargeback/damage risk, provider risk score pricing impact, brand approval pricing workflow, B2B price floor, B2B price ceiling, profit-share waterfall, refund/reversal/clawback rules, recurring/service/high-value/regulated/professional B2B pricing, bottom-line profit target alignment, and contribution type classification.

Document 81 remains final pricing governance, but it must not publish a B2B price or offer until 81E returns a viable or approval-routed B2B pricing result.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
