# Global Document 64 — Selene Product Intelligence Engine

```text id="doc64_status"
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
64

ENGINE:
PH1.PRODUCT / PH1.PXM / PH1.PRODUCT_INTELLIGENCE

FULL NAME:
Selene Product Intelligence, Product Experience, Supplier Data, Commerce Presentation, Product Passport, Media, Market Positioning, and Channel Readiness Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Selene Product Intelligence Engine is the company’s central product brain.

It owns the truth of:

```text
what the product is
how it is described
how it is classified
how it is photographed
how it is presented
how it is priced for display
how it is prepared for POS, e-commerce, B2B, and marketplaces
how it links to suppliers
how it links to compliance
how it links to customer discovery
how it becomes sale-ready
```

This engine is not a stock table.

This engine is not a warehouse count.

This engine is not “product name, price, barcode, good luck.”

That is caveman commerce with a login screen.

Selene Product Engine turns products into intelligent, sellable, searchable, compliant, media-rich, channel-ready commercial objects.

Simple split:

```text
Product Engine = what the product/service is and how it is sold.
Inventory Engine = how many exist, where they are, and what condition they are in.
Supplier Engine = who supplies it and how reliable they are.
Pricing Engine = what selling price is allowed and profitable.
E-Commerce/B2B/POS = where and how the product is sold.
Accounting = financial truth after transactions occur.
```

---

## 2. Why Product Must Come Before Inventory

Product comes first.

Inventory depends on Product.

```text
Product defines:
- item identity
- SKU/barcode
- unit of measure
- variants
- category
- expiry requirement
- batch/serial requirement
- compliance requirement
- sale channels

Inventory then tracks:
- quantity
- location
- batch
- expiry
- reservation
- stock movement
- stock health
```

Example:

```text
Product:
Organic Shampoo 500ml

Inventory:
143 units exist
82 in Store A
41 in Warehouse B
20 reserved for B2B order
6 expire in 40 days
```

Without Product, Inventory does not know what it is counting. It is just counting ghosts in cartons. Very busy. Very useless.

---

## 3. Core Selene Law

```text
Selene does the product setup work.
Humans confirm important truth.
Selene publishes only when rules, evidence, and authority allow.
```

Selene must reduce human effort by:

```text
reading product labels
reading supplier invoices
reading catalogs
reading spreadsheets
reading old websites
capturing product photos
improving product media
detecting duplicate products
classifying categories
suggesting product names
writing product descriptions
suggesting attributes
suggesting variants
suggesting bundles
suggesting market positioning
preparing POS listing
preparing e-commerce listing
preparing B2B listing
preparing product passport data
flagging missing compliance
flagging missing supplier data
flagging missing pricing/margin data
```

Human should not have to understand:

```text
PIM
PXM
GTIN
GDSN
GS1 Digital Link
2D barcode readiness
product schema
merchant feed attributes
Digital Product Passport
channel taxonomy
structured product data
```

Human says:

> “Selene, add this product.”

Selene says:

> “Show me the label, invoice, or product photo. I’ll prepare the listing and ask you to confirm.”

That is the Selene standard. Not “please complete 74 fields before lunch,” because we are not running an onboarding torture boutique.

---

## 4. Current Global Standards Selene Must Be Ready For

Selene Product Engine must be future-ready, not built like a 2009 SKU database wearing a modern font.

GS1 Digital Link is a standardized way to encode identifiers such as GTINs, GLNs, SSCCs, batch numbers, serial numbers, and expiry dates so those identifiers can be scanned in a barcode and connected to online information. ([GS1][1])

GS1’s Sunrise 2027 initiative is pushing readiness for 2D barcodes at point of sale, with GS1 US describing the transition toward smarter 2D barcodes and Digital Link use cases for richer product information. ([GS1 US][2])

The European Commission says technical preparation for Digital Product Passports is underway, including rules on identifiers, data carriers, access rights, a DPP registry, and a web portal. ([Green Forum][3])

Google Merchant Center’s product data specification says structured product data helps Google match products to relevant queries and warns that incorrect, inaccurate, or missing data can prevent products from showing properly. ([Google Help][4])

So Selene must support:

```text
richer product identity
2D/QR/barcode readiness
GS1 identifier readiness
product passport data
structured product attributes
marketplace/e-commerce feeds
product compliance evidence
machine-readable product information
human-friendly product presentation
```

Translation: products are no longer “title + price.” They are tiny commercial data universes wearing packaging.

---

## 5. Engine Boundary

### 5.1 PH1.PRODUCT owns

```text
product identity
product naming
product categories
product attributes
product descriptions
product variants
product bundles
product media
product labels
product barcode / QR / identifier readiness
supplier product data link
product compliance fields
product passport fields
product channel readiness
POS product card
e-commerce product listing
B2B product listing
marketplace feed readiness
product content quality
product media quality
product market positioning
product completeness score
product health score
product lifecycle state
```

### 5.2 PH1.PRODUCT does not own

```text
stock quantity
warehouse location
stock movement
expiry stock rotation
supplier qualification
purchase orders
goods receiving
supplier invoice payment
payment execution
ledger posting
final tax treatment
final pricing approval
marketing campaign execution
```

### 5.3 Correct owner split

```text
PH1.PRODUCT = what it is and how it is presented.
PH1.INVENTORY = how many exist and where.
PH1.SUPPLIER = who supplies it and how good they are.
PH1.PRICING = approved prices, margin guards, discounts.
PH1.ECOMMERCE = online store / checkout.
PH1.B2B = trade catalog, contract pricing, bulk orders.
PH1.POS = in-person sale execution.
PH1.MARKETING = campaigns and promotion execution.
PH1.ACCOUNTING = financial posting.
PH1.TAX = tax treatment.
PH1.AUDIT = proof.
```

No product swamp. No inventory octopus. Everyone gets a job and stays in its chair.

---

## 6. Product Master Record

Every product has a Product DNA record.

```text
product_id
legal_entity_id
brand
manufacturer
supplier_links
product_name
short_name
customer_facing_title
internal_title
category
subcategory
product_type
product_family
variant_group
SKU
supplier_SKU
GTIN / UPC / EAN / ISBN where applicable
internal barcode
GS1 Digital Link where configured
QR code
unit_of_measure
pack_size
carton_size
case_quantity
minimum_order_quantity
dimensions
weight
volume
color
size
material
ingredients
allergens
country_of_origin
warranty
care_instructions
safety_information
restricted_sale_flag
age_restriction_flag
expiry_required
batch_required
serial_required
temperature_required
compliance_documents
certifications
product_passport_fields
images
videos
labels
manuals
technical_specs
POS_status
ecommerce_status
B2B_status
marketplace_status
pricing_presentation_status
inventory_template_status
supplier_status
publication_status
audit_ref
```

Product DNA must be extensible because a restaurant menu item, a spare part, a shampoo bottle, a software subscription, and a machine all need different facts.

One generic product table cannot rule them all. That’s how systems become haunted.

---

## 7. Product Types Selene Must Support

Selene must support:

```text
physical goods
retail products
wholesale products
food and beverage
perishable goods
restaurant menu items
ingredients sold as products
cosmetics
fashion/apparel
electronics
machinery
parts/components
building materials
vehicles/equipment for sale or hire
serialised products
batch/lot products
regulated products
restricted products
digital products
subscriptions
services
appointments
packages
bundles
kits
made-to-order products
custom products
B2B-only products
marketplace-only products
internal-use products
```

Each type has its own required fields.

### Food product

```text
ingredients
allergens
nutrition
expiry
batch
storage temperature
country of origin
serving size
```

### Fashion product

```text
size
colour
material
fit
season
care instructions
variant photos
```

### Machinery part

```text
part number
model compatibility
technical specs
warranty
serial requirement
manual
maintenance compatibility
```

### Service

```text
duration
staff eligibility
booking rules
deposit
cancellation rule
deliverables
location
```

Selene asks simple questions:

> “Does this expire?”

> “Does it come in sizes or colours?”

> “Is this something customers book, or something customers buy?”

Humans answer normal questions. Selene converts them into structured product intelligence.

---

## 8. Autonomous Product Capture

Selene must accept product information from almost anywhere.

### 8.1 Voice/text

User says:

> “Add 2017 Shiraz, 750ml, 14.5%, retail $18.90.”

Selene extracts:

```text
name = 2017 Shiraz
volume = 750ml
alcohol = 14.5%
retail price proposal = 18.90
```

Selene then asks:

> “Should this appear in POS, online store, B2B trade catalog, or all three?”

### 8.2 Camera/photo

Selene opens the camera and reads:

```text
front label
back label
barcode
brand
size
ingredients
warnings
expiry
batch
visual style
```

### 8.3 Supplier invoice

Selene extracts:

```text
supplier
product name
supplier SKU
unit cost
quantity
pack size
tax
invoice number
```

Then:

```text
Product gets identity and supplier data.
Inventory gets accepted quantity after Receiving.
AP gets invoice data.
```

Product does not steal Inventory’s lunch.

### 8.4 Supplier catalog / price list

Selene imports:

```text
product list
supplier SKUs
costs
MOQ
pack sizes
lead times
category
trade price
linked images
```

### 8.5 Existing website / old store

Selene can migrate:

```text
titles
descriptions
images
prices
variants
categories
SEO copy
availability structure
```

### 8.6 Spreadsheet import

Selene reads rows and detects:

```text
duplicate SKUs
missing prices
missing categories
missing tax flags
missing images
invalid variants
```

Selene should not force humans to manually enter 800 products unless the business owner is being punished for sins in a previous CRM.

---

## 9. Product Confirmation Flow

Selene never publishes raw guesses as truth.

```text
capture
→ extract
→ normalize
→ detect duplicates
→ classify
→ enrich
→ check missing fields
→ prepare channel listings
→ show summary
→ human confirms important truth
→ publish / save draft
```

Example:

> “I prepared this product from the label and supplier invoice. I found name, brand, size, barcode, supplier cost, and category. I still need confirmation on retail price and whether it should appear on B2B.”

If uncertain:

> “The label photo is blurry. I am not confident whether this is 500ml or 550ml. Please confirm.”

No fake certainty. We are building Selene, not a hallucination kiosk in a blazer.

---

## 10. Product Identity and Identifier Layer

Selene must support multiple identity systems.

### 10.1 Internal identifiers

```text
product_id
variant_id
bundle_id
service_id
internal SKU
internal barcode
```

### 10.2 External identifiers

```text
GTIN
UPC
EAN
ISBN
manufacturer part number
supplier SKU
model number
serial number
batch / lot
GS1 Digital Link
```

### 10.3 Traceability identifiers

```text
batch
lot
serial
expiry
manufacture date
country of origin
supplier certificate
recall status
```

### 10.4 Identifier rule

```text
Selene may generate internal IDs.
Selene may prepare barcode/QR labels.
Selene must not invent official GS1 identifiers unless the business owns the valid allocation.
```

No fake GTIN goblinry. Retailers and marketplaces tend to notice when identifiers are fiction. Rude of them, but fair.

---

## 11. GS1 / 2D Barcode / Digital Link Readiness

Selene should support current and future product identifiers.

Capabilities:

```text
1D barcode support
2D barcode support
QR code support
GS1 Digital Link readiness
batch/lot/expiry encoding where applicable
serial number readiness
product web URI readiness
label generation
packaging data carrier mapping
```

Product Engine should prepare:

```text
consumer scan destination
B2B scan destination
compliance scan destination
recall information link
warranty/registration link
authenticity link
product passport link
```

Example:

> “This product can use a standard barcode for POS now, but I recommend preparing GS1 Digital Link readiness for richer product data and future packaging.”

Selene is not building for yesterday’s scanner. She is building for commerce where packaging, online data, compliance, and traceability all talk to each other like adults. Allegedly.

---

## 12. Digital Product Passport Readiness

Selene must be DPP-ready for applicable products and markets.

Product passport fields may include:

```text
materials
composition
manufacturer
supplier
country of origin
sustainability data
repair instructions
maintenance instructions
recycling instructions
disposal instructions
spare parts
warranty
certifications
compliance documents
chain-of-custody data
carbon / environmental data where applicable
```

DPP readiness states:

```text
NotRequired
Unknown
RequiredLikely
DataIncomplete
ReadyForReview
ReadyForPublication
Published
```

Selene says:

> “This product may need product passport data if sold into regulated markets. I can prepare the missing information list.”

Selene should not ask a small retailer for every DPP field on day one unless it matters. That would be how to make people throw laptops into the sea.

---

## 13. Product Taxonomy and Classification

Selene must classify products across multiple taxonomies.

```text
internal company category
customer-facing category
POS department
e-commerce category
B2B category
marketplace category
merchant/search category
accounting category
tax category
inventory category
compliance category
```

Example:

```text
Product: Sparkling Water 750ml

Customer category: Beverages > Water
POS department: Drinks
Inventory category: Bottled Beverage
B2B category: Hospitality Supply
Tax category: Food/Beverage tax treatment review
E-commerce category: Drinks > Sparkling Water
```

Selene should auto-suggest and ask for confirmation only when risk matters.

---

## 14. Product Attribute Intelligence

Selene manages required and optional attributes by product type.

Attribute types:

```text
identity attributes
sale attributes
search attributes
variant attributes
compliance attributes
logistics attributes
inventory attributes
tax attributes
B2B attributes
marketing attributes
product passport attributes
```

Attribute quality states:

```text
RequiredComplete
RequiredMissing
OptionalComplete
Suggested
NeedsReview
NotApplicable
```

Selene says:

> “This product is ready for POS, but not ready for online sale because it is missing image, description, and delivery dimensions.”

That is useful. A red required-field asterisk is not useful. It is a tiny UI tantrum.

---

## 15. Product Variant and Bundle Logic

Selene must support:

```text
single product
variants
variant groups
kits
bundles
case packs
cartons
multi-packs
build-to-order products
menu combos
service packages
subscriptions
replacement parts
cross-sell groups
upsell groups
substitute products
```

Variant examples:

```text
size
colour
flavour
material
model
voltage
region
language
pack size
```

Bundle examples:

```text
buy shampoo + conditioner
office starter pack
restaurant meal combo
B2B carton pack
service package
maintenance kit
```

Selene should suggest bundles when evidence supports it:

> “Customers who buy this shampoo often buy the conditioner. I recommend creating a bundle and B2B carton offer.”

Good. That is product intelligence, not “add related item manually” like it’s 2004.

---

## 16. Product Channel Readiness

Each product has separate channel readiness.

```text
POS readiness
E-commerce readiness
B2B readiness
Marketplace readiness
Quote portal readiness
Subscription readiness
Internal catalog readiness
```

### 16.1 POS readiness requires

```text
short display name
barcode/scan ID
price
tax code proposal
unit
refund rule
restricted sale rule if applicable
```

### 16.2 E-commerce readiness requires

```text
title
description
images
price
availability source
delivery data
returns rule
category
structured attributes
customer-facing compliance
```

### 16.3 B2B readiness requires

```text
trade title
trade description
MOQ
case/carton quantity
trade price or contract pricing link
lead time
delivery zones
supplier reliability
bulk discount rules
credit/account eligibility
```

### 16.4 Marketplace readiness requires

```text
feed attributes
GTIN/brand/MPN where available
images
availability
price
shipping
returns
merchant category
policy compliance
```

Selene says:

> “This product can go live in POS now. Online store needs a photo and description. B2B needs MOQ and carton size.”

Clear. Human. Not “Channel readiness partially invalid.” Horrible little phrase.

---

## 17. Product Content Intelligence

Selene must write different content for different contexts.

### POS copy

```text
Organic Shampoo 500ml
```

### E-commerce copy

```text
A gentle 500ml organic-style shampoo designed for daily use, suitable for customers looking for a clean, salon-friendly finish.
```

### B2B copy

```text
500ml shampoo suitable for salon resale. Available in cartons of 24 with trade pricing and regional delivery.
```

### Compliance copy

```text
Ingredients, warnings, storage instructions, country of origin, batch, expiry, certifications.
```

### Marketing copy

```text
A clean everyday shampoo your customers can actually remember to ask for again.
```

Selene must not invent claims.

If product is not certified organic, Selene must not call it certified organic.

Apparently truth still matters. Awkward, but important.

---

## 18. Product Media Studio

Selene Product Engine includes a product media studio.

Capabilities:

```text
photo capture
background cleanup
lighting correction
cropping
label readability enhancement
variant photo mapping
thumbnail generation
e-commerce hero image
B2B spec sheet image
POS thumbnail
short product video creation
social promo asset preparation
before/after comparison where applicable
lifestyle image suggestion
```

Selene can say:

> “The product photo is usable, but the label is dark. I can improve the image and create a clean online listing photo.”

For videos:

> “I can create a short 10-second product video for the online store and campaign draft. Please confirm before publishing.”

Publication requires approval.

The robot can make the shampoo look like a star. The human still confirms before it goes on stage.

---

## 19. Product Market Intelligence

Selene should research and advise product positioning.

Selene analyzes:

```text
similar products
competitor titles
price ranges
category trends
customer reviews
common attributes
bundle opportunities
market gaps
premium vs budget positioning
seasonal demand
B2B suitability
search visibility
```

Outputs:

```text
positioning suggestion
pricing presentation suggestion
content improvement suggestion
bundle suggestion
market risk warning
category saturation warning
B2B opportunity
discount/promo candidate
```

Selene says:

> “Similar products in this category usually highlight origin, ingredients, and pack size. I recommend improving the title and adding a trade carton option.”

Market intelligence is advisory, not deterministic truth.

Selene recommends.

Pricing/authority approves.

---

## 20. Product Pricing Presentation

Product Engine prepares pricing presentation but does not own final pricing authority.

It stores:

```text
retail price proposal
online price proposal
B2B price proposal
case price proposal
subscription price proposal
bundle price proposal
launch price proposal
display discount
price confidence
margin warning
```

Pricing Engine owns:

```text
approved price
margin floor
discount authority
contract price
customer-specific price
promotion rules
```

Selene Product can say:

> “The supplier cost is $10.40. At $14.00, online margin becomes weak after payment and delivery costs. I recommend reviewing price before publishing online.”

But Product must not secretly change final price like a tiny capitalist poltergeist.

---

## 21. Supplier Product Data Link

Product Engine links each product to supplier data.

Supplier fields:

```text
supplier_id
supplier_sku
supplier_product_name
unit_cost
pack_size
MOQ
lead_time
supplier_image
supplier_description
supplier_certification
country_of_origin
supplier_availability
supplier_trust_score
alternative_suppliers
```

Supplier Engine owns supplier performance and approval.

Product Engine uses supplier status for product readiness.

Example:

> “This product is ready for online sale, but the preferred supplier is on Watchlist. I recommend not promoting heavily until supply reliability improves.”

Useful. Mildly suspicious. Correct Selene flavour.

---

## 22. Compliance and Restricted Product Logic

Selene must flag products needing compliance review.

Possible flags:

```text
age-restricted
regulated
hazardous
medical/health
food/allergen
alcohol/tobacco where legal
cosmetic safety
electrical safety
children’s product
import/export controlled
warranty requirement
country-of-origin requirement
product safety certificate required
recall-sensitive
```

Product status:

```text
ComplianceNotRequired
ComplianceUnknown
ComplianceDataMissing
ComplianceReviewRequired
ComplianceApproved
ComplianceHold
Blocked
```

Selene says:

> “This product may need compliance documentation before online sale. I can save it as draft but will not publish it until reviewed.”

No publishing restricted products because the product photo looked friendly. Rules still apply, annoyingly.

---

## 23. Product Lifecycle States

```text
Draft
Captured
Extracted
NeedsReview
MissingRequiredData
ReadyForPOS
ReadyForECommerce
ReadyForB2B
ReadyForMarketplace
Published
Paused
SupplierRiskHold
ComplianceHold
PricingHold
Discontinued
Archived
```

Channel-specific state is allowed.

Example:

```text
POS: Ready
E-Commerce: NeedsPhoto
B2B: MissingMOQ
Marketplace: MissingGTIN
```

Selene says:

> “This product is live in POS. It is not ready for B2B because trade price and carton quantity are missing.”

---

## 24. Product Completeness Score

Selene scores readiness.

```text
identity completeness
attribute completeness
media completeness
supplier completeness
pricing completeness
compliance completeness
POS readiness
e-commerce readiness
B2B readiness
marketplace readiness
product passport readiness
```

Example:

```text
Product: Organic Shampoo 500ml
Completeness: 82%

Ready:
- name
- size
- photo
- supplier
- retail price
- e-commerce description

Missing:
- barcode
- ingredients
- carton quantity
- B2B trade price
```

Selene should tell the user what is missing in plain English.

No one wants to decode a product readiness matrix unless they are trapped in a SaaS demo.

---

## 25. Product Health Score

Product health combines data quality and commercial performance.

Inputs:

```text
completeness
media quality
supplier reliability
margin safety
sales velocity
return rate
customer interest
search visibility
B2B suitability
stock health from Inventory
compliance risk
review sentiment where available
```

Health states:

```text
Excellent
Good
NeedsWork
Risk
Hold
DiscontinueCandidate
```

Selene says:

> “This product has strong margin and good supplier reliability, but weak photos and missing B2B data. I recommend improving media before promotion.”

That is a product manager in a sentence, minus the meeting invite.

---

## 26. Strategic Product Role

Products have commercial roles.

```text
Hero Product
Profit Driver
Traffic Driver
Habit Builder
Bundle Support Product
Seasonal Product
B2B Anchor Product
Subscription Anchor
Clearance Product
Dead Stock Candidate
Discontinue Candidate
```

This matters because a low-margin product may be worth keeping if it creates repeat buying behaviour.

Example:

> “This product has low margin, but it drives repeat visits and higher-margin basket purchases. I recommend keeping it as a traffic driver while controlling stock tightly.”

This connects Product to Inventory, Cashflow, Pricing, Marketing, and Customer Intelligence.

No dumb margin-only decisions. We are trying to be better than a spreadsheet with opinions.

---

## 27. Product Passport and Compliance Evidence Fabric

Product Engine must store evidence.

Evidence types:

```text
supplier certificate
product safety certificate
country-of-origin evidence
ingredients / materials proof
warranty document
manual
technical sheet
lab test
quality certificate
product photos
packaging photos
label photos
import document
product passport data source
audit record
```

Evidence status:

```text
NotRequired
Missing
Captured
NeedsReview
Approved
Expired
Rejected
```

Selene says:

> “The product certificate expires in 30 days. I’ll request renewal from the supplier.”

Good product data is not static. It rots if ignored. Like contracts. And bananas. And teams without documentation.

---

## 28. Custom Attribute Governance

Users can add custom attributes, but Selene must govern them.

Example:

> “Add carbon rating.”

Selene response:

> “Carbon rating is not part of your approved product schema. I can save it as a custom attribute and send it for review before using it in reports, filters, or compliance.”

Rules:

```text
custom attributes can be stored
custom attributes need source/context
custom attributes do not affect automation until approved
schema promotion requires authority
all changes audited
```

This keeps flexibility without letting product data become soup with emojis.

---

## 29. Product Duplicate Detection

Selene must detect duplicate products.

Duplicate signals:

```text
same barcode
same GTIN
same supplier SKU
same manufacturer part number
same name/brand/size
same image
same supplier invoice line
same category and pack size
```

Actions:

```text
auto-merge suggestion
variant suggestion
duplicate warning
supplier alias mapping
human confirmation if merge affects live channels
```

Selene says:

> “This looks like the same product as Organic Shampoo 500ml, but with a slightly different supplier name. Should I link it as the same product or create a supplier alias?”

This prevents the classic “same product entered seven times” tragedy. A moving story, often performed in Excel.

---

## 30. Product Search and Discovery Layer

Selene must support internal and customer-facing product discovery.

Search types:

```text
keyword search
voice search
photo search
barcode search
category search
attribute search
natural language search
similar product search
substitute product search
B2B trade search
internal admin search
```

Customer asks:

> “Show me eco-friendly shampoo under $20.”

Selene should understand:

```text
category = shampoo
attribute = eco-friendly/clean positioning
price limit = under 20
availability = in stock
channel = e-commerce/POS/B2B depending context
```

If compliance claim is not verified, Selene must not overstate it.

---

## 31. Product Translation and Localization

Selene must prepare products for multiple languages, regions, and markets.

Localization includes:

```text
language translation
local unit formats
currency display
regional compliance text
market-specific title
local search terms
country-of-origin display
restricted-claim removal
region-specific channel availability
```

GPT-5.5 may translate and adapt wording.

Selene must preserve regulated meaning.

No turning “mild cleanser” into “medical miracle foam” because translation got excited.

---

## 32. Product Handoff to Inventory

Product creates an inventory template.

Inventory template fields:

```text
product_id
track_stock yes/no
unit_of_measure
batch_required
serial_required
expiry_required
temperature_required
default_location
reorder_category
shelf_life_required
storage_requirement
valuation_category
stocktake_priority
```

Inventory owns actual stock values.

Product says:

> “This product requires expiry tracking.”

Inventory says:

> “Batch A expires in 12 days and needs discount/transfer.”

Clean handoff. No one steals the other’s keys.

---

## 33. Product Handoff to Commerce

Product sends channel packets.

### POS Product Packet

```text
product_id
short_name
barcode
price_ref
tax_ref
restricted_sale_flag
refund_rule
image_thumbnail
```

### E-Commerce Product Packet

```text
product_id
title
description
images
price_ref
availability_ref
delivery_dimensions
returns_rule
category
structured_attributes
```

### B2B Product Packet

```text
product_id
trade_title
trade_description
MOQ
case_quantity
lead_time
trade_price_ref
bulk_discount_ref
delivery_zone
supplier_status
```

Product does not check out customers.

Commerce engines sell.

Product gives them good product truth so they stop making garbage listings.

---

## 34. Product Handoff to Accounting and Tax

Product provides classification hints.

```text
product category
sale type
tax category candidate
inventory/non-inventory flag
COGS category candidate
revenue category candidate
regulated product flag
asset/product distinction
```

Accounting and Tax own final treatment.

Selene says:

> “This looks like inventory for resale, not a fixed asset. I’ll route the classification to Accounting and Inventory.”

No classifying a laptop for resale as company equipment unless evidence says so. Context matters. Humans hate that. Accounting depends on it.

---

## 35. Automation and Exception-Only Review

Selene auto-handles:

```text
product capture from photo/invoice/spreadsheet
basic product extraction
category suggestion
duplicate detection
draft descriptions
photo improvement
channel readiness scoring
missing field detection
POS draft listing
e-commerce draft listing
B2B draft listing
product passport missing-data list
supplier product link
custom attribute storage as unapproved
```

Selene needs human or authority approval for:

```text
publishing product
changing approved product truth
changing final price
making compliance claims
publishing regulated product
merging live products
deleting/discontinuing live product
using official GS1 identifiers
exposing product to B2B network
launching marketing campaign
approving custom schema attribute for automation
```

Routine = Selene handles.

Truth and risk = human confirms.

Everything = audit.

---

## 36. PH1.D / GPT-5.5 Role

GPT-5.5 should be heavily used for product intelligence.

### 36.1 GPT-5.5 may help

```text
draft product descriptions
rewrite titles
summarize supplier catalog data
extract messy invoice meaning
suggest attributes
suggest category
suggest product positioning
draft B2B descriptions
draft e-commerce descriptions
generate customer-friendly wording
translate/localize descriptions
summarize competitive positioning
draft product video script
explain missing readiness items
```

### 36.2 GPT-5.5 must not

```text
invent ingredients
invent certifications
invent country of origin
invent compliance claims
approve product publication
approve official identifiers
change final pricing
override supplier risk
override compliance hold
delete products
merge live products without confirmation
```

GPT-5.5 can make the product sound like a star.

Selene deterministic engines make sure the star is not secretly uncertified glitter glue.

---

## 37. Human-Like Selene Interaction

### New product

> “Show me the product or upload the supplier invoice. I’ll prepare the listing.”

### Missing fields

> “This is almost ready. I only need pack size and whether it should appear on B2B.”

### Product photo

> “The photo is usable, but the label is dark. I can clean it and create a brighter online image.”

### B2B readiness

> “This product can sell to businesses, but I need MOQ, carton quantity, and trade price before publishing it to B2B.”

### Compliance hold

> “I can save this as draft, but I will not publish it until the missing safety certificate is attached.”

### Market positioning

> “This looks like a premium product. I recommend highlighting origin, ingredients, and packaging quality instead of competing on price.”

Human-like. Practical. Slightly bossy. Exactly how product setup should feel when the machine is doing most of the work and the human is mostly nodding wisely.

---

## 38. State Machines

### Product State

```text
Draft
Captured
Extracted
NeedsReview
MissingRequiredData
ReadyForPOS
ReadyForECommerce
ReadyForB2B
ReadyForMarketplace
Published
Paused
SupplierRiskHold
ComplianceHold
PricingHold
Discontinued
Archived
```

### Product Media State

```text
Missing
Captured
NeedsEnhancement
Enhanced
Approved
Published
Rejected
Archived
```

### Channel Readiness State

```text
NotConfigured
MissingRequiredData
ReadyForReview
Ready
Published
Paused
Blocked
```

### Compliance State

```text
NotRequired
Unknown
DataMissing
ReviewRequired
Approved
Hold
Blocked
Expired
```

### Custom Attribute State

```text
Captured
StoredAsCustom
PendingSchemaReview
ApprovedForSchema
Rejected
Active
Archived
```

---

## 39. Reason Codes

```text
PRODUCT_CAPTURED_FROM_VOICE
PRODUCT_CAPTURED_FROM_PHOTO
PRODUCT_CAPTURED_FROM_INVOICE
PRODUCT_CAPTURED_FROM_SPREADSHEET
PRODUCT_DUPLICATE_POSSIBLE
PRODUCT_CATEGORY_SUGGESTED
PRODUCT_ATTRIBUTE_MISSING
PRODUCT_MEDIA_MISSING
PRODUCT_MEDIA_ENHANCED
PRODUCT_POS_READY
PRODUCT_ECOMMERCE_READY
PRODUCT_B2B_READY
PRODUCT_MARKETPLACE_READY
PRODUCT_COMPLIANCE_REVIEW_REQUIRED
PRODUCT_COMPLIANCE_HOLD
PRODUCT_SUPPLIER_RISK_HOLD
PRODUCT_PRICING_REVIEW_REQUIRED
PRODUCT_PASSPORT_DATA_MISSING
PRODUCT_GS1_REVIEW_REQUIRED
PRODUCT_CUSTOM_ATTRIBUTE_CAPTURED
PRODUCT_PUBLICATION_REQUIRES_APPROVAL
PRODUCT_PUBLISHED
PRODUCT_PAUSED
PRODUCT_DISCONTINUED
INVENTORY_TEMPLATE_CREATED
```

---

## 40. Required Simulations

```text
add product by voice
add product from photo
add product from supplier invoice
add product from spreadsheet
detect duplicate product
classify product category
create internal SKU
prepare barcode label
prepare GS1 Digital Link candidate
prepare POS listing
prepare e-commerce listing
prepare B2B listing
enhance product photo
create product video draft
suggest market positioning
detect missing compliance data
detect product passport data gap
create inventory template
handoff to Inventory
handoff to E-Commerce
handoff to B2B
handoff to POS
custom attribute captured
custom attribute approved into schema
supplier-risk hold
compliance hold
publish product after confirmation
```

---

## 41. Integration Map

```text
PH1.PRODUCT / PRODUCT_INTELLIGENCE
↔ PH1.INVENTORY
↔ PH1.SUPPLIER
↔ PH1.PROCUREMENT
↔ PH1.PROC.RECEIVE
↔ PH1.ECOMMERCE
↔ PH1.B2B
↔ PH1.POS
↔ PH1.PRICING
↔ PH1.MARKETING
↔ PH1.CUSTOMER
↔ PH1.LOGISTICS
↔ PH1.RETURNS
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.TAX.OPTIMIZE
↔ PH1.COMPLIANCE
↔ PH1.AUDIT
↔ PH1.ACCESS / AUTHORITY
↔ PH1.WRITE
↔ PH1.D / GPT-5.5
```

---

## 42. Required Logical Packets

```text
ProductCapturePacket
ProductExtractionPacket
ProductIdentityPacket
ProductDNAPacket
ProductAttributePacket
ProductVariantPacket
ProductBundlePacket
ProductMediaPacket
ProductMediaEnhancementPacket
ProductClassificationPacket
ProductDuplicateCandidatePacket
ProductSupplierLinkPacket
ProductCompliancePacket
ProductPassportPacket
ProductChannelReadinessPacket
POSProductPacket
ECommerceProductPacket
B2BProductPacket
MarketplaceProductPacket
InventoryTemplatePacket
ProductPricingPresentationPacket
ProductMarketIntelligencePacket
ProductHealthScorePacket
CustomAttributePacket
ProductPublicationPacket
AuditEvidencePacket
```

Logical only. Codex maps later. No runtime goblin sculpting packets out of markdown.

---

## 43. What Codex Must Not Do

```text
Do not merge Product with Inventory.
Do not let Product own stock quantity.
Do not let Product own supplier approval.
Do not let Product own final pricing authority.
Do not let Product publish compliance claims without evidence.
Do not let GPT-5.5 invent product facts.
Do not create official GS1 identifiers without authority/allocation.
Do not publish regulated products without review.
Do not expose products to B2B without B2B readiness.
Do not delete or merge live products without confirmation.
Do not implement from this document alone.
```

---

## 44. Final Architecture Sentence

Selene Product Intelligence Engine is the autonomous product brain that captures product data from voice, photos, labels, invoices, supplier catalogs, spreadsheets, websites, barcodes, and documents; normalizes and enriches product identity, attributes, media, compliance, supplier data, product passport fields, market positioning, pricing presentation, POS readiness, e-commerce readiness, B2B readiness, marketplace readiness, and inventory template handoff; uses GPT-5.5 to draft, explain, translate, and improve product content; and asks humans only for important confirmation, publication approval, pricing authority, compliance truth, and protected product decisions.

Simple version:

```text
Show Selene the product.
Selene reads it.
Selene understands it.
Selene classifies it.
Selene improves the photos and content.
Selene prepares POS, e-commerce, B2B, and marketplace listings.
Selene checks supplier, compliance, pricing, and product passport gaps.
Selene creates the inventory template.
Human confirms important truth.
Selene publishes when safe.
Everything is audited.
```

That is Global Document 64 — Product Intelligence Engine. Not a product form. Not a SKU table. A commercial product brain with cameras, media intelligence, supplier memory, compliance paranoia, channel readiness, and enough manners to ask before publishing “premium miracle shampoo” like a little marketing criminal.

[1]: https://www.gs1.org/standards/gs1-digital-link?utm_source=chatgpt.com "GS1 Digital Link"
[2]: https://www.gs1us.org/industries-and-insights/by-topic/sunrise-2027?utm_source=chatgpt.com "What is GS1 Sunrise 2027?"
[3]: https://green-forum.ec.europa.eu/implementing-ecodesign-sustainable-products-regulation_en?utm_source=chatgpt.com "Implementing the Ecodesign for Sustainable Products ..."
[4]: https://support.google.com/merchants/answer/7052112?hl=en&utm_source=chatgpt.com "Product data specification - Google Merchant Center Help"
