# Global Document 81C — Selene Customer Value Segmentation + Price Sensitivity Engine

```text id="doc81c_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE SUB-DOCUMENT / PRICING PACK ADDENDUM

PARENT DOCUMENT:
Global Document 81 — Selene Core Pricing, Margin, Discount + Offer Governance Engine

SUB-DOCUMENT:
81C

ENGINE:
PH1.PRICING.81C / PH1.CUSTOMER_VALUE_SEGMENTATION / PH1.PRICE_SENSITIVITY

FULL NAME:
Selene Customer Value Segmentation, Price Sensitivity, Quality Sensitivity, Brand/Status Sensitivity, Service Preference, Loyalty/Rewards Response, Travel/Fare Package Preference, No-Discount Launch Treatment, Scarcity Access, Gift Context, Business Account, Offer Preference, Ethical Guardrail, Confidence Scoring, Trigger Automation, and Customer Pricing Intelligence Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_PRICING_PACK_SUB_DOCUMENT
```

---

## 1. Purpose

Document 81C is Selene’s **customer value intelligence engine** inside the pricing pack.

It answers:

```text id="81c_main_question"
What kind of value does this customer care about in this specific buying context?
```

It does **not** decide final price.

```text id="81c_simple_split"
81C = understands customer value logic.
81 Core = decides final price / offer / margin / approval / audit.
```

A customer is not one fixed pricing type forever.

The same customer may be:

```text id="81c_customer_context_example"
price-sensitive for toilet paper
quality-sensitive for food
brand/status-sensitive for handbags
speed-sensitive for airport car rental
flexibility-sensitive for airline tickets
service-sensitive for hotels
warranty-sensitive for electronics
emotionally reckless for gifts
```

So 81C must classify customer value by:

```text id="81c_classification_scope"
customer
product category
service category
purchase context
recipient
urgency
channel
location
season/event
business/account relationship
confidence level
```

No shoving humans into one dumb bucket like “cheap customer.” That is spreadsheet astrology, and we are already suffering enough.

---

## 2. Core 81C Law

```text id="81c_core_law"
Selene must not assume all customers value the same thing.

Selene must infer, confirm, and update customer value profiles using actual behavior, declared preferences, context, category, purchase history, offer response, payment choices, and confidence scoring.

Selene must use customer value intelligence to improve offers, bundles, service, delivery, rewards, payment options, access, and customer experience.

Selene must not use protected or sensitive attributes for unfair pricing.

Selene must be extremely careful with hidden personalized base pricing.
```

Preferred Selene behavior:

```text id="81c_preferred"
same transparent base price where appropriate
personalized value
personalized offer
personalized bundle
personalized reward
personalized service level
personalized payment option
```

High-risk behavior:

```text id="81c_high_risk"
secretly charging more because Selene believes the customer will pay more
```

That is not genius. That is a future complaint with screenshots.

---

## 3. Relationship to Document 81 Core

Document 81 Core owns:

```text id="81c_core_owns"
final price
final discount
final offer
margin guardrail
customer explanation
business explanation
approval routing
audit
```

Document 81C owns:

```text id="81c_owns"
customer value segment detection
category-specific price sensitivity
context-specific value preference
declared preference capture
offer preference detection
substitution tolerance signal
payment preference signal
scarcity/access preference
flexibility preference
gift/recipient value adjustment
airline/travel fare preference signal
business/account customer pricing signal
confidence scoring
fairness/protected-attribute guardrails
customer value learning loop
customer value signal packet to Document 81
```

Simple version:

```text id="81c_to_81"
81C tells Pricing what the customer is likely to value.
81 decides what price or offer is allowed.
```

---

## 4. Engine Ownership Boundary

### 4.1 81C owns

```text id="81c_owns_list"
customer value profile
category-specific value profile
context-specific value profile
price sensitivity score
quality sensitivity score
brand/status sensitivity score
service/convenience sensitivity score
warranty/trust sensitivity score
loyalty/reward sensitivity score
fare/package sensitivity score
scarcity/access sensitivity score
flexibility/commitment sensitivity score
ancillary/add-on preference
gift/recipient value profile
spend threshold signal
substitution tolerance signal
offer type preference
payment preference signal
urgency/time sensitivity signal
customer lifetime value signal
cold-start segmentation
declared preference intake
confidence scoring
ethical/fairness guardrails
learning triggers
value-segmentation audit evidence
```

### 4.2 81C references but does not own

```text id="81c_not_own"
final price
discount approval
margin rule
brand policy
dynamic price movement
B2B commission
promotion campaign design
reward balance
payment execution
order creation
customer memory master truth
protected identity rules
legal/compliance final interpretation
```

### 4.3 Correct owner split

```text id="81c_owner_split"
81 Core = final price and offer governance.
81A = market / competitor intelligence.
81B = dynamic pricing movement.
81C = customer value and price sensitivity.
81D = brand / luxury guardrail.
81E = B2B commission and profit-share pricing.
81F = promotion experiments.
81G = explainability, fairness, audit.
81H = company capability and service-level value.
81I = geography and delivery-zone cost-to-serve.
81J = product presentation and perceived value.
Customer Engine = durable customer identity, relationship, permissions, memory.
Rewards Engine = points, credits, loyalty, benefit balances.
Payment/Wallet = payment methods, wallet, store accounts, credit.
Order/E-Commerce/POS = live shopping and order contexts.
```

81C is not allowed to become the whole customer brain. It is the pricing-relevant value brain. Tiny but important distinction, because otherwise every document tries to become Selene’s entire personality.

---

## 5. Customer Value Types

81C must classify customer value orientation.

Possible value types:

```text id="81c_value_types"
price-sensitive
quality-focused
brand/status-driven
service-first
speed/convenience-focused
warranty/security-focused
loyalty/reward-driven
VIP/premium
business/account customer
bargain hunter
gift buyer
repeat buyer
relationship customer
subscription-friendly
risk-averse
deal-seeker
time-poor
flexibility-focused
scarcity/access-focused
ancillary/add-on focused
package-value focused
```

The output is not one label.

It is a weighted profile:

```text id="81c_weighted_profile"
price sensitivity: high
brand sensitivity: low
service sensitivity: medium
warranty sensitivity: high
confidence: 78%
category: electronics
context: self-purchase
```

Different category, different result.

Because humans are inconsistent little decision engines wrapped in skin.

---

## 6. Category-Specific Segmentation

81C must segment by category.

Categories may include:

```text id="81c_categories"
groceries
fashion
luxury goods
electronics
restaurants
hotels
airlines
car rental
professional services
beauty/salon
hardware
business supplies
events
subscriptions
real estate
vehicles
travel
gifts
```

Example:

```text id="81c_category_example"
Customer is:
price-sensitive for groceries
brand-sensitive for shoes
service-sensitive for hotels
flexibility-sensitive for airline tickets
warranty-sensitive for electronics
```

Selene must not apply grocery logic to handbags. That is how luxury becomes clearance socks.

---

## 7. Context-Specific Segmentation

81C must understand why the customer is buying.

Purchase contexts:

```text id="81c_contexts"
personal purchase
gift
business purchase
event purchase
urgent need
family purchase
romantic purchase
corporate/client gift
replacement purchase
routine grocery
special occasion
emergency purchase
travel purchase
status purchase
subscription renewal
```

Example:

```text id="81c_context_example"
Customer normally buys cheap wine.
For anniversary dinner, customer may want premium wine.
```

Context can override habit.

Selene must detect that “wine for tonight alone” and “wine for client dinner” are not the same little grape problem.

---

## 8. Price Sensitivity Signals

81C learns price sensitivity from:

```text id="81c_price_signals"
sorting by cheapest
waiting for sales
using coupons
abandoning cart after price
choosing lower-cost substitutes
rejecting delivery fees
buying bulk for savings
using points before cash
asking for cheaper options
responding to cashback
choosing non-refundable cheaper fare/package
switching stores for lower price
declining premium options
```

Trigger examples:

```text id="81c_price_trigger_examples"
Customer repeatedly filters “lowest price” in groceries.
Customer abandons cart after delivery fee appears.
Customer accepts cheaper substitute three times.
Customer chooses non-refundable fare for airline bookings.
```

81C updates category/context sensitivity with confidence.

---

## 9. Quality Sensitivity Signals

81C learns quality sensitivity from:

```text id="81c_quality_signals"
choosing higher-rated products
rejecting cheap alternatives
preferring premium materials
asking about ingredients
asking about freshness
asking about durability
choosing better warranty
preferring certified products
choosing qualified professionals
returning poor-quality substitutes
```

Example:

```text id="81c_quality_example"
Customer pays more for fresh bakery bread but buys budget household goods.
```

81C should feed:

```text id="81c_quality_feeds"
81 Core offer strategy
81J product presentation
E-Commerce recommendations
Order substitution decisions
```

---

## 10. Brand / Status Sensitivity

81C must detect when customer values brand/status.

Signals:

```text id="81c_status_signals"
buys luxury brands
prefers designer labels
values exclusivity
joins waitlists
buys limited editions
responds to VIP access
cares about packaging
rejects generic alternatives
buys premium gifts
chooses status hotels/restaurants
prefers premium cabin/seat/class
```

Example:

```text id="81c_status_example"
Customer wants latest iPhone.
No discount applies.
Selene should offer priority access, rare gift, setup service, premium case, or loyalty benefit.
```

Not:

```text id="81c_bad_status_offer"
Here is 3% off, please enjoy your prestige product now looking like clearance stock.
```

---

## 11. Launch / No-Discount Customer Value Treatment

Some products should not be discounted.

Examples:

```text id="81c_no_discount_examples"
latest iPhone
new model luxury handbag
limited edition sneaker
new-season fashion drop
premium watch
high-demand event ticket
scarce hotel/event package
exclusive restaurant booking
```

81C must identify customers who value:

```text id="81c_no_discount_values"
being first
early access
scarcity
reserved stock
rare gift
premium accessory
VIP setup
priority delivery
exclusive invitation
private appointment
status
```

Output to 81 Core:

```text id="81c_launch_output"
No-discount product.
Customer likely values access/status.
Recommend value-add, not price reduction.
```

Example:

```text id="81c_launch_example"
Do not discount latest iPhone.
Offer premium case, priority delivery, setup service, or rare gift to VIP customers.
```

Selene treats the customer like gold without cheapening the product. Fancy. Sensible. Disturbingly rare.

---

## 12. Service and Convenience Sensitivity

81C must identify customers who value ease and speed.

Signals:

```text id="81c_service_signals"
chooses same-day delivery
pays for faster shipping
uses concierge service
prefers easy returns
requests installation
chooses white-glove delivery
uses personal support
uses multi-address delivery
chooses pickup for speed
accepts higher price for convenience
```

Offer implications:

```text id="81c_service_offers"
priority delivery
installation included
done-for-me setup
concierge support
premium return experience
multi-location delivery
personal shopper
```

Some customers are not buying the product. They are buying “make this easy before I lose patience.”

---

## 13. Warranty / Trust Sensitivity

81C must identify customers who value reduced risk.

Signals:

```text id="81c_trust_signals"
chooses trusted sellers
pays more for warranty
asks about returns
chooses authorized provider
chooses reputable hotel/airline/car rental
rejects grey-market listings
chooses insurance/excess cover
prefers professional certification
chooses high-review providers
```

Examples:

```text id="81c_trust_examples"
Customer pays more for Hilton because trust, service, reliability, and brand reduce risk.
Customer chooses car rental package with insurance excess included.
Customer chooses electronics from authorized seller.
```

Offer implications:

```text id="81c_trust_offers"
extended warranty
easier returns
authorized seller proof
insurance/excess cover
service guarantee
trusted provider badge
```

---

## 14. Loyalty / Rewards Sensitivity

81C must know whether customer responds to rewards.

Signals:

```text id="81c_rewards_signals"
uses points
chooses cashback
uses store credit
waits for birthday credits
responds to free delivery
values membership pricing
accepts loyalty tier offers
uses referral rewards
uses customer benefit pool
```

Offer types:

```text id="81c_rewards_offers"
points
cashback
store credit
birthday credit
free delivery
VIP tier
future credit
customer benefit pool
```

Some customers want price off now.

Others want points because points feel like little victory stickers for adults.

---

## 15. Business / Account Customer Segmentation

81C must identify business/account context.

Business customer types:

```text id="81c_business_types"
consumer customer
business customer
trade account
VIP account
contract customer
employee buyer
approver buyer
procurement buyer
recipient buyer
corporate account
department buyer
project buyer
```

Business/account signals:

```text id="81c_business_signals"
contract pricing
account terms
volume discount
price lock
monthly statement
purchase pattern
preferred suppliers
department budget
approval limits
trade pricing
corporate invoicing
recurring orders
```

Output:

```text id="81c_business_output"
respect account terms
respect contract pricing
respect price lock
suggest volume / package / service terms
do not override with normal consumer offer
```

Business buyers may value payment terms more than a small discount. Astonishingly, cashflow matters.

---

## 16. Gift / Recipient Logic

A customer may buy differently for someone else.

Recipient contexts:

```text id="81c_recipient_contexts"
self
mum
wife/husband
children
friend
client
employee
business partner
romantic partner
corporate gift
event guest
```

Gift value signals:

```text id="81c_gift_signals"
premium packaging
gift wrapping
hide price
delivery timing
recipient preferences
status signal
emotional value
personal note
brand choice
occasion
```

Example:

```text id="81c_gift_example"
Customer buys budget snacks for self,
premium chocolate for mum,
luxury bottle for client.
```

Selene must not assume self-purchase behavior applies to gifts.

Humans become financially dramatic when affection is involved.

---

## 17. Airline / Travel Customer Segmentation

81C must support airline/travel value profiles.

Passenger types:

```text id="81c_airline_types"
business traveler
leisure traveler
family traveler
last-minute urgent traveler
price-sensitive traveler
loyalty/status traveler
flexible-date traveler
premium cabin traveler
baggage-heavy traveler
low-cost/no-frills traveler
corporate account traveler
event traveler
frequent commuter
```

Travel value preferences:

```text id="81c_airline_values"
lowest fare
direct flight
shorter travel time
flexible cancellation
date changes
seat selection
baggage included
priority boarding
lounge access
points/miles
upgrade chance
family seating
travel insurance
corporate invoicing
```

Example:

```text id="81c_airline_example"
Business traveler booking tomorrow:
values direct flight, flexibility, lounge, points, and time.

Holiday traveler booking months ahead:
may value lower fare, baggage bundle, and family seating.
```

81C tells 81 Core and 81B what the traveler values.

81B handles yield/dynamic capacity pricing.

---

## 18. Fare / Package Sensitivity

81C must understand package choice.

Applies to:

```text id="81c_package_categories"
airlines
hotels
car rentals
events
restaurants
professional services
subscriptions
travel packages
service bundles
```

Package types:

```text id="81c_package_types"
basic
standard
premium
all-inclusive
non-refundable
flexible cancellation
insurance included
service included
deposit + balance
upgrade eligible
loyalty eligible
```

Example:

```text id="81c_package_example"
One customer wants cheapest non-refundable room.
Another wants flexible cancellation, breakfast, late checkout, and points.
```

Same room category. Different value logic. Pricing needs to know which universe the customer lives in.

---

## 19. Scarcity / Priority Access Sensitivity

81C must detect customers who value access more than savings.

Signals:

```text id="81c_scarcity_signals"
buys new releases
joins waitlists
pays for early access
accepts premium price for availability
buys limited editions
responds to reserved stock
responds to VIP invitations
books during event peaks
chooses scarce inventory
```

Useful for:

```text id="81c_scarcity_use"
new iPhones
limited fashion drops
concert/event tickets
hotel event periods
car rentals during peak demand
rare restaurant bookings
luxury products
premium services
```

Offer implications:

```text id="81c_scarcity_offers"
priority access
reserved stock
VIP invitation
early notification
limited allocation
premium package
private appointment
```

Not every customer wants cheaper. Some want first. Or special. Or smug. Selene should know.

---

## 20. Flexibility / Commitment Sensitivity

81C must detect whether customer values flexibility.

Flexibility preferences:

```text id="81c_flexibility_values"
free cancellation
date changes
refundability
easy returns
upgrade option
changeable ticket
flexible booking
payment flexibility
refund protection
```

Relevant industries:

```text id="81c_flexibility_industries"
airlines
hotels
car rental
events
professional services
subscriptions
appointments
restaurant bookings
```

Output:

```text id="81c_flexibility_output"
customer may pay more for flexible terms
customer may accept cheaper non-refundable option
customer should be shown package choices
```

---

## 21. Ancillary / Add-On Value Sensitivity

81C must understand which add-ons matter.

Add-ons:

```text id="81c_ancillaries"
baggage
seat selection
priority boarding
lounge
insurance
excess reduction
warranty
gift wrapping
installation
support
delivery upgrade
care kit
setup service
extra driver
child seat
premium packaging
```

Example:

```text id="81c_ancillary_example"
Car rental customer may prefer lower base rate but wants insurance excess included.
Airline customer may accept higher fare with baggage and seat selection included.
```

This feeds:

```text id="81c_ancillary_feeds"
81 Core offer strategy
81B dynamic package optimization
81F promotion testing
E-Commerce/POS display
```

---

## 22. Spend Thresholds

81C should learn customer spend comfort.

Signals:

```text id="81c_spend_signals"
usual spend range
comfortable price band
ask-before-spending threshold
high-value hesitation point
deal trigger point
premium tolerance
category spend ceiling
gift spend limit
business approval threshold
```

Example:

```text id="81c_spend_example"
Customer buys shoes up to $250 comfortably,
but hesitates above $300.
```

Spend thresholds should be category/context-specific.

No one has one universal spending threshold unless they are a spreadsheet pretending to be a person.

---

## 23. Substitution Tolerance

81C must know substitution preference.

Substitution rules:

```text id="81c_substitution_rules"
allows substitutes
same brand only
same store only
same quality only
cheaper substitute allowed
premium substitute allowed
never substitute
ask every time
auto-substitute under threshold
```

Feeds:

```text id="81c_substitution_feeds"
Document 80 Order
E-Commerce
POS
Pricing
Customer Memory
```

Example:

```text id="81c_substitution_example"
Milk can be substituted.
Shoes cannot be substituted.
Bread from Sam Bakery must not be swapped without asking.
```

Correct. No surprise muffin logistics.

---

## 24. Offer Preference

81C must know which offer type works best.

Offer types:

```text id="81c_offer_types"
discount
free delivery
loyalty credit
cashback
gift-with-purchase
bundle
VIP packaging
priority delivery
warranty extension
private sale
early access
payment terms
rare gift
setup service
flexible cancellation
insurance/excess cover
```

Example:

```text id="81c_offer_example"
For a status customer, 10% off may cheapen the experience.
VIP packaging may work better.
```

Offer preference is one of 81C’s most important outputs.

---

## 25. Payment Preference Sensitivity

81C must learn payment behavior.

Payment preferences:

```text id="81c_payment_preferences"
uses points first
uses credit card
uses wallet
uses gift cards
uses store account
prefers installments
prefers interest-free
pays in full
uses account terms
uses cashback
uses business account
```

Feeds:

```text id="81c_payment_feeds"
POS
E-Commerce
Payment / Wallet
Customer Credit
Rewards
Pricing offer strategy
```

Example:

```text id="81c_payment_example"
Customer may value interest-free payment more than a small discount.
```

---

## 26. Urgency / Time Sensitivity

81C must detect urgency.

Urgency contexts:

```text id="81c_urgency_contexts"
needs today
needs this week
can wait
routine reorder
event deadline
gift deadline
travel deadline
business deadline
emergency purchase
limited stock window
```

Urgent customers may value:

```text id="81c_urgency_values"
speed
availability
direct route
same-day delivery
pickup
premium service
priority handling
```

Patient customers may value:

```text id="81c_patient_values"
lower price
bundle
slower delivery
scheduled delivery
subscription savings
```

---

## 27. Customer Lifetime Value Signal

81C should provide relationship value signal.

CLV-related profiles:

```text id="81c_clv_profiles"
new customer
repeat customer
high-value customer
VIP customer
at-risk customer
loyal customer
reactivation target
business account
referral source
high-influence customer
frequent returner
```

Possible uses:

```text id="81c_clv_uses"
welcome offer
goodwill credit
service recovery
retention benefit
VIP service
loyalty benefit
relationship repair
```

Guardrail:

```text id="81c_clv_guardrail"
Do not destroy margin without authority.
```

Selene may invest in the relationship. Selene may not light the profit margin on fire and call it love.

---

## 28. Declared Customer Preferences

Customers should be able to tell Selene what they want.

Examples:

```text id="81c_declared_examples"
always show cheaper options
only show premium brands
do not suggest substitutes
use points first
ask before spending over $100
I care about warranty
I prefer fast delivery
I hate this store
never use this supplier/store
show me flexible fares only
avoid non-refundable bookings
```

Declared preferences should outrank inferred guesses unless clearly outdated or impossible.

Humans are chaotic, but sometimes they do literally tell you the answer. Rare gift. Use it.

---

## 29. Cold Start / New Customer Handling

For new customers, 81C must avoid over-personalization.

Cold-start inputs:

```text id="81c_cold_start_inputs"
declared preferences
onboarding questions
store context
company/customer relationship
first purchase behavior
similar segment
safe defaults
country/region
product category norms
```

Cold-start rules:

```text id="81c_cold_start_rules"
use conservative assumptions
show options
ask simple preference questions
avoid hidden personalization
assign low confidence
update quickly from behavior
```

No “we met five minutes ago, I understand your soul.” Creepy. Also inaccurate.

---

## 30. Learning Triggers and Automation

81C must update automatically from events.

Trigger sources:

```text id="81c_trigger_sources"
E-Commerce search
E-Commerce filter/sort behavior
POS scan-as-you-shop behavior
POS payment choice
Order completion
Order cancellation
Cart abandonment
Offer accepted
Offer rejected
Substitute accepted
Substitute rejected
Return/refund
Warranty claim
Customer complaint
Customer rating/review
Customer declared preference
Travel/fare selection
Gift/recipient order
Loyalty redemption
Payment method selection
```

Trigger flow:

```text id="81c_trigger_flow"
event occurs
event sent to 81C
81C classifies signal
81C updates category/context profile
81C adjusts confidence
81C emits CustomerValueSignalPacket
81 Core / 81B / 81F may use signal
audit recorded
```

Example:

```text id="81c_automation_example"
Customer rejects cheap substitute for bread three times.
81C increases “same-store / quality-sensitive bread preference.”
Order Engine later asks before substituting bread.
```

This is how Selene becomes less stupid over time. A rare and beautiful trajectory.

---

## 31. Scan Cadence and Event-Driven Updates

81C mostly updates from events, but it also needs scheduled review.

Event-driven updates:

```text id="81c_event_updates"
purchase
search
cart action
offer action
return
payment choice
declared preference
customer feedback
```

Scheduled reviews:

```text id="81c_scheduled_reviews"
weekly high-activity customer profile refresh
monthly account customer review
quarterly business customer profile review
after campaign test
after major category behavior change
before contract/price lock review
before high-value/VIP campaign
```

Output:

```text id="81c_scan_output"
profile unchanged
profile updated
confidence changed
human review required
customer preference confirmation recommended
```

No forever-stale customer segment. People change. Mostly for the worse, but still.

---

## 32. Confidence Scoring

Every value signal needs confidence.

Confidence factors:

```text id="81c_confidence_factors"
number of events
recency
category specificity
context specificity
declared vs inferred
consistency
contradictions
data source reliability
customer confirmation
seasonality
gift/business context
```

Confidence outputs:

```text id="81c_confidence_outputs"
high
medium
low
unknown
needs confirmation
```

Example:

```text id="81c_confidence_example"
Price sensitivity for groceries: high confidence.
Luxury preference for handbags: medium confidence.
Hotel flexibility preference: low confidence.
```

Low confidence should lead to:

```text id="81c_low_confidence_action"
show options
ask customer
avoid aggressive personalization
use safe defaults
```

Selene should know when she is guessing. This alone puts her ahead of most software and several executives.

---

## 33. Fairness, Ethics + Trust Guardrail

81C must protect fairness.

Use segmentation to improve:

```text id="81c_fair_use"
offers
service
bundles
rewards
payment options
communication
recommendations
```

Be careful with:

```text id="81c_risky_use"
hidden personalized base pricing
charging more because customer appears wealthy
charging more based on sensitive traits
unfair discrimination
opaque manipulation
exploiting urgency or vulnerability
```

Preferred design:

```text id="81c_preferred_design"
transparent base price
personalized benefit
personalized reward
personalized service
personalized package
```

High-risk design:

```text id="81c_high_risk_design"
secret individual base price without disclosure/governance
```

Do not turn customer intelligence into a wallet-sniffing machine. It is both creepy and likely to become a meeting with lawyers.

---

## 34. Protected / Sensitive Attribute Exclusion

81C must not use protected or sensitive traits for unfair pricing.

Do not use improper signals like:

```text id="81c_protected_attributes"
race
religion
health status
disability
gender
age where protected
sensitive personal data
protected financial vulnerability
protected location/social status in an unfair way
```

Allowed commercial signals may include:

```text id="81c_allowed_signals"
declared preferences
purchase history
product category behavior
loyalty tier
account terms
offer response
service preference
delivery preference
payment preference
business relationship
```

If sensitive-signal risk appears:

```text id="81c_sensitive_risk"
flag fairness/compliance review
do not use signal in pricing
audit exclusion
```

---

## 35. Multi-Person / Household / Business Context

Customer may buy for different people.

Contexts:

```text id="81c_multi_contexts"
self
mum
father
wife/husband
kids
friend
employee
client
business partner
office
project
recipient
gift recipient
```

Each context may have different value profile.

Example:

```text id="81c_multi_context_example"
Customer buys cheap snacks for self.
Premium gift for client.
Bulk value supplies for office.
Flexible flight for business.
Cheapest flight for personal holiday.
```

Selene must identify context before applying customer value profile.

---

## 36. Relationship to 81B Dynamic Pricing

81C feeds 81B:

```text id="81c_to_81b"
price sensitivity
premium tolerance
deal responsiveness
service sensitivity
urgency
scarcity/access sensitivity
flexibility preference
package preference
```

Example:

```text id="81c_81b_example"
For premium segment:
Do not reduce base price.
Offer early access or premium service.

For price-sensitive segment:
Use bundle or loyalty credit.
```

81B changes offer/pricing behavior over time.

81C tells it what customer type is likely to respond.

---

## 37. Relationship to 81D Brand Guardrail

81C must respect brand position.

Example:

```text id="81c_81d_example"
Customer is price-sensitive,
but product is luxury and no-discount.
```

Selene may offer:

```text id="81c_brand_safe_offers"
payment terms
VIP gift
private service
loyalty points
priority access
rare accessory
```

instead of:

```text id="81c_brand_bad_offer"
public discount
```

Brand guardrail can override customer discount preference.

Luxury does not bow to coupon goblins unless policy says so.

---

## 38. Relationship to 81F Promotion Experimentation

81C helps 81F test which offers work.

Test categories:

```text id="81c_81f_tests"
discount response
free delivery response
points response
bundle response
VIP gift response
private sale response
flexible fare response
insurance package response
priority access response
```

Promotion results feed back into 81C.

Example:

```text id="81c_81f_example"
Customer segment responds better to free delivery than 10% off.
81C updates offer preference.
81 Core uses free delivery as preferred offer.
```

---

## 39. Relationship to E-Commerce, POS, and Order

81C helps:

```text id="81c_channel_uses"
E-Commerce recommend better products and offers
POS suggest best private payment/offer
Order apply standing instructions
Pricing choose offer type
Marketing target campaigns
Rewards choose benefit type
B2B choose customer benefit strategy
```

Examples:

```text id="81c_pos_example"
At POS:
Customer has points and usually uses points first.
POS privately suggests using points.

At E-Commerce:
Customer values status product.
E-Commerce shows premium packaging, not cheap substitute.

At Order:
Customer never substitutes bread without asking.
Order pauses line if preferred bakery is out.
```

---

## 40. Human / External Action Orchestration

Most 81C updates are automatic.

But some require human/customer action.

Examples:

```text id="81c_human_actions"
confirm declared preference
resolve conflicting preference
approve use of business/customer segment rule
review fairness flag
confirm VIP/account segmentation
confirm high-value customer treatment
```

Any human action must follow the Human / External Action Orchestration Law:

```text id="81c_action_fields"
owner
recipient
due time
delivery method
required confirmation
reminder
escalation
audit
```

No “ask customer later.” Later is where good intentions go to become bugs.

---

## 41. Outputs from 81C

81C should output:

```text id="81c_outputs"
customer value segment
category-specific sensitivity
context-specific sensitivity
price sensitivity score
quality sensitivity score
brand/status sensitivity score
service sensitivity score
warranty/trust sensitivity score
reward sensitivity score
scarcity/access sensitivity score
flexibility sensitivity score
fare/package preference
ancillary/add-on preference
offer preference
substitution tolerance
payment preference
spend threshold
urgency sensitivity
customer lifetime value signal
declared preference references
confidence score
fairness/compliance flags
recommended offer type
do-not-use signals
audit evidence
```

---

## 42. State Machines

### Customer Value Profile State

```text id="81c_state_profile"
Unknown
ColdStart
InitialProfileCreated
Learning
CategoryProfileCreated
ContextProfileCreated
ConfidenceGrowing
Stable
ConflictDetected
NeedsConfirmation
Updated
Archived
```

### Sensitivity Signal State

```text id="81c_state_signal"
Detected
Classified
CategoryMapped
ContextMapped
ConfidenceScored
Applied
Rejected
Expired
Audited
```

### Declared Preference State

```text id="81c_state_declared"
NotDeclared
Declared
Confirmed
Applied
ContradictedByBehavior
NeedsReconfirmation
Updated
Revoked
Archived
```

### Offer Preference State

```text id="81c_state_offer"
Unknown
Testing
DiscountResponsive
BundleResponsive
FreeDeliveryResponsive
RewardsResponsive
VIPBenefitResponsive
AccessResponsive
FlexibilityResponsive
NoClearWinner
Updated
```

### Fairness Guardrail State

```text id="81c_state_fairness"
Clear
SensitiveSignalDetected
SignalExcluded
ComplianceReviewRequired
ApprovedForUse
RejectedForUse
Audited
Closed
```

---

## 43. Reason Codes

```text id="81c_reason_codes"
CUSTOMER_VALUE_PROFILE_CREATED
CUSTOMER_VALUE_PROFILE_UPDATED
CATEGORY_SPECIFIC_PROFILE_CREATED
CONTEXT_SPECIFIC_PROFILE_CREATED
PRICE_SENSITIVITY_SIGNAL_DETECTED
QUALITY_SENSITIVITY_SIGNAL_DETECTED
BRAND_STATUS_SIGNAL_DETECTED
SERVICE_SENSITIVITY_SIGNAL_DETECTED
WARRANTY_TRUST_SIGNAL_DETECTED
LOYALTY_REWARD_SIGNAL_DETECTED
SCARCITY_ACCESS_SIGNAL_DETECTED
FLEXIBILITY_SIGNAL_DETECTED
ANCILLARY_ADDON_SIGNAL_DETECTED
GIFT_CONTEXT_DETECTED
BUSINESS_ACCOUNT_CONTEXT_DETECTED
AIRLINE_TRAVEL_CONTEXT_DETECTED
NO_DISCOUNT_LAUNCH_CONTEXT_DETECTED
FARE_PACKAGE_PREFERENCE_DETECTED
SUBSTITUTION_TOLERANCE_UPDATED
PAYMENT_PREFERENCE_UPDATED
SPEND_THRESHOLD_UPDATED
CUSTOMER_DECLARED_PREFERENCE_CAPTURED
DECLARED_PREFERENCE_OVERRIDES_INFERENCE
COLD_START_PROFILE_CREATED
PROFILE_CONFIDENCE_LOW
PROFILE_CONFIDENCE_HIGH
SENSITIVE_ATTRIBUTE_EXCLUDED
FAIRNESS_REVIEW_REQUIRED
OFFER_PREFERENCE_UPDATED
CUSTOMER_LIFETIME_VALUE_SIGNAL_CREATED
CUSTOMER_VALUE_SIGNAL_SENT_TO_81
CUSTOMER_VALUE_SIGNAL_SENT_TO_81B
CUSTOMER_VALUE_SIGNAL_SENT_TO_81F
CUSTOMER_VALUE_AUDIT_CAPTURED
```

---

## 44. Required Simulations

```text id="81c_required_simulations"
new customer cold-start onboarding creates low-confidence profile
customer sorts groceries by cheapest and price sensitivity increases for groceries
customer buys premium handbags and brand/status sensitivity increases for fashion
customer rejects bread substitute and substitution tolerance updates
customer buys latest iPhone and no-discount launch treatment recommends rare gift/setup service
customer responds to free delivery more than discount
customer uses points first at POS and payment preference updates
customer books flexible airline fare and flexibility sensitivity updates
business traveler prefers direct flight and lounge package
holiday traveler prefers cheap fare with baggage bundle
customer buys premium gift for mum despite budget self-purchases
customer chooses Hilton over cheaper hotel and trust/service sensitivity increases
customer selects car rental insurance excess package and ancillary preference updates
customer declared “ask before spending over $100” and order/pricing uses threshold
protected attribute signal detected and excluded
low confidence profile results in showing multiple options instead of personalization
promotion test updates offer preference
VIP customer receives access/rare gift instead of discount
business account customer price lock prevents consumer offer logic
```

---

## 45. Integration Map

```text id="81c_integration_map"
PH1.PRICING.81C / CUSTOMER_VALUE_SEGMENTATION
↔ PH1.PRICING / DOCUMENT_81_CORE
↔ PH1.PRICING.81A / MARKET_INTELLIGENCE
↔ PH1.PRICING.81B / DYNAMIC_PRICING
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81F / PROMOTION_EXPERIMENTATION
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_MEMORY
↔ PH1.REWARDS / LOYALTY
↔ PH1.CUSTOMER_CREDIT / WALLET
↔ PH1.ECOMMERCE
↔ PH1.POS
↔ PH1.ORDER
↔ PH1.B2B_PLATFORM
↔ PH1.MARKETING
↔ PH1.EVENTS
↔ PH1.TRAVEL / AIRLINE
↔ PH1.HOTEL / HOSPITALITY
↔ PH1.FLEET / CAR_RENTAL
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

## 46. Required Logical Packets

```text id="81c_packets"
CustomerValueSignalPacket
CustomerValueProfilePacket
CategoryValueProfilePacket
ContextValueProfilePacket
PriceSensitivityPacket
QualitySensitivityPacket
BrandStatusSensitivityPacket
ServiceSensitivityPacket
WarrantyTrustSensitivityPacket
LoyaltyRewardSensitivityPacket
ScarcityAccessSensitivityPacket
FlexibilitySensitivityPacket
FarePackagePreferencePacket
AncillaryAddOnPreferencePacket
GiftContextValuePacket
BusinessAccountValuePacket
NoDiscountLaunchTreatmentPacket
SubstitutionTolerancePacket
PaymentPreferencePacket
SpendThresholdPacket
UrgencySensitivityPacket
CustomerLifetimeValueSignalPacket
DeclaredPreferencePacket
ColdStartSegmentationPacket
CustomerValueConfidencePacket
FairnessGuardrailPacket
SensitiveSignalExclusionPacket
OfferPreferencePacket
CustomerValueAuditEvidencePacket
```

Logical only.

No runtime packet structs. The segmentation goblin can put down the clipboard.

---

## 47. What Codex Must Not Do

```text id="81c_codex_must_not"
Do not make 81C decide final price.
Do not make 81C override Document 81 Core.
Do not use one global customer segment for all categories.
Do not assume customer value profile is permanent.
Do not use protected/sensitive attributes for unfair pricing.
Do not secretly raise base price because customer appears willing to pay.
Do not personalize base price without fairness/compliance governance.
Do not treat inferred preference as higher priority than declared preference without reason.
Do not apply self-purchase behavior to gift/recipient context blindly.
Do not apply grocery price sensitivity to luxury/status products blindly.
Do not apply consumer offers to contract/account customers where price locks apply.
Do not recommend discount for no-discount launch/status products when value-add is appropriate.
Do not ignore confidence scoring.
Do not create human/customer action without Human / External Action Orchestration.
Do not let GPT-5.5 invent customer value profile, income, status, protected traits, or consent.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 48. Final Architecture Sentence

Selene Customer Value Segmentation + Price Sensitivity Engine is the pricing pack sub-engine that learns, confirms, scores, and audits what each customer values by category, context, recipient, channel, urgency, product type, travel/fare package, scarcity/access preference, flexibility needs, loyalty behavior, payment behavior, gift intent, business/account relationship, declared preference, and offer response; then sends governed customer value signals to Document 81 Core, 81B Dynamic Pricing, 81D Brand Guardrail, 81F Promotion Experimentation, E-Commerce, POS, Order, Rewards, and Marketing so Selene can choose the right offer, service, bundle, benefit, payment option, or value treatment without unfairly or creepily personalizing base price.

Simple version:

```text id="81c_simple_version"
81C tells Selene what this customer values in this situation.

Some customers want cheapest.
Some want quality.
Some want status.
Some want speed.
Some want flexibility.
Some want warranty.
Some want points.
Some want access.
Some want a rare gift instead of a discount.
Some want airline baggage and flexible changes.
Some want Hilton because trust matters.

81C does not set price.
81C tells Document 81 what value matters.
Document 81 decides the final price and offer.
Everything important is confidence-scored, fair, and audited.
```

That is 81C: customer value intelligence without becoming a creepy little wallet-sniffer.

---

## 49. 81E Customer Benefit Funding Handoff

81C feeds 81E customer value signals for B2B benefit, service, bundle, rebate, loyalty, access, and value-treatment logic.

81C must not create, recommend, display, or imply unfunded B2B customer benefits. Any B2B benefit pool, discount substitute, service upgrade, recurring benefit, referral benefit, or customer-visible value treatment must be validated by 81E for funding source, margin impact, commission impact, reserve impact, bottom-line target impact, fairness, and auditability.

---

## 50. 81F-81J Customer Value Handoff

81C customer value segments feed 81F promotion test design, offer variant selection, customer-quality analysis, repeat-purchase learning, and promotion fatigue checks.

81C customer-facing value logic must be explainable and auditable through 81G. Service/value preference must coordinate with 81H; location, recipient, and customer context must coordinate with 81I; and product presentation emphasis, perceived value, trust signals, accessibility, localization, and offer framing must coordinate with 81J.

81C must not create personalized benefits, value treatments, or offer recommendations that 81F cannot test, 81E cannot fund where B2B applies, or Document 81 Core cannot approve.

---

## 51. Commerce Stack 82-84 Customer Preference Boundary

81C customer value signals may influence delivery preference, return preference, warranty preference, service promise, packaging/gift preference, offer display, and customer-facing presentation.

Customer preference must not override product terms, terms shown before purchase, compliance/legal handoff, B2B provider responsibility, dispatch safety, brand/official-channel restrictions, return fraud controls, refund authority, or Document 83 eligibility decisions.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
