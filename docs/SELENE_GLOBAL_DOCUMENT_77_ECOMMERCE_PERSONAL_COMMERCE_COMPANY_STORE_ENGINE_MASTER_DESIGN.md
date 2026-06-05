# Global Document 77 — Selene E-Commerce Engine v7

## Build-Ready Separated Version

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
77

ENGINE:
PH1.ECOMMERCE / PH1.PERSONAL_COMMERCE / PH1.COMPANY_STORE_COMMERCE

FULL NAME:
Selene Personal Commerce, Company E-Commerce Store, Multi-Store Customer Relationship, Customer-Facing Search, Visual Product Discovery, Conversational Shopping, One-Command Checkout, Recipient Delivery, Unified Buying Lists, Customer-Facing Returns, Warranty, Order Tracking, B2B Display Handoff, Settlement Trust Handoff, and Human-Like Commerce Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene E-Commerce Engine owns the **customer-facing commerce experience**.

It is the layer where customers:

```text
search
browse
ask
compare
watch
buy
reorder
book
send gifts
track orders
cancel orders
start returns
ask warranty questions
build buying lists
use store accounts
use personal Selene commerce
```

It is not the B2B commercial engine.

It is not the payment engine.

It is not the settlement engine.

It is not the rewards engine.

It is not the dispatch engine.

It is the customer and company store surface that makes the whole commerce world feel simple.

Customer can say:

```text
“Selene, find me the latest shoes.”
“Buy that one and send it to my mum’s house.”
“Add milk to my groceries.”
“Cancel those shoes I bought.”
“Where is that dress I ordered?”
“Book dinner for Friday.”
“Take this photo and find something similar.”
“Find rolls of wire.”
“Use my usual card.”
“Deliver it to the office.”
```

Selene must make that feel natural while the correct backend engines handle truth, money, delivery, B2B attribution, provider responsibility, accounting, and audit.

In short:

```text
Document 77 = customer-facing commerce.
Document 78 = B2B marketplace, provider, commission, settlement, reserves.
Document 80 = order orchestration.
Document 82 = dispatch.
Document 83 = returns.
Future engines = credit, rewards, memory patterns, events, SaaS tenancy.
```

Tiny miracle: one engine does not get to own the entire shopping universe. We are learning restraint. Barely.

---

## 2. Core E-Commerce Law

```text
Customers are free.
Company store relationships are protected.
E-Commerce shows useful results, not backend plumbing.
Company stores control their own store context.
Customers can search broadly through personal Selene commerce.
B2B products can display in E-Commerce only through B2B eligibility and store policy.
Original Provider responsibility is routed through B2B.
Payments and settlement are handled by Payment / Settlement engines.
Protected actions require secure verification.
Everything important is audited.
```

A company can link a customer.

A company can sell to that customer.

A company can market to that customer if permission exists.

A company can offer account terms, events, loyalty, promotions, and B2B-expanded products.

But:

```text
The company does not own the whole customer.
```

Customers can shop anywhere, like the real world, because apparently humans keep insisting on freedom. Very inconvenient for loyalty departments.

---

## 3. Engine Ownership Boundary

### 3.1 E-Commerce owns

```text
customer-facing company store
customer personal Selene commerce home
customer search and product/service discovery
typed / voice / scroll / photo shopping interface
customer-facing product cards, photos, videos, descriptions
customer-facing B2B display eligibility request
store context handling
personal search context handling
one-command checkout interaction
payment permission request
recipient delivery request
recipient address request
gift / quiet mode interface
customer-facing order tracking
customer-facing cancellation request
customer-facing return request
customer-facing warranty request
unified customer buying lists
restaurant/service booking intent interface
customer-facing settlement trust explanation
customer-facing company/store relationship experience
handoffs to B2B, Order, Payment, Dispatch, Returns, Warranty, Accounting, Tax, Rewards, and Audit
```

### 3.2 E-Commerce references but does not own

```text
B2B provider responsibility
Product-to-B2B readiness
Original Provider of Record rules
Channel Store attribution
provider-customer commission
provider deposits/reserves
Selene B2B platform fees
professional-service licence/insurance validation
final reward calculation
payment authorization/capture
settlement release
provider payout
ledger posting
tax treatment
inventory truth
dispatch operations
return logistics
warranty claim execution
global customer memory ownership
customer credit underwriting
SaaS tenancy / hosting / billing
```

### 3.3 Correct owner split

```text
PH1.ECOMMERCE = customer-facing shopping and store experience.
PH1.B2B_PLATFORM = B2B eligibility, provider, channel, commission, reserves, settlement rules.
PH1.PRODUCT = product/service truth and Product-to-B2B readiness.
PH1.INVENTORY = stock truth.
PH1.ORDER = order lifecycle.
PH1.PAYMENT / SETTLEMENT = authorization, capture, refund, settlement hold, release.
PH1.DISPATCH = picking, packing, courier handoff.
PH1.RETURNS = reverse logistics and refund execution.
PH1.WARRANTY = warranty and orphan-provider workflows.
PH1.CUSTOMER = customer identity, relationships, memory, preferences.
PH1.REWARDS = referral, loyalty, attribution, reward math.
PH1.ACCOUNTING = ledger posting.
PH1.AUDIT = evidence.
```

This boundary is not decorative. It is the fence keeping Codex from building a five-headed checkout hydra.

---

## 4. SaaS / Cloud Assumption

Selene E-Commerce runs inside the Selene SaaS platform.

```text
Selene Inc provides the cloud platform.
Each company has a Selene company tenant / environment.
Some companies use shared hosting.
Some companies use dedicated hosting.
Selene may later upgrade companies based on usage, risk, volume, privacy, or enterprise requirements.
B2B sits outside individual company tenants.
Customer devices are thin access surfaces.
Core truth lives in Selene cloud.
```

Document 77 references this only.

Future owner:

```text
Selene SaaS Tenancy, Cloud Hosting, Data Isolation, Device Access + Deployment Engine
```

That future engine owns:

```text
shared vs dedicated hosting
tenant isolation
device access
wake word
button trigger
voice ID
lost-device recovery
usage monitoring
free trial
cost-to-serve pricing
SaaS billing
desktop/iPhone/app continuity
```

Document 77 only needs to know:

```text
Customer can access Selene from many devices.
Device holds minimal data.
Cloud holds durable truth.
```

No data center in the cart. We showed restraint. Someone notify the archives.

---

## 5. Device Access Model

Customers may access Selene through:

```text
desktop
mobile browser
iPhone app
Android app later
tablet
voice device
kiosk
POS-linked customer screen
recipient secure link
future wearable / car / device surfaces
```

Supported interaction modes:

```text
typed chat
voice chat
scrolling
tap/click
camera/photo input
button trigger
wake word where supported
voice ID for quick recognition
biometric/passkey confirmation where available
secret passcode fallback
```

Device rule:

```text
Device holds minimum local data.
Cloud holds durable truth.
```

Device may hold:

```text
session token
notification token
cached UI
temporary product cards
secure local auth reference
limited offline state if allowed
```

Cloud holds:

```text
customer profile
customer memory
company relationships
provider relationships
orders
addresses
payment token references
preferences
audit
device registry
```

If a device is lost or replaced:

```text
Customer re-authenticates.
Old device can be revoked.
Selene restores cloud memory, orders, addresses, store relationships, and preferences.
```

Voice ID is convenience.

Payment, cancellation, refunds, address changes, and other protected actions require stronger confirmation.

```text
Voice ID = recognition.
Passkey / biometric / passcode = protected action confirmation.
```

No sofa purchases because someone sounded “Harry-ish.” We are not running a sitcom.

---

## 6. Commerce Surfaces

Selene commerce has three major surfaces.

```text
1. Company E-Commerce Store
2. Customer Personal Selene Commerce Home
3. Selene B2B Platform behind the scenes
```

### 6.1 Company E-Commerce Store

Each company has its own Selene-powered store.

The company store may display:

```text
company-owned products
company-owned services
approved B2B products/services
auto-added B2B products/services under company policy
competitor/alternative products only if company allows
events, bookings, and promotions
customer account / credit terms where offered
```

Company store context is protected.

Inside the company store, Selene follows that company’s store rules.

### 6.2 Customer Personal Selene Commerce Home

Each customer has a personal Selene commerce space.

The customer can search broadly for:

```text
products
services
groceries
restaurants
takeaway
appointments
fashion
hardware
electronics
professional services
local providers
customer-visible B2B offers
buying list items
gifts
```

No one company controls this personal space.

### 6.3 Selene B2B Platform

Selene Inc owns B2B.

B2B supplies eligible offers to:

```text
company stores
personal Selene commerce
customer search
business B2B search
auto-add programs
```

Document 77 only displays B2B offers after Document 78 confirms eligibility.

---

## 7. Company Store Context vs Personal Selene Context

### 7.1 Company Store Context

When the customer is clearly inside a company store, Selene respects that store.

Inside Company A store, Selene may show:

```text
Company A products
Company A services
Company A-approved B2B offers
Company A auto-added B2B offers
competitor products only if Company A allows competitor inclusion
```

### 7.2 Personal Selene Context

When the customer searches generally through personal Selene, Selene can search broadly across:

```text
stores the customer belongs to
eligible B2B offers
local providers
original providers
providers with prior customer relationship
personal recommendations
delivery-capable offers
```

### 7.3 Customer-facing tone

Selene must not narrate backend ranking logic.

Bad:

```text
“I searched your linked stores first, then B2B, then wider providers.”
```

Good:

```text
“I found a few good options.”
“This one suits your style best.”
“This one has faster delivery.”
“This one is cheaper, but the warranty is weaker.”
```

If the customer asks why, Selene can explain simply:

```text
“I picked these based on your size, price range, delivery location, and past purchases.”
```

Customers want results, not a tour of the ranking basement.

---

## 8. One Customer, Many Company Relationships

A customer has one Selene identity.

That customer may be linked to many businesses and people.

Example:

```text
Tom has one Selene profile.

Tom may be linked to:
- Company A Shoes
- Company B Shoes
- Bakery
- Grocery Store
- Hair Salon
- Restaurant
- Electronics Supplier
- His employer
- A friend
- His mum
- A colleague
```

Each company receives only its own Company-Customer Relationship Record.

Company A can see:

```text
Tom’s Company A orders
Tom’s Company A account terms
Tom’s Company A returns/warranty
Tom’s Company A marketing permission
Tom’s Company A delivery addresses
Tom’s Company A payment/account history
```

Company A cannot see:

```text
Tom’s grocery purchases
Tom’s salon bookings
Tom’s restaurant orders
Tom’s private Selene searches
Tom’s purchases from another shoe shop
Tom’s full personal Selene activity
Tom’s private relationships
```

Rule:

```text
A company owns its relationship with the customer.
It does not own the customer.
```

A wild concept. Terrifying to marketers. Necessary for trust.

---

## 9. Customer Full Selene Access

When a person joins Selene through a company link, employee link, friend link, family link, recipient link, or personal registration, they receive full Selene customer access.

They can:

```text
shop in company stores
search broadly in personal Selene commerce
join more company stores
receive marketing from companies they allow
use payment methods
hold store accounts/credit terms if offered
create buying lists
book restaurants/services
send gifts
track orders
cancel orders
return products
claim warranty
refer others
earn rewards if enabled
use wider Selene search
use future Selene personal life/project/wellbeing tools
```

E-Commerce is one surface of the larger Selene relationship.

Future engines will cover:

```text
Customer Intelligence + Relationship
Personal Selene Life OS
Emotional Wellbeing
Personal Projects
Personal Finance
Personal Documents
Referral / Rewards / Loyalty
```

Document 77 must not imply that Selene is only shopping. Selene is the assistant relationship. Shopping is just where humans keep giving it money.

---

## 10. First Verified Introducer — E-Commerce Responsibility

E-Commerce must capture intro events, but it does not calculate final rewards.

Rule:

```text
First verified introducer wins.
Second and later personal introducers do not receive first-intro reward for the same person.
```

E-Commerce records:

```text
customer joined through link
link source
introducer type
verified onboarding completion
duplicate introducer attempt
fraud/self-referral signal
```

Future owner:

```text
Selene Referral, Rewards, Loyalty + Attribution Engine
```

Document 77 sends events.

It does not own reward math.

No one gets to claim a customer because they sent a link first and the person ignored it. The winning event is verified onboarding, not digital confetti.

---

## 11. E-Commerce Relationship to B2B

Document 77 does **not** define B2B mechanics.

It only knows:

```text
some products/services are company-owned
some are B2B-sourced
B2B determines whether an offer is customer-visible
B2B determines Original Provider / Channel Store / commission / reserve / settlement rules
E-Commerce displays eligible offers and sends order context to B2B
```

When E-Commerce detects a B2B-sourced item, it requests B2B display data:

```text
customer-visible retail price
product/service display content
delivery estimate
return summary
warranty/guarantee summary
provider disclosure if required
eligibility for this company store or personal Selene search
```

E-Commerce must not expose:

```text
profit share
provider payout
B2B platform fee
company commission
deposit/reserve amount
settlement hold mechanics
```

unless required by law/policy.

Customer view is shopping.

Business view is Document 78.

Do not make the customer read the marketplace’s tax diary.

---

## 12. Original Provider Routing — E-Commerce View

E-Commerce must support Original Provider routing, but Document 78 owns the full rules.

From the customer’s perspective:

```text
Customer may buy from a company store.
If item is B2B-sourced, the Original Provider handles product/service responsibility.
Customer does not need a jarring visible store switch.
Selene routes product questions, technical questions, warranty, returns, and future related-product questions to the Original Provider.
```

Example:

```text
Hair Salon displays Wine Store wine.
Customer buys wine through Hair Salon.
Customer later asks about wine pairing.
Selene routes wine-specific question to Wine Store knowledge/provider Selene.
Hair Salon remains channel/introducer where attribution applies.
```

E-Commerce handles the smooth experience.

B2B handles Original Provider responsibility and commission logic.

This is the whole point: customer convenience without dumping wine questions on someone holding scissors.

---

## 13. Embedded Provider Expansion

When a customer is in a company store and wants more details or related products from the Original Provider, Selene should avoid unnecessary visible store switching.

Correct model:

```text
Customer stays in the current shopping experience.
Selene can show Original Provider’s relevant products/specs/variants inside the Channel Store context if allowed.
Original Provider remains responsible.
Channel Store attribution remains protected where applicable.
```

Example:

```text
Company A displays Company C rolls of wire.
Customer asks about size, thickness, and types.
Selene can show Company C wire variants inside Company A experience.
Customer does not need to know a store switch occurred unless disclosure is required.
```

Visible transfer/disclosure occurs when:

```text
law requires provider/seller disclosure
customer asks who supplies it
warranty/support requires provider identity
company policy requires provider branding
customer wants to follow/join the Original Provider store
professional service disclosure is required
```

No “You are now entering Wire Kingdom” unless the law, customer, or policy demands it.

---

## 14. Provider Support Routing — Customer Experience

If a customer asks a product/service-specific question, E-Commerce routes it to the correct knowledge source.

Flow:

```text
Customer asks question.
Selene checks approved product/service data.
If answer exists, Selene answers.
If answer does not exist, Selene routes question to Original Provider / provider Selene.
Provider answers.
Selene captures provider-approved answer where policy allows.
Future similar questions may be answered automatically.
Audit records source.
```

Examples:

```text
“What thickness wire do I need?”
“Is this wine good with steak?”
“Does this screw work outdoors?”
“What warranty applies?”
“Can this accountant handle my tax country?”
```

Selene must not invent specialized product or professional answers.

GPT-5.5 can explain.

Original Provider or deterministic records supply truth.

A language model with confidence is still not a licensed electrician. Annoying, but true.

---

## 15. Customer Search Interface

Selene must support natural shopping through:

```text
typing
voice
scrolling
click/tap
photo/image search
bad spelling
bad speech recognition
partial phrases
previous order references
recipient references
occasion-based search
```

Mode rule:

```text
If customer types, Selene replies in text.
If customer speaks, Selene replies by voice, with visual display where useful.
If customer scrolls, Selene supports visual browsing and context-aware help.
```

Customer can upload or take a photo and ask Selene to find similar products.

Examples:

```text
“Find shoes like this.”
“Find this dress but cheaper.”
“Find the same color.”
“Find this product in a smaller size.”
“Find this part.”
```

Selene can ask clarifying questions only when needed:

```text
“Do you want the same color, same style, or cheaper alternatives?”
```

Good search asks just enough.

Bad search throws 600 products at the customer’s face and calls it choice.

---

## 16. Conversation-to-Action Guardrail

Selene must not rely on fixed phrase matching.

Natural language understanding belongs to:

```text
GPT-5.5
PH1.X live context
PH1.M memory
relevant customer/company/store/order/provider memory
```

Execution authority belongs to deterministic engines.

### 16.1 Conversation Flow

```text
1. Detect channel:
   typed chat, voice, app scroll, desktop, POS, recipient link

2. Identify speaker:
   customer, recipient, company user, employee, admin, provider, support user

3. Resolve live context using PH1.X:
   “that one”
   “send it to mum”
   “same as before”
   “the blue one”
   “continue”
   “cancel those shoes”

4. Resolve durable context using memory:
   mum identity
   usual card
   usual address
   favorite size
   store relationship
   original provider
   prior order
   buying habits

5. Interpret intent using GPT-5.5:
   buy, cancel, return, search, add to list, book, send gift, change address, ask support

6. Verify facts through deterministic engines:
   product identity
   variant
   price
   stock
   payment permission
   account/credit terms
   address
   delivery
   tax/duties
   return eligibility
   warranty/guarantee
   provider responsibility
   company privacy
   authority
   audit

7. Ask clarification only if needed.

8. Execute, prepare draft, or route.

9. Reply naturally in the user’s active mode.

10. Audit important actions.
```

### 16.2 GPT-5.5 may

```text
interpret natural speech/text
repair bad spelling
repair unclear speech
summarize
compare
recommend
translate
draft messages
speak naturally
```

### 16.3 GPT-5.5 must not

```text
approve purchases
execute payments
issue refunds
change addresses without authority
alter records
invent stock
invent product facts
invent prices
invent delivery dates
invent warranty coverage
override policy
bypass verification
replace audit evidence
```

GPT-5.5 makes Selene human-like.

Deterministic engines make Selene safe.

Both are needed. One without the other is either a brick or a charming liability.

---

## 17. Product Discovery and Search Intelligence

Search can use:

```text
keywords
voice intent
photo similarity
product attributes
style matching
previous purchases
customer sizes
customer colors
customer budget
local availability
delivery speed
store relationship
B2B eligibility
provider quality
warranty/return quality
occasion
recipient
buying list
repeat pattern
```

Customer can ask:

```text
“Find lower cost.”
“Show premium options.”
“Only blue.”
“Only large.”
“Something like this but cheaper.”
“Something faster delivery.”
“Something with better warranty.”
“Something my mum would like.”
```

Selene should ask useful narrowing questions only when needed.

Example:

```text
Customer: “Find shoes.”
Selene: “Casual, work, or going out?”
```

But if context is obvious:

```text
Customer just bought a formal dress.
Selene should infer formal/dress shoes first.
```

Less interrogation. More assistance. Selene is not customs control for sandals.

---

## 18. Product Display, Photos, Videos + Product Story

E-Commerce displays Product Engine content.

Product Engine provides:

```text
title
description
category
variants
photos
videos
product story
specs
ingredients/materials
warranty summary
return summary
care instructions
compliance info
product passport link where applicable
```

E-Commerce renders:

```text
product cards
image gallery
video gallery
comparison view
style match explanation
delivery badge
return badge
warranty badge
availability badge
provider/original store disclosure where required
```

Product facts come from Product Engine.

GPT-5.5 may explain them beautifully.

No making up “artisan quantum cotton” because the photo has soft lighting.

---

## 19. Variant Selection

Customer can choose variants naturally.

Selene asks:

```text
“Do you want large or small?”
“Red or blue?”
“Your usual size is 8. Should I use that?”
“Blue works better with that dress.”
“Do you want the 500ml or 1L bottle?”
```

Variant data comes from Product Engine.

Availability comes from Inventory.

Price comes from Pricing.

Order Management locks selected variant.

E-Commerce owns the conversation.

The customer should feel helped, not dropdowned to death.

---

## 20. Company E-Commerce Store Layer

Each company store has:

```text
company brand
company products/services
company approved B2B products/services
store policies
customer accounts
credit terms
loyalty
marketing
events
customer support
delivery/returns rules
warranty rules
```

Company store can be accessed by:

```text
customer link
company website
Selene app
voice command
customer personal Selene home
POS relationship
marketing message
```

Customer can say:

```text
“Open Tom’s Bakery.”
“Order from Company A.”
“Book with Bella Salon.”
```

Selene routes to the correct store context.

---

## 21. Personal Selene Commerce Home

Each customer has a personal Selene commerce space.

This can show:

```text
stores the customer belongs to
recommended products
eligible B2B products
local services
groceries
fashion
hardware
electronics
restaurants
restaurant bookings/takeaway
buying lists
repeat purchases
gift ideas
```

Personal Selene Home is customer-owned context.

No one company controls it.

This is where customers can search broadly.

This is where Selene becomes the buying brain of daily life.

Yes, a little terrifying. Most useful things are, at first.

---

## 22. Unified Customer Buying Lists

Buying lists are item-based, not store-based.

Customer sees one list.

Examples:

```text
Weekly groceries
Monthly household items
Work supplies
Gift list
Restaurant/takeaway list
Project list
Office list
Personal care list
```

Customer says:

```text
“Add milk to groceries.”
“Add toothpaste to end-of-week groceries.”
“Add my favorite cake.”
“Add bread for Friday.”
“Buy my usual weekly list.”
```

Selene knows behind the scenes:

```text
favorite cake = Lin Cake Shop
favorite bread = local bakery
milk = grocery store
carrot cake = another bakery
shampoo = salon / grocery / B2B provider
wire = Supplier 44
```

At order time, Selene resolves providers using:

```text
preferred provider
availability
delivery time
price
customer history
store relationship
provider score
return/warranty rules
B2B eligibility
```

Customer does not need seven lists because seven shops exist.

The human has one life. Selene can handle the supplier mess.

Future deeper owners:

```text
Customer Buying List + Predictive Replenishment Engine
Memory Pattern, Habit, Demand Prediction + Replenishment Intelligence Engine
Order Management
```

---

## 23. Memory Pattern + Habit Handoff

Document 77 uses pattern signals but does not own the future pattern engine.

Future owner:

```text
Selene Memory Pattern, Habit, Demand Prediction + Replenishment Intelligence Engine
```

That future engine detects:

```text
bread every 2 days
shampoo every month
customer prefers blue
customer buys more before holidays
customer usually reorders at 20% remaining
business customer seasonal demand
retail demand pattern
household reorder rhythm
provider recommendation timing
promotion timing
```

E-Commerce uses these patterns for:

```text
buying lists
reorder reminders
recommendations
holiday preparation
customer shopping suggestions
company customer marketing suggestions
B2B product recommendations
```

PH1.M stores governed memory.

Memory Pattern Engine detects habits.

PH1.M is the memory vault. Pattern Engine is the analyst with a clipboard and unsettling accuracy.

---

## 24. Predictive Writing + Input Correction Handoff

Document 77 uses input repair but does not own the future predictive input engine.

Future owner:

```text
Selene Predictive Writing, Input Correction + Assistive Typing Engine
```

It owns:

```text
word prediction
sentence completion
typing correction
spelling correction
grammar correction
phonetic repair
multi-language input correction
commerce search suggestions
business form completion
message drafting suggestions
```

Examples:

```text
Customer types “shooes blue dres”
Selene resolves likely intent as “shoes for blue dress.”

Customer types “add tooth past to grocries”
Selene resolves “add toothpaste to groceries.”
```

No fixed phrase patching.

No keyword caveman.

Selene handles messy human input like a real assistant, not a vending machine with opinions.

---

## 25. Company E-Commerce Setup Advisory Handoff

When a company starts E-Commerce, Selene must help configure store behavior.

Document 77 references this, but future onboarding/store setup owns the deeper process.

Future owner:

```text
Selene Commerce Setup Advisory + Store Configuration Assistant
```

Selene should ask:

```text
“How do you want to run your e-commerce store?”
```

Options include:

```text
Closed Store Mode
Curated B2B Expansion
Autonomous B2B Expansion
Competitor Inclusion Mode
Enterprise Private Ecosystem Mode
Customer account/credit terms
Selene settlement trust / 7-day hold
Marketing permissions
Events/invitations
B2B opt-in for own products/services
Delivery zones
Warranty/return policy
Provider reserve/deposit settings where applicable
```

Selene should explain tradeoffs:

```text
“Closed mode protects your catalog but limits extra revenue.”
“Curated B2B lets you approve add-ons.”
“Autonomous B2B can expand your range automatically under your rules.”
“Competitor inclusion can earn you money even when customers choose alternatives.”
“Selene settlement trust can increase customer confidence but changes cash timing.”
```

Most people will not know Selene’s options. Humans continue to be under-documented.

---

## 26. Payment Permission + Checkout

Customers may have different payment relationships.

Payment methods:

```text
credit card
debit card
wallet
bank account
gift card
store credit
Selene reward credit
company account credit
trade/customer account
buy-now-pay-later where allowed
cash-on-delivery where configured
future Selene virtual money
```

Payment relationships may be:

```text
global Selene payment token
company-store-specific payment method
company account/credit terms
customer wallet/reward balance
business account terms
recipient gift flow
provider direct purchase payment
```

Document 77 owns:

```text
customer payment permission request
checkout interaction
step-up verification prompt
customer-facing confirmation
order intent handoff
```

Document 77 does not own:

```text
payment authorization
capture
settlement release
provider payout
refund execution
customer credit underwriting
wallet balances
ledger posting
```

Future owner for credit/wallet:

```text
Selene Customer Credit, Wallet, Store Account, Installment + Virtual Settlement Engine
```

---

## 27. Selene-Controlled Settlement Trust — E-Commerce View

Selene should consider default settlement trust across Selene commerce.

Customer-facing promise:

```text
Buy through Selene.
Selene protects the order through configured cancellation, return, refund, delivery, and warranty rules.
```

Document 77 only explains this to the customer.

Future owner:

```text
Selene Platform Settlement Trust + Customer Protection Engine
```

Recommended default:

```text
Customer pays through Selene.
Selene holds or controls settlement.
Goods/service delivered.
Customer has defined cancellation/return/hold period.
Funds release only when conditions clear.
```

Exceptions:

```text
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

Why it matters:

```text
customer trust
consistent refund/cancellation experience
higher purchase confidence
protection from poor individual company refund practices
clean accounting/audit
```

If every company has random refund rules, customer trust becomes soup.

Selene should be the adult holding the money bowl.

---

## 28. Step-Up Verification

Protected actions require customer verification.

Protected actions include:

```text
buy now
approve payment
cancel paid order
request refund
return item
change delivery address after order
add new recipient
change payment method
approve high-value purchase
approve recurring purchase
claim warranty with refund/replacement value
share recipient access
change privacy/gift settings
```

Preferred verification:

```text
passkey
public-key credential flow
device biometric through platform authenticator
secure device unlock
```

Fallback:

```text
secret passcode
PIN
secure recovery
manual support verification where policy allows
```

Customer-facing example:

```text
“That’s $129 delivered to your mum’s house. Confirm with Face ID or your secret passcode.”
```

No biometric/passkey available:

```text
“Use your secret passcode to confirm.”
```

Simple. Secure. Not a twelve-screen checkout assault.

---

## 29. One-Command Checkout

Customer can say:

```text
“Buy that one.”
“Send it to mum.”
“Use my usual card.”
“Deliver it to the office.”
“Add it to this week’s groceries.”
```

Selene creates an order draft:

```text
customer_id
company_store_id or personal commerce context
product/service
variant
quantity
price_ref
delivery destination
recipient
privacy mode
payment permission
verification requirement
B2B/provider ref if applicable
return/warranty summary
audit_ref
```

Flow:

```text
customer intent
→ order draft
→ price/delivery/return/warranty summary
→ step-up verification if required
→ payment authorization handoff
→ order submission
→ B2B settlement handoff if applicable
→ confirmation
→ audit
```

Selene should not force traditional checkout when customer already has verified preferences and permissions.

But Selene must never skip security just to feel magical.

Magic without controls is fraud with better lighting.

---

## 30. Address Capture + Delivery Destination

Customer should not have to spell painful addresses.

Selene can capture address from:

```text
photo of document
photo of package label
scan of letter
map pin
contact card
voice
copy/paste
saved address
recipient profile
delivery company validation
company account address
```

Delivery destinations:

```text
home
office
mum’s house
friend’s house
hotel
work site
project site
customer site
pickup point
locker
store pickup
restaurant/table/service location
```

Selene can say:

```text
“I found the address from the photo. Is this correct?”
```

Delivery record includes:

```text
payer
buyer
recipient
delivery contact
address
safe-place instruction
quiet/gift mode
price visibility
sender visibility
return permission
recipient notification
audit_ref
```

---

## 31. Recipient Address Request via Broadcast / Delivery

If customer says:

```text
“Send it to mum.”
```

and mum’s address is missing, Selene should not force the customer to call mum.

Flow:

```text
Selene detects missing recipient address.
Selene asks buyer permission to contact recipient.
Selene sends secure request through Broadcast / Delivery Engine.
Recipient opens secure link or Selene app.
Recipient provides address by typing, voice, photo, map pin, contact card, or saved profile.
Selene validates address.
Selene attaches address to order subject to privacy and permission rules.
```

Applies to:

```text
mum
colleague
friend
wife
husband
client
employee
business site contact
recipient
gift recipient
```

Notification rule:

```text
Do not put private order details in the notification.
Send a secure prompt/link.
Recipient authenticates or verifies as needed.
```

No “Your daughter bought you secret shoes, please send address” appearing on a locked screen. That is gift-mode murder.

---

## 32. Recipient, Gift + Relationship Layer

Selene must support rich relationship roles.

Roles include:

```text
payer
buyer
customer
recipient
gift recipient
return requester
delivery contact
support contact
account holder
family member
friend
colleague
wife
husband
boyfriend
girlfriend
father
mother
father-in-law
mother-in-law
cousin
employee
manager
business approver
```

Customer can say:

```text
“Buy that one and send it to my mum’s house.”
“Keep it quiet.”
“Don’t show the price.”
“Let mum track it.”
“Let mum return it, but refund me.”
```

Recipient access levels:

```text
NoAccess
DeliveryOnly
DeliveryAndReturn
SupportOnly
GiftRecipientAccess
BuyerApprovalRequired
FullRecipientSupport
```

Privacy modes:

```text
normal
gift mode
quiet mode
hide price
hide sender until date/delivery
hide product until delivery
recipient tracking only
recipient return requires buyer approval
recipient may return but refund goes to original payer
```

If recipient says:

```text
“I want to return this.”
```

Selene checks:

```text
recipient permission
buyer/payer rules
return eligibility
refund destination
privacy mode
courier rules
```

Selene may say:

```text
“Return is eligible. I’ll ask the buyer because they paid.”
```

If buyer blocked recipient returns:

```text
“This item is not available for recipient return. I can ask the buyer if you want.”
```

Polite. Human. Slightly trapped in family logistics, as all commerce eventually is.

---

## 33. Order Tracking

Customer can ask:

```text
“Where is my dress?”
“Where is mum’s gift?”
“Did the shoes ship?”
“When is the delivery?”
“Has the return been picked up?”
```

Selene resolves:

```text
which order
which store
which recipient
which delivery
which original provider if B2B
which privacy mode
what can be disclosed
```

Displayed states:

```text
OrderDraft
OrderPlaced
PaymentAuthorized
Preparing
ProviderPending
Picking
Packed
Dispatched
Shipped
OutForDelivery
Delivered
DeliveryIssue
CancellationRequested
Cancelled
ReturnRequested
ReturnInTransit
RefundPending
Refunded
WarrantyClaimOpen
Closed
```

Selene says:

```text
“The dress is out for delivery today.”
“The gift is on track for Friday.”
“The return pickup is booked for tomorrow afternoon.”
```

No customer should need to search email for tracking links like a raccoon rummaging through digital bins.

---

## 34. Cancellation Interface

Customer can cancel conversationally:

```text
“Cancel those shoes.”
“Cancel the dress I ordered yesterday.”
“Stop the order going to mum.”
```

Selene checks:

```text
order identity
payer/buyer identity
step-up verification requirement
order status
dispatch status
shipment status
provider/company fulfillment status
B2B provider rules
payment capture status
refund eligibility
recipient/privacy impact
```

Cancellation outcomes:

```text
cancel immediately
refund immediately
cancel pending provider acknowledgement
stop dispatch
try courier intercept
convert to return
not cancellable
requires review
```

Examples:

```text
“They have not shipped. I cancelled and refunded you.”
“They are being packed. I’ll try to stop dispatch.”
“They have shipped. I’ll try to intercept delivery.”
“They were delivered yesterday. This is return eligible.”
```

E-Commerce owns the customer-facing request.

Order / Payment / Dispatch / Returns execute backend.

Customer should not need to learn five engines just to cancel shoes. That would be cruel, even for software.

---

## 35. Return Request Interface

Detailed returns live in Document 83.

E-Commerce owns customer-facing return initiation.

Customer says:

```text
“Return this.”
“Mum wants to return it.”
“This arrived damaged.”
“It doesn’t fit.”
```

Selene collects:

```text
order item
reason
condition
photos if needed
recipient/payer relationship
pickup/drop-off preference
refund destination
provider/company/B2B status
policy eligibility
```

Customer may choose:

```text
courier pickup
drop-off point
safe-place pickup
store return
provider return
postal return
```

Refund timing is risk-based, but Document 83 owns the full logic.

E-Commerce may explain:

```text
“I can book pickup tomorrow. Refund will go back to your original payment method once the return is scanned.”
```

Or:

```text
“This item needs inspection before refund. I’ll arrange pickup and update you after inspection.”
```

Customer-facing simplicity. Backend complexity locked in the warehouse where it can think about what it did.

---

## 36. Dispatch Handoff

Dispatch is Document 82.

E-Commerce sends dispatch intent after order creation.

Dispatch owns:

```text
pick list
packing
correct product verification
photo/scan proof
box selection
shipping label
courier selection
courier booking
pickup scheduling
warehouse handoff
dispatch proof
customer dispatch notification
delivery handoff
```

E-Commerce displays:

```text
Preparing
Picking
Packed
Dispatched
Shipped
OutForDelivery
Delivered
```

Selene may say:

```text
“Your order is packed and waiting for courier pickup.”
```

E-Commerce does not own box selection. That is how a storefront becomes a warehouse goblin.

---

## 37. Delivery / Courier Interface

All delivery companies should ideally connect to Selene through API or structured integration.

E-Commerce needs customer-facing delivery status.

Dispatch/Logistics owns the detailed courier system.

Selene needs:

```text
book shipment
book pickup
track delivery
track return
proof of delivery
failed delivery reason
safe-place delivery
pickup windows
recipient contact
driver/courier status
international tracking
delivery dispute evidence
```

E-Commerce says:

```text
“Courier pickup is booked for tomorrow between 2 and 5.”
“Your delivery is out for delivery.”
“The courier marked safe-place delivery. I have the photo proof.”
```

No human should call couriers unless something is truly weird. Couriers are already professionally weird.

---

## 38. International Duties, Taxes + Landed Cost Display

For international products, E-Commerce must show customer-facing estimates from B2B/Tax/Compliance.

It may display:

```text
shipping cost
customs duties
import tax
VAT/GST
brokerage fees
delivery timeframe
restricted goods risk
return difficulty
who pays duties
whether duties are prepaid
estimated total landed cost
```

Customer-facing requirement:

```text
No surprise import cost where Selene can reasonably estimate it.
```

Selene says:

```text
“Estimated total delivered cost is $184, including estimated duties and taxes.”
```

Or:

```text
“Duties may be payable on delivery. I can show local alternatives if you prefer.”
```

Tax owns final tax.

E-Commerce owns explanation.

No one likes a $48 customs surprise jumping out of a delivery van.

---

## 39. Restaurant / Service Booking Interface

E-Commerce supports commerce beyond products.

Customer can:

```text
book restaurant
choose dishes ahead
order takeaway
book salon appointment
book service provider
prepay deposit
buy gift card
book event/workshop
```

Examples:

```text
“Book Mario’s for Friday.”
“Order the pasta ahead.”
“What should I eat there?”
“Book my usual haircut.”
“Order takeaway from the restaurant.”
```

E-Commerce owns the customer interface.

Order / POS / Restaurant / Payment engines own deeper execution.

Selene should not cram restaurant operations into e-commerce like a lasagna in a mailbox.

---

## 40. Professional Services Display

E-Commerce may display professional services only after B2B/Product/Compliance confirms eligibility.

Examples:

```text
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

E-Commerce must request professional eligibility from the correct backend.

It must not invent:

```text
licence
registration
professional indemnity insurance
certificate of currency
jurisdiction eligibility
professional scope
disciplinary status
```

If customer asks:

```text
“Find me an accountant.”
```

Selene should search based on:

```text
customer jurisdiction
service type
professional qualification
insurance/registration status
availability
price
reviews/score where allowed
conflict/check requirements where applicable
```

Customer-facing phrase:

```text
“I found eligible accountants for your location and service type.”
```

Not:

```text
“This person is probably legal everywhere.”
```

Professional services are not shampoo. They bite back.

---

## 41. Customer Memory, PH1.X, and PH1.M

Document 77 uses memory but does not own the full memory engine.

### PH1.X — Live Context

Handles:

```text
“that one”
“send it to mum”
“same as last time”
“the blue one”
“continue”
```

### PH1.M — Durable Memory

Handles:

```text
Tom likes black shoes.
Tom buys bread every Friday.
Tom’s mum is Mary.
Tom prefers office delivery.
Tom uses quiet gift mode.
```

### Future Customer Engine

Owns:

```text
multi-store relationships
family/friend relationships
marketing permissions
company-specific purchase history
referral relationships
credit account history
privacy settings
provider-customer relationships
```

Memory layers:

```text
Session memory = short-term context
Preference memory = editable likes/dislikes/habits
Transactional memory = orders/payments/returns/warranty
Company relationship memory = per-company isolated
Provider relationship memory = original provider/product relationship
Sensitive vault = protected private data
Derived memory = predictions with confidence
```

Rule:

```text
Selene remembers what is useful, permitted, justified, and reviewable.
Selene does not hoard every word forever like a sentimental surveillance dragon.
```

---

## 42. Customer Privacy + Data Walls

Company A can see:

```text
Company A purchases
Company A account status
Company A marketing permission
Company A customer support events
Company A returns/warranty
Company A B2B channel sales
```

Company A cannot see:

```text
other store purchases
personal Selene search history
competitor activity
family/friend links
private preferences not shared
reward network details unless relevant/allowed
provider customer data not related to Company A attribution
```

Original Provider can see:

```text
customer’s purchases from that provider
fulfillment/support/warranty data
delivery and return data needed to serve the product/service
marketing permission if customer allows
```

Original Provider cannot see:

```text
customer’s full personal Selene profile
other store purchases
private family/friend relationships
channel store private customer data unrelated to provider attribution
```

Privacy principle:

```text
Selene uses memory to help the customer.
Selene does not expose memory to companies/providers unless permitted.
```

Providers get enough data to do the job, not enough to become creepy.

---

## 43. Events / Invitations Handoff

E-Commerce may display events and allow customers to accept invitations, buy tickets, or RSVP.

Future owner:

```text
Selene Events, Invitations, Ticketing, RSVP + Relationship Experience Engine
```

That future engine owns:

```text
event creation
customer segment selection
invitation design
Broadcast/Delivery send-out
paid ticketing
free RSVP
company-paid event allocation
reminders
check-in
refund/cancellation
post-event follow-up
```

Example:

```text
Company invites 1,000 customers to Grand Prix in LA.
Selene designs invitation.
Selene sends through Broadcast/Delivery.
Selene handles RSVP/tickets/payment.
```

Document 77 may display event offers and capture customer response.

It does not own the event machine. We are not hiding a stadium in checkout.

---

## 44. Accessibility + Multi-Surface Experience

Selene E-Commerce must work across:

```text
desktop
mobile web
iPhone app
Android app later
tablet
voice
chat
kiosk
recipient links
customer portal
```

Accessibility requirements include:

```text
screen-reader support
keyboard navigation
clear focus states
large touch targets
captions/transcripts for videos where required
alt text for images
plain-language errors
color contrast
accessible authentication fallback
clear return/cancellation flows
```

Accessible commerce is better commerce.

Also fewer people yelling at checkout screens, which is a public good.

---

## 45. Accounting / Payment / Tax / Cashflow Handoff

E-Commerce must connect to financial engines, but must not own them.

Ownership split:

```text
E-Commerce = customer permission and checkout experience
Payment/Settlement = authorization, capture, refund execution, wallet, store account settlement
B2B Platform = provider payout, channel commission, platform fee, reserve/deposit
Accounting = ledger, revenue, liabilities, fees, reserves, taxes
BankRec = bank/payment proof
Tax = GST/VAT/duties/international tax handling
Cashflow = liquidity impact
```

E-Commerce sends handoff events:

```text
order intent created
payment permission granted
B2B item included
return requested
refund requested
customer cancelled
recipient delivery created
settlement trust applies
```

Customer sees:

```text
“Done. Delivery Friday.”
```

Backend gets:

```text
payment authorized
order created
inventory reserved
B2B settlement record if applicable
dispatch prepared
provider payout hold if applicable
channel commission if applicable
tax classified
accounting event prepared
audit stored
```

Customer gets magic.

Back office gets evidence.

Everyone receives the illusion they prefer.

---

## 46. Automation and Exception-Only Review

Selene auto-handles:

```text
customer onboarding link flow
store relationship creation
search intent repair
product discovery
photo search interpretation
style matching
variant suggestion
order draft creation
address capture request
recipient secure address request
buying list updates
reorder reminders
routine cancellation eligibility checks
routine return initiation
order tracking responses
warranty request intake
B2B display eligibility request
provider support routing request
reward attribution event creation
```

Selene escalates:

```text
protected payment confirmation
high-value purchase
new recipient with payment impact
address change after order
refund to non-original payer
high-risk return
warranty/orphan provider claim
international duty/tax uncertainty
regulated professional service
missing provider insurance/licence where required
customer privacy conflict
company data boundary conflict
suspicious account/payment behavior
B2B reserve/deposit dispute
provider/customer attribution dispute
```

Rule:

```text
Routine shopping = Selene handles.
Protected money/action = verification.
Complex/risky/legal = route.
Everything important = audit.
```

No approval circus. No free-for-all.

Selene chooses the difficult sensible path, because apparently someone has to.

---

## 47. PH1.D / GPT-5.5 Role

GPT-5.5 is heavily used in E-Commerce.

GPT-5.5 may help:

```text
understand messy search
repair bad spelling
interpret voice errors
compare products
explain differences
suggest matching items
summarize videos/descriptions
explain order status
explain return options
explain warranty terms
draft customer messages
translate customer/provider messages
personalize tone
```

GPT-5.5 must not:

```text
invent product facts
invent stock availability
invent delivery dates
invent warranty coverage
invent refund eligibility
approve payment
bypass customer verification
change delivery address without authorization
expose private customer memory
override return policy
make regulated claims
invent provider insurance/licence status
invent professional eligibility
```

Correct pattern:

```text
GPT-5.5 speaks naturally.
Deterministic engines verify truth.
Customer verifies protected actions.
Audit records everything.
```

Selene sounds human.

Selene does not hallucinate the customer into buying shoes that are out of stock in a size that never existed.

---

## 48. E-Commerce State Machines

### 48.1 Customer Shopping Session State

```text
Started
Browsing
Conversing
RecommendationRequested
ProductViewed
ProductCompared
ItemSelected
OrderIntentCreated
VerificationRequired
OrderSubmitted
Abandoned
Closed
```

### 48.2 Store Relationship State

```text
Invited
Linked
Active
MarketingPermitted
MarketingRestricted
AccountActive
AccountHold
Dormant
Revoked
Closed
```

### 48.3 Search Context State

```text
CompanyStoreContext
PersonalSeleneContext
RecipientContext
GiftContext
POSContext
RestaurantBookingContext
ServiceBookingContext
ContextUnclear
ContextResolved
```

### 48.4 First Introducer Capture State

```text
NoIntroducer
LinkSent
OnboardingPending
FirstVerifiedIntroducerEventCaptured
DuplicateIntroducerIgnored
FraudReview
Closed
```

### 48.5 Order Intent State

```text
Draft
NeedsProductSelection
NeedsVariant
NeedsAddress
NeedsRecipient
NeedsPaymentPermission
NeedsStepUpVerification
ReadyForConfirmation
Confirmed
SubmittedToOrder
Rejected
Expired
Closed
```

### 48.6 B2B Awareness State

```text
NotB2B
B2BDisplayEligibilityRequested
B2BEligibleForDisplay
B2BOrderContextCreated
B2BSettlementHandoffRequired
ProviderSupportRoutingRequired
Closed
```

### 48.7 Customer Verification State

```text
NotRequired
Required
PasskeyRequested
BiometricRequested
PasscodeRequested
Verified
Failed
Expired
Escalated
Closed
```

### 48.8 Delivery Destination State

```text
DefaultAddress
SavedAddressSelected
NewAddressProvided
RecipientAddressSelected
PhotoAddressCaptured
MapPinCaptured
RecipientAddressRequested
Validated
DeliveryUnavailable
NeedsCustomerConfirmation
Closed
```

### 48.9 Recipient Access State

```text
NoAccess
LinkCreated
LinkSent
OnboardingPending
DeliveryOnly
DeliveryAndReturn
SupportOnly
BuyerApprovalRequired
RestrictedByPrivacy
FullRecipientSupport
Closed
```

### 48.10 Cancellation Request State

```text
Requested
VerificationRequired
EligibilityChecking
CancelledInstantly
DispatchStopRequested
ProviderCancellationRequested
CourierInterceptRequested
ConvertedToReturn
NotEligible
RefundIssued
Closed
```

### 48.11 Return Request Interface State

```text
Requested
EligibilityChecking
EvidenceRequired
CourierOptionPresented
BuyerApprovalRequired
SubmittedToReturnsEngine
NotEligible
Closed
```

### 48.12 Warranty Interface State

```text
Requested
OrderMatched
WarrantyEligibilityChecking
ProviderSupportRoutingRequired
ProviderClaimOpened
EvidenceRequested
OrphanProtectionReview
Resolved
Closed
```

### 48.13 Buying List State

```text
Created
ItemAdded
ItemScheduled
ReadyToOrder
CustomerConfirmationRequired
Ordered
PartiallyOrdered
Paused
Archived
```

---

## 49. Reason Codes

```text
CUSTOMER_LINKED_TO_COMPANY
CUSTOMER_ONBOARDED_TO_SELENE
FIRST_VERIFIED_INTRODUCER_EVENT_CAPTURED
DUPLICATE_INTRODUCER_IGNORED
STORE_CONTEXT_ACTIVE
PERSONAL_SELENE_CONTEXT_ACTIVE
SEARCH_CONTEXT_RESOLVED
CUSTOMER_SEARCH_REQUEST_TYPED
CUSTOMER_SEARCH_REQUEST_VOICE
PHOTO_SEARCH_REQUESTED
BAD_SEARCH_INTENT_REPAIRED
PRODUCT_RECOMMENDED_FROM_STORE_RELATIONSHIP
PRODUCT_RECOMMENDED_FROM_B2B
PRODUCT_RECOMMENDED_FROM_CUSTOMER_MEMORY
PRODUCT_VIEWED
PRODUCT_VIDEO_VIEWED
VARIANT_SELECTION_REQUIRED
ORDER_INTENT_CREATED
ORDER_NEEDS_ADDRESS
ORDER_NEEDS_RECIPIENT
RECIPIENT_ADDRESS_REQUESTED
PHOTO_ADDRESS_CAPTURED
PAYMENT_PERMISSION_REQUIRED
STEP_UP_VERIFICATION_REQUIRED
PASSKEY_VERIFICATION_REQUESTED
BIOMETRIC_VERIFICATION_REQUESTED
PASSCODE_FALLBACK_REQUIRED
ORDER_CONFIRMED
B2B_DISPLAY_ELIGIBILITY_REQUESTED
B2B_ORDER_CONTEXT_CREATED
B2B_SETTLEMENT_HANDOFF_REQUIRED
ORIGINAL_PROVIDER_SUPPORT_ROUTING_REQUIRED
DELIVERY_TO_RECIPIENT_REQUESTED
QUIET_GIFT_MODE_ENABLED
RECIPIENT_LINK_SENT
RECIPIENT_RETURN_REQUESTED
CANCELLATION_REQUESTED
CANCELLATION_INSTANT_REFUND_ELIGIBLE
DISPATCH_STOP_REQUESTED
RETURN_REQUESTED
COURIER_PICKUP_REQUESTED
INTERNATIONAL_LANDED_COST_ESTIMATE_REQUIRED
WARRANTY_CLAIM_REQUESTED
ORPHAN_WARRANTY_REVIEW_REQUIRED
PROFESSIONAL_SERVICE_COMPLIANCE_REQUIRED
BUYING_LIST_ITEM_ADDED
PREDICTIVE_REORDER_RECOMMENDED
RESTAURANT_BOOKING_REQUESTED
SERVICE_BOOKING_REQUESTED
REWARD_ATTRIBUTION_EVENT_CREATED
CUSTOMER_PRIVACY_RESTRICTION_APPLIED
CONVERSATION_TO_ACTION_GUARDRAIL_APPLIED
```

---

## 50. Required Simulations

```text
customer joins through company link
customer joins through friend referral link
first verified introducer event captured
second introducer ignored
customer belongs to multiple company stores
customer searches in company store context
customer searches in personal Selene context
Selene silently prioritizes relevant results
customer asks why results were chosen
company views customer purchase history for its own store
company cannot view customer purchases from another store
company enables curated B2B expansion
company enables autonomous B2B expansion
customer sees B2B product in store
customer sees B2B product in personal Selene search
customer buys B2B product through channel store
B2B settlement handoff required
customer buys B2B product through personal Selene search
no channel commission is assumed by E-Commerce
provider support routing required for technical question
customer asks about wire thickness
Selene routes question to Original Provider
provider-approved answer reused later
customer direct personal Selene search for Supplier 44 wire
ABC receives no commission because no channel attribution exists
customer has unified buying list across providers
customer adds milk to groceries
customer orders usual weekly list
customer types bad search phrase and GPT-5.5 repairs intent
customer voice search resolves product
customer photo-searches similar product
customer filters by lower price, size, color, style
customer selects variant conversationally
customer says “buy that one”
PH1.X resolves “that one”
customer says “send it to mum”
Selene resolves mum or requests address securely
mum provides address by photo
Selene confirms payment with passkey
Selene confirms payment with biometric
Selene uses secret passcode fallback
recipient onboarding link sent
recipient asks tracking status
recipient asks return
buyer approval required for recipient return
customer cancels before shipment
customer cancels during dispatch and stop request opens
customer starts return and courier pickup booked
international landed cost shown
customer books restaurant and chooses dishes ahead
customer asks warranty question
professional service requires compliance check before display
provider disappeared and orphan warranty review opens
desktop session continues on iPhone
lost device is replaced and cloud memory restores
```

---

## 51. Integration Map

```text
PH1.ECOMMERCE / PERSONAL_COMMERCE
↔ PH1.CUSTOMER
↔ PH1.CUSTOMER_LINKING / STORE_RELATIONSHIP
↔ PH1.REFERRAL_REWARDS / LOYALTY_ATTRIBUTION
↔ PH1.PRODUCT
↔ PH1.PRODUCT_B2B_READINESS
↔ PH1.INVENTORY
↔ PH1.B2B_PLATFORM
↔ PH1.ORIGINAL_PROVIDER / PROVIDER_OF_RECORD
↔ PH1.ORDER / ORDER_ORCHESTRATION
↔ PH1.PRICING / MARGIN / DISCOUNT
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.SETTLEMENT_TRUST / CUSTOMER_PROTECTION
↔ PH1.CUSTOMER_CREDIT / WALLET / VIRTUAL_SETTLEMENT
↔ PH1.DISPATCH / PACKING / COURIER_HANDOFF
↔ PH1.RETURNS / REFUNDS / REVERSE_LOGISTICS
↔ PH1.WARRANTY / AFTER_SALES
↔ PH1.PROFESSIONAL_SERVICES_COMPLIANCE
↔ PH1.LOGISTICS / DELIVERY
↔ PH1.POS
↔ PH1.RESTAURANT / MENU / BOOKING
↔ PH1.MARKETING
↔ PH1.EVENTS / INVITATIONS / RSVP
↔ PH1.ACCOUNTING
↔ PH1.CASHFLOW
↔ PH1.TAX
↔ PH1.COMPLIANCE
↔ PH1.SAAS_TENANCY / DEVICE_ACCESS
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.REM
↔ PH1.BCAST / DELIVERY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
↔ PH1.PREDICTIVE_INPUT
```

---

## 52. Required Logical Packets

```text
CustomerStorefrontPacket
PersonalSeleneCommercePacket
CompanyCustomerRelationshipPacket
StoreContextPacket
SearchContextPacket
ConversationalShoppingRequestPacket
ConversationToActionGuardrailPacket
VisualProductDiscoveryPacket
PhotoSearchPacket
ProductDisplayPacket
ProductVideoPacket
ProductRecommendationPacket
SearchPrioritySignalPacket
FirstVerifiedIntroducerEventPacket
CustomerMemoryCommerceSignalPacket
VariantSelectionPacket
OrderIntentPacket
CartIntentPacket
B2BDisplayEligibilityRequestPacket
B2BOrderContextPacket
B2BSettlementHandoffPacket
OriginalProviderSupportRoutingPacket
PaymentPermissionRequestPacket
CustomerStepUpVerificationPacket
DeliveryDestinationPacket
PhotoAddressCapturePacket
RecipientAddressRequestPacket
RecipientAccessPacket
GiftPrivacyModePacket
CancellationRequestPacket
InstantRefundEligibilityPacket
DispatchStopRequestPacket
ReturnRequestInterfacePacket
CourierPickupRequestPacket
WarrantyRequestInterfacePacket
ProfessionalServiceEligibilityRequestPacket
BuyingListPacket
PredictiveReorderPacket
RestaurantBookingIntentPacket
RewardAttributionEventPacket
CustomerCommerceAuditEvidencePacket
```

Logical only.

Codex maps later.

No runtime packet structs. The schema goblin can sit quietly and chew on a cable.

---

## 53. What Codex Must Not Do

```text
Do not merge E-Commerce with B2B Platform.
Do not make individual companies owners of B2B platforms.
Do not let E-Commerce own Product-to-B2B readiness.
Do not let E-Commerce own Original Provider responsibility.
Do not let E-Commerce own Channel Store commission logic.
Do not let E-Commerce own provider/customer commission calculation.
Do not let E-Commerce own provider deposits/reserves.
Do not let E-Commerce own product master truth.
Do not let E-Commerce own inventory stock truth.
Do not let E-Commerce own global customer memory truth.
Do not let E-Commerce calculate final rewards.
Do not let E-Commerce own customer credit qualification.
Do not let E-Commerce execute payments directly.
Do not let E-Commerce issue refunds without Payment/Returns authority.
Do not let E-Commerce decide provider payout.
Do not let E-Commerce own dispatch operations.
Do not expose profit share to customers unless required.
Do not expose one company’s customer data to another company.
Do not narrate internal search priority to customers by default.
Do not use fixed phrase matching for customer/company language.
Do not bypass step-up verification for protected actions.
Do not let GPT-5.5 invent product, stock, delivery, warranty, insurance, licence, professional eligibility, or refund facts.
Do not let GPT-5.5 execute protected actions.
Do not ignore recipient privacy / gift mode.
Do not store raw biometric data.
Do not store raw payment-card data casually.
Do not make channel stores responsible for original provider product faults unless explicitly contracted.
Do not allow B2B products/services without B2B eligibility from Document 78/Product readiness.
Do not allow regulated professional services without required compliance/insurance checks.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 54. Final Architecture Sentence

Selene E-Commerce Engine is the customer-facing personal commerce brain that gives every customer one Selene identity, many protected company-store relationships, one personal Selene commerce home, unified buying lists, natural typed/voice/photo shopping, secure one-command checkout, recipient delivery, gift privacy, customer-facing order tracking, cancellation, return, warranty, and service booking; lets companies operate their own stores, customer relationships, marketing, events, account terms, and B2B-expanded catalog surfaces; sends B2B-sourced offers to Document 78 for Original Provider, Channel Store, attribution, commission, reserve, and settlement rules; routes product/service questions to the Original Provider without forcing ugly customer store switching; and uses GPT-5.5 with PH1.X and PH1.M to make commerce feel human while deterministic Selene engines protect truth, money, stock, delivery, privacy, settlement, accounting, and audit.

Simple version:

```text
One customer.
One Selene.
Many company stores.
Customer can search broadly.
Company store context is protected.
E-Commerce shows customer-facing shopping.
B2B owns provider/commission/settlement/reserve rules.
Product owns product truth.
Inventory owns stock.
Payment owns money movement.
Dispatch owns boxes and courier.
Returns owns reverse logistics.
Warranty owns claims.
Rewards owns reward math.
Customer has one unified buying list.
Selene can ask mum for address securely.
Selene can route product questions to the Original Provider.
Customer does not need to see backend profit share.
Customer does not need ugly store switching unless required.
GPT-5.5 understands natural language.
PH1.X resolves live context.
PH1.M resolves durable memory.
Deterministic engines verify before action.
Everything important is audited.
```

That is Global Document 77 — Selene E-Commerce Engine v7. It is now properly separated: E-Commerce owns the customer experience and handoffs; B2B owns the commercial marketplace machinery. Finally, the shopping cart has stopped trying to become an accountant, supplier, courier, lawyer, and loyalty cult leader at the same time.

---

## 55. 81E B2B Customer-Visible Pricing Display Handoff

E-Commerce must display B2B customer-visible pricing only after Document 81 and 81E validate the B2B pricing stack, benefit funding, all-in customer cost, floor/ceiling rules, refund/reversal exposure, brand approval or referral-only outcome, and audit evidence.

E-Commerce must not show unfunded benefits, B2B discounts, provider-channel offers, referral-only alternatives, or customer-visible value claims that violate 81E economics, 81D brand/channel guardrails when available, or Document 81 final pricing governance.

---

## 56. 81F-81J Customer-Facing Pricing Pack Handoff

E-Commerce promotion display must respect 81F eligibility, test exposure, objective, expiry, offer-stacking, refund/reversal, and stop-loss status.

E-Commerce customer-facing prices, discounts, delivery fees, tax/duty estimates, local fees, all-in price where required, offer expiry, return/warranty summaries, claims, and pricing explanations must use 81G evidence and audit references.

E-Commerce must respect 81H service capability and delivery promise truth before offering premium service, packaging, installation, customization, or service upgrades; must respect 81I service availability, delivery-zone, territory, official-region, local fee, currency, and reverse-logistics signals; and must request 81J presentation readiness before displaying premium, luxury, B2B, launch-sensitive, official-channel, or claim-sensitive products.

E-Commerce must not show unsupported claims, unfunded B2B benefits, or presentation-sensitive offers that 81J/81G/81E/Document 81 cannot prove, fund, or approve.

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
