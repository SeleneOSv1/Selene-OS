# Global Document 82 — Selene Dispatch, Packing, Courier Booking + Delivery Network Handoff Engine

```text
DOCUMENT TYPE:
GLOBAL MASTER ARCHITECTURE DESIGN

GLOBAL DOCUMENT NUMBER:
82

ENGINE:
PH1.DISPATCH / PH1.PACKING / PH1.COURIER_HANDOFF / PH1.DELIVERY_NETWORK

FULL NAME:
Selene Dispatch, Picking, Packing, Package Identity, QR/Barcode Label, Warehouse Location, Roster-Aware Picker/Packer, Courier Booking, Local Agent Delivery, Delivery Network, Freight, Container, Cold-Chain, High-Value Proof, B2B Provider Dispatch, Multi-Address Dispatch, In-Transit Intercept, Proof of Delivery, Exception Recovery, Cost Optimization, and Dispatch Audit Engine

STATUS:
FUTURE_CANONICAL_ARCHITECTURE
NOT_RUNTIME_IMPLEMENTATION
PENDING_REPO_TRUTH_ACTIVATION
PENDING_SIMULATION_MAPPING
CODEX_READY_MASTER_DESIGN
```

---

## 1. Purpose

Document 82 is Selene’s **outbound physical fulfillment control engine**.

It turns confirmed order requirements into real-world dispatch action.

It owns:

```text
pick
pack
scan
label
package identity
courier booking
delivery method selection
delivery handoff
tracking
proof of delivery
dispatch exception handling
dispatch audit
```

Simple version:

```text
Order says what must go out.
Dispatch makes it actually leave correctly.
```

Document 82 is not just “book courier.”

It is the engine that makes sure the right item leaves the right location, picked by the right available person, packed in the right package, delivered by the right method, to the right person, with proof.

That is how Selene avoids warehouse folklore, courier mysteries, and Dave insisting he definitely packed the blue shoes while the proof photo shows red sandals. Dave remains a risk factor, but now an auditable one.

---

## 2. Core Dispatch Law

```text
Selene must not allow a product, package, or delivery to leave a dispatch location unless:
- the order line is dispatch-eligible
- the source/location is known
- stock is reserved or otherwise authorized
- the product/variant/quantity is verified
- the picker/packer task is assigned to an available qualified person/system
- packaging is valid for the product
- contamination and compatibility rules are respected
- package identity exists
- label/QR/barcode exists
- delivery method is valid for the product, destination, service promise, and risk level
- proof requirements are defined
- tracking/handoff is recorded
- customer/recipient communication is prepared where required
- audit evidence is captured
```

No “ship it and hope.”

Hope is not a logistics strategy. It is how parcels become myths.

---

## 3. What Document 82 Owns

Document 82 owns:

```text
dispatch eligibility check
fulfillment mode classification
fulfillment source selection handoff
warehouse/source location dispatch handoff
roster-aware picker assignment
roster-aware packer assignment
pick task creation
pack task creation
scan-to-pick validation
scan-to-pack validation
package identity
QR/barcode/human-readable label generation
packing instructions
package/box selection
product compatibility / contamination packing enforcement
brand/gift/privacy packing enforcement
courier/delivery method selection
courier quote comparison
courier booking
delivery provider registry usage
local agent / employee / community delivery tasking
freight/container/customs dispatch coordination
pickup / handover / possession workflows
tracking
delivery notifications
delivery proof
failed delivery handling
in-transit intercept / reroute / return-to-sender handling
dispatch cost actual vs quoted tracking
dispatch SLA monitoring
dispatch exception playbooks
dispatch audit evidence
dispatch learning signals
```

---

## 4. What Document 82 Does Not Own

Document 82 does **not** own:

```text
customer shopping
product master truth
inventory stock truth
purchase order receiving
order creation
pricing
payment execution
refund execution
returns workflow after return request
warranty claims
tax law
accounting posting
B2B commission calculation
brand policy ownership
courier company infrastructure
scheduler infrastructure
human task infrastructure
```

Correct owner split:

```text
Document 80 Order = what was ordered, by whom, for whom, delivery promise, order line lifecycle.
Inventory = stock truth, location, reservation, batch/serial/expiry availability.
Warehouse / Receiving = stock placed into warehouse/bin/shelf/location.
Document 82 Dispatch = outbound pick/pack/label/courier/tracking/proof.
Document 81H = service/capability/cost-to-serve signal.
Document 81I = geography/delivery-zone/location cost signal.
Document 81D = brand/official-channel guardrails.
Document 81E = B2B pricing/commission/return courier economics.
Document 83 Returns = return/refund/reverse logistics after return flow begins.
Payment = money.
Accounting/Tax = financial records and tax treatment.
Audit = proof.
```

82 controls outbound fulfillment. It does not become the whole commerce empire, because that way lies ERP soup with barcodes floating in it.

---

## 5. Relationship to Document 80 Order Management

Document 80 sends dispatch requirements to Document 82.

Order provides:

```text
customer order group
seller order
order line
product/service item
quantity
variant
seller/provider
Original Provider where B2B
Channel Store where B2B
recipient
delivery address
delivery promise
gift/privacy rules
payment status
cancellation state
B2B context
brand/channel restrictions
return/warranty context
customer communication context
```

Document 82 returns:

```text
dispatch accepted
pick task
pack task
package record
tracking number
courier booking
dispatch status
delivery status
proof of dispatch
proof of delivery
delivery exception
actual dispatch cost
dispatch audit evidence
```

Order remains the customer-facing order truth.

82 owns physical outbound execution.

---

## 6. Relationship to Pricing Pack

Document 82 must consume pricing-pack signals where relevant.

### From 81H — Company Capability

```text
delivery capability
packaging capability
gift wrapping capability
installation capability
staff skill/certification
operational capacity
service promise truth
third-party capability
```

### From 81I — Geography

```text
delivery zone
distance/travel/access cost
local courier availability
event/holiday disruption
weather risk
reverse logistics geography
territory restrictions
customs/duty signal
```

### From 81D — Brand Guardrail

```text
official-channel dispatch requirement
brand-compliant packaging
authorized seller/provider route
prestige/unboxing requirements
presentation/insert rules
```

### From 81E — B2B Pricing

```text
Original Provider responsibility
Channel Store attribution
return courier/reverse logistics cost rule
settlement hold context
provider-direct dispatch
commission/clawback context
```

### From 81J — Presentation

```text
customer-facing delivery promise display
gift/packaging display expectations
official-channel display
return/warranty clarity
```

82 must not break what the pricing pack promised. If E-Commerce promised “luxury gift packaging” and 82 ships it in a crushed shoe box, 82 has created a tiny customer-service crime scene.

---

## 7. Dispatch Eligibility Check

Before dispatch starts, Selene checks:

```text
order confirmed
dispatch requirement exists
payment authorized/captured where required
stock reserved or provider confirmed
source location known
address validated
recipient confirmed where required
seller/provider confirmed
fraud/risk clear
brand/channel restrictions clear
B2B provider route clear
customer cancellation state checked
dispatch hold not active
service/capability available
delivery method possible
legal/compliance hold clear
```

If eligibility fails, 82 must not dispatch.

Failure routes:

```text
return to Order
request missing address
request payment confirmation
request customer decision
request provider confirmation
hold for fraud/risk
hold for brand/channel approval
hold for compliance
cancel or backorder if required
```

No shipping unpaid handbags to imaginary addresses. Radical discipline.

---

## 8. Fulfillment Mode Classification

Document 82 must first classify the fulfillment mode.

Not everything is “put in box and courier.”

Fulfillment modes include:

```text
normal parcel dispatch
store pickup / click-and-collect
locker / pickup point
restaurant / food dispatch
perishable / cold-chain dispatch
fragile goods dispatch
high-value goods dispatch
bulky goods / freight dispatch
container / pallet / international freight
provider-direct / dropship dispatch
B2B Original Provider dispatch
internal fleet delivery
local agent / employee / community delivery
white-glove delivery
installation delivery
vehicle handover
boat / dock / marina handover
real estate possession / settlement handover
machinery / site delivery
digital delivery / no physical dispatch
service appointment / no package dispatch
```

Examples:

```text
Supermarket goods = pick/pack/deliver or pickup.
Restaurant food = kitchen ready + driver pickup.
Clothing/shoes = shelf pick + parcel dispatch.
Manufacturer item = manufacturer direct dispatch.
New car = dealer pickup / dealer delivery / car carrier / customer pickup.
Boat = dock/marina delivery or possession handover.
House/apartment = legal possession / settlement handover, not package dispatch.
Machinery = freight, crane, installer, site readiness.
Digital goods = no physical dispatch.
```

Selene must understand the product before choosing the dispatch shape.

A house cannot be delivered in a box. If that shocks anyone, document it as a training issue.

---

## 9. Fulfillment Source Selection

Selene determines where goods ship or hand over from.

Possible sources:

```text
warehouse
store
branch
dark store
supplier/provider direct
Original Provider
manufacturer
restaurant/kitchen
third-party logistics partner
service depot
dealer
marina/dock
construction/development site
pickup location
locker/pickup point
```

Selection inputs:

```text
Inventory location
Order requirement
B2B provider route
brand/official-channel rule
delivery zone
customer preferred source
stock availability
capacity
cost
speed
service promise
return convenience
```

82 does not invent stock location.

Inventory/Warehouse owns stock truth.

82 uses it to act.

---

## 10. Warehouse / Location / Bin Handoff

Products must exist in a known location before picking.

Location fields may include:

```text
warehouse
branch
store
aisle
bay
shelf
bin
pallet
rack
cold room
freezer
secure cage
high-value cabinet
serial/lot location
batch location
quarantine location
ready-to-dispatch staging area
```

Ownership:

```text
Receiving places goods.
Inventory records goods and locations.
82 consumes the location to guide picking.
```

If location is unknown:

```text
block pick
route inventory investigation
assign stock locate task
notify Order if promise at risk
audit exception
```

Pickers should not roam the warehouse like treasure hunters with barcode guns.

---

## 11. Dispatch Location Readiness

Before a location can dispatch, 82 checks:

```text
stock available
packing area available
label printer available
staff/robot available
courier pickup available
security access available
temperature area available if required
brand packaging available
required documents available
```

If not ready:

```text
choose another source
hold dispatch
route task
escalate
adjust delivery promise
```

---

## 12. Roster-Aware Picker / Packer Assignment

82 must assign tasks only to available and qualified people/systems.

Checks:

```text
person is on duty
person is at correct warehouse/location
person has current capacity
person is trained for product type
person has authority for high-value/restricted goods
person can meet deadline
person has required equipment/scanner
person is not overloaded
backup person exists where required
```

If the selected picker/packer is not available:

```text
select another eligible worker
assign to robot/system if available
escalate to supervisor
delay with reason
notify Order if promise at risk
```

The roster must prevent Selene from assigning tasks to ghosts, lunch breaks, or Dave after he went home.

---

## 13. Scheduled Picker / Packer Task Management

Picking and packing must be scheduled tasks.

Each task includes:

```text
task type
order line / package
warehouse/location
assigned person/system
backup owner
deadline
priority
scan requirements
photo requirements
proof requirements
issue reporting path
reminder cadence
escalation path
closure condition
audit reference
```

Uses:

```text
Task / Human Workload
Scheduler / Rosters
Broadcast / Delivery
Reminder
Access / Authority
Audit
```

No “warehouse should pick this.”

The warehouse is a building. It has poor task-completion instincts.

---

## 14. Pick List Creation

82 creates pick instructions.

Pick list fields:

```text
product
variant
quantity
warehouse/location
shelf/bin/pallet
batch/lot if required
serial if required
expiry if required
condition requirement
handling instruction
priority
customer/order reference
picker assigned
proof required
```

Pick list may be:

```text
single order pick
batch pick
wave pick
zone pick
priority pick
cold-chain pick
high-value pick
B2B provider pick
gift order pick
```

---

## 15. Scan-to-Pick Validation

Picking must use scanning where possible.

Scan types:

```text
barcode
QR
RFID
SKU scan
serial scan
batch/lot scan
expiry scan
shelf/bin scan
package scan
pallet scan
```

Goal:

```text
right product
right variant
right quantity
right location
right batch/serial/expiry
right condition
```

If scan mismatch:

```text
block pick
warn picker
route exception
request supervisor override if allowed
audit event
```

---

## 16. Pick Exception Handling

Pick exceptions include:

```text
stock missing
wrong location
damaged item
expired item
wrong batch
quantity short
serial mismatch
restricted item
product recall
quarantine item
system stock mismatch
```

Possible recovery:

```text
source from another location
split dispatch
substitute where allowed
backorder
cancel line
ask customer
notify Inventory
notify Order
notify B2B provider
route investigation
```

82 does not silently proceed with wrong goods. That’s how surprise sandals happen.

---

## 17. Scan-to-Pack Validation Workflow

Packing must validate the package and contents.

Workflow:

```text
1. Selene creates DispatchPackageRecord.
2. Packer scans package/box ID.
3. Packer scans product SKU / barcode / QR.
4. Selene validates product, variant, quantity.
5. Packer scans serial/batch/expiry if required.
6. Selene confirms correct contents.
7. Photo/weight proof if required.
8. Package is sealed.
9. Seal/label is scanned.
10. Package moves to dispatch staging.
```

Package cannot be sealed unless the contents match the order.

This is the difference between dispatch and “Dave filled a box with confidence.”

---

## 18. Package Identity

Every dispatchable package must have an identity.

Package identity includes:

```text
dispatch_package_id
order_id
seller_order_id
order_line references
seller/provider
Original Provider where B2B
recipient
address
package sequence
package count
QR/barcode
human-readable package number
tracking reference when available
return reference where required
status
audit reference
```

Example:

```text
Package 1 of 3
Order Group: Saturday Gifts
Recipient: Mum
Dispatch Package ID: PKG-000123
QR links to Selene package record
```

The QR/barcode links to Selene.

The printed label helps humans.

Both are needed, because humans and scanners fail in different flavours.

---

## 19. QR / Barcode / Human-Readable Label Architecture

Each package label may include:

```text
QR code
barcode
human-readable package ID
recipient name
delivery address where allowed
delivery instructions where allowed
sender/seller/provider reference where allowed
tracking number
return reference where needed
handling icons
fragile / cold-chain / signature flags
customs label where applicable
```

Label must respect:

```text
privacy rules
gift rules
brand rules
courier requirements
regulatory requirements
recipient visibility rules
```

The label is not just a sticker. It is the package’s passport through reality.

---

## 20. Label Printing Options

82 must support multiple printing methods.

Printing modes:

```text
warehouse label station
packing bench printer
portable/mobile printer
store counter printer
driver printer
provider-direct printer
third-party logistics printer
freight label printer
```

Recommended use:

```text
large warehouse = label station / packing bench printer
small store = counter printer
mobile/internal driver = portable printer
provider-direct dispatch = provider prints Selene label
freight/container = station + document pack
```

If printer unavailable:

```text
pause package
route printer issue task
use backup printer
manual label only if policy allows
audit exception
```

No unlabeled mystery boxes. Mystery boxes belong in games, not logistics.

---

## 21. Packing Instruction Generation

Selene must tell packer how to pack.

Packing instructions may include:

```text
items that go together
items that must be separated
box type
packaging material
fragile handling
cold-chain handling
gift wrap
brand packaging
privacy rules
invoice/receipt rules
dangerous/restricted goods rules
photo proof
seal requirement
label placement
return label inclusion
customs documents
```

---

## 22. Box / Packaging Selection

Selene should recommend the correct packaging.

Packaging types:

```text
box
mailer
padded bag
cold-chain box
fragile box
luxury box
gift box
tamper-evident packaging
pallet
crate
hanging garment bag
insulated packaging
secure high-value packaging
container
```

Inputs:

```text
item dimensions
weight
fragility
temperature needs
contamination risk
brand rules
courier rules
dimensional weight
shipping cost
customer experience
return requirements
```

This stops people from shipping a ring in a refrigerator box. Warehouse comedy avoided.

---

## 23. Product Compatibility / Contamination Packing Matrix

Selene must enforce product compatibility.

Items that may require separation:

```text
soap from bread
chemicals from food
perfume from chocolate
raw food from ready-to-eat food
frozen goods from dry goods
heavy goods from fragile goods
liquids from paper goods
hazardous goods from normal goods
strong-smell goods from absorbent goods
regulated goods from unrestricted goods
```

Product Engine owns compatibility metadata.

82 applies it during packing.

If incompatible:

```text
separate package
different courier
different packaging
special handling
block combined packing
ask customer if split changes cost/timing
```

No detergent-flavoured sourdough. Bold cuisine, bad logistics.

---

## 24. Dimensional Weight and Shipping Cost

Shipping cost may depend on:

```text
actual weight
dimensional weight
box size
destination
service level
insurance
fragility
cold-chain
delivery speed
fuel surcharge
remote area surcharge
courier capacity
```

82 must compare actual vs predicted cost and feed differences to:

```text
81H cost-to-serve
81I geography cost
81E B2B viability
81 Core pricing
Accounting
```

---

## 25. Packaging Cost and Margin Handoff

82 tracks packaging cost.

Costs include:

```text
box
mailer
padding
tape/seal
gift wrap
cold-chain material
dry ice/ice packs where allowed
brand packaging
labor time
waste
repackaging
```

Feeds:

```text
81H cost-to-serve
81 Core pricing
81E B2B pricing
Accounting
Order profitability
```

Boxes cost money. Apparently the cardboard fairy unionized.

---

## 26. Gift and Privacy Packing

Gift orders may require:

```text
gift wrap
gift message
hidden price
separate invoice
recipient-safe packing
no marketing insert
sender identity handling
delivery note privacy
surprise protection
recipient communication rules
```

Example:

```text
Customer sends gift to mum.
Mum should not see price unless allowed.
```

Order provides gift/privacy rules.

82 enforces packing.

---

## 27. Brand-Compliant Packing

Premium/luxury/official-channel goods may require:

```text
approved packaging
approved inserts
official warranty card
authenticity proof
premium unboxing
no cheap box
no discount wording
brand-compliant label
official provider documentation
```

Connects to:

```text
81D Brand Guardrail
81J Presentation
81H Packaging Capability
B2B / Official Channel
```

If brand packaging unavailable:

```text
hold dispatch
route issue task
use approved fallback if allowed
block if required
audit
```

---

## 28. Consolidated vs Split Packing

Selene decides whether to:

```text
pack together
pack separately
split by seller
split by provider
split by temperature
split by fragility
split by delivery date
split by recipient
split by brand/privacy
split by warehouse
split by contamination risk
split by courier eligibility
```

The decision balances:

```text
shipping cost
delivery promise
safety
brand rules
customer expectation
return convenience
B2B obligations
```

---

## 29. Multi-Recipient / Multi-Address Dispatch

82 must support one buyer with many recipients.

Example:

```text
Customer buys 100 gifts.
100 recipients.
100 addresses.
100 delivery instructions.
Possibly 100 packages.
```

82 must support:

```text
one buyer
many recipients
many addresses
bulk address validation
bulk label generation
gift privacy
individual tracking
batch dispatch
per-recipient delivery proof
per-recipient failed delivery handling
per-recipient return path
```

Order owns customer order group.

82 owns package execution per address.

---

## 30. Delivery Instruction Handling

Delivery instructions may include:

```text
leave with neighbor
leave at reception
call before delivery
do not leave unattended
use side gate
deliver to loading dock
leave at concierge
recipient must sign
safe drop allowed
safe drop not allowed
delivery after 5pm
avoid doorbell
```

Instructions must be validated against:

```text
courier capability
safe-drop policy
high-value rules
cold-chain rules
gift privacy
signature requirement
building access
brand/official-channel rules
```

If instruction is invalid:

```text
ask customer/recipient
offer alternative
block unsafe instruction
audit decision
```

Example:

```text
“Leave package with the dog.”
```

Selene should reject this as invalid because the dog lacks signature authority, despite probably being more reliable than some humans.

---

## 31. Cold-Chain Dispatch

For cold-chain goods, 82 must handle:

```text
temperature check
insulated packaging
ice packs/dry ice where allowed
cold courier
dispatch cutoff
delivery time limit
temperature proof
failed-delivery rule
recipient availability
temperature breach exception
```

Examples:

```text
ice cream
fresh food
flowers
medicine
cosmetics
wine/chocolate in heat-sensitive zones
```

No sending ice cream into the afternoon sun like a dessert sacrifice.

---

## 32. Perishable Dispatch

For perishables:

```text
expiry check
freshness check
same-day cutoff
food safety label
temperature handling
delivery window
customer availability
return restriction
waste path
```

Perishable dispatch must coordinate with:

```text
Inventory expiry
81B dynamic markdown
81I weather/geography
Order delivery promise
Customer availability
```

---

## 33. Fragile Goods Dispatch

For fragile goods:

```text
fragile packaging
padding
orientation label
courier limitation
photo proof
insurance
handling instruction
damage claim evidence
```

If fragile policy is not satisfied:

```text
block dispatch
upgrade packaging
select different courier
route approval
```

---

## 34. High-Value Goods Dispatch

High-value goods may require:

```text
serial capture
condition photo proof
tamper packaging
insurance
signature required
ID verification
OTP/PIN verification
face scan where lawful/consented/necessary
secure courier
restricted drop-off
delivery appointment
chain-of-custody proof
sender handoff verification
recipient handoff verification
```

Examples:

```text
luxury goods
high-value electronics
jewelry
vehicles
official-channel goods
B2B high-risk goods
regulated goods
```

Biometrics must be governed:

```text
legal basis
consent where required
data minimization
secure storage
alternative verification where required
audit
```

Face scan is powerful, but also creepy if used like a doorbell with ambitions.

---

## 35. Dangerous / Regulated Goods Dispatch

If applicable, 82 must handle:

```text
restricted carrier
special labeling
documentation
legal/compliance review
age/ID verification
route restrictions
package rules
handling rules
handoff proof
```

82 flags and routes to Compliance.

82 does not invent the law. Warehouse software with legal opinions is how tape becomes litigation.

---

## 36. Freight, Container, Pallet + Customs Dispatch Mode

For freight/container/bulky/international dispatch, 82 must support:

```text
freight quote
pallet/crate
container booking
freight forwarder
bill of lading
commercial invoice
packing list
HS code
country of origin
customs declaration
duties/tax estimate
broker details
port/dock delivery
dock appointment
insurance
inspection
release documents
delivery appointment
loading equipment
```

Connects to:

```text
81I geography/duty signal
Tax/Compliance
Order
Courier/Freight
Accounting
B2B
Customer Communication
```

82 coordinates dispatch workflow.

Tax/Compliance owns final duty/tax legality.

---

## 37. Courier / Delivery Method Selection

82 selects or recommends delivery method.

Options:

```text
internal driver
employee delivery
sales agent delivery
local community delivery
courier company
postal service
same-day courier
bike courier
motorbike courier
ride-share delivery where allowed
taxi delivery where allowed
cold-chain courier
white-glove courier
freight carrier
pallet carrier
international courier
boat/dock delivery
B2B provider delivery
customer pickup
pickup point / locker
dealer handover
provider-direct handover
```

Criteria:

```text
cost
speed
reliability
coverage
product type
package size/weight
insurance
proof capability
customer preference
delivery promise
brand/service level
B2B/provider rule
return capability
geography
```

---

## 38. Delivery Provider Registry by Geography and Product Type

82 must use a delivery provider registry.

Provider registry includes:

```text
delivery provider
coverage zones
country/region/city
service types
vehicle types
max weight/size
cold-chain capability
fragile capability
high-value capability
insurance capability
signature/ID proof capability
locker/pickup capability
return capability
cost model
availability
SLA
reliability score
claim history
```

This registry may include:

```text
courier companies
postal services
internal drivers
employees
sales agents
local contractors
bike couriers
freight companies
boat/dock handlers
white-glove providers
provider-direct delivery staff
```

---

## 39. Local Agent / Employee / Community Delivery Availability + Terms

Local agents or people may opt into delivery services.

They must define:

```text
availability days/times
delivery area
transport type: bike / car / van / truck / boat
max package size
max package weight
product restrictions
cold-chain capability
fragile capability
high-value capability
fixed fee
per-delivery fee
per-km fee
per-quantity fee
minimum fee
proof requirements
insurance/licence
identity verification
rating/reliability
current availability
```

Selene may call upon them only if:

```text
available
in correct area
qualified for the product
within their terms
allowed by company/provider
allowed by customer/brand/risk rules
able to meet delivery promise
cost/reliability beats alternatives
```

Example:

```text
Wine agent offers local delivery:
available 5pm–9pm
car only
max 6 cases
local radius 8km
fixed $12 delivery
photo + recipient signature required
```

This is not “hey mate, drop this off.”

This is a controlled delivery resource with terms, proof, and audit. Civilization, barely.

---

## 40. Courier Quote Comparison

82 compares delivery options.

Quote dimensions:

```text
price
ETA
coverage
pickup cutoff
tracking quality
delivery proof support
damage rate
late rate
lost rate
signature support
ID verification support
insurance
return support
customer rating
brand suitability
```

Selection rule:

```text
choose least-cost safe option that satisfies product rules, delivery promise, customer expectation, brand standard, and proof requirements
```

Cheapest courier that loses luxury handbags is not cheap. It is an expensive magic trick.

---

## 41. Least-Cost Safe Dispatch Optimization

82 must reduce delivery cost without breaking safety or promise.

Optimization tactics:

```text
smallest safe box
avoid dimensional weight waste
combine packages to same address
split packages only when necessary
choose courier by cost/reliability
batch deliveries by route
use pickup point if cheaper and allowed
use internal/agent delivery if cheaper and reliable
avoid failed delivery cost
avoid return courier cost
choose closer fulfillment source
consolidate multi-recipient where possible
```

Rule:

```text
Minimize cost without breaking delivery promise, product safety, brand standard, customer privacy, B2B rule, or customer trust.
```

---

## 42. Courier Booking

82 books courier pickup/delivery.

Booking fields:

```text
pickup address
delivery address
package dimensions
weight
package count
service level
insurance
special handling
recipient contact
delivery instructions
customs documents
pickup window
delivery window
proof requirements
return label if required
```

Booking status:

```text
not booked
quote requested
booked
pickup scheduled
pickup failed
cancelled
rebooked
handed over
```

---

## 43. Shipping Label Generation

82 creates or requests:

```text
shipping label
return label where required
customs label
fragile label
cold-chain label
high-value label
dangerous goods label where allowed
internal package label
pallet label
container label
locker/pickup label
```

Label data must match package identity.

If label mismatches package, block dispatch.

---

## 44. Delivery Manifest

82 creates manifest for couriers/drivers.

Manifest fields:

```text
packages
pickup time
courier/driver
tracking numbers
route
delivery sequence
recipient names
special instructions
proof requirements
exceptions
handoff signature
```

Manifest may be:

```text
courier pickup manifest
internal driver manifest
local agent manifest
freight manifest
container packing list
B2B provider dispatch manifest
```

---

## 45. Courier Pickup Handoff

82 tracks courier pickup.

States:

```text
courier booked
courier arrived
packages ready
packages handed over
scan accepted
pickup partial
pickup failed
pickup delayed
manifest closed
```

If pickup fails:

```text
rebook courier
switch courier
alert warehouse/packer
notify customer if promise impacted
escalate
audit
```

---

## 46. Internal Fleet / Delivery Staff Dispatch

If company uses internal drivers/employees:

```text
driver assignment
vehicle assignment
route assignment
delivery schedule
fuel/toll cost
proof of delivery
failed delivery workflow
driver reminders
driver compliance
```

Uses:

```text
Roster
Scheduler
Task
Broadcast/Delivery
Reminder
Audit
```

Drivers are humans, tragically, so reminders apply.

---

## 47. Route Optimization

For internal/local delivery:

```text
delivery sequence
traffic
time windows
customer priority
cold-chain urgency
driver hours
vehicle capacity
package compatibility
return pickups
multi-drop route
fuel/toll cost
event/road restrictions
```

Output:

```text
route plan
driver task list
ETA sequence
exception plan
```

---

## 48. Customer Tracking

82 sends tracking to customer.

Tracking messages may include:

```text
being prepared
picked
packed
waiting for courier
courier collected
tracking number
in transit
out for delivery
driver nearby
delivered
delayed
failed delivery
pickup ready
return to sender
```

Tracking visibility must respect:

```text
gift privacy
recipient permissions
seller/provider data boundaries
B2B restrictions
high-value security
```

---

## 49. Recipient Notification

If recipient differs from buyer, 82 may notify recipient.

Recipient notification may include:

```text
address confirmation
delivery availability request
ETA
safe-drop request
delivery window
access code request
signature/ID requirement
tracking link
gift delivery notice
```

If address is missing or invalid, Order/Customer handles address capture and 82 pauses dispatch.

---

## 50. Delivery-at-Door Notification

82 should support “delivery at door” notification.

Notifications:

```text
driver nearby
package at door
delivery complete
proof photo available
signature captured
OTP accepted
package left at concierge
locker code ready
pickup point ready
```

This is one of the human-friendly parts of dispatch. Shocking, but useful.

---

## 51. Proof of Delivery

Proof types:

```text
courier scan
signature
photo
GPS location
timestamp
recipient OTP/code
ID verification
face scan where lawful/consented/necessary
driver confirmation
smart locker confirmation
concierge receipt
business stamp
handover certificate
possession certificate
```

Proof requirements depend on:

```text
value
risk
product type
gift
B2B
brand/official-channel
regulated item
customer preference
courier capability
delivery method
```

---

## 52. Possession / Handover Mode

Some assets are not “delivered” as parcels.

Examples:

```text
new car
used car
boat
machinery
real estate
large equipment
custom installation
high-value asset
```

Handover methods:

```text
customer pickup
dealer handover
agent pickup
car carrier delivery
dock/marina handover
site handover
settlement/possession handover
installer handover
```

Proof may include:

```text
handover checklist
ID verification
inspection report
signature
photos
condition report
keys/documents handed over
registration/warranty activation
settlement confirmation
possession confirmation
dock/marina receipt
```

A house is not dispatched. A boat is not left with the neighbor. Thank you for attending logistics kindergarten.

---

## 53. Customer Pickup / Click-and-Collect

82 handles pickup orders.

Pickup workflow:

```text
pick item
pack/prepare item
mark ready for pickup
notify customer
verify customer ID/QR
handover
capture proof
expire pickup window if not collected
restock if needed
notify Order
```

Pickup proof:

```text
QR scan
signature
ID check
staff confirmation
photo where needed
```

---

## 54. Pickup Point / Locker Delivery

82 supports:

```text
locker
pickup point
post office
courier depot
partner store
smart locker
```

Track:

```text
locker code
pickup expiry
customer notified
pickup proof
uncollected package
return to sender
```

---

## 55. Restaurant / Food Dispatch

For food delivery:

```text
kitchen ready time
driver pickup time
packaging proof
spill-proof packing
temperature
delivery window
customer location
allergen/dietary label where required
failed delivery rule
```

Food dispatch must coordinate with:

```text
Restaurant/POS
Order
81I geography/weather
81H packaging/capability
Customer availability
```

---

## 56. Bulky Goods / Freight

For bulky goods:

```text
freight quote
pallet/crate
forklift requirement
loading dock
delivery appointment
two-person delivery
site access
damage proof
insurance
redelivery rules
```

Examples:

```text
furniture
appliances
machinery
building materials
large equipment
```

---

## 57. White-Glove Delivery

White-glove delivery may include:

```text
appointment
two-person delivery
unpack
place item
remove packaging
install/assemble
damage inspection
customer signoff
photo proof
service completion proof
```

81H validates capability.

82 executes dispatch workflow.

---

## 58. Installation Handoff

If item requires installation:

```text
dispatch item
schedule installer
confirm site readiness
coordinate delivery and install
capture completion proof
capture customer signoff
```

Order/Scheduler/Service engine may own installation task.

82 coordinates outbound goods and delivery-to-install handoff.

---

## 59. Site Readiness for Delivery

Some deliveries require site readiness.

Checks:

```text
freezer space
refrigerated storage
loading dock
site access
installer access
recipient available
security clearance
floor space
lift/elevator access
power/water connection
equipment required
permit required
dock/marina available
```

If not ready:

```text
delay dispatch
ask recipient/customer
reschedule
route manual review
switch delivery method
```

This mirrors receiving logic but outbound/customer side.

---

## 60. Customer / Recipient Availability Confirmation

For high-risk or time-sensitive deliveries, 82 must confirm availability.

Confirm:

```text
recipient available
delivery window accepted
ID/signature requirement accepted
access code supplied
safe place approved
cold-chain receipt possible
high-value handoff possible
dock/site ready
```

Uses Broadcast/Delivery and Reminder.

---

## 61. Safe-Drop Rules

Safe-drop may be:

```text
allowed
not allowed
customer-approved
courier-approved
forbidden for high-value
forbidden for cold-chain
forbidden for regulated goods
photo required
signature override required
```

Safe-drop must respect:

```text
product risk
brand rules
customer instruction
insurance conditions
delivery location
proof requirement
```

---

## 62. Delivery Insurance

82 must handle insurance requirements.

Insurance may include:

```text
high-value insurance
fragile goods insurance
international insurance
freight insurance
courier liability limit
additional cover
declared value
claim evidence requirement
```

If insurance required but unavailable:

```text
switch courier
hold dispatch
route approval
warn Order/customer where required
```

---

## 63. Lost / Damaged Courier Claim

If courier loses or damages goods:

```text
claim opened
proof attached
photos
declared value
tracking history
courier scan history
package condition proof
courier response
replacement/refund route
customer communication
accounting handoff
```

Connects to:

```text
Returns
Payment
Warranty
Accounting
Audit
Customer Support
```

---

## 64. In-Transit Intercept / Reroute / Return-to-Sender

If an order is with courier but not delivered, Selene checks possible actions.

States:

```text
label created
pickup booked
courier has package
in transit
at depot
out for delivery
delivered
```

Possible actions:

```text
cancel shipment
intercept package
return to sender
reroute address
hold at depot
redirect to pickup point
too late — wait for delivery then return
```

Checks:

```text
courier supports intercept?
current package location?
cost?
customer approval?
seller/provider approval?
refund/cancellation rule?
brand/high-value rule?
```

Sometimes Selene can stop it.

Sometimes courier says “lol no” and reality wins. 82 records which one happened.

---

## 65. Dispatch Cancellation Window

If customer cancels after dispatch starts, 82 evaluates state:

```text
before pick
after pick
after pack
after label
after courier booking
after courier pickup
in transit
out for delivery
delivered
```

Outcomes:

```text
cancel before pick
unpack/restock
void label
cancel courier
intercept package
return to sender
convert to return
deny cancellation under policy
```

Order owns cancellation lifecycle.

82 provides physical dispatch feasibility.

---

## 66. Address Change After Dispatch

Address change depends on package state.

States:

```text
before label
after label
after courier booking
after courier pickup
in transit
out for delivery
delivered
```

Actions:

```text
change address
reprint label
rebook courier
courier redirect
hold at depot
pickup point redirect
deny and convert to return/redelivery
extra cost
manual approval
```

Address changes for high-value or regulated goods may require identity/approval checks.

---

## 67. Split Delivery Handling

If one order ships in multiple packages, 82 tracks each package separately.

Example:

```text
Package 1 delivered.
Package 2 in transit.
Package 3 delayed.
```

Customer summary must be clear:

```text
Cake delivered.
Ice cream arriving 4–5pm.
Shoes delayed until Friday.
```

No “your order delivered” when half the birthday party is missing.

---

## 68. Partial Dispatch

If only some items are ready:

```text
dispatch available items
wait for full order
ask customer
split by urgency
split by seller/provider
split by temperature/fragility
```

Order owns customer decision.

82 owns physical partial dispatch.

---

## 69. Backorder / Pre-Order Dispatch

82 waits for release conditions:

```text
stock arrival
provider confirmation
customer confirmation
payment status
release date
brand launch date
custom build completion
compliance release
```

Then triggers dispatch.

---

## 70. Dropship / Provider-Direct Dispatch

For provider-direct dispatch:

```text
provider receives dispatch requirement
provider confirms pick/pack
provider prints/uses approved label
provider supplies tracking
provider supplies proof
Selene monitors provider SLA
customer sees Selene-tracked status
```

Provider dispatch must still feed Selene.

No black-hole provider shipping.

---

## 71. B2B Dispatch Obligations

For B2B items, 82 must preserve:

```text
Original Provider
Channel Store attribution
provider responsibility
commission/settlement context
brand approval route
provider support route
return route
warranty route
proof requirements
```

Connects to:

```text
Document 78 B2B
81E B2B Pricing
80 Order
83 Returns later
```

---

## 72. Official-Channel Dispatch

Official-channel goods may require:

```text
official provider shipment
authorized seller proof
approved packaging
official warranty card
authenticity proof
brand-compliant insert
restricted delivery method
no unauthorized channel dispatch
```

If official-channel proof missing:

```text
block dispatch
route brand/provider review
switch provider if allowed
audit exception
```

---

## 73. Dispatch Delay Detection

82 must monitor delays.

Delay triggers:

```text
not picked on time
not packed on time
not labelled
courier not booked
courier pickup missed
tracking not moving
customs stuck
delivery promise at risk
recipient confirmation missing
proof missing
```

Actions:

```text
alert owner
remind picker/packer/courier
switch courier
notify customer if needed
reschedule
escalate
audit
```

---

## 74. Delivery Promise Monitoring

82 compares actual progress against promised delivery.

Inputs:

```text
Order promise
courier ETA
warehouse status
picker/packer task status
traffic/weather
cutoff time
customer address
service level
event/holiday conditions
```

If promise breaks:

```text
customer update
offer option
refund delivery fee where policy
compensation/goodwill route
escalate
update Order
audit
```

---

## 75. Failed Delivery Handling

Failure reasons:

```text
customer not home
recipient unavailable
address wrong
access denied
business closed
weather
courier issue
recipient refused
damaged in transit
customs delay
lost package
safe-drop not allowed
ID/signature failed
```

Recovery actions:

```text
redelivery
pickup point
address correction
return to sender
hold at depot
replacement
refund/return workflow
customer contact
courier claim
manual review
```

---

## 76. Delivery Exception Playbooks

Required playbooks:

```text
late courier
lost package
damaged package
wrong address
customer unavailable
recipient refuses
customs hold
cold-chain failure
high-value delivery failed
gift delivery failed
B2B provider dispatch failed
safe-drop dispute
driver failed proof
in-transit intercept request
```

Each playbook defines:

```text
trigger
owner
customer message
carrier/provider action
deadline
escalation
closure condition
audit
```

---

## 77. Dispatch Cost Actual vs Quoted

82 tracks actual cost against expected cost.

Costs:

```text
quoted shipping cost
actual courier cost
packaging cost
labor cost
fuel/toll cost
parking/permit cost
return cost
surcharge
insurance
failed delivery cost
redelivery cost
```

Variance feeds:

```text
81H cost-to-serve
81I geography
81E B2B viability
81 Core pricing
Accounting
Cashflow
```

---

## 78. Dispatch Margin Impact

82 must report if dispatch cost damages margin.

Signals:

```text
shipping cost too high
wrong box increased cost
split shipments increased cost
failed delivery created extra cost
return courier likely expensive
courier surcharge exceeded forecast
white-glove cost exceeded price
```

This feeds pricing learning.

Boxes and couriers are small margin thieves with tracking numbers.

---

## 79. Package Consolidation Optimization

82 should optimize package count and cost.

Tactics:

```text
combine packages to reduce cost
split packages to meet promise
avoid overpacking
avoid underpacking
reduce dimensional weight
protect fragile goods
preserve gift/privacy
respect contamination rules
respect brand packaging
respect courier limits
```

---

## 80. Sustainability / Packaging Waste Signal

Dispatch may track sustainability signals.

Signals:

```text
box size waste
packing material
eco packaging
reusable packaging
route consolidation
carbon estimate
delivery mode
customer eco option
```

If company uses sustainability claims, evidence must feed 81J/81G.

No fake green fluff. Evidence or hush.

---

## 81. Carbon / Delivery Impact Estimate

Optional but useful:

```text
courier emissions estimate
delivery mode
distance
route consolidation
pickup point option
customer eco delivery option
```

This is not a final tax/accounting measure unless another engine owns that.

82 provides operational estimate and evidence.

---

## 82. Dispatch Batching / Wave Picking

For warehouse efficiency, 82 supports:

```text
batch orders
wave pick
zone pick
priority pick
courier cutoff batch
cold-chain batch
gift batch
B2B provider batch
high-value batch
multi-recipient gift batch
```

Batching must not break:

```text
delivery promise
gift/privacy rules
contamination rules
brand packaging
customer priority
```

---

## 83. Courier Cutoff Management

82 tracks courier cutoffs.

Cutoffs:

```text
same-day cutoff
next-day cutoff
international cutoff
weekend cutoff
public holiday cutoff
courier pickup cutoff
cold-chain cutoff
freight/container cutoff
event delivery cutoff
```

If cutoff missed:

```text
adjust ETA
notify customer
rebook
reroute
switch provider
audit
```

---

## 84. Dispatch Priority Rules

Priority may depend on:

```text
delivery promise
VIP customer
paid express
cold-chain urgency
gift deadline
event deadline
B2B SLA
high-value order
customer service recovery
brand/official-channel priority
replacement shipment
```

Priority rules must be auditable.

No “manager likes this customer” priority unless approved and recorded. Petty logistics is still logistics.

---

## 85. Dispatch Hold Rules

Dispatch may be held for:

```text
payment hold
fraud review
address issue
recipient confirmation missing
customer confirmation
brand approval
B2B provider approval
stock issue
recall
legal/compliance hold
weather/courier issue
high-value proof missing
customs document missing
```

Hold must show:

```text
reason
owner
resolution action
deadline
customer communication requirement
audit reference
```

---

## 86. Dispatch Release Rules

Hold releases only when required conditions clear:

```text
payment cleared
address validated
recipient confirmed
customer confirmed
risk cleared
brand/provider approved
stock ready
courier available
compliance approved
customs documents ready
proof requirements configured
```

---

## 87. Dispatch SLA

82 should measure SLA.

Metrics:

```text
order-to-pick time
pick-to-pack time
pack-to-label time
label-to-courier time
courier-to-delivery time
dispatch exception resolution time
promise accuracy
proof completion rate
```

---

## 88. Dispatch Quality Score

82 should score dispatch quality.

Inputs:

```text
on-time dispatch
picking accuracy
packing accuracy
damage rate
wrong item rate
lost package rate
customer complaints
courier reliability
proof completeness
SLA performance
```

---

## 89. Courier Performance Score

82 tracks each courier/provider.

Scores:

```text
on-time rate
damage rate
lost rate
pickup reliability
tracking quality
customer complaints
claim success
cost accuracy
intercept capability
return capability
proof quality
```

Feeds future courier selection.

---

## 90. Warehouse / Branch Performance Score

82 tracks dispatch location performance.

Metrics:

```text
pick accuracy
pack accuracy
dispatch speed
photo proof compliance
missed cutoffs
damage caused
wrong label
staff performance
scan compliance
task overdue rate
exception rate
```

---

## 91. Customer Communication Timeline

Dispatch must communicate status.

Events:

```text
being prepared
picked
packed
label created
waiting for courier
shipped
tracking available
in transit
out for delivery
delivered
delayed
failed delivery
pickup ready
return to sender
exception opened
exception resolved
```

Broadcast/Delivery owns delivery infrastructure.

82 owns the communication need and status truth.

---

## 92. Customer Communication Channels

Channels may include:

```text
Selene app
SMS
email
voice
push notification
recipient link
business portal
B2B provider portal
driver link
support console
```

Communication must respect:

```text
gift privacy
recipient permission
customer preference
quiet hours
business account rules
B2B data boundaries
```

---

## 93. Delivery Issue Customer Options

If delivery issue happens, Selene can offer:

```text
wait
redeliver
pickup point
change address
hold at depot
cancel if possible
refund delivery fee where policy
replacement
return/refund workflow
customer support escalation
```

Order/Payment/Returns may own downstream decisions.

82 provides dispatch feasibility.

---

## 94. Dispatch Dashboard

82 should show operational queues.

Dashboard items:

```text
orders ready to pick
picking late
packing late
labels pending
courier pickup pending
packages staged
in transit
delivery delayed
failed delivery
lost/damaged
SLA breach
high-value proof missing
customer complaints
B2B provider delays
local agent availability
```

---

## 95. Dispatch Audit

Every dispatch action must be audit-backed.

Audit fields:

```text
who picked
who packed
who sealed
who labelled
which product scanned
which package scanned
which courier selected
which delivery method selected
which tracking number
which box/packaging used
which proof captured
which customer messages sent
which exceptions occurred
which override happened
which cost variance occurred
```

No “system marked shipped.” The system has hands, scanners, labels, and suspects.

---

## 96. Security / Access Control

Only authorized users/systems can:

```text
print labels
change address
change courier
override delivery hold
mark dispatched
mark delivered
release high-value goods
approve safe-drop
approve biometric/ID verification
reprint labels
void labels
open package after sealing
change package contents
approve manual dispatch
```

Unauthorized changes must be blocked and audited.

---

## 97. Manual Override Rules

Manual overrides require:

```text
reason
authority
risk check
customer impact
brand/B2B impact
payment/order impact
rollback possibility
audit
```

Examples:

```text
change courier
override scan mismatch
dispatch without photo proof
change address after label
release high-value item
use unapproved packaging
```

Manual override is not “click yes and pray.” It is a controlled exception.

---

## 98. Dispatch Automation Triggers

82 runs when:

```text
order confirmed
payment status changes
stock reserved
dispatch requirement created
customer changes address
recipient confirms address
delivery promise approaching
courier cutoff approaching
warehouse scan occurs
item picked
package packed
label printed
package sealed
courier pickup scan received
tracking status changes
delivery exception occurs
return-to-sender event occurs
customer cancellation requested
address change requested
```

---

## 99. Scheduled Checks

82 should scan queues.

Checks:

```text
ready-to-dispatch queue
late pick tasks
late pack tasks
courier pickup failures
tracking not moving
delivery promise at risk
unconfirmed recipient availability
high-value proof gaps
unsealed packages
label/package mismatch
staged packages not collected
B2B provider dispatch overdue
local agent task overdue
```

Cadence depends on:

```text
delivery promise
risk level
product type
courier cutoff
warehouse volume
customer priority
```

---

## 100. Human / External Action Orchestration

Any human/courier/provider action must follow the Selene Human / External Action Orchestration Law.

Actions include:

```text
picker picks item
packer packs item
packer uploads photo
warehouse supervisor approves exception
courier confirms pickup
provider confirms dispatch
recipient confirms address
recipient confirms availability
local agent accepts delivery task
driver confirms delivery proof
brand/provider approves packaging exception
```

Every action requires:

```text
owner
recipient
deadline
delivery method
confirmation
evidence
reminder
escalation
closure condition
audit
```

No “someone dispatch it.”

Someone is a myth wearing a hi-vis vest until Selene assigns them.

---

## 101. Dispatch Proof Archive

82 must store proof.

Proof types:

```text
pick proof
pack proof
scan proof
photo proof
weight proof
seal proof
label proof
manifest
tracking
courier scan
handoff proof
delivery proof
signature
GPS
ID/OTP/biometric proof where allowed
exception record
customer communication
override approval
cost variance
```

Feeds:

```text
81G Audit
Order
Returns
Warranty
Payment
Accounting
B2B
Customer Support
```

---

## 102. Dispatch Learning Loop

Selene learns from dispatch outcomes.

Learning signals:

```text
which box works best
which courier works best
which local agent performs well
which zones fail
which products damage easily
which products need better packaging
which warehouses delay
which staff need training
which delivery promises are realistic
which instructions cause failed delivery
which courier supports intercept well
which B2B providers delay dispatch
```

Feeds:

```text
81H capability
81I geography
81J presentation
81E B2B pricing
Order
Inventory
B2B
Returns
Customer Support
Marketing
```

---

## 103. Outputs from Document 82

82 should output:

```text
DispatchRequirementAccepted
DispatchEligibilityResult
FulfillmentModeClassification
FulfillmentSourceSelected
PickTaskCreated
PackTaskCreated
PickerAssigned
PackerAssigned
ScanValidationResult
PickException
PackageRecord
PackageIdentity
PackagingInstruction
LabelCreated
CourierQuoteResult
CourierBooking
DeliveryManifest
CourierPickupHandoff
TrackingNumber
DeliveryStatus
ProofOfDispatch
ProofOfDelivery
DeliveryException
InTransitInterceptResult
DispatchCostActual
DispatchSLAResult
DispatchQualitySignal
CourierPerformanceSignal
WarehousePerformanceSignal
DispatchAuditEvidence
```

---

## 104. State Machines

### Dispatch Requirement State

```text
NotStarted
ReceivedFromOrder
EligibilityChecking
Eligible
Blocked
Held
Released
Accepted
Closed
```

### Fulfillment Mode State

```text
Unclassified
NormalParcel
FoodDispatch
ColdChain
FragileGoods
HighValue
Freight
Container
ProviderDirect
B2BProvider
CustomerPickup
LockerPickup
InternalFleet
LocalAgent
VehicleHandover
BoatDockHandover
RealEstatePossession
ServiceNoPhysicalDispatch
Closed
```

### Pick State

```text
NotReady
ReadyForPicking
Assigned
Picking
Picked
ShortPicked
PickException
Reassigned
Closed
```

### Pack State

```text
NotReady
ReadyForPacking
Assigned
Packing
ScanValidation
Packed
PhotoProofRequired
Sealed
PackException
Closed
```

### Package State

```text
NotCreated
Created
BoxSelected
ContentsValidated
LabelPending
Labelled
Staged
AwaitingPickup
HandedToCourier
InTransit
Delivered
ExceptionOpen
ReturnedToSender
Closed
```

### Delivery State

```text
NotBooked
QuoteRequested
Booked
PickupScheduled
PickedUp
InTransit
AtDepot
OutForDelivery
Delivered
FailedDelivery
InterceptRequested
Rerouted
ReturnToSender
Lost
Damaged
Closed
```

### Proof State

```text
NotRequired
Required
Pending
Captured
Rejected
ManualReview
Accepted
Archived
Closed
```

### Local Agent Delivery State

```text
NotApplicable
AvailablePoolChecked
EligibleAgentFound
TaskOffered
Accepted
Rejected
InProgress
Delivered
Failed
Closed
```

---

## 105. Reason Codes

```text
DISPATCH_REQUIREMENT_RECEIVED
DISPATCH_ELIGIBILITY_PASSED
DISPATCH_ELIGIBILITY_BLOCKED
FULFILLMENT_MODE_CLASSIFIED
FULFILLMENT_SOURCE_SELECTED
WAREHOUSE_LOCATION_REQUIRED
STOCK_LOCATION_UNKNOWN
ROSTER_PICKER_SELECTED
ROSTER_PICKER_UNAVAILABLE
PICK_TASK_CREATED
PACK_TASK_CREATED
SCAN_TO_PICK_VALIDATED
SCAN_TO_PACK_VALIDATED
SCAN_MISMATCH_BLOCKED
PICK_EXCEPTION_OPENED
PACKAGE_IDENTITY_CREATED
PACKAGE_LABEL_CREATED
QR_BARCODE_CREATED
LABEL_PRINT_FAILED
BOX_SELECTED
BOX_SIZE_OPTIMIZED
CONTAMINATION_RULE_SPLIT_REQUIRED
GIFT_PRIVACY_PACKING_REQUIRED
BRAND_PACKAGING_REQUIRED
PHOTO_PROOF_REQUIRED
HIGH_VALUE_IDENTITY_PROOF_REQUIRED
COLD_CHAIN_DISPATCH_REQUIRED
FREIGHT_CONTAINER_DISPATCH_REQUIRED
CUSTOMS_DOCUMENT_REQUIRED
COURIER_QUOTE_COMPARED
COURIER_SELECTED
LOCAL_AGENT_DELIVERY_SELECTED
LOCAL_AGENT_UNAVAILABLE
COURIER_BOOKED
COURIER_PICKUP_FAILED
PACKAGE_HANDED_TO_COURIER
TRACKING_CREATED
CUSTOMER_TRACKING_SENT
RECIPIENT_NOTIFICATION_SENT
DELIVERY_AT_DOOR_NOTICE_SENT
PROOF_OF_DELIVERY_CAPTURED
FAILED_DELIVERY_OPENED
IN_TRANSIT_INTERCEPT_REQUESTED
IN_TRANSIT_INTERCEPT_NOT_SUPPORTED
RETURN_TO_SENDER_REQUESTED
ADDRESS_CHANGE_AFTER_DISPATCH_REQUESTED
SAFE_DROP_BLOCKED
DISPATCH_COST_VARIANCE_DETECTED
DELIVERY_PROMISE_AT_RISK
DISPATCH_SLA_BREACH
COURIER_CLAIM_OPENED
DISPATCH_AUDIT_CAPTURED
```

---

## 106. Required Simulations

```text
normal parcel order picked, packed, labelled, courier booked, delivered with proof
picker unavailable and Selene assigns another rostered picker
picker task overdue and reminder/escalation triggers
stock location missing and inventory investigation task created
scan-to-pack blocks wrong variant
package created with QR/barcode and human-readable label
portable printer used by small store dispatch
warehouse label station used by large warehouse dispatch
100 gifts delivered to 100 addresses with individual tracking
delivery instruction invalid because unsafe for high-value goods
smallest safe box reduces dimensional shipping cost
soap and bread split into separate packages due contamination rule
high-value item requires ID/OTP/face verification where lawful/consented
cold-chain order blocked due to recipient unavailable
container shipment requires customs documents and freight forwarder
in-transit order cancellation requests courier intercept
courier does not support intercept and order converts to return after delivery
local agent opts into bike-only deliveries and accepts local small parcel task
agent unavailable and courier selected instead
new car order classified as dealer handover/car carrier delivery
boat order classified as dock/marina handover
real estate order classified as possession/settlement handover
B2B provider-direct dispatch supplies tracking and proof
official-channel product blocked due missing approved packaging
customer pickup order expires and restock workflow triggers
failed delivery due wrong address requests correction
courier loses package and claim opened with proof
dispatch cost exceeds quote and feeds pricing/cost learning
```

---

## 107. Integration Map

```text
PH1.DISPATCH / DOCUMENT_82
↔ PH1.ORDER / DOCUMENT_80
↔ PH1.INVENTORY
↔ PH1.WAREHOUSE / STOCK_LOCATION
↔ PH1.RECEIVING / DOCUMENT_71
↔ PH1.B2B_PLATFORM / DOCUMENT_78
↔ PH1.PRICING.81H / COMPANY_CAPABILITY
↔ PH1.PRICING.81I / GEOGRAPHY_COST_TO_SERVE
↔ PH1.PRICING.81D / BRAND_GUARDRAIL
↔ PH1.PRICING.81E / B2B_COMMISSION_MODEL
↔ PH1.PRICING.81J / PRESENTATION_PERCEIVED_VALUE
↔ PH1.PRICING.81G / EXPLAINABILITY_AUDIT
↔ PH1.ECOMMERCE / DOCUMENT_77
↔ PH1.POS / DOCUMENT_79
↔ PH1.CUSTOMER
↔ PH1.RETURNS / DOCUMENT_83
↔ PH1.WARRANTY
↔ PH1.PAYMENT / SETTLEMENT
↔ PH1.ACCOUNTING
↔ PH1.TAX
↔ PH1.COURIER / DELIVERY_PROVIDER
↔ PH1.FREIGHT / CONTAINER
↔ PH1.CUSTOMS / COMPLIANCE
↔ PH1.ROSTER / SCHEDULER
↔ PH1.TASK / HUMAN_WORKLOAD
↔ PH1.BCAST / DELIVERY
↔ PH1.REM
↔ PH1.ACCESS / AUTHORITY
↔ PH1.AUDIT
↔ PH1.D / GPT-5.5
↔ PH1.X / LIVE_CONTEXT
↔ PH1.M / MEMORY
↔ PH1.MP / MEMORY_PATTERN
```

---

## 108. Required Logical Packets

```text
DispatchRequirementPacket
DispatchEligibilityPacket
FulfillmentModePacket
FulfillmentSourcePacket
StockLocationDispatchPacket
RosterAwareAssignmentPacket
PickTaskPacket
PackTaskPacket
ScanValidationPacket
PickExceptionPacket
PackageIdentityPacket
PackageLabelPacket
PackingInstructionPacket
ProductCompatibilityPacket
BoxSelectionPacket
GiftPrivacyPackingPacket
BrandCompliantPackingPacket
PhotoProofPacket
HighValueProofPacket
ColdChainDispatchPacket
PerishableDispatchPacket
FragileDispatchPacket
FreightContainerDispatchPacket
CourierQuotePacket
CourierBookingPacket
DeliveryProviderRegistryPacket
LocalAgentDeliveryAvailabilityPacket
LocalAgentDeliveryTaskPacket
DeliveryManifestPacket
TrackingPacket
RecipientNotificationPacket
ProofOfDeliveryPacket
FailedDeliveryPacket
DeliveryExceptionPacket
InTransitInterceptPacket
AddressChangeDispatchPacket
CustomerPickupPacket
HandoverPossessionPacket
DispatchCostActualPacket
DispatchSLAPacket
CourierPerformancePacket
WarehousePerformancePacket
DispatchAuditEvidencePacket
```

Logical only.

No runtime packet structs. The warehouse goblin may admire the schema from behind the tape dispenser.

---

## 109. What Codex Must Not Do

```text
Do not make Document 82 own Order.
Do not make Document 82 own Inventory stock truth.
Do not make Document 82 own Product master truth.
Do not make Document 82 own Pricing.
Do not make Document 82 own Payment.
Do not make Document 82 own Returns after return workflow begins.
Do not make Document 82 own Warranty claims.
Do not make Document 82 own Tax law.
Do not assign picker/packer without roster/capacity/skill check.
Do not create dispatch without package identity.
Do not allow package to seal without scan-to-pack validation where required.
Do not ignore product compatibility/contamination rules.
Do not ignore label/QR/barcode requirements.
Do not offer unavailable delivery methods.
Do not ignore local agent availability/terms.
Do not treat every product as parcel dispatch.
Do not dispatch high-value goods without required proof rules.
Do not ignore customs/freight/container requirements.
Do not mark delivered without proof where proof is required.
Do not use vague human/courier tasks without Human / External Action Orchestration.
Do not let GPT-5.5 invent stock location, courier status, delivery proof, customs facts, roster availability, or delivery completion.
Do not create runtime code from this document.
Do not create packet structs.
Do not implement from this document alone.
```

---

## 110. Final Architecture Sentence

Selene Dispatch, Packing, Courier Booking + Delivery Network Handoff Engine is the outbound fulfillment control layer that receives dispatch requirements from Order, classifies the correct fulfillment mode, selects the valid source/location, assigns roster-aware picker and packer tasks, validates picking and packing through scan/photo/proof rules, creates package identity and labels, enforces product compatibility, brand, gift, cold-chain, high-value, B2B, and official-channel dispatch rules, selects the least-cost safe delivery method from courier, internal, local agent, freight, pickup, handover, or provider-direct options, books and tracks delivery, communicates with customer/recipient, captures proof, handles exceptions, monitors SLA, records cost variances, and audits every outbound step until delivery, pickup, handover, possession, return-to-sender, or exception closure is complete.

Simple version:

```text
82 makes sure the right item leaves the right place,
picked by the right available person,
packed in the right package,
labelled with the right identity,
sent by the right delivery method,
to the right person or place,
with proof.

It does not just ship boxes.
It controls outbound reality.
```

That is Document 82: the engine that turns “order confirmed” into “delivered correctly,” instead of “somewhere between Dave, a shelf, a box, a courier, a dog, and an address label, capitalism lost the parcel.”

---

## AGENTS.md Guardrail References

This document explicitly references and remains governed by the Selene Conversation-to-Action Guardrail and the Selene Human / External Action Orchestration Law in AGENTS.md.

Natural-language interpretation may assist drafting, explanation, and context repair, but deterministic Selene engines retain execution authority, evidence verification, permission checks, protected-action gating, and audit responsibility.
