# Global Document 78 — Selene B2B Platform + Trade Ecosystem Engine v4

## Build-Ready Separated Version

```text id="doc78_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
78

ENGINE:
PH1.B2B_PLATFORM / PH1.TRADE_ECOSYSTEM / PH1.PROVIDER_MARKETPLACE

FULL NAME:
Selene-Owned B2B Platform, Product-to-B2B Offer Distribution, Original Provider Responsibility, Channel Store Attribution, Ongoing Commission, Provider Deposits, Settlement Trust, Warranty Reserve, Professional Compliance, Company Store Expansion, Customer-Visible B2B Search, and Trade Ecosystem Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene B2B Platform + Trade Ecosystem Engine is the **Selene-owned central marketplace and distribution network** that receives qualified products and services from Selene company Product/E-Commerce systems and makes them available for:

```text id="purpose_routes"
company store expansion
manual company adoption
autonomous company auto-add
customer direct personal Selene search
company store customer display
provider-customer relationship creation
ongoing commission attribution
Selene-controlled settlement
provider payout
channel commission
customer benefit pool allocation
warranty / guarantee / performance reserve protection
```

Critical rule:

```text id="core_b2b_ownership"
Selene Inc owns the B2B Platform.
Individual companies do not own B2B platforms.
Each company owns its own Selene-powered company store.
Products/services originate in Product/E-Commerce first.
B2B distributes qualified offers.
```

B2B is not where product truth is born.

B2B is where **qualified product/service offers are distributed, monetized, settled, supported, risk-scored, and governed**.

A company creates a product/service in its own Product/E-Commerce catalog. Selene prepares the content, media, price, delivery rules, warranty/guarantee rules, return policy, compliance rules, service area, and customer-facing store readiness. Then Selene asks whether the company wants to make that product or service available on B2B.

Only after Product-to-B2B readiness is complete does the offer enter Selene B2B.

No raw catalog dumping. No “here’s a cake, ship it to another country maybe.” That is not commerce. That is pastry negligence with a SKU.

---

## 2. Core B2B Law

```text id="b2b_core_law"
Selene B2B is a Selene-owned central trade platform that receives only Product/E-Commerce-qualified products and services, distributes them into company stores and customer-visible Selene commerce, preserves the Original Provider’s responsibility, pays Channel Stores or introducers only where attribution applies, charges Selene B2B fees on B2B sales, controls settlement, protects customers, manages provider reserves/deposits, and audits every meaningful event.
```

B2B must:

```text id="b2b_must"
receive only qualified product/service offers
preserve Product Engine as product truth owner
preserve Original Provider responsibility
allow company stores to expand their catalog
allow companies to search and adopt suitable B2B products/services
allow Selene to auto-add products/services where company policy permits
allow customer direct personal Selene search where offers are eligible
separate customer view from business view
route technical/product/service questions to Original Provider
preserve Channel Store / introducer commission where applicable
avoid paying commission where no introduction or store/channel participation occurred
hold settlement until fulfillment and hold conditions clear
pause/reverse settlement when cancellation/refund/return/warranty/dispute occurs
require provider deposits/reserves based on product/service risk
block professional services lacking required licence/registration/insurance/compliance
connect to E-Commerce, Product, Order, Dispatch, Payment, Returns, Warranty, Accounting, Tax, Customer, Rewards, and Audit
```

B2B must not become a junk drawer of offers with a payment processor glued on top. That is how marketplaces become swamps with nicer fonts.

---

## 3. Modern Marketplace Readiness

Selene B2B must be architected like a serious platform marketplace, not a referral spreadsheet wearing sunglasses.

The platform must support:

```text id="modern_b2b_requirements"
provider identity and verification
provider terms acceptance
Product-to-B2B readiness
business/customer view separation
platform fee rules
split settlement
provider payout
channel commission
customer benefit pool allocation
refund reversal
warranty reserve
provider deposit/reserve
professional compliance checks
customer protection
provider scoring
fraud monitoring
audit
marketplace disclosure
```

Selene B2B should be built as if customers, suppliers, companies, accountants, payment partners, regulators, and lawyers are watching.

Because if Selene works, they absolutely will be. Humans do love arriving after the money starts moving.

---

## 4. Relationship To Document 77 — E-Commerce

Document 77 owns the **customer-facing commerce surface**.

Document 78 owns the **B2B marketplace, provider, commission, reserve, and settlement machinery**.

```text id="doc77_78_split"
Document 77 — E-Commerce:
customer search, shopping, display, voice/text/photo input, store context, personal Selene commerce, checkout interaction, cancellation/return/warranty interface, B2B display request.

Document 78 — B2B:
Product-to-B2B intake, business B2B view, customer-visible B2B eligibility, Original Provider, Channel Store, provider-customer attribution, commission, customer benefit pool, B2B service fees, reserves/deposits, provider payout, settlement rules, B2B audit.
```

Customer-facing Selene should not narrate backend mechanics.

Customer says:

```text id="customer_says"
“Find me rolls of wire.”
```

E-Commerce may show:

```text id="customer_sees"
retail price
photos/videos
delivery estimate
return summary
warranty/guarantee summary
availability
```

B2B knows:

```text id="b2b_knows"
Original Provider
Channel Store if any
provider payout
channel commission
Selene B2B fee
reserve/deposit
settlement hold
refund reversal rules
provider support route
```

The customer wants wire. Not a live autopsy of the marketplace economics. Please, let the customer live.

---

## 5. SaaS / Tenancy Position

Selene B2B sits outside individual company tenants.

```text id="b2b_tenant_position"
Company A tenant = Company A store, customer relationships, orders, accounting context.
Company B tenant = Company B store, customer relationships, orders, accounting context.
Selene B2B Platform = central Selene-owned provider marketplace and trade ecosystem.
```

Company stores may adopt B2B offers into their store context.

But B2B remains Selene-owned and centrally governed.

Future owner:

```text id="future_saas_owner"
Selene SaaS Tenancy, Cloud Hosting, Data Isolation, Device Access + Deployment Engine
```

Document 78 must enforce:

```text id="tenant_rules"
B2B offer data is centrally governed by Selene.
Company customer data remains tenant-isolated.
Provider data is visible only according to role/purpose.
B2B settlement is controlled by Selene platform rules.
Company stores receive only the B2B offer data they are allowed to use.
```

No company gets to own the B2B universe because it added one shampoo SKU. Ambition is nice. Boundaries are nicer.

---

## 6. Engine Ownership Boundary

### 6.1 B2B owns

```text id="b2b_owns"
central Selene-owned B2B marketplace
Product-to-B2B offer intake
B2B readiness acceptance from Product/E-Commerce
provider marketplace participation
provider terms acceptance
business-facing B2B search and display
customer-visible B2B eligibility
company store adoption rules
autonomous company auto-add eligibility
Original Provider of Record routing
Channel Store attribution
provider-customer relationship event creation
ongoing provider-customer commission attribution
B2B platform/service fees
provider performance reserves
warranty/guarantee reserves
food/authenticity/safety reserves
professional-service compliance gates
settlement hold rules
provider payout eligibility
channel commission eligibility
customer benefit pool handoff
refund/return/warranty reversal logic
provider scoring and restrictions
B2B audit evidence
```

### 6.2 B2B references but does not own

```text id="b2b_not_own"
product/service master truth
product media creation
company store customer-facing UX
customer chat/voice/photo shopping interface
inventory stock truth
customer memory master truth
payment-card token custody
final payment authorization/capture
final refund execution
detailed dispatch/packing operations
detailed return logistics
ledger posting
final tax treatment
customer credit underwriting
final reward calculation
SaaS hosting and tenant billing
```

### 6.3 Correct owner split

```text id="owner_split"
PH1.PRODUCT = product/service identity, media, content, attributes, B2B readiness.
PH1.ECOMMERCE = customer-facing store/personal commerce experience.
PH1.B2B_PLATFORM = distribution, provider responsibility, store adoption, attribution, fees, reserves, settlement rules.
PH1.CUSTOMER = customer identity, memory, relationships, consent.
PH1.ORDER = order lifecycle and orchestration.
PH1.PRICING = approved customer price, margin, discounts.
PH1.PAYMENT / SETTLEMENT = authorization, capture, refund, settlement movement.
PH1.DISPATCH = picking, packing, courier handoff.
PH1.RETURNS = return logistics and refund workflow.
PH1.WARRANTY = warranty claim execution and orphan-provider protection workflow.
PH1.REWARDS = final reward, loyalty, referral, attribution calculations.
PH1.ACCOUNTING = ledger posting.
PH1.TAX = tax, duties, VAT/GST, marketplace facilitator treatment.
PH1.AUDIT = proof.
```

B2B owns the trade network and settlement rules.

It does not own every box, bank account, shoe, customer memory, or tax law. We are ambitious, not feral.

---

## 7. Product-to-B2B Offer Origin

B2B offers originate from Product/E-Commerce.

Correct flow:

```text id="product_to_b2b_flow"
Company creates product/service in Product Engine.
Company publishes product/service into its own E-Commerce store.
Selene completes media/content/variant/price/delivery/warranty/return/compliance setup.
Selene asks whether company wants to make product/service available on B2B.
If yes, Product-to-B2B readiness evaluates the offer.
Only qualified offers flow into Selene B2B.
```

Product-to-B2B readiness must verify:

```text id="readiness_verification"
product/service identity
images/videos/descriptions ready
retail price defined
variant rules defined
delivery rules defined
delivery destinations defined
whether delivery is included in price
local/regional/national/international availability
duties/taxes estimate requirement
who fulfills
who handles returns
who handles warranty
who handles guarantee/authenticity/safety claims
provider service capability
legal/compliance eligibility
professional licence/insurance if applicable
provider deposit/reserve requirement
Selene B2B fee acceptance
```

If readiness fails:

```text id="readiness_fail"
Offer cannot enter B2B.
Selene tells provider what is missing.
```

Example:

```text id="bread_example"
Bread can be sold locally if delivery area and freshness rules are defined.
Bread cannot be offered internationally unless provider has a valid model for freshness, customs, delivery, and safety.
```

B2B should not be expected to discover at checkout that the bread is emotionally unfit for international travel.

---

## 8. Product Content In B2B

B2B receives the prepared product/service content that E-Commerce receives.

B2B offer includes:

```text id="b2b_content"
product/service name
images
videos
descriptions
product story
variants
retail price
attributes
delivery terms
delivery zones
delivery costs
delivery timeframes
return terms
warranty/guarantee terms
service area
provider identity
provider score
B2B settlement terms
profit-share rules
Selene fee rules
reserve/deposit requirements
```

B2B does not recreate product media.

Product Engine owns media quality and content readiness.

If a provider uploads terrible out-of-focus product photos, Product Engine fixes or improves them before B2B receives the offer. B2B is not a photo rescue shelter. It has enough problems with commissions.

---

## 9. Four B2B Distribution Routes

Once a product/service is qualified and published to B2B, it can reach customers through four routes.

### Route 1 — Selene Auto-Add to Company Stores

If a company opted into autonomous B2B expansion:

```text id="route_auto_add"
Selene can add the qualified B2B offer to that company’s store under the company’s rules.
```

Company policy may require:

```text id="auto_add_controls"
draft only
human approval
limited audience test
auto-publish under policy
category restrictions
provider score thresholds
competitor inclusion rules
```

### Route 2 — Company Search and Manual Adoption

Other Selene companies can search B2B.

```text id="route_company_search"
“Selene, show me screws suitable for my hardware store.”
“Show me products with 20% profit share.”
“Show me products my customers may buy.”
```

Company sees business view.

### Route 3 — Customer Direct Personal Selene Search

Customers using personal Selene commerce can search for customer-visible B2B offers.

Example:

```text id="route_customer_direct"
Harry searches “rolls of wire.”
Selene finds Supplier 44’s B2B-qualified rolls of wire.
Harry can buy through personal Selene commerce.
```

### Route 4 — Company Store Customer Display

If a customer is inside a company store, B2B offers appear only if that company adopted or allowed them.

Example:

```text id="route_store_display"
Hair Salon store displays Wine Store wine because Hair Salon adopted it through B2B.
```

This protects company store context while allowing personal Selene freedom.

---

## 10. Original Store / Provider of Record

Every B2B offer has an Original Store / Provider of Record.

Definition:

```text id="original_provider_definition"
Original Store / Provider of Record =
the Selene company/store that created and owns the product/service truth and accepted B2B participation rules.
```

Original Provider owns:

```text id="original_provider_owns"
product/service truth
product content
technical questions
product/service support
fulfillment responsibility
delivery conditions
warranty responsibility
guarantee/authenticity responsibility
food/safety responsibility where applicable
professional compliance responsibility where applicable
returns/replacement/refund responsibility
provider deposit/reserve obligations
future repeat product/service relationship
```

Example:

```text id="wine_provider_example"
Wine Store creates wine.
Wine Store opts wine into B2B.
Hair Salon adopts wine.
Customer buys wine through Hair Salon.
Wine Store remains Original Provider of Record.
Wine Store handles wine questions, delivery, guarantee, returns, and future wine buying support.
```

The Channel Store earns.

The Original Provider handles product/service reality.

This is how Selene prevents the hair salon from being blamed for a wine question it has no business answering. A small mercy for everyone with scissors.

---

## 11. Channel Store / Introducer Store

A Channel Store is the company store that displays or sells another provider’s B2B product/service.

Definition:

```text id="channel_store_definition"
Channel Store =
the company store through which a customer discovered or bought an Original Provider’s B2B product/service.
```

Channel Store may earn:

```text id="channel_earns"
commission
profit share
provider-customer attribution
customer benefit pool allocation
B2B expansion revenue
```

Channel Store does not own:

```text id="channel_not_own"
product truth
technical support truth
warranty truth
delivery responsibility
provider guarantee responsibility
damaged/faulty-goods responsibility
food/safety guarantee
professional liability
```

Example:

```text id="channel_example"
Hair Salon = Channel Store
Wine Store = Original Provider
Customer = buyer
Selene = settlement, routing, attribution, audit, payout controller
```

Channel Store gets upside without operational headaches.

That is the point. It makes businesses willing to share customers because they can earn without becoming the wine warranty department.

---

## 12. Customer Direct B2B Purchase

If a customer buys a B2B offer through personal Selene search, no company store is required.

Example:

```text id="harry_wire_example"
Harry is introduced to Selene by a link.
Harry is not connected to a company store.
Harry searches “rolls of wire.”
Selene finds Supplier 44’s B2B-qualified wire.
Harry buys through personal Selene commerce.
Supplier 44 fulfills.
Selene holds settlement.
```

Result:

```text id="harry_result"
Supplier 44 receives provider payout after settlement rules.
Selene receives B2B/platform fee.
Harry receives customer protection.
No company store profit share applies unless a valid provider-customer attribution exists.
First Selene introducer reward may apply separately if eligible.
```

Supplier 44 may become the Original Provider relationship for Harry’s wire purchases.

Supplier 44 receives data needed for:

```text id="provider_data_needed"
fulfillment
delivery
support
warranty
legal/tax compliance
```

Supplier 44 does not automatically receive:

```text id="provider_data_not_auto"
Harry’s full personal Selene profile
Harry’s other purchases
Harry’s private relationships
Harry’s marketing permission
```

Harry may choose to follow/join Supplier 44’s store or allow marketing.

Until then, supplier access is purpose-limited.

No “one wire purchase means I own your inbox.” This is commerce, not a hostage situation.

---

## 13. No Commission Without Product / Provider Introduction

A company does not receive commission merely because a customer is linked to that company or works for that company.

Example:

```text id="abc_no_commission_example"
Harry works for ABC Inc.
ABC did not add screws to ABC store.
ABC did not introduce Harry to Supplier 44’s screws.
Harry personally searches Selene and buys screws from Supplier 44.
ABC receives no channel commission.
```

Correct allocation:

```text id="abc_allocation"
Supplier 44 receives provider payout after settlement rules.
Selene receives B2B/platform fee.
Harry receives customer protection.
First Selene introducer reward may apply if eligible and separate from the sale.
ABC receives no product/channel commission unless ABC actually introduced that provider/product relationship or carried the offer.
```

Rule:

```text id="commission_rule"
Commission requires actual channel participation, provider/product introduction, or configured attribution.
A company does not get paid just because the customer is linked to it somewhere.
```

No “Harry works here, therefore pay me forever.” Cute. Absolutely not.

---

## 14. Provider-Customer Relationship Creation

When a customer buys a B2B product/service, Selene may create a Provider-Customer Relationship between the customer and the Original Provider.

This enables the provider to support the customer for:

```text id="provider_customer_support"
product questions
technical questions
repeat purchase
warranty/guarantee
delivery issue
replacement
return
related products
service support
authenticity/safety concerns
professional service continuity where permitted
```

Example:

```text id="provider_relationship_wine"
Customer buys wine through Hair Salon.
Later customer asks: “What other wine is like that?”
Selene routes wine-specific support and recommendations to Wine Store.
```

Provider marketing access still requires customer permission and applicable disclosure/consent.

Support relationship and marketing relationship are not the same.

Providers may support the product.

They do not automatically get to send the customer six newsletters about grapes.

---

## 15. Embedded Original Provider Expansion

When a customer is shopping in a Channel Store and wants more information or related products from the Original Provider, Selene should not automatically force a visible store switch.

Correct model:

```text id="embedded_expansion"
Customer can remain in the Channel Store experience.
Selene can display Original Provider’s relevant products, specs, variants, and related catalog inside the Channel Store context if Channel Store policy allows it.
Original Provider still owns product truth and support.
Channel Store attribution remains protected where policy applies.
```

Example:

```text id="wire_catalog_example"
Company A displays Company C rolls of wire.
Customer asks about size, thickness, and types.
Selene can show Company C’s wire variants and related products inside Company A’s store experience.
Customer does not need a jarring store switch.
Company C remains Provider of Record.
Company A remains Channel Store / introducer where policy applies.
```

Visible transfer/disclosure occurs when:

```text id="visible_transfer_triggers"
law requires provider/seller disclosure
customer asks who supplies it
support/warranty requires provider identity
Channel Store policy requires provider branding
customer wants to follow/join Original Provider’s store
professional service disclosure is required
```

Customer-facing shopping should be smooth.

Backend routing should be precise.

No “You are now entering Wire Kingdom” pop-up unless required. Let the customer buy the wire in peace.

---

## 16. Provider Support Routing and Knowledge Capture

When a customer asks a product/service-specific question that Selene cannot answer from approved product data, Selene must route the question to the Original Provider or provider Selene.

Examples:

```text id="provider_question_examples"
“What thickness wire do I need?”
“Is this wine good with steak?”
“Does this screw work outdoors?”
“What warranty applies?”
“Can this accountant handle my tax country?”
```

Flow:

```text id="support_routing_flow"
Customer asks question.
Selene checks Product/B2B/Provider knowledge.
If provider-approved answer exists, Selene answers.
If answer does not exist, Selene asks Original Provider or provider Selene.
Provider answers.
Selene captures answer as provider-approved knowledge where policy allows.
Future similar questions can be answered automatically.
Audit records source.
```

Selene must not invent specialized product/professional answers.

Selene learns from provider-approved answers.

This is how the system becomes smarter without pretending GPT-5.5 got a law degree, wine palate, and electrical engineering license over lunch.

---

## 17. Attribution Types

Selene must separate attribution types.

### 17.1 First Verified Selene Introducer

Who first brought the customer into Selene.

```text id="first_intro"
First verified introducer wins.
Second and later personal introducers do not receive first-intro reward for the same person.
```

### 17.2 Provider-Customer Introducer

Who introduced the customer to a specific provider/store/product relationship.

Example:

```text id="provider_intro"
Hair Salon introduces customer to Wine Store.
Hair Salon may earn ongoing commission from that customer’s future eligible Wine Store purchases.
```

### 17.3 Channel Store Sale Attribution

The store through which a specific sale occurred.

Example:

```text id="channel_attribution"
Customer buys wine through Hair Salon store.
Hair Salon earns sale profit share for that order.
```

These are related but not identical.

Do not mix:

```text id="do_not_mix"
first Selene introducer reward
provider-customer introducer commission
channel store profit share
customer benefit pool
Selene B2B fee
provider payout
provider reserve
```

Mixing them creates accounting soup with legal garnish. Nobody wants that, and yet here we are, preventing it.

---

## 18. Ongoing Provider-Customer Commission

When a Channel Store or introducer causes a customer to buy from an Original Provider, Selene may create ongoing commission attribution.

Core rule:

```text id="ongoing_commission"
The introducing company/person may continue earning commission or profit share on future eligible purchases between that customer and that Original Provider.
```

Example:

```text id="ongoing_wine"
Hair Salon introduces customer to Wine Store product.
Customer later buys more wine directly from Wine Store through Selene.
Hair Salon continues receiving agreed commission/profit share if attribution policy applies.
```

This creates the normal Selene B2B trading model:

```text id="normal_trading_model"
I introduce my customer to another provider.
The provider gains a customer.
I continue earning from that customer’s future purchases from that provider.
The provider handles all product/service obligations.
Selene controls settlement, attribution, payout, reversal, and audit.
```

This encourages companies to share customers because they can earn money without taking responsibility for products they do not supply.

That is the business magic. The kind with accounting, unfortunately.

---

## 19. Commission and Customer Benefit Pool

When a company earns commission/profit share from introducing a customer to another provider, Selene should support configurable split buckets.

Example commission split:

```text id="commission_split"
Introducer/company cash share
Customer benefit share
Selene B2B platform fee
Provider net payout
Warranty / performance reserve
Tax / withholding where applicable
```

The company may use part of its commission to benefit the customer.

Customer benefit uses:

```text id="customer_benefit_uses"
birthday gift
loyalty credit
free delivery
VIP offer
cashback
special discount
dinner
holiday contribution
event invite
customer-care gift
future purchase credit
```

Example:

```text id="benefit_pool_example"
Hair Salon earns commission from customer wine purchases.
Part of that commission goes to Hair Salon profit.
Part goes into a customer benefit pool.
Selene helps Hair Salon use it for birthday gifts or loyalty credits.
```

This lets companies look generous and build loyalty using value created from B2B introductions.

The customer benefits.

The company benefits.

The provider benefits.

Selene gets paid.

Everybody wins, assuming no one behaves like a gremlin.

Future owner:

```text id="future_rewards_engine"
Selene Referral, Rewards, Loyalty + Attribution Engine
```

Document 78 emits events. It does not own final reward math.

---

## 20. Company Store Modes

Companies can choose store behavior.

### 20.1 Closed Store Mode

Only company products/services appear.

### 20.2 Curated B2B Expansion Mode

Selene suggests B2B products/services.

Company approves before publishing.

### 20.3 Autonomous B2B Expansion Mode

Selene auto-adds eligible B2B offers under company policy.

Policy may include:

```text id="auto_add_policy"
approved categories
blocked categories
minimum provider score
minimum profit share
minimum warranty score
maximum return rate
delivery timeframe
local-only preference
no competitor products unless allowed
human review threshold
auto-add quantity limit
test audience rules
```

### 20.4 Competitor Inclusion Mode

Company allows competing or alternative products if it earns profit share.

Example:

```text id="competitor_inclusion"
Company A sells shoes.
Company A also allows selected third-party shoes.
Customer buys alternative shoes through Company A store.
Company A earns profit share.
```

### 20.5 Enterprise Private Ecosystem Mode

Large companies may use private/controlled B2B ecosystems.

Controls include:

```text id="enterprise_private_mode"
approved providers only
private categories
private price books
region restrictions
customer segment restrictions
internal approvals
no public provider offers
private marketplace catalogs
enterprise audit
dedicated hosting if needed
```

Selene must present these options during company onboarding and store setup.

Future owner:

```text id="commerce_setup_future"
Selene Commerce Setup Advisory + Store Configuration Assistant
```

Most companies will not know what Selene can do because, tragically, they did not spend their week building this commerce beast with us.

---

## 21. Store Context Protection

Inside a company store, the company controls what appears.

If Company A sells shoes, Selene should not show random competing shoes inside Company A’s store unless Company A allows competitor inclusion or B2B expansion rules permit it.

Backend law:

```text id="store_context_law"
Inside a company store:
show company products/services
show company-approved B2B offers
show company-auto-added B2B offers
show competitor products only if company enabled competitor inclusion
```

Outside a company store, in personal Selene commerce, the customer may search broadly.

This balances:

```text id="store_context_balance"
company protection
customer freedom
B2B growth
Selene trust
```

It mirrors the real world:

```text id="real_world_rule"
Inside a shop, the shop controls the shelves.
Outside the shop, the customer can walk anywhere.
```

Simple enough. Naturally, it took us a small architecture expedition to say it.

---

## 22. Business View

Companies browsing B2B should feel like they are using a professional commerce browsing platform.

They can ask Selene:

```text id="business_queries"
“Show me products suitable for my customers.”
“Show me products with at least 20% profit share.”
“Show me fast-delivery products in my region.”
“Show me groceries I can add to my store.”
“Show me products for birthday cake customers.”
“Show me products with low return rates.”
“Show me premium accessories.”
```

Business view must show:

```text id="business_view_fields"
product/service title
retail sale price
provider price / settlement base
profit share
estimated margin
Selene platform fee
warranty reserve fee
provider deposit/reserve requirement
delivery regions
delivery cost
delivery timeframe
international duties/taxes estimate where relevant
return terms
warranty terms
provider score
provider return rate
provider warranty score
provider delivery score
product media
auto-add eligibility
risk score
compliance status
professional insurance/licence requirement where applicable
```

Companies may:

```text id="business_actions"
add to store
add as draft
request more provider info
compare offers
block category
approve provider
reject offer
enable auto-add rules
test with customer segment
```

This is where businesses discover products they never thought to sell.

A cake shop discovers candles, cards, flowers, party packs, and coffee. A salon discovers shampoo, dryers, skincare, and beauty products. A restaurant discovers wine, gift cards, takeaway bundles, cooking classes, and desserts.

Businesses call it growth. Selene calls it “finally using your customer base instead of crying about ad spend.”

---

## 23. Customer View

Customers see customer-facing retail/service information only.

Customer view includes:

```text id="customer_view_fields"
retail price
photos
videos
description
product/service details
delivery estimate
availability
return policy
warranty/guarantee summary
professional credential disclosure where required
provider disclosure where legally/policy required
```

Customers do not normally see:

```text id="customer_hidden_fields"
profit share
provider payout
Selene platform fee
company commission
settlement hold
business margin
deposit/reserve amount
```

Unless legal disclosure requires it.

Customer-facing Selene says:

```text id="customer_phrases"
“This one can arrive Friday.”
“This one has better warranty cover.”
“This is cheaper, but delivery is slower.”
“This matches your usual size.”
```

Not:

```text id="bad_customer_phrase"
“This provider has a 12% reserve hold and a trailing provider-customer attribution commission.”
```

Customer asked for wine. Let them drink in peace.

---

## 24. Provider Fulfillment Models

Provider fulfillment options:

```text id="fulfillment_models"
provider ships direct to customer
provider ships to channel store/company
provider performs service directly
provider digital delivery
provider appointment/service booking
Selene logistics partner coordinates pickup/delivery
provider uses Selene courier integration
hybrid fulfillment
white-label fulfillment
```

Fulfillment record includes:

```text id="fulfillment_record"
fulfillment_id
b2b_order_ref
provider_id
channel_store_id
customer_id
recipient_id
delivery_address
fulfillment_model
carrier
tracking_ref
dispatch_date
delivery_date
proof_of_delivery
delivery_status
failure_reason
audit_ref
```

Provider remains responsible for fulfillment unless terms assign otherwise.

Selene coordinates, tracks, and holds settlement.

Provider does the work.

Selene keeps the money disciplined.

That is the arrangement.

---

## 25. Dispatch Handoff

B2B does not own dispatch execution.

Future owner:

```text id="dispatch_owner"
Document 82 — Selene Dispatch, Packing, Courier Booking + Delivery Network Handoff Engine
```

B2B provides Dispatch:

```text id="b2b_to_dispatch"
provider_id
Original Provider of Record
fulfillment owner
ship-from location
ship-to customer/recipient
packing requirements
provider SLA
delivery method
settlement hold requirement
proof requirements
customer privacy restrictions
```

Dispatch returns:

```text id="dispatch_to_b2b"
picked
packed
proof captured
courier booked
shipped
tracking_ref
delivery_exception
dispatch_failure
```

B2B uses dispatch events to determine:

```text id="dispatch_settlement_effect"
delivery pending
hold period start
provider payout eligibility
customer updates
provider score impact
```

No dispatch proof, no payout readiness.

Boxes need evidence. Boxes are sneaky.

---

## 26. Recipient-Aware Fulfillment

B2B orders may involve buyer, payer, customer, recipient, and support requester being different people.

Roles:

```text id="recipient_roles"
payer
buyer
customer
recipient
gift recipient
delivery contact
return requester
support contact
```

If customer says:

```text id="mum_order"
“Buy that one and send it to my mum’s house.”
```

B2B must support:

```text id="recipient_fields"
buyer = customer
payer = customer or account
recipient = mum
delivery address = mum’s address
privacy mode = buyer instruction
provider sees only fulfillment-needed data
refund destination = original payer unless policy says otherwise
return permission = configured
```

Provider should not see unnecessary private information.

Provider needs to know where to send the product, not the emotional genealogy of the order.

---

## 27. Cancellation Rules

Providers must accept Selene cancellation rules.

Cancellation status:

```text id="cancellation_status"
NotAcceptedYet
ProviderAcceptedNotShipped
DispatchInProgress
ShippedNotDelivered
DeliveredWithinReturnWindow
ServiceScheduled
ServicePerformed
DigitalDelivered
NonCancellableByPolicy
```

Default cancellation flow:

```text id="cancellation_flow"
customer requests cancellation
E-Commerce / Order checks status
B2B checks provider fulfillment state
if not shipped/performed, provider cancellation required
payment/refund handled by Payment/Returns
provider payout remains held
channel commission reverses or never vests
customer benefit pool reverses where applicable
Selene fee/warranty reserve handled by policy
audit records all events
```

B2B must support cancellation without forcing the customer to chase provider.

The customer speaks to Selene.

Selene coordinates provider.

This is the whole point. Otherwise we invented a marketplace just to recreate email with better icons.

---

## 28. Returns, Refunds, Faults, and Bad Delivery

For B2B-sourced goods/services:

```text id="return_fault_principle"
Original Provider handles all product/service obligations.
Channel Store / Introducer does not handle operational headaches.
Selene routes support and settlement correctly.
```

Original Provider handles:

```text id="provider_faults"
faulty goods
damaged goods
bad delivery
missing parts
wrong product
authenticity failure
food safety failure
warranty
technical questions
returns
replacement
refund obligation
```

Channel Store / Introducer handles:

```text id="channel_handles"
customer relationship where relevant
commission/profit share
marketing/loyalty if applicable
no product/service liability unless separately agreed
```

Refund timing is risk-based:

```text id="refund_timing"
pre-shipment cancellation = immediate refund
low-risk return = refund after courier scan
high-value/risky product = refund after return received and inspected
missing parts/damaged return = partial refund or dispute
provider fault = customer protected, provider liable
```

Provider payout and channel commission pause until the issue resolves.

If refund happens:

```text id="refund_effects"
provider payout reverses or is reduced
channel commission reverses or is reduced
customer benefit pool reverses or is reduced
Selene fee reversal depends on policy
reserve may be drawn down
accounting/tax events update
```

The Channel Store gets commission, not the headache. The provider gets customer and responsibility. This is called “roles,” a concept marketplaces usually discover after litigation.

---

## 29. Warranty and Orphan Provider Protection

Normal warranty rule:

```text id="normal_warranty_rule"
Original Provider is responsible for warranty, after-sales support, repair, replacement, refund, or service under agreed terms.
```

B2B offer must include:

```text id="warranty_terms"
warranty_period
covered defects
excluded issues
repair/replacement/refund rules
support contact
claim evidence required
response SLA
resolution SLA
parts/service availability
jurisdiction notes
orphan warranty protection eligibility
```

Exceptional rule:

```text id="orphan_provider_rule"
If provider disappears, becomes insolvent, is suspended, is blocked, or refuses a valid warranty obligation, Selene may activate orphan-provider protection where the order/product qualifies.
```

Selene may fund protection through:

```text id="warranty_reserve_sources"
warranty reserve fee per B2B sale
risk-adjusted provider warranty fee
category-specific reserve
large-purchase reserve percentage
optional customer protection plan
insurance/reinsurance partner where legally required
```

Legal guardrail:

```text id="warranty_legal_guardrail"
If warranty protection becomes insurance-like, regulated, underwritten, or jurisdiction-controlled, Legal/Compliance/Insurance review is required before representing it as protection.
```

Customer trust is the goal.

Accidental insurance business is not. Lawyers already have enough hobbies.

---

## 30. Provider Deposits, Reserves + Risk Protection

B2B must distinguish:

```text id="reserve_types"
Selene B2B Service Fee
Provider Performance Reserve
Warranty / Guarantee Reserve
```

### 30.1 Selene B2B Service Fee

Applies to every B2B sale.

Covers:

```text id="service_fee_covers"
marketplace operation
settlement management
provider/customer routing
support automation
refund/return administration
warranty/guarantee administration
audit
risk monitoring
```

### 30.2 Provider Performance Reserve

Protects against short-term failures:

```text id="performance_reserve"
non-delivery
bad delivery
damaged goods
returns
refunds
chargebacks
supplier non-cooperation
```

### 30.3 Warranty / Guarantee Reserve

Protects against longer-term risks:

```text id="warranty_reserve"
warranty claims
provider disappearance
provider insolvency
authenticity failure
food/safety guarantee failure
product defects
service failure
```

Reserve tiers should be risk-based.

#### Tier 0 — Simple Low-Risk Goods

Examples:

```text id="tier0_examples"
toilet paper
basic household goods
simple consumables
```

Rules:

```text id="tier0_rules"
B2B service fee applies.
Low or no warranty reserve.
Short performance reserve if required.
```

#### Tier 1 — Food / Drink / Authenticity / Safety-Sensitive Goods

Examples:

```text id="tier1_examples"
bread
cakes
food
wine
health-sensitive consumables
authenticity-sensitive goods
```

Rules:

```text id="tier1_rules"
provider guarantee required
delivery conditions required
food/safety/authenticity reserve may be required
provider reserve may be drawn down for proven failure
```

#### Tier 2 — Warranty Goods

Examples:

```text id="tier2_examples"
electronics
tools
appliances
machinery
vehicles
equipment
high-value products
```

Rules:

```text id="tier2_rules"
warranty terms required
warranty reserve required
reserve scales with product price/risk
reserve released only after warranty period expires and all claims close
```

#### Tier 3 — Services

Examples:

```text id="tier3_examples"
installation
maintenance
salon services
restaurant services
repair work
consulting
```

Rules:

```text id="tier3_rules"
service area required
completion proof required
refund/cancellation terms required
performance reserve may apply
```

#### Tier 4 — Regulated / Professional Services

Examples:

```text id="tier4_examples"
lawyers
accountants
tax agents
financial advisers
credit advisers
engineers
architects
migration agents
health professionals
other regulated professionals
```

Rules:

```text id="tier4_rules"
licence / registration required
jurisdiction eligibility required
professional body membership where applicable
professional indemnity insurance required where law, regulator, professional body, or Selene policy requires
certificate of currency required where applicable
minimum cover validation required
service scope validation required
expiry monitoring required
disciplinary / complaints status where available
```

Professional indemnity insurance does not replace Selene platform reserve/deposit rules.

For professional services:

```text id="professional_split"
PI insurance = professional-liability protection where required.
Performance reserve = service/refund/platform-risk protection.
Selene B2B service fee = platform handling fee.
```

If licence, registration, insurance, jurisdiction eligibility, or professional coverage fails:

```text id="professional_fail"
Selene must block, suspend, or remove the B2B offer.
```

A lawyer without required insurance in a marketplace is not innovation. It is a courtroom origin story.

---

## 31. Reserve Release Rules

Reserve/deposit is released only when all conditions are satisfied.

```text id="reserve_release_conditions"
return period cleared
chargeback/dispute period cleared where applicable
warranty period expired if warranty reserve applies
no open returns
no open refunds
no open warranty claims
no open food/safety/authenticity claims
no fraud review
provider no longer sells the product if reserve is tied to discontinued product line
all accounting/tax settlement complete
```

If provider stops selling a warranty product:

```text id="reserve_not_auto_release"
reserve is not automatically released.
reserve remains until warranty exposure expires.
```

If provider exits Selene:

```text id="provider_exit_reserve"
reserve remains until all obligations expire or settle.
```

If provider fails:

```text id="drawdown_rule"
Selene may draw from reserve for refunds, repairs, replacements, warranty claims, safety claims, chargebacks, penalties defined in terms, or provider non-performance.
```

Correct wording:

```text id="drawdown_wording"
Selene draws down the reserve for proven obligations.
```

Not:

```text id="bad_drawdown"
Selene takes the deposit because it is annoyed.
```

Tempting, but no. That is mood-based accounting, and even accounting deserves dignity.

---

## 32. Settlement Hold Rule

Default B2B settlement model:

```text id="settlement_hold_rule"
Customer pays through Selene-controlled checkout or approved payment context.
Selene holds or controls funds through platform settlement.
Original Provider fulfills.
Customer/recipient receives product or service is accepted.
Hold period begins.
If no cancellation, return, refund, dispute, chargeback, or warranty hold occurs:
provider payout releases
channel/introducer commission releases
customer benefit pool share records if applicable
Selene fee books
reserve rules apply
```

Default hold period:

```text id="default_hold_period"
delivery confirmed + 7 days
```

Hold period may vary by:

```text id="hold_variations"
product category
service type
provider score
customer risk
return window
warranty risk
high-value item
fragile/perishable category
cross-border delivery
regulatory rule
```

Settlement states:

```text id="settlement_states"
PaymentCollected
FundsHeld
ProviderFulfillmentPending
DispatchPending
DeliveryPending
DeliveryConfirmed
HoldPeriodActive
DisputeHold
ReturnHold
WarrantyHold
ChargebackHold
Refunded
PartiallyRefunded
ProviderPayable
ChannelCommissionPayable
CustomerBenefitPoolCredited
SeleneFeeBooked
WarrantyReserveBooked
PerformanceReserveBooked
PaidOut
Reversed
Closed
```

---

## 33. Payment Source and Settlement Overlay

Customer may pay through:

```text id="payment_sources"
company store checkout
personal Selene checkout
store account
customer wallet
future Selene credit
approved payment method
POS-linked flow
enterprise account terms
```

But every B2B-sourced sale creates a B2B settlement overlay.

B2B settlement overlay controls:

```text id="settlement_overlay"
customer payment hold
provider payout
channel store commission / profit share
provider-customer introducer commission if applicable
Selene B2B fee
warranty/performance reserve
customer benefit pool share
refund reversal
return hold
chargeback hold
accounting handoff
audit
```

This solves multi-store payments.

Customer may feel they paid through Company A’s store or personal Selene.

Selene still knows whether the item is B2B-sourced and what settlement rules apply.

The customer sees a smooth checkout.

The system sees the terrifying truth. Everyone is happier this way.

---

## 34. Selene Settlement Trust Default

Selene should consider controlling settlement for all Selene-powered commerce by default, not only B2B.

Reason:

```text id="settlement_trust_reason"
customer trust
consistent refund/cancellation experience
higher purchase confidence
protection from poor individual company refund practices
cleaner accounting
better audit
```

Recommended default:

```text id="settlement_trust_default"
Customer pays through Selene.
Selene holds or controls settlement.
Goods/service delivered.
Customer has defined cancellation/return/hold period.
Funds release only when conditions clear.
```

Exceptions may include:

```text id="settlement_exceptions"
trade credit accounts
company-approved account terms
subscriptions
cash/POS payment
regulated payments
enterprise custom settlement
professional retainers
milestone services
approved alternate payment models
```

Future owner:

```text id="settlement_trust_future"
Selene Platform Settlement Trust + Customer Protection Engine
```

Document 78 defines B2B settlement needs.

The future settlement trust engine defines platform-wide settlement behavior.

Customer trust is not a feature. It is the thing that makes the purchase happen before the human changes their tiny anxious mind.

---

## 35. Payment Split and Payout Model

B2B payments may split into:

```text id="payment_split"
provider payout
channel store / introducer commission
customer benefit pool
Selene platform/service fee
payment processing fee
warranty reserve fee
performance reserve
tax/withholding reserve where applicable
refund/return reserve
first-Selene-introducer reward event where applicable
```

Payment split timing may happen:

```text id="split_timing"
at authorization
at capture
at settlement hold release
at provider payout
after refund window
after service completion
after milestone acceptance
```

Provider payout is blocked if:

```text id="provider_payout_blockers"
delivery not confirmed
service not completed
hold period active
return opened
refund requested
customer dispute
chargeback
warranty hold
provider bank unsafe
provider suspended/blocked
compliance hold
fraud signal
tax/withholding hold
```

Channel commission is blocked if:

```text id="commission_blockers"
sale refunded
return open
customer dispute
chargeback
settlement not matured
provider-customer attribution invalid
channel policy breach
fraud/self-dealing
```

Selene should never distribute money just because everyone is excited.

Excitement is not settlement proof.

---

## 36. Provider Payout

Provider payout depends on:

```text id="provider_payout_conditions"
customer payment captured
order not cancelled
provider fulfilled
dispatch proof captured where required
delivery confirmed or service accepted
hold period complete
no return/refund/dispute/chargeback/warranty hold
provider bank safe
provider not blocked/suspended
tax/withholding checks complete if applicable
settlement calculation complete
reserve/deposit obligations satisfied
```

Provider payout states:

```text id="provider_payout_states"
NotEligible
PendingFulfillment
PendingDispatch
PendingDelivery
HoldPeriodActive
DisputeHold
ReturnHold
WarrantyHold
PayoutReady
PayoutScheduled
Paid
PartiallyPaid
Reversed
Closed
```

Provider Payment handoff goes to:

```text id="provider_payment_handoff"
Supplier Payment / Provider Payout Engine
Banking / Payment Provider
BankRec
Accounting
```

B2B determines eligibility.

Payment executes.

BankRec proves.

Accounting posts.

Everyone stays in their lane. Again. The lanes are not decorative.

---

## 37. Channel / Introducer Commission

Channel or introducer commission depends on:

```text id="commission_conditions"
valid channel sale
valid provider-customer introduction
customer payment captured
order not refunded
delivery/service accepted
hold period complete
commission attribution active
fraud/self-dealing cleared
commission calculation complete
```

Commission may be:

```text id="commission_forms"
cash payout
company account credit
offset against Selene subscription/fees
marketing reward pool
customer benefit pool
store credit
monthly settlement
per-order settlement
```

Commission states:

```text id="commission_states"
Expected
PendingDelivery
PendingHoldPeriod
Payable
HeldDispute
AdjustedForRefund
Paid
Reversed
Closed
```

Channel/introducer gets paid only when it actually participated in the provider/customer relationship.

No passive “I know Harry” tax. Harry has suffered enough.

---

## 38. Customer Benefit Pool

A portion of commission may be directed to a customer benefit pool.

Customer benefit pool may fund:

```text id="customer_benefit_pool"
birthday gifts
loyalty credits
free delivery
discounts
cashback
event tickets
dinners
holidays
VIP experiences
customer-care gestures
future purchase credits
```

Rules:

```text id="benefit_pool_rules"
funded only from eligible commission/profit share
reversed if original sale is refunded where policy requires
tracked per customer / company / campaign
redeemable as configured
cash redemption only if legally/accounting permitted
```

This gives companies a reason to treat customers well using profit created through Selene.

Not just “we earned money from you.” More like “we earned money and used some of it to make you like us.” Manipulative? Mildly. Effective? Probably.

---

## 39. Selene Platform Fees

Every B2B offer may carry Selene fees.

Fee types:

```text id="selene_fee_types"
platform transaction fee
settlement fee
payment processing margin / pass-through
catalog hosting fee
premium placement fee
auto-recommendation fee
marketing/boost fee
return handling fee
dispute admin fee
warranty reserve admin fee
provider subscription fee
B2B API/network fee
private ecosystem fee
```

Default model:

```text id="default_fee_model"
Customer pays retail price.
Selene controls settlement.
Selene deducts platform/service fees and reserve where applicable.
Provider receives net payout after settlement rules.
Channel/introducer receives commission after attribution and hold rules.
Customer benefit pool receives configured allocation.
```

All fee logic must be transparent to providers/companies in business view and settlement reports.

Customers do not normally see platform fee splits unless legally required.

Selene is doing marketplace hosting, routing, settlement, audit, warranty administration, refund management, and risk monitoring. Shockingly, this costs money. Someone alert the CFO.

---

## 40. Provider Score and Marketplace Ranking

Provider score affects:

```text id="provider_score_affects"
B2B search ranking
company recommendations
auto-add eligibility
personal Selene visibility
provider payout risk
settlement hold length
reserve/deposit requirements
warranty reserve percentage
restriction/suspension
professional-service visibility
customer-visible trust signals where allowed
```

Provider score inputs:

```text id="provider_score_inputs"
delivery reliability
dispatch speed
return rate
damage rate
refund cooperation
warranty response
customer satisfaction
support response time
catalog accuracy
compliance completeness
professional insurance/licence currency where applicable
chargeback rate
dispute rate
settlement reliability
provider bank safety
profit-share attractiveness
```

Provider score states:

```text id="provider_score_states"
Excellent
Good
Watch
Restricted
Suspended
Blocked
```

Selene may tell a company:

```text id="company_warning"
“This provider has strong profit share but weak warranty response. I recommend manual review before adding their products.”
```

Selene may tell provider:

```text id="provider_warning"
“Your products are no longer eligible for auto-add because return rate exceeded policy.”
```

Not mean. Just math with receipts.

---

## 41. Provider Catalog Governance

Provider can submit changes through Product/E-Commerce and B2B update flows.

Change types:

```text id="catalog_change_types"
new product/service
price change
profit-share change
delivery zone change
warranty change
return policy change
professional insurance/licence update
media update
description update
compliance update
stock/availability update
```

Selene validates:

```text id="catalog_validation"
product/service identity
media readiness
product claims
pricing/profit-share
warranty/return terms
compliance evidence
professional eligibility
restricted product/service status
delivery feasibility
provider score
customer suitability
company adoption impact
```

Live offer changes require versioning.

```text id="versioning_rules"
Existing orders keep terms from time of purchase unless policy/law allows change.
Existing company store listings may require review if terms materially change.
Company stores must be notified of material changes.
Customer-facing display must update after approval.
```

Example:

```text id="warranty_change_example"
Provider changes warranty from 12 months to 6 months.
Selene preserves old warranty for existing orders.
Selene reviews new terms before future display.
```

Providers do not get to rewrite history because their warranty team panicked.

---

## 42. B2B Services

B2B supports services, not only products.

Examples:

```text id="b2b_service_examples"
installation
maintenance
consulting
design
cleaning
training
repair
appointment service
restaurant booking / pre-order service
digital service
subscription service
logistics service
warranty service
professional service
```

Service fields:

```text id="service_fields"
service_area
jurisdiction
duration
availability
booking rules
cancellation rules
service deliverables
provider staff qualifications
service evidence required
completion acceptance
service guarantee / warranty
refund rules
profit share
professional insurance/licence if applicable
```

Service settlement:

```text id="service_settlement"
customer pays
Selene holds funds
provider performs service
customer/service acceptance recorded
hold period begins if applicable
provider paid
channel/introducer commission paid if applicable
Selene fee kept
```

Service proof matters.

“We did the job” is not proof. It is a sentence with an invoice attached.

---

## 43. Professional Services Compliance Gate

Professional and regulated services require a stricter B2B gate.

Professional examples:

```text id="professional_services"
lawyers
accountants
tax agents
financial advisers
credit advisers
engineers
architects
migration agents
health professionals
regulated consultants
```

Required before B2B display:

```text id="professional_requirements"
licence / registration
jurisdiction eligibility
professional body membership where applicable
professional indemnity insurance where required
certificate of currency where applicable
minimum cover validation
service scope validation
expiry monitoring
disciplinary / complaints status where available
customer jurisdiction matching
```

If any required element is missing:

```text id="professional_block"
Selene blocks, suspends, or removes the professional service offer.
```

Professional indemnity insurance is separate from provider deposit/reserve.

```text id="pi_vs_reserve"
PI insurance = professional-liability cover.
Selene reserve/deposit = platform/refund/performance/customer protection cover.
```

A professional may need both.

Because one protects against professional negligence, and the other protects against platform commerce failure. Two problems, two buckets. Truly, what a feast.

---

## 44. International Shipping, Duties, and Taxes

B2B offers must include international feasibility data where relevant.

Business view shows:

```text id="international_business_fields"
delivery countries/regions
international delivery estimate
shipping cost
customs duty estimate
import tax / VAT / GST estimate
brokerage/admin fee estimate
restricted goods risk
return difficulty
who pays duties
whether duties are prepaid
landed cost estimate
```

Customer view shows:

```text id="international_customer_fields"
estimated delivered total
estimated duties/taxes if applicable
who pays duties
delivery timeframe
return warning if international return is difficult
```

Tax Engine owns final tax treatment.

B2B provides facts and triggers.

E-Commerce shows customer estimates.

Accounting posts.

No surprise customs mugging at the doorstep. Selene should not let customs show up like a villain at the door.

---

## 45. Marketplace Disclosure Rules

Selene must support disclosure rules by jurisdiction and business model.

Disclosure may include:

```text id="disclosure_types"
provider/seller identity
fulfilled-by provider
provider location
warranty provider
return responsibility
delivery estimate
tax-inclusive/exclusive pricing
third-party marketplace indicator
service provider identity
professional credentials
merchant-of-record information
```

Disclosure configuration depends on:

```text id="disclosure_config"
jurisdiction
customer type
merchant of record
product/service category
provider risk
marketplace law
consumer law
professional services law
tax law
company policy
```

Customer-facing disclosure should be simple.

Example:

```text id="disclosure_example"
“Fulfilled by an approved Selene provider. Returns and warranty are managed through Selene.”
```

Not:

```text id="bad_disclosure_example"
“Here is the platform liability stack and settlement waterfall.”
```

No one invited the legal octopus to the product card.

---

## 46. Merchant of Record / Platform Role Configuration

Legal/commercial model must be configurable.

Possible roles:

```text id="merchant_roles"
company store as merchant of record
Original Provider as merchant of record
Selene marketplace as settlement/platform layer
licensed payment partner / platform facilitator
jurisdiction-specific hybrid model
```

This affects:

```text id="mor_effects"
tax
refund responsibility
chargeback responsibility
customer disclosure
settlement timing
payout permissions
payment licensing
accounting
legal liability
warranty obligations
```

B2B stores configured model per jurisdiction/store/provider/order.

B2B does not decide legal treatment alone.

Legal/Compliance/Tax engines must review where required.

This is where architecture whispers: “Do not accidentally become a regulated payments/tax/insurance creature.” We listen, because fines have no sense of humor.

---

## 47. Customer Privacy and Company Data Walls

B2B must respect customer and company data walls.

Company A can see:

```text id="company_can_see"
Company A customer relationship
Company A store B2B sales
Company A profit share
Company A customer orders through its store
Company A marketing/customer account permissions
Company A channel attribution where applicable
```

Company A cannot see:

```text id="company_cannot_see"
customer purchases from Company B
customer personal Selene search history
customer purchases through personal Selene home unless attributed/shared by policy
other companies’ customers
provider private terms not made available
```

Original Provider can see:

```text id="provider_can_see"
customer purchases from that provider
fulfillment-required order data
support/warranty/return data for its products/services
marketing permission if granted
```

Original Provider cannot see:

```text id="provider_cannot_see"
customer full personal Selene profile
unrelated store activity
unrelated family/friend relationships
Channel Store private customer data unrelated to provider attribution
```

Privacy rule:

```text id="privacy_rule"
Data visibility follows role, purpose, consent, law, and audit.
```

Selene should not become creepy just because it can. This will be difficult for a platform. We shall try to behave like adults.

---

## 48. B2B and Customer Memory

B2B may use permitted customer signals to improve recommendations, but it does not own customer memory.

Permitted signals may include:

```text id="customer_signals"
category interest
style preferences
size preferences
purchase history
wishlist
delivery region
price comfort
return preferences
gift recipient patterns
store relationships
provider-customer relationships
```

Rules:

```text id="memory_rules"
Customer Engine owns customer memory.
B2B uses only allowed signals.
Providers do not receive raw customer memory.
Companies only receive relationship data they are allowed to see.
```

Example:

```text id="memory_example"
Selene may recommend shoes that match a dress.
Provider does not learn the customer’s closet history.
```

Providers get fulfillment/support data, not a personal-shopping diary.

Let’s not be horrifying just because the database can.

---

## 49. B2B and Search Priority

B2B provides search signals.

E-Commerce/Search decides customer-facing display.

B2B provides:

```text id="b2b_search_signals"
offer eligibility
provider score
delivery estimate
return/warranty quality
customer-visible price
company adoption state
store mode
personal commerce eligibility
provider-customer attribution
commission eligibility
professional compliance status
reserve/deposit risk class
```

Search priority respects:

```text id="search_priority"
current store context
company store modes
linked company relationships
provider-customer attribution
first verified introducer boost where relevant
customer preference
provider quality
availability
delivery
price
warranty/returns
```

B2B must provide clean signals.

E-Commerce decides how to speak and display.

Selene does not narrate B2B search layers to customers unless asked.

Customers want results, not a guided tour of the ranking basement.

---

## 50. B2B and Referral / Rewards

B2B emits events to the future Rewards Engine.

Events include:

```text id="reward_events"
B2B sale completed
B2B sale refunded
B2B return completed
channel commission earned
channel commission reversed
provider-customer attribution created
provider-customer attribution expired
first verified introducer eligible purchase
customer benefit pool credit created
provider sale attributed
employee/friend/family referral eligible purchase
```

B2B does not calculate final referral rewards.

Future owner:

```text id="reward_owner"
Selene Referral, Rewards, Loyalty + Attribution Engine
```

Key rule:

```text id="reward_key_rule"
First verified introducer wins personal intro reward.
Second and later personal introducers do not receive intro reward for the same person.
```

Provider-customer commission is separate from first-introducer reward.

Channel profit share is separate from both.

Do not mix the buckets unless you enjoy arguing with accounting and lawyers while holding a spreadsheet on fire.

---

## 51. B2B Platform Fraud and Abuse Detection

Fraud and abuse signals:

```text id="fraud_signals"
fake provider
counterfeit products
fake professional credentials
expired PI insurance
provider self-buying
fake reviews
abnormal refund rate
abnormal chargeback rate
customer complaints spike
provider changes bank details before payout
provider fulfillment proof mismatch
provider tries to bypass Selene settlement
reseller collusion
profit-share manipulation
duplicate provider listings
warranty avoidance
self-referral abuse
first-introducer fraud
provider-customer attribution manipulation
customer benefit pool abuse
```

Actions:

```text id="fraud_actions"
hold payout
remove product/service
suspend provider
block auto-add
freeze channel commission
freeze reward attribution
draw reserve where valid
route fraud review
notify affected engines
preserve audit evidence
```

Marketplace fraud loves vague systems.

Selene should be extremely inconvenient. Ideally exhausting.

---

## 52. Provider Performance and Restrictions

Provider performance affects:

```text id="provider_performance_effects"
store adoption eligibility
personal commerce visibility
auto-add eligibility
payout risk
settlement hold length
reserve/deposit requirement
warranty reserve percentage
professional-service visibility
recommendation ranking
provider status
```

Restriction actions:

```text id="restriction_actions"
watchlist
restrict categories
disable auto-add
require manual approval
increase settlement hold
increase reserve/deposit
pause offers
suspend provider
block provider
retire provider
```

Selene may tell a company:

```text id="company_warning"
“This provider has strong profit share but weak warranty response. I recommend manual review before adding their products.”
```

Selene may tell provider:

```text id="provider_warning"
“Your products are no longer eligible for auto-add because return rate exceeded policy.”
```

Not mean. Just math with receipts.

---

## 53. Refund, Commission, and Reserve Reversal

B2B settlement must reverse correctly.

If order is refunded:

```text id="refund_reversal"
provider payout not released or reversed if already released
channel commission not released or reversed if already released
customer benefit pool reduced/reversed where policy requires
Selene fee reversal depends on policy
reserve may be drawn down
reward events reversed where applicable
accounting handoff updated
tax treatment updated
```

If partial refund:

```text id="partial_refund"
provider payout adjusted
channel commission adjusted
customer benefit pool adjusted
fees adjusted if policy
reserve adjusted if policy
tax/accounting event adjusted
```

If replacement instead of refund:

```text id="replacement"
provider obligation remains open until replacement accepted
provider payout may stay held
channel commission may stay held
warranty/return score updated
```

Selene B2B does not let money become final before the customer outcome is stable.

This is how marketplaces avoid turning customer service into medieval debt collection.

---

## 54. B2B and Accounting Handoff

B2B sends structured financial events.

Handoff includes:

```text id="accounting_handoff"
customer gross payment
tax amount
provider payable
channel commission
customer benefit pool amount
Selene platform fee
warranty reserve fee
performance reserve
payment processing fee
refund reserve
chargeback reserve
settlement hold
delivery confirmation
hold release
payout release
refund reversal
commission reversal
provider liability
reserve drawdown
audit_ref
```

Accounting owner split:

```text id="accounting_owner_split"
Company Accounting = company revenue/profit share/store treatment
Provider Accounting = provider revenue/payout treatment
Selene Accounting = platform fee, reserve, settlement liabilities
BankRec = cash proof
Tax = tax/duties/VAT/GST/withholding treatment
```

B2B does not post ledger.

B2B provides structured truth.

Accounting posts.

This line appears often because financial modules try to wander. Like cats. Expensive cats.

---

## 55. B2B and Tax / Duties

B2B must provide tax-relevant data.

Tax-relevant fields:

```text id="tax_fields"
provider jurisdiction
company/store jurisdiction
customer jurisdiction
recipient/delivery jurisdiction
product/service tax category
merchant-of-record model
marketplace facilitator model if applicable
VAT/GST/sales tax candidate
import duties
withholding tax flag
digital service tax flag
professional service tax classification
shipping/tax treatment
refund/reversal tax impact
```

Tax Engine owns final treatment.

B2B provides facts and triggers.

E-Commerce shows customer estimates.

Accounting posts.

If international shipment:

```text id="international_tax_flow"
B2B supplies duty/tax estimate inputs.
Tax/Compliance evaluates.
E-Commerce displays landed cost estimate.
Order/Payment captures/settles according to configured model.
```

No surprise duty ambush. Selene should not let customs show up like a villain at the door.

---

## 56. B2B and Customer Credit / Wallet

B2B orders may be paid by:

```text id="credit_wallet_methods"
card
wallet
store account
Selene wallet
reward credit
customer credit
installment plan
company account terms
future virtual Selene money
```

B2B must know whether:

```text id="credit_impacts"
provider can be paid before customer repayment
Selene carries credit risk
company carries credit risk
provider payout waits for payment clearance
installment/refund reversal applies
reward credit reduces settlement base
```

Future owner:

```text id="credit_wallet_owner"
Selene Customer Credit, Wallet, Store Account, Installment + Virtual Settlement Engine
```

B2B must integrate but not own credit underwriting.

We are not letting a marketplace quietly become a lender behind the curtain like a raccoon opening a bank.

---

## 57. Conversation-to-Action Guardrail

B2B conversations are not fixed phrase matching.

Business user may say:

```text id="business_phrases"
“Show me products suitable for my customers.”
“Add these to my store.”
“Only show products above 20% profit share.”
“Do not add competitor shoes.”
“Find products with local delivery.”
```

Provider may say:

```text id="provider_phrases"
“Update warranty terms.”
“Pause this product.”
“Where is my payout?”
“Change delivery zones.”
“Add this to B2B.”
```

Customer may trigger B2B indirectly:

```text id="customer_phrases"
“Find shoes like this.”
“Send it to mum.”
“Return this.”
“Find rolls of wire.”
```

Correct flow:

```text id="conversation_flow"
GPT-5.5 interprets natural language.
PH1.X resolves live context.
PH1.M / relevant company/provider/customer memory resolves durable context.
B2B deterministic rules verify provider, offer, eligibility, terms, payout, visibility, settlement, reserves, attribution, and policy before action.
Protected actions require authority.
Everything important is audited.
```

GPT-5.5 may:

```text id="gpt_may"
explain offers
compare provider products
summarize profit-share opportunity
draft provider messages
draft company recommendations
translate terms into plain language
summarize payout status
```

GPT-5.5 must not:

```text id="gpt_must_not"
approve provider
release payout
invent provider terms
invent warranty
invent professional insurance
invent compliance certificates
invent fulfillment proof
auto-add restricted products without policy
override company store mode
override refund rules
approve settlement exceptions
decide legal merchant-of-record status
post accounting
```

GPT speaks.

B2B verifies.

Authority approves protected moves.

Audit records.

The shopping toaster has again been avoided.

---

## 58. Human-Like Selene Interaction

### Company browsing B2B

```text id="company_browse_phrase"
“I found products that fit your customers and meet your profit-share rules.”
```

### Company auto-add

```text id="auto_add_phrase"
“I added these as draft listings because your policy requires review before publishing.”
```

### Provider payout

```text id="provider_payout_phrase"
“The customer received the order. Your payout will release after the hold period if no return opens.”
```

### Channel commission

```text id="channel_commission_phrase"
“You earned commission on this B2B sale. It becomes payable after the customer hold clears.”
```

### Provider support routing

```text id="provider_support_phrase"
“This is a product-specific question. I’ll route it to the original provider and keep the answer for future customers if approved.”
```

### Warranty issue

```text id="warranty_phrase"
“This warranty claim is now overdue. I’m restricting new auto-add eligibility until it is resolved.”
```

### Customer-facing B2B offer

```text id="customer_offer_phrase"
“This one can arrive Friday and has better return cover.”
```

Customer does not hear about profit share.

Provider does not receive private customer memory.

Company sees business opportunity.

Selene speaks to each party like a human who knows what they need, not like one dashboard trying to impress four species at once.

---

## 59. Automation and Exception-Only Review

Selene auto-handles:

```text id="auto_handles"
Product-to-B2B intake after readiness
provider document extraction
provider terms tracking
provider catalog validation under policy
provider score updates
business B2B search
product/service recommendation to companies
auto-draft listings
auto-add under policy
customer-visible B2B eligibility
provider support routing
provider-customer relationship event creation
profit-share calculation
commission attribution event creation
settlement hold tracking
provider payout eligibility
channel commission eligibility
customer benefit pool crediting
Selene fee calculation
warranty reserve calculation
performance reserve calculation
provider performance warnings
routine payout reports
routine customer/provider/company updates
reward attribution event generation
```

Selene escalates:

```text id="escalates"
high-risk provider
provider missing terms
provider compliance issue
restricted product/service
professional-service compliance failure
missing PI insurance where required
large payout
provider bank change
high return/chargeback rate
orphan warranty claim
provider insolvency/disappearance
consumer harm risk
legal/regulatory issue
tax/MoR uncertainty
cross-border risk
reserve/deposit dispute
provider-customer attribution dispute
self-referral/reward abuse
manual override
```

Rule:

```text id="automation_rule"
Routine marketplace operations = Selene handles.
Risky provider/customer/money/warranty issues = Selene escalates.
Protected legal/payment/tax/professional issues = authority/review.
Everything important = audited.
```

Maximum automation. Minimum stupidity. A noble, if frequently ignored, software principle.

---

## 60. State Machines

### 60.1 Product-to-B2B Offer State

```text id="state_product_to_b2b"
ProductCreated
ECommerceReady
B2BOptInOffered
B2BOptInAccepted
B2BReadinessChecking
B2BReadinessFailed
B2BQualified
PublishedToB2B
Paused
Withdrawn
Archived
```

### 60.2 Provider State

```text id="state_provider"
Draft
PendingInformation
PendingVerification
PendingTermsAcceptance
PendingApproval
Approved
Active
Watchlist
Restricted
Suspended
Blocked
Retired
```

### 60.3 B2B Offer State

```text id="state_offer"
Draft
ProviderSubmitted
ValidationPending
Approved
EligibleForRecommendation
Recommended
AdoptedByCompany
AutoAddedDraft
PublishedToStore
EligibleForPersonalSelene
Paused
Restricted
Withdrawn
Archived
```

### 60.4 Company Adoption State

```text id="state_adoption"
NotOffered
Recommended
Viewed
DraftCreated
Approved
Published
TestAudience
Paused
Rejected
Removed
Archived
```

### 60.5 Provider-Customer Attribution State

```text id="state_provider_customer_attribution"
NoAttribution
ChannelPurchaseDetected
ProviderCustomerRelationshipCreated
OngoingCommissionActive
CommissionPaused
CommissionReversed
AttributionDisputed
AttributionExpired
Closed
```

### 60.6 B2B Settlement State

```text id="state_settlement"
PaymentCollected
FundsHeld
ProviderFulfillmentPending
DispatchPending
DeliveryPending
DeliveryConfirmed
HoldPeriodActive
DisputeHold
ReturnHold
WarrantyHold
ChargebackHold
Refunded
PartiallyRefunded
ProviderPayable
ChannelCommissionPayable
CustomerBenefitPoolCredited
SeleneFeeBooked
WarrantyReserveBooked
PerformanceReserveBooked
PaidOut
Reversed
Closed
```

### 60.7 Provider Payout State

```text id="state_provider_payout"
NotEligible
PendingFulfillment
PendingDispatch
PendingDelivery
HoldPeriodActive
DisputeHold
ReturnHold
WarrantyHold
PayoutReady
PayoutScheduled
Paid
PartiallyPaid
Reversed
Closed
```

### 60.8 Channel Commission State

```text id="state_channel_commission"
Expected
PendingDelivery
PendingHoldPeriod
Payable
HeldDispute
AdjustedForRefund
Paid
Reversed
Closed
```

### 60.9 Reserve State

```text id="state_reserve"
NotRequired
Required
Funded
Active
DrawdownPending
PartiallyDrawn
FullyDrawn
ReleaseEligible
Released
Disputed
Closed
```

### 60.10 Warranty Protection State

```text id="state_warranty_protection"
ProviderResponsible
ClaimOpened
ProviderReviewing
ProviderFailed
OrphanProtectionReview
ReserveEligible
RepairApproved
ReplacementApproved
RefundApproved
Rejected
Resolved
Closed
```

### 60.11 B2B Auto-Add State

```text id="state_auto_add"
Disabled
RecommendationOnly
DraftOnly
LimitedAudienceTest
AutoPublishUnderPolicy
HumanApprovalRequired
Paused
Blocked
```

### 60.12 Professional Compliance State

```text id="state_professional_compliance"
NotProfessionalService
ComplianceRequired
LicencePending
InsurancePending
CertificatePending
ComplianceVerified
ComplianceExpired
Suspended
Blocked
```

---

## 61. Reason Codes

```text id="reason_codes"
PRODUCT_B2B_OPT_IN_OFFERED
PRODUCT_B2B_OPT_IN_ACCEPTED
PRODUCT_B2B_READINESS_CHECK_STARTED
PRODUCT_B2B_READINESS_FAILED
PRODUCT_B2B_QUALIFIED
B2B_PROVIDER_CAPTURED
B2B_PROVIDER_PENDING_TERMS
B2B_PROVIDER_APPROVED
B2B_PROVIDER_RESTRICTED
B2B_PROVIDER_BLOCKED
B2B_OFFER_SUBMITTED
B2B_OFFER_VALIDATED
B2B_OFFER_COMPLIANCE_HOLD
B2B_OFFER_RECOMMENDED
B2B_OFFER_AUTO_ADDED_DRAFT
B2B_OFFER_PUBLISHED_TO_STORE
B2B_OFFER_ELIGIBLE_PERSONAL_SELENE
B2B_PROVIDER_SCORE_LOW
ORIGINAL_PROVIDER_OF_RECORD_IDENTIFIED
CHANNEL_STORE_IDENTIFIED
PROVIDER_CUSTOMER_RELATIONSHIP_CREATED
ONGOING_PROVIDER_COMMISSION_ACTIVE
NO_CHANNEL_COMMISSION_APPLIES
B2B_PROFIT_SHARE_CALCULATED
B2B_COMMISSION_ATTRIBUTION_CREATED
B2B_CUSTOMER_BENEFIT_POOL_CREDITED
B2B_SETTLEMENT_FUNDS_HELD
B2B_DELIVERY_CONFIRMED
B2B_HOLD_PERIOD_ACTIVE
B2B_PROVIDER_PAYOUT_READY
B2B_PROVIDER_PAYOUT_HELD
B2B_CHANNEL_COMMISSION_PAYABLE
B2B_PLATFORM_FEE_BOOKED
B2B_PERFORMANCE_RESERVE_BOOKED
B2B_WARRANTY_RESERVE_BOOKED
B2B_RETURN_OPENED
B2B_REFUND_ISSUED
B2B_PROVIDER_WARRANTY_FAILED
B2B_ORPHAN_WARRANTY_REVIEW
B2B_AUTO_ADD_POLICY_BLOCKED
B2B_FRAUD_REVIEW_REQUIRED
B2B_BANK_CHANGE_PAYOUT_HOLD
B2B_FIRST_INTRODUCER_EVENT_CREATED
B2B_REWARD_REVERSAL_EVENT_CREATED
B2B_PROVIDER_TERMS_CHANGED
B2B_MERCHANT_OF_RECORD_REVIEW_REQUIRED
B2B_INTERNATIONAL_DUTY_ESTIMATE_REQUIRED
B2B_PROFESSIONAL_SERVICE_COMPLIANCE_REQUIRED
B2B_PROFESSIONAL_INSURANCE_REQUIRED
B2B_PROFESSIONAL_COMPLIANCE_EXPIRED
B2B_RESERVE_DRAWDOWN_REQUIRED
```

---

## 62. Required Simulations

```text id="required_simulations"
provider product originates from Product/E-Commerce B2B opt-in
product fails B2B readiness due to no delivery model
product qualifies for B2B with delivery/warranty/return rules
provider accepts Selene B2B terms
provider rejects refund terms and is blocked
provider product submitted
provider service submitted
professional service blocked due to missing PI insurance
B2B offer recommended to company
company approves B2B offer
Selene auto-adds B2B offer as draft
Selene auto-publishes under company policy
company blocks competitor category
company allows competitor inclusion
customer buys B2B product through company store
customer buys B2B product through personal Selene search
Harry buys Supplier 44 wire directly through personal Selene
ABC receives no commission because it did not introduce Supplier 44/product
Hair Salon sells Wine Store wine
Wine Store remains Original Provider
Hair Salon earns channel commission
customer later buys directly from Wine Store
Hair Salon earns ongoing commission under attribution policy
Original Provider handles faulty goods
Channel Store has no operational responsibility
provider support question routed to Original Provider
provider-approved answer reused later
provider fulfills order
dispatch proof created
delivery confirmed starts hold period
provider payout released after hold
channel commission paid after hold
customer benefit pool credited
Selene platform fee deducted
warranty reserve booked
performance reserve booked
provider reserve drawn down for proven failure
customer cancels before dispatch
customer return pauses provider payout
refund reverses channel commission
refund reverses customer benefit pool
provider warranty claim succeeds
provider disappears and orphan warranty review opens
provider score drops from high return rate
provider bank change blocks payout
B2B international duties estimated
business user searches B2B by profit share
business user asks Selene to add products to store
GPT-5.5 interprets request but B2B policy verifies before action
first verified introducer event created
second introducer ignored by Reward Engine handoff
B2B accounting handoff created
```

---

## 63. Integration Map

```text id="integration_map"
PH1.B2B_PLATFORM / TRADE_ECOSYSTEM
↔ PH1.PRODUCT
↔ PH1.PRODUCT_B2B_READINESS
↔ PH1.ECOMMERCE
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_LINKING / STORE_RELATIONSHIP
↔ PH1.REFERRAL_REWARDS / LOYALTY_ATTRIBUTION
↔ PH1.ORIGINAL_PROVIDER / PROVIDER_OF_RECORD
↔ PH1.PROVIDER_CUSTOMER_RELATIONSHIP
↔ PH1.INVENTORY
↔ PH1.ORDER / ORCHESTRATION
↔ PH1.PRICING / MARGIN / DISCOUNT
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.CUSTOMER_CREDIT / WALLET / VIRTUAL_SETTLEMENT
↔ PH1.SETTLEMENT_TRUST / CUSTOMER_PROTECTION
↔ PH1.DISPATCH / PACKING / COURIER_HANDOFF
↔ PH1.RETURNS / REFUNDS / REVERSE_LOGISTICS
↔ PH1.WARRANTY / AFTER_SALES
↔ PH1.PROFESSIONAL_SERVICES_COMPLIANCE
↔ PH1.SUPPLIER / PROVIDER
↔ PH1.SUPPLIER.BANK_TRUST
↔ PH1.SUPPLIER_PAYMENT / PROVIDER_PAYOUT
↔ PH1.BANKING / PAYMENT_PROVIDER
↔ PH1.BANKREC / TREASURY
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.COMPLIANCE
↔ PH1.LEGAL / CONTRACTS
↔ PH1.SAAS_TENANCY / DEVICE_ACCESS
↔ PH1.LOGISTICS / DELIVERY
↔ PH1.MARKETING
↔ PH1.EVENTS / INVITATIONS / RSVP
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.REM
↔ PH1.BCAST / DELIVERY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
```

---

## 64. Required Logical Packets

```text id="logical_packets"
ProductToB2BOptInPacket
ProductToB2BReadinessPacket
B2BQualifiedOfferPacket
B2BProviderPacket
B2BProviderOnboardingPacket
B2BProviderTermsPacket
B2BProviderVerificationPacket
OriginalProviderOfRecordPacket
ChannelStoreAttributionPacket
ProviderCustomerRelationshipPacket
ProviderCustomerCommissionPacket
CustomerBenefitPoolPacket
B2BOfferPacket
B2BProductOfferPacket
B2BServiceOfferPacket
B2BOfferEligibilityPacket
B2BBusinessViewPacket
B2BCustomerViewPacket
B2BRecommendationPacket
B2BAutoAddPolicyPacket
B2BCompanyAdoptionPacket
B2BCustomerDisplayPacket
B2BOrderParticipationPacket
B2BFulfillmentObligationPacket
B2BDispatchHandoffPacket
B2BSettlementPacket
B2BPaymentSplitPacket
B2BProviderPayoutPacket
B2BChannelCommissionPacket
B2BPlatformFeePacket
B2BPerformanceReservePacket
B2BWarrantyReservePacket
B2BReserveDrawdownPacket
B2BReturnObligationPacket
B2BRefundAdjustmentPacket
B2BWarrantyProtectionPacket
B2BProfessionalCompliancePacket
B2BProviderScorePacket
B2BProviderRiskPacket
B2BFraudRiskPacket
B2BMerchantOfRecordConfigPacket
B2BTaxDutyEstimatePacket
B2BRewardAttributionEventPacket
B2BAccountingHandoffPacket
B2BAuditEvidencePacket
```

Logical only.

Codex maps later.

No runtime packet structs. The schema goblin can keep its tiny claws off the architecture.

---

## 65. What Codex Must Not Do

```text id="codex_must_not"
Do not make individual companies owners of B2B platforms.
Do not let B2B create product truth from scratch.
Do not bypass Product-to-B2B readiness.
Do not merge B2B Platform into E-Commerce.
Do not let B2B own customer-facing shopping conversation.
Do not let B2B own customer memory master truth.
Do not let B2B own product identity master truth.
Do not let B2B own inventory stock truth.
Do not pay provider before fulfillment/hold/dispute rules clear.
Do not pay channel commission where no valid channel/provider attribution exists.
Do not pay channel commission after refunded/returned orders unless policy explicitly says.
Do not make Channel Store responsible for Original Provider faults unless explicitly contracted.
Do not bypass provider terms acceptance.
Do not bypass warranty/guarantee obligations.
Do not bypass professional licence/insurance/compliance checks.
Do not make Selene warranty protection look like insurance without legal/compliance review.
Do not expose customer private memory to providers.
Do not expose business profit share to customers unless required.
Do not let multiple personal introducers earn intro reward for the same customer.
Do not let GPT-5.5 invent provider terms, warranty, certifications, insurance, licences, fulfillment proof, provider scores, or payout status.
Do not let GPT-5.5 approve protected B2B actions.
Do not create runtime marketplace/payment code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 66. Final Architecture Sentence

Selene B2B Platform + Trade Ecosystem Engine is the Selene-owned central marketplace and trade brain that receives only Product/E-Commerce-qualified products and services; governs Product-to-B2B opt-in, business view, customer view, company adoption, autonomous B2B expansion, Original Provider responsibility, Channel Store attribution, provider-customer relationship creation, ongoing provider-customer commission, customer benefit pools, Selene B2B fees, provider performance reserves, warranty/guarantee reserves, professional-service compliance, settlement holds, provider payout, commission reversal, refund/return/warranty obligations, international duties/taxes, merchant-of-record configuration, provider scoring, fraud control, and accounting handoff; while keeping customer-facing commerce simple, human-like, private, and result-focused through E-Commerce.

Simple version:

```text id="simple_version"
Selene owns B2B.
Companies own their stores.
Products/services start in Product/E-Commerce.
Provider opts into B2B during product setup.
Only qualified offers enter B2B.
Original Provider owns product/service responsibility.
Channel Store can sell and earn without headaches.
Customer can buy through company store or personal Selene.
If customer buys directly from B2B, no random company gets commission.
Commission requires actual channel/provider attribution.
Future provider purchases can keep paying the introducer if policy applies.
B2B always charges Selene platform/service fees.
Provider may need reserves/deposits based on risk.
Food/authenticity/safety/warranty products may require reserves.
Professional services may require licence, registration, and professional indemnity insurance.
Selene holds settlement until fulfillment and hold rules clear.
Refunds, returns, disputes, and warranty issues pause or reverse money.
Customer sees retail product info, not backend profit share.
Company sees business profit/margin/provider details.
GPT-5.5 makes interactions human-like.
Deterministic B2B rules verify before action.
Everything important is audited.
```

That is Global Document 78 — Selene B2B Platform + Trade Ecosystem Engine v4. It is not a wholesale catalog. It is the Selene-owned trade network where Product-qualified offers become distributable, companies earn by introducing or carrying provider products, original providers keep responsibility, customers stay protected, Selene controls settlement, deposits keep providers honest, and the whole thing does not collapse into a commission swamp with better product photos.

---

## 67. 81E B2B Pricing Stack + Brand Approval Pricing Handoff

Document 78 must request 81E for B2B pricing stack viability, provider net, Channel Store commission, Selene B2B fee, customer benefit pool, warranty/performance reserve, delivery cost, return courier/reverse logistics cost, payment cost, refund/reversal/clawback exposure, settlement hold, provider risk score pricing effect, contribution classification, bottom-line profit target alignment, B2B price floor, B2B price ceiling, and profit-share waterfall.

Document 78 must enforce brand approval or referral-only outcomes before listing or channel adoption. Brand approval outcomes from 81D, when available, must feed 81E before final B2B listing, adoption, commission, customer benefit, or dynamic pricing decisions are allowed.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
