# Selene Credit Cards Addendum — Card Lifecycle + Instant Receipt Capture + Card Issuance + Spend Limit Intelligence

```text
DOCUMENT TYPE:
MASTER DESIGN ADDENDUM / CREDIT CARDS + EMPLOYEE SPEND + CARD LIFECYCLE + RECEIPT CAPTURE

STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

APPLIES TO:
Document 6 — Selene Credit Cards + Employee Spend + Reimbursements Master Design

PURPOSE:
Strengthen Document 6 with immediate card stop rules, role-change card review, card issuance and limit intelligence, instant receipt capture, weekly/monthly review cadence, and all-transaction evidence discipline.
```

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design addendum.

No runtime code was changed.

This document does not authorize implementation.

Current repo truth does not prove complete runtime Credit Card Lifecycle, Card Issuance, Instant Receipt Capture, Insurance, Fleet, or Employee Wellbeing ownership. This document defines future architecture pending Grand Architecture Reconciliation.

Future standalone engines referenced here are future design targets only and are not created by this batch.

## 1. Master Addendum Law

Every company card must have an active business need.

Every cardholder must have an active eligible role.

Every card transaction should be captured with receipt/evidence as close to the spend time as possible.

Every card limit must match the user's role, activity, budget, and risk.

Every resignation, termination, retirement, or role downgrade must trigger immediate card control.

Every card increase requires policy, authority, and audit.

No card survives because humans forgot it existed.

Very simple. The plastic rectangle does not get sentimental.

## 2. Immediate Card Stop / Role Change Law

When HR changes a person's status, Selene must react immediately.

Trigger events:

```text
resignation submitted
termination approved
retirement confirmed
suspension
leave of absence where policy restricts spend
role transfer
demotion / move to lesser position
department change
position no longer requires card
project ended
contractor engagement ended
```

Default actions:

```text
termination -> stop/suspend card immediately
resignation -> stop/suspend card immediately or on approved final work date
retirement -> stop/suspend card immediately or on approved final work date
suspension -> suspend immediately
transfer to non-spend role -> cancel or suspend
transfer to lower-spend role -> reduce limits
temporary project ended -> remove project card/limit
```

Exception:

```text
approved transition spend
approved travel already in progress
approved offboarding purchase
approved executive exception
```

Exception requires:

```text
authority
reason
effective period
limit
audit
```

Selene says:

```text
Tom's role no longer requires a company card. I've suspended new spending and prepared the open-transaction review.
```

## 3. Card Limit Increase / Decrease Law

Card spend increases are permitted only through policy.

Selene may recommend increases based on:

```text
role
position
travel frequency
business entertainment need
vehicle/fuel responsibility
project requirement
department budget
historic approved spend
cashflow
risk
manager approval
country policy
```

Limit types:

```text
daily limit
weekly limit
monthly limit
single transaction limit
merchant category limit
travel limit
fuel limit
entertainment limit
project limit
temporary event limit
```

Selene can reduce limits automatically where policy allows:

```text
role downgrade
repeated missing receipts
personal spend pattern
budget pressure
suspicious spend
project ended
inactive card
```

Selene can increase limits only when:

```text
role policy allows auto increase
or required approvals pass
or temporary authority is granted
```

Example:

```text
Sarah is now Regional Sales Manager.
Travel and entertainment spend is expected.
Selene recommends increasing monthly card limit from AUD 2,000 to AUD 8,000 with travel and client-meal categories enabled.
```

If high-risk:

```text
This limit increase needs Finance approval because it exceeds the standard role template.
```

## 4. Card Issuance Intelligence

Selene should determine who likely needs a card and what limits should apply.

Selene considers:

```text
position
department
travel requirement
fleet responsibility
customer entertainment
procurement duties
field work
remote work
project responsibility
expected monthly spend
budget owner
approval authority
risk profile
```

Card templates:

```text
Sales travel card
Executive card
Warehouse fuel card
Project card
Department purchasing card
Single-use supplier card
Virtual subscription card
Fleet card
```

If bank/provider supports card creation:

```text
Selene prepares named card request.
Selene sets daily/overall/category limits.
Selene routes approval.
Bank/provider creates card.
Selene records card assignment.
```

Hard rule:

```text
Selene may automate card creation only through approved bank/provider rails and Access/Authority gates.
```

No random card spawning.

## 5. Instant Receipt Capture Law

Selene should train users to capture receipts immediately.

Preferred flow:

```text
transaction happens
-> user says: "Selene, capture receipt"
-> Selene opens camera
-> user takes photo
-> Selene attaches receipt to transaction or pending spend record
-> Selene extracts merchant/date/amount/tax candidate
-> Selene asks for purpose if needed
-> transaction becomes evidence-complete or pending review
```

Applies to:

```text
fuel
charging
meals
parking
tolls
travel
hotel
tools
office supplies
repairs
maintenance
client entertainment
subscriptions
materials
training
any company card spend
```

Selene says:

```text
Got it - take a photo of the receipt and I'll file it against this card transaction.
```

If the transaction has not appeared in bank/card feed yet:

```text
Receipt captured now.
Card transaction matched later when feed arrives.
```

This needs:

```text
PendingReceiptCapturePacket
CardTransactionCandidateMatch
```

## 6. Receipt Capture Before Feed Match

Sometimes the user captures receipt before the card transaction appears.

Selene should store a pending receipt.

Fields:

```text
pending_receipt_id
employee_id
card_id
merchant_candidate
amount_candidate
date_candidate
currency_candidate
photo_artifact_ref
business_purpose_candidate
transaction_match_status
matched_card_transaction_id
audit_ref
```

When card feed arrives, Selene matches by:

```text
amount
merchant
date/time
cardholder
currency
location
receipt OCR
```

If high confidence:

```text
auto-match receipt to card transaction
```

If ambiguous:

```text
ask user or finance
```

## 7. Weekly / Monthly Review Cadence

Receipt capture should happen immediately, but Selene still runs scheduled reviews.

Daily:

```text
new card transactions checked
missing receipts detected
obvious personal spend flagged
suspicious transactions flagged
```

Weekly:

```text
cardholder summary
missing receipt reminders
manager exception summary
budget drift warning
personal spend candidates
```

Monthly:

```text
card statement reconciliation
Finance card close
tax claimability review
personal recovery summary
employee spend report
executive card review
budget variance
```

So answer to the question:

```text
Do not wait weekly/monthly to collect receipts.
Collect immediately.
Use weekly/monthly to review exceptions and close the period.
```

## 8. All-Transaction Evidence Rule

Unless policy explicitly exempts a category, every card transaction needs evidence.

Evidence may include:

```text
receipt photo
merchant e-receipt
tax invoice
business purpose
vehicle/odometer evidence
project/task reference
customer meeting reference
travel itinerary
approved subscription owner
manager approval
```

Fuel and vehicle transactions require stronger evidence:

```text
vehicle ID
odometer
litres/kWh
fuel/charge receipt
driver
trip/purpose if required
```

This links to future PH1.FLEET. The fleet document is a future standalone target and should require every fuel/charge record to carry odometer evidence and every vehicle cost to link to a vehicle.

## 9. No Receipt Behavior

If receipt is missing:

```text
first reminder: friendly
second reminder: firm
weekly summary: cardholder + manager if policy says
monthly close: unresolved exception
```

Possible outcomes:

```text
tax claim blocked
manager approval required
employee declaration required
personal recovery candidate
card limit reduced
card suspended for repeat failure
```

Selene says:

```text
I'm still missing the receipt for the AUD 86.40 parking charge. Please take a photo or tell me if the receipt is unavailable.
```

If repeated:

```text
You have four missing card receipts this month. I need these cleared before Finance can close the card statement.
```

## 10. Card Suspension / Limit Reduction for Missing Evidence

Repeated missing receipts can trigger:

```text
card warning
lower card limit
category block
temporary card suspension
manager review
Finance review
```

Selene must apply company policy.

Example:

```text
Policy allows 3 missing receipts per month.
Sarah has 4.
Selene routes manager review and may suspend new card spend until resolved.
```

## 11. Role-Based Card Templates

Selene should use position/access templates.

Examples:

```text
Field Sales Rep
- travel card
- meals/client entertainment allowed
- monthly limit based on region
- receipt required for all spend

Warehouse Supervisor
- fuel/tools/supplies allowed
- no entertainment
- lower daily limit

Executive
- broad spend categories
- higher limit
- stronger review
- board/finance visibility rules

Fleet Driver
- fuel/tolls/parking only
- odometer required
- vehicle ID required

Project Manager
- project card
- project budget limit
- project end date disables card
```

If position changes, card template changes.

## 12. What Must Not Happen Additions

```text
no active card after termination unless explicit approved exception
no card left unchanged after role downgrade
no card limit increase without policy/authority/audit
no card issuance without business need and owner
no receipt capture delayed until month-end as normal process
no fuel/vehicle transaction without receipt and vehicle/odometer evidence where required
no missing receipt ignored
no cardholder punished without policy and evidence
no bank/provider card creation outside approved rails
no role template card limit applied without company policy
no executive card exemption from review
```

## 13. Required Logical Packets

Future logical packets:

```text
CardLifecycleEventPacket
RoleChangeCardReviewPacket
ImmediateCardStopPacket
CardLimitRecommendationPacket
CardLimitChangeRequestPacket
CardIssuanceRecommendationPacket
NamedCardProviderRequestPacket
InstantReceiptCapturePacket
PendingReceiptCapturePacket
ReceiptToCardTransactionMatchPacket
WeeklyCardExceptionReviewPacket
MonthlyCardClosePacket
MissingReceiptEscalationPacket
RoleBasedCardTemplatePacket
FleetFuelCardEvidencePacket
```

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 14. Example - Termination

```text
Tom is terminated.
HR event triggers card review.
Selene suspends Tom's company card immediately.
Selene checks pending transactions.
Selene asks for missing receipts.
Selene prepares personal charge recovery if needed.
Selene sends Finance offboarding summary.
```

Selene says:

```text
Tom's company card is now suspended. I found two pending card transactions and one missing receipt that need review before offboarding can close.
```

## 15. Example - User Captures Receipt Immediately

```text
Tom buys fuel.
Tom says: "Selene, capture receipt for fuel."
Selene opens camera.
Tom takes photo.
Selene extracts amount, litres, merchant, date.
Selene asks for odometer if missing.
Card feed arrives later.
Selene matches receipt to transaction.
Fleet receives vehicle evidence.
Accounting receives fuel expense evidence.
```

Selene says:

```text
Receipt captured. I'll match it to the card transaction when it comes through. What's the van's current odometer reading?
```

## 16. Example - Selene Recommends New Card

```text
Sarah becomes Regional Sales Manager.
Position requires frequent travel and client meetings.
Selene recommends Sales Travel Card.
Limit: AUD 8,000 monthly, AUD 1,500 daily.
Allowed: flights, hotels, meals, transport.
Blocked: cash withdrawals, personal retail.
Approval required: Finance Manager.
```

Selene says:

```text
Sarah's new role includes travel and client meetings. I recommend issuing a Sales Travel Card with a monthly limit of AUD 8,000, subject to Finance approval.
```

## 17. Insurance Management Review

The insurance design should become a standalone engine, not a Document 6 addendum.

Create later:

```text
PH1.INSURANCE
Insurance Policy + Renewal + Claims + Risk Coverage Engine
```

Why standalone?

Because insurance covers far more than employee card spend:

```text
vehicles
buildings
workers compensation
public liability
professional indemnity
product liability
cyber
business interruption
marine/cargo/transit
management liability
directors and officers
plant/equipment
trade credit
key person
travel
contract-specific insurance
```

Selene should later manage all policy registers, renewals, coverage gaps, market checks, premium payments, claims, compliance, certificates, and accounting treatment, while regulated advice, binding/cancelling cover, liability admission, and settlement acceptance stay under authority/legal/licensed-provider rules.

Insurance connects to Document 6 only where card or employee spend touches:

```text
travel insurance
card-paid insurance premiums
vehicle insurance invoices
claims excess paid by card
employee travel claims
```

But the insurance brain is not credit-card management.

Final recommendation:

```text
Document 6 needs the addendum above.
PH1.INSURANCE becomes its own standalone future engine.
PH1.FLEET remains standalone and connects to insurance, cards, AP, accounting, and tax.
```

This addendum makes immediate card stopping, receipt capture timing, card issuance, and spend-limit intelligence explicit for future Codex reconciliation.

## 18. Final Addendum Architecture Sentence

Selene Credit Cards must treat every card as a governed lifecycle object: every active card needs an active role and business need, resignation/termination/role downgrade must trigger immediate card control, card limits must match role, budget, spend pattern, and risk, card issuance must pass provider and Access/Authority gates, receipts should be captured at spend time and matched later if the feed has not arrived, daily/weekly/monthly reviews must clear exceptions without delaying receipt capture, and fuel, vehicle, insurance, fleet, benefits, reimbursement, tax, accounting, and audit handoffs must stay evidence-bound and owner-scoped.
